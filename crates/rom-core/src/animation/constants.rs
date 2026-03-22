//! Animation Constants for Super Punch-Out!!
//!
//! ROM addresses and constants for animation data based on reverse engineering.

use super::types::*;

// ============================================================================
// FIGHTER HEADER OFFSET CONSTANTS (Bank $09)
// ============================================================================

/// Fighter header table base (SNES $09:8000)
pub const FIGHTER_HEADER_BASE: usize = 0x048000;

/// Size of each fighter header entry (32 bytes)
pub const FIGHTER_HEADER_SIZE: usize = 32;

/// Number of fighters
pub const FIGHTER_COUNT: usize = 16;

/// Offset within fighter header to pose table pointer (2 bytes)
pub const POSE_TABLE_PTR_OFFSET: usize = 0x06;

/// Offset within fighter header to AI script pointer (2 bytes)
pub const AI_SCRIPT_PTR_OFFSET: usize = 0x08;

// ============================================================================
// POSE DATA CONSTANTS
// ============================================================================

/// Maximum number of poses per fighter
pub const MAX_POSES_PER_FIGHTER: usize = 128;

/// Size of each pose entry in the pose table (5 bytes)
pub const POSE_ENTRY_SIZE: usize = 5;

/// Offsets within pose entry
pub const POSE_TILESET1_OFFSET: usize = 0x00;
pub const POSE_TILESET2_OFFSET: usize = 0x01;
pub const POSE_PALETTE_ID_OFFSET: usize = 0x02;
pub const POSE_DATA_PTR_OFFSET: usize = 0x03;

// ============================================================================
// ANIMATION SEQUENCE CONSTANTS
// ============================================================================

/// Animation sequence terminator value (marks end of sequence)
pub const ANIMATION_TERMINATOR: u8 = 0xFF;

/// Maximum frames per animation sequence
pub const MAX_FRAMES_PER_ANIMATION: usize = 64;

/// Animation frame entry size (variable, but typically 2-4 bytes)
pub const ANIMATION_FRAME_MIN_SIZE: usize = 2;

// ============================================================================
// HITBOX/HURTBOX CONSTANTS
// ============================================================================

/// Hitbox entry size in ROM (6 bytes)
pub const HITBOX_ENTRY_SIZE: usize = 6;

/// Hurtbox entry size in ROM (6 bytes)
pub const HURTBOX_ENTRY_SIZE: usize = 6;

/// Maximum hitboxes per frame
pub const MAX_HITBOXES_PER_FRAME: usize = 4;

/// Maximum hurtboxes per frame
pub const MAX_HURTBOXES_PER_FRAME: usize = 4;

// ============================================================================
// ANIMATION TYPE IDENTIFIERS
// ============================================================================

/// Animation type IDs used in the ROM
/// These are used to identify different animation categories
pub const ANIM_TYPE_IDLE: u8 = 0x00;
pub const ANIM_TYPE_JAB: u8 = 0x01;
pub const ANIM_TYPE_HOOK: u8 = 0x02;
pub const ANIM_TYPE_UPPERCUT: u8 = 0x03;
pub const ANIM_TYPE_DODGE_LEFT: u8 = 0x04;
pub const ANIM_TYPE_DODGE_RIGHT: u8 = 0x05;
pub const ANIM_TYPE_BLOCK: u8 = 0x06;
pub const ANIM_TYPE_HIT_REACTION: u8 = 0x07;
pub const ANIM_TYPE_KNOCKDOWN: u8 = 0x08;
pub const ANIM_TYPE_GET_UP: u8 = 0x09;
pub const ANIM_TYPE_SPECIAL: u8 = 0x0A;
pub const ANIM_TYPE_VICTORY: u8 = 0x0B;
pub const ANIM_TYPE_TAUNT: u8 = 0x0C;

// ============================================================================
// FRAME EFFECT FLAGS
// ============================================================================

/// Effect flag: Screen shake
pub const EFFECT_SHAKE: u8 = 0x01;

/// Effect flag: Flash effect
pub const EFFECT_FLASH: u8 = 0x02;

