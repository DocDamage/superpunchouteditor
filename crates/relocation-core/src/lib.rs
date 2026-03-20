use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod bank_manager;
pub mod free_space;
pub mod manifest_update;
pub mod relocation;

pub use bank_manager::*;
pub use free_space::{find_free_regions, FreeSpaceRegion};
pub use manifest_update::{update_manifest_address, ManifestUpdateError};
pub use relocation::{validate_relocation, PointerUpdate, RelocationPlanner, RelocationValidation, RelocationSafetyReport, RiskLevel};

/// Errors that can occur during relocation operations
#[derive(Error, Debug)]
pub enum RelocationError {
    #[error("Invalid offset: {0}")]
    InvalidOffset(usize),
    #[error("Invalid size: {0}")]
    InvalidSize(usize),
    #[error("Source range overlaps with destination")]
    OverlappingRanges,
    #[error("Destination range exceeds ROM size")]
    ExceedsRomSize,
    #[error("Not enough free space at destination")]
    InsufficientSpace,
    #[error("Source data not found at specified offset")]
    SourceNotFound,
    #[error("Relocation would overwrite existing data")]
    WouldOverwrite,
    #[error("ROM error: {0}")]
    RomError(String),
    #[error("Manifest error: {0}")]
    ManifestError(String),
}

/// Represents a region of used/allocated space in the ROM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocatedRegion {
    pub start_pc: usize,
    pub end_pc: usize,
    pub size: usize,
    pub description: String,
    pub asset_file: Option<String>,
}

/// Information about a potential relocation operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelocationInfo {
    pub source: AllocatedRegion,
    pub proposed_destination: usize,
    pub new_size: usize,
    pub valid: bool,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub pointer_updates_needed: Vec<PointerUpdate>,
}

/// Summary of ROM space usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RomSpaceSummary {
    pub total_size: usize,
    pub allocated_bytes: usize,
    pub free_bytes: usize,
    pub allocated_regions: Vec<AllocatedRegion>,
    pub free_regions: Vec<FreeSpaceRegion>,
    pub utilization_percent: f32,
}

impl RomSpaceSummary {
    pub fn new(rom_size: usize) -> Self {
        Self {
            total_size: rom_size,
            allocated_bytes: 0,
            free_bytes: rom_size,
            allocated_regions: Vec::new(),
            free_regions: vec![FreeSpaceRegion {
                start_pc: 0,
                end_pc: rom_size - 1,
                size: rom_size,
            }],
            utilization_percent: 0.0,
        }
    }

    pub fn add_allocated_region(&mut self, region: AllocatedRegion) {
        self.allocated_regions.push(region);
        self.recalculate();
    }

    fn recalculate(&mut self) {
        // Sort allocated regions
        self.allocated_regions.sort_by_key(|r| r.start_pc);

        // Calculate free regions (gaps between allocated regions)
        self.free_regions = self.calculate_free_regions();

        // Calculate totals
        self.allocated_bytes = self.allocated_regions.iter().map(|r| r.size).sum();
        self.free_bytes = self.total_size - self.allocated_bytes;
        self.utilization_percent = (self.allocated_bytes as f32 / self.total_size as f32) * 100.0;
    }

    fn calculate_free_regions(&self) -> Vec<FreeSpaceRegion> {
        let mut free_regions = Vec::new();
        let mut current_pos = 0usize;

        for region in &self.allocated_regions {
            if region.start_pc > current_pos {
                free_regions.push(FreeSpaceRegion {
                    start_pc: current_pos,
                    end_pc: region.start_pc - 1,
                    size: region.start_pc - current_pos,
                });
            }
            current_pos = region.end_pc + 1;
        }

        // Add trailing free space if any
        if current_pos < self.total_size {
            free_regions.push(FreeSpaceRegion {
                start_pc: current_pos,
                end_pc: self.total_size - 1,
                size: self.total_size - current_pos,
            });
        }

        free_regions
    }
}

/// Scans manifest data to build a complete picture of allocated ROM regions
pub fn build_space_summary_from_manifest(
    manifest: &manifest_core::Manifest,
    rom_size: usize,
) -> RomSpaceSummary {
    let mut summary = RomSpaceSummary::new(rom_size);

    // Collect all asset files from all fighters
    let mut all_assets: Vec<(String, manifest_core::AssetFile)> = Vec::new();

    for (fighter_name, boxer) in &manifest.fighters {
        let assets: Vec<_> = boxer
            .palette_files
            .iter()
            .chain(boxer.icon_files.iter())
            .chain(boxer.portrait_files.iter())
            .chain(boxer.large_portrait_files.iter())
            .chain(boxer.unique_sprite_bins.iter())
            .chain(boxer.shared_sprite_bins.iter())
            .chain(boxer.other_files.iter())
            .cloned()
            .map(|a| (fighter_name.clone(), a))
            .collect();
        all_assets.extend(assets);
    }

    // Parse addresses and add to summary
    for (fighter_name, asset) in all_assets {
        if let (Ok(start), Ok(end)) = (
            parse_hex_or_dec(&asset.start_pc),
            parse_hex_or_dec(&asset.end_pc),
        ) {
            summary.add_allocated_region(AllocatedRegion {
                start_pc: start,
                end_pc: end,
                size: asset.size,
                description: format!("{} - {}", fighter_name, asset.subtype),
                asset_file: Some(asset.file.clone()),
            });
        }
    }

    summary
}

/// Parse a hex string (with or without 0x prefix) or decimal string
fn parse_hex_or_dec(s: &str) -> Result<usize, std::num::ParseIntError> {
    let s = s.trim();
    if s.starts_with("0x") || s.starts_with("0X") {
        usize::from_str_radix(&s[2..], 16)
    } else {
        s.parse::<usize>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rom_space_summary() {
        let mut summary = RomSpaceSummary::new(1024);

        summary.add_allocated_region(AllocatedRegion {
            start_pc: 100,
            end_pc: 199,
            size: 100,
            description: "Test Region".to_string(),
            asset_file: None,
        });

        assert_eq!(summary.allocated_bytes, 100);
        assert_eq!(summary.free_bytes, 924);
        assert_eq!(summary.free_regions.len(), 2); // Before and after
    }

    #[test]
    fn test_parse_hex_or_dec() {
        assert_eq!(parse_hex_or_dec("0x100").unwrap(), 256);
        assert_eq!(parse_hex_or_dec("100").unwrap(), 100);
        assert_eq!(parse_hex_or_dec("0x1A").unwrap(), 26);
    }
}
