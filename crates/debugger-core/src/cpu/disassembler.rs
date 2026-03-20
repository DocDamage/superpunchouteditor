//! 65816 Disassembler
//!
//! Decodes WDC 65816 instructions with full support for all addressing modes.
//! Handles both 8-bit and 16-bit modes based on processor status flags.

use crate::types::{DisassembledInstruction, SnesAddress};
use crate::cpu::MemoryAccess;

/// Addressing mode for 65816 instructions
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AddressingMode {
    /// Immediate - #const
    Immediate,
    /// Immediate 16-bit - #const (for M=0 or X=0)
    Immediate16,
    /// Absolute - addr
    Absolute,
    /// Absolute Long - longaddr
    AbsoluteLong,
    /// Direct Page - dp
    Direct,
    /// Direct Page Indirect - (dp)
    DirectIndirect,
    /// Direct Page Indirect Long - [dp]
    DirectIndirectLong,
    /// Absolute Indexed X - addr,X
    AbsoluteX,
    /// Absolute Long Indexed X - longaddr,X
    AbsoluteLongX,
    /// Absolute Indexed Y - addr,Y
    AbsoluteY,
    /// Direct Page Indexed X - dp,X
    DirectX,
    /// Direct Page Indexed Y - dp,Y
    DirectY,
    /// Direct Page Indirect Indexed X - (dp,X)
    DirectIndirectX,
    /// Direct Page Indirect Indexed Y - (dp),Y
    DirectIndirectY,
    /// Direct Page Indirect Long Indexed Y - [dp],Y
    DirectIndirectLongY,
    /// Accumulator - A
    Accumulator,
    /// Stack Relative - sr,S
    StackRelative,
    /// Stack Relative Indirect Indexed Y - (sr,S),Y
    StackRelativeIndirectY,
    /// Block Move - srcbank,destbank
    BlockMove,
    /// Implied/Implicit
    Implied,
    /// Program Counter Relative - rel
    Relative,
    /// Program Counter Relative Long - rel
    RelativeLong,
    /// Absolute Indirect - (addr)
    AbsoluteIndirect,
    /// Absolute Indirect Long - [addr]
    AbsoluteIndirectLong,
    /// Absolute Indexed Indirect - (addr,X)
    AbsoluteIndexedIndirect,
}

impl AddressingMode {
    /// Get the instruction size in bytes (excluding opcode byte)
    pub fn operand_size(&self, is_8bit_accumulator: bool, is_8bit_index: bool) -> u8 {
        match self {
            AddressingMode::Implied => 0,
            AddressingMode::Accumulator => 0,
            AddressingMode::Immediate => {
                if is_8bit_accumulator {
                    1
                } else {
                    2
                }
            }
            AddressingMode::Immediate16 => {
                if is_8bit_index {
                    1
                } else {
                    2
                }
            }
            AddressingMode::Direct => 1,
            AddressingMode::DirectIndirect => 1,
            AddressingMode::DirectIndirectLong => 1,
            AddressingMode::DirectX => 1,
            AddressingMode::DirectY => 1,
            AddressingMode::DirectIndirectX => 1,
            AddressingMode::DirectIndirectY => 1,
            AddressingMode::DirectIndirectLongY => 1,
            AddressingMode::StackRelative => 1,
            AddressingMode::StackRelativeIndirectY => 1,
            AddressingMode::Absolute => 2,
            AddressingMode::AbsoluteX => 2,
            AddressingMode::AbsoluteY => 2,
            AddressingMode::AbsoluteIndirect => 2,
            AddressingMode::AbsoluteLong => 3,
            AddressingMode::AbsoluteLongX => 3,
            AddressingMode::AbsoluteIndirectLong => 2,
            AddressingMode::AbsoluteIndexedIndirect => 2,
            AddressingMode::Relative => 1,
            AddressingMode::RelativeLong => 2,
            AddressingMode::BlockMove => 2,
        }
    }

    /// Format operands based on addressing mode
    pub fn format_operands(
        &self,
        bytes: &[u8],
        pc: u32,
        is_8bit_accumulator: bool,
        is_8bit_index: bool,
    ) -> String {
        match self {
            AddressingMode::Implied => String::new(),
            AddressingMode::Accumulator => String::new(),
            AddressingMode::Immediate => {
                if is_8bit_accumulator {
                    format!("#${:02X}", bytes[0])
                } else {
                    let value = u16::from_le_bytes([bytes[0], bytes[1]]);
                    format!("#${:04X}", value)
                }
            }
            AddressingMode::Immediate16 => {
                if is_8bit_index {
                    format!("#${:02X}", bytes[0])
                } else {
                    let value = u16::from_le_bytes([bytes[0], bytes[1]]);
                    format!("#${:04X}", value)
                }
            }
            AddressingMode::Absolute => {
                let addr = u16::from_le_bytes([bytes[0], bytes[1]]);
                format!("${:04X}", addr)
            }
            AddressingMode::AbsoluteLong => {
                let addr = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], 0]);
                format!("${:06X}", addr)
            }
            AddressingMode::Direct => {
                format!("${:02X}", bytes[0])
            }
            AddressingMode::DirectIndirect => {
                format!("(${:02X})", bytes[0])
            }
            AddressingMode::DirectIndirectLong => {
                format!("[${:02X}]", bytes[0])
            }
            AddressingMode::AbsoluteX => {
                let addr = u16::from_le_bytes([bytes[0], bytes[1]]);
                format!("${:04X},X", addr)
            }
            AddressingMode::AbsoluteLongX => {
                let addr = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], 0]);
                format!("${:06X},X", addr)
            }
            AddressingMode::AbsoluteY => {
                let addr = u16::from_le_bytes([bytes[0], bytes[1]]);
                format!("${:04X},Y", addr)
            }
            AddressingMode::DirectX => {
                format!("${:02X},X", bytes[0])
            }
            AddressingMode::DirectY => {
                format!("${:02X},Y", bytes[0])
            }
            AddressingMode::DirectIndirectX => {
                format!("(${:02X},X)", bytes[0])
            }
            AddressingMode::DirectIndirectY => {
                format!("(${:02X}),Y", bytes[0])
            }
            AddressingMode::DirectIndirectLongY => {
                format!("[${:02X}],Y", bytes[0])
            }
            AddressingMode::StackRelative => {
                format!("${:02X},S", bytes[0])
            }
            AddressingMode::StackRelativeIndirectY => {
                format!("(${:02X},S),Y", bytes[0])
            }
            AddressingMode::Relative => {
                let offset = bytes[0] as i8;
                // For branches, calculate target within the current 64KB bank
                // Use the lower 16 bits of PC and ignore the LoROM mapping
                let current_addr = (pc & 0x7FFF) | 0x8000;
                let target = (current_addr as u32).wrapping_add(2).wrapping_add(offset as u32) & 0xFFFF;
                format!("${:04X}", target)
            }
            AddressingMode::RelativeLong => {
                let offset = i16::from_le_bytes([bytes[0], bytes[1]]);
                let current_addr = (pc & 0x7FFF) | 0x8000;
                let target = (current_addr as u32).wrapping_add(offset as u32).wrapping_add(3) & 0xFFFF;
                format!("${:04X}", target)
            }
            AddressingMode::AbsoluteIndirect => {
                let addr = u16::from_le_bytes([bytes[0], bytes[1]]);
                format!("(${:04X})", addr)
            }
            AddressingMode::AbsoluteIndirectLong => {
                let addr = u16::from_le_bytes([bytes[0], bytes[1]]);
                format!("[${:04X}]", addr)
            }
            AddressingMode::AbsoluteIndexedIndirect => {
                let addr = u16::from_le_bytes([bytes[0], bytes[1]]);
                format!("(${:04X},X)", addr)
            }
            AddressingMode::BlockMove => {
                format!("${:02X},${:02X}", bytes[1], bytes[0])
            }
        }
    }
}

