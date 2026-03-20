//! # Cartridge Dumper Module
//!
//! Provides interfaces for dumping cartridges and detecting reproduction carts.
//! Essential for ROM preservation and verification.
//!
//! ## Features
//!
//! - **ROM dumping** - Read cartridge ROM data
//! - **SRAM dumping** - Read/write cartridge save data
//! - **Chip type detection** - Identify ROM/RAM chips
//! - **Copier detection** - Detect reproduction/bootleg cartridges
//!
//! ## Example Usage
//!
//! ```rust,no_run
//! use console_dev_core::dumper::{CartridgeDumper, DumperError};
//!
//! // Dump a cartridge
//! let dumper = MyDumper::new();
//! let rom_data = dumper.read_rom()?;
//! ```

use rom_core::{RomData, RomHeader};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use thiserror::Error;

/// Errors that can occur during dumping operations
#[derive(Debug, Error)]
pub enum DumperError {
    /// Dumper not connected
    #[error("Dumper not connected: {0}")]
    NotConnected(String),

    /// Read failed
    #[error("Read failed at address ${address:06X}: {message}")]
    ReadFailed {
        /// Address that failed
        address: u32,
        /// Error message
        message: String,
    },

    /// Write failed (for SRAM)
    #[error("Write failed at address ${address:06X}: {message}")]
    WriteFailed {
        /// Address that failed
        address: u32,
        /// Error message
        message: String,
    },

    /// Chip detection failed
    #[error("Chip detection failed: {0}")]
    ChipDetectionFailed(String),

    /// Cartridge not detected
    #[error("No cartridge detected")]
    NoCartridge,

    /// Unsupported cartridge type
    #[error("Unsupported cartridge type: {0}")]
    UnsupportedType(String),

    /// Verification failed
    #[error("Verification failed at address ${address:06X}: expected {expected:02X}, got {actual:02X}")]
    VerificationFailed {
        /// Address that failed verification
        address: u32,
        /// Expected value
        expected: u8,
        /// Actual value
        actual: u8,
    },

    /// Timeout
    #[error("Operation timed out")]
    Timeout,

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type for dumper operations
pub type Result<T> = std::result::Result<T, DumperError>;

/// Trait for cartridge dumpers
///
/// Implement this trait for any device that can read cartridge data,
/// such as the Retrode, Sanni's Cart Reader, or other dumper hardware.
pub trait CartridgeDumper: Send + fmt::Debug {
    /// Get the dumper name
    fn name(&self) -> &str;

    /// Get the dumper version/firmware
    fn version(&self) -> Option<&str>;

    /// Check if the dumper is connected
    fn is_connected(&self) -> bool;

    /// Connect to the dumper
    fn connect(&mut self) -> Result<()>;

    /// Disconnect from the dumper
    fn disconnect(&mut self);

    /// Check if a cartridge is inserted
    fn has_cartridge(&self) -> bool;

    /// Read the ROM from the cartridge
    ///
    /// # Arguments
    ///
    /// * `progress_callback` - Optional callback for progress updates
    ///
    /// # Errors
    ///
    /// Returns an error if the read fails
    fn read_rom(&mut self, progress_callback: Option<&dyn Fn(usize, usize)>) -> Result<RomData>;

    /// Read a specific range of ROM data
    ///
    /// # Arguments
    ///
    /// * `address` - Start address
    /// * `size` - Number of bytes to read
    ///
    /// # Errors
    ///
    /// Returns an error if the read fails
    fn read_rom_range(&mut self, address: u32, size: usize) -> Result<Vec<u8>>;

    /// Read the SRAM from the cartridge
    ///
    /// # Arguments
    ///
    /// * `size` - Expected SRAM size (or 0 to auto-detect)
    /// * `progress_callback` - Optional callback for progress updates
    ///
    /// # Errors
    ///
    /// Returns an error if the read fails
    fn read_sram(
        &mut self,
        size: usize,
        progress_callback: Option<&dyn Fn(usize, usize)>,
    ) -> Result<Vec<u8>>;

    /// Write SRAM to the cartridge
    ///
    /// # Arguments
    ///
    /// * `data` - SRAM data to write
    ///
    /// # Errors
    ///
    /// Returns an error if the write fails
    fn write_sram(&mut self, data: &[u8]) -> Result<()>;

