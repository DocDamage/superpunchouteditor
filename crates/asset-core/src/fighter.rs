use crate::compression::Decompressor;
use crate::gfx::decode_4bpp_sheet;
use crate::palette::{decode_palette, Color};
use manifest_core::{AssetFile, BoxerRecord};
use rom_core::Rom;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BoxerMetadata {
    pub id: usize,
    pub name: String,
    pub header_addr: u16, // SNES Address in Bank 09
}

/// Deprecated: Use `BoxerMetadata` instead
#[deprecated(since = "0.1.0", note = "Use BoxerMetadata instead")]
pub type FighterMetadata = BoxerMetadata;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PoseInfo {
    pub index: usize,
    pub tileset1_id: u8,
    pub tileset2_id: u8,
    pub palette_id: u8,
    pub data_addr: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OamSpriteEntry {
    pub x: i8,
    pub y: i8,
    pub tile: u8,
    pub attr: u8,
}

pub struct BoxerManager<'a> {
    rom: &'a Rom,
}

/// Deprecated: Use `BoxerManager` instead
#[deprecated(since = "0.1.0", note = "Use BoxerManager instead")]
pub type FighterManager<'a> = BoxerManager<'a>;

impl<'a> BoxerManager<'a> {
    pub fn new(rom: &'a Rom) -> Self {
        Self { rom }
    }

    /// Get list of all boxers
    pub fn get_boxer_list(&self) -> Vec<BoxerMetadata> {
        self._get_boxer_list()
    }

    /// Internal implementation
    fn _get_boxer_list(&self) -> Vec<BoxerMetadata> {
        let names = vec![
            "Gabby Jay",
            "Bear Hugger",
            "Piston Hurricane",
            "Bald Bull",
            "Bob Charlie",
            "Dragon Chan",
            "Masked Muscle",
            "Mr. Sandman",
            "Aran Ryan",
            "Heike Kagero",
            "Mad Clown",
            "Super Macho Man",
            "Narcis Prince",
            "Hoy Quarlow",
            "Rick Bruiser",
            "Nick Bruiser",
        ];

        let mut boxers = Vec::new();
        for (i, name) in names.iter().enumerate() {
            let addr = 0x8000 + (i as u16 * 0x20);
            boxers.push(BoxerMetadata {
                id: i,
                name: name.to_string(),
                header_addr: addr,
            });
        }
        boxers
    }

    /// Deprecated: Use `get_boxer_list` instead
    #[deprecated(since = "0.1.0", note = "Use get_boxer_list instead")]
    pub fn get_fighter_list(&self) -> Vec<BoxerMetadata> {
        self.get_boxer_list()
    }

    /// Get poses for a boxer
    pub fn get_poses(&self, boxer_index: usize) -> Vec<PoseInfo> {
        self._get_poses(boxer_index)
    }

    fn _get_poses(&self, fighter_index: usize) -> Vec<PoseInfo> {
        let addr = 0x8000 + (fighter_index as u16 * 0x20);
        let pc_offset = self.rom.snes_to_pc(0x09, addr);

        let pose_table_ptr =
            u16::from_le_bytes([self.rom.data[pc_offset + 6], self.rom.data[pc_offset + 7]]);
        let pose_table_pc = self.rom.snes_to_pc(0x09, pose_table_ptr);

        let mut poses = Vec::new();
        for i in 0..128 {
            let entry_offset = pose_table_pc + i * 2;
            if entry_offset + 2 > self.rom.data.len() {
                break;
            }
            let pose_ptr =
                u16::from_le_bytes([self.rom.data[entry_offset], self.rom.data[entry_offset + 1]]);
            if pose_ptr < 0x8000 {
                break;
            }

            let pose_pc = self.rom.snes_to_pc(0x09, pose_ptr);
            if pose_pc + 5 > self.rom.data.len() {
                break;
            }
            poses.push(PoseInfo {
                index: i,
                tileset1_id: self.rom.data[pose_pc],
                tileset2_id: self.rom.data[pose_pc + 1],
                palette_id: self.rom.data[pose_pc + 2],
                data_addr: u16::from_le_bytes([
                    self.rom.data[pose_pc + 3],
                    self.rom.data[pose_pc + 4],
                ]),
            });
        }
        poses
    }

    pub fn parse_meta_sprite(&self, snes_addr: u16) -> Vec<OamSpriteEntry> {
        let pc = self.rom.snes_to_pc(0x09, snes_addr);
        let mut entries = Vec::new();
        let mut i = 0;
        loop {
            let offset = pc + i * 4;
            if offset + 4 > self.rom.data.len() {
                break;
            }

            let x = self.rom.data[offset] as i8;
            // The terminator is likely $C0 or similar. In some metasprites it's 0xF0.
            if self.rom.data[offset] == 0xC0 || self.rom.data[offset] == 0xF0 {
                break;
            }

            entries.push(OamSpriteEntry {
                x,
                y: self.rom.data[offset + 1] as i8,
                tile: self.rom.data[offset + 2],
                attr: self.rom.data[offset + 3],
            });
            i += 1;
            if i > 128 {
                break;
            }
        }
        entries
    }