/// Effect flag: Sound trigger
pub const EFFECT_SOUND: u8 = 0x04;

/// Effect flag: Hitbox active
pub const EFFECT_HITBOX: u8 = 0x08;

/// Effect flag: Invincibility frames
pub const EFFECT_INVINCIBLE: u8 = 0x10;

// ============================================================================
// DEFAULT ANIMATION DATA
// ============================================================================

/// Default frame duration (in 60fps frames)
pub const DEFAULT_FRAME_DURATION: u8 = 4;

/// Default hitbox damage
pub const DEFAULT_HITBOX_DAMAGE: u8 = 10;

/// Default hitbox hitstun (frames)
pub const DEFAULT_HITBOX_HITSTUN: u8 = 10;

/// Default knockback angle (degrees)
pub const DEFAULT_KNOCKBACK_ANGLE: u16 = 45;

/// Default knockback power
pub const DEFAULT_KNOCKBACK_POWER: u8 = 5;

/// Get animation type name from ID
pub fn animation_type_name(type_id: u8) -> &'static str {
    match type_id {
        ANIM_TYPE_IDLE => "Idle",
        ANIM_TYPE_JAB => "Jab",
        ANIM_TYPE_HOOK => "Hook",
        ANIM_TYPE_UPPERCUT => "Uppercut",
        ANIM_TYPE_DODGE_LEFT => "Dodge Left",
        ANIM_TYPE_DODGE_RIGHT => "Dodge Right",
        ANIM_TYPE_BLOCK => "Block",
        ANIM_TYPE_HIT_REACTION => "Hit Reaction",
        ANIM_TYPE_KNOCKDOWN => "Knockdown",
        ANIM_TYPE_GET_UP => "Get Up",
        ANIM_TYPE_SPECIAL => "Special",
        ANIM_TYPE_VICTORY => "Victory",
        ANIM_TYPE_TAUNT => "Taunt",
        _ => "Unknown",
    }
}

/// Get animation type ID from name
pub fn animation_type_from_name(name: &str) -> Option<u8> {
    match name {
        "Idle" => Some(ANIM_TYPE_IDLE),
        "Jab" => Some(ANIM_TYPE_JAB),
        "Hook" => Some(ANIM_TYPE_HOOK),
        "Uppercut" => Some(ANIM_TYPE_UPPERCUT),
        "Dodge Left" => Some(ANIM_TYPE_DODGE_LEFT),
        "Dodge Right" => Some(ANIM_TYPE_DODGE_RIGHT),
        "Block" => Some(ANIM_TYPE_BLOCK),
        "Hit Reaction" => Some(ANIM_TYPE_HIT_REACTION),
        "Knockdown" => Some(ANIM_TYPE_KNOCKDOWN),
        "Get Up" => Some(ANIM_TYPE_GET_UP),
        "Special" => Some(ANIM_TYPE_SPECIAL),
        "Victory" => Some(ANIM_TYPE_VICTORY),
        "Taunt" => Some(ANIM_TYPE_TAUNT),
        _ => None,
    }
}

/// Get animation category from type ID
pub fn animation_category_from_type(type_id: u8) -> AnimationCategory {
    match type_id {
        ANIM_TYPE_IDLE => AnimationCategory::Idle,
        ANIM_TYPE_JAB | ANIM_TYPE_HOOK | ANIM_TYPE_UPPERCUT => AnimationCategory::Punch,
        ANIM_TYPE_DODGE_LEFT | ANIM_TYPE_DODGE_RIGHT | ANIM_TYPE_BLOCK => AnimationCategory::Dodge,
        ANIM_TYPE_HIT_REACTION => AnimationCategory::Hit,
        ANIM_TYPE_KNOCKDOWN | ANIM_TYPE_GET_UP => AnimationCategory::Knockdown,
        ANIM_TYPE_SPECIAL => AnimationCategory::Special,
        ANIM_TYPE_VICTORY | ANIM_TYPE_TAUNT => AnimationCategory::Taunt,
        _ => AnimationCategory::Custom("Unknown".to_string()),
    }
}
