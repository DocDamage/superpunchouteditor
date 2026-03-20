//! Execution Tracer
//!
//! Provides instruction-level execution tracing with filtering capabilities.
//! Uses a circular buffer to maintain a history of executed instructions.

use crate::types::{DisassembledInstruction, RegisterState, SnesAddress};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// A single trace entry recording an executed instruction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceEntry {
    /// Address where the instruction was executed
    pub address: SnesAddress,
    /// Disassembled instruction
    pub instruction: DisassembledInstruction,
    /// CPU register state after execution
    pub registers: RegisterState,
    /// Stack pointer at time of execution
    pub stack_pointer: u16,
    /// Cycle count
    pub cycle: u64,
    /// Frame number (if applicable)
    pub frame: u64,
    /// Scanline (if applicable)
    pub scanline: u16,
    /// Horizontal counter (if applicable)
    pub h_counter: u16,
    /// Additional context/metadata
    pub metadata: TraceMetadata,
}

/// Additional metadata for trace entries
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TraceMetadata {
    /// Memory values accessed by this instruction
    pub memory_accesses: Vec<MemoryAccessRecord>,
    /// Custom notes/tags
    pub tags: Vec<String>,
    /// Was this a breakpoint hit?
    pub breakpoint_hit: bool,
    /// Was this a DMA transfer?
    pub dma_transfer: bool,
    /// IRQ/NMI triggered?
    pub interrupt: Option<String>,
}

/// Record of a memory access during instruction execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryAccessRecord {
    /// Address accessed
    pub address: u32,
    /// Type of access
    pub access_type: MemoryAccessType,
    /// Value read or written
    pub value: u16,
    /// Was this an 8-bit or 16-bit access
    pub is_16bit: bool,
}

/// Type of memory access
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemoryAccessType {
    Read,
    Write,
}

impl TraceEntry {
    /// Create a new trace entry
    pub fn new(
        address: SnesAddress,
        instruction: DisassembledInstruction,
        registers: RegisterState,
        cycle: u64,
    ) -> Self {
        Self {
            address,
            instruction,
            registers,
            stack_pointer: registers.sp,
            cycle,
            frame: 0,
            scanline: 0,
            h_counter: 0,
            metadata: TraceMetadata::default(),
        }
    }

    /// Add a memory access record
    pub fn add_memory_access(&mut self, addr: u32, access_type: MemoryAccessType, value: u16, is_16bit: bool) {
        self.metadata.memory_accesses.push(MemoryAccessRecord {
            address: addr,
            access_type,
            value,
            is_16bit,
        });
    }

    /// Tag this entry
    pub fn tag(&mut self, tag: impl Into<String>) {
        self.metadata.tags.push(tag.into());
    }

    /// Check if this entry has a specific tag
    pub fn has_tag(&self, tag: &str) -> bool {
        self.metadata.tags.iter().any(|t| t == tag)
    }

    /// Get the instruction as a formatted string
    pub fn formatted_instruction(&self) -> String {
        format!("{}  {}", self.instruction.mnemonic, self.instruction.operands)
    }

    /// Get a summary line for display
    pub fn summary(&self) -> String {
        format!(
            "{:06X}  {}  A:{:04X} X:{:04X} Y:{:04X} S:{:04X} P:{:02X}",
            self.address.to_pc(),
            self.formatted_instruction(),
            self.registers.a,
            self.registers.x,
            self.registers.y,
            self.stack_pointer,
            self.registers.p.0
        )
    }
}

/// Filter criteria for execution tracing
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TraceFilter {
    /// Only trace addresses in these ranges (inclusive)
    pub address_ranges: Vec<std::ops::RangeInclusive<u32>>,
    /// Exclude these address ranges
    pub exclude_ranges: Vec<std::ops::RangeInclusive<u32>>,
    /// Only trace specific instruction types
    pub instruction_types: InstructionTypeFilter,
    /// Only trace when register meets condition
    pub register_conditions: Vec<RegisterCondition>,
    /// Maximum number of entries to record (0 = unlimited)
    pub max_entries: usize,
    /// Only trace every Nth instruction (for sampling)
    pub sample_rate: u64,
    /// Enable memory access recording
    pub record_memory_accesses: bool,
    /// Stop after N cycles (0 = don't stop)
    pub stop_after_cycles: u64,
}

