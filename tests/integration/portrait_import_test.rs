//! Integration tests for portrait import functionality
//!
//! Tests verify that when a user imports a portrait PNG and saves the ROM,
//! the pixel data is correctly encoded and stored.

use tempfile::TempDir;

mod common;
use common::portrait_utils::{
    calculate_4bpp_size, calculate_tile_count, create_test_icon, create_test_large_portrait,
    create_test_portrait, get_test_palette_colors, roundtrip_test, TestPattern,
};

// Import core modules
use asset_core::{
    decode_4bpp_sheet, decode_palette, encode_4bpp_sheet, encode_palette, image_to_tiles,
    tiles_to_image, Color,
};
use rom_core::Rom;

/// Test 1: Small Icon Import (32x32 = 4x4 tiles)
///
/// Creates a 32x32 test image with known pattern, converts to 4bpp tiles,
/// imports to ROM, saves, reloads, and verifies round-trip integrity.
#[test]
fn test_small_icon_import() {
    // Setup: Create test ROM
    let rom_data = vec![0u8; rom_core::EXPECTED_SIZE];
    let mut rom = Rom::new(rom_data);

    // Create test icon with checkerboard pattern
    let pattern = TestPattern::Checkerboard {
        color1: 1,
        color2: 7,
    }; // Red and white
    let original_img = create_test_icon(pattern);

    // Get test palette
    let palette = get_test_palette_colors();

    // Verify roundtrip before ROM operations
    roundtrip_test(&original_img, &palette)
        .expect("Round-trip test should pass before ROM operations");

    // Convert image to 4bpp tiles
    let tiles = image_to_tiles(&original_img, &palette);
    let expected_tile_count = calculate_tile_count(32, 32);
    assert_eq!(
        tiles.len(),
        expected_tile_count,
        "Should have {} tiles for 32x32 image",
        expected_tile_count
    );

    // Encode tiles to 4bpp binary data
    let tile_data = encode_4bpp_sheet(&tiles);
    let expected_data_size = calculate_4bpp_size(32, 32);
    assert_eq!(
        tile_data.len(),
        expected_data_size,
        "4bpp data should be {} bytes",
        expected_data_size
    );

    // Use a fixed offset in the ROM for testing (icon-sized slot)
    let portrait_offset = 0x10000; // 64KB into ROM, safely in empty space

    // Write tiles to ROM
    rom.write_bytes(portrait_offset, &tile_data)
        .expect("Should write tile data to ROM");

    // Write palette after the tiles (16 colors * 2 bytes = 32 bytes)
    let palette_offset = portrait_offset + tile_data.len();
    let palette_data = encode_palette(&palette[..16]);
    rom.write_bytes(palette_offset, &palette_data)
        .expect("Should write palette to ROM");

    // Save ROM to temp file
    let temp_dir = TempDir::new().expect("Should create temp directory");
    let rom_path = temp_dir.path().join("test_icon_import.sfc");
    rom.save(&rom_path).expect("Should save ROM");

    // Load saved ROM
    let loaded_rom = Rom::load(&rom_path).expect("Should load saved ROM");

    // Read bytes at portrait offset
    let read_tile_data = loaded_rom
        .read_bytes(portrait_offset, tile_data.len())
        .expect("Should read tile data from ROM");

    // Verify tile data matches
    assert_eq!(
        read_tile_data,
        &tile_data[..],
        "Tile data read from ROM should match original"
    );

    // Decode back to image
    let decoded_tiles = decode_4bpp_sheet(read_tile_data);
    let reconstructed = tiles_to_image(&decoded_tiles, 4, &palette); // 4 tiles wide

    // Verify dimensions
    assert_eq!(
        reconstructed.dimensions(),
        original_img.dimensions(),
        "Reconstructed image dimensions should match original"
    );

    // Verify pixels match original
    for y in 0..32 {
        for x in 0..32 {
            let orig = original_img.get_pixel(x, y);
            let recon = reconstructed.get_pixel(x, y);

            // Both should be non-transparent and match palette indices
            let orig_idx = if orig[3] == 0 {
                0
            } else {
                palette
                    .iter()
                    .position(|c| c.r == orig[0] && c.g == orig[1] && c.b == orig[2])
                    .unwrap_or(0) as u8
            };
            let recon_idx = if recon[3] == 0 {
                0
            } else {
                palette
                    .iter()
                    .position(|c| c.r == recon[0] && c.g == recon[1] && c.b == recon[2])
                    .unwrap_or(0) as u8
            };

            assert_eq!(
                orig_idx, recon_idx,
                "Pixel mismatch at ({}, {}): original index {} vs reconstructed index {}",
                x, y, orig_idx, recon_idx
            );
        }
    }

    // Cleanup happens automatically when temp_dir is dropped
    println!("Test 1: Small Icon Import - PASSED");
}

