//! Layout Pack Commands
//!
//! Commands for importing/exporting layout packs.

use std::path::PathBuf;

use chrono::Utc;
use tauri::State;

use crate::app_state::AppState;
use crate::types::*;
use crate::utils::parse_offset;

/// Export a layout pack for selected boxers
#[tauri::command]
pub fn export_layout_pack(
    state: State<AppState>,
    boxer_keys: Vec<String>,
    metadata: LayoutPackMetadata,
    output_path: String,
) -> Result<(), String> {
    let manifest = state.manifest.lock();
    let mut layouts = Vec::new();

    for key in boxer_keys {
        let boxer = manifest
            .fighters
            .values()
            .find(|f| f.key == key)
            .ok_or_else(|| format!("Boxer '{}' not found", key))?;

        let mut bins = Vec::new();

        // Add unique bins
        for bin in &boxer.unique_sprite_bins {
            bins.push(LayoutBin {
                filename: bin.filename.clone(),
                pc_offset: bin.start_pc.clone(),
                size: bin.size,
                category: bin.category.clone(),
                label: None,
            });
        }

        // Add shared bins
        for bin in &boxer.shared_sprite_bins {
            bins.push(LayoutBin {
                filename: bin.filename.clone(),
                pc_offset: bin.start_pc.clone(),
                size: bin.size,
                category: bin.category.clone(),
                label: Some(format!("Shared with {}", bin.shared_with.join(", "))),
            });
        }

        layouts.push(PackBoxerLayout {
            boxer_key: key.clone(),
            version: "1.0".to_string(),
            layout_type: "reference".to_string(),
            bins,
            notes: None,
        });
    }

    drop(manifest);

    let pack = LayoutPack {
        version: LAYOUT_PACK_VERSION.to_string(),
        name: metadata.name,
        author: metadata.author,
        description: metadata.description,
        created_at: Utc::now().to_rfc3339(),
        layouts,
    };

    let json = serde_json::to_string_pretty(&pack)
        .map_err(|e| format!("Failed to serialize pack: {}", e))?;

    std::fs::write(&output_path, json).map_err(|e| format!("Failed to write pack: {}", e))?;

    Ok(())
}

/// Import a layout pack from file
#[tauri::command]
pub fn import_layout_pack(pack_path: String) -> Result<LayoutPack, String> {
    let content =
        std::fs::read_to_string(&pack_path).map_err(|e| format!("Failed to read pack: {}", e))?;

    let pack: LayoutPack =
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse pack: {}", e))?;

    Ok(pack)
}

/// Validate a layout pack without applying it
#[tauri::command]
pub fn validate_layout_pack(
    state: State<AppState>,
    pack_path: String,
) -> Result<ValidationReport, String> {
    let content =
        std::fs::read_to_string(&pack_path).map_err(|e| format!("Failed to read pack: {}", e))?;

    let pack: LayoutPack =
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse pack: {}", e))?;

    let manifest = state.manifest.lock();

    let mut boxer_validations = Vec::new();
    let mut warnings = Vec::new();
    let errors = Vec::new();

    // Check version compatibility
    let version_compatible = pack.version == LAYOUT_PACK_VERSION;
    if !version_compatible {
        warnings.push(format!(
            "Pack version {} may not be fully compatible with expected version {}",
            pack.version, LAYOUT_PACK_VERSION
        ));
    }

    // Validate each boxer layout
    for layout in &pack.layouts {
        let mut boxer_warnings = Vec::new();
        let mut boxer_errors = Vec::new();

        // Check if boxer exists in manifest
        let boxer = manifest
            .fighters
            .values()
            .find(|f| f.key == layout.boxer_key);

        if let Some(boxer) = boxer {
            // Validate bins
            let mut bins_valid = true;
            let mut size_matches = true;

            for bin in &layout.bins {
                // Find matching bin in manifest
                let manifest_bin = boxer
                    .unique_sprite_bins
                    .iter()
                    .chain(boxer.shared_sprite_bins.iter())
                    .find(|b| b.filename == bin.filename);

                if let Some(manifest_bin) = manifest_bin {
                    if manifest_bin.start_pc != bin.pc_offset {
                        boxer_warnings.push(format!(
                            "Bin {} has different offset: pack={}, manifest={}",
                            bin.filename, bin.pc_offset, manifest_bin.start_pc
                        ));
                    }
                    if manifest_bin.size != bin.size {
                        boxer_warnings.push(format!(
                            "Bin {} has different size: pack={}, manifest={}",
                            bin.filename, bin.size, manifest_bin.size
                        ));
                        size_matches = false;
                    }
                } else {
                    boxer_warnings.push(format!(
                        "Bin {} not found in manifest for {}",
                        bin.filename, layout.boxer_key
                    ));
                    bins_valid = false;
                }
            }

            boxer_validations.push(BoxerValidation {
                boxer_key: layout.boxer_key.clone(),
                exists_in_manifest: true,
                bins_valid,
                size_matches,
                warnings: boxer_warnings,
                errors: boxer_errors,
            });
        } else {
            boxer_errors.push(format!(
                "Boxer '{}' not found in manifest",
                layout.boxer_key
            ));
            boxer_validations.push(BoxerValidation {
                boxer_key: layout.boxer_key.clone(),
                exists_in_manifest: false,
                bins_valid: false,
                size_matches: false,
                warnings: boxer_warnings,
                errors: boxer_errors,
            });
        }
    }

    let valid = errors.is_empty() && boxer_validations.iter().all(|b| b.errors.is_empty());

    Ok(ValidationReport {
        valid,
        version_compatible,
        boxer_validations,
        warnings,
        errors,
    })
}

