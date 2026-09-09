//! Settings Import/Export Commands for Tauri
//!
//! Provides functionality for backing up, restoring, and sharing
//! editor configuration settings.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Settings file version
const SETTINGS_VERSION: &str = "2.0";

/// App settings structure for import/export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    /// Settings version
    pub version: String,
    /// UI theme
    pub theme: Option<String>,
    /// UI scale factor
    pub ui_scale: Option<f32>,
    /// Sidebar collapsed state
    pub sidebar_collapsed: Option<bool>,
    /// Default export format
    pub default_export_format: Option<String>,
    /// Auto-save interval in minutes
    pub auto_save_interval: Option<i32>,
    /// Confirm on close
    pub confirm_on_close: Option<bool>,
    /// Show tooltips
    pub show_tooltips: Option<bool>,
    /// Emulator path
    pub emulator_path: Option<String>,
    /// Emulator type
    pub emulator_type: Option<String>,
    /// Auto-save before launch
    pub auto_save_before_launch: Option<bool>,
    /// Default export directory
    pub default_export_directory: Option<String>,
    /// Recent projects
    pub recent_projects: Option<Vec<String>>,
    /// Recent ROMs
    pub recent_roms: Option<Vec<String>>,
    /// External tools
    pub external_tools: Option<Vec<serde_json::Value>>,
    /// Default tool IDs
    pub default_tool_ids: Option<HashMap<String, String>>,
    /// Custom shortcuts
    pub custom_shortcuts: Option<HashMap<String, Vec<String>>>,
    /// Panel layout
    pub panel_layout: Option<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            version: SETTINGS_VERSION.to_string(),
            theme: Some("system".to_string()),
            ui_scale: Some(1.0),
            sidebar_collapsed: Some(false),
            default_export_format: Some("png".to_string()),
            auto_save_interval: Some(5),
            confirm_on_close: Some(true),
            show_tooltips: Some(true),
            emulator_path: None,
            emulator_type: None,
            auto_save_before_launch: Some(true),
            default_export_directory: None,
            recent_projects: Some(vec![]),
            recent_roms: Some(vec![]),
            external_tools: Some(vec![]),
            default_tool_ids: Some(HashMap::new()),
            custom_shortcuts: Some(HashMap::new()),
            panel_layout: Some("default".to_string()),
        }
    }
}

/// Settings export file structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsExport {
    /// Export format version
    pub version: String,
    /// Export timestamp (ISO 8601)
    pub exported_at: String,
    /// Application name
    pub app: String,
    /// Settings data
    pub settings: AppSettings,
}

/// Import report showing what settings were changed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportReport {
    /// Whether the import was successful
    pub success: bool,
    /// List of settings that were imported
    pub imported: Vec<String>,
    /// List of settings that were merged
    pub merged: Vec<String>,
    /// List of settings that were skipped
    pub skipped: Vec<String>,
    /// Any errors encountered
    pub errors: Vec<String>,
    /// Warnings for the user
    pub warnings: Vec<String>,
}

/// Settings change preview for import dialog
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsChangePreview {
    /// Category of the setting
    pub category: String,
    /// Setting key
    pub key: String,
    /// Human-readable name
    pub display_name: String,
    /// Current value (as JSON)
    pub current_value: Option<serde_json::Value>,
    /// New value from import
    pub new_value: serde_json::Value,
    /// Whether this will change the current value
    pub will_change: bool,
    /// Whether there's a conflict
    pub has_conflict: bool,
}

/// Get the path to the app settings configuration file
fn get_settings_config_path() -> Result<PathBuf, String> {
    let config_dir = dirs::config_dir()
        .ok_or("Could not find config directory")?
        .join("super-punch-out-editor");

    std::fs::create_dir_all(&config_dir)
        .map_err(|e| format!("Failed to create config directory: {}", e))?;

    Ok(config_dir.join("app-settings.json"))
}

/// Load app settings from disk
pub fn load_app_settings() -> Result<AppSettings, String> {
    let path = get_settings_config_path()?;

    if !path.exists() {
        return Ok(AppSettings::default());
    }

    let content =
        std::fs::read_to_string(&path).map_err(|e| format!("Failed to read settings: {}", e))?;

    let settings: AppSettings =
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse settings: {}", e))?;

    Ok(settings)
}

