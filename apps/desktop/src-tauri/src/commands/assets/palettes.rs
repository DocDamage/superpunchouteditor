//! Palette Asset Commands
//!
//! Commands for reading and encoding palette data.

use tauri::State;

use asset_core::Color;

use crate::app_state::AppState;
use crate::utils::parse_offset;

use super::{encode_palette_bytes, read_palette_colors, AssetResult};

#[tauri::command]
pub fn get_palette(
    state: State<AppState>,
    pc_offset: String,
    size: usize,
) -> AssetResult<Vec<Color>> {
    let palette_pc_offset = parse_offset(&pc_offset)?;
    read_palette_colors(state.inner(), palette_pc_offset, size)
}

#[tauri::command]
pub fn encode_palette_for_preview(colors: Vec<Color>) -> AssetResult<Vec<u8>> {
    Ok(encode_palette_bytes(&colors))
}
