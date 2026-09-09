//! Script/AI Commands
//!
//! Commands for script reading and fighter parameter editing.

use tauri::State;

use crate::app_state::AppState;
use script_core::ai_behavior::bytecode::{
    decode_ai_stream, encode_ai_stream, validate_script_target, AiInstruction,
};
use script_core::ai_behavior::bytecode::{AiScriptRegion, VERIFIED_USA_AI_REGIONS};
use script_core::{
    BoxerHeader, EditableFighterParams, ParamValidationResult, ScriptReader, ScriptRecord,
};

/// Source catalog only: the caller must not assume these USA ranges apply to
/// other regions or relocated hacks solely because their bytes decode.
#[tauri::command]
pub fn get_usa_ai_script_regions(state: State<AppState>) -> Result<Vec<AiScriptRegion>, String> {
    if state.get_rom_sha1().as_deref() != Some(USA_SHA1) {
        return Err("Verified AI editing requires the supported USA base ROM".into());
    }
    Ok(VERIFIED_USA_AI_REGIONS.to_vec())
}

const USA_SHA1: &str = "3604c855790f37db567e9b425252625045f86697";

// Exact interval in the pinned upstream animation disassembler example.
#[cfg(test)]
const ANIMATION_REFERENCE_PC: usize = 0x68ebe;
#[cfg(test)]
const ANIMATION_REFERENCE_LEN: usize = 133;

fn animation_region(
    id: Option<&str>,
) -> Result<&'static script_core::animation_bytecode::AnimationRegion, String> {
    script_core::animation_bytecode::VERIFIED_USA_ANIMATION_REGIONS
        .iter()
        .find(|region| region.id == id.unwrap_or("reference_0d8ebe"))
        .ok_or_else(|| "Unknown verified animation region".into())
}

#[tauri::command]
pub fn get_usa_animation_regions(
    state: State<AppState>,
) -> Result<Vec<script_core::animation_bytecode::AnimationRegion>, String> {
    if state.get_rom_sha1().as_deref() != Some(USA_SHA1) {
        return Err("Verified animation editing requires the supported USA base ROM".into());
    }
    Ok(script_core::animation_bytecode::VERIFIED_USA_ANIMATION_REGIONS.to_vec())
}

#[tauri::command]
pub fn read_reference_animation(
    state: State<AppState>,
    region_id: Option<String>,
) -> Result<AnimationSnapshot, String> {
    read_animation_snapshot(&state, region_id.as_deref())
}

fn read_animation_snapshot(
    state: &AppState,
    region_id: Option<&str>,
) -> Result<AnimationSnapshot, String> {
    let guard = state.rom_session.lock();
    let session = guard.as_ref().ok_or("No ROM loaded")?;
    if session.base().sha1() != USA_SHA1 {
        return Err("Verified animation editing requires the supported USA base ROM".into());
    }
    let current = session.materialize().map_err(|e| e.to_string())?;
    drop(guard);
    let region = animation_region(region_id)?;
    let bytes = current
        .bytes
        .get(region.pc_offset..region.pc_offset + region.length)
        .ok_or("Animation reference is outside ROM")?;
    Ok(AnimationSnapshot {
        timing: script_core::animation_bytecode::trace_animation_timing(
            bytes,
            region.pc_offset,
            4096,
        )?,
        instructions: script_core::animation_bytecode::decode_animation_stream(bytes)?,
        references: script_core::animation_bytecode::analyze_animation_catalog(
            &current.bytes,
            script_core::animation_bytecode::VERIFIED_USA_ANIMATION_REGIONS,
        )?,
    })
}

#[derive(serde::Serialize)]
pub struct AnimationSnapshot {
    timing: script_core::animation_bytecode::TimingTrace,
    instructions: Vec<script_core::animation_bytecode::AnimationInstruction>,
    references: Vec<script_core::animation_bytecode::CatalogReference>,
}

