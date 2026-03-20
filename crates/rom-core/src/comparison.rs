//! ROM Comparison Module
//!
//! Provides functionality to compare the current modified ROM state against
//! the original ROM to identify and visualize changes.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a complete comparison between original and modified ROMs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RomComparison {
    /// SHA1 hash of the original ROM
    pub original_sha1: String,
    /// SHA1 hash of the modified ROM (includes pending writes)
    pub modified_sha1: String,
    /// List of all detected differences
    pub differences: Vec<Difference>,
    /// Summary statistics
    pub summary: ComparisonSummary,
}

/// Summary statistics for a comparison
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComparisonSummary {
    pub total_changes: usize,
    pub palettes_modified: usize,
    pub sprite_bins_changed: usize,
    pub tiles_changed: usize,
    pub fighter_headers_edited: usize,
    pub animation_timings_adjusted: usize,
    pub total_bytes_changed: usize,
}

/// Represents a single detected difference
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Difference {
    /// Palette color changes
    Palette {
        /// PC offset of the palette
        offset: usize,
        /// Asset identifier (e.g., "gabby_jay/palette_1")
        asset_id: String,
        /// Fighter/boxer name
        boxer: String,
        /// Original colors (RGB)
        original_colors: Vec<ColorDiff>,
        /// Modified colors (RGB)
        modified_colors: Vec<ColorDiff>,
        /// Indices of changed colors
        changed_indices: Vec<usize>,
    },
    /// Sprite/tile data changes
    Sprite {
        /// Boxer name
        boxer: String,
        /// Binary asset name
        bin_name: String,
        /// PC offset of the sprite bin
        pc_offset: usize,
        /// Number of tiles in this bin
        total_tiles: usize,
        /// Indices of tiles that changed
        changed_tile_indices: Vec<usize>,
        /// Per-tile change count
        tile_change_counts: HashMap<usize, usize>,
    },
    /// Fighter header parameter changes
    Header {
        /// Boxer name
        boxer: String,
        /// Fighter index
        fighter_index: usize,
        /// Changed fields with before/after values
        changed_fields: Vec<HeaderFieldChange>,
    },
    /// Animation timing/frame changes
    Animation {
        /// Boxer name
        boxer: String,
        /// Animation name/identifier
        anim_name: String,
        /// Frame index that changed
        frame_index: usize,
        /// Type of change
        change_type: AnimationChangeType,
    },
    /// Raw binary changes
    Binary {
        /// PC offset
        offset: usize,
        /// Size of changed region
        size: usize,
        /// Number of bytes that differ
        bytes_changed: usize,
        /// Description of what changed
        description: String,
    },
}

/// RGB Color representation for diffs
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ColorDiff {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl ColorDiff {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Convert from SNES 15-bit BGR format
    pub fn from_snes_bytes(low: u8, high: u8) -> Self {
        let color = (high as u16) << 8 | (low as u16);
        let r5 = (color & 0x1F) as u8;
        let g5 = ((color >> 5) & 0x1F) as u8;
        let b5 = ((color >> 10) & 0x1F) as u8;

        // Convert 5-bit to 8-bit
        Self {
            r: (r5 << 3) | (r5 >> 2),
            g: (g5 << 3) | (g5 >> 2),
            b: (b5 << 3) | (b5 >> 2),
        }
    }

    /// Convert to hex string
    pub fn to_hex(&self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }
}

/// A changed header field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeaderFieldChange {
    pub field_name: String,
    pub original_value: u32,
    pub modified_value: u32,
    pub display_name: String,
}

/// Types of animation changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnimationChangeType {
    FrameCount { original: u8, modified: u8 },
    Timing { original: u8, modified: u8 },
    FrameData { description: String },
}

/// Palette diff view data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaletteDiff {
    pub offset: usize,
    pub boxer: String,
    pub asset_id: String,
    pub colors: Vec<ColorComparison>,
}

/// Individual color comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorComparison {
    pub index: usize,
    pub original: ColorDiff,
    pub modified: ColorDiff,
    pub changed: bool,
}

/// Sprite diff view data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpriteDiff {
    pub pc_offset: usize,
    pub boxer: String,
    pub bin_name: String,
    pub total_tiles: usize,
    pub changed_tiles: Vec<TileDiff>,
}

/// Individual tile diff
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileDiff {
    pub tile_index: usize,
    pub pixel_diffs: Vec<PixelDiff>,
    pub has_changes: bool,
}

/// Individual pixel difference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PixelDiff {
    pub x: usize,
    pub y: usize,
    pub original_pixel: u8, // Palette index
    pub modified_pixel: u8,
    pub changed: bool,
}

/// Header diff view data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeaderDiff {
    pub boxer: String,
    pub fighter_index: usize,
    pub fields: Vec<HeaderFieldComparison>,
}

