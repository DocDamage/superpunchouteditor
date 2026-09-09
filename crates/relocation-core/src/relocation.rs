use crate::{FreeSpaceRegion, RelocationError};
use serde::{Deserialize, Serialize};

/// Information about a pointer that needs to be updated
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PointerUpdate {
    /// PC offset where the pointer is stored in the ROM
    pub pointer_location: usize,
    /// The current value of the pointer (PC offset it points to)
    pub current_target: usize,
    /// The new value the pointer should have after relocation
    pub new_target: usize,
    /// Size of the pointer in bytes (2 or 3 for SNES)
    pub pointer_size: u8,
    /// Description of what this pointer references
    pub description: String,
    /// Whether this pointer uses SNES address format
    pub is_snes_address: bool,
}

impl PointerUpdate {
    /// Convert a PC offset to SNES LoROM address format
    pub fn pc_to_snes_addr(pc: usize) -> u32 {
        let bank = ((pc / 0x8000) | 0x80) & 0xFF;
        let addr = (pc & 0x7FFF) | 0x8000;
        ((bank as u32) << 16) | (addr as u32)
    }

    /// Get the bytes that should be written for this pointer update
    pub fn get_pointer_bytes(&self) -> Vec<u8> {
        let target = if self.is_snes_address {
            Self::pc_to_snes_addr(self.new_target)
        } else {
            self.new_target as u32
        };

        match self.pointer_size {
            2 => vec![(target & 0xFF) as u8, ((target >> 8) & 0xFF) as u8],
            3 => vec![
                (target & 0xFF) as u8,
                ((target >> 8) & 0xFF) as u8,
                ((target >> 16) & 0xFF) as u8,
            ],
            4 => vec![
                (target & 0xFF) as u8,
                ((target >> 8) & 0xFF) as u8,
                ((target >> 16) & 0xFF) as u8,
                ((target >> 24) & 0xFF) as u8,
            ],
            _ => vec![(target & 0xFF) as u8, ((target >> 8) & 0xFF) as u8],
        }
    }
}

/// Result of validating a proposed relocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelocationValidation {
    pub valid: bool,
    pub source_pc: usize,
    pub dest_pc: usize,
    pub size: usize,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    /// Estimated number of pointers that may need updating
    pub estimated_pointer_updates: usize,
    /// Regions that would be affected by this relocation
    pub affected_regions: Vec<AffectedRegion>,
}

/// A region that would be affected by a relocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AffectedRegion {
    pub start_pc: usize,
    pub end_pc: usize,
    pub description: String,
    pub impact: RegionImpact,
}

/// The impact level of an affected region
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RegionImpact {
    /// Source data will be moved
    Source,
    /// Destination will be overwritten
    Destination,
    /// Pointers to this region need updating
    Referenced,
    /// Region is adjacent and may be affected
    Adjacent,
}