    /// Detect the chip types in the cartridge
    ///
    /// Attempts to identify the ROM and RAM chips by their
    /// response patterns and timing characteristics.
    ///
    /// # Errors
    ///
    /// Returns an error if detection fails
    fn detect_chip_type(&mut self) -> Result<ChipInfo>;

    /// Detect if this is a reproduction/copier cartridge
    ///
    /// Analyzes various characteristics to determine if the
    /// cartridge is an original or a reproduction/bootleg.
    ///
    /// # Errors
    ///
    /// Returns an error if detection fails
    fn detect_copier(&mut self) -> Result<CopierDetectionResult>;

    /// Get cartridge information without full dump
    fn get_cartridge_info(&mut self) -> Result<CartridgeInfo>;

    /// Verify ROM integrity by re-reading and comparing
    ///
    /// # Arguments
    ///
    /// * `original_data` - The original ROM data to verify against
    ///
    /// # Errors
    ///
    /// Returns an error if verification fails
    fn verify_rom(&mut self, original_data: &RomData) -> Result<()>;
}

/// Information about detected chips
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChipInfo {
    /// ROM chip information
    pub rom_chips: Vec<RomChipInfo>,
    /// RAM chip information
    pub ram_chips: Vec<RamChipInfo>,
    /// Additional chips detected
    pub extra_chips: Vec<String>,
    /// Detection confidence (0.0 - 1.0)
    pub confidence: f64,
}

/// ROM chip information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RomChipInfo {
    /// Chip manufacturer
    pub manufacturer: String,
    /// Chip model/part number
    pub part_number: String,
    /// Chip size in bytes
    pub size: usize,
    /// Memory type
    pub memory_type: MemoryType,
    /// Access time in nanoseconds
    pub access_time_ns: Option<u32>,
    /// Pin count
    pub pin_count: Option<u8>,
    /// Package type
    pub package: Option<String>,
}

/// RAM chip information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RamChipInfo {
    /// Chip manufacturer
    pub manufacturer: String,
    /// Chip model/part number
    pub part_number: String,
    /// Chip size in bytes
    pub size: usize,
    /// RAM type
    pub ram_type: RamType,
    /// Battery backed
    pub battery_backed: bool,
    /// Access time in nanoseconds
    pub access_time_ns: Option<u32>,
}

/// Memory type for ROM chips
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemoryType {
    /// Mask ROM (factory programmed)
    MaskRom,
    /// EPROM (erasable)
    Eprom,
    /// EEPROM (electrically erasable)
    Eeprom,
    /// Flash memory
    Flash,
    /// OTP ROM (one-time programmable)
    OtpRom,
    /// Unknown type
    Unknown,
}

impl fmt::Display for MemoryType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MemoryType::MaskRom => write!(f, "Mask ROM"),
            MemoryType::Eprom => write!(f, "EPROM"),
            MemoryType::Eeprom => write!(f, "EEPROM"),
            MemoryType::Flash => write!(f, "Flash"),
            MemoryType::OtpRom => write!(f, "OTP ROM"),
            MemoryType::Unknown => write!(f, "Unknown"),
        }
    }
}

/// RAM type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RamType {
    /// Static RAM
    Sram,
    /// Dynamic RAM
    Dram,
    /// Pseudo-static RAM
    Psram,
    /// Ferroelectric RAM
    FeRam,
    /// Unknown type
    Unknown,
}

impl fmt::Display for RamType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RamType::Sram => write!(f, "SRAM"),
            RamType::Dram => write!(f, "DRAM"),
            RamType::Psram => write!(f, "PSRAM"),
            RamType::FeRam => write!(f, "FeRAM"),
            RamType::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Result of copier/reproduction detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopierDetectionResult {
    /// Whether this is likely a reproduction
    pub is_reproduction: bool,
    /// Confidence level (0.0 - 1.0)
    pub confidence: f64,
    /// List of indicators found
    pub indicators: Vec<CopierIndicator>,
    /// Detected copier type (if known)
    pub copier_type: Option<String>,
    /// Detailed analysis
    pub analysis: String,
}

impl CopierDetectionResult {
    /// Create a result indicating an original cartridge
    pub fn original() -> Self {
        Self {
            is_reproduction: false,
            confidence: 1.0,
            indicators: vec![],
            copier_type: None,
            analysis: "No reproduction indicators detected".to_string(),
        }
    }

