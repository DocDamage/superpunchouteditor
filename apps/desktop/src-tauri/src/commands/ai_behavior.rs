//! AI Behavior Commands
//!
//! Commands for parsing, editing, and simulating boxer AI behavior.

use tauri::State;

use crate::app_state::AppState;
use script_core::ai_behavior::{
    AiBehavior, AiBehaviorManager, AiParser, AiPresets, AiTrigger, AttackPattern, DefenseBehavior,
    DefenseType, DifficultyCurve, MoveType, SimulationResult, MAX_FIGHTERS,
};

/// Get AI behavior for a specific fighter
///
/// Parses the AI data from the currently loaded ROM for the given fighter.
#[tauri::command]
pub fn get_ai_behavior(state: State<AppState>, fighter_id: usize) -> Result<AiBehavior, String> {
    let rom_opt = state.rom.lock();
    let rom = rom_opt.as_ref().ok_or("No ROM loaded")?;

    AiParser::parse_from_rom(&rom.data, fighter_id)
        .map_err(|e| format!("Failed to parse AI behavior: {}", e))
}

/// Get AI behavior for all fighters
///
/// Returns a list of AI behaviors for all fighters in the roster.
#[tauri::command]
pub fn get_all_ai_behaviors(state: State<AppState>) -> Result<Vec<AiBehavior>, String> {
    let rom_opt = state.rom.lock();
    let rom = rom_opt.as_ref().ok_or("No ROM loaded")?;

    let mut behaviors = Vec::with_capacity(MAX_FIGHTERS);

    for id in 0..MAX_FIGHTERS {
        match AiParser::parse_from_rom(&rom.data, id) {
            Ok(behavior) => behaviors.push(behavior),
            Err(e) => return Err(format!("Failed to parse AI for fighter {}: {}", id, e)),
        }
    }

    Ok(behaviors)
}

/// Update an attack pattern for a fighter
///
/// Modifies a specific attack pattern and queues the changes for writing to ROM.
#[tauri::command]
pub fn update_attack_pattern(
    state: State<AppState>,
    fighter_id: usize,
    pattern_index: usize,
    pattern: AttackPattern,
) -> Result<AiBehavior, String> {
    let rom_opt = state.rom.lock();
    let rom = rom_opt.as_ref().ok_or("No ROM loaded")?;

    // Parse current behavior
    let mut behavior = AiParser::parse_from_rom(&rom.data, fighter_id)
        .map_err(|e| format!("Failed to parse AI behavior: {}", e))?;

    // Update the pattern
    if pattern_index < behavior.attack_patterns.len() {
        behavior.attack_patterns[pattern_index] = pattern;
    } else if pattern_index == behavior.attack_patterns.len() {
        behavior.attack_patterns.push(pattern);
    } else {
        return Err(format!("Invalid pattern index: {}", pattern_index));
    }

    // Serialize and queue for writing
    let bytes = AiParser::serialize_to_bytes(&behavior, fighter_id as u8)
        .map_err(|e| format!("Failed to serialize AI behavior: {}", e))?;

    let ai_offset = AiParser::get_ai_offset(fighter_id)
        .map_err(|e| format!("Failed to get AI offset: {}", e))?;

    drop(rom_opt);

    // Store in pending writes
    let pc_offset_str = format!("0x{:X}", ai_offset);
    state
        .pending_writes
        .lock()
        .insert(pc_offset_str, bytes);

    Ok(behavior)
}

/// Update a defense behavior for a fighter
///
/// Modifies a specific defense behavior and queues the changes for writing to ROM.
#[tauri::command]
pub fn update_defense_behavior(
    state: State<AppState>,
    fighter_id: usize,
    defense_index: usize,
    defense: DefenseBehavior,
) -> Result<AiBehavior, String> {
    let rom_opt = state.rom.lock();
    let rom = rom_opt.as_ref().ok_or("No ROM loaded")?;

    // Parse current behavior
    let mut behavior = AiParser::parse_from_rom(&rom.data, fighter_id)
        .map_err(|e| format!("Failed to parse AI behavior: {}", e))?;

    // Update the defense behavior
    if defense_index < behavior.defense_behaviors.len() {
        behavior.defense_behaviors[defense_index] = defense;
    } else if defense_index == behavior.defense_behaviors.len() {
        behavior.defense_behaviors.push(defense);
    } else {
        return Err(format!("Invalid defense index: {}", defense_index));
    }

    // Serialize and queue for writing
    let bytes = AiParser::serialize_to_bytes(&behavior, fighter_id as u8)
        .map_err(|e| format!("Failed to serialize AI behavior: {}", e))?;

    let ai_offset = AiParser::get_ai_offset(fighter_id)
        .map_err(|e| format!("Failed to get AI offset: {}", e))?;

    drop(rom_opt);

    // Store in pending writes
    let pc_offset_str = format!("0x{:X}", ai_offset);
    state
        .pending_writes
        .lock()
        .insert(pc_offset_str, bytes);

    Ok(behavior)
}

