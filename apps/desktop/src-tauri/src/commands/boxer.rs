//! Boxer/Fighter Commands
//!
//! Commands for querying boxer information from the manifest.

use tauri::State;

use crate::app_state::AppState;
use crate::utils::parse_offset;
use asset_core::fighter::{BoxerManager, BoxerMetadata, PoseInfo};
use manifest_core::{
    comparison::{find_similar_boxers, BoxerComparison, BoxerSimilarity, FighterStats, StatField},
    AssetFile, BoxerRecord, Manifest,
};
use rom_core::{MaterializedRom, Rom};
use script_core::ScriptReader;

fn format_manifest_snes_address(rom: &Rom, pc_offset: usize) -> String {
    let (bank, addr) = rom.pc_to_snes(pc_offset);
    format!("${:02X}{:04X}", bank, addr)
}

fn asset_alignment(asset: &AssetFile) -> usize {
    if asset.subtype == "palette" {
        2
    } else {
        32
    }
}

fn slugify_boxer_key(value: &str) -> String {
    let mut slug = String::with_capacity(value.len());
    let mut last_was_separator = false;

    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            last_was_separator = false;
        } else if !last_was_separator {
            slug.push('_');
            last_was_separator = true;
        }
    }

    slug.trim_matches('_').to_string()
}

fn make_unique_asset_owner_name(manifest: &Manifest, requested_name: &str) -> String {
    let trimmed = requested_name.trim();
    let base = format!("{} (Assets)", trimmed);

    if !manifest.fighters.contains_key(&base) {
        return base;
    }

    for suffix in 2..1000 {
        let candidate = format!("{} (Assets {})", trimmed, suffix);
        if !manifest.fighters.contains_key(&candidate) {
            return candidate;
        }
    }

    format!("{} (Assets Copy)", trimmed)
}

fn make_unique_asset_owner_key(
    manifest: &Manifest,
    owner_name: &str,
    preferred_key: Option<&str>,
) -> String {
    let preferred = preferred_key.unwrap_or_default().trim();
    let base_seed = if preferred.is_empty() {
        format!("creator_asset_{}", slugify_boxer_key(owner_name))
    } else {
        slugify_boxer_key(preferred)
    };
    let base = if base_seed.is_empty() {
        "creator_asset_owner".to_string()
    } else {
        base_seed
    };

    let key_in_use = |candidate: &str| {
        manifest
            .fighters
            .values()
            .any(|boxer| boxer.key == candidate)
    };

    if !key_in_use(&base) {
        return base;
    }

    for suffix in 2..1000 {
        let candidate = format!("{}_{}", base, suffix);
        if !key_in_use(&candidate) {
            return candidate;
        }
    }

    format!("{}_copy", base)
}

fn read_materialized_asset_payloads(
    materialized: &MaterializedRom,
    source_assets: &[AssetFile],
) -> Result<Vec<Vec<u8>>, String> {
    source_assets
        .iter()
        .map(|asset| {
            let source_offset = parse_offset(&asset.start_pc)?;
            let range =
                rom_core::validate_range(source_offset, asset.size, materialized.bytes.len())
                    .map_err(|error| error.to_string())?;
            Ok(materialized.bytes[range].to_vec())
        })
        .collect()
}

