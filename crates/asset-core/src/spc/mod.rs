//! SPC File Format Support
//!
//! SPC files are save states of the SPC700 audio chip, commonly used for
//! ripping and playing SNES music. The format was originally created by
//! the SPC700 emulator author.
//!
//! # SPC File Format (Version 0.30)
//! - Header: 33 bytes (signature + version)
//! - Registers: 9 bytes (PC, A, X, Y, PSW, SP)
//! - Reserved: 2 bytes
//! - ID666 Tag: 210 bytes (optional song info)
//! - Extended ID666: Additional fields (v0.31+)
//! - SPC700 RAM: 65536 bytes
//! - DSP Registers: 128 bytes
//! - Extra RAM: 64 bytes (unused port values)
//!
//! # File Structure
//! ```
//! Offset  Size  Description
//! 0x00    33    Header signature
//! 0x21    1     DOS end-of-file marker (0x1A)
//! 0x22    1     Minor version (0x30 = '0')
//! 0x23    1     Major version (0x30 = '0')
//! 0x25    2     PC register
//! 0x27    1     A register
//! 0x28    1     X register
//! 0x29    1     Y register
//! 0x2A    1     PSW register
//! 0x2B    1     SP register
//! 0x2C    2     Reserved
//! 0x2E    210   ID666 tag (optional)
//! 0x100   65536 SPC700 RAM
//! 0x10100 128   DSP registers
//! 0x10180 64    Extra RAM
//! ```
//!
//! ## Example
//! ```
//! use asset_core::spc::{SpcFile, SpcBuilder, Id666Tag};
//! use std::path::Path;
//!
//! // Load an SPC file
//! // let data = SpcFile::load("music.spc").unwrap();
//!
//! // Create a new SPC file
//! let builder = SpcBuilder::new()
//!     .with_song_title("Test Song")
//!     .with_game_title("Test Game");
//! ```

use crate::audio::{resample_to_rate, write_wav_file, Spc700Data, WavError, SPC_SAMPLE_RATE};
use crate::brr::BrrDecoder;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

mod constants;
mod version;
mod id666;
mod builder;

// Re-exports to maintain the same public API
pub use constants::*;
pub use version::SpcVersion;
pub use id666::Id666Tag;
pub use builder::SpcBuilder;

/// SPC file handler.
///
/// Provides methods for loading, saving, and manipulating SPC files.
///
/// # Example
/// ```
/// use asset_core::spc::SpcFile;
///
/// let spc = SpcFile::new();
/// // Load: let data = SpcFile::load("file.spc").unwrap();
/// ```
pub struct SpcFile;

/// SPC load error types.
#[derive(Debug, Clone)]
pub enum SpcError {
    /// File not found
    NotFound(String),
    /// Invalid signature
    InvalidSignature,
    /// IO error
    IoError(String),
    /// Invalid format
    InvalidFormat(String),
    /// Unsupported version
    UnsupportedVersion(String),
}

impl std::fmt::Display for SpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SpcError::NotFound(path) => write!(f, "SPC file not found: {}", path),
            SpcError::InvalidSignature => write!(f, "Invalid SPC file signature"),
            SpcError::IoError(msg) => write!(f, "IO error: {}", msg),
            SpcError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
            SpcError::UnsupportedVersion(ver) => write!(f, "Unsupported version: {}", ver),
        }
    }
}

impl std::error::Error for SpcError {}

impl From<std::io::Error> for SpcError {
    fn from(err: std::io::Error) -> Self {
        SpcError::IoError(err.to_string())
    }
}

impl From<WavError> for SpcError {
    fn from(err: WavError) -> Self {
        SpcError::InvalidFormat(format!("WAV error: {}", err))
    }
}

impl SpcFile {
    /// Creates a new SPC file handler.
    pub fn new() -> Self {
        Self
    }