/// Validates whether a relocation from source to destination is safe
pub fn validate_relocation(
    rom_size: usize,
    free_regions: &[FreeSpaceRegion],
    source_pc: usize,
    dest_pc: usize,
    size: usize,
    check_overlaps: bool,
) -> RelocationValidation {
    let mut warnings = Vec::new();
    let mut errors = Vec::new();
    let mut affected_regions = Vec::new();
    let mut valid = true;

    // Basic bounds checks
    if source_pc >= rom_size {
        errors.push(format!("Source offset 0x{:X} exceeds ROM size", source_pc));
        valid = false;
    }

    if dest_pc >= rom_size {
        errors.push(format!(
            "Destination offset 0x{:X} exceeds ROM size",
            dest_pc
        ));
        valid = false;
    }

    if size == 0 {
        errors.push("Size cannot be zero".to_string());
        valid = false;
    }

    if source_pc + size > rom_size {
        errors.push(format!(
            "Source range (0x{:X} - 0x{:X}) exceeds ROM size",
            source_pc,
            source_pc + size - 1
        ));
        valid = false;
    }

    if dest_pc + size > rom_size {
        errors.push(format!(
            "Destination range (0x{:X} - 0x{:X}) exceeds ROM size",
            dest_pc,
            dest_pc + size - 1
        ));
        valid = false;
    }

    // Check for overlap between source and destination
    if check_overlaps
        && ranges_overlap(source_pc, source_pc + size - 1, dest_pc, dest_pc + size - 1)
    {
        // This is a warning, not necessarily an error, as partial overlaps can be handled
        warnings.push("Source and destination ranges overlap".to_string());
        affected_regions.push(AffectedRegion {
            start_pc: source_pc.max(dest_pc),
            end_pc: (source_pc + size - 1).min(dest_pc + size - 1),
            description: "Overlapping region".to_string(),
            impact: RegionImpact::Source,
        });
    }

    // Check if destination has enough free space
    let dest_fits = free_regions
        .iter()
        .any(|r| r.contains(dest_pc) && r.end_pc >= dest_pc + size - 1);

    if !dest_fits {
        errors.push(format!(
            "Destination range (0x{:X} - 0x{:X}) is not entirely within free space",
            dest_pc,
            dest_pc + size - 1
        ));
        valid = false;
    }

    // Check if source region is in free space (shouldn't be - we're moving allocated data)
    let source_in_free = free_regions
        .iter()
        .any(|r| r.contains(source_pc) && r.contains(source_pc + size - 1));

    if source_in_free {
        warnings
            .push("Source region appears to be in free space (may already be empty)".to_string());
    }

    // Add affected regions
    affected_regions.push(AffectedRegion {
        start_pc: source_pc,
        end_pc: source_pc + size - 1,
        description: "Source data to be moved".to_string(),
        impact: RegionImpact::Source,
    });

    affected_regions.push(AffectedRegion {
        start_pc: dest_pc,
        end_pc: dest_pc + size - 1,
        description: "Destination region".to_string(),
        impact: RegionImpact::Destination,
    });

    // Estimate pointer updates (rough heuristic based on typical game data)
    let estimated_pointer_updates = if valid {
        // Typically 2-4 pointers per asset (data pointers, sometimes length)
        4
    } else {
        0
    };

    RelocationValidation {
        valid: valid && errors.is_empty(),
        source_pc,
        dest_pc,
        size,
        warnings,
        errors,
        estimated_pointer_updates,
        affected_regions,
    }
}

/// Check if two ranges overlap
fn ranges_overlap(start1: usize, end1: usize, start2: usize, end2: usize) -> bool {
    start1 <= end2 && start2 <= end1
}

/// Plans and executes relocation operations
pub struct RelocationPlanner {
    rom_size: usize,
    free_regions: Vec<FreeSpaceRegion>,
    pending_relocations: Vec<PendingRelocation>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct PendingRelocation {
    source_pc: usize,
    dest_pc: usize,
    size: usize,
    pointer_updates: Vec<PointerUpdate>,
}

impl RelocationPlanner {
    pub fn new(rom_size: usize, free_regions: Vec<FreeSpaceRegion>) -> Self {
        Self {
            rom_size,
            free_regions,
            pending_relocations: Vec::new(),
        }
    }

    /// Add a relocation to the plan
    pub fn plan_relocation(
        &mut self,
        source_pc: usize,
        dest_pc: usize,
        size: usize,
    ) -> Result<RelocationValidation, RelocationError> {
        let validation = validate_relocation(
            self.rom_size,
            &self.free_regions,
            source_pc,
            dest_pc,
            size,
            true,
        );

        if !validation.valid {
            return Err(RelocationError::WouldOverwrite);
        }

        self.pending_relocations.push(PendingRelocation {
            source_pc,
            dest_pc,
            size,
            pointer_updates: Vec::new(),
        });

        Ok(validation)
    }

    /// Estimate the pointer updates needed for a relocation
    pub fn estimate_pointer_updates(
        &self,
        source_pc: usize,
        dest_pc: usize,
        _size: usize,
        search_ranges: &[(usize, usize)], // Ranges to scan for pointers
    ) -> Vec<PointerUpdate> {
        let mut updates = Vec::new();

        // This is a simplified estimation - in a real implementation,
        // you would scan the ROM for values matching the source address
        // in various formats (SNES address, PC offset, etc.)

        // Common patterns in SNES games:
        // - 24-bit SNES addresses (3 bytes)
        // - 16-bit offsets within a bank (2 bytes)
        // - Sometimes 32-bit pointers

        for (search_start, _search_end) in search_ranges {
            // Heuristic: check if any known pointer tables overlap this range
            // For now, we'll add placeholder pointer updates for common scenarios

            // Data header pointer (common in graphics data)
            updates.push(PointerUpdate {
                pointer_location: *search_start, // Placeholder
                current_target: source_pc,
                new_target: dest_pc,
                pointer_size: 3,
                description: "Data pointer".to_string(),
                is_snes_address: true,
            });
        }

        updates
    }

