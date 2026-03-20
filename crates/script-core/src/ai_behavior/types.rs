use serde::{Deserialize, Serialize};

/// An attack pattern is a sequence of moves with conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttackPattern {
    /// Unique identifier for this pattern
    pub id: String,
    /// Human-readable name (e.g., "Left Hook Combo")
    pub name: String,
    /// Sequence of moves in this pattern
    pub sequence: Vec<AttackMove>,
    /// Base frequency: 0-255 chance per frame of initiating
    pub frequency: u8,
    /// Conditions that must be met for this pattern to be available
    pub conditions: Vec<Condition>,
    /// Minimum difficulty level for this pattern (0-255)
    pub difficulty_min: u8,
    /// Maximum difficulty level for this pattern (0-255)
    pub difficulty_max: u8,
    /// Whether this pattern can be used in round 1
    pub available_round_1: bool,
    /// Whether this pattern can be used in round 2
    pub available_round_2: bool,
    /// Whether this pattern can be used in round 3
    pub available_round_3: bool,
    /// Pattern weight for random selection (higher = more likely)
    pub weight: u8,
}

impl Default for AttackPattern {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            sequence: Vec::new(),
            frequency: 50,
            conditions: Vec::new(),
            difficulty_min: 0,
            difficulty_max: 255,
            available_round_1: true,
            available_round_2: true,
            available_round_3: true,
            weight: 10,
        }
    }
}

/// A single move within an attack pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttackMove {
    /// Type of punch/move
    pub move_type: MoveType,
    /// Windup frames before the punch becomes active
    pub windup_frames: u8,
    /// Active frames where the punch can connect
    pub active_frames: u8,
    /// Recovery frames after the punch
    pub recovery_frames: u8,
    /// Damage dealt on hit
    pub damage: u8,
    /// Stun amount dealt on hit
    pub stun: u8,
    /// Hitbox for this move
    pub hitbox: Hitbox,
    /// Animation/pose ID to use
    pub pose_id: u8,
    /// Sound effect to play
    pub sound_effect: Option<u8>,
}

impl Default for AttackMove {
    fn default() -> Self {
        Self {
            move_type: MoveType::LeftJab,
            windup_frames: 12,
            active_frames: 8,
            recovery_frames: 20,
            damage: 10,
            stun: 5,
            hitbox: Hitbox::default(),
            pose_id: 0,
            sound_effect: None,
        }
    }
}

/// Types of punches and moves available to boxers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MoveType {
    LeftJab,
    RightJab,
    LeftHook,
    RightHook,
    LeftUppercut,
    RightUppercut,
    /// Boxer-specific special move
    Special,
    /// Taunt/distract animation
    Taunt,
    /// Movement step
    StepLeft,
    StepRight,
    StepForward,
    StepBack,
}

impl MoveType {
    /// Get a human-readable display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::LeftJab => "Left Jab",
            Self::RightJab => "Right Jab",
            Self::LeftHook => "Left Hook",
            Self::RightHook => "Right Hook",
            Self::LeftUppercut => "Left Uppercut",
            Self::RightUppercut => "Right Uppercut",
            Self::Special => "Special Move",
            Self::Taunt => "Taunt",
            Self::StepLeft => "Step Left",
            Self::StepRight => "Step Right",
            Self::StepForward => "Step Forward",
            Self::StepBack => "Step Back",
        }
    }

    /// Get an icon/emoji representation
    pub fn icon(&self) -> &'static str {
        match self {
            Self::LeftJab => "🥊",
            Self::RightJab => "🥊",
            Self::LeftHook => "🪝",
            Self::RightHook => "🪝",
            Self::LeftUppercut => "⬆️",
            Self::RightUppercut => "⬆️",
            Self::Special => "⭐",
            Self::Taunt => "😤",
            Self::StepLeft => "⬅️",
            Self::StepRight => "➡️",
            Self::StepForward => "⬆️",
            Self::StepBack => "⬇️",
        }
    }

    /// Check if this is a left-handed move
    pub fn is_left(&self) -> bool {
        matches!(self, Self::LeftJab | Self::LeftHook | Self::LeftUppercut)
    }

    /// Check if this is a right-handed move
    pub fn is_right(&self) -> bool {
        matches!(self, Self::RightJab | Self::RightHook | Self::RightUppercut)
    }
}

