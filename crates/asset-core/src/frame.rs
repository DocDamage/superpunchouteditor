use crate::compression::Decompressor;
use crate::gfx::{decode_4bpp_sheet, Tile};
use crate::palette::Color;
use manifest_core::BoxerRecord;
use rom_core::Rom;
use serde::{Deserialize, Serialize};

/// A sprite entry in a frame (OAM-style)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SpriteEntry {
    pub x: i16,       // Screen X position (can be negative for off-screen)
    pub y: i16,       // Screen Y position
    pub tile_id: u16, // Which tile to use
    pub palette: u8,  // Palette number (0-7)
    pub h_flip: bool, // Horizontal flip
    pub v_flip: bool, // Vertical flip
    pub priority: u8, // Sprite priority (0-3)
}

impl SpriteEntry {
    /// Create a new sprite entry at the given position
    pub fn new(tile_id: u16, x: i16, y: i16) -> Self {
        Self {
            x,
            y,
            tile_id,
            palette: 0,
            h_flip: false,
            v_flip: false,
            priority: 0,
        }
    }

    /// Parse a sprite entry from 4 bytes of ROM data
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < 4 {
            return None;
        }

        // Check for terminator bytes
        if data[0] == 0xC0 || data[0] == 0xF0 {
            return None;
        }

        let x = data[0] as i8 as i16;
        let y = data[1] as i8 as i16;
        let tile_id = data[2] as u16;
        let attr = data[3];

        // Parse OAM attributes
        let palette = (attr >> 1) & 0x07;
        let priority = (attr >> 4) & 0x03;
        let h_flip = (attr & 0x40) != 0;
        let v_flip = (attr & 0x80) != 0;

        Some(Self {
            x,
            y,
            tile_id,
            palette,
            h_flip,
            v_flip,
            priority,
        })
    }

    /// Serialize this sprite entry to 4 bytes
    pub fn to_bytes(&self) -> [u8; 4] {
        let mut attr: u8 = 0;
        attr |= (self.palette & 0x07) << 1;
        attr |= (self.priority & 0x03) << 4;
        if self.h_flip {
            attr |= 0x40;
        }
        if self.v_flip {
            attr |= 0x80;
        }

        [
            self.x as i8 as u8,
            self.y as i8 as u8,
            self.tile_id as u8,
            attr,
        ]
    }

    /// Get the attribute byte (for display/debugging)
    pub fn get_attr_byte(&self) -> u8 {
        let mut attr: u8 = 0;
        attr |= (self.palette & 0x07) << 1;
        attr |= (self.priority & 0x03) << 4;
        if self.h_flip {
            attr |= 0x40;
        }
        if self.v_flip {
            attr |= 0x80;
        }
        attr
    }
}

/// Hitbox for collision detection
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Hitbox {
    pub x: i16,
    pub y: i16,
    pub w: u16,
    pub h: u16,
    pub damage: u8,
    pub stun: u8,
}

impl Hitbox {
    pub fn new(x: i16, y: i16, w: u16, h: u16) -> Self {
        Self {
            x,
            y,
            w,
            h,
            damage: 0,
            stun: 0,
        }
    }
}

/// A complete frame (one pose)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Frame {
    pub name: String,
    pub sprites: Vec<SpriteEntry>,
    pub width: u16,
    pub height: u16,
    pub hitbox: Option<Hitbox>,
    pub tileset1_id: u8,
    pub tileset2_id: u8,
    pub palette_id: u8,
    pub data_addr: u16,
}

