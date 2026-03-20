//! Graphics processing for SNES 4bpp (16-color) tiles.
//!
//! This module handles encoding and decoding of SNES graphics data, including:
//! - 4bpp tile format (4 bits per pixel, 16 colors)
//! - Tile sheet management
//! - Conversion to/from PNG images
//!
//! ## SNES 4bpp Tile Format
//! Each tile is 32 bytes (8×8 pixels at 4 bits per pixel).
//! - Bytes 0-15: Bitplanes 0-1 (low bits)
//! - Bytes 16-31: Bitplanes 2-3 (high bits)
//!
//! ## Example
//! ```
//! use asset_core::{decode_4bpp_tile, encode_4bpp_tile, Tile};
//!
//! let tile_data = [0u8; 32]; // 32 bytes = 1 tile
//! let tile = decode_4bpp_tile(&tile_data);
//! assert_eq!(tile.pixels.len(), 64); // 8x8 pixels
//!
//! let encoded = encode_4bpp_tile(&tile);
//! assert_eq!(encoded.len(), 32);
//! ```

use crate::palette::Color;
use serde::{Deserialize, Serialize};

/// Size of one 4bpp tile in bytes (32 bytes)
pub const TILE_SIZE_4BPP: usize = 32;

/// Width of a tile in pixels (8 pixels)
pub const TILE_WIDTH: usize = 8;

/// Height of a tile in pixels (8 pixels)
pub const TILE_HEIGHT: usize = 8;

/// Number of pixels in a tile (64 pixels)
pub const TILE_PIXEL_COUNT: usize = TILE_WIDTH * TILE_HEIGHT;

/// A single 8×8 tile containing 64 pixel values (0-15).
///
/// Each pixel value is a palette index (0-15). Index 0 is typically
/// transparent. The tile can be rendered using a 16-color palette.
///
/// # Example
/// ```
/// use asset_core::Tile;
///
/// // Create a tile with all pixels set to palette index 1
/// let tile = Tile::new(vec![1u8; 64]);
/// assert_eq!(tile.pixels.len(), 64);
/// ```
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Tile {
    /// 8×8 pixels, values 0-15 (palette indices)
    pub pixels: Vec<u8>,
}

impl Tile {
    /// Creates a new tile with the given pixel data.
    ///
    /// # Arguments
    /// - `pixels`: A vector of 64 palette indices (0-15)
    ///
    /// # Panics
    /// Panics if the pixel vector length is not 64.
    ///
    /// # Example
    /// ```
    /// use asset_core::Tile;
    ///
    /// let pixels: Vec<u8> = (0..64).map(|i| (i % 16) as u8).collect();
    /// let tile = Tile::new(pixels);
    /// ```
    pub fn new(pixels: Vec<u8>) -> Self {
        assert_eq!(
            pixels.len(),
            TILE_PIXEL_COUNT,
            "Tile must have exactly {} pixels",
            TILE_PIXEL_COUNT
        );
        Self { pixels }
    }

    /// Creates a new tile filled with the given palette index.
    ///
    /// # Arguments
    /// - `color_idx`: The palette index to fill with (0-15)
    ///
    /// # Example
    /// ```
    /// use asset_core::Tile;
    ///
    /// let tile = Tile::filled(0); // Transparent tile
    /// assert!(tile.pixels.iter().all(|&p| p == 0));
    /// ```
    pub fn filled(color_idx: u8) -> Self {
        Self {
            pixels: vec![color_idx; TILE_PIXEL_COUNT],
        }
    }

    /// Gets the pixel value at the given coordinates.
    ///
    /// # Arguments
    /// - `x`: X coordinate (0-7)
    /// - `y`: Y coordinate (0-7)
    ///
    /// # Returns
    /// The palette index at the specified position (0-15)
    ///
    /// # Example
    /// ```
    /// use asset_core::Tile;
    ///
    /// let tile = Tile::filled(5);
    /// assert_eq!(tile.get_pixel(3, 4), 5);
    /// ```
    pub fn get_pixel(&self, x: usize, y: usize) -> u8 {
        self.pixels[y * TILE_WIDTH + x]
    }

    /// Sets the pixel value at the given coordinates.
    ///
    /// # Arguments
    /// - `x`: X coordinate (0-7)
    /// - `y`: Y coordinate (0-7)
    /// - `value`: Palette index (0-15)
    ///
    /// # Example
    /// ```
    /// use asset_core::Tile;
    ///
    /// let mut tile = Tile::filled(0);
    /// tile.set_pixel(4, 4, 7);
    /// assert_eq!(tile.get_pixel(4, 4), 7);
    /// ```
    pub fn set_pixel(&mut self, x: usize, y: usize, value: u8) {
        self.pixels[y * TILE_WIDTH + x] = value & 0x0F;
    }

