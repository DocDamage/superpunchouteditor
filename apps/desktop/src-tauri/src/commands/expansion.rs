//! In-ROM expansion commands.
//!
//! These commands are the first backend surface for true in-game expansion:
//! - Expand roster tables beyond vanilla 16 entries.
//! - Install ROM-side editor bootstrap metadata/stub.
//! - Optionally patch a hook with a JML redirection.

use expansion_core::{
    analyze_ingame_hook_sites, apply_ingame_editor_expansion, verify_ingame_hook_site,
    ExpansionOptions, HookSiteCandidate,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tauri::State;

use crate::app_state::AppState;
use crate::utils::{format_hex, parse_offset};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyInGameExpansionRequest {
    pub target_boxer_count: usize,
    pub patch_editor_hook: bool,
    /// Optional hook offset string ("0x..." or decimal)
    pub editor_hook_pc_offset: Option<String>,
    /// Optional exact instruction-aligned overwrite length for hook patching.
    pub editor_hook_overwrite_len: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyInGameExpansionResponse {
    pub boxer_count: usize,
    pub header_pc: String,
    pub editor_stub_pc: String,
    pub editor_hook_patched: bool,
    pub editor_hook_overwrite_len: usize,
    pub name_pointer_table_pc: String,
    pub name_long_pointer_table_pc: String,
    pub name_blob_pc: String,
    pub circuit_table_pc: String,
    pub unlock_table_pc: String,
    pub intro_table_pc: String,
    pub write_ranges: Vec<ExpansionWriteRange>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpansionWriteRange {
    pub start_pc: String,
    pub size: usize,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzeInGameHookSitesRequest {
    /// Optional start PC offset ("0x..." or decimal)
    pub start_pc_offset: Option<String>,
    /// Optional end PC offset ("0x..." or decimal)
    pub end_pc_offset: Option<String>,
    /// Maximum number of candidates to return (defaults to 25)
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookSiteCandidateResponse {
    pub hook_pc: String,
    pub overwrite_len: usize,
    pub return_pc: String,
    pub first_instruction: String,
    pub preview_bytes_hex: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyInGameHookSiteRequest {
    /// Hook PC offset to validate ("0x..." or decimal)
    pub hook_pc_offset: String,
    /// Optional exact instruction-aligned overwrite length
    pub overwrite_len: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InGameHookPresetResponse {
    pub id: String,
    pub label: String,
    pub description: String,
    pub region: String,
    pub source: String,
    pub verified: bool,
    pub hook_pc: String,
    pub overwrite_len: usize,
    pub return_pc: String,
    pub first_instruction: String,
    pub preview_bytes_hex: String,
}

#[derive(Debug, Clone)]
struct HookPresetSeed {
    id: &'static str,
    label: &'static str,
    description: &'static str,
    hook_pc: usize,
    overwrite_len: Option<usize>,
}

#[derive(Debug, Clone)]
struct ResolvedHookOptions {
    patch_editor_hook: bool,
    hook_pc: Option<usize>,
    overwrite_len: Option<usize>,
    notes: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
struct ExistingCreatorHook {
    hook_pc: usize,
    stub_pc: usize,
}

#[tauri::command]
pub fn apply_in_game_expansion(
    state: State<AppState>,
    request: ApplyInGameExpansionRequest,
) -> Result<ApplyInGameExpansionResponse, String> {
    let mut rom_guard = state.rom.lock();
    let rom = rom_guard.as_mut().ok_or("No ROM loaded")?;
    let resolved_hook = resolve_hook_options(rom, &request)?;

    let mut report = apply_ingame_editor_expansion(
        rom,
        &ExpansionOptions {
            target_boxer_count: request.target_boxer_count,
            patch_editor_hook: resolved_hook.patch_editor_hook,
            editor_hook_pc_offset: resolved_hook.hook_pc,
            editor_hook_overwrite_len: resolved_hook.overwrite_len,
        },
    )
    .map_err(|err| err.to_string())?;
    report.notes.extend(resolved_hook.notes);

    *state.modified.lock() = true;

    Ok(ApplyInGameExpansionResponse {
        boxer_count: report.layout.boxer_count,
        header_pc: format_hex(report.header_pc),
        editor_stub_pc: format_hex(report.editor_stub_pc),
        editor_hook_patched: report.editor_hook_patched,
        editor_hook_overwrite_len: report.editor_hook_overwrite_len,
        name_pointer_table_pc: format_hex(report.layout.name_pointer_table_pc),
        name_long_pointer_table_pc: format_hex(report.layout.name_long_pointer_table_pc),
        name_blob_pc: format_hex(report.layout.name_blob_pc),
        circuit_table_pc: format_hex(report.layout.circuit_table_pc),
        unlock_table_pc: format_hex(report.layout.unlock_table_pc),
        intro_table_pc: format_hex(report.layout.intro_table_pc),
        write_ranges: report
            .write_ranges
            .into_iter()
            .map(|range| ExpansionWriteRange {
                start_pc: format_hex(range.start_pc),
                size: range.size,
                description: range.description,
            })
            .collect(),
        notes: report.notes,
    })
}

fn resolve_hook_options(
    rom: &rom_core::Rom,
    request: &ApplyInGameExpansionRequest,
) -> Result<ResolvedHookOptions, String> {
    if !request.patch_editor_hook {
        return Ok(ResolvedHookOptions {
            patch_editor_hook: false,
            hook_pc: None,
            overwrite_len: None,
            notes: Vec::new(),
        });
    }

    let manual_hook_pc = match request.editor_hook_pc_offset.as_deref() {
        Some(offset) if !offset.trim().is_empty() => Some(parse_offset(offset)?),
        _ => None,
    };

    if let Some(hook_pc) = manual_hook_pc {
        if let Some(existing) = detect_existing_creator_hook_at(rom, hook_pc) {
            return Ok(ResolvedHookOptions {
                patch_editor_hook: false,
                hook_pc: None,
                overwrite_len: None,
                notes: vec![format!(
                    "Creator hook already installed at {} (stub {}). Skipping hook re-patch.",
                    format_hex(existing.hook_pc),
                    format_hex(existing.stub_pc)
                )],
            });
        }

        return Ok(ResolvedHookOptions {
            patch_editor_hook: true,
            hook_pc: Some(hook_pc),
            overwrite_len: request.editor_hook_overwrite_len,
            notes: vec![format!(
                "Using manually provided hook PC {}.",
                format_hex(hook_pc)
            )],
        });
    }

    let region = rom.detect_region().unwrap_or(rom_core::RomRegion::Usa);
    let region_name = region.code().to_string();
    let requested_len = request.editor_hook_overwrite_len;

    for seed in curated_hook_preset_seeds(region) {
        let verify_len = requested_len.or(seed.overwrite_len);
        if let Ok(candidate) = verify_ingame_hook_site(rom, seed.hook_pc, verify_len) {
            return Ok(ResolvedHookOptions {
                patch_editor_hook: true,
                hook_pc: Some(candidate.hook_pc),
                overwrite_len: Some(candidate.overwrite_len),
                notes: vec![format!(
                    "Auto-selected {} hook preset '{}' at {} ({} bytes).",
                    region_name,
                    seed.label,
                    format_hex(candidate.hook_pc),
                    candidate.overwrite_len
                )],
            });
        }
    }

    for (start_pc, end_pc) in default_hook_scan_ranges(rom) {
        for scanned in analyze_ingame_hook_sites(rom, start_pc, end_pc, 64) {
            let verify_len = requested_len.or(Some(scanned.overwrite_len));
            if let Ok(candidate) = verify_ingame_hook_site(rom, scanned.hook_pc, verify_len) {
                return Ok(ResolvedHookOptions {
                    patch_editor_hook: true,
                    hook_pc: Some(candidate.hook_pc),
                    overwrite_len: Some(candidate.overwrite_len),
                    notes: vec![format!(
                        "Auto-selected scanned {} hook candidate at {} ({} bytes).",
                        region_name,
                        format_hex(candidate.hook_pc),
                        candidate.overwrite_len
                    )],
                });
            }
        }
    }

    if let Some(existing) = find_existing_creator_hook(rom, region) {
        return Ok(ResolvedHookOptions {
            patch_editor_hook: false,
            hook_pc: None,
            overwrite_len: None,
            notes: vec![format!(
                "Existing creator hook detected at {} (stub {}). Skipping hook re-patch.",
                format_hex(existing.hook_pc),
                format_hex(existing.stub_pc)
            )],
        });
    }

    Err(format!(
        "Failed to auto-resolve a safe hook location for {} ROM. Use advanced hook controls to provide/verify a manual hook.",
        region_name
    ))
}

fn find_existing_creator_hook(
    rom: &rom_core::Rom,
    region: rom_core::RomRegion,
) -> Option<ExistingCreatorHook> {
    let mut seen = HashSet::new();

    for seed in curated_hook_preset_seeds(region) {
        if !seen.insert(seed.hook_pc) {
            continue;
        }
        if let Some(existing) = detect_existing_creator_hook_at(rom, seed.hook_pc) {
            return Some(existing);
        }
    }

    for (start_pc, end_pc) in default_hook_scan_ranges(rom) {
        let range = rom.read_bytes(start_pc, end_pc - start_pc).ok()?;
        for (idx, byte) in range.iter().enumerate() {
            if *byte != 0x5C {
                continue;
            }
            let hook_pc = start_pc + idx;
            if !seen.insert(hook_pc) {
                continue;
            }
            if let Some(existing) = detect_existing_creator_hook_at(rom, hook_pc) {
                return Some(existing);
            }
        }
    }

    None
}

fn detect_existing_creator_hook_at(rom: &rom_core::Rom, hook_pc: usize) -> Option<ExistingCreatorHook> {
    let hook_bytes = rom.read_bytes(hook_pc, 4).ok()?;
    if hook_bytes[0] != 0x5C {
        return None;
    }

    let stub_pc = jml_target_to_pc(rom, hook_bytes[1], hook_bytes[2], hook_bytes[3])?;
    let stub_head = rom.read_bytes(stub_pc, 3).ok()?;
    if stub_head != [0x08, 0xE2, 0x20] {
        return None;
    }

    let scan_len = rom.size().saturating_sub(stub_pc).min(512);
    if scan_len < 6 {
        return None;
    }
    let stub_bytes = rom.read_bytes(stub_pc, scan_len).ok()?;
    if !stub_bytes.windows(6).any(|window| window == b"INGAME") {
        return None;
    }

    Some(ExistingCreatorHook { hook_pc, stub_pc })
}

fn jml_target_to_pc(rom: &rom_core::Rom, lo: u8, hi: u8, bank: u8) -> Option<usize> {
    if bank < 0x80 {
        return None;
    }
    let addr = u16::from_le_bytes([lo, hi]);
    if addr < 0x8000 {
        return None;
    }
    let pc = rom.snes_to_pc(bank, addr);
    (pc < rom.size()).then_some(pc)
}

#[tauri::command]
pub fn analyze_in_game_hook_sites(
    state: State<AppState>,
    request: Option<AnalyzeInGameHookSitesRequest>,
) -> Result<Vec<HookSiteCandidateResponse>, String> {
    let rom_guard = state.rom.lock();
    let rom = rom_guard.as_ref().ok_or("No ROM loaded")?;

    let req = request.unwrap_or(AnalyzeInGameHookSitesRequest {
        start_pc_offset: None,
        end_pc_offset: None,
        limit: None,
    });

    let limit = req.limit.unwrap_or(25).clamp(1, 200);

    let ranges = if req.start_pc_offset.as_deref().is_some_and(|value| !value.trim().is_empty())
        || req.end_pc_offset.as_deref().is_some_and(|value| !value.trim().is_empty())
    {
        let start_pc = match req.start_pc_offset.as_deref() {
            Some(value) if !value.trim().is_empty() => parse_offset(value)?,
            _ => 0usize,
        };
        let end_pc = match req.end_pc_offset.as_deref() {
            Some(value) if !value.trim().is_empty() => parse_offset(value)?,
            _ => rom.size().min(0x20_0000),
        };
        vec![(start_pc, end_pc)]
    } else {
        default_hook_scan_ranges(rom)
    };

    let mut seen = HashSet::new();
    let mut collected = Vec::new();

    for (start_pc, end_pc) in ranges {
        if collected.len() >= limit {
            break;
        }

        let remaining = limit - collected.len();
        for candidate in analyze_ingame_hook_sites(rom, start_pc, end_pc, remaining) {
            if seen.insert(candidate.hook_pc) {
                collected.push(candidate);
            }
            if collected.len() >= limit {
                break;
            }
        }
    }

    Ok(collected.into_iter().map(map_candidate_response).collect())
}

#[tauri::command]
pub fn verify_in_game_hook_site(
    state: State<AppState>,
    request: VerifyInGameHookSiteRequest,
) -> Result<HookSiteCandidateResponse, String> {
    let hook_pc = parse_offset(&request.hook_pc_offset)?;
    let rom_guard = state.rom.lock();
    let rom = rom_guard.as_ref().ok_or("No ROM loaded")?;

    let candidate = verify_ingame_hook_site(rom, hook_pc, request.overwrite_len)
        .map_err(|err| err.to_string())?;
    Ok(map_candidate_response(candidate))
}

#[tauri::command]
pub fn get_in_game_hook_presets(
    state: State<AppState>,
    limit: Option<usize>,
) -> Result<Vec<InGameHookPresetResponse>, String> {
    let rom_guard = state.rom.lock();
    let rom = rom_guard.as_ref().ok_or("No ROM loaded")?;

    let region = rom.detect_region().unwrap_or(rom_core::RomRegion::Usa);
    let region_name = region.code().to_string();
    let limit = limit.unwrap_or(8).clamp(1, 32);
    let mut seen = HashSet::new();
    let mut presets = Vec::new();

    for seed in curated_hook_preset_seeds(region) {
        if presets.len() >= limit {
            break;
        }
        if seen.contains(&seed.hook_pc) {
            continue;
        }

        if let Ok(candidate) = verify_ingame_hook_site(rom, seed.hook_pc, seed.overwrite_len) {
            seen.insert(seed.hook_pc);
            presets.push(InGameHookPresetResponse {
                id: seed.id.to_string(),
                label: seed.label.to_string(),
                description: seed.description.to_string(),
                region: region_name.clone(),
                source: "curated".to_string(),
                verified: true,
                hook_pc: format_hex(candidate.hook_pc),
                overwrite_len: candidate.overwrite_len,
                return_pc: format_hex(candidate.return_pc),
                first_instruction: candidate.first_instruction,
                preview_bytes_hex: candidate
                    .preview_bytes
                    .iter()
                    .map(|b| format!("{:02X}", b))
                    .collect::<Vec<_>>()
                    .join(" "),
            });
        }
    }

    if presets.len() < limit {
        let remaining = limit - presets.len();
        let mut scanned_idx = 1usize;
        for (start_pc, end_pc) in default_hook_scan_ranges(rom) {
            if presets.len() >= limit {
                break;
            }
            for candidate in analyze_ingame_hook_sites(rom, start_pc, end_pc, remaining) {
                if presets.len() >= limit {
                    break;
                }
                if !seen.insert(candidate.hook_pc) {
                    continue;
                }
                presets.push(InGameHookPresetResponse {
                    id: format!("auto_{}_{}", region_name.to_lowercase(), scanned_idx),
                    label: format!("Auto Candidate {}", scanned_idx),
                    description: format!(
                        "Discovered by scan in default {} region range.",
                        region_name
                    ),
                    region: region_name.clone(),
                    source: "scanned".to_string(),
                    verified: true,
                    hook_pc: format_hex(candidate.hook_pc),
                    overwrite_len: candidate.overwrite_len,
                    return_pc: format_hex(candidate.return_pc),
                    first_instruction: candidate.first_instruction,
                    preview_bytes_hex: candidate
                        .preview_bytes
                        .iter()
                        .map(|b| format!("{:02X}", b))
                        .collect::<Vec<_>>()
                        .join(" "),
                });
                scanned_idx += 1;
            }
        }
    }

    Ok(presets)
}

fn map_candidate_response(candidate: HookSiteCandidate) -> HookSiteCandidateResponse {
    HookSiteCandidateResponse {
        hook_pc: format_hex(candidate.hook_pc),
        overwrite_len: candidate.overwrite_len,
        return_pc: format_hex(candidate.return_pc),
        first_instruction: candidate.first_instruction,
        preview_bytes_hex: candidate
            .preview_bytes
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(" "),
    }
}

fn default_hook_scan_ranges(rom: &rom_core::Rom) -> Vec<(usize, usize)> {
    let rom_size = rom.size().min(0x20_0000);
    let ranges = match rom.detect_region() {
        // Region-specific windows tuned for executable banks and existing SPO patching workflows.
        Some(rom_core::RomRegion::Usa) => vec![
            (0x008000, 0x030000),
            (0x040000, 0x070000),
        ],
        Some(rom_core::RomRegion::Jpn) => vec![
            (0x008000, 0x030000),
            (0x03F800, 0x06F800),
        ],
        Some(rom_core::RomRegion::Pal) => vec![
            (0x008000, 0x030000),
            (0x03FFC0, 0x06FFC0),
        ],
        None => vec![(0x008000, rom_size)],
    };

    ranges
        .into_iter()
        .filter_map(|(start, end)| {
            let bounded_start = start.min(rom_size);
            let bounded_end = end.min(rom_size);
            (bounded_start < bounded_end).then_some((bounded_start, bounded_end))
        })
        .collect()
}

fn curated_hook_preset_seeds(region: rom_core::RomRegion) -> Vec<HookPresetSeed> {
    match region {
        rom_core::RomRegion::Usa => vec![
            HookPresetSeed {
                id: "usa_mainloop_a",
                label: "USA Main Loop A",
                description: "Primary executable bank scan anchor (verified at runtime).",
                hook_pc: 0x008000,
                overwrite_len: None,
            },
            HookPresetSeed {
                id: "usa_mainloop_b",
                label: "USA Main Loop B",
                description: "Secondary executable anchor for editor dispatch.",
                hook_pc: 0x040000,
                overwrite_len: None,
            },
        ],
        rom_core::RomRegion::Jpn => vec![
            HookPresetSeed {
                id: "jpn_mainloop_a",
                label: "JPN Main Loop A",
                description: "Japanese build anchor candidate.",
                hook_pc: 0x008000,
                overwrite_len: None,
            },
            HookPresetSeed {
                id: "jpn_mainloop_b",
                label: "JPN Main Loop B",
                description: "Secondary Japanese executable anchor.",
                hook_pc: 0x03F800,
                overwrite_len: None,
            },
        ],
        rom_core::RomRegion::Pal => vec![
            HookPresetSeed {
                id: "pal_mainloop_a",
                label: "PAL Main Loop A",
                description: "PAL build anchor candidate.",
                hook_pc: 0x008000,
                overwrite_len: None,
            },
            HookPresetSeed {
                id: "pal_mainloop_b",
                label: "PAL Main Loop B",
                description: "Secondary PAL executable anchor.",
                hook_pc: 0x03FFC0,
                overwrite_len: None,
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jml_target_to_pc_decodes_lorom_address() {
        let rom = rom_core::Rom::new(vec![0; 0x20_0000]);
        let target_pc = 0x012345usize;
        let (bank, addr) = rom.pc_to_snes(target_pc);
        let [lo, hi] = addr.to_le_bytes();

        let resolved = jml_target_to_pc(&rom, lo, hi, bank);
        assert_eq!(resolved, Some(target_pc));
    }

    #[test]
    fn detect_existing_creator_hook_at_finds_ingame_signature() {
        let mut rom = rom_core::Rom::new(vec![0; 0x20_0000]);
        let hook_pc = 0x009000usize;
        let stub_pc = 0x018000usize;

        rom.write_bytes(stub_pc, &[0x08, 0xE2, 0x20, 0xEA, 0xEA, b'I', b'N', b'G', b'A', b'M', b'E'])
            .expect("write stub bytes");
        let (bank, addr) = rom.pc_to_snes(stub_pc);
        let [lo, hi] = addr.to_le_bytes();
        rom.write_bytes(hook_pc, &[0x5C, lo, hi, bank])
            .expect("write hook bytes");

        let existing = detect_existing_creator_hook_at(&rom, hook_pc);
        assert!(existing.is_some());
        let existing = existing.expect("hook should be detected");
        assert_eq!(existing.hook_pc, hook_pc);
        assert_eq!(existing.stub_pc, stub_pc);
    }
}