    /// Loads an SPC file from a path.
    ///
    /// # Arguments
    /// - `path`: Path to the SPC file
    ///
    /// # Returns
    /// - `Ok(Spc700Data)` containing the loaded SPC data
    /// - `Err(SpcError)` if loading fails
    ///
    /// # Example
    /// ```
    /// use asset_core::spc::SpcFile;
    ///
    /// // let data = SpcFile::load("music.spc").unwrap();
    /// ```
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Spc700Data, SpcError> {
        let path = path.as_ref();
        let mut file = File::open(path)
            .map_err(|e| SpcError::NotFound(format!("{}: {}", path.display(), e)))?;

        // Read header (33 bytes)
        let mut header = [0u8; 33];
        file.read_exact(&mut header)
            .map_err(|e| SpcError::IoError(e.to_string()))?;

        // Verify signature
        if !header.starts_with(SPC_SIGNATURE) {
            return Err(SpcError::InvalidSignature);
        }

        // Read registers (9 bytes at offset 0x25)
        file.seek(SeekFrom::Start(0x25))
            .map_err(|e| SpcError::IoError(e.to_string()))?;

        let mut reg_bytes = [0u8; 9];
        file.read_exact(&mut reg_bytes)
            .map_err(|e| SpcError::IoError(e.to_string()))?;

        let pc = u16::from_le_bytes([reg_bytes[0], reg_bytes[1]]);
        let a = reg_bytes[2];
        let x = reg_bytes[3];
        let y = reg_bytes[4];
        let psw = reg_bytes[5];
        let sp = reg_bytes[6];

        // Read RAM (64KB at offset 0x100)
        file.seek(SeekFrom::Start(SPC_RAM_OFFSET as u64))
            .map_err(|e| SpcError::IoError(e.to_string()))?;

        let mut ram = [0u8; 65536];
        file.read_exact(&mut ram)
            .map_err(|e| SpcError::IoError(e.to_string()))?;

        // Read DSP registers (128 bytes at offset 0x10100)
        file.seek(SeekFrom::Start(SPC_DSP_OFFSET as u64))
            .map_err(|e| SpcError::IoError(e.to_string()))?;

        let mut dsp_registers = [0u8; 128];
        file.read_exact(&mut dsp_registers)
            .map_err(|e| SpcError::IoError(e.to_string()))?;

        Ok(Spc700Data {
            ram: ram.to_vec(),
            dsp_registers: dsp_registers.to_vec(),
            sample_table: Vec::new(), // Extracted on demand
            sequences: Vec::new(),
            pc,
            a,
            x,
            y,
            sp,
            psw,
        })
    }

    /// Saves SPC700 data to a file.
    ///
    /// # Arguments
    /// - `data`: SPC700 data to save
    /// - `path`: Output file path
    /// - `tag`: Optional ID666 metadata
    ///
    /// # Example
    /// ```
    /// use asset_core::spc::SpcFile;
    /// use asset_core::audio::Spc700Data;
    ///
    /// let data = Spc700Data::new();
    /// // SpcFile::save(&data, "output.spc", None).unwrap();
    /// ```
    pub fn save<P: AsRef<Path>>(
        data: &Spc700Data,
        path: P,
        tag: Option<&Id666Tag>,
    ) -> Result<(), SpcError> {
        let path = path.as_ref();
        let mut file = File::create(path)
            .map_err(|e| SpcError::IoError(format!("Cannot create {}: {}", path.display(), e)))?;

        // Write header
        let mut header = [0u8; SPC_HEADER_SIZE];

        // Copy signature (33 bytes including null terminator)
        let sig_len = SPC_SIGNATURE.len().min(33);
        header[0..sig_len].copy_from_slice(&SPC_SIGNATURE[0..sig_len]);
        header[0x21] = 0x1A; // DOS end-of-file marker

        // Version (assume 0.30)
        header[0x22] = 0x30;
        header[0x23] = 0x30;

        // PC register (2 bytes, little-endian)
        let pc_bytes = data.pc.to_le_bytes();
        header[0x25] = pc_bytes[0];
        header[0x26] = pc_bytes[1];

        // Other registers
        header[0x27] = data.a;
        header[0x28] = data.x;
        header[0x29] = data.y;
        header[0x2A] = data.psw;
        header[0x2B] = data.sp;

        // Reserved bytes
        header[0x2C] = 0x00;
        header[0x2D] = 0x00;

        // Write ID666 tag if provided (binary format)
        if let Some(tag) = tag {
            Self::write_id666_binary(&mut header[0x2E..0x100], tag);
        }

        file.write_all(&header)
            .map_err(|e| SpcError::IoError(e.to_string()))?;

        // Write RAM (64KB)
        file.write_all(&data.ram)
            .map_err(|e| SpcError::IoError(e.to_string()))?;

        // Write DSP registers (128 bytes)
        file.write_all(&data.dsp_registers)
            .map_err(|e| SpcError::IoError(e.to_string()))?;

        // Write extra RAM (64 bytes)
        file.write_all(&[0u8; 64])
            .map_err(|e| SpcError::IoError(e.to_string()))?;

        Ok(())
    }

