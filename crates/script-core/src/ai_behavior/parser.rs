use super::constants::*;
use super::types::*;

/// Parser for AI behavior data from ROM
pub struct AiParser;

/// Error types for AI parsing
#[derive(Debug, Clone)]
pub enum AiParseError {
    InvalidFighterId(usize),
    RomTooSmall,
    InvalidOffset(usize),
    InvalidPointer(u16),
    CorruptedData(String),
    PatternTooLarge,
    DefenseOverflow,
}

impl std::fmt::Display for AiParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidFighterId(id) => write!(f, "Invalid fighter ID: {} (must be 0-15)", id),
            Self::RomTooSmall => write!(f, "ROM data is too small to contain AI data"),
            Self::InvalidOffset(off) => write!(f, "Invalid ROM offset: 0x{:06X}", off),
            Self::InvalidPointer(ptr) => write!(f, "Invalid AI pointer: 0x{:04X}", ptr),
            Self::CorruptedData(msg) => write!(f, "Corrupted AI data: {}", msg),
            Self::PatternTooLarge => write!(f, "Attack pattern exceeds maximum size"),
            Self::DefenseOverflow => write!(f, "Too many defense behaviors defined"),
        }
    }
}

impl std::error::Error for AiParseError {}

impl AiParser {
    /// Convert SNES LoROM address to PC offset
    fn snes_to_pc(bank: u8, addr: u16) -> usize {
        // LoROM mapping: PC = (Bank & 0x7F) * 0x8000 + (Addr & 0x7FFF)
        ((bank as usize & 0x7F) * 0x8000) | (addr as usize & 0x7FFF)
    }

    /// Read a single byte from ROM at PC offset
    fn read_u8(rom: &[u8], offset: usize) -> Result<u8, AiParseError> {
        rom.get(offset)
            .copied()
            .ok_or(AiParseError::InvalidOffset(offset))
    }

    /// Read a 16-bit little-endian value from ROM
    fn read_u16(rom: &[u8], offset: usize) -> Result<u16, AiParseError> {
        let lo = Self::read_u8(rom, offset)? as u16;
        let hi = Self::read_u8(rom, offset + 1)? as u16;
        Ok((hi << 8) | lo)
    }

    /// Validate fighter ID
    fn validate_fighter_id(fighter_id: usize) -> Result<(), AiParseError> {
        if fighter_id >= MAX_FIGHTERS {
            Err(AiParseError::InvalidFighterId(fighter_id))
        } else {
            Ok(())
        }
    }

    /// Parse AI behavior from ROM data
    ///
    /// # Arguments
    /// * `rom` - The ROM data
    /// * `fighter_id` - Fighter index (0-15)
    ///
    /// # Returns
    /// AI behavior structure or error
    pub fn parse_from_rom(rom: &[u8], fighter_id: usize) -> Result<super::AiBehavior, AiParseError> {
        Self::validate_fighter_id(fighter_id)?;

        if rom.len() < AI_TABLE_BASE + 0x1000 {
            return Err(AiParseError::RomTooSmall);
        }

        // Read fighter header to get AI pointer
        let header_addr = FIGHTER_HEADER_BASE + (fighter_id * FIGHTER_HEADER_SIZE);
        let ai_ptr = Self::read_u16(rom, header_addr + 8)?;

        // Validate AI pointer (should be in bank $0B range)
        if ai_ptr < 0x8000 {
            return Err(AiParseError::InvalidPointer(ai_ptr));
        }

        let fighter_name = FIGHTER_NAMES
            .get(fighter_id)
            .map(|&s| s.to_string())
            .unwrap_or_else(|| format!("Fighter {}", fighter_id));

        // Parse AI data sections
        let attack_patterns = Self::parse_patterns(rom, fighter_id, ai_ptr)?;
        let defense_behaviors = Self::parse_defense(rom, fighter_id)?;
        let difficulty_curve = Self::parse_difficulty(rom, fighter_id)?;
        let triggers = Self::parse_triggers(rom, fighter_id)?;

        // Read raw bytes for debugging
        let ai_base_pc = Self::snes_to_pc(0x0B, ai_ptr);
        let raw_bytes = rom
            .get(ai_base_pc..ai_base_pc + 256)
            .unwrap_or(&[])
            .to_vec();

        Ok(super::AiBehavior {
            fighter_id,
            fighter_name,
            attack_patterns,
            defense_behaviors,
            difficulty_curve,
            triggers,
            raw_bytes,
            pc_offset: Some(ai_base_pc),
        })
    }

