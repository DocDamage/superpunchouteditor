//! Relocation Commands
//!
//! Commands for asset relocation and free space management.

use tauri::State;

use crate::app_state::AppState;
use crate::types::*;
use crate::utils::parse_offset;
use relocation_core::{
    free_space::{find_free_regions, FreeSpaceStats},
    relocation::validate_relocation,
    RelocationSafetyReport, RiskLevel,
};

/// Get all free space regions in the ROM
#[tauri::command]
pub fn get_free_space_regions(
    state: State<AppState>,
    min_size: Option<usize>,
) -> Result<Vec<FreeSpaceRegionInfo>, String> {
    let rom_opt = state.rom.lock();
    let rom = rom_opt.as_ref().ok_or("No ROM loaded")?;
    let rom_size = rom.data.len();
    drop(rom_opt);

    let manifest = state.manifest.lock();

    // Build list of allocated regions from manifest
    let mut allocated: Vec<(usize, usize)> = Vec::new();

    for (_fighter_name, boxer) in &manifest.fighters {
        let all_assets: Vec<_> = boxer
            .palette_files
            .iter()
            .chain(boxer.icon_files.iter())
            .chain(boxer.portrait_files.iter())
            .chain(boxer.large_portrait_files.iter())
            .chain(boxer.unique_sprite_bins.iter())
            .chain(boxer.shared_sprite_bins.iter())
            .chain(boxer.other_files.iter())
            .collect();

        for asset in all_assets {
            if let (Ok(start), Ok(end)) =
                (parse_offset(&asset.start_pc), parse_offset(&asset.end_pc))
            {
                allocated.push((start, end));
            }
        }
    }

    let regions = find_free_regions(rom_size, &allocated, min_size);
    Ok(regions.into_iter().map(Into::into).collect())
}

/// Get comprehensive ROM space usage information
#[tauri::command]
pub fn get_rom_space_info(state: State<AppState>) -> Result<RomSpaceInfo, String> {
    let rom_opt = state.rom.lock();
    let rom = rom_opt.as_ref().ok_or("No ROM loaded")?;
    let rom_size = rom.data.len();
    drop(rom_opt);

    let manifest = state.manifest.lock();

    let mut allocated: Vec<(usize, usize)> = Vec::new();
    let mut allocated_bytes = 0usize;

    for (_fighter_name, boxer) in &manifest.fighters {
        let all_assets: Vec<_> = boxer
            .palette_files
            .iter()
            .chain(boxer.icon_files.iter())
            .chain(boxer.portrait_files.iter())
            .chain(boxer.large_portrait_files.iter())
            .chain(boxer.unique_sprite_bins.iter())
            .chain(boxer.shared_sprite_bins.iter())
            .chain(boxer.other_files.iter())
            .collect();

        for asset in all_assets {
            if let (Ok(start), Ok(end)) =
                (parse_offset(&asset.start_pc), parse_offset(&asset.end_pc))
            {
                allocated.push((start, end));
                allocated_bytes += end - start + 1;
            }
        }
    }

    let free_regions = find_free_regions(rom_size, &allocated, None);
    let stats = FreeSpaceStats::calculate(&free_regions, rom_size);
    let free_bytes = rom_size - allocated_bytes;
    let utilization = (allocated_bytes as f32 / rom_size as f32) * 100.0;

    Ok(RomSpaceInfo {
        total_size: rom_size,
        allocated_bytes,
        free_bytes,
        utilization_percent: utilization,
        free_regions: free_regions.into_iter().map(Into::into).collect(),
        fragmentation_score: stats.fragmentation_score,
    })
}

