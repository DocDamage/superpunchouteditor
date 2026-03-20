//! Bank Management Commands
//!
//! Commands for ROM bank visualization, analysis, and optimization.

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::app_state::AppState;

/// Bank visualization data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankVisualizationData {
    pub bank_number: u8,
    pub segments: Vec<BankSegmentData>,
    pub used_percent: f32,
    pub free_bytes: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankSegmentData {
    pub offset: usize,
    pub size: usize,
    pub color: [u8; 4],
    pub region_type: String,
    pub description: Option<String>,
}

/// Get bank map visualization
#[tauri::command]
pub fn get_bank_visualization(
    state: State<AppState>,
) -> Result<Vec<BankVisualizationData>, String> {
    let rom_opt = state.rom.lock();
    let rom = rom_opt.as_ref().ok_or("No ROM loaded")?;
    
    // Create bank map from ROM data
    let bank_map = relocation_core::bank_manager::BankMap::from_rom(&rom.data);
    let viz = bank_map.get_visualization();
    
    let result: Vec<BankVisualizationData> = viz.rows.iter().map(|row| {
        BankVisualizationData {
            bank_number: row.bank_number,
            segments: row.segments.iter().map(|seg| BankSegmentData {
                offset: seg.offset,
                size: seg.size,
                color: seg.color,
                region_type: format!("{:?}", seg.region_type),
                description: seg.description.clone(),
            }).collect(),
            used_percent: row.used_percent,
            free_bytes: 0, // Would get from stats
        }
    }).collect();
    
    Ok(result)
}

/// Free region information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreeRegionInfo {
    pub bank: u8,
    pub offset: usize,
    pub size: usize,
    pub pc_offset: usize,
}

/// Find free regions in ROM
#[tauri::command]
pub fn find_free_regions(
    state: State<AppState>,
    min_size: usize,
) -> Result<Vec<FreeRegionInfo>, String> {
    let rom_opt = state.rom.lock();
    let rom = rom_opt.as_ref().ok_or("No ROM loaded")?;
    
    let bank_map = relocation_core::bank_manager::BankMap::from_rom(&rom.data);
    let free_regions = bank_map.find_free_regions(min_size);
    
    Ok(free_regions.iter().map(|r| FreeRegionInfo {
        bank: r.bank,
        offset: r.offset,
        size: r.size,
        pc_offset: r.pc_offset,
    }).collect())
}

/// Fragmentation analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FragmentationAnalysisResult {
    pub overall_fragmentation: f32,
    pub total_free_bytes: usize,
    pub total_used_bytes: usize,
    pub largest_free_block: usize,
    pub gaps: Vec<GapInfo>,
    defrag_suggestions: Vec<DefragSuggestionInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GapInfo {
    pub bank: u8,
    pub offset: usize,
    pub size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefragSuggestionInfo {
    pub gap_bank: u8,
    pub gap_offset: usize,
    pub gap_size: usize,
    pub potential_savings: usize,
    pub movable_regions: Vec<MovableRegionInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovableRegionInfo {
    pub bank: u8,
    pub offset: usize,
    pub size: usize,
    pub region_type: String,
    pub description: Option<String>,
}

/// Analyze ROM fragmentation
#[tauri::command]
pub fn analyze_fragmentation(
    state: State<AppState>,
) -> Result<FragmentationAnalysisResult, String> {
    let rom_opt = state.rom.lock();
    let rom = rom_opt.as_ref().ok_or("No ROM loaded")?;
    
    let bank_map = relocation_core::bank_manager::BankMap::from_rom(&rom.data);
    let analysis = bank_map.analyze_fragmentation();
    
    Ok(FragmentationAnalysisResult {
        overall_fragmentation: analysis.overall_fragmentation,
        total_free_bytes: 0, // Would get from bank_map
        total_used_bytes: 0,
        largest_free_block: 0,
        gaps: analysis.gaps.iter().map(|g| GapInfo {
            bank: g.bank,
            offset: g.offset,
            size: g.size,
        }).collect(),
        defrag_suggestions: analysis.suggestions.iter().map(|s| DefragSuggestionInfo {
            gap_bank: s.gap.bank,
            gap_offset: s.gap.offset,
            gap_size: s.gap.size,
            potential_savings: s.potential_savings,
            movable_regions: s.movable_regions.iter().map(|r| MovableRegionInfo {
                bank: r.bank,
                offset: r.offset,
                size: r.size,
                region_type: format!("{:?}", r.region_type),
                description: r.description.clone(),
            }).collect(),
        }).collect(),
    })
}

/// Defragmentation operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefragOperation {
    pub source_bank: u8,
    pub source_offset: usize,
    pub dest_bank: u8,
    pub dest_offset: usize,
    pub size: usize,
    pub description: String,
}

/// Generate defragmentation plan
#[tauri::command]
pub fn generate_defrag_plan(
    state: State<AppState>,
) -> Result<Vec<DefragOperation>, String> {
    let rom_opt = state.rom.lock();
    let rom = rom_opt.as_ref().ok_or("No ROM loaded")?;
    
    let bank_map = relocation_core::bank_manager::BankMap::from_rom(&rom.data);
    let analysis = bank_map.analyze_fragmentation();
    let planner = relocation_core::bank_manager::DefragmentationPlanner::from_analysis(&analysis);
    
    Ok(planner.plan.iter().map(|op| DefragOperation {
        source_bank: op.source_bank,
        source_offset: op.source_offset,
        dest_bank: op.dest_bank,
        dest_offset: op.dest_offset,
        size: op.size,
        description: op.description.clone(),
    }).collect())
}

/// Execute defragmentation plan
#[tauri::command]
pub fn execute_defrag_plan(
    _state: State<AppState>,
    operations: Vec<DefragOperation>,
) -> Result<(), String> {
    // This would execute the defragmentation
    // For safety, this should create a backup first
    Err("Defragmentation not yet fully implemented".into())
}

/// Mark a region in the bank map
#[tauri::command]
pub fn mark_bank_region(
    _state: State<AppState>,
    bank: u8,
    offset: usize,
    size: usize,
    region_type: String,
    description: Option<String>,
) -> Result<(), String> {
    // Placeholder - would update the bank map
    Ok(())
}

/// Get ROM statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RomStatistics {
    pub total_size: usize,
    pub used_bytes: usize,
    pub free_bytes: usize,
    pub bank_count: usize,
    pub fragmentation_score: f32,
}

#[tauri::command]
pub fn get_rom_statistics(state: State<AppState>) -> Result<RomStatistics, String> {
    let rom_opt = state.rom.lock();
    let rom = rom_opt.as_ref().ok_or("No ROM loaded")?;
    
    let bank_map = relocation_core::bank_manager::BankMap::from_rom(&rom.data);
    
    Ok(RomStatistics {
        total_size: rom.data.len(),
        used_bytes: bank_map.total_stats.used_bytes,
        free_bytes: bank_map.total_stats.free_bytes,
        bank_count: bank_map.banks.len(),
        fragmentation_score: bank_map.total_stats.fragmentation,
    })
}
