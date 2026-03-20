//! # assembly-core
//!
//! Deep System Layer - Assembly tools for 65816/SNES ROM manipulation.
//!
//! This crate provides comprehensive assembly/disassembly capabilities:
//! - Full ROM disassembler with function detection
//! - Inline assembler with macro support
//! - Code patching with trampoline generation

use thiserror::Error;

pub mod assembler;
pub mod disassembler;
pub mod patcher;

/// Re-export commonly used types
pub use assembler::Assembler;
pub use disassembler::Disassembler;
pub use patcher::CodePatcher;

/// Errors that can occur in assembly operations
#[derive(Error, Debug, Clone, PartialEq)]
pub enum AssemblyError {
    #[error("Invalid opcode: {0:#04X}")]
    InvalidOpcode(u8),

    #[error("Invalid addressing mode: {0}")]
    InvalidAddressingMode(String),

    #[error("Undefined label: {0}")]
    UndefinedLabel(String),

    #[error("Duplicate label: {0}")]
    DuplicateLabel(String),

    #[error("Invalid instruction syntax: {0}")]
    InvalidSyntax(String),

    #[error("Address out of range: {0:#06X}")]
    AddressOutOfRange(u32),

    #[error("Patch too large: {size} bytes at address {address:#06X}")]
    PatchTooLarge { size: usize, address: u32 },

    #[error("No space for trampoline")]
    NoTrampolineSpace,

    #[error("ROM error: {0}")]
    RomError(String),

    #[error("IO error: {0}")]
    IoError(String),
}

/// Result type for assembly operations
pub type Result<T> = std::result::Result<T, AssemblyError>;

/// CPU mode for 65816
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CpuMode {
    /// Native mode (16-bit)
    Native,
    /// Emulation mode (8-bit, 6502 compatible)
    Emulation,
}

impl Default for CpuMode {
    fn default() -> Self {
        CpuMode::Native
    }
}

/// Instruction size information
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InstructionInfo {
    /// Size of the instruction in bytes
    pub size: u8,
    /// Number of cycles (minimum)
    pub cycles: u8,
    /// Whether this is a branch/jump instruction
    pub is_branch: bool,
    /// Whether this is a subroutine call
    pub is_call: bool,
    /// Whether this is a return instruction
    pub is_return: bool,
}

/// Addressing mode for 65816 instructions
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AddressingMode {
    /// Implied (no operand)
    Implied,
    /// Accumulator
    Accumulator,
    /// Immediate (#$xx or #$xxxx)
    Immediate8,
    Immediate16,
    /// Direct Page ($xx)
    Direct,
    /// Direct Page Indexed,X ($xx,X)
    DirectIndexedX,
    /// Direct Page Indexed,Y ($xx,Y)
    DirectIndexedY,
    /// Direct Page Indirect (($xx))
    DirectIndirect,
    /// Direct Page Indexed Indirect (($xx,X))
    DirectIndexedIndirect,
    /// Direct Page Indirect Indexed (($xx),Y)
    DirectIndirectIndexed,
    /// Direct Page Indirect Long ([$xx])
    DirectIndirectLong,
    /// Direct Page Indirect Long Indexed,Y ([$xx],Y)
    DirectIndirectLongIndexed,
    /// Absolute ($xxxx)
    Absolute,
    /// Absolute Indexed,X ($xxxx,X)
    AbsoluteIndexedX,
    /// Absolute Indexed,Y ($xxxx,Y)
    AbsoluteIndexedY,
    /// Absolute Long ($xxxxxx)
    AbsoluteLong,
    /// Absolute Long Indexed,X ($xxxxxx,X)
    AbsoluteLongIndexed,
    /// Absolute Indirect (($xxxx))
    AbsoluteIndirect,
    /// Absolute Indexed Indirect (($xxxx,X))
    AbsoluteIndexedIndirect,
    /// Absolute Indirect Long ([$xxxx])
    AbsoluteIndirectLong,
    /// Stack Relative ($xx,S)
    StackRelative,
    /// Stack Relative Indirect Indexed (($xx,S),Y)
    StackRelativeIndirectIndexed,
    /// Block Move
    BlockMove,
    /// Relative (branch, $xx offset)
    Relative8,
    /// Relative Long (branch, $xxxx offset)
    Relative16,
}

