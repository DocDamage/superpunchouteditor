//! Animation data management for Super Punch-Out!!
//!
//! This module handles animation frame data including:
//! - Pose sequences and frame timing
//! - Hitbox and hurtbox collision data
//! - Frame effects (screen shake, flash, sound triggers)
//!
//! # ROM Structure
//!
//! Based on Super Punch-Out!! (USA) ROM research:
//!
//! ## Fighter Header Table (Bank $09)
//! Each fighter has a 32-byte header at $09:8000 + (fighter_id * 0x20)
//! - Offset 0x06-0x07: pose_table_ptr (pointer to pose/animation table)
//!
//! ## Pose Table Structure
//! Each pose entry is 5 bytes:
//! - Byte 0: tileset1_id
//! - Byte 1: tileset2_id  
//! - Byte 2: palette_id
//! - Bytes 3-4: data_addr (pointer to sprite/metasprite data)
//!
//! ## Animation Frame Data
//! Animation frames reference poses and include:
//! - Duration (frames to display)
//! - Hitbox/hurtbox data
//! - Effect triggers

mod constants;
mod types;
mod loader;
mod writer;

// Re-export all public types from submodules
pub use constants::*;
pub use types::*;
pub use loader::AnimationLoader;
pub use writer::AnimationWriter;

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_animation_frame_default() {
        let frame = AnimationFrame::default();
        assert_eq!(frame.pose_id, 0);
        assert_eq!(frame.duration, 4);
        assert_eq!(frame.tileset_id, 0);
        assert!(frame.effects.is_empty());
        assert!(frame.hitboxes.is_empty());
        assert!(frame.hurtboxes.is_empty());
    }

    #[test]
    fn test_hitbox_default() {
        let hitbox = Hitbox::default();
        assert_eq!(hitbox.hitbox_type, HitboxType::Attack);
        assert_eq!(hitbox.x, 0);
        assert_eq!(hitbox.y, 0);
        assert_eq!(hitbox.width, 16);
        assert_eq!(hitbox.height, 16);
    }

    #[test]
    fn test_hurtbox_default() {
        let hurtbox = Hurtbox::default();
        assert_eq!(hurtbox.x, 0);
        assert_eq!(hurtbox.y, 0);
        assert_eq!(hurtbox.width, 32);
        assert_eq!(hurtbox.height, 48);
    }

    #[test]
    fn test_animation_total_duration() {
        let mut animation = Animation::new("Test", ANIM_TYPE_IDLE);
        animation.add_frame(AnimationFrame {
            pose_id: 0,
            duration: 8,
            ..Default::default()
        });
        animation.add_frame(AnimationFrame {
            pose_id: 1,
            duration: 12,
            ..Default::default()
        });
        assert_eq!(animation.total_duration(), 20);
    }

    #[test]
    fn test_animation_category_display() {
        assert_eq!(AnimationCategory::Idle.display_name(), "Idle");
        assert_eq!(AnimationCategory::Punch.display_name(), "Punch");
        assert_eq!(AnimationCategory::Hit.display_name(), "Hit Reaction");
    }
}
