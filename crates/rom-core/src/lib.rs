use sha1::{Digest, Sha1};
use std::fs;
use std::path::Path;
use thiserror::Error;

pub mod comparison;
pub use comparison::*;

pub mod roster;
pub use roster::*;

pub mod text;
pub use text::*;

pub mod region;
pub use region::*;

pub const EXPECTED_SHA1: &str = "3604c855790f37db567e9b425252625045f86697";
pub const EXPECTED_SIZE: usize = 2097152;
pub const HEADER_SIZE: usize = 512;

/// All known valid SHA1 hashes for supported ROM versions
pub const KNOWN_SHA1_HASHES: &[&str] = &[
    // USA version (verified)
    "3604c855790f37db567e9b425252625045f86697",
    // NOTE: JPN and PAL hashes to be added when those regions are supported
];

/// Standard SNES ROM sizes
pub const SIZE_2MB: usize = 2 * 1024 * 1024;
pub const SIZE_2_5MB: usize = 2560 * 1024; // 2.5MB (20Mbit) - common expansion
pub const SIZE_4MB: usize = 4 * 1024 * 1024;

/// Common free space regions in the original SPO ROM (PC offsets)
/// These are areas that appear to be unused or padding
pub const KNOWN_FREE_REGIONS: &[(usize, usize)] = &[
    // Extended header area (sometimes unused)
    (0x007FC0, 0x008000), // 64 bytes at end of header
];

/// Information about free space found in the ROM
#[derive(Debug, Clone)]
pub struct FreeSpaceInfo {
    pub offset: usize,
    pub size: usize,
    pub region_type: FreeSpaceRegion,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FreeSpaceRegion {
    /// Empty area within the original ROM
    Internal,
    /// Space available after ROM expansion
    Expanded,
    /// End of a bank with some free bytes
    EndOfBank,
}

impl std::fmt::Display for FreeSpaceRegion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FreeSpaceRegion::Internal => write!(f, "internal"),
            FreeSpaceRegion::Expanded => write!(f, "expanded"),
            FreeSpaceRegion::EndOfBank => write!(f, "end_of_bank"),
        }
    }
}

#[derive(Error, Debug)]
pub enum RomError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("ROM is too small")]
    TooSmall,
    #[error("ROM checksum mismatch. Expected {expected}, got {actual}")]
    ChecksumMismatch { expected: String, actual: String },
    #[error("Invalid offset: {0}")]
    InvalidOffset(usize),
    #[error("Region not supported: {0}")]
    RegionNotSupported(String),
    #[error("Region configuration incomplete for {0}")]
    RegionConfigurationIncomplete(String),
}

pub struct Rom {
    pub data: Vec<u8>,
}