/// Instruction definition
#[derive(Debug, Clone)]
pub struct InstructionDef {
    pub opcode: u8,
    pub mnemonic: &'static str,
    pub addressing_mode: AddressingMode,
    pub cycles: u8,
    pub is_branch: bool,
    pub is_call: bool,
    pub is_return: bool,
}

/// Disassembler for 65816 instructions
#[derive(Debug, Clone, Default)]
pub struct Disassembler;

impl Disassembler {
    /// Create a new disassembler
    pub fn new() -> Self {
        Self
    }

    /// Disassemble a single instruction at the given address
    pub fn disassemble(
        &self,
        address: SnesAddress,
        memory: &dyn MemoryAccess,
        is_8bit_accumulator: bool,
        is_8bit_index: bool,
    ) -> Option<DisassembledInstruction> {
        decode_instruction_at(address, memory, is_8bit_accumulator, is_8bit_index)
    }
}

/// Disassembly range specification
#[derive(Debug, Clone)]
pub struct DisassemblyRange {
    pub start: SnesAddress,
    pub end: SnesAddress,
}

impl DisassemblyRange {
    pub fn new(start: SnesAddress, end: SnesAddress) -> Self {
        Self { start, end }
    }
}

/// Decode a single instruction from memory
/// This is the main entry point used by the CPU debugger
pub fn decode_instruction(
    address: SnesAddress,
    _opcode: u8,
    memory: &dyn MemoryAccess,
) -> Option<DisassembledInstruction> {
    // Default to 8-bit mode for basic decoding
    decode_instruction_at(address, memory, true, true)
}

/// Decode instruction with explicit mode flags
fn decode_instruction_at(
    address: SnesAddress,
    memory: &dyn MemoryAccess,
    is_8bit_accumulator: bool,
    is_8bit_index: bool,
) -> Option<DisassembledInstruction> {
    let pc = address.to_pc();
    let opcode = memory.read_byte(pc)?;
    
    let def = get_instruction_def(opcode)?;
    
    // Determine operand size based on instruction and flags
    let operand_size = if def.mnemonic == "REP" || def.mnemonic == "SEP" {
        // REP/SEP always take 1 byte immediate
        1
    } else if is_immediate_for_index(def.mnemonic) {
        def.addressing_mode.operand_size(is_8bit_accumulator, is_8bit_index)
    } else if is_immediate_for_accumulator(def.mnemonic) {
        def.addressing_mode.operand_size(is_8bit_accumulator, is_8bit_index)
    } else {
        def.addressing_mode.operand_size(true, true)
    };
    
    let total_size = 1 + operand_size;
    
    // Read operand bytes
    let mut bytes = vec![opcode];
    for i in 0..operand_size {
        if let Some(b) = memory.read_byte(pc + 1 + i as u32) {
            bytes.push(b);
        }
    }
    
    // Format operands
    let operands = if operand_size > 0 {
        let operand_bytes = &bytes[1..];
        def.addressing_mode.format_operands(
            operand_bytes,
            pc,
            is_8bit_accumulator,
            is_8bit_index,
        )
    } else {
        String::new()
    };
    
    Some(DisassembledInstruction {
        address,
        bytes,
        mnemonic: def.mnemonic.to_string(),
        operands,
        size: total_size,
        cycles: def.cycles,
        is_branch: def.is_branch,
        is_call: def.is_call,
        is_return: def.is_return,
    })
}

/// Check if instruction uses immediate addressing for accumulator
fn is_immediate_for_accumulator(mnemonic: &str) -> bool {
    matches!(
        mnemonic,
        "ADC" | "AND" | "CMP" | "EOR" | "LDA" | "ORA" | "SBC"
    )
}

/// Check if instruction uses immediate addressing for index registers
fn is_immediate_for_index(mnemonic: &str) -> bool {
    matches!(mnemonic, "CPX" | "CPY" | "LDX" | "LDY")
}

