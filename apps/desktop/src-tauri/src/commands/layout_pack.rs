//! Layout pack commands.
//!
//! Version 2 packs contain sparse, preimage-checked edits, never whole base ROMs.

use std::io::Read;
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
    let mut content = String::new();
    std::fs::File::open(path)
        .map_err(|e| format!("Failed to open pack: {e}"))?
        .take(MAX_LAYOUT_PACK_BYTES + 1)
        .read_to_string(&mut content)
        .map_err(|e| format!("Failed to read pack: {e}"))?;
    if content.len() as u64 > MAX_LAYOUT_PACK_BYTES {
        return Err("Layout pack exceeds size limit".into());
    }
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
    include_shared_for: Option<Vec<String>>,
) -> Result<(), String> {
    export_pack_internal(
        &state,
        boxer_keys,
        metadata,
        &output_path,
        &include_shared_for.unwrap_or_default(),
    )
}

fn export_pack_internal(
    state: &AppState,
    boxer_keys: Vec<String>,
    metadata: LayoutPackMetadata,
    output_path: &str,
    include_shared_for: &[String],
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
                edits: Vec::new(),
            });
        }
        for bin in boxer
            .shared_sprite_bins
            .iter()
            .filter(|_| include_shared_for.contains(&key))
        {
            bins.push(LayoutBin {
                filename: bin.filename.clone(),
                pc_offset: bin.start_pc.clone(),
                size: bin.size,
                category: bin.category.clone(),
                label: Some(format!("Shared with {}", bin.shared_with.join(", "))),
                edits: Vec::new(),
            });
        }
        layouts.push(PackBoxerLayout {
            boxer_key: key,
            version: "2.0".to_string(),
            layout_type: "custom".to_string(),
            bins,
            notes: None,
        });
    }

    drop(manifest);
    let guard = state.rom_session.lock();
    let session = guard.as_ref().ok_or("No ROM loaded")?;
    let current = session.materialize().map_err(|error| error.to_string())?;
    for layout in &mut layouts {
        for bin in &mut layout.bins {
            let offset = crate::utils::parse_offset(&bin.pc_offset)?;
            let range = rom_core::validate_range(offset, bin.size, session.base().len())
                .map_err(|error| error.to_string())?;
            let current_range = rom_core::validate_range(offset, bin.size, current.bytes.len())
                .map_err(|error| error.to_string())?;
            bin.edits = sparse_edits(
                &session.base().bytes()[range],
                &current.bytes[current_range],
            );
        }
    }
    let source_sha1 = Some(session.base().sha1().to_string());
    drop(guard);
    let pack = LayoutPack {
        version: LAYOUT_PACK_VERSION.to_string(),
        name: metadata.name,
        author: metadata.author,
        description: metadata.description,
        created_at: Utc::now().to_rfc3339(),
        layouts,
        source_sha1,
    };
    let json = serde_json::to_string_pretty(&pack)
        .map_err(|e| format!("Failed to serialize layout pack: {e}"))?;
    if json.len() as u64 > MAX_LAYOUT_PACK_BYTES {
        return Err("Exported layout pack exceeds size limit".into());
    }
    std::fs::write(output_path, json).map_err(|e| format!("Failed to write layout pack: {e}"))
}

