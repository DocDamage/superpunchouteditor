use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Result of AI simulation/testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationResult {
    /// Average fight duration in seconds
    pub average_fight_time: f32,
    /// Average damage player takes per fight
    pub player_damage_taken: f32,
    /// Average damage AI deals per fight
    pub ai_damage_dealt: f32,
    /// Pattern usage statistics (pattern_id -> usage percentage)
    pub pattern_usage: HashMap<String, f32>,
    /// Calculated difficulty rating
    pub difficulty_rating: DifficultyRating,
    /// Estimated win rate against average player (0-100)
    pub estimated_win_rate: f32,
    /// Key insights from simulation
    pub insights: Vec<String>,
    /// Warnings about potential issues
    pub warnings: Vec<String>,
}

/// Difficulty rating categories
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub enum DifficultyRating {
    VeryEasy,
    Easy,
    Medium,
    Hard,
    VeryHard,
    Extreme,
}

impl DifficultyRating {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::VeryEasy => "Very Easy",
            Self::Easy => "Easy",
            Self::Medium => "Medium",
            Self::Hard => "Hard",
            Self::VeryHard => "Very Hard",
            Self::Extreme => "Extreme",
        }
    }

    pub fn color(&self) -> &'static str {
        match self {
            Self::VeryEasy => "#4ade80", // green
            Self::Easy => "#60a5fa",     // blue
            Self::Medium => "#fbbf24",   // yellow
            Self::Hard => "#fb923c",     // orange
            Self::VeryHard => "#f87171", // red
            Self::Extreme => "#a855f7",  // purple
        }
    }
}
