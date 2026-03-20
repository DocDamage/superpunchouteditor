//! Core types for the debugger

use serde::{Deserialize, Serialize};

/// 24-bit SNES address (bank: 8 bits, address: 16 bits)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SnesAddress {
    pub bank: u8,
    pub addr: u16,
}

impl SnesAddress {
    pub fn new(bank: u8, addr: u16) -> Self {
        Self { bank, addr }
    }

    pub fn to_pc(&self) -> u32 {
        // LoROM mapping
        ((self.bank as u32) << 15) | ((self.addr as u32) & 0x7FFF)
    }

    pub fn from_pc(pc: u32) -> Self {
        // LoROM unmapping
        let bank = ((pc >> 15) & 0xFF) as u8;
        let addr = (pc & 0x7FFF) as u16;
        Self { bank, addr: addr | 0x8000 }
    }
}

impl std::fmt::Display for SnesAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "${:02X}:{:04X}", self.bank, self.addr)
    }
}

/// 65816 CPU registers
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct RegisterState {
    /// Accumulator (8 or 16 bits depending on M flag)
    pub a: u16,
    /// X index register (8 or 16 bits depending on X flag)
    pub x: u16,
    /// Y index register (8 or 16 bits depending on X flag)
    pub y: u16,
    /// Stack pointer
    pub sp: u16,
    /// Direct page register
    pub dp: u16,
    /// Data bank register
    pub db: u8,
    /// Program bank register
    pub pb: u8,
    /// Program counter
    pub pc: u16,
    /// Processor status register
    pub p: StatusRegister,
}

impl RegisterState {
    pub fn full_pc(&self) -> SnesAddress {
        SnesAddress::new(self.pb, self.pc)
    }

    pub fn is_8bit_accumulator(&self) -> bool {
        self.p.contains(StatusRegister::M)
    }

    pub fn is_8bit_index(&self) -> bool {
        self.p.contains(StatusRegister::X)
    }
}

/// Processor status register flags
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct StatusRegister(pub u8);

impl StatusRegister {
    pub const N: Self = Self(0x80); // Negative
    pub const V: Self = Self(0x40); // Overflow
    pub const M: Self = Self(0x20); // Memory/Accumulator size (0=16-bit, 1=8-bit)
    pub const X: Self = Self(0x10); // Index register size (0=16-bit, 1=8-bit)
    pub const D: Self = Self(0x08); // Decimal mode
    pub const I: Self = Self(0x04); // IRQ disable
    pub const Z: Self = Self(0x02); // Zero
    pub const C: Self = Self(0x01); // Carry

    pub fn contains(&self, flag: Self) -> bool {
        (self.0 & flag.0) != 0
    }

    pub fn insert(&mut self, flag: Self) {
        self.0 |= flag.0;
    }

    pub fn remove(&mut self, flag: Self) {
        self.0 &= !flag.0;
    }
}

/// Breakpoint definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Breakpoint {
    pub id: Option<u64>,
    pub address: SnesAddress,
    pub condition: BreakCondition,
    pub enabled: bool,
    pub hit_count: u64,
    pub hit_limit: Option<u64>,
    pub description: Option<String>,
}

impl Breakpoint {
    pub fn simple(addr: SnesAddress) -> Self {
        Self {
            id: None,
            address: addr,
            condition: BreakCondition::Always,
            enabled: true,
            hit_count: 0,
            hit_limit: None,
            description: None,
        }
    }

    pub fn with_condition(addr: SnesAddress, condition: BreakCondition) -> Self {
        Self {
            id: None,
            address: addr,
            condition,
            enabled: true,
            hit_count: 0,
            hit_limit: None,
            description: None,
        }
    }
}

/// Breakpoint condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BreakCondition {
    /// Always break
    Always,
    /// Break on execute
    OnExecute,
    /// Break on memory read
    OnRead { min_addr: u32, max_addr: u32 },
    /// Break on memory write
    OnWrite { min_addr: u32, max_addr: u32 },
    /// Break on specific register value
    RegisterEquals { reg: Register, value: u16 },
    /// Break when expression is true
    Expression(String),
}

/// Register identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Register {
    A,
    X,
    Y,
    SP,
    DP,
    DB,
    PB,
    PC,
}

impl std::fmt::Display for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Register::A => write!(f, "A"),
            Register::X => write!(f, "X"),
            Register::Y => write!(f, "Y"),
            Register::SP => write!(f, "SP"),
            Register::DP => write!(f, "DP"),
            Register::DB => write!(f, "DB"),
            Register::PB => write!(f, "PB"),
            Register::PC => write!(f, "PC"),
        }
    }
}

/// Memory region information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRegion {
    pub start: u32,
    pub end: u32,
    pub name: String,
    pub region_type: MemoryRegionType,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum MemoryRegionType {
    Rom,
    Ram,
    Sram,
    Register,
    Unknown,
}

/// Disassembled instruction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisassembledInstruction {
    pub address: SnesAddress,
    pub bytes: Vec<u8>,
    pub mnemonic: String,
    pub operands: String,
    pub size: u8,
    pub cycles: u8,
    pub is_branch: bool,
    pub is_call: bool,
    pub is_return: bool,
}

impl std::fmt::Display for DisassembledInstruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let bytes_str = self
            .bytes
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(" ");
        write!(
            f,
            "{:06X}  {:<12}  {} {}",
            self.address.to_pc(),
            bytes_str,
            self.mnemonic,
            self.operands
        )
    }
}
