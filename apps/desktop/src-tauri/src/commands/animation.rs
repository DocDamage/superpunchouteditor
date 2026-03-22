//! Animation and Hitbox Commands
//!
//! Tauri commands for animation preview, frame data, and hitbox editing.
//! Reads and writes animation data via the rom-core animation module.

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::app_state::AppState;
use rom_core::animation::{
    AnimationLoader, AnimationWriter, AnimationFrame, Animation, FighterAnimations,
    AnimationCategory, Hitbox, Hurtbox, FrameEffect, HitboxType,
    ANIM_TYPE_IDLE,
};

// ============================================================================
// FRONTEND DATA TYPES
// ============================================================================

/// Animation frame data for frontend display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationFrameData {
    pub frame_index: usize,
    pub duration: u8,
    pub pose_id: u8,
    pub tileset_id: u8,
    pub hitboxes: Vec<HitboxData>,
    pub hurtboxes: Vec<HurtboxData>,
}

/// Hitbox data for frontend display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HitboxData {
    pub hitbox_type: String,
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
    pub damage: u8,
    pub hitstun: u8,
    pub knockback_angle: u8,
    pub knockback_power: u8,
    pub color: [u8; 4],
}

/// Hurtbox data for frontend display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HurtboxData {
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
    pub damage_taken_mult: u16,
    pub color: [u8; 4],
}

/// Animation player state for frontend playback control
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationPlayerState {
    pub current_frame: usize,
    pub frame_time: f32,
    pub is_playing: bool,
    pub total_frames: usize,
}

impl AnimationPlayerState {
    fn stopped() -> Self {
        Self { current_frame: 0, frame_time: 0.0, is_playing: false, total_frames: 0 }
    }

    fn playing(total_frames: usize) -> Self {
        Self { current_frame: 0, frame_time: 0.0, is_playing: true, total_frames }
    }

    fn at_frame(frame: usize, total_frames: usize) -> Self {
        Self { current_frame: frame.min(total_frames.saturating_sub(1)), frame_time: 0.0, is_playing: false, total_frames }
    }
}

/// Hitbox editor state for frontend editor control
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HitboxEditorState {
    pub selected_hitbox: Option<usize>,
    pub selected_hurtbox: Option<usize>,
    pub edit_mode: String,
    pub show_hitboxes: bool,
    pub show_hurtboxes: bool,
    pub snap_to_grid: bool,
    pub grid_size: u16,
}

impl Default for HitboxEditorState {
    fn default() -> Self {
        Self {
            selected_hitbox: None,
            selected_hurtbox: None,
            edit_mode: "select".to_string(),
            show_hitboxes: true,
            show_hurtboxes: true,
            snap_to_grid: false,
            grid_size: 8,
        }
    }
}

/// Interpolated frame data for smooth animation preview
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterpolatedFrameData {
    pub position_x: f32,
    pub position_y: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub rotation: f32,
    pub opacity: f32,
}

// ============================================================================
// CONVERSION HELPERS
// ============================================================================

fn hitbox_to_frontend(h: &Hitbox) -> HitboxData {
    HitboxData {
        hitbox_type: h.hitbox_type.display_name().to_string(),
        x: h.x,
        y: h.y,
        width: h.width,
        height: h.height,
        damage: h.damage,
        hitstun: h.hitstun,
        knockback_angle: h.knockback_angle,
        knockback_power: h.knockback_power,
        color: h.color,
    }
}

fn hurtbox_to_frontend(h: &Hurtbox) -> HurtboxData {
    HurtboxData {
        x: h.x,
        y: h.y,
        width: h.width,
        height: h.height,
        damage_taken_mult: h.damage_multiplier,
        color: h.color,
    }
}

fn frontend_to_hitbox(h: &HitboxData) -> Hitbox {
    let hitbox_type = match h.hitbox_type.as_str() {
        "Attack" => HitboxType::Attack,
        "Counter" => HitboxType::Counter,
        "Grab" => HitboxType::Grab,
        "Projectile" => HitboxType::Projectile,
        _ => HitboxType::Attack,
    };
    Hitbox {
        hitbox_type,
        x: h.x,
        y: h.y,
        width: h.width,
        height: h.height,
        damage: h.damage,
        hitstun: h.hitstun,
        knockback_angle: h.knockback_angle,
        knockback_power: h.knockback_power,
        color: h.color,
        ..Default::default()
    }
}