fn clone_asset_list_into_rom(
    rom: &mut Rom,
    owner_key: &str,
    source_assets: &[AssetFile],
    payloads: &[Vec<u8>],
) -> Result<Vec<AssetFile>, String> {
    if source_assets.len() != payloads.len() {
        return Err("Asset metadata and payload counts do not match".to_string());
    }

    payloads
        .iter()
        .enumerate()
        .map(|(index, bytes)| {
            let asset = &source_assets[index];
            let allocation = rom
                .find_or_expand_free_space(bytes.len(), asset_alignment(asset))
                .ok_or_else(|| {
                    format!(
                        "Failed to allocate {} bytes for cloned {} asset '{}'",
                        bytes.len(),
                        asset.subtype,
                        asset.filename
                    )
                })?;

            rom.write_bytes(allocation.offset, bytes)
                .map_err(|e| e.to_string())?;

            let end_pc = allocation.offset + bytes.len();
            let generated_name = if asset.subtype == "sprite_bin" {
                let source_name = asset
                    .filename
                    .rsplit(['/', '\\'])
                    .next()
                    .unwrap_or(&asset.filename);
                // Keep the original IndexXX token so pose/tile-set lookup can
                // connect the cloned bin to the game's tile-set id.
                format!("{}_{}", owner_key, source_name)
            } else {
                format!("{}_{}_{}", owner_key, asset.subtype, index + 1)
            };
            let generated_filename = if asset.subtype == "sprite_bin" {
                generated_name.clone()
            } else {
                format!("{}.bin", generated_name)
            };
            Ok(AssetFile {
                file: format!("Generated/{}", generated_filename),
                filename: generated_filename,
                category: asset.category.clone(),
                subtype: asset.subtype.clone(),
                size: bytes.len(),
                start_snes: format_manifest_snes_address(rom, allocation.offset),
                end_snes: format_manifest_snes_address(rom, end_pc),
                start_pc: format_hex(allocation.offset),
                end_pc: format_hex(end_pc),
                shared_with: Vec::new(),
            })
        })
        .collect()
}

fn create_boxer_asset_owner_inner(
    state: &AppState,
    template_boxer_key: &str,
    owner_display_name: &str,
    preferred_key: Option<&str>,
) -> Result<BoxerRecord, String> {
    let trimmed_name = owner_display_name.trim();
    if trimmed_name.is_empty() {
        return Err("Asset owner display name cannot be empty.".to_string());
    }

    let (template_boxer, owner_name, owner_key) = {
        let manifest = state.manifest.lock();
        let template_boxer = manifest
            .fighters
            .values()
            .find(|boxer| boxer.key == template_boxer_key)
            .cloned()
            .ok_or_else(|| {
                format!(
                    "Template boxer '{}' not found in manifest.",
                    template_boxer_key
                )
            })?;
        let owner_name = make_unique_asset_owner_name(&manifest, trimmed_name);
        let owner_key = make_unique_asset_owner_key(&manifest, &owner_name, preferred_key);
        (template_boxer, owner_name, owner_key)
    };

    let materialized = state.materialize_current_rom()?;
    let palette_payloads =
        read_materialized_asset_payloads(&materialized, &template_boxer.palette_files)?;
    let icon_payloads =
        read_materialized_asset_payloads(&materialized, &template_boxer.icon_files)?;
    let portrait_payloads =
        read_materialized_asset_payloads(&materialized, &template_boxer.portrait_files)?;
    let large_portrait_payloads =
        read_materialized_asset_payloads(&materialized, &template_boxer.large_portrait_files)?;
    let source_sprite_bins = template_boxer
        .unique_sprite_bins
        .iter()
        .chain(template_boxer.shared_sprite_bins.iter())
        .cloned()
        .collect::<Vec<_>>();
    let sprite_payloads = read_materialized_asset_payloads(&materialized, &source_sprite_bins)?;

    let ((palette_files, icon_files, portrait_files, large_portrait_files, unique_sprite_bins), _) =
        state.commit_rom_transform("Create dedicated boxer asset owner", |rom| {
            let palette_files = clone_asset_list_into_rom(
                rom,
                &owner_key,
                &template_boxer.palette_files,
                &palette_payloads,
            )?;
            let icon_files = clone_asset_list_into_rom(
                rom,
                &owner_key,
                &template_boxer.icon_files,
                &icon_payloads,
            )?;
            let portrait_files = clone_asset_list_into_rom(
                rom,
                &owner_key,
                &template_boxer.portrait_files,
                &portrait_payloads,
            )?;
            let large_portrait_files = clone_asset_list_into_rom(
                rom,
                &owner_key,
                &template_boxer.large_portrait_files,
                &large_portrait_payloads,
            )?;
            let unique_sprite_bins =
                clone_asset_list_into_rom(rom, &owner_key, &source_sprite_bins, &sprite_payloads)?;
            Ok((
                palette_files,
                icon_files,
                portrait_files,
                large_portrait_files,
                unique_sprite_bins,
            ))
        })?;

    let created = BoxerRecord {
        name: owner_name.clone(),
        key: owner_key.clone(),
        reference_sheet: template_boxer.reference_sheet,
        palette_files,
        icon_files,
        portrait_files,
        large_portrait_files,
        unique_sprite_bins,
        shared_sprite_bins: Vec::new(),
        other_files: Vec::new(),
    };

    let mut manifest = state.manifest.lock();
    for asset in created
        .palette_files
        .iter()
        .chain(created.icon_files.iter())
        .chain(created.portrait_files.iter())
        .chain(created.large_portrait_files.iter())
        .chain(created.unique_sprite_bins.iter())
    {
        *manifest
            .asset_counts
            .entry(asset.category.clone())
            .or_insert(0) += 1;
    }
    manifest.fighters.insert(owner_name, created.clone());

    Ok(created)
}

