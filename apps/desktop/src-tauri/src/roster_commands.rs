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
use emulator_core::CreatorSessionState;
use rom_core::{
    roster::{
        BoxerIntro, BoxerRosterEntry, Circuit, CircuitType, RosterData, RosterLoader,
        ValidationReport, BOXER_INTRO_TABLE, BOXER_NAME_POINTERS, CIRCUIT_TABLE, INTRO_FIELD_SIZE,
        MAX_NAME_LENGTH, UNLOCK_ORDER_TABLE,
    },
    SpoTextEncoder, CREATOR_ERROR_BOXER_NOT_FOUND, CREATOR_ERROR_GENERIC,
    CREATOR_ERROR_INVALID_INTRO_SLOT, CREATOR_ERROR_INVALID_INTRO_TEXT, CREATOR_ERROR_INVALID_NAME,
    CREATOR_SESSION_STATUS_COMMIT_FAILED, CREATOR_SESSION_STATUS_DRAFT_READY,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatorCommitResponse {
    pub boxer: BoxerRosterEntry,
    pub intro_text_id: u8,
    pub intro_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatorSessionValidationResponse {
    pub valid: bool,
    pub status: u8,
    pub error_code: u8,
    pub message: Option<String>,
}

fn validate_creator_session_payload(
    rom: &rom_core::Rom,
    session: &CreatorSessionState,
) -> CreatorSessionValidationResponse {
    let loader = RosterLoader::new(rom);
    let roster = match loader.load_roster() {
        Ok(roster) => roster,
        Err(error) => {
            return CreatorSessionValidationResponse {
                valid: false,
                status: CREATOR_SESSION_STATUS_COMMIT_FAILED,
                error_code: CREATOR_ERROR_GENERIC,
                message: Some(error.to_string()),
            };
        }
    };

    if roster.get_boxer(session.boxer_id).is_none() {
        return CreatorSessionValidationResponse {
            valid: false,
            status: CREATOR_SESSION_STATUS_COMMIT_FAILED,
            error_code: CREATOR_ERROR_BOXER_NOT_FOUND,
            message: Some(format!("Boxer with ID {} not found", session.boxer_id)),
        };
    }

    if loader.load_boxer_intro(session.intro_text_id).is_err() {
        return CreatorSessionValidationResponse {
            valid: false,
            status: CREATOR_SESSION_STATUS_COMMIT_FAILED,
            error_code: CREATOR_ERROR_INVALID_INTRO_SLOT,
            message: Some(format!(
                "Intro text slot {} not found",
                session.intro_text_id
            )),
        };
    }

    let encoder = SpoTextEncoder::new();
    if let Err(invalid) = encoder.validate(&session.name_text) {
        return CreatorSessionValidationResponse {
            valid: false,
            status: CREATOR_SESSION_STATUS_COMMIT_FAILED,
            error_code: CREATOR_ERROR_INVALID_NAME,
            message: Some(format!("Invalid name characters: {:?}", invalid)),
        };
    }
    if encoder.encode(&session.name_text).len() > MAX_NAME_LENGTH {
        return CreatorSessionValidationResponse {
            valid: false,
            status: CREATOR_SESSION_STATUS_COMMIT_FAILED,
            error_code: CREATOR_ERROR_INVALID_NAME,
            message: Some(format!(
                "Name too long: {} bytes (max {})",
                encoder.encode(&session.name_text).len(),
                MAX_NAME_LENGTH
            )),
        };
    }

    if !encoder.can_encode(&session.intro_text) {
        let unsupported: Vec<char> = session
            .intro_text
            .chars()
            .filter(|c| !encoder.can_encode(&c.to_string()))
            .collect();
        return CreatorSessionValidationResponse {
            valid: false,
            status: CREATOR_SESSION_STATUS_COMMIT_FAILED,
            error_code: CREATOR_ERROR_INVALID_INTRO_TEXT,
            message: Some(format!("Invalid intro text characters: {:?}", unsupported)),
        };
    }
    let intro_encoded_len = encoder.encode(&session.intro_text).len();
    if intro_encoded_len > INTRO_FIELD_SIZE {
        return CreatorSessionValidationResponse {
            valid: false,
            status: CREATOR_SESSION_STATUS_COMMIT_FAILED,
            error_code: CREATOR_ERROR_INVALID_INTRO_TEXT,
            message: Some(format!(
                "Intro text too long: {} bytes (max {})",
                intro_encoded_len, INTRO_FIELD_SIZE
            )),
        };
    }

    CreatorSessionValidationResponse {
        valid: true,
        status: CREATOR_SESSION_STATUS_DRAFT_READY,
        error_code: 0,
        message: None,
    }
}

pub(crate) fn validate_creator_session_internal(
    state: &AppState,
    session: &CreatorSessionState,
) -> Result<CreatorSessionValidationResponse, String> {
    let rom_guard = state.rom.lock();
    if let Some(ref rom) = *rom_guard {
        Ok(validate_creator_session_payload(rom, session))
    } else {
        Err("No ROM loaded".to_string())
    }
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
    pub validation: IntroValidation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntroValidation {
    pub name_valid: bool,
    pub name_length: usize,
    pub origin_valid: bool,
    pub origin_length: usize,
    pub record_valid: bool,
    pub record_length: usize,
    pub rank_valid: bool,
    pub rank_length: usize,
    pub quote_valid: bool,
    pub quote_length: usize,
    pub all_valid: bool,
    pub unsupported_chars: Vec<char>,
}

fn compute_intro_validation(intro: &BoxerIntro, encoder: &SpoTextEncoder) -> IntroValidation {
    let name_len = encoder.encode(&intro.name_text).len();
    let origin_len = encoder.encode(&intro.origin_text).len();
    let record_len = encoder.encode(&intro.record_text).len();
    let rank_len = encoder.encode(&intro.rank_text).len();
    let quote_len = encoder.encode(&intro.intro_quote).len();

    let name_valid = name_len <= INTRO_FIELD_SIZE && encoder.can_encode(&intro.name_text);
    let origin_valid = origin_len <= INTRO_FIELD_SIZE && encoder.can_encode(&intro.origin_text);
    let record_valid = record_len <= INTRO_FIELD_SIZE && encoder.can_encode(&intro.record_text);
    let rank_valid = rank_len <= INTRO_FIELD_SIZE && encoder.can_encode(&intro.rank_text);
    let quote_valid = quote_len <= INTRO_FIELD_SIZE && encoder.can_encode(&intro.intro_quote);

    let mut unsupported: Vec<char> = Vec::new();
    unsupported.extend(encoder.get_unsupported_chars(&intro.name_text));
    unsupported.extend(encoder.get_unsupported_chars(&intro.origin_text));
    unsupported.extend(encoder.get_unsupported_chars(&intro.record_text));
    unsupported.extend(encoder.get_unsupported_chars(&intro.rank_text));
    unsupported.extend(encoder.get_unsupported_chars(&intro.intro_quote));
    unsupported.sort();
    unsupported.dedup();

    IntroValidation {
        name_valid,
        name_length: name_len,
        origin_valid,
        origin_length: origin_len,
        record_valid,
        record_length: record_len,
        rank_valid,
        rank_length: rank_len,
        quote_valid,
        quote_length: quote_len,
        all_valid: name_valid && origin_valid && record_valid && rank_valid && quote_valid,
        unsupported_chars: unsupported,
    }
}

impl From<BoxerIntro> for BoxerIntroResponse {
    fn from(intro: BoxerIntro) -> Self {
        let encoder = SpoTextEncoder::new();
        let validation = compute_intro_validation(&intro, &encoder);
        Self {
            fighter_id: get_boxer_id_from_key(&intro.boxer_key).unwrap_or(255),
            boxer_key: intro.boxer_key,
            name_text: intro.name_text,
            origin_text: intro.origin_text,
            record_text: intro.record_text,
            rank_text: intro.rank_text,
            intro_quote: intro.intro_quote,
            validation,
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

/// Cornerman text response — matches the frontend CornermanTextDto shape
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CornermanTextResponse {
    pub id: u8,
    pub boxer_key: String,
    pub fighter_id: u8,
    pub round: u8,
    pub condition: String,
    pub condition_value: u8,
    pub text: String,
    pub byte_length: usize,
    pub max_length: usize,
    pub is_valid: bool,
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

/// Validate a creator session payload against the current ROM state.
#[tauri::command]
pub fn validate_creator_session(
    state: State<AppState>,
    session: CreatorSessionState,
) -> Result<CreatorSessionValidationResponse, String> {
    validate_creator_session_internal(&state, &session)
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
        let unsupported: Vec<char> = name
            .chars()
            .filter(|c| !encoder.can_encode(&c.to_string()))
            .collect();
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
// INTRO TEXT COMMANDS
// ============================================================================

/// Get intro text for a boxer
#[tauri::command]
pub fn get_boxer_intro(
    state: State<AppState>,
    boxer_key: String,
) -> Result<BoxerIntroResponse, String> {
    let fighter_id = get_boxer_id_from_key(&boxer_key)
        .ok_or_else(|| format!("Unknown boxer key: {}", boxer_key))?;

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
        let unsupported: Vec<char> = text
            .chars()
            .filter(|c| !encoder.can_encode(&c.to_string()))
            .collect();
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
    boxer_key: String,
) -> Result<Vec<CornermanTextResponse>, String> {
    let fighter_id = get_boxer_id_from_key(&boxer_key)
        .ok_or_else(|| format!("Unknown boxer key: {}", boxer_key))?;

    let rom_guard = state.rom.lock();

    if let Some(ref rom) = *rom_guard {
        let loader = RosterLoader::new(rom);
        let texts = loader
            .load_cornerman_texts(fighter_id)
            .map_err(|e| e.to_string())?;

        let encoder = SpoTextEncoder::new();
        Ok(texts
            .into_iter()
            .map(|t| {
                let byte_length = encoder.encode(&t.text).len();
                let is_valid = byte_length <= t.max_length && encoder.can_encode(&t.text);
                CornermanTextResponse {
                    id: t.id,
                    boxer_key: t.boxer_key.clone(),
                    fighter_id: get_boxer_id_from_key(&t.boxer_key).unwrap_or(255),
                    round: t.round,
                    condition: t.condition.display_name().to_string(),
                    condition_value: t.condition.to_byte(),
                    text: t.text,
                    byte_length,
                    max_length: t.max_length,
                    is_valid,
                }
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
    boxer_key: String,
) -> Result<Vec<VictoryQuoteResponse>, String> {
    let fighter_id = get_boxer_id_from_key(&boxer_key)
        .ok_or_else(|| format!("Unknown boxer key: {}", boxer_key))?;

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

pub(crate) fn get_boxer_id_from_key(key: &str) -> Option<u8> {
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
