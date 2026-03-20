//! Boxer/Fighter Commands
//!
//! Commands for querying boxer information from the manifest.

use tauri::State;

use crate::app_state::AppState;
use asset_core::fighter::BoxerManager;
use manifest_core::{
    comparison::{find_similar_boxers, BoxerComparison, BoxerSimilarity, FighterStats, StatField},
    BoxerRecord,
};
use script_core::ScriptReader;

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
