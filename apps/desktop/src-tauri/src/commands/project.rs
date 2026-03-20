//! Project Management Commands
//!
//! Commands for creating, loading, and saving project files.

use std::path::PathBuf;

use chrono::Utc;
use tauri::State;

use crate::app_state::AppState;
use project_core::{
    ChangeSummary, EditType, OutputFormat, PatchNotes, Project, ProjectEdit, ProjectFile,
    ProjectMetadata,
};

/// Create a new project at the specified path
#[tauri::command]
pub fn create_project(
    state: State<AppState>,
    project_path: String,
    name: String,
    author: Option<String>,
    description: Option<String>,
) -> Result<ProjectFile, String> {
    let rom_opt = state.rom.lock();
    let rom = rom_opt.as_ref().ok_or("No ROM loaded")?;
    let rom_sha1 = rom.calculate_sha1();
    drop(rom_opt);

    let metadata = ProjectMetadata {
        name,
        author,
        description,
        created_at: Utc::now(),
        modified_at: Utc::now(),
        version: "0.1.0".to_string(),
    };

    let path = PathBuf::from(project_path);
    let manifest_version = "1.0".to_string();

    let project = Project::create(&path, &rom_sha1, &manifest_version, metadata)
        .map_err(|e| e.to_string())?;

    // Store as current project
    let project_file = project.file.clone();
    *state.current_project.lock() = Some(project);

    Ok(project_file)
}

/// Save the current state as a project
#[tauri::command]
pub fn save_project(
    state: State<AppState>,
    project_path: Option<String>,
    metadata: Option<ProjectMetadata>,
) -> Result<ProjectFile, String> {
    let rom_opt = state.rom.lock();
    let rom = rom_opt.as_ref().ok_or("No ROM loaded")?;
    let rom_sha1 = rom.calculate_sha1();
    drop(rom_opt);

    let pending = state.pending_writes.lock();

    // Get or create project
    let project = if let Some(path) = project_path {
        // Create new project at path
        let path = PathBuf::from(path);
        let manifest_version = "1.0".to_string();
        let meta = metadata.unwrap_or_else(|| ProjectMetadata {
            name: "Untitled Project".to_string(),
            author: None,
            description: None,
            created_at: Utc::now(),
            modified_at: Utc::now(),
            version: "0.1.0".to_string(),
        });
        Project::create(&path, &rom_sha1, &manifest_version, meta).map_err(|e| e.to_string())?
    } else {
        // Update existing project
        let mut current = state.current_project.lock();
        if let Some(ref mut proj) = current.as_mut() {
            // Update metadata if provided
            if let Some(meta) = metadata {
                proj.file.metadata = meta;
            }
            proj.file.metadata.modified_at = Utc::now();
            proj.file.rom_base_sha1 = rom_sha1;

            // Clear old edits and rebuild from pending writes
            proj.file.edits.clear();

            // Add edits for each pending write
            for (pc_offset, bytes) in pending.iter() {
                let edit = ProjectEdit {
                    asset_id: format!("edit_{}", pc_offset),
                    edit_type: EditType::SpriteBin,
                    description: Some("Sprite bin modification".to_string()),
                    original_hash: String::new(),
                    edited_hash: format!("{:x}", md5::compute(bytes)),
                    pc_offset: pc_offset.clone(),
                    size: bytes.len(),
                    timestamp: Utc::now(),
                    asset_path: None,
                };
                proj.file.add_edit(edit);
            }

            // Clone to return
            let proj_clone = proj.clone();
            drop(current);
            proj_clone
        } else {
            return Err(
                "No project open. Provide project_path to create a new project.".to_string(),
            );
        }
    };

    drop(pending);

    // Save the project
    project.save().map_err(|e| e.to_string())?;

    // Update current project reference
    let file = project.file.clone();
    *state.current_project.lock() = Some(project);

    Ok(file)
}

