//! Bank duplication tracking and management for shared bank cloning.
//!
//! This module provides functionality to clone shared graphics banks so that
//! one boxer can have unique modifications without affecting the other boxer
//! that originally shared the bank.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a single bank duplication operation.
/// When a shared bank is duplicated, this record tracks:
/// - The original bank location and data
/// - The new location in ROM
/// - Which boxer now uses the duplicated bank
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankDuplication {
    /// Unique ID for this duplication (UUID v4)
    pub id: String,

    /// The PC offset of the original shared bank
    pub original_pc_offset: usize,

    /// The size of the original bank in bytes
    pub original_size: usize,

    /// The PC offset where the duplicated bank was placed
    pub new_pc_offset: usize,

    /// The boxer key that now uses the duplicated bank
    pub target_boxer_key: String,

    /// The original bank filename (for display purposes)
    pub original_filename: String,

    /// Whether the bank was compressed
    pub compressed: bool,

    /// Hash of the original data at time of duplication
    pub original_hash: String,

    /// Hash of the duplicated data
    pub duplicated_hash: String,

    /// Timestamp when the duplication was created
    pub created_at: DateTime<Utc>,

    /// Optional notes about why this duplication was created
    pub notes: Option<String>,

    /// The new size if recompressed differently
    pub current_size: usize,

    /// Whether this duplication has been modified from the original
    pub is_modified: bool,
}

/// Status result from attempting to duplicate a bank
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicationResult {
    pub success: bool,
    pub duplication: Option<BankDuplication>,
    pub error: Option<String>,
    pub warnings: Vec<String>,
    /// Information about space usage
    pub space_info: Option<SpaceInfo>,
}

/// Information about ROM space usage for a duplication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpaceInfo {
    pub original_offset: usize,
    pub new_offset: usize,
    pub size: usize,
    pub space_found: String, // "free_space", "expanded_rom", "relocated"
    pub bytes_remaining: usize,
}

/// Manager for tracking all bank duplications in a project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankDuplicationManager {
    /// Map of duplication ID -> BankDuplication
    pub duplications: HashMap<String, BankDuplication>,

    /// Map of boxer_key -> list of duplication IDs owned by that boxer
    pub boxer_duplications: HashMap<String, Vec<String>>,

    /// Map of original PC offset -> list of duplication IDs that cloned from it
    pub original_to_duplications: HashMap<usize, Vec<String>>,

    /// Track used regions in the ROM to avoid overlaps
    /// Each tuple is (start_pc, end_pc) for a used region
    pub used_regions: Vec<(usize, usize)>,
}

impl Default for BankDuplicationManager {
    fn default() -> Self {
        Self::new()
    }
}

impl BankDuplicationManager {
    /// Create a new empty duplication manager
    pub fn new() -> Self {
        Self {
            duplications: HashMap::new(),
            boxer_duplications: HashMap::new(),
            original_to_duplications: HashMap::new(),
            used_regions: Vec::new(),
        }
    }