/// Individual header field comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeaderFieldComparison {
    pub field_name: String,
    pub display_name: String,
    pub original: u32,
    pub modified: u32,
    pub changed: bool,
    pub delta: i32,
}

/// Binary/hex diff view data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryDiff {
    pub offset: usize,
    pub size: usize,
    pub rows: Vec<HexRow>,
}

/// A row of hex bytes (16 bytes typically)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HexRow {
    pub address: String,
    pub bytes: Vec<HexByte>,
    pub ascii: String,
}

/// Individual byte with diff status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HexByte {
    pub value: u8,
    pub changed: bool,
    pub original_value: Option<u8>,
}

/// Comparison view mode for rendering
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ComparisonViewMode {
    SideBySide,
    Overlay,
    Difference,
    Split,
    Blink,
}

/// Request to render a comparison view
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonRenderRequest {
    pub boxer_key: String,
    pub view_type: ComparisonViewType,
    pub mode: ComparisonViewMode,
    pub show_original: bool,
    pub show_modified: bool,
    pub asset_offset: Option<usize>,
    pub palette_offset: Option<usize>,
}

/// Type of asset being compared
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ComparisonViewType {
    Sprite,
    Frame,
    Animation,
    Palette,
    Portrait,
    Icon,
}

/// Comparison report export format
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ExportFormat {
    Html,
    Json,
    Text,
    IpsPreview,
}

/// Export options for comparison report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportOptions {
    pub format: ExportFormat,
    pub include_images: bool,
    pub include_unchanged: bool,
    pub boxer_filter: Option<String>,
}

impl RomComparison {
    /// Create a new empty comparison
    pub fn new(original_sha1: String, modified_sha1: String) -> Self {
        Self {
            original_sha1,
            modified_sha1,
            differences: Vec::new(),
            summary: ComparisonSummary::default(),
        }
    }

    /// Add a difference to the comparison
    pub fn add_difference(&mut self, diff: Difference) {
        // Update summary based on difference type
        match &diff {
            Difference::Palette {
                changed_indices, ..
            } => {
                self.summary.palettes_modified += 1;
                self.summary.total_bytes_changed += changed_indices.len() * 2; // 2 bytes per color
            }
            Difference::Sprite {
                changed_tile_indices,
                ..
            } => {
                self.summary.sprite_bins_changed += 1;
                self.summary.tiles_changed += changed_tile_indices.len();
                self.summary.total_bytes_changed += changed_tile_indices.len() * 32;
                // 32 bytes per tile
            }
            Difference::Header { changed_fields, .. } => {
                self.summary.fighter_headers_edited += 1;
                self.summary.total_bytes_changed += changed_fields.len();
            }
            Difference::Animation { .. } => {
                self.summary.animation_timings_adjusted += 1;
            }
            Difference::Binary { bytes_changed, .. } => {
                self.summary.total_bytes_changed += bytes_changed;
            }
        }
        self.summary.total_changes += 1;
        self.differences.push(diff);
    }

    /// Get all palette differences
    pub fn get_palette_diffs(&self) -> Vec<&Difference> {
        self.differences
            .iter()
            .filter(|d| matches!(d, Difference::Palette { .. }))
            .collect()
    }

    /// Get all sprite differences
    pub fn get_sprite_diffs(&self) -> Vec<&Difference> {
        self.differences
            .iter()
            .filter(|d| matches!(d, Difference::Sprite { .. }))
            .collect()
    }

    /// Get all header differences
    pub fn get_header_diffs(&self) -> Vec<&Difference> {
        self.differences
            .iter()
            .filter(|d| matches!(d, Difference::Header { .. }))
            .collect()
    }

    /// Get differences for a specific boxer
    pub fn get_boxer_diffs(&self, boxer: &str) -> Vec<&Difference> {
        self.differences
            .iter()
            .filter(|d| {
                let boxer_name = match d {
                    Difference::Palette { boxer, .. } => boxer,
                    Difference::Sprite { boxer, .. } => boxer,
                    Difference::Header { boxer, .. } => boxer,
                    Difference::Animation { boxer, .. } => boxer,
                    Difference::Binary { .. } => return false,
                };
                boxer_name == boxer
            })
            .collect()
    }

    /// Filter differences by type
    pub fn filter_by_type(&self, filter: DifferenceFilter) -> Vec<&Difference> {
        self.differences
            .iter()
            .filter(|d| match (filter, d) {
                (DifferenceFilter::Palette, Difference::Palette { .. }) => true,
                (DifferenceFilter::Sprite, Difference::Sprite { .. }) => true,
                (DifferenceFilter::Header, Difference::Header { .. }) => true,
                (DifferenceFilter::Animation, Difference::Animation { .. }) => true,
                (DifferenceFilter::Binary, Difference::Binary { .. }) => true,
                (DifferenceFilter::All, _) => true,
                _ => false,
            })
            .collect()
    }
}