fn frontend_to_hurtbox(h: &HurtboxData) -> Hurtbox {
    Hurtbox {
        x: h.x,
        y: h.y,
        width: h.width,
        height: h.height,
        damage_multiplier: h.damage_taken_mult,
        color: h.color,
    }
}

fn frame_to_frontend(frame_index: usize, f: &AnimationFrame) -> AnimationFrameData {
    AnimationFrameData {
        frame_index,
        duration: f.duration,
        pose_id: f.pose_id,
        tileset_id: f.tileset_id,
        hitboxes: f.hitboxes.iter().map(hitbox_to_frontend).collect(),
        hurtboxes: f.hurtboxes.iter().map(hurtbox_to_frontend).collect(),
    }
}

fn animation_to_frames(anim: &Animation) -> Vec<AnimationFrameData> {
    anim.frames
        .iter()
        .enumerate()
        .map(|(i, f)| frame_to_frontend(i, f))
        .collect()
}

fn frontend_to_frame(frame: &AnimationFrameData) -> AnimationFrame {
    let mut rom_frame = AnimationFrame::default();
    rom_frame.pose_id = frame.pose_id;
    rom_frame.duration = frame.duration;
    rom_frame.tileset_id = frame.tileset_id;
    rom_frame.hitboxes = frame.hitboxes.iter().map(frontend_to_hitbox).collect();
    rom_frame.hurtboxes = frame.hurtboxes.iter().map(frontend_to_hurtbox).collect();
    rom_frame
}

/// Parse a boxer key (fighter name or numeric ID) to a fighter ID
fn parse_boxer_key(boxer_key: &str) -> Result<u8, String> {
    if let Ok(id) = boxer_key.parse::<u8>() {
        if (id as usize) < 16 {
            return Ok(id);
        }
    }

    const NAMES: &[&str] = &[
        "Gabby Jay", "Bear Hugger", "Piston Hurricane", "Bald Bull",
        "Bob Charlie", "Dragon Chan", "Masked Muscle", "Mr. Sandman",
        "Aran Ryan", "Heike Kagero", "Mad Clown", "Super Macho Man",
        "Narcis Prince", "Hoy Quarlow", "Rick Bruiser", "Nick Bruiser",
    ];

    for (id, &name) in NAMES.iter().enumerate() {
        if boxer_key.eq_ignore_ascii_case(name) {
            return Ok(id as u8);
        }
    }

    Err(format!("Invalid boxer key: {}", boxer_key))
}

/// Internal helper: load an animation by boxer key + name
fn load_animation_internal(
    state: &AppState,
    boxer_key: &str,
    animation_name: &str,
) -> Result<Animation, String> {
    let rom_guard = state.rom.lock();
    let rom = rom_guard.as_ref().ok_or("No ROM loaded")?;
    let loader = AnimationLoader::new(rom);
    let fighter_id = parse_boxer_key(boxer_key)?;
    let animations = loader.get_animations(fighter_id).map_err(|e| e.to_string())?;
    animations
        .get_animation_by_name(animation_name)
        .ok_or_else(|| format!("Animation '{}' not found", animation_name))
}

/// Internal helper: load all animations for a boxer
fn load_fighter_animations_internal(
    state: &AppState,
    boxer_key: &str,
) -> Result<FighterAnimations, String> {
    let rom_guard = state.rom.lock();
    let rom = rom_guard.as_ref().ok_or("No ROM loaded")?;
    let loader = AnimationLoader::new(rom);
    let fighter_id = parse_boxer_key(boxer_key)?;
    loader.get_animations(fighter_id).map_err(|e| e.to_string())
}

