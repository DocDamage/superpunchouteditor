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
        description: String,
    },
}

impl EditAction {
    /// Get a human-readable description of the action
    pub fn description(&self) -> &str {
        match self {
            EditAction::PaletteEdit { description, .. } => description,
            EditAction::SpriteBinEdit { description, .. } => description,
            EditAction::AssetImport { description, .. } => description,
        }
    }

    /// Get the PC offset affected by this action
    pub fn pc_offset(&self) -> Option<&str> {
        match self {
            EditAction::PaletteEdit { pc_offset, .. } => Some(pc_offset),
            EditAction::SpriteBinEdit { pc_offset, .. } => Some(pc_offset),
            EditAction::AssetImport { pc_offset, .. } => Some(pc_offset),
        }
    }

    /// Get the action type name
    pub fn action_type(&self) -> &'static str {
        match self {
            EditAction::PaletteEdit { .. } => "Palette Edit",
            EditAction::SpriteBinEdit { .. } => "Sprite Edit",
            EditAction::AssetImport { .. } => "Import",
        }
    }

    /// Get the old bytes for undo
    pub fn old_bytes(&self) -> Option<&Vec<u8>> {
        match self {
            EditAction::PaletteEdit { old_bytes, .. } => Some(old_bytes),
            EditAction::SpriteBinEdit { old_bytes, .. } => Some(old_bytes),
            EditAction::AssetImport { old_bytes, .. } => Some(old_bytes),
        }
    }

    /// Get the new bytes for redo
    pub fn new_bytes(&self) -> Option<&Vec<u8>> {
        match self {
            EditAction::PaletteEdit { new_bytes, .. } => Some(new_bytes),
            EditAction::SpriteBinEdit { new_bytes, .. } => Some(new_bytes),
            EditAction::AssetImport { new_bytes, .. } => Some(new_bytes),
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
    #[cfg(test)]
    pub fn undo_count(&self) -> usize {
        self.undo_stack.len()
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
}

#[cfg(test)]
fn create_palette_edit(pc_offset: &str, old_color: &[u8], new_color: &[u8]) -> EditAction {
    EditAction::PaletteEdit {
        pc_offset: pc_offset.to_string(),
        old_bytes: old_color.to_vec(),
        new_bytes: new_color.to_vec(),
        description: format!("Changed palette color at {}", pc_offset),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edit_history_push_and_undo() {
        let mut history = EditHistory::new();

        // Create a simple edit action
        let action = create_palette_edit("0x1000", &[0, 0], &[255, 255]);
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
        let action1 = create_palette_edit("0x1000", &[0, 0], &[255, 255]);
        history.push(action1);
        history.undo();

        assert!(history.can_redo());

        // Adding new action should clear redo stack
        let action2 = create_palette_edit("0x1000", &[0, 0], &[128, 128]);
        history.push(action2);

        assert!(!history.can_redo());
        assert!(history.can_undo());
    }

    #[test]
    fn test_history_capacity_limit() {
        let mut history = EditHistory::with_capacity(3);

        // Add 5 actions (exceeds capacity of 3)
        for _ in 0..5 {
            let action = create_palette_edit("0x1000", &[0, 0], &[255, 255]);
            history.push(action);
        }

        // Should only have 3 actions in history
        assert_eq!(history.undo_count(), 3);
    }

    #[test]
    fn test_clear_history() {
        let mut history = EditHistory::new();

        let action = create_palette_edit("0x1000", &[0, 0], &[255, 255]);
        history.push(action);
        history.undo();

        assert!(history.can_undo() || history.can_redo());

        history.clear();

        assert!(!history.can_undo());
        assert!(!history.can_redo());
    }
}
