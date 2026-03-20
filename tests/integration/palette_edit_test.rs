//! Integration tests for palette editing functionality
//!
//! Tests verify that when a user edits palette colors and saves the ROM,
//! the palette data is correctly encoded and stored.

use tempfile::TempDir;

mod common;
use common::palette_utils::{
    assert_palettes_equal, bytes_remaining_in_bank, create_alternative_palette,
    create_full_test_palette, create_mock_rom_with_palettes, create_test_palette,
    expand_5bit_to_8bit, get_bank_for_offset, read_palette_from_rom, verify_color_roundtrip,
    write_palette_to_rom, FULL_PALETTE_BYTES, PALETTE_SIZE, TEST_PALETTE_BANK_BOUNDARY,
    TEST_PALETTE_OFFSET_1, TEST_PALETTE_OFFSET_2, TEST_PALETTE_OFFSET_3,
};

// Import core modules
use asset_core::{decode_palette, encode_palette, Color};
use rom_core::Rom;

// ============================================================================
// Test 1: Single Color Edit
// ============================================================================

/// Test 1: Edit one color in a palette
///
/// Creates a mock ROM with test palette, modifies a single color,
/// saves, reloads, and verifies the change persisted while other
/// colors remain unchanged.
#[test]
fn test_single_color_edit() {
    // Setup: Create mock ROM with test palettes
    let mut rom = create_mock_rom_with_palettes();

    // Read the original palette
    let original_palette = read_palette_from_rom(&rom, TEST_PALETTE_OFFSET_1);
    assert_eq!(original_palette.len(), PALETTE_SIZE);

    // Verify original color at index 5
    let _original_color5 = original_palette[5].clone();

    // Modify a single color (index 5 - Magenta)
    let new_color = Color {
        r: 200,
        g: 100,
        b: 50,
    };
    let mut modified_palette = original_palette.clone();
    modified_palette[5] = new_color.clone();

    // Write the modified palette back to ROM
    write_palette_to_rom(&mut rom, TEST_PALETTE_OFFSET_1, &modified_palette);

    // Verify in-memory change (accounting for SNES bit replication)
    let in_memory_palette = read_palette_from_rom(&rom, TEST_PALETTE_OFFSET_1);
    assert_eq!(in_memory_palette[5].r, expand_5bit_to_8bit(new_color.r));
    assert_eq!(in_memory_palette[5].g, expand_5bit_to_8bit(new_color.g));
    assert_eq!(in_memory_palette[5].b, expand_5bit_to_8bit(new_color.b));

    // Save ROM to temp file
    let temp_dir = TempDir::new().expect("Should create temp directory");
    let rom_path = temp_dir.path().join("test_single_color_edit.sfc");
    rom.save(&rom_path).expect("Should save ROM");

    // Load saved ROM
    let loaded_rom = Rom::load(&rom_path).expect("Should load saved ROM");

    // Read palette from loaded ROM
    let loaded_palette = read_palette_from_rom(&loaded_rom, TEST_PALETTE_OFFSET_1);

    // Verify modified color persisted (accounting for SNES bit replication)
    assert_eq!(
        (
            loaded_palette[5].r,
            loaded_palette[5].g,
            loaded_palette[5].b
        ),
        (
            expand_5bit_to_8bit(new_color.r),
            expand_5bit_to_8bit(new_color.g),
            expand_5bit_to_8bit(new_color.b)
        ),
        "Modified color should persist after save/load"
    );

    // Verify all other colors remain unchanged (accounting for SNES bit replication)
    for i in 0..PALETTE_SIZE {
        if i != 5 {
            let expected_r = expand_5bit_to_8bit(original_palette[i].r);
            let expected_g = expand_5bit_to_8bit(original_palette[i].g);
            let expected_b = expand_5bit_to_8bit(original_palette[i].b);
            assert_eq!(
                (
                    loaded_palette[i].r,
                    loaded_palette[i].g,
                    loaded_palette[i].b
                ),
                (expected_r, expected_g, expected_b),
                "Color {} should remain unchanged",
                i
            );
        }
    }

    // Cleanup happens automatically when temp_dir is dropped
    println!("Test 1: Single Color Edit - PASSED");
}

// ============================================================================
// Test 2: Full Palette Replace
// ============================================================================

