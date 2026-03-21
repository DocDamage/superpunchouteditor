//! Plugin System Commands
//!
//! Commands for managing plugins, running scripts, and batch operations.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;

use crate::app_state::AppState;
use plugin_core::{ScriptRunner, PluginApi};

/// Plugin information response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfoResponse {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub enabled: bool,
    pub path: String,
    pub loaded_at: String,
}

impl From<plugin_core::PluginInfo> for PluginInfoResponse {
    fn from(info: plugin_core::PluginInfo) -> Self {
        Self {
            id: info.id,
            name: info.name,
            version: info.version,
            author: info.author,
            description: info.description,
            enabled: info.enabled,
            path: info.path.to_string_lossy().into_owned(),
            loaded_at: info.loaded_at.to_rfc3339(),
        }
    }
}

/// List all loaded plugins
#[tauri::command]
pub fn list_plugins(state: State<AppState>) -> Result<Vec<PluginInfoResponse>, String> {
    let manager = state.plugin_manager.lock();
    let plugins = manager.get_all_plugins();
    Ok(plugins.into_iter().map(PluginInfoResponse::from).collect())
}

/// Load a plugin from file
#[tauri::command]
pub fn load_plugin(
    state: State<AppState>,
    path: String,
) -> Result<PluginInfoResponse, String> {
    let manager = state.plugin_manager.lock();
    let info = manager.load_plugin(&path).map_err(|e| e.to_string())?;
    Ok(PluginInfoResponse::from(info))
}

/// Unload a plugin
#[tauri::command]
pub fn unload_plugin(
    state: State<AppState>,
    plugin_id: String,
) -> Result<(), String> {
    let manager = state.plugin_manager.lock();
    manager.unload_plugin(&plugin_id).map_err(|e| e.to_string())
}

/// Enable a plugin
#[tauri::command]
pub fn enable_plugin(
    state: State<AppState>,
    plugin_id: String,
) -> Result<(), String> {
    let manager = state.plugin_manager.lock();
    manager.enable_plugin(&plugin_id).map_err(|e| e.to_string())
}

/// Disable a plugin
#[tauri::command]
pub fn disable_plugin(
    state: State<AppState>,
    plugin_id: String,
) -> Result<(), String> {
    let manager = state.plugin_manager.lock();
    manager.disable_plugin(&plugin_id).map_err(|e| e.to_string())
}

/// Execute a plugin command
#[tauri::command]
pub fn execute_plugin_command(
    state: State<AppState>,
    plugin_id: String,
    command: String,
    args: serde_json::Value,
) -> Result<serde_json::Value, String> {
    let manager = state.plugin_manager.lock();
    manager.execute_command(&plugin_id, &command, &args)
        .map_err(|e| e.to_string())
}

/// Script execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptExecutionResult {
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
    pub execution_time_ms: u64,
}

impl From<plugin_core::ScriptResult> for ScriptExecutionResult {
    fn from(result: plugin_core::ScriptResult) -> Self {
        Self {
            success: result.success,
            output: result.output,
            error: result.error,
            execution_time_ms: result.execution_time_ms,
        }
    }
}

/// Run a Lua script file
#[tauri::command]
pub fn run_script_file(
    _state: State<AppState>,
    path: String,
) -> Result<ScriptExecutionResult, String> {
    // Create a temporary API for the script runner
    let context = Arc::new(parking_lot::RwLock::new(plugin_core::PluginContext::new(
        std::path::PathBuf::from("."),
        std::path::PathBuf::from("."),
    )));
    let api = Arc::new(PluginApi::new(context));
    
    let runner = ScriptRunner::new(api);
    let result = runner.run_file(&path).map_err(|e| e.to_string())?;
    Ok(ScriptExecutionResult::from(result))
}

/// Run a Lua script from string
#[tauri::command]
pub fn run_script(
    _state: State<AppState>,
    script: String,
) -> Result<ScriptExecutionResult, String> {
    // Create a temporary API for the script runner
    let context = Arc::new(parking_lot::RwLock::new(plugin_core::PluginContext::new(
        std::path::PathBuf::from("."),
        std::path::PathBuf::from("."),
    )));
    let api = Arc::new(PluginApi::new(context));
    
    let runner = ScriptRunner::new(api);
    let result = runner.run_string(&script, None).map_err(|e| e.to_string())?;
    Ok(ScriptExecutionResult::from(result))
}

/// Batch job information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchJobInfo {
    pub id: String,
    pub name: String,
    pub status: String,
    pub progress: f32,
}

/// Get all batch jobs
#[tauri::command]
pub fn list_batch_jobs(_state: State<AppState>) -> Result<Vec<BatchJobInfo>, String> {
    // Placeholder - batch job system not yet implemented
    Ok(Vec::new())
}

/// Create a new batch job
#[tauri::command]
pub fn create_batch_job(
    _state: State<AppState>,
    _name: String,
    _script: String,
    _inputs: Vec<serde_json::Value>,
) -> Result<String, String> {
    // Placeholder - batch job system not yet implemented
    Err("Batch system not yet fully implemented".into())
}

/// Cancel a batch job
#[tauri::command]
pub fn cancel_batch_job(
    _state: State<AppState>,
    _job_id: String,
) -> Result<(), String> {
    // Placeholder - batch job system not yet implemented
    Err("Batch system not yet fully implemented".into())
}

/// Get the plugins directory path
#[tauri::command]
pub fn get_plugins_directory() -> Result<String, String> {
    let config_dir = dirs::config_dir()
        .ok_or("Could not find config directory")?
        .join("super-punch-out-editor")
        .join("plugins");
    
    Ok(config_dir.to_string_lossy().into_owned())
}

/// Open the plugins directory in the file manager
#[tauri::command]
pub fn open_plugins_directory() -> Result<(), String> {
    let plugins_dir = get_plugins_directory()?;
    
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&plugins_dir)
            .spawn()
            .map_err(|e| format!("Failed to open explorer: {}", e))?;
    }
    
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&plugins_dir)
            .spawn()
            .map_err(|e| format!("Failed to open Finder: {}", e))?;
    }
    
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&plugins_dir)
            .spawn()
            .map_err(|e| format!("Failed to open file manager: {}", e))?;
    }
    
    Ok(())
}

/// Reload all plugins
#[tauri::command]
pub fn reload_all_plugins(state: State<AppState>) -> Result<Vec<PluginInfoResponse>, String> {
    let manager = state.plugin_manager.lock();
    
    // Get list of currently loaded plugins
    let current_plugins = manager.get_all_plugins();
    let plugin_ids: Vec<String> = current_plugins.iter().map(|p| p.id.clone()).collect();
    
    // Unload all
    for id in &plugin_ids {
        let _ = manager.unload_plugin(id);
    }
    
    // Reload all
    let loaded = manager.load_all_plugins().map_err(|e| e.to_string())?;
    Ok(loaded.into_iter().map(PluginInfoResponse::from).collect())
}
