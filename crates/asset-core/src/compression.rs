//! HAL8-style compression/decompression for Super Punch-Out!! assets.
//!
//! Super Punch-Out!! uses a variant of HAL Laboratory's compression format
//! (HAL8) for its graphics data. This format uses bitplane-interleaved LZ
//! compression with multiple command types.
//!
//! ## Compression Commands
//! - `0`: Literals - Copy bytes directly from input
//! - `1`: Byte RLE - Repeat a single byte
//! - `2`: Word RLE - Repeat a 2-byte pattern
//! - `3`: Incremental RLE - Repeat with incrementing values
//! - `4`: LZ Copy - Copy from previously decompressed data
//! - `5-7`: Reserved/End
//!
//! ## Format
//! Each control byte is structured as:
//! - Bits 5-7: Command type (0-7)
//! - Bits 0-4: Length - 1 (0-31 means 1-32 bytes)
//!
//! The value `0xFF` signals end of stream.
//!
//! ## Example
//! ```
//! use asset_core::Decompressor;
//!
//! let compressed_data = vec![0x00, 0xAB, 0xCD, 0xFF]; // Literal 2 bytes, then end
//! let mut decompressor = Decompressor::new(&compressed_data);
//! let result = decompressor.decompress_interleaved(256);
//! ```

/// Maximum length for a single command (32 bytes)
pub const MAX_COMMAND_LENGTH: usize = 32;

/// End-of-stream marker
pub const END_OF_STREAM: u8 = 0xFF;

/// Command type mask (bits 5-7)
pub const COMMAND_MASK: u8 = 0xE0;

/// Length mask (bits 0-4)
pub const LENGTH_MASK: u8 = 0x1F;

/// Shift for command type
pub const COMMAND_SHIFT: u8 = 5;

/// Super Punch-Out!! Decompressor (HAL8-style bitplane-interleaved LZ)
///
/// This decompressor handles the custom compression format used by
/// Super Punch-Out!! for graphics data. Assets are typically stored
/// in two compressed blocks: one for bitplanes 0/1 and one for bitplanes 2/3.
///
/// # Example
/// ```
/// use asset_core::Decompressor;
///
/// let compressed = vec![0x00, 0xAB, 0xCD, 0xFF];
/// let mut decompressor = Decompressor::new(&compressed);
/// let data = decompressor.decompress_interleaved(256);
/// ```
pub struct Decompressor<'a> {
    /// Input compressed data
    input: &'a [u8],
    /// Current position in input
    pos: usize,
}

impl<'a> Decompressor<'a> {
    /// Creates a new decompressor with the given input data.
    ///
    /// # Arguments
    /// - `input`: The compressed data to decompress
    ///
    /// # Example
    /// ```
    /// use asset_core::Decompressor;
    ///
    /// let data = vec![0xFF]; // End-of-stream marker
    /// let decompressor = Decompressor::new(&data);
    /// ```
    pub fn new(input: &'a [u8]) -> Self {
        Self { input, pos: 0 }
    }

    /// Reads a single byte from the input.
    ///
    /// Returns 0 if at end of input.
    fn read_byte(&mut self) -> u8 {
        if self.pos >= self.input.len() {
            return 0;
        }
        let b = self.input[self.pos];
        self.pos += 1;
        b
    }

    /// Reads a 16-bit little-endian value from the input.
    fn read_u16(&mut self) -> u16 {
        let l = self.read_byte() as u16;
        let h = self.read_byte() as u16;
        (h << 8) | l
    }