/// Test 2: Replace all 16 colors in a palette
///
/// Creates a mock ROM, replaces the entire palette with new colors,
/// saves, reloads, and verifies all colors are correctly replaced.
#[test]
fn test_full_palette_replace() {
    // Setup: Create mock ROM
    let mut rom = create_mock_rom_with_palettes();

    // Get the original palette for comparison
    let original_palette = read_palette_from_rom(&rom, TEST_PALETTE_OFFSET_1);

    // Create a completely new palette
    let new_palette = create_full_test_palette();

    // Verify palettes are different (after SNES conversion)
    // Compare color 8: original has Dark Red (128,0,0), new has Orange (255,128,0)
    let orig_r_expanded = expand_5bit_to_8bit(original_palette[8].r);
    let new_r_expanded = expand_5bit_to_8bit(new_palette[8].r);
    assert_ne!(
        orig_r_expanded, new_r_expanded,
        "New palette should be different from original after SNES conversion"
    );

    // Write new palette to ROM
    write_palette_to_rom(&mut rom, TEST_PALETTE_OFFSET_1, &new_palette);

    // Save ROM to temp file
    let temp_dir = TempDir::new().expect("Should create temp directory");
    let rom_path = temp_dir.path().join("test_full_palette_replace.sfc");
    rom.save(&rom_path).expect("Should save ROM");

    // Load saved ROM
    let loaded_rom = Rom::load(&rom_path).expect("Should load saved ROM");

    // Read palette from loaded ROM
    let loaded_palette = read_palette_from_rom(&loaded_rom, TEST_PALETTE_OFFSET_1);

    // Verify all 16 colors were replaced correctly
    assert_palettes_equal(&loaded_palette, &new_palette);

    // Verify each specific color (accounting for SNES bit replication)
    assert_eq!(
        (
            loaded_palette[0].r,
            loaded_palette[0].g,
            loaded_palette[0].b
        ),
        (0, 0, 0),
        "Color 0 should be Black"
    );
    assert_eq!(
        (
            loaded_palette[1].r,
            loaded_palette[1].g,
            loaded_palette[1].b
        ),
        (expand_5bit_to_8bit(255), expand_5bit_to_8bit(64), 0),
        "Color 1 should be Orange-Red"
    );
    assert_eq!(
        (
            loaded_palette[7].r,
            loaded_palette[7].g,
            loaded_palette[7].b
        ),
        (
            expand_5bit_to_8bit(255),
            expand_5bit_to_8bit(255),
            expand_5bit_to_8bit(255)
        ),
        "Color 7 should be White (248,248,248)"
    );
    assert_eq!(
        (
            loaded_palette[15].r,
            loaded_palette[15].g,
            loaded_palette[15].b
        ),
        (
            expand_5bit_to_8bit(128),
            expand_5bit_to_8bit(128),
            expand_5bit_to_8bit(255)
        ),
        "Color 15 should be Periwinkle"
    );

    println!("Test 2: Full Palette Replace - PASSED");
}

// ============================================================================
// Test 3: Multiple Palette Edits
// ============================================================================

