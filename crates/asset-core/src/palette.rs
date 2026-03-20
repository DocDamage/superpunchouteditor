//! Palette encoding and decoding for SNES BGR555 format.
//!
//! The SNES uses a 15-bit BGR555 color format where each color component
//! (blue, green, red) is stored as a 5-bit value. This module provides
//! conversion between SNES format and standard 8-bit RGB.
//!
//! ## SNES BGR555 Format
//! - Bits 0-4: Red (5 bits)
//! - Bits 5-9: Green (5 bits)
//! - Bits 10-14: Blue (5 bits)
//! - Bit 15: Unused
//!
//! ## Example
//! ```
//! use asset_core::{Color, decode_palette};
//!
//! // Decode SNES palette data
//! let snes_bytes = vec![0x00, 0x00, 0xFF, 0x03]; // Black + bright blue
//! let colors = decode_palette(&snes_bytes);
//!
//! // Create a color and convert to SNES format
//! let color = Color { r: 255, g: 0, b: 0 };
//! let snes_value = color.to_snes();
//! ```

use serde::{Deserialize, Serialize};

/// A single color in RGB format.
///
/// SNES uses 15-bit BGR555 color format internally, but this struct
/// uses standard 8-bit RGB for ease of use.
///
/// # Example
/// ```
/// use asset_core::Color;
///
/// let color = Color { r: 255, g: 0, b: 0 }; // Pure red
/// let snes_format = color.to_snes();
/// ```
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub struct Color {
    /// Red component (0-255)
    pub r: u8,
    /// Green component (0-255)
    pub g: u8,
    /// Blue component (0-255)
    pub b: u8,
}

impl Color {
    /// Creates a new color from RGB components.
    ///
    /// # Arguments
    /// - `r`: Red component (0-255)
    /// - `g`: Green component (0-255)
    /// - `b`: Blue component (0-255)
    ///
    /// # Example
    /// ```
    /// use asset_core::Color;
    ///
    /// let color = Color::new(255, 128, 0); // Orange
    /// ```
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Creates a color from a SNES BGR555 16-bit value.
    ///
    /// # Arguments
    /// - `word`: A 16-bit value in SNES BGR555 format
    ///
    /// # Format
    /// - Bits 0-4: Red (5 bits, scaled to 0-255)
    /// - Bits 5-9: Green (5 bits, scaled to 0-255)
    /// - Bits 10-14: Blue (5 bits, scaled to 0-255)
    ///
    /// # Example
    /// ```
    /// use asset_core::Color;
    ///
    /// // White in SNES format (all 5-bit components at max)
    /// let white = Color::from_snes(0x7FFF);
    /// assert_eq!(white.r, 248); // 31 << 3 = 248
    /// ```
    pub fn from_snes(word: u16) -> Self {
        let r = (word & 0x1F) as u8;
        let g = ((word >> 5) & 0x1F) as u8;
        let b = ((word >> 10) & 0x1F) as u8;

        // Scale 5-bit values (0-31) to 8-bit (0-255)
        // Using the formula: (value << 3) | (value >> 2)
        // This ensures 31 maps to 255 (not 248)
        Self {
            r: (r << 3) | (r >> 2),
            g: (g << 3) | (g >> 2),
            b: (b << 3) | (b >> 2),
        }
    }

    /// Converts this color to SNES BGR555 format.
    ///
    /// # Returns
    /// A 16-bit value in SNES BGR555 format.
    ///
    /// # Example
    /// ```
    /// use asset_core::Color;
    ///
    /// let color = Color { r: 255, g: 0, b: 0 }; // Pure red
    /// let snes = color.to_snes();
    /// // Red: 255 >> 3 = 31 (max 5-bit value)
    /// assert_eq!(snes & 0x1F, 31);
    /// ```
    pub fn to_snes(&self) -> u16 {
        // Convert 8-bit to 5-bit by shifting right 3 bits
        let r = (self.r >> 3) as u16;
        let g = (self.g >> 3) as u16;
        let b = (self.b >> 3) as u16;
        (r & 0x1F) | ((g & 0x1F) << 5) | ((b & 0x1F) << 10)
    }

    /// Returns this color as a hex string (e.g., "#FF0000").
    ///
    /// # Example
    /// ```
    /// use asset_core::Color;
    ///
    /// let color = Color { r: 255, g: 128, b: 0 };
    /// assert_eq!(color.to_hex(), "#FF8000");
    /// ```
    pub fn to_hex(&self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }

