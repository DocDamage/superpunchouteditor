//! # Disassembler Module
//!
//! Full ROM disassembler with function detection, jump table detection,
//! data section detection, and label management.

use crate::{
    AddressingMode, AssemblyError, CpuMode, ExportFormat, Instruction, Result, Section,
    SectionType,
};
use rom_core::Rom;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};

/// A detected function
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Function {
    /// Function entry point address
    pub entry_point: u32,
    /// Function name/label
    pub name: String,
    /// Start address (may include prologue before entry)
    pub start: u32,
    /// End address (exclusive)
    pub end: u32,
    /// Size in bytes
    pub size: u32,
    /// Subroutines called by this function
    pub calls: Vec<u32>,
    /// Functions that call this function
    pub callers: Vec<u32>,
    /// Local labels within the function
    pub local_labels: HashMap<u32, String>,
    /// Instructions in the function
    #[serde(skip)]
    pub instructions: Vec<Instruction>,
}

impl Function {
    /// Create a new function
    pub fn new(entry_point: u32, name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            entry_point,
            name,
            start: entry_point,
            end: entry_point,
            size: 0,
            calls: Vec::new(),
            callers: Vec::new(),
            local_labels: HashMap::new(),
            instructions: Vec::new(),
        }
    }

    /// Add a local label
    pub fn add_local_label(&mut self, address: u32, name: impl Into<String>) {
        self.local_labels.insert(address, name.into());
    }

    /// Get local label for an address (auto-generate if needed)
    pub fn get_or_create_label(&mut self, address: u32) -> String {
        if let Some(label) = self.local_labels.get(&address) {
            return label.clone();
        }
        let label = format!("{}_loc_{:06X}", self.name, address);
        self.local_labels.insert(address, label.clone());
        label
    }
}

/// A detected jump table
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JumpTable {
    /// Table start address
    pub start: u32,
    /// Table end address (exclusive)
    pub end: u32,
    /// Number of entries
    pub entry_count: usize,
    /// Target addresses
    pub targets: Vec<u32>,
    /// Size of each entry (2 or 3 bytes)
    pub entry_size: u8,
    /// Label name
    pub label: String,
}

/// Label management
#[derive(Debug, Clone, Default)]
pub struct LabelManager {
    /// Global labels (address -> name)
    labels: HashMap<u32, String>,
    /// Reverse mapping (name -> address)
    addresses: HashMap<String, u32>,
    /// Auto-generated label counter
    auto_counter: usize,
}

impl LabelManager {
    /// Create a new label manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a label
    pub fn add_label(&mut self, address: u32, name: impl Into<String>) -> Result<()> {
        let name = name.into();
        if let Some(existing) = self.addresses.get(&name) {
            if *existing != address {
                return Err(AssemblyError::DuplicateLabel(name));
            }
        }
        self.labels.insert(address, name.clone());
        self.addresses.insert(name, address);
        Ok(())
    }

    /// Get label for an address
    pub fn get_label(&self, address: u32) -> Option<&String> {
        self.labels.get(&address)
    }

    /// Get address for a label
    pub fn get_address(&self, name: &str) -> Option<u32> {
        self.addresses.get(name).copied()
    }

    /// Auto-generate a label
    pub fn auto_label(&mut self, address: u32, prefix: &str) -> String {
        if let Some(label) = self.labels.get(&address) {
            return label.clone();
        }
        let label = format!("{}_{:06X}", prefix, address);
        let _ = self.add_label(address, label.clone());
        label
    }

    /// Generate a function label
    pub fn generate_function_label(&mut self, address: u32) -> String {
        self.auto_label(address, "func")
    }

    /// Generate a data label
    pub fn generate_data_label(&mut self, address: u32) -> String {
        self.auto_label(address, "data")
    }

    /// Get all labels
    pub fn all_labels(&self) -> &HashMap<u32, String> {
        &self.labels
    }
}

/// Disassembler configuration
#[derive(Debug, Clone)]
pub struct DisassemblerConfig {
    /// Start address for disassembly
    pub start_address: u32,
    /// End address for disassembly (exclusive)
    pub end_address: u32,
    /// Known entry points to start analysis from
    pub entry_points: Vec<u32>,
    /// CPU mode
    pub cpu_mode: CpuMode,
    /// Assume 8-bit accumulator
    pub m_flag: bool,
    /// Assume 8-bit index registers
    pub x_flag: bool,
    /// Enable jump table detection
    pub detect_jump_tables: bool,
    /// Enable function boundary detection
    pub detect_functions: bool,
    /// Minimum function size
    pub min_function_size: u32,
}

impl Default for DisassemblerConfig {
    fn default() -> Self {
        Self {
            start_address: 0x8000,
            end_address: 0x10000,
            entry_points: vec![0x8000],
            cpu_mode: CpuMode::Native,
            m_flag: false,
            x_flag: false,
            detect_jump_tables: true,
            detect_functions: true,
            min_function_size: 4,
        }
    }
}

