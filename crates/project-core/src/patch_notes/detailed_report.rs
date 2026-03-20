//! Detailed asset report with granular change tracking

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::patch_notes::types::ChangeSummary;
use crate::patch_notes::PatchNotes;

/// Warning level for shared assets indicating impact scope
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WarningLevel {
    /// Only affects selected boxer (unique asset)
    Safe,
    /// Shared but owner consented
    Caution,
    /// Shared, affects other boxers
    Warning,
    /// Core game data
    Critical,
}

impl WarningLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            WarningLevel::Safe => "safe",
            WarningLevel::Caution => "caution",
            WarningLevel::Warning => "warning",
            WarningLevel::Critical => "critical",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            WarningLevel::Safe => "Safe",
            WarningLevel::Caution => "Caution",
            WarningLevel::Warning => "Warning",
            WarningLevel::Critical => "Critical",
        }
    }

    pub fn color(&self) -> &'static str {
        match self {
            WarningLevel::Safe => "#4ade80",
            WarningLevel::Caution => "#fbbf24",
            WarningLevel::Warning => "#f87171",
            WarningLevel::Critical => "#dc2626",
        }
    }
}

/// A color in RGB format with optional alpha
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorRgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: Option<u8>,
}

impl ColorRgb {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: None }
    }

    pub fn to_hex(&self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }
}

/// Detailed information about a palette change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaletteChangeDetail {
    pub name: String,
    pub pc_offset: String,
    pub snes_offset: String,
    pub colors_changed: Vec<usize>,
    pub preview_before: Vec<ColorRgb>,
    pub preview_after: Vec<ColorRgb>,
    pub total_colors: usize,
}

/// Detailed information about a sprite/tile change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpriteChangeDetail {
    pub bin_name: String,
    pub pc_offset: String,
    pub snes_offset: String,
    pub tiles_modified: Vec<usize>,
    pub total_tiles: usize,
    pub size_change: i64,
    pub original_size: usize,
    pub new_size: usize,
    pub is_compressed: bool,
    pub compression_ratio: Option<f32>,
}

/// Detailed information about a stat/parameter change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatChangeDetail {
    pub field: String,
    pub before: String,
    pub after: String,
    pub numeric_change: Option<f64>,
    pub percent_change: Option<f64>,
    pub significant: bool,
}

/// Detailed information about an animation change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationChangeDetail {
    pub name: String,
    pub pc_offset: String,
    pub frames_changed: Vec<usize>,
    pub total_frames: usize,
    pub duration_change: Option<i64>,
}

/// Information about a shared asset that will be affected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedAssetReport {
    pub asset_name: String,
    pub pc_offset: String,
    pub shared_between: Vec<String>,
    pub change_type: String,
    pub warning_level: WarningLevel,
    pub description: String,
}

/// Binary change summary (ROM level)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BinaryChangeSummary {
    pub total_bytes_changed: usize,
    pub total_regions_affected: usize,
    pub largest_single_change: usize,
    pub estimated_patch_size: usize,
    pub original_sha1: String,
    pub modified_sha1: String,
}

/// Per-boxer asset report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoxerAssetReport {
    pub boxer_name: String,
    pub boxer_key: String,
    pub palettes: Vec<PaletteChangeDetail>,
    pub sprites: Vec<SpriteChangeDetail>,
    pub stats: Vec<StatChangeDetail>,
    pub animations: Vec<AnimationChangeDetail>,
    pub total_changes: usize,
}

impl BoxerAssetReport {
    pub fn new(boxer_name: String, boxer_key: String) -> Self {
        Self {
            boxer_name,
            boxer_key,
            palettes: Vec::new(),
            sprites: Vec::new(),
            stats: Vec::new(),
            animations: Vec::new(),
            total_changes: 0,
        }
    }

    pub fn update_total(&mut self) {
        self.total_changes =
            self.palettes.len() + self.sprites.len() + self.stats.len() + self.animations.len();
    }
}

/// Complete detailed asset report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedAssetReport {
    pub summary: ChangeSummary,
    pub boxer_reports: Vec<BoxerAssetReport>,
    pub shared_assets_touched: Vec<SharedAssetReport>,
    pub binary_changes: BinaryChangeSummary,
    pub generated_at: String,
    pub project_name: String,
    pub project_version: String,
}