#[tauri::command]
pub fn get_animation_references(
    state: State<AppState>,
) -> Result<Vec<script_core::animation_bytecode::CatalogReference>, String> {
    if state.get_rom_sha1().as_deref() != Some(USA_SHA1) {
        return Err("Verified animation analysis requires the supported USA base ROM".into());
    }
    let current = state.materialize_current_rom()?;
    script_core::animation_bytecode::analyze_animation_catalog(
        &current.bytes,
        script_core::animation_bytecode::VERIFIED_USA_ANIMATION_REGIONS,
    )
}

#[tauri::command]
pub fn update_reference_animation_duration(
    state: State<AppState>,
    region_id: Option<String>,
    expected_bytes: Vec<u8>,
    instruction_offset: usize,
    duration: u8,
) -> Result<(), String> {
    update_animation_duration_internal(
        &state,
        region_id.as_deref(),
        &expected_bytes,
        instruction_offset,
        duration,
    )
}

fn update_animation_duration_internal(
    state: &AppState,
    region_id: Option<&str>,
    expected_bytes: &[u8],
    instruction_offset: usize,
    duration: u8,
) -> Result<(), String> {
    update_animation_durations_internal(
        state,
        region_id,
        expected_bytes,
        &[script_core::animation_bytecode::DurationEdit {
            instruction_offset,
            duration,
        }],
    )
}

#[tauri::command]
pub fn update_animation_durations(
    state: State<AppState>,
    region_id: String,
    expected_bytes: Vec<u8>,
    edits: Vec<script_core::animation_bytecode::DurationEdit>,
) -> Result<(), String> {
    update_animation_durations_internal(&state, Some(&region_id), &expected_bytes, &edits)
}

fn update_animation_durations_internal(
    state: &AppState,
    region_id: Option<&str>,
    expected_bytes: &[u8],
    edits: &[script_core::animation_bytecode::DurationEdit],
) -> Result<(), String> {
    let region = animation_region(region_id)?;
    if expected_bytes.len() != region.length {
        return Err("Expected the complete verified animation interval".into());
    }
    use script_core::animation_bytecode::{analyze_animation_references, TargetResolution};
    for reference in analyze_animation_references(expected_bytes, region.pc_offset)? {
        if matches!(
            reference.resolution,
            TargetResolution::InsideOperand | TargetResolution::InvalidAddress
        ) {
            return Err(format!(
                "Animation has an invalid script reference at +{:#x}",
                reference.instruction_offset
            ));
        }
    }
    let replacement = script_core::animation_bytecode::edit_pose_durations(expected_bytes, edits)?;
    state.commit_rom_transform_for_base(
        "Edit animation frame duration",
        Some(USA_SHA1),
        |rom| {
            if rom
                .read_bytes(region.pc_offset, region.length)
                .map_err(|e| e.to_string())?
                != expected_bytes
            {
                return Err("Animation changed; reload before editing".into());
            }
            rom.write_bytes(region.pc_offset, &replacement)
                .map_err(|e| e.to_string())
        },
    )?;
    Ok(())
}

#[cfg(test)]
mod bytecode_tests {
    use super::*;