    /// Parse attack patterns from AI data
    fn parse_patterns(
        rom: &[u8],
        fighter_id: usize,
        _base_addr: u16,
    ) -> Result<Vec<AttackPattern>, AiParseError> {
        let pattern_base = AI_PATTERN_TABLE + (fighter_id * 0x40); // 64 bytes per fighter
        let count = Self::read_u8(rom, pattern_base)? as usize;
        let count = count.min(MAX_PATTERNS_PER_FIGHTER);

        let mut patterns = Vec::with_capacity(count);

        for i in 0..count {
            let pattern_addr = pattern_base + 1 + (i * 12); // Each pattern is 12 bytes

            let move_type_byte = Self::read_u8(rom, pattern_addr)?;
            let move_type = Self::parse_move_type(move_type_byte);

            let pattern = AttackPattern {
                id: format!("pattern_{}", i),
                name: Self::generate_pattern_name(&move_type, i),
                sequence: vec![AttackMove {
                    move_type,
                    windup_frames: Self::read_u8(rom, pattern_addr + 1)?,
                    active_frames: Self::read_u8(rom, pattern_addr + 2)?,
                    recovery_frames: Self::read_u8(rom, pattern_addr + 3)?,
                    damage: Self::read_u8(rom, pattern_addr + 4)?,
                    stun: Self::read_u8(rom, pattern_addr + 5)?,
                    hitbox: Self::parse_hitbox(rom, pattern_addr + 6)?,
                    pose_id: Self::read_u8(rom, pattern_addr + 8)?,
                    sound_effect: Self::read_u8(rom, pattern_addr + 9).ok(),
                }],
                frequency: Self::read_u8(rom, pattern_addr + 10)?,
                conditions: Self::parse_conditions(Self::read_u8(rom, pattern_addr + 11)?),
                difficulty_min: 0,
                difficulty_max: 255,
                available_round_1: true,
                available_round_2: true,
                available_round_3: true,
                weight: 10,
            };

            patterns.push(pattern);
        }

        // If no patterns found, generate defaults
        if patterns.is_empty() {
            patterns = Self::generate_default_patterns(fighter_id);
        }

        Ok(patterns)
    }

    /// Parse move type from byte
    fn parse_move_type(byte: u8) -> MoveType {
        match byte {
            0x00 => MoveType::LeftJab,
            0x01 => MoveType::RightJab,
            0x02 => MoveType::LeftHook,
            0x03 => MoveType::RightHook,
            0x04 => MoveType::LeftUppercut,
            0x05 => MoveType::RightUppercut,
            0x06 => MoveType::Special,
            0x07 => MoveType::Taunt,
            0x08 => MoveType::StepLeft,
            0x09 => MoveType::StepRight,
            0x0A => MoveType::StepForward,
            0x0B => MoveType::StepBack,
            _ => MoveType::LeftJab, // Default fallback
        }
    }

    /// Serialize move type to byte
    fn move_type_to_byte(move_type: &MoveType) -> u8 {
        match move_type {
            MoveType::LeftJab => 0x00,
            MoveType::RightJab => 0x01,
            MoveType::LeftHook => 0x02,
            MoveType::RightHook => 0x03,
            MoveType::LeftUppercut => 0x04,
            MoveType::RightUppercut => 0x05,
            MoveType::Special => 0x06,
            MoveType::Taunt => 0x07,
            MoveType::StepLeft => 0x08,
            MoveType::StepRight => 0x09,
            MoveType::StepForward => 0x0A,
            MoveType::StepBack => 0x0B,
        }
    }

    /// Generate a human-readable pattern name
    fn generate_pattern_name(move_type: &MoveType, index: usize) -> String {
        format!("{} Pattern {}", move_type.display_name(), index + 1)
    }