impl AddressingMode {
    /// Get the size of the operand in bytes (not including opcode)
    pub fn operand_size(&self) -> u8 {
        match self {
            AddressingMode::Implied | AddressingMode::Accumulator => 0,
            AddressingMode::Immediate8
            | AddressingMode::Direct
            | AddressingMode::DirectIndexedX
            | AddressingMode::DirectIndexedY
            | AddressingMode::DirectIndirect
            | AddressingMode::DirectIndexedIndirect
            | AddressingMode::DirectIndirectIndexed
            | AddressingMode::DirectIndirectLong
            | AddressingMode::DirectIndirectLongIndexed
            | AddressingMode::StackRelative
            | AddressingMode::StackRelativeIndirectIndexed
            | AddressingMode::Relative8 => 1,
            AddressingMode::Immediate16
            | AddressingMode::Absolute
            | AddressingMode::AbsoluteIndexedX
            | AddressingMode::AbsoluteIndexedY
            | AddressingMode::AbsoluteIndirect
            | AddressingMode::AbsoluteIndexedIndirect
            | AddressingMode::Relative16 => 2,
            AddressingMode::AbsoluteLong
            | AddressingMode::AbsoluteLongIndexed
            | AddressingMode::AbsoluteIndirectLong
            | AddressingMode::BlockMove => 3,
        }
    }
}

/// A decoded instruction
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Instruction {
    /// Address of the instruction
    pub address: u32,
    /// Opcode byte
    pub opcode: u8,
    /// Mnemonic (e.g., "LDA", "JSR")
    pub mnemonic: String,
    /// Addressing mode
    pub mode: AddressingMode,
    /// Operand bytes (raw)
    #[serde(skip)]
    pub operand: Vec<u8>,
    /// Resolved operand value (if applicable)
    pub resolved_operand: Option<u32>,
    /// Instruction bytes (opcode + operand)
    #[serde(skip)]
    pub bytes: Vec<u8>,
}

impl Instruction {
    /// Get the total size of the instruction
    pub fn size(&self) -> usize {
        self.bytes.len()
    }

    /// Check if this is a branch/jump instruction
    pub fn is_branch(&self) -> bool {
        matches!(
            self.mnemonic.as_str(),
            "BCC" | "BCS" | "BEQ" | "BMI" | "BNE" | "BPL" | "BRA" | "BRL" | "BVC" | "BVS"
        )
    }

    /// Check if this is a jump instruction
    pub fn is_jump(&self) -> bool {
        matches!(self.mnemonic.as_str(), "JMP" | "JML")
    }

    /// Check if this is a subroutine call
    pub fn is_call(&self) -> bool {
        matches!(self.mnemonic.as_str(), "JSR" | "JSL")
    }

    /// Check if this is a return instruction
    pub fn is_return(&self) -> bool {
        matches!(self.mnemonic.as_str(), "RTS" | "RTL" | "RTI")
    }

    /// Get the target address for branch/jump/call instructions
    pub fn target_address(&self) -> Option<u32> {
        if self.is_branch() {
            // Calculate relative target
            let offset = match self.mode {
                AddressingMode::Relative8 => {
                    self.resolved_operand? as i8 as i32
                }
                AddressingMode::Relative16 => {
                    self.resolved_operand? as i16 as i32
                }
                _ => return None,
            };
            let target = (self.address as i32 + self.size() as i32 + offset) as u32;
            Some(target)
        } else if self.is_jump() || self.is_call() {
            self.resolved_operand
        } else {
            None
        }
    }
}

/// Section type for disassembly
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SectionType {
    /// Code section
    Code,
    /// Data section
    Data,
    /// Unknown/unclassified
    Unknown,
}

/// A section in the ROM
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Section {
    /// Start address
    pub start: u32,
    /// End address (exclusive)
    pub end: u32,
    /// Section type
    pub section_type: SectionType,
    /// Label name (if any)
    pub label: Option<String>,
}

/// Export format for disassembly
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    /// Standard assembly format
    Assembly,
    /// With binary bytes as comments
    AssemblyWithBytes,
    /// HTML with syntax highlighting
    Html,
    /// JSON format
    Json,
    /// Plain text
    Text,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_addressing_mode_sizes() {
        assert_eq!(AddressingMode::Implied.operand_size(), 0);
        assert_eq!(AddressingMode::Immediate8.operand_size(), 1);
        assert_eq!(AddressingMode::Immediate16.operand_size(), 2);
        assert_eq!(AddressingMode::Absolute.operand_size(), 2);
        assert_eq!(AddressingMode::AbsoluteLong.operand_size(), 3);
    }

    #[test]
    fn test_cpu_mode_default() {
        assert_eq!(CpuMode::default(), CpuMode::Native);
    }
}
