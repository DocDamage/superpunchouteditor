//! Application State for Super Punch-Out!! Editor
//!
//! Centralizes all shared application state managed by Tauri. The canonical ROM authority is a
//! `rom_core::RomSession`; the legacy `rom` field remains temporarily as a read projection while
//! command modules are migrated vertical-slice by vertical-slice.

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use crate::audio_commands::AudioState;
use crate::emulator::EmulatorSettings;
use crate::emulator_embedded::EmbeddedEmulatorState;
use crate::undo::EditHistory;
use asset_core::frame_tags::FrameTagManager;
use manifest_core::Manifest;
use plugin_core::{PluginApi, PluginManager};
use project_core::{Project, ToolHooksConfig};
use rom_core::{EditRequest, EditStateProjection, MaterializedRom, Rom, RomSession};

/// Batch job entry — mirrors the `BatchJob` type expected by the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchJobInfo {
    pub id: String,
    pub name: String,
    pub plugin_id: String,
    pub status: String,
    pub progress: u32,
    pub total: u32,
    pub current_item: Option<String>,
    pub error: Option<String>,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
}

/// Central application state shared across all Tauri commands.
pub struct AppState {
    /// Canonical immutable-base-plus-journal ROM session.
    pub rom_session: Mutex<Option<RomSession>>,

    /// Transitional read projection for command modules not yet migrated to `RomSession`.
    /// Stable mutation commands must not write this field directly.
    pub rom: Mutex<Option<Rom>>,

    pub manifest: Mutex<Manifest>,

    /// Transitional compatibility view for old frontend inspection APIs. This is no longer an
    /// authoritative representation of supported edits.
    pub pending_writes: Mutex<HashMap<String, Vec<u8>>>,

    pub current_project: Mutex<Option<Project>>,
    pub rom_path: Mutex<Option<String>>,
    pub edit_history: Mutex<EditHistory>,
    pub emulator_settings: Mutex<EmulatorSettings>,
    pub frame_tag_manager: Mutex<FrameTagManager>,
    pub external_tools: Mutex<ToolHooksConfig>,
    pub audio_state: Mutex<AudioState>,
    pub embedded_emulator: Mutex<EmbeddedEmulatorState>,

    /// Transitional dirty projection. Stable code derives dirty state from the journal revision.
    pub modified: Mutex<bool>,

    /// Experimental plugin manager. Discovery is intentionally not run at application startup;
    /// plugins execute only after an explicit user action while the feature remains experimental.
    pub plugin_manager: Mutex<Arc<PluginManager>>,
    pub batch_jobs: Mutex<Vec<BatchJobInfo>>,
    pub batch_cancel_flags: Mutex<HashMap<String, Arc<AtomicBool>>>,
}

impl AppState {
    pub fn new(manifest: Manifest) -> Self {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("super-punch-out-editor");
        let plugins_dir = config_dir.join("plugins");
        let plugin_config_dir = config_dir.join("plugin-config");
        let plugin_data_dir = config_dir.join("plugin-data");

        let context = Arc::new(parking_lot::RwLock::new(plugin_core::PluginContext::new(
            plugin_config_dir.clone(),
            plugin_data_dir.clone(),
        )));
        let api = Arc::new(PluginApi::new(context));
        let plugin_manager = Arc::new(PluginManager::new(
            plugins_dir,
            plugin_config_dir,
            plugin_data_dir,
            api,
        ));

        Self {
            rom_session: Mutex::new(None),
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
            batch_jobs: Mutex::new(Vec::new()),
            batch_cancel_flags: Mutex::new(HashMap::new()),
        }
    }

    pub fn has_rom(&self) -> bool {
        self.rom_session.lock().is_some() || self.rom.lock().is_some()
    }

    pub fn get_rom_sha1(&self) -> Option<String> {
        if let Some(session) = self.rom_session.lock().as_ref() {
            return Some(session.base().sha1().to_string());
        }
        self.rom.lock().as_ref().map(Rom::calculate_sha1)
    }