    /// Parse hitbox from ROM bytes
    fn parse_hitbox(rom: &[u8], offset: usize) -> Result<Hitbox, AiParseError> {
        Ok(Hitbox {
            x: Self::read_u8(rom, offset)? as i8,
            y: Self::read_u8(rom, offset + 1)? as i8,
            width: Self::read_u8(rom, offset + 2)?,
            height: Self::read_u8(rom, offset + 3)?,
            height_zone: HeightZone::Mid, // Default, determined by animation
        })
    }

    /// Parse conditions from condition byte
    fn parse_conditions(condition_byte: u8) -> Vec<Condition> {
        let mut conditions = Vec::new();

        if condition_byte & 0x01 != 0 {
            conditions.push(Condition::Round(1));
        }
        if condition_byte & 0x02 != 0 {
            conditions.push(Condition::Round(2));
        }
        if condition_byte & 0x04 != 0 {
            conditions.push(Condition::Round(3));
        }
        if condition_byte & 0x08 != 0 {
            conditions.push(Condition::HealthBelow(50));
        }
        if condition_byte & 0x10 != 0 {
            conditions.push(Condition::PlayerStunned);
        }
        if condition_byte & 0x20 != 0 {
            conditions.push(Condition::RandomChance(128));
        }

        if conditions.is_empty() {
            conditions.push(Condition::Always);
        }

        conditions
    }

    /// Serialize conditions to byte
    fn conditions_to_byte(conditions: &[Condition]) -> u8 {
        let mut byte = 0u8;

        for condition in conditions {
            match condition {
                Condition::Round(1) => byte |= 0x01,
                Condition::Round(2) => byte |= 0x02,
                Condition::Round(3) => byte |= 0x04,
                Condition::HealthBelow(_) => byte |= 0x08,
                Condition::PlayerStunned => byte |= 0x10,
                Condition::RandomChance(_) => byte |= 0x20,
                _ => {}
            }
        }

        byte
    }

    /// Parse defense behaviors
    fn parse_defense(rom: &[u8], fighter_id: usize) -> Result<Vec<DefenseBehavior>, AiParseError> {
        let defense_base = AI_DEFENSE_TABLE + (fighter_id * 0x20); // 32 bytes per fighter
        let count = Self::read_u8(rom, defense_base)? as usize;
        let count = count.min(MAX_DEFENSE_PER_FIGHTER);

        let mut defenses = Vec::with_capacity(count);

        for i in 0..count {
            let defense_addr = defense_base + 1 + (i * 6); // Each defense is 6 bytes

            let behavior_type = Self::parse_defense_type(Self::read_u8(rom, defense_addr)?);

            let defense = DefenseBehavior {
                behavior_type,
                frequency: Self::read_u8(rom, defense_addr + 1)?,
                conditions: Self::parse_conditions(Self::read_u8(rom, defense_addr + 2)?),
                success_rate: Self::read_u8(rom, defense_addr + 3)?,
                recovery_frames: Self::read_u8(rom, defense_addr + 4)?,
                leads_to_counter: Self::read_u8(rom, defense_addr + 5)? != 0,
                counter_pattern_id: None, // Will be set if leads_to_counter is true
            };

            defenses.push(defense);
        }

        // If no defenses found, generate defaults
        if defenses.is_empty() {
            defenses = Self::generate_default_defense();
        }

        Ok(defenses)
    }

    /// Parse defense type from byte
    fn parse_defense_type(byte: u8) -> DefenseType {
        match byte {
            0x00 => DefenseType::DodgeLeft,
            0x01 => DefenseType::DodgeRight,
            0x02 => DefenseType::Duck,
            0x03 => DefenseType::BlockHigh,
            0x04 => DefenseType::BlockLow,
            0x05 => DefenseType::Counter,
            0x06 => DefenseType::SwayBack,
            0x07 => DefenseType::Clinch,
            _ => DefenseType::BlockHigh,
        }
    }

    /// Serialize defense type to byte
    fn defense_type_to_byte(defense_type: &DefenseType) -> u8 {
        match defense_type {
            DefenseType::DodgeLeft => 0x00,
            DefenseType::DodgeRight => 0x01,
            DefenseType::Duck => 0x02,
            DefenseType::BlockHigh => 0x03,
            DefenseType::BlockLow => 0x04,
            DefenseType::Counter => 0x05,
            DefenseType::SwayBack => 0x06,
            DefenseType::Clinch => 0x07,
        }
    }

