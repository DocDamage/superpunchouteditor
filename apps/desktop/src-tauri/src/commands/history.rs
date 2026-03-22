//! Undo / Redo / Clear-History Tauri Commands
//!
//! Exposes the `EditHistory` in `AppState` to the frontend under the command
//! names `undo`, `redo`, and `clear_history` — the names already used by
//! `editStore.ts` and `useStore.ts`.

use tauri::State;

use crate::app_state::AppState;
use crate::undo::EditAction;
use rom_core::Rom;

// ============================================================================
// COMMANDS
// ============================================================================

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

/// Clear all undo/redo history without affecting the ROM.
#[tauri::command]
pub fn clear_history(state: State<AppState>) -> Result<(), String> {
    state.edit_history.lock().clear();
    Ok(())
}

// ============================================================================
// HELPERS
// ============================================================================

/// Apply an `EditAction` to the ROM.
///
/// * `revert = true`  → write back `old_bytes` (undo)
/// * `revert = false` → write back `new_bytes` (redo)
fn apply_action(rom: &mut Rom, action: &EditAction, revert: bool) -> Result<(), String> {
    match action {
        EditAction::PaletteEdit { pc_offset, old_bytes, new_bytes, .. }
        | EditAction::SpriteBinEdit { pc_offset, old_bytes, new_bytes, .. }
        | EditAction::AssetImport { pc_offset, old_bytes, new_bytes, .. } => {
            let bytes = if revert { old_bytes } else { new_bytes };
            let offset = parse_hex_offset(pc_offset)?;
            rom.write_bytes(offset, bytes).map_err(|e| e.to_string())
        }
    }
}

/// Parse a hex offset string such as `"0x1A2B3C"` or `"1A2B3C"` to `usize`.
fn parse_hex_offset(s: &str) -> Result<usize, String> {
    let clean = s.trim_start_matches("0x").trim_start_matches("0X");
    usize::from_str_radix(clean, 16)
        .map_err(|e| format!("Invalid ROM offset '{}': {}", s, e))
}
