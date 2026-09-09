use crate::compression::Decompressor;
use crate::gfx::{decode_4bpp_sheet, Tile};
use crate::palette::{decode_palette, Color};
use manifest_core::{AssetFile, BoxerRecord};
use rom_core::Rom;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BoxerMetadata {
    pub id: usize,
    pub name: String,
    pub header_addr: u16, // SNES address in the fighter's graphics bank
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

#[derive(Debug, Clone, Copy)]
struct DecodedSprite {
    x: i16,
    y: i16,
    tile: u8,
    attr: u8,
    large: bool,
}

struct FighterGraphicsTables {
    source_pc: usize,
    bank_pc: usize,
    size_pc: usize,
    config_pc: usize,
}

struct ObjectTileTable {
    tiles: Vec<Option<Tile>>,
    fallback_banks: Vec<Vec<Tile>>,
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
            let addr = fighter_header_location(i)
                .map(|(_, address)| address)
                .unwrap_or(0x8000 + (i as u16 * 0x20));
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
        let Some((header_bank, addr)) = fighter_header_location(fighter_index) else {
            return Vec::new();
        };
        let pc_offset = self.rom.snes_to_pc(header_bank, addr);
        if pc_offset + 8 > self.rom.data.len() {
            return Vec::new();
        }

        let pose_table_ptr =
            u16::from_le_bytes([self.rom.data[pc_offset + 6], self.rom.data[pc_offset + 7]]);
        let pose_table_pc = self.rom.snes_to_pc(header_bank, pose_table_ptr);

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

            let pose_pc = self.rom.snes_to_pc(header_bank, pose_ptr);
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

            // These metasprite lists use a compact two-byte terminator in
            // most poses (`D2 C0`). A few lists leave the two bytes aligned
            // as the tile/attribute half of a final four-byte slot, so check
            // both positions before decoding the slot as a sprite.
            if (self.rom.data[offset] == 0xD2 && self.rom.data[offset + 1] == 0xC0)
                || (self.rom.data[offset + 2] == 0xD2 && self.rom.data[offset + 3] == 0xC0)
            {
                break;
            }

            let x = self.rom.data[offset] as i8;
            // Keep the older single-byte sentinels for regional/legacy data.
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

        let pose_bank = fighter_header_location(fighter_index)
            .map(|(bank, _)| bank)
            .ok_or("Fighter graphics header is unavailable")?;
        let data_start = self.rom.snes_to_pc(pose_bank, pose.data_addr);
        if data_start >= self.rom.data.len() {
            return Err("Pose data address is outside the ROM".to_string());
        }
        let data_end = poses
            .iter()
            .skip(pose_index + 1)
            .map(|next| self.rom.snes_to_pc(pose_bank, next.data_addr))
            .find(|next| *next > data_start && *next <= self.rom.data.len())
            .unwrap_or(self.rom.data.len());
        // The pose data is decoded by CODE_01F800 in the original game. Its
        // first pass expands compact coordinate commands into OAM entries;
        // the second pass expands the tile stream. The common match setup
        // uses palette/priority attribute $28 and an OAM origin of $80/$88.
        let sprites = parse_game_pose(&self.rom.data[data_start..data_end], 0x28, 0x80, 0x88);
        if sprites.is_empty() {
            return Err("Pose contains no drawable sprite entries".to_string());
        }
        // The three pose IDs are indices into the ROM's DMA source tables,
        // not filenames. Resolve each source address to the bank asset that
        // the game copies into the OBJ tile table, then apply the configured
        // VRAM destinations before looking up OAM tile numbers.
        let tile_table = self.build_object_tile_table(fighter_index, pose, boxer)?;

        // Get Palette
        let pal_asset = boxer
            .palette_files
            .first()
            .ok_or("No palette found for boxer")?;
        let pal_pc = parse_pc_offset(&pal_asset.start_pc)?;
        if pal_pc + pal_asset.size > self.rom.data.len() {
            return Err("Boxer palette extends beyond the ROM".to_string());
        }
        let pal_bytes = &self.rom.data[pal_pc..pal_pc + pal_asset.size];
        let palette = decode_palette(pal_bytes);

        // Render to image
        let mut img = image::ImageBuffer::from_pixel(256, 256, image::Rgba([8, 8, 12, 255]));
        for sprite in sprites {
            paint_game_sprite(&mut img, &tile_table, sprite, &palette);
        }

        let mut png_bytes = Vec::new();
        let mut cursor = std::io::Cursor::new(&mut png_bytes);
        img.write_to(&mut cursor, image::ImageFormat::Png)
            .map_err(|e| e.to_string())?;
        Ok(png_bytes)
    }

    fn graphics_tables(&self, fighter_index: usize) -> Result<FighterGraphicsTables, String> {
        let (header_bank, header_addr) = fighter_header_location(fighter_index)
            .ok_or("Fighter graphics header is unavailable")?;
        let header_pc = self.rom.snes_to_pc(header_bank, header_addr);
        if header_pc + 14 > self.rom.data.len() {
            return Err("Fighter graphics header is outside the ROM".to_string());
        }

        let source_ptr = read_u16(&self.rom.data, header_pc);
        let bank_ptr = read_u16(&self.rom.data, header_pc + 2);
        let size_ptr = read_u16(&self.rom.data, header_pc + 4);
        let config_ref_ptr = read_u16(&self.rom.data, header_pc + 12);
        let config_ref_pc = self.rom.snes_to_pc(header_bank, config_ref_ptr);
        if config_ref_pc + 2 > self.rom.data.len() {
            return Err("Fighter graphics config reference is outside the ROM".to_string());
        }
        let config_ptr = read_u16(&self.rom.data, config_ref_pc);

        Ok(FighterGraphicsTables {
            source_pc: self.rom.snes_to_pc(header_bank, source_ptr),
            bank_pc: self.rom.snes_to_pc(header_bank, bank_ptr),
            size_pc: self.rom.snes_to_pc(header_bank, size_ptr),
            config_pc: self.rom.snes_to_pc(header_bank, config_ptr),
        })
    }

    fn build_object_tile_table(
        &self,
        fighter_index: usize,
        pose: &PoseInfo,
        boxer: &BoxerRecord,
    ) -> Result<ObjectTileTable, String> {
        let tables = self.graphics_tables(fighter_index)?;
        let ids = [pose.tileset1_id, pose.tileset2_id, pose.palette_id];
        let mut table = ObjectTileTable {
            tiles: vec![None; 512],
            fallback_banks: Vec::new(),
        };

        for (channel, id) in ids.into_iter().enumerate() {
            if id == 0 {
                continue;
            }
            let index = usize::from(id - 1);
            let source_offset = tables
                .source_pc
                .checked_add(index * 2)
                .ok_or("Sprite source table offset overflow")?;
            let bank_offset = tables
                .bank_pc
                .checked_add(index)
                .ok_or("Sprite bank table offset overflow")?;
            let size_offset = tables
                .size_pc
                .checked_add(index * 2)
                .ok_or("Sprite size table offset overflow")?;
            if source_offset + 2 > self.rom.data.len()
                || bank_offset >= self.rom.data.len()
                || size_offset + 2 > self.rom.data.len()
            {
                return Err("Sprite source table entry is outside the ROM".to_string());
            }

            let source_addr = read_u16(&self.rom.data, source_offset);
            let source_bank = self.rom.data[bank_offset];
            let size_bytes = usize::from(read_u16(&self.rom.data, size_offset));
            let (chunk, full_bank) =
                self.load_source_graphics(boxer, source_bank, source_addr, size_bytes)?;
            if !full_bank.is_empty() {
                table.fallback_banks.push(full_bank);
            }

            let destination_offset = tables
                .config_pc
                .checked_add(channel * 2)
                .ok_or("Sprite VRAM config offset overflow")?;
            if destination_offset + 2 > self.rom.data.len() {
                return Err("Sprite VRAM config is outside the ROM".to_string());
            }
            let destination_word = read_u16(&self.rom.data, destination_offset);
            let destination_tile = usize::from(destination_word.saturating_sub(0x6000)) / 16;
            for (offset, tile) in chunk.into_iter().enumerate() {
                if let Some(slot) = table.tiles.get_mut(destination_tile + offset) {
                    *slot = Some(tile);
                }
            }
        }

        Ok(table)
    }

    fn load_source_graphics(
        &self,
        boxer: &BoxerRecord,
        source_bank: u8,
        source_addr: u16,
        size_bytes: usize,
    ) -> Result<(Vec<Tile>, Vec<Tile>), String> {
        let tile_count = size_bytes / 32;
        if tile_count == 0 {
            return Ok((Vec::new(), Vec::new()));
        }

        if source_bank == 0x7E || source_bank == 0x7F {
            if let Some((asset, origin_addr)) =
                self.compressed_asset_for_source(boxer, source_bank, source_addr)
            {
                let full_bank = self.load_asset_tiles(asset)?;
                let offset_bytes = usize::from(source_addr.saturating_sub(origin_addr));
                let start_tile = offset_bytes / 32;
                let chunk = full_bank
                    .iter()
                    .skip(start_tile)
                    .take(tile_count)
                    .cloned()
                    .collect();
                return Ok((chunk, full_bank));
            }
        }

        let source_long = (u32::from(source_bank) << 16) | u32::from(source_addr);
        if let Some(asset) = self.all_sprite_assets(boxer).into_iter().find(|asset| {
            if asset.subtype == "compressed_sprite_bin" {
                return false;
            }
            let Some(start) = parse_snes_address(&asset.start_snes) else {
                return false;
            };
            let Some(end) = parse_snes_address(&asset.end_snes) else {
                return false;
            };
            source_long >= start && source_long.saturating_add(size_bytes as u32) <= end
        }) {
            let full_bank = self.load_asset_tiles(asset)?;
            let asset_start = parse_snes_address(&asset.start_snes)
                .ok_or("Sprite asset has an invalid SNES start address")?;
            let offset_bytes = usize::try_from(source_long - asset_start)
                .map_err(|_| "Sprite source offset overflow")?;
            let start_tile = offset_bytes / 32;
            let chunk = full_bank
                .iter()
                .skip(start_tile)
                .take(tile_count)
                .cloned()
                .collect();
            return Ok((chunk, full_bank));
        }

        let source_pc = self.rom.snes_to_pc(source_bank, source_addr);
        if source_pc.saturating_add(size_bytes) > self.rom.data.len() {
            return Err(format!(
                "Sprite source ${source_bank:02X}:{source_addr:04X} extends beyond the ROM"
            ));
        }
        let full_bank = decode_4bpp_sheet(&self.rom.data[source_pc..source_pc + size_bytes]);
        Ok((full_bank.clone(), full_bank))
    }

    fn compressed_asset_for_source<'b>(
        &self,
        boxer: &'b BoxerRecord,
        source_bank: u8,
        source_addr: u16,
    ) -> Option<(&'b AssetFile, u16)> {
        let mut assets = self
            .all_sprite_assets(boxer)
            .into_iter()
            .filter(|asset| asset.subtype == "compressed_sprite_bin")
            .collect::<Vec<_>>();
        assets.sort_by_key(|asset| compressed_bank_number(asset).unwrap_or(u8::MAX));

        // The game decompresses the three graphics streams into fixed WRAM
        // windows.  Their order in the ROM header is therefore not the same
        // as the order in each fighter's source table (Bear/Piston/Mad Clown
        // are the obvious cases): stream 1 -> $7E:8000, stream 2 ->
        // $7F:0000, stream 3 -> $7F:8000.
        let origins = [(0x7E, 0x8000), (0x7F, 0x0000), (0x7F, 0x8000)];
        let origin_index = origins.iter().position(|(bank, addr)| {
            source_bank == *bank
                && source_addr >= *addr
                && source_addr < addr.saturating_add(0x8000)
        })?;
        let asset = assets.get(origin_index).copied()?;
        Some((asset, origins[origin_index].1))
    }

    fn all_sprite_assets<'b>(&self, boxer: &'b BoxerRecord) -> Vec<&'b AssetFile> {
        boxer
            .shared_sprite_bins
            .iter()
            .chain(boxer.unique_sprite_bins.iter())
            .collect()
    }

    fn load_asset_tiles(&self, asset: &AssetFile) -> Result<Vec<Tile>, String> {
        let pc = parse_pc_offset(&asset.start_pc)?;
        if pc.saturating_add(asset.size) > self.rom.data.len() {
            return Err(format!(
                "Sprite asset {} extends beyond the ROM",
                asset.filename
            ));
        }
        let data = &self.rom.data[pc..pc + asset.size];
        let gfx_data = if asset.category.contains("Compressed") {
            let mut decomp = Decompressor::new(data);
            decomp.decompress_sprite_graphics_exact().map_err(|error| {
                format!(
                    "SPO graphics decompression failed for {}: {error}",
                    asset.filename
                )
            })?
        } else {
            data.to_vec()
        };
        Ok(decode_4bpp_sheet(&gfx_data))
    }
}

