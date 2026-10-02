//! Generic Graphic Asset Commands
//!
//! Commands for exporting and importing portrait/icon style assets as PNGs.

use tauri::State;

use crate::app_state::AppState;
use crate::utils::parse_offset;

use super::{
    decode_asset_tiles, default_width_tiles, encode_tiles_for_asset, encode_tiles_to_snes_bytes,
    find_asset_by_offset, first_subpalette, icon_tiles_swap_layout, is_icon_asset,
    load_png_as_tiles, load_png_fitted_as_tiles, png_bytes, read_current_asset_bytes,
    read_palette_colors, render_tile_strip, save_png, set_pending_write, AssetResult,
    ImportOutcome,
};

// Stable Tauri IPC contract: keep the frontend-visible parameters explicit.
#[allow(clippy::too_many_arguments)]
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
    let mut tiles = decode_asset_tiles(&asset_bytes, compressed)?;
    if is_icon_asset(state.inner(), asset_pc_offset) {
        tiles = icon_tiles_swap_layout(&tiles);
    }
    let palette = first_subpalette(&read_palette_colors(
        state.inner(),
        palette_pc,
        palette_size,
    )?);
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
    let palette = first_subpalette(&read_palette_colors(
        state.inner(),
        palette_pc,
        palette_size,
    )?);
    let tiles = load_png_as_tiles(&png_path, &palette)?;
    Ok(encode_tiles_to_snes_bytes(&tiles))
}

/// Replace an icon or portrait with a picture file.
///
/// The picture does not need to be the right size or use the right colours: it
/// is fitted to the slot and matched to the boxer's palette automatically.
#[tauri::command]
pub fn import_graphic_asset_from_png(
    state: State<AppState>,
    pc_offset: String,
    original_size: usize,
    palette_pc_offset: String,
    palette_size: usize,
    png_path: String,
    width_tiles: Option<usize>,
) -> AssetResult<ImportOutcome> {
    let asset_pc_offset = parse_offset(&pc_offset)?;
    let palette_pc = parse_offset(&palette_pc_offset)?;
    let manifest = state.manifest.lock();
    let asset = find_asset_by_offset(&manifest, asset_pc_offset)
        .ok_or_else(|| format!("Asset at {} not found in manifest", pc_offset))?;
    drop(manifest);
    let compressed = asset.category.contains("Compressed");

    let palette = first_subpalette(&read_palette_colors(
        state.inner(),
        palette_pc,
        palette_size,
    )?);

    let current_bytes = read_current_asset_bytes(state.inner(), asset_pc_offset, original_size)?;
    let tile_count = decode_asset_tiles(&current_bytes, compressed)?.len();
    let width_tiles = width_tiles.unwrap_or_else(|| default_width_tiles(tile_count));
    let (mut tiles, fit) = load_png_fitted_as_tiles(&png_path, &palette, tile_count, width_tiles)?;
    if asset.subtype == "icon" {
        // The picture is in on-screen order; the game stores icons differently.
        tiles = icon_tiles_swap_layout(&tiles);
    }

    let new_bytes = encode_tiles_for_asset(&tiles, compressed);
    if new_bytes.len() > original_size {
        return Err(format!(
            "This picture has too much detail to fit in the game ({} bytes, but only {} are available). Try a simpler picture with fewer colors and bigger flat areas",
            new_bytes.len(), original_size
        ));
    }
    let new_size = new_bytes.len();
    set_pending_write(state.inner(), asset_pc_offset, new_bytes)?;

    Ok(ImportOutcome {
        new_size,
        original_size,
        fits: true,
        fit,
    })
}

/// Draw an icon or portrait from the current edited ROM as PNG bytes, so the
/// editor can show the picture itself instead of a file name.
#[tauri::command]
pub fn render_asset_preview(
    state: State<AppState>,
    pc_offset: String,
    size: usize,
    category: String,
    palette_pc_offset: String,
    palette_size: usize,
    width_tiles: Option<usize>,
) -> AssetResult<Vec<u8>> {
    let asset_pc_offset = parse_offset(&pc_offset)?;
    let palette_pc = parse_offset(&palette_pc_offset)?;
    let compressed = category.contains("Compressed");

    let asset_bytes = read_current_asset_bytes(state.inner(), asset_pc_offset, size)?;
    let mut tiles = decode_asset_tiles(&asset_bytes, compressed)?;
    if is_icon_asset(state.inner(), asset_pc_offset) {
        tiles = icon_tiles_swap_layout(&tiles);
    }
    let palette = first_subpalette(&read_palette_colors(
        state.inner(),
        palette_pc,
        palette_size,
    )?);
    let width_tiles = width_tiles.unwrap_or_else(|| default_width_tiles(tiles.len()));
    png_bytes(&render_tile_strip(&tiles, &palette, width_tiles))
}
