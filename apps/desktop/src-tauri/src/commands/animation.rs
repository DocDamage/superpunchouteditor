//! Animation and Hitbox Commands
//!
//! Commands for animation preview, frame interpolation, and hitbox editing.

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::app_state::AppState;

/// Animation frame data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationFrameData {
    pub frame_index: usize,
    pub duration: u8,
    pub hitboxes: Vec<HitboxData>,
    pub hurtboxes: Vec<HurtboxData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HitboxData {
    pub hitbox_type: String,
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
    pub damage: u8,
    pub hitstun: u8,
    pub knockback_angle: u16,
    pub knockback_power: u8,
    pub color: [u8; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HurtboxData {
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
    pub damage_taken_mult: u16,
    pub color: [u8; 4],
}

/// Get animation for a boxer
#[tauri::command]
pub fn get_boxer_animation(
    _state: State<AppState>,
    boxer_key: String,
    animation_name: String,
) -> Result<Vec<AnimationFrameData>, String> {
    // Placeholder - would load animation sequence
    Err("Animation system not yet fully integrated".into())
}

/// Animation player state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationPlayerState {
    pub current_frame: usize,
    pub frame_time: f32,
    pub is_playing: bool,
    pub total_frames: usize,
}

/// Start animation playback
#[tauri::command]
pub fn play_animation(
    _state: State<AppState>,
    boxer_key: String,
    animation_name: String,
) -> Result<AnimationPlayerState, String> {
    Err("Animation system not yet fully integrated".into())
}

/// Pause animation
#[tauri::command]
pub fn pause_animation(_state: State<AppState>) -> Result<(), String> {
    Ok(())
}

/// Stop animation
#[tauri::command]
pub fn stop_animation(_state: State<AppState>) -> Result<(), String> {
    Ok(())
}

/// Seek to specific frame
#[tauri::command]
pub fn seek_animation_frame(
    _state: State<AppState>,
    frame: usize,
) -> Result<AnimationPlayerState, String> {
    Err("Animation system not yet fully integrated".into())
}

/// Update animation (advance frame)
#[tauri::command]
pub fn update_animation(
    _state: State<AppState>,
    delta_time_ms: f32,
) -> Result<AnimationPlayerState, String> {
    Err("Animation system not yet fully integrated".into())
}

/// Get interpolated frame data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterpolatedFrameData {
    pub position_x: f32,
    pub position_y: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub rotation: f32,
    pub opacity: f32,
}

#[tauri::command]
pub fn get_interpolated_frame(
    _state: State<AppState>,
    boxer_key: String,
    animation_name: String,
    frame1: usize,
    frame2: usize,
    t: f32,
) -> Result<InterpolatedFrameData, String> {
    Err("Animation system not yet fully integrated".into())
}

/// Hitbox editor state
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

#[tauri::command]
pub fn get_hitbox_editor_state(_state: State<AppState>) -> Result<HitboxEditorState, String> {
    Ok(HitboxEditorState {
        selected_hitbox: None,
        selected_hurtbox: None,
        edit_mode: "select".into(),
        show_hitboxes: true,
        show_hurtboxes: true,
        snap_to_grid: false,
        grid_size: 8,
    })
}

/// Create a new hitbox
#[tauri::command]
pub fn create_hitbox(
    _state: State<AppState>,
    boxer_key: String,
    frame_index: usize,
    x: i16,
    y: i16,
    hitbox_type: String,
) -> Result<HitboxData, String> {
    // Placeholder
    Ok(HitboxData {
        hitbox_type,
        x,
        y,
        width: 32,
        height: 32,
        damage: 10,
        hitstun: 10,
        knockback_angle: 45,
        knockback_power: 5,
        color: [255, 0, 0, 128],
    })
}

/// Create a new hurtbox
#[tauri::command]
pub fn create_hurtbox(
    _state: State<AppState>,
    boxer_key: String,
    frame_index: usize,
    x: i16,
    y: i16,
) -> Result<HurtboxData, String> {
    // Placeholder
    Ok(HurtboxData {
        x,
        y,
        width: 48,
        height: 64,
        damage_taken_mult: 100,
        color: [0, 255, 0, 100],
    })
}

/// Update hitbox
#[tauri::command]
pub fn update_hitbox(
    _state: State<AppState>,
    boxer_key: String,
    frame_index: usize,
    hitbox_index: usize,
    hitbox: HitboxData,
) -> Result<(), String> {
    // Placeholder
    Ok(())
}

/// Delete hitbox
#[tauri::command]
pub fn delete_hitbox(
    _state: State<AppState>,
    boxer_key: String,
    frame_index: usize,
    hitbox_index: usize,
) -> Result<(), String> {
    // Placeholder
    Ok(())
}

/// Set hitbox editor option
#[tauri::command]
pub fn set_hitbox_editor_option(
    _state: State<AppState>,
    option: String,
    value: serde_json::Value,
) -> Result<(), String> {
    // Placeholder
    Ok(())
}
