//! Animation support for frame reconstruction and preview

use crate::frame::Frame;
use serde::{Deserialize, Serialize};

/// Animation frame with timing information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationFrame {
    /// Frame data
    pub frame: Frame,
    /// Duration in frames (at 60 FPS)
    pub duration: u8,
    /// Frame index in the animation sequence
    pub sequence_index: usize,
    /// Combat hitboxes for this frame
    pub hitboxes: Vec<CombatHitbox>,
    /// Hurtboxes for this frame (vulnerable areas)
    pub hurtboxes: Vec<Hurtbox>,
    /// Frame-specific metadata
    pub metadata: FrameMetadata,
}

/// Types of hitboxes for boxing moves
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HitboxType {
    /// Left jab
    LeftJab,
    /// Right jab
    RightJab,
    /// Left hook
    LeftHook,
    /// Right hook
    RightHook,
    /// Left uppercut
    LeftUppercut,
    /// Right uppercut
    RightUppercut,
    /// Special move hitbox
    Special,
}

/// A hitbox represents an active attacking area (combat hitbox with fighting game data)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatHitbox {
    /// Type of hitbox
    pub hitbox_type: HitboxType,
    /// X position (relative to frame center)
    pub x: i16,
    /// Y position (relative to frame center)
    pub y: i16,
    /// Width of hitbox
    pub width: u16,
    /// Height of hitbox
    pub height: u16,
    /// Damage this hitbox deals
    pub damage: u8,
    /// Frame advantage on hit
    pub hitstun: u8,
    /// Knockback direction (0-360 degrees)
    pub knockback_angle: u16,
    /// Knockback power
    pub knockback_power: u8,
    /// Color for visualization (RGBA)
    pub color: [u8; 4],
}

impl Default for CombatHitbox {
    fn default() -> Self {
        Self {
            hitbox_type: HitboxType::LeftJab,
            x: 0,
            y: 0,
            width: 32,
            height: 32,
            damage: 10,
            hitstun: 10,
            knockback_angle: 45,
            knockback_power: 5,
            color: [255, 0, 0, 128], // Semi-transparent red
        }
    }
}

/// A hurtbox represents a vulnerable area that can be hit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hurtbox {
    /// X position (relative to frame center)
    pub x: i16,
    /// Y position (relative to frame center)
    pub y: i16,
    /// Width of hurtbox
    pub width: u16,
    /// Height of hurtbox
    pub height: u16,
    /// Damage multiplier when hit (100 = normal)
    pub damage_taken_mult: u16,
    /// Color for visualization (RGBA)
    pub color: [u8; 4],
}

impl Default for Hurtbox {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            width: 48,
            height: 64,
            damage_taken_mult: 100,
            color: [0, 255, 0, 100], // Semi-transparent green
        }
    }
}

/// Metadata for a frame
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FrameMetadata {
    /// Frame triggers (sound effects, etc.)
    pub triggers: Vec<FrameTrigger>,
    /// Frame state flags
    pub flags: FrameFlags,
    /// Blend mode for interpolation
    pub blend_mode: BlendMode,
}

/// Triggers that can occur on a specific frame
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FrameTrigger {
    SoundEffect { id: u8 },
    ScreenShake { intensity: u8 },
    FlashEffect { color: [u8; 3], duration: u8 },
    ParticleEffect { effect_type: String, x: i16, y: i16 },
}

/// Frame state flags
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FrameFlags {
    /// Frame is invincible
    pub invincible: bool,
    /// Frame has armor (reduced damage)
    pub armored: bool,
    /// Frame is a counter
    pub counter: bool,
    /// Frame can be cancelled into other moves
    pub cancellable: bool,
}

/// Blend mode for frame interpolation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlendMode {
    /// No interpolation
    None,
    /// Linear interpolation
    Linear,
    /// Ease in
    EaseIn,
    /// Ease out
    EaseOut,
    /// Ease in-out
    EaseInOut,
}

impl Default for BlendMode {
    fn default() -> Self {
        BlendMode::Linear
    }
}

