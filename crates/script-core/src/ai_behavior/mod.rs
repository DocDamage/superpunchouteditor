//! AI Behavior System for Super Punch-Out!!
//!
//! This module provides data structures and parsers for boxer AI behavior.
//!
//! ## AI Table Addresses (Bank $0B)
//!
//! Based on SPO disassembly research, the AI data is located in Bank $0B:
//!
//! | Address | Description |
//! |---------|-------------|
//! | $0B:8000 | AI Table Base - Fighter AI headers |
//! | $0B:8200 | AI Pattern Table - Attack patterns |
//! | $0B:8800 | AI Defense Table - Defense behaviors |
//! | $0B:9000 | AI Trigger Table - Condition triggers |
//!
//! ## Fighter Header Structure (Bank $09)
//!
//! Each fighter has a 32-byte header at $09:8000 + (fighter_id * 0x20):
//! - Offset +0: Palette ID
//! - Offset +1: Attack power
//! - Offset +2: Defense rating
//! - Offset +3: Speed rating
//! - Offset +6: Pose table pointer (SNES addr)
//! - Offset +8: AI script pointer (SNES addr)
//! - Offset +10: Corner man pointer (SNES addr)

use serde::{Deserialize, Serialize};

pub mod constants;
pub mod manager;
pub mod parser;
pub mod presets;
pub mod simulation;
pub mod types;

// Re-export all public items to maintain the same API
pub use constants::*;
pub use manager::AiBehaviorManager;
pub use parser::{AiParseError, AiParser};
pub use presets::AiPresets;
pub use simulation::{DifficultyRating, SimulationResult};
pub use types::*;

/// Complete AI behavior profile for a boxer
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AiBehavior {
    /// Fighter identifier (0-15 for main roster)
    pub fighter_id: usize,
    /// Human-readable name
    pub fighter_name: String,
    /// Available attack patterns
    pub attack_patterns: Vec<AttackPattern>,
    /// Defense behavior configurations
    pub defense_behaviors: Vec<DefenseBehavior>,
    /// Round-by-round difficulty scaling
    pub difficulty_curve: DifficultyCurve,
    /// Special triggers and reactions
    pub triggers: Vec<AiTrigger>,
    /// Raw bytes from ROM (for debugging/research)
    pub raw_bytes: Vec<u8>,
    /// PC offset in ROM where this data is stored
    pub pc_offset: Option<usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_move_type_display() {
        assert_eq!(MoveType::LeftJab.display_name(), "Left Jab");
        assert_eq!(MoveType::Special.display_name(), "Special Move");
    }

    #[test]
    fn test_default_attack_pattern() {
        let pattern = AttackPattern::default();
        assert_eq!(pattern.frequency, 50);
        assert!(pattern.available_round_1);
        assert!(pattern.available_round_2);
        assert!(pattern.available_round_3);
    }

    #[test]
    fn test_ai_parser_generates_patterns() {
        // Create a minimal ROM buffer for testing
        let mut rom = vec![0u8; 0x060000]; // Large enough for AI tables

        // Set up minimal AI data for fighter 0
        let ai_ptr: u16 = 0x8200;
        let header_addr = FIGHTER_HEADER_BASE + 8; // AI pointer offset
        rom[header_addr] = (ai_ptr & 0xFF) as u8;
        rom[header_addr + 1] = (ai_ptr >> 8) as u8;

        // Set pattern count to 1
        rom[AI_PATTERN_TABLE] = 1;
        // Set pattern data
        rom[AI_PATTERN_TABLE + 1] = 0x00; // Left Jab
        rom[AI_PATTERN_TABLE + 2] = 15; // windup
        rom[AI_PATTERN_TABLE + 3] = 8; // active
        rom[AI_PATTERN_TABLE + 4] = 20; // recovery
        rom[AI_PATTERN_TABLE + 5] = 10; // damage
        rom[AI_PATTERN_TABLE + 6] = 5; // stun

        let behavior = AiParser::parse_from_rom(&rom, 0).expect("Should parse AI");
        assert!(!behavior.attack_patterns.is_empty());
    }

    #[test]
    fn test_ai_parse_error_display() {
        let err = AiParseError::InvalidFighterId(20);
        assert!(err.to_string().contains("Invalid fighter ID"));

        let err = AiParseError::RomTooSmall;
        assert!(err.to_string().contains("too small"));
    }

    #[test]
    fn test_move_type_roundtrip() {
        use MoveType::*;
        let types = vec![
            LeftJab,
            RightJab,
            LeftHook,
            RightHook,
            LeftUppercut,
            RightUppercut,
            Special,
            Taunt,
        ];

        for mt in types {
            let byte = AiParser::move_type_to_byte_pub(&mt);
            let parsed = AiParser::parse_move_type_pub(byte);
            assert_eq!(mt, parsed);
        }
    }

    #[test]
    fn test_serialize_to_bytes() {
        let behavior = AiPresets::beginner();
        let bytes = AiParser::serialize_to_bytes(&behavior, 0).expect("Should serialize");
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_get_ai_offset() {
        let offset = AiParser::get_ai_offset(0).expect("Should get offset");
        assert_eq!(offset, AI_TABLE_BASE);

        let offset = AiParser::get_ai_offset(15).expect("Should get offset");
        assert_eq!(offset, AI_TABLE_BASE + 0xF00);

        assert!(AiParser::get_ai_offset(16).is_err());
    }

    #[test]
    fn test_simulation_runs() {
        let behavior = AiPresets::beginner();
        let result = AiBehaviorManager::simulate(&behavior, 100);
        assert!(result.average_fight_time > 0.0);
        assert!(result.estimated_win_rate >= 0.0 && result.estimated_win_rate <= 100.0);
    }

    #[test]
    fn test_validation_catches_issues() {
        let mut behavior = AiBehavior::default();
        behavior.attack_patterns.push(AttackPattern {
            id: "empty".to_string(),
            name: "Empty Pattern".to_string(),
            sequence: vec![], // Empty sequence - should warn
            ..Default::default()
        });

        let issues = AiBehaviorManager::validate(&behavior);
        assert!(!issues.is_empty());
        assert!(issues.iter().any(|i| i.contains("no moves")));
    }
}