/// The game stores four fighter graphics headers in each of the first four
/// fighter banks, but the roster order interleaves those banks. Keep the
/// editor's stable roster IDs separate from the ROM's local header offsets.
fn fighter_header_location(fighter_index: usize) -> Option<(u8, u16)> {
    Some(match fighter_index {
        0 => (0x09, 0x8000),  // Gabby Jay
        1 => (0x09, 0x8020),  // Bear Hugger
        2 => (0x0A, 0x8000),  // Piston Hurricane
        3 => (0x0A, 0x8020),  // Bald Bull
        4 => (0x09, 0x8040),  // Bob Charlie
        5 => (0x0B, 0x8000),  // Dragon Chan
        6 => (0x0B, 0x8020),  // Masked Muscle
        7 => (0x0A, 0x8040),  // Mr. Sandman
        8 => (0x0A, 0x8060),  // Aran Ryan
        9 => (0x0B, 0x8040),  // Heike Kagero
        10 => (0x09, 0x8060), // Mad Clown
        11 => (0x0B, 0x8060), // Super Macho Man
        12 => (0x0C, 0x8000), // Narcis Prince
        13 => (0x0D, 0x8000), // Hoy Quarlow
        14 => (0x0C, 0x8040), // Rick Bruiser
        15 => (0x0C, 0x8020), // Nick Bruiser
        _ => return None,
    })
}

