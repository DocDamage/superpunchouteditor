use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub mod bank_duplication;
pub use bank_duplication::*;

pub mod patch_notes;
pub use patch_notes::*;

pub mod tools;
pub use tools::*;

/// Current project file format version
pub const PROJECT_VERSION: u32 = 1;

/// File extension for Super Punch-Out project files
pub const PROJECT_EXTENSION: &str = "spo";

/// Error type for project operations
#[derive(Debug, thiserror::Error)]
pub enum ProjectError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Project validation failed: {0}")]
    Validation(String),
    #[error("SHA1 mismatch: expected {expected}, got {actual}")]
    Sha1Mismatch { expected: String, actual: String },
    #[error("Project directory structure invalid")]
    InvalidStructure,
    #[error("Project not found: {0}")]
    NotFound(PathBuf),
}

/// Type of edit performed
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EditType {
    Palette,
    TileImport,
    SpriteBin,
    Script,
    Other,
}

/// Represents a single edit made to the ROM
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectEdit {
    pub asset_id: String,
    #[serde(rename = "type")]
    pub edit_type: EditType,
    pub description: Option<String>,
    pub original_hash: String,
    pub edited_hash: String,
    pub pc_offset: String,
    pub size: usize,
    pub timestamp: DateTime<Utc>,
    /// Optional path to exported asset file (relative to assets/)
    pub asset_path: Option<String>,
}

/// Project metadata (user-facing information)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectMetadata {
    pub name: String,
    pub author: Option<String>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub version: String,
}

impl Default for ProjectMetadata {
    fn default() -> Self {
        Self {
            name: "Untitled Project".to_string(),
            author: None,
            description: None,
            created_at: Utc::now(),
            modified_at: Utc::now(),
            version: "0.1.0".to_string(),
        }
    }
}

/// Asset entry in the project
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectAsset {
    pub id: String,
    pub name: String,
    pub asset_type: String,
    pub source_pc_offset: String,
    pub filename: String,
    pub exported_at: DateTime<Utc>,
}

/// Information about a duplicated bank stored in the project file
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DuplicatedBankInfo {
    /// The boxer that owns this duplicated bank
    pub boxer_key: String,
    /// Original PC offset of the shared bank
    pub original_pc_offset: String,
    /// New PC offset where the duplicate is stored
    pub new_pc_offset: String,
    /// Size of the bank
    pub size: usize,
    /// Original filename
    pub filename: String,
    /// Whether it's compressed
    pub compressed: bool,
    /// When it was created
    pub created_at: DateTime<Utc>,
    /// Unique ID for this duplication
    pub id: String,
}

/// Project thumbnail metadata and data
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectThumbnail {
    /// PNG data as base64 string
    pub data_base64: String,
    /// Width of the thumbnail in pixels
    pub width: u32,
    /// Height of the thumbnail in pixels
    pub height: u32,
    /// ISO 8601 timestamp when the thumbnail was captured
    pub captured_at: DateTime<Utc>,
    /// Which view/tab was active when captured (e.g., "editor", "viewer", "animation")
    pub captured_view: String,
}

impl ProjectThumbnail {
    /// Create a new thumbnail from PNG bytes
    pub fn from_png_bytes(
        png_bytes: &[u8],
        width: u32,
        height: u32,
        captured_view: String,
    ) -> Self {
        use base64::{engine::general_purpose, Engine as _};
        Self {
            data_base64: general_purpose::STANDARD.encode(png_bytes),
            width,
            height,
            captured_at: Utc::now(),
            captured_view,
        }
    }

    /// Get the PNG bytes from base64
    pub fn to_png_bytes(&self) -> Result<Vec<u8>, base64::DecodeError> {
        use base64::{engine::general_purpose, Engine as _};
        general_purpose::STANDARD.decode(&self.data_base64)
    }

    /// Get a data URL for use in HTML img tags
    pub fn to_data_url(&self) -> String {
        format!("data:image/png;base64,{}", self.data_base64)
    }
}

