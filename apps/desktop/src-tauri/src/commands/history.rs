//! Undo / redo compatibility commands.
//!
//! This module is retained while edit producers migrate to `rom_core::EditJournal`. It exposes the
//! complete frontend contract and never reports a history mutation unless an action is recorded.

use tauri::State;

use crate::app_state::AppState;
use crate::undo::{EditAction, EditSummary};
use rom_core::Rom;

#[tauri::command]
pub fn can_undo(state: State<AppState>) -> bool {
    state.edit_history.lock().can_undo()
}

#[tauri::command]
pub fn can_redo(state: State<AppState>) -> bool {
    state.edit_history.lock().can_redo()
}

#[tauri::command]
pub fn get_undo_stack(state: State<AppState>) -> Vec<EditSummary> {
    state.edit_history.lock().get_undo_summary()
}

#[tauri::command]
pub fn get_redo_stack(state: State<AppState>) -> Vec<EditSummary> {
    state.edit_history.lock().get_redo_summary()
}

#[tauri::command]
pub fn record_palette_edit(
    state: State<AppState>,
    pc_offset: String,
    color_index: usize,
    old_color: Vec<u8>,
    new_color: Vec<u8>,
) -> Result<(), String> {
    if old_color == new_color {
        return Err("Palette edit does not change any bytes".to_string());
    }
    state.edit_history.lock().push(EditAction::PaletteEdit {
        pc_offset: pc_offset.clone(),
        old_bytes: old_color,
        new_bytes: new_color,
        description: format!("Palette color {} at {}", color_index, pc_offset),
    });
    Ok(())
}

#[tauri::command]
pub fn record_sprite_bin_edit(
    state: State<AppState>,
    pc_offset: String,
    old_bytes: Vec<u8>,
    new_bytes: Vec<u8>,
) -> Result<(), String> {
    if old_bytes == new_bytes {
        return Err("Sprite edit does not change any bytes".to_string());
    }
    state.edit_history.lock().push(EditAction::SpriteBinEdit {
        pc_offset: pc_offset.clone(),
        old_bytes,
        new_bytes,
        description: format!("Sprite edit at {}", pc_offset),
    });
    Ok(())
}

#[tauri::command]
pub fn record_asset_import(
    state: State<AppState>,
    pc_offset: String,
    old_bytes: Vec<u8>,
    new_bytes: Vec<u8>,
    source_path: String,
) -> Result<(), String> {
    if old_bytes == new_bytes {
        return Err("Imported asset does not change any bytes".to_string());
    }
    state.edit_history.lock().push(EditAction::AssetImport {
        pc_offset: pc_offset.clone(),
        old_bytes,
        new_bytes,
        description: format!("Imported {} at {}", source_path, pc_offset),
    });
    Ok(())
}

/// Undo the last edit, writing the previous bytes back to the ROM.
#[tauri::command]
pub fn undo(state: State<AppState>) -> Result<(), String> {
    let action = state.edit_history.lock().undo();
    if let Some(action) = action {
        let mut rom_guard = state.rom.lock();
        let rom = rom_guard.as_mut().ok_or("No ROM loaded")?;
        apply_action(rom, &action, true)?;
    }
    Ok(())
}

/// Redo the last undone edit, writing the new bytes back to the ROM.
#[tauri::command]
pub fn redo(state: State<AppState>) -> Result<(), String> {
    let action = state.edit_history.lock().redo();
    if let Some(action) = action {
        let mut rom_guard = state.rom.lock();
        let rom = rom_guard.as_mut().ok_or("No ROM loaded")?;
        apply_action(rom, &action, false)?;
    }
    Ok(())
}

#[tauri::command]
pub fn clear_history(state: State<AppState>) -> Result<(), String> {
    state.edit_history.lock().clear();
    Ok(())
}

fn apply_action(rom: &mut Rom, action: &EditAction, revert: bool) -> Result<(), String> {
    match action {
        EditAction::PaletteEdit {
            pc_offset,
            old_bytes,
            new_bytes,
            ..
        }
        | EditAction::SpriteBinEdit {
            pc_offset,
            old_bytes,
            new_bytes,
            ..
        }
        | EditAction::AssetImport {
            pc_offset,
            old_bytes,
            new_bytes,
            ..
        } => {
            let bytes = if revert { old_bytes } else { new_bytes };
            let offset = parse_hex_offset(pc_offset)?;
            rom.write_bytes(offset, bytes).map_err(|e| e.to_string())
        }
    }
}

fn parse_hex_offset(s: &str) -> Result<usize, String> {
    let clean = s.trim_start_matches("0x").trim_start_matches("0X");
    usize::from_str_radix(clean, 16).map_err(|e| format!("Invalid ROM offset '{}': {}", s, e))
}
