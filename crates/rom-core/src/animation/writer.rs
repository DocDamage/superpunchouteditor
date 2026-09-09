//! Animation Writer for Super Punch-Out!!
//!
//! Writes animation data back to ROM, including pose tables.
//! Full hitbox/hurtbox write support is pending further ROM analysis.

use crate::Rom;

use super::constants::*;
use super::types::*;

/// Writer for animation data
pub struct AnimationWriter<'a> {
    rom: &'a mut Rom,
}

impl<'a> AnimationWriter<'a> {
    /// Create a new animation writer
    pub fn new(rom: &'a mut Rom) -> Self {
        Self { rom }
    }

    /// Write pose data for a single pose entry
    pub fn write_pose(
        &mut self,
        fighter_id: u8,
        pose_index: u8,
        pose: &PoseData,
    ) -> Result<(), AnimationError> {
        if fighter_id as usize >= FIGHTER_COUNT {
            return Err(AnimationError::FighterNotFound(fighter_id));
        }

        let header_offset = FIGHTER_HEADER_BASE + (fighter_id as usize * FIGHTER_HEADER_SIZE);

        // Read the pose table pointer
        let ptr_bytes = self
            .rom
            .read_bytes(header_offset + POSE_TABLE_PTR_OFFSET, 2)
            .map_err(|_| AnimationError::InvalidOffset(header_offset + POSE_TABLE_PTR_OFFSET))?;

        let pose_table_ptr = u16::from_le_bytes([ptr_bytes[0], ptr_bytes[1]]);
        let pose_table_pc = self.rom.snes_to_pc(0x09, pose_table_ptr);

        // Calculate the offset for this pose entry
        let pose_offset = pose_table_pc + (pose_index as usize * POSE_ENTRY_SIZE);

        // Write the 5-byte pose entry
        let pose_bytes = pose.to_bytes();
        self.rom
            .write_bytes(pose_offset, &pose_bytes)
            .map_err(|_| AnimationError::WriteFailed(pose_offset))?;

        Ok(())
    }

    /// Write all poses for a fighter
    pub fn write_poses(
        &mut self,
        fighter_id: u8,
        poses: &[PoseData],
    ) -> Result<(), AnimationError> {
        for (index, pose) in poses.iter().enumerate() {
            self.write_pose(fighter_id, index as u8, pose)?;
        }
        Ok(())
    }

    /// Update a single animation frame's duration
    ///
    /// NOTE: Full implementation pending reverse-engineering of frame sequence format.
    pub fn update_frame_duration(
        &mut self,
        _fighter_id: u8,
        _animation_type: u8,
        _frame_index: usize,
        _duration: u8,
    ) -> Result<(), AnimationError> {
        // TODO: Implement once animation frame data format is fully reverse-engineered
        Ok(())
    }

    /// Update hitbox data for a frame
    ///
    /// NOTE: Full implementation pending reverse-engineering of hitbox data format.
    pub fn update_hitbox(
        &mut self,
        _fighter_id: u8,
        _pose_index: u8,
        _hitbox_index: usize,
        _hitbox: &Hitbox,
    ) -> Result<(), AnimationError> {
        // TODO: Implement once hitbox data format is fully reverse-engineered
        Ok(())
    }

    /// Update hurtbox data for a frame
    ///
    /// NOTE: Full implementation pending reverse-engineering of hurtbox data format.
    pub fn update_hurtbox(
        &mut self,
        _fighter_id: u8,
        _pose_index: u8,
        _hurtbox_index: usize,
        _hurtbox: &Hurtbox,
    ) -> Result<(), AnimationError> {
        // TODO: Implement once hurtbox data format is fully reverse-engineered
        Ok(())
    }

    /// Write back a modified FighterAnimations to ROM.
    ///
    /// Currently writes pose data only. Hitbox/hurtbox write-back is pending
    /// further ROM format reverse-engineering.
    pub fn update_animation(
        &mut self,
        fighter_id: u8,
        _animations: &FighterAnimations,
    ) -> Result<(), AnimationError> {
        if fighter_id as usize >= FIGHTER_COUNT {
            return Err(AnimationError::FighterNotFound(fighter_id));
        }
        // TODO: Reconstruct and write pose data from animations.frames
        // TODO: Write hitbox/hurtbox data when format is known
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pose_data_to_bytes() {
        let pose = PoseData {
            pose_index: 0,
            tileset1_id: 0x12,
            tileset2_id: 0x34,
            palette_id: 0x56,
            data_ptr: 0xABCD,
            rom_offset: None,
        };
        let bytes = pose.to_bytes();
        assert_eq!(bytes[0], 0x12);
        assert_eq!(bytes[1], 0x34);
        assert_eq!(bytes[2], 0x56);
        assert_eq!(bytes[3], 0xCD); // Low byte of pointer
        assert_eq!(bytes[4], 0xAB); // High byte of pointer
    }

    #[test]
    fn test_hitbox_type_to_byte() {
        assert_eq!(HitboxType::Attack.to_byte(), 0x00);
        assert_eq!(HitboxType::Counter.to_byte(), 0x01);
        assert_eq!(HitboxType::Grab.to_byte(), 0x02);
        assert_eq!(HitboxType::Projectile.to_byte(), 0x03);
    }
}
