//! ROM commands.
//!
//! The canonical current image is always materialized from `AppState::rom_session`. The legacy
//! pending-write map remains available only as a compatibility projection for older frontend views.

use std::fs;
use std::io::Write;
use std::path::Path;

use tauri::{AppHandle, Manager, State};

use crate::app_state::AppState;
use crate::utils::{load_manifest_for_region, parse_offset, validation::validate_rom_path};
use rom_core::EditOperation;

pub(crate) fn build_current_rom_image(state: &AppState) -> Result<Vec<u8>, String> {
    Ok(state.materialize_current_rom()?.bytes)
}

#[tauri::command]
pub fn open_rom(app: AppHandle, state: State<AppState>, path: String) -> Result<String, String> {
    validate_rom_path(&path)?;

    let rom = Rom::load(&path).map_err(|e| e.to_string())?;
    let region = rom
        .detect_region()
        .ok_or_else(|| format!("Unknown ROM region. SHA1: {}", rom.calculate_sha1()))?;

    let resource_dir = app.path().resource_dir().ok();
    let manifest = load_manifest_for_region(region, resource_dir.as_deref())?;
    let sha1 = rom.calculate_sha1();

    // Validate everything before replacing the existing session.
    *state.manifest.lock() = manifest;
    state.install_rom_session(rom, path);

    Ok(sha1)
}

#[tauri::command]
pub fn get_rom_sha1(state: State<AppState>) -> Result<String, String> {
    state
        .get_rom_sha1()
        .ok_or_else(|| "No ROM loaded".to_string())
}

#[tauri::command]
pub fn get_rom_path(state: State<AppState>) -> Option<String> {
    state.rom_path.lock().clone()
}

/// Save the exact materialized current revision through a same-filesystem temporary file.
#[tauri::command]
pub fn save_rom_as(state: State<AppState>, output_path: String) -> Result<(), String> {
    let materialized = state.materialize_current_rom()?;
    let destination = Path::new(&output_path);
    let parent = destination.parent().unwrap_or_else(|| Path::new("."));
    if !parent.exists() {
        return Err(format!(
            "Destination directory does not exist: {}",
            parent.display()
        ));
    }

    let mut temp = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
    temp.write_all(&materialized.bytes)
        .map_err(|e| e.to_string())?;
    temp.as_file().sync_all().map_err(|e| e.to_string())?;

    let verify = fs::read(temp.path()).map_err(|e| e.to_string())?;
    if verify.len() != materialized.bytes.len() {
        return Err("Temporary ROM verification failed: length mismatch".to_string());
    }
    let verify_hash = Rom::new(verify).calculate_sha1();
    if verify_hash != materialized.current_sha1 {
        return Err("Temporary ROM verification failed: SHA-1 mismatch".to_string());
    }

    // Keep a best-effort backup before overwriting an existing destination. `persist` performs the
    // same-filesystem rename; if it fails, the previous destination remains available as itself or
    // the backup copy.
    if destination.exists() {
        let backup = destination.with_extension(
            destination
                .extension()
                .and_then(|value| value.to_str())
                .map(|ext| format!("{ext}.bak"))
                .unwrap_or_else(|| "bak".to_string()),
        );
        fs::copy(destination, &backup).map_err(|e| {
            format!(
                "Could not create backup {} before overwrite: {}",
                backup.display(),
                e
            )
        })?;
    }

    temp.persist(destination)
        .map_err(|error| format!("Could not atomically persist ROM: {}", error.error))?;

    state.mark_rom_saved()?;
    Ok(())
}

/// Compatibility projection of changed ranges. The journal remains authoritative.
#[tauri::command]
pub fn get_pending_writes(state: State<AppState>) -> Vec<String> {
    state
        .materialize_current_rom()
        .map(|materialized| {
            materialized
                .change_ranges
                .into_iter()
                .map(|range| format!("0x{:X}", range.start))
                .collect()
        })
        .unwrap_or_default()
}

#[tauri::command]
pub fn get_pending_bytes(state: State<AppState>, pc_offset: String) -> Result<Vec<u8>, String> {
    let offset = parse_offset(&pc_offset)?;
    let session_guard = state.rom_session.lock();
    let session = session_guard.as_ref().ok_or("No ROM loaded")?;

    for transaction in session.journal().active_transactions().iter().rev() {
        for operation in transaction.operations.iter().rev() {
            if let EditOperation::WriteBytes {
                offset: write_offset,
                after,
                ..
            } = operation
            {
                if *write_offset == offset {
                    return Ok(after.clone());
                }
            }
        }
    }

    Err(format!("No journal write begins at offset {pc_offset}"))
}

/// Read bytes from the immutable source ROM rather than the mutable working projection.
#[tauri::command]
pub fn get_rom_bytes(
    state: State<AppState>,
    pc_offset: String,
    size: usize,
) -> Result<Vec<u8>, String> {
    let offset = parse_offset(&pc_offset)?;
    let session_guard = state.rom_session.lock();
    let session = session_guard.as_ref().ok_or("No ROM loaded")?;
    let range =
        rom_core::validate_range(offset, size, session.base().len()).map_err(|e| e.to_string())?;
    Ok(session.base().bytes()[range].to_vec())
}

#[tauri::command]
pub fn get_loaded_rom_image(state: State<AppState>) -> Result<Vec<u8>, String> {
    build_current_rom_image(&state)
}

/// Arbitrary removal of a middle journal transaction is intentionally not supported. Undo keeps
/// history deterministic and validates before-bytes.
#[tauri::command]
pub fn discard_bin_edit(_state: State<AppState>, pc_offset: String) -> Result<bool, String> {
    Err(format!(
        "Selective discard at {pc_offset} is unsupported with the canonical journal; use Undo"
    ))
}

#[tauri::command]
pub fn is_rom_loaded(state: State<AppState>) -> bool {
    state.has_rom()
}

pub use rom_core::Rom;