/// Get instruction definition for an opcode
fn get_instruction_def(opcode: u8) -> Option<InstructionDef> {
    Some(match opcode {
        // LDA - Load Accumulator
        0xA9 => InstructionDef { opcode, mnemonic: "LDA", addressing_mode: AddressingMode::Immediate, cycles: 2, is_branch: false, is_call: false, is_return: false },
        0xA5 => InstructionDef { opcode, mnemonic: "LDA", addressing_mode: AddressingMode::Direct, cycles: 3, is_branch: false, is_call: false, is_return: false },
        0xB5 => InstructionDef { opcode, mnemonic: "LDA", addressing_mode: AddressingMode::DirectX, cycles: 4, is_branch: false, is_call: false, is_return: false },
        0xAD => InstructionDef { opcode, mnemonic: "LDA", addressing_mode: AddressingMode::Absolute, cycles: 4, is_branch: false, is_call: false, is_return: false },
        0xBD => InstructionDef { opcode, mnemonic: "LDA", addressing_mode: AddressingMode::AbsoluteX, cycles: 4, is_branch: false, is_call: false, is_return: false },
        0xB9 => InstructionDef { opcode, mnemonic: "LDA", addressing_mode: AddressingMode::AbsoluteY, cycles: 4, is_branch: false, is_call: false, is_return: false },
        0xA1 => InstructionDef { opcode, mnemonic: "LDA", addressing_mode: AddressingMode::DirectIndirectX, cycles: 6, is_branch: false, is_call: false, is_return: false },
        0xB1 => InstructionDef { opcode, mnemonic: "LDA", addressing_mode: AddressingMode::DirectIndirectY, cycles: 5, is_branch: false, is_call: false, is_return: false },
        0xB2 => InstructionDef { opcode, mnemonic: "LDA", addressing_mode: AddressingMode::DirectIndirect, cycles: 5, is_branch: false, is_call: false, is_return: false },
        0xA7 => InstructionDef { opcode, mnemonic: "LDA", addressing_mode: AddressingMode::DirectIndirectLong, cycles: 6, is_branch: false, is_call: false, is_return: false },
        0xB7 => InstructionDef { opcode, mnemonic: "LDA", addressing_mode: AddressingMode::DirectIndirectLongY, cycles: 6, is_branch: false, is_call: false, is_return: false },
        0xAF => InstructionDef { opcode, mnemonic: "LDA", addressing_mode: AddressingMode::AbsoluteLong, cycles: 5, is_branch: false, is_call: false, is_return: false },
        0xBF => InstructionDef { opcode, mnemonic: "LDA", addressing_mode: AddressingMode::AbsoluteLongX, cycles: 5, is_branch: false, is_call: false, is_return: false },
        0xA3 => InstructionDef { opcode, mnemonic: "LDA", addressing_mode: AddressingMode::StackRelative, cycles: 4, is_branch: false, is_call: false, is_return: false },
        0xB3 => InstructionDef { opcode, mnemonic: "LDA", addressing_mode: AddressingMode::StackRelativeIndirectY, cycles: 7, is_branch: false, is_call: false, is_return: false },

        // STA - Store Accumulator
        0x85 => InstructionDef { opcode, mnemonic: "STA", addressing_mode: AddressingMode::Direct, cycles: 3, is_branch: false, is_call: false, is_return: false },
        0x95 => InstructionDef { opcode, mnemonic: "STA", addressing_mode: AddressingMode::DirectX, cycles: 4, is_branch: false, is_call: false, is_return: false },
        0x8D => InstructionDef { opcode, mnemonic: "STA", addressing_mode: AddressingMode::Absolute, cycles: 4, is_branch: false, is_call: false, is_return: false },
        0x9D => InstructionDef { opcode, mnemonic: "STA", addressing_mode: AddressingMode::AbsoluteX, cycles: 5, is_branch: false, is_call: false, is_return: false },
        0x99 => InstructionDef { opcode, mnemonic: "STA", addressing_mode: AddressingMode::AbsoluteY, cycles: 5, is_branch: false, is_call: false, is_return: false },
        0x81 => InstructionDef { opcode, mnemonic: "STA", addressing_mode: AddressingMode::DirectIndirectX, cycles: 6, is_branch: false, is_call: false, is_return: false },
        0x91 => InstructionDef { opcode, mnemonic: "STA", addressing_mode: AddressingMode::DirectIndirectY, cycles: 6, is_branch: false, is_call: false, is_return: false },
        0x92 => InstructionDef { opcode, mnemonic: "STA", addressing_mode: AddressingMode::DirectIndirect, cycles: 5, is_branch: false, is_call: false, is_return: false },
        0x87 => InstructionDef { opcode, mnemonic: "STA", addressing_mode: AddressingMode::DirectIndirectLong, cycles: 6, is_branch: false, is_call: false, is_return: false },
        0x97 => InstructionDef { opcode, mnemonic: "STA", addressing_mode: AddressingMode::DirectIndirectLongY, cycles: 6, is_branch: false, is_call: false, is_return: false },
        0x8F => InstructionDef { opcode, mnemonic: "STA", addressing_mode: AddressingMode::AbsoluteLong, cycles: 5, is_branch: false, is_call: false, is_return: false },
        0x9F => InstructionDef { opcode, mnemonic: "STA", addressing_mode: AddressingMode::AbsoluteLongX, cycles: 5, is_branch: false, is_call: false, is_return: false },
        0x83 => InstructionDef { opcode, mnemonic: "STA", addressing_mode: AddressingMode::StackRelative, cycles: 4, is_branch: false, is_call: false, is_return: false },
        0x93 => InstructionDef { opcode, mnemonic: "STA", addressing_mode: AddressingMode::StackRelativeIndirectY, cycles: 7, is_branch: false, is_call: false, is_return: false },

        // LDX - Load X
        0xA2 => InstructionDef { opcode, mnemonic: "LDX", addressing_mode: AddressingMode::Immediate16, cycles: 2, is_branch: false, is_call: false, is_return: false },
        0xA6 => InstructionDef { opcode, mnemonic: "LDX", addressing_mode: AddressingMode::Direct, cycles: 3, is_branch: false, is_call: false, is_return: false },
        0xB6 => InstructionDef { opcode, mnemonic: "LDX", addressing_mode: AddressingMode::DirectY, cycles: 4, is_branch: false, is_call: false, is_return: false },
        0xAE => InstructionDef { opcode, mnemonic: "LDX", addressing_mode: AddressingMode::Absolute, cycles: 4, is_branch: false, is_call: false, is_return: false },
        0xBE => InstructionDef { opcode, mnemonic: "LDX", addressing_mode: AddressingMode::AbsoluteY, cycles: 4, is_branch: false, is_call: false, is_return: false },

        // LDY - Load Y
        0xA0 => InstructionDef { opcode, mnemonic: "LDY", addressing_mode: AddressingMode::Immediate16, cycles: 2, is_branch: false, is_call: false, is_return: false },
        0xA4 => InstructionDef { opcode, mnemonic: "LDY", addressing_mode: AddressingMode::Direct, cycles: 3, is_branch: false, is_call: false, is_return: false },
        0xB4 => InstructionDef { opcode, mnemonic: "LDY", addressing_mode: AddressingMode::DirectX, cycles: 4, is_branch: false, is_call: false, is_return: false },
        0xAC => InstructionDef { opcode, mnemonic: "LDY", addressing_mode: AddressingMode::Absolute, cycles: 4, is_branch: false, is_call: false, is_return: false },
        0xBC => InstructionDef { opcode, mnemonic: "LDY", addressing_mode: AddressingMode::AbsoluteX, cycles: 4, is_branch: false, is_call: false, is_return: false },

        // STX - Store X
        0x86 => InstructionDef { opcode, mnemonic: "STX", addressing_mode: AddressingMode::Direct, cycles: 3, is_branch: false, is_call: false, is_return: false },
        0x96 => InstructionDef { opcode, mnemonic: "STX", addressing_mode: AddressingMode::DirectY, cycles: 4, is_branch: false, is_call: false, is_return: false },
        0x8E => InstructionDef { opcode, mnemonic: "STX", addressing_mode: AddressingMode::Absolute, cycles: 4, is_branch: false, is_call: false, is_return: false },

        // STY - Store Y
        0x84 => InstructionDef { opcode, mnemonic: "STY", addressing_mode: AddressingMode::Direct, cycles: 3, is_branch: false, is_call: false, is_return: false },
        0x94 => InstructionDef { opcode, mnemonic: "STY", addressing_mode: AddressingMode::DirectX, cycles: 4, is_branch: false, is_call: false, is_return: false },
        0x8C => InstructionDef { opcode, mnemonic: "STY", addressing_mode: AddressingMode::Absolute, cycles: 4, is_branch: false, is_call: false, is_return: false },

        // STZ - Store Zero
        0x64 => InstructionDef { opcode, mnemonic: "STZ", addressing_mode: AddressingMode::Direct, cycles: 3, is_branch: false, is_call: false, is_return: false },
        0x74 => InstructionDef { opcode, mnemonic: "STZ", addressing_mode: AddressingMode::DirectX, cycles: 4, is_branch: false, is_call: false, is_return: false },
        0x9C => InstructionDef { opcode, mnemonic: "STZ", addressing_mode: AddressingMode::Absolute, cycles: 4, is_branch: false, is_call: false, is_return: false },
        0x9E => InstructionDef { opcode, mnemonic: "STZ", addressing_mode: AddressingMode::AbsoluteX, cycles: 5, is_branch: false, is_call: false, is_return: false },

        // JMP - Jump
        0x4C => InstructionDef { opcode, mnemonic: "JMP", addressing_mode: AddressingMode::Absolute, cycles: 3, is_branch: true, is_call: false, is_return: false },
        0x6C => InstructionDef { opcode, mnemonic: "JMP", addressing_mode: AddressingMode::AbsoluteIndirect, cycles: 5, is_branch: true, is_call: false, is_return: false },
        0x7C => InstructionDef { opcode, mnemonic: "JMP", addressing_mode: AddressingMode::AbsoluteIndexedIndirect, cycles: 6, is_branch: true, is_call: false, is_return: false },
        0x5C => InstructionDef { opcode, mnemonic: "JML", addressing_mode: AddressingMode::AbsoluteLong, cycles: 4, is_branch: true, is_call: false, is_return: false },
        0xDC => InstructionDef { opcode, mnemonic: "JML", addressing_mode: AddressingMode::AbsoluteIndirectLong, cycles: 6, is_branch: true, is_call: false, is_return: false },

        // JSR - Jump Subroutine
        0x20 => InstructionDef { opcode, mnemonic: "JSR", addressing_mode: AddressingMode::Absolute, cycles: 6, is_branch: false, is_call: true, is_return: false },
        0xFC => InstructionDef { opcode, mnemonic: "JSR", addressing_mode: AddressingMode::AbsoluteIndexedIndirect, cycles: 8, is_branch: false, is_call: true, is_return: false },
        0x22 => InstructionDef { opcode, mnemonic: "JSL", addressing_mode: AddressingMode::AbsoluteLong, cycles: 8, is_branch: false, is_call: true, is_return: false },

        // RTS - Return from Subroutine
        0x60 => InstructionDef { opcode, mnemonic: "RTS", addressing_mode: AddressingMode::Implied, cycles: 6, is_branch: false, is_call: false, is_return: true },
        0x6B => InstructionDef { opcode, mnemonic: "RTL", addressing_mode: AddressingMode::Implied, cycles: 6, is_branch: false, is_call: false, is_return: true },
        0x40 => InstructionDef { opcode, mnemonic: "RTI", addressing_mode: AddressingMode::Implied, cycles: 6, is_branch: false, is_call: false, is_return: true },

        // Branch instructions
        0x10 => InstructionDef { opcode, mnemonic: "BPL", addressing_mode: AddressingMode::Relative, cycles: 2, is_branch: true, is_call: false, is_return: false },
        0x30 => InstructionDef { opcode, mnemonic: "BMI", addressing_mode: AddressingMode::Relative, cycles: 2, is_branch: true, is_call: false, is_return: false },
        0x50 => InstructionDef { opcode, mnemonic: "BVC", addressing_mode: AddressingMode::Relative, cycles: 2, is_branch: true, is_call: false, is_return: false },
        0x70 => InstructionDef { opcode, mnemonic: "BVS", addressing_mode: AddressingMode::Relative, cycles: 2, is_branch: true, is_call: false, is_return: false },
        0x90 => InstructionDef { opcode, mnemonic: "BCC", addressing_mode: AddressingMode::Relative, cycles: 2, is_branch: true, is_call: false, is_return: false },
        0xB0 => InstructionDef { opcode, mnemonic: "BCS", addressing_mode: AddressingMode::Relative, cycles: 2, is_branch: true, is_call: false, is_return: false },
        0xD0 => InstructionDef { opcode, mnemonic: "BNE", addressing_mode: AddressingMode::Relative, cycles: 2, is_branch: true, is_call: false, is_return: false },
        0xF0 => InstructionDef { opcode, mnemonic: "BEQ", addressing_mode: AddressingMode::Relative, cycles: 2, is_branch: true, is_call: false, is_return: false },
        0x80 => InstructionDef { opcode, mnemonic: "BRA", addressing_mode: AddressingMode::Relative, cycles: 3, is_branch: true, is_call: false, is_return: false },
        0x82 => InstructionDef { opcode, mnemonic: "BRL", addressing_mode: AddressingMode::RelativeLong, cycles: 4, is_branch: true, is_call: false, is_return: false },

        // BIT - Bit Test
        0x89 => InstructionDef { opcode, mnemonic: "BIT", addressing_mode: AddressingMode::Immediate, cycles: 2, is_branch: false, is_call: false, is_return: false },
        0x24 => InstructionDef { opcode, mnemonic: "BIT", addressing_mode: AddressingMode::Direct, cycles: 3, is_branch: false, is_call: false, is_return: false },
        0x34 => InstructionDef { opcode, mnemonic: "BIT", addressing_mode: AddressingMode::DirectX, cycles: 4, is_branch: false, is_call: false, is_return: false },
        0x2C => InstructionDef { opcode, mnemonic: "BIT", addressing_mode: AddressingMode::Absolute, cycles: 4, is_branch: false, is_call: false, is_return: false },
        0x3C => InstructionDef { opcode, mnemonic: "BIT", addressing_mode: AddressingMode::AbsoluteX, cycles: 4, is_branch: false, is_call: false, is_return: false },

        // Compare instructions
        0xC9 => InstructionDef { opcode, mnemonic: "CMP", addressing_mode: AddressingMode::Immediate, cycles: 2, is_branch: false, is_call: false, is_return: false },
        0xC5 => InstructionDef { opcode, mnemonic: "CMP", addressing_mode: AddressingMode::Direct, cycles: 3, is_branch: false, is_call: false, is_return: false },
        0xD5 => InstructionDef { opcode, mnemonic: "CMP", addressing_mode: AddressingMode::DirectX, cycles: 4, is_branch: false, is_call: false, is_return: false },
        0xCD => InstructionDef { opcode, mnemonic: "CMP", addressing_mode: AddressingMode::Absolute, cycles: 4, is_branch: false, is_call: false, is_return: false },
        0xDD => InstructionDef { opcode, mnemonic: "CMP", addressing_mode: AddressingMode::AbsoluteX, cycles: 4, is_branch: false, is_call: false, is_return: false },
        0xD9 => InstructionDef { opcode, mnemonic: "CMP", addressing_mode: AddressingMode::AbsoluteY, cycles: 4, is_branch: false, is_call: false, is_return: false },

        0xE0 => InstructionDef { opcode, mnemonic: "CPX", addressing_mode: AddressingMode::Immediate16, cycles: 2, is_branch: false, is_call: false, is_return: false },
        0xE4 => InstructionDef { opcode, mnemonic: "CPX", addressing_mode: AddressingMode::Direct, cycles: 3, is_branch: false, is_call: false, is_return: false },
        0xEC => InstructionDef { opcode, mnemonic: "CPX", addressing_mode: AddressingMode::Absolute, cycles: 4, is_branch: false, is_call: false, is_return: false },

        0xC0 => InstructionDef { opcode, mnemonic: "CPY", addressing_mode: AddressingMode::Immediate16, cycles: 2, is_branch: false, is_call: false, is_return: false },
        0xC4 => InstructionDef { opcode, mnemonic: "CPY", addressing_mode: AddressingMode::Direct, cycles: 3, is_branch: false, is_call: false, is_return: false },
        0xCC => InstructionDef { opcode, mnemonic: "CPY", addressing_mode: AddressingMode::Absolute, cycles: 4, is_branch: false, is_call: false, is_return: false },

        // Arithmetic
        0x69 => InstructionDef { opcode, mnemonic: "ADC", addressing_mode: AddressingMode::Immediate, cycles: 2, is_branch: false, is_call: false, is_return: false },
        0x65 => InstructionDef { opcode, mnemonic: "ADC", addressing_mode: AddressingMode::Direct, cycles: 3, is_branch: false, is_call: false, is_return: false },
        0x75 => InstructionDef { opcode, mnemonic: "ADC", addressing_mode: AddressingMode::DirectX, cycles: 4, is_branch: false, is_call: false, is_return: false },
        0x6D => InstructionDef { opcode, mnemonic: "ADC", addressing_mode: AddressingMode::Absolute, cycles: 4, is_branch: false, is_call: false, is_return: false },
        0x7D => InstructionDef { opcode, mnemonic: "ADC", addressing_mode: AddressingMode::AbsoluteX, cycles: 4, is_branch: false, is_call: false, is_return: false },
        0x79 => InstructionDef { opcode, mnemonic: "ADC", addressing_mode: AddressingMode::AbsoluteY, cycles: 4, is_branch: false, is_call: false, is_return: false },

        0xE9 => InstructionDef { opcode, mnemonic: "SBC", addressing_mode: AddressingMode::Immediate, cycles: 2, is_branch: false, is_call: false, is_return: false },
        0xE5 => InstructionDef { opcode, mnemonic: "SBC", addressing_mode: AddressingMode::Direct, cycles: 3, is_branch: false, is_call: false, is_return: false },
        0xF5 => InstructionDef { opcode, mnemonic: "SBC", addressing_mode: AddressingMode::DirectX, cycles: 4, is_branch: false, is_call: false, is_return: false },
        0xED => InstructionDef { opcode, mnemonic: "SBC", addressing_mode: AddressingMode::Absolute, cycles: 4, is_branch: false, is_call: false, is_return: false },
        0xFD => InstructionDef { opcode, mnemonic: "SBC", addressing_mode: AddressingMode::AbsoluteX, cycles: 4, is_branch: false, is_call: false, is_return: false },
        0xF9 => InstructionDef { opcode, mnemonic: "SBC", addressing_mode: AddressingMode::AbsoluteY, cycles: 4, is_branch: false, is_call: false, is_return: false },

        // Increment/Decrement
        0x1A => InstructionDef { opcode, mnemonic: "INC", addressing_mode: AddressingMode::Accumulator, cycles: 2, is_branch: false, is_call: false, is_return: false },
        0xE6 => InstructionDef { opcode, mnemonic: "INC", addressing_mode: AddressingMode::Direct, cycles: 5, is_branch: false, is_call: false, is_return: false },
        0xF6 => InstructionDef { opcode, mnemonic: "INC", addressing_mode: AddressingMode::DirectX, cycles: 6, is_branch: false, is_call: false, is_return: false },
        0xEE => InstructionDef { opcode, mnemonic: "INC", addressing_mode: AddressingMode::Absolute, cycles: 6, is_branch: false, is_call: false, is_return: false },
        0xFE => InstructionDef { opcode, mnemonic: "INC", addressing_mode: AddressingMode::AbsoluteX, cycles: 7, is_branch: false, is_call: false, is_return: false },

        0x3A => InstructionDef { opcode, mnemonic: "DEC", addressing_mode: AddressingMode::Accumulator, cycles: 2, is_branch: false, is_call: false, is_return: false },
        0xC6 => InstructionDef { opcode, mnemonic: "DEC", addressing_mode: AddressingMode::Direct, cycles: 5, is_branch: false, is_call: false, is_return: false },
        0xD6 => InstructionDef { opcode, mnemonic: "DEC", addressing_mode: AddressingMode::DirectX, cycles: 6, is_branch: false, is_call: false, is_return: false },
        0xCE => InstructionDef { opcode, mnemonic: "DEC", addressing_mode: AddressingMode::Absolute, cycles: 6, is_branch: false, is_call: false, is_return: false },
        0xDE => InstructionDef { opcode, mnemonic: "DEC", addressing_mode: AddressingMode::AbsoluteX, cycles: 7, is_branch: false, is_call: false, is_return: false },

        0xE8 => InstructionDef { opcode, mnemonic: "INX", addressing_mode: AddressingMode::Implied, cycles: 2, is_branch: false, is_call: false, is_return: false },
        0xCA => InstructionDef { opcode, mnemonic: "DEX", addressing_mode: AddressingMode::Implied, cycles: 2, is_branch: false, is_call: false, is_return: false },
        0xC8 => InstructionDef { opcode, mnemonic: "INY", addressing_mode: AddressingMode::Implied, cycles: 2, is_branch: false, is_call: false, is_return: false },
        0x88 => InstructionDef { opcode, mnemonic: "DEY", addressing_mode: AddressingMode::Implied, cycles: 2, is_branch: false, is_call: false, is_return: false },

        // Stack operations
        0x48 => InstructionDef { opcode, mnemonic: "PHA", addressing_mode: AddressingMode::Implied, cycles: 3, is_branch: false, is_call: false, is_return: false },
        0x68 => InstructionDef { opcode, mnemonic: "PLA", addressing_mode: AddressingMode::Implied, cycles: 4, is_branch: false, is_call: false, is_return: false },
        0xDA => InstructionDef { opcode, mnemonic: "PHX", addressing_mode: AddressingMode::Implied, cycles: 3, is_branch: false, is_call: false, is_return: false },
        0xFA => InstructionDef { opcode, mnemonic: "PLX", addressing_mode: AddressingMode::Implied, cycles: 4, is_branch: false, is_call: false, is_return: false },
        0x5A => InstructionDef { opcode, mnemonic: "PHY", addressing_mode: AddressingMode::Implied, cycles: 3, is_branch: false, is_call: false, is_return: false },
        0x7A => InstructionDef { opcode, mnemonic: "PLY", addressing_mode: AddressingMode::Implied, cycles: 4, is_branch: false, is_call: false, is_return: false },
        0x08 => InstructionDef { opcode, mnemonic: "PHP", addressing_mode: AddressingMode::Implied, cycles: 3, is_branch: false, is_call: false, is_return: false },
        0x28 => InstructionDef { opcode, mnemonic: "PLP", addressing_mode: AddressingMode::Implied, cycles: 4, is_branch: false, is_call: false, is_return: false },
        0xF4 => InstructionDef { opcode, mnemonic: "PEA", addressing_mode: AddressingMode::Absolute, cycles: 5, is_branch: false, is_call: false, is_return: false },
        0x62 => InstructionDef { opcode, mnemonic: "PER", addressing_mode: AddressingMode::RelativeLong, cycles: 6, is_branch: false, is_call: false, is_return: false },
        0xD4 => InstructionDef { opcode, mnemonic: "PEI", addressing_mode: AddressingMode::DirectIndirect, cycles: 6, is_branch: false, is_call: false, is_return: false },
        0x8B => InstructionDef { opcode, mnemonic: "PHB", addressing_mode: AddressingMode::Implied, cycles: 3, is_branch: false, is_call: false, is_return: false },
        0xAB => InstructionDef { opcode, mnemonic: "PLB", addressing_mode: AddressingMode::Implied, cycles: 4, is_branch: false, is_call: false, is_return: false },
        0x0B => InstructionDef { opcode, mnemonic: "PHD", addressing_mode: AddressingMode::Implied, cycles: 4, is_branch: false, is_call: false, is_return: false },
        0x2B => InstructionDef { opcode, mnemonic: "PLD", addressing_mode: AddressingMode::Implied, cycles: 5, is_branch: false, is_call: false, is_return: false },
        0x4B => InstructionDef { opcode, mnemonic: "PHK", addressing_mode: AddressingMode::Implied, cycles: 3, is_branch: false, is_call: false, is_return: false },

        // Register transfers
        0xAA => InstructionDef { opcode, mnemonic: "TAX", addressing_mode: AddressingMode::Implied, cycles: 2, is_branch: false, is_call: false, is_return: false },
        0x8A => InstructionDef { opcode, mnemonic: "TXA", addressing_mode: AddressingMode::Implied, cycles: 2, is_branch: false, is_call: false, is_return: false },
        0xA8 => InstructionDef { opcode, mnemonic: "TAY", addressing_mode: AddressingMode::Implied, cycles: 2, is_branch: false, is_call: false, is_return: false },
        0x98 => InstructionDef { opcode, mnemonic: "TYA", addressing_mode: AddressingMode::Implied, cycles: 2, is_branch: false, is_call: false, is_return: false },
        0x9B => InstructionDef { opcode, mnemonic: "TXY", addressing_mode: AddressingMode::Implied, cycles: 2, is_branch: false, is_call: false, is_return: false },
        0xBB => InstructionDef { opcode, mnemonic: "TYX", addressing_mode: AddressingMode::Implied, cycles: 2, is_branch: false, is_call: false, is_return: false },
        0xBA => InstructionDef { opcode, mnemonic: "TSX", addressing_mode: AddressingMode::Implied, cycles: 2, is_branch: false, is_call: false, is_return: false },
        0x9A => InstructionDef { opcode, mnemonic: "TXS", addressing_mode: AddressingMode::Implied, cycles: 2, is_branch: false, is_call: false, is_return: false },
        0x5B => InstructionDef { opcode, mnemonic: "TCD", addressing_mode: AddressingMode::Implied, cycles: 2, is_branch: false, is_call: false, is_return: false },
        0x7B => InstructionDef { opcode, mnemonic: "TDC", addressing_mode: AddressingMode::Implied, cycles: 2, is_branch: false, is_call: false, is_return: false },
        0x1B => InstructionDef { opcode, mnemonic: "TCS", addressing_mode: AddressingMode::Implied, cycles: 2, is_branch: false, is_call: false, is_return: false },
        0x3B => InstructionDef { opcode, mnemonic: "TSC", addressing_mode: AddressingMode::Implied, cycles: 2, is_branch: false, is_call: false, is_return: false },

        // Flag operations
        0x18 => InstructionDef { opcode, mnemonic: "CLC", addressing_mode: AddressingMode::Implied, cycles: 2, is_branch: false, is_call: false, is_return: false },
        0x38 => InstructionDef { opcode, mnemonic: "SEC", addressing_mode: AddressingMode::Implied, cycles: 2, is_branch: false, is_call: false, is_return: false },
        0x58 => InstructionDef { opcode, mnemonic: "CLI", addressing_mode: AddressingMode::Implied, cycles: 2, is_branch: false, is_call: false, is_return: false },
        0x78 => InstructionDef { opcode, mnemonic: "SEI", addressing_mode: AddressingMode::Implied, cycles: 2, is_branch: false, is_call: false, is_return: false },
        0xD8 => InstructionDef { opcode, mnemonic: "CLD", addressing_mode: AddressingMode::Implied, cycles: 2, is_branch: false, is_call: false, is_return: false },
        0xF8 => InstructionDef { opcode, mnemonic: "SED", addressing_mode: AddressingMode::Implied, cycles: 2, is_branch: false, is_call: false, is_return: false },
        0xB8 => InstructionDef { opcode, mnemonic: "CLV", addressing_mode: AddressingMode::Implied, cycles: 2, is_branch: false, is_call: false, is_return: false },
        0xC2 => InstructionDef { opcode, mnemonic: "REP", addressing_mode: AddressingMode::Immediate, cycles: 3, is_branch: false, is_call: false, is_return: false },
        0xE2 => InstructionDef { opcode, mnemonic: "SEP", addressing_mode: AddressingMode::Immediate, cycles: 3, is_branch: false, is_call: false, is_return: false },
        0xFB => InstructionDef { opcode, mnemonic: "XCE", addressing_mode: AddressingMode::Implied, cycles: 2, is_branch: false, is_call: false, is_return: false },

        // Logical operations
        0x29 => InstructionDef { opcode, mnemonic: "AND", addressing_mode: AddressingMode::Immediate, cycles: 2, is_branch: false, is_call: false, is_return: false },
        0x25 => InstructionDef { opcode, mnemonic: "AND", addressing_mode: AddressingMode::Direct, cycles: 3, is_branch: false, is_call: false, is_return: false },
        0x35 => InstructionDef { opcode, mnemonic: "AND", addressing_mode: AddressingMode::DirectX, cycles: 4, is_branch: false, is_call: false, is_return: false },
        0x2D => InstructionDef { opcode, mnemonic: "AND", addressing_mode: AddressingMode::Absolute, cycles: 4, is_branch: false, is_call: false, is_return: false },
        0x3D => InstructionDef { opcode, mnemonic: "AND", addressing_mode: AddressingMode::AbsoluteX, cycles: 4, is_branch: false, is_call: false, is_return: false },
        0x39 => InstructionDef { opcode, mnemonic: "AND", addressing_mode: AddressingMode::AbsoluteY, cycles: 4, is_branch: false, is_call: false, is_return: false },

        0x09 => InstructionDef { opcode, mnemonic: "ORA", addressing_mode: AddressingMode::Immediate, cycles: 2, is_branch: false, is_call: false, is_return: false },
        0x05 => InstructionDef { opcode, mnemonic: "ORA", addressing_mode: AddressingMode::Direct, cycles: 3, is_branch: false, is_call: false, is_return: false },
        0x15 => InstructionDef { opcode, mnemonic: "ORA", addressing_mode: AddressingMode::DirectX, cycles: 4, is_branch: false, is_call: false, is_return: false },
        0x0D => InstructionDef { opcode, mnemonic: "ORA", addressing_mode: AddressingMode::Absolute, cycles: 4, is_branch: false, is_call: false, is_return: false },
        0x1D => InstructionDef { opcode, mnemonic: "ORA", addressing_mode: AddressingMode::AbsoluteX, cycles: 4, is_branch: false, is_call: false, is_return: false },
        0x19 => InstructionDef { opcode, mnemonic: "ORA", addressing_mode: AddressingMode::AbsoluteY, cycles: 4, is_branch: false, is_call: false, is_return: false },

        0x49 => InstructionDef { opcode, mnemonic: "EOR", addressing_mode: AddressingMode::Immediate, cycles: 2, is_branch: false, is_call: false, is_return: false },
        0x45 => InstructionDef { opcode, mnemonic: "EOR", addressing_mode: AddressingMode::Direct, cycles: 3, is_branch: false, is_call: false, is_return: false },
        0x55 => InstructionDef { opcode, mnemonic: "EOR", addressing_mode: AddressingMode::DirectX, cycles: 4, is_branch: false, is_call: false, is_return: false },
        0x4D => InstructionDef { opcode, mnemonic: "EOR", addressing_mode: AddressingMode::Absolute, cycles: 4, is_branch: false, is_call: false, is_return: false },
        0x5D => InstructionDef { opcode, mnemonic: "EOR", addressing_mode: AddressingMode::AbsoluteX, cycles: 4, is_branch: false, is_call: false, is_return: false },
        0x59 => InstructionDef { opcode, mnemonic: "EOR", addressing_mode: AddressingMode::AbsoluteY, cycles: 4, is_branch: false, is_call: false, is_return: false },

        // Shift/Rotate
        0x0A => InstructionDef { opcode, mnemonic: "ASL", addressing_mode: AddressingMode::Accumulator, cycles: 2, is_branch: false, is_call: false, is_return: false },
        0x06 => InstructionDef { opcode, mnemonic: "ASL", addressing_mode: AddressingMode::Direct, cycles: 5, is_branch: false, is_call: false, is_return: false },
        0x16 => InstructionDef { opcode, mnemonic: "ASL", addressing_mode: AddressingMode::DirectX, cycles: 6, is_branch: false, is_call: false, is_return: false },
        0x0E => InstructionDef { opcode, mnemonic: "ASL", addressing_mode: AddressingMode::Absolute, cycles: 6, is_branch: false, is_call: false, is_return: false },
        0x1E => InstructionDef { opcode, mnemonic: "ASL", addressing_mode: AddressingMode::AbsoluteX, cycles: 7, is_branch: false, is_call: false, is_return: false },

        0x4A => InstructionDef { opcode, mnemonic: "LSR", addressing_mode: AddressingMode::Accumulator, cycles: 2, is_branch: false, is_call: false, is_return: false },
        0x46 => InstructionDef { opcode, mnemonic: "LSR", addressing_mode: AddressingMode::Direct, cycles: 5, is_branch: false, is_call: false, is_return: false },
        0x56 => InstructionDef { opcode, mnemonic: "LSR", addressing_mode: AddressingMode::DirectX, cycles: 6, is_branch: false, is_call: false, is_return: false },
        0x4E => InstructionDef { opcode, mnemonic: "LSR", addressing_mode: AddressingMode::Absolute, cycles: 6, is_branch: false, is_call: false, is_return: false },
        0x5E => InstructionDef { opcode, mnemonic: "LSR", addressing_mode: AddressingMode::AbsoluteX, cycles: 7, is_branch: false, is_call: false, is_return: false },

        0x2A => InstructionDef { opcode, mnemonic: "ROL", addressing_mode: AddressingMode::Accumulator, cycles: 2, is_branch: false, is_call: false, is_return: false },
        0x26 => InstructionDef { opcode, mnemonic: "ROL", addressing_mode: AddressingMode::Direct, cycles: 5, is_branch: false, is_call: false, is_return: false },
        0x36 => InstructionDef { opcode, mnemonic: "ROL", addressing_mode: AddressingMode::DirectX, cycles: 6, is_branch: false, is_call: false, is_return: false },
        0x2E => InstructionDef { opcode, mnemonic: "ROL", addressing_mode: AddressingMode::Absolute, cycles: 6, is_branch: false, is_call: false, is_return: false },
        0x3E => InstructionDef { opcode, mnemonic: "ROL", addressing_mode: AddressingMode::AbsoluteX, cycles: 7, is_branch: false, is_call: false, is_return: false },

        0x6A => InstructionDef { opcode, mnemonic: "ROR", addressing_mode: AddressingMode::Accumulator, cycles: 2, is_branch: false, is_call: false, is_return: false },
        0x66 => InstructionDef { opcode, mnemonic: "ROR", addressing_mode: AddressingMode::Direct, cycles: 5, is_branch: false, is_call: false, is_return: false },
        0x76 => InstructionDef { opcode, mnemonic: "ROR", addressing_mode: AddressingMode::DirectX, cycles: 6, is_branch: false, is_call: false, is_return: false },
        0x6E => InstructionDef { opcode, mnemonic: "ROR", addressing_mode: AddressingMode::Absolute, cycles: 6, is_branch: false, is_call: false, is_return: false },
        0x7E => InstructionDef { opcode, mnemonic: "ROR", addressing_mode: AddressingMode::AbsoluteX, cycles: 7, is_branch: false, is_call: false, is_return: false },

        // Block Move
        0x54 => InstructionDef { opcode, mnemonic: "MVN", addressing_mode: AddressingMode::BlockMove, cycles: 7, is_branch: false, is_call: false, is_return: false },
        0x44 => InstructionDef { opcode, mnemonic: "MVP", addressing_mode: AddressingMode::BlockMove, cycles: 7, is_branch: false, is_call: false, is_return: false },

        // Misc
        0x00 => InstructionDef { opcode, mnemonic: "BRK", addressing_mode: AddressingMode::Implied, cycles: 7, is_branch: false, is_call: false, is_return: false },
        0x02 => InstructionDef { opcode, mnemonic: "COP", addressing_mode: AddressingMode::Immediate, cycles: 7, is_branch: false, is_call: false, is_return: false },
        0xEB => InstructionDef { opcode, mnemonic: "XBA", addressing_mode: AddressingMode::Implied, cycles: 3, is_branch: false, is_call: false, is_return: false },
        0xEA => InstructionDef { opcode, mnemonic: "NOP", addressing_mode: AddressingMode::Implied, cycles: 2, is_branch: false, is_call: false, is_return: false },
        0xCB => InstructionDef { opcode, mnemonic: "WAI", addressing_mode: AddressingMode::Implied, cycles: 3, is_branch: false, is_call: false, is_return: false },
        0xDB => InstructionDef { opcode, mnemonic: "STP", addressing_mode: AddressingMode::Implied, cycles: 3, is_branch: false, is_call: false, is_return: false },
        0x42 => InstructionDef { opcode, mnemonic: "WDM", addressing_mode: AddressingMode::Immediate, cycles: 2, is_branch: false, is_call: false, is_return: false },

        // WAI (Wait for Interrupt)
        0xEF => InstructionDef { opcode, mnemonic: "WAI", addressing_mode: AddressingMode::Implied, cycles: 3, is_branch: false, is_call: false, is_return: false },
        // STP (Stop)
        0xFF => InstructionDef { opcode, mnemonic: "STP", addressing_mode: AddressingMode::Implied, cycles: 3, is_branch: false, is_call: false, is_return: false },

        // Unknown opcode
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestMemory {
        data: Vec<u8>,
    }

    impl TestMemory {
        fn new(data: Vec<u8>) -> Self {
            Self { data }
        }
    }

    impl MemoryAccess for TestMemory {
        fn read_byte(&self, addr: u32) -> Option<u8> {
            self.data.get(addr as usize).copied()
        }
        fn write_byte(&mut self, _addr: u32, _value: u8) {}
        fn write_bytes(&mut self, _addr: u32, _values: &[u8]) {}
    }

    #[test]
    fn test_decode_lda_immediate() {
        let memory = TestMemory::new(vec![0xA9, 0x42]);
        let addr = SnesAddress::new(0x00, 0x8000);
        
        let result = decode_instruction_at(addr, &memory, true, true).unwrap();
        
        assert_eq!(result.mnemonic, "LDA");
        assert_eq!(result.operands, "#$42");
        assert_eq!(result.size, 2);
        assert_eq!(result.bytes, vec![0xA9, 0x42]);
    }

    #[test]
    fn test_decode_lda_immediate_16bit() {
        let memory = TestMemory::new(vec![0xA9, 0x34, 0x12]);
        let addr = SnesAddress::new(0x00, 0x8000);
        
        let result = decode_instruction_at(addr, &memory, false, true).unwrap();
        
        assert_eq!(result.mnemonic, "LDA");
        assert_eq!(result.operands, "#$1234");
        assert_eq!(result.size, 3);
    }

    #[test]
    fn test_decode_jmp_absolute() {
        let memory = TestMemory::new(vec![0x4C, 0x00, 0x90]);
        let addr = SnesAddress::new(0x00, 0x8000);
        
        let result = decode_instruction_at(addr, &memory, true, true).unwrap();
        
        assert_eq!(result.mnemonic, "JMP");
        assert_eq!(result.operands, "$9000");
        assert!(result.is_branch);
    }

    #[test]
    fn test_decode_jsr() {
        let memory = TestMemory::new(vec![0x20, 0x00, 0x90]);
        let addr = SnesAddress::new(0x00, 0x8000);
        
        let result = decode_instruction_at(addr, &memory, true, true).unwrap();
        
        assert_eq!(result.mnemonic, "JSR");
        assert!(result.is_call);
        assert!(!result.is_return);
    }

    #[test]
    fn test_decode_rts() {
        let memory = TestMemory::new(vec![0x60]);
        let addr = SnesAddress::new(0x00, 0x8000);
        
        let result = decode_instruction_at(addr, &memory, true, true).unwrap();
        
        assert_eq!(result.mnemonic, "RTS");
        assert!(result.is_return);
        assert!(!result.is_call);
    }

    #[test]
    fn test_decode_branch() {
        let memory = TestMemory::new(vec![0xD0, 0x10]);  // BNE $+10
        let addr = SnesAddress::new(0x00, 0x8000);
        
        let result = decode_instruction_at(addr, &memory, true, true).unwrap();
        
        assert_eq!(result.mnemonic, "BNE");
        assert_eq!(result.operands, "$8012");  // PC + 2 + 0x10
        assert!(result.is_branch);
    }

    #[test]
    fn test_decode_nop() {
        let memory = TestMemory::new(vec![0xEA]);
        let addr = SnesAddress::new(0x00, 0x8000);
        
        let result = decode_instruction_at(addr, &memory, true, true).unwrap();
        
        assert_eq!(result.mnemonic, "NOP");
        assert_eq!(result.size, 1);
    }
}
