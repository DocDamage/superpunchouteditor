//! Advanced bank management for ROM optimization and visualization

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

/// Represents a single bank in the ROM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bank {
    /// Bank number (0x80-0xFF for LoROM)
    pub bank_number: u8,
    /// PC offset where this bank starts
    pub pc_start: usize,
    /// Size of the bank (typically 0x8000 = 32KB)
    pub size: usize,
    /// Regions within this bank
    pub regions: Vec<BankRegion>,
    /// Bank usage statistics
    pub stats: BankStats,
}

/// Type of region within a bank
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RegionType {
    /// Free/unused space
    Free,
    /// Graphics data (compressed)
    GraphicsCompressed,
    /// Graphics data (uncompressed)
    GraphicsUncompressed,
    /// Palette data
    Palette,
    /// Sound/music data
    Audio,
    /// Code/executable
    Code,
    /// Text/dialog data
    Text,
    /// Unknown/unspecified
    Unknown,
}

impl RegionType {
    /// Get color for visualization (RGBA)
    pub fn color(&self) -> [u8; 4] {
        match self {
            RegionType::Free => [50, 200, 50, 255],        // Green
            RegionType::GraphicsCompressed => [200, 50, 50, 255], // Red
            RegionType::GraphicsUncompressed => [200, 100, 50, 255], // Orange
            RegionType::Palette => [200, 200, 50, 255],    // Yellow
            RegionType::Audio => [50, 50, 200, 255],       // Blue
            RegionType::Code => [100, 100, 100, 255],      // Gray
            RegionType::Text => [200, 50, 200, 255],       // Magenta
            RegionType::Unknown => [150, 150, 150, 255],   // Light gray
        }
    }
    
    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            RegionType::Free => "Free Space",
            RegionType::GraphicsCompressed => "Graphics (Compressed)",
            RegionType::GraphicsUncompressed => "Graphics (Uncompressed)",
            RegionType::Palette => "Palettes",
            RegionType::Audio => "Audio",
            RegionType::Code => "Code",
            RegionType::Text => "Text",
            RegionType::Unknown => "Unknown",
        }
    }
}

/// A region within a bank
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankRegion {
    /// Offset within the bank (0-0x7FFF)
    pub offset: usize,
    /// Size of the region
    pub size: usize,
    /// Type of data in this region
    pub region_type: RegionType,
    /// Description of this region
    pub description: Option<String>,
    /// Asset ID if this region contains a known asset
    pub asset_id: Option<String>,
    /// Whether this region can be safely moved
    pub movable: bool,
    /// Whether this region is shared between multiple boxers
    pub shared: bool,
    /// Boxer keys that use this region (if shared)
    pub shared_with: Vec<String>,
}

/// Statistics for a bank
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BankStats {
    /// Total bytes used
    pub used_bytes: usize,
    /// Total bytes free
    pub free_bytes: usize,
    /// Breakdown by region type
    pub type_breakdown: HashMap<RegionType, usize>,
    /// Fragmentation score (0.0 = no fragmentation, 1.0 = highly fragmented)
    pub fragmentation: f32,
    /// Largest contiguous free block
    pub largest_free_block: usize,
}

/// Complete bank map for a ROM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankMap {
    /// Banks indexed by bank number
    pub banks: BTreeMap<u8, Bank>,
    /// Total ROM size
    pub total_size: usize,
    /// Overall statistics
    pub total_stats: BankStats,
}