impl Frame {
    /// Create a new empty frame
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            sprites: Vec::new(),
            width: 0,
            height: 0,
            hitbox: None,
            tileset1_id: 0,
            tileset2_id: 0,
            palette_id: 0,
            data_addr: 0,
        }
    }

    /// Parse a frame from ROM data
    pub fn from_bytes(data: &[u8], name: impl Into<String>) -> Result<Self, String> {
        let mut sprites = Vec::new();
        let mut i = 0;

        while i * 4 + 4 <= data.len() {
            let offset = i * 4;
            if let Some(entry) = SpriteEntry::from_bytes(&data[offset..offset + 4]) {
                sprites.push(entry);
                i += 1;
            } else {
                break;
            }
        }

        let mut frame = Self::new(name);
        frame.sprites = sprites;
        frame.calculate_bounds();

        Ok(frame)
    }

    /// Serialize this frame to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.sprites.len() * 4 + 4);

        for sprite in &self.sprites {
            bytes.extend_from_slice(&sprite.to_bytes());
        }

        // Add terminator
        bytes.extend_from_slice(&[0xC0, 0x00, 0x00, 0x00]);

        bytes
    }

    /// Add a sprite at position
    pub fn add_sprite(&mut self, tile_id: u16, x: i16, y: i16) -> &mut SpriteEntry {
        let entry = SpriteEntry::new(tile_id, x, y);
        self.sprites.push(entry);
        self.calculate_bounds();
        self.sprites.last_mut().unwrap()
    }

    /// Remove sprite at index
    pub fn remove_sprite(&mut self, index: usize) -> Option<SpriteEntry> {
        if index < self.sprites.len() {
            let removed = self.sprites.remove(index);
            self.calculate_bounds();
            Some(removed)
        } else {
            None
        }
    }

    /// Move a sprite to a new position
    pub fn move_sprite(&mut self, index: usize, x: i16, y: i16) -> Result<(), String> {
        if let Some(sprite) = self.sprites.get_mut(index) {
            sprite.x = x;
            sprite.y = y;
            self.calculate_bounds();
            Ok(())
        } else {
            Err("Invalid sprite index".to_string())
        }
    }

    /// Update sprite flags
    pub fn update_sprite_flags(
        &mut self,
        index: usize,
        h_flip: bool,
        v_flip: bool,
        palette: u8,
    ) -> Result<(), String> {
        if let Some(sprite) = self.sprites.get_mut(index) {
            sprite.h_flip = h_flip;
            sprite.v_flip = v_flip;
            sprite.palette = palette.min(7);
            Ok(())
        } else {
            Err("Invalid sprite index".to_string())
        }
    }

    /// Change the tile ID of a sprite
    pub fn set_sprite_tile(&mut self, index: usize, tile_id: u16) -> Result<(), String> {
        if let Some(sprite) = self.sprites.get_mut(index) {
            sprite.tile_id = tile_id;
            Ok(())
        } else {
            Err("Invalid sprite index".to_string())
        }
    }

    /// Duplicate a sprite
    pub fn duplicate_sprite(&mut self, index: usize) -> Result<usize, String> {
        if let Some(sprite) = self.sprites.get(index).cloned() {
            // Offset slightly so it's visible
            let mut new_sprite = sprite;
            new_sprite.x += 8;
            new_sprite.y += 8;
            self.sprites.push(new_sprite);
            self.calculate_bounds();
            Ok(self.sprites.len() - 1)
        } else {
            Err("Invalid sprite index".to_string())
        }
    }

    /// Calculate the bounding box of this frame
    pub fn calculate_bounds(&mut self) {
        if self.sprites.is_empty() {
            self.width = 0;
            self.height = 0;
            return;
        }

        let mut min_x = i16::MAX;
        let mut max_x = i16::MIN;
        let mut min_y = i16::MAX;
        let mut max_y = i16::MIN;

        for sprite in &self.sprites {
            min_x = min_x.min(sprite.x);
            max_x = max_x.max(sprite.x + 8);
            min_y = min_y.min(sprite.y);
            max_y = max_y.max(sprite.y + 8);
        }

        self.width = (max_x - min_x) as u16;
        self.height = (max_y - min_y) as u16;
    }

    /// Get the bounds as (x, y, width, height)
    pub fn get_bounds(&self) -> (i16, i16, u16, u16) {
        if self.sprites.is_empty() {
            return (0, 0, 0, 0);
        }

        let mut min_x = i16::MAX;
        let mut max_x = i16::MIN;
        let mut min_y = i16::MAX;
        let mut max_y = i16::MIN;

        for sprite in &self.sprites {
            min_x = min_x.min(sprite.x);
            max_x = max_x.max(sprite.x + 8);
            min_y = min_y.min(sprite.y);
            max_y = max_y.max(sprite.y + 8);
        }

        (min_x, min_y, (max_x - min_x) as u16, (max_y - min_y) as u16)
    }

    /// Render this frame to an image buffer
    pub fn render(
        &self,
        tiles: &[Tile],
        palette: &[Color],
        canvas_width: u32,
        canvas_height: u32,
        center_x: i16,
        center_y: i16,
    ) -> image::ImageBuffer<image::Rgba<u8>, Vec<u8>> {
        let mut img = image::ImageBuffer::from_pixel(
            canvas_width,
            canvas_height,
            image::Rgba([0, 0, 0, 0]), // Transparent background
        );

        for sprite in &self.sprites {
            let tile_idx = sprite.tile_id as usize;
            if let Some(tile) = tiles.get(tile_idx) {
                self.render_sprite(&mut img, sprite, tile, palette, center_x, center_y);
            }
        }

        img
    }

    fn render_sprite(
        &self,
        img: &mut image::ImageBuffer<image::Rgba<u8>, Vec<u8>>,
        sprite: &SpriteEntry,
        tile: &Tile,
        palette: &[Color],
        center_x: i16,
        center_y: i16,
    ) {
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

                let px = center_x + sprite.x + tx as i16;
                let py = center_y + sprite.y + ty as i16;

                if px >= 0 && px < img.width() as i16 && py >= 0 && py < img.height() as i16 {
                    img.put_pixel(
                        px as u32,
                        py as u32,
                        image::Rgba([color.r, color.g, color.b, 255]),
                    );
                }
            }
        }
    }

    /// Get a summary of this frame
    pub fn summary(&self) -> FrameSummary {
        FrameSummary {
            name: self.name.clone(),
            sprite_count: self.sprites.len(),
            width: self.width,
            height: self.height,
            has_hitbox: self.hitbox.is_some(),
        }
    }
}

