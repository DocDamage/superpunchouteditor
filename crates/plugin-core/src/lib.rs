//! Plugin System for Super Punch-Out!! Editor
//!
//! This crate provides a flexible plugin architecture that allows extending
//! the editor with custom functionality, including:
//! - Lua scripting for automation and custom tools
//! - WASM plugins for performance-critical extensions (future)
//! - Built-in plugin API for common operations

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;

pub mod api;
pub mod lua_runtime;
pub mod manager;
pub mod types;

pub use api::*;
pub use manager::*;
pub use types::*;

/// Current plugin API version
pub const PLUGIN_API_VERSION: u32 = 1;

/// Error type for plugin operations
#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    #[error("Plugin not found: {0}")]
    NotFound(String),
    
    #[error("Plugin already loaded: {0}")]
    AlreadyLoaded(String),
    
    #[error("Plugin version mismatch: expected {expected}, got {actual}")]
    VersionMismatch { expected: u32, actual: u32 },
    
    #[error("Plugin API error: {0}")]
    ApiError(String),
    
    #[error("Lua error: {0}")]
    LuaError(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("Plugin crashed: {0}")]
    PluginCrashed(String),
}

#[cfg(feature = "lua-scripting")]
impl From<mlua::Error> for PluginError {
    fn from(e: mlua::Error) -> Self {
        PluginError::LuaError(e.to_string())
    }
}

// Suppress warnings about unused items when lua-scripting is disabled
#[cfg(not(feature = "lua-scripting"))]
impl PluginError {
    pub fn lua_error(msg: impl Into<String>) -> Self {
        PluginError::LuaError(msg.into())
    }
}

/// Result type for plugin operations
pub type PluginResult<T> = Result<T, PluginError>;

/// Information about a loaded plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    /// Unique plugin identifier
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Plugin version
    pub version: String,
    /// Plugin author
    pub author: String,
    /// Plugin description
    pub description: String,
    /// API version this plugin targets
    pub api_version: u32,
    /// Plugin type
    pub plugin_type: PluginType,
    /// Whether the plugin is currently enabled
    pub enabled: bool,
    /// Plugin file path
    pub path: PathBuf,
    /// Last load time
    pub loaded_at: chrono::DateTime<chrono::Utc>,
}

/// Type of plugin
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginType {
    /// Lua script plugin
    Lua,
    /// WASM binary plugin (future)
    Wasm,
    /// Native Rust plugin (built-in)
    Native,
}

/// Event types that plugins can listen to
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EditorEvent {
    /// ROM was loaded
    RomLoaded,
    /// ROM is about to be saved
    RomSaving,
    /// Asset was modified
    AssetModified,
    /// Palette was edited
    PaletteEdited,
    /// Sprite was edited
    SpriteEdited,
    /// Animation was edited
    AnimationEdited,
    /// Project was created
    ProjectCreated,
    /// Project was opened
    ProjectOpened,
    /// Custom event (plugin-defined)
    Custom(String),
}

/// Context passed to plugins during operations
#[derive(Debug, Clone)]
pub struct PluginContext {
    /// Currently loaded ROM data (if any)
    pub rom_data: Option<Arc<RwLock<Vec<u8>>>>,
    /// Current project path (if any)
    pub project_path: Option<PathBuf>,
    /// Plugin configuration directory
    pub config_dir: PathBuf,
    /// Plugin data directory
    pub data_dir: PathBuf,
    /// Currently selected boxer (if any)
    pub selected_boxer: Option<String>,
}

impl PluginContext {
    pub fn new(config_dir: PathBuf, data_dir: PathBuf) -> Self {
        Self {
            rom_data: None,
            project_path: None,
            config_dir,
            data_dir,
            selected_boxer: None,
        }
    }
    
    pub fn with_rom(mut self, rom: Arc<RwLock<Vec<u8>>>) -> Self {
        self.rom_data = Some(rom);
        self
    }
}

/// Trait for implementing plugins
pub trait EditorPlugin: Send + Sync {
    /// Get plugin information
    fn info(&self) -> &PluginInfo;
    
    /// Initialize the plugin
    fn initialize(&mut self, ctx: &PluginContext) -> PluginResult<()>;
    
    /// Shutdown the plugin
    fn shutdown(&mut self) -> PluginResult<()>;
    
    /// Handle an editor event
    fn on_event(&mut self, event: &EditorEvent, ctx: &PluginContext) -> PluginResult<()>;
    
    /// Execute a plugin command
    fn execute_command(&mut self, command: &str, args: &serde_json::Value) -> PluginResult<serde_json::Value>;
    
    /// Get list of available commands
    fn available_commands(&self) -> Vec<PluginCommand>;
}

/// Definition of a plugin command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginCommand {
    /// Command name
    pub name: String,
    /// Command description
    pub description: String,
    /// JSON schema for command arguments
    pub args_schema: serde_json::Value,
    /// JSON schema for return value
    pub return_schema: serde_json::Value,
}

impl PluginCommand {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            args_schema: serde_json::json!({"type": "object"}),
            return_schema: serde_json::json!({"type": "any"}),
        }
    }
    
    pub fn with_args_schema(mut self, schema: serde_json::Value) -> Self {
        self.args_schema = schema;
        self
    }
    
    pub fn with_return_schema(mut self, schema: serde_json::Value) -> Self {
        self.return_schema = schema;
        self
    }
}
