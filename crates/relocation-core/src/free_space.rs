use serde::{Deserialize, Serialize};

/// Represents a region of free (unallocated) space in the ROM
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct FreeSpaceRegion {
    pub start_pc: usize,
    pub end_pc: usize,
    pub size: usize,
}

impl FreeSpaceRegion {
    /// Check if this region can accommodate data of the given size
    pub fn can_fit(&self, size: usize) -> bool {
        self.size >= size
    }

    /// Check if this region can accommodate data with alignment requirements
    pub fn can_fit_aligned(&self, size: usize, alignment: usize) -> Option<usize> {
        let aligned_start = ((self.start_pc + alignment - 1) / alignment) * alignment;
        let available = self.end_pc.saturating_sub(aligned_start) + 1;
        if available >= size {
            Some(aligned_start)
        } else {
            None
        }
    }

    /// Check if a given offset falls within this region
    pub fn contains(&self, offset: usize) -> bool {
        offset >= self.start_pc && offset <= self.end_pc
    }

    /// Get the SNES LoROM address for the start of this region
    pub fn start_snes(&self) -> u32 {
        pc_to_snes(self.start_pc as u32)
    }

    /// Get the SNES LoROM address for the end of this region
    pub fn end_snes(&self) -> u32 {
        pc_to_snes(self.end_pc as u32)
    }
}

/// Convert PC offset to SNES LoROM address
fn pc_to_snes(pc: u32) -> u32 {
    // LoROM: Bank = (PC / 0x8000) | 0x80, Address = (PC & 0x7FFF) | 0x8000
    let bank = ((pc / 0x8000) | 0x80) & 0xFF;
    let addr = (pc & 0x7FFF) | 0x8000;
    (bank << 16) | addr
}

/// Find all free space regions in a ROM given known allocated regions
///
/// This function takes a slice of the ROM data and a list of allocated regions,
/// and returns all gaps (free space) that are not allocated.
pub fn find_free_regions(
    rom_size: usize,
    allocated_regions: &[(usize, usize)], // (start_pc, end_pc) pairs
    min_size: Option<usize>,
) -> Vec<FreeSpaceRegion> {
    let min_size = min_size.unwrap_or(1);
    let mut free_regions = Vec::new();

    if allocated_regions.is_empty() {
        // Entire ROM is free
        if rom_size >= min_size {
            free_regions.push(FreeSpaceRegion {
                start_pc: 0,
                end_pc: rom_size - 1,
                size: rom_size,
            });
        }
        return free_regions;
    }

    // Sort and merge overlapping regions
    let mut sorted = allocated_regions.to_vec();
    sorted.sort_by_key(|(start, _)| *start);

    let mut merged: Vec<(usize, usize)> = Vec::new();
    for (start, end) in sorted {
        if let Some((_, last_end)) = merged.last_mut() {
            if start <= *last_end + 1 {
                // Overlapping or adjacent, merge them
                *last_end = (*last_end).max(end);
            } else {
                merged.push((start, end));
            }
        } else {
            merged.push((start, end));
        }
    }

    // Find gaps between merged regions
    let mut current_pos = 0usize;

    for (start, end) in merged {
        if start > current_pos {
            let gap_size = start - current_pos;
            if gap_size >= min_size {
                free_regions.push(FreeSpaceRegion {
                    start_pc: current_pos,
                    end_pc: start - 1,
                    size: gap_size,
                });
            }
        }
        current_pos = end + 1;
    }

    // Check for trailing free space
    if current_pos < rom_size {
        let trailing_size = rom_size - current_pos;
        if trailing_size >= min_size {
            free_regions.push(FreeSpaceRegion {
                start_pc: current_pos,
                end_pc: rom_size - 1,
                size: trailing_size,
            });
        }
    }

    free_regions
}

/// Find the largest contiguous free region
pub fn find_largest_free_region(
    rom_size: usize,
    allocated_regions: &[(usize, usize)],
) -> Option<FreeSpaceRegion> {
    let regions = find_free_regions(rom_size, allocated_regions, None);
    regions.into_iter().max_by_key(|r| r.size)
}

