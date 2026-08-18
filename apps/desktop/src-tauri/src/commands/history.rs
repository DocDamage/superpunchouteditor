//! Canonical edit-history commands.
//!
//! Undo/redo are projections of `rom_core::EditJournal`. Legacy `record_*` calls are retained as
//! idempotent compatibility adapters while the frontend finishes migrating to mutation responses.

use chrono::Utc;
use tauri::State;

use crate::app_state::AppState;
use crate::undo::EditSummary;
use rom_core::EditOperation;

fn first_offset(transaction: &rom_core::EditTransaction) -> Option<String> {
    transaction.operations.iter().find_map(|operation| {
        if let EditOperation::WriteBytes { offset, .. } = operation {
            Some(format!("0x{offset:X}"))
        } else {
            None
        }
    })
}

#[tauri::command]
pub fn can_undo(state: State<AppState>) -> bool {
    state
        .rom_session
        .lock()
        .as_ref()
        .map(|session| session.journal().can_undo())
        .unwrap_or(false)
}

#[tauri::command]
pub fn can_redo(state: State<AppState>) -> bool {
    state
        .rom_session
        .lock()
        .as_ref()
        .map(|session| session.journal().can_redo())
        .unwrap_or(false)
}

#[tauri::command]
pub fn get_undo_stack(state: State<AppState>) -> Vec<EditSummary> {
    let guard = state.rom_session.lock();
    let Some(session) = guard.as_ref() else {
        return Vec::new();
    };
    session
        .journal()
        .active_transactions()
        .iter()
        .rev()
        .map(|transaction| EditSummary {
            id: transaction.id as usize,
            action_type: "Transaction".to_string(),
            description: transaction.label.clone(),
            pc_offset: first_offset(transaction),
            timestamp: Utc::now(),
        })
        .collect()
}

#[tauri::command]
pub fn get_redo_stack(state: State<AppState>) -> Vec<EditSummary> {
    let guard = state.rom_session.lock();
    let Some(session) = guard.as_ref() else {
        return Vec::new();
    };
    session
        .journal()
        .transactions()
        .iter()
        .skip(session.journal().cursor())
        .rev()
        .map(|transaction| EditSummary {
            id: transaction.id as usize,
            action_type: "Transaction".to_string(),
            description: transaction.label.clone(),
            pc_offset: first_offset(transaction),
            timestamp: Utc::now(),
        })
        .collect()
}

fn parse_hex_offset(value: &str) -> Result<usize, String> {
    let clean = value.trim_start_matches("0x").trim_start_matches("0X");
    usize::from_str_radix(clean, 16)
        .map_err(|error| format!("Invalid ROM offset '{value}': {error}"))
}

fn record_compat_edit(
    state: &AppState,
    label: String,
    pc_offset: String,
    old_bytes: Vec<u8>,
    new_bytes: Vec<u8>,
) -> Result<(), String> {
    if old_bytes.len() != new_bytes.len() {
        return Err("Compatibility history writes must preserve byte length".to_string());
    }
    if old_bytes == new_bytes {
        return Err("Edit does not change any bytes".to_string());
    }
    let offset = parse_hex_offset(&pc_offset)?;
    let current = state.materialize_current_rom()?.bytes;
    let range = rom_core::validate_range(offset, new_bytes.len(), current.len())
        .map_err(|error| error.to_string())?;
    let actual = &current[range];
    if actual == new_bytes.as_slice() {
        return Ok(());
    }
    if actual != old_bytes.as_slice() {
        return Err(format!(
            "History record conflicts with journal revision at {pc_offset}; expected old bytes do not match"
        ));
    }
    state.commit_rom_write(label, offset, new_bytes, None, None)?;
    Ok(())
}

#[tauri::command]
pub fn record_palette_edit(
    state: State<AppState>,
    pc_offset: String,
    color_index: usize,
    old_color: Vec<u8>,
    new_color: Vec<u8>,
) -> Result<(), String> {
    let base_offset = parse_hex_offset(&pc_offset)?;
    let byte_offset = base_offset
        .checked_add(color_index.saturating_mul(2))
        .ok_or("Palette color offset overflow")?;
    record_compat_edit(
        &state,
        format!("Palette color {color_index} at {pc_offset}"),
        format!("0x{byte_offset:X}"),
        old_color,
        new_color,
    )
}

#[tauri::command]
pub fn record_sprite_bin_edit(
    state: State<AppState>,
    pc_offset: String,
    old_bytes: Vec<u8>,
    new_bytes: Vec<u8>,
) -> Result<(), String> {
    record_compat_edit(
        &state,
        format!("Sprite edit at {pc_offset}"),
        pc_offset,
        old_bytes,
        new_bytes,
    )
}

#[tauri::command]
pub fn record_asset_import(
    state: State<AppState>,
    pc_offset: String,
    old_bytes: Vec<u8>,
    new_bytes: Vec<u8>,
    source_path: String,
) -> Result<(), String> {
    record_compat_edit(
        &state,
        format!("Imported {source_path} at {pc_offset}"),
        pc_offset,
        old_bytes,
        new_bytes,
    )
}

#[tauri::command]
pub fn undo(state: State<AppState>) -> Result<(), String> {
    state.undo_journal()?.ok_or("Nothing to undo")?;
    Ok(())
}

#[tauri::command]
pub fn redo(state: State<AppState>) -> Result<(), String> {
    state.redo_journal()?.ok_or("Nothing to redo")?;
    Ok(())
}

#[tauri::command]
pub fn clear_history(_state: State<AppState>) -> Result<(), String> {
    Err(
        "Clearing canonical edit history independently of the edited ROM is unsupported; save or start a new session"
            .to_string(),
    )
}