fn read_u16(data: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([data[offset], data[offset + 1]])
}

fn read_signed(data: &[u8], cursor: &mut usize) -> i16 {
    let value = data.get(*cursor).copied().unwrap_or(0) as i8 as i16;
    *cursor += 1;
    value
}

fn transformed_x(raw: u8, mirrored: bool, small_x: bool, base_x: i16) -> i16 {
    let adjusted = if mirrored {
        // FB90 (the small-object helper) uses EB=$F9, while FBD9 (the
        // large-object helper) uses EA=$F1, with E9=$FF in both cases.
        let add = if small_x { 0xF9 } else { 0xF1 };
        raw.wrapping_neg().wrapping_sub(1).wrapping_add(add)
    } else {
        raw
    };
    adjusted as i8 as i16 + base_x
}

fn emit_row(
    coords: &mut Vec<(i16, i16, bool)>,
    target: usize,
    count_code: u8,
    x: i16,
    y: i16,
    step_x: i16,
    large: bool,
) {
    for index in 0..=usize::from(count_code) {
        if coords.len() < target {
            coords.push((x + step_x * index as i16, y, large));
        }
    }
}

fn emit_large_row(
    coords: &mut Vec<(i16, i16, bool)>,
    target: usize,
    count_code: u8,
    x: i16,
    f4: &mut i16,
    step_x: i16,
) {
    emit_row(coords, target, count_code, x, *f4, step_x, true);
    // CODE_01FBF5 adds $0F and then increments the result, advancing the
    // next 16x16 object's row by sixteen pixels.
    *f4 += 16;
}

