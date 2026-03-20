use crate::BoxerRecord;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashSet;

/// Simple RGB color for palette comparison
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

/// Complete comparison between two boxers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoxerComparison {
    pub boxer_a: String,
    pub boxer_b: String,
    pub boxer_a_key: String,
    pub boxer_b_key: String,
    pub stat_comparison: StatComparison,
    pub asset_comparison: AssetComparison,
    pub palette_comparison: Vec<PaletteComparison>,
    pub similarity_score: f32,
}

/// Statistical comparison between two boxers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatComparison {
    pub attack: (u8, u8),
    pub defense: (u8, u8),
    pub speed: (u8, u8),
    pub palette_id: (u8, u8),
    pub differences: Vec<String>,
}

impl StatComparison {
    pub fn new() -> Self {
        Self {
            attack: (0, 0),
            defense: (0, 0),
            speed: (0, 0),
            palette_id: (0, 0),
            differences: Vec::new(),
        }
    }

    pub fn identify_differences(&mut self) {
        self.differences.clear();
        if self.attack.0 != self.attack.1 {
            self.differences.push("attack".to_string());
        }
        if self.defense.0 != self.defense.1 {
            self.differences.push("defense".to_string());
        }
        if self.speed.0 != self.speed.1 {
            self.differences.push("speed".to_string());
        }
        if self.palette_id.0 != self.palette_id.1 {
            self.differences.push("palette_id".to_string());
        }
    }
}

impl Default for StatComparison {
    fn default() -> Self {
        Self::new()
    }
}

/// Asset comparison between two boxers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetComparison {
    pub unique_a: Vec<String>,
    pub unique_b: Vec<String>,
    pub shared: Vec<String>,
    pub total_size_a: usize,
    pub total_size_b: usize,
    pub unique_count_a: usize,
    pub unique_count_b: usize,
    pub shared_count: usize,
}

impl AssetComparison {
    pub fn new() -> Self {
        Self {
            unique_a: Vec::new(),
            unique_b: Vec::new(),
            shared: Vec::new(),
            total_size_a: 0,
            total_size_b: 0,
            unique_count_a: 0,
            unique_count_b: 0,
            shared_count: 0,
        }
    }
}

impl Default for AssetComparison {
    fn default() -> Self {
        Self::new()
    }
}

/// Palette comparison between two boxers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaletteComparison {
    pub name: String,
    pub file_a: String,
    pub file_b: String,
    pub size_a: usize,
    pub size_b: usize,
    pub color_count_a: usize,
    pub color_count_b: usize,
    pub differences: Vec<usize>,
}

/// Boxer similarity information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoxerSimilarity {
    pub boxer_name: String,
    pub boxer_key: String,
    pub similarity_score: f32,
    pub similarity_percentage: u8,
    pub reason: String,
}

/// Stat field for copy operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StatField {
    Attack,
    Defense,
    Speed,
    PaletteId,
}

impl StatField {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "attack" => Some(Self::Attack),
            "defense" => Some(Self::Defense),
            "speed" => Some(Self::Speed),
            "palette_id" | "palette" => Some(Self::PaletteId),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Attack => "attack",
            Self::Defense => "defense",
            Self::Speed => "speed",
            Self::PaletteId => "palette_id",
        }
    }
}

impl BoxerComparison {
    /// Compare two boxers and return a comprehensive comparison
    pub fn compare(
        boxer_a: &BoxerRecord,
        boxer_b: &BoxerRecord,
        stats_a: Option<&FighterStats>,
        stats_b: Option<&FighterStats>,
    ) -> Self {
        let stat_comparison = Self::compare_stats(stats_a, stats_b);
        let asset_comparison = Self::compare_assets(boxer_a, boxer_b);
        let palette_comparison = Self::compare_palettes(boxer_a, boxer_b);
        let similarity_score =
            Self::calculate_similarity(boxer_a, boxer_b, stats_a, stats_b, &asset_comparison);

        Self {
            boxer_a: boxer_a.name.clone(),
            boxer_b: boxer_b.name.clone(),
            boxer_a_key: boxer_a.key.clone(),
            boxer_b_key: boxer_b.key.clone(),
            stat_comparison,
            asset_comparison,
            palette_comparison,
            similarity_score,
        }
    }

    fn compare_stats(
        stats_a: Option<&FighterStats>,
        stats_b: Option<&FighterStats>,
    ) -> StatComparison {
        let mut comparison = StatComparison::new();

        if let (Some(a), Some(b)) = (stats_a, stats_b) {
            comparison.attack = (a.attack_power, b.attack_power);
            comparison.defense = (a.defense_rating, b.defense_rating);
            comparison.speed = (a.speed_rating, b.speed_rating);
            comparison.palette_id = (a.palette_id, b.palette_id);
        }

        comparison.identify_differences();
        comparison
    }