/// Filter for instruction types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstructionTypeFilter {
    /// Trace branches
    pub branches: bool,
    /// Trace calls
    pub calls: bool,
    /// Trace returns
    pub returns: bool,
    /// Trace loads
    pub loads: bool,
    /// Trace stores
    pub stores: bool,
    /// Trace arithmetic operations
    pub arithmetic: bool,
    /// Trace only these specific opcodes (empty = all)
    pub specific_opcodes: HashSet<u8>,
    /// Exclude these opcodes
    pub excluded_opcodes: HashSet<u8>,
}

impl Default for InstructionTypeFilter {
    fn default() -> Self {
        // Default to allowing all instruction types
        Self::all()
    }
}

impl InstructionTypeFilter {
    /// Allow all instruction types
    pub fn all() -> Self {
        Self {
            branches: true,
            calls: true,
            returns: true,
            loads: true,
            stores: true,
            arithmetic: true,
            specific_opcodes: HashSet::new(),
            excluded_opcodes: HashSet::new(),
        }
    }

    /// Only control flow (branches, calls, returns)
    pub fn control_flow_only() -> Self {
        Self {
            branches: true,
            calls: true,
            returns: true,
            loads: false,
            stores: false,
            arithmetic: false,
            specific_opcodes: HashSet::new(),
            excluded_opcodes: HashSet::new(),
        }
    }

    /// Only memory operations (loads and stores)
    pub fn memory_only() -> Self {
        Self {
            branches: false,
            calls: false,
            returns: false,
            loads: true,
            stores: true,
            arithmetic: false,
            specific_opcodes: HashSet::new(),
            excluded_opcodes: HashSet::new(),
        }
    }

    /// Check if an instruction should be traced based on this filter
    pub fn matches(&self, instruction: &DisassembledInstruction) -> bool {
        // Check excluded opcodes first
        if self.excluded_opcodes.contains(&instruction.bytes[0]) {
            return false;
        }

        // If specific opcodes are specified, only trace those
        if !self.specific_opcodes.is_empty() {
            return self.specific_opcodes.contains(&instruction.bytes[0]);
        }

        // Check instruction type
        let mnemonic = instruction.mnemonic.as_str();

        if instruction.is_branch && !self.branches {
            return false;
        }
        if instruction.is_call && !self.calls {
            return false;
        }
        if instruction.is_return && !self.returns {
            return false;
        }

        // Check loads/stores
        let is_load = mnemonic.starts_with("LD") || mnemonic == "PLA" || mnemonic == "PLX" || mnemonic == "PLY";
        let is_store = mnemonic.starts_with("ST") || mnemonic.starts_with("PH");
        let is_arithmetic = matches!(
            mnemonic,
            "ADC" | "SBC" | "INC" | "DEC" | "INX" | "DEX" | "INY" | "DEY"
                | "AND" | "ORA" | "EOR" | "ASL" | "LSR" | "ROL" | "ROR"
                | "CMP" | "CPX" | "CPY" | "BIT"
        );

        if is_load && !self.loads {
            return false;
        }
        if is_store && !self.stores {
            return false;
        }
        if is_arithmetic && !self.arithmetic {
            return false;
        }

        true
    }
}

/// Register condition for filtering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterCondition {
    /// Which register to check
    pub register: RegisterFilter,
    /// Condition to evaluate
    pub condition: FilterCondition,
}

/// Register to filter on
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RegisterFilter {
    A,
    X,
    Y,
    SP,
    DP,
    DB,
    PB,
    PC,
    P,
}

/// Condition for register filtering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterCondition {
    /// Equal to value
    Equal(u16),
    /// Not equal to value
    NotEqual(u16),
    /// Greater than value
    GreaterThan(u16),
    /// Less than value
    LessThan(u16),
    /// Bitwise AND with mask is non-zero
    BitSet(u16),
    /// Bitwise AND with mask is zero
    BitClear(u16),
}

impl FilterCondition {
    /// Evaluate the condition against a value
    pub fn evaluate(&self, value: u16) -> bool {
        match self {
            FilterCondition::Equal(v) => value == *v,
            FilterCondition::NotEqual(v) => value != *v,
            FilterCondition::GreaterThan(v) => value > *v,
            FilterCondition::LessThan(v) => value < *v,
            FilterCondition::BitSet(mask) => (value & *mask) != 0,
            FilterCondition::BitClear(mask) => (value & *mask) == 0,
        }
    }
}

