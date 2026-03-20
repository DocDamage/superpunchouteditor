use image::{ImageBuffer, Rgba};

use crate::frame::{Frame, SpriteEntry};
use crate::gfx::Tile;
use crate::palette::Color;

/// Options for rendering a frame
#[derive(Debug, Clone)]
pub struct RenderOptions {
    pub canvas_width: u32,
    pub canvas_height: u32,
    pub center_x: i16,
    pub center_y: i16,
    pub show_grid: bool,
    pub grid_size: u32,
    pub grid_color: Rgba<u8>,
    pub background_color: Rgba<u8>,
    pub show_selection: bool,
    pub selected_sprite: Option<usize>,
    pub selection_color: Rgba<u8>,
    pub zoom: f32,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            canvas_width: 512,
            canvas_height: 512,
            center_x: 256,
            center_y: 256,
            show_grid: true,
            grid_size: 8,
            grid_color: Rgba([60, 60, 80, 100]),
            background_color: Rgba([30, 30, 40, 255]),
            show_selection: true,
            selected_sprite: None,
            selection_color: Rgba([0, 150, 255, 200]),
            zoom: 2.0,
        }
    }
}

/// Extended frame renderer with additional visual options
pub struct FrameRenderer;

impl FrameRenderer {
    /// Render a frame with full visual options
    pub fn render_with_options(
        frame: &Frame,
        tiles: &[Tile],
        palette: &[Color],
        options: &RenderOptions,
    ) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        // Create base canvas
        let mut img = ImageBuffer::from_pixel(
            options.canvas_width,
            options.canvas_height,
            options.background_color,
        );

        // Draw grid if enabled
        if options.show_grid {
            Self::draw_grid(&mut img, options);
        }

        // Draw sprites
        for (idx, sprite) in frame.sprites.iter().enumerate() {
            let is_selected = options.selected_sprite == Some(idx);
            Self::draw_sprite(&mut img, sprite, tiles, palette, options, is_selected);
        }

