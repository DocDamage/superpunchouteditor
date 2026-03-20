use super::types::*;
use super::AiBehavior;

/// Preset AI behavior templates
pub struct AiPresets;

impl AiPresets {
    /// Create a beginner-friendly AI (low aggression, predictable)
    pub fn beginner() -> AiBehavior {
        AiBehavior {
            fighter_name: "Beginner Template".to_string(),
            attack_patterns: vec![AttackPattern {
                id: "slow_jab".to_string(),
                name: "Slow Jab".to_string(),
                sequence: vec![AttackMove {
                    move_type: MoveType::LeftJab,
                    windup_frames: 20,
                    active_frames: 8,
                    recovery_frames: 30,
                    damage: 8,
                    stun: 3,
                    ..Default::default()
                }],
                frequency: 40,
                difficulty_max: 80,
                ..Default::default()
            }],
            defense_behaviors: vec![DefenseBehavior {
                behavior_type: DefenseType::BlockHigh,
                frequency: 60,
                success_rate: 180,
                ..Default::default()
            }],
            difficulty_curve: DifficultyCurve {
                rounds: vec![
                    RoundDifficulty {
                        round: 1,
                        aggression: 60,
                        defense: 70,
                        speed: 90,
                        pattern_complexity: 30,
                        damage_multiplier: 80,
                        reaction_time: 10,
                    },
                    RoundDifficulty {
                        round: 2,
                        aggression: 70,
                        defense: 80,
                        speed: 95,
                        pattern_complexity: 50,
                        damage_multiplier: 90,
                        reaction_time: 9,
                    },
                    RoundDifficulty {
                        round: 3,
                        aggression: 80,
                        defense: 90,
                        speed: 100,
                        pattern_complexity: 70,
                        damage_multiplier: 100,
                        reaction_time: 8,
                    },
                ],
                base_aggression: 60,
                base_defense: 70,
                base_speed: 90,
            },
            ..Default::default()
        }
    }

    /// Create a challenging AI (high aggression, complex patterns)
    pub fn challenging() -> AiBehavior {
        use MoveType::*;

        AiBehavior {
            fighter_name: "Challenging Template".to_string(),
            attack_patterns: vec![
                AttackPattern {
                    id: "quick_jab".to_string(),
                    name: "Quick Jab".to_string(),
                    sequence: vec![AttackMove {
                        move_type: LeftJab,
                        windup_frames: 10,
                        active_frames: 6,
                        recovery_frames: 15,
                        damage: 12,
                        stun: 5,
                        ..Default::default()
                    }],
                    frequency: 80,
                    ..Default::default()
                },
                AttackPattern {
                    id: "hook_combo".to_string(),
                    name: "Hook Combo".to_string(),
                    sequence: vec![
                        AttackMove {
                            move_type: LeftHook,
                            windup_frames: 15,
                            active_frames: 8,
                            recovery_frames: 20,
                            damage: 18,
                            stun: 8,
                            ..Default::default()
                        },
                        AttackMove {
                            move_type: RightUppercut,
                            windup_frames: 18,
                            active_frames: 10,
                            recovery_frames: 25,
                            damage: 22,
                            stun: 12,
                            ..Default::default()
                        },
                    ],
                    frequency: 50,
                    difficulty_min: 50,
                    ..Default::default()
                },
            ],
            defense_behaviors: vec![
                DefenseBehavior {
                    behavior_type: DefenseType::DodgeLeft,
                    frequency: 60,
                    success_rate: 200,
                    leads_to_counter: true,
                    ..Default::default()
                },
                DefenseBehavior {
                    behavior_type: DefenseType::Counter,
                    frequency: 40,
                    success_rate: 180,
                    ..Default::default()
                },
            ],
            difficulty_curve: DifficultyCurve {
                rounds: vec![
                    RoundDifficulty::default_round(1),
                    RoundDifficulty {
                        round: 2,
                        aggression: 130,
                        defense: 120,
                        speed: 115,
                        pattern_complexity: 120,
                        damage_multiplier: 110,
                        reaction_time: 5,
                    },
                    RoundDifficulty {
                        round: 3,
                        aggression: 160,
                        defense: 140,
                        speed: 130,
                        pattern_complexity: 180,
                        damage_multiplier: 125,
                        reaction_time: 3,
                    },
                ],
                base_aggression: 120,
                base_defense: 110,
                base_speed: 115,
            },
            ..Default::default()
        }
    }
}