/// Execution tracer with circular buffer
#[derive(Debug)]
pub struct ExecutionTracer {
    /// Circular buffer of trace entries
    entries: Vec<TraceEntry>,
    /// Current position in circular buffer
    position: usize,
    /// Capacity of the buffer
    capacity: usize,
    /// Whether tracing is currently active
    active: bool,
    /// Current filter
    filter: TraceFilter,
    /// Total instructions traced (including filtered out)
    total_instructions: u64,
    /// Instructions that passed the filter
    matched_instructions: u64,
    /// Start cycle
    start_cycle: u64,
    /// Current cycle
    current_cycle: u64,
}

impl Default for ExecutionTracer {
    fn default() -> Self {
        Self::new()
    }
}

impl ExecutionTracer {
    /// Create a new execution tracer with default capacity (10000 entries)
    pub fn new() -> Self {
        Self::with_capacity(10000)
    }

    /// Create a tracer with a specific buffer capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            entries: Vec::with_capacity(capacity),
            position: 0,
            capacity,
            active: false,
            filter: TraceFilter::default(),
            total_instructions: 0,
            matched_instructions: 0,
            start_cycle: 0,
            current_cycle: 0,
        }
    }

    /// Start tracing with the given filter
    pub fn start(&mut self, filter: TraceFilter) {
        self.active = true;
        self.filter = filter;
        self.total_instructions = 0;
        self.matched_instructions = 0;
        self.start_cycle = self.current_cycle;
    }

    /// Stop tracing and return the captured entries
    pub fn stop(&mut self) -> Vec<TraceEntry> {
        self.active = false;
        self.get_entries()
    }

    /// Check if tracing is active
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Add an entry to the trace
    pub fn trace(&mut self, entry: TraceEntry) {
        if !self.active {
            return;
        }

        self.total_instructions += 1;

        // Check sample rate
        if self.filter.sample_rate > 0 && (self.total_instructions % self.filter.sample_rate) != 0 {
            return;
        }

        // Apply filter
        if !self.should_trace(&entry) {
            return;
        }

        self.matched_instructions += 1;

        // Check max cycles
        if self.filter.stop_after_cycles > 0 {
            let elapsed = entry.cycle.saturating_sub(self.start_cycle);
            if elapsed >= self.filter.stop_after_cycles {
                self.active = false;
            }
        }

        // Add to circular buffer
        if self.entries.len() < self.capacity {
            self.entries.push(entry);
        } else {
            self.entries[self.position] = entry;
        }
        self.position = (self.position + 1) % self.capacity;

        // Check max entries
        if self.filter.max_entries > 0 && self.matched_instructions >= self.filter.max_entries as u64 {
            self.active = false;
        }
    }

    /// Determine if an entry should be traced based on the filter
    fn should_trace(&self, entry: &TraceEntry) -> bool {
        // Check address ranges
        let addr = entry.address.to_pc();

        if !self.filter.address_ranges.is_empty() {
            let in_range = self.filter.address_ranges.iter()
                .any(|r| r.contains(&addr));
            if !in_range {
                return false;
            }
        }

        // Check excluded ranges
        let excluded = self.filter.exclude_ranges.iter()
            .any(|r| r.contains(&addr));
        if excluded {
            return false;
        }

        // Check instruction type filter
        if !self.filter.instruction_types.matches(&entry.instruction) {
            return false;
        }

        // Check register conditions
        for condition in &self.filter.register_conditions {
            let reg_value = match condition.register {
                RegisterFilter::A => entry.registers.a,
                RegisterFilter::X => entry.registers.x,
                RegisterFilter::Y => entry.registers.y,
                RegisterFilter::SP => entry.registers.sp,
                RegisterFilter::DP => entry.registers.dp,
                RegisterFilter::DB => entry.registers.db as u16,
                RegisterFilter::PB => entry.registers.pb as u16,
                RegisterFilter::PC => entry.registers.pc,
                RegisterFilter::P => entry.registers.p.0 as u16,
            };

            if !condition.condition.evaluate(reg_value) {
                return false;
            }
        }

        true
    }

    /// Get all entries in order (oldest first)
    pub fn get_entries(&self) -> Vec<TraceEntry> {
        if self.entries.len() < self.capacity {
            self.entries.clone()
        } else {
            // Reorder circular buffer
            let mut result = Vec::with_capacity(self.capacity);
            for i in 0..self.capacity {
                let idx = (self.position + i) % self.capacity;
                result.push(self.entries[idx].clone());
            }
            result
        }
    }

    /// Get the most recent N entries
    pub fn get_recent(&self, count: usize) -> Vec<TraceEntry> {
        let entries = self.get_entries();
        let start = entries.len().saturating_sub(count);
        entries[start..].to_vec()
    }

    /// Get the last entry
    pub fn last(&self) -> Option<&TraceEntry> {
        if self.entries.is_empty() {
            return None;
        }
        let idx = if self.entries.len() < self.capacity {
            self.entries.len() - 1
        } else {
            (self.position + self.capacity - 1) % self.capacity
        };
        self.entries.get(idx)
    }

    /// Clear all entries
    pub fn clear(&mut self) {
        self.entries.clear();
        self.position = 0;
    }

    /// Get current statistics
    pub fn stats(&self) -> TraceStats {
        TraceStats {
            total_instructions: self.total_instructions,
            matched_instructions: self.matched_instructions,
            buffered_entries: self.entries.len() as u64,
            is_active: self.active,
            elapsed_cycles: self.current_cycle.saturating_sub(self.start_cycle),
        }
    }

    /// Update the current cycle counter
    pub fn tick_cycle(&mut self) {
        self.current_cycle += 1;
    }

    /// Set the current cycle counter
    pub fn set_cycle(&mut self, cycle: u64) {
        self.current_cycle = cycle;
    }

    /// Search for entries matching a condition
    pub fn search<F>(&self, predicate: F) -> Vec<&TraceEntry>
    where
        F: Fn(&TraceEntry) -> bool,
    {
        self.entries.iter().filter(|e| predicate(e)).collect()
    }

    /// Find entries by address
    pub fn find_by_address(&self, addr: SnesAddress) -> Vec<&TraceEntry> {
        self.search(|e| e.address == addr)
    }

    /// Find entries by mnemonic
    pub fn find_by_mnemonic(&self, mnemonic: &str) -> Vec<&TraceEntry> {
        let mnemonic_upper = mnemonic.to_uppercase();
        self.search(|e| e.instruction.mnemonic == mnemonic_upper)
    }

    /// Export trace to a string format
    pub fn export(&self) -> String {
        let mut output = String::new();
        for entry in self.get_entries() {
            output.push_str(&entry.summary());
            output.push('\n');
        }
        output
    }
}