impl DetailedAssetReport {
    pub fn new(project_name: String, project_version: String) -> Self {
        Self {
            summary: ChangeSummary::default(),
            boxer_reports: Vec::new(),
            shared_assets_touched: Vec::new(),
            binary_changes: BinaryChangeSummary::default(),
            generated_at: Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
            project_name,
            project_version,
        }
    }

    /// Generate a detailed report from pending writes and ROM data
    pub fn generate(
        rom: &dyn crate::patch_notes::RomAccess,
        pending: &HashMap<String, Vec<u8>>,
        manifest: &serde_json::Value,
        boxer_names: &HashMap<String, String>,
    ) -> Result<Self, String> {
        let mut report = Self::new("Untitled Project".to_string(), "1.0.0".to_string());

        // Track which assets we've processed
        let mut processed_offsets: std::collections::HashSet<String> = std::collections::HashSet::new();

        for (pc_offset_str, new_bytes) in pending {
            if processed_offsets.contains(pc_offset_str) {
                continue;
            }
            processed_offsets.insert(pc_offset_str.clone());

            // Parse the offset
            let pc_offset = Self::parse_offset(pc_offset_str)?;

            // Get original bytes from ROM
            let original_bytes = rom
                .read_bytes(pc_offset, new_bytes.len())
                .map_err(|e| format!("Failed to read ROM at {}: {}", pc_offset_str, e))?;

            // Find which boxer this belongs to
            let boxer_key = boxer_names
                .get(pc_offset_str)
                .cloned()
                .unwrap_or_else(|| "unknown".to_string());

            // Find asset info in manifest
            let asset_info = Self::find_asset_in_manifest(manifest, pc_offset_str);

            // Determine change type and create appropriate detail
            if let Some(ref info) = asset_info {
                let category = info.get("category").and_then(|v| v.as_str()).unwrap_or("");

                if category.contains("Palette") || category.contains("palette") {
                    if let Some(detail) =
                        Self::analyze_palette_change(pc_offset_str, original_bytes, new_bytes, info)
                    {
                        report.add_palette_change(&boxer_key, detail);
                        report.summary.total_palettes_changed += 1;
                    }
                } else if category.contains("Sprite")
                    || category.contains("Compressed")
                    || category.contains("Uncompressed")
                {
                    if let Some(detail) =
                        Self::analyze_sprite_change(pc_offset_str, original_bytes, new_bytes, info)
                    {
                        report.add_sprite_change(&boxer_key, detail);
                        report.summary.total_sprites_edited += 1;
                    }
                } else if category.contains("Script") || category.contains("Header") {
                    report.summary.total_animations_modified += 1;
                } else {
                    report.summary.total_headers_edited += 1;
                }

                // Check if this is a shared asset
                if let Some(shared_with) = info.get("shared_with").and_then(|v| v.as_array()) {
                    if !shared_with.is_empty() {
                        let shared_names: Vec<String> = shared_with
                            .iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect();

                        let warning_level = if shared_names.len() > 1 {
                            WarningLevel::Warning
                        } else {
                            WarningLevel::Caution
                        };

                        report.shared_assets_touched.push(SharedAssetReport {
                            asset_name: info
                                .get("filename")
                                .and_then(|v| v.as_str())
                                .unwrap_or("Unknown")
                                .to_string(),
                            pc_offset: pc_offset_str.clone(),
                            shared_between: shared_names,
                            change_type: category.to_string(),
                            warning_level,
                            description: format!("This asset is shared between multiple boxers"),
                        });
                    }
                }
            } else {
                // Unknown asset - count as header/stat change
                report.summary.total_headers_edited += 1;
            }

            report.summary.total_changes += 1;
        }

        // Calculate binary change summary
        report.binary_changes.total_bytes_changed = pending.values().map(|v| v.len()).sum();
        report.binary_changes.total_regions_affected = pending.len();
        report.binary_changes.largest_single_change =
            pending.values().map(|v| v.len()).max().unwrap_or(0);
        report.binary_changes.estimated_patch_size =
            report.binary_changes.total_bytes_changed + (pending.len() * 5); // IPS header overhead estimate

        // Update boxer totals
        for boxer_report in &mut report.boxer_reports {
            boxer_report.update_total();
        }

        report.summary.total_boxers_modified = report.boxer_reports.len();

        Ok(report)
    }

    fn parse_offset(s: &str) -> Result<usize, String> {
        if s.starts_with("0x") || s.starts_with("0X") {
            usize::from_str_radix(&s[2..], 16).map_err(|e| e.to_string())
        } else {
            s.parse::<usize>().map_err(|e| e.to_string())
        }
    }