    /// Decompresses a single pass (e.g., either bitplanes 0/1 or 2/3).
    ///
    /// This method decompresses one interleaved pass, writing to every
    /// `step` bytes starting at `start_offset`.
    ///
    /// # Arguments
    /// - `output`: The output buffer to write to
    /// - `start_offset`: Where to start writing in the output
    /// - `step`: Number of bytes to skip between writes (usually 2 for interleaving)
    ///
    /// # Example
    /// ```
    /// use asset_core::Decompressor;
    ///
    /// let compressed = vec![0x00, 0xAB, 0xCD, 0xFF];
    /// let mut decompressor = Decompressor::new(&compressed);
    /// let mut output = vec![0u8; 256];
    /// decompressor.decompress_pass(&mut output, 0, 2); // Even bytes
    /// ```
    pub fn decompress_pass(&mut self, output: &mut [u8], start_offset: usize, step: usize) {
        let mut out_idx = start_offset;

        while self.pos < self.input.len() {
            let ctrl = self.read_byte();
            if ctrl == END_OF_STREAM {
                break;
            }

            let cmd = ctrl >> COMMAND_SHIFT;
            let len = ((ctrl & LENGTH_MASK) as usize) + 1;

            match cmd {
                0 => {
                    // Literals - copy bytes directly
                    for _ in 0..len {
                        if out_idx < output.len() {
                            output[out_idx] = self.read_byte();
                            out_idx += step;
                        }
                    }
                }
                1 => {
                    // Byte RLE - repeat a single byte
                    let val = self.read_byte();
                    for _ in 0..len {
                        if out_idx < output.len() {
                            output[out_idx] = val;
                            out_idx += step;
                        }
                    }
                }
                2 => {
                    // Word RLE - repeat a 2-byte pattern
                    let b1 = self.read_byte();
                    let b2 = self.read_byte();
                    for _ in 0..len {
                        if out_idx < output.len() {
                            output[out_idx] = b1;
                            out_idx += step;
                        }
                        if out_idx < output.len() {
                            output[out_idx] = b2;
                            out_idx += step;
                        }
                    }
                }
                3 => {
                    // Incremental RLE - repeat with incrementing values
                    let mut val = self.read_byte();
                    for _ in 0..len {
                        if out_idx < output.len() {
                            output[out_idx] = val;
                            out_idx += step;
                            val = val.wrapping_add(1);
                        }
                    }
                }
                4 => {
                    // LZ Copy from already decompressed data
                    let lz_offset = self.read_u16() as usize;
                    // Copy from the same bitplane stream
                    for j in 0..len {
                        let src_idx = lz_offset + j * step;
                        if out_idx < output.len() && src_idx < out_idx {
                            output[out_idx] = output[src_idx];
                            out_idx += step;
                        }
                    }
                }
                _ => break, // Unknown command - stop decompression
            }
        }
    }

    /// Decompresses a full Super Punch-Out!! 4bpp asset.
    ///
    /// Many SPO assets use two-pass decompression for interleaved bitplanes:
    /// - Pass 1: Bitplanes 0/1 (even bytes)
    /// - Pass 2: Bitplanes 2/3 (odd bytes)
    ///
    /// # Arguments
    /// - `expected_size`: Expected decompressed size in bytes
    ///
    /// # Returns
    /// The decompressed data as a vector of bytes
    ///
    /// # Example
    /// ```
    /// use asset_core::Decompressor;
    ///
    /// // Example compressed data (would be actual compressed data in practice)
    /// let compressed = vec![0xFF]; // Minimal: just end marker
    /// let mut decompressor = Decompressor::new(&compressed);
    /// let data = decompressor.decompress_interleaved(1024);
    /// ```
    pub fn decompress_interleaved(&mut self, expected_size: usize) -> Vec<u8> {
        let mut output = vec![0u8; expected_size];

        // Pass 1: Bitplanes 0/1 (Even bytes)
        self.decompress_pass(&mut output, 0, 2);

        // Pass 2: Bitplanes 2/3 (Odd bytes)
        // The second pass usually follows immediately in the input stream
        self.decompress_pass(&mut output, 1, 2);

        output
    }

    /// Returns the current position in the input stream.
    ///
    /// Useful for debugging or determining how much data was consumed.
    pub fn position(&self) -> usize {
        self.pos
    }

    /// Returns the total input size.
    pub fn input_size(&self) -> usize {
        self.input.len()
    }

    /// Returns the number of bytes remaining in the input.
    pub fn bytes_remaining(&self) -> usize {
        self.input.len().saturating_sub(self.pos)
    }
}

/// Compression command types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CommandType {
    /// Copy literal bytes from input
    Literals = 0,
    /// Repeat a single byte
    ByteRle = 1,
    /// Repeat a 2-byte pattern
    WordRle = 2,
    /// Repeat with incrementing values
    IncrementalRle = 3,
    /// Copy from previously decompressed data
    LzCopy = 4,
}