    /// Install a newly validated ROM as the immutable session base and refresh the transitional
    /// projection used by legacy read-only commands.
    pub fn install_rom_session(&self, rom: Rom, source_path: String) {
        let session = RomSession::from_rom(&rom, Some(source_path.clone()));
        *self.rom_session.lock() = Some(session);
        *self.rom.lock() = Some(rom);
        *self.rom_path.lock() = Some(source_path);
        self.pending_writes.lock().clear();
        self.edit_history.lock().clear();
        *self.modified.lock() = false;
    }

    /// Commit one canonical byte write. `before` bytes are captured by the backend from the current
    /// materialized revision. The legacy ROM mirror is refreshed only after the transaction commits.
    pub fn commit_rom_write(
        &self,
        label: impl Into<String>,
        offset: usize,
        after: Vec<u8>,
        asset_id: Option<String>,
        description: Option<String>,
    ) -> Result<EditStateProjection, String> {
        let mut session_guard = self.rom_session.lock();
        let session = session_guard.as_mut().ok_or("No ROM loaded")?;
        let projection = session
            .commit(
                label,
                vec![EditRequest::WriteBytes {
                    offset,
                    after: after.clone(),
                    asset_id,
                    description,
                }],
            )
            .map_err(|e| e.to_string())?;
        let materialized = session.materialize().map_err(|e| e.to_string())?;
        drop(session_guard);

        *self.rom.lock() = Some(Rom::new(materialized.bytes));
        *self.modified.lock() = projection.dirty;
        self.pending_writes
            .lock()
            .insert(format!("0x{offset:X}"), after);
        Ok(projection)
    }

    pub fn materialize_current_rom(&self) -> Result<MaterializedRom, String> {
        let session_guard = self.rom_session.lock();
        let session = session_guard.as_ref().ok_or("No ROM loaded")?;
        session.materialize().map_err(|e| e.to_string())
    }

    pub fn mark_rom_saved(&self) -> Result<EditStateProjection, String> {
        let mut session_guard = self.rom_session.lock();
        let session = session_guard.as_mut().ok_or("No ROM loaded")?;
        session.mark_saved();
        let projection = session.state_projection().map_err(|e| e.to_string())?;
        *self.modified.lock() = projection.dirty;
        Ok(projection)
    }

    pub fn undo_journal(&self) -> Result<Option<EditStateProjection>, String> {
        let mut session_guard = self.rom_session.lock();
        let session = session_guard.as_mut().ok_or("No ROM loaded")?;
        let projection = session.undo().map_err(|e| e.to_string())?;
        if projection.is_some() {
            let bytes = session.materialize().map_err(|e| e.to_string())?.bytes;
            *self.rom.lock() = Some(Rom::new(bytes));
            self.pending_writes.lock().clear();
            *self.modified.lock() = session.journal().is_dirty();
        }
        Ok(projection)
    }

    pub fn redo_journal(&self) -> Result<Option<EditStateProjection>, String> {
        let mut session_guard = self.rom_session.lock();
        let session = session_guard.as_mut().ok_or("No ROM loaded")?;
        let projection = session.redo().map_err(|e| e.to_string())?;
        if projection.is_some() {
            let bytes = session.materialize().map_err(|e| e.to_string())?.bytes;
            *self.rom.lock() = Some(Rom::new(bytes));
            self.pending_writes.lock().clear();
            *self.modified.lock() = session.journal().is_dirty();
        }
        Ok(projection)
    }

    pub fn clear_for_new_rom(&self) {
        *self.rom_session.lock() = None;
        *self.rom.lock() = None;
        *self.rom_path.lock() = None;
        self.pending_writes.lock().clear();
        self.edit_history.lock().clear();
        *self.modified.lock() = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_state_has_no_rom_initially() {
        let state = AppState::new(Manifest::empty());
        assert!(!state.has_rom());
        assert!(state.get_rom_sha1().is_none());
    }

    #[test]
    fn canonical_write_materializes_and_marks_dirty() {
        let state = AppState::new(Manifest::empty());
        state.install_rom_session(Rom::new(vec![0, 1, 2, 3]), "synthetic.sfc".into());
        let result = state
            .commit_rom_write("test", 1, vec![9, 8], None, None)
            .unwrap();
        assert!(result.dirty);
        assert_eq!(state.materialize_current_rom().unwrap().bytes, vec![0, 9, 8, 3]);
    }
}
