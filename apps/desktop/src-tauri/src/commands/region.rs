//! Region Commands
//!
//! Commands for working with ROM regions (NTSC/PAL).

use rom_core::{RegionDetectionResult, Rom, RomRegionInfo};
use crate::utils::{load_manifest_for_region, validate_rom_path};

/// Result type for region operations
pub type RegionResult<T> = Result<T, String>;

/// Return all known regions for the region selector UI.
#[tauri::command]
pub fn get_supported_regions() -> RegionResult<Vec<RomRegionInfo>> {
    Ok(RomRegionInfo::all_regions())
}

/// Detect the ROM region for a selected ROM file.
#[tauri::command]
pub fn detect_rom_region(rom_path: String) -> RegionResult<RegionDetectionResult> {
    validate_rom_path(&rom_path)?;

    let rom = Rom::load(&rom_path).map_err(|e| e.to_string())?;
    Ok(RegionDetectionResult::from_rom(&rom))
}

/// Check whether the manifest for a detected region can be loaded.
#[tauri::command]
pub fn validate_region_manifest(region: String) -> RegionResult<bool> {
    let normalized = region.to_lowercase();
    let rom_region = match normalized.as_str() {
        "usa" => rom_core::RomRegion::Usa,
        "jpn" | "japan" => rom_core::RomRegion::Jpn,
        "pal" | "europe" => rom_core::RomRegion::Pal,
        _ => return Err(format!("Unknown region '{}'", region)),
    };

    load_manifest_for_region(rom_region, None).map(|_| true)
}
