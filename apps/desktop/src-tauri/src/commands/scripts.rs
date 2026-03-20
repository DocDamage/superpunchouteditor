//! Script/AI Commands
//!
//! Commands for script reading and fighter parameter editing.

use tauri::State;

use crate::app_state::AppState;
use script_core::{
    BoxerHeader, EditableFighterParams, ParamValidationResult, ScriptReader, ScriptRecord,
};

/// Get all script records
#[tauri::command]
pub fn get_all_scripts(state: State<AppState>) -> Result<Vec<ScriptRecord>, String> {
    let rom_opt = state.rom.lock();
    let rom = rom_opt.as_ref().ok_or("No ROM loaded")?;
    let reader = ScriptReader::new(rom);
    Ok(reader.get_all_scripts())
}

/// Get scripts for a specific fighter
#[tauri::command]
pub fn get_scripts_for_fighter(
    state: State<AppState>,
    fighter_name: String,
) -> Result<Vec<ScriptRecord>, String> {
    let rom_opt = state.rom.lock();
    let rom = rom_opt.as_ref().ok_or("No ROM loaded")?;
    let reader = ScriptReader::new(rom);
    Ok(reader.get_scripts_for_fighter(&fighter_name))
}

/// Get fighter header
#[tauri::command]
pub fn get_fighter_header(
    state: State<AppState>,
    _fighter_index: usize,
) -> Result<BoxerHeader, String> {
    let rom_opt = state.rom.lock();
    let rom = rom_opt.as_ref().ok_or("No ROM loaded")?;
    let _reader = ScriptReader::new(rom);
    // TODO: Check if decode_boxer_header method exists
    // For now, return an error
    Err("decode_fighter_header not available".to_string())
}

/// Get editable fighter parameters
#[tauri::command]
pub fn get_editable_fighter_params(
    state: State<AppState>,
    fighter_index: usize,
) -> Result<EditableFighterParams, String> {
    let rom_opt = state.rom.lock();
    let rom = rom_opt.as_ref().ok_or("No ROM loaded")?;
    let reader = ScriptReader::new(rom);
    Ok(reader.get_editable_params(fighter_index))
}

/// Validate fighter parameters
#[tauri::command]
pub fn validate_fighter_params(
    params: EditableFighterParams,
) -> Result<ParamValidationResult, String> {
    Ok(params.validate_with_warnings())
}

/// Update fighter parameters
#[tauri::command]
pub fn update_fighter_params(
    state: State<AppState>,
    fighter_index: usize,
    params: EditableFighterParams,
) -> Result<EditableFighterParams, String> {
    // Validate params first
    params
        .validate()
        .map_err(|e| format!("Validation failed: {}", e))?;

    let rom_opt = state.rom.lock();
    let rom = rom_opt.as_ref().ok_or("No ROM loaded")?;
    let reader = ScriptReader::new(rom);

    // Generate the new header bytes
    let (header_bytes, pc_offset) = reader.generate_header_with_params(fighter_index, &params)?;
    drop(rom_opt);

    // Store in pending writes
    let pc_offset_str = format!("0x{:X}", pc_offset);
    state
        .pending_writes
        .lock()
        .insert(pc_offset_str, header_bytes);

    Ok(params)
}