/// Full ROM disassembler
#[derive(Debug)]
pub struct Disassembler {
    config: DisassemblerConfig,
    label_manager: LabelManager,
    functions: BTreeMap<u32, Function>,
    jump_tables: Vec<JumpTable>,
    sections: Vec<Section>,
    analyzed_addresses: HashSet<u32>,
    data_references: HashMap<u32, Vec<u32>>,
}

impl Disassembler {
    /// Create a new disassembler
    pub fn new(config: DisassemblerConfig) -> Self {
        let mut label_manager = LabelManager::new();
        
        // Add entry points as labels
        for &entry in &config.entry_points {
            let _ = label_manager.add_label(entry, format!("entry_{:06X}", entry));
        }

        Self {
            config,
            label_manager,
            functions: BTreeMap::new(),
            jump_tables: Vec::new(),
            sections: Vec::new(),
            analyzed_addresses: HashSet::new(),
            data_references: HashMap::new(),
        }
    }

    /// Get reference to label manager
    pub fn label_manager(&self) -> &LabelManager {
        &self.label_manager
    }

    /// Get mutable reference to label manager
    pub fn label_manager_mut(&mut self) -> &mut LabelManager {
        &mut self.label_manager
    }

    /// Disassemble a ROM
    pub fn disassemble(&mut self, rom: &Rom) -> Result<()> {
        // Phase 1: Trace from entry points to identify code
        self.trace_code(rom)?;

        // Phase 2: Detect functions
        if self.config.detect_functions {
            self.detect_functions(rom)?;
        }

        // Phase 3: Detect jump tables
        if self.config.detect_jump_tables {
            self.detect_jump_tables(rom)?;
        }

        // Phase 4: Identify data sections
        self.identify_data_sections(rom)?;

        Ok(())
    }

    /// Trace code execution paths from entry points
    fn trace_code(&mut self, rom: &Rom) -> Result<()> {
        let mut queue: VecDeque<u32> = self.config.entry_points.iter().copied().collect();

        while let Some(address) = queue.pop_front() {
            if self.analyzed_addresses.contains(&address) {
                continue;
            }
            if address < self.config.start_address || address >= self.config.end_address {
                continue;
            }

            self.trace_from(rom, address, &mut queue)?;
        }

        Ok(())
    }

    /// Trace execution from a specific address
    fn trace_from(&mut self, rom: &Rom, start: u32, queue: &mut VecDeque<u32>) -> Result<()> {
        let mut current = start;

        loop {
            if current >= self.config.end_address {
                break;
            }
            if self.analyzed_addresses.contains(&current) {
                break;
            }

            let instruction = self.decode_instruction(rom, current)?;
            let size = instruction.size() as u32;

            self.analyzed_addresses.insert(current);

            // Mark as code
            self.sections.push(Section {
                start: current,
                end: current + size,
                section_type: SectionType::Code,
                label: None,
            });

            // Handle branches and jumps
            if instruction.is_branch() || instruction.is_jump() {
                if let Some(target) = instruction.target_address() {
                    if target >= self.config.start_address && target < self.config.end_address {
                        self.label_manager.auto_label(target, "loc");
                        queue.push_back(target);
                    }
                }
            }

            // Handle subroutine calls
            if instruction.is_call() {
                if let Some(target) = instruction.target_address() {
                    self.label_manager.generate_function_label(target);
                    queue.push_back(target);
                }
            }

            // Stop at returns and unconditional jumps
            if instruction.is_return() || (instruction.is_jump() && instruction.mnemonic == "JMP")
            {
                break;
            }

            current += size;
        }

        Ok(())
    }

    /// Decode a single instruction
    fn decode_instruction(&self, rom: &Rom, address: u32) -> Result<Instruction> {
        let opcode = rom.read_byte(address)
            .map_err(|e| AssemblyError::RomError(e.to_string()))?;
        
        let (mnemonic, mode) = decode_opcode(opcode)?;
        let operand_size = mode.operand_size() as u32;
        
        let mut operand = Vec::new();
        let mut bytes = vec![opcode];
        
        for i in 0..operand_size {
            let byte = rom.read_byte(address + 1 + i)
                .map_err(|e| AssemblyError::RomError(e.to_string()))?;
            operand.push(byte);
            bytes.push(byte);
        }

        let resolved_operand = resolve_operand(address, &mode, &operand, bytes.len() as u32);

        Ok(Instruction {
            address,
            opcode,
            mnemonic: mnemonic.to_string(),
            mode,
            operand,
            resolved_operand,
            bytes,
        })
    }

    /// Detect function boundaries
    fn detect_functions(&mut self, rom: &Rom) -> Result<()> {
        let function_entries: Vec<u32> = self
            .label_manager
            .all_labels()
            .iter()
            .filter(|(_, name)| name.starts_with("func_"))
            .map(|(addr, _)| *addr)
            .collect();

        for entry in function_entries {
            let function = self.analyze_function(rom, entry)?;
            if function.size >= self.config.min_function_size {
                self.functions.insert(entry, function);
            }
        }

        Ok(())
    }

