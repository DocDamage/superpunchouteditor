//! Experimental plugin command surface.
//!
//! Plugins are deliberately excluded from the stable 2.0 product. The previous implementation
//! executed unrestricted Lua and even ran discovery-time code. Keeping those commands live would
//! expose arbitrary local-code execution through IPC. Every execution/management command therefore
//! fails explicitly until a constrained capability model is implemented and the feature is rebuilt
//! as experimental-only.

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::app_state::{AppState, BatchJobInfo};

const PLUGIN_DISABLED: &str = "Plugins are disabled in stable builds. The experimental plugin runtime does not yet meet the required sandbox, permission, trust, timeout, and memory-limit security model.";

fn disabled<T>() -> Result<T, String> {
    Err(PLUGIN_DISABLED.to_string())
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptExecutionResult {
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
    pub execution_time_ms: u64,
}

#[tauri::command]
pub fn list_plugins(_state: State<AppState>) -> Result<Vec<PluginInfoResponse>, String> {
    disabled()
}

#[tauri::command]
pub fn load_plugin(_state: State<AppState>, _path: String) -> Result<PluginInfoResponse, String> {
    disabled()
}

#[tauri::command]
pub fn unload_plugin(_state: State<AppState>, _plugin_id: String) -> Result<(), String> {
    disabled()
}

#[tauri::command]
pub fn enable_plugin(_state: State<AppState>, _plugin_id: String) -> Result<(), String> {
    disabled()
}

#[tauri::command]
pub fn disable_plugin(_state: State<AppState>, _plugin_id: String) -> Result<(), String> {
    disabled()
}

#[tauri::command]
pub fn execute_plugin_command(
    _state: State<AppState>,
    _plugin_id: String,
    _command: String,
    _args: serde_json::Value,
) -> Result<serde_json::Value, String> {
    disabled()
}

#[tauri::command]
pub fn run_script_file(
    _state: State<AppState>,
    _path: String,
) -> Result<ScriptExecutionResult, String> {
    disabled()
}

#[tauri::command]
pub fn run_script(
    _state: State<AppState>,
    _script: String,
) -> Result<ScriptExecutionResult, String> {
    disabled()
}

#[tauri::command]
pub fn list_batch_jobs(_state: State<AppState>) -> Result<Vec<BatchJobInfo>, String> {
    disabled()
}

#[tauri::command]
pub fn create_batch_job(
    _app_handle: tauri::AppHandle,
    _state: State<AppState>,
    _name: String,
    _script: String,
    _inputs: Vec<serde_json::Value>,
) -> Result<String, String> {
    disabled()
}

#[tauri::command]
pub fn cancel_batch_job(_state: State<AppState>, _job_id: String) -> Result<(), String> {
    disabled()
}

#[tauri::command]
pub fn get_plugins_directory() -> Result<String, String> {
    disabled()
}

#[tauri::command]
pub fn open_plugins_directory() -> Result<(), String> {
    disabled()
}

#[tauri::command]
pub fn reload_all_plugins(_state: State<AppState>) -> Result<Vec<PluginInfoResponse>, String> {
    disabled()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stable_plugin_execution_is_explicitly_disabled() {
        assert!(disabled::<()>()
            .unwrap_err()
            .contains("disabled in stable builds"));
    }
}
