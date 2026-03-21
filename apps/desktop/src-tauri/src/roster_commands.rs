//! Tauri commands for Roster Metadata Editor
//!
//! These commands provide access to game-level roster data including:
//! - Boxer names and text encoding
//! - Circuit assignments
//! - Unlock order
//! - Introductory text
//!
//! All commands now support ROM read/write operations.

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::AppState;
use rom_core::{
    SpoTextEncoder,
    roster::{
        BoxerIntro, BoxerRosterEntry, Circuit, CircuitType, RosterData,
        RosterLoader, RosterWriter, ValidationReport, BOXER_INTRO_TABLE,
        BOXER_NAME_POINTERS, CIRCUIT_TABLE, INTRO_FIELD_SIZE, MAX_NAME_LENGTH, UNLOCK_ORDER_TABLE,
    },
};

/// Roster data response for the frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RosterDataResponse {
    pub boxers: Vec<BoxerRosterEntry>,
    pub circuits: Vec<Circuit>,
}

impl From<RosterData> for RosterDataResponse {
    fn from(data: RosterData) -> Self {
        Self {
            boxers: data.boxers,
            circuits: data.circuits,
        }
    }
}

/// Request to update a boxer name
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateNameRequest {
    pub fighter_id: u8,
    pub name: String,
}

/// Request to update circuit assignment
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCircuitRequest {
    pub fighter_id: u8,
    pub circuit: CircuitType,
}

/// Request to update unlock order
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUnlockOrderRequest {
    pub fighter_id: u8,
    pub order: u8,
}

/// Response for intro text
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntroTextResponse {
    pub text_id: u8,
    pub text: String,
    pub fighter_id: u8,
}

/// Full boxer intro response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoxerIntroResponse {
    pub fighter_id: u8,
    pub boxer_key: String,
    pub name_text: String,
    pub origin_text: String,
    pub record_text: String,
    pub rank_text: String,
    pub intro_quote: String,
}

impl From<BoxerIntro> for BoxerIntroResponse {
    fn from(intro: BoxerIntro) -> Self {
        Self {
            fighter_id: get_boxer_id_from_key(&intro.boxer_key).unwrap_or(255),
            boxer_key: intro.boxer_key,
            name_text: intro.name_text,
            origin_text: intro.origin_text,
            record_text: intro.record_text,
            rank_text: intro.rank_text,
            intro_quote: intro.intro_quote,
        }
    }
}

/// Text encoding info response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextEncodingInfo {
    pub supported_chars: Vec<char>,
    pub max_name_length: usize,
    pub max_intro_field_length: usize,
}

/// Name validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NameValidationResult {
    pub valid: bool,
    pub encoded_length: usize,
    pub max_length: usize,
    pub can_encode: bool,
    pub error: Option<String>,
}

/// Cornerman text response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CornermanTextResponse {
    pub id: u8,
    pub boxer_key: String,
    pub fighter_id: u8,
    pub condition: String,
    pub text: String,
}

/// Victory quote response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VictoryQuoteResponse {
    pub id: u8,
    pub boxer_key: String,
    pub fighter_id: u8,
    pub text: String,
    pub is_loss_quote: bool,
}

/// ROM offset information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RomOffsetInfo {
    pub name_table_offset: usize,
    pub name_pointers_offset: usize,
    pub circuit_table_offset: usize,
    pub unlock_table_offset: usize,
    pub intro_table_offset: usize,
}

// ============================================================================
// ROSTER DATA COMMANDS
// ============================================================================

/// Get the complete roster data
///
/// If a ROM is loaded, reads from ROM; otherwise returns defaults
#[tauri::command]
pub fn get_roster_data(state: State<AppState>) -> Result<RosterDataResponse, String> {
    let rom_guard = state.rom.lock();

    let roster = if let Some(ref rom) = *rom_guard {
        let loader = RosterLoader::new(rom);
        loader.load_roster().map_err(|e| e.to_string())?
    } else {
        RosterData::new()
    };

    Ok(roster.into())
}

/// Get a single boxer by ID
#[tauri::command]
pub fn get_boxer_roster_entry(
    state: State<AppState>,
    fighter_id: u8,
) -> Result<BoxerRosterEntry, String> {
    let rom_guard = state.rom.lock();

    if let Some(ref rom) = *rom_guard {
        let loader = RosterLoader::new(rom);
        let roster = loader.load_roster().map_err(|e| e.to_string())?;

        roster
            .get_boxer(fighter_id)
            .cloned()
            .ok_or_else(|| format!("Boxer with ID {} not found", fighter_id))
    } else {
        let roster = RosterData::new();
        roster
            .get_boxer(fighter_id)
            .cloned()
            .ok_or_else(|| format!("Boxer with ID {} not found", fighter_id))
    }
}

