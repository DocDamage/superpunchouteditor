//! Compression/decompression helpers for Super Punch-Out!! assets.
//!
//! Fighter graphics in Super Punch-Out!! use a small base/flag codec. The
//! first byte is a continuation mask, followed by groups containing a base
//! byte and one or more eight-bit flag masks. A clear flag bit copies the base
//! byte and a set flag bit reads a literal byte. The game's decompressor uses
//! the output address, masked by the continuation mask, to decide when to
//! start a new base group.
//!
//! The older command-based HAL helpers below are retained for compatibility
//! with legacy synthetic fixtures and non-fighter tooling. They are not the
//! codec used by the compressed fighter sprite banks.
//!
//! ## Compression Commands
//! - `0`: Literals - Copy bytes directly from input
//! - `1`: Byte RLE - Repeat a single byte
//! - `2`: Word RLE - Repeat a 2-byte pattern
//! - `3`: Incremental RLE - Repeat with incrementing values
//! - `4`: Backreference - Copy previously decompressed bytes
//! - `5`: Rotated backreference - Copy with each byte bit-reversed
//! - `6`: Reversed backreference - Copy bytes in reverse order
//! - `7`: Backreference alias used by the original decompressor
//!
//! ## Legacy command format
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

/// Maximum length for a short command (32 bytes).
pub const MAX_COMMAND_LENGTH: usize = 32;

/// Maximum length for a long command (1024 bytes/items).
pub const MAX_LONG_COMMAND_LENGTH: usize = 1024;

/// Maximum output size accepted by the original HAL decompressor.
pub const MAX_DECOMPRESSED_SIZE: usize = 64 * 1024;

/// Continuation mask used by the game's compressed fighter graphics banks.
///
/// The original game normally stores two eight-byte flag chunks per base
/// group, which corresponds to a mask of `0x0F` while the decompressor's
/// output offset starts at `$8000`.
pub const SPO_GRAPHICS_CONTINUATION_MASK: u8 = 0x0F;

/// End-of-stream marker
pub const END_OF_STREAM: u8 = 0xFF;

/// Command type mask (bits 5-7)
pub const COMMAND_MASK: u8 = 0xE0;

/// Length mask (bits 0-4)
pub const LENGTH_MASK: u8 = 0x1F;

/// Shift for command type
pub const COMMAND_SHIFT: u8 = 5;