    /// Analyze a single function
    fn analyze_function(&mut self, rom: &Rom, entry: u32) -> Result<Function> {
        let name = self
            .label_manager
            .get_label(entry)
            .cloned()
            .unwrap_or_else(|| format!("func_{:06X}", entry));
        
        let mut function = Function::new(entry, name);
        let mut current = entry;
        let mut calls = Vec::new();

        loop {
            if !self.analyzed_addresses.contains(&current) {
                break;
            }

            let instruction = self.decode_instruction(rom, current)?;
            let size = instruction.size() as u32;

            // Collect calls
            if instruction.is_call() {
                if let Some(target) = instruction.target_address() {
                    calls.push(target);
                }
            }

            // Add instruction to function
            function.instructions.push(instruction.clone());

            // Check for return
            if instruction.is_return() {
                current += size;
                break;
            }

            current += size;

            // Stop if we hit another function entry
            if self.functions.contains_key(&current) {
                break;
            }
        }

        function.end = current;
        function.size = current - entry;
        function.calls = calls;

        Ok(function)
    }

    /// Detect jump tables
    fn detect_jump_tables(&mut self, rom: &Rom) -> Result<()> {
        // Look for patterns that suggest jump tables
        // Common patterns:
        // 1. JMP (addr,X) or JMP [addr,X]
        // 2. RTL after JSL to a dispatch routine
        
        let code_addresses: Vec<u32> = self.analyzed_addresses.iter().copied().collect();

        for &addr in &code_addresses {
            let instruction = self.decode_instruction(rom, addr)?;
            
            // Check for jump table access patterns
            if instruction.mnemonic == "JMP" {
                match instruction.mode {
                    AddressingMode::AbsoluteIndexedIndirect => {
                        if let Some(table_addr) = instruction.resolved_operand {
                            if let Some(table) = self.analyze_jump_table(rom, table_addr)? {
                                self.jump_tables.push(table);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }

    /// Analyze a potential jump table
    fn analyze_jump_table(&self, rom: &Rom, start: u32) -> Result<Option<JumpTable>> {
        let mut targets = Vec::new();
        let mut current = start;
        let entry_size = 2; // Assume 2-byte entries initially
        let max_entries = 256;

        for _ in 0..max_entries {
            if current + 1 >= self.config.end_address {
                break;
            }

            let low = rom.read_byte(current)
                .map_err(|e| AssemblyError::RomError(e.to_string()))?;
            let high = rom.read_byte(current + 1)
                .map_err(|e| AssemblyError::RomError(e.to_string()))?;
            
            let target = ((high as u32) << 8) | (low as u32);

            // Check if target looks like valid code
            if target >= self.config.start_address && target < self.config.end_address {
                if self.analyzed_addresses.contains(&target) || self.is_likely_code(rom, target)? {
                    targets.push(target);
                    current += entry_size;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        if targets.len() >= 2 {
            let end = start + (targets.len() as u32 * entry_size as u32);
            let label = format!("jumptab_{:06X}", start);
            
            Ok(Some(JumpTable {
                start,
                end,
                entry_count: targets.len(),
                targets,
                entry_size: entry_size as u8,
                label,
            }))
        } else {
            Ok(None)
        }
    }

    /// Check if an address looks like code
    fn is_likely_code(&self, rom: &Rom, address: u32) -> Result<bool> {
        if address >= self.config.end_address {
            return Ok(false);
        }

        let opcode = rom.read_byte(address)
            .map_err(|e| AssemblyError::RomError(e.to_string()))?;
        
        // Check if it's a valid opcode
        Ok(decode_opcode(opcode).is_ok())
    }

    /// Identify data sections
    fn identify_data_sections(&mut self, rom: &Rom) -> Result<()> {
        let mut current_start = self.config.start_address;
        
        while current_start < self.config.end_address {
            // Check if this address is code
            if self.analyzed_addresses.contains(&current_start) {
                current_start += 1;
                continue;
            }

            // Find the end of the data section
            let mut current_end = current_start;
            while current_end < self.config.end_address
                && !self.analyzed_addresses.contains(&current_end)
            {
                current_end += 1;
            }

            // Create data section
            if current_end > current_start {
                let label = self.label_manager.generate_data_label(current_start);
                self.sections.push(Section {
                    start: current_start,
                    end: current_end,
                    section_type: SectionType::Data,
                    label: Some(label),
                });
            }

            current_start = current_end + 1;
        }

        Ok(())
    }

    /// Get all detected functions
    pub fn functions(&self) -> &BTreeMap<u32, Function> {
        &self.functions
    }

    /// Get all detected jump tables
    pub fn jump_tables(&self) -> &[JumpTable] {
        &self.jump_tables
    }

    /// Get all sections
    pub fn sections(&self) -> &[Section] {
        &self.sections
    }

    /// Export disassembly to a file
    pub fn export_to_file(&self, path: &str, format: ExportFormat) -> Result<()> {
        let content = match format {
            ExportFormat::Assembly => self.export_assembly(false),
            ExportFormat::AssemblyWithBytes => self.export_assembly(true),
            ExportFormat::Html => self.export_html(),
            ExportFormat::Json => self.export_json()?,            ExportFormat::Text => self.export_text(),
        };

        std::fs::write(path, content)
            .map_err(|e| AssemblyError::IoError(e.to_string()))?;

        Ok(())
    }

    /// Export as assembly
    fn export_assembly(&self, with_bytes: bool) -> String {
        let mut output = String::new();
        
        output.push_str("; Generated by assembly-core disassembler\n");
        output.push_str("; ======================================\n\n");

        // Output functions
        for (addr, func) in &self.functions {
            output.push_str(&format!("; Function: {} @ ${:06X}\n", func.name, addr));
            output.push_str(&format!("{}:\n", func.name));

            for instr in &func.instructions {
                // Check for local label
                if let Some(label) = func.local_labels.get(&instr.address) {
                    output.push_str(&format!("{}:\n", label));
                }

                let bytes_str = if with_bytes {
                    let bytes: Vec<String> = instr.bytes.iter()
                        .map(|b| format!("{:02X}", b))
                        .collect();
                    format!("; {:12} ", bytes.join(" "))
                } else {
                    String::new()
                };

                let operand_str = format_operand(&instr);
                output.push_str(&format!(
                    "    {}{} {}\n",
                    bytes_str,
                    instr.mnemonic,
                    operand_str
                ));
            }

            output.push('\n');
        }

        // Output jump tables
        for table in &self.jump_tables {
            output.push_str(&format!("; Jump Table: {}\n", table.label));
            output.push_str(&format!("{}:\n", table.label));
            
            for (i, target) in table.targets.iter().enumerate() {
                if let Some(label) = self.label_manager.get_label(*target) {
                    output.push_str(&format!("    .addr {} ; Entry {}\n", label, i));
                } else {
                    output.push_str(&format!("    .addr ${:06X} ; Entry {}\n", target, i));
                }
            }
            output.push('\n');
        }

        output
    }

    /// Export as HTML
    fn export_html(&self) -> String {
        let mut output = String::new();
        
        output.push_str("<!DOCTYPE html>\n");
        output.push_str("<html><head><title>Disassembly</title>\n");
        output.push_str("<style>\n");
        output.push_str("body { font-family: monospace; background: #1e1e1e; color: #d4d4d4; }\n");
        output.push_str(".address { color: #808080; }\n");
        output.push_str(".bytes { color: #b5cea8; }\n");
        output.push_str(".mnemonic { color: #569cd6; font-weight: bold; }\n");
        output.push_str(".operand { color: #ce9178; }\n");
        output.push_str(".label { color: #4ec9b0; }\n");
        output.push_str(".comment { color: #6a9955; }\n");
        output.push_str("</style></head><body><pre>\n");

        for (addr, func) in &self.functions {
            output.push_str(&format!(
                "<span class=\"comment\">; Function: {} @ ${:06X}</span>\n",
                func.name, addr
            ));
            output.push_str(&format!(
                "<span class=\"label\">{}:</span>\n",
                func.name
            ));

            for instr in &func.instructions {
                let bytes_str: Vec<String> = instr.bytes.iter()
                    .map(|b| format!("{:02X}", b))
                    .collect();
                
                output.push_str(&format!(
                    "  <span class=\"address\">{:06X}</span>  ",
                    instr.address
                ));
                output.push_str(&format!(
                    "<span class=\"bytes\">{:12}</span>  ",
                    bytes_str.join(" ")
                ));
                output.push_str(&format!(
                    "<span class=\"mnemonic\">{}</span> ",
                    instr.mnemonic
                ));
                output.push_str(&format!(
                    "<span class=\"operand\">{}</span>\n",
                    format_operand(instr)
                ));
            }
            output.push('\n');
        }

        output.push_str("</pre></body></html>");
        output
    }

    /// Export as JSON
    fn export_json(&self) -> Result<String> {
        serde_json::to_string_pretty(&self.functions.values().collect::<Vec<_>>())
            .map_err(|e| AssemblyError::IoError(e.to_string()))
    }

    /// Export as plain text
    fn export_text(&self) -> String {
        let mut output = String::new();
        
        output.push_str("DISASSEMBLY REPORT\n");
        output.push_str("==================\n\n");

        output.push_str(&format!("Functions found: {}\n", self.functions.len()));
        output.push_str(&format!("Jump tables found: {}\n", self.jump_tables.len()));
        output.push_str(&format!("Sections: {}\n\n", self.sections.len()));

        for (addr, func) in &self.functions {
            output.push_str(&format!(
                "Function ${:06X} (size: {} bytes)\n",
                addr, func.size
            ));
            output.push_str(&format!("  Calls: {:?}\n", func.calls));
            output.push_str(&format!("  Callers: {:?}\n\n", func.callers));
        }

        output
    }
}

/// Decode an opcode to mnemonic and addressing mode
fn decode_opcode(opcode: u8) -> Result<(&'static str, AddressingMode)> {
    // 65816 opcode table (subset of common instructions)
    let result = match opcode {
        // BRK
        0x00 => ("BRK", AddressingMode::Implied),
        // ORA
        0x01 => ("ORA", AddressingMode::DirectIndexedIndirect),
        0x05 => ("ORA", AddressingMode::Direct),
        0x09 => ("ORA", AddressingMode::Immediate8),
        0x0D => ("ORA", AddressingMode::Absolute),
        0x11 => ("ORA", AddressingMode::DirectIndirectIndexed),
        0x15 => ("ORA", AddressingMode::DirectIndexedX),
        0x19 => ("ORA", AddressingMode::AbsoluteIndexedY),
        0x1D => ("ORA", AddressingMode::AbsoluteIndexedX),
        // ASL
        0x06 => ("ASL", AddressingMode::Direct),
        0x0A => ("ASL", AddressingMode::Accumulator),
        0x0E => ("ASL", AddressingMode::Absolute),
        // PHP, PLP
        0x08 => ("PHP", AddressingMode::Implied),
        0x28 => ("PLP", AddressingMode::Implied),
        // BPL
        0x10 => ("BPL", AddressingMode::Relative8),
        // JSR, RTS, RTL
        0x20 => ("JSR", AddressingMode::Absolute),
        0x22 => ("JSL", AddressingMode::AbsoluteLong),
        0x60 => ("RTS", AddressingMode::Implied),
        0x6B => ("RTL", AddressingMode::Implied),
        // JMP
        0x4C => ("JMP", AddressingMode::Absolute),
        0x5C => ("JML", AddressingMode::AbsoluteLong),
        0x6C => ("JMP", AddressingMode::AbsoluteIndirect),
        0x7C => ("JMP", AddressingMode::AbsoluteIndexedIndirect),
        0xDC => ("JMP", AddressingMode::AbsoluteIndirectLong),
        // Branch instructions
        0x80 => ("BRA", AddressingMode::Relative8),
        0x82 => ("BRL", AddressingMode::Relative16),
        0x90 => ("BCC", AddressingMode::Relative8),
        0xB0 => ("BCS", AddressingMode::Relative8),
        0xF0 => ("BEQ", AddressingMode::Relative8),
        0x30 => ("BMI", AddressingMode::Relative8),
        0xD0 => ("BNE", AddressingMode::Relative8),
        0x10 => ("BPL", AddressingMode::Relative8),
        0x50 => ("BVC", AddressingMode::Relative8),
        0x70 => ("BVS", AddressingMode::Relative8),
        // LDA
        0xA1 => ("LDA", AddressingMode::DirectIndexedIndirect),
        0xA5 => ("LDA", AddressingMode::Direct),
        0xA9 => ("LDA", AddressingMode::Immediate8),
        0xAD => ("LDA", AddressingMode::Absolute),
        0xAF => ("LDA", AddressingMode::AbsoluteLong),
        0xB1 => ("LDA", AddressingMode::DirectIndirectIndexed),
        0xB2 => ("LDA", AddressingMode::DirectIndirect),
        0xB5 => ("LDA", AddressingMode::DirectIndexedX),
        0xB7 => ("LDA", AddressingMode::DirectIndirectLongIndexed),
        0xB9 => ("LDA", AddressingMode::AbsoluteIndexedY),
        0xBD => ("LDA", AddressingMode::AbsoluteIndexedX),
        0xBF => ("LDA", AddressingMode::AbsoluteLongIndexed),
        // LDX
        0xA2 => ("LDX", AddressingMode::Immediate8),
        0xA6 => ("LDX", AddressingMode::Direct),
        0xAE => ("LDX", AddressingMode::Absolute),
        0xB6 => ("LDX", AddressingMode::DirectIndexedY),
        0xBE => ("LDX", AddressingMode::AbsoluteIndexedY),
        // LDY
        0xA0 => ("LDY", AddressingMode::Immediate8),
        0xA4 => ("LDY", AddressingMode::Direct),
        0xAC => ("LDY", AddressingMode::Absolute),
        0xB4 => ("LDY", AddressingMode::DirectIndexedX),
        0xBC => ("LDY", AddressingMode::AbsoluteIndexedX),
        // STA
        0x81 => ("STA", AddressingMode::DirectIndexedIndirect),
        0x85 => ("STA", AddressingMode::Direct),
        0x8D => ("STA", AddressingMode::Absolute),
        0x8F => ("STA", AddressingMode::AbsoluteLong),
        0x91 => ("STA", AddressingMode::DirectIndirectIndexed),
        0x92 => ("STA", AddressingMode::DirectIndirect),
        0x95 => ("STA", AddressingMode::DirectIndexedX),
        0x97 => ("STA", AddressingMode::DirectIndirectLongIndexed),
        0x99 => ("STA", AddressingMode::AbsoluteIndexedY),
        0x9D => ("STA", AddressingMode::AbsoluteIndexedX),
        0x9F => ("STA", AddressingMode::AbsoluteLongIndexed),
        // STX
        0x86 => ("STX", AddressingMode::Direct),
        0x8E => ("STX", AddressingMode::Absolute),
        0x96 => ("STX", AddressingMode::DirectIndexedY),
        // STY
        0x84 => ("STY", AddressingMode::Direct),
        0x8C => ("STY", AddressingMode::Absolute),
        0x94 => ("STY", AddressingMode::DirectIndexedX),
        // INX, INY, DEX, DEY
        0xE8 => ("INX", AddressingMode::Implied),
        0xC8 => ("INY", AddressingMode::Implied),
        0xCA => ("DEX", AddressingMode::Implied),
        0x88 => ("DEY", AddressingMode::Implied),
        // INC, DEC
        0xE6 => ("INC", AddressingMode::Direct),
        0xEE => ("INC", AddressingMode::Absolute),
        0xF6 => ("INC", AddressingMode::DirectIndexedX),
        0xFE => ("INC", AddressingMode::AbsoluteIndexedX),
        0xC6 => ("DEC", AddressingMode::Direct),
        0xCE => ("DEC", AddressingMode::Absolute),
        0xD6 => ("DEC", AddressingMode::DirectIndexedX),
        0xDE => ("DEC", AddressingMode::AbsoluteIndexedX),
        // CMP
        0xC1 => ("CMP", AddressingMode::DirectIndexedIndirect),
        0xC5 => ("CMP", AddressingMode::Direct),
        0xC9 => ("CMP", AddressingMode::Immediate8),
        0xCD => ("CMP", AddressingMode::Absolute),
        0xD1 => ("CMP", AddressingMode::DirectIndirectIndexed),
        0xD5 => ("CMP", AddressingMode::DirectIndexedX),
        0xD9 => ("CMP", AddressingMode::AbsoluteIndexedY),
        0xDD => ("CMP", AddressingMode::AbsoluteIndexedX),
        // CPX, CPY
        0xE0 => ("CPX", AddressingMode::Immediate8),
        0xE4 => ("CPX", AddressingMode::Direct),
        0xEC => ("CPX", AddressingMode::Absolute),
        0xC0 => ("CPY", AddressingMode::Immediate8),
        0xC4 => ("CPY", AddressingMode::Direct),
        0xCC => ("CPY", AddressingMode::Absolute),
        // TAX, TXA, TAY, TYA, TXS, TSX
        0xAA => ("TAX", AddressingMode::Implied),
        0x8A => ("TXA", AddressingMode::Implied),
        0xA8 => ("TAY", AddressingMode::Implied),
        0x98 => ("TYA", AddressingMode::Implied),
        0x9A => ("TXS", AddressingMode::Implied),
        0xBA => ("TSX", AddressingMode::Implied),
        // PHA, PLA, PHX, PLX, PHY, PLY
        0x48 => ("PHA", AddressingMode::Implied),
        0x68 => ("PLA", AddressingMode::Implied),
        0xDA => ("PHX", AddressingMode::Implied),
        0xFA => ("PLX", AddressingMode::Implied),
        0x5A => ("PHY", AddressingMode::Implied),
        0x7A => ("PLY", AddressingMode::Implied),
        // CLC, SEC, CLD, SED, CLI, SEI, CLV
        0x18 => ("CLC", AddressingMode::Implied),
        0x38 => ("SEC", AddressingMode::Implied),
        0xD8 => ("CLD", AddressingMode::Implied),
        0xF8 => ("SED", AddressingMode::Implied),
        0x58 => ("CLI", AddressingMode::Implied),
        0x78 => ("SEI", AddressingMode::Implied),
        0xB8 => ("CLV", AddressingMode::Implied),
        // AND, ORA, EOR
        0x21 => ("AND", AddressingMode::DirectIndexedIndirect),
        0x25 => ("AND", AddressingMode::Direct),
        0x29 => ("AND", AddressingMode::Immediate8),
        0x2D => ("AND", AddressingMode::Absolute),
        0x31 => ("AND", AddressingMode::DirectIndirectIndexed),
        0x35 => ("AND", AddressingMode::DirectIndexedX),
        0x39 => ("AND", AddressingMode::AbsoluteIndexedY),
        0x3D => ("AND", AddressingMode::AbsoluteIndexedX),
        // ADC
        0x61 => ("ADC", AddressingMode::DirectIndexedIndirect),
        0x65 => ("ADC", AddressingMode::Direct),
        0x69 => ("ADC", AddressingMode::Immediate8),
        0x6D => ("ADC", AddressingMode::Absolute),
        0x71 => ("ADC", AddressingMode::DirectIndirectIndexed),
        0x75 => ("ADC", AddressingMode::DirectIndexedX),
        0x79 => ("ADC", AddressingMode::AbsoluteIndexedY),
        0x7D => ("ADC", AddressingMode::AbsoluteIndexedX),
        // SBC
        0xE1 => ("SBC", AddressingMode::DirectIndexedIndirect),
        0xE5 => ("SBC", AddressingMode::Direct),
        0xE9 => ("SBC", AddressingMode::Immediate8),
        0xED => ("SBC", AddressingMode::Absolute),
        0xF1 => ("SBC", AddressingMode::DirectIndirectIndexed),
        0xF5 => ("SBC", AddressingMode::DirectIndexedX),
        0xF9 => ("SBC", AddressingMode::AbsoluteIndexedY),
        0xFD => ("SBC", AddressingMode::AbsoluteIndexedX),
        // NOP, RTI, WAI, STP
        0xEA => ("NOP", AddressingMode::Implied),
        0x40 => ("RTI", AddressingMode::Implied),
        0xCB => ("WAI", AddressingMode::Implied),
        0xDB => ("STP", AddressingMode::Implied),
        // REP, SEP
        0xC2 => ("REP", AddressingMode::Immediate8),
        0xE2 => ("SEP", AddressingMode::Immediate8),
        // XCE
        0xFB => ("XCE", AddressingMode::Implied),
        // TCS, TSC
        0x1B => ("TCS", AddressingMode::Implied),
        0x3B => ("TSC", AddressingMode::Implied),
        // TCD, TDC
        0x5B => ("TCD", AddressingMode::Implied),
        0x7B => ("TDC", AddressingMode::Implied),
        // MVN, MVP
        0x54 => ("MVN", AddressingMode::BlockMove),
        0x44 => ("MVP", AddressingMode::BlockMove),
        // PEI, PER, PEA
        0xD4 => ("PEI", AddressingMode::Direct),
        0x62 => ("PER", AddressingMode::Relative16),
        0xF4 => ("PEA", AddressingMode::Absolute),
        // PHB, PLB, PHD, PLD, PHK, PHB
        0x8B => ("PHB", AddressingMode::Implied),
        0xAB => ("PLB", AddressingMode::Implied),
        0x0B => ("PHD", AddressingMode::Implied),
        0x2B => ("PLD", AddressingMode::Implied),
        0x4B => ("PHK", AddressingMode::Implied),
        // STZ
        0x64 => ("STZ", AddressingMode::Direct),
        0x74 => ("STZ", AddressingMode::DirectIndexedX),
        0x9C => ("STZ", AddressingMode::Absolute),
        0x9E => ("STZ", AddressingMode::AbsoluteIndexedX),
        // TRB, TSB
        0x14 => ("TRB", AddressingMode::Direct),
        0x1C => ("TRB", AddressingMode::Absolute),
        0x04 => ("TSB", AddressingMode::Direct),
        0x0C => ("TSB", AddressingMode::Absolute),
        // BIT
        0x24 => ("BIT", AddressingMode::Direct),
        0x2C => ("BIT", AddressingMode::Absolute),
        0x34 => ("BIT", AddressingMode::DirectIndexedX),
        0x3C => ("BIT", AddressingMode::AbsoluteIndexedX),
        0x89 => ("BIT", AddressingMode::Immediate8),
        // LSR, ROL, ROR
        0x46 => ("LSR", AddressingMode::Direct),
        0x4A => ("LSR", AddressingMode::Accumulator),
        0x4E => ("LSR", AddressingMode::Absolute),
        0x26 => ("ROL", AddressingMode::Direct),
        0x2A => ("ROL", AddressingMode::Accumulator),
        0x2E => ("ROL", AddressingMode::Absolute),
        0x66 => ("ROR", AddressingMode::Direct),
        0x6A => ("ROR", AddressingMode::Accumulator),
        0x6E => ("ROR", AddressingMode::Absolute),
        _ => return Err(AssemblyError::InvalidOpcode(opcode)),
    };

    Ok(result)
}

/// Resolve operand to an address value
fn resolve_operand(address: u32, mode: &AddressingMode, operand: &[u8], instr_size: u32) -> Option<u32> {
    match mode {
        AddressingMode::Absolute | AddressingMode::AbsoluteIndexedX | 
        AddressingMode::AbsoluteIndexedY | AddressingMode::AbsoluteIndirect |
        AddressingMode::AbsoluteIndexedIndirect => {
            if operand.len() >= 2 {
                let addr = (operand[0] as u32) | ((operand[1] as u32) << 8);
                // Add bank byte from instruction address
                let bank = (address >> 16) & 0xFF;
                Some(addr | (bank << 16))
            } else {
                None
            }
        }
        AddressingMode::AbsoluteLong | AddressingMode::AbsoluteLongIndexed |
        AddressingMode::AbsoluteIndirectLong => {
            if operand.len() >= 3 {
                Some((operand[0] as u32) | ((operand[1] as u32) << 8) | ((operand[2] as u32) << 16))
            } else {
                None
            }
        }
        AddressingMode::Relative8 => {
            operand.get(0).map(|&b| {
                let offset = b as i8 as i32;
                (address as i32 + instr_size as i32 + offset) as u32
            })
        }
        AddressingMode::Relative16 => {
            if operand.len() >= 2 {
                let offset = (operand[0] as i16) | ((operand[1] as i16) << 8);
                Some((address as i32 + instr_size as i32 + offset as i32) as u32)
            } else {
                None
            }
        }
        AddressingMode::Direct | AddressingMode::DirectIndexedX | 
        AddressingMode::DirectIndexedY | AddressingMode::DirectIndirect |
        AddressingMode::DirectIndexedIndirect | AddressingMode::DirectIndirectIndexed |
        AddressingMode::DirectIndirectLong | AddressingMode::DirectIndirectLongIndexed => {
            operand.get(0).map(|&b| b as u32)
        }
        _ => None,
    }
}

/// Format operand for display
fn format_operand(instr: &Instruction) -> String {
    match instr.mode {
        AddressingMode::Implied => String::new(),
        AddressingMode::Accumulator => "A".to_string(),
        AddressingMode::Immediate8 => format!("#${:02X}", instr.operand.get(0).unwrap_or(&0)),
        AddressingMode::Immediate16 => {
            if instr.operand.len() >= 2 {
                format!("#${:04X}", (instr.operand[0] as u16) | ((instr.operand[1] as u16) << 8))
            } else {
                "#$00".to_string()
            }
        }
        AddressingMode::Direct => format!("${:02X}", instr.operand.get(0).unwrap_or(&0)),
        AddressingMode::DirectIndexedX => format!("${:02X},X", instr.operand.get(0).unwrap_or(&0)),
        AddressingMode::DirectIndexedY => format!("${:02X},Y", instr.operand.get(0).unwrap_or(&0)),
        AddressingMode::DirectIndirect => format!("(${:02X})", instr.operand.get(0).unwrap_or(&0)),
        AddressingMode::DirectIndexedIndirect => format!("(${:02X},X)", instr.operand.get(0).unwrap_or(&0)),
        AddressingMode::DirectIndirectIndexed => format!("(${:02X}),Y", instr.operand.get(0).unwrap_or(&0)),
        AddressingMode::DirectIndirectLong => format!("[${:02X}]", instr.operand.get(0).unwrap_or(&0)),
        AddressingMode::DirectIndirectLongIndexed => format!("[${:02X}],Y", instr.operand.get(0).unwrap_or(&0)),
        AddressingMode::Absolute => {
            if let Some(addr) = instr.resolved_operand {
                format!("${:04X}", addr & 0xFFFF)
            } else {
                String::new()
            }
        }
        AddressingMode::AbsoluteIndexedX => {
            if let Some(addr) = instr.resolved_operand {
                format!("${:04X},X", addr & 0xFFFF)
            } else {
                String::new()
            }
        }
        AddressingMode::AbsoluteIndexedY => {
            if let Some(addr) = instr.resolved_operand {
                format!("${:04X},Y", addr & 0xFFFF)
            } else {
                String::new()
            }
        }
        AddressingMode::AbsoluteLong => {
            if let Some(addr) = instr.resolved_operand {
                format!("${:06X}", addr)
            } else {
                String::new()
            }
        }
        AddressingMode::AbsoluteLongIndexed => {
            if let Some(addr) = instr.resolved_operand {
                format!("${:06X},X", addr)
            } else {
                String::new()
            }
        }
        AddressingMode::AbsoluteIndirect => {
            if let Some(addr) = instr.resolved_operand {
                format!("(${:04X})", addr & 0xFFFF)
            } else {
                String::new()
            }
        }
        AddressingMode::AbsoluteIndexedIndirect => {
            if let Some(addr) = instr.resolved_operand {
                format!("(${:04X},X)", addr & 0xFFFF)
            } else {
                String::new()
            }
        }
        AddressingMode::AbsoluteIndirectLong => {
            if let Some(addr) = instr.resolved_operand {
                format!("[${:04X}]", addr & 0xFFFF)
            } else {
                String::new()
            }
        }
        AddressingMode::StackRelative => format!("${:02X},S", instr.operand.get(0).unwrap_or(&0)),
        AddressingMode::StackRelativeIndirectIndexed => format!("(${:02X},S),Y", instr.operand.get(0).unwrap_or(&0)),
        AddressingMode::BlockMove => {
            if instr.operand.len() >= 2 {
                format!("${:02X},${:02X}", instr.operand[1], instr.operand[0])
            } else {
                String::new()
            }
        }
        AddressingMode::Relative8 | AddressingMode::Relative16 => {
            if let Some(target) = instr.target_address() {
                format!("${:06X}", target)
            } else {
                String::new()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_opcode_lda_immediate() {
        let (mnemonic, mode) = decode_opcode(0xA9).unwrap();
        assert_eq!(mnemonic, "LDA");
        assert_eq!(mode, AddressingMode::Immediate8);
    }

    #[test]
    fn test_decode_opcode_jmp_absolute() {
        let (mnemonic, mode) = decode_opcode(0x4C).unwrap();
        assert_eq!(mnemonic, "JMP");
        assert_eq!(mode, AddressingMode::Absolute);
    }

    #[test]
    fn test_decode_opcode_jsr_absolute() {
        let (mnemonic, mode) = decode_opcode(0x20).unwrap();
        assert_eq!(mnemonic, "JSR");
        assert_eq!(mode, AddressingMode::Absolute);
    }

    #[test]
    fn test_decode_opcode_rts() {
        let (mnemonic, mode) = decode_opcode(0x60).unwrap();
        assert_eq!(mnemonic, "RTS");
        assert_eq!(mode, AddressingMode::Implied);
    }

    #[test]
    fn test_function_new() {
        let func = Function::new(0x8000, "test_func");
        assert_eq!(func.entry_point, 0x8000);
        assert_eq!(func.name, "test_func");
    }

    #[test]
    fn test_label_manager() {
        let mut lm = LabelManager::new();
        lm.add_label(0x8000, "start").unwrap();
        assert_eq!(lm.get_label(0x8000), Some(&"start".to_string()));
        assert_eq!(lm.get_address("start"), Some(0x8000));
    }
}