    fn find_asset_in_manifest(
        manifest: &serde_json::Value,
        pc_offset: &str,
    ) -> Option<serde_json::Value> {
        if let Some(fighters) = manifest.get("fighters").and_then(|v| v.as_object()) {
            for (_fighter_name, fighter_data) in fighters {
                let asset_arrays = vec![
                    fighter_data.get("palette_files"),
                    fighter_data.get("unique_sprite_bins"),
                    fighter_data.get("shared_sprite_bins"),
                    fighter_data.get("icon_files"),
                    fighter_data.get("portrait_files"),
                    fighter_data.get("large_portrait_files"),
                    fighter_data.get("other_files"),
                ];

                for array_opt in asset_arrays {
                    if let Some(array) = array_opt.and_then(|v| v.as_array()) {
                        for asset in array {
                            if let Some(start_pc) = asset.get("start_pc").and_then(|v| v.as_str()) {
                                if start_pc == pc_offset {
                                    return Some(asset.clone());
                                }
                            }
                        }
                    }
                }
            }
        }
        None
    }

    fn analyze_palette_change(
        pc_offset: &str,
        original: &[u8],
        modified: &[u8],
        asset_info: &serde_json::Value,
    ) -> Option<PaletteChangeDetail> {
        let name = asset_info
            .get("filename")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown Palette")
            .to_string();

        let snes = asset_info
            .get("start_snes")
            .and_then(|v| v.as_str())
            .unwrap_or("0x000000")
            .to_string();

        // Find which colors changed (each color is 2 bytes in SNES BGR format)
        let mut colors_changed = Vec::new();
        let mut before_colors = Vec::new();
        let mut after_colors = Vec::new();

        let color_count = original.len() / 2;
        for i in 0..color_count {
            let idx = i * 2;
            if idx + 1 < original.len() && idx + 1 < modified.len() {
                let orig_color = u16::from_le_bytes([original[idx], original[idx + 1]]);
                let mod_color = u16::from_le_bytes([modified[idx], modified[idx + 1]]);

                // Convert SNES BGR to RGB
                let orig_rgb = Self::snes_color_to_rgb(orig_color);
                let mod_rgb = Self::snes_color_to_rgb(mod_color);

                before_colors.push(orig_rgb.clone());
                after_colors.push(mod_rgb.clone());

                if orig_color != mod_color {
                    colors_changed.push(i);
                }
            }
        }

        if colors_changed.is_empty() {
            return None;
        }

        Some(PaletteChangeDetail {
            name,
            pc_offset: pc_offset.to_string(),
            snes_offset: snes,
            colors_changed,
            preview_before: before_colors,
            preview_after: after_colors,
            total_colors: color_count,
        })
    }

    fn snes_color_to_rgb(snes_color: u16) -> ColorRgb {
        // SNES BGR format: bbbbbgggggrrrrr
        let r = ((snes_color & 0x1F) << 3) as u8;
        let g = (((snes_color >> 5) & 0x1F) << 3) as u8;
        let b = (((snes_color >> 10) & 0x1F) << 3) as u8;
        ColorRgb::new(r, g, b)
    }

    fn analyze_sprite_change(
        pc_offset: &str,
        original: &[u8],
        modified: &[u8],
        asset_info: &serde_json::Value,
    ) -> Option<SpriteChangeDetail> {
        let name = asset_info
            .get("filename")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown Sprite")
            .to_string();

        let snes = asset_info
            .get("start_snes")
            .and_then(|v| v.as_str())
            .unwrap_or("0x000000")
            .to_string();

        let is_compressed = asset_info
            .get("category")
            .and_then(|v| v.as_str())
            .map(|c| c.contains("Compressed"))
            .unwrap_or(false);

        // Each tile is 32 bytes (4bpp: 16 bytes for bitplanes 0-1, 16 bytes for bitplanes 2-3)
        let tile_size = 32;
        let original_tiles = original.len() / tile_size;
        let modified_tiles = modified.len() / tile_size;
        let total_tiles = original_tiles.max(modified_tiles);

        // Find which tiles changed
        let mut tiles_modified = Vec::new();
        for i in 0..total_tiles {
            let start = i * tile_size;
            let end = start + tile_size;

            let orig_tile = &original.get(start..end.min(original.len())).unwrap_or(&[]);
            let mod_tile = &modified.get(start..end.min(modified.len())).unwrap_or(&[]);

            if orig_tile != mod_tile {
                tiles_modified.push(i);
            }
        }

        if tiles_modified.is_empty() {
            return None;
        }

        let size_change = modified.len() as i64 - original.len() as i64;

        Some(SpriteChangeDetail {
            bin_name: name,
            pc_offset: pc_offset.to_string(),
            snes_offset: snes,
            tiles_modified,
            total_tiles,
            size_change,
            original_size: original.len(),
            new_size: modified.len(),
            is_compressed,
            compression_ratio: None, // Would need to decompress to calculate
        })
    }

