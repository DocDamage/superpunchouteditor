//! Patch notes generation for Super Punch-Out!! mods
//!
//! This module provides comprehensive patch note generation with multiple output formats
//! and detailed asset change tracking.
//!
//! ## Usage
//!
//! ```rust
//! use project_core::patch_notes::{PatchNotes, OutputFormat};
//!
//! // Create new patch notes
//! let notes = PatchNotes::new(
//!     "My Mod".to_string(),
//!     "Author Name".to_string(),
//!     "1.0.0".to_string(),
//! );
//!
//! // Render to different formats
//! let markdown = notes.render(OutputFormat::Markdown);
//! let html = notes.render(OutputFormat::Html);
//! ```

pub mod adapter;
pub mod detailed_report;
pub mod renderer;
pub mod types;

pub use adapter::{RomAccess, RomAdapter};
pub use detailed_report::{
    AnimationChangeDetail, BinaryChangeSummary, BoxerAssetReport, ColorRgb, DetailedAssetReport,
    PaletteChangeDetail, SharedAssetReport, SpriteChangeDetail, StatChangeDetail, WarningLevel,
};
pub use types::{
    BoxerChangeSet, Change, ChangeSummary, Color, OutputFormat, SystemChange, SPO_ROM_SHA1,
};

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::{EditType, ProjectEdit, ProjectFile};

/// Complete patch notes structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchNotes {
    pub title: String,
    pub author: String,
    pub version: String,
    pub date: String,
    pub base_rom_sha1: String,
    pub summary: ChangeSummary,
    pub boxer_changes: Vec<BoxerChangeSet>,
    pub system_changes: Vec<SystemChange>,
}

impl PatchNotes {
    /// Create new empty patch notes
    pub fn new(title: String, author: String, version: String) -> Self {
        Self {
            title,
            author,
            version,
            date: Utc::now().format("%Y-%m-%d").to_string(),
            base_rom_sha1: SPO_ROM_SHA1.to_string(),
            summary: ChangeSummary::default(),
            boxer_changes: Vec::new(),
            system_changes: Vec::new(),
        }
    }

    /// Generate patch notes from a project file
    pub fn generate_from_project(project: &ProjectFile) -> Self {
        let mut notes = Self::new(
            project.metadata.name.clone(),
            project.metadata.author.clone().unwrap_or_default(),
            project.metadata.version.clone(),
        );
        notes.base_rom_sha1 = project.rom_base_sha1.clone();

        // Group edits by boxer/asset
        let mut boxer_changes: HashMap<String, Vec<Change>> = HashMap::new();
        let mut system_changes: Vec<SystemChange> = Vec::new();

        for edit in &project.edits {
            let change = Self::edit_to_change(edit);

            // Try to determine which boxer this edit belongs to
            let boxer_key = Self::extract_boxer_key(&edit.asset_id, &edit.pc_offset);

            if boxer_key.is_empty() {
                system_changes.push(SystemChange {
                    category: format!("{:?}", edit.edit_type),
                    description: edit.description.clone().unwrap_or_default(),
                });
            } else {
                boxer_changes
                    .entry(boxer_key)
                    .or_insert_with(Vec::new)
                    .push(change);
            }

            // Update summary counts
            match edit.edit_type {
                EditType::Palette => notes.summary.total_palettes_changed += 1,
                EditType::TileImport | EditType::SpriteBin => {
                    notes.summary.total_sprites_edited += 1
                }
                EditType::Script => notes.summary.total_animations_modified += 1,
                EditType::Other => notes.summary.total_headers_edited += 1,
            }
            notes.summary.total_changes += 1;
        }

        // Convert to BoxerChangeSet
        for (boxer_key, changes) in boxer_changes {
            notes.boxer_changes.push(BoxerChangeSet {
                boxer_name: Self::format_boxer_name(&boxer_key),
                boxer_key,
                changes,
            });
        }

        notes.summary.total_boxers_modified = notes.boxer_changes.len();
        notes.system_changes = system_changes;

        notes
    }

    /// Generate patch notes from pending writes
    pub fn generate_from_pending_writes(
        project: Option<&ProjectFile>,
        pending: &HashMap<String, Vec<u8>>,
        boxer_names: &HashMap<String, String>, // pc_offset -> boxer name
    ) -> Self {
        let mut notes = if let Some(proj) = project {
            Self::new(
                proj.metadata.name.clone(),
                proj.metadata.author.clone().unwrap_or_default(),
                proj.metadata.version.clone(),
            )
        } else {
            Self::new(
                "Untitled Mod".to_string(),
                String::new(),
                "1.0.0".to_string(),
            )
        };

        let mut boxer_changes: HashMap<String, Vec<Change>> = HashMap::new();

        for (pc_offset, _bytes) in pending {
            let boxer_key = boxer_names
                .get(pc_offset)
                .cloned()
                .unwrap_or_else(|| "unknown".to_string());

            // Determine change type from offset
            let change = if pc_offset.contains("palette") || pc_offset.contains("PAL") {
                notes.summary.total_palettes_changed += 1;
                Change::Palette {
                    name: format!("Palette at {}", pc_offset),
                    colors_changed: 1,
                    description: "Modified palette colors".to_string(),
                }
            } else {
                notes.summary.total_sprites_edited += 1;
                Change::Sprite {
                    bin_name: format!("Bin at {}", pc_offset),
                    tiles_modified: 1,
                    description: "Updated sprite graphics".to_string(),
                }
            };

            boxer_changes
                .entry(boxer_key.clone())
                .or_insert_with(Vec::new)
                .push(change);

            notes.summary.total_changes += 1;
        }

        // Convert to BoxerChangeSet
        for (boxer_key, changes) in boxer_changes {
            notes.boxer_changes.push(BoxerChangeSet {
                boxer_name: boxer_names
                    .get(&boxer_key)
                    .cloned()
                    .unwrap_or_else(|| Self::format_boxer_name(&boxer_key)),
                boxer_key,
                changes,
            });
        }

        notes.summary.total_boxers_modified = notes.boxer_changes.len();

        notes
    }