/// Get all boxers in a circuit
#[tauri::command]
pub fn get_boxers_by_circuit(
    state: State<AppState>,
    circuit: CircuitType,
) -> Result<Vec<BoxerRosterEntry>, String> {
    let rom_guard = state.rom.lock();

    let roster = if let Some(ref rom) = *rom_guard {
        let loader = RosterLoader::new(rom);
        loader.load_roster().map_err(|e| e.to_string())?
    } else {
        RosterData::new()
    };

    Ok(roster
        .get_boxers_in_circuit(circuit)
        .into_iter()
        .cloned()
        .collect())
}

/// Get boxers in unlock order
#[tauri::command]
pub fn get_boxers_by_unlock_order(state: State<AppState>) -> Result<Vec<BoxerRosterEntry>, String> {
    let rom_guard = state.rom.lock();

    let roster = if let Some(ref rom) = *rom_guard {
        let loader = RosterLoader::new(rom);
        loader.load_roster().map_err(|e| e.to_string())?
    } else {
        RosterData::new()
    };

    Ok(roster
        .get_boxers_by_unlock_order()
        .into_iter()
        .cloned()
        .collect())
}

// ============================================================================
// NAME EDITING COMMANDS
// ============================================================================

/// Update a boxer's name
///
/// Writes to ROM if loaded, otherwise just validates
#[tauri::command]
pub fn update_boxer_name(
    state: State<AppState>,
    fighter_id: u8,
    new_name: String,
) -> Result<BoxerRosterEntry, String> {
    // Validate name first
    let encoder = SpoTextEncoder::new();
    encoder
        .validate(&new_name)
        .map_err(|invalid| format!("Invalid characters: {:?}", invalid))?;

    let encoded = encoder.encode(&new_name);
    if encoded.len() > MAX_NAME_LENGTH {
        return Err(format!(
            "Name too long: {} bytes (max {})",
            encoded.len(),
            MAX_NAME_LENGTH
        ));
    }

    let mut rom_guard = state.rom.lock();

    if let Some(ref mut rom) = *rom_guard {
        // Write to ROM
        let mut writer = RosterWriter::new(rom);
        writer
            .write_boxer_name(fighter_id, &new_name)
            .map_err(|e| e.to_string())?;

        // Mark ROM as modified
        drop(rom_guard);
        let mut modified = state.modified.lock();
        *modified = true;

        // Return updated entry
        let rom_guard = state.rom.lock();
        let loader = RosterLoader::new(rom_guard.as_ref().unwrap());
        let roster = loader.load_roster().map_err(|e| e.to_string())?;

        roster
            .get_boxer(fighter_id)
            .cloned()
            .ok_or_else(|| format!("Boxer with ID {} not found", fighter_id))
    } else {
        Err("No ROM loaded".to_string())
    }
}

/// Validate a boxer name (check encoding and length)
#[tauri::command]
pub fn validate_boxer_name(_state: State<AppState>, name: String) -> NameValidationResult {
    let encoder = SpoTextEncoder::new();
    let can_encode = encoder.can_encode(&name);
    let encoded = encoder.encode(&name);
    let encoded_length = encoded.len();

    let mut error = None;
    if encoded_length > MAX_NAME_LENGTH {
        error = Some(format!(
            "Name too long: {} bytes (max {})",
            encoded_length, MAX_NAME_LENGTH
        ));
    }
    if !can_encode {
        let unsupported: Vec<char> = name.chars().filter(|c| !encoder.can_encode(&c.to_string())).collect();
        error = Some(format!("Unsupported characters: {:?}", unsupported));
    }

    NameValidationResult {
        valid: error.is_none() && can_encode,
        encoded_length,
        max_length: MAX_NAME_LENGTH,
        can_encode,
        error,
    }
}

/// Preview how a name will be encoded
#[tauri::command]
pub fn preview_name_encoding(_state: State<AppState>, name: String) -> Result<String, String> {
    let encoder = SpoTextEncoder::new();
    let encoded = encoder.encode(&name);
    let decoded = encoder.decode(&encoded);
    Ok(decoded)
}