    /// Check if a boxer already has a duplicated version of a specific bank
    pub fn has_duplication(&self, boxer_key: &str, original_pc_offset: usize) -> bool {
        if let Some(dup_ids) = self.boxer_duplications.get(boxer_key) {
            for id in dup_ids {
                if let Some(dup) = self.duplications.get(id) {
                    if dup.original_pc_offset == original_pc_offset {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Get the duplication for a specific boxer and original bank
    pub fn get_duplication_for_boxer(
        &self,
        boxer_key: &str,
        original_pc_offset: usize,
    ) -> Option<&BankDuplication> {
        if let Some(dup_ids) = self.boxer_duplications.get(boxer_key) {
            for id in dup_ids {
                if let Some(dup) = self.duplications.get(id) {
                    if dup.original_pc_offset == original_pc_offset {
                        return Some(dup);
                    }
                }
            }
        }
        None
    }

    /// Get all duplications for a specific boxer
    pub fn get_boxer_duplications(&self, boxer_key: &str) -> Vec<&BankDuplication> {
        self.boxer_duplications
            .get(boxer_key)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.duplications.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Register a new duplication
    pub fn register_duplication(&mut self, duplication: BankDuplication) {
        let id = duplication.id.clone();
        let boxer_key = duplication.target_boxer_key.clone();
        let original_offset = duplication.original_pc_offset;
        let start = duplication.new_pc_offset;
        let end = start + duplication.current_size;

        // Add to main map
        self.duplications.insert(id.clone(), duplication);

        // Add to boxer index
        self.boxer_duplications
            .entry(boxer_key)
            .or_default()
            .push(id.clone());

        // Add to original offset index
        self.original_to_duplications
            .entry(original_offset)
            .or_default()
            .push(id);

        // Mark region as used
        self.used_regions.push((start, end));
        // Sort by start for efficient lookups
        self.used_regions.sort_by_key(|(s, _)| *s);
    }

    /// Check if a region overlaps with any used region
    pub fn region_overlaps(&self, start: usize, end: usize) -> bool {
        self.used_regions.iter().any(|(used_start, used_end)| {
            // Check for overlap: two ranges [a,b] and [c,d] overlap if a <= d && c <= b
            start < *used_end && *used_start < end
        })
    }

    /// Find the next available region of at least the requested size
    pub fn find_available_region(
        &self,
        start_search: usize,
        size: usize,
        max_offset: usize,
    ) -> Option<usize> {
        let mut current = start_search;

        for (used_start, used_end) in &self.used_regions {
            // If current position is before the used region
            if current + size <= *used_start {
                // We found a gap
                return Some(current);
            }
            // Move current to after this used region
            current = current.max(*used_end);

            if current + size > max_offset {
                return None;
            }
        }

        // Check if there's space after the last used region
        if current + size <= max_offset {
            Some(current)
        } else {
            None
        }
    }

    /// Update a duplication's modified status
    pub fn mark_as_modified(&mut self, duplication_id: &str, new_size: Option<usize>) {
        if let Some(dup) = self.duplications.get_mut(duplication_id) {
            dup.is_modified = true;
            if let Some(size) = new_size {
                dup.current_size = size;
            }
        }
    }

    /// Remove a duplication (use with caution - may break boxer references)
    pub fn remove_duplication(&mut self, duplication_id: &str) -> Option<BankDuplication> {
        let dup = self.duplications.remove(duplication_id)?;

        // Remove from boxer index
        if let Some(ids) = self.boxer_duplications.get_mut(&dup.target_boxer_key) {
            ids.retain(|id| id != duplication_id);
        }

        // Remove from original offset index
        if let Some(ids) = self
            .original_to_duplications
            .get_mut(&dup.original_pc_offset)
        {
            ids.retain(|id| id != duplication_id);
        }

        // Remove from used regions
        let start = dup.new_pc_offset;
        let end = start + dup.current_size;
        self.used_regions
            .retain(|(s, e)| !(*s == start && *e == end));

        Some(dup)
    }

    /// Get total bytes used by all duplications
    pub fn total_duplicated_bytes(&self) -> usize {
        self.duplications.values().map(|d| d.current_size).sum()
    }

    /// Get count of duplications per boxer
    pub fn duplication_counts(&self) -> HashMap<String, usize> {
        self.boxer_duplications
            .iter()
            .map(|(k, v)| (k.clone(), v.len()))
            .collect()
    }
}

/// Request to duplicate a shared bank
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateBankRequest {
    /// PC offset of the original bank
    pub original_pc_offset: usize,

    /// Size of the bank
    pub size: usize,

    /// The boxer key that will own the duplicated bank
    pub target_boxer_key: String,

    /// Original filename for reference
    pub original_filename: String,

    /// Whether the original is compressed
    pub compressed: bool,

    /// Hash of original data
    pub original_hash: String,

    /// Optional notes
    pub notes: Option<String>,

    /// Preferred location strategy
    pub strategy: DuplicationStrategy,
}

/// Strategy for placing the duplicated bank
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DuplicationStrategy {
    /// Find free space within existing ROM bounds
    FindFreeSpace,

    /// Expand ROM to 2.5MB or 4MB if needed
    ExpandIfNeeded,

    /// Specific location (advanced)
    SpecificLocation(usize),
}

impl Default for DuplicationStrategy {
    fn default() -> Self {
        DuplicationStrategy::FindFreeSpace
    }
}

/// Helper to compute a simple hash for bank data
pub fn compute_bank_hash(data: &[u8]) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_region_overlap_detection() {
        let mut manager = BankDuplicationManager::new();

        // No regions initially
        assert!(!manager.region_overlaps(0, 100));

        // Add a used region
        manager.used_regions.push((1000, 2000));

        // These should overlap
        assert!(manager.region_overlaps(500, 1500)); // Overlaps start
        assert!(manager.region_overlaps(1500, 2500)); // Overlaps end
        assert!(manager.region_overlaps(1200, 1800)); // Inside
        assert!(manager.region_overlaps(500, 2500)); // Contains

        // These should not overlap
        assert!(!manager.region_overlaps(0, 500)); // Before
        assert!(!manager.region_overlaps(2000, 3000)); // After (end is exclusive)
        assert!(!manager.region_overlaps(2001, 3000)); // Clearly after
    }

    #[test]
    fn test_find_available_region() {
        let mut manager = BankDuplicationManager::new();
        manager.used_regions.push((1000, 2000));
        manager.used_regions.push((3000, 4000));

        // Should find space before first region
        assert_eq!(manager.find_available_region(0, 500, 5000), Some(0));

        // Should find space between regions
        assert_eq!(manager.find_available_region(0, 500, 5000), Some(0));
        assert_eq!(manager.find_available_region(1500, 500, 5000), Some(2000));

        // Should find space after last region
        assert_eq!(manager.find_available_region(3500, 500, 5000), Some(4000));

        // Should return None if no space
        assert_eq!(manager.find_available_region(4500, 600, 5000), None);
    }
}
