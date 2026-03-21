//! Sprite Asset Commands
//!
//! Commands for sprite-bin export/import, diffing, and boxer sheet rendering.

use image::{ImageBuffer, Rgba};
use tauri::State;
use asset_core::fighter::BoxerManager;
use manifest_core::{AssetFile, BoxerRecord};

use crate::app_state::AppState;
use crate::utils::parse_offset;

use super::{
    current_tile_diff, decode_asset_tiles, find_asset_by_offset,
    find_boxer_by_key, first_subpalette, png_bytes as encode_png_bytes, read_current_asset_bytes,
    read_original_rom_bytes, read_palette_colors, render_tile_strip, save_png, set_pending_write,
    AssetResult, DEFAULT_TILE_STRIP_WIDTH,
};

fn resolve_tileset_asset(boxer: &BoxerRecord, tileset_id: u8) -> Option<&AssetFile> {
    let hex_id = format!("{:02X}", tileset_id);
    let pattern = format!("Index {}", hex_id);
    boxer
        .shared_sprite_bins
        .iter()
        .chain(boxer.unique_sprite_bins.iter())
        .find(|asset| asset.filename.contains(&pattern))
}

fn combined_sheet_png(bin_images: &[ImageBuffer<Rgba<u8>, Vec<u8>>]) -> AssetResult<Vec<u8>> {
    let width = bin_images.iter().map(|img| img.width()).max().unwrap_or(8);
    let height = bin_images
        .iter()
        .map(|img| img.height().max(8))
        .sum::<u32>()
        .max(8);

    let mut sheet = ImageBuffer::from_pixel(width, height, Rgba([0, 0, 0, 0]));
    let mut y_offset = 0u32;

    for img in bin_images {
        for y in 0..img.height() {
            for x in 0..img.width() {
                sheet.put_pixel(x, y + y_offset, *img.get_pixel(x, y));
            }
        }
        y_offset += img.height().max(8);
    }

    encode_png_bytes(&sheet)
}

#[tauri::command]
pub fn get_bin_original_bytes(
    state: State<AppState>,
    pc_offset: String,
    size: usize,
) -> AssetResult<Vec<u8>> {
    let asset_pc_offset = parse_offset(&pc_offset)?;
    read_original_rom_bytes(state.inner(), asset_pc_offset, size)
}

#[tauri::command]
pub fn export_sprite_bin_to_png(
    state: State<AppState>,
    pc_offset: String,
    size: usize,
    width_tiles: usize,
    palette_pc_offset: String,
    palette_size: usize,
    output_path: String,
) -> AssetResult<usize> {
    let asset_pc_offset = parse_offset(&pc_offset)?;
    let palette_pc = parse_offset(&palette_pc_offset)?;
    let manifest = state.manifest.lock();
    let asset = find_asset_by_offset(&manifest, asset_pc_offset)
        .ok_or_else(|| format!("Asset at {} not found in manifest", pc_offset))?;
    drop(manifest);

    let bytes = read_current_asset_bytes(state.inner(), asset_pc_offset, size)?;
    let tiles = decode_asset_tiles(&bytes, asset.category.contains("Compressed"))?;
    let palette = first_subpalette(&read_palette_colors(state.inner(), palette_pc, palette_size)?);
    let img = render_tile_strip(&tiles, &palette, width_tiles);
    save_png(&img, &output_path)?;
    Ok(tiles.len())
}

#[tauri::command]
pub fn import_sprite_bin_from_png(
    state: State<AppState>,
    pc_offset: String,
    original_size: usize,
    palette_pc_offset: String,
    palette_size: usize,
    png_path: String,
) -> AssetResult<(usize, usize, bool)> {
    let asset_pc_offset = parse_offset(&pc_offset)?;
    let palette_pc = parse_offset(&palette_pc_offset)?;
    let manifest = state.manifest.lock();
    let asset = find_asset_by_offset(&manifest, asset_pc_offset)
        .ok_or_else(|| format!("Asset at {} not found in manifest", pc_offset))?;
    drop(manifest);

    let palette = first_subpalette(&read_palette_colors(state.inner(), palette_pc, palette_size)?);
    let img = image::open(&png_path)
        .map_err(|e| format!("Failed to open PNG '{}': {}", png_path, e))?
        .to_rgba8();

    if img.width() % 8 != 0 || img.height() % 8 != 0 {
        return Err(format!(
            "PNG dimensions must be multiples of 8 pixels, got {}x{}",
            img.width(),
            img.height()
        ));
    }

    let tiles = asset_core::image_to_tiles(&img, &palette);
    let raw_bytes = asset_core::encode_4bpp_sheet(&tiles);
    let compressed = asset.category.contains("Compressed");
    let new_bytes = if compressed {
        super::compress_interleaved(&raw_bytes)
    } else {
        raw_bytes
    };
    let fits = new_bytes.len() <= original_size;

    set_pending_write(state.inner(), asset_pc_offset, new_bytes.clone());

    Ok((new_bytes.len(), original_size, fits))
}

