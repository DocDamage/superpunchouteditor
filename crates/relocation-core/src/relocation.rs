use crate::{FreeSpaceRegion, RelocationError};
use serde::{Deserialize, Serialize};

/// A byte match is evidence for investigation, not proof of a live reference.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PointerCandidate {
    pub pointer_location: usize,
    pub current_target: usize,
    pub new_target: usize,
    pub expected: [u8; 3],
    pub replacement: [u8; 3],
}

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
    pub fn get_pointer_bytes(&self) -> Result<Vec<u8>, RelocationError> {
        if !matches!(self.pointer_size, 2..=4) {
            return Err(RelocationError::InvalidSize(self.pointer_size as usize));
        }
        if self.is_snes_address {
            // This encoder describes ordinary LoROM, not ExLoROM or an implicit
            // bank-register update. A short pointer cannot express a bank move.
            if self.new_target >= 0x400000 || self.current_target >= 0x400000 {
                return Err(RelocationError::InvalidOffset(self.new_target));
            }
            if self.pointer_size == 2 && self.current_target / 0x8000 != self.new_target / 0x8000 {
                return Err(RelocationError::RomError(
                    "16-bit SNES pointer relocation requires the same target bank".into(),
                ));
            }
        } else {
            let maximum = match self.pointer_size {
                2 => 0xffff,
                3 => 0xffffff,
                _ => u32::MAX as usize,
            };
            if self.new_target > maximum {
                return Err(RelocationError::InvalidOffset(self.new_target));
            }
        }
        let target = if self.is_snes_address {
            Self::pc_to_snes_addr(self.new_target)
        } else {
            self.new_target as u32
        };

        Ok(match self.pointer_size {
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
            _ => unreachable!("pointer width validated above"),
        })
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

    if size == 0 || source_pc.checked_add(size).is_none() || dest_pc.checked_add(size).is_none() {
        return RelocationValidation {
            valid: false,
            source_pc,
            dest_pc,
            size,
            warnings,
            errors: vec!["Relocation size must be nonzero and ranges must not overflow".into()],
            estimated_pointer_updates: 0,
            affected_regions,
        };
    }

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

    /// Scan half-open byte ranges for long LoROM pointer candidates, including
    /// references into the asset interior. This cannot prove reference coverage:
    /// short/banked pointers and executable/data coincidences require analysis.
    pub fn scan_long_pointer_candidates(
        &self,
        rom: &[u8],
        source_pc: usize,
        dest_pc: usize,
        size: usize,
        search_ranges: &[(usize, usize)],
    ) -> Result<Vec<PointerCandidate>, RelocationError> {
        let source_end = source_pc
            .checked_add(size)
            .ok_or(RelocationError::ExceedsRomSize)?;
        let dest_end = dest_pc
            .checked_add(size)
            .ok_or(RelocationError::ExceedsRomSize)?;
        if size == 0 {
            return Err(RelocationError::InvalidSize(size));
        }
        if source_end > rom.len()
            || dest_end > rom.len()
            || source_end > 0x400000
            || dest_end > 0x400000
        {
            return Err(RelocationError::ExceedsRomSize);
        }
        let mut candidates = std::collections::BTreeMap::new();
        for &(start, end) in search_ranges {
            let bytes = rom
                .get(start..end)
                .ok_or(RelocationError::InvalidOffset(start))?;
            for (index, bytes) in bytes.windows(3).enumerate() {
                let addr = u16::from_le_bytes([bytes[0], bytes[1]]);
                let bank = bytes[2];
                if addr < 0x8000 || bank == 0x7e || bank == 0x7f {
                    continue;
                }
                let target = usize::from(bank & 0x7f) * 0x8000 + usize::from(addr & 0x7fff);
                if target < source_pc || target >= source_end {
                    continue;
                }
                let new_target = dest_pc + target - source_pc;
                let encoded = PointerUpdate::pc_to_snes_addr(new_target);
                let replacement_bank = ((encoded >> 16) as u8 & 0x7f) | (bank & 0x80);
                if replacement_bank == 0x7e || replacement_bank == 0x7f {
                    return Err(RelocationError::RomError(
                        "Destination would map a low-bank pointer into WRAM".into(),
                    ));
                }
                candidates.insert(
                    start + index,
                    PointerCandidate {
                        pointer_location: start + index,
                        current_target: target,
                        new_target,
                        expected: [bytes[0], bytes[1], bank],
                        replacement: [encoded as u8, (encoded >> 8) as u8, replacement_bank],
                    },
                );
            }
        }
        Ok(candidates.into_values().collect())
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
    #[test]
    fn invalid_ranges_return_errors_without_arithmetic_panics() {
        for (source, destination, size) in [
            (0, 0, 0),
            (usize::MAX, 0, 1),
            (0, usize::MAX, 1),
            (1, 1, usize::MAX),
        ] {
            let report =
                super::validate_relocation(usize::MAX, &[], source, destination, size, true);
            assert!(!report.valid);
            assert!(!report.errors.is_empty());
            assert!(report.affected_regions.is_empty());
        }
    }
    use super::*;

    #[test]
    fn long_pointer_scan_requires_byte_evidence_and_preserves_bank_aliases() {
        let mut rom = vec![0; 0x60000];
        let planner = RelocationPlanner::new(rom.len(), vec![]);
        assert!(planner
            .scan_long_pointer_candidates(&rom, 0x48000, 0x50000, 32, &[(0, 64)])
            .unwrap()
            .is_empty());
        rom[16..19].copy_from_slice(&[2, 0x80, 0x09]);
        rom[22..25].copy_from_slice(&[0, 0x80, 0x89]);
        let candidates = planner
            .scan_long_pointer_candidates(&rom, 0x48000, 0x50000, 32, &[(0, 64), (16, 25)])
            .unwrap();
        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates[0].pointer_location, 16);
        assert_eq!(candidates[0].current_target, 0x48002);
        assert_eq!(candidates[0].new_target, 0x50002);
        assert_eq!(candidates[0].expected, [2, 0x80, 9]);
        assert_eq!(candidates[0].replacement, [2, 0x80, 10]);
        assert_eq!(candidates[1].replacement, [0, 0x80, 0x8a]);
        assert!(planner
            .scan_long_pointer_candidates(&rom, 0x48000, 0x50000, 32, &[(0, usize::MAX)])
            .is_err());
        assert!(planner
            .scan_long_pointer_candidates(&rom, 0x48000, 0x50000, 0, &[])
            .is_err());
        assert!(planner
            .scan_long_pointer_candidates(&rom, usize::MAX, 0, 2, &[])
            .is_err());
    }

    #[test]
    fn pointer_encoding_rejects_bank_loss_and_truncation() {
        let mut pointer = PointerUpdate {
            pointer_location: 0,
            current_target: 0x48000,
            new_target: 0x50000,
            pointer_size: 2,
            description: String::new(),
            is_snes_address: true,
        };
        assert!(pointer.get_pointer_bytes().is_err());
        pointer.new_target = 0x48123;
        assert_eq!(pointer.get_pointer_bytes().unwrap(), vec![0x23, 0x81]);
        pointer.pointer_size = 3;
        pointer.new_target = 0x50000;
        assert_eq!(pointer.get_pointer_bytes().unwrap(), vec![0, 0x80, 0x8a]);
        pointer.new_target = 0x400000;
        assert!(pointer.get_pointer_bytes().is_err());
        pointer.is_snes_address = false;
        for (width, target) in [(2, 0x10000), (3, 0x1000000), (1, 1), (5, 1)] {
            pointer.pointer_size = width;
            pointer.new_target = target;
            assert!(pointer.get_pointer_bytes().is_err());
        }
    }

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

        let bytes = update.get_pointer_bytes().unwrap();
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

        let bytes = update.get_pointer_bytes().unwrap();
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