/// Get all boxers from the manifest
///
/// Returns a list of all boxer records defined in the manifest.
#[tauri::command]
pub fn get_boxers(state: State<AppState>) -> Vec<BoxerRecord> {
    state.manifest.lock().fighters.values().cloned().collect()
}

/// Get a specific boxer by key
///
/// Returns the boxer record matching the given key, or None if not found.
#[tauri::command]
pub fn get_boxer(state: State<AppState>, key: String) -> Option<BoxerRecord> {
    state
        .manifest
        .lock()
        .fighters
        .values()
        .find(|f| f.key == key)
        .cloned()
}

/// Create a dedicated manifest boxer that owns cloned graphic assets for a new roster slot.
///
/// The new owner is inserted into the in-memory manifest and its graphic assets
/// are copied into fresh ROM space so later PNG imports no longer mutate a donor boxer.
#[tauri::command]
pub fn create_boxer_asset_owner(
    state: State<AppState>,
    template_boxer_key: String,
    owner_display_name: String,
    preferred_key: Option<String>,
) -> Result<BoxerRecord, String> {
    create_boxer_asset_owner_inner(
        state.inner(),
        &template_boxer_key,
        &owner_display_name,
        preferred_key.as_deref(),
    )
}

/// Get the list of fighters/boxers available in the loaded ROM.
///
/// Compatibility wrapper kept for existing frontend components that still use
/// the legacy fighter-viewer command names.
#[tauri::command]
pub fn get_fighter_list(state: State<AppState>) -> Result<Vec<BoxerMetadata>, String> {
    let rom_guard = state.rom.lock();
    let rom = rom_guard.as_ref().ok_or("No ROM loaded")?;
    Ok(BoxerManager::new(rom).get_boxer_list())
}

/// Get all poses for a fighter by ROM roster index.
#[tauri::command]
pub fn get_fighter_poses(
    state: State<AppState>,
    fighter_id: usize,
) -> Result<Vec<PoseInfo>, String> {
    let rom_guard = state.rom.lock();
    let rom = rom_guard.as_ref().ok_or("No ROM loaded")?;
    Ok(BoxerManager::new(rom).get_poses(fighter_id))
}

/// Render a fighter pose to PNG bytes.
#[tauri::command]
pub fn render_fighter_pose(
    state: State<AppState>,
    fighter_id: usize,
    pose_id: usize,
) -> Result<Vec<u8>, String> {
    let rom_guard = state.rom.lock();
    let rom = rom_guard.as_ref().ok_or("No ROM loaded")?;

    let manifest = state.manifest.lock();
    let fighters = BoxerManager::new(rom).get_boxer_list();
    let fighter = fighters
        .get(fighter_id)
        .ok_or_else(|| format!("Invalid fighter id {}", fighter_id))?;
    let boxer = manifest
        .fighters
        .get(&fighter.name)
        .ok_or_else(|| format!("Boxer '{}' not found in manifest", fighter.name))?;

    BoxerManager::new(rom).render_pose(fighter_id, pose_id, boxer)
}

/// Get the curated layout for a boxer
///
/// Returns the layout JSON from the data/boxer-layouts/ directory.
#[tauri::command]
pub fn get_boxer_layout(boxer_key: String) -> Option<serde_json::Value> {
    let filename = boxer_key.to_lowercase().replace(' ', "_").replace('.', "");
    let path = format!("../../data/boxer-layouts/{}.json", filename);
    let content = std::fs::read_to_string(&path).ok()?;
    serde_json::from_str(&content).ok()
}