    /// Writes ID666 tag in binary format.
    fn write_id666_binary(buffer: &mut [u8], tag: &Id666Tag) {
        // Song title (32 bytes)
        Self::write_padded_string(&mut buffer[0x00..0x20], &tag.song_title, 32);

        // Game title (32 bytes)
        Self::write_padded_string(&mut buffer[0x20..0x40], &tag.game_title, 32);

        // Dumper name (16 bytes)
        Self::write_padded_string(&mut buffer[0x40..0x50], &tag.dumper, 16);

        // Comments (32 bytes)
        Self::write_padded_string(&mut buffer[0x50..0x70], &tag.comments, 32);

        // Dump date (11 bytes, MMDDYYYY format)
        Self::write_padded_string(&mut buffer[0x70..0x7B], &tag.dump_date, 11);

        // Seconds to play (3 bytes, BCD)
        let seconds = tag.seconds_to_play.min(999);
        buffer[0x7B] = ((seconds / 100) as u8) << 4 | ((seconds / 10) % 10) as u8;
        buffer[0x7C] = ((seconds % 10) as u8) << 4;
        buffer[0x7D] = 0;

        // Fade length (4 bytes)
        let fade = tag.fade_length_ms.to_le_bytes();
        buffer[0x7E] = fade[0];
        buffer[0x7F] = fade[1];
        buffer[0x80] = fade[2];
        buffer[0x81] = fade[3];

        // Artist (32 bytes)
        Self::write_padded_string(&mut buffer[0x82..0xA2], &tag.artist, 32);

        // Channel disables (1 byte)
        buffer[0xA2] = tag.channel_disables;

        // Emulator (1 byte)
        buffer[0xA3] = tag.emulator;
    }

    /// Writes a padded string to buffer.
    fn write_padded_string(buffer: &mut [u8], text: &str, max_len: usize) {
        let bytes = text.as_bytes();
        let len = bytes.len().min(max_len);
        buffer[..len].copy_from_slice(&bytes[..len]);
        for i in len..max_len {
            buffer[i] = 0;
        }
    }

    /// Reads a padded string from buffer.
    fn read_padded_string(bytes: &[u8]) -> String {
        let len = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
        String::from_utf8_lossy(&bytes[..len]).to_string()
    }

    /// Reads 3-byte BCD value.
    fn read_bcd3(bytes: &[u8]) -> u32 {
        let b0 = bytes[0];
        let b1 = bytes[1];

        let hundreds = ((b0 >> 4) & 0x0F) as u32 * 100;
        let tens = (b0 & 0x0F) as u32 * 10;
        let ones = ((b1 >> 4) & 0x0F) as u32;

        hundreds + tens + ones
    }