    /// Parse difficulty curve from ROM
    fn parse_difficulty(rom: &[u8], fighter_id: usize) -> Result<DifficultyCurve, AiParseError> {
        let diff_base = AI_TABLE_BASE + 0x1000 + (fighter_id * 0x10); // After pattern/defense tables

        let base_aggression = Self::read_u8(rom, diff_base)?;
        let base_defense = Self::read_u8(rom, diff_base + 1)?;
        let base_speed = Self::read_u8(rom, diff_base + 2)?;

        let mut rounds = Vec::with_capacity(3);
        for round in 1..=3 {
            let round_offset = diff_base + 3 + ((round - 1) * 6);
            rounds.push(RoundDifficulty {
                round: round as u8,
                aggression: Self::read_u8(rom, round_offset)?,
                defense: Self::read_u8(rom, round_offset + 1)?,
                speed: Self::read_u8(rom, round_offset + 2)?,
                pattern_complexity: Self::read_u8(rom, round_offset + 3)?,
                damage_multiplier: Self::read_u8(rom, round_offset + 4)?,
                reaction_time: Self::read_u8(rom, round_offset + 5)?,
            });
        }

        Ok(DifficultyCurve {
            rounds,
            base_aggression,
            base_defense,
            base_speed,
        })
    }

    /// Parse triggers from ROM
    fn parse_triggers(rom: &[u8], fighter_id: usize) -> Result<Vec<AiTrigger>, AiParseError> {
        let trigger_base = AI_TRIGGER_TABLE + (fighter_id * 0x20);
        let count = Self::read_u8(rom, trigger_base)? as usize;
        let count = count.min(MAX_TRIGGERS_PER_FIGHTER);

        let mut triggers = Vec::with_capacity(count);

        for i in 0..count {
            let trigger_addr = trigger_base + 1 + (i * 5); // Each trigger is 5 bytes

            let condition = Self::parse_trigger_condition(Self::read_u8(rom, trigger_addr)?);
            let action = Self::parse_trigger_action(Self::read_u8(rom, trigger_addr + 1)?);

            let trigger = AiTrigger {
                condition,
                action,
                priority: Self::read_u8(rom, trigger_addr + 2)?,
                cooldown: Self::read_u16(rom, trigger_addr + 3)?,
                once_per_round: false,
            };

            triggers.push(trigger);
        }

        Ok(triggers)
    }

    /// Parse trigger condition from byte
    fn parse_trigger_condition(byte: u8) -> Condition {
        match byte {
            0x00 => Condition::Always,
            0x01 => Condition::HealthBelow(25),
            0x02 => Condition::HealthBelow(50),
            0x03 => Condition::PlayerStunned,
            0x04 => Condition::PlayerMissed,
            0x05 => Condition::Round(3),
            0x06 => Condition::ComboCount(3),
            0x07 => Condition::RandomChance(64),
            _ => Condition::Always,
        }
    }

    /// Serialize trigger condition to byte
    fn trigger_condition_to_byte(condition: &Condition) -> u8 {
        match condition {
            Condition::Always => 0x00,
            Condition::HealthBelow(25) => 0x01,
            Condition::HealthBelow(50) => 0x02,
            Condition::HealthBelow(_) => 0x02,
            Condition::PlayerStunned => 0x03,
            Condition::PlayerMissed => 0x04,
            Condition::Round(3) => 0x05,
            Condition::Round(_) => 0x00,
            Condition::ComboCount(_) => 0x06,
            Condition::RandomChance(_) => 0x07,
            _ => 0x00,
        }
    }

    /// Parse trigger action from byte
    fn parse_trigger_action(byte: u8) -> AiAction {
        match byte {
            0x00 => AiAction::Taunt,
            0x01 => AiAction::SpecialMove,
            0x02 => AiAction::Defend(DefenseType::DodgeLeft),
            0x03 => AiAction::Defend(DefenseType::Counter),
            0x04 => AiAction::ChangeBehavior("aggressive".to_string()),
            0x05 => AiAction::ChangeBehavior("defensive".to_string()),
            0x06 => AiAction::ResetBehavior,
            _ => AiAction::Taunt,
        }
    }