/// Filter for difference queries
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DifferenceFilter {
    All,
    Palette,
    Sprite,
    Header,
    Animation,
    Binary,
}

/// Comparison engine for analyzing ROM changes
pub struct ComparisonEngine;

impl ComparisonEngine {
    /// Compare two byte arrays and return indices of differences
    pub fn compare_bytes(original: &[u8], modified: &[u8]) -> Vec<usize> {
        let min_len = original.len().min(modified.len());
        let mut diffs = Vec::new();

        for i in 0..min_len {
            if original[i] != modified[i] {
                diffs.push(i);
            }
        }

        // Handle length differences
        if modified.len() > original.len() {
            for i in original.len()..modified.len() {
                diffs.push(i);
            }
        }

        diffs
    }

    /// Compare tiles (32 bytes each) and return indices of changed tiles
    pub fn compare_tiles(original: &[u8], modified: &[u8]) -> Vec<usize> {
        let tile_size = 32;
        let original_tile_count = original.len() / tile_size;
        let modified_tile_count = modified.len() / tile_size;
        let tile_count = original_tile_count.max(modified_tile_count);

        let mut changed = Vec::new();

        for i in 0..tile_count {
            let start = i * tile_size;
            let end = start + tile_size;

            let orig_tile = original.get(start..end.min(original.len()));
            let mod_tile = modified.get(start..end.min(modified.len()));

            if orig_tile != mod_tile {
                changed.push(i);
            }
        }

        changed
    }

    /// Generate a hex diff view
    pub fn generate_hex_diff(original: &[u8], modified: &[u8], offset: usize) -> BinaryDiff {
        let bytes_per_row = 16;
        let diff_indices: std::collections::HashSet<usize> =
            Self::compare_bytes(original, modified)
                .into_iter()
                .collect();

        let max_len = original.len().max(modified.len());
        let row_count = (max_len + bytes_per_row - 1) / bytes_per_row;

        let mut rows = Vec::with_capacity(row_count);

        for row in 0..row_count {
            let row_start = row * bytes_per_row;
            let row_end = (row_start + bytes_per_row).min(max_len);

            let mut bytes = Vec::with_capacity(bytes_per_row);
            let mut ascii = String::with_capacity(bytes_per_row);

            for i in row_start..row_end {
                let modified_val = modified.get(i).copied();
                let original_val = original.get(i).copied();
                let changed = diff_indices.contains(&i);

                bytes.push(HexByte {
                    value: modified_val.unwrap_or(0),
                    changed,
                    original_value: original_val,
                });

                // Build ASCII representation
                let ch = modified_val
                    .map(|b| {
                        if b.is_ascii_graphic() || b == b' ' {
                            b as char
                        } else {
                            '.'
                        }
                    })
                    .unwrap_or('.');
                ascii.push(ch);
            }

            rows.push(HexRow {
                address: format!("0x{:06X}", offset + row_start),
                bytes,
                ascii,
            });
        }

        BinaryDiff {
            offset,
            size: max_len,
            rows,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compare_bytes() {
        let original = vec![0x00, 0x01, 0x02, 0x03, 0x04];
        let modified = vec![0x00, 0xFF, 0x02, 0xFE, 0x04];

        let diffs = ComparisonEngine::compare_bytes(&original, &modified);
        assert_eq!(diffs, vec![1, 3]);
    }

    #[test]
    fn test_compare_tiles() {
        // 2 tiles (32 bytes each)
        let mut original = vec![0u8; 64];
        let mut modified = vec![0u8; 64];

        // Change byte in second tile
        modified[33] = 0xFF;

        let changed = ComparisonEngine::compare_tiles(&original, &modified);
        assert_eq!(changed, vec![1]);
    }

    #[test]
    fn test_color_from_snes() {
        // Test conversion from SNES 15-bit BGR
        // White: 0b0_11111_11111_11111 = 0x7FFF
        let white = ColorDiff::from_snes_bytes(0xFF, 0x7F);
        assert_eq!(white.r, 0xF8);
        assert_eq!(white.g, 0xF8);
        assert_eq!(white.b, 0xF8);
    }

    #[test]
    fn test_comparison_summary() {
        let mut comp = RomComparison::new("abc123".to_string(), "def456".to_string());

        comp.add_difference(Difference::Palette {
            offset: 0x1000,
            asset_id: "test/palette".to_string(),
            boxer: "Test".to_string(),
            original_colors: vec![ColorDiff::new(0, 0, 0)],
            modified_colors: vec![ColorDiff::new(255, 255, 255)],
            changed_indices: vec![0],
        });

        assert_eq!(comp.summary.total_changes, 1);
        assert_eq!(comp.summary.palettes_modified, 1);
    }
}