/// Get all layout metadata
///
/// Returns the shared bank metadata JSON from data/boxer-layouts/shared_banks.json.
#[tauri::command]
pub fn get_all_layouts() -> serde_json::Value {
    let content = std::fs::read_to_string("../../data/boxer-layouts/shared_banks.json")
        .unwrap_or_else(|_| "{}".into());
    serde_json::from_str(&content).unwrap_or(serde_json::Value::Null)
}

/// Compare two boxers and return a comprehensive comparison
///
/// Analyzes both manifest data and ROM stats to provide detailed comparison.
#[tauri::command]
pub fn compare_boxers(
    state: State<AppState>,
    boxer_a_key: String,
    boxer_b_key: String,
) -> Result<BoxerComparison, String> {
    let manifest = state.manifest.lock();

    let boxer_a = manifest
        .fighters
        .values()
        .find(|f| f.key == boxer_a_key)
        .ok_or_else(|| format!("Boxer '{}' not found", boxer_a_key))?;

    let boxer_b = manifest
        .fighters
        .values()
        .find(|f| f.key == boxer_b_key)
        .ok_or_else(|| format!("Boxer '{}' not found", boxer_b_key))?;

    // Get fighter stats if ROM is loaded
    let rom_opt = state.rom.lock();
    let stats_a = if let Some(rom) = rom_opt.as_ref() {
        let reader = ScriptReader::new(rom);
        let fighters = BoxerManager::new(rom).get_boxer_list();
        fighters
            .iter()
            .position(|f| f.name == boxer_a.name)
            .map(|idx| {
                let params = reader.get_editable_params(idx);
                FighterStats::new(
                    params.attack_power,
                    params.defense_rating,
                    params.speed_rating,
                    params.palette_id,
                )
            })
    } else {
        None
    };

    let stats_b = if let Some(rom) = rom_opt.as_ref() {
        let reader = ScriptReader::new(rom);
        let fighters = BoxerManager::new(rom).get_boxer_list();
        fighters
            .iter()
            .position(|f| f.name == boxer_b.name)
            .map(|idx| {
                let params = reader.get_editable_params(idx);
                FighterStats::new(
                    params.attack_power,
                    params.defense_rating,
                    params.speed_rating,
                    params.palette_id,
                )
            })
    } else {
        None
    };

    Ok(BoxerComparison::compare(
        boxer_a,
        boxer_b,
        stats_a.as_ref(),
        stats_b.as_ref(),
    ))
}

/// Find boxers similar to a reference boxer
///
/// Uses manifest data and ROM stats to find boxers with similar characteristics.
#[tauri::command]
pub fn get_similar_boxers(
    state: State<AppState>,
    reference_key: String,
    limit: Option<usize>,
) -> Result<Vec<BoxerSimilarity>, String> {
    let manifest = state.manifest.lock();

    let reference = manifest
        .fighters
        .values()
        .find(|f| f.key == reference_key)
        .ok_or_else(|| format!("Reference boxer '{}' not found", reference_key))?;

    let all_boxers: Vec<_> = manifest.fighters.values().cloned().collect();

    // Get stats for all boxers if ROM is loaded
    let rom_opt = state.rom.lock();
    let all_stats: Vec<(String, FighterStats)> = if let Some(rom) = rom_opt.as_ref() {
        let reader = ScriptReader::new(rom);
        let fighters = BoxerManager::new(rom).get_boxer_list();

        fighters
            .iter()
            .enumerate()
            .filter_map(|(idx, f_meta)| {
                manifest.fighters.get(&f_meta.name).map(|boxer| {
                    let params = reader.get_editable_params(idx);
                    (
                        boxer.key.clone(),
                        FighterStats::new(
                            params.attack_power,
                            params.defense_rating,
                            params.speed_rating,
                            params.palette_id,
                        ),
                    )
                })
            })
            .collect()
    } else {
        Vec::new()
    };

    let reference_stats = all_stats
        .iter()
        .find(|(key, _)| key == &reference_key)
        .map(|(_, stats)| stats.clone());

    let limit = limit.unwrap_or(5);
    Ok(find_similar_boxers(
        reference,
        &all_boxers,
        reference_stats.as_ref(),
        &all_stats,
        limit,
    ))
}

