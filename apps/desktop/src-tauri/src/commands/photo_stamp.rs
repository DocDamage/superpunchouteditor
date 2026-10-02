//! Photo Stamp Commands
//!
//! Put a picture onto a boxer as they appear in the ring. The picture is
//! placed over an assembled pose; this module works out which stored tile
//! pixel the game shows at each spot and writes the picture back into those
//! tiles, in the colours that sprite is allowed to use.
//!
//! The whole stamp is one journal transaction, so one Undo removes it.

use std::collections::{BTreeMap, BTreeSet};

use asset_core::{
    compress_sprite_graphics_with_mask, decode_4bpp_sheet, decode_palette, encode_4bpp_sheet,
    sprite_stream_header_for_len, sprite_stream_len_from_header, BoxerManager, Color, Decompressor,
    PosePixelOwner, TileOrigin, POSE_CANVAS_SIZE,
};
use image::{ImageBuffer, Rgba};
use serde::Serialize;
use tauri::State;

use crate::app_state::AppState;
use crate::utils::parse_offset;

use super::assets::find_asset_by_offset;

/// Marker at the start of the error returned when a stamp has more detail than
/// the boxer's packed graphics have room for. The frontend looks for it and
/// retries with a simpler picture.
pub const STAMP_TOO_DETAILED: &str = "STAMP_TOO_DETAILED";

const TILE_BYTES: usize = 32;
const TILE_PIXELS: usize = 64;

/// New palette indices for some pixels of one tile; `None` leaves a pixel alone.
type TileEdit = [Option<u8>; TILE_PIXELS];

/// What a stamp changed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StampOutcome {
    /// Picture pixels that were written into the boxer.
    pub pixels_written: usize,
    /// Picture pixels that were not over any part of the boxer.
    pub pixels_skipped: usize,
    /// Number of 8x8 tiles that changed.
    pub tiles_changed: usize,
    /// Other poses of this boxer that show the same tiles and so changed too.
    pub other_poses_changed: Vec<usize>,
    /// Other boxers that share the changed graphics.
    pub also_affects: Vec<String>,
}

/// Closest allowed colour for a pixel. Index 0 is see-through and never chosen.
fn nearest_row_color(pixel: [u8; 3], palette: &[Color], palette_row: u8) -> u8 {
    let row_start = usize::from(palette_row) * 16;
    // The renderer falls back to the first row when a boxer palette is short.
    let row = if palette.len() >= row_start + 16 {
        &palette[row_start..row_start + 16]
    } else {
        &palette[..palette.len().min(16)]
    };

    let mut best = 1u8;
    let mut best_distance = u32::MAX;
    for (index, color) in row.iter().enumerate().skip(1) {
        let dr = i32::from(pixel[0]) - i32::from(color.r);
        let dg = i32::from(pixel[1]) - i32::from(color.g);
        let db = i32::from(pixel[2]) - i32::from(color.b);
        let distance = (2 * dr * dr + 4 * dg * dg + 3 * db * db) as u32;
        if distance < best_distance {
            best_distance = distance;
            best = index as u8;
        }
    }
    best
}

/// Work out, tile by tile, what the overlay changes. Returns the edits plus the
/// number of overlay pixels written and skipped.
fn collect_tile_edits(
    overlay: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    owners: &[Option<PosePixelOwner>],
    palette: &[Color],
) -> (BTreeMap<TileOrigin, TileEdit>, usize, usize) {
    let mut edits: BTreeMap<TileOrigin, TileEdit> = BTreeMap::new();
    let mut written = 0usize;
    let mut skipped = 0usize;

    for (x, y, pixel) in overlay.enumerate_pixels() {
        if pixel[3] < 128 {
            continue;
        }
        let Some(owner) = owners[y as usize * POSE_CANVAS_SIZE + x as usize] else {
            skipped += 1;
            continue;
        };
        let index = nearest_row_color([pixel[0], pixel[1], pixel[2]], palette, owner.palette_row);
        edits.entry(owner.origin).or_insert([None; TILE_PIXELS])[usize::from(owner.tile_pixel)] =
            Some(index);
        written += 1;
    }
    (edits, written, skipped)
}