    #[test]
    #[ignore = "requires user-supplied USA ROM via SPO_USA_ROM"]
    fn usa_animation_batch_is_one_undoable_transaction() {
        use script_core::animation_bytecode::DurationEdit;
        let original = std::fs::read(std::env::var("SPO_USA_ROM").unwrap()).unwrap();
        let state = AppState::new(manifest_core::Manifest::empty());
        state.install_rom_session(rom_core::Rom::new(original.clone()), "local.sfc".into());
        let id = "gabby_jay_098267";
        let snapshot = read_animation_snapshot(&state, Some(id)).unwrap();
        let expected =
            script_core::animation_bytecode::encode_animation_stream(&snapshot.instructions)
                .unwrap();
        let edits: Vec<_> = snapshot
            .instructions
            .iter()
            .filter(|i| i.displayed_pose().is_some())
            .map(|i| DurationEdit {
                instruction_offset: i.offset,
                duration: i.operands[1].wrapping_add(1),
            })
            .collect();
        assert!(edits.len() > 1);
        let mut invalid = edits.clone();
        invalid.push(DurationEdit {
            instruction_offset: usize::MAX,
            duration: 1,
        });
        assert!(
            update_animation_durations_internal(&state, Some(id), &expected, &invalid).is_err()
        );
        assert_eq!(state.materialize_current_rom().unwrap().bytes, original);
        update_animation_durations_internal(&state, Some(id), &expected, &edits).unwrap();
        let edited = state.materialize_current_rom().unwrap().bytes;
        let differences = original.iter().zip(&edited).filter(|(a, b)| a != b).count();
        assert_eq!(differences, edits.len());
        state.undo_journal().unwrap();
        assert_eq!(state.materialize_current_rom().unwrap().bytes, original);
        state.redo_journal().unwrap();
        assert_eq!(state.materialize_current_rom().unwrap().bytes, edited);
    }

    #[test]
    fn animation_snapshot_rejects_unsupported_source() {
        let state = AppState::new(manifest_core::Manifest::empty());
        state.install_rom_session(rom_core::Rom::new(vec![0; 0x80000]), "fixture.sfc".into());
        assert!(read_animation_snapshot(&state, None).is_err());
    }

    #[test]
    #[ignore = "requires user-supplied USA ROM via SPO_USA_ROM"]
    fn usa_animation_snapshot_tracks_current_journal_revision() {
        let original = std::fs::read(std::env::var("SPO_USA_ROM").unwrap()).unwrap();
        let state = AppState::new(manifest_core::Manifest::empty());
        state.install_rom_session(rom_core::Rom::new(original), "local.sfc".into());
        let before = read_animation_snapshot(&state, None).unwrap();
        let expected =
            script_core::animation_bytecode::encode_animation_stream(&before.instructions).unwrap();
        let frame = before
            .instructions
            .iter()
            .find(|i| i.displayed_pose().is_some())
            .unwrap();
        let duration = frame.operands[1].wrapping_add(1);
        update_animation_duration_internal(&state, None, &expected, frame.offset, duration)
            .unwrap();
        let after = read_animation_snapshot(&state, None).unwrap();
        assert_eq!(
            after
                .instructions
                .iter()
                .find(|i| i.offset == frame.offset)
                .unwrap()
                .displayed_pose(),
            Some((frame.operands[0], duration))
        );
        let current = state.materialize_current_rom().unwrap();
        let region = animation_region(None).unwrap();
        let expected_timing = script_core::animation_bytecode::trace_animation_timing(
            &current.bytes[region.pc_offset..region.pc_offset + region.length],
            region.pc_offset,
            4096,
        )
        .unwrap();
        assert_eq!(
            serde_json::to_value(&after.timing).unwrap(),
            serde_json::to_value(&expected_timing).unwrap()
        );
        let expected_graph = script_core::animation_bytecode::analyze_animation_catalog(
            &current.bytes,
            script_core::animation_bytecode::VERIFIED_USA_ANIMATION_REGIONS,
        )
        .unwrap();
        assert_eq!(
            serde_json::to_value(&after.references).unwrap(),
            serde_json::to_value(&expected_graph).unwrap()
        );
        state.undo_journal().unwrap();
        let restored = read_animation_snapshot(&state, None).unwrap();
        assert_eq!(restored.instructions, before.instructions);
        assert_eq!(
            serde_json::to_value(&restored.timing).unwrap(),
            serde_json::to_value(&before.timing).unwrap()
        );
        assert!(read_animation_snapshot(&state, Some("unknown")).is_err());
    }

