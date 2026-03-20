//! Test utilities for portrait import tests
//!
//! Provides helpers for creating test images, converting to 4bpp tiles,
//! and verifying round-trip encoding/decoding.

use image::{ImageBuffer, Rgba, RgbaImage};

/// Standard test palette (16 colors for 4bpp)
pub const TEST_PALETTE: [(u8, u8, u8); 16] = [
    (0, 0, 0),       // 0: Transparent/Black
    (255, 0, 0),     // 1: Red
    (0, 255, 0),     // 2: Green
    (0, 0, 255),     // 3: Blue
    (255, 255, 0),   // 4: Yellow
    (255, 0, 255),   // 5: Magenta
    (0, 255, 255),   // 6: Cyan
    (255, 255, 255), // 7: White
    (128, 0, 0),     // 8: Dark Red
    (0, 128, 0),     // 9: Dark Green
    (0, 0, 128),     // 10: Dark Blue
    (128, 128, 0),   // 11: Dark Yellow
    (128, 0, 128),   // 12: Dark Magenta
    (0, 128, 128),   // 13: Dark Cyan
    (192, 192, 192), // 14: Light Gray
    (128, 128, 128), // 15: Gray
];

/// Pattern types for test images
#[derive(Debug, Clone, Copy)]
pub enum TestPattern {
    /// Checkerboard pattern alternating between color indices
    Checkerboard { color1: u8, color2: u8 },
    /// Horizontal stripes
    HorizontalStripes {
        color1: u8,
        color2: u8,
        stripe_height: u32,
    },
    /// Vertical stripes
    VerticalStripes {
        color1: u8,
        color2: u8,
        stripe_width: u32,
    },
    /// Gradient pattern cycling through all colors
    Gradient,
    /// Solid color fill
    Solid { color: u8 },
    /// Border pattern with different colors for edges
    Border { outer: u8, inner: u8 },
}

impl Default for TestPattern {
    fn default() -> Self {
        TestPattern::Checkerboard {
            color1: 1,
            color2: 7,
        }
    }
}

/// Convert palette index to RGBA color
fn index_to_rgba(index: u8, palette: &[(u8, u8, u8)]) -> Rgba<u8> {
    let idx = (index as usize).min(palette.len() - 1);
    let (r, g, b) = palette[idx];
    let alpha = if index == 0 { 0 } else { 255 };
    Rgba([r, g, b, alpha])
}

/// Create a test portrait image with a specific pattern
pub fn create_test_portrait(width: u32, height: u32, pattern: TestPattern) -> RgbaImage {
    let mut img = ImageBuffer::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let color_idx = match pattern {
                TestPattern::Checkerboard { color1, color2 } => {
                    let tile_x = x / 8;
                    let tile_y = y / 8;
                    if (tile_x + tile_y) % 2 == 0 {
                        color1
                    } else {
                        color2
                    }
                }
                TestPattern::HorizontalStripes {
                    color1,
                    color2,
                    stripe_height,
                } => {
                    if (y / stripe_height) % 2 == 0 {
                        color1
                    } else {
                        color2
                    }
                }
                TestPattern::VerticalStripes {
                    color1,
                    color2,
                    stripe_width,
                } => {
                    if (x / stripe_width) % 2 == 0 {
                        color1
                    } else {
                        color2
                    }
                }
                TestPattern::Gradient => {
                    let tile_x = (x / 8) as u8;
                    let tile_y = (y / 8) as u8;
                    (tile_x + tile_y * 2) % 16
                }
                TestPattern::Solid { color } => color,
                TestPattern::Border { outer, inner } => {
                    let is_border = x == 0
                        || x == width - 1
                        || y == 0
                        || y == height - 1
                        || x == width / 2
                        || y == height / 2;
                    if is_border {
                        outer
                    } else {
                        inner
                    }
                }
            };

            img.put_pixel(x, y, index_to_rgba(color_idx, &TEST_PALETTE));
        }
    }

    img
}

/// Create a 32x32 icon image (4x4 tiles)
pub fn create_test_icon(pattern: TestPattern) -> RgbaImage {
    create_test_portrait(32, 32, pattern)
}

/// Create a 128x128 large portrait image (16x16 tiles)
pub fn create_test_large_portrait(pattern: TestPattern) -> RgbaImage {
    create_test_portrait(128, 128, pattern)
}

/// Get the test palette as asset_core Color structs
pub fn get_test_palette_colors() -> Vec<asset_core::Color> {
    TEST_PALETTE
        .iter()
        .map(|(r, g, b)| asset_core::Color {
            r: *r,
            g: *g,
            b: *b,
        })
        .collect()
}

/// Convert RGBA image to palette indices using the test palette
pub fn image_to_palette_indices(img: &RgbaImage) -> Vec<u8> {
    img.pixels()
        .map(|pixel| rgba_to_palette_index(pixel, &TEST_PALETTE))
        .collect()
}