impl BankMap {
    /// Create a bank map from ROM data
    pub fn from_rom(rom_data: &[u8]) -> Self {
        let bank_size = 0x8000; // 32KB per bank in LoROM
        let num_banks = rom_data.len() / bank_size;
        
        let mut banks = BTreeMap::new();
        
        for i in 0..num_banks {
            let bank_num = 0x80 + i as u8;
            let pc_start = i * bank_size;
            
            let bank = Bank {
                bank_number: bank_num,
                pc_start,
                size: bank_size,
                regions: vec![BankRegion {
                    offset: 0,
                    size: bank_size,
                    region_type: RegionType::Unknown,
                    description: None,
                    asset_id: None,
                    movable: false,
                    shared: false,
                    shared_with: Vec::new(),
                }],
                stats: BankStats::default(),
            };
            
            banks.insert(bank_num, bank);
        }
        
        Self {
            banks,
            total_size: rom_data.len(),
            total_stats: BankStats::default(),
        }
    }
    
    /// Mark a region with a specific type
    pub fn mark_region(
        &mut self,
        bank_num: u8,
        offset: usize,
        size: usize,
        region_type: RegionType,
        description: Option<String>,
    ) {
        if let Some(bank) = self.banks.get_mut(&bank_num) {
            // Find and split existing regions
            let mut new_regions = Vec::new();
            
            for region in &bank.regions {
                let region_start = region.offset;
                let region_end = region.offset + region.size;
                let mark_start = offset;
                let mark_end = offset + size;
                
                if mark_end <= region_start || mark_start >= region_end {
                    // No overlap
                    new_regions.push(region.clone());
                } else {
                    // Split the region
                    if mark_start > region_start {
                        new_regions.push(BankRegion {
                            offset: region_start,
                            size: mark_start - region_start,
                            ..region.clone()
                        });
                    }
                    
                    new_regions.push(BankRegion {
                        offset: mark_start.max(region_start),
                        size: (mark_end.min(region_end) - mark_start.max(region_start)),
                        region_type,
                        description: description.clone(),
                        ..region.clone()
                    });
                    
                    if mark_end < region_end {
                        new_regions.push(BankRegion {
                            offset: mark_end,
                            size: region_end - mark_end,
                            ..region.clone()
                        });
                    }
                }
            }
            
            bank.regions = new_regions;
            bank.regions.sort_by_key(|r| r.offset);
            self.recalculate_stats(bank_num);
        }
    }
    
    /// Mark free space in a bank
    pub fn mark_free(&mut self, bank_num: u8, offset: usize, size: usize) {
        self.mark_region(bank_num, offset, size, RegionType::Free, None);
    }
    
    /// Find all free regions across all banks
    pub fn find_free_regions(&self, min_size: usize) -> Vec<FreeRegion> {
        let mut free_regions = Vec::new();
        
        for (bank_num, bank) in &self.banks {
            for region in &bank.regions {
                if region.region_type == RegionType::Free && region.size >= min_size {
                    free_regions.push(FreeRegion {
                        bank: *bank_num,
                        offset: region.offset,
                        size: region.size,
                        pc_offset: bank.pc_start + region.offset,
                    });
                }
            }
        }
        
        free_regions.sort_by(|a, b| b.size.cmp(&a.size));
        free_regions
    }
    
    /// Find the best location for data of a given size
    pub fn find_best_location(&self, size: usize, alignment: usize) -> Option<FreeRegion> {
        self.find_free_regions(size)
            .into_iter()
            .find(|r| r.size >= size && (r.pc_offset % alignment == 0))
    }
    
    /// Recalculate statistics for a bank
    fn recalculate_stats(&mut self, bank_num: u8) {
        if let Some(bank) = self.banks.get_mut(&bank_num) {
            let mut stats = BankStats::default();
            let mut free_regions = Vec::new();
            
            for region in &bank.regions {
                match region.region_type {
                    RegionType::Free => {
                        stats.free_bytes += region.size;
                        free_regions.push(region.size);
                    }
                    _ => stats.used_bytes += region.size,
                }
                
                *stats.type_breakdown.entry(region.region_type).or_insert(0) += region.size;
            }
            
            stats.largest_free_block = free_regions.iter().copied().max().unwrap_or(0);
            
            // Calculate fragmentation
            if stats.free_bytes > 0 {
                let num_free_regions = free_regions.len();
                if num_free_regions > 1 {
                    let avg_free = stats.free_bytes as f32 / num_free_regions as f32;
                    let variance: f32 = free_regions.iter()
                        .map(|&s| (s as f32 - avg_free).powi(2))
                        .sum::<f32>() / num_free_regions as f32;
                    stats.fragmentation = (variance / (avg_free * avg_free)).min(1.0);
                }
            }
            
            bank.stats = stats;
        }
        
        self.recalculate_total_stats();
    }
    
