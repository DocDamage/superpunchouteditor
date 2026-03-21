//! Generic Graphic Asset Commands
//!
//! Commands for exporting and importing portrait/icon style assets as PNGs.

use tauri::State;

use crate::app_state::AppState;
use crate::utils::parse_offset;

use super::{
    decode_asset_tiles, encode_tiles_to_snes_bytes, first_subpalette, load_png_as_tiles,
    read_current_asset_bytes, read_palette_colors, render_tile_strip, save_png, AssetResult,
};

#[tauri::command]
pub fn export_asset_to_png(
    state: State<AppState>,
    pc_offset: String,
    size: usize,
    width_tiles: usize,
    category: String,
    palette_pc_offset: String,
    palette_size: usize,
    output_path: String,
) -> AssetResult<usize> {
    let asset_pc_offset = parse_offset(&pc_offset)?;
    let palette_pc = parse_offset(&palette_pc_offset)?;
    let compressed = category.contains("Compressed");

    let asset_bytes = read_current_asset_bytes(state.inner(), asset_pc_offset, size)?;
    let tiles = decode_asset_tiles(&asset_bytes, compressed)?;
    let palette = first_subpalette(&read_palette_colors(state.inner(), palette_pc, palette_size)?);
    let img = render_tile_strip(&tiles, &palette, width_tiles);
    save_png(&img, &output_path)?;
    Ok(tiles.len())
}

#[tauri::command]
pub fn import_asset_from_png(
    state: State<AppState>,
    png_path: String,
    palette_pc_offset: String,
    palette_size: usize,
) -> AssetResult<Vec<u8>> {
    let palette_pc = parse_offset(&palette_pc_offset)?;
    let palette = first_subpalette(&read_palette_colors(state.inner(), palette_pc, palette_size)?);
    let tiles = load_png_as_tiles(&png_path, &palette)?;
    Ok(encode_tiles_to_snes_bytes(&tiles))
}
