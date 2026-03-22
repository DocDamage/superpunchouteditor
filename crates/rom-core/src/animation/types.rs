//! Animation Types for Super Punch-Out!!
//!
//! Data structures for animation, frame, hitbox, and hurtbox data.

use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

// ============================================================================
// AnimationError
// ============================================================================

#[derive(Debug, Error)]
pub enum AnimationError {
    #[error("Fighter not found: {0}")]
    FighterNotFound(u8),
    #[error("Invalid ROM offset: 0x{0:x}")]
    InvalidOffset(usize),
    #[error("Pose out of range: {0}")]
    PoseOutOfRange(usize),
    #[error("Write failed at offset: 0x{0:x}")]
    WriteFailed(usize),
    #[error("Invalid data: {0}")]
    InvalidData(&'static str),
    #[error("Animation not found: {0}")]
    AnimationNotFound(String),
}

impl From<AnimationError> for String {
    fn from(e: AnimationError) -> String {
        e.to_string()
    }
}

// ============================================================================
// AnimationCategory
// ============================================================================

/// Categories of animations for organization
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnimationCategory {
    /// Idle stance
    Idle,
    /// Punch attacks (jab, hook, uppercut)
    Punch,
    /// Dodge moves (left, right, block)
    Dodge,
    /// Hit reaction / stun
    Hit,
    /// Knockdown and get up
    Knockdown,
    /// Special moves
    Special,
    /// Taunt / victory poses
    Taunt,
    /// Custom animation
    Custom(String),
}

impl Default for AnimationCategory {
    fn default() -> Self {
        Self::Idle
    }
}

impl AnimationCategory {
    /// Get display name for the category
    pub fn display_name(&self) -> &str {
        match self {
            Self::Idle => "Idle",
            Self::Punch => "Punch",
            Self::Dodge => "Dodge",
            Self::Hit => "Hit Reaction",
            Self::Knockdown => "Knockdown",
            Self::Special => "Special",
            Self::Taunt => "Taunt",
            Self::Custom(name) => name.as_str(),
        }
    }

    /// Get icon for the category
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Idle => "⏸️",
            Self::Punch => "👊",
            Self::Dodge => "💨",
            Self::Hit => "😵",
            Self::Knockdown => "💫",
            Self::Special => "⭐",
            Self::Taunt => "😏",
            Self::Custom(_) => "📦",
        }
    }

    /// Whether this animation loops
    pub fn is_looping(&self) -> bool {
        matches!(self, Self::Idle | Self::Taunt)
    }
}

impl fmt::Display for AnimationCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

// ============================================================================
// FrameEffect
// ============================================================================

/// Effects that can be triggered on a specific frame
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum FrameEffect {
    /// Screen shake effect
    Shake,
    /// Flash effect
    Flash,
    /// Sound effect trigger
    Sound(u8),
    /// Invincibility frames
    Invincible,
    /// Hitbox active this frame
    HitboxActive,
}

impl FrameEffect {
    /// Get effect name
    pub fn name(&self) -> &'static str {
        match self {
            Self::Shake => "Shake",
            Self::Flash => "Flash",
            Self::Sound(_) => "Sound",
            Self::Invincible => "Invincible",
            Self::HitboxActive => "Hitbox",
        }
    }

    /// Get effect description
    pub fn description(&self) -> String {
        match self {
            Self::Shake => "Screen shake effect".to_string(),
            Self::Flash => "Flash effect".to_string(),
            Self::Sound(id) => format!("Sound effect #{}", id),
            Self::Invincible => "Invincibility frames".to_string(),
            Self::HitboxActive => "Hitbox active this frame".to_string(),
        }
    }

    /// Convert to effect flags byte
    pub fn to_flags(&self) -> u8 {
        use super::constants::*;
        match self {
            Self::Shake => EFFECT_SHAKE,
            Self::Flash => EFFECT_FLASH,
            Self::Sound(_) => EFFECT_SOUND,
            Self::Invincible => EFFECT_INVINCIBLE,
            Self::HitboxActive => EFFECT_HITBOX,
        }
    }

    /// Create from effect flags byte
    pub fn from_flags(flags: u8) -> Vec<Self> {
        use super::constants::*;
        let mut effects = Vec::new();
        if flags & EFFECT_SHAKE != 0 {
            effects.push(Self::Shake);
        }
        if flags & EFFECT_FLASH != 0 {
            effects.push(Self::Flash);
        }
        if flags & EFFECT_INVINCIBLE != 0 {
            effects.push(Self::Invincible);
        }
        if flags & EFFECT_HITBOX != 0 {
            effects.push(Self::HitboxActive);
        }
        effects
    }
}