    #[test]
    #[ignore = "requires user-supplied USA ROM via SPO_USA_ROM"]
    fn usa_animation_catalog_edits_target_selected_region() {
        let original = std::fs::read(std::env::var("SPO_USA_ROM").unwrap()).unwrap();
        let state = AppState::new(manifest_core::Manifest::empty());
        state.install_rom_session(rom_core::Rom::new(original.clone()), "local.sfc".into());
        for region in script_core::animation_bytecode::VERIFIED_USA_ANIMATION_REGIONS {
            let expected = &original[region.pc_offset..region.pc_offset + region.length];
            let instructions =
                script_core::animation_bytecode::decode_animation_stream(expected).unwrap();
            let frame = instructions
                .iter()
                .find(|i| i.displayed_pose().is_some())
                .unwrap();
            update_animation_duration_internal(
                &state,
                Some(region.id),
                expected,
                frame.offset,
                frame.operands[1].wrapping_add(1),
            )
            .unwrap();
            let edited = state.materialize_current_rom().unwrap().bytes;
            let changed: Vec<_> = original
                .iter()
                .zip(&edited)
                .enumerate()
                .filter_map(|(index, (a, b))| (a != b).then_some(index))
                .collect();
            assert_eq!(changed, vec![region.pc_offset + frame.offset + 2]);
            state.undo_journal().unwrap();
            assert_eq!(state.materialize_current_rom().unwrap().bytes, original);
        }
    }

    #[test]
    #[ignore = "requires user-supplied USA ROM via SPO_USA_ROM"]
    fn usa_animation_duration_is_atomic_and_undoable() {
        let original = std::fs::read(std::env::var("SPO_USA_ROM").unwrap()).unwrap();
        let state = AppState::new(manifest_core::Manifest::empty());
        state.install_rom_session(rom_core::Rom::new(original.clone()), "local.sfc".into());
        let expected =
            &original[ANIMATION_REFERENCE_PC..ANIMATION_REFERENCE_PC + ANIMATION_REFERENCE_LEN];
        let instructions =
            script_core::animation_bytecode::decode_animation_stream(expected).unwrap();
        let frame = instructions
            .iter()
            .find(|i| i.displayed_pose().is_some())
            .unwrap();
        let duration = frame.operands[1].wrapping_add(1);
        update_animation_duration_internal(&state, None, expected, frame.offset, duration).unwrap();
        let edited = state.materialize_current_rom().unwrap().bytes;
        let changed: Vec<_> = original
            .iter()
            .zip(&edited)
            .enumerate()
            .filter_map(|(index, (a, b))| (a != b).then_some(index))
            .collect();
        assert_eq!(changed, vec![ANIMATION_REFERENCE_PC + frame.offset + 2]);
        assert!(
            update_animation_duration_internal(&state, None, expected, frame.offset, duration)
                .is_err()
        );
        assert_eq!(state.materialize_current_rom().unwrap().bytes, edited);
        state.undo_journal().unwrap();
        assert_eq!(state.materialize_current_rom().unwrap().bytes, original);
        state.redo_journal().unwrap();
        assert_eq!(state.materialize_current_rom().unwrap().bytes, edited);
    }