/// Find a free region that can fit data of the specified size
pub fn find_suitable_region(
    rom_size: usize,
    allocated_regions: &[(usize, usize)],
    size: usize,
    alignment: Option<usize>,
) -> Option<FreeSpaceRegion> {
    let regions = find_free_regions(rom_size, allocated_regions, Some(size));

    if let Some(align) = alignment {
        regions
            .into_iter()
            .find(|r| r.can_fit_aligned(size, align).is_some())
    } else {
        regions.into_iter().find(|r| r.can_fit(size))
    }
}

/// Analyze free space distribution and return statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreeSpaceStats {
    pub total_free: usize,
    pub largest_block: usize,
    pub region_count: usize,
    pub average_block_size: usize,
    pub fragmentation_score: f32, // 0.0 = no fragmentation, 1.0 = highly fragmented
}

impl FreeSpaceStats {
    pub fn calculate(regions: &[FreeSpaceRegion], _rom_size: usize) -> Self {
        let total_free: usize = regions.iter().map(|r| r.size).sum();
        let largest_block = regions.iter().map(|r| r.size).max().unwrap_or(0);
        let region_count = regions.len();
        let average_block_size = if region_count > 0 {
            total_free / region_count
        } else {
            0
        };

        // Fragmentation score: 1 - (largest_block / total_free)
        // If all free space is in one block, fragmentation is 0
        // If free space is scattered in many small blocks, fragmentation approaches 1
        let fragmentation_score = if total_free > 0 {
            1.0 - (largest_block as f32 / total_free as f32)
        } else {
            0.0
        };

        Self {
            total_free,
            largest_block,
            region_count,
            average_block_size,
            fragmentation_score,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_free_regions_empty() {
        let regions = find_free_regions(1024, &[], None);
        assert_eq!(regions.len(), 1);
        assert_eq!(regions[0].size, 1024);
    }

    #[test]
    fn test_find_free_regions_with_gaps() {
        let allocated = vec![(100, 199), (300, 399), (500, 599)];
        let regions = find_free_regions(1024, &allocated, None);

        assert_eq!(regions.len(), 4);
        assert_eq!(regions[0].size, 100); // 0-99
        assert_eq!(regions[1].size, 100); // 200-299
        assert_eq!(regions[2].size, 100); // 400-499
        assert_eq!(regions[3].size, 424); // 600-1023
    }

    #[test]
    fn test_find_free_regions_merges_overlapping() {
        let allocated = vec![(100, 200), (150, 250), (300, 400)];
        let regions = find_free_regions(1024, &allocated, None);

        // Should merge (100,200) and (150,250) into (100,250)
        assert_eq!(regions.len(), 3);
    }

    #[test]
    fn test_free_space_region_can_fit() {
        let region = FreeSpaceRegion {
            start_pc: 100,
            end_pc: 299,
            size: 200,
        };

        assert!(region.can_fit(100));
        assert!(region.can_fit(200));
        assert!(!region.can_fit(201));
    }

    #[test]
    fn test_free_space_region_can_fit_aligned() {
        let region = FreeSpaceRegion {
            start_pc: 100,
            end_pc: 399,
            size: 300,
        };

        // Alignment of 256: next aligned address after 100 is 256
        let aligned = region.can_fit_aligned(100, 256);
        assert_eq!(aligned, Some(256));

        // Check that there's enough space from 256 to end (399)
        // 399 - 256 + 1 = 144 bytes available, not enough for 200
        let not_fit = region.can_fit_aligned(200, 256);
        assert_eq!(not_fit, None);
    }

    #[test]
    fn test_free_space_stats() {
        let regions = vec![
            FreeSpaceRegion {
                start_pc: 0,
                end_pc: 99,
                size: 100,
            },
            FreeSpaceRegion {
                start_pc: 200,
                end_pc: 249,
                size: 50,
            },
        ];

        let stats = FreeSpaceStats::calculate(&regions, 1024);

        assert_eq!(stats.total_free, 150);
        assert_eq!(stats.largest_block, 100);
        assert_eq!(stats.region_count, 2);
        assert_eq!(stats.average_block_size, 75);
        assert!(stats.fragmentation_score > 0.0); // Should be fragmented
    }

    #[test]
    fn test_pc_to_snes_conversion() {
        // Test some known LoROM conversions
        assert_eq!(pc_to_snes(0x0000), 0x808000); // Bank 80, address 8000
        assert_eq!(pc_to_snes(0x7FFF), 0x80FFFF); // Bank 80, address FFFF
        assert_eq!(pc_to_snes(0x8000), 0x818000); // Bank 81, address 8000
    }
}