/// Save app settings to disk
fn save_app_settings(settings: &AppSettings) -> Result<(), String> {
    let path = get_settings_config_path()?;

    let content = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;

    std::fs::write(&path, content).map_err(|e| format!("Failed to write settings: {}", e))?;

    Ok(())
}

/// Export all settings to a file
#[tauri::command]
pub fn export_settings(output_path: String) -> Result<(), String> {
    let settings = load_app_settings()?;

    let export = SettingsExport {
        version: SETTINGS_VERSION.to_string(),
        exported_at: chrono::Utc::now().to_rfc3339(),
        app: "Super Punch-Out!! Editor".to_string(),
        settings,
    };

    let content = serde_json::to_string_pretty(&export)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;

    std::fs::write(&output_path, content)
        .map_err(|e| format!("Failed to write settings file: {}", e))?;

    Ok(())
}

/// Import settings from a file
#[tauri::command]
pub fn import_settings(settings_path: String, merge: bool) -> Result<ImportReport, String> {
    let content = std::fs::read_to_string(&settings_path)
        .map_err(|e| format!("Failed to read settings file: {}", e))?;

    let export: SettingsExport = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse settings file: {}", e))?;

    // Validate version
    let file_major = export.version.split('.').next().unwrap_or("0");
    let current_major = SETTINGS_VERSION.split('.').next().unwrap_or("0");

    let mut warnings = Vec::new();
    if file_major != current_major {
        warnings.push(format!(
            "Version mismatch: file is v{}, expected v{}",
            export.version, SETTINGS_VERSION
        ));
    }

    let current_settings = load_app_settings()?;

    let mut imported = Vec::new();
    let mut merged = Vec::new();
    let mut skipped = Vec::new();
    let errors: Vec<String> = Vec::new();

    if merge {
        // Merge imported settings with current settings
        let merged_settings = merge_app_settings(&current_settings, &export.settings);

        // Track what changed
        track_settings_changes(
            &current_settings,
            &export.settings,
            &mut imported,
            &mut merged,
            &mut skipped,
        );

        // Save merged settings
        save_app_settings(&merged_settings)?;
    } else {
        // Replace all settings (except version)
        let mut new_settings = export.settings;
        new_settings.version = SETTINGS_VERSION.to_string();

        // Track what will be imported
        track_settings_changes(
            &current_settings,
            &new_settings,
            &mut imported,
            &mut merged,
            &mut skipped,
        );

        // Save new settings
        save_app_settings(&new_settings)?;
    }

    Ok(ImportReport {
        success: errors.is_empty(),
        imported,
        merged,
        skipped,
        errors,
        warnings,
    })
}

