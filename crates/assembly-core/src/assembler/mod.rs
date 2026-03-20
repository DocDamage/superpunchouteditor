//! # Assembler Module
//!
//! Inline assembler with 65816 assembly syntax parsing, label resolution,
//! macro support, and machine code generation.

use crate::{AddressingMode, AssemblyError, CpuMode, Instruction, Result};
use std::collections::HashMap;

/// An assembly statement
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    /// Label definition (e.g., "label:")
    Label(String),
    /// Instruction (e.g., "LDA #$10")
    Instruction(InstructionLine),
    /// Directive (e.g., ".db $10, $20")
    Directive(Directive),
    /// Macro invocation
    Macro(String, Vec<String>),
    /// Comment line
    Comment(String),
    /// Empty line
    Empty,
}

/// An instruction line
#[derive(Debug, Clone, PartialEq)]
pub struct InstructionLine {
    /// Mnemonic (e.g., "LDA")
    pub mnemonic: String,
    /// Operand expression
    pub operand: Option<Operand>,
    /// CPU mode override (if any)
    pub cpu_mode: Option<CpuMode>,
}

/// Operand types
#[derive(Debug, Clone, PartialEq)]
pub enum Operand {
    /// Immediate value (#$xx or #label)
    Immediate(String),
    /// Absolute address ($xxxx or label)
    Absolute(String),
    /// Absolute long address ($xxxxxx)
    AbsoluteLong(String),
    /// Direct page address ($xx)
    Direct(String),
    /// Indirect address
    Indirect(String),
    /// Indexed operand
    Indexed(Box<Operand>, IndexRegister),
    /// Indirect long ([$xx] or [label])
    IndirectLong(String),
    /// Stack relative
    StackRelative(String),
}

/// Index register
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IndexRegister {
    X,
    Y,
    S, // Stack
}

/// Assembler directives
#[derive(Debug, Clone, PartialEq)]
pub enum Directive {
    /// Define byte(s)
    Db(Vec<u8>),
    /// Define word(s)
    Dw(Vec<u16>),
    /// Define long(s)
    Dl(Vec<u32>),
    /// Define string
    Ds(String),
    /// Set origin
    Org(u32),
    /// Include file
    Include(String),
    /// Define macro
    MacroDef(String, Vec<String>, Vec<String>),
    /// Bank byte
    Bank(String),
    /// High byte
    High(String),
    /// Low byte
    Low(String),
}

/// Macro definition
#[derive(Debug, Clone)]
pub struct MacroDef {
    /// Macro name
    pub name: String,
    /// Parameter names
    pub params: Vec<String>,
    /// Macro body (lines)
    pub body: Vec<String>,
}

/// Symbol table entry
#[derive(Debug, Clone)]
pub struct Symbol {
    /// Symbol value (address or constant)
    pub value: u32,
    /// Whether this is a constant (vs address)
    pub is_constant: bool,
    /// Bank byte (if applicable)
    pub bank: Option<u8>,
}

/// Label information
#[derive(Debug, Clone)]
pub struct LabelInfo {
    /// Label address
    pub address: u32,
    /// Whether this is a local label
    pub is_local: bool,
    /// Parent function (for local labels)
    pub parent: Option<String>,
}

/// Assembler configuration
#[derive(Debug, Clone)]
pub struct AssemblerConfig {
    /// Default CPU mode
    pub cpu_mode: CpuMode,
    /// Assume 8-bit accumulator
    pub m_flag: bool,
    /// Assume 8-bit index registers
    pub x_flag: bool,
    /// Direct page register value
    pub direct_page: u16,
    /// Data bank register value
    pub data_bank: u8,
    /// Program bank register value
    pub program_bank: u8,
    /// Allow undefined labels (for first pass)
    pub allow_undefined: bool,
}

impl Default for AssemblerConfig {
    fn default() -> Self {
        Self {
            cpu_mode: CpuMode::Native,
            m_flag: false,
            x_flag: false,
            direct_page: 0,
            data_bank: 0,
            program_bank: 0,
            allow_undefined: false,
        }
    }
}