/// Hitbox definition for collision detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hitbox {
    /// X offset from fighter center
    pub x: i8,
    /// Y offset from fighter center  
    pub y: i8,
    /// Width of hitbox
    pub width: u8,
    /// Height of hitbox
    pub height: u8,
    /// Hitbox type (high/mid/low)
    pub height_zone: HeightZone,
}

impl Default for Hitbox {
    fn default() -> Self {
        Self {
            x: -8,
            y: -20,
            width: 32,
            height: 48,
            height_zone: HeightZone::Mid,
        }
    }
}

/// Vertical hit zone for attacks
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum HeightZone {
    High, // Must block high
    Mid,  // Can block either
    Low,  // Must duck/block low
}

impl HeightZone {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::High => "High",
            Self::Mid => "Mid",
            Self::Low => "Low",
        }
    }
}

/// Defense behavior configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefenseBehavior {
    /// Type of defensive action
    pub behavior_type: DefenseType,
    /// Base frequency: 0-255 chance per frame
    pub frequency: u8,
    /// Conditions that must be met
    pub conditions: Vec<Condition>,
    /// Success rate (0-255, 255 = always successful)
    pub success_rate: u8,
    /// Recovery frames after defense
    pub recovery_frames: u8,
    /// Whether this triggers a counter-attack
    pub leads_to_counter: bool,
    /// Counter-attack pattern ID (if leads_to_counter is true)
    pub counter_pattern_id: Option<String>,
}

impl Default for DefenseBehavior {
    fn default() -> Self {
        Self {
            behavior_type: DefenseType::BlockHigh,
            frequency: 50,
            conditions: Vec::new(),
            success_rate: 200,
            recovery_frames: 15,
            leads_to_counter: false,
            counter_pattern_id: None,
        }
    }
}

/// Types of defensive actions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DefenseType {
    DodgeLeft,
    DodgeRight,
    Duck,
    BlockHigh,
    BlockLow,
    Counter,
    SwayBack,
    Clinch,
}

impl DefenseType {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::DodgeLeft => "Dodge Left",
            Self::DodgeRight => "Dodge Right",
            Self::Duck => "Duck",
            Self::BlockHigh => "Block High",
            Self::BlockLow => "Block Low",
            Self::Counter => "Counter",
            Self::SwayBack => "Sway Back",
            Self::Clinch => "Clinch",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::DodgeLeft => "⬅️",
            Self::DodgeRight => "➡️",
            Self::Duck => "🦆",
            Self::BlockHigh => "🛡️",
            Self::BlockLow => "🧱",
            Self::Counter => "⚔️",
            Self::SwayBack => "↩️",
            Self::Clinch => "🤼",
        }
    }
}

/// Difficulty curve across 3 rounds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DifficultyCurve {
    /// Round-specific difficulty settings
    pub rounds: Vec<RoundDifficulty>,
    /// Base aggression before round modifiers
    pub base_aggression: u8,
    /// Base defense before round modifiers
    pub base_defense: u8,
    /// Base speed before round modifiers
    pub base_speed: u8,
}

impl Default for DifficultyCurve {
    fn default() -> Self {
        Self {
            rounds: vec![
                RoundDifficulty::default_round(1),
                RoundDifficulty::default_round(2),
                RoundDifficulty::default_round(3),
            ],
            base_aggression: 100,
            base_defense: 100,
            base_speed: 100,
        }
    }
}

/// Difficulty settings for a specific round
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoundDifficulty {
    /// Round number (1-3)
    pub round: u8,
    /// Attack frequency multiplier (0-255)
    pub aggression: u8,
    /// Defense frequency multiplier (0-255)
    pub defense: u8,
    /// Animation speed multiplier (0-255)
    pub speed: u8,
    /// Pattern complexity level (0-255)
    pub pattern_complexity: u8,
    /// Damage multiplier percentage (100 = normal)
    pub damage_multiplier: u8,
    /// AI reaction time in frames (lower = faster)
    pub reaction_time: u8,
}