    /// Recalculate overall statistics
    fn recalculate_total_stats(&mut self) {
        let mut total = BankStats::default();
        
        for bank in self.banks.values() {
            total.used_bytes += bank.stats.used_bytes;
            total.free_bytes += bank.stats.free_bytes;
            
            for (ty, size) in &bank.stats.type_breakdown {
                *total.type_breakdown.entry(*ty).or_insert(0) += size;
            }
            
            total.largest_free_block = total.largest_free_block.max(bank.stats.largest_free_block);
        }
        
        // Average fragmentation
        if !self.banks.is_empty() {
            total.fragmentation = self.banks.values()
                .map(|b| b.stats.fragmentation)
                .sum::<f32>() / self.banks.len() as f32;
        }
        
        self.total_stats = total;
    }
    
    /// Get visualization data for the bank map
    pub fn get_visualization(&self) -> BankVisualization {
        let mut rows = Vec::new();
        
        for (bank_num, bank) in &self.banks {
            let mut segments = Vec::new();
            
            for region in &bank.regions {
                segments.push(BankSegment {
                    offset: region.offset,
                    size: region.size,
                    color: region.region_type.color(),
                    region_type: region.region_type,
                    description: region.description.clone(),
                });
            }
            
            rows.push(BankRow {
                bank_number: *bank_num,
                segments,
                used_percent: (bank.stats.used_bytes as f32 / bank.size as f32) * 100.0,
            });
        }
        
        BankVisualization {
            rows,
            total_stats: self.total_stats.clone(),
        }
    }
    
    /// Analyze fragmentation and suggest optimizations
    pub fn analyze_fragmentation(&self) -> FragmentationAnalysis {
        let mut movable_regions = Vec::new();
        let mut gaps = Vec::new();
        
        for (bank_num, bank) in &self.banks {
            let mut last_end = 0;
            
            for region in &bank.regions {
                if region.offset > last_end {
                    gaps.push(GapInfo {
                        bank: *bank_num,
                        offset: last_end,
                        size: region.offset - last_end,
                        can_merge: true,
                    });
                }
                
                if region.movable && !region.shared {
                    movable_regions.push(MovableRegion {
                        bank: *bank_num,
                        offset: region.offset,
                        size: region.size,
                        region_type: region.region_type,
                        description: region.description.clone(),
                    });
                }
                
                last_end = region.offset + region.size;
            }
        }
        
        // Generate defragmentation suggestions
        let mut suggestions = Vec::new();
        
        // Sort gaps by size (descending)
        gaps.sort_by(|a, b| b.size.cmp(&a.size));
        
        // Try to fit movable regions into larger gaps
        for gap in &gaps {
            let candidates: Vec<_> = movable_regions.iter()
                .filter(|r| r.size <= gap.size && r.bank != gap.bank)
                .cloned()
                .collect();
            
            if !candidates.is_empty() {
                let potential_savings: usize = candidates.iter().map(|r| r.size).sum();
                suggestions.push(DefragSuggestion {
                    gap: gap.clone(),
                    movable_regions: candidates,
                    potential_savings,
                });
            }
        }
        
        FragmentationAnalysis {
            overall_fragmentation: self.total_stats.fragmentation,
            gaps,
            movable_regions,
            suggestions,
        }
    }
}

/// Information about a free region
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreeRegion {
    pub bank: u8,
    pub offset: usize,
    pub size: usize,
    pub pc_offset: usize,
}