/// Update difficulty curve for a fighter
///
/// Modifies the round-by-round difficulty scaling.
#[tauri::command]
pub fn update_difficulty_curve(
    state: State<AppState>,
    fighter_id: usize,
    difficulty: DifficultyCurve,
) -> Result<AiBehavior, String> {
    let rom_opt = state.rom.lock();
    let rom = rom_opt.as_ref().ok_or("No ROM loaded")?;

    // Parse current behavior
    let mut behavior = AiParser::parse_from_rom(&rom.data, fighter_id)
        .map_err(|e| format!("Failed to parse AI behavior: {}", e))?;

    // Update difficulty curve
    behavior.difficulty_curve = difficulty;

    // Serialize and queue for writing
    let bytes = AiParser::serialize_to_bytes(&behavior, fighter_id as u8)
        .map_err(|e| format!("Failed to serialize AI behavior: {}", e))?;

    let ai_offset = AiParser::get_ai_offset(fighter_id)
        .map_err(|e| format!("Failed to get AI offset: {}", e))?;

    drop(rom_opt);

    // Store in pending writes
    let pc_offset_str = format!("0x{:X}", ai_offset);
    state
        .pending_writes
        .lock()
        .insert(pc_offset_str, bytes);

    Ok(behavior)
}

/// Update triggers for a fighter
///
/// Replaces all triggers for the specified fighter.
#[tauri::command]
pub fn update_triggers(
    state: State<AppState>,
    fighter_id: usize,
    triggers: Vec<AiTrigger>,
) -> Result<AiBehavior, String> {
    let rom_opt = state.rom.lock();
    let rom = rom_opt.as_ref().ok_or("No ROM loaded")?;

    // Parse current behavior
    let mut behavior = AiParser::parse_from_rom(&rom.data, fighter_id)
        .map_err(|e| format!("Failed to parse AI behavior: {}", e))?;

    // Update triggers
    behavior.triggers = triggers;

    // Serialize and queue for writing
    let bytes = AiParser::serialize_to_bytes(&behavior, fighter_id as u8)
        .map_err(|e| format!("Failed to serialize AI behavior: {}", e))?;

    let ai_offset = AiParser::get_ai_offset(fighter_id)
        .map_err(|e| format!("Failed to get AI offset: {}", e))?;

    drop(rom_opt);

    // Store in pending writes
    let pc_offset_str = format!("0x{:X}", ai_offset);
    state
        .pending_writes
        .lock()
        .insert(pc_offset_str, bytes);

    Ok(behavior)
}

/// Test AI behavior through simulation
///
/// Runs a simulation to analyze the AI behavior and provide statistics.
#[tauri::command]
pub fn test_ai_behavior(
    state: State<AppState>,
    fighter_id: usize,
    iterations: Option<u32>,
) -> Result<SimulationResult, String> {
    let rom_opt = state.rom.lock();
    let rom = rom_opt.as_ref().ok_or("No ROM loaded")?;

    let behavior = AiParser::parse_from_rom(&rom.data, fighter_id)
        .map_err(|e| format!("Failed to parse AI behavior: {}", e))?;

    let iters = iterations.unwrap_or(100);
    Ok(AiBehaviorManager::simulate(&behavior, iters))
}

/// Validate AI behavior
///
/// Checks for potential issues in the AI configuration.
#[tauri::command]
pub fn validate_ai_behavior(
    state: State<AppState>,
    fighter_id: usize,
) -> Result<Vec<String>, String> {
    let rom_opt = state.rom.lock();
    let rom = rom_opt.as_ref().ok_or("No ROM loaded")?;

    let behavior = AiParser::parse_from_rom(&rom.data, fighter_id)
        .map_err(|e| format!("Failed to parse AI behavior: {}", e))?;

    Ok(AiBehaviorManager::validate(&behavior))
}

