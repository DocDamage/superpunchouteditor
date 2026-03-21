//! Frame Reconstructor Commands
//!
//! Commands for frame editing and reconstruction.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::app_state::AppState;
use asset_core::{render_frame_to_png, FrameData, FrameManager, FrameSummary};

/// Per-frame annotation data (project-side metadata, not stored in ROM)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FrameAnnotation {
    pub frame_index: u32,
    pub tags: Vec<String>,
    pub notes: String,
    pub hitbox_description: Option<String>,
}

/// Get frame annotations for a fighter.
///
/// Annotations are project-level metadata and are not read from ROM.
/// Returns an empty map until project save/load is wired up.
#[tauri::command]
pub fn get_fighter_annotations(
    _state: State<AppState>,
    _fighter_id: String,
) -> Option<HashMap<String, FrameAnnotation>> {
    // Annotations are not yet persisted — return empty map so the UI degrades gracefully.
    Some(HashMap::new())
}

/// Get all frames for a fighter
#[tauri::command]
pub fn get_fighter_frames(
    state: State<AppState>,
    fighter_id: usize,
) -> Result<Vec<FrameSummary>, String> {
    let rom_opt = state.rom.lock();
    let rom = rom_opt.as_ref().ok_or("No ROM loaded")?;

    let fm = asset_core::fighter::BoxerManager::new(rom);
    let fighters = fm.get_boxer_list();
    let f_meta = fighters.get(fighter_id).ok_or("Invalid fighter ID")?;

    let manifest = state.manifest.lock();
    let boxer = manifest
        .fighters
        .get(&f_meta.name)
        .ok_or(format!("Fighter {} not found in manifest", f_meta.name))?;

    let frame_manager = FrameManager::new(rom, boxer);
    let frames = frame_manager.load_frames()?;

    Ok(frames.iter().map(|f| f.summary()).collect())
}

/// Get detailed frame data
#[tauri::command]
pub fn get_frame_detail(
    state: State<AppState>,
    fighter_id: usize,
    frame_index: usize,
) -> Result<FrameData, String> {
    let rom_opt = state.rom.lock();
    let rom = rom_opt.as_ref().ok_or("No ROM loaded")?;

    let fm = asset_core::fighter::BoxerManager::new(rom);
    let fighters = fm.get_boxer_list();
    let f_meta = fighters.get(fighter_id).ok_or("Invalid fighter ID")?;

    let manifest = state.manifest.lock();
    let boxer = manifest
        .fighters
        .get(&f_meta.name)
        .ok_or(format!("Fighter {} not found in manifest", f_meta.name))?;

    let frame_manager = FrameManager::new(rom, boxer);
    let frames = frame_manager.load_frames()?;

    let frame = frames.get(frame_index).ok_or("Frame index out of range")?;

    Ok(FrameData::from(frame.clone()))
}

/// Render a frame to PNG for preview
#[tauri::command]
pub fn render_frame_preview(
    state: State<AppState>,
    fighter_id: usize,
    frame_index: usize,
) -> Result<Vec<u8>, String> {
    let rom_opt = state.rom.lock();
    let rom = rom_opt.as_ref().ok_or("No ROM loaded")?;

    let fm = asset_core::fighter::BoxerManager::new(rom);
    let fighters = fm.get_boxer_list();
    let f_meta = fighters.get(fighter_id).ok_or("Invalid fighter ID")?;

    let manifest = state.manifest.lock();
    let boxer = manifest
        .fighters
        .get(&f_meta.name)
        .ok_or(format!("Fighter {} not found in manifest", f_meta.name))?;

    let frame_manager = FrameManager::new(rom, boxer);
    let frames = frame_manager.load_frames()?;

    let frame = frames.get(frame_index).ok_or("Frame index out of range")?;

    // Get the pose info to load tiles and palette
    let poses = fm.get_poses(fighter_id);
    let pose = poses.get(frame_index).ok_or("Pose not found")?;

    let tiles = frame_manager.load_tiles_for_pose(pose)?;
    let palette = frame_manager.load_palette_for_pose(pose)?;

    render_frame_to_png(frame, &tiles, &palette)
}
