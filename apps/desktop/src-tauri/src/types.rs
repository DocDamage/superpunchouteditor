//! Shared Type Definitions for Super Punch-Out!! Editor
//!
//! Common types, structs, and enums used across multiple command modules.
//! These are primarily DTOs (Data Transfer Objects) for serialization
//! between the Rust backend and the TypeScript frontend.

use serde::{Deserialize, Serialize};

// ============================================================================
// Layout Pack Types
// ============================================================================

/// Layout Pack Metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutPackMetadata {
    pub name: String,
    pub author: String,
    pub description: String,
}

/// Individual bin in a layout
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutBin {
    pub filename: String,
    pub pc_offset: String,
    pub size: usize,
    pub category: String,
    pub label: Option<String>,
}

/// Boxer layout within a pack
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackBoxerLayout {
    pub boxer_key: String,
    pub version: String,
    pub layout_type: String, // 'reference' or 'custom'
    pub bins: Vec<LayoutBin>,
    pub notes: Option<String>,
}

/// Full layout pack structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutPack {
    pub version: String,
    pub name: String,
    pub author: String,
    pub description: String,
    pub created_at: String,
    pub layouts: Vec<PackBoxerLayout>,
}

/// Layout pack info for listing (without full layouts)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutPackInfo {
    pub filename: String,
    pub name: String,
    pub author: String,
    pub description: String,
    pub created_at: String,
    pub boxer_count: usize,
}

/// Validation report for layout pack import
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    pub valid: bool,
    pub version_compatible: bool,
    pub boxer_validations: Vec<BoxerValidation>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

/// Individual boxer validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoxerValidation {
    pub boxer_key: String,
    pub exists_in_manifest: bool,
    pub bins_valid: bool,
    pub size_matches: bool,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

// ============================================================================
// Relocation Types
// ============================================================================

/// Information about a free space region for the frontend
#[derive(Debug, Clone, Serialize)]
pub struct FreeSpaceRegionInfo {
    pub start_pc: usize,
    pub end_pc: usize,
    pub size: usize,
    pub start_snes: String,
    pub end_snes: String,
}

/// Information about ROM space usage
#[derive(Debug, Clone, Serialize)]
pub struct RomSpaceInfo {
    pub total_size: usize,
    pub allocated_bytes: usize,
    pub free_bytes: usize,
    pub utilization_percent: f32,
    pub free_regions: Vec<FreeSpaceRegionInfo>,
    pub fragmentation_score: f32,
}

/// Relocation validation result for the frontend
#[derive(Debug, Clone, Serialize)]
pub struct RelocationValidationResult {
    pub valid: bool,
    pub source_pc: usize,
    pub dest_pc: usize,
    pub size: usize,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub estimated_pointer_updates: usize,
    pub risk_level: String,
    pub risk_color: String,
}

/// Asset information for relocation UI
#[derive(Debug, Clone, Serialize)]
pub struct AssetInfo {
    pub file: String,
    pub category: String,
    pub subtype: String,
    pub start_pc: String,
    pub end_pc: String,
    pub size: usize,
}

impl From<&manifest_core::AssetFile> for AssetInfo {
    fn from(asset: &manifest_core::AssetFile) -> Self {
        Self {
            file: asset.file.clone(),
            category: asset.category.clone(),
            subtype: asset.subtype.clone(),
            start_pc: asset.start_pc.clone(),
            end_pc: asset.end_pc.clone(),
            size: asset.size,
        }
    }
}

/// Result of a relocation operation
#[derive(Debug, Clone, Serialize)]
pub struct RelocationResult {
    pub success: bool,
    pub old_address: String,
    pub new_address: String,
    pub size: usize,
    pub boxer_key: String,
    pub asset_file: String,
    pub warnings: Vec<String>,
}

/// Preview of a relocation operation
#[derive(Debug, Clone, Serialize)]
pub struct RelocationPreview {
    pub source_region: (usize, usize),
    pub dest_region: (usize, usize),
    pub valid: bool,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub dest_occupied_by: Vec<String>,
    pub estimated_pointers_to_update: usize,
}

// ============================================================================
// Comparison Types
// ============================================================================

/// Available comparison view modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComparisonViewMode {
    SideBySide,
    Overlay,
    Difference,
    Split,
}

impl From<&str> for ComparisonViewMode {
    fn from(s: &str) -> Self {
        match s {
            "overlay" => Self::Overlay,
            "difference" => Self::Difference,
            "split" => Self::Split,
            _ => Self::SideBySide,
        }
    }
}

// ============================================================================
// Constants
// ============================================================================

/// Layout pack file format version
pub const LAYOUT_PACK_VERSION: &str = "1.0";

/// Path to default layouts directory
#[allow(dead_code)]
pub const DEFAULT_LAYOUTS_DIR: &str = "../../data/boxer-layouts/default";

/// Path to community layouts directory
pub const COMMUNITY_LAYOUTS_DIR: &str = "../../data/boxer-layouts/community";