    fn compare_assets(boxer_a: &BoxerRecord, boxer_b: &BoxerRecord) -> AssetComparison {
        let mut comparison = AssetComparison::new();

        // Collect all unique bins
        let unique_a: HashSet<String> = boxer_a
            .unique_sprite_bins
            .iter()
            .map(|a| a.file.clone())
            .collect();
        let unique_b: HashSet<String> = boxer_b
            .unique_sprite_bins
            .iter()
            .map(|b| b.file.clone())
            .collect();

        // Collect shared bins
        let shared_a: HashSet<String> = boxer_a
            .shared_sprite_bins
            .iter()
            .map(|a| a.file.clone())
            .collect();
        let shared_b: HashSet<String> = boxer_b
            .shared_sprite_bins
            .iter()
            .map(|b| b.file.clone())
            .collect();

        // Unique to A
        for file in &unique_a {
            if !unique_b.contains(file) {
                comparison.unique_a.push(file.clone());
            }
        }

        // Unique to B
        for file in &unique_b {
            if !unique_a.contains(file) {
                comparison.unique_b.push(file.clone());
            }
        }

        // Shared between both (same filename)
        let all_shared: HashSet<String> = shared_a.intersection(&shared_b).cloned().collect();
        comparison.shared = all_shared.into_iter().collect();

        // Calculate sizes
        comparison.total_size_a = boxer_a
            .unique_sprite_bins
            .iter()
            .map(|b| b.size)
            .sum::<usize>()
            + boxer_a
                .shared_sprite_bins
                .iter()
                .map(|b| b.size)
                .sum::<usize>();
        comparison.total_size_b = boxer_b
            .unique_sprite_bins
            .iter()
            .map(|b| b.size)
            .sum::<usize>()
            + boxer_b
                .shared_sprite_bins
                .iter()
                .map(|b| b.size)
                .sum::<usize>();

        comparison.unique_count_a = boxer_a.unique_sprite_bins.len();
        comparison.unique_count_b = boxer_b.unique_sprite_bins.len();
        comparison.shared_count = boxer_a.shared_sprite_bins.len();

        comparison
    }

    fn compare_palettes(boxer_a: &BoxerRecord, boxer_b: &BoxerRecord) -> Vec<PaletteComparison> {
        let mut comparisons = Vec::new();

        // Compare palettes by position
        let max_palettes = boxer_a.palette_files.len().max(boxer_b.palette_files.len());

        for i in 0..max_palettes {
            let pal_a = boxer_a.palette_files.get(i);
            let pal_b = boxer_b.palette_files.get(i);

            let name = format!("Palette {}", i + 1);

            comparisons.push(PaletteComparison {
                name,
                file_a: pal_a.map(|p| p.file.clone()).unwrap_or_default(),
                file_b: pal_b.map(|p| p.file.clone()).unwrap_or_default(),
                size_a: pal_a.map(|p| p.size).unwrap_or(0),
                size_b: pal_b.map(|p| p.size).unwrap_or(0),
                color_count_a: pal_a.map(|p| p.size / 2).unwrap_or(0),
                color_count_b: pal_b.map(|p| p.size / 2).unwrap_or(0),
                differences: Vec::new(), // Would need actual palette data to compare
            });
        }

        comparisons
    }

    fn calculate_similarity(
        boxer_a: &BoxerRecord,
        boxer_b: &BoxerRecord,
        stats_a: Option<&FighterStats>,
        stats_b: Option<&FighterStats>,
        _asset_comparison: &AssetComparison,
    ) -> f32 {
        let mut score = 0.0;
        let mut total = 0.0;

        // Compare stats (weight: 40%)
        if let (Some(a), Some(b)) = (stats_a, stats_b) {
            let attack_diff = (a.attack_power as f32 - b.attack_power as f32).abs();
            let defense_diff = (a.defense_rating as f32 - b.defense_rating as f32).abs();
            let speed_diff = (a.speed_rating as f32 - b.speed_rating as f32).abs();

            score += 40.0 * (1.0 - attack_diff / 255.0);
            score += 40.0 * (1.0 - defense_diff / 255.0);
            score += 40.0 * (1.0 - speed_diff / 255.0);
            total += 120.0;
        }

        // Compare asset counts (weight: 20%)
        let unique_diff = (boxer_a.unique_sprite_bins.len() as f32
            - boxer_b.unique_sprite_bins.len() as f32)
            .abs();
        score += 20.0 * (1.0 - (unique_diff / 50.0).min(1.0));
        total += 20.0;

        // Shared assets similarity (weight: 20%)
        if !boxer_a.shared_sprite_bins.is_empty() || !boxer_b.shared_sprite_bins.is_empty() {
            let shared_a: HashSet<_> = boxer_a.shared_sprite_bins.iter().map(|b| &b.file).collect();
            let shared_b: HashSet<_> = boxer_b.shared_sprite_bins.iter().map(|b| &b.file).collect();

            if !shared_a.is_empty() || !shared_b.is_empty() {
                let common = shared_a.intersection(&shared_b).count() as f32;
                let max_shared = shared_a.len().max(shared_b.len()) as f32;
                if max_shared > 0.0 {
                    score += 20.0 * (common / max_shared);
                } else {
                    score += 20.0;
                }
                total += 20.0;
            }
        }

        // Palette count similarity (weight: 10%)
        let palette_diff =
            (boxer_a.palette_files.len() as f32 - boxer_b.palette_files.len() as f32).abs();
        score += 10.0 * (1.0 - (palette_diff / 5.0).min(1.0));
        total += 10.0;

        // Both have no shared bins (safe editing targets) (weight: 10%)
        if boxer_a.shared_sprite_bins.is_empty() && boxer_b.shared_sprite_bins.is_empty() {
            score += 10.0;
        } else if boxer_a.shared_sprite_bins.is_empty() || boxer_b.shared_sprite_bins.is_empty() {
            score += 5.0;
        }
        total += 10.0;

        if total > 0.0 {
            score / total
        } else {
            0.0
        }
    }
}

