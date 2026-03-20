//! 65816 CPU debugger

use crate::types::*;
use crate::{BreakpointId, DebuggerResult, StackFrame};
use std::collections::HashMap;

mod disassembler;

pub use disassembler::{Disassembler, DisassemblyRange};

/// CPU debugger for the 65816 processor
pub struct CpuDebugger {
    registers: RegisterState,
    breakpoints: HashMap<u64, Breakpoint>,
    next_breakpoint_id: u64,
    step_over_breakpoint: Option<u64>,
    step_out_target: Option<u32>,
    call_stack: Vec<StackFrame>,
    memory: Box<dyn MemoryAccess>,
    cycles: u64,
}

impl CpuDebugger {
    pub fn new() -> Self {
        Self {
            registers: RegisterState::default(),
            breakpoints: HashMap::new(),
            next_breakpoint_id: 1,
            step_over_breakpoint: None,
            step_out_target: None,
            call_stack: Vec::new(),
            memory: Box::new(DummyMemory),
            cycles: 0,
        }
    }

    pub fn with_memory(mut self, memory: Box<dyn MemoryAccess>) -> Self {
        self.memory = memory;
        self
    }

    /// Get current register state
    pub fn registers(&self) -> &RegisterState {
        &self.registers
    }

    /// Set register value
    pub fn set_register(&mut self, reg: Register, value: u16) -> DebuggerResult<()> {
        match reg {
            Register::A => self.registers.a = value,
            Register::X => self.registers.x = value,
            Register::Y => self.registers.y = value,
            Register::SP => self.registers.sp = value,
            Register::DP => self.registers.dp = value,
            Register::DB => self.registers.db = value as u8,
            Register::PB => self.registers.pb = value as u8,
            Register::PC => self.registers.pc = value,
        }
        Ok(())
    }

    /// Add a breakpoint
    pub fn add_breakpoint(&mut self, mut breakpoint: Breakpoint) -> BreakpointId {
        let id = self.next_breakpoint_id;
        self.next_breakpoint_id += 1;
        
        breakpoint.id = Some(id);
        self.breakpoints.insert(id, breakpoint);
        
        BreakpointId(id)
    }

    /// Remove a breakpoint
    pub fn remove_breakpoint(&mut self, id: BreakpointId) -> DebuggerResult<()> {
        self.breakpoints
            .remove(&id.0)
            .ok_or(crate::DebuggerError::BreakpointNotFound(id))?;
        Ok(())
    }

    /// Enable/disable a breakpoint
    pub fn toggle_breakpoint(&mut self, id: BreakpointId, enabled: bool) -> DebuggerResult<()> {
        if let Some(bp) = self.breakpoints.get_mut(&id.0) {
            bp.enabled = enabled;
            Ok(())
        } else {
            Err(crate::DebuggerError::BreakpointNotFound(id))
        }
    }

    /// Get all breakpoints
    pub fn breakpoints(&self) -> Vec<&Breakpoint> {
        self.breakpoints.values().collect()
    }

    /// Step one instruction
    pub fn step(&mut self) -> StepResult {
        let pc = self.registers.full_pc();
        let instruction = self.fetch_and_decode();
        let cycles = instruction.cycles;
        
        // Check for breakpoints
        let hit_breakpoint = self.check_breakpoints(&pc, &instruction);
        
        // Execute instruction (simplified - real implementation would execute)
        self.execute_instruction(&instruction);
        
        // Update call stack
        self.update_call_stack(&instruction);
        
        self.cycles += cycles as u64;
        
        StepResult {
            instruction,
            hit_breakpoint,
            cycles: cycles as u64,
        }
    }

    /// Step over (don't enter subroutines)
    pub fn step_over(&mut self) -> StepResult {
        let current_pc = self.registers.full_pc();
        let instruction = self.fetch_and_decode();
        
        if instruction.is_call {
            // Set temporary breakpoint after call
            let return_addr = current_pc.to_pc() + instruction.size as u32;
            let temp_bp = self.add_breakpoint(Breakpoint {
                id: None,
                address: SnesAddress::from_pc(return_addr),
                condition: BreakCondition::Always,
                enabled: true,
                hit_count: 0,
                hit_limit: Some(1),
                description: Some("Step over".to_string()),
            });
            self.step_over_breakpoint = Some(temp_bp.0);
            
            // Run until we hit it
            self.run()
        } else {
            self.step()
        }
    }

    /// Step out (return from subroutine)
    pub fn step_out(&mut self) -> StepResult {
        if let Some(frame) = self.call_stack.last() {
            self.step_out_target = Some(frame.return_addr);
            self.run()
        } else {
            // Not in a subroutine, just step
            self.step()
        }
    }

    /// Run until breakpoint
    pub fn run(&mut self) -> StepResult {
        loop {
            let result = self.step();
            
            if result.hit_breakpoint.is_some() {
                return result;
            }
            
            // Check for step out
            if let Some(target) = self.step_out_target {
                if self.registers.full_pc().to_pc() == target {
                    self.step_out_target = None;
                    return result;
                }
            }
        }
    }

    /// Get call stack
    pub fn call_stack(&self) -> Vec<StackFrame> {
        self.call_stack.clone()
    }

    /// Get current PC
    pub fn pc(&self) -> SnesAddress {
        self.registers.full_pc()
    }

    /// Disassemble at address
    pub fn disassemble_at(&self, addr: SnesAddress, count: usize) -> Vec<DisassembledInstruction> {
        let mut result = Vec::new();
        let mut current_addr = addr;
        
        for _ in 0..count {
            if let Some(instr) = self.fetch_and_decode_at(current_addr) {
                current_addr = SnesAddress::from_pc(
                    current_addr.to_pc() + instr.size as u32
                );
                result.push(instr);
            } else {
                break;
            }
        }
        
        result
    }