/// Internal helper: mutate an animation and write back to ROM
fn mutate_animation<F>(
    state: &AppState,
    boxer_key: &str,
    animation_name: &str,
    mutate_fn: F,
) -> Result<(), String>
where
    F: FnOnce(&mut Animation) -> Result<(), String>,
{
    let mut rom_guard = state.rom.lock();
    let rom = rom_guard.as_mut().ok_or("No ROM loaded")?;

    let fighter_id = parse_boxer_key(boxer_key)?;

    // Load current animations
    let mut animations = {
        let loader = AnimationLoader::new(rom);
        loader.get_animations(fighter_id).map_err(|e| e.to_string())?
    };

    // Mutate the target animation
    let anim = animations
        .get_animation_by_name_mut(animation_name)
        .ok_or_else(|| format!("Animation '{}' not found", animation_name))?;

    mutate_fn(anim)?;

    // Write back
    let mut writer = AnimationWriter::new(rom);
    writer.update_animation(fighter_id, &animations).map_err(|e| e.to_string())?;

    // Mark ROM modified
    *state.modified.lock() = true;

    Ok(())
}

// ============================================================================
// READ COMMANDS
// ============================================================================

/// Get a specific animation for a boxer
#[tauri::command]
pub fn get_boxer_animation(
    state: State<AppState>,
    boxer_key: String,
    animation_name: String,
) -> Result<Animation, String> {
    load_animation_internal(&state, &boxer_key, &animation_name)
}

/// Get all animations for a boxer
#[tauri::command]
pub fn get_boxer_animations(
    state: State<AppState>,
    boxer_key: String,
) -> Result<FighterAnimations, String> {
    load_fighter_animations_internal(&state, &boxer_key)
}

/// Get animation frames for a boxer
#[tauri::command]
pub fn get_animation_frames(
    state: State<AppState>,
    boxer_key: String,
    animation_name: String,
) -> Result<Vec<AnimationFrameData>, String> {
    let anim = load_animation_internal(&state, &boxer_key, &animation_name)?;
    Ok(animation_to_frames(&anim))
}

/// Get a single frame from an animation
#[tauri::command]
pub fn get_animation_frame(
    state: State<AppState>,
    boxer_key: String,
    animation_name: String,
    frame_index: usize,
) -> Result<AnimationFrameData, String> {
    let anim = load_animation_internal(&state, &boxer_key, &animation_name)?;
    match anim.frames.get(frame_index) {
        Some(frame) => Ok(frame_to_frontend(frame_index, frame)),
        None => Err(format!("Frame {} out of range", frame_index)),
    }
}

// ============================================================================
// PLAYBACK COMMANDS (stateless — frontend manages the animation loop)
// ============================================================================

/// Start animation playback — returns initial player state with total_frames set
#[tauri::command]
pub fn play_animation(
    state: State<AppState>,
    boxer_key: String,
    animation_name: String,
) -> Result<AnimationPlayerState, String> {
    let anim = load_animation_internal(&state, &boxer_key, &animation_name)?;
    Ok(AnimationPlayerState::playing(anim.frames.len()))
}

/// Pause animation playback
#[tauri::command]
pub fn pause_animation(_state: State<AppState>) -> Result<(), String> {
    Ok(())
}

/// Stop animation playback and reset
#[tauri::command]
pub fn stop_animation(_state: State<AppState>) -> Result<(), String> {
    Ok(())
}

/// Seek to a specific frame
#[tauri::command]
pub fn seek_animation_frame(
    state: State<AppState>,
    boxer_key: String,
    animation_name: String,
    frame: usize,
) -> Result<AnimationPlayerState, String> {
    let anim = load_animation_internal(&state, &boxer_key, &animation_name)?;
    Ok(AnimationPlayerState::at_frame(frame, anim.frames.len()))
}

/// Advance animation by delta time — client-side timing; returns current state unchanged
#[tauri::command]
pub fn update_animation(
    _state: State<AppState>,
    _delta_time_ms: f32,
) -> Result<AnimationPlayerState, String> {
    Ok(AnimationPlayerState::stopped())
}

// ============================================================================
// INTERPOLATION
// ============================================================================

