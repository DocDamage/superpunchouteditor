//! Fighter Graphics Bin Editing Integration Tests
//!
//! Tests the complete workflow of editing fighter sprite bins:
//! - Reading original sprite data
//! - Decoding 4bpp tiles
//! - Modifying tiles
//! - Encoding back to bytes
//! - Writing to ROM
//! - Verifying persistence after save/reload
//!
//! Covers both uncompressed and compressed (HAL8) bins.

use std::fs;
use std::path::PathBuf;

use asset_core::{decode_4bpp_sheet, encode_4bpp_sheet, Decompressor, Tile};
use rom_core::Rom;

// ============================================================================
// Sprite Utility Functions (inline for integration test)
// ============================================================================

/// Creates a tile with an alternating checkerboard pattern
fn create_checkerboard_tile(color1: u8, color2: u8) -> Tile {
    let mut pixels = vec![0u8; 64];
    for y in 0..8 {
        for x in 0..8 {
            let idx = y * 8 + x;
            pixels[idx] = if (x + y) % 2 == 0 { color1 } else { color2 };
        }
    }
    Tile { pixels }
}

/// Creates a tile with horizontal stripes
fn _create_horizontal_stripes_tile(color1: u8, color2: u8) -> Tile {
    let mut pixels = vec![0u8; 64];
    for y in 0..8 {
        for x in 0..8 {
            let idx = y * 8 + x;
            pixels[idx] = if y % 2 == 0 { color1 } else { color2 };
        }
    }
    Tile { pixels }
}

/// Creates a tile with vertical stripes
fn _create_vertical_stripes_tile(color1: u8, color2: u8) -> Tile {
    let mut pixels = vec![0u8; 64];
    for y in 0..8 {
        for x in 0..8 {
            let idx = y * 8 + x;
            pixels[idx] = if x % 2 == 0 { color1 } else { color2 };
        }
    }
    Tile { pixels }
}

/// Creates a solid color tile
fn create_solid_tile(color: u8) -> Tile {
    Tile {
        pixels: vec![color & 0x0F; 64],
    }
}

/// Creates a tile with a specific test pattern (corners marked)
fn create_test_pattern_tile() -> Tile {
    let mut pixels = vec![0u8; 64];
    // Mark corners with specific colors
    pixels[0] = 1; // Top-left
    pixels[7] = 2; // Top-right
    pixels[56] = 3; // Bottom-left
    pixels[63] = 4; // Bottom-right
                    // Mark center
    pixels[27] = 5;
    pixels[28] = 5;
    pixels[35] = 5;
    pixels[36] = 5;
    Tile { pixels }
}

/// Modifies a tile by applying a transformation
fn modify_tile(tile: &mut Tile, transform: impl Fn(usize, u8) -> u8) {
    for (idx, pixel) in tile.pixels.iter_mut().enumerate() {
        *pixel = transform(idx, *pixel) & 0x0F;
    }
}

/// Generates a test sprite sheet with multiple tiles
fn generate_test_sprite_sheet(num_tiles: usize) -> Vec<Tile> {
    let mut tiles = Vec::with_capacity(num_tiles);

    for i in 0..num_tiles {
        let tile = match i % 4 {
            0 => create_checkerboard_tile(1, 2),
            1 => create_solid_tile(3),
            2 => create_checkerboard_tile(4, 5),
            3 => create_test_pattern_tile(),
            _ => create_solid_tile(7),
        };
        tiles.push(tile);
    }

    tiles
}

/// HAL8-style compression encoder for testing
fn _hal8_compress_simple(data: &[u8]) -> Vec<u8> {
    let mut output = Vec::new();

    // Process in chunks of 32 bytes (one tile)
    for chunk in data.chunks(32) {
        let len = chunk.len().min(32);
        output.push((len - 1) as u8); // cmd 0, length-1
        output.extend(&chunk[..len]);
    }

    output.push(0xFF); // End marker
    output
}

/// Creates a more efficient HAL8 compressed block using RLE when possible
fn hal8_compress_with_rle(data: &[u8]) -> Vec<u8> {
    let mut output = Vec::new();
    let mut i = 0;

    while i < data.len() {
        // Check for run of identical bytes
        let mut run_len = 1;
        while i + run_len < data.len() && run_len < 32 && data[i + run_len] == data[i] {
            run_len += 1;
        }

        if run_len >= 4 {
            // Use byte RLE command (cmd 1): 0x20 | (len - 1)
            output.push(0x20 | ((run_len - 1) as u8));
            output.push(data[i]);
            i += run_len;
        } else {
            // Use literal command (cmd 0)
            let lit_len = (32 - run_len + 1).min(data.len() - i).min(32);
            output.push((lit_len - 1) as u8);
            output.extend(&data[i..i + lit_len]);
            i += lit_len;
        }
    }

    output.push(0xFF);
    output
}