/// Test 3: Edit 2 palettes in one save
///
/// Creates a mock ROM, modifies two different palettes at different
/// locations, saves once, and verifies both changes persisted.
#[test]
fn test_multiple_palette_edits() {
    // Setup: Create mock ROM
    let mut rom = create_mock_rom_with_palettes();

    // Read original palettes
    let original_palette1 = read_palette_from_rom(&rom, TEST_PALETTE_OFFSET_1);
    let original_palette2 = read_palette_from_rom(&rom, TEST_PALETTE_OFFSET_2);

    // Create modified versions of both palettes
    let mut modified_palette1 = original_palette1.clone();
    modified_palette1[1] = Color {
        r: 255,
        g: 128,
        b: 0,
    }; // Orange
    modified_palette1[2] = Color {
        r: 128,
        g: 255,
        b: 0,
    }; // Lime

    let mut modified_palette2 = original_palette2.clone();
    modified_palette2[3] = Color {
        r: 0,
        g: 128,
        b: 255,
    }; // Light Blue
    modified_palette2[7] = Color {
        r: 240,
        g: 240,
        b: 240,
    }; // Near White

    // Write both modified palettes to ROM
    write_palette_to_rom(&mut rom, TEST_PALETTE_OFFSET_1, &modified_palette1);
    write_palette_to_rom(&mut rom, TEST_PALETTE_OFFSET_2, &modified_palette2);

    // Save ROM to temp file (single save for both edits)
    let temp_dir = TempDir::new().expect("Should create temp directory");
    let rom_path = temp_dir.path().join("test_multiple_palette_edits.sfc");
    rom.save(&rom_path).expect("Should save ROM");

    // Load saved ROM
    let loaded_rom = Rom::load(&rom_path).expect("Should load saved ROM");

    // Read both palettes from loaded ROM
    let loaded_palette1 = read_palette_from_rom(&loaded_rom, TEST_PALETTE_OFFSET_1);
    let loaded_palette2 = read_palette_from_rom(&loaded_rom, TEST_PALETTE_OFFSET_2);

    // Verify palette 1 changes (accounting for SNES bit replication)
    assert_eq!(
        (
            loaded_palette1[1].r,
            loaded_palette1[1].g,
            loaded_palette1[1].b
        ),
        (expand_5bit_to_8bit(255), expand_5bit_to_8bit(128), 0),
        "Palette 1 color 1 should be Orange"
    );
    assert_eq!(
        (
            loaded_palette1[2].r,
            loaded_palette1[2].g,
            loaded_palette1[2].b
        ),
        (expand_5bit_to_8bit(128), expand_5bit_to_8bit(255), 0),
        "Palette 1 color 2 should be Lime"
    );

    // Verify palette 2 changes (accounting for SNES bit replication)
    assert_eq!(
        (
            loaded_palette2[3].r,
            loaded_palette2[3].g,
            loaded_palette2[3].b
        ),
        (0, expand_5bit_to_8bit(128), expand_5bit_to_8bit(255)),
        "Palette 2 color 3 should be Light Blue"
    );
    assert_eq!(
        (
            loaded_palette2[7].r,
            loaded_palette2[7].g,
            loaded_palette2[7].b
        ),
        (
            expand_5bit_to_8bit(240),
            expand_5bit_to_8bit(240),
            expand_5bit_to_8bit(240)
        ),
        "Palette 2 color 7 should be Near White"
    );

    // Verify unmodified colors remain unchanged (accounting for SNES bit replication)
    assert_eq!(
        (
            loaded_palette1[3].r,
            loaded_palette1[3].g,
            loaded_palette1[3].b
        ),
        (
            expand_5bit_to_8bit(original_palette1[3].r),
            expand_5bit_to_8bit(original_palette1[3].g),
            expand_5bit_to_8bit(original_palette1[3].b)
        ),
        "Palette 1 unmodified colors should remain unchanged"
    );
    assert_eq!(
        (
            loaded_palette2[1].r,
            loaded_palette2[1].g,
            loaded_palette2[1].b
        ),
        (
            expand_5bit_to_8bit(original_palette2[1].r),
            expand_5bit_to_8bit(original_palette2[1].g),
            expand_5bit_to_8bit(original_palette2[1].b)
        ),
        "Palette 2 unmodified colors should remain unchanged"
    );

    println!("Test 3: Multiple Palette Edits - PASSED");
}

// ============================================================================
// Test 4: Palette Bank Boundary
// ============================================================================