    fn add_palette_change(&mut self, boxer_key: &str, detail: PaletteChangeDetail) {
        let boxer_name = PatchNotes::format_boxer_name(boxer_key);

        if let Some(report) = self
            .boxer_reports
            .iter_mut()
            .find(|r| r.boxer_key == boxer_key)
        {
            report.palettes.push(detail);
        } else {
            let mut report = BoxerAssetReport::new(boxer_name, boxer_key.to_string());
            report.palettes.push(detail);
            self.boxer_reports.push(report);
        }
    }

    fn add_sprite_change(&mut self, boxer_key: &str, detail: SpriteChangeDetail) {
        let boxer_name = PatchNotes::format_boxer_name(boxer_key);

        if let Some(report) = self
            .boxer_reports
            .iter_mut()
            .find(|r| r.boxer_key == boxer_key)
        {
            report.sprites.push(detail);
        } else {
            let mut report = BoxerAssetReport::new(boxer_name, boxer_key.to_string());
            report.sprites.push(detail);
            self.boxer_reports.push(report);
        }
    }

    /// Render the detailed report to HTML format
    pub fn render_html(&self) -> String {
        let mut output = String::new();

        output.push_str("<!DOCTYPE html>\n");
        output.push_str("<html lang=\"en\">\n<head>\n");
        output.push_str(&format!(
            "<title>Detailed Asset Report - {}</title>\n",
            self.project_name
        ));
        output.push_str("<style>\n");
        output.push_str(Self::detailed_report_css());
        output.push_str("</style>\n");
        output.push_str("</head>\n<body>\n");

        // Header
        output.push_str(&format!("<h1>{}</h1>\n", self.project_name));
        output.push_str(&format!(
            "<p class=\"version\">Version {} &bull; Generated {}</p>\n",
            self.project_version, self.generated_at
        ));

        // Summary
        output.push_str("<h2>Summary</h2>\n");
        output.push_str("<div class=\"summary-grid\">\n");
        output.push_str(&format!("<div class=\"stat\"><span class=\"stat-value\">{}</span><span class=\"stat-label\">Boxers Modified</span></div>\n", 
            self.summary.total_boxers_modified));
        output.push_str(&format!("<div class=\"stat\"><span class=\"stat-value\">{}</span><span class=\"stat-label\">Palettes Changed</span></div>\n", 
            self.summary.total_palettes_changed));
        output.push_str(&format!("<div class=\"stat\"><span class=\"stat-value\">{}</span><span class=\"stat-label\">Sprites Edited</span></div>\n", 
            self.summary.total_sprites_edited));
        output.push_str(&format!("<div class=\"stat\"><span class=\"stat-value\">{}</span><span class=\"stat-label\">Animations Modified</span></div>\n", 
            self.summary.total_animations_modified));
        output.push_str(&format!("<div class=\"stat\"><span class=\"stat-value\">{}</span><span class=\"stat-label\">Total Changes</span></div>\n", 
            self.summary.total_changes));
        output.push_str("</div>\n");

        // Shared Assets Warnings
        if !self.shared_assets_touched.is_empty() {
            output.push_str("<h2>Shared Assets</h2>\n");
            output.push_str("<div class=\"shared-assets\">\n");
            for asset in &self.shared_assets_touched {
                let warning_class = asset.warning_level.as_str();
                output.push_str(&format!(
                    "<div class=\"shared-asset {}\">\n<h4>{}</h4>\n<p>{} at {}</p>\n<p class=\"shared-with\">Shared with: {}</p>\n</div>\n",
                    warning_class,
                    asset.asset_name,
                    asset.change_type,
                    asset.pc_offset,
                    asset.shared_between.join(", ")
                ));
            }
            output.push_str("</div>\n");
        }

        // Per-boxer details
        if !self.boxer_reports.is_empty() {
            output.push_str("<h2>Changes by Boxer</h2>\n");
            for boxer in &self.boxer_reports {
                output.push_str(&format!(
                    "<div class=\"boxer-section\">\n<h3>{}</h3>\n",
                    boxer.boxer_name
                ));

                // Palettes
                if !boxer.palettes.is_empty() {
                    output.push_str("<h4>Palettes</h4>\n<div class=\"palette-grid\">\n");
                    for palette in &boxer.palettes {
                        output.push_str(&format!(
                            "<div class=\"palette-item\">\n<h5>{}</h5>\n<p>{} colors changed</p>\n<div class=\"color-swatches\">\n",
                            palette.name,
                            palette.colors_changed.len()
                        ));
                        for idx in &palette.colors_changed {
                            if let Some(color) = palette.preview_after.get(*idx) {
                                output.push_str(&format!(
                                    "<div class=\"color-swatch\" style=\"background-color: {}\" title=\"Color {}\"></div>\n",
                                    color.to_hex(), idx
                                ));
                            }
                        }
                        output.push_str("</div>\n</div>\n");
                    }
                    output.push_str("</div>\n");
                }

                // Sprites
                if !boxer.sprites.is_empty() {
                    output.push_str("<h4>Sprites</h4>\n<div class=\"sprite-list\">\n");
                    for sprite in &boxer.sprites {
                        let size_class = if sprite.size_change > 0 {
                            "size-increase"
                        } else if sprite.size_change < 0 {
                            "size-decrease"
                        } else {
                            "size-same"
                        };
                        output.push_str(&format!(
                            "<div class=\"sprite-item\">\n<h5>{}</h5>\n<p>{} tiles modified ({} total)</p>\n<p class=\"{}\">Size change: {:+} bytes</p>\n</div>\n",
                            sprite.bin_name,
                            sprite.tiles_modified.len(),
                            sprite.total_tiles,
                            size_class,
                            sprite.size_change
                        ));
                    }
                    output.push_str("</div>\n");
                }

                output.push_str("</div>\n");
            }
        }

        output.push_str("</body>\n</html>");
        output
    }