    /// Get all pending relocations
    #[allow(private_interfaces)]
    pub fn get_pending_relocations(&self) -> &Vec<PendingRelocation> {
        &self.pending_relocations
    }

    /// Clear all pending relocations
    pub fn clear_pending(&mut self) {
        self.pending_relocations.clear();
    }

    /// Get the number of pending relocations
    pub fn pending_count(&self) -> usize {
        self.pending_relocations.len()
    }
}

/// Safety recommendations for a relocation operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelocationSafetyReport {
    pub overall_risk: RiskLevel,
    pub recommendations: Vec<String>,
    pub required_steps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl RelocationSafetyReport {
    pub fn from_validation(validation: &RelocationValidation) -> Self {
        let mut risk = RiskLevel::Low;
        let mut recommendations = Vec::new();
        let mut required_steps = Vec::new();

        if !validation.warnings.is_empty() {
            risk = RiskLevel::Medium;
            recommendations.extend(validation.warnings.clone());
        }

        if validation
            .affected_regions
            .iter()
            .any(|r| r.impact == RegionImpact::Source)
        {
            required_steps.push("Create a backup of the ROM before proceeding".to_string());
        }

        if validation.estimated_pointer_updates > 0 {
            required_steps.push(format!(
                "Update {} pointer(s) after data relocation",
                validation.estimated_pointer_updates
            ));
        }

        if validation.errors.len() > 1 {
            risk = RiskLevel::High;
        }

        if !validation.errors.is_empty() {
            risk = RiskLevel::Critical;
        }

        RelocationSafetyReport {
            overall_risk: risk,
            recommendations,
            required_steps,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pointer_update_get_bytes_16bit() {
        let update = PointerUpdate {
            pointer_location: 0x100,
            current_target: 0x2000,
            new_target: 0x3000,
            pointer_size: 2,
            description: "Test".to_string(),
            is_snes_address: false,
        };

        let bytes = update.get_pointer_bytes();
        assert_eq!(bytes, vec![0x00, 0x30]);
    }

    #[test]
    fn test_pointer_update_get_bytes_24bit_snes() {
        let update = PointerUpdate {
            pointer_location: 0x100,
            current_target: 0x8000, // PC offset
            new_target: 0x8000,     // Same PC offset
            pointer_size: 3,
            description: "Test".to_string(),
            is_snes_address: true,
        };

        let bytes = update.get_pointer_bytes();
        // PC 0x8000 -> SNES 0x818000
        assert_eq!(bytes, vec![0x00, 0x80, 0x81]);
    }

    #[test]
    fn test_validate_relocation_valid() {
        let free_regions = vec![FreeSpaceRegion {
            start_pc: 0,
            end_pc: 0x1FFFFF,
            size: 0x200000,
        }];

        let result = validate_relocation(0x200000, &free_regions, 0x10000, 0x80000, 0x1000, true);

        assert!(result.valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_validate_relocation_out_of_bounds() {
        let free_regions = vec![FreeSpaceRegion {
            start_pc: 0,
            end_pc: 0x1FFFFF,
            size: 0x200000,
        }];

        let result = validate_relocation(
            0x200000,
            &free_regions,
            0x1FFF00,
            0x80000,
            0x2000, // Would exceed ROM size
            true,
        );

        assert!(!result.valid);
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_ranges_overlap() {
        assert!(ranges_overlap(100, 200, 150, 250));
        assert!(ranges_overlap(150, 250, 100, 200));
        assert!(ranges_overlap(100, 200, 200, 300)); // Adjacent
        assert!(!ranges_overlap(100, 200, 201, 300)); // Not overlapping
    }

    #[test]
    fn test_relocation_safety_report() {
        let validation = RelocationValidation {
            valid: true,
            source_pc: 0x1000,
            dest_pc: 0x8000,
            size: 0x100,
            warnings: vec!["Test warning".to_string()],
            errors: vec![],
            estimated_pointer_updates: 2,
            affected_regions: vec![],
        };

        let report = RelocationSafetyReport::from_validation(&validation);

        assert_eq!(report.overall_risk, RiskLevel::Medium);
        assert!(!report.recommendations.is_empty());
        assert!(!report.required_steps.is_empty());
    }
}
