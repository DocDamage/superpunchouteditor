//! Test utilities for palette editing tests
//!
//! Provides helpers for creating test palettes, SNES color conversion,
//! and ROM manipulation for palette testing.

use asset_core::Color;

/// Standard SNES palette size (16 colors)
pub const PALETTE_SIZE: usize = 16;

/// Size of one palette entry in bytes (2 bytes per color)
pub const PALETTE_ENTRY_SIZE: usize = 2;

/// Size of a full palette in bytes (16 colors * 2 bytes)
pub const FULL_PALETTE_BYTES: usize = PALETTE_SIZE * PALETTE_ENTRY_SIZE;

/// SNES bank size for LoROM
pub const SNES_BANK_SIZE: usize = 0x8000;

/// Test palette offset locations in mock ROM
pub const TEST_PALETTE_OFFSET_1: usize = 0x10000;
pub const TEST_PALETTE_OFFSET_2: usize = 0x10020;
pub const TEST_PALETTE_OFFSET_3: usize = 0x10040;

/// Test palette near a bank boundary (at bank 2 end - 32 bytes)
pub const TEST_PALETTE_BANK_BOUNDARY: usize = SNES_BANK_SIZE * 2 - FULL_PALETTE_BYTES;

/// Creates a standard test palette with 16 colors
pub fn create_test_palette() -> Vec<Color> {
    vec![
        Color { r: 0, g: 0, b: 0 },   // 0: Black (transparent)
        Color { r: 255, g: 0, b: 0 }, // 1: Pure Red
        Color { r: 0, g: 255, b: 0 }, // 2: Pure Green
        Color { r: 0, g: 0, b: 255 }, // 3: Pure Blue
        Color {
            r: 255,
            g: 255,
            b: 0,
        }, // 4: Yellow
        Color {
            r: 255,
            g: 0,
            b: 255,
        }, // 5: Magenta
        Color {
            r: 0,
            g: 255,
            b: 255,
        }, // 6: Cyan
        Color {
            r: 255,
            g: 255,
            b: 255,
        }, // 7: White
        Color { r: 128, g: 0, b: 0 }, // 8: Dark Red
        Color { r: 0, g: 128, b: 0 }, // 9: Dark Green
        Color { r: 0, g: 0, b: 128 }, // 10: Dark Blue
        Color {
            r: 128,
            g: 128,
            b: 0,
        }, // 11: Olive
        Color {
            r: 128,
            g: 0,
            b: 128,
        }, // 12: Purple
        Color {
            r: 0,
            g: 128,
            b: 128,
        }, // 13: Teal
        Color {
            r: 192,
            g: 192,
            b: 192,
        }, // 14: Light Gray
        Color {
            r: 128,
            g: 128,
            b: 128,
        }, // 15: Gray
    ]
}

/// Creates a palette with all unique colors for testing full replacement
pub fn create_full_test_palette() -> Vec<Color> {
    vec![
        Color { r: 0, g: 0, b: 0 }, // 0: Black
        Color {
            r: 255,
            g: 64,
            b: 0,
        }, // 1: Orange-Red (different from test palette)
        Color { r: 0, g: 255, b: 0 }, // 2: Green
        Color { r: 0, g: 0, b: 255 }, // 3: Blue
        Color {
            r: 255,
            g: 255,
            b: 0,
        }, // 4: Yellow
        Color {
            r: 255,
            g: 0,
            b: 255,
        }, // 5: Magenta
        Color {
            r: 0,
            g: 255,
            b: 255,
        }, // 6: Cyan
        Color {
            r: 255,
            g: 255,
            b: 255,
        }, // 7: White
        Color {
            r: 255,
            g: 128,
            b: 0,
        }, // 8: Orange
        Color {
            r: 128,
            g: 255,
            b: 0,
        }, // 9: Lime
        Color {
            r: 0,
            g: 255,
            b: 128,
        }, // 10: Spring
        Color {
            r: 0,
            g: 128,
            b: 255,
        }, // 11: Azure
        Color {
            r: 128,
            g: 0,
            b: 255,
        }, // 12: Violet
        Color {
            r: 255,
            g: 0,
            b: 128,
        }, // 13: Rose
        Color {
            r: 255,
            g: 128,
            b: 128,
        }, // 14: Pink
        Color {
            r: 128,
            g: 128,
            b: 255,
        }, // 15: Periwinkle
    ]
}

