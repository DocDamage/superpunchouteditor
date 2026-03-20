//! External Emulator Integration Commands
//!
//! Commands for launching ROMs in external emulators.

use std::path::PathBuf;

use tauri::State;

use crate::app_state::AppState;
use crate::emulator::EmulatorLauncher;
use crate::utils::parse_offset;

/// Launch the current ROM in the configured emulator
#[tauri::command]
pub async fn test_in_emulator(
    state: State<'_, AppState>,
    auto_save: bool,
    quick_load_slot: Option<u8>,
    boxer_key: Option<String>,
    round: u8,
) -> Result<(), String> {
    // boxer_key and round are for future use with save state management
    let _ = boxer_key;
    let _ = round;

    // 1. Check if ROM is loaded
    let rom_path = state
        .rom_path
        .lock()
        .clone()
        .ok_or("No ROM loaded")?;

    // 2. Get emulator settings
    let settings = state
        .emulator_settings
        .lock()
        .clone();

    if settings.emulator_path.is_empty() {
        return Err(
            "No emulator configured. Please configure an emulator in settings.".to_string(),
        );
    }

    let emulator_path = PathBuf::from(&settings.emulator_path);
    if !emulator_path.exists() {
        return Err(format!("Emulator not found at: {}", settings.emulator_path));
    }

    // 3. Determine which ROM file to use
    let rom_to_launch = if auto_save {
        // Save to temp file with current edits
        let temp_dir = std::env::temp_dir().join("super-punch-out-editor");
        std::fs::create_dir_all(&temp_dir)
            .map_err(|e| format!("Failed to create temp directory: {}", e))?;

        let temp_rom_path = temp_dir.join("testing_rom.sfc");

        // Get current ROM data with pending writes applied
        let rom_opt = state.rom.lock();
        let rom = rom_opt.as_ref().ok_or("No ROM loaded")?;
        let mut rom_data = rom.data.clone();
        drop(rom_opt);

        // Apply pending writes
        let pending = state.pending_writes.lock();
        for (offset_str, bytes) in pending.iter() {
            let offset = parse_offset(offset_str)?;
            let len = bytes.len().min(rom_data.len() - offset);
            rom_data[offset..offset + len].copy_from_slice(&bytes[..len]);
        }
        drop(pending);

        // Write temp ROM
        std::fs::write(&temp_rom_path, rom_data)
            .map_err(|e| format!("Failed to write temp ROM: {}", e))?;

        temp_rom_path
    } else {
        PathBuf::from(&rom_path)
    };

    // 4. Parse additional command line arguments
    let extra_args: Vec<String> = if settings.command_line_args.is_empty() {
        vec![]
    } else {
        settings
            .command_line_args
            .split_whitespace()
            .map(String::from)
            .collect()
    };

    // 5. Launch emulator
    let _child = if let Some(slot) = quick_load_slot {
        let state_path = EmulatorLauncher::get_save_state_path(
            settings.emulator_type,
            &rom_to_launch,
            Some(slot),
        );
        EmulatorLauncher::launch_with_state(
            &rom_to_launch,
            &emulator_path,
            settings.emulator_type,
            &state_path,
            &extra_args,
        )
    } else {
        EmulatorLauncher::launch(
            &rom_to_launch,
            &emulator_path,
            settings.emulator_type,
            &extra_args,
        )
    }
    .map_err(|e| format!("Failed to launch emulator: {}", e))?;

    Ok(())
}

/// Get emulator presets for quick testing
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