/// Decompressor for legacy command streams and SPO fighter graphics.
///
/// The command-based methods are kept for older callers. Use
/// [`Decompressor::decompress_sprite_graphics_exact`] for compressed fighter
/// sprite banks.
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

    fn read_byte_checked(&mut self) -> Result<u8, String> {
        let byte = self
            .input
            .get(self.pos)
            .copied()
            .ok_or_else(|| "Compressed stream ends unexpectedly".to_string())?;
        self.pos += 1;
        Ok(byte)
    }

    fn read_bytes_checked(&mut self, len: usize) -> Result<&'a [u8], String> {
        let end = self
            .pos
            .checked_add(len)
            .ok_or_else(|| "Compressed command length overflowed".to_string())?;
        let bytes = self
            .input
            .get(self.pos..end)
            .ok_or_else(|| "Compressed stream ends mid-command".to_string())?;
        self.pos = end;
        Ok(bytes)
    }

    /// Reads one HAL command and returns its command number and item count.
    ///
    /// Values with the high three bits set are long commands. Their command
    /// number is stored in bits 2-4 and their ten-bit length uses bits 0-1
    /// plus the following byte. `0xFF` is the stream terminator.
    fn read_command(&mut self) -> Result<Option<(u8, usize)>, String> {
        let control = self.read_byte_checked()?;
        if control == END_OF_STREAM {
            return Ok(None);
        }

        if (control & COMMAND_MASK) == COMMAND_MASK {
            let low = self.read_byte_checked()? as usize;
            let command = (control >> 2) & 0x07;
            let length = (((control & 0x03) as usize) << 8) | low;
            Ok(Some((command, length + 1)))
        } else {
            let command = control >> COMMAND_SHIFT;
            let length = ((control & LENGTH_MASK) as usize) + 1;
            Ok(Some((command, length)))
        }
    }

    fn ensure_output_capacity(output: &[u8], additional: usize) -> Result<(), String> {
        let new_len = output
            .len()
            .checked_add(additional)
            .ok_or_else(|| "Decompressed output length overflowed".to_string())?;
        if new_len > MAX_DECOMPRESSED_SIZE {
            return Err(format!(
                "Decompressed stream exceeds the {} byte HAL limit",
                MAX_DECOMPRESSED_SIZE
            ));
        }
        Ok(())
    }

    fn rotate_bits(byte: u8) -> u8 {
        byte.reverse_bits()
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
        if step == 0 {
            return;
        }

        let Ok(pass) = self.decompress_pass_exact() else {
            return;
        };

        for (index, byte) in pass.into_iter().enumerate() {
            let Some(output_index) = start_offset.checked_add(index.saturating_mul(step)) else {
                break;
            };
            if output_index >= output.len() {
                break;
            }
            output[output_index] = byte;
        }
    }

    /// Decompresses exactly one HAL stream, stopping after its `0xFF` marker.
    ///
    /// The returned bytes are a single bitplane pass. The decompressor's
    /// position is left immediately after the terminator so a second pass can
    /// be decoded from a concatenated asset.
    pub fn decompress_pass_exact(&mut self) -> Result<Vec<u8>, String> {
        let mut output = Vec::new();

        loop {
            let Some((command, length)) = self.read_command()? else {
                return Ok(output);
            };

            match command {
                0 => {
                    Self::ensure_output_capacity(&output, length)?;
                    output.extend_from_slice(self.read_bytes_checked(length)?);
                }
                1 => {
                    Self::ensure_output_capacity(&output, length)?;
                    let value = self.read_byte_checked()?;
                    output.extend(std::iter::repeat_n(value, length));
                }
                2 => {
                    let output_length = length
                        .checked_mul(2)
                        .ok_or_else(|| "Word RLE length overflowed".to_string())?;
                    Self::ensure_output_capacity(&output, output_length)?;
                    let value = [self.read_byte_checked()?, self.read_byte_checked()?];
                    for _ in 0..length {
                        output.extend_from_slice(&value);
                    }
                }
                3 => {
                    Self::ensure_output_capacity(&output, length)?;
                    let start = self.read_byte_checked()?;
                    output.extend((0..length).map(|index| start.wrapping_add(index as u8)));
                }
                4..=7 => {
                    Self::ensure_output_capacity(&output, length)?;
                    let offset = ((self.read_byte_checked()? as usize) << 8)
                        | self.read_byte_checked()? as usize;

                    for index in 0..length {
                        let source_index = match command {
                            6 => offset.checked_sub(index),
                            _ => offset.checked_add(index),
                        }
                        .ok_or_else(|| {
                            format!(
                                "HAL backreference at output byte {} points before the stream",
                                output.len()
                            )
                        })?;

                        let source = *output.get(source_index).ok_or_else(|| {
                            format!(
                                "HAL backreference at output byte {} points to unavailable byte {}",
                                output.len(),
                                source_index
                            )
                        })?;

                        output.push(if command == 5 {
                            Self::rotate_bits(source)
                        } else {
                            source
                        });
                    }
                }
                _ => {
                    return Err(format!("Unsupported HAL command {}", command));
                }
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
        let Ok(pass1) = self.decompress_pass_exact() else {
            return output;
        };
        let Ok(pass2) = self.decompress_pass_exact() else {
            return output;
        };

        for (index, byte) in pass1.into_iter().enumerate() {
            let output_index = index.saturating_mul(2);
            if output_index >= output.len() {
                break;
            }
            output[output_index] = byte;
        }
        for (index, byte) in pass2.into_iter().enumerate() {
            let output_index = index.saturating_mul(2).saturating_add(1);
            if output_index >= output.len() {
                break;
            }
            output[output_index] = byte;
        }
        output
    }

    /// Decompresses one Super Punch-Out!! compressed fighter graphics bank.
    ///
    /// The manifest points at the byte after the bank's two-byte end offset,
    /// so the input begins with the continuation mask. The output is the
    /// game's native SNES 4bpp tile byte stream.
    pub fn decompress_sprite_graphics_exact(&mut self) -> Result<Vec<u8>, String> {
        let mask = self.read_byte_checked()?;
        let mut output = Vec::new();
        let mut output_offset = 0x8000usize;

        while self.pos < self.input.len() {
            let base = self.read_byte_checked()?;

            loop {
                let flags = self.read_byte_checked()?;
                for bit in (0..8).rev() {
                    let byte = if flags & (1 << bit) != 0 {
                        self.read_byte_checked()?
                    } else {
                        base
                    };
                    Self::ensure_output_capacity(&output, 1)?;
                    output.push(byte);
                }

                output_offset = output_offset
                    .checked_add(8)
                    .ok_or_else(|| "Sprite graphics output offset overflowed".to_string())?;

                // The original routine tests the updated output address,
                // not the last byte written. When the masked address is
                // non-zero it consumes another flag byte using the same base.
                if output_offset & usize::from(mask) == 0 {
                    break;
                }
            }
        }

        Ok(output)
    }

    /// Decompresses one fighter graphics bank.
    ///
    /// This name is retained for callers that previously assumed the game
    /// stored two interleaved HAL passes. In the actual fighter format there
    /// is one base/flag stream and it already expands to the final 4bpp byte
    /// order.
    pub fn decompress_interleaved_exact(&mut self) -> Result<Vec<u8>, String> {
        self.decompress_sprite_graphics_exact()
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

/// Compresses a fighter graphics byte stream using the game's base/flag
/// format.
///
/// Tile data is normally a multiple of 32 bytes, so the encoder uses the
/// game's usual `0x0F` mask and emits two eight-byte chunks per base group.
/// A shorter valid multiple of eight uses a zero mask and one chunk per base
/// group.
pub fn compress_sprite_graphics_exact(data: &[u8]) -> Result<Vec<u8>, String> {
    if data.len() % 8 != 0 {
        return Err(format!(
            "Sprite graphics data must be aligned to eight-byte flag chunks ({} bytes)",
            data.len()
        ));
    }

    let mask = if data.len() % 16 == 0 {
        SPO_GRAPHICS_CONTINUATION_MASK
    } else {
        0
    };
    let chunks_per_group = if mask == 0 { 1 } else { 2 };
    let mut output = vec![mask];
    let mut cursor = 0usize;

    while cursor < data.len() {
        let base = data[cursor];
        output.push(base);

        for _ in 0..chunks_per_group {
            let chunk = &data[cursor..cursor + 8];
            let mut flags = 0u8;
            let mut literals = Vec::new();

            for (index, &byte) in chunk.iter().enumerate() {
                if byte != base {
                    flags |= 1 << (7 - index);
                    literals.push(byte);
                }
            }

            output.push(flags);
            output.extend(literals);
            cursor += 8;
        }
    }

    Ok(output)
}

/// Convenience wrapper for callers that already know their tile data is
/// correctly aligned.
pub fn compress_sprite_graphics(data: &[u8]) -> Vec<u8> {
    compress_sprite_graphics_exact(data).unwrap_or_default()
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
    /// Copy from previously decompressed data with each byte bit-reversed.
    RotatedLzCopy = 5,
    /// Copy from previously decompressed data in reverse order.
    ReversedLzCopy = 6,
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
            5 => Some(Self::RotatedLzCopy),
            6 => Some(Self::ReversedLzCopy),
            7 => Some(Self::LzCopy),
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
            Self::RotatedLzCopy => "Rotated LZ Copy",
            Self::ReversedLzCopy => "Reversed LZ Copy",
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

        let (cmd, len) = if (ctrl & COMMAND_MASK) == COMMAND_MASK {
            if pos >= data.len() {
                break;
            }
            let command = (ctrl >> 2) & 0x07;
            let length = (((ctrl & 0x03) as usize) << 8) | data[pos] as usize;
            pos += 1;
            stats.bytes_read += 1;
            (command, length + 1)
        } else {
            (ctrl >> COMMAND_SHIFT, ((ctrl & LENGTH_MASK) as usize) + 1)
        };

        *stats.commands_by_type.entry(cmd).or_insert(0) += 1;
        stats.command_count += 1;
        stats.bytes_written += if cmd == 2 { len * 2 } else { len };

        // Skip command data
        match cmd {
            0 => pos += len,   // Literals
            1 => pos += 1,     // Byte RLE
            2 => pos += 2,     // Word RLE
            3 => pos += 1,     // Incremental RLE
            4..=7 => pos += 2, // Backreferences
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
    fn test_decompress_rotated_and_reversed_backrefs() {
        // Literals: [0x01, 0x02, 0x04, 0x08]
        // Rotated backref of length 4 from offset 0.
        // Reversed backref of length 4 from offset 7.
        let data = vec![
            0x03, 0x01, 0x02, 0x04, 0x08, 0xA3, 0x00, 0x00, 0xC3, 0x00, 0x07, 0xFF,
        ];
        let mut decompressor = Decompressor::new(&data);
        let output = decompressor.decompress_pass_exact().unwrap();

        assert_eq!(&output[..4], &[0x01, 0x02, 0x04, 0x08]);
        assert_eq!(&output[4..8], &[0x80, 0x40, 0x20, 0x10]);
        assert_eq!(&output[8..12], &[0x10, 0x20, 0x40, 0x80]);
    }

    #[test]
    fn test_decompress_long_literal_command() {
        let mut data = vec![0xE0, 0x20]; // Long literal: 0x21 bytes.
        data.extend((0..=0x20).map(|value| value as u8));
        data.push(END_OF_STREAM);

        let mut decompressor = Decompressor::new(&data);
        let output = decompressor.decompress_pass_exact().unwrap();
        assert_eq!(output.len(), 0x21);
        assert_eq!(output[0], 0);
        assert_eq!(output[0x20], 0x20);
    }

    #[test]
    fn test_spo_sprite_graphics_roundtrip() {
        let data = (0..64)
            .map(|index| match index % 8 {
                0 | 1 | 7 => 0x30,
                2 => 0x63,
                3 => 0x67,
                4 => 0x64,
                5 => 0xAC,
                _ => 0xEB,
            })
            .collect::<Vec<_>>();

        let compressed = compress_sprite_graphics_exact(&data).unwrap();
        assert_eq!(compressed[0], SPO_GRAPHICS_CONTINUATION_MASK);

        let mut decompressor = Decompressor::new(&compressed);
        let decompressed = decompressor.decompress_sprite_graphics_exact().unwrap();

        assert_eq!(decompressed, data);
        assert_eq!(decompressor.position(), compressed.len());
    }

    #[test]
    fn test_spo_sprite_graphics_continuation_uses_output_offset() {
        let compressed = vec![
            0x0F, 0x30, 0x3F, 0x63, 0x63, 0x67, 0x64, 0xAC, 0xEB, 0xFF, 0xB8, 0xF7, 0xBF, 0xF3,
            0xBF, 0xE4, 0x5F, 0x7B,
        ];
        let expected = vec![
            0x30, 0x30, 0x63, 0x63, 0x67, 0x64, 0xAC, 0xEB, 0xB8, 0xF7, 0xBF, 0xF3, 0xBF, 0xE4,
            0x5F, 0x7B,
        ];

        let mut decompressor = Decompressor::new(&compressed);
        assert_eq!(
            decompressor.decompress_sprite_graphics_exact().unwrap(),
            expected
        );
        assert_eq!(decompressor.position(), compressed.len());
    }

    #[test]
    fn test_spo_sprite_graphics_rejects_partial_flag_chunk() {
        let error = compress_sprite_graphics_exact(&[0x00; 7]).unwrap_err();
        assert!(error.contains("eight-byte"));
    }

    #[test]
    fn test_command_type() {
        assert_eq!(CommandType::from_u8(0), Some(CommandType::Literals));
        assert_eq!(CommandType::from_u8(1), Some(CommandType::ByteRle));
        assert_eq!(CommandType::from_u8(4), Some(CommandType::LzCopy));
        assert_eq!(CommandType::from_u8(5), Some(CommandType::RotatedLzCopy));
        assert_eq!(CommandType::from_u8(6), Some(CommandType::ReversedLzCopy));
        assert_eq!(CommandType::from_u8(7), Some(CommandType::LzCopy));
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
