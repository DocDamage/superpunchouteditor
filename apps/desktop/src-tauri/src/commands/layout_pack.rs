//! Layout pack commands.
//!
//! Layout packs remain experimental. Import/list/install/delete are hardened and deterministic;
//! apply is intentionally rejected because the current pack schema describes layout metadata but
//! does not contain replacement bytes. Returning success would be a false no-op.

use std::path::{Component, Path, PathBuf};

use chrono::Utc;
use tauri::{AppHandle, Manager, State};

use crate::app_state::AppState;
use crate::types::*;

const MAX_LAYOUT_PACK_BYTES: u64 = 2 * 1024 * 1024;

fn read_pack(path: &Path) -> Result<LayoutPack, String> {
    let metadata = std::fs::metadata(path).map_err(|e| format!("Failed to inspect pack: {e}"))?;
    if metadata.len() > MAX_LAYOUT_PACK_BYTES {
        return Err(format!(
            "Layout pack exceeds {} byte safety limit",
            MAX_LAYOUT_PACK_BYTES
        ));
    }
    let content = std::fs::read_to_string(path).map_err(|e| format!("Failed to read pack: {e}"))?;
    serde_json::from_str(&content).map_err(|e| format!("Failed to parse pack: {e}"))
}

fn safe_logical_filename(value: &str) -> bool {
    let path = Path::new(value);
    !value.is_empty()
        && !path.is_absolute()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}

fn community_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("layout-packs");
    std::fs::create_dir_all(&dir).map_err(|e| format!("Failed to create layout-pack dir: {e}"))?;
    Ok(dir)
}

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
            .find(|fighter| fighter.key == key)
            .ok_or_else(|| format!("Boxer '{key}' not found"))?;
        let mut bins = Vec::new();
        for bin in &boxer.unique_sprite_bins {
            bins.push(LayoutBin {
                filename: bin.filename.clone(),
                pc_offset: bin.start_pc.clone(),
                size: bin.size,
                category: bin.category.clone(),
                label: None,
            });
        }
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
            boxer_key: key,
            version: "1.0".to_string(),
            layout_type: "reference".to_string(),
            bins,
            notes: None,
        });
    }

    let pack = LayoutPack {
        version: LAYOUT_PACK_VERSION.to_string(),
        name: metadata.name,
        author: metadata.author,
        description: metadata.description,
        created_at: Utc::now().to_rfc3339(),
        layouts,
    };
    let json = serde_json::to_string_pretty(&pack)
        .map_err(|e| format!("Failed to serialize layout pack: {e}"))?;
    std::fs::write(&output_path, json).map_err(|e| format!("Failed to write layout pack: {e}"))
}

#[tauri::command]
pub fn import_layout_pack(pack_path: String) -> Result<LayoutPack, String> {
    read_pack(Path::new(&pack_path))
}