/// 65816 Inline Assembler
#[derive(Debug)]
pub struct Assembler {
    config: AssemblerConfig,
    /// Symbol table (name -> symbol)
    symbols: HashMap<String, Symbol>,
    /// Labels (name -> label info)
    labels: HashMap<String, LabelInfo>,
    /// Macros (name -> macro definition)
    pub macros: HashMap<String, MacroDef>,
    /// Current assembly address
    current_address: u32,
    /// Output bytes
    output: Vec<u8>,
    /// Unresolved references (to fix up in second pass)
    unresolved: Vec<UnresolvedRef>,
    /// Current local label prefix
    current_label_prefix: Option<String>,
}

/// Unresolved reference for second pass
#[derive(Debug, Clone)]
struct UnresolvedRef {
    /// Position in output where fixup is needed
    position: usize,
    /// Label name to resolve
    label: String,
    /// Size of the reference (1, 2, or 3 bytes)
    size: u8,
    /// Offset to add
    offset: i32,
    /// Type of reference
    ref_type: RefType,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum RefType {
    Absolute,
    Relative,
    Long,
    Bank,
    High,
    Low,
}

impl Assembler {
    /// Create a new assembler
    pub fn new(config: AssemblerConfig) -> Self {
        Self {
            config,
            symbols: HashMap::new(),
            labels: HashMap::new(),
            macros: HashMap::new(),
            current_address: 0,
            output: Vec::new(),
            unresolved: Vec::new(),
            current_label_prefix: None,
        }
    }

    /// Define a symbol (constant)
    pub fn define_symbol(&mut self, name: impl Into<String>, value: u32) {
        self.symbols.insert(
            name.into(),
            Symbol {
                value,
                is_constant: true,
                bank: None,
            },
        );
    }

    /// Define a label
    pub fn define_label(&mut self, name: impl Into<String>, address: u32) -> Result<()> {
        let name = name.into();
        
        if self.labels.contains_key(&name) {
            return Err(AssemblyError::DuplicateLabel(name));
        }

        let is_local = name.starts_with('.') || name.starts_with('@');
        
        self.labels.insert(
            name.clone(),
            LabelInfo {
                address,
                is_local,
                parent: self.current_label_prefix.clone(),
            },
        );

        // Update current label prefix for local labels
        if !is_local {
            self.current_label_prefix = Some(name);
        }

        Ok(())
    }

    /// Parse assembly source code
    pub fn parse(&mut self, source: &str) -> Result<Vec<Statement>> {
        let mut statements = Vec::new();

        for (_line_num, line) in source.lines().enumerate() {
            let line = line.trim();
            
            // Skip empty lines
            if line.is_empty() {
                statements.push(Statement::Empty);
                continue;
            }

            // Handle comments
            if line.starts_with(';') {
                statements.push(Statement::Comment(line[1..].to_string()));
                continue;
            }

            // Extract inline comment
            let (code_part, _comment) = if let Some(pos) = line.find(';') {
                (&line[..pos], Some(line[pos..].to_string()))
            } else {
                (line, None)
            };

            let code_part = code_part.trim();

            // Try to parse as label
            if let Some(label) = self.try_parse_label(code_part) {
                statements.push(Statement::Label(label));
                continue;
            }

            // Try to parse as directive
            if code_part.starts_with('.') || code_part.starts_with('%') {
                let directive = self.parse_directive(code_part)?;
                statements.push(Statement::Directive(directive));
                continue;
            }

            // Try to parse as macro invocation
            if let Some(macro_stmt) = self.try_parse_macro_invocation(code_part) {
                statements.push(macro_stmt);
                continue;
            }

            // Parse as instruction
            let instruction = self.parse_instruction(code_part)?;
            statements.push(Statement::Instruction(instruction));
        }

        Ok(statements)
    }

    /// Try to parse a label definition
    fn try_parse_label(&self, line: &str) -> Option<String> {
        if let Some(pos) = line.find(':') {
            let label = line[..pos].trim();
            if is_valid_label(label) {
                return Some(label.to_string());
            }
        }
        None
    }

    /// Parse an instruction
    fn parse_instruction(&self, line: &str) -> Result<InstructionLine> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        
        if parts.is_empty() {
            return Err(AssemblyError::InvalidSyntax("Empty instruction".to_string()));
        }