/// Test 4: Edit palette near bank boundary
///
/// Creates a mock ROM with a palette placed right at a bank boundary,
/// edits it, saves, and verifies the data is correctly stored.
#[test]
fn test_palette_bank_boundary() {
    // Setup: Create mock ROM (palette already written at boundary)
    let mut rom = create_mock_rom_with_palettes();

    // Verify we're at a bank boundary
    let bank = get_bank_for_offset(TEST_PALETTE_BANK_BOUNDARY);
    let bytes_remaining = bytes_remaining_in_bank(TEST_PALETTE_BANK_BOUNDARY);

    println!(
        "Palette at offset 0x{:X}, bank 0x{:X}, bytes remaining in bank: {}",
        TEST_PALETTE_BANK_BOUNDARY, bank, bytes_remaining
    );

    // The palette should fit exactly at the bank boundary
    assert_eq!(
        bytes_remaining, FULL_PALETTE_BYTES,
        "Test setup: palette should fit exactly at bank boundary"
    );

    // Read original palette
    let original_palette = read_palette_from_rom(&rom, TEST_PALETTE_BANK_BOUNDARY);

    // Modify the palette
    let mut modified_palette = original_palette.clone();
    modified_palette[0] = Color { r: 0, g: 0, b: 0 }; // Keep black
    modified_palette[1] = Color {
        r: 255,
        g: 255,
        b: 0,
    }; // Yellow
    modified_palette[15] = Color {
        r: 128,
        g: 128,
        b: 128,
    }; // Gray

    // Write modified palette
    write_palette_to_rom(&mut rom, TEST_PALETTE_BANK_BOUNDARY, &modified_palette);

    // Save ROM to temp file
    let temp_dir = TempDir::new().expect("Should create temp directory");
    let rom_path = temp_dir.path().join("test_palette_bank_boundary.sfc");
    rom.save(&rom_path).expect("Should save ROM");

    // Load saved ROM
    let loaded_rom = Rom::load(&rom_path).expect("Should load saved ROM");

    // Read palette from boundary location
    let loaded_palette = read_palette_from_rom(&loaded_rom, TEST_PALETTE_BANK_BOUNDARY);

    // Verify modified colors (accounting for SNES bit replication)
    assert_eq!(
        (
            loaded_palette[1].r,
            loaded_palette[1].g,
            loaded_palette[1].b
        ),
        (expand_5bit_to_8bit(255), expand_5bit_to_8bit(255), 0),
        "Boundary palette color 1 should be Yellow"
    );
    assert_eq!(
        (
            loaded_palette[15].r,
            loaded_palette[15].g,
            loaded_palette[15].b
        ),
        (
            expand_5bit_to_8bit(128),
            expand_5bit_to_8bit(128),
            expand_5bit_to_8bit(128)
        ),
        "Boundary palette color 15 should be Gray"
    );

    // Verify color 0 (black/transparent) remains
    assert_eq!(
        (
            loaded_palette[0].r,
            loaded_palette[0].g,
            loaded_palette[0].b
        ),
        (0, 0, 0),
        "Boundary palette color 0 should remain Black"
    );

    println!("Test 4: Palette Bank Boundary - PASSED");
}

// ============================================================================
// Test 5: Color Roundtrip
// ============================================================================

/// Test 5: Verify RGB→SNES→RGB conversion
///
/// Tests that converting colors from RGB to SNES format and back
/// produces the expected results with proper precision loss.
#[test]
fn test_color_roundtrip() {
    // Test various colors
    let test_colors = vec![
        Color { r: 0, g: 0, b: 0 }, // Black
        Color {
            r: 255,
            g: 255,
            b: 255,
        }, // White
        Color { r: 255, g: 0, b: 0 }, // Red
        Color { r: 0, g: 255, b: 0 }, // Green
        Color { r: 0, g: 0, b: 255 }, // Blue
        Color {
            r: 128,
            g: 128,
            b: 128,
        }, // Mid gray
        Color {
            r: 255,
            g: 128,
            b: 64,
        }, // Orange-ish
        Color {
            r: 200,
            g: 100,
            b: 50,
        }, // Random color
        Color { r: 7, g: 15, b: 31 }, // Dark colors
        Color {
            r: 248,
            g: 240,
            b: 224,
        }, // Light colors
    ];

    for color in &test_colors {
        // Verify roundtrip conversion
        assert!(
            verify_color_roundtrip(color),
            "Color ({}, {}, {}) should roundtrip correctly",
            color.r,
            color.g,
            color.b
        );

        // Test explicit conversion
        let snes_value = color.to_snes();
        let roundtrip = Color::from_snes(snes_value);

        // SNES uses 5-bit per channel with bit replication
        let expected_r = expand_5bit_to_8bit(color.r);
        let expected_g = expand_5bit_to_8bit(color.g);
        let expected_b = expand_5bit_to_8bit(color.b);

        assert_eq!(
            (roundtrip.r, roundtrip.g, roundtrip.b),
            (expected_r, expected_g, expected_b),
            "Color ({}, {}, {}) roundtrip mismatch",
            color.r,
            color.g,
            color.b
        );
    }

    // Test palette encode/decode roundtrip
    let palette = create_test_palette();
    let encoded = encode_palette(&palette);
    let decoded = decode_palette(&encoded);

    // SNES conversion uses bit replication
    for (i, (original, decoded)) in palette.iter().zip(decoded.iter()).enumerate() {
        let expected_r = expand_5bit_to_8bit(original.r);
        let expected_g = expand_5bit_to_8bit(original.g);
        let expected_b = expand_5bit_to_8bit(original.b);

        assert_eq!(
            (decoded.r, decoded.g, decoded.b),
            (expected_r, expected_g, expected_b),
            "Palette color {} roundtrip mismatch",
            i
        );
    }

    println!("Test 5: Color Roundtrip - PASSED");
}