#[tauri::command]
pub fn get_sprite_bin_diff(
    state: State<AppState>,
    pc_offset: String,
    size: usize,
) -> AssetResult<Vec<bool>> {
    let asset_pc_offset = parse_offset(&pc_offset)?;
    let manifest = state.manifest.lock();
    let asset = find_asset_by_offset(&manifest, asset_pc_offset)
        .ok_or_else(|| format!("Asset at {} not found in manifest", pc_offset))?;
    drop(manifest);

    let original_bytes = read_original_rom_bytes(state.inner(), asset_pc_offset, size)?;
    let current_bytes = read_current_asset_bytes(state.inner(), asset_pc_offset, size)?;
    let compressed = asset.category.contains("Compressed");

    let original_tiles = decode_asset_tiles(&original_bytes, compressed)?;
    let current_tiles = decode_asset_tiles(&current_bytes, compressed)?;
    Ok(current_tile_diff(&original_tiles, &current_tiles))
}

#[tauri::command]
pub fn get_fighter_tiles(
    state: State<AppState>,
    fighter_id: usize,
    tileset_id: usize,
) -> AssetResult<Vec<usize>> {
    let rom_guard = state.rom.lock();
    let rom = rom_guard.as_ref().ok_or("No ROM loaded")?;
    let fighter_list = BoxerManager::new(rom).get_boxer_list();
    let fighter = fighter_list
        .get(fighter_id)
        .ok_or_else(|| format!("Invalid fighter id {}", fighter_id))?;

    let manifest = state.manifest.lock();
    let boxer = manifest
        .fighters
        .get(&fighter.name)
        .ok_or_else(|| format!("Boxer '{}' not found in manifest", fighter.name))?;
    let asset = resolve_tileset_asset(boxer, tileset_id as u8)
        .ok_or_else(|| format!("Tileset {} not found for {}", tileset_id, fighter.name))?
        .clone();
    drop(manifest);
    drop(rom_guard);

    let asset_pc_offset = parse_offset(&asset.start_pc)?;
    let bytes = read_current_asset_bytes(state.inner(), asset_pc_offset, asset.size)?;
    let tiles = decode_asset_tiles(&bytes, asset.category.contains("Compressed"))?;
    Ok(vec![tiles.len()])
}

#[tauri::command]
pub fn render_sprite_sheet(
    state: State<AppState>,
    boxer_key: String,
    include_shared: bool,
) -> AssetResult<Vec<u8>> {
    let manifest = state.manifest.lock();
    let boxer = find_boxer_by_key(&manifest, &boxer_key)
        .ok_or_else(|| format!("Boxer '{}' not found in manifest", boxer_key))?;
    drop(manifest);

    let palette_asset = boxer
        .palette_files
        .first()
        .ok_or_else(|| format!("Boxer '{}' has no palette assets", boxer.name))?;
    let palette_pc = parse_offset(&palette_asset.start_pc)?;
    let palette = first_subpalette(&read_palette_colors(state.inner(), palette_pc, palette_asset.size)?);

    let mut bin_images = Vec::new();
    for bin in &boxer.unique_sprite_bins {
        let bin_pc = parse_offset(&bin.start_pc)?;
        let bytes = read_current_asset_bytes(state.inner(), bin_pc, bin.size)?;
        let tiles = decode_asset_tiles(&bytes, bin.category.contains("Compressed"))?;
        if !tiles.is_empty() {
            bin_images.push(render_tile_strip(&tiles, &palette, DEFAULT_TILE_STRIP_WIDTH));
        }
    }

    if include_shared {
        for bin in &boxer.shared_sprite_bins {
            let bin_pc = parse_offset(&bin.start_pc)?;
            let bytes = read_current_asset_bytes(state.inner(), bin_pc, bin.size)?;
            let tiles = decode_asset_tiles(&bytes, bin.category.contains("Compressed"))?;
            if !tiles.is_empty() {
                bin_images.push(render_tile_strip(&tiles, &palette, DEFAULT_TILE_STRIP_WIDTH));
            }
        }
    }

    if bin_images.is_empty() {
        return Err(format!("Boxer '{}' has no sprite bins to render", boxer.name));
    }

    combined_sheet_png(&bin_images)
}