/// Get text encoding information
#[tauri::command]
pub fn get_text_encoding_info(_state: State<AppState>) -> TextEncodingInfo {
    let encoder = SpoTextEncoder::new();
    TextEncodingInfo {
        supported_chars: encoder.supported_chars(),
        max_name_length: MAX_NAME_LENGTH,
        max_intro_field_length: INTRO_FIELD_SIZE,
    }
}

// ============================================================================
// CIRCUIT EDITING COMMANDS
// ============================================================================

/// Update a boxer's circuit assignment
#[tauri::command]
pub fn update_boxer_circuit(
    state: State<AppState>,
    fighter_id: u8,
    circuit: CircuitType,
) -> Result<RosterDataResponse, String> {
    let mut rom_guard = state.rom.lock();

    if let Some(ref mut rom) = *rom_guard {
        let mut writer = RosterWriter::new(rom);
        writer
            .write_circuit_assignment(fighter_id, circuit)
            .map_err(|e| e.to_string())?;

        drop(rom_guard);
        let mut modified = state.modified.lock();
        *modified = true;

        let rom_guard = state.rom.lock();
        let loader = RosterLoader::new(rom_guard.as_ref().unwrap());
        let roster = loader.load_roster().map_err(|e| e.to_string())?;

        Ok(roster.into())
    } else {
        Err("No ROM loaded".to_string())
    }
}

/// Get all circuits
#[tauri::command]
pub fn get_circuits(state: State<AppState>) -> Result<Vec<Circuit>, String> {
    let rom_guard = state.rom.lock();

    let roster = if let Some(ref rom) = *rom_guard {
        let loader = RosterLoader::new(rom);
        loader.load_roster().map_err(|e| e.to_string())?
    } else {
        RosterData::new()
    };

    Ok(roster.circuits)
}

/// Get circuit types
#[tauri::command]
pub fn get_circuit_types(_state: State<AppState>) -> Vec<serde_json::Value> {
    vec![
        CircuitType::Minor,
        CircuitType::Major,
        CircuitType::World,
        CircuitType::Special,
    ]
    .into_iter()
    .map(|c| {
        serde_json::json!({
            "value": c.number(),
            "label": c.display_name(),
            "name": format!("{:?}", c),
        })
    })
    .collect()
}

// ============================================================================
// UNLOCK ORDER COMMANDS
// ============================================================================

/// Update a boxer's unlock order
#[tauri::command]
pub fn update_unlock_order(
    state: State<AppState>,
    fighter_id: u8,
    order: u8,
) -> Result<BoxerRosterEntry, String> {
    let mut rom_guard = state.rom.lock();

    if let Some(ref mut rom) = *rom_guard {
        let mut writer = RosterWriter::new(rom);
        writer
            .write_unlock_order(fighter_id, order)
            .map_err(|e| e.to_string())?;

        drop(rom_guard);
        let mut modified = state.modified.lock();
        *modified = true;

        let rom_guard = state.rom.lock();
        let loader = RosterLoader::new(rom_guard.as_ref().unwrap());
        let roster = loader.load_roster().map_err(|e| e.to_string())?;

        roster
            .get_boxer(fighter_id)
            .cloned()
            .ok_or_else(|| format!("Boxer with ID {} not found", fighter_id))
    } else {
        Err("No ROM loaded".to_string())
    }
}

/// Set champion flag for a boxer
#[tauri::command]
pub fn set_champion_status(
    state: State<AppState>,
    fighter_id: u8,
    is_champion: bool,
) -> Result<BoxerRosterEntry, String> {
    // Note: Champion status is derived from position in circuit
    // This is a convenience command that updates the boxer entry
    let rom_guard = state.rom.lock();

    if rom_guard.is_none() {
        return Err("No ROM loaded".to_string());
    }

    let loader = RosterLoader::new(rom_guard.as_ref().unwrap());
    let mut roster = loader.load_roster().map_err(|e| e.to_string())?;
    drop(rom_guard);

    if let Some(boxer) = roster.get_boxer_mut(fighter_id) {
        boxer.is_champion = is_champion;
        Ok(boxer.clone())
    } else {
        Err(format!("Boxer with ID {} not found", fighter_id))
    }
}

// ============================================================================
// INTRO TEXT COMMANDS
// ============================================================================