fn emit_large_column(
    coords: &mut Vec<(i16, i16, bool)>,
    target: usize,
    count_code: u8,
    x: i16,
    f4: &mut i16,
) {
    for _ in 0..=usize::from(count_code) {
        if coords.len() < target {
            coords.push((x, *f4, true));
        }
        // CODE_01FD2C has the same add-$0F-then-increment sequence.
        *f4 += 16;
    }
}

fn emit_small_column(
    coords: &mut Vec<(i16, i16, bool)>,
    target: usize,
    count_code: u8,
    x: i16,
    e4: &mut i16,
) {
    for _ in 0..=usize::from(count_code) {
        if coords.len() < target {
            coords.push((x, *e4, false));
        }
        *e4 += 8;
    }
}

fn parse_game_pose(data: &[u8], initial_attr: u8, base_x: i16, base_y: i16) -> Vec<DecodedSprite> {
    if data.len() < 2 {
        return Vec::new();
    }

    let target = usize::from(data[0]);
    if target == 0 {
        return Vec::new();
    }

    let mirrored = initial_attr & 0x40 != 0;
    let mut cursor = 2usize;
    let mut coords = Vec::with_capacity(target);
    let mut f4 = base_y + data[1] as i8 as i16;
    let mut e4: i16;
    let mut ec = if mirrored { -16 } else { 16 };
    let mut ee = if mirrored { -8 } else { 8 };

    while coords.len() < target && cursor < data.len() {
        let command = data[cursor];
        cursor += 1;

        if command & 1 != 0 {
            if command & 2 != 0 {
                // F928: three packed large rows.
                let packed = command >> 2;
                emit_large_row(
                    &mut coords,
                    target,
                    command >> 6,
                    transformed_x(
                        read_signed(data, &mut cursor) as u8,
                        mirrored,
                        false,
                        base_x,
                    ),
                    &mut f4,
                    ec,
                );
                emit_large_row(
                    &mut coords,
                    target,
                    (packed >> 2) & 0x03,
                    transformed_x(
                        read_signed(data, &mut cursor) as u8,
                        mirrored,
                        false,
                        base_x,
                    ),
                    &mut f4,
                    ec,
                );
                emit_large_row(
                    &mut coords,
                    target,
                    packed & 0x03,
                    transformed_x(
                        read_signed(data, &mut cursor) as u8,
                        mirrored,
                        false,
                        base_x,
                    ),
                    &mut f4,
                    ec,
                );
            } else if command & 4 != 0 {
                if command & 8 != 0 {
                    // F906: repeated large rows with explicit x/y origins.
                    for _ in 0..=usize::from(command >> 4) {
                        let x = transformed_x(
                            read_signed(data, &mut cursor) as u8,
                            mirrored,
                            false,
                            base_x,
                        );
                        f4 = base_y + read_signed(data, &mut cursor);
                        emit_large_row(&mut coords, target, 0, x, &mut f4, ec);
                    }
                } else {
                    // F95E: reverse the row/column directions without
                    // consuming any payload bytes.
                    ec = (ec as u16 ^ 0xFFE0) as i16;
                    ee = (ee as u16 ^ 0xFFF0) as i16;
                }
            } else if command & 8 != 0 {
                // F91D: one small vertical column.
                let x = transformed_x(read_signed(data, &mut cursor) as u8, mirrored, true, base_x);
                let y = base_y + read_signed(data, &mut cursor);
                e4 = y;
                emit_small_column(&mut coords, target, command >> 4, x, &mut e4);
            } else {
                // F8CB: one small horizontal row.
                let x = transformed_x(read_signed(data, &mut cursor) as u8, mirrored, true, base_x);
                let y = base_y + read_signed(data, &mut cursor);
                e4 = y;
                emit_row(&mut coords, target, command >> 4, x, e4, ee, false);
            }
        } else if command & 2 == 0 {
            // F8EA: the low two command bits are clear.
            if command & 4 == 0 {
                if command & 8 != 0 {
                    // F93F: explicit x/y followed by a large row.
                    let x = transformed_x(
                        read_signed(data, &mut cursor) as u8,
                        mirrored,
                        false,
                        base_x,
                    );
                    f4 = base_y + read_signed(data, &mut cursor);
                    emit_large_row(&mut coords, target, command >> 4, x, &mut f4, ec);
                }
                // F99E is an intentional no-op.
            } else if command & 8 == 0 {
                // F950: explicit x/y followed by a large column.
                let x = transformed_x(
                    read_signed(data, &mut cursor) as u8,
                    mirrored,
                    false,
                    base_x,
                );
                f4 = base_y + read_signed(data, &mut cursor);
                emit_large_column(&mut coords, target, command >> 4, x, &mut f4);
            } else {
                // F8C1: explicit x followed by the current large row.
                let x = transformed_x(
                    read_signed(data, &mut cursor) as u8,
                    mirrored,
                    false,
                    base_x,
                );
                emit_large_row(&mut coords, target, command >> 4, x, &mut f4, ec);
            }
        } else if command & 4 == 0 {
            // F8DF/F8B5: large columns or two packed large rows.
            if command & 8 == 0 {
                let x = transformed_x(
                    read_signed(data, &mut cursor) as u8,
                    mirrored,
                    false,
                    base_x,
                );
                emit_large_column(&mut coords, target, command >> 4, x, &mut f4);
            } else {
                let first_count = command >> 6;
                let second_count = (command >> 4) & 0x03;
                let first_x = transformed_x(
                    read_signed(data, &mut cursor) as u8,
                    mirrored,
                    false,
                    base_x,
                );
                emit_large_row(&mut coords, target, first_count, first_x, &mut f4, ec);
                let second_x = transformed_x(
                    read_signed(data, &mut cursor) as u8,
                    mirrored,
                    false,
                    base_x,
                );
                emit_large_row(&mut coords, target, second_count, second_x, &mut f4, ec);
            }
        } else if command & 8 == 0 {
            // F973: three large rows with packed row lengths.
            let packed = data.get(cursor).copied().unwrap_or(0);
            cursor += 1;
            emit_large_row(
                &mut coords,
                target,
                command >> 4,
                transformed_x(
                    read_signed(data, &mut cursor) as u8,
                    mirrored,
                    false,
                    base_x,
                ),
                &mut f4,
                ec,
            );
            emit_large_row(
                &mut coords,
                target,
                packed >> 4,
                transformed_x(
                    read_signed(data, &mut cursor) as u8,
                    mirrored,
                    false,
                    base_x,
                ),
                &mut f4,
                ec,
            );
            emit_large_row(
                &mut coords,
                target,
                packed & 0x0F,
                transformed_x(
                    read_signed(data, &mut cursor) as u8,
                    mirrored,
                    false,
                    base_x,
                ),
                &mut f4,
                ec,
            );
        } else {
            // F8F7: repeated small rows with explicit x/y origins.
            for _ in 0..=usize::from(command >> 4) {
                let x = transformed_x(read_signed(data, &mut cursor) as u8, mirrored, true, base_x);
                let y = base_y + read_signed(data, &mut cursor);
                e4 = y;
                emit_row(&mut coords, target, 0, x, e4, ee, false);
            }
        }
    }

    let mut tiles = Vec::with_capacity(target);
    let mut attr = initial_attr;
    while tiles.len() < target && cursor < data.len() {
        let command = data[cursor];
        cursor += 1;

        if command & 0x80 != 0 {
            let value = command & 0x7F;
            match value & 0x07 {
                0 | 6 => {
                    for _ in 0..=usize::from(value >> 3) {
                        let tile = data.get(cursor).copied().unwrap_or(0);
                        cursor += 1;
                        if tiles.len() < target {
                            tiles.push((tile, attr));
                        }
                    }
                }
                1 => attr ^= 0x40,
                2 => {
                    let start = data.get(cursor).copied().unwrap_or(0);
                    cursor += 1;
                    for offset in 0..=usize::from(value >> 3) {
                        if tiles.len() < target {
                            tiles.push((start.wrapping_add(offset as u8), attr));
                        }
                    }
                }
                3 => {
                    let mask = data.get(cursor).copied().unwrap_or(0);
                    cursor += 1;
                    let value = data.get(cursor).copied().unwrap_or(0);
                    cursor += 1;
                    attr = (attr & mask) | value;
                }
                // $84/$85/$87 are the stream's no-op forms.
                4 | 5 | 7 => {}
                _ => unreachable!(),
            }
        } else if command & 0x40 != 0 {
            let value = command & 0x3F;
            let base = match value & 0x07 {
                0 => 0x10,
                1 => 0x30,
                2 => 0x50,
                3 => 0x70,
                4 => 0x90,
                5 => 0xB0,
                6 => 0xD0,
                _ => data.get(cursor).copied().unwrap_or(0),
            };
            if value & 0x07 == 7 {
                cursor += 1;
            }
            for offset in 0..=usize::from(value >> 3) {
                if tiles.len() < target {
                    tiles.push((base.wrapping_add((offset as u8).wrapping_mul(2)), attr));
                }
            }
        } else {
            let count = command >> 3;
            let base = match command & 0x07 {
                0 => 0x00,
                1 => 0x20,
                2 => 0x40,
                3 => 0x60,
                4 => 0x80,
                5 => 0xA0,
                6 => 0xC0,
                _ => data.get(cursor).copied().unwrap_or(0),
            };
            if command & 0x07 == 7 {
                cursor += 1;
            }
            for offset in 0..=usize::from(count) {
                if tiles.len() < target {
                    tiles.push((base.wrapping_add((offset as u8).wrapping_mul(2)), attr));
                }
            }
        }
    }

    let count = coords.len().min(tiles.len());
    let mut sprites = coords
        .into_iter()
        .zip(tiles)
        .take(count)
        .map(|((x, y, large), (tile, attr))| DecodedSprite {
            x,
            y,
            tile,
            attr,
            large,
        })
        .collect::<Vec<_>>();

    normalize_game_sprites(&mut sprites);
    sprites
}