/// Verifies that two tiles have identical pixel data
fn tiles_equal(a: &Tile, b: &Tile) -> bool {
    a.pixels == b.pixels
}

/// Gets the color at a specific position in a tile
fn pixel_at(tile: &Tile, x: usize, y: usize) -> Option<u8> {
    if x < 8 && y < 8 {
        tile.pixels.get(y * 8 + x).copied()
    } else {
        None
    }
}

// ============================================================================
// Test Configuration
// ============================================================================

/// PC offset for a mock fighter graphics bin (uncompressed)
const TEST_UNCOMPRESSED_OFFSET: usize = 0x200000; // 2MB mark (in expanded region)
const TEST_COMPRESSED_OFFSET: usize = 0x210000;
const TEST_SECOND_BIN_OFFSET: usize = 0x220000;

/// Number of tiles in a test sprite bin
const TEST_TILE_COUNT: usize = 16;

/// ROM size for tests (4MB expanded)
const TEST_ROM_SIZE: usize = 4 * 1024 * 1024;

// ============================================================================
// Helper Functions
// ============================================================================

/// Creates a test ROM with expanded size
fn create_test_rom() -> Rom {
    let data = vec![0u8; TEST_ROM_SIZE];
    Rom::new(data)
}

/// Creates a test ROM with pre-populated sprite data
fn create_test_rom_with_sprites() -> Rom {
    let mut rom = create_test_rom();

    let tiles = generate_test_sprite_sheet(TEST_TILE_COUNT);
    let sprite_data = encode_4bpp_sheet(&tiles);
    rom.write_bytes(TEST_UNCOMPRESSED_OFFSET, &sprite_data)
        .expect("Failed to write test sprite data");

    rom
}

/// Creates compressed test data at the specified offset
fn create_compressed_test_data(rom: &mut Rom, offset: usize, num_tiles: usize) {
    let tiles = generate_test_sprite_sheet(num_tiles);
    let raw_data = encode_4bpp_sheet(&tiles);

    let pass1: Vec<u8> = raw_data
        .iter()
        .enumerate()
        .filter(|(i, _)| i % 2 == 0)
        .map(|(_, &b)| b)
        .collect();

    let pass2: Vec<u8> = raw_data
        .iter()
        .enumerate()
        .filter(|(i, _)| i % 2 == 1)
        .map(|(_, &b)| b)
        .collect();

    let mut compressed = hal8_compress_with_rle(&pass1);
    compressed.extend(hal8_compress_with_rle(&pass2));

    rom.write_bytes(offset, &compressed)
        .expect("Failed to write compressed data");
}

/// Decompresses HAL8 interleaved data from the ROM
fn decompress_from_rom(rom: &Rom, offset: usize, expected_size: usize) -> Vec<u8> {
    let compressed_data = rom
        .read_bytes(offset, 8192)
        .expect("Failed to read compressed data");

    let mut decompressor = Decompressor::new(compressed_data);
    decompressor.decompress_interleaved(expected_size)
}

/// Simulates saving and reloading a ROM
fn save_and_reload_rom(rom: &Rom, temp_path: &PathBuf) -> Rom {
    rom.save(temp_path).expect("Failed to save ROM");
    Rom::load(temp_path).expect("Failed to load ROM")
}

// ============================================================================
// Test 1: Uncompressed Bin Edit
// ============================================================================