/// Find the closest palette index for an RGBA color
fn rgba_to_palette_index(pixel: &Rgba<u8>, palette: &[(u8, u8, u8)]) -> u8 {
    // Transparent pixels map to index 0
    if pixel[3] < 128 {
        return 0;
    }

    let (r, g, b) = (pixel[0], pixel[1], pixel[2]);

    let mut best_idx = 0;
    let mut min_dist = f32::MAX;

    for (idx, (pr, pg, pb)) in palette.iter().enumerate() {
        let dr = r as f32 - *pr as f32;
        let dg = g as f32 - *pg as f32;
        let db = b as f32 - *pb as f32;
        let dist = dr * dr + dg * dg + db * db;

        if dist < min_dist {
            min_dist = dist;
            best_idx = idx;
        }
    }

    best_idx as u8
}

/// Calculate expected tile count for a given image size
pub fn calculate_tile_count(width: u32, height: u32) -> usize {
    assert!(width % 8 == 0, "Width must be multiple of 8");
    assert!(height % 8 == 0, "Height must be multiple of 8");
    ((width / 8) * (height / 8)) as usize
}

/// Calculate expected 4bpp data size for a given image size
pub fn calculate_4bpp_size(width: u32, height: u32) -> usize {
    calculate_tile_count(width, height) * 32
}

/// Create a test ROM with minimum valid size
pub fn create_test_rom() -> rom_core::Rom {
    let data = vec![0u8; rom_core::EXPECTED_SIZE];
    rom_core::Rom::new(data)
}

/// Verify that two images are pixel-perfect identical
pub fn assert_images_equal(img1: &RgbaImage, img2: &RgbaImage) {
    assert_eq!(
        img1.dimensions(),
        img2.dimensions(),
        "Image dimensions must match"
    );

    let (width, height) = img1.dimensions();

    for y in 0..height {
        for x in 0..width {
            let p1 = img1.get_pixel(x, y);
            let p2 = img2.get_pixel(x, y);
            assert_eq!(
                p1, p2,
                "Pixel mismatch at ({}, {}): {:?} vs {:?}",
                x, y, p1, p2
            );
        }
    }
}

/// Convert image to tiles, encode to 4bpp, decode back, and compare
pub fn roundtrip_test(img: &RgbaImage, palette: &[asset_core::Color]) -> Result<(), String> {
    // Convert image to tiles
    let tiles = asset_core::image_to_tiles(img, palette);

    // Encode tiles to 4bpp
    let encoded = asset_core::encode_4bpp_sheet(&tiles);

    // Decode back to tiles
    let decoded_tiles = asset_core::decode_4bpp_sheet(&encoded);

    // Convert back to image
    let width_tiles = (img.width() / 8) as usize;
    let reconstructed = asset_core::tiles_to_image(&decoded_tiles, width_tiles, palette);

    // Compare dimensions
    if img.dimensions() != reconstructed.dimensions() {
        return Err(format!(
            "Dimension mismatch: {:?} vs {:?}",
            img.dimensions(),
            reconstructed.dimensions()
        ));
    }

    // Compare pixels (allowing for slight palette quantization differences)
    let (w, h) = img.dimensions();
    for y in 0..h {
        for x in 0..w {
            let orig = img.get_pixel(x, y);
            let recon = reconstructed.get_pixel(x, y);

            // Skip transparent pixels
            if orig[3] < 128 && recon[3] < 128 {
                continue;
            }

            // Check if colors match (after palette quantization)
            let orig_idx = rgba_to_palette_index(orig, &TEST_PALETTE);
            let recon_idx = rgba_to_palette_index(recon, &TEST_PALETTE);

            if orig_idx != recon_idx {
                return Err(format!(
                    "Pixel mismatch at ({}, {}): original index {} vs reconstructed index {}",
                    x, y, orig_idx, recon_idx
                ));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_test_portrait() {
        let img = create_test_portrait(32, 32, TestPattern::default());
        assert_eq!(img.dimensions(), (32, 32));
    }

    #[test]
    fn test_calculate_tile_count() {
        assert_eq!(calculate_tile_count(32, 32), 16); // 4x4 tiles
        assert_eq!(calculate_tile_count(128, 128), 256); // 16x16 tiles
        assert_eq!(calculate_tile_count(64, 32), 32); // 8x4 tiles
    }

    #[test]
    fn test_calculate_4bpp_size() {
        assert_eq!(calculate_4bpp_size(32, 32), 16 * 32); // 512 bytes
        assert_eq!(calculate_4bpp_size(128, 128), 256 * 32); // 8192 bytes
    }

    #[test]
    fn test_roundtrip_checkerboard() {
        let img = create_test_icon(TestPattern::Checkerboard {
            color1: 1,
            color2: 2,
        });
        let palette = get_test_palette_colors();
        roundtrip_test(&img, &palette).expect("Round-trip test should pass");
    }

    #[test]
    fn test_roundtrip_gradient() {
        let img = create_test_large_portrait(TestPattern::Gradient);
        let palette = get_test_palette_colors();
        roundtrip_test(&img, &palette).expect("Round-trip test should pass");
    }
}