fn normalize_game_sprites(sprites: &mut [DecodedSprite]) {
    if sprites.is_empty() {
        return;
    }

    let min_x = sprites
        .iter()
        .map(|sprite| i32::from(sprite.x))
        .min()
        .unwrap_or(0);
    let max_x = sprites
        .iter()
        .map(|sprite| i32::from(sprite.x) + if sprite.large { 16 } else { 8 })
        .max()
        .unwrap_or(0);
    let min_y = sprites
        .iter()
        .map(|sprite| i32::from(sprite.y))
        .min()
        .unwrap_or(0);
    let max_y = sprites
        .iter()
        .map(|sprite| i32::from(sprite.y) + if sprite.large { 16 } else { 8 })
        .max()
        .unwrap_or(0);
    let offset_x = 128 - (min_x + max_x) / 2;
    let offset_y = 128 - (min_y + max_y) / 2;

    for sprite in sprites {
        sprite.x = (i32::from(sprite.x) + offset_x) as i16;
        sprite.y = (i32::from(sprite.y) + offset_y) as i16;
    }
}

fn push_transparent_score(tile: &Tile) -> usize {
    tile.pixels.iter().filter(|pixel| **pixel != 0).count()
}

fn lookup_object_tile<'a>(
    table: &'a ObjectTileTable,
    tile_index: usize,
    attr: u8,
) -> Option<&'a Tile> {
    if let Some(Some(tile)) = table.tiles.get(tile_index) {
        return Some(tile);
    }

    let preferred = usize::from(attr & 1);
    let mut best: Option<&Tile> = None;
    let mut best_score = 0usize;
    for (index, bank) in table.fallback_banks.iter().enumerate() {
        let Some(tile) = bank.get(tile_index) else {
            continue;
        };
        let score = push_transparent_score(tile) + usize::from(index == preferred);
        if best.is_none() || score > best_score {
            best = Some(tile);
            best_score = score;
        }
    }
    best
}