/// Preview what settings will change during import
#[tauri::command]
pub fn preview_settings_import(
    settings_path: String,
    current_settings: Option<AppSettings>,
) -> Result<Vec<SettingsChangePreview>, String> {
    let content = std::fs::read_to_string(&settings_path)
        .map_err(|e| format!("Failed to read settings file: {}", e))?;

    let export: SettingsExport = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse settings file: {}", e))?;

    let current = current_settings.unwrap_or_else(|| load_app_settings().unwrap_or_default());

    let mut preview = Vec::new();
    let settings_json = serde_json::to_value(&export.settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;
    let current_json = serde_json::to_value(&current)
        .map_err(|e| format!("Failed to serialize current settings: {}", e))?;

    if let serde_json::Value::Object(settings_map) = settings_json {
        for (key, new_value) in settings_map {
            // Skip null values
            if new_value.is_null() {
                continue;
            }

            let current_value = current_json.get(&key).cloned();
            let has_conflict = current_value.is_some()
                && current_value != Some(new_value.clone())
                && !is_empty_value(&current_value)
                && !is_empty_value(&Some(new_value.clone()));

            let will_change = current_value != Some(new_value.clone())
                || (current_value.is_none() && !is_empty_value(&Some(new_value.clone())));

            preview.push(SettingsChangePreview {
                category: get_setting_category(&key),
                key: key.clone(),
                display_name: get_display_name(&key),
                current_value,
                new_value,
                will_change,
                has_conflict,
            });
        }
    }

    // Sort by category then key
    preview.sort_by(|a, b| a.category.cmp(&b.category).then_with(|| a.key.cmp(&b.key)));

    Ok(preview)
}

/// Reset settings to defaults
#[tauri::command]
pub fn reset_settings_to_defaults() -> Result<(), String> {
    let default_settings = AppSettings::default();
    save_app_settings(&default_settings)?;
    Ok(())
}

/// Validate a settings file without importing
#[tauri::command]
pub fn validate_settings_file(settings_path: String) -> Result<serde_json::Value, String> {
    let content = std::fs::read_to_string(&settings_path)
        .map_err(|e| format!("Failed to read settings file: {}", e))?;

    let export: SettingsExport =
        serde_json::from_str(&content).map_err(|e| format!("Invalid settings file: {}", e))?;

    let file_major = export.version.split('.').next().unwrap_or("0");
    let current_major = SETTINGS_VERSION.split('.').next().unwrap_or("0");
    let version_compatible = file_major == current_major;

    let errors: Vec<String> = Vec::new();
    let mut warnings = Vec::new();

    if !version_compatible {
        warnings.push(format!(
            "Version mismatch: file is v{}, expected v{}",
            export.version, SETTINGS_VERSION
        ));
    }

    if export.exported_at.is_empty() {
        warnings.push("Missing export timestamp".to_string());
    }

    // Validate external tools if present
    if let Some(tools) = &export.settings.external_tools {
        for (i, tool) in tools.iter().enumerate() {
            if let Some(path) = tool.get("executable_path").and_then(|p| p.as_str()) {
                if !path.is_empty() && !std::path::Path::new(path).exists() {
                    warnings.push(format!(
                        "External tool #{} executable not found: {}",
                        i + 1,
                        path
                    ));
                }
            }
        }
    }

    // Validate emulator path if present
    if let Some(path) = &export.settings.emulator_path {
        if !path.is_empty() && !std::path::Path::new(path).exists() {
            warnings.push(format!("Emulator path not found: {}", path));
        }
    }

    let valid = errors.is_empty();

    Ok(serde_json::json!({
        "valid": valid,
        "version_compatible": version_compatible,
        "version": export.version,
        "exported_at": export.exported_at,
        "errors": errors,
        "warnings": warnings,
    }))
}

/// Get current app settings
#[tauri::command]
pub fn get_app_settings() -> Result<AppSettings, String> {
    load_app_settings()
}

/// Save app settings
#[tauri::command]
pub fn save_settings(settings: AppSettings) -> Result<(), String> {
    save_app_settings(&settings)
}

/// Update specific settings fields
#[tauri::command]
pub fn update_settings(updates: serde_json::Value) -> Result<AppSettings, String> {
    let mut current = load_app_settings()?;

    // Apply updates
    if let serde_json::Value::Object(map) = updates {
        for (key, value) in map {
            apply_setting_update(&mut current, &key, value)?;
        }
    }

    save_app_settings(&current)?;
    Ok(current)
}

/// Helper: Merge two AppSettings
fn merge_app_settings(current: &AppSettings, imported: &AppSettings) -> AppSettings {
    let mut merged = current.clone();

    // Merge simple fields (imported overrides current if Some)
    if imported.theme.is_some() {
        merged.theme = imported.theme.clone();
    }
    if imported.ui_scale.is_some() {
        merged.ui_scale = imported.ui_scale;
    }
    if imported.sidebar_collapsed.is_some() {
        merged.sidebar_collapsed = imported.sidebar_collapsed;
    }
    if imported.default_export_format.is_some() {
        merged.default_export_format = imported.default_export_format.clone();
    }
    if imported.auto_save_interval.is_some() {
        merged.auto_save_interval = imported.auto_save_interval;
    }
    if imported.confirm_on_close.is_some() {
        merged.confirm_on_close = imported.confirm_on_close;
    }
    if imported.show_tooltips.is_some() {
        merged.show_tooltips = imported.show_tooltips;
    }
    if imported.emulator_path.is_some() {
        merged.emulator_path = imported.emulator_path.clone();
    }
    if imported.emulator_type.is_some() {
        merged.emulator_type = imported.emulator_type.clone();
    }
    if imported.auto_save_before_launch.is_some() {
        merged.auto_save_before_launch = imported.auto_save_before_launch;
    }
    if imported.default_export_directory.is_some() {
        merged.default_export_directory = imported.default_export_directory.clone();
    }
    if imported.panel_layout.is_some() {
        merged.panel_layout = imported.panel_layout.clone();
    }

    // Merge arrays (combine and deduplicate)
    if let Some(new_projects) = &imported.recent_projects {
        let mut combined = merged.recent_projects.unwrap_or_default();
        combined.extend(new_projects.clone());
        // Deduplicate and limit to 10
        let seen: std::collections::HashSet<_> = combined.iter().cloned().collect();
        merged.recent_projects = Some(seen.into_iter().take(10).collect());
    }

    if let Some(new_roms) = &imported.recent_roms {
        let mut combined = merged.recent_roms.unwrap_or_default();
        combined.extend(new_roms.clone());
        let seen: std::collections::HashSet<_> = combined.iter().cloned().collect();
        merged.recent_roms = Some(seen.into_iter().take(10).collect());
    }

    // Merge external tools (replace all for now, could be smarter)
    if imported.external_tools.is_some() {
        merged.external_tools = imported.external_tools.clone();
    }

    // Merge maps
    if let Some(new_defaults) = &imported.default_tool_ids {
        let mut combined = merged.default_tool_ids.unwrap_or_default();
        combined.extend(new_defaults.clone());
        merged.default_tool_ids = Some(combined);
    }

    if let Some(new_shortcuts) = &imported.custom_shortcuts {
        let mut combined = merged.custom_shortcuts.unwrap_or_default();
        combined.extend(new_shortcuts.clone());
        merged.custom_shortcuts = Some(combined);
    }

    merged
}

/// Helper: Track what settings changed
fn track_settings_changes(
    current: &AppSettings,
    imported: &AppSettings,
    imported_list: &mut Vec<String>,
    merged_list: &mut Vec<String>,
    skipped_list: &mut Vec<String>,
) {
    macro_rules! track_field {
        ($field:ident) => {
            if imported.$field.is_some() {
                if current.$field == imported.$field {
                    skipped_list.push(stringify!($field).to_string());
                } else if current.$field.is_some() {
                    merged_list.push(stringify!($field).to_string());
                } else {
                    imported_list.push(stringify!($field).to_string());
                }
            }
        };
    }

    track_field!(theme);
    track_field!(ui_scale);
    track_field!(sidebar_collapsed);
    track_field!(default_export_format);
    track_field!(auto_save_interval);
    track_field!(confirm_on_close);
    track_field!(show_tooltips);
    track_field!(emulator_path);
    track_field!(emulator_type);
    track_field!(auto_save_before_launch);
    track_field!(default_export_directory);
    track_field!(recent_projects);
    track_field!(recent_roms);
    track_field!(external_tools);
    track_field!(default_tool_ids);
    track_field!(custom_shortcuts);
    track_field!(panel_layout);
}

/// Helper: Get category for a setting key
fn get_setting_category(key: &str) -> String {
    match key {
        "theme" | "ui_scale" | "sidebar_collapsed" => "Appearance".to_string(),
        "default_export_format" | "auto_save_interval" | "confirm_on_close" | "show_tooltips" => {
            "Editor".to_string()
        }
        "emulator_path" | "emulator_type" | "auto_save_before_launch" => "Emulator".to_string(),
        "default_export_directory" | "recent_projects" | "recent_roms" => "Paths".to_string(),
        "external_tools" | "default_tool_ids" => "External Tools".to_string(),
        "custom_shortcuts" => "Keyboard Shortcuts".to_string(),
        "panel_layout" => "Layout".to_string(),
        _ => "Other".to_string(),
    }
}

/// Helper: Get display name for a setting key
fn get_display_name(key: &str) -> String {
    match key {
        "theme" => "UI Theme",
        "ui_scale" => "UI Scale",
        "sidebar_collapsed" => "Sidebar Collapsed",
        "default_export_format" => "Default Export Format",
        "auto_save_interval" => "Auto-Save Interval",
        "confirm_on_close" => "Confirm on Close",
        "show_tooltips" => "Show Tooltips",
        "emulator_path" => "Emulator Path",
        "emulator_type" => "Emulator Type",
        "auto_save_before_launch" => "Auto-Save Before Launch",
        "default_export_directory" => "Default Export Directory",
        "recent_projects" => "Recent Projects",
        "recent_roms" => "Recent ROMs",
        "external_tools" => "External Tools",
        "default_tool_ids" => "Default Tool Assignments",
        "custom_shortcuts" => "Custom Shortcuts",
        "panel_layout" => "Panel Layout",
        _ => key,
    }
    .to_string()
}

/// Helper: Check if a value is empty/undefined
fn is_empty_value(value: &Option<serde_json::Value>) -> bool {
    match value {
        None => true,
        Some(v) => {
            v.is_null()
                || (v.is_array() && v.as_array().map(|a| a.is_empty()).unwrap_or(false))
                || (v.is_object() && v.as_object().map(|o| o.is_empty()).unwrap_or(false))
                || (v.is_string() && v.as_str().map(|s| s.is_empty()).unwrap_or(false))
        }
    }
}

/// Helper: Apply a single setting update
fn apply_setting_update(
    settings: &mut AppSettings,
    key: &str,
    value: serde_json::Value,
) -> Result<(), String> {
    match key {
        "theme" => settings.theme = value.as_str().map(|s| s.to_string()),
        "ui_scale" => settings.ui_scale = value.as_f64().map(|f| f as f32),
        "sidebar_collapsed" => settings.sidebar_collapsed = value.as_bool(),
        "default_export_format" => {
            settings.default_export_format = value.as_str().map(|s| s.to_string())
        }
        "auto_save_interval" => settings.auto_save_interval = value.as_i64().map(|i| i as i32),
        "confirm_on_close" => settings.confirm_on_close = value.as_bool(),
        "show_tooltips" => settings.show_tooltips = value.as_bool(),
        "emulator_path" => settings.emulator_path = value.as_str().map(|s| s.to_string()),
        "emulator_type" => settings.emulator_type = value.as_str().map(|s| s.to_string()),
        "auto_save_before_launch" => settings.auto_save_before_launch = value.as_bool(),
        "default_export_directory" => {
            settings.default_export_directory = value.as_str().map(|s| s.to_string())
        }
        "panel_layout" => settings.panel_layout = value.as_str().map(|s| s.to_string()),
        _ => return Err(format!("Unknown setting: {}", key)),
    }
    Ok(())
}

// ============================================================================
// Theme Settings Commands
// ============================================================================

/// Theme settings structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeSettings {
    /// Selected theme: "dark", "light", or "system"
    pub theme: String,
}

impl Default for ThemeSettings {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
        }
    }
}

/// Load theme settings from app settings
#[tauri::command]
pub fn load_theme_settings() -> Result<ThemeSettings, String> {
    let app_settings = load_app_settings()?;

    let theme = app_settings.theme.unwrap_or_else(|| "dark".to_string());

    // Validate theme value
    let valid_theme = match theme.as_str() {
        "dark" | "light" | "system" => theme,
        _ => "dark".to_string(),
    };

    Ok(ThemeSettings { theme: valid_theme })
}

/// Save theme settings to app settings
#[tauri::command]
pub fn save_theme_settings(theme: String) -> Result<(), String> {
    // Validate theme value
    let valid_theme = match theme.as_str() {
        "dark" | "light" | "system" => theme,
        _ => {
            return Err(format!(
                "Invalid theme value: {}. Must be 'dark', 'light', or 'system'",
                theme
            ))
        }
    };

    let mut app_settings = load_app_settings()?;
    app_settings.theme = Some(valid_theme);
    save_app_settings(&app_settings)?;

    Ok(())
}