fn sparse_edits(before: &[u8], after: &[u8]) -> Vec<LayoutEdit> {
    let mut edits = Vec::new();
    let mut index = 0;
    while index < before.len() {
        if before[index] == after[index] {
            index += 1;
            continue;
        }
        let start = index;
        while index < before.len() && before[index] != after[index] {
            index += 1;
        }
        edits.push(LayoutEdit {
            offset: start,
            expected: before[start..index].to_vec(),
            replacement: after[start..index].to_vec(),
        });
    }
    edits
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exported_file_imports_into_a_fresh_session_and_undoes() {
        let (source, _) = fixture();
        source
            .commit_rom_transform("Author bin edit", |rom| {
                rom.data[9] = 42;
                rom.data[12] = 7;
                rom.data[20] = 99; // Outside selected bins: must not be exported.
                Ok(())
            })
            .unwrap();
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("round-trip.json");
        export_pack_internal(
            &source,
            vec!["gabby_jay".into()],
            LayoutPackMetadata {
                name: "Round trip".into(),
                author: "Test".into(),
                description: String::new(),
            },
            path.to_str().unwrap(),
            &[],
        )
        .unwrap();
        let imported = read_pack(&path).unwrap();
        assert_eq!(imported.version, LAYOUT_PACK_VERSION);
        assert_eq!(imported.source_sha1, source.get_rom_sha1());
        let edits = &imported.layouts[0].bins[0].edits;
        assert_eq!(edits.len(), 2);
        assert_eq!(
            edits
                .iter()
                .map(|edit| edit.replacement.len())
                .sum::<usize>(),
            2
        );
        let (destination, _) = fixture();
        apply_pack_internal(&destination, &imported, &["gabby_jay".into()]).unwrap();
        let mut expected = vec![0; 32];
        expected[9] = 42;
        expected[12] = 7;
        assert_eq!(
            destination.materialize_current_rom().unwrap().bytes,
            expected
        );
        destination.undo_journal().unwrap();
        assert_eq!(
            destination.materialize_current_rom().unwrap().bytes,
            vec![0; 32]
        );
    }

    #[test]
    fn oversized_pack_file_is_rejected() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("large.json");
        std::fs::File::create(&path)
            .unwrap()
            .set_len(MAX_LAYOUT_PACK_BYTES + 1)
            .unwrap();
        assert!(read_pack(&path).unwrap_err().contains("limit"));
    }

    fn fixture() -> (AppState, LayoutPack) {
        let bin = serde_json::json!({"file":"tiles", "filename":"tiles", "category":"Raw", "subtype":"", "size":8,
            "start_snes":"", "end_snes":"", "start_pc":"0x8", "end_pc":"0x10", "shared_with":[]});
        let boxer = serde_json::from_value(
            serde_json::json!({"name":"Gabby Jay", "key":"gabby_jay", "reference_sheet":"",
            "palette_files":[], "icon_files":[], "portrait_files":[], "large_portrait_files":[],
            "unique_sprite_bins":[bin], "shared_sprite_bins":[], "other_files":[]}),
        )
        .unwrap();
        let mut manifest = manifest_core::Manifest::empty();
        manifest.fighters.insert("Gabby Jay".into(), boxer);
        let state = AppState::new(manifest);
        state.install_rom_session(rom_core::Rom::new(vec![0; 32]), "synthetic.sfc".into());
        let pack = serde_json::from_value(serde_json::json!({"version":"2.0", "name":"Test", "author":"", "description":"", "created_at":"",
            "source_sha1":state.get_rom_sha1(), "layouts":[{"boxer_key":"gabby_jay", "version":"2.0", "layout_type":"custom", "notes":null,
                "bins":[{"filename":"tiles", "pc_offset":"0x08", "size":8, "category":"Raw", "label":null,
                    "edits":[{"offset":1,"expected":[0],"replacement":[9]}]}]}]})).unwrap();
        (state, pack)
    }

    #[test]
    fn export_shared_graphics_requires_explicit_selection() {
        let (state, _) = fixture();
        {
            let mut manifest = state.manifest.lock();
            let boxer = manifest.fighters.get_mut("Gabby Jay").unwrap();
            let mut shared = boxer.unique_sprite_bins[0].clone();
            shared.filename = "shared_tiles".into();
            boxer.shared_sprite_bins.push(shared);
        }
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("selection.json");
        for include in [false, true] {
            let shared = if include {
                vec!["gabby_jay".into()]
            } else {
                vec![]
            };
            export_pack_internal(
                &state,
                vec!["gabby_jay".into()],
                LayoutPackMetadata {
                    name: "Selection".into(),
                    author: String::new(),
                    description: String::new(),
                },
                path.to_str().unwrap(),
                &shared,
            )
            .unwrap();
            let pack = read_pack(&path).unwrap();
            assert_eq!(pack.layouts[0].bins.len(), if include { 2 } else { 1 });
        }
    }

    #[test]
    fn validation_limits_conflicts_to_selected_boxers() {
        let (state, mut pack) = fixture();
        let mut unrelated = pack.layouts[0].clone();
        unrelated.boxer_key = "unknown_boxer".into();
        pack.layouts.push(unrelated);
        assert!(!validate_pack_internal(&state, &pack, None).unwrap().valid);
        let selected = validate_pack_internal(&state, &pack, Some(&["gabby_jay".into()])).unwrap();
        assert!(selected.valid, "{:?}", selected.errors);
        assert_eq!(selected.boxer_validations.len(), 1);
        assert!(
            !validate_pack_internal(&state, &pack, Some(&[]))
                .unwrap()
                .valid
        );
        assert!(
            !validate_pack_internal(&state, &pack, Some(&["missing".into()]))
                .unwrap()
                .valid
        );
        assert_eq!(state.materialize_current_rom().unwrap().bytes, vec![0; 32]);
    }

    #[test]
    fn preflight_is_read_only_and_matches_application_checks() {
        let (state, mut pack) = fixture();
        let keys = ["gabby_jay".into()];
        preflight_pack(&state, &pack, &keys).unwrap();
        assert_eq!(state.materialize_current_rom().unwrap().bytes, vec![0; 32]);
        pack.layouts[0].bins[0].edits[0].expected = vec![7];
        assert!(preflight_pack(&state, &pack, &keys)
            .unwrap_err()
            .contains("conflicts"));
        pack.layouts[0].bins[0].edits[0].expected = vec![0];
        apply_pack_internal(&state, &pack, &keys).unwrap();
        assert!(preflight_pack(&state, &pack, &keys)
            .unwrap_err()
            .contains("already applied"));
        state.undo_journal().unwrap();
        preflight_pack(&state, &pack, &keys).unwrap();
    }

    #[test]
    fn payload_application_is_undoable_and_preserves_other_bytes() {
        let (state, pack) = fixture();
        apply_pack_internal(&state, &pack, &["gabby_jay".into()]).unwrap();
        let mut expected = vec![0; 32];
        expected[9] = 9;
        assert_eq!(state.materialize_current_rom().unwrap().bytes, expected);
        state.undo_journal().unwrap();
        assert_eq!(state.materialize_current_rom().unwrap().bytes, vec![0; 32]);
        state.redo_journal().unwrap();
        assert_eq!(state.materialize_current_rom().unwrap().bytes, expected);
    }

    #[test]
    fn conflicts_and_wrong_identity_reject_the_whole_pack() {
        let (state, mut pack) = fixture();
        pack.layouts[0].bins[0].edits.push(LayoutEdit {
            offset: 2,
            expected: vec![7],
            replacement: vec![8],
        });
        assert!(apply_pack_internal(&state, &pack, &["gabby_jay".into()]).is_err());
        assert_eq!(state.materialize_current_rom().unwrap().bytes, vec![0; 32]);
        pack.layouts[0].bins[0].edits.pop();
        pack.source_sha1 = Some("wrong".into());
        assert!(apply_pack_internal(&state, &pack, &["gabby_jay".into()]).is_err());
        assert_eq!(state.materialize_current_rom().unwrap().bytes, vec![0; 32]);
    }

    #[test]
    fn sparse_export_and_overlap_checks() {
        let edits = sparse_edits(&[0; 5], &[0, 1, 2, 0, 3]);
        assert_eq!(edits.len(), 2);
        assert_eq!(edits[0].offset, 1);
        assert_eq!(edits[0].replacement, vec![1, 2]);
        let (state, mut pack) = fixture();
        pack.layouts[0].bins[0].edits.push(LayoutEdit {
            offset: 1,
            expected: vec![0],
            replacement: vec![10],
        });
        assert!(apply_pack_internal(&state, &pack, &["gabby_jay".into()])
            .unwrap_err()
            .contains("overlapping"));
    }
}