fn paint_game_sprite(
    image: &mut image::ImageBuffer<image::Rgba<u8>, Vec<u8>>,
    table: &ObjectTileTable,
    sprite: DecodedSprite,
    palette: &[Color],
) {
    let object_size = if sprite.large { 2usize } else { 1usize };
    for row in 0..object_size {
        for column in 0..object_size {
            let tile_index = (usize::from(sprite.tile) + row * 16 + column) & 0x1FF;
            let Some(tile) = lookup_object_tile(table, tile_index, sprite.attr) else {
                continue;
            };
            let draw_column = if sprite.attr & 0x40 != 0 {
                object_size - 1 - column
            } else {
                column
            };
            let draw_row = if sprite.attr & 0x80 != 0 {
                object_size - 1 - row
            } else {
                row
            };

            for tile_y in 0..8usize {
                for tile_x in 0..8usize {
                    let source_x = if sprite.attr & 0x40 != 0 {
                        7 - tile_x
                    } else {
                        tile_x
                    };
                    let source_y = if sprite.attr & 0x80 != 0 {
                        7 - tile_y
                    } else {
                        tile_y
                    };
                    let color_index = usize::from(tile.pixels[source_y * 8 + source_x]);
                    if color_index == 0 {
                        continue;
                    }
                    let palette_index = ((usize::from(sprite.attr >> 1) & 0x07) * 16) + color_index;
                    let color = palette
                        .get(palette_index)
                        .or_else(|| palette.get(color_index))
                        .cloned()
                        .unwrap_or(Color { r: 0, g: 0, b: 0 });
                    let px = i32::from(sprite.x) + (draw_column * 8 + tile_x) as i32;
                    let py = i32::from(sprite.y) + (draw_row * 8 + tile_y) as i32;
                    if (0..256).contains(&px) && (0..256).contains(&py) {
                        image.put_pixel(
                            px as u32,
                            py as u32,
                            image::Rgba([color.r, color.g, color.b, 255]),
                        );
                    }
                }
            }
        }
    }
}