/// Get intro text for a boxer
#[tauri::command]
pub fn get_boxer_intro(
    state: State<AppState>,
    fighter_id: u8,
) -> Result<BoxerIntroResponse, String> {
    let rom_guard = state.rom.lock();

    if let Some(ref rom) = *rom_guard {
        let loader = RosterLoader::new(rom);
        let intro = loader
            .load_boxer_intro(fighter_id)
            .map_err(|e| e.to_string())?;
        Ok(intro.into())
    } else {
        Err("No ROM loaded".to_string())
    }
}

/// Update a specific intro field for a boxer
///
/// Fields: 0=name, 1=origin, 2=record, 3=rank, 4=quote
#[tauri::command]
pub fn update_boxer_intro_field(
    state: State<AppState>,
    fighter_id: u8,
    field_index: u8,
    text: String,
) -> Result<BoxerIntroResponse, String> {
    let mut rom_guard = state.rom.lock();

    if let Some(ref mut rom) = *rom_guard {
        let mut writer = RosterWriter::new(rom);
        writer
            .write_boxer_intro_field(fighter_id, field_index, &text)
            .map_err(|e| e.to_string())?;

        drop(rom_guard);
        let mut modified = state.modified.lock();
        *modified = true;

        let rom_guard = state.rom.lock();
        let loader = RosterLoader::new(rom_guard.as_ref().unwrap());
        let intro = loader
            .load_boxer_intro(fighter_id)
            .map_err(|e| e.to_string())?;

        Ok(intro.into())
    } else {
        Err("No ROM loaded".to_string())
    }
}

/// Get intro text (legacy - use get_boxer_intro instead)
#[tauri::command]
pub fn get_intro_text(state: State<AppState>, text_id: u8) -> Result<IntroTextResponse, String> {
    let rom_guard = state.rom.lock();

    if let Some(ref rom) = *rom_guard {
        let loader = RosterLoader::new(rom);
        let intro = loader
            .load_boxer_intro(text_id)
            .map_err(|e| e.to_string())?;

        Ok(IntroTextResponse {
            text_id,
            text: intro.intro_quote,
            fighter_id: text_id,
        })
    } else {
        Ok(IntroTextResponse {
            text_id,
            text: format!("Intro text for boxer {} (no ROM loaded)", text_id),
            fighter_id: text_id,
        })
    }
}

/// Update intro text (legacy - use update_boxer_intro_field instead)
#[tauri::command]
pub fn update_intro_text(
    state: State<AppState>,
    text_id: u8,
    text: String,
) -> Result<IntroTextResponse, String> {
    update_boxer_intro_field(state, text_id, 4, text)?; // 4 = quote field

    Ok(IntroTextResponse {
        text_id,
        text: format!("Updated intro text for boxer {}", text_id),
        fighter_id: text_id,
    })
}

/// Validate intro text
#[tauri::command]
pub fn validate_intro_text(
    _state: State<AppState>,
    text: String,
) -> Result<NameValidationResult, String> {
    let encoder = SpoTextEncoder::new();
    let can_encode = encoder.can_encode(&text);
    let encoded = encoder.encode(&text);
    let encoded_length = encoded.len();

    let mut error = None;
    if encoded_length > INTRO_FIELD_SIZE {
        error = Some(format!(
            "Text too long: {} bytes (max {})",
            encoded_length, INTRO_FIELD_SIZE
        ));
    }
    if !can_encode {
        let unsupported: Vec<char> = text.chars().filter(|c| !encoder.can_encode(&c.to_string())).collect();
        error = Some(format!("Unsupported characters: {:?}", unsupported));
    }

    Ok(NameValidationResult {
        valid: error.is_none() && can_encode,
        encoded_length,
        max_length: INTRO_FIELD_SIZE,
        can_encode,
        error,
    })
}

// ============================================================================
// CORNERMAN TEXT COMMANDS
// ============================================================================

/// Get cornerman texts for a boxer
#[tauri::command]
pub fn get_cornerman_texts(
    state: State<AppState>,
    fighter_id: u8,
) -> Result<Vec<CornermanTextResponse>, String> {
    let rom_guard = state.rom.lock();

    if let Some(ref rom) = *rom_guard {
        let loader = RosterLoader::new(rom);
        let texts = loader
            .load_cornerman_texts(fighter_id)
            .map_err(|e| e.to_string())?;

        Ok(texts
            .into_iter()
            .map(|t| CornermanTextResponse {
                id: t.id,
                boxer_key: t.boxer_key.clone(),
                fighter_id: get_boxer_id_from_key(&t.boxer_key).unwrap_or(255),
                condition: format!("{:?}", t.condition),
                text: t.text,
            })
            .collect())
    } else {
        Ok(vec![])
    }
}