    /// Serialize trigger action to byte
    fn trigger_action_to_byte(action: &AiAction) -> u8 {
        match action {
            AiAction::Taunt => 0x00,
            AiAction::SpecialMove => 0x01,
            AiAction::Defend(DefenseType::DodgeLeft) => 0x02,
            AiAction::Defend(DefenseType::Counter) => 0x03,
            AiAction::ChangeBehavior(s) if s == "aggressive" => 0x04,
            AiAction::ChangeBehavior(s) if s == "defensive" => 0x05,
            AiAction::ResetBehavior => 0x06,
            _ => 0x00,
        }
    }

    /// Generate default patterns for a fighter
    fn generate_default_patterns(fighter_id: usize) -> Vec<AttackPattern> {
        use MoveType::*;

        match fighter_id {
            0 => vec![
                // Gabby Jay - Simple patterns
                AttackPattern {
                    id: "jab".to_string(),
                    name: "Left Jab".to_string(),
                    sequence: vec![AttackMove {
                        move_type: LeftJab,
                        windup_frames: 15,
                        active_frames: 8,
                        recovery_frames: 25,
                        damage: 8,
                        stun: 4,
                        ..Default::default()
                    }],
                    frequency: 60,
                    difficulty_max: 100,
                    ..Default::default()
                },
            ],
            11 => vec![
                // Super Macho Man - Complex
                AttackPattern {
                    id: "spin_punch".to_string(),
                    name: "Spin Punch".to_string(),
                    sequence: vec![AttackMove {
                        move_type: Special,
                        windup_frames: 30,
                        active_frames: 15,
                        recovery_frames: 40,
                        damage: 35,
                        stun: 20,
                        ..Default::default()
                    }],
                    frequency: 25,
                    difficulty_min: 100,
                    ..Default::default()
                },
            ],
            _ => vec![AttackPattern {
                id: "basic_jab".to_string(),
                name: "Basic Jab".to_string(),
                sequence: vec![AttackMove {
                    move_type: LeftJab,
                    windup_frames: 12,
                    active_frames: 8,
                    recovery_frames: 20,
                    damage: 10,
                    stun: 5,
                    ..Default::default()
                }],
                frequency: 50,
                ..Default::default()
            }],
        }
    }

    /// Generate default defense behaviors
    fn generate_default_defense() -> Vec<DefenseBehavior> {
        vec![
            DefenseBehavior {
                behavior_type: DefenseType::BlockHigh,
                frequency: 80,
                success_rate: 220,
                ..Default::default()
            },
            DefenseBehavior {
                behavior_type: DefenseType::DodgeLeft,
                frequency: 40,
                success_rate: 180,
                leads_to_counter: true,
                ..Default::default()
            },
        ]
    }