        let mnemonic = parts[0].to_uppercase();
        let operand = if parts.len() > 1 {
            let operand_str = parts[1..].join(" ");
            Some(self.parse_operand(&operand_str)?)
        } else {
            None
        };

        Ok(InstructionLine {
            mnemonic,
            operand,
            cpu_mode: None,
        })
    }

    /// Parse an operand
    fn parse_operand(&self, s: &str) -> Result<Operand> {
        let s = s.trim();

        // Immediate
        if s.starts_with('#') {
            return Ok(Operand::Immediate(s[1..].to_string()));
        }

        // Stack relative
        if s.contains(",S") || s.contains(",s") {
            let base = s.split(',').next().unwrap_or("").trim();
            return Ok(Operand::StackRelative(base.to_string()));
        }

        // Check for indexing
        let (base, index) = if s.contains(",X") || s.contains(",x") {
            (s.split(",X").next().unwrap_or("").trim(), Some(IndexRegister::X))
        } else if s.contains(",Y") || s.contains(",y") {
            (s.split(",Y").next().unwrap_or("").trim(), Some(IndexRegister::Y))
        } else {
            (s, None)
        };

        // Indirect long [addr]
        if base.starts_with('[') && base.ends_with(']') {
            let inner = &base[1..base.len() - 1];
            let operand = Operand::IndirectLong(inner.to_string());
            return match index {
                Some(IndexRegister::Y) => Ok(Operand::Indexed(Box::new(operand), IndexRegister::Y)),
                _ => Ok(operand),
            };
        }

        // Indirect (addr)
        if base.starts_with('(') && base.ends_with(')') {
            let inner = &base[1..base.len() - 1];
            
            // Check for (addr,X) - indexed indirect
            if inner.contains(",X") || inner.contains(",x") {
                let addr = inner.split(",X").next().unwrap_or("").trim();
                return Ok(Operand::Indexed(
                    Box::new(Operand::Indirect(addr.to_string())),
                    IndexRegister::X,
                ));
            }
            
            let operand = Operand::Indirect(inner.to_string());
            return match index {
                Some(IndexRegister::Y) => Ok(Operand::Indexed(Box::new(operand), IndexRegister::Y)),
                _ => Ok(operand),
            };
        }

        // Determine address size from prefix/suffix
        let operand = if base.starts_with('$') {
            let hex_str = &base[1..];
            if hex_str.len() <= 2 {
                Operand::Direct(base.to_string())
            } else if hex_str.len() <= 4 {
                Operand::Absolute(base.to_string())
            } else {
                Operand::AbsoluteLong(base.to_string())
            }
        } else if base.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
            // Numeric literal (assume hex)
            Operand::Absolute(format!("${}", base))
        } else {
            // Label reference - assume absolute
            Operand::Absolute(base.to_string())
        };