impl CommandType {
    /// Converts a command byte value to a CommandType.
    ///
    /// Returns `None` for unknown command types.
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Literals),
            1 => Some(Self::ByteRle),
            2 => Some(Self::WordRle),
            3 => Some(Self::IncrementalRle),
            4 => Some(Self::LzCopy),
            _ => None,
        }
    }

    /// Returns a human-readable name for this command type.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Literals => "Literals",
            Self::ByteRle => "Byte RLE",
            Self::WordRle => "Word RLE",
            Self::IncrementalRle => "Incremental RLE",
            Self::LzCopy => "LZ Copy",
        }
    }
}

/// Statistics about decompression.
#[derive(Debug, Clone, Default)]
pub struct DecompressionStats {
    /// Total bytes read from input
    pub bytes_read: usize,
    /// Total bytes written to output
    pub bytes_written: usize,
    /// Number of commands processed
    pub command_count: usize,
    /// Breakdown by command type
    pub commands_by_type: std::collections::HashMap<u8, usize>,
}

/// Analyzes compressed data without fully decompressing it.
///
/// Returns statistics about the compression structure.
///
/// # Example
/// ```
/// use asset_core::analyze_compression;
///
/// let data = vec![0x00, 0xAB, 0xCD, 0xFF];
/// let stats = analyze_compression(&data);
/// ```
pub fn analyze_compression(data: &[u8]) -> DecompressionStats {
    let mut stats = DecompressionStats::default();
    let mut pos = 0;

    while pos < data.len() {
        let ctrl = data[pos];
        pos += 1;
        stats.bytes_read += 1;

        if ctrl == END_OF_STREAM {
            break;
        }

        let cmd = ctrl >> COMMAND_SHIFT;
        let len = ((ctrl & LENGTH_MASK) as usize) + 1;

        *stats.commands_by_type.entry(cmd).or_insert(0) += 1;
        stats.command_count += 1;
        stats.bytes_written += len;

        // Skip command data
        match cmd {
            0 => pos += len, // Literals
            1 => pos += 1,   // Byte RLE
            2 => pos += 2,   // Word RLE
            3 => pos += 1,   // Incremental RLE
            4 => pos += 2,   // LZ Copy
            _ => break,
        }
    }

    stats
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decompressor_new() {
        let data = vec![0xFF];
        let decompressor = Decompressor::new(&data);
        assert_eq!(decompressor.position(), 0);
        assert_eq!(decompressor.input_size(), 1);
    }

    #[test]
    fn test_decompress_empty() {
        let data = vec![0xFF]; // Just end marker
        let mut decompressor = Decompressor::new(&data);
        let output = decompressor.decompress_interleaved(16);
        assert_eq!(output.len(), 16);
        assert!(output.iter().all(|&b| b == 0));
    }

    #[test]
    fn test_decompress_literals() {
        // Command 0 (Literals), length 3: 0x00 | (3-1) = 0x02
        let data = vec![0x02, 0xAA, 0xBB, 0xCC, 0xFF];
        let mut decompressor = Decompressor::new(&data);
        let mut output = vec![0u8; 8];
        decompressor.decompress_pass(&mut output, 0, 1);

        assert_eq!(output[0], 0xAA);
        assert_eq!(output[1], 0xBB);
        assert_eq!(output[2], 0xCC);
    }

    #[test]
    fn test_decompress_byte_rle() {
        // Command 1 (Byte RLE), length 4: 0x20 | (4-1) = 0x23
        let data = vec![0x23, 0xAB, 0xFF];
        let mut decompressor = Decompressor::new(&data);
        let mut output = vec![0u8; 8];
        decompressor.decompress_pass(&mut output, 0, 1);

        assert_eq!(output[0], 0xAB);
        assert_eq!(output[1], 0xAB);
        assert_eq!(output[2], 0xAB);
        assert_eq!(output[3], 0xAB);
    }

    #[test]
    fn test_command_type() {
        assert_eq!(CommandType::from_u8(0), Some(CommandType::Literals));
        assert_eq!(CommandType::from_u8(1), Some(CommandType::ByteRle));
        assert_eq!(CommandType::from_u8(4), Some(CommandType::LzCopy));
        assert_eq!(CommandType::from_u8(7), None);
    }

    #[test]
    fn test_analyze_compression() {
        let data = vec![0x23, 0xAB, 0x02, 0x11, 0x22, 0x33, 0xFF];
        let stats = analyze_compression(&data);

        assert_eq!(stats.command_count, 2);
        assert!(stats.commands_by_type.contains_key(&1));
        assert!(stats.commands_by_type.contains_key(&0));
    }
}