// ============================================================================
// VICTORY QUOTE COMMANDS
// ============================================================================

/// Get victory quotes for a boxer
#[tauri::command]
pub fn get_victory_quotes(
    state: State<AppState>,
    fighter_id: u8,
) -> Result<Vec<VictoryQuoteResponse>, String> {
    let rom_guard = state.rom.lock();

    if let Some(ref rom) = *rom_guard {
        let loader = RosterLoader::new(rom);
        let quotes = loader
            .load_victory_quotes(fighter_id)
            .map_err(|e| e.to_string())?;

        Ok(quotes
            .into_iter()
            .map(|q| VictoryQuoteResponse {
                id: q.id,
                boxer_key: q.boxer_key.clone(),
                fighter_id: get_boxer_id_from_key(&q.boxer_key).unwrap_or(255),
                text: q.text,
                is_loss_quote: q.is_loss_quote,
            })
            .collect())
    } else {
        Ok(vec![])
    }
}

// ============================================================================
// VALIDATION COMMANDS
// ============================================================================

/// Validate all roster changes
#[tauri::command]
pub fn validate_roster_changes(state: State<AppState>) -> Result<ValidationReport, String> {
    let rom_guard = state.rom.lock();

    let roster = if let Some(ref rom) = *rom_guard {
        let loader = RosterLoader::new(rom);
        loader.load_roster().map_err(|e| e.to_string())?
    } else {
        RosterData::new()
    };

    Ok(roster.validate())
}

/// Reset roster to defaults
#[tauri::command]
pub fn reset_roster_to_defaults(_state: State<AppState>) -> Result<RosterDataResponse, String> {
    let roster = RosterData::new();
    Ok(roster.into())
}

// ============================================================================
// ROM OFFSET/INFO COMMANDS
// ============================================================================

/// Get ROM offsets for roster data
#[tauri::command]
pub fn get_roster_offsets(_state: State<AppState>) -> RomOffsetInfo {
    RomOffsetInfo {
        name_table_offset: BOXER_NAME_POINTERS - 0x100,
        name_pointers_offset: BOXER_NAME_POINTERS,
        circuit_table_offset: CIRCUIT_TABLE,
        unlock_table_offset: UNLOCK_ORDER_TABLE,
        intro_table_offset: BOXER_INTRO_TABLE,
    }
}

/// Scan ROM for potential text tables (research tool)
#[tauri::command]
pub fn scan_for_text_tables(state: State<AppState>) -> Result<Vec<serde_json::Value>, String> {
    let rom_guard = state.rom.lock();

    if rom_guard.is_none() {
        return Ok(vec![]);
    }

    // Return known text locations
    Ok(vec![
        serde_json::json!({
            "address": format!("0x{:06X}", BOXER_NAME_POINTERS),
            "description": "Boxer name pointer table",
            "confidence": 100,
        }),
        serde_json::json!({
            "address": format!("0x{:06X}", CIRCUIT_TABLE),
            "description": "Circuit assignment table",
            "confidence": 100,
        }),
        serde_json::json!({
            "address": format!("0x{:06X}", UNLOCK_ORDER_TABLE),
            "description": "Unlock order table",
            "confidence": 100,
        }),
        serde_json::json!({
            "address": format!("0x{:06X}", BOXER_INTRO_TABLE),
            "description": "Boxer intro data table",
            "confidence": 100,
        }),
    ])
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

fn get_boxer_id_from_key(key: &str) -> Option<u8> {
    match key {
        "gabby_jay" => Some(0),
        "bear_hugger" => Some(1),
        "piston_hurricane" => Some(2),
        "bald_bull" => Some(3),
        "bob_charlie" => Some(4),
        "dragon_chan" => Some(5),
        "masked_muscle" => Some(6),
        "mr_sandman" => Some(7),
        "aran_ryan" => Some(8),
        "heike_kagero" => Some(9),
        "mad_clown" => Some(10),
        "super_macho_man" => Some(11),
        "narcis_prince" => Some(12),
        "hoy_quarlow" => Some(13),
        "rick_bruiser" => Some(14),
        "nick_bruiser" => Some(15),
        _ => None,
    }
}