/// Get interpolated frame data for smooth animation preview
#[tauri::command]
pub fn get_interpolated_frame(
    state: State<AppState>,
    boxer_key: String,
    animation_name: String,
    frame1: usize,
    frame2: usize,
    t: f32,
) -> Result<InterpolatedFrameData, String> {
    let anim = load_animation_internal(&state, &boxer_key, &animation_name)?;

    let f1 = anim.frames.get(frame1).ok_or("frame1 out of range")?;
    let f2 = anim.frames.get(frame2).ok_or("frame2 out of range")?;

    let _ = (f1, f2, t); // acknowledged — no positional interpolation in SPO animation format

    Ok(InterpolatedFrameData {
        position_x: 0.0,
        position_y: 0.0,
        scale_x: 1.0,
        scale_y: 1.0,
        rotation: 0.0,
        opacity: 1.0,
    })
}

// ============================================================================
// FRAME MUTATION COMMANDS
// ============================================================================

/// Update a frame in an animation
#[tauri::command]
pub fn update_animation_frame(
    state: State<AppState>,
    boxer_key: String,
    animation_name: String,
    frame_index: usize,
    frame: AnimationFrameData,
) -> Result<(), String> {
    mutate_animation(&state, &boxer_key, &animation_name, |anim| {
        if frame_index >= anim.frames.len() {
            return Err(format!("Frame index {} out of range", frame_index));
        }
        let original_offset = anim.frames[frame_index].rom_offset;
        let mut rom_frame = frontend_to_frame(&frame);
        rom_frame.rom_offset = original_offset;
        anim.frames[frame_index] = rom_frame;
        Ok(())
    })
}

/// Add a new frame to an animation
#[tauri::command]
pub fn add_animation_frame(
    state: State<AppState>,
    boxer_key: String,
    animation_name: String,
    frame: AnimationFrameData,
) -> Result<usize, String> {
    let mut new_index = 0;
    mutate_animation(&state, &boxer_key, &animation_name, |anim| {
        new_index = anim.frames.len();
        anim.frames.push(frontend_to_frame(&frame));
        Ok(())
    })?;
    Ok(new_index)
}

/// Insert a frame at a specific index
#[tauri::command]
pub fn insert_animation_frame(
    state: State<AppState>,
    boxer_key: String,
    animation_name: String,
    frame_index: usize,
    frame: AnimationFrameData,
) -> Result<(), String> {
    mutate_animation(&state, &boxer_key, &animation_name, |anim| {
        if frame_index > anim.frames.len() {
            return Err(format!("Frame index {} out of range", frame_index));
        }
        anim.frames.insert(frame_index, frontend_to_frame(&frame));
        Ok(())
    })
}

/// Remove a frame from an animation
#[tauri::command]
pub fn remove_animation_frame(
    state: State<AppState>,
    boxer_key: String,
    animation_name: String,
    frame_index: usize,
) -> Result<(), String> {
    mutate_animation(&state, &boxer_key, &animation_name, |anim| {
        if frame_index >= anim.frames.len() {
            return Err(format!("Frame index {} out of range", frame_index));
        }
        anim.frames.remove(frame_index);
        Ok(())
    })
}

/// Move a frame from one index to another
#[tauri::command]
pub fn move_animation_frame(
    state: State<AppState>,
    boxer_key: String,
    animation_name: String,
    from_index: usize,
    to_index: usize,
) -> Result<(), String> {
    mutate_animation(&state, &boxer_key, &animation_name, |anim| {
        if from_index >= anim.frames.len() || to_index > anim.frames.len() {
            return Err("Invalid frame indices".to_string());
        }
        let frame = anim.frames.remove(from_index);
        let insert_idx = if to_index > from_index { to_index - 1 } else { to_index };
        anim.frames.insert(insert_idx, frame);
        Ok(())
    })
}

// ============================================================================
// HITBOX EDITOR COMMANDS
// ============================================================================

/// Get hitbox editor state (returns default; persistent state is managed by frontend)
#[tauri::command]
pub fn get_hitbox_editor_state(_state: State<AppState>) -> Result<HitboxEditorState, String> {
    Ok(HitboxEditorState::default())
}

/// Set a hitbox editor option (acknowledged; state is managed by frontend)
#[tauri::command]
pub fn set_hitbox_editor_option(
    _state: State<AppState>,
    _option: String,
    _value: serde_json::Value,
) -> Result<(), String> {
    Ok(())
}