// ============================================================================
// Test 6: ROM Size Preservation
// ============================================================================

/// Test 6: Verify ROM size unchanged after palette edits
///
/// Creates a ROM, performs multiple palette edits, saves,
/// and verifies the ROM size remains exactly the same.
#[test]
fn test_rom_size_preservation() {
    // Setup: Create mock ROM
    let mut rom = create_mock_rom_with_palettes();
    let original_size = rom.size();

    println!(
        "Original ROM size: {} bytes ({} KB)",
        original_size,
        original_size / 1024
    );

    // Perform multiple palette edits
    let palette2 = create_alternative_palette();
    let palette3 = create_full_test_palette();

    // Edit palette 1 - replace with alternative
    write_palette_to_rom(&mut rom, TEST_PALETTE_OFFSET_1, &palette2);

    // Edit palette 2 - replace with full test palette
    write_palette_to_rom(&mut rom, TEST_PALETTE_OFFSET_2, &palette3);

    // Edit at bank boundary - modify single color
    let mut modified_boundary = read_palette_from_rom(&rom, TEST_PALETTE_BANK_BOUNDARY);
    modified_boundary[5] = Color {
        r: 180,
        g: 60,
        b: 120,
    };
    write_palette_to_rom(&mut rom, TEST_PALETTE_BANK_BOUNDARY, &modified_boundary);

    // Verify in-memory size unchanged
    assert_eq!(
        rom.size(),
        original_size,
        "ROM size should not change after in-memory edits"
    );

    // Save ROM to temp file
    let temp_dir = TempDir::new().expect("Should create temp directory");
    let rom_path = temp_dir.path().join("test_rom_size_preservation.sfc");
    rom.save(&rom_path).expect("Should save ROM");

    // Check file size on disk
    let file_size = std::fs::metadata(&rom_path)
        .expect("Should get file metadata")
        .len() as usize;

    assert_eq!(
        file_size, original_size,
        "Saved ROM file size should match original: expected {} bytes, got {} bytes",
        original_size, file_size
    );

    // Load ROM and verify size
    let loaded_rom = Rom::load(&rom_path).expect("Should load ROM");
    assert_eq!(
        loaded_rom.size(),
        original_size,
        "Loaded ROM size should match original"
    );

    // Verify all edits are still present
    let loaded_p1 = read_palette_from_rom(&loaded_rom, TEST_PALETTE_OFFSET_1);
    let loaded_p2 = read_palette_from_rom(&loaded_rom, TEST_PALETTE_OFFSET_2);
    let loaded_p3 = read_palette_from_rom(&loaded_rom, TEST_PALETTE_BANK_BOUNDARY);

    // Check palette 1 (alternative)
    assert_palettes_equal(&loaded_p1, &palette2);

    // Check palette 2 (full test)
    assert_palettes_equal(&loaded_p2, &palette3);

    // Check boundary palette modification (accounting for SNES bit replication)
    assert_eq!(
        (loaded_p3[5].r, loaded_p3[5].g, loaded_p3[5].b),
        (
            expand_5bit_to_8bit(180),
            expand_5bit_to_8bit(60),
            expand_5bit_to_8bit(120)
        ),
        "Boundary palette color 5 should be preserved"
    );

    println!("Test 6: ROM Size Preservation - PASSED");
}

// ============================================================================
// Test 7: Edge Case - Edit Color 0 (Transparent)
// ============================================================================

