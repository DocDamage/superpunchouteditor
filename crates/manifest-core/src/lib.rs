use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use thiserror::Error;

pub mod comparison;
pub use comparison::*;

#[derive(Error, Debug)]
pub enum ManifestError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SourceRomInfo {
    pub filename: String,
    pub sha1: String,
    pub size_bytes: usize,
    pub format: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AssetFile {
    pub file: String,
    pub filename: String,
    pub category: String,
    pub subtype: String,
    pub size: usize,
    pub start_snes: String,
    pub end_snes: String,
    pub start_pc: String,
    pub end_pc: String,
    pub shared_with: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BoxerRecord {
    /// Boxer display name.
    /// Older manifests (JPN, PAL) may use the legacy field name `fighter`;
    /// both forms are accepted during deserialization.
    #[serde(alias = "fighter")]
    pub name: String,
    /// Boxer key (unique identifier)
    pub key: String,
    pub reference_sheet: String,
    pub palette_files: Vec<AssetFile>,
    pub icon_files: Vec<AssetFile>,
    pub portrait_files: Vec<AssetFile>,
    pub large_portrait_files: Vec<AssetFile>,
    pub unique_sprite_bins: Vec<AssetFile>,
    pub shared_sprite_bins: Vec<AssetFile>,
    pub other_files: Vec<AssetFile>,
}

impl BoxerRecord {
    /// Deprecated: Use `name` field directly
    #[deprecated(since = "0.1.0", note = "Use name field instead")]
    pub fn fighter(&self) -> &String {
        &self.name
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Manifest {
    pub source_rom: SourceRomInfo,
    pub asset_counts: HashMap<String, usize>,
    pub fighters: HashMap<String, BoxerRecord>,
}

impl Manifest {
    /// Construct an empty placeholder manifest.
    ///
    /// Use this when no ROM is loaded yet. The real manifest is loaded by
    /// `open_rom` once the user selects a ROM file.
    pub fn empty() -> Self {
        Self {
            source_rom: SourceRomInfo {
                filename: String::new(),
                sha1: String::new(),
                size_bytes: 0,
                format: String::new(),
            },
            asset_counts: HashMap::new(),
            fighters: HashMap::new(),
        }
    }

    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, ManifestError> {
        let content = fs::read_to_string(path)?;
        let manifest: Manifest = serde_json::from_str(&content)?;
        Ok(manifest)
    }

    /// Get a boxer by name
    pub fn get_boxer(&self, name: &str) -> Option<&BoxerRecord> {
        self.fighters.get(name)
    }

    /// List all boxers
    pub fn list_boxers(&self) -> Vec<&BoxerRecord> {
        self.fighters.values().collect()
    }

    /// Deprecated: Use `get_boxer` instead
    #[deprecated(since = "0.1.0", note = "Use get_boxer instead")]
    pub fn get_fighter(&self, name: &str) -> Option<&BoxerRecord> {
        self.get_boxer(name)
    }

    /// Deprecated: Use `list_boxers` instead
    #[deprecated(since = "0.1.0", note = "Use list_boxers instead")]
    pub fn list_fighters(&self) -> Vec<&BoxerRecord> {
        self.list_boxers()
    }
}
