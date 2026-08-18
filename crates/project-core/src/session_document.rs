//! Portable project-session format v2.
//!
//! A v2 project stores source identity plus the complete canonical edit journal. It never embeds the
//! copyrighted base ROM. The manifest is integrity protected and written atomically with a recovery
//! copy of the previous valid manifest.

use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use base64::{engine::general_purpose, Engine as _};
use chrono::{DateTime, Utc};
use rom_core::{BaseRom, EditJournal, RomSession};
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};

use crate::{DuplicatedBankInfo, ProjectError, ProjectFile, ProjectMetadata, ProjectThumbnail};

pub const PROJECT_V2_SCHEMA: u32 = 2;
pub const PROJECT_V2_FILENAME: &str = "project-v2.json";
pub const PROJECT_V2_RECOVERY_FILENAME: &str = "project-v2.recovery.json";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BaseRomIdentity {
    pub sha1: String,
    pub size: usize,
    pub region: Option<String>,
    pub display_filename: Option<String>,
}

impl BaseRomIdentity {
    pub fn from_base(base: &BaseRom, display_filename: Option<String>) -> Self {
        Self {
            sha1: base.sha1().to_string(),
            size: base.len(),
            region: base.region().map(|region| region.as_str().to_string()),
            display_filename,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ImportedAssetReference {
    pub id: String,
    pub relative_path: String,
    pub sha1: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub embedded_base64: Option<String>,
}

impl ImportedAssetReference {
    pub fn embedded(id: String, relative_path: String, bytes: &[u8]) -> Self {
        Self {
            id,
            relative_path,
            sha1: sha1_hex(bytes),
            embedded_base64: Some(general_purpose::STANDARD.encode(bytes)),
        }
    }

    pub fn verify_embedded(&self) -> Result<(), ProjectError> {
        if let Some(encoded) = &self.embedded_base64 {
            let bytes = general_purpose::STANDARD
                .decode(encoded)
                .map_err(|error| ProjectError::Validation(error.to_string()))?;
            if sha1_hex(&bytes) != self.sha1 {
                return Err(ProjectError::Validation(format!(
                    "embedded asset hash mismatch: {}",
                    self.id
                )));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectDocumentV2 {
    pub schema_version: u32,
    pub application_version: String,
    pub metadata: ProjectMetadata,
    pub base_rom: BaseRomIdentity,
    pub journal: EditJournal,
    pub settings: HashMap<String, serde_json::Value>,
    pub duplicated_banks: Vec<DuplicatedBankInfo>,
    pub imported_assets: Vec<ImportedAssetReference>,
    pub thumbnail: Option<ProjectThumbnail>,
    pub last_saved_revision: u64,
    pub expected_current_sha1: String,
    pub written_at: DateTime<Utc>,
}

impl ProjectDocumentV2 {
    pub fn from_session(
        application_version: impl Into<String>,
        metadata: ProjectMetadata,
        session: &RomSession,
        display_filename: Option<String>,
    ) -> Result<Self, ProjectError> {
        let materialized = session
            .materialize()
            .map_err(|error| ProjectError::Validation(error.to_string()))?;
        Ok(Self {
            schema_version: PROJECT_V2_SCHEMA,
            application_version: application_version.into(),
            metadata,
            base_rom: BaseRomIdentity::from_base(session.base(), display_filename),
            journal: session.journal().clone(),
            settings: HashMap::new(),
            duplicated_banks: Vec::new(),
            imported_assets: Vec::new(),
            thumbnail: None,
            last_saved_revision: session.journal().revision(),
            expected_current_sha1: materialized.current_sha1,
            written_at: Utc::now(),
        })
    }

    pub fn validate_against_base(&self, base: &BaseRom) -> Result<String, ProjectError> {
        if self.schema_version != PROJECT_V2_SCHEMA {
            return Err(ProjectError::Validation(format!(
                "unsupported project-v2 schema {}",
                self.schema_version
            )));
        }
        if self.base_rom.sha1 != base.sha1() || self.base_rom.size != base.len() {
            return Err(ProjectError::Sha1Mismatch {
                expected: self.base_rom.sha1.clone(),
                actual: base.sha1().to_string(),
            });
        }
        for asset in &self.imported_assets {
            asset.verify_embedded()?;
        }
        self.journal
            .validate_against(base)
            .map_err(|error| ProjectError::Validation(error.to_string()))?;
        let bytes = self
            .journal
            .materialize(base)
            .map_err(|error| ProjectError::Validation(error.to_string()))?;
        let actual = sha1_hex(&bytes);
        if actual != self.expected_current_sha1 {
            return Err(ProjectError::Sha1Mismatch {
                expected: self.expected_current_sha1.clone(),
                actual,
            });
        }
        Ok(self.expected_current_sha1.clone())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProjectEnvelopeV2 {
    document: ProjectDocumentV2,
    integrity_sha1: String,
}

fn document_integrity(document: &ProjectDocumentV2) -> Result<String, ProjectError> {
    // Convert through `Value` so object keys (including HashMap-backed settings) are emitted in
    // deterministic map order before hashing. Integrity must survive deserialize/serialize cycles.
    let canonical_value = serde_json::to_value(document)?;
    let canonical = serde_json::to_vec(&canonical_value)?;
    Ok(sha1_hex(&canonical))
}

fn sha1_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha1::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

pub fn save_project_v2(directory: &Path, document: &ProjectDocumentV2) -> Result<(), ProjectError> {
    fs::create_dir_all(directory)?;
    fs::create_dir_all(directory.join("assets"))?;
    fs::create_dir_all(directory.join("patches"))?;

    for asset in &document.imported_assets {
        asset.verify_embedded()?;
        let logical = Path::new(&asset.relative_path);
        if logical.is_absolute()
            || logical
                .components()
                .any(|component| matches!(component, std::path::Component::ParentDir))
        {
            return Err(ProjectError::Validation(format!(
                "unsafe project asset path: {}",
                asset.relative_path
            )));
        }
    }

    let envelope = ProjectEnvelopeV2 {
        document: document.clone(),
        integrity_sha1: document_integrity(document)?,
    };
    let encoded = serde_json::to_vec_pretty(&envelope)?;
    let destination = directory.join(PROJECT_V2_FILENAME);
    let recovery = directory.join(PROJECT_V2_RECOVERY_FILENAME);

    if destination.exists() {
        // Only preserve the previous file as recovery if it is still parseable and integrity-valid.
        if load_project_v2_file(&destination).is_ok() {
            fs::copy(&destination, &recovery)?;
        }
    }

    let mut temp = tempfile::NamedTempFile::new_in(directory)?;
    temp.write_all(&encoded)?;
    temp.as_file().sync_all()?;
    let verify = fs::read(temp.path())?;
    let parsed: ProjectEnvelopeV2 = serde_json::from_slice(&verify)?;
    if parsed.integrity_sha1 != document_integrity(&parsed.document)? {
        return Err(ProjectError::Validation(
            "temporary project manifest failed integrity verification".to_string(),
        ));
    }
    temp.persist(&destination)
        .map_err(|error| ProjectError::Io(error.error))?;
    Ok(())
}

pub fn load_project_v2(directory: &Path) -> Result<ProjectDocumentV2, ProjectError> {
    load_project_v2_file(&directory.join(PROJECT_V2_FILENAME))
}

pub fn load_project_v2_file(path: &Path) -> Result<ProjectDocumentV2, ProjectError> {
    if !path.exists() {
        return Err(ProjectError::NotFound(path.to_path_buf()));
    }
    let bytes = fs::read(path)?;
    let envelope: ProjectEnvelopeV2 = serde_json::from_slice(&bytes)?;
    let expected = document_integrity(&envelope.document)?;
    if expected != envelope.integrity_sha1 {
        return Err(ProjectError::Validation(
            "project-v2 integrity hash mismatch".to_string(),
        ));
    }
    Ok(envelope.document)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct V1MigrationAssessment {
    pub can_reconstruct_edits: bool,
    pub metadata_only_import_available: bool,
    pub edit_count: usize,
    pub explanation: String,
}

pub fn assess_v1_migration(file: &ProjectFile) -> V1MigrationAssessment {
    if file.edits.is_empty() {
        V1MigrationAssessment {
            can_reconstruct_edits: true,
            metadata_only_import_available: true,
            edit_count: 0,
            explanation: "The v1 project contains no edit records; its metadata can migrate to an empty v2 journal."
                .to_string(),
        }
    } else {
        V1MigrationAssessment {
            can_reconstruct_edits: false,
            metadata_only_import_available: true,
            edit_count: file.edits.len(),
            explanation: format!(
                "The v1 project contains {} edit metadata record(s) but no replacement byte payloads; edits cannot be reconstructed safely. Metadata-only import is available.",
                file.edits.len()
            ),
        }
    }
}

pub fn recovery_manifest_path(directory: &Path) -> PathBuf {
    directory.join(PROJECT_V2_RECOVERY_FILENAME)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rom_core::{EditRequest, Rom};

    fn edited_session() -> RomSession {
        let rom = Rom::new(vec![0, 1, 2, 3, 4]);
        let mut session = RomSession::from_rom(&rom, Some("synthetic.sfc".to_string()));
        session
            .commit(
                "edit",
                vec![EditRequest::WriteBytes {
                    offset: 2,
                    after: vec![9, 8],
                    asset_id: None,
                    description: None,
                }],
            )
            .unwrap();
        session
    }

    #[test]
    fn project_round_trip_restores_identical_hash() {
        let directory = tempfile::tempdir().unwrap();
        let session = edited_session();
        let document = ProjectDocumentV2::from_session(
            "2.0.0",
            ProjectMetadata::default(),
            &session,
            Some("synthetic.sfc".to_string()),
        )
        .unwrap();
        save_project_v2(directory.path(), &document).unwrap();
        let restored = load_project_v2(directory.path()).unwrap();
        assert_eq!(
            restored.validate_against_base(session.base()).unwrap(),
            document.expected_current_sha1
        );
    }

    #[test]
    fn project_manifest_never_contains_base_rom_bytes() {
        let directory = tempfile::tempdir().unwrap();
        let session = edited_session();
        let document =
            ProjectDocumentV2::from_session("2.0.0", ProjectMetadata::default(), &session, None)
                .unwrap();
        save_project_v2(directory.path(), &document).unwrap();
        let text = fs::read_to_string(directory.path().join(PROJECT_V2_FILENAME)).unwrap();
        assert!(!text.contains("AAECAwQ="));
    }

    #[test]
    fn corrupt_integrity_is_rejected() {
        let directory = tempfile::tempdir().unwrap();
        let session = edited_session();
        let document =
            ProjectDocumentV2::from_session("2.0.0", ProjectMetadata::default(), &session, None)
                .unwrap();
        save_project_v2(directory.path(), &document).unwrap();
        let path = directory.path().join(PROJECT_V2_FILENAME);
        let mut text = fs::read_to_string(&path).unwrap();
        text = text.replace("Untitled Project", "Tampered Project");
        fs::write(&path, text).unwrap();
        assert!(load_project_v2(directory.path()).is_err());
    }
}