    #[test]
    #[ignore = "requires user-supplied USA ROM via SPO_USA_ROM"]
    fn usa_wait_edit_changes_one_byte_and_undo_restores_source() {
        let original = std::fs::read(std::env::var("SPO_USA_ROM").unwrap()).unwrap();
        let state = AppState::new(manifest_core::Manifest::empty());
        state.install_rom_session(
            rom_core::Rom::new(original.clone()),
            "local-test.sfc".into(),
        );
        assert_eq!(state.get_rom_sha1().as_deref(), Some(USA_SHA1));
        let region = &VERIFIED_USA_AI_REGIONS[0];
        let stream = original[region.pc_offset..region.pc_offset + region.length].to_vec();
        let decoded = decode_ai_stream(&stream).unwrap();
        let call = decoded
            .iter()
            .find(|instruction| instruction.opcode == 0x44)
            .unwrap();
        let invalid_target = ((region.pc_offset & 0x7fff) as u16 | 0x8000) + 1;
        assert!(update_ai_script_operands_for_base(
            &state,
            region.pc_offset,
            stream.clone(),
            call.offset,
            invalid_target.to_le_bytes().to_vec(),
            Some(USA_SHA1),
        )
        .unwrap_err()
        .contains("instruction boundary"));
        assert_eq!(state.materialize_current_rom().unwrap().bytes, original);
        let wait = decoded
            .iter()
            .find(|instruction| instruction.opcode == 0x2a)
            .unwrap();
        let value = wait.operands[0].wrapping_add(1);
        update_ai_script_operands_for_base(
            &state,
            region.pc_offset,
            stream,
            wait.offset,
            vec![value],
            Some(USA_SHA1),
        )
        .unwrap();
        let edited = state.materialize_current_rom().unwrap().bytes;
        let changed: Vec<_> = original
            .iter()
            .zip(&edited)
            .enumerate()
            .filter(|(_, (a, b))| a != b)
            .map(|(index, _)| index)
            .collect();
        assert_eq!(changed, vec![region.pc_offset + wait.offset + 1]);
        assert_eq!(edited[changed[0]], value);
        state.undo_journal().unwrap();
        assert_eq!(state.materialize_current_rom().unwrap().bytes, original);
        state.redo_journal().unwrap();
        assert_eq!(state.materialize_current_rom().unwrap().bytes, edited);
    }

    #[test]
    fn wrong_base_identity_cannot_commit_even_when_stream_matches() {
        let state = AppState::new(manifest_core::Manifest::empty());
        let original = vec![0x2a, 30, 0x00];
        state.install_rom_session(rom_core::Rom::new(original.clone()), "synthetic.sfc".into());
        let error = update_ai_script_operands_for_base(
            &state,
            0,
            original.clone(),
            0,
            vec![60],
            Some(USA_SHA1),
        )
        .unwrap_err();
        assert!(error.contains("identity"));
        assert_eq!(state.materialize_current_rom().unwrap().bytes, original);
    }

    #[test]
    fn invalid_legacy_parameter_mapping_cannot_corrupt_pointers() {
        let state = AppState::new(manifest_core::Manifest::empty());
        let original = vec![0x55; 0x50000];
        state.install_rom_session(rom_core::Rom::new(original.clone()), "synthetic.sfc".into());
        let params = EditableFighterParams {
            palette_id: 2,
            attack_power: 30,
            defense_rating: 40,
            speed_rating: 50,
        };
        let error = update_fighter_params_internal(&state, 0, params.clone()).unwrap_err();
        assert!(error.contains("ROM pointers"));
        for index in [16, 65536, usize::MAX] {
            assert!(update_fighter_params_internal(&state, index, params.clone()).is_err());
        }
        let invalid = EditableFighterParams {
            speed_rating: 255,
            ..params
        };
        assert!(update_fighter_params_internal(&state, 0, invalid).is_err());
        assert_eq!(state.materialize_current_rom().unwrap().bytes, original);
    }

    #[test]
    fn operand_commit_preserves_neighbors_and_supports_undo_redo() {
        let state = AppState::new(manifest_core::Manifest::empty());
        let original = vec![0xff, 0x2a, 30, 0x00, 0xff];
        state.install_rom_session(rom_core::Rom::new(original.clone()), "synthetic.sfc".into());
        let expected = vec![0x2a, 30, 0x00];
        let result =
            update_ai_script_operands_internal(&state, 1, expected.clone(), 0, vec![60]).unwrap();
        assert_eq!(result[0].operands, vec![60]);
        let edited = vec![0xff, 0x2a, 60, 0x00, 0xff];
        assert_eq!(state.materialize_current_rom().unwrap().bytes, edited);
        assert!(update_ai_script_operands_internal(&state, 1, expected, 0, vec![90]).is_err());
        assert_eq!(state.materialize_current_rom().unwrap().bytes, edited);
        state.undo_journal().unwrap();
        assert_eq!(state.materialize_current_rom().unwrap().bytes, original);
        state.redo_journal().unwrap();
        assert_eq!(state.materialize_current_rom().unwrap().bytes, edited);
    }

