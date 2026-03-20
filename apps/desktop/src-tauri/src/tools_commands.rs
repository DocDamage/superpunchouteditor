//! External Tools Commands for Tauri
//!
//! These commands provide the interface between the frontend and
//! the external tool hooks system in project_core.

use crate::AppState;
use project_core::tools::{ExternalTool, ToolCategory, ToolContext, ToolError, ToolHooksConfig};
use std::path::PathBuf;
use tauri::State;

/// Get the path to the external tools configuration file
fn get_tools_config_path() -> Result<PathBuf, String> {
    let config_dir = dirs::config_dir()
        .ok_or("Could not find config directory")?
        .join("super-punch-out-editor");

    std::fs::create_dir_all(&config_dir)
        .map_err(|e| format!("Failed to create config directory: {}", e))?;

    Ok(config_dir.join("external-tools.json"))
}

/// Load external tools configuration from disk
pub fn load_external_tools_config() -> Result<ToolHooksConfig, String> {
    let path = get_tools_config_path()?;

    if !path.exists() {
        return Ok(ToolHooksConfig::default());
    }

    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read tools config: {}", e))?;

    let config: ToolHooksConfig = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse tools config: {}", e))?;

    Ok(config)
}

/// Save external tools configuration to disk
fn save_external_tools_config(config: &ToolHooksConfig) -> Result<(), String> {
    let path = get_tools_config_path()?;

    let content = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize tools config: {}", e))?;

    std::fs::write(&path, content).map_err(|e| format!("Failed to write tools config: {}", e))?;

    Ok(())
}

/// Get all configured external tools
#[tauri::command]
pub fn get_external_tools(state: State<'_, AppState>) -> Result<Vec<ExternalTool>, String> {
    let config = state.external_tools.lock();
    Ok(config.tools.clone())
}

/// Add a new external tool
#[tauri::command]
pub fn add_external_tool(state: State<'_, AppState>, tool: ExternalTool) -> Result<(), String> {
    let mut config = state.external_tools.lock();

    // Validate the tool
    if tool.executable_path.is_empty() {
        return Err("Executable path cannot be empty".to_string());
    }

    if tool.name.is_empty() {
        return Err("Tool name cannot be empty".to_string());
    }

    if tool.id.is_empty() {
        return Err("Tool ID cannot be empty".to_string());
    }

    // Add the tool
    config.add_tool(tool);

    // Save to disk
    drop(config);
    let config = state.external_tools.lock();
    save_external_tools_config(&*config)?;

    Ok(())
}

/// Remove an external tool by ID
#[tauri::command]
pub fn remove_external_tool(state: State<'_, AppState>, tool_id: String) -> Result<(), String> {
    let mut config = state.external_tools.lock();

    if !config.remove_tool(&tool_id) {
        return Err(format!("Tool '{}' not found", tool_id));
    }

    // Save to disk
    drop(config);
    let config = state.external_tools.lock();
    save_external_tools_config(&*config)?;

    Ok(())
}

/// Update an existing external tool
#[tauri::command]
pub fn update_external_tool(state: State<'_, AppState>, tool: ExternalTool) -> Result<(), String> {
    let mut config = state.external_tools.lock();

    // Check if tool exists
    if config.get_tool(&tool.id).is_none() {
        return Err(format!("Tool '{}' not found", tool.id));
    }

    // Update the tool
    config.add_tool(tool);

    // Save to disk
    drop(config);
    let config = state.external_tools.lock();
    save_external_tools_config(&*config)?;

    Ok(())
}

/// Launch a file with a specific tool
#[tauri::command]
pub fn launch_with_tool(
    state: State<'_, AppState>,
    tool_id: String,
    file_path: String,
    context: Option<ToolContext>,
) -> Result<(), String> {
    let config = state.external_tools.lock();

    let tool = config
        .get_tool(&tool_id)
        .ok_or_else(|| format!("Tool '{}' not found", tool_id))?
        .clone();

    drop(config);

    tool.launch(&file_path, context.as_ref())
        .map_err(|e| e.to_string())
}

/// Get all tools that can handle a given file extension
#[tauri::command]
pub fn get_compatible_tools(
    state: State<'_, AppState>,
    file_extension: String,
) -> Result<Vec<ExternalTool>, String> {
    let config = state.external_tools.lock();
    let tools = config.get_compatible_tools(&file_extension);
    Ok(tools.into_iter().cloned().collect())
}

/// Get all available tool categories
#[tauri::command]
pub fn get_tool_categories() -> Vec<serde_json::Value> {
    vec![
        (
            ToolCategory::GraphicsEditor,
            "Graphics editors like Photoshop, Aseprite, GIMP",
        ),
        (ToolCategory::HexEditor, "Hex editors like HxD, 010 Editor"),
        (
            ToolCategory::TileEditor,
            "Tile editors like Tile Layer Pro, YY-CHR",
        ),
        (ToolCategory::Emulator, "SNES emulators"),
        (ToolCategory::Other, "Other tools"),
    ]
    .into_iter()
    .map(|(cat, desc)| {
        serde_json::json!({
            "value": cat,
            "label": cat.display_name(),
            "icon": cat.icon(),
            "description": desc,
            "extensions": cat.supported_extensions(),
        })
    })
    .collect()
}

/// Get preset tools for common applications
#[tauri::command]
pub fn get_preset_tools() -> Vec<ExternalTool> {
    ToolHooksConfig::get_preset_tools()
}

/// Set the default tool for a file extension
#[tauri::command]
pub fn set_default_tool(
    state: State<'_, AppState>,
    file_extension: String,
    tool_id: String,
) -> Result<(), String> {
    let mut config = state.external_tools.lock();

    // Verify the tool exists
    if config.get_tool(&tool_id).is_none() {
        return Err(format!("Tool '{}' not found", tool_id));
    }

    config.set_default_tool(&file_extension, &tool_id);

    // Save to disk
    drop(config);
    let config = state.external_tools.lock();
    save_external_tools_config(&*config)?;

    Ok(())
}

/// Get the default tool for a file extension
#[tauri::command]
pub fn get_default_tool(
    state: State<'_, AppState>,
    file_extension: String,
) -> Result<Option<ExternalTool>, String> {
    let config = state.external_tools.lock();
    Ok(config.get_default_tool(&file_extension).cloned())
}

/// Verify that a tool executable exists and is accessible
#[tauri::command]
pub fn verify_tool(tool: ExternalTool) -> Result<serde_json::Value, String> {
    match tool.verify() {
        Ok(()) => Ok(serde_json::json!({
            "valid": true,
            "message": "Tool is valid and accessible",
        })),
        Err(ToolError::ExecutableNotFound(path)) => Ok(serde_json::json!({
            "valid": false,
            "message": format!("Executable not found: {}", path),
        })),
        Err(ToolError::InvalidExecutable(path)) => Ok(serde_json::json!({
            "valid": false,
            "message": format!("Invalid executable: {}", path),
        })),
        Err(e) => Ok(serde_json::json!({
            "valid": false,
            "message": e.to_string(),
        })),
    }
}
