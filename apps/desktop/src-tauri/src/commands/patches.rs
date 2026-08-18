//! Patch export commands backed by the canonical ROM session.

use std::collections::HashMap;
use std::io::Write;
use std::path::Path;

use tauri::State;

use crate::app_state::AppState;
use rom_core::EditOperation;

struct PatchSnapshot {
    base: Vec<u8>,
    current: Vec<u8>,
    transaction_count: usize,
    current_sha1: String,
}

fn patch_snapshot(state: &AppState) -> Result<PatchSnapshot, String> {
    let session_guard = state.rom_session.lock();
    let session = session_guard.as_ref().ok_or("No ROM loaded")?;
    let current = session.materialize().map_err(|e| e.to_string())?;
    Ok(PatchSnapshot {
        base: session.base().bytes().to_vec(),
        current: current.bytes,
        transaction_count: session.journal().active_transactions().len(),
        current_sha1: current.current_sha1,
    })
}

fn write_atomic(path: &str, bytes: &[u8]) -> Result<(), String> {
    let destination = Path::new(path);
    let parent = destination.parent().unwrap_or_else(|| Path::new("."));
    if !parent.exists() {
        return Err(format!("Destination directory does not exist: {}", parent.display()));
    }
    let mut temp = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
    temp.write_all(bytes).map_err(|e| e.to_string())?;
    temp.as_file().sync_all().map_err(|e| e.to_string())?;
    temp.persist(destination)
        .map_err(|error| format!("Could not persist patch atomically: {}", error.error))?;
    Ok(())
}

#[tauri::command]
pub fn export_ips_patch(state: State<AppState>, output_path: String) -> Result<usize, String> {
    let snapshot = patch_snapshot(&state)?;
    let patch = patch_core::generate_ips_bytes(&snapshot.base, &snapshot.current)
        .map_err(|e| e.to_string())?;
    let verified = patch_core::apply_ips(&snapshot.base, &patch).map_err(|e| e.to_string())?;
    if verified != snapshot.current {
        return Err(format!(
            "IPS verification failed for editor revision hash {}",
            snapshot.current_sha1
        ));
    }
    write_atomic(&output_path, &patch)?;
    Ok(snapshot.transaction_count)
}

#[tauri::command]
pub fn export_bps_patch(state: State<AppState>, output_path: String) -> Result<usize, String> {
    let snapshot = patch_snapshot(&state)?;
    let metadata = patch_core::BpsMetadata {
        patch_name: Some("Super Punch-Out!! Editor Patch".to_string()),
        author: None,
        description: Some(format!("working-sha1={}", snapshot.current_sha1)),
    };
    let patch = patch_core::generate_bps(&snapshot.base, &snapshot.current, &metadata)
        .map_err(|e| e.to_string())?;
    let verified = patch_core::apply_bps(&snapshot.base, &patch).map_err(|e| e.to_string())?;
    if verified != snapshot.current {
        return Err(format!(
            "BPS verification failed for editor revision hash {}",
            snapshot.current_sha1
        ));
    }
    write_atomic(&output_path, &patch)?;
    Ok(snapshot.transaction_count)
}

fn journal_pending_projection(state: &AppState) -> Result<HashMap<String, Vec<u8>>, String> {
    let session_guard = state.rom_session.lock();
    let session = session_guard.as_ref().ok_or("No ROM loaded")?;
    let mut projected = HashMap::new();
    for transaction in session.journal().active_transactions() {
        for operation in &transaction.operations {
            if let EditOperation::WriteBytes { offset, after, .. } = operation {
                projected.insert(format!("0x{offset:X}"), after.clone());
            }
        }
    }
    Ok(projected)
}

#[tauri::command]
pub fn export_patch_notes_with_patch(
    state: State<AppState>,
    patch_path: String,
    notes_path: String,
    format: String,
    title: Option<String>,
    author: Option<String>,
    version: Option<String>,
) -> Result<(), String> {
    use project_core::{OutputFormat, PatchNotes};

    let snapshot = patch_snapshot(&state)?;
    let patch = patch_core::generate_ips_bytes(&snapshot.base, &snapshot.current)
        .map_err(|e| e.to_string())?;
    let verified = patch_core::apply_ips(&snapshot.base, &patch).map_err(|e| e.to_string())?;
    if verified != snapshot.current {
        return Err("Patch verification failed before writing patch notes".to_string());
    }
    write_atomic(&patch_path, &patch)?;

    let current_project = state.current_project.lock();
    let projected = journal_pending_projection(&state)?;
    let manifest = state.manifest.lock();

    let mut boxer_names = HashMap::new();
    for boxer in manifest.fighters.values() {
        for asset in boxer
            .palette_files
            .iter()
            .chain(boxer.unique_sprite_bins.iter())
            .chain(boxer.shared_sprite_bins.iter())
        {
            boxer_names.insert(asset.start_pc.clone(), boxer.name.clone());
        }
    }

    let mut notes = if let Some(project) = current_project.as_ref() {
        PatchNotes::generate_from_project(&project.file)
    } else {
        PatchNotes::generate_from_pending_writes(None, &projected, &boxer_names)
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

    let output_format = OutputFormat::from_string(&format).unwrap_or(OutputFormat::Markdown);
    let mut content = notes.render(output_format);
    content.push_str(&format!(
        "\n\nWorking ROM SHA-1: {}\nTransactions: {}\n",
        snapshot.current_sha1, snapshot.transaction_count
    ));
    write_atomic(&notes_path, content.as_bytes())
}