    /// Flips the tile horizontally.
    ///
    /// # Example
    /// ```
    /// use asset_core::Tile;
    ///
    /// let mut tile = Tile::new((0..64).map(|i| i as u8).collect());
    /// tile.flip_h();
    /// // Pixels are now mirrored horizontally
    /// ```
    pub fn flip_h(&mut self) {
        for y in 0..TILE_HEIGHT {
            for x in 0..TILE_WIDTH / 2 {
                let left_idx = y * TILE_WIDTH + x;
                let right_idx = y * TILE_WIDTH + (TILE_WIDTH - 1 - x);
                self.pixels.swap(left_idx, right_idx);
            }
        }
    }

    /// Flips the tile vertically.
    ///
    /// # Example
    /// ```
    /// use asset_core::Tile;
    ///
    /// let mut tile = Tile::new((0..64).map(|i| i as u8).collect());
    /// tile.flip_v();
    /// // Pixels are now mirrored vertically
    /// ```
    pub fn flip_v(&mut self) {
        for y in 0..TILE_HEIGHT / 2 {
            for x in 0..TILE_WIDTH {
                let top_idx = y * TILE_WIDTH + x;
                let bottom_idx = (TILE_HEIGHT - 1 - y) * TILE_WIDTH + x;
                self.pixels.swap(top_idx, bottom_idx);
            }
        }
    }
}

impl Default for Tile {
    fn default() -> Self {
        Self::filled(0)
    }
}

/// Decodes a 4bpp tile from raw bytes.
///
/// SNES 4bpp tiles use bitplane interleaving:
/// - Bytes 0-15 contain bitplanes 0 and 1 (low 2 bits of each pixel)
/// - Bytes 16-31 contain bitplanes 2 and 3 (high 2 bits of each pixel)
///
/// # Arguments
/// - `data`: Raw tile data (32 bytes)
///
/// # Returns
/// A `Tile` with 64 pixel values (0-15)
///
/// # Example
/// ```
/// use asset_core::decode_4bpp_tile;
///
/// let tile_data = [0u8; 32]; // 32 bytes = 1 tile
/// let tile = decode_4bpp_tile(&tile_data);
/// assert_eq!(tile.pixels.len(), 64); // 8x8 pixels
/// assert_eq!(tile.pixels[0], 0); // All zeros = transparent
/// ```
pub fn decode_4bpp_tile(data: &[u8]) -> Tile {
    let mut pixels = vec![0u8; TILE_PIXEL_COUNT];

    for y in 0..TILE_HEIGHT {
        for x in 0..TILE_WIDTH {
            let mut val = 0u8;
            let bit = 7 - x;

            // Plane 1 & 2 (low planes, bytes 0-15)
            if (data[y * 2] & (1 << bit)) != 0 {
                val |= 1;
            }
            if (data[y * 2 + 1] & (1 << bit)) != 0 {
                val |= 2;
            }

            // Plane 3 & 4 (high planes, bytes 16-31)
            if (data[16 + y * 2] & (1 << bit)) != 0 {
                val |= 4;
            }
            if (data[16 + y * 2 + 1] & (1 << bit)) != 0 {
                val |= 8;
            }

            pixels[y * TILE_WIDTH + x] = val;
        }
    }

    Tile { pixels }
}

/// Encodes a 4bpp tile to raw bytes.
///
/// # Arguments
/// - `tile`: A `Tile` with 64 pixel values (0-15)
///
/// # Returns
/// A vector of 32 bytes in SNES 4bpp format
///
/// # Example
/// ```
/// use asset_core::{Tile, encode_4bpp_tile};
///
/// let tile = Tile::filled(5);
/// let data = encode_4bpp_tile(&tile);
/// assert_eq!(data.len(), 32);
/// ```
pub fn encode_4bpp_tile(tile: &Tile) -> Vec<u8> {
    let mut data = vec![0u8; TILE_SIZE_4BPP];

    for y in 0..TILE_HEIGHT {
        for x in 0..TILE_WIDTH {
            let val = tile.pixels[y * TILE_WIDTH + x] & 0x0F;
            let bit = 7 - x;

            if (val & 1) != 0 {
                data[y * 2] |= 1 << bit;
            }
            if (val & 2) != 0 {
                data[y * 2 + 1] |= 1 << bit;
            }
            if (val & 4) != 0 {
                data[16 + y * 2] |= 1 << bit;
            }
            if (val & 8) != 0 {
                data[16 + y * 2 + 1] |= 1 << bit;
            }
        }
    }

    data
}