    /// Serialize AI behavior back to ROM format
    ///
    /// # Arguments
    /// * `behavior` - The AI behavior to serialize
    /// * `fighter_id` - The fighter ID (0-15)
    ///
    /// # Returns
    /// Serialized bytes ready to write to ROM
    pub fn serialize_to_bytes(
        behavior: &super::AiBehavior,
        fighter_id: u8,
    ) -> Result<Vec<u8>, AiParseError> {
        if fighter_id as usize >= MAX_FIGHTERS {
            return Err(AiParseError::InvalidFighterId(fighter_id as usize));
        }

        let mut bytes = Vec::new();

        // === Pattern Section ===
        // Pattern count
        let pattern_count = behavior.attack_patterns.len().min(MAX_PATTERNS_PER_FIGHTER) as u8;
        bytes.push(pattern_count);

        // Serialize each pattern (12 bytes each)
        for pattern in behavior
            .attack_patterns
            .iter()
            .take(MAX_PATTERNS_PER_FIGHTER)
        {
            if let Some(first_move) = pattern.sequence.first() {
                bytes.push(Self::move_type_to_byte(&first_move.move_type));
                bytes.push(first_move.windup_frames);
                bytes.push(first_move.active_frames);
                bytes.push(first_move.recovery_frames);
                bytes.push(first_move.damage);
                bytes.push(first_move.stun);
                // Hitbox (4 bytes) - simplified
                bytes.push(first_move.hitbox.x as u8);
                bytes.push(first_move.hitbox.y as u8);
                bytes.push(first_move.hitbox.width);
                bytes.push(first_move.hitbox.height);
                bytes.push(first_move.pose_id);
                bytes.push(pattern.frequency);
                bytes.push(Self::conditions_to_byte(&pattern.conditions));
            }
        }

        // Pad pattern section to 64 bytes per fighter
        while bytes.len() < 64 {
            bytes.push(0);
        }

        // === Defense Section ===
        let defense_start = bytes.len();
        let defense_count = behavior
            .defense_behaviors
            .len()
            .min(MAX_DEFENSE_PER_FIGHTER) as u8;
        bytes.push(defense_count);

        // Serialize each defense behavior (6 bytes each)
        for defense in behavior
            .defense_behaviors
            .iter()
            .take(MAX_DEFENSE_PER_FIGHTER)
        {
            bytes.push(Self::defense_type_to_byte(&defense.behavior_type));
            bytes.push(defense.frequency);
            bytes.push(Self::conditions_to_byte(&defense.conditions));
            bytes.push(defense.success_rate);
            bytes.push(defense.recovery_frames);
            bytes.push(if defense.leads_to_counter { 1 } else { 0 });
        }

        // Pad defense section to 32 bytes per fighter
        while bytes.len() - defense_start < 32 {
            bytes.push(0);
        }

        // === Difficulty Section ===
        bytes.push(behavior.difficulty_curve.base_aggression);
        bytes.push(behavior.difficulty_curve.base_defense);
        bytes.push(behavior.difficulty_curve.base_speed);

        // Round data (6 bytes per round)
        for round in &behavior.difficulty_curve.rounds {
            bytes.push(round.aggression);
            bytes.push(round.defense);
            bytes.push(round.speed);
            bytes.push(round.pattern_complexity);
            bytes.push(round.damage_multiplier);
            bytes.push(round.reaction_time);
        }

        // === Trigger Section ===
        let trigger_count = behavior.triggers.len().min(MAX_TRIGGERS_PER_FIGHTER) as u8;
        bytes.push(trigger_count);

        // Serialize each trigger (5 bytes each)
        for trigger in behavior.triggers.iter().take(MAX_TRIGGERS_PER_FIGHTER) {
            bytes.push(Self::trigger_condition_to_byte(&trigger.condition));
            bytes.push(Self::trigger_action_to_byte(&trigger.action));
            bytes.push(trigger.priority);
            bytes.push((trigger.cooldown & 0xFF) as u8);
            bytes.push(((trigger.cooldown >> 8) & 0xFF) as u8);
        }

        Ok(bytes)
    }

    // Helper methods for Tauri commands - exposed as public

    /// Generate default patterns for a fighter (public helper)
    pub fn generate_default_patterns_helper(fighter_id: usize) -> Vec<AttackPattern> {
        Self::generate_default_patterns(fighter_id)
    }

    /// Generate default defense behaviors (public helper)
    pub fn generate_default_defense_helper() -> Vec<DefenseBehavior> {
        Self::generate_default_defense()
    }

    /// Parse move type from byte (public)
    pub fn parse_move_type_pub(byte: u8) -> MoveType {
        Self::parse_move_type(byte)
    }

    /// Serialize move type to byte (public)
    pub fn move_type_to_byte_pub(move_type: &MoveType) -> u8 {
        Self::move_type_to_byte(move_type)
    }

    /// Parse defense type from byte (public)
    pub fn parse_defense_type_pub(byte: u8) -> DefenseType {
        Self::parse_defense_type(byte)
    }

    /// Serialize defense type to byte (public)
    pub fn defense_type_to_byte_pub(defense_type: &DefenseType) -> u8 {
        Self::defense_type_to_byte(defense_type)
    }

    /// Get the expected PC offset for a fighter's AI data
    pub fn get_ai_offset(fighter_id: usize) -> Result<usize, AiParseError> {
        Self::validate_fighter_id(fighter_id)?;
        Ok(AI_TABLE_BASE + (fighter_id * 0x100))
    }