    /// Render the detailed report to Markdown format
    pub fn render_markdown(&self) -> String {
        let mut output = String::new();

        output.push_str(&format!(
            "# Detailed Asset Report: {}\n\n",
            self.project_name
        ));
        output.push_str(&format!("**Version:** {}  \n", self.project_version));
        output.push_str(&format!("**Generated:** {}\n\n", self.generated_at));

        // Summary
        output.push_str("## Summary\n\n");
        output.push_str(&format!(
            "- **Boxers Modified:** {}\n",
            self.summary.total_boxers_modified
        ));
        output.push_str(&format!(
            "- **Palettes Changed:** {}\n",
            self.summary.total_palettes_changed
        ));
        output.push_str(&format!(
            "- **Sprites Edited:** {}\n",
            self.summary.total_sprites_edited
        ));
        output.push_str(&format!(
            "- **Animations Modified:** {}\n",
            self.summary.total_animations_modified
        ));
        output.push_str(&format!(
            "- **Total Changes:** {}\n\n",
            self.summary.total_changes
        ));

        // Binary changes
        output.push_str("## Binary Impact\n\n");
        output.push_str(&format!(
            "- **Total Bytes Changed:** {}\n",
            self.binary_changes.total_bytes_changed
        ));
        output.push_str(&format!(
            "- **Regions Affected:** {}\n",
            self.binary_changes.total_regions_affected
        ));
        output.push_str(&format!(
            "- **Estimated Patch Size:** {} bytes\n\n",
            self.binary_changes.estimated_patch_size
        ));

        // Shared assets
        if !self.shared_assets_touched.is_empty() {
            output.push_str("## Shared Assets Warning\n\n");
            for asset in &self.shared_assets_touched {
                output.push_str(&format!(
                    "### {} ({})\n\n",
                    asset.asset_name,
                    asset.warning_level.display_name()
                ));
                output.push_str(&format!("- **Location:** {}\n", asset.pc_offset));
                output.push_str(&format!("- **Type:** {}\n", asset.change_type));
                output.push_str(&format!(
                    "- **Shared with:** {}\n\n",
                    asset.shared_between.join(", ")
                ));
            }
        }

        // Per-boxer details
        if !self.boxer_reports.is_empty() {
            output.push_str("## Changes by Boxer\n\n");
            for boxer in &self.boxer_reports {
                output.push_str(&format!("### {}\n\n", boxer.boxer_name));

                if !boxer.palettes.is_empty() {
                    output.push_str("#### Palettes\n\n");
                    for palette in &boxer.palettes {
                        output.push_str(&format!(
                            "- **{}:** {} colors changed at {}\n",
                            palette.name,
                            palette.colors_changed.len(),
                            palette.pc_offset
                        ));
                    }
                    output.push('\n');
                }

                if !boxer.sprites.is_empty() {
                    output.push_str("#### Sprites\n\n");
                    for sprite in &boxer.sprites {
                        output.push_str(&format!(
                            "- **{}:** {} tiles modified, size change: {:+} bytes\n",
                            sprite.bin_name,
                            sprite.tiles_modified.len(),
                            sprite.size_change
                        ));
                    }
                    output.push('\n');
                }
            }
        }

        output
    }