/// Complete animation sequence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationSequence {
    /// Animation name
    pub name: String,
    /// Frames in the sequence
    pub frames: Vec<AnimationFrame>,
    /// Total duration in frames
    pub total_duration: usize,
    /// Whether the animation loops
    pub loops: bool,
    /// Loop start frame index
    pub loop_start: usize,
    /// Next animation to transition to (if any)
    pub next_animation: Option<String>,
}

impl AnimationSequence {
    /// Create a new animation sequence
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            frames: Vec::new(),
            total_duration: 0,
            loops: false,
            loop_start: 0,
            next_animation: None,
        }
    }
    
    /// Add a frame to the sequence
    pub fn add_frame(&mut self, frame: AnimationFrame) {
        self.total_duration += frame.duration as usize;
        self.frames.push(frame);
    }
    
    /// Get the frame at a specific time (in frames)
    pub fn frame_at(&self, time: usize) -> Option<&AnimationFrame> {
        let mut current_time = 0;
        
        for frame in &self.frames {
            current_time += frame.duration as usize;
            if time < current_time {
                return Some(frame);
            }
        }
        
        // Handle looping
        if self.loops && !self.frames.is_empty() {
            let loop_duration = self.total_duration - self.loop_start;
            let loop_time = (time - self.total_duration) % loop_duration;
            let adjusted_time = self.loop_start + loop_time;
            return self.frame_at(adjusted_time);
        }
        
        self.frames.last()
    }
    
    /// Get interpolated frame data between two frames
    pub fn interpolate_frames(&self, frame1: usize, frame2: usize, t: f32) -> Option<InterpolatedFrame> {
        let f1 = self.frames.get(frame1)?;
        let f2 = self.frames.get(frame2)?;
        
        // Calculate position delta based on sprite bounding box changes
        let dx = (f2.frame.width as i16 - f1.frame.width as i16) as f32;
        let dy = (f2.frame.height as i16 - f1.frame.height as i16) as f32;
        
        Some(InterpolatedFrame {
            position_x: Self::lerp(0.0, dx, t),
            position_y: Self::lerp(0.0, dy, t),
            scale_x: Self::lerp(1.0, 1.0, t), // Could vary based on frame data
            scale_y: Self::lerp(1.0, 1.0, t),
            rotation: Self::lerp(0.0, 0.0, t), // Could be extracted from sprite transforms
            opacity: Self::lerp(1.0, 1.0, t),
        })
    }
    
    fn lerp(a: f32, b: f32, t: f32) -> f32 {
        a + (b - a) * t
    }
}

/// Interpolated frame data for smooth animation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterpolatedFrame {
    pub position_x: f32,
    pub position_y: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub rotation: f32,
    pub opacity: f32,
}

impl InterpolatedFrame {
    /// Apply interpolation to get final transform values
    pub fn apply_easing(&self, mode: BlendMode, t: f32) -> Self {
        let eased_t = match mode {
            BlendMode::None => if t < 1.0 { 0.0 } else { 1.0 },
            BlendMode::Linear => t,
            BlendMode::EaseIn => t * t,
            BlendMode::EaseOut => 1.0 - (1.0 - t) * (1.0 - t),
            BlendMode::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
                }
            }
        };
        
        Self {
            position_x: self.position_x * eased_t,
            position_y: self.position_y * eased_t,
            scale_x: 1.0 + (self.scale_x - 1.0) * eased_t,
            scale_y: 1.0 + (self.scale_y - 1.0) * eased_t,
            rotation: self.rotation * eased_t,
            opacity: self.opacity * eased_t,
        }
    }
}

/// Animation player state
#[derive(Debug, Clone)]
pub struct AnimationPlayer {
    /// Current animation sequence
    pub sequence: AnimationSequence,
    /// Current frame in the animation
    pub current_frame: usize,
    /// Current time within the frame (0.0 - 1.0)
    pub frame_time: f32,
    /// Playback speed (1.0 = normal, 2.0 = double)
    pub playback_speed: f32,
    /// Whether the animation is playing
    pub is_playing: bool,
    /// Whether to interpolate between frames
    pub interpolate: bool,
}