// ============================================================================
// HitboxType
// ============================================================================

/// Types of hitboxes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HitboxType {
    /// Offensive hitbox (deals damage)
    Attack,
    /// Counter hitbox (triggers counter when hit)
    Counter,
    /// Grab hitbox (for special moves)
    Grab,
    /// Projectile hitbox
    Projectile,
}

impl Default for HitboxType {
    fn default() -> Self {
        Self::Attack
    }
}

impl HitboxType {
    /// Get type from byte
    pub fn from_byte(byte: u8) -> Self {
        match byte {
            0 => Self::Attack,
            1 => Self::Counter,
            2 => Self::Grab,
            3 => Self::Projectile,
            _ => Self::Attack,
        }
    }

    /// Convert to byte representation
    pub fn to_byte(self) -> u8 {
        match self {
            Self::Attack => 0,
            Self::Counter => 1,
            Self::Grab => 2,
            Self::Projectile => 3,
        }
    }

    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Attack => "Attack",
            Self::Counter => "Counter",
            Self::Grab => "Grab",
            Self::Projectile => "Projectile",
        }
    }
}

// ============================================================================
// Hitbox
// ============================================================================

/// Hitbox data for attack collision areas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hitbox {
    /// Hitbox category
    pub hitbox_type: HitboxType,
    /// X offset from sprite origin (signed)
    pub x: i16,
    /// Y offset from sprite origin (signed)
    pub y: i16,
    /// Width in pixels
    pub width: u16,
    /// Height in pixels
    pub height: u16,
    /// Base damage dealt on hit
    pub damage: u8,
    /// Hitstun duration in frames
    pub hitstun: u8,
    /// Knockback angle in degrees
    pub knockback_angle: u8,
    /// Knockback power
    pub knockback_power: u8,
    /// Damage multiplier (percentage, 100 = normal)
    pub damage_multiplier: u16,
    /// Display color (RGBA) for editor visualization
    pub color: [u8; 4],
}

impl Default for Hitbox {
    fn default() -> Self {
        use super::constants::*;
        Self {
            hitbox_type: HitboxType::Attack,
            x: 0,
            y: 0,
            width: 16,
            height: 16,
            damage: DEFAULT_HITBOX_DAMAGE,
            hitstun: DEFAULT_HITBOX_HITSTUN,
            knockback_angle: DEFAULT_KNOCKBACK_ANGLE as u8,
            knockback_power: DEFAULT_KNOCKBACK_POWER,
            damage_multiplier: 100,
            color: [255, 64, 64, 180],
        }
    }
}

impl Hitbox {
    /// Create an attack hitbox at the given position with damage
    pub fn attack(x: i16, y: i16, width: u16, height: u16, damage: u8) -> Self {
        Self {
            hitbox_type: HitboxType::Attack,
            x,
            y,
            width,
            height,
            damage,
            ..Default::default()
        }
    }
}

// ============================================================================
// Hurtbox
// ============================================================================

/// Hurtbox data for vulnerability areas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hurtbox {
    /// X offset from sprite origin (signed)
    pub x: i16,
    /// Y offset from sprite origin (signed)
    pub y: i16,
    /// Width in pixels
    pub width: u16,
    /// Height in pixels
    pub height: u16,
    /// Damage multiplier when this box is hit (percentage, 100 = normal)
    pub damage_multiplier: u16,
    /// Display color (RGBA) for editor visualization
    pub color: [u8; 4],
}

impl Default for Hurtbox {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            width: 32,
            height: 48,
            damage_multiplier: 100,
            color: [64, 255, 64, 120],
        }
    }
}

// ============================================================================
// PoseData
// ============================================================================

/// A single pose entry from the fighter's pose table
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoseData {
    /// Index within the pose table
    pub pose_index: usize,
    /// First tileset ID
    pub tileset1_id: u8,
    /// Second tileset ID
    pub tileset2_id: u8,
    /// Palette ID
    pub palette_id: u8,
    /// Pointer to sprite/metasprite data
    pub data_ptr: u16,
    /// ROM offset of this pose entry (if known)
    pub rom_offset: Option<usize>,
}