/// Apply an edit to the 32 stored bytes of one tile.
fn edit_tile_bytes(bytes: &[u8], edit: &TileEdit) -> Result<Vec<u8>, String> {
    let mut tile = decode_4bpp_sheet(bytes)
        .into_iter()
        .next()
        .ok_or("Tile data is incomplete")?;
    for (pixel, value) in edit.iter().enumerate() {
        if let Some(value) = value {
            tile.pixels[pixel] = *value;
        }
    }
    Ok(encode_4bpp_sheet(&[tile]))
}

/// Rewrite a compressed graphics stream with some tiles edited. The new stream
/// uses the original grouping mask and must fit in `slot_size` bytes.
fn rebuild_compressed_stream(
    stream: &[u8],
    tile_edits: &BTreeMap<usize, TileEdit>,
    slot_size: usize,
) -> Result<Vec<u8>, String> {
    let mask = *stream.first().ok_or("Graphics stream is empty")?;
    let mut raw = Decompressor::new(stream).decompress_sprite_graphics_exact()?;

    for (tile_index, edit) in tile_edits {
        let start = tile_index
            .checked_mul(TILE_BYTES)
            .filter(|start| start + TILE_BYTES <= raw.len())
            .ok_or("A stamped tile lies outside its graphics block")?;
        let edited = edit_tile_bytes(&raw[start..start + TILE_BYTES], edit)?;
        raw[start..start + TILE_BYTES].copy_from_slice(&edited);
    }

    let rebuilt = compress_sprite_graphics_with_mask(&raw, mask)?;
    if rebuilt.len() > slot_size {
        return Err(format!(
            "{STAMP_TOO_DETAILED}: the picture needs {} more bytes than this boxer's graphics have room for",
            rebuilt.len() - slot_size
        ));
    }
    Ok(rebuilt)
}

/// Turn tile edits into the byte ranges to write.
fn plan_writes(
    rom: &[u8],
    edits: &BTreeMap<TileOrigin, TileEdit>,
    slot_size_of: impl Fn(usize) -> Option<usize>,
) -> Result<Vec<(usize, Vec<u8>)>, String> {
    let mut writes: Vec<(usize, Vec<u8>)> = Vec::new();
    let mut compressed_blocks: BTreeMap<usize, BTreeMap<usize, TileEdit>> = BTreeMap::new();

    for (origin, edit) in edits {
        if origin.compressed {
            compressed_blocks
                .entry(origin.block_pc)
                .or_default()
                .insert(origin.tile_index, *edit);
            continue;
        }
        let pc = origin
            .tile_index
            .checked_mul(TILE_BYTES)
            .and_then(|offset| origin.block_pc.checked_add(offset))
            .filter(|pc| pc + TILE_BYTES <= rom.len())
            .ok_or("A stamped tile lies outside the ROM")?;
        writes.push((pc, edit_tile_bytes(&rom[pc..pc + TILE_BYTES], edit)?));
    }

    for (block_pc, tile_edits) in compressed_blocks {
        let slot_size = slot_size_of(block_pc)
            .ok_or("A packed graphics block could not be found in the boxer list")?;
        let header_length = sprite_stream_len_from_header(rom, block_pc);
        let current_length = header_length
            .filter(|length| *length <= slot_size)
            .unwrap_or(slot_size);
        if block_pc + slot_size > rom.len() {
            return Err("A packed graphics block lies outside the ROM".to_string());
        }

        let rebuilt = rebuild_compressed_stream(
            &rom[block_pc..block_pc + current_length],
            &tile_edits,
            slot_size,
        )?;

        if rebuilt.len() != current_length {
            // The game finds the end of the stream from the two bytes in front
            // of it. Without that header a different length cannot be recorded.
            if header_length.is_none() {
                return Err(format!(
                    "{STAMP_TOO_DETAILED}: this part of the boxer cannot change size"
                ));
            }
            writes.push((
                block_pc - 2,
                sprite_stream_header_for_len(rebuilt.len())?.to_vec(),
            ));
        }
        writes.push((block_pc, rebuilt));
    }

    Ok(writes)
}

