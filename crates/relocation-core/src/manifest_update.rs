use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Errors that can occur during manifest updates
#[derive(Error, Debug)]
pub enum ManifestUpdateError {
    #[error("Asset not found: {0}")]
    AssetNotFound(String),
    #[error("Invalid address format: {0}")]
    InvalidAddressFormat(String),
    #[error("Multiple assets at address: {0}")]
    MultipleAssetsAtAddress(String),
    #[error("Update would create conflicts: {0}")]
    ConflictError(String),
}

/// Represents an address change for an asset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressChange {
    /// The asset file identifier
    pub file: String,
    /// Old PC offset (start)
    pub old_start_pc: usize,
    /// Old end PC offset
    pub old_end_pc: usize,
    /// New PC offset (start)
    pub new_start_pc: usize,
    /// New end PC offset
    pub new_end_pc: usize,
    /// New size (may differ from old if data was modified)
    pub new_size: usize,
    /// Whether the SNES addresses should be updated too
    pub update_snes_addresses: bool,
}

impl AddressChange {
    /// Create a new address change with calculated end addresses
    pub fn new(
        file: String,
        old_start_pc: usize,
        old_size: usize,
        new_start_pc: usize,
        new_size: usize,
    ) -> Self {
        Self {
            file,
            old_start_pc,
            old_end_pc: old_start_pc + old_size - 1,
            new_start_pc,
            new_end_pc: new_start_pc + new_size - 1,
            new_size,
            update_snes_addresses: true,
        }
    }

    /// Calculate the new SNES LoROM start address
    pub fn new_start_snes(&self) -> String {
        format!("0x{:06X}", pc_to_snes(self.new_start_pc))
    }

    /// Calculate the new SNES LoROM end address
    pub fn new_end_snes(&self) -> String {
        format!("0x{:06X}", pc_to_snes(self.new_end_pc))
    }
}

/// Convert PC offset to SNES LoROM address
fn pc_to_snes(pc: usize) -> u32 {
    let bank = ((pc / 0x8000) | 0x80) & 0xFF;
    let addr = (pc & 0x7FFF) | 0x8000;
    ((bank as u32) << 16) | (addr as u32)
}

/// Result of updating a manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestUpdateResult {
    pub success: bool,
    pub updated_files: Vec<String>,
    pub updated_fighters: Vec<String>,
    pub warnings: Vec<String>,
    /// The updated manifest JSON (if serialization was successful)
    pub updated_manifest_json: Option<String>,
}

/// Update an asset's address in a manifest
///
/// This function takes a manifest (as a serde_json::Value), finds the specified asset,
/// and updates its address fields to reflect the new location.
pub fn update_manifest_address(
    manifest: &mut serde_json::Value,
    boxer_key: &str,
    asset_file: &str,
    new_pc_offset: usize,
    new_size: Option<usize>,
) -> Result<ManifestUpdateResult, ManifestUpdateError> {
    let mut result = ManifestUpdateResult {
        success: false,
        updated_files: Vec::new(),
        updated_fighters: Vec::new(),
        warnings: Vec::new(),
        updated_manifest_json: None,
    };

    // Find the fighter
    let fighters = manifest
        .get_mut("fighters")
        .and_then(|f| f.as_object_mut())
        .ok_or_else(|| {
            ManifestUpdateError::AssetNotFound("fighters object not found".to_string())
        })?;

    // Find the boxer by key
    let mut boxer_found = false;
    for (fighter_name, boxer_data) in fighters.iter_mut() {
        let key = boxer_data.get("key").and_then(|k| k.as_str()).unwrap_or("");

        if key == boxer_key || fighter_name == boxer_key {
            boxer_found = true;

            // Search through all asset arrays
            let asset_arrays = [
                "palette_files",
                "icon_files",
                "portrait_files",
                "large_portrait_files",
                "unique_sprite_bins",
                "shared_sprite_bins",
                "other_files",
            ];

            let mut asset_found = false;

            for array_name in &asset_arrays {
                if let Some(array) = boxer_data
                    .get_mut(array_name)
                    .and_then(|a| a.as_array_mut())
                {
                    for asset in array.iter_mut() {
                        if let Some(file) = asset.get("file").and_then(|f| f.as_str()) {
                            if file == asset_file {
                                asset_found = true;

                                // Clone file before mutating asset
                                let file_string = file.to_string();

                                // Get old address for change tracking
                                let _old_start_pc = asset
                                    .get("start_pc")
                                    .and_then(|s| parse_hex_string(s.as_str()?))
                                    .unwrap_or(0);

                                let old_size =
                                    asset.get("size").and_then(|s| s.as_u64()).unwrap_or(0)
                                        as usize;

                                // Calculate new size
                                let size = new_size.unwrap_or(old_size);
                                let new_end = new_pc_offset + size - 1;

                                // Update PC addresses
                                asset["start_pc"] =
                                    serde_json::json!(format!("0x{:X}", new_pc_offset));
                                asset["end_pc"] = serde_json::json!(format!("0x{:X}", new_end));
                                asset["size"] = serde_json::json!(size);

                                // Update SNES addresses
                                asset["start_snes"] = serde_json::json!(format!(
                                    "0x{:06X}",
                                    pc_to_snes(new_pc_offset)
                                ));
                                asset["end_snes"] =
                                    serde_json::json!(format!("0x{:06X}", pc_to_snes(new_end)));

                                result.updated_files.push(file_string);
                                if !result.updated_fighters.contains(fighter_name) {
                                    result.updated_fighters.push(fighter_name.clone());
                                }

                                // If this is a shared asset, update references in other fighters
                                if let Some(shared_with) =
                                    asset.get("shared_with").and_then(|s| s.as_array())
                                {
                                    if !shared_with.is_empty() {
                                        result.warnings.push(format!(
                                            "Asset {} is shared with other fighters: {:?}",
                                            asset_file, shared_with
                                        ));
                                    }
                                }

                                break;
                            }
                        }
                    }
                }

                if asset_found {
                    break;
                }
            }

            if !asset_found {
                return Err(ManifestUpdateError::AssetNotFound(format!(
                    "Asset {} not found in boxer {}",
                    asset_file, boxer_key
                )));
            }

            break;
        }
    }

    if !boxer_found {
        return Err(ManifestUpdateError::AssetNotFound(format!(
            "Boxer with key '{}' not found",
            boxer_key
        )));
    }

    // Serialize updated manifest
    match serde_json::to_string_pretty(manifest) {
        Ok(json) => {
            result.updated_manifest_json = Some(json);
            result.success = true;
        }
        Err(e) => {
            return Err(ManifestUpdateError::ConflictError(format!(
                "Failed to serialize updated manifest: {}",
                e
            )));
        }
    }

    Ok(result)
}