/// Validate a proposed relocation operation
#[tauri::command]
pub fn validate_relocation_command(
    state: State<AppState>,
    src_pc: String,
    dst_pc: String,
    size: usize,
) -> Result<RelocationValidationResult, String> {
    let src = parse_offset(&src_pc)?;
    let dst = parse_offset(&dst_pc)?;

    let rom_opt = state.rom.lock();
    let rom = rom_opt.as_ref().ok_or("No ROM loaded")?;
    let rom_size = rom.data.len();
    drop(rom_opt);

    let manifest = state.manifest.lock();

    let mut allocated: Vec<(usize, usize)> = Vec::new();
    for (_fighter_name, boxer) in &manifest.fighters {
        let all_assets: Vec<_> = boxer
            .palette_files
            .iter()
            .chain(boxer.icon_files.iter())
            .chain(boxer.portrait_files.iter())
            .chain(boxer.large_portrait_files.iter())
            .chain(boxer.unique_sprite_bins.iter())
            .chain(boxer.shared_sprite_bins.iter())
            .chain(boxer.other_files.iter())
            .collect();

        for asset in all_assets {
            if let (Ok(start), Ok(end)) =
                (parse_offset(&asset.start_pc), parse_offset(&asset.end_pc))
            {
                allocated.push((start, end));
            }
        }
    }

    let free_regions = find_free_regions(rom_size, &allocated, None);
    let validation = validate_relocation(rom_size, &free_regions, src, dst, size, true);
    let safety = RelocationSafetyReport::from_validation(&validation);

    let (risk_level, risk_color) = match safety.overall_risk {
        RiskLevel::Low => ("Low".to_string(), "#4ade80".to_string()),
        RiskLevel::Medium => ("Medium".to_string(), "#fbbf24".to_string()),
        RiskLevel::High => ("High".to_string(), "#f87171".to_string()),
        RiskLevel::Critical => ("Critical".to_string(), "#dc2626".to_string()),
    };

    Ok(RelocationValidationResult {
        valid: validation.valid,
        source_pc: src,
        dest_pc: dst,
        size,
        warnings: validation.warnings,
        errors: validation.errors,
        estimated_pointer_updates: validation.estimated_pointer_updates,
        risk_level,
        risk_color,
    })
}

/// Get assets at a specific address
#[tauri::command]
pub fn get_assets_at_address(
    state: State<AppState>,
    pc_offset: String,
) -> Result<Vec<AssetInfo>, String> {
    let offset = parse_offset(&pc_offset)?;

    let manifest = state.manifest.lock();
    let manifest_json = serde_json::to_value(&*manifest).map_err(|e| e.to_string())?;
    drop(manifest);

    let found =
        relocation_core::manifest_update::find_assets_at_address(&manifest_json, offset, None);

    Ok(found
        .into_iter()
        .map(|f| AssetInfo {
            file: f.file,
            category: f.category,
            subtype: f.subtype,
            start_pc: format!("0x{:X}", f.start_pc),
            end_pc: format!("0x{:X}", f.end_pc),
            size: f.size,
        })
        .collect())
}

/// Get asset information by file name
#[tauri::command]
pub fn get_asset_by_file(
    state: State<AppState>,
    boxer_key: String,
    asset_file: String,
) -> Result<AssetInfo, String> {
    let manifest = state.manifest.lock();

    let boxer = manifest
        .fighters
        .values()
        .find(|f| f.key == boxer_key)
        .ok_or_else(|| format!("Boxer '{}' not found", boxer_key))?;

    let all_assets: Vec<_> = boxer
        .palette_files
        .iter()
        .chain(boxer.icon_files.iter())
        .chain(boxer.portrait_files.iter())
        .chain(boxer.large_portrait_files.iter())
        .chain(boxer.unique_sprite_bins.iter())
        .chain(boxer.shared_sprite_bins.iter())
        .chain(boxer.other_files.iter())
        .collect();

    for asset in all_assets {
        if asset.file == asset_file {
            return Ok(AssetInfo::from(asset));
        }
    }

    Err(format!(
        "Asset '{}' not found for boxer '{}'",
        asset_file, boxer_key
    ))
}