/// Test 7: Verify editing color 0 (typically transparent)
///
/// Color 0 is often treated as transparent in SNES graphics.
/// This test ensures it can still be edited like any other color.
#[test]
fn test_edit_transparent_color() {
    // Setup: Create mock ROM
    let mut rom = create_mock_rom_with_palettes();

    // Read original palette
    let original_palette = read_palette_from_rom(&rom, TEST_PALETTE_OFFSET_1);
    assert_eq!(
        (
            original_palette[0].r,
            original_palette[0].g,
            original_palette[0].b
        ),
        (0, 0, 0),
        "Color 0 should start as black"
    );

    // Modify color 0 to a different color
    let mut modified_palette = original_palette.clone();
    modified_palette[0] = Color { r: 255, g: 0, b: 0 }; // Make it red

    // Write to ROM
    write_palette_to_rom(&mut rom, TEST_PALETTE_OFFSET_1, &modified_palette);

    // Save and reload
    let temp_dir = TempDir::new().expect("Should create temp directory");
    let rom_path = temp_dir.path().join("test_edit_transparent_color.sfc");
    rom.save(&rom_path).expect("Should save ROM");

    let loaded_rom = Rom::load(&rom_path).expect("Should load ROM");
    let loaded_palette = read_palette_from_rom(&loaded_rom, TEST_PALETTE_OFFSET_1);

    // Color 0 SNES format: 255 -> 248 (5-bit max with bit replication)
    assert_eq!(
        loaded_palette[0].r,
        expand_5bit_to_8bit(255),
        "Color 0 R should be 248 after SNES conversion"
    );
    assert_eq!(loaded_palette[0].g, 0, "Color 0 G should be 0");
    assert_eq!(loaded_palette[0].b, 0, "Color 0 B should be 0");

    println!("Test 7: Edit Transparent Color - PASSED");
}

// ============================================================================
// Test 8: Consecutive Palette Edits
// ============================================================================

/// Test 8: Edit palettes at consecutive offsets
///
/// Tests editing multiple palettes that are stored back-to-back
/// to ensure no data overlap or corruption.
#[test]
fn test_consecutive_palette_edits() {
    // Setup: Create mock ROM
    let mut rom = create_mock_rom_with_palettes();

    // Palettes are already at consecutive offsets:
    // - TEST_PALETTE_OFFSET_1 (0x10000)
    // - TEST_PALETTE_OFFSET_2 (0x10020 = +32 bytes)
    // - TEST_PALETTE_OFFSET_3 (0x10040 = +64 bytes)

    let offset1 = TEST_PALETTE_OFFSET_1;
    let offset2 = TEST_PALETTE_OFFSET_2;
    let offset3 = TEST_PALETTE_OFFSET_3;

    // Verify offsets are consecutive (each palette is 32 bytes)
    assert_eq!(
        offset2 - offset1,
        FULL_PALETTE_BYTES,
        "Offset 2 should be 32 bytes after offset 1"
    );
    assert_eq!(
        offset3 - offset2,
        FULL_PALETTE_BYTES,
        "Offset 3 should be 32 bytes after offset 2"
    );

    // Create three different palettes
    let palette1 = create_test_palette();
    let mut palette2 = create_alternative_palette();
    let mut palette3 = create_full_test_palette();

    // Modify each palette uniquely
    palette2[5] = Color {
        r: 100,
        g: 200,
        b: 50,
    };
    palette3[10] = Color {
        r: 50,
        g: 100,
        b: 200,
    };

    // Write all three palettes
    write_palette_to_rom(&mut rom, offset1, &palette1);
    write_palette_to_rom(&mut rom, offset2, &palette2);
    write_palette_to_rom(&mut rom, offset3, &palette3);

    // Save and reload
    let temp_dir = TempDir::new().expect("Should create temp directory");
    let rom_path = temp_dir.path().join("test_consecutive_palette_edits.sfc");
    rom.save(&rom_path).expect("Should save ROM");

    let loaded_rom = Rom::load(&rom_path).expect("Should load ROM");

    // Read all three palettes
    let loaded1 = read_palette_from_rom(&loaded_rom, offset1);
    let loaded2 = read_palette_from_rom(&loaded_rom, offset2);
    let loaded3 = read_palette_from_rom(&loaded_rom, offset3);

    // Verify each palette is correct
    assert_palettes_equal(&loaded1, &palette1);
    assert_palettes_equal(&loaded2, &palette2);
    assert_palettes_equal(&loaded3, &palette3);

    // Verify specific modified colors (accounting for SNES bit replication)
    assert_eq!(
        (loaded2[5].r, loaded2[5].g, loaded2[5].b),
        (
            expand_5bit_to_8bit(100),
            expand_5bit_to_8bit(200),
            expand_5bit_to_8bit(50)
        ),
        "Palette 2 color 5 should match (with SNES conversion)"
    );
    assert_eq!(
        (loaded3[10].r, loaded3[10].g, loaded3[10].b),
        (
            expand_5bit_to_8bit(50),
            expand_5bit_to_8bit(100),
            expand_5bit_to_8bit(200)
        ),
        "Palette 3 color 10 should match (with SNES conversion)"
    );

    println!("Test 8: Consecutive Palette Edits - PASSED");
}