    /// Create a result indicating a reproduction
    pub fn reproduction(indicators: Vec<CopierIndicator>) -> Self {
        let confidence = indicators.iter().map(|i| i.confidence()).sum::<f64>() 
            / indicators.len().max(1) as f64;
        
        Self {
            is_reproduction: true,
            confidence: confidence.min(1.0),
            indicators,
            copier_type: None,
            analysis: String::new(),
        }
    }
}

/// Indicators of a reproduction/copier cartridge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CopierIndicator {
    /// Wrong chip type for the game's era
    WrongChipType {
        /// Expected type
        expected: String,
        /// Found type
        found: String,
    },
    /// Modern flash chips in old game
    ModernFlashChip {
        /// Chip identification
        chip_id: String,
    },
    /// Incorrect PCB layout
    IncorrectPcbLayout {
        /// Description
        description: String,
    },
    /// Header doesn't match ROM content
    HeaderMismatch {
        /// Header checksum
        header_checksum: u16,
        /// Calculated checksum
        calculated_checksum: u16,
    },
    /// Suspicious manufacturing marks
    SuspiciousMarks {
        /// Description
        description: String,
    },
    /// Wrong voltage levels
    WrongVoltage {
        /// Expected
        expected: String,
        /// Measured
        measured: String,
    },
    /// Unusual timing characteristics
    UnusualTiming {
        /// Description
        description: String,
    },
}

impl CopierIndicator {
    /// Get the confidence level for this indicator
    pub fn confidence(&self) -> f64 {
        match self {
            CopierIndicator::WrongChipType { .. } => 0.9,
            CopierIndicator::ModernFlashChip { .. } => 0.95,
            CopierIndicator::IncorrectPcbLayout { .. } => 0.7,
            CopierIndicator::HeaderMismatch { .. } => 0.8,
            CopierIndicator::SuspiciousMarks { .. } => 0.6,
            CopierIndicator::WrongVoltage { .. } => 0.85,
            CopierIndicator::UnusualTiming { .. } => 0.5,
        }
    }

    /// Get a description of this indicator
    pub fn description(&self) -> String {
        match self {
            CopierIndicator::WrongChipType { expected, found } => {
                format!("Wrong chip type: expected {}, found {}", expected, found)
            }
            CopierIndicator::ModernFlashChip { chip_id } => {
                format!("Modern flash chip detected: {}", chip_id)
            }
            CopierIndicator::IncorrectPcbLayout { description } => {
                format!("Incorrect PCB layout: {}", description)
            }
            CopierIndicator::HeaderMismatch { header_checksum, calculated_checksum } => {
                format!("Header checksum mismatch: header=${:04X}, calc=${:04X}",
                    header_checksum, calculated_checksum)
            }
            CopierIndicator::SuspiciousMarks { description } => {
                format!("Suspicious marks: {}", description)
            }
            CopierIndicator::WrongVoltage { expected, measured } => {
                format!("Wrong voltage: expected {}, measured {}", expected, measured)
            }
            CopierIndicator::UnusualTiming { description } => {
                format!("Unusual timing: {}", description)
            }
        }
    }
}

