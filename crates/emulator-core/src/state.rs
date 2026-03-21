//! # Save State Management
//!
//! Provides save/load state functionality for the emulator.
//! Save states allow the user to save and restore the complete
//! state of the emulated system at any point.

use crate::{EmulatorError, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Metadata for a save state
#[derive(Debug, Clone)]
pub struct SaveStateMetadata {
    /// Frame count when the state was saved
    pub frame_count: u64,
    /// ROM identifier (checksum)
    pub rom_id: String,
    /// Timestamp when the state was created
    pub timestamp: SystemTime,
}

impl SaveStateMetadata {
    /// Create new metadata
    pub fn new(frame_count: u64, rom_id: impl Into<String>) -> Self {
        Self {
            frame_count,
            rom_id: rom_id.into(),
            timestamp: SystemTime::now(),
        }
    }
}

/// A save state containing the complete emulator state
#[derive(Debug, Clone)]
pub struct SaveState {
    /// Save state metadata
    pub metadata: SaveStateMetadata,
    /// Core state data (opaque to this module, handled by the core)
    pub core_state: Vec<u8>,
    /// Controller button states
    pub controller_states: Vec<u16>,
    /// Slot number (0-9 for quick saves, or custom slots)
    pub slot: u8,
    /// State data (opaque to this module, handled by the core)
    pub data: Vec<u8>,
    /// Timestamp when the state was created
    pub timestamp: SystemTime,
    /// Optional description/name for the state
    pub description: Option<String>,
    /// ROM checksum/identifier
    pub rom_identifier: Option<String>,
}

impl SaveState {
    /// Create a new save state
    pub fn new(slot: u8, data: Vec<u8>) -> Self {
        Self {
            metadata: SaveStateMetadata::new(0, "unknown"),
            core_state: data.clone(),
            controller_states: Vec::new(),
            slot,
            data,
            timestamp: SystemTime::now(),
            description: None,
            rom_identifier: None,
        }
    }

    /// Create a save state with metadata
    pub fn with_metadata(
        metadata: SaveStateMetadata,
        core_state: Vec<u8>,
        controller_states: Vec<u16>,
    ) -> Self {
        Self {
            metadata,
            core_state: core_state.clone(),
            controller_states,
            slot: 0,
            data: core_state,
            timestamp: SystemTime::now(),
            description: None,
            rom_identifier: None,
        }
    }

    /// Create a save state with description
    pub fn with_description(slot: u8, data: Vec<u8>, description: impl Into<String>) -> Self {
        Self {
            metadata: SaveStateMetadata::new(0, "unknown"),
            core_state: data.clone(),
            controller_states: Vec::new(),
            slot,
            data,
            timestamp: SystemTime::now(),
            description: Some(description.into()),
            rom_identifier: None,
        }
    }

    /// Set the ROM identifier
    pub fn with_rom_identifier(mut self, identifier: impl Into<String>) -> Self {
        self.rom_identifier = Some(identifier.into());
        self
    }

    /// Get the size of the state data
    pub fn size(&self) -> usize {
        self.core_state.len()
    }

    /// Check if this state is valid (has data)
    pub fn is_valid(&self) -> bool {
        !self.core_state.is_empty()
    }

    /// Get formatted timestamp string
    pub fn timestamp_string(&self) -> String {
        use std::time::UNIX_EPOCH;
        let duration = self
            .timestamp
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        let secs = duration.as_secs();
        let _datetime = std::time::UNIX_EPOCH + duration;
        // Simple formatting - can be enhanced with chrono if needed
        format!("Timestamp: {}", secs)
    }
}

/// Metadata for a save state slot
#[derive(Debug, Clone)]
pub struct SlotInfo {
    /// Slot number
    pub slot: u8,
    /// Whether the slot has a save state
    pub occupied: bool,
    /// Timestamp when the state was created
    pub timestamp: Option<SystemTime>,
    /// Description of the save state
    pub description: Option<String>,
    /// Size of the save state in bytes
    pub size: usize,
}

impl SlotInfo {
    /// Create info for an empty slot
    pub fn empty(slot: u8) -> Self {
        Self {
            slot,
            occupied: false,
            timestamp: None,
            description: None,
            size: 0,
        }
    }

    /// Create info for an occupied slot
    pub fn from_state(state: &SaveState) -> Self {
        Self {
            slot: state.slot,
            occupied: true,
            timestamp: Some(state.timestamp),
            description: state.description.clone(),
            size: state.core_state.len(),
        }
    }
}

/// Manager for save states
pub struct StateManager {
    /// Directory where save states are stored
    save_dir: PathBuf,
    /// In-memory cache of recent states
    cache: HashMap<u8, SaveState>,
    /// Maximum number of slots (typically 10 for quick saves)
    max_slots: u8,
    /// File extension for save state files
    extension: String,
}

impl StateManager {
    /// Create a new state manager
    pub fn new(save_dir: impl AsRef<Path>) -> Result<Self> {
        let save_dir = save_dir.as_ref().to_path_buf();

        // Create directory if it doesn't exist
        std::fs::create_dir_all(&save_dir).map_err(|e| EmulatorError::IoError(e))?;

        Ok(Self {
            save_dir,
            cache: HashMap::new(),
            max_slots: 10,
            extension: "s9x".to_string(),
        })
    }

    /// Create a new state manager with default config directory
    pub fn new_default() -> Self {
        let save_dir = dirs::config_dir()
            .map(|d| d.join("super-punch-out-editor").join("states"))
            .unwrap_or_else(|| PathBuf::from("./states"));

        let _ = std::fs::create_dir_all(&save_dir);

        Self {
            save_dir,
            cache: HashMap::new(),
            max_slots: 10,
            extension: "s9x".to_string(),
        }
    }

    /// Save a quick state to a slot (0-9)
    pub fn save_quick(&mut self, slot: usize, state: SaveState) -> Result<()> {
        let slot = slot as u8;
        self.cache.insert(slot, state);
        Ok(())
    }

    /// Load a quick state from a slot (0-9)
    pub fn load_quick(&self, slot: usize) -> Result<Option<SaveState>> {
        let slot = slot as u8;
        Ok(self.cache.get(&slot).cloned())
    }

    /// Create with custom extension
    pub fn with_extension(mut self, extension: impl Into<String>) -> Self {
        self.extension = extension.into();
        self
    }

    /// Get the path for a specific slot
    fn slot_path(&self, slot: u8) -> PathBuf {
        self.save_dir
            .join(format!("slot_{:02}.{}", slot, self.extension))
    }

    /// Save a state to a slot
    pub fn save(&mut self, state: SaveState) -> Result<()> {
        let path = self.slot_path(state.slot);

        // Write to file
        std::fs::write(&path, &state.core_state)
            .map_err(|e| EmulatorError::StateError(format!("Failed to write state: {}", e)))?;

        // Update cache
        self.cache.insert(state.slot, state);

        Ok(())
    }

    /// Load a state from a slot
    pub fn load(&mut self, slot: u8) -> Result<SaveState> {
        // Check cache first
        if let Some(state) = self.cache.get(&slot) {
            return Ok(state.clone());
        }

        // Load from file
        let path = self.slot_path(slot);

        if !path.exists() {
            return Err(EmulatorError::StateError(format!(
                "No save state in slot {}",
                slot
            )));
        }

        let data = std::fs::read(&path)
            .map_err(|e| EmulatorError::StateError(format!("Failed to read state: {}", e)))?;

        let state = SaveState::new(slot, data);

        // Update cache
        self.cache.insert(slot, state.clone());

        Ok(state)
    }

    /// Check if a slot has a save state
    pub fn has_state(&self, slot: u8) -> bool {
        if self.cache.contains_key(&slot) {
            return true;
        }

        self.slot_path(slot).exists()
    }

    /// Get info for a slot
    pub fn get_slot_info(&self, slot: u8) -> SlotInfo {
        if let Some(state) = self.cache.get(&slot) {
            return SlotInfo::from_state(state);
        }

        let path = self.slot_path(slot);
        if path.exists() {
            if let Ok(metadata) = std::fs::metadata(&path) {
                if let Ok(modified) = metadata.modified() {
                    return SlotInfo {
                        slot,
                        occupied: true,
                        timestamp: Some(modified),
                        description: None,
                        size: metadata.len() as usize,
                    };
                }
            }
        }

        SlotInfo::empty(slot)
    }

    /// Get info for all slots
    pub fn get_all_slots(&self) -> Vec<SlotInfo> {
        (0..self.max_slots)
            .map(|slot| self.get_slot_info(slot))
            .collect()
    }

    /// Delete a save state
    pub fn delete(&mut self, slot: u8) -> Result<()> {
        let path = self.slot_path(slot);

        if path.exists() {
            std::fs::remove_file(&path)
                .map_err(|e| EmulatorError::StateError(format!("Failed to delete state: {}", e)))?;
        }

        self.cache.remove(&slot);

        Ok(())
    }

    /// Clear all save states
    pub fn clear_all(&mut self) -> Result<()> {
        for slot in 0..self.max_slots {
            let _ = self.delete(slot);
        }

        self.cache.clear();

        Ok(())
    }

    /// Import a state from a file
    pub fn import(&mut self, slot: u8, source_path: impl AsRef<Path>) -> Result<()> {
        let data = std::fs::read(&source_path)
            .map_err(|e| EmulatorError::StateError(format!("Failed to read source file: {}", e)))?;

        let state = SaveState::new(slot, data);
        self.save(state)
    }

    /// Export a state to a file
    pub fn export(&mut self, slot: u8, dest_path: impl AsRef<Path>) -> Result<()> {
        let state = self.load(slot)?;

        std::fs::write(&dest_path, &state.core_state).map_err(|e| {
            EmulatorError::StateError(format!("Failed to write destination file: {}", e))
        })?;

        Ok(())
    }

    /// Get the number of occupied slots
    pub fn occupied_count(&self) -> usize {
        (0..self.max_slots)
            .filter(|&slot| self.has_state(slot))
            .count()
    }

    /// Get the next free slot
    pub fn next_free_slot(&self) -> Option<u8> {
        (0..self.max_slots).find(|&slot| !self.has_state(slot))
    }

    /// Clear the in-memory cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_state_creation() {
        let data = vec![1, 2, 3, 4, 5];
        let state = SaveState::new(0, data.clone());

        assert_eq!(state.slot, 0);
        assert_eq!(state.data, data);
        assert!(state.is_valid());
        assert_eq!(state.size(), 5);
    }

    #[test]
    fn test_save_state_with_description() {
        let state = SaveState::with_description(1, vec![1, 2, 3], "Test State");

        assert_eq!(state.slot, 1);
        assert_eq!(state.description, Some("Test State".to_string()));
    }

    #[test]
    fn test_slot_info_empty() {
        let info = SlotInfo::empty(5);

        assert_eq!(info.slot, 5);
        assert!(!info.occupied);
        assert_eq!(info.size, 0);
    }

    #[test]
    fn test_slot_info_from_state() {
        let state = SaveState::new(3, vec![1, 2, 3, 4, 5]);
        let info = SlotInfo::from_state(&state);

        assert_eq!(info.slot, 3);
        assert!(info.occupied);
        assert_eq!(info.size, 5);
    }
}