    // Private methods
    
    fn fetch_and_decode(&self) -> DisassembledInstruction {
        let pc = self.registers.full_pc();
        self.fetch_and_decode_at(pc)
            .expect("Failed to decode instruction")
    }

    fn fetch_and_decode_at(&self, addr: SnesAddress) -> Option<DisassembledInstruction> {
        // Read opcode byte
        let opcode = self.memory.read_byte(addr.to_pc())?;
        
        // Decode based on opcode (simplified)
        disassembler::decode_instruction(addr, opcode, &*self.memory)
    }

    fn execute_instruction(&mut self, instruction: &DisassembledInstruction) {
        // Simplified execution - real implementation would properly execute 65816
        self.registers.pc = (self.registers.pc + instruction.size as u16) & 0xFFFF;
        
        // Update program bank on long jumps
        if instruction.mnemonic == "JML" || instruction.mnemonic == "JSL" {
            // Would update PB register
        }
    }

    fn check_breakpoints(
        &mut self,
        pc: &SnesAddress,
        _instruction: &DisassembledInstruction,
    ) -> Option<BreakpointId> {
        // Pre-compute all register values to avoid borrow issues
        let reg_values: std::collections::HashMap<Register, u16> = [
            (Register::A, self.registers.a),
            (Register::X, self.registers.x),
            (Register::Y, self.registers.y),
            (Register::SP, self.registers.sp),
            (Register::DP, self.registers.dp),
            (Register::DB, self.registers.db as u16),
            (Register::PB, self.registers.pb as u16),
            (Register::PC, self.registers.pc),
        ]
        .into_iter()
        .collect();
        
        let mut to_remove = None;
        let mut result = None;
        
        for (id, bp) in self.breakpoints.iter_mut() {
            if !bp.enabled {
                continue;
            }
            
            let matches = match &bp.condition {
                BreakCondition::Always => true,
                BreakCondition::OnExecute => bp.address == *pc,
                BreakCondition::OnRead { min_addr: _, max_addr: _ } => {
                    // Check if instruction reads from this range
                    false // Simplified
                }
                BreakCondition::OnWrite { min_addr: _, max_addr: _ } => {
                    // Check if instruction writes to this range
                    false // Simplified
                }
                BreakCondition::RegisterEquals { reg, value } => {
                    reg_values.get(reg).copied().unwrap_or(0) == *value
                }
                BreakCondition::Expression(_) => {
                    // Would evaluate expression
                    false
                }
            };
            
            if matches {
                bp.hit_count += 1;
                
                if let Some(limit) = bp.hit_limit {
                    if bp.hit_count >= limit {
                        // Mark temporary breakpoints for removal
                        if bp.description.as_ref().map(|d| d.contains("Step")).unwrap_or(false) {
                            to_remove = Some(*id);
                        }
                        result = Some(BreakpointId(*id));
                        break;
                    }
                } else {
                    result = Some(BreakpointId(*id));
                    break;
                }
            }
        }
        
        // Remove temporary breakpoint outside the loop
        if let Some(id) = to_remove {
            self.breakpoints.remove(&id);
        }
        
        result
    }

    fn get_register_value(&self, reg: Register) -> u16 {
        match reg {
            Register::A => self.registers.a,
            Register::X => self.registers.x,
            Register::Y => self.registers.y,
            Register::SP => self.registers.sp,
            Register::DP => self.registers.dp,
            Register::DB => self.registers.db as u16,
            Register::PB => self.registers.pb as u16,
            Register::PC => self.registers.pc,
        }
    }

    fn update_call_stack(&mut self, instruction: &DisassembledInstruction) {
        if instruction.is_call {
            let return_addr = instruction.address.to_pc() + instruction.size as u32;
            self.call_stack.push(StackFrame {
                return_addr,
                function_addr: Some(instruction.address.to_pc()),
                stack_depth: self.call_stack.len(),
            });
        } else if instruction.is_return {
            self.call_stack.pop();
        }
    }
}

impl Default for CpuDebugger {
    fn default() -> Self {
        Self::new()
    }
}

/// Memory access trait for the CPU debugger
pub trait MemoryAccess: Send + Sync {
    fn read_byte(&self, addr: u32) -> Option<u8>;
    fn read_word(&self, addr: u32) -> Option<u16> {
        let lo = self.read_byte(addr)?;
        let hi = self.read_byte(addr + 1)?;
        Some(u16::from_le_bytes([lo, hi]))
    }
    fn read_bytes(&self, addr: u32, count: usize) -> Vec<u8> {
        (0..count)
            .filter_map(|i| self.read_byte(addr + i as u32))
            .collect()
    }
    fn write_byte(&mut self, addr: u32, value: u8);
    fn write_bytes(&mut self, addr: u32, values: &[u8]);
}

struct DummyMemory;

impl MemoryAccess for DummyMemory {
    fn read_byte(&self, _addr: u32) -> Option<u8> {
        Some(0xEA) // NOP
    }
    
    fn write_byte(&mut self, _addr: u32, _value: u8) {}
    fn write_bytes(&mut self, _addr: u32, _values: &[u8]) {}
}

/// Result of a step operation
#[derive(Debug, Clone)]
pub struct StepResult {
    pub instruction: DisassembledInstruction,
    pub hit_breakpoint: Option<BreakpointId>,
    pub cycles: u64,
}

impl StepResult {
    pub fn stopped(&self) -> bool {
        self.hit_breakpoint.is_some()
    }
}