/// Information about a cartridge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CartridgeInfo {
    /// ROM header (if readable)
    pub header: Option<RomHeader>,
    /// Detected ROM size
    pub rom_size: usize,
    /// Detected SRAM size
    pub sram_size: usize,
    /// Mapper/chipset type
    pub mapper_type: String,
    /// Region
    pub region: String,
    /// Has battery
    pub has_battery: bool,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl CartridgeInfo {
    /// Create new cartridge info
    pub fn new() -> Self {
        Self {
            header: None,
            rom_size: 0,
            sram_size: 0,
            mapper_type: String::new(),
            region: String::new(),
            has_battery: false,
            metadata: HashMap::new(),
        }
    }

    /// Set the header
    pub fn with_header(mut self, header: RomHeader) -> Self {
        self.header = Some(header);
        self
    }

    /// Set ROM size
    pub fn with_rom_size(mut self, size: usize) -> Self {
        self.rom_size = size;
        self
    }

    /// Set SRAM size
    pub fn with_sram_size(mut self, size: usize) -> Self {
        self.sram_size = size;
        self
    }

    /// Set mapper type
    pub fn with_mapper(mut self, mapper: impl Into<String>) -> Self {
        self.mapper_type = mapper.into();
        self
    }

    /// Set region
    pub fn with_region(mut self, region: impl Into<String>) -> Self {
        self.region = region.into();
        self
    }

    /// Set battery status
    pub fn with_battery(mut self, has_battery: bool) -> Self {
        self.has_battery = has_battery;
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

impl Default for CartridgeInfo {
    fn default() -> Self {
        Self::new()
    }
}

/// Utility functions for dumper operations
pub mod utils {
    use super::*;

    /// Calculate the checksum of ROM data
    pub fn calculate_checksum(data: &[u8]) -> u16 {
        let sum: u32 = data.iter().map(|&b| b as u32).sum();
        (sum & 0xFFFF) as u16
    }

    /// Detect ROM size by checking mirrored addresses
    pub fn detect_rom_size(dumper: &mut dyn CartridgeDumper) -> Result<usize> {
        // Read header at different addresses to detect mirroring
        let header_0 = dumper.read_rom_range(0x7FB0, 16)?;
        
        // Check sizes from 256KB to 4MB
        let sizes = [0x40000, 0x80000, 0x100000, 0x200000, 0x400000];
        
        for &size in &sizes {
            let mirror_addr = size + 0x7FB0;
            if let Ok(mirror_data) = dumper.read_rom_range(mirror_addr as u32, 16) {
                if mirror_data == header_0 {
                    return Ok(size);
                }
            }
        }
        
        // Default to largest size if no mirroring detected
        Ok(0x400000)
    }

    /// Verify checksum against header
    pub fn verify_checksum(data: &[u8]) -> Result<()> {
        let calculated = calculate_checksum(data);
        
        // Read checksum from header (offset depends on LoROM/HiROM)
        let header_offset = if data.len() >= 0x8000 && data[0x7FD5] == 0x20 {
            // LoROM
            0x7FDC
        } else if data.len() >= 0x10000 {
            // HiROM
            0xFFDC
        } else {
            return Err(DumperError::ReadFailed {
                address: 0,
                message: "ROM too small for header".to_string(),
            });
        };
        
        if data.len() < header_offset + 2 {
            return Err(DumperError::ReadFailed {
                address: header_offset as u32,
                message: "Cannot read header checksum".to_string(),
            });
        }
        
        let header_checksum = u16::from_le_bytes([
            data[header_offset],
            data[header_offset + 1],
        ]);
        
        if calculated != header_checksum {
            return Err(DumperError::VerificationFailed {
                address: header_offset as u32,
                expected: (header_checksum >> 8) as u8,
                actual: (calculated >> 8) as u8,
            });
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_copier_detection_result() {
        let original = CopierDetectionResult::original();
        assert!(!original.is_reproduction);
        assert_eq!(original.confidence, 1.0);

        let indicators = vec![
            CopierIndicator::ModernFlashChip { chip_id: "MX29F160".to_string() },
        ];
        let repro = CopierDetectionResult::reproduction(indicators);
        assert!(repro.is_reproduction);
        assert!(repro.confidence > 0.0);
    }

    #[test]
    fn test_copier_indicator_confidence() {
        let indicator = CopierIndicator::ModernFlashChip { 
            chip_id: "TEST".to_string() 
        };
        assert_eq!(indicator.confidence(), 0.95);
        assert!(indicator.description().contains("Modern flash"));
    }

    #[test]
    fn test_memory_type_display() {
        assert_eq!(format!("{}", MemoryType::MaskRom), "Mask ROM");
        assert_eq!(format!("{}", MemoryType::Flash), "Flash");
    }

    #[test]
    fn test_ram_type_display() {
        assert_eq!(format!("{}", RamType::Sram), "SRAM");
        assert_eq!(format!("{}", RamType::Dram), "DRAM");
    }

    #[test]
    fn test_cartridge_info() {
        let info = CartridgeInfo::new()
            .with_rom_size(0x100000)
            .with_sram_size(0x2000)
            .with_mapper("LoROM")
            .with_region("NTSC")
            .with_battery(true);

        assert_eq!(info.rom_size, 0x100000);
        assert_eq!(info.sram_size, 0x2000);
        assert_eq!(info.mapper_type, "LoROM");
        assert!(info.has_battery);
    }

    #[test]
    fn test_calculate_checksum() {
        let data = vec![0x01, 0x02, 0x03, 0x04];
        let checksum = utils::calculate_checksum(&data);
        assert_eq!(checksum, 0x000A); // 1+2+3+4 = 10
    }
}