    /// Reads an ID666 tag from a file.
    ///
    /// # Arguments
    /// - `path`: Path to the SPC file
    ///
    /// # Returns
    /// - `Ok(Some(Id666Tag))` if a tag is present
    /// - `Ok(None)` if no tag or text format
    /// - `Err(SpcError)` if reading fails
    pub fn read_id666<P: AsRef<Path>>(path: P) -> Result<Option<Id666Tag>, SpcError> {
        let path = path.as_ref();
        let mut file = File::open(path)
            .map_err(|e| SpcError::NotFound(format!("{}: {}", path.display(), e)))?;

        // Seek to ID666 area
        file.seek(SeekFrom::Start(0x2E))
            .map_err(|e| SpcError::IoError(e.to_string()))?;

        let mut tag_data = [0u8; 210];
        file.read_exact(&mut tag_data)
            .map_err(|e| SpcError::IoError(e.to_string()))?;

        // Check if first byte indicates text format (first byte should be non-zero for binary)
        if tag_data[0] == 0 {
            return Ok(None); // No tag or text format
        }

        let tag = Id666Tag {
            song_title: Self::read_padded_string(&tag_data[0x00..0x20]),
            game_title: Self::read_padded_string(&tag_data[0x20..0x40]),
            dumper: Self::read_padded_string(&tag_data[0x40..0x50]),
            comments: Self::read_padded_string(&tag_data[0x50..0x70]),
            dump_date: Self::read_padded_string(&tag_data[0x70..0x7B]),
            seconds_to_play: Self::read_bcd3(&tag_data[0x7B..0x7E]),
            fade_length_ms: u32::from_le_bytes([
                tag_data[0x7E],
                tag_data[0x7F],
                tag_data[0x80],
                tag_data[0x81],
            ]),
            artist: Self::read_padded_string(&tag_data[0x82..0xA2]),
            channel_disables: tag_data[0xA2],
            emulator: tag_data[0xA3],
        };

        Ok(Some(tag))
    }

    /// Checks if a file is a valid SPC file.
    ///
    /// # Arguments
    /// - `path`: Path to check
    ///
    /// # Returns
    /// `true` if the file has a valid SPC signature
    ///
    /// # Example
    /// ```
    /// use asset_core::spc::SpcFile;
    ///
    /// // if SpcFile::is_valid_spc("music.spc") {
    /// //     println!("Valid SPC file!");
    /// // }
    /// ```
    pub fn is_valid_spc<P: AsRef<Path>>(path: P) -> bool {
        if let Ok(mut file) = File::open(path) {
            let mut header = [0u8; 33];
            if file.read_exact(&mut header).is_ok() {
                return header.starts_with(SPC_SIGNATURE);
            }
        }
        false
    }

    /// Gets SPC file information without loading full data.
    ///
    /// # Arguments
    /// - `path`: Path to the SPC file
    ///
    /// # Returns
    /// File information including size and metadata
    pub fn get_info<P: AsRef<Path>>(path: P) -> Result<SpcFileInfo, SpcError> {
        let path = path.as_ref();
        let tag = Self::read_id666(path)?;
        let size = std::fs::metadata(path)
            .map(|m| m.len() as usize)
            .unwrap_or(0);

        Ok(SpcFileInfo {
            path: path.to_path_buf(),
            size,
            has_id666: tag.is_some(),
            tag,
        })
    }

    /// Exports SPC700 samples to a single WAV file.
    ///
    /// This is a preview/export function that decodes BRR samples from
    /// the SPC700 RAM and writes them as a WAV file.
    ///
    /// # Arguments
    /// - `data`: SPC700 data
    /// - `path`: Output WAV file path
    /// - `sample_addrs`: List of sample addresses in RAM
    /// - `target_rate`: Target sample rate for output
    ///
    /// # Returns
    /// `Ok(())` on success, `Err(SpcError)` on failure
    pub fn export_samples_to_wav<P: AsRef<Path>>(
        data: &Spc700Data,
        path: P,
        sample_addrs: &[u16],
        target_rate: u32,
    ) -> Result<(), SpcError> {
        let mut all_samples = Vec::new();
        let decoder = BrrDecoder::new();

        for &addr in sample_addrs {
            // Scan for BRR blocks starting at this address
            let mut pos = addr as usize;
            let mut brr_data = Vec::new();

            while pos + 9 <= data.ram.len() {
                let header = data.ram[pos];
                let end_flag = (header & 0x01) != 0;

                // Copy this block
                brr_data.extend_from_slice(&data.ram[pos..pos + 9]);
                pos += 9;

                if end_flag {
                    break;
                }

                // Safety limit
                if brr_data.len() > 65536 {
                    break;
                }
            }

            if !brr_data.is_empty() {
                let pcm = decoder.decode(&brr_data);
                all_samples.extend(pcm);
                // Add small silence between samples
                all_samples.extend(vec![0i16; 1000]);
            }
        }

        if all_samples.is_empty() {
            return Err(SpcError::InvalidFormat("No samples found".to_string()));
        }

        // Resample if needed
        let final_samples = if target_rate != SPC_SAMPLE_RATE {
            resample_to_rate(&all_samples, SPC_SAMPLE_RATE, target_rate)
        } else {
            all_samples
        };

        write_wav_file(path, &final_samples, target_rate)?;
        Ok(())
    }
}