impl AnimationPlayer {
    pub fn new(sequence: AnimationSequence) -> Self {
        Self {
            sequence,
            current_frame: 0,
            frame_time: 0.0,
            playback_speed: 1.0,
            is_playing: false,
            interpolate: true,
        }
    }
    
    /// Update the animation by delta time (in seconds)
    pub fn update(&mut self, delta_time: f32) {
        if !self.is_playing {
            return;
        }
        
        let frame_duration = 1.0 / 60.0; // 60 FPS
        self.frame_time += delta_time * self.playback_speed / frame_duration;
        
        while self.frame_time >= 1.0 {
            self.frame_time -= 1.0;
            self.advance_frame();
        }
    }
    
    fn advance_frame(&mut self) {
        self.current_frame += 1;
        
        if self.current_frame >= self.sequence.frames.len() {
            if self.sequence.loops {
                self.current_frame = self.sequence.loop_start;
            } else {
                self.current_frame = self.sequence.frames.len().saturating_sub(1);
                self.is_playing = false;
            }
        }
    }
    
    /// Get the current display frame with interpolation
    pub fn get_current_frame(&self) -> Option<(&AnimationFrame, Option<InterpolatedFrame>)> {
        let frame = self.sequence.frames.get(self.current_frame)?;
        
        let interp = if self.interpolate && self.frame_time > 0.0 {
            let next_frame = (self.current_frame + 1) % self.sequence.frames.len();
            self.sequence.interpolate_frames(self.current_frame, next_frame, self.frame_time)
        } else {
            None
        };
        
        Some((frame, interp))
    }
    
    /// Play the animation
    pub fn play(&mut self) {
        self.is_playing = true;
    }
    
    /// Pause the animation
    pub fn pause(&mut self) {
        self.is_playing = false;
    }
    
    /// Stop and reset to beginning
    pub fn stop(&mut self) {
        self.is_playing = false;
        self.current_frame = 0;
        self.frame_time = 0.0;
    }
    
    /// Seek to a specific frame
    pub fn seek(&mut self, frame: usize) {
        self.current_frame = frame.min(self.sequence.frames.len().saturating_sub(1));
        self.frame_time = 0.0;
    }
}

/// Hitbox editor for creating and modifying hitboxes
pub struct HitboxEditor {
    pub selected_hitbox: Option<usize>,
    pub selected_hurtbox: Option<usize>,
    pub edit_mode: EditMode,
    pub show_hitboxes: bool,
    pub show_hurtboxes: bool,
    pub snap_to_grid: bool,
    pub grid_size: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditMode {
    Select,
    CreateHitbox,
    CreateHurtbox,
    Move,
    Resize,
}

impl Default for HitboxEditor {
    fn default() -> Self {
        Self {
            selected_hitbox: None,
            selected_hurtbox: None,
            edit_mode: EditMode::Select,
            show_hitboxes: true,
            show_hurtboxes: true,
            snap_to_grid: false,
            grid_size: 8,
        }
    }
}

impl HitboxEditor {
    /// Create a new hitbox at the given position
    pub fn create_hitbox(&self, x: i16, y: i16) -> CombatHitbox {
        let (sx, sy) = if self.snap_to_grid {
            (
                (x / self.grid_size as i16) * self.grid_size as i16,
                (y / self.grid_size as i16) * self.grid_size as i16,
            )
        } else {
            (x, y)
        };
        
        CombatHitbox {
            x: sx,
            y: sy,
            ..Default::default()
        }
    }
    
    /// Create a new hurtbox at the given position
    pub fn create_hurtbox(&self, x: i16, y: i16) -> Hurtbox {
        let (sx, sy) = if self.snap_to_grid {
            (
                (x / self.grid_size as i16) * self.grid_size as i16,
                (y / self.grid_size as i16) * self.grid_size as i16,
            )
        } else {
            (x, y)
        };
        
        Hurtbox {
            x: sx,
            y: sy,
            ..Default::default()
        }
    }
}