/// Batch update multiple asset addresses
pub fn batch_update_manifest_addresses(
    manifest: &mut serde_json::Value,
    changes: Vec<AddressChange>,
) -> Result<ManifestUpdateResult, ManifestUpdateError> {
    let mut result = ManifestUpdateResult {
        success: false,
        updated_files: Vec::new(),
        updated_fighters: Vec::new(),
        warnings: Vec::new(),
        updated_manifest_json: None,
    };

    // Build lookup of changes by file
    let change_map: HashMap<String, &AddressChange> =
        changes.iter().map(|c| (c.file.clone(), c)).collect();

    // Find fighters
    let fighters = manifest
        .get_mut("fighters")
        .and_then(|f| f.as_object_mut())
        .ok_or_else(|| {
            ManifestUpdateError::AssetNotFound("fighters object not found".to_string())
        })?;

    // Update all matching assets
    for (fighter_name, boxer_data) in fighters.iter_mut() {
        let asset_arrays = [
            "palette_files",
            "icon_files",
            "portrait_files",
            "large_portrait_files",
            "unique_sprite_bins",
            "shared_sprite_bins",
            "other_files",
        ];

        for array_name in &asset_arrays {
            if let Some(array) = boxer_data
                .get_mut(array_name)
                .and_then(|a| a.as_array_mut())
            {
                for asset in array.iter_mut() {
                    if let Some(file) = asset.get("file").and_then(|f| f.as_str()) {
                        if let Some(change) = change_map.get(file) {
                            // Clone file before mutating asset
                            let file_string = file.to_string();

                            // Update addresses
                            asset["start_pc"] =
                                serde_json::json!(format!("0x{:X}", change.new_start_pc));
                            asset["end_pc"] =
                                serde_json::json!(format!("0x{:X}", change.new_end_pc));
                            asset["size"] = serde_json::json!(change.new_size);

                            if change.update_snes_addresses {
                                asset["start_snes"] = serde_json::json!(change.new_start_snes());
                                asset["end_snes"] = serde_json::json!(change.new_end_snes());
                            }

                            if !result.updated_files.contains(&file_string) {
                                result.updated_files.push(file_string);
                            }
                            if !result.updated_fighters.contains(fighter_name) {
                                result.updated_fighters.push(fighter_name.clone());
                            }
                        }
                    }
                }
            }
        }
    }

    // Serialize updated manifest
    match serde_json::to_string_pretty(manifest) {
        Ok(json) => {
            result.updated_manifest_json = Some(json);
            result.success = true;
        }
        Err(e) => {
            return Err(ManifestUpdateError::ConflictError(format!(
                "Failed to serialize updated manifest: {}",
                e
            )));
        }
    }

    Ok(result)
}