/// Fighter stats for comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FighterStats {
    pub attack_power: u8,
    pub defense_rating: u8,
    pub speed_rating: u8,
    pub palette_id: u8,
}

impl FighterStats {
    pub fn new(attack: u8, defense: u8, speed: u8, palette: u8) -> Self {
        Self {
            attack_power: attack,
            defense_rating: defense,
            speed_rating: speed,
            palette_id: palette,
        }
    }
}

/// Find similar boxers to a reference boxer
pub fn find_similar_boxers(
    reference: &BoxerRecord,
    all_boxers: &[BoxerRecord],
    reference_stats: Option<&FighterStats>,
    all_stats: &[(String, FighterStats)],
    limit: usize,
) -> Vec<BoxerSimilarity> {
    let mut similarities: Vec<BoxerSimilarity> = Vec::new();

    for boxer in all_boxers {
        if boxer.key == reference.key {
            continue;
        }

        let boxer_stats = all_stats
            .iter()
            .find(|(key, _)| key == &boxer.key)
            .map(|(_, stats)| stats);

        let asset_comparison = BoxerComparison::compare_assets(reference, boxer);
        let similarity_score = BoxerComparison::calculate_similarity(
            reference,
            boxer,
            reference_stats,
            boxer_stats,
            &asset_comparison,
        );

        let reason = generate_similarity_reason(reference, boxer, &asset_comparison);

        similarities.push(BoxerSimilarity {
            boxer_name: boxer.name.clone(),
            boxer_key: boxer.key.clone(),
            similarity_score,
            similarity_percentage: (similarity_score * 100.0) as u8,
            reason,
        });
    }

    // Sort by similarity score (descending)
    // Using unwrap_or(Ordering::Equal) to handle potential NaN values safely
    similarities.sort_by(|a, b| {
        b.similarity_score
            .partial_cmp(&a.similarity_score)
            .unwrap_or(Ordering::Equal)
    });

    // Limit results
    similarities.truncate(limit);

    similarities
}

fn generate_similarity_reason(
    reference: &BoxerRecord,
    boxer: &BoxerRecord,
    asset_comparison: &AssetComparison,
) -> String {
    let mut reasons = Vec::new();

    // Check if they share all graphics
    if asset_comparison.unique_a.is_empty() && asset_comparison.unique_b.is_empty() {
        reasons.push("Shares all graphics".to_string());
    }

    // Check complexity similarity
    let unique_diff = (reference.unique_sprite_bins.len() as isize
        - boxer.unique_sprite_bins.len() as isize)
        .abs();
    if unique_diff <= 5 {
        reasons.push("Similar complexity".to_string());
    }

    // Check palette count
    if reference.palette_files.len() == boxer.palette_files.len() {
        reasons.push("Same palette count".to_string());
    }

    // Check if both have no shared bins
    if reference.shared_sprite_bins.is_empty() && boxer.shared_sprite_bins.is_empty() {
        reasons.push("Both safe to edit".to_string());
    }

    if reasons.is_empty() {
        "Different style".to_string()
    } else {
        reasons.join(", ")
    }
}

/// Compare two palettes and return indices of different colors
pub fn compare_palette_colors(colors_a: &[Color], colors_b: &[Color]) -> Vec<usize> {
    let min_len = colors_a.len().min(colors_b.len());
    let mut differences = Vec::new();

    for i in 0..min_len {
        if colors_a[i].r != colors_b[i].r
            || colors_a[i].g != colors_b[i].g
            || colors_a[i].b != colors_b[i].b
        {
            differences.push(i);
        }
    }

    differences
}