/// Decodes a sheet of 4bpp tiles from raw bytes.
///
/// # Arguments
/// - `data`: Raw tile data (multiple of 32 bytes)
///
/// # Returns
/// A vector of `Tile` structs
///
/// # Example
/// ```
/// use asset_core::decode_4bpp_sheet;
///
/// // 64 bytes = 2 tiles
/// let sheet_data = vec![0u8; 64];
/// let tiles = decode_4bpp_sheet(&sheet_data);
/// assert_eq!(tiles.len(), 2);
/// ```
pub fn decode_4bpp_sheet(data: &[u8]) -> Vec<Tile> {
    data.chunks_exact(TILE_SIZE_4BPP)
        .map(decode_4bpp_tile)
        .collect()
}

/// Encodes a sheet of 4bpp tiles to raw bytes.
///
/// # Arguments
/// - `tiles`: A slice of `Tile` structs
///
/// # Returns
/// A vector of bytes in SNES 4bpp format
///
/// # Example
/// ```
/// use asset_core::{Tile, encode_4bpp_sheet};
///
/// let tiles = vec![Tile::filled(0), Tile::filled(1)];
/// let data = encode_4bpp_sheet(&tiles);
/// assert_eq!(data.len(), 64); // 2 tiles × 32 bytes
/// ```
pub fn encode_4bpp_sheet(tiles: &[Tile]) -> Vec<u8> {
    let mut out = Vec::with_capacity(tiles.len() * TILE_SIZE_4BPP);
    for tile in tiles {
        out.extend(encode_4bpp_tile(tile));
    }
    out
}

/// Renders tiles to an image using the specified palette.
///
/// # Arguments
/// - `tiles`: A slice of `Tile` structs to render
/// - `width_tiles`: Number of tiles per row in the output image
/// - `palette`: A slice of `Color` structs (at least 16 colors)
///
/// # Returns
/// An RGBA image buffer
///
/// # Example
/// ```
/// use asset_core::{decode_4bpp_sheet, tiles_to_image, Color};
///
/// let tiles = vec![/* tiles */];
/// let palette = vec![Color::new(0, 0, 0), Color::new(255, 255, 255)];
/// // Render 16 tiles wide
/// let img = tiles_to_image(&tiles, 16, &palette);
/// ```
pub fn tiles_to_image(
    tiles: &[Tile],
    width_tiles: usize,
    palette: &[Color],
) -> image::ImageBuffer<image::Rgba<u8>, Vec<u8>> {
    let height_tiles = (tiles.len() + width_tiles - 1) / width_tiles;
    let mut img = image::ImageBuffer::new(
        (width_tiles * TILE_WIDTH) as u32,
        (height_tiles * TILE_HEIGHT) as u32,
    );

    for (idx, tile) in tiles.iter().enumerate() {
        let tx = idx % width_tiles;
        let ty = idx / width_tiles;

        for y in 0..TILE_HEIGHT {
            for x in 0..TILE_WIDTH {
                let color_idx = tile.pixels[y * TILE_WIDTH + x] as usize;
                let color = palette
                    .get(color_idx)
                    .cloned()
                    .unwrap_or(Color::new(0, 0, 0));
                // Index 0 is transparent
                let alpha = if color_idx == 0 { 0 } else { 255 };
                img.put_pixel(
                    (tx * TILE_WIDTH + x) as u32,
                    (ty * TILE_HEIGHT + y) as u32,
                    image::Rgba([color.r, color.g, color.b, alpha]),
                );
            }
        }
    }

    img
}

/// Converts an image to tiles using the specified palette.
///
/// This function quantizes the image colors to the nearest palette entry.
/// Transparent pixels (alpha < 128) are mapped to palette index 0.
///
/// # Arguments
/// - `img`: An RGBA image buffer
/// - `palette`: A slice of `Color` structs
///
/// # Returns
/// A vector of `Tile` structs
///
/// # Example
/// ```
/// use asset_core::{image_to_tiles, Color};
///
/// // Load an image (must be 8×8 pixel aligned)
/// // let img = image::open("sheet.png").unwrap().to_rgba8();
/// // let palette = vec![Color::new(0, 0, 0), Color::new(255, 255, 255)];
/// // let tiles = image_to_tiles(&img, &palette);
/// ```
pub fn image_to_tiles(
    img: &image::ImageBuffer<image::Rgba<u8>, Vec<u8>>,
    palette: &[Color],
) -> Vec<Tile> {
    let width_tiles = img.width() as usize / TILE_WIDTH;
    let height_tiles = img.height() as usize / TILE_HEIGHT;
    let mut tiles = Vec::with_capacity(width_tiles * height_tiles);

    for ty in 0..height_tiles {
        for tx in 0..width_tiles {
            let mut pixels = vec![0u8; TILE_PIXEL_COUNT];
            for y in 0..TILE_HEIGHT {
                for x in 0..TILE_WIDTH {
                    let pixel =
                        img.get_pixel((tx * TILE_WIDTH + x) as u32, (ty * TILE_HEIGHT + y) as u32);
                    // Match nearest color in palette
                    pixels[y * TILE_WIDTH + x] = match_color(pixel, palette) as u8;
                }
            }
            tiles.push(Tile { pixels });
        }
    }

    tiles
}