/// Find all assets that reference a given address range
///
/// This is useful for finding what needs to be updated when relocating data
pub fn find_assets_at_address(
    manifest: &serde_json::Value,
    start_pc: usize,
    end_pc: Option<usize>,
) -> Vec<FoundAsset> {
    let mut found = Vec::new();
    let end_pc = end_pc.unwrap_or(start_pc);

    if let Some(fighters) = manifest.get("fighters").and_then(|f| f.as_object()) {
        for (fighter_name, boxer_data) in fighters.iter() {
            let asset_arrays = [
                "palette_files",
                "icon_files",
                "portrait_files",
                "large_portrait_files",
                "unique_sprite_bins",
                "shared_sprite_bins",
                "other_files",
            ];

            for array_name in &asset_arrays {
                if let Some(array) = boxer_data.get(array_name).and_then(|a| a.as_array()) {
                    for asset in array.iter() {
                        if let (Some(file), Some(start), Some(size)) = (
                            asset.get("file").and_then(|f| f.as_str()),
                            asset
                                .get("start_pc")
                                .and_then(|s| parse_hex_string(s.as_str()?)),
                            asset.get("size").and_then(|s| s.as_u64()),
                        ) {
                            let asset_end = start + size as usize - 1;

                            // Check if ranges overlap
                            if start <= end_pc && asset_end >= start_pc {
                                found.push(FoundAsset {
                                    fighter: fighter_name.clone(),
                                    file: file.to_string(),
                                    category: array_name.to_string(),
                                    start_pc: start,
                                    end_pc: asset_end,
                                    size: size as usize,
                                    subtype: asset
                                        .get("subtype")
                                        .and_then(|s| s.as_str())
                                        .unwrap_or("unknown")
                                        .to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    found
}

/// Represents a found asset in the manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoundAsset {
    pub fighter: String,
    pub file: String,
    pub category: String,
    pub start_pc: usize,
    pub end_pc: usize,
    pub size: usize,
    pub subtype: String,
}

/// Parse a hex string that may have 0x prefix
fn parse_hex_string(s: &str) -> Option<usize> {
    let s = s.trim();
    if s.starts_with("0x") || s.starts_with("0X") {
        usize::from_str_radix(&s[2..], 16).ok()
    } else {
        s.parse::<usize>().ok()
    }
}

/// Generate a report of all address changes in a batch operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchRelocationReport {
    pub total_changes: usize,
    pub fighters_affected: Vec<String>,
    pub files_updated: Vec<String>,
    pub shared_assets: Vec<String>,
    pub warnings: Vec<String>,
}

impl BatchRelocationReport {
    pub fn from_result(result: &ManifestUpdateResult) -> Self {
        Self {
            total_changes: result.updated_files.len(),
            fighters_affected: result.updated_fighters.clone(),
            files_updated: result.updated_files.clone(),
            shared_assets: Vec::new(), // Would be populated from analysis
            warnings: result.warnings.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_manifest() -> serde_json::Value {
        serde_json::json!({
            "source_rom": {
                "filename": "test.smc",
                "sha1": "abc123"
            },
            "fighters": {
                "TestFighter": {
                    "fighter": "Test Fighter",
                    "key": "test",
                    "reference_sheet": "test.png",
                    "palette_files": [
                        {
                            "file": "test_palette.bin",
                            "filename": "palette.bin",
                            "category": "palette",
                            "subtype": "main",
                            "size": 32,
                            "start_snes": "0x818000",
                            "end_snes": "0x81801F",
                            "start_pc": "0x8000",
                            "end_pc": "0x801F",
                            "shared_with": []
                        }
                    ],
                    "icon_files": [],
                    "portrait_files": [],
                    "large_portrait_files": [],
                    "unique_sprite_bins": [],
                    "shared_sprite_bins": [],
                    "other_files": []
                }
            }
        })
    }

    #[test]
    fn test_update_manifest_address() {
        let mut manifest = create_test_manifest();

        let result =
            update_manifest_address(&mut manifest, "test", "test_palette.bin", 0x10000, Some(64));

        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.success);
        assert_eq!(result.updated_files, vec!["test_palette.bin"]);
        assert_eq!(result.updated_fighters, vec!["TestFighter"]);
    }

    #[test]
    fn test_find_assets_at_address() {
        let manifest = create_test_manifest();

        let found = find_assets_at_address(&manifest, 0x8000, None);

        assert_eq!(found.len(), 1);
        assert_eq!(found[0].file, "test_palette.bin");
        assert_eq!(found[0].start_pc, 0x8000);
    }

    #[test]
    fn test_address_change() {
        let change = AddressChange::new("test.bin".to_string(), 0x8000, 32, 0x10000, 64);

        assert_eq!(change.new_start_pc, 0x10000);
        assert_eq!(change.new_size, 64);
        assert_eq!(change.new_end_pc, 0x1003F);
    }

    #[test]
    fn test_parse_hex_string() {
        assert_eq!(parse_hex_string("0x100"), Some(256));
        assert_eq!(parse_hex_string("100"), Some(100));
        assert_eq!(parse_hex_string("0xFF"), Some(255));
        assert_eq!(parse_hex_string("invalid"), None);
    }

    #[test]
    fn test_pc_to_snes() {
        // PC 0x8000 -> Bank 0x81, Address 0x8000 -> 0x818000
        assert_eq!(super::pc_to_snes(0x8000), 0x818000);
        // PC 0x0000 -> Bank 0x80, Address 0x8000 -> 0x808000
        assert_eq!(super::pc_to_snes(0x0000), 0x808000);
    }
}
