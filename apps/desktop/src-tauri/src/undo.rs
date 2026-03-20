//! Undo/Redo System for Super Punch-Out!! Editor
//!
//! Provides in-memory edit history tracking for all ROM modifications.
//! History is cleared when loading a new ROM (can't undo across different ROMs).

use chrono::{DateTime, Utc};
use serde::Serialize;

/// Maximum number of actions to keep in the undo stack
const DEFAULT_MAX_HISTORY: usize = 50;

/// Types of edit actions that can be undone/redone
#[derive(Clone, Debug)]
pub enum EditAction {
    /// Single color change in a palette
    PaletteEdit {
        pc_offset: String,
        #[allow(dead_code)]
        color_index: usize,
        old_bytes: Vec<u8>,
        new_bytes: Vec<u8>,
        description: String,
    },
    /// Full palette replacement
    #[allow(dead_code)]
    PaletteReplace {
        pc_offset: String,
        old_bytes: Vec<u8>,
        new_bytes: Vec<u8>,
        description: String,
    },
    /// Sprite bin modification
    SpriteBinEdit {
        pc_offset: String,
        old_bytes: Vec<u8>,
        new_bytes: Vec<u8>,
        description: String,
    },
    /// Asset import from PNG
    AssetImport {
        pc_offset: String,
        old_bytes: Vec<u8>,
        new_bytes: Vec<u8>,
        #[allow(dead_code)]
        source_path: String,
        description: String,
    },
    /// Asset relocation in ROM
    #[allow(dead_code)]
    Relocation {
        boxer_key: String,
        asset_file: String,
        old_pc_offset: String,
        new_pc_offset: String,
        size: usize,
        old_data: Vec<u8>,
        description: String,
    },
    /// Batch of multiple actions (for multi-asset operations)
    #[allow(dead_code)]
    BatchEdit {
        actions: Vec<EditAction>,
        description: String,
    },
}

impl EditAction {
    /// Get a human-readable description of the action
    pub fn description(&self) -> &str {
        match self {
            EditAction::PaletteEdit { description, .. } => description,
            EditAction::PaletteReplace { description, .. } => description,
            EditAction::SpriteBinEdit { description, .. } => description,
            EditAction::AssetImport { description, .. } => description,
            EditAction::Relocation { description, .. } => description,
            EditAction::BatchEdit { description, .. } => description,
        }
    }

    /// Get the PC offset affected by this action
    pub fn pc_offset(&self) -> Option<&str> {
        match self {
            EditAction::PaletteEdit { pc_offset, .. } => Some(pc_offset),
            EditAction::PaletteReplace { pc_offset, .. } => Some(pc_offset),
            EditAction::SpriteBinEdit { pc_offset, .. } => Some(pc_offset),
            EditAction::AssetImport { pc_offset, .. } => Some(pc_offset),
            EditAction::Relocation { old_pc_offset, .. } => Some(old_pc_offset),
            EditAction::BatchEdit { .. } => None,
        }
    }

    /// Get the action type name
    pub fn action_type(&self) -> &'static str {
        match self {
            EditAction::PaletteEdit { .. } => "Palette Edit",
            EditAction::PaletteReplace { .. } => "Palette Replace",
            EditAction::SpriteBinEdit { .. } => "Sprite Edit",
            EditAction::AssetImport { .. } => "Import",
            EditAction::Relocation { .. } => "Relocation",
            EditAction::BatchEdit { .. } => "Batch Edit",
        }
    }

    /// Get the old bytes for undo
    pub fn old_bytes(&self) -> Option<&Vec<u8>> {
        match self {
            EditAction::PaletteEdit { old_bytes, .. } => Some(old_bytes),
            EditAction::PaletteReplace { old_bytes, .. } => Some(old_bytes),
            EditAction::SpriteBinEdit { old_bytes, .. } => Some(old_bytes),
            EditAction::AssetImport { old_bytes, .. } => Some(old_bytes),
            EditAction::Relocation { .. } => None,
            EditAction::BatchEdit { .. } => None,
        }
    }

    /// Get the new bytes for redo
    pub fn new_bytes(&self) -> Option<&Vec<u8>> {
        match self {
            EditAction::PaletteEdit { new_bytes, .. } => Some(new_bytes),
            EditAction::PaletteReplace { new_bytes, .. } => Some(new_bytes),
            EditAction::SpriteBinEdit { new_bytes, .. } => Some(new_bytes),
            EditAction::AssetImport { new_bytes, .. } => Some(new_bytes),
            EditAction::Relocation { .. } => None,
            EditAction::BatchEdit { .. } => None,
        }
    }
}