/// Compare AI between two fighters
///
/// Returns a detailed comparison of AI behaviors.
#[tauri::command]
pub fn compare_ai_behavior(
    state: State<AppState>,
    fighter_a_id: usize,
    fighter_b_id: usize,
) -> Result<serde_json::Value, String> {
    let rom_opt = state.rom.lock();
    let rom = rom_opt.as_ref().ok_or("No ROM loaded")?;

    let behavior_a = AiParser::parse_from_rom(&rom.data, fighter_a_id)
        .map_err(|e| format!("Failed to parse AI for fighter {}: {}", fighter_a_id, e))?;

    let behavior_b = AiParser::parse_from_rom(&rom.data, fighter_b_id)
        .map_err(|e| format!("Failed to parse AI for fighter {}: {}", fighter_b_id, e))?;

    // Calculate differences
    let pattern_diff =
        behavior_a.attack_patterns.len() as i32 - behavior_b.attack_patterns.len() as i32;
    let defense_diff =
        behavior_a.defense_behaviors.len() as i32 - behavior_b.defense_behaviors.len() as i32;

    let avg_freq_a: f32 = behavior_a
        .attack_patterns
        .iter()
        .map(|p| p.frequency as f32)
        .sum::<f32>()
        / behavior_a.attack_patterns.len().max(1) as f32;

    let avg_freq_b: f32 = behavior_b
        .attack_patterns
        .iter()
        .map(|p| p.frequency as f32)
        .sum::<f32>()
        / behavior_b.attack_patterns.len().max(1) as f32;

    let round3_a = behavior_a
        .difficulty_curve
        .rounds
        .get(2)
        .map(|r| r.aggression)
        .unwrap_or(100);

    let round3_b = behavior_b
        .difficulty_curve
        .rounds
        .get(2)
        .map(|r| r.aggression)
        .unwrap_or(100);

    Ok(serde_json::json!({
        "fighter_a": {
            "name": behavior_a.fighter_name,
            "pattern_count": behavior_a.attack_patterns.len(),
            "defense_count": behavior_a.defense_behaviors.len(),
            "avg_frequency": avg_freq_a,
            "round3_aggression": round3_a,
        },
        "fighter_b": {
            "name": behavior_b.fighter_name,
            "pattern_count": behavior_b.attack_patterns.len(),
            "defense_count": behavior_b.defense_behaviors.len(),
            "avg_frequency": avg_freq_b,
            "round3_aggression": round3_b,
        },
        "differences": {
            "pattern_count": pattern_diff,
            "defense_count": defense_diff,
            "frequency_diff": avg_freq_a - avg_freq_b,
            "aggression_diff": round3_a as i32 - round3_b as i32,
        },
    }))
}

/// Get AI presets
///
/// Returns the beginner and challenging AI preset templates.
#[tauri::command]
pub fn get_ai_presets() -> serde_json::Value {
    serde_json::json!({
        "beginner": AiPresets::beginner(),
        "challenging": AiPresets::challenging(),
    })
}

/// Apply AI preset to a fighter
///
/// Applies a preset AI behavior template to the specified fighter.
#[tauri::command]
pub fn apply_ai_preset(
    state: State<AppState>,
    fighter_id: usize,
    preset: String,
) -> Result<AiBehavior, String> {
    let rom_opt = state.rom.lock();
    let rom = rom_opt.as_ref().ok_or("No ROM loaded")?;

    // Get current behavior to preserve fighter name
    let current = AiParser::parse_from_rom(&rom.data, fighter_id)
        .map_err(|e| format!("Failed to parse AI behavior: {}", e))?;

    // Apply preset
    let mut behavior = match preset.as_str() {
        "beginner" => AiPresets::beginner(),
        "challenging" => AiPresets::challenging(),
        _ => return Err(format!("Unknown preset: {}", preset)),
    };

    // Preserve fighter ID and name
    behavior.fighter_id = current.fighter_id;
    behavior.fighter_name = current.fighter_name;

    // Serialize and queue for writing
    let bytes = AiParser::serialize_to_bytes(&behavior, fighter_id as u8)
        .map_err(|e| format!("Failed to serialize AI behavior: {}", e))?;

    let ai_offset = AiParser::get_ai_offset(fighter_id)
        .map_err(|e| format!("Failed to get AI offset: {}", e))?;

    drop(rom_opt);

    // Store in pending writes
    let pc_offset_str = format!("0x{:X}", ai_offset);
    state
        .pending_writes
        .lock()
        .insert(pc_offset_str, bytes);

    Ok(behavior)
}