/// Test 2: Large Portrait Import (128x128 = 16x16 tiles)
///
/// Creates a 128x128 test image, converts to 4bpp tiles,
/// imports to ROM at portrait offset, saves, reloads, and verifies.
#[test]
fn test_large_portrait_import() {
    // Setup: Create test ROM
    let rom_data = vec![0u8; rom_core::EXPECTED_SIZE];
    let mut rom = Rom::new(rom_data);

    // Create test large portrait with gradient pattern
    let pattern = TestPattern::Gradient;
    let original_img = create_test_large_portrait(pattern);

    // Get test palette
    let palette = get_test_palette_colors();

    // Verify roundtrip before ROM operations
    roundtrip_test(&original_img, &palette)
        .expect("Round-trip test should pass before ROM operations");

    // Convert image to 4bpp tiles
    let tiles = image_to_tiles(&original_img, &palette);
    let expected_tile_count = calculate_tile_count(128, 128);
    assert_eq!(
        tiles.len(),
        expected_tile_count,
        "Should have {} tiles for 128x128 image",
        expected_tile_count
    );

    // Encode tiles to 4bpp binary data
    let tile_data = encode_4bpp_sheet(&tiles);
    let expected_data_size = calculate_4bpp_size(128, 128);
    assert_eq!(
        tile_data.len(),
        expected_data_size,
        "4bpp data should be {} bytes",
        expected_data_size
    );

    // Use a fixed offset in the ROM for testing
    let portrait_offset = 0x20000; // 128KB into ROM

    // Write tiles to ROM
    rom.write_bytes(portrait_offset, &tile_data)
        .expect("Should write tile data to ROM");

    // Save ROM to temp file
    let temp_dir = TempDir::new().expect("Should create temp directory");
    let rom_path = temp_dir.path().join("test_large_portrait_import.sfc");
    rom.save(&rom_path).expect("Should save ROM");

    // Load saved ROM
    let loaded_rom = Rom::load(&rom_path).expect("Should load saved ROM");

    // Read bytes at portrait offset
    let read_tile_data = loaded_rom
        .read_bytes(portrait_offset, tile_data.len())
        .expect("Should read tile data from ROM");

    // Verify tile data matches exactly
    assert_eq!(
        read_tile_data,
        &tile_data[..],
        "Tile data read from ROM should match original"
    );

    // Decode back to tiles and image
    let decoded_tiles = decode_4bpp_sheet(read_tile_data);
    let reconstructed = tiles_to_image(&decoded_tiles, 16, &palette); // 16 tiles wide

    // Verify dimensions
    assert_eq!(
        reconstructed.dimensions(),
        original_img.dimensions(),
        "Reconstructed image dimensions should match original"
    );

    // Sample pixel verification (check corners and center)
    let check_points = vec![(0, 0), (63, 63), (127, 0), (0, 127), (127, 127)];

    for (x, y) in check_points {
        let orig = original_img.get_pixel(x, y);
        let recon = reconstructed.get_pixel(x, y);

        // Find palette indices
        let orig_idx = if orig[3] == 0 {
            0
        } else {
            palette
                .iter()
                .position(|c| c.r == orig[0] && c.g == orig[1] && c.b == orig[2])
                .unwrap_or(0) as u8
        };
        let recon_idx = if recon[3] == 0 {
            0
        } else {
            palette
                .iter()
                .position(|c| c.r == recon[0] && c.g == recon[1] && c.b == recon[2])
                .unwrap_or(0) as u8
        };

        assert_eq!(
            orig_idx, recon_idx,
            "Pixel mismatch at ({}, {}): original index {} vs reconstructed index {}",
            x, y, orig_idx, recon_idx
        );
    }

    println!("Test 2: Large Portrait Import - PASSED");
}