/// Summary info for a frame (for UI lists)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameSummary {
    pub name: String,
    pub sprite_count: usize,
    pub width: u16,
    pub height: u16,
    pub has_hitbox: bool,
}

/// Frame data as received from/sent to the frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameData {
    pub name: String,
    pub sprites: Vec<SpriteEntry>,
    pub width: u16,
    pub height: u16,
    pub hitbox: Option<Hitbox>,
    pub tileset1_id: u8,
    pub tileset2_id: u8,
    pub palette_id: u8,
    pub data_addr: u16,
}

impl From<Frame> for FrameData {
    fn from(frame: Frame) -> Self {
        Self {
            name: frame.name,
            sprites: frame.sprites,
            width: frame.width,
            height: frame.height,
            hitbox: frame.hitbox,
            tileset1_id: frame.tileset1_id,
            tileset2_id: frame.tileset2_id,
            palette_id: frame.palette_id,
            data_addr: frame.data_addr,
        }
    }
}

impl From<FrameData> for Frame {
    fn from(data: FrameData) -> Self {
        Self {
            name: data.name,
            sprites: data.sprites,
            width: data.width,
            height: data.height,
            hitbox: data.hitbox,
            tileset1_id: data.tileset1_id,
            tileset2_id: data.tileset2_id,
            palette_id: data.palette_id,
            data_addr: data.data_addr,
        }
    }
}

/// Manager for fighter frames
pub struct FrameManager<'a> {
    rom: &'a Rom,
    boxer: &'a BoxerRecord,
}

impl<'a> FrameManager<'a> {
    pub fn new(rom: &'a Rom, boxer: &'a BoxerRecord) -> Self {
        Self { rom, boxer }
    }

    /// Load all frames for this fighter
    pub fn load_frames(&self) -> Result<Vec<Frame>, String> {
        // Get pose info from fighter data
        let fighter_manager = crate::fighter::BoxerManager::new(self.rom);
        let fighter_id = self.get_fighter_id()?;
        let poses = fighter_manager.get_poses(fighter_id);

        let mut frames = Vec::new();

        for (idx, pose) in poses.iter().enumerate() {
            let entries = fighter_manager.parse_meta_sprite(pose.data_addr);

            let mut frame = Frame::new(format!("Pose {}", idx));
            frame.sprites = entries
                .into_iter()
                .map(|e| SpriteEntry {
                    x: e.x as i16,
                    y: e.y as i16,
                    tile_id: e.tile as u16,
                    palette: (e.attr >> 1) & 0x07,
                    h_flip: (e.attr & 0x40) != 0,
                    v_flip: (e.attr & 0x80) != 0,
                    priority: (e.attr >> 4) & 0x03,
                })
                .collect();

            frame.tileset1_id = pose.tileset1_id;
            frame.tileset2_id = pose.tileset2_id;
            frame.palette_id = pose.palette_id;
            frame.data_addr = pose.data_addr;
            frame.calculate_bounds();

            frames.push(frame);
        }

        Ok(frames)
    }