#[tauri::command]
pub fn validate_layout_pack(
    state: State<AppState>,
    pack_path: String,
) -> Result<ValidationReport, String> {
    let pack = read_pack(Path::new(&pack_path))?;
    let manifest = state.manifest.lock();
    let mut boxer_validations = Vec::new();
    let warnings = Vec::new();
    let mut errors = Vec::new();

    let version_compatible = pack.version == LAYOUT_PACK_VERSION;
    if !version_compatible {
        errors.push(format!(
            "Unsupported layout-pack schema {}; expected {}",
            pack.version, LAYOUT_PACK_VERSION
        ));
    }

    for layout in &pack.layouts {
        let mut boxer_errors = Vec::new();
        let boxer_warnings = Vec::new();
        let boxer = manifest
            .fighters
            .values()
            .find(|fighter| fighter.key == layout.boxer_key);
        let Some(boxer) = boxer else {
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
            continue;
        };

        let mut bins_valid = true;
        let mut size_matches = true;
        for bin in &layout.bins {
            if !safe_logical_filename(&bin.filename) {
                boxer_errors.push(format!("Unsafe bin filename: {}", bin.filename));
                bins_valid = false;
                continue;
            }
            let manifest_bin = boxer
                .unique_sprite_bins
                .iter()
                .chain(boxer.shared_sprite_bins.iter())
                .find(|candidate| candidate.filename == bin.filename);
            match manifest_bin {
                Some(expected) => {
                    if expected.start_pc != bin.pc_offset {
                        boxer_errors.push(format!(
                            "Unsafe offset mismatch for {}: pack={}, manifest={}",
                            bin.filename, bin.pc_offset, expected.start_pc
                        ));
                        bins_valid = false;
                    }
                    if expected.size != bin.size {
                        boxer_errors.push(format!(
                            "Unsafe size mismatch for {}: pack={}, manifest={}",
                            bin.filename, bin.size, expected.size
                        ));
                        size_matches = false;
                    }
                }
                None => {
                    boxer_errors.push(format!(
                        "Bin {} not found in manifest for {}",
                        bin.filename, layout.boxer_key
                    ));
                    bins_valid = false;
                }
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
    }

    let valid = errors.is_empty()
        && boxer_validations
            .iter()
            .all(|validation| validation.errors.is_empty());
    Ok(ValidationReport {
        valid,
        version_compatible,
        boxer_validations,
        warnings,
        errors,
    })
}

#[tauri::command]
pub fn get_available_layout_packs(app: AppHandle) -> Result<Vec<LayoutPackInfo>, String> {
    let dir = community_dir(&app)?;
    let mut packs = Vec::new();
    for entry in std::fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        if let Ok(pack) = read_pack(&path) {
            packs.push(LayoutPackInfo {
                filename: path
                    .file_name()
                    .map(|name| name.to_string_lossy().to_string())
                    .unwrap_or_default(),
                name: pack.name,
                author: pack.author,
                description: pack.description,
                created_at: pack.created_at,
                boxer_count: pack.layouts.len(),
            });
        }
    }
    packs.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(packs)
}

#[tauri::command]
pub fn delete_layout_pack(app: AppHandle, filename: String) -> Result<(), String> {
    if !safe_logical_filename(&filename) || !filename.ends_with(".json") {
        return Err("Invalid layout-pack filename".to_string());
    }
    let path = community_dir(&app)?.join(&filename);
    if !path.exists() {
        return Err(format!("Pack '{filename}' not found"));
    }
    std::fs::remove_file(path).map_err(|e| format!("Failed to delete layout pack: {e}"))
}

#[tauri::command]
pub fn install_layout_pack(app: AppHandle, source_path: String) -> Result<LayoutPackInfo, String> {
    let source = Path::new(&source_path);
    let pack = read_pack(source)?;
    if pack.version != LAYOUT_PACK_VERSION {
        return Err(format!(
            "Cannot install schema {}; expected {}",
            pack.version, LAYOUT_PACK_VERSION
        ));
    }
    let stem: String = pack
        .name
        .to_lowercase()
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character
            } else {
                '_'
            }
        })
        .collect();
    let stem = stem.trim_matches('_');
    if stem.is_empty() {
        return Err("Layout-pack name cannot produce a safe filename".to_string());
    }
    let filename = format!("{stem}.json");
    let destination = community_dir(&app)?.join(&filename);
    std::fs::copy(source, destination)
        .map_err(|e| format!("Failed to install layout pack: {e}"))?;
    Ok(LayoutPackInfo {
        filename,
        name: pack.name,
        author: pack.author,
        description: pack.description,
        created_at: pack.created_at,
        boxer_count: pack.layouts.len(),
    })
}

#[tauri::command]
pub fn apply_layout_pack(
    _state: State<AppState>,
    _pack_path: String,
    _boxer_keys: Vec<String>,
) -> Result<(), String> {
    Err(
        "Layout-pack application is experimental: the current schema contains reference layout metadata but no replacement byte payloads, so applying it would be a no-op"
            .to_string(),
    )
}