    /// Generate example attack patterns for a fighter
    #[allow(dead_code)]
    fn generate_example_patterns(fighter_id: usize) -> Vec<AttackPattern> {
        use MoveType::*;

        let patterns = match fighter_id {
            0 => vec![
                // Gabby Jay - Simple patterns
                AttackPattern {
                    id: "jab".to_string(),
                    name: "Left Jab".to_string(),
                    sequence: vec![AttackMove {
                        move_type: LeftJab,
                        windup_frames: 15,
                        active_frames: 8,
                        recovery_frames: 25,
                        damage: 8,
                        stun: 4,
                        ..Default::default()
                    }],
                    frequency: 60,
                    difficulty_max: 100,
                    ..Default::default()
                },
                AttackPattern {
                    id: "hook_combo".to_string(),
                    name: "Hook Combo".to_string(),
                    sequence: vec![
                        AttackMove {
                            move_type: LeftHook,
                            windup_frames: 20,
                            active_frames: 10,
                            recovery_frames: 15,
                            damage: 12,
                            stun: 6,
                            ..Default::default()
                        },
                        AttackMove {
                            move_type: RightHook,
                            windup_frames: 18,
                            active_frames: 10,
                            recovery_frames: 30,
                            damage: 15,
                            stun: 8,
                            ..Default::default()
                        },
                    ],
                    frequency: 40,
                    difficulty_min: 20,
                    ..Default::default()
                },
            ],
            11 => vec![
                // Super Macho Man - Complex patterns
                AttackPattern {
                    id: "spin_punch".to_string(),
                    name: "Spin Punch".to_string(),
                    sequence: vec![AttackMove {
                        move_type: Special,
                        windup_frames: 30,
                        active_frames: 15,
                        recovery_frames: 40,
                        damage: 35,
                        stun: 20,
                        ..Default::default()
                    }],
                    frequency: 25,
                    difficulty_min: 100,
                    ..Default::default()
                },
            ],
            _ => vec![
                // Default patterns for other fighters
                AttackPattern {
                    id: "basic_jab".to_string(),
                    name: "Basic Jab".to_string(),
                    sequence: vec![AttackMove {
                        move_type: LeftJab,
                        windup_frames: 12,
                        active_frames: 8,
                        recovery_frames: 20,
                        damage: 10,
                        stun: 5,
                        ..Default::default()
                    }],
                    frequency: 50,
                    ..Default::default()
                },
            ],
        };

        patterns
    }

    /// Generate example defense behaviors
    #[allow(dead_code)]
    fn generate_example_defense() -> Vec<DefenseBehavior> {
        vec![
            DefenseBehavior {
                behavior_type: DefenseType::BlockHigh,
                frequency: 80,
                success_rate: 220,
                ..Default::default()
            },
            DefenseBehavior {
                behavior_type: DefenseType::DodgeLeft,
                frequency: 40,
                success_rate: 180,
                leads_to_counter: true,
                counter_pattern_id: Some("counter_left".to_string()),
                ..Default::default()
            },
            DefenseBehavior {
                behavior_type: DefenseType::DodgeRight,
                frequency: 40,
                success_rate: 180,
                leads_to_counter: true,
                counter_pattern_id: Some("counter_right".to_string()),
                ..Default::default()
            },
        ]
    }

    /// Generate example triggers
    #[allow(dead_code)]
    fn generate_example_triggers() -> Vec<AiTrigger> {
        vec![
            AiTrigger {
                condition: Condition::HealthBelow(25),
                action: AiAction::ChangeBehavior("desperate".to_string()),
                priority: 200,
                cooldown: 600,
                once_per_round: true,
            },
            AiTrigger {
                condition: Condition::PlayerStunned,
                action: AiAction::UsePattern("finisher".to_string()),
                priority: 150,
                cooldown: 60,
                once_per_round: false,
            },
            AiTrigger {
                condition: Condition::Round(3),
                action: AiAction::Taunt,
                priority: 50,
                cooldown: 300,
                once_per_round: true,
            },
        ]
    }
}