impl Default for SpcFile {
    fn default() -> Self {
        Self::new()
    }
}

/// Information about an SPC file.
#[derive(Debug, Clone)]
pub struct SpcFileInfo {
    /// File path
    pub path: std::path::PathBuf,
    /// File size in bytes
    pub size: usize,
    /// Has ID666 metadata
    pub has_id666: bool,
    /// ID666 tag if present
    pub tag: Option<Id666Tag>,
}

/// Extracts all BRR samples from SPC700 RAM.
///
/// Scans RAM for valid BRR blocks and extracts them as samples.
/// This is a heuristic-based extraction.
///
/// # Arguments
/// - `ram`: SPC700 RAM data (64KB)
///
/// # Returns
/// Vector of found samples with their start addresses
pub fn extract_samples_from_ram(ram: &[u8]) -> Vec<(u16, Vec<u8>)> {
    let mut samples = Vec::new();
    let mut pos = 0;

    while pos + 9 <= ram.len() {
        // Check if this looks like a valid BRR header
        let header = ram[pos];
        let range = (header >> 4) & 0x0F;
        let filter = (header >> 2) & 0x03;

        // Basic sanity checks
        if range <= 12 && filter <= 3 {
            // Try to read the sample
            let mut sample_data = Vec::new();
            let start_pos = pos;

            loop {
                if pos + 9 > ram.len() {
                    break;
                }

                let block_header = ram[pos];
                sample_data.extend_from_slice(&ram[pos..pos + 9]);
                pos += 9;

                let end_flag = (block_header & 0x01) != 0;
                if end_flag {
                    // Valid sample found
                    if sample_data.len() >= 18 {
                        // At least 2 blocks
                        samples.push((start_pos as u16, sample_data));
                    }
                    break;
                }

                // Safety limit
                if sample_data.len() > 32768 {
                    break;
                }
            }
        } else {
            pos += 1;
        }
    }

    samples
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_spc_signature() {
        assert_eq!(SPC_SIGNATURE.len(), 33);
        assert!(SPC_SIGNATURE.starts_with(b"SNES-SPC700"));
    }

    #[test]
    fn test_spc_builder() {
        let builder = SpcBuilder::new()
            .with_song_title("Test Song")
            .with_game_title("Test Game")
            .with_artist("Test Artist");

        assert!(builder.tag.is_some());
        let tag = builder.tag.unwrap();
        assert_eq!(tag.song_title, "Test Song");
        assert_eq!(tag.game_title, "Test Game");
        assert_eq!(tag.artist, "Test Artist");
    }

    #[test]
    fn test_padded_string_roundtrip() {
        let original = "Hello";
        let mut buffer = [0u8; 10];

        SpcFile::write_padded_string(&mut buffer, original, 10);
        let recovered = SpcFile::read_padded_string(&buffer);

        assert_eq!(original, recovered);
    }

    #[test]
    fn test_is_valid_spc_invalid() {
        let mut temp = NamedTempFile::new().unwrap();
        temp.write_all(b"NOT AN SPC FILE").unwrap();

        assert!(!SpcFile::is_valid_spc(temp.path()));
    }

    #[test]
    fn test_id666_tag_builder() {
        let tag = Id666Tag::new()
            .with_song_title("Test")
            .with_game_title("Game")
            .with_artist("Artist")
            .with_duration(120)
            .with_fade(5000);

        assert_eq!(tag.song_title, "Test");
        assert_eq!(tag.seconds_to_play, 120);
        assert_eq!(tag.fade_length_ms, 5000);
    }

    #[test]
    fn test_spc_constants() {
        assert_eq!(SPC_HEADER_SIZE, 256);
        assert_eq!(SPC_RAM_OFFSET, 0x100);
        assert_eq!(SPC_DSP_OFFSET, 0x10100);
        assert_eq!(SPC_FILE_SIZE, 0x10200);
    }

    #[test]
    fn test_spc_save_and_load() {
        // Create test SPC data
        let mut data = Spc700Data::new();
        data.pc = 0x0400;
        data.a = 0x12;
        data.x = 0x34;
        data.y = 0x56;
        data.sp = 0xEF;
        data.psw = 0x00;

        // Put some test data in RAM
        data.ram[0x0400] = 0xCD;
        data.ram[0x0401] = 0xEF;

        // Set DSP register
        data.dsp_registers[0] = 0x7F;

        // Create tag
        let tag = Id666Tag::new()
            .with_song_title("Test Song")
            .with_game_title("Test Game");

        // Save to temp file
        let temp = NamedTempFile::new().unwrap();
        SpcFile::save(&data, temp.path(), Some(&tag)).unwrap();

        // Load it back
        let loaded = SpcFile::load(temp.path()).unwrap();

        assert_eq!(loaded.pc, 0x0400);
        assert_eq!(loaded.a, 0x12);
        assert_eq!(loaded.x, 0x34);
        assert_eq!(loaded.y, 0x56);
        assert_eq!(loaded.sp, 0xEF);
        assert_eq!(loaded.ram[0x0400], 0xCD);
        assert_eq!(loaded.ram[0x0401], 0xEF);
        assert_eq!(loaded.dsp_registers[0], 0x7F);

        // Check ID666
        let read_tag = SpcFile::read_id666(temp.path()).unwrap();
        assert!(read_tag.is_some());
        let read_tag = read_tag.unwrap();
        assert_eq!(read_tag.song_title, "Test Song");
        assert_eq!(read_tag.game_title, "Test Game");
    }

    #[test]
    fn test_version_detection() {
        assert_eq!(SpcVersion::from_bytes(0x30, 0x30), SpcVersion::V030);
        assert_eq!(SpcVersion::from_bytes(0x30, 0x31), SpcVersion::V031);
        assert_eq!(SpcVersion::from_bytes(0x00, 0x00), SpcVersion::Unknown);
    }

    #[test]
    fn test_bcd_reading() {
        // Test BCD reading: 123 = 0x12, 0x30 (BCD encoded)
        let bcd_bytes = [0x12, 0x30, 0x00];
        let value = SpcFile::read_bcd3(&bcd_bytes);
        assert_eq!(value, 123);
    }

    #[test]
    fn test_get_info() {
        // Create and save test SPC
        let data = Spc700Data::new();
        let tag = Id666Tag::new()
            .with_song_title("Info Test")
            .with_game_title("Test Game");

        let temp = NamedTempFile::new().unwrap();
        SpcFile::save(&data, temp.path(), Some(&tag)).unwrap();

        // Get info
        let info = SpcFile::get_info(temp.path()).unwrap();

        assert!(info.has_id666);
        assert!(info.tag.is_some());
        assert_eq!(info.tag.unwrap().song_title, "Info Test");
        assert_eq!(info.size, SPC_FILE_SIZE);
    }

    #[test]
    fn test_extract_samples_from_ram() {
        // Create RAM with a simple BRR sample
        let mut ram = vec![0u8; 65536];

        // Block 1: header with range=1, filter=0, no flags
        ram[0x1000] = 0x10;
        // Sample data (16 nibbles = 8 bytes)
        for i in 0..8 {
            ram[0x1001 + i] = 0x12;
        }

        // Block 2: header with end flag
        ram[0x1009] = 0x11; // range=1, filter=0, end flag set
        for i in 0..8 {
            ram[0x100A + i] = 0x34;
        }

        let samples = extract_samples_from_ram(&ram);

        // Should find at least one sample
        assert!(!samples.is_empty());

        // Check the first sample
        let (addr, data) = &samples[0];
        assert_eq!(*addr, 0x1000);
        assert_eq!(data.len(), 18); // 2 blocks
    }
}