        img
    }

    fn draw_grid(img: &mut ImageBuffer<Rgba<u8>, Vec<u8>>, options: &RenderOptions) {
        let grid_pixels = (options.grid_size as f32 * options.zoom) as u32;

        // Vertical lines
        let mut x = options.center_x as i16 % grid_pixels as i16;
        if x < 0 {
            x += grid_pixels as i16;
        }

        while (x as u32) < options.canvas_width {
            for y in 0..options.canvas_height {
                img.put_pixel(x as u32, y, options.grid_color);
            }
            x += grid_pixels as i16;
        }

        // Horizontal lines
        let mut y = options.center_y as i16 % grid_pixels as i16;
        if y < 0 {
            y += grid_pixels as i16;
        }

        while (y as u32) < options.canvas_height {
            for x in 0..options.canvas_width {
                img.put_pixel(x, y as u32, options.grid_color);
            }
            y += grid_pixels as i16;
        }

        // Draw center crosshair
        let cx = options.center_x as u32;
        let cy = options.center_y as u32;

        for i in 0..options.canvas_width {
            img.put_pixel(i, cy, Rgba([100, 100, 120, 150]));
        }
        for i in 0..options.canvas_height {
            img.put_pixel(cx, i, Rgba([100, 100, 120, 150]));
        }
    }

    fn draw_sprite(
        img: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
        sprite: &SpriteEntry,
        tiles: &[Tile],
        palette: &[Color],
        options: &RenderOptions,
        is_selected: bool,
    ) {
        let tile_idx = sprite.tile_id as usize;
        let Some(tile) = tiles.get(tile_idx) else {
            return;
        };

        let zoom = options.zoom;
        let center_x = options.center_x;
        let center_y = options.center_y;

        // Draw sprite pixels
        for ty in 0..8 {
            for tx in 0..8 {
                // Apply flips
                let src_x = if sprite.h_flip { 7 - tx } else { tx };
                let src_y = if sprite.v_flip { 7 - ty } else { ty };

                let color_idx = tile.pixels[src_y * 8 + src_x] as usize;
                if color_idx == 0 {
                    continue; // Transparent
                }

                // Use palette offset
                let palette_idx =
                    ((sprite.palette as usize) * 16 + color_idx).min(palette.len() - 1);
                let color = &palette[palette_idx];

                // Calculate screen position with zoom
                let base_x = center_x + (sprite.x as f32 * zoom) as i16;
                let base_y = center_y + (sprite.y as f32 * zoom) as i16;

                // Draw zoomed pixel
                for dy in 0..zoom as u32 {
                    for dx in 0..zoom as u32 {
                        let px = base_x + (tx as f32 * zoom) as i16 + dx as i16;
                        let py = base_y + (ty as f32 * zoom) as i16 + dy as i16;

                        if px >= 0 && px < img.width() as i16 && py >= 0 && py < img.height() as i16
                        {
                            img.put_pixel(
                                px as u32,
                                py as u32,
                                Rgba([color.r, color.g, color.b, 255]),
                            );
                        }
                    }
                }
            }
        }

        // Draw selection highlight
        if is_selected && options.show_selection {
            Self::draw_selection_box(img, sprite, options);
        }
    }

    fn draw_selection_box(
        img: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
        sprite: &SpriteEntry,
        options: &RenderOptions,
    ) {
        let zoom = options.zoom;
        let center_x = options.center_x;
        let center_y = options.center_y;

        let x1 = center_x + (sprite.x as f32 * zoom) as i16;
        let y1 = center_y + (sprite.y as f32 * zoom) as i16;
        let x2 = x1 + (8.0 * zoom) as i16;
        let y2 = y1 + (8.0 * zoom) as i16;

        // Draw rectangle outline
        for x in x1..=x2 {
            if x >= 0 && x < img.width() as i16 {
                if y1 >= 0 && y1 < img.height() as i16 {
                    img.put_pixel(x as u32, y1 as u32, options.selection_color);
                }
                if y2 >= 0 && y2 < img.height() as i16 {
                    img.put_pixel(x as u32, y2 as u32, options.selection_color);
                }
            }
        }
        for y in y1..=y2 {
            if y >= 0 && y < img.height() as i16 {
                if x1 >= 0 && x1 < img.width() as i16 {
                    img.put_pixel(x1 as u32, y as u32, options.selection_color);
                }
                if x2 >= 0 && x2 < img.width() as i16 {
                    img.put_pixel(x2 as u32, y as u32, options.selection_color);
                }
            }
        }
    }

    /// Find the sprite at the given screen coordinates
    pub fn hit_test(
        frame: &Frame,
        screen_x: i32,
        screen_y: i32,
        options: &RenderOptions,
    ) -> Option<usize> {
        let zoom = options.zoom;
        let center_x = options.center_x;
        let center_y = options.center_y;

        // Convert screen coords to sprite space
        let sprite_space_x = ((screen_x - center_x as i32) as f32 / zoom) as i16;
        let sprite_space_y = ((screen_y - center_y as i32) as f32 / zoom) as i16;

        // Check sprites in reverse order (top to bottom)
        for (idx, sprite) in frame.sprites.iter().enumerate().rev() {
            if sprite_space_x >= sprite.x
                && sprite_space_x < sprite.x + 8
                && sprite_space_y >= sprite.y
                && sprite_space_y < sprite.y + 8
            {
                return Some(idx);
            }
        }

        None
    }

    /// Convert screen coordinates to sprite position (snapped to grid)
    pub fn screen_to_sprite_pos(
        screen_x: i32,
        screen_y: i32,
        options: &RenderOptions,
        snap_to_grid: bool,
        grid_size: i16,
    ) -> (i16, i16) {
        let zoom = options.zoom;
        let center_x = options.center_x;
        let center_y = options.center_y;

        let mut x = ((screen_x - center_x as i32) as f32 / zoom) as i16;
        let mut y = ((screen_y - center_y as i32) as f32 / zoom) as i16;

        if snap_to_grid {
            x = (x / grid_size) * grid_size;
            y = (y / grid_size) * grid_size;
        }

        (x, y)
    }
}

/// Generate a preview image for a tile palette
pub fn render_tile_palette(
    tiles: &[Tile],
    palette: &[Color],
    tiles_per_row: usize,
) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let rows = (tiles.len() + tiles_per_row - 1) / tiles_per_row;
    let img_width = tiles_per_row * 8;
    let img_height = rows * 8;

    let mut img = ImageBuffer::new(img_width as u32, img_height as u32);

    for (idx, tile) in tiles.iter().enumerate() {
        let tx = idx % tiles_per_row;
        let ty = idx / tiles_per_row;

        for y in 0..8 {
            for x in 0..8 {
                let color_idx = tile.pixels[y * 8 + x] as usize;
                let alpha = if color_idx == 0 { 0 } else { 255 };
                let color = palette
                    .get(color_idx)
                    .cloned()
                    .unwrap_or(Color { r: 0, g: 0, b: 0 });

                let px = (tx * 8 + x) as u32;
                let py = (ty * 8 + y) as u32;
                img.put_pixel(px, py, Rgba([color.r, color.g, color.b, alpha]));
            }
        }
    }

    img
}

/// Render a single tile as a small image
pub fn render_single_tile(tile: &Tile, palette: &[Color]) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let mut img = ImageBuffer::new(8, 8);

    for y in 0..8 {
        for x in 0..8 {
            let color_idx = tile.pixels[y * 8 + x] as usize;
            let alpha = if color_idx == 0 { 0 } else { 255 };
            let color = palette
                .get(color_idx)
                .cloned()
                .unwrap_or(Color { r: 0, g: 0, b: 0 });
            img.put_pixel(x as u32, y as u32, Rgba([color.r, color.g, color.b, alpha]));
        }
    }

    img
}
