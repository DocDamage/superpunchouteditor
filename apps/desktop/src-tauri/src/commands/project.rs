//! Project management commands.
//!
//! Project format v2 is the persistent source of truth. `ProjectFile` v1 values are retained only
//! as a compatibility DTO for the current frontend while its project UI migrates.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use chrono::Utc;
use tauri::State;

use crate::app_state::AppState;
use project_core::{
    assess_v1_migration, load_project_v2, save_project_v2, ChangeSummary, EditType, OutputFormat,
    PatchNotes, Project, ProjectDocumentV2, ProjectEdit, ProjectFile, ProjectMetadata,
    PROJECT_V2_FILENAME,
};
use rom_core::{EditJournal, EditOperation, Rom};

fn current_base_sha1(state: &AppState) -> Result<String, String> {
    let session_guard = state.rom_session.lock();
    let session = session_guard.as_ref().ok_or("No ROM loaded")?;
    Ok(session.base().sha1().to_string())
}

fn legacy_file_from_session(
    state: &AppState,
    metadata: ProjectMetadata,
    template: Option<&ProjectFile>,
) -> Result<ProjectFile, String> {
    let session_guard = state.rom_session.lock();
    let session = session_guard.as_ref().ok_or("No ROM loaded")?;
    let mut file = ProjectFile::new(session.base().sha1(), "2.0", metadata);
    if let Some(template) = template {
        file.assets = template.assets.clone();
        file.settings = template.settings.clone();
        file.duplicated_banks = template.duplicated_banks.clone();
        file.thumbnail = template.thumbnail.clone();
        file.source_region = template.source_region.clone();
    } else {
        file.source_region = session
            .base()
            .region()
            .map(|region| region.as_str().to_string());
    }

    for transaction in session.journal().active_transactions() {
        for (index, operation) in transaction.operations.iter().enumerate() {
            if let EditOperation::WriteBytes {
                offset,
                before,
                after,
                asset_id,
                description,
            } = operation
            {
                file.edits.push(ProjectEdit {
                    asset_id: asset_id
                        .clone()
                        .unwrap_or_else(|| format!("tx{}_{}", transaction.id, index)),
                    edit_type: EditType::Other,
                    description: description
                        .clone()
                        .or_else(|| Some(transaction.label.clone())),
                    original_hash: format!("{:x}", md5::compute(before)),
                    edited_hash: format!("{:x}", md5::compute(after)),
                    pc_offset: format!("0x{offset:X}"),
                    size: after.len(),
                    timestamp: Utc::now(),
                    asset_path: None,
                });
            }
        }
    }
    Ok(file)
}

fn build_v2_document(
    state: &AppState,
    metadata: ProjectMetadata,
    legacy: Option<&ProjectFile>,
) -> Result<ProjectDocumentV2, String> {
    let display_filename = state
        .rom_path
        .lock()
        .as_ref()
        .and_then(|path| Path::new(path).file_name())
        .map(|name| name.to_string_lossy().to_string());
    let session_guard = state.rom_session.lock();
    let session = session_guard.as_ref().ok_or("No ROM loaded")?;
    let mut document = ProjectDocumentV2::from_session(
        env!("CARGO_PKG_VERSION"),
        metadata,
        session,
        display_filename,
    )
    .map_err(|e| e.to_string())?;
    if let Some(legacy) = legacy {
        document.settings = legacy.settings.clone();
        document.duplicated_banks = legacy.duplicated_banks.clone();
        document.thumbnail = legacy.thumbnail.clone();
    }
    Ok(document)
}

fn install_loaded_document(state: &AppState, document: &ProjectDocumentV2) -> Result<(), String> {
    let mut session_guard = state.rom_session.lock();
    let session = session_guard.as_mut().ok_or("No ROM loaded")?;
    document
        .validate_against_base(session.base())
        .map_err(|e| e.to_string())?;

    // `replace_journal` validates the entire materialization before mutating the active session.
    session
        .replace_journal(document.journal.clone())
        .map_err(|e| e.to_string())?;
    let materialized = session.materialize().map_err(|e| e.to_string())?;
    let dirty = session.journal().is_dirty();
    drop(session_guard);

    *state.rom.lock() = Some(Rom::new(materialized.bytes));
    *state.modified.lock() = dirty;
    state.pending_writes.lock().clear();
    state.edit_history.lock().clear();
    Ok(())
}