/// Get list of available community layout packs
#[tauri::command]
pub fn get_available_layout_packs() -> Vec<LayoutPackInfo> {
    let mut packs = Vec::new();

    let dir = PathBuf::from(COMMUNITY_LAYOUTS_DIR);
    if !dir.exists() {
        return packs;
    }

    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(pack) = serde_json::from_str::<LayoutPack>(&content) {
                        packs.push(LayoutPackInfo {
                            filename: path
                                .file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_default(),
                            name: pack.name,
                            author: pack.author,
                            description: pack.description,
                            created_at: pack.created_at,
                            boxer_count: pack.layouts.len(),
                        });
                    }
                }
            }
        }
    }

    // Sort by name
    packs.sort_by(|a, b| a.name.cmp(&b.name));
    packs
}

/// Delete a community layout pack
#[tauri::command]
pub fn delete_layout_pack(filename: String) -> Result<(), String> {
    let dir = ensure_community_dir()?;
    let path = dir.join(&filename);

    if !path.exists() {
        return Err(format!("Pack '{}' not found", filename));
    }

    std::fs::remove_file(&path).map_err(|e| format!("Failed to delete pack: {}", e))
}

/// Copy a layout pack to community directory
#[tauri::command]
pub fn install_layout_pack(source_path: String) -> Result<LayoutPackInfo, String> {
    ensure_community_dir()?;

    let content = std::fs::read_to_string(&source_path)
        .map_err(|e| format!("Failed to read source pack: {}", e))?;

    let pack: LayoutPack =
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse pack: {}", e))?;

    // Generate filename from pack name
    let filename = format!(
        "{}.json",
        pack.name
            .to_lowercase()
            .replace(" ", "_")
            .replace(|c: char| !c.is_alphanumeric() && c != '_', "")
    );

    let dest_path = PathBuf::from(COMMUNITY_LAYOUTS_DIR).join(&filename);

    std::fs::copy(&source_path, &dest_path)
        .map_err(|e| format!("Failed to install pack: {}", e))?;

    Ok(LayoutPackInfo {
        filename,
        name: pack.name,
        author: pack.author,
        description: pack.description,
        created_at: pack.created_at,
        boxer_count: pack.layouts.len(),
    })
}

/// Helper: Ensure community layouts directory exists
fn ensure_community_dir() -> Result<PathBuf, String> {
    let dir = PathBuf::from(COMMUNITY_LAYOUTS_DIR);
    if !dir.exists() {
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create community dir: {}", e))?;
    }
    Ok(dir)
}

/// Apply a layout pack to selected boxers
#[tauri::command]
pub fn apply_layout_pack(
    state: State<AppState>,
    pack_path: String,
    boxer_keys: Vec<String>,
) -> Result<(), String> {
    let content =
        std::fs::read_to_string(&pack_path).map_err(|e| format!("Failed to read pack: {}", e))?;

    let pack: LayoutPack =
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse pack: {}", e))?;

    let rom_opt = state.rom.lock();
    let rom = rom_opt.as_ref().ok_or("No ROM loaded")?;

    for layout in pack.layouts {
        if !boxer_keys.contains(&layout.boxer_key) {
            continue;
        }

        // Read and store each bin to pending writes
        for bin in layout.bins {
            let offset = parse_offset(&bin.pc_offset)?;
            let data = rom
                .read_bytes(offset, bin.size)
                .map_err(|e| format!("Failed to read bin {}: {}", bin.filename, e))?;

            state
                .pending_writes
                .lock()
                .insert(bin.pc_offset, data.to_vec());
        }
    }

    Ok(())
}