    /// Load tiles for a specific pose
    pub fn load_tiles_for_pose(
        &self,
        pose: &crate::fighter::PoseInfo,
    ) -> Result<Vec<Tile>, String> {
        let mut all_tiles = Vec::new();

        // Load Tileset 1
        if pose.tileset1_id != 0 {
            if let Some(asset) = self.resolve_asset(pose.tileset1_id) {
                let tiles = self.load_asset_tiles(asset)?;
                all_tiles.extend(tiles);
            }
        }

        // Load Tileset 2
        if pose.tileset2_id != 0 {
            if let Some(asset) = self.resolve_asset(pose.tileset2_id) {
                let tiles = self.load_asset_tiles(asset)?;
                all_tiles.extend(tiles);
            }
        }

        Ok(all_tiles)
    }

    /// Load palette for a specific pose
    pub fn load_palette_for_pose(
        &self,
        pose: &crate::fighter::PoseInfo,
    ) -> Result<Vec<Color>, String> {
        let pal_asset = self
            .boxer
            .palette_files
            .first()
            .ok_or("No palette found for fighter")?;

        let pal_pc = parse_pc_offset(&pal_asset.start_pc)?;
        let pal_bytes = &self.rom.data[pal_pc..pal_pc + pal_asset.size];
        let full_palette = crate::palette::decode_palette(pal_bytes);

        // Use sub-palette from pose
        let pal_idx = (pose.palette_id & 0x07) as usize;
        let start = pal_idx * 16;

        if full_palette.len() >= start + 16 {
            Ok(full_palette[start..start + 16].to_vec())
        } else {
            Ok(full_palette)
        }
    }

    fn resolve_asset(&self, index: u8) -> Option<&manifest_core::AssetFile> {
        let hex_id = format!("{:02X}", index);
        let pattern = format!("Index {}", hex_id);

        self.boxer
            .shared_sprite_bins
            .iter()
            .chain(self.boxer.unique_sprite_bins.iter())
            .find(|a| a.filename.contains(&pattern))
    }

    fn load_asset_tiles(&self, asset: &manifest_core::AssetFile) -> Result<Vec<Tile>, String> {
        let pc = parse_pc_offset(&asset.start_pc)?;
        let data = &self.rom.data[pc..pc + asset.size];

        let gfx_data = if asset.category.contains("Compressed") {
            let mut decomp = Decompressor::new(data);
            decomp.decompress_interleaved(32 * 1024)
        } else {
            data.to_vec()
        };

        Ok(decode_4bpp_sheet(&gfx_data))
    }

    fn get_fighter_id(&self) -> Result<usize, String> {
        let name_lower = self.boxer.name.to_lowercase();
        let names = vec![
            "gabby jay",
            "bear hugger",
            "piston hurricane",
            "bald bull",
            "bob charlie",
            "dragon chan",
            "masked muscle",
            "mr. sandman",
            "aran ryan",
            "heike kagero",
            "mad clown",
            "super macho man",
            "narcis prince",
            "hoy quarlow",
            "rick bruiser",
            "nick bruiser",
        ];

        names
            .iter()
            .position(|n| n == &name_lower)
            .ok_or_else(|| "Fighter not found".to_string())
    }
}

fn parse_pc_offset(s: &str) -> Result<usize, String> {
    if s.starts_with("0x") {
        usize::from_str_radix(&s[2..], 16).map_err(|e| e.to_string())
    } else {
        s.parse::<usize>().map_err(|e| e.to_string())
    }
}

/// Render a frame to PNG bytes
pub fn render_frame_to_png(
    frame: &Frame,
    tiles: &[Tile],
    palette: &[Color],
) -> Result<Vec<u8>, String> {
    let img = frame.render(tiles, palette, 256, 256, 128, 128);

    let mut png_bytes = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut png_bytes);
    img.write_to(&mut cursor, image::ImageFormat::Png)
        .map_err(|e| e.to_string())?;

    Ok(png_bytes)
}