/// Main project file structure (saved as project.json)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectFile {
    pub version: u32,
    pub rom_base_sha1: String,
    pub manifest_version: String,
    pub metadata: ProjectMetadata,
    pub edits: Vec<ProjectEdit>,
    pub assets: Vec<ProjectAsset>,
    pub settings: HashMap<String, serde_json::Value>,
    /// Bank duplications created in this project
    pub duplicated_banks: Vec<DuplicatedBankInfo>,
    /// Optional thumbnail for visual project identification
    pub thumbnail: Option<ProjectThumbnail>,
    /// Source ROM region (usa, jpn, pal) for multi-region support
    #[serde(default = "default_region")]
    pub source_region: Option<String>,
}

fn default_region() -> Option<String> {
    Some("usa".to_string())
}

impl ProjectFile {
    /// Create a new project file
    pub fn new(rom_sha1: &str, manifest_version: &str, metadata: ProjectMetadata) -> Self {
        Self {
            version: PROJECT_VERSION,
            rom_base_sha1: rom_sha1.to_string(),
            manifest_version: manifest_version.to_string(),
            metadata,
            edits: Vec::new(),
            assets: Vec::new(),
            settings: HashMap::new(),
            duplicated_banks: Vec::new(),
            thumbnail: None,
            source_region: Some("usa".to_string()),
        }
    }

    /// Add a duplicated bank to the project
    pub fn add_duplicated_bank(&mut self, info: DuplicatedBankInfo) {
        // Remove any existing duplication for this boxer + original offset combo
        self.duplicated_banks.retain(|d| {
            !(d.boxer_key == info.boxer_key && d.original_pc_offset == info.original_pc_offset)
        });
        self.duplicated_banks.push(info);
        self.metadata.modified_at = Utc::now();
    }

    /// Get all duplicated banks for a specific boxer
    pub fn get_boxer_duplicated_banks(&self, boxer_key: &str) -> Vec<&DuplicatedBankInfo> {
        self.duplicated_banks
            .iter()
            .filter(|d| d.boxer_key == boxer_key)
            .collect()
    }

    /// Check if a boxer has a duplicated version of a bank
    pub fn has_duplicated_bank(&self, boxer_key: &str, original_pc_offset: &str) -> bool {
        self.duplicated_banks
            .iter()
            .any(|d| d.boxer_key == boxer_key && d.original_pc_offset == original_pc_offset)
    }

    /// Get the duplicated bank info for a specific boxer and original offset
    pub fn get_duplicated_bank(
        &self,
        boxer_key: &str,
        original_pc_offset: &str,
    ) -> Option<&DuplicatedBankInfo> {
        self.duplicated_banks
            .iter()
            .find(|d| d.boxer_key == boxer_key && d.original_pc_offset == original_pc_offset)
    }

    /// Remove a duplicated bank from the project
    pub fn remove_duplicated_bank(&mut self, boxer_key: &str, original_pc_offset: &str) -> bool {
        let original_len = self.duplicated_banks.len();
        self.duplicated_banks
            .retain(|d| !(d.boxer_key == boxer_key && d.original_pc_offset == original_pc_offset));
        let removed = self.duplicated_banks.len() < original_len;
        if removed {
            self.metadata.modified_at = Utc::now();
        }
        removed
    }

    /// Create a new project with default metadata
    pub fn new_with_defaults(rom_sha1: &str, manifest_version: &str) -> Self {
        Self::new(rom_sha1, manifest_version, ProjectMetadata::default())
    }

    /// Add an edit to the project
    pub fn add_edit(&mut self, edit: ProjectEdit) {
        self.edits.push(edit);
        self.metadata.modified_at = Utc::now();
    }

    /// Add an asset to the project
    pub fn add_asset(&mut self, asset: ProjectAsset) {
        self.assets.push(asset);
        self.metadata.modified_at = Utc::now();
    }

    /// Save the project file to a directory
    pub fn save_to_directory(&self, dir_path: &Path) -> Result<(), ProjectError> {
        fs::create_dir_all(dir_path)?;

        let project_file_path = dir_path.join("project.json");
        let json = serde_json::to_string_pretty(self)?;
        fs::write(&project_file_path, json)?;

        // Ensure assets directory exists
        let assets_dir = dir_path.join("assets");
        fs::create_dir_all(&assets_dir)?;

        // Ensure patches directory exists
        let patches_dir = dir_path.join("patches");
        fs::create_dir_all(&patches_dir)?;

        Ok(())
    }