        match index {
            Some(reg) => Ok(Operand::Indexed(Box::new(operand), reg)),
            None => Ok(operand),
        }
    }

    /// Parse a directive
    fn parse_directive(&self, line: &str) -> Result<Directive> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        
        if parts.is_empty() {
            return Err(AssemblyError::InvalidSyntax("Empty directive".to_string()));
        }

        let directive = parts[0].to_lowercase();
        let args = &parts[1..];

        match directive.as_str() {
            ".db" | ".byte" => {
                let bytes = self.parse_data_bytes(args)?;
                Ok(Directive::Db(bytes))
            }
            ".dw" | ".word" => {
                let words = self.parse_data_words(args)?;
                Ok(Directive::Dw(words))
            }
            ".dl" | ".long" => {
                let longs = self.parse_data_longs(args)?;
                Ok(Directive::Dl(longs))
            }
            ".ds" | ".asc" | ".string" => {
                let s = args.join(" ");
                Ok(Directive::Ds(parse_string(&s)?))
            }
            ".org" | "*=" => {
                let addr = parse_number(args.first().unwrap_or(&"0"))?;
                Ok(Directive::Org(addr))
            }
            ".include" | ".incsrc" => {
                let path = args.join(" ");
                Ok(Directive::Include(parse_string(&path)?))
            }
            ".macro" => {
                if args.is_empty() {
                    return Err(AssemblyError::InvalidSyntax(
                        "Macro name required".to_string(),
                    ));
                }
                let name = args[0].to_string();
                let params: Vec<String> = args[1..].iter().map(|s| s.to_string()).collect();
                // Body would be collected separately
                Ok(Directive::MacroDef(name, params, Vec::new()))
            }
            ".bank" => {
                let expr = args.join(" ");
                Ok(Directive::Bank(expr))
            }
            ".high" | ">" => {
                let expr = args.join(" ");
                Ok(Directive::High(expr))
            }
            ".low" | "<" => {
                let expr = args.join(" ");
                Ok(Directive::Low(expr))
            }
            _ => Err(AssemblyError::InvalidSyntax(format!(
                "Unknown directive: {}",
                directive
            ))),
        }
    }

    /// Try to parse macro invocation
    fn try_parse_macro_invocation(&self, line: &str) -> Option<Statement> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            return None;
        }

        let name = parts[0];
        if self.macros.contains_key(name) {
            let args: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();
            return Some(Statement::Macro(name.to_string(), args));
        }

        None
    }

    /// Parse data bytes
    fn parse_data_bytes(&self, args: &[&str]) -> Result<Vec<u8>> {
        let mut bytes = Vec::new();
        for arg in args {
            let arg = arg.trim_end_matches(',');
            bytes.push(parse_number(arg)? as u8);
        }
        Ok(bytes)
    }

    /// Parse data words
    fn parse_data_words(&self, args: &[&str]) -> Result<Vec<u16>> {
        let mut words = Vec::new();
        for arg in args {
            let arg = arg.trim_end_matches(',');
            words.push(parse_number(arg)? as u16);
        }
        Ok(words)
    }

    /// Parse data longs
    fn parse_data_longs(&self, args: &[&str]) -> Result<Vec<u32>> {
        let mut longs = Vec::new();
        for arg in args {
            let arg = arg.trim_end_matches(',');
            longs.push(parse_number(arg)?);
        }
        Ok(longs)
    }

    /// Assemble source code
    pub fn assemble(&mut self, source: &str) -> Result<Vec<u8>> {
        let statements = self.parse(source)?;

        // First pass: collect labels and calculate addresses
        self.current_address = 0;
        self.output.clear();
        self.unresolved.clear();

        let mut temp_output = Vec::new();
        std::mem::swap(&mut self.output, &mut temp_output);

        // First pass
        self.config.allow_undefined = true;
        for statement in &statements {
            self.assemble_statement(statement)?;
        }

        // Second pass: resolve references
        std::mem::swap(&mut self.output, &mut temp_output);
        self.config.allow_undefined = false;
        self.current_address = 0;

        for statement in &statements {
            self.assemble_statement(statement)?;
        }

        // Apply fixups
        self.apply_fixups()?;

        Ok(self.output.clone())
    }

    /// Assemble a single statement
    fn assemble_statement(&mut self, statement: &Statement) -> Result<()> {
        match statement {
            Statement::Empty | Statement::Comment(_) => Ok(()),
            Statement::Label(name) => {
                self.define_label(name.clone(), self.current_address)?;
                Ok(())
            }
            Statement::Instruction(instr) => {
                let bytes = self.assemble_instruction(instr)?;
                self.output.extend(&bytes);
                self.current_address += bytes.len() as u32;
                Ok(())
            }
            Statement::Directive(dir) => self.assemble_directive(dir),
            Statement::Macro(name, args) => self.expand_macro(name, args),
        }
    }

    /// Assemble an instruction
    fn assemble_instruction(&mut self, instr: &InstructionLine) -> Result<Vec<u8>> {
        let opcode_info = self.select_opcode(&instr.mnemonic, &instr.operand)?;
        let mut bytes = vec![opcode_info.opcode];

        if let Some(ref operand) = instr.operand {
            let operand_bytes = self.encode_operand(operand, opcode_info.mode, opcode_info.size - 1)?;
            bytes.extend(operand_bytes);
        }

        Ok(bytes)
    }

    /// Select opcode based on mnemonic and operand
    fn select_opcode(&self, mnemonic: &str, operand: &Option<Operand>) -> Result<OpcodeInfo> {
        // This is a simplified opcode selection
        // In a full implementation, we'd have a complete opcode table

        let mode = operand.as_ref().map(|o| self.infer_mode(o)).unwrap_or(AddressingMode::Implied);

        let opcode = match (mnemonic, mode) {
            ("NOP", AddressingMode::Implied) => 0xEA,
            ("RTS", AddressingMode::Implied) => 0x60,
            ("RTL", AddressingMode::Implied) => 0x6B,
            ("RTI", AddressingMode::Implied) => 0x40,
            ("CLC", AddressingMode::Implied) => 0x18,
            ("SEC", AddressingMode::Implied) => 0x38,
            ("CLI", AddressingMode::Implied) => 0x58,
            ("SEI", AddressingMode::Implied) => 0x78,
            ("CLV", AddressingMode::Implied) => 0xB8,
            ("CLD", AddressingMode::Implied) => 0xD8,
            ("SED", AddressingMode::Implied) => 0xF8,
            ("INX", AddressingMode::Implied) => 0xE8,
            ("DEX", AddressingMode::Implied) => 0xCA,
            ("INY", AddressingMode::Implied) => 0xC8,
            ("DEY", AddressingMode::Implied) => 0x88,
            ("TAX", AddressingMode::Implied) => 0xAA,
            ("TXA", AddressingMode::Implied) => 0x8A,
            ("TAY", AddressingMode::Implied) => 0xA8,
            ("TYA", AddressingMode::Implied) => 0x98,
            ("TXS", AddressingMode::Implied) => 0x9A,
            ("TSX", AddressingMode::Implied) => 0xBA,
            ("PHA", AddressingMode::Implied) => 0x48,
            ("PLA", AddressingMode::Implied) => 0x68,
            ("PHP", AddressingMode::Implied) => 0x08,
            ("PLP", AddressingMode::Implied) => 0x28,
            ("PHX", AddressingMode::Implied) => 0xDA,
            ("PLX", AddressingMode::Implied) => 0xFA,
            ("PHY", AddressingMode::Implied) => 0x5A,
            ("PLY", AddressingMode::Implied) => 0x7A,
            ("XCE", AddressingMode::Implied) => 0xFB,
            _ => return Err(AssemblyError::InvalidSyntax(format!(
                "Cannot encode {} with {:?}", mnemonic, mode
            ))),
        };

        let size = 1 + mode.operand_size() as u8;

        Ok(OpcodeInfo { opcode, mode, size })
    }

    /// Infer addressing mode from operand
    fn infer_mode(&self, operand: &Operand) -> AddressingMode {
        match operand {
            Operand::Immediate(_) => {
                if self.config.m_flag {
                    AddressingMode::Immediate8
                } else {
                    AddressingMode::Immediate16
                }
            }
            Operand::Direct(_) => AddressingMode::Direct,
            Operand::Absolute(_) => AddressingMode::Absolute,
            Operand::AbsoluteLong(_) => AddressingMode::AbsoluteLong,
            Operand::Indirect(_) => AddressingMode::DirectIndirect,
            Operand::IndirectLong(_) => AddressingMode::DirectIndirectLong,
            Operand::StackRelative(_) => AddressingMode::StackRelative,
            Operand::Indexed(base, reg) => {
                let base_mode = self.infer_mode(base);
                match (base_mode, reg) {
                    (AddressingMode::Direct, IndexRegister::X) => AddressingMode::DirectIndexedX,
                    (AddressingMode::Direct, IndexRegister::Y) => AddressingMode::DirectIndexedY,
                    (AddressingMode::Absolute, IndexRegister::X) => AddressingMode::AbsoluteIndexedX,
                    (AddressingMode::Absolute, IndexRegister::Y) => AddressingMode::AbsoluteIndexedY,
                    (AddressingMode::AbsoluteLong, IndexRegister::X) => AddressingMode::AbsoluteLongIndexed,
                    (AddressingMode::DirectIndirect, IndexRegister::Y) => AddressingMode::DirectIndirectIndexed,
                    (AddressingMode::DirectIndirectLong, IndexRegister::Y) => AddressingMode::DirectIndirectLongIndexed,
                    (AddressingMode::StackRelative, IndexRegister::Y) => AddressingMode::StackRelativeIndirectIndexed,
                    _ => base_mode,
                }
            }
        }
    }

    /// Encode operand to bytes
    fn encode_operand(&mut self, operand: &Operand, mode: AddressingMode, size: u8) -> Result<Vec<u8>> {
        let value = self.resolve_operand_value(operand)?;

        match size {
            0 => Ok(Vec::new()),
            1 => Ok(vec![value as u8]),
            2 => Ok(vec![(value & 0xFF) as u8, ((value >> 8) & 0xFF) as u8]),
            3 => Ok(vec![
                (value & 0xFF) as u8,
                ((value >> 8) & 0xFF) as u8,
                ((value >> 16) & 0xFF) as u8,
            ]),
            _ => Err(AssemblyError::InvalidAddressingMode(format!(
                "Invalid operand size: {}",
                size
            ))),
        }
    }

    /// Resolve operand value
    fn resolve_operand_value(&mut self, operand: &Operand) -> Result<u32> {
        match operand {
            Operand::Immediate(s) | Operand::Direct(s) | Operand::Absolute(s) |
            Operand::AbsoluteLong(s) | Operand::Indirect(s) | Operand::IndirectLong(s) |
            Operand::StackRelative(s) => self.evaluate_expression(s),
            Operand::Indexed(base, _) => self.resolve_operand_value(base),
        }
    }

    /// Evaluate an expression string
    fn evaluate_expression(&mut self, expr: &str) -> Result<u32> {
        let expr = expr.trim();

        // Hex literal
        if expr.starts_with('$') {
            return u32::from_str_radix(&expr[1..], 16)
                .map_err(|_| AssemblyError::InvalidSyntax(format!("Invalid hex: {}", expr)));
        }

        // Binary literal
        if expr.starts_with('%') {
            return u32::from_str_radix(&expr[1..], 2)
                .map_err(|_| AssemblyError::InvalidSyntax(format!("Invalid binary: {}", expr)));
        }

        // Decimal literal
        if expr.chars().all(|c| c.is_ascii_digit()) {
            return expr.parse::<u32>()
                .map_err(|_| AssemblyError::InvalidSyntax(format!("Invalid number: {}", expr)));
        }

        // Label reference
        if let Some(label_info) = self.labels.get(expr) {
            return Ok(label_info.address);
        }

        // Symbol reference
        if let Some(symbol) = self.symbols.get(expr) {
            return Ok(symbol.value);
        }

        // Unresolved - add to fixup list if in first pass
        if self.config.allow_undefined {
            return Ok(0);
        }

        Err(AssemblyError::UndefinedLabel(expr.to_string()))
    }

    /// Assemble a directive
    fn assemble_directive(&mut self, directive: &Directive) -> Result<()> {
        match directive {
            Directive::Db(bytes) => {
                for &b in bytes {
                    self.output.push(b);
                }
                self.current_address += bytes.len() as u32;
            }
            Directive::Dw(words) => {
                for &w in words {
                    self.output.push((w & 0xFF) as u8);
                    self.output.push(((w >> 8) & 0xFF) as u8);
                }
                self.current_address += (words.len() * 2) as u32;
            }
            Directive::Dl(longs) => {
                for &l in longs {
                    self.output.push((l & 0xFF) as u8);
                    self.output.push(((l >> 8) & 0xFF) as u8);
                    self.output.push(((l >> 16) & 0xFF) as u8);
                }
                self.current_address += (longs.len() * 3) as u32;
            }
            Directive::Ds(s) => {
                for b in s.bytes() {
                    self.output.push(b);
                }
                self.current_address += s.len() as u32;
            }
            Directive::Org(addr) => {
                self.current_address = *addr;
            }
            Directive::Include(_) => {
                // Would need file system access
                return Err(AssemblyError::InvalidSyntax(
                    "Include not yet implemented".to_string(),
                ));
            }
            Directive::MacroDef(name, params, body) => {
                self.macros.insert(
                    name.clone(),
                    MacroDef {
                        name: name.clone(),
                        params: params.clone(),
                        body: body.clone(),
                    },
                );
            }
            Directive::Bank(expr) => {
                let value = self.evaluate_expression(expr)?;
                self.output.push(((value >> 16) & 0xFF) as u8);
                self.current_address += 1;
            }
            Directive::High(expr) => {
                let value = self.evaluate_expression(expr)?;
                self.output.push(((value >> 8) & 0xFF) as u8);
                self.current_address += 1;
            }
            Directive::Low(expr) => {
                let value = self.evaluate_expression(expr)?;
                self.output.push((value & 0xFF) as u8);
                self.current_address += 1;
            }
        }
        Ok(())
    }

    /// Expand a macro
    fn expand_macro(&mut self, name: &str, args: &[String]) -> Result<()> {
        let macro_def = self
            .macros
            .get(name)
            .ok_or_else(|| AssemblyError::InvalidSyntax(format!("Unknown macro: {}", name)))?
            .clone();

        if args.len() != macro_def.params.len() {
            return Err(AssemblyError::InvalidSyntax(format!(
                "Macro {} expects {} arguments, got {}",
                name,
                macro_def.params.len(),
                args.len()
            )));
        }

        // Create substitution map
        let substitutions: HashMap<&str, &str> = macro_def
            .params
            .iter()
            .zip(args.iter())
            .map(|(p, a)| (p.as_str(), a.as_str()))
            .collect();

        // Process macro body
        for line in &macro_def.body {
            let expanded = substitute_params(line, &substitutions);
            let statement = self.parse_instruction(&expanded)?;
            self.assemble_statement(&Statement::Instruction(statement))?;
        }

        Ok(())
    }

    /// Apply fixups for unresolved references
    fn apply_fixups(&mut self) -> Result<()> {
        for fixup in &self.unresolved {
            let value = if let Some(label_info) = self.labels.get(&fixup.label) {
                label_info.address
            } else if let Some(symbol) = self.symbols.get(&fixup.label) {
                symbol.value
            } else {
                return Err(AssemblyError::UndefinedLabel(fixup.label.clone()));
            };

            let adjusted = (value as i32 + fixup.offset) as u32;

            match fixup.size {
                1 => self.output[fixup.position] = (adjusted & 0xFF) as u8,
                2 => {
                    self.output[fixup.position] = (adjusted & 0xFF) as u8;
                    self.output[fixup.position + 1] = ((adjusted >> 8) & 0xFF) as u8;
                }
                3 => {
                    self.output[fixup.position] = (adjusted & 0xFF) as u8;
                    self.output[fixup.position + 1] = ((adjusted >> 8) & 0xFF) as u8;
                    self.output[fixup.position + 2] = ((adjusted >> 16) & 0xFF) as u8;
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Get the current assembly address
    pub fn current_address(&self) -> u32 {
        self.current_address
    }

    /// Get the assembled output
    pub fn output(&self) -> &[u8] {
        &self.output
    }
}

/// Opcode information
#[derive(Debug, Clone)]
struct OpcodeInfo {
    opcode: u8,
    mode: AddressingMode,
    size: u8,
}

/// Check if a string is a valid label
fn is_valid_label(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    let first = s.chars().next().unwrap();
    if !first.is_alphabetic() && first != '_' && first != '.' && first != '@' {
        return false;
    }

    s.chars().all(|c| c.is_alphanumeric() || c == '_')
}

/// Parse a number from string (hex, binary, or decimal)
fn parse_number(s: &str) -> Result<u32> {
    let s = s.trim();

    if s.starts_with('$') {
        u32::from_str_radix(&s[1..], 16)
            .map_err(|_| AssemblyError::InvalidSyntax(format!("Invalid hex: {}", s)))
    } else if s.starts_with('%') {
        u32::from_str_radix(&s[1..], 2)
            .map_err(|_| AssemblyError::InvalidSyntax(format!("Invalid binary: {}", s)))
    } else {
        s.parse::<u32>()
            .map_err(|_| AssemblyError::InvalidSyntax(format!("Invalid number: {}", s)))
    }
}

/// Parse a string literal
fn parse_string(s: &str) -> Result<String> {
    let s = s.trim();
    
    if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
        Ok(s[1..s.len() - 1].to_string())
    } else {
        Ok(s.to_string())
    }
}

/// Substitute parameters in a macro body
fn substitute_params(line: &str, substitutions: &HashMap<&str, &str>) -> String {
    let mut result = line.to_string();
    for (param, arg) in substitutions {
        result = result.replace(&format!("{{{{{}}}}}", param), arg);
    }
    result
}

/// Define a macro
#[macro_export]
macro_rules! define_asm_macro {
    ($assembler:expr, $name:expr, [$($param:expr),*], [$($line:expr),*]) => {
        {
            let params = vec![$($param.to_string()),*];
            let body = vec![$($line.to_string()),*];
            $assembler.macros.insert(
                $name.to_string(),
                crate::assembler::MacroDef {
                    name: $name.to_string(),
                    params,
                    body,
                }
            );
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_label() {
        let asm = Assembler::new(AssemblerConfig::default());
        assert_eq!(asm.try_parse_label("start:"), Some("start".to_string()));
        assert_eq!(asm.try_parse_label("loop_1:"), Some("loop_1".to_string()));
        assert_eq!(asm.try_parse_label("lda #$10"), None);
    }

    #[test]
    fn test_parse_operand_immediate() {
        let asm = Assembler::new(AssemblerConfig::default());
        let operand = asm.parse_operand("#$10").unwrap();
        assert_eq!(operand, Operand::Immediate("$10".to_string()));
    }

    #[test]
    fn test_parse_operand_absolute() {
        let asm = Assembler::new(AssemblerConfig::default());
        let operand = asm.parse_operand("$2000").unwrap();
        assert_eq!(operand, Operand::Absolute("$2000".to_string()));
    }

    #[test]
    fn test_parse_operand_indexed() {
        let asm = Assembler::new(AssemblerConfig::default());
        let operand = asm.parse_operand("$2000,X").unwrap();
        assert_eq!(
            operand,
            Operand::Indexed(
                Box::new(Operand::Absolute("$2000".to_string())),
                IndexRegister::X
            )
        );
    }

    #[test]
    fn test_parse_number() {
        assert_eq!(parse_number("$10").unwrap(), 16);
        assert_eq!(parse_number("%1010").unwrap(), 10);
        assert_eq!(parse_number("255").unwrap(), 255);
        assert_eq!(parse_number("$FF").unwrap(), 255);
    }

    #[test]
    fn test_assemble_nop() {
        let mut asm = Assembler::new(AssemblerConfig::default());
        let result = asm.assemble("NOP").unwrap();
        assert_eq!(result, vec![0xEA]);
    }

    #[test]
    fn test_assemble_rts() {
        let mut asm = Assembler::new(AssemblerConfig::default());
        let result = asm.assemble("RTS").unwrap();
        assert_eq!(result, vec![0x60]);
    }

    #[test]
    fn test_assemble_with_label() {
        let mut asm = Assembler::new(AssemblerConfig::default());
        let source = r#"
start:
    NOP
    RTS
"#;
        let result = asm.assemble(source).unwrap();
        assert_eq!(result, vec![0xEA, 0x60]);
        assert_eq!(asm.labels.get("start").unwrap().address, 0);
    }

    #[test]
    fn test_assemble_db_directive() {
        let mut asm = Assembler::new(AssemblerConfig::default());
        let result = asm.assemble(".db $10, $20, $30").unwrap();
        assert_eq!(result, vec![0x10, 0x20, 0x30]);
    }

    #[test]
    fn test_assemble_dw_directive() {
        let mut asm = Assembler::new(AssemblerConfig::default());
        let result = asm.assemble(".dw $1234, $5678").unwrap();
        assert_eq!(result, vec![0x34, 0x12, 0x78, 0x56]);
    }

    #[test]
    fn test_define_symbol() {
        let mut asm = Assembler::new(AssemblerConfig::default());
        asm.define_symbol("MY_CONST", 0x1234);
        assert_eq!(asm.symbols.get("MY_CONST").unwrap().value, 0x1234);
    }
}