impl PoseData {
    /// Parse from 5 raw ROM bytes
    pub fn from_bytes(pose_index: usize, bytes: &[u8]) -> Result<Self, AnimationError> {
        if bytes.len() < 5 {
            return Err(AnimationError::InvalidData("Pose data too short"));
        }
        Ok(Self {
            pose_index,
            tileset1_id: bytes[0],
            tileset2_id: bytes[1],
            palette_id: bytes[2],
            data_ptr: u16::from_le_bytes([bytes[3], bytes[4]]),
            rom_offset: None,
        })
    }

    /// Serialize back to 5 ROM bytes
    pub fn to_bytes(&self) -> [u8; 5] {
        let ptr = self.data_ptr.to_le_bytes();
        [self.tileset1_id, self.tileset2_id, self.palette_id, ptr[0], ptr[1]]
    }
}

// ============================================================================
// AnimationFrame
// ============================================================================

/// A single frame within an animation sequence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationFrame {
    /// Pose index to display for this frame
    pub pose_id: u8,
    /// Duration in 60fps frames
    pub duration: u8,
    /// Tileset override (0 = use pose default)
    pub tileset_id: u8,
    /// Active hitboxes during this frame
    pub hitboxes: Vec<Hitbox>,
    /// Active hurtboxes during this frame
    pub hurtboxes: Vec<Hurtbox>,
    /// Effects triggered on this frame
    pub effects: Vec<FrameEffect>,
    /// ROM offset if this frame was read from ROM
    pub rom_offset: Option<usize>,
}

impl Default for AnimationFrame {
    fn default() -> Self {
        Self {
            pose_id: 0,
            duration: 4,
            tileset_id: 0,
            hitboxes: Vec::new(),
            hurtboxes: Vec::new(),
            effects: Vec::new(),
            rom_offset: None,
        }
    }
}

impl AnimationFrame {
    /// Create a new frame with the given pose and duration
    pub fn new(pose_id: u8, duration: u8) -> Self {
        Self {
            pose_id,
            duration,
            ..Default::default()
        }
    }
}

// ============================================================================
// Animation
// ============================================================================

/// A named sequence of animation frames
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Animation {
    /// Display name (e.g. "Idle", "Jab")
    pub name: String,
    /// Animation type identifier (one of the ANIM_TYPE_* constants)
    pub anim_type: u8,
    /// Category derived from anim_type
    pub category: AnimationCategory,
    /// Whether this animation loops
    pub looping: bool,
    /// Ordered list of frames
    pub frames: Vec<AnimationFrame>,
}

impl Animation {
    /// Create a new empty animation
    pub fn new(name: &str, anim_type: u8) -> Self {
        use super::constants::animation_category_from_type;
        let category = animation_category_from_type(anim_type);
        let looping = matches!(category, AnimationCategory::Idle | AnimationCategory::Taunt);
        Self {
            name: name.to_string(),
            anim_type,
            category,
            looping,
            frames: Vec::new(),
        }
    }

    /// Append a frame to the animation
    pub fn add_frame(&mut self, frame: AnimationFrame) {
        self.frames.push(frame);
    }

    /// Total duration in 60fps frames
    pub fn total_duration(&self) -> u32 {
        self.frames.iter().map(|f| f.duration as u32).sum()
    }
}

// ============================================================================
// FighterAnimations
// ============================================================================

/// All animations for a single fighter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FighterAnimations {
    /// Fighter ID (0–15)
    pub fighter_id: u8,
    /// Fighter display name
    pub fighter_name: String,
    /// All animations for this fighter
    pub animations: Vec<Animation>,
    /// PC offset of this fighter's pose table (if known)
    pub pose_table_offset: Option<usize>,
}

impl FighterAnimations {
    /// Create an empty FighterAnimations
    pub fn new(fighter_id: u8, fighter_name: String) -> Self {
        Self {
            fighter_id,
            fighter_name,
            animations: Vec::new(),
            pose_table_offset: None,
        }
    }

    /// Add an animation
    pub fn add_animation(&mut self, anim: Animation) {
        self.animations.push(anim);
    }

    /// Find an animation by name (returns a clone)
    pub fn get_animation_by_name(&self, name: &str) -> Option<Animation> {
        self.animations.iter().find(|a| a.name == name).cloned()
    }

    /// Find an animation by name (mutable reference)
    pub fn get_animation_by_name_mut(&mut self, name: &str) -> Option<&mut Animation> {
        self.animations.iter_mut().find(|a| a.name == name)
    }
}