    /// Load a project file from a directory
    pub fn load_from_directory(dir_path: &Path) -> Result<Self, ProjectError> {
        let project_file_path = dir_path.join("project.json");

        if !project_file_path.exists() {
            return Err(ProjectError::NotFound(project_file_path));
        }

        let json = fs::read_to_string(&project_file_path)?;
        let project: ProjectFile = serde_json::from_str(&json)?;

        // Validate version
        if project.version != PROJECT_VERSION {
            return Err(ProjectError::Validation(format!(
                "Unsupported project version: {}",
                project.version
            )));
        }

        Ok(project)
    }

    /// Validate that the project matches the given ROM SHA1
    pub fn validate_rom_sha1(&self, rom_sha1: &str) -> Result<(), ProjectError> {
        if self.rom_base_sha1 != rom_sha1 {
            Err(ProjectError::Sha1Mismatch {
                expected: self.rom_base_sha1.clone(),
                actual: rom_sha1.to_string(),
            })
        } else {
            Ok(())
        }
    }

    /// Get the full path for an asset
    pub fn get_asset_path(&self, project_dir: &Path, asset_id: &str) -> Option<PathBuf> {
        self.assets
            .iter()
            .find(|a| a.id == asset_id)
            .map(|asset| project_dir.join("assets").join(&asset.filename))
    }
}

/// Full project including directory path and file
#[derive(Debug, Clone)]
pub struct Project {
    /// Path to the project directory (the .spo folder)
    pub path: PathBuf,
    /// The project file data
    pub file: ProjectFile,
}

impl Project {
    /// Create a new project at the given path
    pub fn create(
        project_path: &Path,
        rom_sha1: &str,
        manifest_version: &str,
        metadata: ProjectMetadata,
    ) -> Result<Self, ProjectError> {
        let file = ProjectFile::new(rom_sha1, manifest_version, metadata);
        file.save_to_directory(project_path)?;

        Ok(Self {
            path: project_path.to_path_buf(),
            file,
        })
    }

    /// Load an existing project from a directory
    pub fn load(project_path: &Path) -> Result<Self, ProjectError> {
        let file = ProjectFile::load_from_directory(project_path)?;

        Ok(Self {
            path: project_path.to_path_buf(),
            file,
        })
    }

    /// Save the project to its directory
    pub fn save(&self) -> Result<(), ProjectError> {
        self.file.save_to_directory(&self.path)
    }

    /// Save the project to a new directory (Save As functionality)
    pub fn save_as(&self, new_path: &Path) -> Result<(), ProjectError> {
        // Create new directory structure if it doesn't exist
        if !new_path.exists() {
            fs::create_dir_all(new_path)?;
        }

        // Save the project file
        self.file.save_to_directory(new_path)?;

        // Copy assets directory if it exists
        let old_assets = self.assets_dir();
        let new_assets = new_path.join("assets");
        if old_assets.exists() && !new_assets.exists() {
            fs::create_dir_all(&new_assets)?;
            for entry in fs::read_dir(&old_assets)? {
                let entry = entry?;
                let src = entry.path();
                let dst = new_assets.join(entry.file_name());
                if src.is_file() {
                    fs::copy(&src, &dst)?;
                }
            }
        }

        // Copy patches directory if it exists
        let old_patches = self.patches_dir();
        let new_patches = new_path.join("patches");
        if old_patches.exists() && !new_patches.exists() {
            fs::create_dir_all(&new_patches)?;
            for entry in fs::read_dir(&old_patches)? {
                let entry = entry?;
                let src = entry.path();
                let dst = new_patches.join(entry.file_name());
                if src.is_file() {
                    fs::copy(&src, &dst)?;
                }
            }
        }

        Ok(())
    }

    /// Save the project with updated metadata
    pub fn save_with_metadata(&mut self, metadata: ProjectMetadata) -> Result<(), ProjectError> {
        self.file.metadata = metadata;
        self.file.metadata.modified_at = Utc::now();
        self.save()
    }

    /// Validate the project against a ROM SHA1
    pub fn validate_rom(&self, rom_sha1: &str) -> Result<(), ProjectError> {
        self.file.validate_rom_sha1(rom_sha1)
    }

