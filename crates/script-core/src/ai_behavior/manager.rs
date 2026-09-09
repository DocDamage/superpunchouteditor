use std::collections::HashMap;

use super::constants::*;
use super::parser::{AiParseError, AiParser};
use super::simulation::{DifficultyRating, SimulationResult};
use super::types::*;
use super::AiBehavior;

/// AI behavior manager for working with multiple fighters
pub struct AiBehaviorManager;

impl AiBehaviorManager {
    /// Load AI behavior for all fighters
    pub fn load_all(rom: &[u8]) -> Result<Vec<AiBehavior>, AiParseError> {
        let mut behaviors = Vec::with_capacity(MAX_FIGHTERS);
        for id in 0..MAX_FIGHTERS {
            behaviors.push(AiParser::parse_from_rom(rom, id)?);
        }
        Ok(behaviors)
    }

    /// Run a simple simulation to test AI behavior
    ///
    /// # Arguments
    /// * `behavior` - The AI behavior to simulate
    /// * `iterations` - Number of simulated fights
    ///
    /// # Returns
    /// Simulation results with statistics
    pub fn simulate(behavior: &AiBehavior, _iterations: u32) -> SimulationResult {
        // Simple simulation model
        let total_patterns = behavior.attack_patterns.len().max(1) as f32;
        let avg_damage: f32 = behavior
            .attack_patterns
            .iter()
            .map(|p| p.sequence.iter().map(|m| m.damage as f32).sum::<f32>())
            .sum::<f32>()
            / total_patterns;

        let avg_frequency: f32 = behavior
            .attack_patterns
            .iter()
            .map(|p| p.frequency as f32)
            .sum::<f32>()
            / total_patterns;

        let difficulty_curve = &behavior.difficulty_curve;
        let round3_aggression = difficulty_curve
            .rounds
            .get(2)
            .map(|r| r.aggression as f32)
            .unwrap_or(100.0);

        // Calculate estimated fight time based on aggression
        let base_time = 180.0f32; // 3 minutes max
        let aggression_factor = (avg_frequency * round3_aggression) / 10000.0;
        let avg_fight_time = base_time / (1.0 + aggression_factor);

        // Calculate damage estimates
        let player_damage = avg_damage * aggression_factor * 10.0;

        // Determine difficulty rating
        let difficulty_rating = if aggression_factor < 0.5 {
            DifficultyRating::Easy
        } else if aggression_factor < 1.0 {
            DifficultyRating::Medium
        } else if aggression_factor < 1.5 {
            DifficultyRating::Hard
        } else if aggression_factor < 2.0 {
            DifficultyRating::VeryHard
        } else {
            DifficultyRating::Extreme
        };

        // Pattern usage distribution
        let total_weight: f32 = behavior
            .attack_patterns
            .iter()
            .map(|p| p.weight as f32)
            .sum();
        let pattern_usage: HashMap<String, f32> = behavior
            .attack_patterns
            .iter()
            .map(|p| {
                let usage = if total_weight > 0.0 {
                    (p.weight as f32 / total_weight) * 100.0
                } else {
                    100.0 / total_patterns
                };
                (p.id.clone(), usage)
            })
            .collect();

        // Generate insights
        let mut insights = Vec::new();
        let mut warnings = Vec::new();

        if behavior.attack_patterns.len() < 3 {
            warnings.push("Few attack patterns - AI may be predictable".to_string());
        }

        if avg_frequency > 200.0 {
            warnings.push("Very high attack frequency - may be frustrating".to_string());
        }

        if round3_aggression > 200.0 {
            insights.push("Significant difficulty spike in round 3".to_string());
        }

        // Estimated win rate (simplified model)
        let estimated_win_rate = (aggression_factor * 30.0).min(95.0);

        SimulationResult {
            average_fight_time: avg_fight_time,
            player_damage_taken: player_damage,
            ai_damage_dealt: player_damage * 0.7, // AI typically takes less damage
            pattern_usage,
            difficulty_rating,
            estimated_win_rate,
            insights,
            warnings,
        }
    }

    /// Validate AI behavior for potential issues
    pub fn validate(behavior: &AiBehavior) -> Vec<String> {
        let mut issues = Vec::new();

        // Check for patterns without moves
        for pattern in &behavior.attack_patterns {
            if pattern.sequence.is_empty() {
                issues.push(format!("Pattern '{}' has no moves", pattern.id));
            }
            if pattern.frequency == 0 {
                issues.push(format!(
                    "Pattern '{}' has 0 frequency (unusable)",
                    pattern.id
                ));
            }
        }

        // Check for duplicate IDs
        let mut seen_ids = std::collections::HashSet::new();
        for pattern in &behavior.attack_patterns {
            if !seen_ids.insert(&pattern.id) {
                issues.push(format!("Duplicate pattern ID: {}", pattern.id));
            }
        }

        // Check difficulty curve
        if behavior.difficulty_curve.rounds.len() != 3 {
            issues.push("Difficulty curve should have exactly 3 rounds".to_string());
        }

        // Check for triggers referencing non-existent patterns
        for trigger in &behavior.triggers {
            if let AiAction::UsePattern(pattern_id) = &trigger.action {
                if !behavior.attack_patterns.iter().any(|p| &p.id == pattern_id) {
                    issues.push(format!(
                        "Trigger references non-existent pattern '{}'",
                        pattern_id
                    ));
                }
            }
        }

        issues
    }
}
