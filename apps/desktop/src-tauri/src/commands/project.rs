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
            .map(|region| region.code().to_string());
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

// ============================================================================
// Automatic safekeeping
// ============================================================================

/// Folder that holds automatically kept work for one base ROM. It lives in the
/// app's own data directory and is keyed by the ROM's SHA-1, so work can only
/// ever be restored onto the exact game it was made with.
fn autosave_directory(root: &Path, base_sha1: &str) -> PathBuf {
    root.join("autosave").join(base_sha1)
}

fn autosave_root() -> Result<PathBuf, String> {
    dirs::data_local_dir()
        .map(|dir| dir.join("super-punch-out-editor"))
        .ok_or_else(|| "Could not find the app data folder".to_string())
}

/// Keep the current edit journal in `root` so it survives closing the editor.
///
/// When there is nothing to keep (no changes, or every change was undone and
/// no redo remains) any earlier copy is removed instead. Returns the number of
/// changes kept. Like a project, the copy never contains the base ROM.
pub fn autosave_session_to(state: &AppState, root: &Path) -> Result<usize, String> {
    let (base_sha1, kept) = {
        let session_guard = state.rom_session.lock();
        let session = session_guard.as_ref().ok_or("No ROM loaded")?;
        (
            session.base().sha1().to_string(),
            session.journal().transactions().len(),
        )
    };
    let directory = autosave_directory(root, &base_sha1);

    if kept == 0 {
        if directory.exists() {
            std::fs::remove_dir_all(&directory)
                .map_err(|e| format!("Could not clear kept work: {e}"))?;
        }
        return Ok(0);
    }

    let metadata = ProjectMetadata {
        name: "Automatically kept work".to_string(),
        author: None,
        description: Some("Kept automatically so changes survive closing the editor".to_string()),
        created_at: Utc::now(),
        modified_at: Utc::now(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    };
    let document = build_v2_document(state, metadata, None)?;
    save_project_v2(&directory, &document).map_err(|e| e.to_string())?;
    Ok(state
        .rom_session
        .lock()
        .as_ref()
        .map(|session| session.journal().active_transactions().len())
        .unwrap_or(0))
}

/// Bring back automatically kept work for the loaded ROM.
///
/// Nothing is restored when the session already has changes, when no copy
/// exists for this exact ROM, or when the copy fails validation; in each of
/// those cases the current session is left untouched. Returns the number of
/// changes restored.
pub fn restore_autosave_from(state: &AppState, root: &Path) -> Result<usize, String> {
    let base_sha1 = {
        let session_guard = state.rom_session.lock();
        let session = session_guard.as_ref().ok_or("No ROM loaded")?;
        if !session.journal().transactions().is_empty() {
            return Ok(0);
        }
        session.base().sha1().to_string()
    };

    let directory = autosave_directory(root, &base_sha1);
    if !directory.join(PROJECT_V2_FILENAME).exists() {
        return Ok(0);
    }

    let document = load_project_v2(&directory).map_err(|e| e.to_string())?;
    install_loaded_document(state, &document)?;
    Ok(state
        .rom_session
        .lock()
        .as_ref()
        .map(|session| session.journal().active_transactions().len())
        .unwrap_or(0))
}

/// Keep the current changes safe without the user having to save.
#[tauri::command]
pub fn autosave_session(state: State<AppState>) -> Result<usize, String> {
    autosave_session_to(state.inner(), &autosave_root()?)
}

/// Bring back the changes that were kept automatically for this ROM.
#[tauri::command]
pub fn restore_autosave(state: State<AppState>) -> Result<usize, String> {
    restore_autosave_from(state.inner(), &autosave_root()?)
}

#[cfg(test)]
mod autosave_tests {
    use super::*;
    use manifest_core::Manifest;

    fn state_with_rom(bytes: Vec<u8>) -> AppState {
        let state = AppState::new(Manifest::empty());
        state.install_rom_session(Rom::new(bytes), "synthetic.sfc".into());
        state
    }

    #[test]
    fn kept_work_comes_back_after_reopening_the_same_rom() {
        let root = tempfile::tempdir().unwrap();
        let first = state_with_rom(vec![0; 32]);
        first
            .commit_rom_write("one", 2, vec![7, 7], None, None)
            .unwrap();
        first
            .commit_rom_write("two", 10, vec![9], None, None)
            .unwrap();
        assert_eq!(autosave_session_to(&first, root.path()).unwrap(), 2);
        let edited = first.materialize_current_rom().unwrap().bytes;

        // A new editor session with the same ROM.
        let second = state_with_rom(vec![0; 32]);
        assert_eq!(restore_autosave_from(&second, root.path()).unwrap(), 2);
        assert_eq!(second.materialize_current_rom().unwrap().bytes, edited);

        // The restored changes are still individual Undo steps.
        second.undo_journal().unwrap().unwrap();
        let after_undo = second.materialize_current_rom().unwrap().bytes;
        assert_eq!(after_undo[10], 0);
        assert_eq!(&after_undo[2..4], &[7, 7]);
    }

    #[test]
    fn kept_work_is_never_applied_to_a_different_rom() {
        let root = tempfile::tempdir().unwrap();
        let first = state_with_rom(vec![0; 32]);
        first
            .commit_rom_write("one", 2, vec![7], None, None)
            .unwrap();
        autosave_session_to(&first, root.path()).unwrap();

        let other = state_with_rom(vec![1; 32]);
        assert_eq!(restore_autosave_from(&other, root.path()).unwrap(), 0);
        assert_eq!(other.materialize_current_rom().unwrap().bytes, vec![1; 32]);
    }

    #[test]
    fn restore_never_replaces_changes_already_in_the_session() {
        let root = tempfile::tempdir().unwrap();
        let first = state_with_rom(vec![0; 32]);
        first
            .commit_rom_write("kept", 2, vec![7], None, None)
            .unwrap();
        autosave_session_to(&first, root.path()).unwrap();

        let second = state_with_rom(vec![0; 32]);
        second
            .commit_rom_write("new", 5, vec![3], None, None)
            .unwrap();
        assert_eq!(restore_autosave_from(&second, root.path()).unwrap(), 0);
        let bytes = second.materialize_current_rom().unwrap().bytes;
        assert_eq!(bytes[5], 3);
        assert_eq!(bytes[2], 0);
    }

    #[test]
    fn a_session_with_no_changes_clears_the_kept_copy() {
        let root = tempfile::tempdir().unwrap();
        let state = state_with_rom(vec![0; 32]);
        state
            .commit_rom_write("one", 2, vec![7], None, None)
            .unwrap();
        autosave_session_to(&state, root.path()).unwrap();

        let fresh = state_with_rom(vec![0; 32]);
        assert_eq!(autosave_session_to(&fresh, root.path()).unwrap(), 0);
        assert_eq!(restore_autosave_from(&fresh, root.path()).unwrap(), 0);
    }

    #[test]
    fn a_damaged_kept_copy_leaves_the_session_untouched() {
        let root = tempfile::tempdir().unwrap();
        let state = state_with_rom(vec![0; 32]);
        state
            .commit_rom_write("one", 2, vec![7], None, None)
            .unwrap();
        autosave_session_to(&state, root.path()).unwrap();

        let sha1 = state.get_rom_sha1().unwrap();
        let manifest = autosave_directory(root.path(), &sha1).join(PROJECT_V2_FILENAME);
        std::fs::write(&manifest, b"{ not valid json").unwrap();

        let fresh = state_with_rom(vec![0; 32]);
        assert!(restore_autosave_from(&fresh, root.path()).is_err());
        assert_eq!(fresh.materialize_current_rom().unwrap().bytes, vec![0; 32]);
    }
}