/// Summary of an edit action for UI display
#[derive(Clone, Debug, Serialize)]
pub struct EditSummary {
    pub id: usize,
    pub action_type: String,
    pub description: String,
    pub pc_offset: Option<String>,
    pub timestamp: DateTime<Utc>,
}

/// The edit history manager
#[derive(Clone, Debug)]
pub struct EditHistory {
    undo_stack: Vec<(EditAction, DateTime<Utc>)>,
    redo_stack: Vec<(EditAction, DateTime<Utc>)>,
    max_history: usize,
    next_id: usize,
}

impl Default for EditHistory {
    fn default() -> Self {
        Self::new()
    }
}

impl EditHistory {
    /// Create a new edit history with default capacity
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_MAX_HISTORY)
    }

    /// Create a new edit history with specified capacity
    pub fn with_capacity(max_history: usize) -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_history,
            next_id: 0,
        }
    }

    /// Push a new action to the history
    /// This clears the redo stack (standard undo/redo behavior)
    pub fn push(&mut self, action: EditAction) {
        // Clear redo stack when new action is performed
        self.redo_stack.clear();

        let timestamp = Utc::now();
        self.undo_stack.push((action, timestamp));

        // Remove oldest entries if we exceed max history
        if self.undo_stack.len() > self.max_history {
            self.undo_stack.remove(0);
        }

        self.next_id += 1;
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Get the number of undoable actions
    #[allow(dead_code)]
    pub fn undo_count(&self) -> usize {
        self.undo_stack.len()
    }

    /// Get the number of redoable actions
    #[allow(dead_code)]
    pub fn redo_count(&self) -> usize {
        self.redo_stack.len()
    }

    /// Pop the last action for undo
    /// Returns the action and moves it to redo stack
    pub fn undo(&mut self) -> Option<EditAction> {
        if let Some((action, _)) = self.undo_stack.pop() {
            self.redo_stack.push((action.clone(), Utc::now()));
            Some(action)
        } else {
            None
        }
    }

    /// Pop the last undone action for redo
    /// Returns the action and moves it back to undo stack
    pub fn redo(&mut self) -> Option<EditAction> {
        if let Some((action, _)) = self.redo_stack.pop() {
            self.undo_stack.push((action.clone(), Utc::now()));
            Some(action)
        } else {
            None
        }
    }

    /// Clear all history
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.next_id = 0;
    }

    /// Get a summary of all undoable actions (most recent first)
    pub fn get_undo_summary(&self) -> Vec<EditSummary> {
        self.undo_stack
            .iter()
            .enumerate()
            .rev()
            .map(|(idx, (action, timestamp))| EditSummary {
                id: self.next_id - self.undo_stack.len() + idx,
                action_type: action.action_type().to_string(),
                description: action.description().to_string(),
                pc_offset: action.pc_offset().map(|s| s.to_string()),
                timestamp: *timestamp,
            })
            .collect()
    }

    /// Get a summary of all redoable actions (most recent first)
    pub fn get_redo_summary(&self) -> Vec<EditSummary> {
        self.redo_stack
            .iter()
            .enumerate()
            .rev()
            .map(|(idx, (action, timestamp))| EditSummary {
                id: self.next_id + idx,
                action_type: action.action_type().to_string(),
                description: action.description().to_string(),
                pc_offset: action.pc_offset().map(|s| s.to_string()),
                timestamp: *timestamp,
            })
            .collect()
    }

    /// Get the full edit history (both undo and redo stacks)
    pub fn get_full_history(&self) -> Vec<EditSummary> {
        let mut history = self.get_undo_summary();
        history.reverse(); // Undo stack is stored oldest first for display
        history.extend(self.get_redo_summary());
        history
    }

    /// Peek at the last undo action without modifying stacks
    #[allow(dead_code)]
    pub fn peek_undo(&self) -> Option<&EditAction> {
        self.undo_stack.last().map(|(action, _)| action)
    }

    /// Peek at the last redo action without modifying stacks
    #[allow(dead_code)]
    pub fn peek_redo(&self) -> Option<&EditAction> {
        self.redo_stack.last().map(|(action, _)| action)
    }
}

