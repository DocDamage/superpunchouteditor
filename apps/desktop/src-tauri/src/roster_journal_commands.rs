//! Journal-backed roster mutation commands.
//!
//! Read/validation commands remain in `roster_commands`; these wrappers replace only mutations that
//! have proven ROM writers. They use `AppState::commit_rom_transform` so each logical user action is
//! one atomic canonical journal transaction.

use tauri::State;

use crate::roster_commands::{
    BoxerIntroResponse, CreatorCommitResponse, IntroTextResponse, RosterDataResponse,
};
use crate::AppState;
use emulator_core::CreatorSessionState;
use rom_core::roster::{BoxerRosterEntry, CircuitType, RosterLoader, RosterWriter};
use rom_core::SpoTextEncoder;

fn load_boxer(state: &AppState, fighter_id: u8) -> Result<BoxerRosterEntry, String> {
    let rom_guard = state.rom.lock();
    let rom = rom_guard.as_ref().ok_or("No ROM loaded")?;
    let roster = RosterLoader::new(rom)
        .load_roster()
        .map_err(|error| error.to_string())?;
    roster
        .get_boxer(fighter_id)
        .cloned()
        .ok_or_else(|| format!("Boxer with ID {fighter_id} not found"))
}

fn load_roster(state: &AppState) -> Result<RosterDataResponse, String> {
    let rom_guard = state.rom.lock();
    let rom = rom_guard.as_ref().ok_or("No ROM loaded")?;
    let roster = RosterLoader::new(rom)
        .load_roster()
        .map_err(|error| error.to_string())?;
    Ok(roster.into())
}

#[tauri::command]
pub fn update_boxer_name(
    state: State<AppState>,
    fighter_id: u8,
    new_name: String,
) -> Result<BoxerRosterEntry, String> {
    let encoder = SpoTextEncoder::new();
    encoder
        .validate(&new_name)
        .map_err(|invalid| format!("Invalid characters: {invalid:?}"))?;
    let name = new_name.trim().to_string();
    state.commit_rom_transform(format!("Rename boxer {fighter_id}"), |rom| {
        RosterWriter::new(rom)
            .write_boxer_name(fighter_id, &name)
            .map_err(|error| error.to_string())
    })?;
    load_boxer(&state, fighter_id)
}

#[tauri::command]
pub fn update_boxer_circuit(
    state: State<AppState>,
    fighter_id: u8,
    circuit: CircuitType,
) -> Result<RosterDataResponse, String> {
    state.commit_rom_transform(format!("Change boxer {fighter_id} circuit"), |rom| {
        RosterWriter::new(rom)
            .write_circuit_assignment(fighter_id, circuit)
            .map_err(|error| error.to_string())
    })?;
    load_roster(&state)
}

#[tauri::command]
pub fn update_unlock_order(
    state: State<AppState>,
    fighter_id: u8,
    order: u8,
) -> Result<BoxerRosterEntry, String> {
    state.commit_rom_transform(format!("Change boxer {fighter_id} unlock order"), |rom| {
        RosterWriter::new(rom)
            .write_unlock_order(fighter_id, order)
            .map_err(|error| error.to_string())
    })?;
    load_boxer(&state, fighter_id)
}

#[tauri::command]
pub fn update_boxer_intro_field(
    state: State<AppState>,
    fighter_id: u8,
    field_index: u8,
    text: String,
) -> Result<BoxerIntroResponse, String> {
    if field_index > 4 {
        return Err(format!(
            "Invalid intro field index {field_index}; expected 0..=4"
        ));
    }
    let normalized = text.trim().to_string();
    state.commit_rom_transform(
        format!("Update boxer {fighter_id} intro field {field_index}"),
        |rom| {
            RosterWriter::new(rom)
                .write_boxer_intro_field(fighter_id, field_index, &normalized)
                .map_err(|error| error.to_string())
        },
    )?;
    let rom_guard = state.rom.lock();
    let intro = RosterLoader::new(rom_guard.as_ref().ok_or("No ROM loaded")?)
        .load_boxer_intro(fighter_id)
        .map_err(|error| error.to_string())?;
    Ok(intro.into())
}

#[tauri::command]
pub fn update_intro_text(
    state: State<AppState>,
    text_id: u8,
    text: String,
) -> Result<IntroTextResponse, String> {
    let normalized = text.trim().to_string();
    update_boxer_intro_field(state, text_id, 4, normalized.clone())?;
    Ok(IntroTextResponse {
        text_id,
        text: normalized,
        fighter_id: text_id,
    })
}

#[tauri::command]
pub fn commit_creator_session(
    state: State<AppState>,
    session: CreatorSessionState,
) -> Result<CreatorCommitResponse, String> {
    let validation = crate::roster_commands::validate_creator_session_internal(&state, &session)?;
    if !validation.valid {
        return Err(validation
            .message
            .unwrap_or_else(|| "Creator session validation failed".to_string()));
    }

    let boxer_id = session.boxer_id;
    let intro_text_id = session.intro_text_id;
    let name = session.name_text.trim().to_string();
    let intro = session.intro_text.trim().to_string();
    let circuit = CircuitType::from_byte(session.circuit);
    let unlock_order = session.unlock_order;

    state.commit_rom_transform(
        format!("Commit creator session for boxer {boxer_id}"),
        |rom| {
            let mut writer = RosterWriter::new(rom);
            writer
                .write_boxer_name(boxer_id, &name)
                .map_err(|error| error.to_string())?;
            writer
                .write_circuit_assignment(boxer_id, circuit)
                .map_err(|error| error.to_string())?;
            writer
                .write_unlock_order(boxer_id, unlock_order)
                .map_err(|error| error.to_string())?;
            writer
                .write_boxer_intro_field(intro_text_id, 4, &intro)
                .map_err(|error| error.to_string())?;
            Ok(())
        },
    )?;

    Ok(CreatorCommitResponse {
        boxer: load_boxer(&state, boxer_id)?,
        intro_text_id,
        intro_text: intro,
    })
}

#[tauri::command]
pub fn set_champion_status(
    _state: State<AppState>,
    _fighter_id: u8,
    _is_champion: bool,
) -> Result<BoxerRosterEntry, String> {
    Err("Champion status is derived from roster structure; no proven persistent ROM writer exists. This control is experimental and disabled.".to_string())
}

#[tauri::command]
pub fn reset_roster_to_defaults(_state: State<AppState>) -> Result<RosterDataResponse, String> {
    Err("Reset-to-defaults has no proven byte-level restoration map and is disabled rather than returning a false success.".to_string())
}
