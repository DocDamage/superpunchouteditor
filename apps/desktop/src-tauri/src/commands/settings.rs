//! Settings Commands
//!
//! Commands for loading and saving application settings.

use tauri::State;

use crate::app_state::AppState;
use crate::emulator::EmulatorSettings;

/// Load emulator settings from disk
pub fn load_emulator_settings() -> Result<EmulatorSettings, String> {
    let config_dir = dirs::config_dir()
        .ok_or("Could not find config directory")?
        .join("super-punch-out-editor");

    let settings_path = config_dir.join("emulator-settings.json");

    if !settings_path.exists() {
        return Ok(EmulatorSettings::default());
    }

    let content = std::fs::read_to_string(&settings_path)
        .map_err(|e| format!("Failed to read settings: {}", e))?;

    let settings: EmulatorSettings =
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse settings: {}", e))?;

    Ok(settings)
}

/// Save emulator settings to disk
fn save_emulator_settings(settings: &EmulatorSettings) -> Result<(), String> {
    let config_dir = dirs::config_dir()
        .ok_or("Could not find config directory")?
        .join("super-punch-out-editor");

    std::fs::create_dir_all(&config_dir)
        .map_err(|e| format!("Failed to create config directory: {}", e))?;

    let settings_path = config_dir.join("emulator-settings.json");

    let content = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;

    std::fs::write(&settings_path, content)
        .map_err(|e| format!("Failed to write settings: {}", e))?;

    Ok(())
}

/// Get the current emulator settings
#[tauri::command]
pub fn get_emulator_settings(state: State<AppState>) -> Result<EmulatorSettings, String> {
    Ok(state.emulator_settings.lock().clone())
}

/// Update emulator settings
#[tauri::command]
pub fn set_emulator_settings(
    state: State<AppState>,
    settings: EmulatorSettings,
) -> Result<(), String> {
    // Validate settings
    if !settings.emulator_path.is_empty() {
        let path = std::path::Path::new(&settings.emulator_path);
        if !path.exists() {
            return Err(format!("Emulator not found at: {}", settings.emulator_path));
        }
    }

    // Save to disk
    save_emulator_settings(&settings)?;

    // Update in-memory state
    *state.emulator_settings.lock() = settings;

    Ok(())
}

/// Browse for emulator executable
#[tauri::command]
pub async fn browse_for_emulator() -> Result<Option<String>, String> {
    // This would need to be called from the frontend with the app handle
    // For now, return None - frontend should handle the dialog
    Ok(None)
}

/// Verify the configured emulator
#[tauri::command]
pub fn verify_emulator(state: State<AppState>) -> Result<serde_json::Value, String> {
    let settings = state
        .emulator_settings
        .lock();

    if settings.emulator_path.is_empty() {
        return Ok(serde_json::json!({
            "valid": false,
            "message": "No emulator configured"
        }));
    }

    let path = std::path::Path::new(&settings.emulator_path);
    if !path.exists() {
        return Ok(serde_json::json!({
            "valid": false,
            "message": format!("Emulator not found at: {}", settings.emulator_path)
        }));
    }

    // Try to determine emulator type from path
    let path_str = settings.emulator_path.to_lowercase();
    let detected_type = if path_str.contains("snes9x") {
        "snes9x"
    } else if path_str.contains("bsnes") || path_str.contains("higan") {
        "bsnes"
    } else if path_str.contains("mesen-s") || path_str.contains("mesens") {
        "mesen-s"
    } else {
        "unknown"
    };

    Ok(serde_json::json!({
        "valid": true,
        "message": "Emulator found",
        "path": settings.emulator_path,
        "detected_type": detected_type,
        "configured_type": format!("{:?}", settings.emulator_type)
    }))
}

/// Create a quick save state for testing
#[tauri::command]
pub fn create_quick_save_state(
    _state: State<AppState>,
    _slot: u8,
    _boxer_index: Option<u8>,
    _round: Option<u8>,
) -> Result<(), String> {
    // Placeholder - actual save state creation requires emulator-specific implementation
    Err("Automatic save state creation is not implemented. Please create save states manually in the emulator.".to_string())
}

/// Get available save state slots
#[tauri::command]
pub fn get_save_state_slots(state: State<AppState>) -> Vec<u8> {
    let settings = state.emulator_settings.lock().clone();

    if settings.emulator_path.is_empty() {
        return Vec::new();
    }

    let rom_path = state.rom_path.lock().clone();

    let Some(rom_path) = rom_path else {
        return Vec::new();
    };

    let mut slots = Vec::new();

    for slot in 0..10 {
        let state_path = crate::emulator::EmulatorLauncher::get_save_state_path(
            settings.emulator_type,
            std::path::Path::new(&rom_path),
            Some(slot),
        );

        if state_path.exists() {
            slots.push(slot);
        }
    }

    slots
}