/// Creates an alternative palette for testing multiple edits
pub fn create_alternative_palette() -> Vec<Color> {
    vec![
        Color { r: 0, g: 0, b: 0 }, // 0: Black
        Color {
            r: 255,
            g: 64,
            b: 64,
        }, // 1: Light Red
        Color {
            r: 64,
            g: 255,
            b: 64,
        }, // 2: Light Green
        Color {
            r: 64,
            g: 64,
            b: 255,
        }, // 3: Light Blue
        Color {
            r: 255,
            g: 255,
            b: 64,
        }, // 4: Light Yellow
        Color {
            r: 255,
            g: 64,
            b: 255,
        }, // 5: Light Magenta
        Color {
            r: 64,
            g: 255,
            b: 255,
        }, // 6: Light Cyan
        Color {
            r: 224,
            g: 224,
            b: 224,
        }, // 7: Off-White
        Color {
            r: 160,
            g: 32,
            b: 32,
        }, // 8: Maroon
        Color {
            r: 32,
            g: 160,
            b: 32,
        }, // 9: Forest
        Color {
            r: 32,
            g: 32,
            b: 160,
        }, // 10: Navy
        Color {
            r: 160,
            g: 160,
            b: 32,
        }, // 11: Gold
        Color {
            r: 160,
            g: 32,
            b: 160,
        }, // 12: Plum
        Color {
            r: 32,
            g: 160,
            b: 160,
        }, // 13: Sea Green
        Color {
            r: 208,
            g: 208,
            b: 208,
        }, // 14: Silver
        Color {
            r: 96,
            g: 96,
            b: 96,
        }, // 15: Dark Gray
    ]
}

/// Creates a mock SNES ROM with valid header and test palettes
///
/// Returns a 2MB ROM with:
/// - Valid SNES header at 0x7FC0
/// - Test palettes at predefined locations
pub fn create_mock_rom_with_palettes() -> rom_core::Rom {
    let mut data = vec![0u8; rom_core::EXPECTED_SIZE];

    // Write SNES header (simplified - just the essential parts)
    // Header is located at 0x7FC0-0x7FFF in LoROM
    let header_offset = 0x7FC0;

    // Game title (21 bytes)
    let title = b"SUPER PUNCH-OUT!!  ";
    data[header_offset..header_offset + title.len()].copy_from_slice(title);

    // Map mode and cartridge type
    data[header_offset + 21] = 0x20; // Map mode (LoROM, FastROM)
    data[header_offset + 22] = 0x05; // Cartridge type (ROM + RAM + Battery)
    data[header_offset + 23] = 0x0A; // ROM size (2MB = 10)
    data[header_offset + 24] = 0x05; // RAM size (32KB = 5)

    // Write test palettes at predefined locations
    let palette1 = create_test_palette();
    let encoded1 = asset_core::encode_palette(&palette1);
    data[TEST_PALETTE_OFFSET_1..TEST_PALETTE_OFFSET_1 + encoded1.len()].copy_from_slice(&encoded1);

    let palette2 = create_alternative_palette();
    let encoded2 = asset_core::encode_palette(&palette2);
    data[TEST_PALETTE_OFFSET_2..TEST_PALETTE_OFFSET_2 + encoded2.len()].copy_from_slice(&encoded2);

    // Write palette at bank boundary
    let encoded3 = asset_core::encode_palette(&palette1);
    data[TEST_PALETTE_BANK_BOUNDARY..TEST_PALETTE_BANK_BOUNDARY + encoded3.len()]
        .copy_from_slice(&encoded3);

    rom_core::Rom::new(data)
}

/// Read a palette from ROM at the specified offset
pub fn read_palette_from_rom(rom: &rom_core::Rom, offset: usize) -> Vec<Color> {
    let palette_data = rom
        .read_bytes(offset, FULL_PALETTE_BYTES)
        .expect("Failed to read palette from ROM");
    asset_core::decode_palette(palette_data)
}

/// Write a palette to ROM at the specified offset
pub fn write_palette_to_rom(rom: &mut rom_core::Rom, offset: usize, palette: &[Color]) {
    let encoded = asset_core::encode_palette(palette);
    rom.write_bytes(offset, &encoded)
        .expect("Failed to write palette to ROM");
}

/// Modify a single color in a palette
pub fn modify_palette_color(palette: &mut [Color], index: usize, new_color: Color) {
    assert!(index < palette.len(), "Palette index out of bounds");
    palette[index] = new_color;
}