    /// Convert a project edit to a change entry
    fn edit_to_change(edit: &ProjectEdit) -> Change {
        let description = edit
            .description
            .clone()
            .unwrap_or_else(|| format!("{:?} modification", edit.edit_type));

        match edit.edit_type {
            EditType::Palette => Change::Palette {
                name: edit.asset_id.clone(),
                colors_changed: edit.size / 2, // 2 bytes per color
                description,
            },
            EditType::TileImport | EditType::SpriteBin => Change::Sprite {
                bin_name: edit.asset_id.clone(),
                tiles_modified: edit.size / 32, // 32 bytes per tile
                description,
            },
            EditType::Script => Change::Animation {
                name: edit.asset_id.clone(),
                frames_changed: 1,
                description,
            },
            EditType::Other => Change::Other { description },
        }
    }

    /// Extract boxer key from asset_id or pc_offset
    fn extract_boxer_key(asset_id: &str, pc_offset: &str) -> String {
        // Try to extract boxer from asset_id first (format: "boxer_name_asset_type")
        if let Some(idx) = asset_id.find('_') {
            return asset_id[..idx].to_string();
        }

        // Otherwise use pc_offset as identifier
        pc_offset.to_string()
    }

    /// Save patch notes to a file
    pub fn save(&self, path: &Path, format: OutputFormat) -> Result<(), String> {
        let content = self.render(format);
        fs::write(path, content).map_err(|e| format!("Failed to save patch notes: {}", e))
    }

    /// Generate a filename based on the title and format
    pub fn generate_filename(&self, format: OutputFormat) -> String {
        let safe_title = self
            .title
            .to_lowercase()
            .replace(' ', "_")
            .replace(|c: char| !c.is_alphanumeric() && c != '_', "");
        format!(
            "patch_notes_{}_{}.{}",
            safe_title,
            self.version,
            format.file_extension()
        )
    }
}

/// Get a change summary from pending writes
pub fn get_change_summary(
    pending: &HashMap<String, Vec<u8>>,
    boxer_names: &HashMap<String, String>,
) -> ChangeSummary {
    let mut summary = ChangeSummary::default();

    for (pc_offset, _bytes) in pending {
        summary.total_changes += 1;

        // Rough heuristic to determine change type
        if pc_offset.contains("palette") || pc_offset.contains("PAL") {
            summary.total_palettes_changed += 1;
        } else if pc_offset.contains("header") || pc_offset.contains("stat") {
            summary.total_headers_edited += 1;
        } else if _bytes.len() > 256 {
            summary.total_sprites_edited += 1;
        } else {
            summary.total_animations_modified += 1;
        }
    }

    summary.total_boxers_modified = boxer_names.len();
    summary
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_format_from_string() {
        assert_eq!(
            OutputFormat::from_string("markdown"),
            Some(OutputFormat::Markdown)
        );
        assert_eq!(
            OutputFormat::from_string("md"),
            Some(OutputFormat::Markdown)
        );
        assert_eq!(OutputFormat::from_string("html"), Some(OutputFormat::Html));
        assert_eq!(OutputFormat::from_string("json"), Some(OutputFormat::Json));
        assert_eq!(OutputFormat::from_string("unknown"), None);
    }

    #[test]
    fn test_format_boxer_name() {
        assert_eq!(PatchNotes::format_boxer_name("glass_joe"), "Glass Joe");
        assert_eq!(PatchNotes::format_boxer_name("mr_sandman"), "Mr Sandman");
    }

    #[test]
    fn test_render_markdown() {
        let notes = PatchNotes::new(
            "Test Mod".to_string(),
            "Test Author".to_string(),
            "1.0.0".to_string(),
        );

        let md = notes.render(OutputFormat::Markdown);
        assert!(md.contains("# Test Mod"));
        assert!(md.contains("Test Author"));
        assert!(md.contains("1.0.0"));
        assert!(md.contains(SPO_ROM_SHA1));
    }

    #[test]
    fn test_render_json() {
        let notes = PatchNotes::new(
            "Test Mod".to_string(),
            "Test Author".to_string(),
            "1.0.0".to_string(),
        );

        let json = notes.render(OutputFormat::Json);
        let parsed: PatchNotes = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.title, "Test Mod");
        assert_eq!(parsed.author, "Test Author");
    }

    #[test]
    fn test_warning_level() {
        assert_eq!(WarningLevel::Safe.as_str(), "safe");
        assert_eq!(WarningLevel::Warning.display_name(), "Warning");
        assert_eq!(WarningLevel::Critical.color(), "#dc2626");
    }

    #[test]
    fn test_color_rgb() {
        let color = ColorRgb::new(255, 128, 64);
        assert_eq!(color.to_hex(), "#FF8040");
    }
}