impl RoundDifficulty {
    pub fn default_round(round: u8) -> Self {
        match round {
            1 => Self {
                round: 1,
                aggression: 80,
                defense: 90,
                speed: 100,
                pattern_complexity: 50,
                damage_multiplier: 100,
                reaction_time: 8,
            },
            2 => Self {
                round: 2,
                aggression: 120,
                defense: 110,
                speed: 110,
                pattern_complexity: 100,
                damage_multiplier: 110,
                reaction_time: 6,
            },
            3 => Self {
                round: 3,
                aggression: 150,
                defense: 130,
                speed: 120,
                pattern_complexity: 150,
                damage_multiplier: 120,
                reaction_time: 4,
            },
            _ => Self {
                round,
                aggression: 100,
                defense: 100,
                speed: 100,
                pattern_complexity: 100,
                damage_multiplier: 100,
                reaction_time: 6,
            },
        }
    }
}

/// AI trigger - condition-action pair
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiTrigger {
    /// Condition that activates this trigger
    pub condition: Condition,
    /// Action to take when condition is met
    pub action: AiAction,
    /// Priority of this trigger (higher = checked first)
    pub priority: u8,
    /// Cooldown frames before this trigger can fire again
    pub cooldown: u16,
    /// Whether this trigger can only fire once per round
    pub once_per_round: bool,
}

/// Conditions for AI decision making
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Condition {
    /// Health below percentage (0-100)
    HealthBelow(u8),
    /// Health above percentage (0-100)
    HealthAbove(u8),
    /// Specific round (1-3)
    Round(u8),
    /// Time remaining below value (in seconds)
    TimeBelow(u8),
    /// Player is currently stunned
    PlayerStunned,
    /// Player is currently blocking
    PlayerBlocking,
    /// Random chance (0-255 = 0-100%)
    RandomChance(u8),
    /// Consecutive hits on player
    ComboCount(u8),
    /// Player just missed a punch
    PlayerMissed,
    /// Player is attacking (any type)
    PlayerAttacking,
    /// Player is using specific move type
    PlayerUsing(MoveType),
    /// Player is in specific health range
    PlayerHealthBelow(u8),
    /// AI has been hit X times
    TimesHit(u8),
    /// Always true (default condition)
    Always,
    /// Multiple conditions must all be true
    All(Vec<Condition>),
    /// Any of the conditions must be true
    Any(Vec<Condition>),
}

impl Condition {
    pub fn display_name(&self) -> String {
        match self {
            Self::HealthBelow(pct) => format!("Health < {}%", pct),
            Self::HealthAbove(pct) => format!("Health > {}%", pct),
            Self::Round(r) => format!("Round {}", r),
            Self::TimeBelow(t) => format!("Time < {}s", t),
            Self::PlayerStunned => "Player Stunned".to_string(),
            Self::PlayerBlocking => "Player Blocking".to_string(),
            Self::RandomChance(c) => format!("{}% Chance", c * 100 / 255),
            Self::ComboCount(n) => format!("{} Hit Combo", n),
            Self::PlayerMissed => "Player Missed".to_string(),
            Self::PlayerAttacking => "Player Attacking".to_string(),
            Self::PlayerUsing(mt) => format!("Player Using {}", mt.display_name()),
            Self::PlayerHealthBelow(pct) => format!("Player Health < {}%", pct),
            Self::TimesHit(n) => format!("Hit {} Times", n),
            Self::Always => "Always".to_string(),
            Self::All(conds) => format!("All of: {}", conds.len()),
            Self::Any(conds) => format!("Any of: {}", conds.len()),
        }
    }
}

/// Actions the AI can take when triggered
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AiAction {
    /// Use a specific attack pattern
    UsePattern(String),
    /// Change to a different behavior mode
    ChangeBehavior(String),
    /// Perform a taunt
    Taunt,
    /// Use boxer-specific special move
    SpecialMove,
    /// Use defensive action
    Defend(DefenseType),
    /// Move in a direction
    Move(Direction),
    /// Reset to default behavior
    ResetBehavior,
    /// Execute multiple actions in sequence
    Sequence(Vec<AiAction>),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    Left,
    Right,
    Forward,
    Back,
}