/// Test 3: Import with Custom Palette
///
/// Creates image with specific colors, quantizes to palette,
/// imports to ROM, and verifies palette indices are preserved.
#[test]
fn test_import_with_custom_palette() {
    // Setup: Create test ROM
    let rom_data = vec![0u8; rom_core::EXPECTED_SIZE];
    let mut rom = Rom::new(rom_data);

    // Define custom test palette with specific colors
    let custom_palette: Vec<Color> = vec![
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
    ];

    // Create test image with border pattern using indices 1-7
    let pattern = TestPattern::Border { outer: 1, inner: 7 }; // Red border, White center
    let original_img = create_test_portrait(64, 64, pattern);

    // Convert image to 4bpp tiles (this quantizes to palette)
    let tiles = image_to_tiles(&original_img, &custom_palette);

    // Verify that specific palette indices were preserved
    // Check first tile (should be red border)
    let first_tile = &tiles[0];
    let red_pixels = first_tile.pixels.iter().filter(|&&p| p == 1).count();
    let _white_pixels = first_tile.pixels.iter().filter(|&&p| p == 7).count();

    // The first tile (top-left corner) should have some red pixels
    assert!(
        red_pixels > 0,
        "First tile should contain red pixels (index 1)"
    );

    // Check a middle tile (should be mostly white)
    let middle_tile_idx = 4 * 4 + 4; // Center-ish tile in 8x8 grid
    if middle_tile_idx < tiles.len() {
        let middle_tile = &tiles[middle_tile_idx];
        let white_count = middle_tile.pixels.iter().filter(|&&p| p == 7).count();
        assert!(
            white_count > 0,
            "Middle tile should contain white pixels (index 7)"
        );
    }

    // Encode tiles
    let tile_data = encode_4bpp_sheet(&tiles);

    // Write to ROM
    let portrait_offset = 0x30000;
    rom.write_bytes(portrait_offset, &tile_data)
        .expect("Should write tile data to ROM");

    // Write palette to ROM
    let palette_offset = portrait_offset + tile_data.len();
    let palette_data = encode_palette(&custom_palette);
    rom.write_bytes(palette_offset, &palette_data)
        .expect("Should write palette to ROM");

    // Save and reload
    let temp_dir = TempDir::new().expect("Should create temp directory");
    let rom_path = temp_dir.path().join("test_custom_palette.sfc");
    rom.save(&rom_path).expect("Should save ROM");

    let loaded_rom = Rom::load(&rom_path).expect("Should load ROM");

    // Read and verify palette data
    let read_palette_data = loaded_rom
        .read_bytes(palette_offset, palette_data.len())
        .expect("Should read palette from ROM");
    assert_eq!(
        read_palette_data,
        &palette_data[..],
        "Palette data should match"
    );

    // Decode palette and verify colors
    let decoded_palette = decode_palette(read_palette_data);
    assert_eq!(decoded_palette.len(), 16, "Should have 16 palette colors");

    // Verify specific colors are preserved
    assert_eq!(decoded_palette[1].r, 255, "Red color should be preserved");
    assert_eq!(decoded_palette[1].g, 0);
    assert_eq!(decoded_palette[1].b, 0);

    assert_eq!(decoded_palette[7].r, 255, "White color should be preserved");
    assert_eq!(decoded_palette[7].g, 255);
    assert_eq!(decoded_palette[7].b, 255);

    // Read and verify tile data preserves palette indices
    let read_tile_data = loaded_rom
        .read_bytes(portrait_offset, tile_data.len())
        .expect("Should read tile data");
    let decoded_tiles = decode_4bpp_sheet(read_tile_data);

    // Verify first tile still has red pixels
    let first_decoded = &decoded_tiles[0];
    let decoded_red_count = first_decoded.pixels.iter().filter(|&&p| p == 1).count();
    assert_eq!(
        decoded_red_count, red_pixels,
        "Red pixel count should match after round-trip"
    );

    println!("Test 3: Import with Custom Palette - PASSED");
}