    #[test]
    fn malformed_edits_never_change_rom() {
        let state = AppState::new(manifest_core::Manifest::empty());
        let original = vec![0x2a, 30, 0x00];
        state.install_rom_session(rom_core::Rom::new(original.clone()), "synthetic.sfc".into());
        for (offset, operands) in [(1, vec![20]), (0, vec![20, 30]), (100, vec![])] {
            assert!(update_ai_script_operands_internal(
                &state,
                0,
                original.clone(),
                offset,
                operands
            )
            .is_err());
            assert_eq!(state.materialize_current_rom().unwrap().bytes, original);
        }
        assert!(validate_ai_range(usize::MAX, 2).is_err());
        assert!(validate_ai_range(0x7fff, 2).is_err());
        assert!(validate_ai_range(0, 0).is_err());
        assert!(validate_ai_range(0x8000, 0x8000).is_ok());
    }
}

fn validate_ai_range(offset: usize, length: usize) -> Result<(), String> {
    if length == 0 || length > 0x8000 || offset.checked_add(length).is_none() {
        return Err("AI stream must contain 1 to 32768 bytes".into());
    }
    if offset / 0x8000 != (offset + length - 1) / 0x8000 {
        return Err("AI stream cannot cross a LoROM bank boundary".into());
    }
    Ok(())
}

/// Research tooling: the caller supplies a known stream allocation, not a guessed
/// fighter table. Decoding proves instruction boundaries, not gameplay semantics.
#[tauri::command]
pub fn read_ai_script_stream(
    state: State<AppState>,
    pc_offset: usize,
    length: usize,
) -> Result<Vec<AiInstruction>, String> {
    validate_ai_range(pc_offset, length)?;
    let rom = state.materialize_current_rom()?;
    let bytes = rom
        .bytes
        .get(pc_offset..pc_offset + length)
        .ok_or("AI stream is outside ROM")?;
    decode_ai_stream(bytes)
}

#[tauri::command]
pub fn update_ai_script_operands(
    state: State<AppState>,
    pc_offset: usize,
    expected_bytes: Vec<u8>,
    instruction_offset: usize,
    operands: Vec<u8>,
) -> Result<Vec<AiInstruction>, String> {
    if !VERIFIED_USA_AI_REGIONS
        .iter()
        .any(|region| region.pc_offset == pc_offset && region.length == expected_bytes.len())
    {
        return Err("AI edit must target a cataloged complete stream".into());
    }
    update_ai_script_operands_for_base(
        &state,
        pc_offset,
        expected_bytes,
        instruction_offset,
        operands,
        Some(USA_SHA1),
    )
}

#[cfg(test)]
fn update_ai_script_operands_internal(
    state: &AppState,
    pc_offset: usize,
    expected_bytes: Vec<u8>,
    instruction_offset: usize,
    operands: Vec<u8>,
) -> Result<Vec<AiInstruction>, String> {
    update_ai_script_operands_for_base(
        state,
        pc_offset,
        expected_bytes,
        instruction_offset,
        operands,
        None,
    )
}