    fn resolve_asset<'b>(&self, boxer: &'b BoxerRecord, index: u8) -> Option<&'b AssetFile> {
        let hex_id = format!("{:02X}", index);
        let pattern = format!("Index {}", hex_id);

        boxer
            .shared_sprite_bins
            .iter()
            .chain(boxer.unique_sprite_bins.iter())
            .find(|a| a.filename.contains(&pattern))
    }

    /// Render a boxer pose
    pub fn render_pose(
        &self,
        boxer_index: usize,
        pose_index: usize,
        boxer: &BoxerRecord,
    ) -> Result<Vec<u8>, String> {
        self._render_pose(boxer_index, pose_index, boxer)
    }

    fn _render_pose(
        &self,
        fighter_index: usize,
        pose_index: usize,
        boxer: &BoxerRecord,
    ) -> Result<Vec<u8>, String> {
        let poses = self.get_poses(fighter_index);
        let pose = poses.get(pose_index).ok_or("Pose index out of range")?;

        let mut all_tiles = Vec::new();

        // Load Tileset 1
        if pose.tileset1_id != 0 {
            if let Some(asset) = self.resolve_asset(boxer, pose.tileset1_id) {
                let pc =
                    usize::from_str_radix(&asset.start_pc[2..], 16).map_err(|e| e.to_string())?;
                let data = &self.rom.data[pc..pc + asset.size];
                let gfx_data = if asset.category.contains("Compressed") {
                    let mut decomp = Decompressor::new(data);
                    decomp.decompress_interleaved(32 * 1024)
                } else {
                    data.to_vec()
                };
                all_tiles.extend(decode_4bpp_sheet(&gfx_data));
            }
        }

        // Load Tileset 2
        if pose.tileset2_id != 0 {
            if let Some(asset) = self.resolve_asset(boxer, pose.tileset2_id) {
                let pc =
                    usize::from_str_radix(&asset.start_pc[2..], 16).map_err(|e| e.to_string())?;
                let data = &self.rom.data[pc..pc + asset.size];
                let gfx_data = if asset.category.contains("Compressed") {
                    let mut decomp = Decompressor::new(data);
                    decomp.decompress_interleaved(32 * 1024)
                } else {
                    data.to_vec()
                };
                all_tiles.extend(decode_4bpp_sheet(&gfx_data));
            }
        }

        // Get Palette
        let pal_asset = boxer
            .palette_files
            .first()
            .ok_or("No palette found for boxer")?;
        let pal_pc =
            usize::from_str_radix(&pal_asset.start_pc[2..], 16).map_err(|e| e.to_string())?;
        let pal_bytes = &self.rom.data[pal_pc..pal_pc + pal_asset.size];
        let full_palette = decode_palette(pal_bytes);

        // Use sub-palette from pose?
        // Usually index 0-7. Each subpalette is 16 colors.
        let pal_idx = (pose.palette_id & 0x07) as usize;
        let start = pal_idx * 16;
        let palette = if full_palette.len() >= start + 16 {
            full_palette[start..start + 16].to_vec()
        } else {
            full_palette.clone()
        };

        let entries = self.parse_meta_sprite(pose.data_addr);

        // Render to image
        let mut img = image::ImageBuffer::new(256, 256);
        for entry in entries {
            let tile_idx = entry.tile as usize;
            if let Some(tile) = all_tiles.get(tile_idx) {
                for ty in 0..8 {
                    for tx in 0..8 {
                        let px = 128 + entry.x as i32 + tx as i32;
                        let py = 128 + entry.y as i32 + ty as i32;
                        if px >= 0 && px < 256 && py >= 0 && py < 256 {
                            let color_idx = tile.pixels[ty * 8 + tx] as usize;
                            if color_idx != 0 {
                                let c = palette.get(color_idx).cloned().unwrap_or(Color {
                                    r: 0,
                                    g: 0,
                                    b: 0,
                                });
                                img.put_pixel(
                                    px as u32,
                                    py as u32,
                                    image::Rgba([c.r, c.g, c.b, 255]),
                                );
                            }
                        }
                    }
                }
            }
        }

        let mut png_bytes = Vec::new();
        let mut cursor = std::io::Cursor::new(&mut png_bytes);
        img.write_to(&mut cursor, image::ImageFormat::Png)
            .map_err(|e| e.to_string())?;
        Ok(png_bytes)
    }
}