/// Stamp `overlay` (a `POSE_CANVAS_SIZE` square picture, transparent where
/// nothing should change) onto one pose of a boxer.
pub fn stamp_overlay_on_pose(
    state: &AppState,
    fighter_id: usize,
    pose_id: usize,
    overlay: &ImageBuffer<Rgba<u8>, Vec<u8>>,
) -> Result<StampOutcome, String> {
    let canvas = POSE_CANVAS_SIZE as u32;
    if overlay.width() != canvas || overlay.height() != canvas {
        return Err(format!(
            "The picture must be {canvas}x{canvas} pixels, got {}x{}",
            overlay.width(),
            overlay.height()
        ));
    }

    // Everything is planned against one snapshot of the current edited ROM.
    let rom = rom_core::Rom::new(state.materialize_current_rom()?.bytes);
    let manifest = state.manifest.lock().clone();
    let manager = BoxerManager::new(&rom);
    let fighter = manager
        .get_boxer_list()
        .get(fighter_id)
        .cloned()
        .ok_or_else(|| format!("Invalid fighter id {fighter_id}"))?;
    let boxer = manifest
        .fighters
        .get(&fighter.name)
        .ok_or_else(|| format!("Boxer '{}' not found in manifest", fighter.name))?;

    let palette_asset = boxer
        .palette_files
        .first()
        .ok_or("This boxer has no colors to draw with")?;
    let palette_pc = parse_offset(&palette_asset.start_pc)?;
    let palette_range = rom_core::validate_range(palette_pc, palette_asset.size, rom.data.len())
        .map_err(|error| error.to_string())?;
    let palette = decode_palette(&rom.data[palette_range]);
    if palette.len() < 2 {
        return Err("This boxer has no colors to draw with".to_string());
    }

    let owners = manager.pose_pixel_owners(fighter_id, pose_id, boxer)?;
    let (edits, pixels_written, pixels_skipped) = collect_tile_edits(overlay, &owners, &palette);
    if pixels_written == 0 {
        return Err(
            "The picture is not over the boxer. Move it onto the boxer and try again".to_string(),
        );
    }

    let writes = plan_writes(&rom.data, &edits, |block_pc| {
        find_asset_by_offset(&manifest, block_pc).map(|asset| asset.size)
    })?;

    let label = format!("Photo on {} (pose {})", fighter.name, pose_id + 1);
    state.commit_rom_transform(label, |scratch| {
        for (pc, bytes) in &writes {
            let range = rom_core::validate_range(*pc, bytes.len(), scratch.data.len())
                .map_err(|error| error.to_string())?;
            scratch.data[range].copy_from_slice(bytes);
        }
        Ok(())
    })?;

    // Other poses that draw any of the changed tiles now look different too.
    let changed: BTreeSet<TileOrigin> = edits.keys().copied().collect();
    let pose_count = manager.get_poses(fighter_id).len();
    let other_poses_changed = (0..pose_count)
        .filter(|index| *index != pose_id)
        .filter(|index| {
            manager
                .pose_pixel_owners(fighter_id, *index, boxer)
                .map(|pose_owners| {
                    pose_owners
                        .iter()
                        .flatten()
                        .any(|owner| owner.opaque && changed.contains(&owner.origin))
                })
                .unwrap_or(false)
        })
        .collect();

    let mut also_affects: BTreeSet<String> = BTreeSet::new();
    for origin in &changed {
        if let Some(asset) = find_asset_by_offset(&manifest, origin.block_pc) {
            for name in &asset.shared_with {
                if !name.eq_ignore_ascii_case(&fighter.name) {
                    also_affects.insert(name.clone());
                }
            }
        }
    }

    Ok(StampOutcome {
        pixels_written,
        pixels_skipped,
        tiles_changed: edits.len(),
        other_poses_changed,
        also_affects: also_affects.into_iter().collect(),
    })
}