/// Test 4: Multiple Portrait Import
///
/// Tests importing multiple portraits to different ROM locations.
#[test]
fn test_multiple_portrait_import() {
    // Setup: Create test ROM
    let rom_data = vec![0u8; rom_core::EXPECTED_SIZE];
    let mut rom = Rom::new(rom_data);

    let palette = get_test_palette_colors();

    // Define multiple portrait locations
    let portrait_configs = vec![
        (
            0x40000,
            32,
            32,
            TestPattern::Checkerboard {
                color1: 1,
                color2: 2,
            },
        ),
        (
            0x41000,
            32,
            32,
            TestPattern::HorizontalStripes {
                color1: 3,
                color2: 4,
                stripe_height: 4,
            },
        ),
        (
            0x42000,
            64,
            64,
            TestPattern::VerticalStripes {
                color1: 5,
                color2: 6,
                stripe_width: 8,
            },
        ),
    ];

    // Import each portrait
    for (offset, width, height, pattern) in &portrait_configs {
        let img = create_test_portrait(*width, *height, *pattern);
        let tiles = image_to_tiles(&img, &palette);
        let tile_data = encode_4bpp_sheet(&tiles);

        rom.write_bytes(*offset, &tile_data)
            .expect(&format!("Should write portrait at offset 0x{:X}", offset));
    }

    // Save and reload
    let temp_dir = TempDir::new().expect("Should create temp directory");
    let rom_path = temp_dir.path().join("test_multiple_portraits.sfc");
    rom.save(&rom_path).expect("Should save ROM");

    let loaded_rom = Rom::load(&rom_path).expect("Should load ROM");

    // Verify each portrait
    for (offset, width, height, pattern) in &portrait_configs {
        let expected_size = calculate_4bpp_size(*width, *height);
        let read_data = loaded_rom
            .read_bytes(*offset, expected_size)
            .expect(&format!("Should read portrait at offset 0x{:X}", offset));

        // Decode and verify dimensions
        let tiles = decode_4bpp_sheet(read_data);
        let expected_tiles = calculate_tile_count(*width, *height);
        assert_eq!(
            tiles.len(),
            expected_tiles,
            "Portrait at 0x{:X} should have {} tiles",
            offset,
            expected_tiles
        );

        println!(
            "Verified portrait at 0x{:X}: {}x{} with {:?}",
            offset, width, height, pattern
        );
    }

    println!("Test 4: Multiple Portrait Import - PASSED");
}

/// Test 5: Portrait at Edge of ROM
///
/// Tests importing a portrait near the end of the ROM.
#[test]
fn test_portrait_at_rom_edge() {
    // Setup: Create test ROM
    let rom_data = vec![0u8; rom_core::EXPECTED_SIZE];
    let mut rom = Rom::new(rom_data);

    let palette = get_test_palette_colors();
    let img = create_test_icon(TestPattern::Solid { color: 3 });
    let tiles = image_to_tiles(&img, &palette);
    let tile_data = encode_4bpp_sheet(&tiles);

    // Place portrait near end of ROM (but with enough space)
    let portrait_offset = rom_core::EXPECTED_SIZE - tile_data.len() - 256;

    rom.write_bytes(portrait_offset, &tile_data)
        .expect("Should write portrait near ROM edge");

    // Save and verify
    let temp_dir = TempDir::new().expect("Should create temp directory");
    let rom_path = temp_dir.path().join("test_edge_portrait.sfc");
    rom.save(&rom_path).expect("Should save ROM");

    let loaded_rom = Rom::load(&rom_path).expect("Should load ROM");
    let read_data = loaded_rom
        .read_bytes(portrait_offset, tile_data.len())
        .expect("Should read portrait from edge");

    assert_eq!(
        read_data,
        &tile_data[..],
        "Portrait at ROM edge should match"
    );

    println!("Test 5: Portrait at ROM Edge - PASSED");
}