fn parse_pc_offset(value: &str) -> Result<usize, String> {
    let trimmed = value.trim();
    let digits = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
        .or_else(|| trimmed.strip_prefix('$'))
        .unwrap_or(trimmed);
    usize::from_str_radix(digits, 16).map_err(|error| error.to_string())
}

fn parse_snes_address(value: &str) -> Option<u32> {
    let trimmed = value.trim();
    let digits = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
        .or_else(|| trimmed.strip_prefix('$'))
        .unwrap_or(trimmed);
    u32::from_str_radix(digits, 16).ok()
}

fn compressed_bank_number(asset: &AssetFile) -> Option<u8> {
    let stem = asset
        .filename
        .strip_suffix(".bin")
        .unwrap_or(asset.filename.as_str());
    let digits: String = stem
        .chars()
        .rev()
        .take_while(|character| character.is_ascii_digit())
        .collect::<String>()
        .chars()
        .rev()
        .collect();
    (!digits.is_empty())
        .then(|| digits.parse::<u8>().ok())
        .flatten()
}

/// Match a pose's tileset id to the manifest asset that stores that bank.
///
/// Most raw sprite banks carry an explicit `IndexXX` token. The two large
/// compressed fighter banks in the USA/PAL manifests predate that naming
/// convention and are named with a trailing bank number instead (for example
/// `...GabbyJay2_BobCharlie2.bin`). Keep the explicit match first, then use
/// that trailing number only for compressed sprite-bank assets so icons and
/// portraits cannot be selected accidentally.
#[cfg(test)]
fn matches_tileset_asset(asset: &AssetFile, index: u8, patterns: &[String; 2]) -> bool {
    if patterns
        .iter()
        .any(|pattern| asset.filename.contains(pattern))
    {
        return true;
    }

    if asset.subtype != "compressed_sprite_bin" {
        return false;
    }

    let stem = asset
        .filename
        .strip_suffix(".bin")
        .unwrap_or(asset.filename.as_str());
    let trailing_digits: String = stem
        .chars()
        .rev()
        .take_while(|character| character.is_ascii_digit())
        .collect::<String>()
        .chars()
        .rev()
        .collect();

    !trailing_digits.is_empty() && trailing_digits.parse::<u8>().ok() == Some(index)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn asset(filename: &str, subtype: &str) -> AssetFile {
        AssetFile {
            file: filename.to_string(),
            filename: filename.to_string(),
            category: "Graphics/Compressed".to_string(),
            subtype: subtype.to_string(),
            size: 0,
            start_snes: String::new(),
            end_snes: String::new(),
            start_pc: String::new(),
            end_pc: String::new(),
            shared_with: Vec::new(),
        }
    }

    #[test]
    fn matches_legacy_trailing_number_for_compressed_sprite_banks() {
        let patterns = ["Index01".to_string(), "Index 01".to_string()];
        let compressed = asset(
            "GFX_Sprite_GabbyJay2_BobCharlie2.bin",
            "compressed_sprite_bin",
        );
        let icon = asset("GFX_Sprite_GabbyJayIcon.bin", "icon");

        assert!(matches_tileset_asset(&compressed, 2, &patterns));
        assert!(!matches_tileset_asset(&compressed, 1, &patterns));
        assert!(!matches_tileset_asset(&icon, 0, &patterns));
    }
}
