//! External Emulator Integration Commands
//!
//! Commands for launching the exact current editor revision in external emulators.

use std::path::PathBuf;

use tauri::State;

use crate::app_state::AppState;
use crate::emulator::EmulatorLauncher;

/// Launch the current materialized ROM in the configured external emulator.
///
/// `auto_save` is retained for IPC compatibility with existing frontend callers. External testing
/// always uses the canonical base-plus-journal materialization so it cannot silently launch stale
/// source bytes when the editor has unsaved changes.
#[tauri::command]
pub async fn test_in_emulator(
    state: State<'_, AppState>,
    auto_save: bool,
    quick_load_slot: Option<u8>,
    boxer_key: Option<String>,
    round: u8,
) -> Result<(), String> {
    let _ = auto_save;
    // boxer_key and round are reserved for future save-state/test-preset integration.
    let _ = boxer_key;
    let _ = round;

    // Keep the original source path only for save-state naming/location compatibility.
    let source_rom_path = state.rom_path.lock().clone().ok_or("No ROM loaded")?;
    let source_rom = PathBuf::from(source_rom_path);

    let settings = state.emulator_settings.lock().clone();
    if settings.emulator_path.is_empty() {
        return Err(
            "No emulator configured. Please configure an emulator in settings.".to_string(),
        );
    }

    let emulator_path = PathBuf::from(&settings.emulator_path);
    if !emulator_path.exists() {
        return Err(format!("Emulator not found at: {}", settings.emulator_path));
    }

    // The canonical editor session is the sole source of truth for external testing.
    let materialized = state.materialize_current_rom()?;
    let temp_dir = std::env::temp_dir().join("super-punch-out-editor");
    std::fs::create_dir_all(&temp_dir)
        .map_err(|e| format!("Failed to create temp directory: {e}"))?;

    let extension = source_rom
        .extension()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .unwrap_or("sfc");
    let hash_prefix = materialized
        .current_sha1
        .get(..8)
        .unwrap_or(materialized.current_sha1.as_str());
    let temp_rom_path = temp_dir.join(format!(
        "testing_rom_r{}_{}.{}",
        materialized.revision, hash_prefix, extension
    ));

    std::fs::write(&temp_rom_path, &materialized.bytes)
        .map_err(|e| format!("Failed to write materialized test ROM: {e}"))?;

    let extra_args: Vec<String> = if settings.command_line_args.is_empty() {
        vec![]
    } else {
        settings
            .command_line_args
            .split_whitespace()
            .map(String::from)
            .collect()
    };

    let _child = if let Some(slot) = quick_load_slot {
        // Locate save states using the original ROM identity while launching the materialized image.
        let state_path = EmulatorLauncher::get_save_state_path(
            settings.emulator_type,
            &source_rom,
            Some(slot),
        );
        EmulatorLauncher::launch_with_state(
            &temp_rom_path,
            &emulator_path,
            settings.emulator_type,
            &state_path,
            &extra_args,
        )
    } else {
        EmulatorLauncher::launch(
            &temp_rom_path,
            &emulator_path,
            settings.emulator_type,
            &extra_args,
        )
    }
    .map_err(|e| format!("Failed to launch emulator: {e}"))?;

    Ok(())
}

/// Get emulator presets for quick testing.
#[tauri::command]
pub fn get_emulator_presets() -> Vec<serde_json::Value> {
    vec![
        serde_json::json!({
            "id": "round_1",
            "name": "Round 1",
            "description": "Start at Round 1, full health",
            "boxer_index": null,
            "round": 1,
            "player_health": 128,
            "opponent_health": 128,
            "time_seconds": 180,
        }),
        serde_json::json!({
            "id": "knockdown",
            "name": "Knockdown Test",
            "description": "Low opponent health for easy KO",
            "boxer_index": null,
            "round": 1,
            "player_health": 128,
            "opponent_health": 10,
            "time_seconds": 180,
        }),
        serde_json::json!({
            "id": "low_health",
            "name": "Low Health",
            "description": "Test low health scenarios",
            "boxer_index": null,
            "round": 2,
            "player_health": 20,
            "opponent_health": 128,
            "time_seconds": 60,
        }),
    ]
}
