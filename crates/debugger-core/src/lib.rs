//! # Debugger Core
//!
//! Advanced debugging tools for Super Punch-Out!! ROM hacking.
//!
//! This crate provides:
//! - 65816 CPU debugger with breakpoints and stepping
//! - SPC700 audio debugger
//! - Memory heatmap and access tracking
//! - Execution tracing
//!
//! ## Example
//!
//! ```rust
//! use debugger_core::{SnesDebugger, Breakpoint, BreakCondition, SnesAddress};
//!
//! let mut debugger = SnesDebugger::new();
//!
//! // Set a breakpoint on health modification
//! let bp = Breakpoint::with_condition(
//!     SnesAddress::new(0x05, 0x8234),
//!     BreakCondition::OnWrite { min_addr: 0x7E0100, max_addr: 0x7E0101 }
//! );
//! debugger.add_breakpoint(bp);
//! ```

pub mod cpu;
pub mod memory;
pub mod spc700;
pub mod tracer;
pub mod types;

pub use cpu::{CpuDebugger, StepResult};
pub use memory::{MemoryHeatmap, MemoryWatcher, AccessType};
pub use spc700::{Spc700Debugger, DspRegisterState, AudioChannelState};
pub use tracer::{ExecutionTracer, TraceEntry, TraceFilter};
pub use types::*;



/// Main debugger facade that coordinates all debugging subsystems
pub struct SnesDebugger {
    cpu: CpuDebugger,
    memory: MemoryWatcher,
    spc700: Option<Spc700Debugger>,
    tracer: ExecutionTracer,
    #[allow(dead_code)]
    state: DebuggerState,
}

impl SnesDebugger {
    pub fn new() -> Self {
        Self {
            cpu: CpuDebugger::new(),
            memory: MemoryWatcher::new(),
            spc700: None,
            tracer: ExecutionTracer::new(),
            state: DebuggerState::new(),
        }
    }

    /// Enable SPC700 audio debugging
    pub fn with_spc700(mut self) -> Self {
        self.spc700 = Some(Spc700Debugger::new());
        self
    }

    /// Get current CPU register state
    pub fn cpu_registers(&self) -> &RegisterState {
        self.cpu.registers()
    }

    /// Add a breakpoint
    pub fn add_breakpoint(&mut self, breakpoint: Breakpoint) -> BreakpointId {
        self.cpu.add_breakpoint(breakpoint)
    }

    /// Remove a breakpoint
    pub fn remove_breakpoint(&mut self, id: BreakpointId) {
        let _ = self.cpu.remove_breakpoint(id);
    }

    /// Step one instruction
    pub fn step(&mut self) -> StepResult {
        let result = self.cpu.step();
        
        if let Some(ref _spc700) = self.spc700 {
            // Sync SPC700 state if needed
        }
        
        result
    }

    /// Step over (skip into subroutines)
    pub fn step_over(&mut self) -> StepResult {
        self.cpu.step_over()
    }

    /// Step out (return from subroutine)
    pub fn step_out(&mut self) -> StepResult {
        self.cpu.step_out()
    }

    /// Run until breakpoint
    pub fn run(&mut self) -> StepResult {
        self.cpu.run()
    }

    /// Get memory at address
    pub fn read_memory(&self, addr: u32, size: usize) -> Vec<u8> {
        self.memory.read_silent(addr, size)
    }

    /// Write memory
    pub fn write_memory(&mut self, addr: u32, data: &[u8]) {
        self.memory.write(addr, data);
    }

    /// Get memory heatmap
    pub fn memory_heatmap(&self, range: std::ops::Range<u32>) -> MemoryHeatmap {
        self.memory.heatmap(range)
    }

    /// Start execution tracing
    pub fn start_tracing(&mut self, filter: TraceFilter) {
        self.tracer.start(filter);
    }

    /// Stop execution tracing
    pub fn stop_tracing(&mut self) -> Vec<TraceEntry> {
        self.tracer.stop()
    }

    /// Get call stack
    pub fn call_stack(&self) -> Vec<StackFrame> {
        self.cpu.call_stack()
    }
}

impl Default for SnesDebugger {
    fn default() -> Self {
        Self::new()
    }
}

/// Debugger state
#[derive(Debug, Clone)]
pub struct DebuggerState {
    pub running: bool,
    pub paused: bool,
    pub hit_breakpoint: Option<BreakpointId>,
    pub step_count: u64,
}

impl DebuggerState {
    pub fn new() -> Self {
        Self {
            running: false,
            paused: false,
            hit_breakpoint: None,
            step_count: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BreakpointId(pub u64);

#[derive(Debug, Clone)]
pub struct StackFrame {
    pub return_addr: u32,
    pub function_addr: Option<u32>,
    pub stack_depth: usize,
}

#[derive(Debug, Clone)]
pub struct RunResult {
    pub hit_breakpoint: Option<BreakpointId>,
    pub steps_executed: u64,
    pub cycles_elapsed: u64,
}

/// Errors that can occur during debugging
#[derive(Debug, thiserror::Error)]
pub enum DebuggerError {
    #[error("Invalid address: {0:06X}")]
    InvalidAddress(u32),
    
    #[error("Breakpoint not found: {0:?}")]
    BreakpointNotFound(BreakpointId),
    
    #[error("Emulator not running")]
    EmulatorNotRunning,
    
    #[error("Invalid register: {0}")]
    InvalidRegister(String),
    
    #[error("Memory access violation: {0:06X}")]
    MemoryViolation(u32),
}

pub type DebuggerResult<T> = Result<T, DebuggerError>;
