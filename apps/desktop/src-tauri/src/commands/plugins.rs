//! Plugin System Commands
//!
//! Commands for managing plugins, running scripts, and batch operations.

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::State;

use crate::app_state::{AppState, BatchJobInfo};
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

/// Returns the current Unix timestamp as a string (ISO-like seconds since epoch).
fn now_timestamp() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .to_string()
}

/// Generate a unique job ID.
fn new_job_id() -> String {
    format!(
        "batch_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    )
}

/// Get all batch jobs (active and recently completed).
#[tauri::command]
pub fn list_batch_jobs(state: State<AppState>) -> Result<Vec<BatchJobInfo>, String> {
    Ok(state.batch_jobs.lock().clone())
}

/// Create and immediately start a new batch job.
///
/// The script runs once per input item. Each input is injected into the Lua
/// environment as a global called `INPUT` before the script executes.
///
/// Returns the job ID so the frontend can poll `list_batch_jobs` for progress.
#[tauri::command]
pub fn create_batch_job(
    app_handle: tauri::AppHandle,
    state: State<AppState>,
    name: String,
    script: String,
    inputs: Vec<serde_json::Value>,
) -> Result<String, String> {
    let job_id = new_job_id();
    let total = inputs.len() as u32;

    let job = BatchJobInfo {
        id: job_id.clone(),
        name,
        plugin_id: String::new(),
        status: "pending".to_string(),
        progress: 0,
        total,
        current_item: None,
        error: None,
        started_at: Some(now_timestamp()),
        completed_at: None,
    };

    state.batch_jobs.lock().push(job);

    let cancel_flag = Arc::new(AtomicBool::new(false));
    state
        .batch_cancel_flags
        .lock()
        .insert(job_id.clone(), Arc::clone(&cancel_flag));

    let job_id_thread = job_id.clone();

    std::thread::spawn(move || {
        let state = app_handle.state::<AppState>();

        // Mark running
        if let Some(job) = state
            .batch_jobs
            .lock()
            .iter_mut()
            .find(|j| j.id == job_id_thread)
        {
            job.status = "running".to_string();
        }

        // Build a script runner with a fresh Lua context (same as run_script).
        let context = Arc::new(parking_lot::RwLock::new(plugin_core::PluginContext::new(
            std::path::PathBuf::from("."),
            std::path::PathBuf::from("."),
        )));
        let api = Arc::new(PluginApi::new(context));
        let runner = ScriptRunner::new(api);

        let mut last_error: Option<String> = None;

        for (i, input) in inputs.iter().enumerate() {
            if cancel_flag.load(Ordering::Relaxed) {
                if let Some(job) = state
                    .batch_jobs
                    .lock()
                    .iter_mut()
                    .find(|j| j.id == job_id_thread)
                {
                    job.status = "failed".to_string();
                    job.error = Some("Cancelled by user".to_string());
                    job.completed_at = Some(now_timestamp());
                }
                state.batch_cancel_flags.lock().remove(&job_id_thread);
                return;
            }

            let input_str = input.to_string();

            if let Some(job) = state
                .batch_jobs
                .lock()
                .iter_mut()
                .find(|j| j.id == job_id_thread)
            {
                job.progress = i as u32;
                job.current_item = Some(input_str.clone());
            }

            // Prepend `INPUT = <json>` so the script can read the current item.
            let script_with_input = format!("INPUT = {}\n{}", input_str, script);
            match runner.run_string(&script_with_input, Some(&job_id_thread)) {
                Ok(result) if !result.success => last_error = result.error,
                Err(e) => last_error = Some(e.to_string()),
                _ => {}
            }
        }

        let final_status = if last_error.is_some() { "failed" } else { "completed" };

        if let Some(job) = state
            .batch_jobs
            .lock()
            .iter_mut()
            .find(|j| j.id == job_id_thread)
        {
            job.status = final_status.to_string();
            job.progress = total;
            job.current_item = None;
            job.error = last_error;
            job.completed_at = Some(now_timestamp());
        }

        state.batch_cancel_flags.lock().remove(&job_id_thread);
    });

    Ok(job_id)
}

/// Cancel a running batch job.
///
/// Sets the job's cancellation flag. The worker thread checks this flag
/// between inputs and exits cleanly when it is set.
#[tauri::command]
pub fn cancel_batch_job(
    state: State<AppState>,
    job_id: String,
) -> Result<(), String> {
    let flags = state.batch_cancel_flags.lock();
    if let Some(flag) = flags.get(&job_id) {
        flag.store(true, Ordering::Relaxed);
    }
    // If the flag is not present the job is already finished — not an error.
    Ok(())
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