/// Get available move types
///
/// Returns all available move types for attack patterns.
#[tauri::command]
pub fn get_move_types() -> Vec<serde_json::Value> {
    use MoveType::*;

    vec![
        LeftJab,
        RightJab,
        LeftHook,
        RightHook,
        LeftUppercut,
        RightUppercut,
        Special,
        Taunt,
        StepLeft,
        StepRight,
        StepForward,
        StepBack,
    ]
    .into_iter()
    .map(|mt| {
        serde_json::json!({
            "type": format!("{:?}", mt),
            "name": mt.display_name(),
            "icon": mt.icon(),
            "is_left": mt.is_left(),
            "is_right": mt.is_right(),
        })
    })
    .collect()
}

/// Get available defense types
///
/// Returns all available defense behavior types.
#[tauri::command]
pub fn get_defense_types() -> Vec<serde_json::Value> {
    use DefenseType::*;

    vec![
        DodgeLeft, DodgeRight, Duck, BlockHigh, BlockLow, Counter, SwayBack, Clinch,
    ]
    .into_iter()
    .map(|dt| {
        serde_json::json!({
            "type": format!("{:?}", dt),
            "name": dt.display_name(),
            "icon": dt.icon(),
        })
    })
    .collect()
}

/// Get available condition types
///
/// Returns all available trigger conditions.
#[tauri::command]
pub fn get_condition_types() -> Vec<serde_json::Value> {
    vec![
        ("Always", "Always active"),
        ("HealthBelow25", "Health < 25%"),
        ("HealthBelow50", "Health < 50%"),
        ("PlayerStunned", "Player is stunned"),
        ("PlayerMissed", "Player missed punch"),
        ("Round3", "Round 3"),
        ("Random", "Random chance"),
    ]
    .into_iter()
    .map(|(id, name)| {
        serde_json::json!({
            "id": id,
            "name": name,
        })
    })
    .collect()
}

/// Reset AI to defaults for a fighter
///
/// Resets the AI behavior to the default patterns for the fighter type.
#[tauri::command]
pub fn reset_ai_to_defaults(
    state: State<AppState>,
    fighter_id: usize,
) -> Result<AiBehavior, String> {
    let rom_opt = state.rom.lock();
    let rom = rom_opt.as_ref().ok_or("No ROM loaded")?;

    // Parse current to get fighter name
    let current = AiParser::parse_from_rom(&rom.data, fighter_id)
        .map_err(|e| format!("Failed to parse AI behavior: {}", e))?;

    // Generate default behavior
    let behavior = AiBehavior {
        fighter_id: current.fighter_id,
        fighter_name: current.fighter_name,
        attack_patterns: AiParser::generate_default_patterns_helper(fighter_id),
        defense_behaviors: AiParser::generate_default_defense_helper(),
        difficulty_curve: DifficultyCurve::default(),
        triggers: Vec::new(),
        raw_bytes: Vec::new(),
        pc_offset: current.pc_offset,
    };

    // Serialize and queue for writing
    let bytes = AiParser::serialize_to_bytes(&behavior, fighter_id as u8)
        .map_err(|e| format!("Failed to serialize AI behavior: {}", e))?;

    let ai_offset = AiParser::get_ai_offset(fighter_id)
        .map_err(|e| format!("Failed to get AI offset: {}", e))?;

    drop(rom_opt);

    // Store in pending writes
    let pc_offset_str = format!("0x{:X}", ai_offset);
    state
        .pending_writes
        .lock()
        .insert(pc_offset_str, bytes);

    Ok(behavior)
}

/// Get AI table addresses
///
/// Returns the ROM addresses for AI data tables.
#[tauri::command]
pub fn get_ai_table_addresses() -> serde_json::Value {
    use script_core::ai_behavior::{
        AI_DEFENSE_TABLE, AI_PATTERN_TABLE, AI_TABLE_BASE, AI_TRIGGER_TABLE, FIGHTER_HEADER_BASE,
    };

    serde_json::json!({
        "ai_table_base": format!("0x{:06X}", AI_TABLE_BASE),
        "ai_pattern_table": format!("0x{:06X}", AI_PATTERN_TABLE),
        "ai_defense_table": format!("0x{:06X}", AI_DEFENSE_TABLE),
        "ai_trigger_table": format!("0x{:06X}", AI_TRIGGER_TABLE),
        "fighter_header_base": format!("0x{:06X}", FIGHTER_HEADER_BASE),
    })
}
