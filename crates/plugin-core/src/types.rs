//! Type definitions for the plugin system

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Asset types that plugins can work with
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetType {
    Palette,
    Sprite,
    Portrait,
    Icon,
    Animation,
    Script,
    Sound,
    Music,
    Text,
    Other,
}

/// Information about an asset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetInfo {
    pub id: String,
    pub asset_type: AssetType,
    pub name: String,
    pub description: Option<String>,
    pub size: usize,
    pub offset: Option<usize>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Result of a batch operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResult {
    pub success_count: usize,
    pub failure_count: usize,
    pub errors: Vec<String>,
    pub results: Vec<serde_json::Value>,
}

/// Configuration for a plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub enabled: bool,
    pub settings: HashMap<String, serde_json::Value>,
    pub hotkeys: Vec<PluginHotkey>,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            settings: HashMap::new(),
            hotkeys: Vec::new(),
        }
    }
}

/// Hotkey binding for a plugin command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginHotkey {
    pub command: String,
    pub key: String,
    pub modifiers: Vec<String>,
    pub context: HotkeyContext,
}

/// Context in which a hotkey is active
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HotkeyContext {
    Global,
    RomLoaded,
    EditorFocused,
    AssetSelected,
}

/// Menu item contributed by a plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMenuItem {
    pub path: String,  // e.g., "Tools/My Plugin/Do Something"
    pub command: String,
    pub shortcut: Option<String>,
    pub icon: Option<String>,
}

/// Toolbar button contributed by a plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginToolbarButton {
    pub id: String,
    pub tooltip: String,
    pub command: String,
    pub icon: String,
    pub position: ToolbarPosition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolbarPosition {
    Left,
    Right,
    Main,
}

/// Script execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptResult {
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
    pub return_value: Option<serde_json::Value>,
    pub execution_time_ms: u64,
}

/// Batch processing job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchJob {
    pub id: String,
    pub name: String,
    pub script_path: String,
    pub inputs: Vec<serde_json::Value>,
    pub status: BatchStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BatchStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}