/// Copy a stat from one boxer to another
///
/// Modifies the target boxer's fighter header with the source boxer's stat value.
#[tauri::command]
pub fn copy_boxer_stat(
    state: State<AppState>,
    source_key: String,
    target_key: String,
    stat_field: String,
) -> Result<(), String> {
    let field = StatField::from_str(&stat_field)
        .ok_or_else(|| format!("Invalid stat field: {}", stat_field))?;

    let rom_opt = state.rom.lock();
    let rom = rom_opt.as_ref().ok_or("No ROM loaded")?;

    let fighters = BoxerManager::new(rom).get_boxer_list();
    let reader = ScriptReader::new(rom);
    let manifest = state.manifest.lock();

    // Find source fighter index
    let source_idx = fighters
        .iter()
        .position(|f| {
            manifest
                .fighters
                .get(&f.name)
                .map(|b| b.key == source_key)
                .unwrap_or(false)
        })
        .ok_or_else(|| format!("Source boxer '{}' not found", source_key))?;

    // Find target fighter index
    let target_idx = fighters
        .iter()
        .position(|f| {
            manifest
                .fighters
                .get(&f.name)
                .map(|b| b.key == target_key)
                .unwrap_or(false)
        })
        .ok_or_else(|| format!("Target boxer '{}' not found", target_key))?;

    drop(manifest);

    // Get source params
    let source_params = reader.get_editable_params(source_idx);

    // Get target params
    let mut target_params = reader.get_editable_params(target_idx);

    // Copy the specific field
    match field {
        StatField::Attack => target_params.attack_power = source_params.attack_power,
        StatField::Defense => target_params.defense_rating = source_params.defense_rating,
        StatField::Speed => target_params.speed_rating = source_params.speed_rating,
        StatField::PaletteId => target_params.palette_id = source_params.palette_id,
    }

    // Generate new header bytes
    let (header_bytes, pc_offset) = reader
        .generate_header_with_params(target_idx, &target_params)
        .map_err(|e| e.to_string())?;

    drop(rom_opt);

    // Store in pending writes
    let pc_offset_str = format_hex(pc_offset);
    state
        .pending_writes
        .lock()
        .insert(pc_offset_str, header_bytes);

    Ok(())
}

/// Copy all stats from one boxer to another
///
/// Modifies the target boxer's fighter header with all stat values from the source boxer.
#[tauri::command]
pub fn copy_all_boxer_stats(
    state: State<AppState>,
    source_key: String,
    target_key: String,
) -> Result<(), String> {
    let rom_opt = state.rom.lock();
    let rom = rom_opt.as_ref().ok_or("No ROM loaded")?;

    let fighters = BoxerManager::new(rom).get_boxer_list();
    let reader = ScriptReader::new(rom);
    let manifest = state.manifest.lock();

    // Find source fighter index
    let source_idx = fighters
        .iter()
        .position(|f| {
            manifest
                .fighters
                .get(&f.name)
                .map(|b| b.key == source_key)
                .unwrap_or(false)
        })
        .ok_or_else(|| format!("Source boxer '{}' not found", source_key))?;

    // Find target fighter index
    let target_idx = fighters
        .iter()
        .position(|f| {
            manifest
                .fighters
                .get(&f.name)
                .map(|b| b.key == target_key)
                .unwrap_or(false)
        })
        .ok_or_else(|| format!("Target boxer '{}' not found", target_key))?;

    drop(manifest);

    // Copy all params from source to target
    let source_params = reader.get_editable_params(source_idx);

    // Generate new header bytes
    let (header_bytes, pc_offset) = reader
        .generate_header_with_params(target_idx, &source_params)
        .map_err(|e| e.to_string())?;

    drop(rom_opt);

    // Store in pending writes
    let pc_offset_str = format_hex(pc_offset);
    state
        .pending_writes
        .lock()
        .insert(pc_offset_str, header_bytes);

    Ok(())
}