#[tauri::command]
pub fn create_project(
    state: State<AppState>,
    project_path: String,
    name: String,
    author: Option<String>,
    description: Option<String>,
) -> Result<ProjectFile, String> {
    let metadata = ProjectMetadata {
        name,
        author,
        description,
        created_at: Utc::now(),
        modified_at: Utc::now(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    };
    let path = PathBuf::from(project_path);
    let document = build_v2_document(&state, metadata.clone(), None)?;
    save_project_v2(&path, &document).map_err(|e| e.to_string())?;

    let file = legacy_file_from_session(&state, metadata, None)?;
    *state.current_project.lock() = Some(Project {
        path,
        file: file.clone(),
    });
    Ok(file)
}

#[tauri::command]
pub fn save_project(
    state: State<AppState>,
    project_path: Option<String>,
    metadata: Option<ProjectMetadata>,
) -> Result<ProjectFile, String> {
    let (path, template, existing_metadata) = {
        let current = state.current_project.lock();
        let path = if let Some(path) = project_path {
            PathBuf::from(path)
        } else if let Some(project) = current.as_ref() {
            project.path.clone()
        } else {
            return Err("No project open. Provide project_path to create one.".to_string());
        };
        let template = current.as_ref().map(|project| project.file.clone());
        let existing_metadata = template.as_ref().map(|file| file.metadata.clone());
        (path, template, existing_metadata)
    };

    let mut metadata = metadata.or(existing_metadata).unwrap_or_default();
    metadata.modified_at = Utc::now();
    metadata.version = env!("CARGO_PKG_VERSION").to_string();

    let document = build_v2_document(&state, metadata.clone(), template.as_ref())?;
    save_project_v2(&path, &document).map_err(|e| e.to_string())?;
    let file = legacy_file_from_session(&state, metadata, template.as_ref())?;
    *state.current_project.lock() = Some(Project {
        path,
        file: file.clone(),
    });
    Ok(file)
}

#[tauri::command]
pub fn load_project(state: State<AppState>, project_path: String) -> Result<ProjectFile, String> {
    let path = PathBuf::from(project_path);
    if !state.has_rom() {
        return Err("Load the project's base ROM before opening the project".to_string());
    }

    if path.join(PROJECT_V2_FILENAME).exists() {
        let document = load_project_v2(&path).map_err(|e| e.to_string())?;
        install_loaded_document(&state, &document)?;
        let file = legacy_file_from_session(&state, document.metadata.clone(), None)?;
        *state.current_project.lock() = Some(Project {
            path,
            file: file.clone(),
        });
        return Ok(file);
    }

    // Read-only v1 compatibility path. Never claim that metadata-only edit records were restored.
    let legacy = Project::load(&path).map_err(|e| e.to_string())?;
    let base_sha1 = current_base_sha1(&state)?;
    legacy.validate_rom(&base_sha1).map_err(|e| e.to_string())?;
    let assessment = assess_v1_migration(&legacy.file);
    if !assessment.can_reconstruct_edits {
        return Err(assessment.explanation);
    }

    let dirty = state
        .rom_session
        .lock()
        .as_ref()
        .map(|session| session.journal().is_dirty())
        .unwrap_or(false);
    if dirty {
        return Err(
            "Current ROM session has unsaved edits; save or discard them before loading a v1 project"
                .to_string(),
        );
    }
    {
        let mut session_guard = state.rom_session.lock();
        let session = session_guard.as_mut().ok_or("No ROM loaded")?;
        session
            .replace_journal(EditJournal::new())
            .map_err(|e| e.to_string())?;
    }
    let file = legacy.file.clone();
    *state.current_project.lock() = Some(legacy);
    Ok(file)
}

#[tauri::command]
pub fn validate_project(state: State<AppState>, project_path: String) -> Result<bool, String> {
    let path = PathBuf::from(project_path);
    if path.join(PROJECT_V2_FILENAME).exists() {
        let document = load_project_v2(&path).map_err(|e| e.to_string())?;
        let session_guard = state.rom_session.lock();
        let session = session_guard.as_ref().ok_or("No ROM loaded")?;
        return Ok(document.validate_against_base(session.base()).is_ok());
    }

    let legacy = Project::load(&path).map_err(|e| e.to_string())?;
    let base_sha1 = current_base_sha1(&state)?;
    let assessment = assess_v1_migration(&legacy.file);
    Ok(legacy.validate_rom(&base_sha1).is_ok() && assessment.can_reconstruct_edits)
}

#[tauri::command]
pub fn get_current_project(state: State<AppState>) -> Option<ProjectFile> {
    state
        .current_project
        .lock()
        .as_ref()
        .map(|project| project.file.clone())
}

#[tauri::command]
pub fn get_current_project_path(state: State<AppState>) -> Option<String> {
    state
        .current_project
        .lock()
        .as_ref()
        .map(|project| project.path.to_string_lossy().to_string())
}

#[tauri::command]
pub fn close_project(state: State<AppState>) {
    *state.current_project.lock() = None;
}

fn journal_projection(state: &AppState) -> Result<HashMap<String, Vec<u8>>, String> {
    let session_guard = state.rom_session.lock();
    let session = session_guard.as_ref().ok_or("No ROM loaded")?;
    let mut writes = HashMap::new();
    for transaction in session.journal().active_transactions() {
        for operation in &transaction.operations {
            if let EditOperation::WriteBytes { offset, after, .. } = operation {
                writes.insert(format!("0x{offset:X}"), after.clone());
            }
        }
    }
    Ok(writes)
}

fn boxer_names(state: &AppState) -> HashMap<String, String> {
    let manifest = state.manifest.lock();
    let mut names = HashMap::new();
    for boxer in manifest.fighters.values() {
        for asset in boxer
            .palette_files
            .iter()
            .chain(boxer.unique_sprite_bins.iter())
            .chain(boxer.shared_sprite_bins.iter())
        {
            names.insert(asset.start_pc.clone(), boxer.name.clone());
        }
    }
    names
}

#[tauri::command]
pub fn generate_patch_notes(
    state: State<AppState>,
    format: String,
    title: Option<String>,
    author: Option<String>,
    version: Option<String>,
) -> Result<String, String> {
    let current_project = state.current_project.lock();
    let projected = journal_projection(&state)?;
    let names = boxer_names(&state);
    let mut notes = if let Some(project) = current_project.as_ref() {
        PatchNotes::generate_from_project(&project.file)
    } else {
        PatchNotes::generate_from_pending_writes(None, &projected, &names)
    };
    if let Some(value) = title {
        notes.title = value;
    }
    if let Some(value) = author {
        notes.author = value;
    }
    if let Some(value) = version {
        notes.version = value;
    }
    let format = OutputFormat::from_string(&format).unwrap_or(OutputFormat::Markdown);
    Ok(notes.render(format))
}

#[tauri::command]
pub fn get_change_summary(state: State<AppState>) -> Result<ChangeSummary, String> {
    let current_project = state.current_project.lock();
    if let Some(project) = current_project.as_ref() {
        return Ok(PatchNotes::generate_from_project(&project.file).summary);
    }
    let projected = journal_projection(&state)?;
    let names = boxer_names(&state);
    Ok(project_core::patch_notes::get_change_summary(
        &projected, &names,
    ))
}

#[tauri::command]
pub fn save_patch_notes(content: String, output_path: String) -> Result<(), String> {
    std::fs::write(&output_path, content).map_err(|e| format!("Failed to save patch notes: {e}"))
}
