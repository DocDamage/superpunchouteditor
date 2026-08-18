//! Safe output preflight for ROM save and patch export.

use std::path::Path;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::app_state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputPreflight {
    pub source_sha1: String,
    pub current_sha1: String,
    pub region: Option<String>,
    pub revision: u64,
    pub transaction_count: usize,
    pub changed_byte_count: usize,
    pub changed_ranges: Vec<PreflightRange>,
    pub destination_path: String,
    pub destination_exists: bool,
    pub backup_will_be_created: bool,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreflightRange {
    pub start: usize,
    pub end: usize,
}

#[tauri::command]
pub fn get_output_preflight(
    state: State<AppState>,
    destination_path: String,
) -> Result<OutputPreflight, String> {
    if destination_path.trim().is_empty() {
        return Err("Destination path cannot be empty".to_string());
    }
    let destination = Path::new(&destination_path);
    let parent = destination.parent().unwrap_or_else(|| Path::new("."));
    if !parent.exists() {
        return Err(format!(
            "Destination directory does not exist: {}",
            parent.display()
        ));
    }

    let materialized = state.materialize_current_rom()?;
    let destination_exists = destination.exists();
    let mut warnings = Vec::new();
    if materialized.transaction_count == 0 {
        warnings.push(
            "No journal transactions are active; output will match the base ROM.".to_string(),
        );
    }
    if materialized.bytes.len()
        != state
            .rom_session
            .lock()
            .as_ref()
            .ok_or("No ROM loaded")?
            .base()
            .len()
    {
        warnings.push("The working ROM size differs from the base ROM; IPS export may be unavailable and BPS should be used.".to_string());
    }

    Ok(OutputPreflight {
        source_sha1: materialized.base_sha1,
        current_sha1: materialized.current_sha1,
        region: materialized
            .region
            .map(|region| region.as_str().to_string()),
        revision: materialized.revision,
        transaction_count: materialized.transaction_count,
        changed_byte_count: materialized.changed_byte_count,
        changed_ranges: materialized
            .change_ranges
            .into_iter()
            .map(|range| PreflightRange {
                start: range.start,
                end: range.end,
            })
            .collect(),
        destination_path,
        destination_exists,
        backup_will_be_created: destination_exists,
        warnings,
    })
}