#[tauri::command]
pub fn import_layout_pack(pack_path: String) -> Result<LayoutPack, String> {
    read_pack(Path::new(&pack_path))
}

#[tauri::command]
pub fn validate_layout_pack(
    state: State<AppState>,
    pack_path: String,
    boxer_keys: Option<Vec<String>>,
) -> Result<ValidationReport, String> {
    let pack = read_pack(Path::new(&pack_path))?;
    validate_pack_internal(&state, &pack, boxer_keys.as_deref())
}

fn validate_pack_internal(
    state: &AppState,
    pack: &LayoutPack,
    boxer_keys: Option<&[String]>,
) -> Result<ValidationReport, String> {
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
    if pack.source_sha1.is_none() || pack.source_sha1 != state.get_rom_sha1() {
        errors.push("Pack base ROM identity does not match the loaded ROM".into());
    }

    for layout in &pack.layouts {
        if boxer_keys.is_some_and(|keys| !keys.contains(&layout.boxer_key)) {
            continue;
        }
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
            for edit in &bin.edits {
                if edit.expected.is_empty()
                    || edit.expected.len() != edit.replacement.len()
                    || rom_core::validate_range(edit.offset, edit.expected.len(), bin.size).is_err()
                {
                    boxer_errors.push(format!("Invalid edit payload in {}", bin.filename));
                    bins_valid = false;
                }
            }
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
                    if crate::utils::parse_offset(&expected.start_pc).ok()
                        != crate::utils::parse_offset(&bin.pc_offset).ok()
                    {
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

    drop(manifest);
    let keys: Vec<_> = boxer_keys.map(|keys| keys.to_vec()).unwrap_or_else(|| {
        pack.layouts
            .iter()
            .map(|layout| layout.boxer_key.clone())
            .collect()
    });
    if let Err(error) = preflight_pack(state, pack, &keys) {
        errors.push(error);
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
                path: path.to_string_lossy().to_string(),
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
    std::fs::copy(source, &destination)
        .map_err(|e| format!("Failed to install layout pack: {e}"))?;
    Ok(LayoutPackInfo {
        path: destination.to_string_lossy().to_string(),
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
    state: State<AppState>,
    pack_path: String,
    boxer_keys: Vec<String>,
) -> Result<(), String> {
    let pack = read_pack(Path::new(&pack_path))?;
    apply_pack_internal(&state, &pack, &boxer_keys)
}

fn apply_pack_internal(
    state: &AppState,
    pack: &LayoutPack,
    boxer_keys: &[String],
) -> Result<(), String> {
    let writes = plan_pack_writes(state, pack, boxer_keys)?;
    state.commit_rom_transform_for_base(
        "Apply layout pack",
        pack.source_sha1.as_deref(),
        |rom| {
            validate_pack_preimages(&rom.data, &writes)?;
            for (&offset, &(_, after)) in &writes {
                rom.data[offset] = after;
            }
            Ok(())
        },
    )?;
    Ok(())
}

type PackWrites = std::collections::BTreeMap<usize, (u8, u8)>;

fn preflight_pack(
    state: &AppState,
    pack: &LayoutPack,
    boxer_keys: &[String],
) -> Result<(), String> {
    let writes = plan_pack_writes(state, pack, boxer_keys)?;
    if pack.source_sha1 != state.get_rom_sha1() {
        return Err("Pack base ROM identity does not match the loaded ROM".into());
    }
    validate_pack_preimages(&state.materialize_current_rom()?.bytes, &writes)
}

fn validate_pack_preimages(bytes: &[u8], writes: &PackWrites) -> Result<(), String> {
    let mut changed = false;
    for (&offset, &(before, after)) in writes {
        let current = *bytes.get(offset).ok_or("Layout edit is outside ROM")?;
        if current != before && current != after {
            return Err(format!(
                "Layout edit conflicts with current byte at {offset:#x}"
            ));
        }
        changed |= current != after;
    }
    if !changed {
        return Err("Selected layout edits are already applied".into());
    }
    Ok(())
}

fn plan_pack_writes(
    state: &AppState,
    pack: &LayoutPack,
    boxer_keys: &[String],
) -> Result<PackWrites, String> {
    if pack.version != LAYOUT_PACK_VERSION {
        return Err("Only version 2 payload packs can be applied".into());
    }
    pack.source_sha1
        .as_deref()
        .ok_or("Pack has no base ROM identity")?;
    if boxer_keys.is_empty() {
        return Err("Select at least one boxer".into());
    }
    let manifest = state.manifest.lock();
    let mut writes = std::collections::BTreeMap::new();
    for key in boxer_keys {
        let layout = pack
            .layouts
            .iter()
            .find(|layout| &layout.boxer_key == key)
            .ok_or("Selected boxer is missing from pack")?;
        let boxer = manifest
            .fighters
            .values()
            .find(|boxer| &boxer.key == key)
            .ok_or("Selected boxer is missing from manifest")?;
        for bin in &layout.bins {
            let asset = boxer
                .unique_sprite_bins
                .iter()
                .chain(&boxer.shared_sprite_bins)
                .find(|asset| asset.filename == bin.filename)
                .ok_or("Pack bin is missing from manifest")?;
            let offset = crate::utils::parse_offset(&bin.pc_offset)?;
            if offset != crate::utils::parse_offset(&asset.start_pc)? || bin.size != asset.size {
                return Err("Pack bin range differs from manifest".into());
            }
            for edit in &bin.edits {
                if edit.expected.is_empty() || edit.expected.len() != edit.replacement.len() {
                    return Err("Invalid layout edit lengths".into());
                }
                rom_core::validate_range(edit.offset, edit.expected.len(), bin.size)
                    .map_err(|error| error.to_string())?;
                let start = offset
                    .checked_add(edit.offset)
                    .ok_or("Layout edit offset overflow")?;
                for (index, (&before, &after)) in
                    edit.expected.iter().zip(&edit.replacement).enumerate()
                {
                    let address = start
                        .checked_add(index)
                        .ok_or("Layout edit offset overflow")?;
                    if let Some(previous) = writes.insert(address, (before, after)) {
                        if previous != (before, after) {
                            return Err("Conflicting overlapping layout edits".into());
                        }
                    }
                }
            }
        }
    }
    drop(manifest);
    if writes.is_empty() {
        return Err("Selected layouts contain no edit payloads".into());
    }
    Ok(writes)
}