/// Finds the closest palette index for a given pixel color.
///
/// # Arguments
/// - `pixel`: An RGBA pixel
/// - `palette`: A slice of `Color` structs
///
/// # Returns
/// The index of the closest palette color (0-15)
fn match_color(pixel: &image::Rgba<u8>, palette: &[Color]) -> usize {
    // Transparent pixels map to index 0
    if pixel[3] < 128 {
        return 0;
    }

    let mut best_idx = 0;
    let mut min_dist = f32::MAX;

    for (idx, color) in palette.iter().enumerate() {
        let dr = pixel[0] as f32 - color.r as f32;
        let dg = pixel[1] as f32 - color.g as f32;
        let db = pixel[2] as f32 - color.b as f32;
        let dist = dr * dr + dg * dg + db * db;

        if dist < min_dist {
            min_dist = dist;
            best_idx = idx;
        }
    }

    best_idx
}

/// Calculates the number of tiles needed for a given pixel area.
///
/// # Arguments
/// - `width`: Width in pixels
/// - `height`: Height in pixels
///
/// # Returns
/// The number of tiles required
///
/// # Example
/// ```
/// use asset_core::calculate_tile_count;
///
/// assert_eq!(calculate_tile_count(16, 16), 4); // 2×2 tiles
/// assert_eq!(calculate_tile_count(8, 8), 1);   // 1 tile
/// ```
pub const fn calculate_tile_count(width: usize, height: usize) -> usize {
    let tiles_w = (width + TILE_WIDTH - 1) / TILE_WIDTH;
    let tiles_h = (height + TILE_HEIGHT - 1) / TILE_HEIGHT;
    tiles_w * tiles_h
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tile_new() {
        let pixels: Vec<u8> = (0..64).map(|i| (i % 16) as u8).collect();
        let tile = Tile::new(pixels.clone());
        assert_eq!(tile.pixels, pixels);
    }

    #[test]
    fn test_tile_filled() {
        let tile = Tile::filled(5);
        assert!(tile.pixels.iter().all(|&p| p == 5));
    }

    #[test]
    fn test_tile_get_set_pixel() {
        let mut tile = Tile::filled(0);
        tile.set_pixel(3, 4, 7);
        assert_eq!(tile.get_pixel(3, 4), 7);
    }

    #[test]
    fn test_tile_flip_h() {
        let mut tile = Tile::new((0..64).map(|i| i as u8).collect());
        let original = tile.pixels.clone();
        tile.flip_h();

        // Check that pixels are mirrored
        for y in 0..8 {
            for x in 0..4 {
                let left = original[y * 8 + x];
                let right = tile.pixels[y * 8 + (7 - x)];
                assert_eq!(left, right);
            }
        }
    }

    #[test]
    fn test_decode_4bpp_tile() {
        let tile_data = [0u8; TILE_SIZE_4BPP];
        let tile = decode_4bpp_tile(&tile_data);
        assert_eq!(tile.pixels.len(), TILE_PIXEL_COUNT);
        assert!(tile.pixels.iter().all(|&p| p == 0));
    }

    #[test]
    fn test_encode_4bpp_tile() {
        let tile = Tile::filled(5);
        let data = encode_4bpp_tile(&tile);
        assert_eq!(data.len(), TILE_SIZE_4BPP);
    }

    #[test]
    fn test_roundtrip() {
        // Create a tile with varying pixel values
        let original = Tile::new((0..64).map(|i| (i % 16) as u8).collect());
        let encoded = encode_4bpp_tile(&original);
        let decoded = decode_4bpp_tile(&encoded);
        assert_eq!(original.pixels, decoded.pixels);
    }

    #[test]
    fn test_decode_4bpp_sheet() {
        let sheet_data = vec![0u8; TILE_SIZE_4BPP * 3];
        let tiles = decode_4bpp_sheet(&sheet_data);
        assert_eq!(tiles.len(), 3);
    }

    #[test]
    fn test_calculate_tile_count() {
        assert_eq!(calculate_tile_count(8, 8), 1);
        assert_eq!(calculate_tile_count(16, 16), 4);
        assert_eq!(calculate_tile_count(256, 256), 1024);
    }
}