#[test]
fn test_uncompressed_bin_edit() {
    // Setup
    let mut rom = create_test_rom_with_sprites();

    // Step 1: Read original sprite data
    let original_bytes = rom
        .read_bytes(TEST_UNCOMPRESSED_OFFSET, TEST_TILE_COUNT * 32)
        .expect("Failed to read original sprite data")
        .to_vec();

    let original_tiles = decode_4bpp_sheet(&original_bytes);
    assert_eq!(original_tiles.len(), TEST_TILE_COUNT);

    // Step 2: Modify 4 tiles
    let mut modified_tiles = original_tiles.clone();

    modified_tiles[0] = create_checkerboard_tile(1, 2);
    modified_tiles[1] = create_test_pattern_tile();
    modify_tile(&mut modified_tiles[2], |_, p| p ^ 0x0F);
    modified_tiles[3] = create_solid_tile(7);

    // Step 3: Encode back to bytes
    let modified_bytes = encode_4bpp_sheet(&modified_tiles);
    assert_eq!(modified_bytes.len(), original_bytes.len());

    // Step 4: Write to ROM
    rom.write_bytes(TEST_UNCOMPRESSED_OFFSET, &modified_bytes)
        .expect("Failed to write modified sprite data");

    // Step 5: Save and reload
    let temp_path = PathBuf::from("test_uncompressed_edit.smc");
    let reloaded_rom = save_and_reload_rom(&rom, &temp_path);

    // Step 6: Verify changes persisted
    let reloaded_bytes = reloaded_rom
        .read_bytes(TEST_UNCOMPRESSED_OFFSET, TEST_TILE_COUNT * 32)
        .expect("Failed to read reloaded sprite data")
        .to_vec();

    let reloaded_tiles = decode_4bpp_sheet(&reloaded_bytes);

    // Verify each modified tile
    assert!(
        tiles_equal(&reloaded_tiles[0], &modified_tiles[0]),
        "Tile 0 (checkerboard) should persist"
    );
    assert!(
        tiles_equal(&reloaded_tiles[1], &modified_tiles[1]),
        "Tile 1 (test pattern) should persist"
    );
    assert!(
        tiles_equal(&reloaded_tiles[2], &modified_tiles[2]),
        "Tile 2 (inverted) should persist"
    );
    assert!(
        tiles_equal(&reloaded_tiles[3], &modified_tiles[3]),
        "Tile 3 (solid) should persist"
    );

    // Verify unmodified tiles are unchanged
    for i in 4..TEST_TILE_COUNT {
        assert!(
            tiles_equal(&reloaded_tiles[i], &original_tiles[i]),
            "Tile {} should remain unmodified",
            i
        );
    }

    // Verify specific pixel values
    assert_eq!(pixel_at(&reloaded_tiles[0], 0, 0), Some(1));
    assert_eq!(pixel_at(&reloaded_tiles[0], 1, 0), Some(2));
    assert_eq!(pixel_at(&reloaded_tiles[0], 0, 1), Some(2));

    // Cleanup
    let _ = fs::remove_file(&temp_path);
}

// ============================================================================
// Test 2: Compressed Bin Edit (Round-trip)
// ============================================================================

#[test]
fn test_compressed_bin_edit_roundtrip() {
    // Setup
    let mut rom = create_test_rom();
    create_compressed_test_data(&mut rom, TEST_COMPRESSED_OFFSET, TEST_TILE_COUNT);

    // Step 1: Decompress
    let expected_size = TEST_TILE_COUNT * 32;
    let decompressed = decompress_from_rom(&rom, TEST_COMPRESSED_OFFSET, expected_size);
    assert_eq!(decompressed.len(), expected_size);

    // Step 2: Decode tiles
    let original_tiles = decode_4bpp_sheet(&decompressed);
    assert_eq!(original_tiles.len(), TEST_TILE_COUNT);

    // Step 3: Modify tiles
    let mut modified_tiles = original_tiles.clone();

    modified_tiles[0] = create_solid_tile(15);
    modified_tiles[5] = create_checkerboard_tile(8, 7);
    modified_tiles[10] = create_test_pattern_tile();

    // Step 4: Encode back
    let modified_bytes = encode_4bpp_sheet(&modified_tiles);

    // Step 5: Recompress
    let pass1: Vec<u8> = modified_bytes
        .iter()
        .enumerate()
        .filter(|(i, _)| i % 2 == 0)
        .map(|(_, &b)| b)
        .collect();

    let pass2: Vec<u8> = modified_bytes
        .iter()
        .enumerate()
        .filter(|(i, _)| i % 2 == 1)
        .map(|(_, &b)| b)
        .collect();

    let mut recompressed = hal8_compress_with_rle(&pass1);
    recompressed.extend(hal8_compress_with_rle(&pass2));

    // Step 6: Write to ROM
    rom.write_bytes(TEST_COMPRESSED_OFFSET, &recompressed)
        .expect("Failed to write recompressed data");

    // Step 7: Save and reload
    let temp_path = PathBuf::from("test_compressed_edit.smc");
    let reloaded_rom = save_and_reload_rom(&rom, &temp_path);

    // Step 8: Verify
    let final_decompressed =
        decompress_from_rom(&reloaded_rom, TEST_COMPRESSED_OFFSET, expected_size);
    let final_tiles = decode_4bpp_sheet(&final_decompressed);

    // Verify modified tiles
    assert!(
        tiles_equal(&final_tiles[0], &modified_tiles[0]),
        "Compressed tile 0 should match after round-trip"
    );
    assert!(
        tiles_equal(&final_tiles[5], &modified_tiles[5]),
        "Compressed tile 5 should match after round-trip"
    );
    assert!(
        tiles_equal(&final_tiles[10], &modified_tiles[10]),
        "Compressed tile 10 should match after round-trip"
    );

    assert_eq!(pixel_at(&final_tiles[0], 0, 0), Some(15));

    // Cleanup
    let _ = fs::remove_file(&temp_path);
}

