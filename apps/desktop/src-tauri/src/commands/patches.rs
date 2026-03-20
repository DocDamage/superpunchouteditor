//! Patch Export Commands
//!
//! Commands for exporting IPS/BPS patches and related functionality.

use tauri::State;

use crate::app_state::AppState;
use crate::utils::parse_offset;

/// Export an IPS patch of all pending writes vs the original ROM
#[tauri::command]
pub fn export_ips_patch(state: State<AppState>, output_path: String) -> Result<usize, String> {
    let rom_opt = state.rom.lock();
    let rom = rom_opt.as_ref().ok_or("No ROM loaded")?;

    // Build a scratch copy of the ROM with pending edits applied
    let mut edited = rom.data.clone();
    let pending = state.pending_writes.lock();
    let patch_count = pending.len();

    for (offset_str, bytes) in pending.iter() {
        let offset = parse_offset(offset_str)?;
        let len = bytes.len().min(edited.len() - offset);
        edited[offset..offset + len].copy_from_slice(&bytes[..len]);
    }

    drop(pending);

    patch_core::generate_ips(&rom.data, &edited, &output_path).map_err(|e| e.to_string())?;

    Ok(patch_count)
}

/// Export a BPS patch of all pending writes vs the original ROM
#[tauri::command]
pub fn export_bps_patch(state: State<AppState>, output_path: String) -> Result<usize, String> {
    let rom_opt = state.rom.lock();
    let rom = rom_opt.as_ref().ok_or("No ROM loaded")?;

    // Build a scratch copy of the ROM with pending edits applied
    let mut edited = rom.data.clone();
    let pending = state.pending_writes.lock();
    let patch_count = pending.len();

    for (offset_str, bytes) in pending.iter() {
        let offset = parse_offset(offset_str)?;
        let len = bytes.len().min(edited.len() - offset);
        edited[offset..offset + len].copy_from_slice(&bytes[..len]);
    }

    drop(pending);

    // Create BPS metadata
    let metadata = patch_core::BpsMetadata {
        patch_name: Some("Super Punch-Out!! Editor Patch".to_string()),
        author: None,
        description: None,
    };
    let patch_data = patch_core::generate_bps(&rom.data, &edited, &metadata).map_err(|e| e.to_string())?;
    std::fs::write(&output_path, patch_data).map_err(|e| e.to_string())?;

    Ok(patch_count)
}

/// Export patch notes along with a patch file
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

    // First generate the patch
    let rom_opt = state.rom.lock();
    let rom = rom_opt.as_ref().ok_or("No ROM loaded")?;
    let rom_data = rom.data.clone();

    let mut edited = rom_data.clone();
    let pending = state.pending_writes.lock();

    for (offset_str, bytes) in pending.iter() {
        let offset = parse_offset(offset_str)?;
        let len = bytes.len().min(edited.len() - offset);
        edited[offset..offset + len].copy_from_slice(&bytes[..len]);
    }

    drop(pending);
    drop(rom_opt);

    // Generate IPS patch
    patch_core::generate_ips(&rom_data, &edited, &patch_path)
        .map_err(|e| format!("Failed to generate patch: {}", e))?;

    // Generate patch notes
    let current_project = state.current_project.lock();
    let pending = state.pending_writes.lock();
    let manifest = state.manifest.lock();

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

    let mut notes = if let Some(project) = current_project.as_ref() {
        PatchNotes::generate_from_project(&project.file)
    } else {
        PatchNotes::generate_from_pending_writes(None, &pending, &boxer_names)
    };

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
    let content = notes.render(output_format);

    // Save patch notes
    std::fs::write(&notes_path, content)
        .map_err(|e| format!("Failed to save patch notes: {}", e))?;

    Ok(())
}