/// Statistics for the tracer
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct TraceStats {
    /// Total instructions processed
    pub total_instructions: u64,
    /// Instructions that passed the filter
    pub matched_instructions: u64,
    /// Entries currently in buffer
    pub buffered_entries: u64,
    /// Is tracing currently active
    pub is_active: bool,
    /// Elapsed cycles since start
    pub elapsed_cycles: u64,
}

impl TraceStats {
    /// Get the filter acceptance rate
    pub fn filter_rate(&self) -> f64 {
        if self.total_instructions == 0 {
            0.0
        } else {
            self.matched_instructions as f64 / self.total_instructions as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::DisassembledInstruction;

    fn create_test_instruction(mnemonic: &str) -> DisassembledInstruction {
        DisassembledInstruction {
            address: SnesAddress::new(0x00, 0x8000),
            bytes: vec![0xEA],
            mnemonic: mnemonic.to_string(),
            operands: String::new(),
            size: 1,
            cycles: 2,
            is_branch: mnemonic.starts_with("B") || mnemonic == "JMP" || mnemonic == "JML",
            is_call: mnemonic == "JSR" || mnemonic == "JSL",
            is_return: mnemonic == "RTS" || mnemonic == "RTL" || mnemonic == "RTI",
        }
    }

    fn create_test_entry(mnemonic: &str, addr: u32) -> TraceEntry {
        let mut regs = RegisterState::default();
        regs.a = 0x1234;
        regs.x = 0x5678;
        
        TraceEntry::new(
            SnesAddress::from_pc(addr),
            create_test_instruction(mnemonic),
            regs,
            0,
        )
    }

    #[test]
    fn test_trace_entry_creation() {
        let entry = create_test_entry("LDA", 0x008000);
        
        assert_eq!(entry.instruction.mnemonic, "LDA");
        assert_eq!(entry.registers.a, 0x1234);
        assert_eq!(entry.address.to_pc(), 0x008000);
    }

    #[test]
    fn test_circular_buffer() {
        let mut tracer = ExecutionTracer::with_capacity(3);
        
        tracer.start(TraceFilter::default());
        
        tracer.trace(create_test_entry("LDA", 0x008000));
        tracer.trace(create_test_entry("STA", 0x008001));
        tracer.trace(create_test_entry("INX", 0x008002));
        tracer.trace(create_test_entry("INY", 0x008003)); // Should wrap around
        
        let entries = tracer.get_entries();
        
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].instruction.mnemonic, "STA"); // Oldest
        assert_eq!(entries[2].instruction.mnemonic, "INY"); // Newest
    }

