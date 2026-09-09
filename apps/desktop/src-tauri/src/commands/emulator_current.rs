//! Canonical embedded-emulator loading.

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::app_state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmulatorLoadReceipt {
    pub revision: u64,
    pub current_sha1: String,
    pub byte_length: usize,
}

/// Load the exact current editor revision into the embedded emulator.
#[tauri::command]
pub fn emulator_load_current_rom(
    state: State<'_, AppState>,
) -> Result<EmulatorLoadReceipt, String> {
    let materialized = state.materialize_current_rom()?;
    if materialized.bytes.len() < 0x8000 {
        return Err("Materialized ROM is too small for the SNES emulator".to_string());
    }

    let emulator_state = state.embedded_emulator.lock();
    let mut core_guard = emulator_state.core.lock();
    let core = core_guard
        .as_mut()
        .ok_or("Emulator not initialized. Call init_emulator first.")?;
    core.load_rom(materialized.bytes.clone())
        .map_err(|error| format!("Failed to load materialized ROM: {error}"))?;
    *emulator_state.loaded_rom_path.lock() = None;

    Ok(EmulatorLoadReceipt {
        revision: materialized.revision,
        current_sha1: materialized.current_sha1,
        byte_length: materialized.bytes.len(),
    })
}