/// Helper function to create a palette edit action
#[allow(dead_code)]
pub fn create_palette_edit(
    pc_offset: &str,
    color_index: usize,
    old_color: &[u8],
    new_color: &[u8],
) -> EditAction {
    EditAction::PaletteEdit {
        pc_offset: pc_offset.to_string(),
        color_index,
        old_bytes: old_color.to_vec(),
        new_bytes: new_color.to_vec(),
        description: format!("Changed color {} at {}", color_index, pc_offset),
    }
}

/// Helper function to create a sprite bin edit action
#[allow(dead_code)]
pub fn create_sprite_bin_edit(pc_offset: &str, old_bytes: &[u8], new_bytes: &[u8]) -> EditAction {
    EditAction::SpriteBinEdit {
        pc_offset: pc_offset.to_string(),
        old_bytes: old_bytes.to_vec(),
        new_bytes: new_bytes.to_vec(),
        description: format!(
            "Edited sprite bin at {} ({} bytes)",
            pc_offset,
            new_bytes.len()
        ),
    }
}

/// Helper function to create an asset import action
#[allow(dead_code)]
pub fn create_asset_import(
    pc_offset: &str,
    old_bytes: &[u8],
    new_bytes: &[u8],
    source_path: &str,
) -> EditAction {
    EditAction::AssetImport {
        pc_offset: pc_offset.to_string(),
        old_bytes: old_bytes.to_vec(),
        new_bytes: new_bytes.to_vec(),
        source_path: source_path.to_string(),
        description: format!(
            "Imported from {}",
            std::path::Path::new(source_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(source_path)
        ),
    }
}

/// Helper function to create a relocation action
#[allow(dead_code)]
pub fn create_relocation(
    boxer_key: &str,
    asset_file: &str,
    old_pc_offset: &str,
    new_pc_offset: &str,
    size: usize,
    old_data: &[u8],
) -> EditAction {
    EditAction::Relocation {
        boxer_key: boxer_key.to_string(),
        asset_file: asset_file.to_string(),
        old_pc_offset: old_pc_offset.to_string(),
        new_pc_offset: new_pc_offset.to_string(),
        size,
        old_data: old_data.to_vec(),
        description: format!(
            "Relocated {} from {} to {}",
            asset_file, old_pc_offset, new_pc_offset
        ),
    }
}

/// Helper function to create a batch edit action
#[allow(dead_code)]
pub fn create_batch_edit(actions: Vec<EditAction>, description: &str) -> EditAction {
    EditAction::BatchEdit {
        actions,
        description: description.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edit_history_push_and_undo() {
        let mut history = EditHistory::new();

        // Create a simple edit action
        let action = create_palette_edit("0x1000", 0, &[0, 0], &[255, 255]);
        history.push(action);

        assert!(history.can_undo());
        assert!(!history.can_redo());
        assert_eq!(history.undo_count(), 1);

        // Undo should move to redo stack
        let undone = history.undo();
        assert!(undone.is_some());
        assert!(!history.can_undo());
        assert!(history.can_redo());
    }

    #[test]
    fn test_redo_stack_cleared_on_new_action() {
        let mut history = EditHistory::new();

        // Add and undo an action
        let action1 = create_palette_edit("0x1000", 0, &[0, 0], &[255, 255]);
        history.push(action1);
        history.undo();

        assert!(history.can_redo());

        // Adding new action should clear redo stack
        let action2 = create_palette_edit("0x1000", 1, &[0, 0], &[128, 128]);
        history.push(action2);

        assert!(!history.can_redo());
        assert!(history.can_undo());
    }

    #[test]
    fn test_history_capacity_limit() {
        let mut history = EditHistory::with_capacity(3);

        // Add 5 actions (exceeds capacity of 3)
        for i in 0..5 {
            let action = create_palette_edit("0x1000", i, &[0, 0], &[255, 255]);
            history.push(action);
        }

        // Should only have 3 actions in history
        assert_eq!(history.undo_count(), 3);
    }

    #[test]
    fn test_clear_history() {
        let mut history = EditHistory::new();

        let action = create_palette_edit("0x1000", 0, &[0, 0], &[255, 255]);
        history.push(action);
        history.undo();

        assert!(history.can_undo() || history.can_redo());

        history.clear();

        assert!(!history.can_undo());
        assert!(!history.can_redo());
    }
}