/// Load a project from the specified path
#[tauri::command]
pub fn load_project(state: State<AppState>, project_path: String) -> Result<ProjectFile, String> {
    let path = PathBuf::from(project_path);
    let project = Project::load(&path).map_err(|e| e.to_string())?;

    // Validate ROM is loaded and matches
    let rom_opt = state.rom.lock();
    if let Some(rom) = rom_opt.as_ref() {
        let rom_sha1 = rom.calculate_sha1();
        if let Err(e) = project.validate_rom(&rom_sha1) {
            return Err(format!("ROM mismatch: {}", e));
        }
    }
    drop(rom_opt);

    let file = project.file.clone();
    *state.current_project.lock() = Some(project);

    Ok(file)
}

/// Validate a project against the currently loaded ROM
#[tauri::command]
pub fn validate_project(state: State<AppState>, project_path: String) -> Result<bool, String> {
    let path = PathBuf::from(project_path);
    let project = Project::load(&path).map_err(|e| e.to_string())?;

    let rom_opt = state.rom.lock();
    if let Some(rom) = rom_opt.as_ref() {
        let rom_sha1 = rom.calculate_sha1();
        Ok(project.validate_rom(&rom_sha1).is_ok())
    } else {
        Err("No ROM loaded".to_string())
    }
}

/// Get the currently open project info
#[tauri::command]
pub fn get_current_project(state: State<AppState>) -> Option<ProjectFile> {
    state
        .current_project
        .lock()
        .as_ref()
        .map(|proj| proj.file.clone())
}

/// Get the path of the currently open project
#[tauri::command]
pub fn get_current_project_path(state: State<AppState>) -> Option<String> {
    state.current_project.lock().as_ref()
        .map(|proj| proj.path.to_string_lossy().to_string())
}

/// Close the current project
#[tauri::command]
pub fn close_project(state: State<AppState>) {
    *state.current_project.lock() = None;
}

// ============================================================================
// Patch Notes Generator Commands
// ============================================================================

/// Generate patch notes from the current project and pending writes
#[tauri::command]
pub fn generate_patch_notes(
    state: State<AppState>,
    format: String,
    title: Option<String>,
    author: Option<String>,
    version: Option<String>,
) -> Result<String, String> {
    let current_project = state.current_project.lock();
    let pending = state.pending_writes.lock();
    let manifest = state.manifest.lock();

    // Build boxer name mapping from manifest
    let mut boxer_names: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();
    for (_fighter_name, boxer) in &manifest.fighters {
        for asset in boxer
            .palette_files
            .iter()
            .chain(boxer.unique_sprite_bins.iter())
            .chain(boxer.shared_sprite_bins.iter())
        {
            boxer_names.insert(asset.start_pc.clone(), boxer.name.clone());
        }
    }

    // Generate patch notes
    let mut notes = if let Some(project) = current_project.as_ref() {
        PatchNotes::generate_from_project(&project.file)
    } else {
        PatchNotes::generate_from_pending_writes(None, &pending, &boxer_names)
    };

    // Override with provided metadata
    if let Some(t) = title {
        notes.title = t;
    }
    if let Some(a) = author {
        notes.author = a;
    }
    if let Some(v) = version {
        notes.version = v;
    }

    let output_format = OutputFormat::from_string(&format).unwrap_or(OutputFormat::Markdown);

    Ok(notes.render(output_format))
}

/// Get a change summary for the current project
#[tauri::command]
pub fn get_change_summary(state: State<AppState>) -> Result<ChangeSummary, String> {
    let current_project = state.current_project.lock();
    let pending = state.pending_writes.lock();
    let manifest = state.manifest.lock();

    // Build boxer name mapping from manifest
    let mut boxer_names: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();
    for (_fighter_name, boxer) in &manifest.fighters {
        for asset in boxer
            .palette_files
            .iter()
            .chain(boxer.unique_sprite_bins.iter())
            .chain(boxer.shared_sprite_bins.iter())
        {
            boxer_names.insert(asset.start_pc.clone(), boxer.name.clone());
        }
    }

    let summary = if let Some(project) = current_project.as_ref() {
        let notes = PatchNotes::generate_from_project(&project.file);
        notes.summary
    } else {
        project_core::patch_notes::get_change_summary(&pending, &boxer_names)
    };

    Ok(summary)
}

/// Save patch notes to a file
#[tauri::command]
pub fn save_patch_notes(content: String, output_path: String) -> Result<(), String> {
    std::fs::write(&output_path, content).map_err(|e| format!("Failed to save patch notes: {}", e))
}