    /// Render to JSON format
    pub fn render_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }

    /// Render to CSV format (simplified)
    pub fn render_csv(&self) -> String {
        let mut output = String::new();
        output.push_str("Boxer,Asset Type,Asset Name,PC Offset,Change Details\n");

        for boxer in &self.boxer_reports {
            for palette in &boxer.palettes {
                output.push_str(&format!(
                    "\"{}\",Palette,\"{}\",\"{}\",\"{} colors changed\"\n",
                    boxer.boxer_name,
                    palette.name,
                    palette.pc_offset,
                    palette.colors_changed.len()
                ));
            }
            for sprite in &boxer.sprites {
                output.push_str(&format!(
                    "\"{}\",Sprite,\"{}\",\"{}\",\"{} tiles modified, {:+} bytes\"\n",
                    boxer.boxer_name,
                    sprite.bin_name,
                    sprite.pc_offset,
                    sprite.tiles_modified.len(),
                    sprite.size_change
                ));
            }
        }

        output
    }

    fn detailed_report_css() -> &'static str {
        r#"
body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    max-width: 1200px;
    margin: 0 auto;
    padding: 2rem;
    line-height: 1.6;
    color: #333;
    background: #f5f5f5;
}
h1 { color: #2c3e50; border-bottom: 3px solid #3498db; padding-bottom: 0.5rem; }
h2 { color: #34495e; margin-top: 2rem; border-bottom: 1px solid #bdc3c7; padding-bottom: 0.3rem; }
h3 { color: #7f8c8d; margin-top: 1.5rem; }
.version { color: #7f8c8d; font-size: 0.9rem; }
.summary-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
    gap: 1rem;
    margin: 1rem 0;
}
.stat {
    background: white;
    padding: 1rem;
    border-radius: 8px;
    text-align: center;
}
.stat-value {
    display: block;
    font-size: 2rem;
    font-weight: bold;
    color: #3498db;
}
.stat-label {
    display: block;
    font-size: 0.85rem;
    color: #7f8c8d;
    margin-top: 0.25rem;
}
.shared-assets { margin: 1rem 0; }
.shared-asset {
    background: white;
    padding: 1rem;
    border-radius: 8px;
    margin-bottom: 0.5rem;
    border-left: 4px solid;
}
.shared-asset.safe { border-color: #4ade80; }
.shared-asset.caution { border-color: #fbbf24; }
.shared-asset.warning { border-color: #f87171; }
.shared-asset.critical { border-color: #dc2626; }
.shared-asset h4 { margin: 0 0 0.5rem 0; }
.shared-with { font-size: 0.85rem; color: #7f8c8d; }
.boxer-section {
    background: white;
    padding: 1.5rem;
    border-radius: 8px;
    margin: 1rem 0;
}
.palette-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 1rem;
}
.palette-item {
    border: 1px solid #e0e0e0;
    border-radius: 6px;
    padding: 0.75rem;
}
.palette-item h5 { margin: 0 0 0.5rem 0; font-size: 0.9rem; }
.color-swatches {
    display: flex;
    flex-wrap: wrap;
    gap: 2px;
    margin-top: 0.5rem;
}
.color-swatch {
    width: 20px;
    height: 20px;
    border-radius: 3px;
    border: 1px solid rgba(0,0,0,0.1);
}
.sprite-list { display: grid; gap: 0.5rem; }
.sprite-item {
    border: 1px solid #e0e0e0;
    border-radius: 6px;
    padding: 0.75rem;
}
.sprite-item h5 { margin: 0 0 0.25rem 0; }
.size-increase { color: #e74c3c; }
.size-decrease { color: #27ae60; }
.size-same { color: #7f8c8d; }
"#
    }
}
