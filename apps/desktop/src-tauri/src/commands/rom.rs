//! ROM Commands
//!
//! Commands for ROM loading, validation, and basic operations.

use tauri::{AppHandle, Manager, State};

use crate::app_state::AppState;
use crate::utils::{load_manifest_for_region, parse_offset, validation::validate_rom_path};

/// Open a ROM file from the specified path
///
/// Validates the ROM, calculates its SHA1 hash, and stores it in the app state.
/// Clears any pending writes and edit history from previous ROMs.
#[tauri::command]
pub fn open_rom(app: AppHandle, state: State<AppState>, path: String) -> Result<String, String> {
    // Validate path first
    validate_rom_path(&path)?;

    let rom = Rom::load(&path).map_err(|e| e.to_string())?;
    let region = rom.detect_region().ok_or_else(|| {
        format!(
            "Unknown ROM region. SHA1: {}",
            rom.calculate_sha1()
        )
    })?;

    // Resolve the Tauri resource directory for manifest loading.
    // This is populated in packaged builds and may be absent in some dev setups.
    let resource_dir = app.path().resource_dir().ok();
    let manifest = load_manifest_for_region(region, resource_dir.as_deref())?;

    let sha1 = rom.calculate_sha1();

    // Update state
    *state.rom.lock() = Some(rom);
    *state.rom_path.lock() = Some(path);
    *state.manifest.lock() = manifest;

    // Clear pending writes when new ROM is loaded
    state.pending_writes.lock().clear();

    // Clear edit history when loading new ROM (can't undo across different ROMs)
    state.edit_history.lock().clear();
    *state.modified.lock() = false;

    Ok(sha1)
}

/// Get the SHA1 hash of the currently loaded ROM
#[tauri::command]
pub fn get_rom_sha1(state: State<AppState>) -> Result<String, String> {
    state
        .get_rom_sha1()
        .ok_or_else(|| "No ROM loaded".to_string())
}

/// Get the path of the currently loaded ROM
#[tauri::command]
pub fn get_rom_path(state: State<AppState>) -> Option<String> {
    state.rom_path.lock().clone()
}

/// Save the ROM with all pending writes applied
///
/// Applies all pending modifications to the ROM data and writes it to disk.
#[tauri::command]
pub fn save_rom_as(state: State<AppState>, output_path: String) -> Result<(), String> {
    let mut rom_opt = state.rom.lock();
    let rom = rom_opt.as_mut().ok_or("No ROM loaded")?;

    let pending = state.pending_writes.lock();

    // Apply pending writes to ROM data
    for (offset_str, bytes) in pending.iter() {
        let offset = parse_offset(offset_str)?;
        // Write only as many bytes as fit in original space
        let len = bytes.len().min(rom.data.len() - offset);
        rom.data[offset..offset + len].copy_from_slice(&bytes[..len]);
    }

    drop(pending);

    rom.save(&output_path).map_err(|e| e.to_string())?;
    Ok(())
}

/// Get a list of all pending write offsets
///
/// Returns a list of PC offsets (as hex strings) that have pending modifications.
#[tauri::command]
pub fn get_pending_writes(state: State<AppState>) -> Vec<String> {
    state.pending_writes.lock().keys().cloned().collect()
}

/// Get pending bytes for a specific offset
///
/// Returns the modified bytes stored for the given PC offset.
#[tauri::command]
pub fn get_pending_bytes(state: State<AppState>, pc_offset: String) -> Result<Vec<u8>, String> {
    let pending = state.pending_writes.lock();

    pending
        .get(&pc_offset)
        .cloned()
        .ok_or_else(|| format!("No pending bytes for offset {}", pc_offset))
}

/// Get original bytes from the ROM for a given offset
///
/// Reads bytes directly from the loaded ROM without any pending modifications.
#[tauri::command]
pub fn get_rom_bytes(
    state: State<AppState>,
    pc_offset: String,
    size: usize,
) -> Result<Vec<u8>, String> {
    let offset = parse_offset(&pc_offset)?;

    let rom_opt = state.rom.lock();
    let rom = rom_opt.as_ref().ok_or("No ROM loaded")?;

    rom.read_bytes(offset, size)
        .map(|bytes| bytes.to_vec())
        .map_err(|e| e.to_string())
}

/// Discard a pending write for a specific offset
///
/// Removes the pending modification, reverting to the original ROM data.
#[tauri::command]
pub fn discard_bin_edit(state: State<AppState>, pc_offset: String) -> bool {
    state.pending_writes.lock().remove(&pc_offset).is_some()
}

/// Check if a ROM is currently loaded
#[tauri::command]
pub fn is_rom_loaded(state: State<AppState>) -> bool {
    state.has_rom()
}

// Re-export Rom type for use in other modules
pub use rom_core::Rom;