// Helper function for formatting hex
fn format_hex(value: usize) -> String {
    format!("0x{:X}", value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rom_core::{Rom, EXPECTED_SIZE};

    fn test_asset(
        start_pc: usize,
        size: usize,
        category: &str,
        subtype: &str,
        filename: &str,
    ) -> AssetFile {
        AssetFile {
            file: filename.to_string(),
            filename: filename.to_string(),
            category: category.to_string(),
            subtype: subtype.to_string(),
            size,
            start_snes: "$808000".to_string(),
            end_snes: "$808000".to_string(),
            start_pc: format_hex(start_pc),
            end_pc: format_hex(start_pc + size),
            shared_with: Vec::new(),
        }
    }

    #[test]
    fn create_boxer_asset_owner_clones_graphic_assets_into_new_rom_space() {
        let mut manifest = Manifest::empty();
        manifest.fighters.insert(
            "Template Boxer".to_string(),
            BoxerRecord {
                name: "Template Boxer".to_string(),
                key: "template_boxer".to_string(),
                reference_sheet: "sprites/Template Boxer.png".to_string(),
                palette_files: vec![test_asset(0x1000, 4, "Palettes", "palette", "Palette.bin")],
                icon_files: vec![test_asset(0x2000, 32, "Graphics", "icon", "Icon.bin")],
                portrait_files: vec![test_asset(
                    0x3000,
                    64,
                    "Graphics",
                    "portrait",
                    "Portrait.bin",
                )],
                large_portrait_files: vec![test_asset(
                    0x4000,
                    96,
                    "Graphics/Compressed",
                    "large_portrait",
                    "LargePortrait.bin",
                )],
                unique_sprite_bins: vec![test_asset(
                    0x5000,
                    128,
                    "Graphics",
                    "sprite_bin",
                    "GFX_Sprite_TemplateIndex04.bin",
                )],
                shared_sprite_bins: vec![test_asset(
                    0x6000,
                    160,
                    "Graphics",
                    "sprite_bin",
                    "GFX_Sprite_SharedIndex05.bin",
                )],
                other_files: Vec::new(),
            },
        );

        let state = AppState::new(manifest);
        let mut rom = Rom::new(vec![0; EXPECTED_SIZE]);
        rom.write_bytes(0x1000, &[0x10, 0x20, 0x30, 0x40]).unwrap();
        rom.write_bytes(0x2000, &[0xAA; 32]).unwrap();
        rom.write_bytes(0x3000, &[0xBB; 64]).unwrap();
        rom.write_bytes(0x4000, &[0xCC; 96]).unwrap();
        rom.write_bytes(0x5000, &[0xDD; 128]).unwrap();
        rom.write_bytes(0x6000, &[0xEE; 160]).unwrap();
        state.install_rom_session(rom, "synthetic.sfc".to_string());

        let created = create_boxer_asset_owner_inner(
            &state,
            "template_boxer",
            "New Challenger",
            Some("creator_asset_slot_24"),
        )
        .unwrap();

        assert_eq!(created.name, "New Challenger (Assets)");
        assert_eq!(created.key, "creator_asset_slot_24");
        assert_eq!(created.palette_files.len(), 1);
        assert_eq!(created.icon_files.len(), 1);
        assert_eq!(created.portrait_files.len(), 1);
        assert_eq!(created.large_portrait_files.len(), 1);
        assert_eq!(created.unique_sprite_bins.len(), 2);
        assert!(created.shared_sprite_bins.is_empty());
        assert!(created.unique_sprite_bins[0].filename.contains("Index04"));
        assert_ne!(created.icon_files[0].start_pc, "0x2000");

        let projection = state
            .rom_session
            .lock()
            .as_ref()
            .expect("canonical ROM session should remain loaded")
            .state_projection()
            .unwrap();
        assert_eq!(projection.active_transaction_count, 1);
        assert!(projection.dirty);
        assert!(state.pending_writes.lock().is_empty());

        let materialized = state.materialize_current_rom().unwrap();
        for asset in created
            .palette_files
            .iter()
            .chain(created.icon_files.iter())
            .chain(created.portrait_files.iter())
            .chain(created.large_portrait_files.iter())
            .chain(created.unique_sprite_bins.iter())
        {
            let start = parse_offset(&asset.start_pc).unwrap();
            let end = start + asset.size;
            assert!(materialized.bytes[start..end].iter().any(|byte| *byte != 0));
        }

        let manifest = state.manifest.lock();
        assert!(manifest.fighters.contains_key("New Challenger (Assets)"));
        assert!(
            manifest
                .asset_counts
                .get("Graphics")
                .copied()
                .unwrap_or_default()
                >= 2
        );
        assert!(manifest
            .fighters
            .values()
            .any(|boxer| boxer.key == "creator_asset_slot_24"));
    }
}