/// Verify that two palettes are identical after SNES conversion
/// Uses bit replication to match SNES color precision
pub fn assert_palettes_equal(actual: &[Color], expected: &[Color]) {
    assert_eq!(
        actual.len(),
        expected.len(),
        "Palette length mismatch: {} vs {}",
        actual.len(),
        expected.len()
    );

    for (i, (a, e)) in actual.iter().zip(expected.iter()).enumerate() {
        // Apply SNES bit replication to expected values
        let expected_r = expand_5bit_to_8bit(e.r);
        let expected_g = expand_5bit_to_8bit(e.g);
        let expected_b = expand_5bit_to_8bit(e.b);

        assert_eq!(
            (a.r, a.g, a.b),
            (expected_r, expected_g, expected_b),
            "Color mismatch at index {}: ({}, {}, {}) vs ({}, {}, {})",
            i,
            a.r,
            a.g,
            a.b,
            expected_r,
            expected_g,
            expected_b
        );
    }
}

/// Convert an 8-bit value to 5-bit and back using SNES bit replication
/// This matches the formula in Color::from_snes: (val << 3) | (val >> 2)
pub fn expand_5bit_to_8bit(val: u8) -> u8 {
    let five_bit = (val >> 3) & 0x1F;
    (five_bit << 3) | (five_bit >> 2)
}

/// Verify RGB -> SNES -> RGB roundtrip conversion
pub fn verify_color_roundtrip(color: &Color) -> bool {
    let snes = color.to_snes();
    let roundtrip = Color::from_snes(snes);

    // SNES uses 5-bit per channel with bit replication
    // We need to check against the expected expanded value
    let expected_r = expand_5bit_to_8bit(color.r);
    let expected_g = expand_5bit_to_8bit(color.g);
    let expected_b = expand_5bit_to_8bit(color.b);

    roundtrip.r == expected_r && roundtrip.g == expected_g && roundtrip.b == expected_b
}

/// Get the bank number for a given offset
pub fn get_bank_for_offset(offset: usize) -> u8 {
    ((offset / SNES_BANK_SIZE) | 0x80) as u8
}

/// Check if offset is at a bank boundary
pub fn is_at_bank_boundary(offset: usize) -> bool {
    offset % SNES_BANK_SIZE == 0
}

/// Get the remaining bytes in the current bank from the given offset
pub fn bytes_remaining_in_bank(offset: usize) -> usize {
    SNES_BANK_SIZE - (offset % SNES_BANK_SIZE)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_test_palette() {
        let palette = create_test_palette();
        assert_eq!(palette.len(), PALETTE_SIZE);
    }

    #[test]
    fn test_color_roundtrip() {
        let color = Color {
            r: 255,
            g: 128,
            b: 64,
        };
        assert!(verify_color_roundtrip(&color));
    }

    #[test]
    fn test_palette_encoding_size() {
        let palette = create_test_palette();
        let encoded = asset_core::encode_palette(&palette);
        assert_eq!(encoded.len(), FULL_PALETTE_BYTES);
    }

    #[test]
    fn test_bank_boundary_calculations() {
        assert_eq!(get_bank_for_offset(0), 0x80);
        assert_eq!(get_bank_for_offset(SNES_BANK_SIZE), 0x81);
        assert!(is_at_bank_boundary(0));
        assert!(is_at_bank_boundary(SNES_BANK_SIZE));
        assert!(!is_at_bank_boundary(100));
        assert_eq!(bytes_remaining_in_bank(SNES_BANK_SIZE - 10), 10);
    }

    #[test]
    fn test_modify_palette_color() {
        let mut palette = create_test_palette();
        let new_color = Color {
            r: 100,
            g: 150,
            b: 200,
        };
        modify_palette_color(&mut palette, 5, new_color.clone());
        assert_eq!(palette[5].r, new_color.r);
        assert_eq!(palette[5].g, new_color.g);
        assert_eq!(palette[5].b, new_color.b);
    }

    #[test]
    fn test_palette_equality() {
        let p1 = create_test_palette();
        // Simulate SNES encoding/decoding roundtrip for p1
        let encoded = asset_core::encode_palette(&p1);
        let p1_roundtrip = asset_core::decode_palette(&encoded);

        let p2 = create_test_palette();
        assert_palettes_equal(&p1_roundtrip, &p2);
    }
}