/// Put a picture onto a boxer's pose. `overlay_png` is a PNG the size of the
/// pose canvas, transparent everywhere except where the picture goes.
#[tauri::command]
pub fn stamp_photo_on_pose(
    state: State<AppState>,
    fighter_id: usize,
    pose_id: usize,
    overlay_png: Vec<u8>,
) -> Result<StampOutcome, String> {
    // A 256x256 PNG is small; anything large is not a stamp from this app.
    if overlay_png.len() > 2 * 1024 * 1024 {
        return Err("The picture data is too large".to_string());
    }
    let overlay = image::load_from_memory_with_format(&overlay_png, image::ImageFormat::Png)
        .map_err(|error| format!("The picture could not be read: {error}"))?
        .to_rgba8();
    stamp_overlay_on_pose(state.inner(), fighter_id, pose_id, &overlay)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn palette() -> Vec<Color> {
        let mut colors = vec![Color::new(0, 0, 0); 32];
        colors[1] = Color::new(255, 0, 0);
        colors[2] = Color::new(0, 255, 0);
        colors[17] = Color::new(0, 0, 255);
        colors[18] = Color::new(255, 255, 255);
        colors
    }

    fn origin(block_pc: usize, tile_index: usize, compressed: bool) -> TileOrigin {
        TileOrigin {
            block_pc,
            tile_index,
            compressed,
        }
    }

    #[test]
    fn nearest_color_stays_in_the_sprites_row_and_skips_see_through() {
        let palette = palette();
        assert_eq!(nearest_row_color([250, 10, 10], &palette, 0), 1);
        assert_eq!(nearest_row_color([10, 250, 10], &palette, 0), 2);
        assert_eq!(nearest_row_color([10, 10, 250], &palette, 1), 1);
        assert_eq!(nearest_row_color([250, 250, 250], &palette, 1), 2);
        // Black matches the see-through slot exactly but must not use it.
        assert_ne!(nearest_row_color([0, 0, 0], &palette, 0), 0);
        // A row the palette does not have falls back to the first row.
        assert_eq!(nearest_row_color([250, 10, 10], &palette, 5), 1);
    }

    #[test]
    fn overlay_pixels_map_to_the_owning_tile_pixel() {
        let mut owners = vec![None; POSE_CANVAS_SIZE * POSE_CANVAS_SIZE];
        let tile = origin(0x100, 3, false);
        owners[5 * POSE_CANVAS_SIZE + 4] = Some(PosePixelOwner {
            origin: tile,
            tile_pixel: 9,
            palette_row: 0,
            opaque: true,
        });

        let mut overlay = ImageBuffer::from_pixel(
            POSE_CANVAS_SIZE as u32,
            POSE_CANVAS_SIZE as u32,
            Rgba([0, 0, 0, 0]),
        );
        overlay.put_pixel(4, 5, Rgba([0, 255, 0, 255])); // over the boxer
        overlay.put_pixel(100, 100, Rgba([0, 255, 0, 255])); // over nothing

        let (edits, written, skipped) = collect_tile_edits(&overlay, &owners, &palette());
        assert_eq!((written, skipped), (1, 1));
        assert_eq!(edits.len(), 1);
        assert_eq!(edits[&tile][9], Some(2));
        assert_eq!(edits[&tile].iter().flatten().count(), 1);
    }

    #[test]
    fn plain_tiles_are_rewritten_in_place() {
        let rom = vec![0u8; 0x200];
        let mut edit = [None; TILE_PIXELS];
        edit[0] = Some(5);
        let mut edits = BTreeMap::new();
        edits.insert(origin(0x100, 2, false), edit);

        let writes = plan_writes(&rom, &edits, |_| None).unwrap();
        assert_eq!(writes.len(), 1);
        assert_eq!(writes[0].0, 0x100 + 2 * TILE_BYTES);
        let tile = &decode_4bpp_sheet(&writes[0].1)[0];
        assert_eq!(tile.pixels[0], 5);
        assert!(tile.pixels[1..].iter().all(|pixel| *pixel == 0));
    }

    /// A ROM with one compressed block of flat tiles at the start of a bank.
    fn rom_with_packed_block(tile_count: usize) -> (Vec<u8>, usize, usize) {
        let block_pc = 0x8002;
        let stream =
            compress_sprite_graphics_with_mask(&vec![0u8; tile_count * TILE_BYTES], 0x0F).unwrap();
        let mut rom = vec![0xEEu8; 0x10000];
        rom[0x8000..0x8002].copy_from_slice(&sprite_stream_header_for_len(stream.len()).unwrap());
        rom[block_pc..block_pc + stream.len()].copy_from_slice(&stream);
        (rom, block_pc, stream.len())
    }

    #[test]
    fn packed_blocks_are_rewritten_and_decode_to_the_edited_tiles() {
        let (rom, block_pc, original_len) = rom_with_packed_block(4);
        let mut edit = [None; TILE_PIXELS];
        edit[10] = Some(7);
        let mut edits = BTreeMap::new();
        edits.insert(origin(block_pc, 1, true), edit);

        // A generous slot: the edited stream is allowed to be longer.
        let writes = plan_writes(&rom, &edits, |_| Some(original_len + 64)).unwrap();
        let mut patched = rom.clone();
        for (pc, bytes) in &writes {
            patched[*pc..*pc + bytes.len()].copy_from_slice(bytes);
        }

        let new_len = sprite_stream_len_from_header(&patched, block_pc).unwrap();
        assert!(new_len > original_len);
        let raw = Decompressor::new(&patched[block_pc..block_pc + new_len])
            .decompress_sprite_graphics_exact()
            .unwrap();
        let tiles = decode_4bpp_sheet(&raw);
        assert_eq!(tiles.len(), 4);
        assert_eq!(tiles[1].pixels[10], 7);
        assert_eq!(
            tiles[1].pixels.iter().filter(|pixel| **pixel != 0).count(),
            1
        );
        assert!(tiles[0].pixels.iter().all(|pixel| *pixel == 0));
    }

    #[test]
    fn a_stamp_that_does_not_fit_is_refused_without_writing() {
        let (rom, block_pc, original_len) = rom_with_packed_block(4);
        let mut edit = [None; TILE_PIXELS];
        for (pixel, slot) in edit.iter_mut().enumerate() {
            *slot = Some((pixel % 15 + 1) as u8);
        }
        let mut edits = BTreeMap::new();
        edits.insert(origin(block_pc, 0, true), edit);

        // No spare room at all: the slot is exactly the original stream.
        let error = plan_writes(&rom, &edits, |_| Some(original_len)).unwrap_err();
        assert!(error.starts_with(STAMP_TOO_DETAILED));
    }

    /// Opt-in check against a user-supplied USA ROM (`SPO_USA_ROM`): stamp a
    /// simple picture over each boxer's head in their first pose, confirm the
    /// drawn pose changes, and confirm one Undo restores the ROM exactly.
    /// Set `SPO_PROBE_DIR` to also save before/after pictures there.
    #[test]
    #[ignore]
    fn usa_photo_stamp_changes_the_pose_and_undo_restores_the_rom() {
        let rom_path = std::env::var("SPO_USA_ROM").expect("set SPO_USA_ROM to a local USA ROM");
        let rom = rom_core::Rom::load(&rom_path).unwrap();
        let region = rom.detect_region().expect("recognised region");
        let manifest = crate::utils::load_manifest_for_region(region, None).unwrap();
        let state = AppState::new(manifest.clone());
        state.install_rom_session(rom, rom_path);
        let original = state.materialize_current_rom().unwrap().bytes;
        let probe_dir = std::env::var("SPO_PROBE_DIR").ok();

        let fighter_count = {
            let rom = rom_core::Rom::new(original.clone());
            BoxerManager::new(&rom).get_boxer_list().len()
        };
        let mut stamped = 0;
        for fighter_id in 0..fighter_count.min(16) {
            let rom = rom_core::Rom::new(original.clone());
            let manager = BoxerManager::new(&rom);
            let fighter = manager.get_boxer_list()[fighter_id].clone();
            let Some(boxer) = manifest.fighters.get(&fighter.name) else {
                continue;
            };
            let owners = manager.pose_pixel_owners(fighter_id, 0, boxer).unwrap();

            // The boxer's outline, then a head-sized box at the top middle.
            let mut min_x = usize::MAX;
            let mut max_x = 0;
            let mut min_y = usize::MAX;
            let mut max_y = 0;
            for (index, owner) in owners.iter().enumerate() {
                if owner.is_some_and(|owner| owner.opaque) {
                    let (x, y) = (index % POSE_CANVAS_SIZE, index / POSE_CANVAS_SIZE);
                    min_x = min_x.min(x);
                    max_x = max_x.max(x);
                    min_y = min_y.min(y);
                    max_y = max_y.max(y);
                }
            }
            let centre_x = (min_x + max_x) / 2;
            let side = ((max_y - min_y) / 5).max(12);

            // A simple smiley: flat areas, a few colours, like a cartoon photo.
            let mut overlay = ImageBuffer::from_pixel(
                POSE_CANVAS_SIZE as u32,
                POSE_CANVAS_SIZE as u32,
                Rgba([0, 0, 0, 0]),
            );
            for dy in 0..side {
                for dx in 0..side {
                    let (fx, fy) = (dx as f32 / side as f32 - 0.5, dy as f32 / side as f32 - 0.5);
                    if fx * fx + fy * fy > 0.25 {
                        continue;
                    }
                    let eye = (fy + 0.12).abs() < 0.07 && ((fx.abs() - 0.18).abs() < 0.07);
                    let mouth = (fy - 0.2).abs() < 0.05 && fx.abs() < 0.2;
                    let color = if eye || mouth {
                        Rgba([20, 20, 20, 255])
                    } else {
                        Rgba([250, 210, 60, 255])
                    };
                    overlay.put_pixel(
                        (centre_x - side / 2 + dx) as u32,
                        (min_y + dy) as u32,
                        color,
                    );
                }
            }

            let before = manager.render_pose(fighter_id, 0, boxer).unwrap();
            match stamp_overlay_on_pose(&state, fighter_id, 0, &overlay) {
                Ok(outcome) => {
                    stamped += 1;
                    let after_rom =
                        rom_core::Rom::new(state.materialize_current_rom().unwrap().bytes);
                    let after = BoxerManager::new(&after_rom)
                        .render_pose(fighter_id, 0, boxer)
                        .unwrap();
                    assert_ne!(
                        before, after,
                        "{}: the drawn pose should change",
                        fighter.name
                    );
                    println!(
                        "{:18} OK   wrote {:4} px in {:3} tiles, {:2} other poses, shares with {:?}",
                        fighter.name,
                        outcome.pixels_written,
                        outcome.tiles_changed,
                        outcome.other_poses_changed.len(),
                        outcome.also_affects
                    );
                    if let Some(dir) = &probe_dir {
                        let name = fighter.name.replace([' ', '.'], "");
                        std::fs::write(
                            std::path::Path::new(dir).join(format!("{name}-after.png")),
                            &after,
                        )
                        .unwrap();
                    }
                    state.undo_journal().unwrap().unwrap();
                    assert_eq!(
                        state.materialize_current_rom().unwrap().bytes,
                        original,
                        "{}: Undo must restore the ROM exactly",
                        fighter.name
                    );
                }
                Err(error) => {
                    println!("{:18} FAIL {error}", fighter.name);
                    assert_eq!(state.materialize_current_rom().unwrap().bytes, original);
                }
            }
        }
        assert!(
            stamped > 0,
            "at least one boxer should accept a simple stamp"
        );
    }

    #[test]
    fn wrong_sized_overlays_are_rejected() {
        let state = AppState::new(manifest_core::Manifest::empty());
        let overlay = ImageBuffer::from_pixel(32, 32, Rgba([0, 0, 0, 255]));
        assert!(stamp_overlay_on_pose(&state, 0, 0, &overlay).is_err());
    }
}
