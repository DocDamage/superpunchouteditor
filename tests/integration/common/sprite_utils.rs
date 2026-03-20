//! Sprite utility functions for integration tests
//!
//! Provides helper functions for creating test tiles and sprite patterns.

use asset_core::Tile;

/// Creates a tile with an alternating checkerboard pattern
pub fn create_checkerboard_tile(color1: u8, color2: u8) -> Tile {
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
pub fn create_horizontal_stripes_tile(color1: u8, color2: u8) -> Tile {
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
pub fn create_vertical_stripes_tile(color1: u8, color2: u8) -> Tile {
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
pub fn create_solid_tile(color: u8) -> Tile {
    Tile {
        pixels: vec![color & 0x0F; 64],
    }
}

/// Creates a tile with a specific test pattern (corners marked)
pub fn create_test_pattern_tile() -> Tile {
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
pub fn modify_tile(tile: &mut Tile, transform: impl Fn(usize, u8) -> u8) {
    for (idx, pixel) in tile.pixels.iter_mut().enumerate() {
        *pixel = transform(idx, *pixel) & 0x0F;
    }
}

/// Inverts all colors in a tile (XOR with 0x0F)
pub fn invert_tile(tile: &mut Tile) {
    modify_tile(tile, |_, p| p ^ 0x0F);
}

/// Shifts all colors by a fixed amount
pub fn shift_tile_colors(tile: &mut Tile, shift: u8) {
    let shift = shift & 0x0F;
    modify_tile(tile, |_, p| p.wrapping_add(shift));
}

/// Generates a test sprite sheet with multiple tiles
pub fn generate_test_sprite_sheet(num_tiles: usize) -> Vec<Tile> {
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

/// Verifies that two tiles have identical pixel data
pub fn tiles_equal(a: &Tile, b: &Tile) -> bool {
    a.pixels == b.pixels
}

/// Gets the color at a specific position in a tile
pub fn pixel_at(tile: &Tile, x: usize, y: usize) -> Option<u8> {
    if x < 8 && y < 8 {
        tile.pixels.get(y * 8 + x).copied()
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checkerboard_pattern() {
        let tile = create_checkerboard_tile(1, 2);
        assert_eq!(pixel_at(&tile, 0, 0), Some(1));
        assert_eq!(pixel_at(&tile, 1, 0), Some(2));
        assert_eq!(pixel_at(&tile, 0, 1), Some(2));
        assert_eq!(pixel_at(&tile, 1, 1), Some(1));
    }

    #[test]
    fn test_tile_modification() {
        let mut tile = create_solid_tile(5);
        shift_tile_colors(&mut tile, 3);
        assert_eq!(pixel_at(&tile, 0, 0), Some(8));
    }

    #[test]
    fn test_solid_tile() {
        let tile = create_solid_tile(7);
        assert!(tile.pixels.iter().all(|&p| p == 7));
    }

    #[test]
    fn test_test_pattern_tile() {
        let tile = create_test_pattern_tile();
        assert_eq!(pixel_at(&tile, 0, 0), Some(1)); // Top-left
        assert_eq!(pixel_at(&tile, 7, 0), Some(2)); // Top-right
        assert_eq!(pixel_at(&tile, 0, 7), Some(3)); // Bottom-left
        assert_eq!(pixel_at(&tile, 7, 7), Some(4)); // Bottom-right
    }
}