    #[test]
    fn test_address_filter() {
        let mut tracer = ExecutionTracer::new();
        let mut filter = TraceFilter::default();
        filter.address_ranges.push(0x008000..=0x0080FF);
        
        tracer.start(filter);
        
        tracer.trace(create_test_entry("LDA", 0x007FFF)); // Outside range
        tracer.trace(create_test_entry("STA", 0x008000)); // In range
        tracer.trace(create_test_entry("INX", 0x008100)); // Outside range
        
        let entries = tracer.get_entries();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].instruction.mnemonic, "STA");
    }

    #[test]
    fn test_instruction_type_filter() {
        let mut tracer = ExecutionTracer::new();
        let mut filter = TraceFilter::default();
        filter.instruction_types = InstructionTypeFilter::control_flow_only();
        
        tracer.start(filter);
        
        tracer.trace(create_test_entry("LDA", 0x008000));
        tracer.trace(create_test_entry("JSR", 0x008001));
        tracer.trace(create_test_entry("STA", 0x008002));
        tracer.trace(create_test_entry("RTS", 0x008003));
        
        let entries = tracer.get_entries();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].instruction.mnemonic, "JSR");
        assert_eq!(entries[1].instruction.mnemonic, "RTS");
    }

    #[test]
    fn test_sample_rate() {
        let mut tracer = ExecutionTracer::new();
        let mut filter = TraceFilter::default();
        filter.sample_rate = 2; // Every 2nd instruction
        
        tracer.start(filter);
        
        for i in 0..10 {
            tracer.trace(create_test_entry("LDA", 0x008000 + i));
        }
        
        let entries = tracer.get_entries();
        assert_eq!(entries.len(), 5); // 0, 2, 4, 6, 8
    }

    #[test]
    fn test_filter_condition() {
        let cond = FilterCondition::Equal(0x1234);
        assert!(cond.evaluate(0x1234));
        assert!(!cond.evaluate(0x1235));

        let cond = FilterCondition::GreaterThan(100);
        assert!(cond.evaluate(101));
        assert!(!cond.evaluate(100));

        let cond = FilterCondition::BitSet(0x8000);
        assert!(cond.evaluate(0x8000));
        assert!(cond.evaluate(0x8001));
        assert!(!cond.evaluate(0x7FFF));
    }

    #[test]
    fn test_search() {
        let mut tracer = ExecutionTracer::new();
        tracer.start(TraceFilter::default());
        
        tracer.trace(create_test_entry("LDA", 0x008000));
        tracer.trace(create_test_entry("STA", 0x008001));
        tracer.trace(create_test_entry("LDA", 0x008002));
        
        let lda_entries = tracer.find_by_mnemonic("LDA");
        assert_eq!(lda_entries.len(), 2);
        
        let addr_entries = tracer.find_by_address(SnesAddress::from_pc(0x008001));
        assert_eq!(addr_entries.len(), 1);
    }

    #[test]
    fn test_trace_stats() {
        let mut tracer = ExecutionTracer::new();
        
        let mut filter = TraceFilter::default();
        filter.instruction_types = InstructionTypeFilter::memory_only();
        
        tracer.start(filter);
        
        tracer.trace(create_test_entry("LDA", 0x008000)); // Load - matches
        tracer.trace(create_test_entry("STA", 0x008001)); // Store - matches
        tracer.trace(create_test_entry("INX", 0x008002)); // Arithmetic - no match
        tracer.trace(create_test_entry("JMP", 0x008003)); // Branch - no match
        
        let stats = tracer.stats();
        
        assert_eq!(stats.total_instructions, 4);
        assert_eq!(stats.matched_instructions, 2);
        assert_eq!(stats.filter_rate(), 0.5);
    }
}