// ============================================================================
// Test 3: Multi-Bin Edit
// ============================================================================

#[test]
fn test_multi_bin_edit() {
    // Setup
    let mut rom = create_test_rom();

    // Bin 1
    let tiles1 = generate_test_sprite_sheet(TEST_TILE_COUNT);
    let data1 = encode_4bpp_sheet(&tiles1);
    rom.write_bytes(TEST_UNCOMPRESSED_OFFSET, &data1).unwrap();

    // Bin 2
    let tiles2 = generate_test_sprite_sheet(TEST_TILE_COUNT);
    let data2 = encode_4bpp_sheet(&tiles2);
    rom.write_bytes(TEST_SECOND_BIN_OFFSET, &data2).unwrap();

    // Step 1: Read both
    let read_data1 = rom
        .read_bytes(TEST_UNCOMPRESSED_OFFSET, TEST_TILE_COUNT * 32)
        .unwrap()
        .to_vec();
    let read_data2 = rom
        .read_bytes(TEST_SECOND_BIN_OFFSET, TEST_TILE_COUNT * 32)
        .unwrap()
        .to_vec();

    let mut mod_tiles1 = decode_4bpp_sheet(&read_data1);
    let mut mod_tiles2 = decode_4bpp_sheet(&read_data2);

    // Step 2: Modify both differently
    mod_tiles1[0] = create_solid_tile(1);
    mod_tiles1[1] = create_solid_tile(2);

    mod_tiles2[0] = create_checkerboard_tile(3, 4);
    mod_tiles2[2] = create_test_pattern_tile();

    // Step 3: Write both
    let mod_data1 = encode_4bpp_sheet(&mod_tiles1);
    let mod_data2 = encode_4bpp_sheet(&mod_tiles2);

    rom.write_bytes(TEST_UNCOMPRESSED_OFFSET, &mod_data1)
        .unwrap();
    rom.write_bytes(TEST_SECOND_BIN_OFFSET, &mod_data2).unwrap();

    // Step 4: Save and reload
    let temp_path = PathBuf::from("test_multi_bin_edit.smc");
    let reloaded_rom = save_and_reload_rom(&rom, &temp_path);

    // Step 5: Verify both
    let verify_data1 = reloaded_rom
        .read_bytes(TEST_UNCOMPRESSED_OFFSET, TEST_TILE_COUNT * 32)
        .unwrap()
        .to_vec();
    let verify_data2 = reloaded_rom
        .read_bytes(TEST_SECOND_BIN_OFFSET, TEST_TILE_COUNT * 32)
        .unwrap()
        .to_vec();

    let verify_tiles1 = decode_4bpp_sheet(&verify_data1);
    let verify_tiles2 = decode_4bpp_sheet(&verify_data2);

    // Verify bin 1
    assert_eq!(pixel_at(&verify_tiles1[0], 0, 0), Some(1));
    assert_eq!(pixel_at(&verify_tiles1[1], 0, 0), Some(2));

    // Verify bin 2
    assert!(tiles_equal(&verify_tiles2[0], &mod_tiles2[0]));
    assert!(tiles_equal(&verify_tiles2[2], &mod_tiles2[2]));

    // Verify independence
    assert_eq!(pixel_at(&verify_tiles2[0], 0, 0), Some(3));

    // Cleanup
    let _ = fs::remove_file(&temp_path);
}

// ============================================================================
// Test 4: Edit Size Validation
// ============================================================================