    /// Creates a color from a hex string.
    ///
    /// # Arguments
    /// - `hex`: A hex string like "#FF0000" or "FF0000"
    ///
    /// # Returns
    /// - `Some(Color)` if parsing succeeds
    /// - `None` if the string is invalid
    ///
    /// # Example
    /// ```
    /// use asset_core::Color;
    ///
    /// let color = Color::from_hex("#FF8000").unwrap();
    /// assert_eq!(color.r, 255);
    /// assert_eq!(color.g, 128);
    /// assert_eq!(color.b, 0);
    /// ```
    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 {
            return None;
        }
        let r = u8::from_str_radix(&hex[0..2], 8).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        Some(Self { r, g, b })
    }
}

impl Default for Color {
    fn default() -> Self {
        Self { r: 0, g: 0, b: 0 }
    }
}

/// Decodes a palette from raw SNES BGR555 bytes.
///
/// # Arguments
/// - `data`: Raw bytes in SNES format (2 bytes per color)
///
/// # Returns
/// A vector of `Color` structs in RGB format.
///
/// # Example
/// ```
/// use asset_core::{decode_palette, Color};
///
/// // Two colors: black (0x0000) and white (0x7FFF)
/// let bytes = vec![0x00, 0x00, 0xFF, 0x7F];
/// let palette = decode_palette(&bytes);
///
/// assert_eq!(palette.len(), 2);
/// assert_eq!(palette[0], Color::new(0, 0, 0));
/// assert_eq!(palette[1], Color::new(248, 248, 248));
/// ```
pub fn decode_palette(data: &[u8]) -> Vec<Color> {
    data.chunks_exact(2)
        .map(|chunk| {
            let word = u16::from_le_bytes([chunk[0], chunk[1]]);
            Color::from_snes(word)
        })
        .collect()
}

/// Encodes a palette to SNES BGR555 bytes.
///
/// # Arguments
/// - `colors`: A slice of `Color` structs
///
/// # Returns
/// A vector of bytes in SNES format (2 bytes per color)
///
/// # Example
/// ```
/// use asset_core::{encode_palette, Color};
///
/// let colors = vec![
///     Color::new(0, 0, 0),       // Black
///     Color::new(255, 255, 255), // White
/// ];
/// let bytes = encode_palette(&colors);
///
/// assert_eq!(bytes.len(), 4); // 2 colors × 2 bytes
/// ```
pub fn encode_palette(colors: &[Color]) -> Vec<u8> {
    let mut out = Vec::with_capacity(colors.len() * 2);
    for color in colors {
        let word = color.to_snes();
        out.extend_from_slice(&word.to_le_bytes());
    }
    out
}

/// Standard SNES palette size (16 colors × 2 bytes = 32 bytes)
pub const PALETTE_SIZE: usize = 32;

/// Standard number of colors in a SNES palette
pub const PALETTE_COLOR_COUNT: usize = 16;

/// Size of one color in bytes (2 bytes for BGR555)
pub const COLOR_SIZE_BYTES: usize = 2;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_from_snes() {
        // Black
        let black = Color::from_snes(0x0000);
        assert_eq!(black.r, 0);
        assert_eq!(black.g, 0);
        assert_eq!(black.b, 0);

        // White (all 5-bit components at max)
        let white = Color::from_snes(0x7FFF);
        assert_eq!(white.r, 248); // 31 << 3 = 248
        assert_eq!(white.g, 248);
        assert_eq!(white.b, 248);
    }

    #[test]
    fn test_color_to_snes() {
        let black = Color::new(0, 0, 0);
        assert_eq!(black.to_snes(), 0x0000);

        let white = Color::new(255, 255, 255);
        assert_eq!(white.to_snes(), 0x7FFF);
    }

    #[test]
    fn test_roundtrip() {
        // Test that encoding and decoding preserves color values
        let original = Color::new(128, 64, 200);
        let snes = original.to_snes();
        let decoded = Color::from_snes(snes);

        // Due to 5-bit quantization, values may differ slightly
        assert!((original.r as i16 - decoded.r as i16).abs() <= 8);
        assert!((original.g as i16 - decoded.g as i16).abs() <= 8);
        assert!((original.b as i16 - decoded.b as i16).abs() <= 8);
    }

    #[test]
    fn test_decode_palette() {
        let bytes = vec![0x00, 0x00, 0xFF, 0x7F];
        let palette = decode_palette(&bytes);
        assert_eq!(palette.len(), 2);
    }

    #[test]
    fn test_encode_palette() {
        let colors = vec![Color::new(0, 0, 0), Color::new(255, 255, 255)];
        let bytes = encode_palette(&colors);
        assert_eq!(bytes.len(), 4);
    }

    #[test]
    fn test_hex_conversion() {
        let color = Color::new(255, 128, 0);
        assert_eq!(color.to_hex(), "#FF8000");

        let parsed = Color::from_hex("#FF8000").unwrap();
        assert_eq!(parsed.r, 255);
        assert_eq!(parsed.g, 128);
        assert_eq!(parsed.b, 0);
    }
}