/// Visualization data for a bank
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankVisualization {
    pub rows: Vec<BankRow>,
    pub total_stats: BankStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankRow {
    pub bank_number: u8,
    pub segments: Vec<BankSegment>,
    pub used_percent: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BankSegment {
    pub offset: usize,
    pub size: usize,
    pub color: [u8; 4],
    pub region_type: RegionType,
    pub description: Option<String>,
}

/// Information about a gap (contiguous free space)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GapInfo {
    pub bank: u8,
    pub offset: usize,
    pub size: usize,
    pub can_merge: bool,
}

/// Information about a movable region
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovableRegion {
    pub bank: u8,
    pub offset: usize,
    pub size: usize,
    pub region_type: RegionType,
    pub description: Option<String>,
}

/// Analysis of fragmentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FragmentationAnalysis {
    pub overall_fragmentation: f32,
    pub gaps: Vec<GapInfo>,
    pub movable_regions: Vec<MovableRegion>,
    pub suggestions: Vec<DefragSuggestion>,
}

/// Defragmentation suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefragSuggestion {
    pub gap: GapInfo,
    pub movable_regions: Vec<MovableRegion>,
    pub potential_savings: usize,
}

/// Bank defragmentation planner
pub struct DefragmentationPlanner {
    pub plan: Vec<MoveOperation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveOperation {
    pub source_bank: u8,
    pub source_offset: usize,
    pub dest_bank: u8,
    pub dest_offset: usize,
    pub size: usize,
    pub description: String,
}

impl DefragmentationPlanner {
    /// Create a defragmentation plan from analysis
    pub fn from_analysis(analysis: &FragmentationAnalysis) -> Self {
        let mut plan = Vec::new();
        
        for suggestion in &analysis.suggestions {
            let mut remaining_space = suggestion.gap.size;
            let mut current_offset = suggestion.gap.offset;
            
            for region in &suggestion.movable_regions {
                if region.size <= remaining_space {
                    plan.push(MoveOperation {
                        source_bank: region.bank,
                        source_offset: region.offset,
                        dest_bank: suggestion.gap.bank,
                        dest_offset: current_offset,
                        size: region.size,
                        description: region.description.clone().unwrap_or_else(|| "Move".into()),
                    });
                    
                    remaining_space -= region.size;
                    current_offset += region.size;
                }
            }
        }
        
        Self { plan }
    }
    
    /// Estimate the safety of this plan
    pub fn estimate_safety(&self) -> SafetyRating {
        let moves = self.plan.len();
        
        if moves == 0 {
            SafetyRating::Safe
        } else if moves <= 3 {
            SafetyRating::LowRisk
        } else if moves <= 10 {
            SafetyRating::MediumRisk
        } else {
            SafetyRating::HighRisk
        }
    }
    
    /// Generate a summary of the plan
    pub fn summary(&self) -> String {
        let total_bytes: usize = self.plan.iter().map(|m| m.size).sum();
        format!(
            "Defragmentation Plan: {} operations, {} bytes to move",
            self.plan.len(),
            total_bytes
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SafetyRating {
    Safe,
    LowRisk,
    MediumRisk,
    HighRisk,
    Dangerous,
}

impl SafetyRating {
    pub fn display_name(&self) -> &'static str {
        match self {
            SafetyRating::Safe => "Safe",
            SafetyRating::LowRisk => "Low Risk",
            SafetyRating::MediumRisk => "Medium Risk",
            SafetyRating::HighRisk => "High Risk",
            SafetyRating::Dangerous => "Dangerous",
        }
    }
    
    pub fn color(&self) -> [u8; 3] {
        match self {
            SafetyRating::Safe => [50, 200, 50],
            SafetyRating::LowRisk => [150, 200, 50],
            SafetyRating::MediumRisk => [200, 200, 50],
            SafetyRating::HighRisk => [200, 100, 50],
            SafetyRating::Dangerous => [200, 50, 50],
        }
    }
}