/// Perform a relocation of an asset
#[tauri::command]
pub fn relocate_asset(
    state: State<AppState>,
    boxer_key: String,
    asset_file: String,
    new_pc_offset: String,
) -> Result<RelocationResult, String> {
    let new_offset = parse_offset(&new_pc_offset)?;

    // Get asset info first
    let asset_info = get_asset_by_file(state.clone(), boxer_key.clone(), asset_file.clone())?;
    let old_offset = parse_offset(&asset_info.start_pc)?;
    let size = asset_info.size;

    // Get ROM data
    let mut rom_opt = state.rom.lock();
    let rom = rom_opt.as_mut().ok_or("No ROM loaded")?;

    // Read the data from the old location
    let data = rom
        .read_bytes(old_offset, size)
        .map_err(|e| e.to_string())?
        .to_vec();

    // Write to the new location
    rom.write_bytes(new_offset, &data)
        .map_err(|e| e.to_string())?;

    // Clear the old location
    let clear_bytes = vec![0xFFu8; size];
    rom.write_bytes(old_offset, &clear_bytes)
        .map_err(|e| e.to_string())?;

    drop(rom_opt);

    Ok(RelocationResult {
        success: true,
        old_address: asset_info.start_pc,
        new_address: new_pc_offset,
        size,
        boxer_key,
        asset_file,
        warnings: vec!["Note: Manifest and pointer updates must be done separately".to_string()],
    })
}

/// Preview what would be affected by a relocation
#[tauri::command]
pub fn preview_relocation(
    state: State<AppState>,
    src_pc: String,
    dst_pc: String,
    size: usize,
) -> Result<RelocationPreview, String> {
    let src = parse_offset(&src_pc)?;
    let dst = parse_offset(&dst_pc)?;

    let rom_opt = state.rom.lock();
    let rom = rom_opt.as_ref().ok_or("No ROM loaded")?;
    let rom_size = rom.data.len();
    drop(rom_opt);

    let manifest = state.manifest.lock();

    let mut allocated: Vec<(usize, usize)> = Vec::new();
    for (_fighter_name, boxer) in &manifest.fighters {
        let all_assets: Vec<_> = boxer
            .palette_files
            .iter()
            .chain(boxer.icon_files.iter())
            .chain(boxer.portrait_files.iter())
            .chain(boxer.large_portrait_files.iter())
            .chain(boxer.unique_sprite_bins.iter())
            .chain(boxer.shared_sprite_bins.iter())
            .chain(boxer.other_files.iter())
            .collect();

        for asset in all_assets {
            if let (Ok(start), Ok(end)) =
                (parse_offset(&asset.start_pc), parse_offset(&asset.end_pc))
            {
                allocated.push((start, end));
            }
        }
    }

    let manifest_json = serde_json::to_value(&*manifest).map_err(|e| e.to_string())?;
    drop(manifest);

    let free_regions = find_free_regions(rom_size, &allocated, None);
    let validation = validate_relocation(rom_size, &free_regions, src, dst, size, true);

    let dest_assets = relocation_core::manifest_update::find_assets_at_address(
        &manifest_json,
        dst,
        Some(dst + size - 1),
    );

    Ok(RelocationPreview {
        source_region: (src, src + size - 1),
        dest_region: (dst, dst + size - 1),
        valid: validation.valid,
        warnings: validation.warnings,
        errors: validation.errors,
        dest_occupied_by: dest_assets.into_iter().map(|f| f.file).collect(),
        estimated_pointers_to_update: validation.estimated_pointer_updates,
    })
}

// Conversion implementations
impl From<relocation_core::FreeSpaceRegion> for FreeSpaceRegionInfo {
    fn from(region: relocation_core::FreeSpaceRegion) -> Self {
        Self {
            start_pc: region.start_pc,
            end_pc: region.end_pc,
            size: region.size,
            start_snes: format!("0x{:06X}", region.start_snes()),
            end_snes: format!("0x{:06X}", region.end_snes()),
        }
    }
}