impl Rom {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }

    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, RomError> {
        let mut data = fs::read(path)?;

        // Detect and strip SMC header if present
        if Self::detect_header(&data) {
            data.drain(0..HEADER_SIZE);
        }

        Ok(Self { data })
    }

    pub fn detect_header(data: &[u8]) -> bool {
        // SNES ROMs are usually multiples of 0x8000 (32KB)
        // If it has a 512-byte header, size % 1024 == 512.
        data.len() % 1024 == HEADER_SIZE
    }

    pub fn validate(&self) -> Result<(), RomError> {
        self.validate_with_hashes(&[EXPECTED_SHA1])
    }

    /// Validate the ROM against a list of known good SHA1 hashes
    pub fn validate_with_hashes(&self, valid_hashes: &[&str]) -> Result<(), RomError> {
        if self.data.len() < EXPECTED_SIZE {
            return Err(RomError::TooSmall);
        }

        let actual_sha1 = self.calculate_sha1();
        if !valid_hashes.contains(&actual_sha1.as_str()) {
            return Err(RomError::ChecksumMismatch {
                expected: if valid_hashes.len() == 1 {
                    valid_hashes[0].to_string()
                } else {
                    format!("one of {} known hashes", valid_hashes.len())
                },
                actual: actual_sha1,
            });
        }

        Ok(())
    }

    /// Validate the ROM against all known hashes (multi-region support)
    pub fn validate_any_region(&self) -> Result<RomRegion, RomError> {
        if self.data.len() < EXPECTED_SIZE {
            return Err(RomError::TooSmall);
        }

        let actual_sha1 = self.calculate_sha1();

        // Check all known hashes
        for (i, &hash) in KNOWN_SHA1_HASHES.iter().enumerate() {
            if actual_sha1 == hash {
                return match i {
                    0 => Ok(RomRegion::Usa),
                    // 1 => Ok(RomRegion::Jpn), // NOTE: Enable when JPN hash added
                    // 2 => Ok(RomRegion::Pal), // NOTE: Enable when PAL hash added
                    _ => Err(RomError::ChecksumMismatch {
                        expected: "known region".to_string(),
                        actual: actual_sha1,
                    }),
                };
            }
        }

        Err(RomError::ChecksumMismatch {
            expected: format!("one of {} known hashes", KNOWN_SHA1_HASHES.len()),
            actual: actual_sha1,
        })
    }

    /// Detect the region of this ROM
    pub fn detect_region(&self) -> Option<RomRegion> {
        RomRegion::detect(self)
    }

    /// Get the region configuration for this ROM
    pub fn get_region_config(&self) -> Option<RegionConfig> {
        self.detect_region().map(RegionConfig::for_region)
    }

    pub fn calculate_sha1(&self) -> String {
        let mut hasher = Sha1::new();
        hasher.update(&self.data);
        format!("{:x}", hasher.finalize())
    }

    pub fn read_bytes(&self, offset: usize, len: usize) -> Result<&[u8], RomError> {
        if offset + len > self.data.len() {
            return Err(RomError::InvalidOffset(offset));
        }
        Ok(&self.data[offset..offset + len])
    }

    pub fn write_bytes(&mut self, offset: usize, bytes: &[u8]) -> Result<(), RomError> {
        if offset + bytes.len() > self.data.len() {
            return Err(RomError::InvalidOffset(offset));
        }
        self.data[offset..offset + bytes.len()].copy_from_slice(bytes);
        Ok(())
    }

    pub fn snes_to_pc(&self, bank: u8, addr: u16) -> usize {
        // LoROM mapping: PC = (Bank & 0x7F) * 0x8000 + (Addr & 0x7FFF)
        ((bank as usize & 0x7F) * 0x8000) | (addr as usize & 0x7FFF)
    }

    /// Convert PC offset to SNES LoROM address
    pub fn pc_to_snes(&self, pc: usize) -> (u8, u16) {
        // LoROM: Bank = (PC / 0x8000) | 0x80, Addr = (PC % 0x8000) | 0x8000
        let bank = ((pc / 0x8000) | 0x80) as u8;
        let addr = ((pc % 0x8000) | 0x8000) as u16;
        (bank, addr)
    }

    /// Get the current ROM size
    pub fn size(&self) -> usize {
        self.data.len()
    }

    /// Check if ROM is expanded beyond original size
    pub fn is_expanded(&self) -> bool {
        self.data.len() > EXPECTED_SIZE
    }

    /// Expand ROM to a specific size (padding with 0x00 or 0xFF)
    pub fn expand_to(&mut self, new_size: usize, fill_byte: u8) {
        if new_size > self.data.len() {
            self.data.resize(new_size, fill_byte);
        }
    }

    /// Expand ROM to 2.5MB (common intermediate size)
    pub fn expand_to_2_5mb(&mut self) {
        self.expand_to(SIZE_2_5MB, 0x00);
    }

    /// Expand ROM to 4MB (maximum LoROM size)
    pub fn expand_to_4mb(&mut self) {
        self.expand_to(SIZE_4MB, 0x00);
    }

    /// Find a contiguous block of free space in the ROM
    ///
    /// # Arguments
    /// * `size` - Minimum size needed
    /// * `start_search` - PC offset to start searching from
    /// * `alignment` - Byte alignment required (e.g., 1 for any, 32 for tile-aligned)
    ///
    /// Returns the PC offset where free space was found, or None if not found
    pub fn find_free_space(
        &self,
        size: usize,
        start_search: usize,
        alignment: usize,
    ) -> Option<FreeSpaceInfo> {
        // First, check known free regions within original ROM bounds
        for (start, end) in KNOWN_FREE_REGIONS {
            if *start >= start_search && end.saturating_sub(*start) >= size {
                let aligned = Self::align_up(*start, alignment);
                if aligned + size <= *end {
                    return Some(FreeSpaceInfo {
                        offset: aligned,
                        size: end.saturating_sub(aligned),
                        region_type: FreeSpaceRegion::Internal,
                    });
                }
            }
        }

        // Look for end-of-bank gaps
        // Check the space at the end of each 32KB bank
        let bank_size = 0x8000;
        let num_banks = self.data.len() / bank_size;

        for bank in 0..num_banks {
            let bank_start = bank * bank_size;
            if bank_start < start_search {
                continue;
            }

            // Find the last non-zero byte in this bank
            let bank_end = (bank_start + bank_size).min(self.data.len());
            let mut last_used = bank_start;

            for i in (bank_start..bank_end).rev() {
                if self.data.get(i).copied().unwrap_or(0) != 0x00
                    && self.data.get(i).copied().unwrap_or(0) != 0xFF
                {
                    last_used = i + 1;
                    break;
                }
            }

            let available = bank_end.saturating_sub(last_used);
            if available >= size {
                let aligned = Self::align_up(last_used, alignment);
                if aligned + size <= bank_end {
                    return Some(FreeSpaceInfo {
                        offset: aligned,
                        size: available,
                        region_type: FreeSpaceRegion::EndOfBank,
                    });
                }
            }
        }

        // If ROM is expanded, check the expanded region
        if self.data.len() > EXPECTED_SIZE {
            let expanded_start = EXPECTED_SIZE.max(start_search);
            let aligned = Self::align_up(expanded_start, alignment);
            if aligned + size <= self.data.len() {
                return Some(FreeSpaceInfo {
                    offset: aligned,
                    size: self.data.len() - aligned,
                    region_type: FreeSpaceRegion::Expanded,
                });
            }
        }

        None
    }

    /// Find free space, expanding ROM if necessary
    pub fn find_or_expand_free_space(
        &mut self,
        size: usize,
        alignment: usize,
    ) -> Option<FreeSpaceInfo> {
        // Try to find space without expanding
        if let Some(info) = self.find_free_space(size, 0, alignment) {
            return Some(info);
        }

        // Try expanding to 2.5MB first
        if self.data.len() < SIZE_2_5MB {
            self.expand_to_2_5mb();
            if let Some(info) = self.find_free_space(size, EXPECTED_SIZE, alignment) {
                return Some(FreeSpaceInfo {
                    offset: info.offset,
                    size: info.size,
                    region_type: FreeSpaceRegion::Expanded,
                });
            }
        }

        // Try expanding to 4MB
        if self.data.len() < SIZE_4MB {
            self.expand_to_4mb();
            if let Some(info) = self.find_free_space(size, SIZE_2_5MB.max(EXPECTED_SIZE), alignment)
            {
                return Some(FreeSpaceInfo {
                    offset: info.offset,
                    size: info.size,
                    region_type: FreeSpaceRegion::Expanded,
                });
            }
        }

        None
    }

    /// Helper: Align a value up to the nearest multiple
    fn align_up(value: usize, alignment: usize) -> usize {
        if alignment <= 1 {
            value
        } else {
            ((value + alignment - 1) / alignment) * alignment
        }
    }

    /// Get the bank number for a PC offset
    pub fn get_bank(&self, pc: usize) -> u8 {
        ((pc / 0x8000) | 0x80) as u8
    }

    /// Check if a region is all zeros (likely free)
    pub fn is_region_empty(&self, start: usize, size: usize) -> bool {
        self.data
            .get(start..start.saturating_add(size))
            .map(|region| region.iter().all(|&b| b == 0x00 || b == 0xFF))
            .unwrap_or(false)
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), RomError> {
        fs::write(path, &self.data)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rom_expansion() {
        let data = vec![0u8; EXPECTED_SIZE];
        let mut rom = Rom::new(data);

        assert!(!rom.is_expanded());
        assert_eq!(rom.size(), EXPECTED_SIZE);

        rom.expand_to_2_5mb();
        assert!(rom.is_expanded());
        assert_eq!(rom.size(), SIZE_2_5MB);

        rom.expand_to_4mb();
        assert_eq!(rom.size(), SIZE_4MB);
    }

    #[test]
    fn test_align_up() {
        assert_eq!(Rom::align_up(0, 32), 0);
        assert_eq!(Rom::align_up(31, 32), 32);
        assert_eq!(Rom::align_up(32, 32), 32);
        assert_eq!(Rom::align_up(33, 32), 64);
        assert_eq!(Rom::align_up(1, 1), 1);
    }

    #[test]
    fn test_pc_to_snes() {
        let data = vec![0u8; EXPECTED_SIZE];
        let rom = Rom::new(data);

        // Test conversion: PC 0 -> Bank 0x80, Addr 0x8000
        let (bank, addr) = rom.pc_to_snes(0);
        assert_eq!(bank, 0x80);
        assert_eq!(addr, 0x8000);

        // Test conversion: PC 0x8000 -> Bank 0x81, Addr 0x8000
        let (bank, addr) = rom.pc_to_snes(0x8000);
        assert_eq!(bank, 0x81);
        assert_eq!(addr, 0x8000);

        // Test round-trip
        let pc = 0x12345;
        let (b, a) = rom.pc_to_snes(pc);
        assert_eq!(rom.snes_to_pc(b, a), pc);
    }

    #[test]
    fn test_is_region_empty() {
        let mut data = vec![0u8; EXPECTED_SIZE];
        let rom = Rom::new(data.clone());

        // All zeros should be empty
        assert!(rom.is_region_empty(0, 100));

        // Modify a byte
        data[50] = 0xAB;
        let rom = Rom::new(data);

        // Should not be empty where we modified
        assert!(!rom.is_region_empty(0, 100));
        // But should be empty elsewhere
        assert!(rom.is_region_empty(100, 100));
    }

    #[test]
    fn test_find_free_space_in_expanded_rom() {
        let data = vec![0u8; EXPECTED_SIZE];
        let mut rom = Rom::new(data);

        // Find space for 1KB
        let info = rom.find_or_expand_free_space(1024, 1);

        assert!(info.is_some());
        let info = info.unwrap();

        // Should be in expanded region since original is "full" (all zeros treated as used in end-of-bank search)
        // or in the known free regions
        assert!(info.size >= 1024);
        assert!(info.offset + 1024 <= rom.size());
    }

    #[test]
    fn test_get_bank() {
        let data = vec![0u8; EXPECTED_SIZE];
        let rom = Rom::new(data);

        assert_eq!(rom.get_bank(0), 0x80);
        assert_eq!(rom.get_bank(0x8000), 0x81);
        assert_eq!(rom.get_bank(0x10000), 0x82);
    }
}