    /// Get the assets directory path
    pub fn assets_dir(&self) -> PathBuf {
        self.path.join("assets")
    }

    /// Get the patches directory path
    pub fn patches_dir(&self) -> PathBuf {
        self.path.join("patches")
    }

    /// Save an asset file to the project's assets directory
    pub fn save_asset(
        &mut self,
        asset_id: &str,
        name: &str,
        asset_type: &str,
        source_pc_offset: &str,
        data: &[u8],
        extension: &str,
    ) -> Result<PathBuf, ProjectError> {
        let filename = format!(
            "{}_{}.{}",
            asset_id,
            name.to_lowercase().replace(' ', "_"),
            extension
        );
        let asset_path = self.assets_dir().join(&filename);

        fs::write(&asset_path, data)?;

        let asset = ProjectAsset {
            id: asset_id.to_string(),
            name: name.to_string(),
            asset_type: asset_type.to_string(),
            source_pc_offset: source_pc_offset.to_string(),
            filename,
            exported_at: Utc::now(),
        };

        // Remove existing asset with same ID if present
        self.file.assets.retain(|a| a.id != asset_id);
        self.file.add_asset(asset);

        Ok(asset_path)
    }

    /// Export an IPS patch to the patches directory
    pub fn export_patch(&self, name: &str, data: &[u8]) -> Result<PathBuf, ProjectError> {
        let filename = format!("{}.ips", name);
        let patch_path = self.patches_dir().join(&filename);
        fs::write(&patch_path, data)?;
        Ok(patch_path)
    }
}

/// Recent projects list for quick access
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct RecentProjects {
    pub projects: Vec<RecentProjectEntry>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RecentProjectEntry {
    pub path: PathBuf,
    pub name: String,
    pub last_opened: DateTime<Utc>,
    pub rom_sha1: String,
}

impl RecentProjects {
    pub fn load(config_path: &Path) -> Result<Self, ProjectError> {
        if !config_path.exists() {
            return Ok(Self::default());
        }
        let json = fs::read_to_string(config_path)?;
        Ok(serde_json::from_str(&json)?)
    }

    pub fn save(&self, config_path: &Path) -> Result<(), ProjectError> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(config_path, json)?;
        Ok(())
    }

    pub fn add(&mut self, path: PathBuf, name: String, rom_sha1: String) {
        // Remove existing entry if present
        self.projects.retain(|p| p.path != path);

        // Add to front
        self.projects.insert(
            0,
            RecentProjectEntry {
                path,
                name,
                last_opened: Utc::now(),
                rom_sha1,
            },
        );

        // Keep only last 10
        self.projects.truncate(10);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_project_file_serialization() {
        let metadata = ProjectMetadata {
            name: "Test Project".to_string(),
            author: Some("Test Author".to_string()),
            description: Some("A test project".to_string()),
            created_at: Utc::now(),
            modified_at: Utc::now(),
            version: "1.0.0".to_string(),
        };

        let project = ProjectFile::new("abc123", "1.0", metadata);

        let json = serde_json::to_string_pretty(&project).unwrap();
        let deserialized: ProjectFile = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.version, PROJECT_VERSION);
        assert_eq!(deserialized.rom_base_sha1, "abc123");
        assert_eq!(deserialized.metadata.name, "Test Project");
    }

    #[test]
    fn test_project_save_load() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().join("test_project.spo");

        let metadata = ProjectMetadata {
            name: "Test Project".to_string(),
            author: Some("Test Author".to_string()),
            description: None,
            created_at: Utc::now(),
            modified_at: Utc::now(),
            version: "0.1.0".to_string(),
        };

        let project = Project::create(&project_path, "sha1_test", "1.0", metadata.clone()).unwrap();

        // Load it back
        let loaded = Project::load(&project_path).unwrap();

        assert_eq!(loaded.file.metadata.name, metadata.name);
        assert_eq!(loaded.file.rom_base_sha1, "sha1_test");
    }

    #[test]
    fn test_sha1_validation() {
        let metadata = ProjectMetadata::default();
        let project = ProjectFile::new("correct_sha1", "1.0", metadata);

        assert!(project.validate_rom_sha1("correct_sha1").is_ok());
        assert!(project.validate_rom_sha1("wrong_sha1").is_err());
    }
}