/// Create a new hitbox for a frame
#[tauri::command]
pub fn create_hitbox(
    state: State<AppState>,
    boxer_key: String,
    animation_name: String,
    frame_index: usize,
    hitbox: HitboxData,
) -> Result<HitboxData, String> {
    mutate_animation(&state, &boxer_key, &animation_name, |anim| {
        if frame_index >= anim.frames.len() {
            return Err(format!("Frame index {} out of range", frame_index));
        }
        anim.frames[frame_index].hitboxes.push(frontend_to_hitbox(&hitbox));
        Ok(())
    })?;
    Ok(hitbox)
}

/// Create a new hurtbox for a frame
#[tauri::command]
pub fn create_hurtbox(
    state: State<AppState>,
    boxer_key: String,
    animation_name: String,
    frame_index: usize,
    hurtbox: HurtboxData,
) -> Result<HurtboxData, String> {
    mutate_animation(&state, &boxer_key, &animation_name, |anim| {
        if frame_index >= anim.frames.len() {
            return Err(format!("Frame index {} out of range", frame_index));
        }
        anim.frames[frame_index].hurtboxes.push(frontend_to_hurtbox(&hurtbox));
        Ok(())
    })?;
    Ok(hurtbox)
}

/// Update a hitbox in a frame
#[tauri::command]
pub fn update_hitbox(
    state: State<AppState>,
    boxer_key: String,
    animation_name: String,
    frame_index: usize,
    hitbox_index: usize,
    hitbox: HitboxData,
) -> Result<(), String> {
    mutate_animation(&state, &boxer_key, &animation_name, |anim| {
        let frame = anim.frames.get_mut(frame_index)
            .ok_or_else(|| format!("Frame index {} out of range", frame_index))?;
        if hitbox_index >= frame.hitboxes.len() {
            return Err(format!("Hitbox index {} out of range", hitbox_index));
        }
        frame.hitboxes[hitbox_index] = frontend_to_hitbox(&hitbox);
        Ok(())
    })
}

/// Delete a hitbox from a frame
#[tauri::command]
pub fn delete_hitbox(
    state: State<AppState>,
    boxer_key: String,
    animation_name: String,
    frame_index: usize,
    hitbox_index: usize,
) -> Result<(), String> {
    mutate_animation(&state, &boxer_key, &animation_name, |anim| {
        let frame = anim.frames.get_mut(frame_index)
            .ok_or_else(|| format!("Frame index {} out of range", frame_index))?;
        if hitbox_index >= frame.hitboxes.len() {
            return Err(format!("Hitbox index {} out of range", hitbox_index));
        }
        frame.hitboxes.remove(hitbox_index);
        Ok(())
    })
}

/// Update a hurtbox in a frame
#[tauri::command]
pub fn update_hurtbox(
    state: State<AppState>,
    boxer_key: String,
    animation_name: String,
    frame_index: usize,
    hurtbox_index: usize,
    hurtbox: HurtboxData,
) -> Result<(), String> {
    mutate_animation(&state, &boxer_key, &animation_name, |anim| {
        let frame = anim.frames.get_mut(frame_index)
            .ok_or_else(|| format!("Frame index {} out of range", frame_index))?;
        if hurtbox_index >= frame.hurtboxes.len() {
            return Err(format!("Hurtbox index {} out of range", hurtbox_index));
        }
        frame.hurtboxes[hurtbox_index] = frontend_to_hurtbox(&hurtbox);
        Ok(())
    })
}

/// Delete a hurtbox from a frame
#[tauri::command]
pub fn delete_hurtbox(
    state: State<AppState>,
    boxer_key: String,
    animation_name: String,
    frame_index: usize,
    hurtbox_index: usize,
) -> Result<(), String> {
    mutate_animation(&state, &boxer_key, &animation_name, |anim| {
        let frame = anim.frames.get_mut(frame_index)
            .ok_or_else(|| format!("Frame index {} out of range", frame_index))?;
        if hurtbox_index >= frame.hurtboxes.len() {
            return Err(format!("Hurtbox index {} out of range", hurtbox_index));
        }
        frame.hurtboxes.remove(hurtbox_index);
        Ok(())
    })
}