fn update_ai_script_operands_for_base(
    state: &AppState,
    pc_offset: usize,
    expected_bytes: Vec<u8>,
    instruction_offset: usize,
    operands: Vec<u8>,
    expected_base: Option<&str>,
) -> Result<Vec<AiInstruction>, String> {
    validate_ai_range(pc_offset, expected_bytes.len())?;
    let mut instructions = decode_ai_stream(&expected_bytes)?;
    let instruction = instructions
        .iter_mut()
        .find(|entry| entry.offset == instruction_offset)
        .ok_or("Offset is not an AI instruction boundary")?;
    if operands.len() != instruction.operands.len() {
        return Err("Operand edits cannot resize an AI instruction; relocation is required".into());
    }
    let old_target = instruction.script_target();
    instruction.operands = operands;
    let new_target = instruction.script_target();
    let replacement = encode_ai_stream(&instructions)?;
    state.commit_rom_transform_for_base("Edit AI script operands", expected_base, |rom| {
        if rom
            .read_bytes(pc_offset, expected_bytes.len())
            .map_err(|e| e.to_string())?
            != expected_bytes
        {
            return Err("AI script changed; reload before editing".into());
        }
        if new_target != old_target {
            if let Some(target) = new_target {
                validate_script_target(&rom.data, pc_offset, target, VERIFIED_USA_AI_REGIONS)?;
            }
        }
        rom.write_bytes(pc_offset, &replacement)
            .map_err(|e| e.to_string())
    })?;
    Ok(instructions)
}

/// Get all script records
#[tauri::command]
pub fn get_all_scripts(state: State<AppState>) -> Result<Vec<ScriptRecord>, String> {
    let rom_opt = state.rom.lock();
    let rom = rom_opt.as_ref().ok_or("No ROM loaded")?;
    let reader = ScriptReader::new(rom);
    Ok(reader.get_all_scripts())
}

/// Get scripts for a specific fighter
#[tauri::command]
pub fn get_scripts_for_fighter(
    state: State<AppState>,
    fighter_name: String,
) -> Result<Vec<ScriptRecord>, String> {
    let rom_opt = state.rom.lock();
    let rom = rom_opt.as_ref().ok_or("No ROM loaded")?;
    let reader = ScriptReader::new(rom);
    Ok(reader.get_scripts_for_fighter(&fighter_name))
}

/// Get fighter header
#[tauri::command]
pub fn get_fighter_header(
    state: State<AppState>,
    fighter_index: usize,
) -> Result<BoxerHeader, String> {
    if fighter_index >= script_core::MAX_FIGHTERS {
        return Err("Invalid fighter index".into());
    }
    let rom_opt = state.rom.lock();
    let rom = rom_opt.as_ref().ok_or("No ROM loaded")?;
    let reader = ScriptReader::new(rom);
    Ok(reader.decode_boxer_header(fighter_index))
}

/// Get editable fighter parameters
#[tauri::command]
pub fn get_editable_fighter_params(
    state: State<AppState>,
    fighter_index: usize,
) -> Result<EditableFighterParams, String> {
    if fighter_index >= script_core::MAX_FIGHTERS {
        return Err("Invalid fighter index".into());
    }
    let rom_opt = state.rom.lock();
    let rom = rom_opt.as_ref().ok_or("No ROM loaded")?;
    let reader = ScriptReader::new(rom);
    Ok(reader.get_editable_params(fighter_index))
}

/// Validate fighter parameters
#[tauri::command]
pub fn validate_fighter_params(
    params: EditableFighterParams,
) -> Result<ParamValidationResult, String> {
    Ok(params.validate_with_warnings())
}

/// Update fighter parameters
#[tauri::command]
pub fn update_fighter_params(
    state: State<AppState>,
    fighter_index: usize,
    params: EditableFighterParams,
) -> Result<EditableFighterParams, String> {
    update_fighter_params_internal(&state, fighter_index, params)
}

fn update_fighter_params_internal(
    state: &AppState,
    fighter_index: usize,
    params: EditableFighterParams,
) -> Result<EditableFighterParams, String> {
    // Validate params first
    params
        .validate()
        .map_err(|e| format!("Validation failed: {}", e))?;

    state.commit_rom_transform("Update fighter parameters", |rom| {
        let (header_bytes, pc_offset) =
            ScriptReader::new(rom).generate_header_with_params(fighter_index, &params)?;
        rom.write_bytes(pc_offset, &header_bytes)
            .map_err(|e| e.to_string())
    })?;

    Ok(params)
}