#[test]
fn test_edit_size_validation() {
    // Setup
    let mut rom = create_test_rom_with_sprites();

    // Test 1: Writing exact size should succeed
    let exact_size_tiles = generate_test_sprite_sheet(TEST_TILE_COUNT);
    let exact_size_data = encode_4bpp_sheet(&exact_size_tiles);
    assert_eq!(exact_size_data.len(), TEST_TILE_COUNT * 32);

    let result = rom.write_bytes(TEST_UNCOMPRESSED_OFFSET, &exact_size_data);
    assert!(result.is_ok(), "Writing exact size should succeed");

    // Test 2: Writing smaller data should succeed
    let smaller_tiles = generate_test_sprite_sheet(TEST_TILE_COUNT / 2);
    let smaller_data = encode_4bpp_sheet(&smaller_tiles);

    let result = rom.write_bytes(TEST_UNCOMPRESSED_OFFSET, &smaller_data);
    assert!(result.is_ok(), "Writing smaller size should succeed");

    // Test 3: Writing beyond ROM bounds should fail
    let large_data = vec![0u8; TEST_ROM_SIZE + 1000];
    let result = rom.write_bytes(TEST_ROM_SIZE - 100, &large_data);
    assert!(result.is_err(), "Writing beyond ROM size should fail");

    // Test 4: Writing at boundary should succeed
    let boundary_offset = TEST_ROM_SIZE - 32;
    let single_tile = encode_4bpp_sheet(&[create_solid_tile(5)]);
    let result = rom.write_bytes(boundary_offset, &single_tile);
    assert!(result.is_ok(), "Writing at exact boundary should succeed");

    // Test 5: Overflow by one byte should fail
    let result = rom.write_bytes(boundary_offset + 1, &single_tile);
    assert!(
        result.is_err(),
        "Writing one byte past boundary should fail"
    );
}

// ============================================================================
// Additional Tests
// ============================================================================

#[test]
fn test_decode_encode_roundtrip_integrity() {
    let tiles = generate_test_sprite_sheet(TEST_TILE_COUNT);
    let encoded = encode_4bpp_sheet(&tiles);
    let decoded = decode_4bpp_sheet(&encoded);
    let re_encoded = encode_4bpp_sheet(&decoded);

    assert_eq!(encoded, re_encoded, "Decode -> encode should be lossless");

    for (i, (orig, round)) in tiles.iter().zip(decoded.iter()).enumerate() {
        assert!(
            tiles_equal(orig, round),
            "Tile {} should match after round-trip",
            i
        );
    }
}

#[test]
fn test_rom_save_persists_all_data() {
    let mut rom = create_test_rom();

    let data1 = vec![0xAA; 256];
    let data2 = vec![0xBB; 256];
    let data3 = vec![0xCC; 256];

    rom.write_bytes(0x1000, &data1).unwrap();
    rom.write_bytes(0x2000, &data2).unwrap();
    rom.write_bytes(0x3000, &data3).unwrap();

    let temp_path = PathBuf::from("test_persist.smc");
    let reloaded = save_and_reload_rom(&rom, &temp_path);

    assert_eq!(reloaded.read_bytes(0x1000, 256).unwrap(), &data1[..]);
    assert_eq!(reloaded.read_bytes(0x2000, 256).unwrap(), &data2[..]);
    assert_eq!(reloaded.read_bytes(0x3000, 256).unwrap(), &data3[..]);

    let _ = fs::remove_file(&temp_path);
}

#[test]
fn test_large_bin_edit() {
    const LARGE_TILE_COUNT: usize = 64;

    let mut rom = create_test_rom();
    let tiles = generate_test_sprite_sheet(LARGE_TILE_COUNT);
    let data = encode_4bpp_sheet(&tiles);

    rom.write_bytes(TEST_UNCOMPRESSED_OFFSET, &data).unwrap();

    // Modify every 4th tile
    let mut modified = tiles.clone();
    for i in (0..LARGE_TILE_COUNT).step_by(4) {
        modified[i] = create_solid_tile((i % 16) as u8);
    }

    let mod_data = encode_4bpp_sheet(&modified);
    rom.write_bytes(TEST_UNCOMPRESSED_OFFSET, &mod_data)
        .unwrap();

    let temp_path = PathBuf::from("test_large_bin.smc");
    let reloaded = save_and_reload_rom(&rom, &temp_path);

    let verify_data = reloaded
        .read_bytes(TEST_UNCOMPRESSED_OFFSET, LARGE_TILE_COUNT * 32)
        .unwrap();
    let verify_tiles = decode_4bpp_sheet(&verify_data);

    for i in (0..LARGE_TILE_COUNT).step_by(4) {
        assert!(
            tiles_equal(&verify_tiles[i], &modified[i]),
            "Large bin tile {} should be modified",
            i
        );
    }

    let _ = fs::remove_file(&temp_path);
}
