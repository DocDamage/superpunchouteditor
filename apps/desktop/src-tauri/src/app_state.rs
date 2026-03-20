//! Application State for Super Punch-Out!! Editor
//!
//! Centralizes all shared application state managed by Tauri.
//! This module provides the `AppState` struct that is passed to all command handlers.
//!
//! # Thread Safety
//!
//! All fields are wrapped in `parking_lot::Mutex` for safe concurrent access.
//! Unlike `std::sync::Mutex`, `parking_lot::Mutex` does not poison on panic,
//! eliminating the need for `.map_err(|_| "Lock poisoned")` error handling.
//!
//! # Access Pattern
//!
//! ```rust
//! // Simple lock - never fails
//! let rom = state.rom.lock();
//!
//! // Use rom...
//! drop(rom); // Explicit drop for clarity (optional - dropped at scope end)
//! ```

use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;

use asset_core::frame_tags::FrameTagManager;
use manifest_core::Manifest;
use project_core::{Project, ToolHooksConfig};
use crate::undo::EditHistory;
use rom_core::Rom;

use crate::audio_commands::AudioState;
use crate::emulator::EmulatorSettings;
use crate::emulator_embedded::EmbeddedEmulatorState;
use plugin_core::{PluginManager, PluginApi};

/// Central application state shared across all Tauri commands
///
/// This struct is managed by Tauri and passed to all command handlers
/// via `tauri::State<AppState>`. All fields are wrapped in `Mutex` to
/// allow safe concurrent access from multiple threads.
///
/// Uses `parking_lot::Mutex` which:
/// - Does not poison on panic (no need for error handling on lock)
/// - Is more efficient than std::sync::Mutex
/// - Provides constant-time lock/unlock operations
// Note: Debug not derived due to fields that don't implement Debug (Rom, AudioState, etc.)
pub struct AppState {
    /// The currently loaded ROM, if any
    pub rom: Mutex<Option<Rom>>,

    /// Manifest containing metadata about all fighters and assets
    pub manifest: Mutex<Manifest>,

    /// Key = pc_offset (hex string), value = replacement bytes
    /// These are pending modifications not yet written to disk
    pub pending_writes: Mutex<HashMap<String, Vec<u8>>>,

    /// The currently open project, if any
    pub current_project: Mutex<Option<Project>>,

    /// Path to the currently loaded ROM file
    pub rom_path: Mutex<Option<String>>,

    /// Edit history for undo/redo functionality
    pub edit_history: Mutex<EditHistory>,

    /// External emulator settings (Snes9x, bsnes, etc.)
    pub emulator_settings: Mutex<EmulatorSettings>,

    /// Frame tag manager for annotations
    pub frame_tag_manager: Mutex<FrameTagManager>,

    /// External tools configuration
    pub external_tools: Mutex<ToolHooksConfig>,

    /// Audio editor state - now wrapped in Mutex for thread safety
    pub audio_state: Mutex<AudioState>,

    /// Embedded emulator state - now wrapped in Mutex for thread safety
    pub embedded_emulator: Mutex<EmbeddedEmulatorState>,

    /// Flag indicating if the ROM has been modified
    pub modified: Mutex<bool>,

    /// Plugin manager for loading and running plugins
    pub plugin_manager: Mutex<Arc<PluginManager>>,
}

impl AppState {
    /// Create a new AppState with the given manifest
    ///
    /// Initializes all mutex-wrapped fields with their default values.
    pub fn new(manifest: Manifest) -> Self {
        // Set up plugin directories
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("super-punch-out-editor");
        let plugins_dir = config_dir.join("plugins");
        let plugin_config_dir = config_dir.join("plugin-config");
        let plugin_data_dir = config_dir.join("plugin-data");
        
        // Create plugin API
        let context = Arc::new(parking_lot::RwLock::new(plugin_core::PluginContext::new(
            plugin_config_dir.clone(),
            plugin_data_dir.clone(),
        )));
        let api = Arc::new(PluginApi::new(context));
        
        // Create and initialize plugin manager
        let plugin_manager = Arc::new(PluginManager::new(
            plugins_dir,
            plugin_config_dir,
            plugin_data_dir,
            api,
        ));
        
        // Try to initialize plugins (log error but don't fail)
        if let Err(e) = plugin_manager.initialize() {
            eprintln!("Failed to initialize plugin manager: {}", e);
        }
        
        Self {
            rom: Mutex::new(None),
            manifest: Mutex::new(manifest),
            pending_writes: Mutex::new(HashMap::new()),
            current_project: Mutex::new(None),
            rom_path: Mutex::new(None),
            edit_history: Mutex::new(EditHistory::new()),
            emulator_settings: Mutex::new(EmulatorSettings::default()),
            frame_tag_manager: Mutex::new(FrameTagManager::with_default_tags()),
            external_tools: Mutex::new(ToolHooksConfig::default()),
            audio_state: Mutex::new(AudioState::new()),
            embedded_emulator: Mutex::new(EmbeddedEmulatorState::new()),
            modified: Mutex::new(false),
            plugin_manager: Mutex::new(plugin_manager),
        }
    }

    /// Check if a ROM is currently loaded
    pub fn has_rom(&self) -> bool {
        self.rom.lock().is_some()
    }

    /// Get the SHA1 hash of the currently loaded ROM, if any
    pub fn get_rom_sha1(&self) -> Option<String> {
        self.rom.lock().as_ref().map(|r| r.calculate_sha1())
    }

    /// Clear all pending writes and edit history
    /// Should be called when loading a new ROM
    pub fn clear_for_new_rom(&self) {
        self.pending_writes.lock().clear();
        self.edit_history.lock().clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use manifest_core::Manifest;

    #[test]
    fn test_app_state_new() {
        // Note: This test requires a valid manifest
        // In practice, the manifest is loaded from a file
    }

    #[test]
    fn test_app_state_has_rom_initially_false() {
        let manifest = Manifest::default();
        let state = AppState::new(manifest);
        assert!(!state.has_rom());
        assert!(state.get_rom_sha1().is_none());
    }
}
