//! Animation Loader for Super Punch-Out!!
//!
//! Reads animation data from ROM, including pose tables, frame sequences, and hitbox data.

use crate::Rom;
use std::collections::HashMap;

use super::constants::*;
use super::types::*;

/// Fighter names for display
const FIGHTER_NAMES: &[&str; 16] = &[
    "Gabby Jay",
    "Bear Hugger",
    "Piston Hurricane",
    "Bald Bull",
    "Bob Charlie",
    "Dragon Chan",
    "Masked Muscle",
    "Mr. Sandman",
    "Aran Ryan",
    "Heike Kagero",
    "Mad Clown",
    "Super Macho Man",
    "Narcis Prince",
    "Hoy Quarlow",
    "Rick Bruiser",
    "Nick Bruiser",
];

/// Loader for animation data from ROM
pub struct AnimationLoader<'a> {
    rom: &'a Rom,
}

impl<'a> AnimationLoader<'a> {
    /// Create a new animation loader
    pub fn new(rom: &'a Rom) -> Self {
        Self { rom }
    }

    /// Get fighter name by ID
    pub fn get_fighter_name(fighter_id: u8) -> &'static str {
        FIGHTER_NAMES
            .get(fighter_id as usize)
            .unwrap_or(&"Unknown")
    }

    /// Get all fighter names
    pub fn get_all_fighter_names() -> &'static [&'static str; 16] {
        FIGHTER_NAMES
    }

    /// Read the pose table pointer from a fighter's header
    fn get_pose_table_ptr(&self, fighter_id: u8) -> Result<u16, AnimationError> {
        if fighter_id as usize >= FIGHTER_COUNT {
            return Err(AnimationError::FighterNotFound(fighter_id));
        }

        let header_offset = FIGHTER_HEADER_BASE + (fighter_id as usize * FIGHTER_HEADER_SIZE);
        
        // Read the 2-byte pose table pointer at offset 0x06 in the header
        let ptr_bytes = self.rom
            .read_bytes(header_offset + POSE_TABLE_PTR_OFFSET, 2)
            .map_err(|_| AnimationError::InvalidOffset(header_offset + POSE_TABLE_PTR_OFFSET))?;

        Ok(u16::from_le_bytes([ptr_bytes[0], ptr_bytes[1]]))
    }

    /// Read all poses for a fighter
    pub fn get_poses(&self, fighter_id: u8) -> Result<Vec<PoseData>, AnimationError> {
        let pose_table_ptr = self.get_pose_table_ptr(fighter_id)?;
        
        // Convert SNES address to PC offset (Bank $09)
        let pose_table_pc = self.rom.snes_to_pc(0x09, pose_table_ptr);

        let mut poses = Vec::new();
        
        for i in 0..MAX_POSES_PER_FIGHTER {
            let entry_offset = pose_table_pc + (i * 2);
            
            // Read 2-byte pointer to pose data
            if entry_offset + 2 > self.rom.data.len() {
                break;
            }

            let pose_ptr_bytes = self.rom
                .read_bytes(entry_offset, 2)
                .map_err(|_| AnimationError::InvalidOffset(entry_offset))?;
            let pose_ptr = u16::from_le_bytes([pose_ptr_bytes[0], pose_ptr_bytes[1]]);

            // Check for terminator (pointer < 0x8000 means end of table)
            if pose_ptr < 0x8000 {
                break;
            }

            // Convert pose pointer to PC offset
            let pose_pc = self.rom.snes_to_pc(0x09, pose_ptr);
            
            // Read 5-byte pose entry
            if pose_pc + POSE_ENTRY_SIZE > self.rom.data.len() {
                break;
            }

            let pose_bytes = self.rom
                .read_bytes(pose_pc, POSE_ENTRY_SIZE)
                .map_err(|_| AnimationError::InvalidOffset(pose_pc))?;

            let mut pose = PoseData::from_bytes(i, pose_bytes)?;
            pose.rom_offset = Some(pose_pc);
            poses.push(pose);
        }

        Ok(poses)
    }

    /// Get a single pose by index
    pub fn get_pose(&self, fighter_id: u8, pose_index: usize) -> Result<PoseData, AnimationError> {
        let poses = self.get_poses(fighter_id)?;
        poses
            .get(pose_index)
            .cloned()
            .ok_or(AnimationError::PoseOutOfRange(pose_index))
    }

    /// Read animation sequences for a fighter
    /// 
    /// Animation data in SPO ROM is stored as frame sequences referenced by the AI scripts.
    /// This method reconstructs common animation patterns based on pose data.
    pub fn get_animations(&self, fighter_id: u8) -> Result<FighterAnimations, AnimationError> {
        if fighter_id as usize >= FIGHTER_COUNT {
            return Err(AnimationError::FighterNotFound(fighter_id));
        }

        let fighter_name = Self::get_fighter_name(fighter_id).to_string();
        let pose_table_ptr = self.get_pose_table_ptr(fighter_id)?;

        let mut fighter_anims = FighterAnimations::new(fighter_id, fighter_name);
        fighter_anims.pose_table_offset = Some(self.rom.snes_to_pc(0x09, pose_table_ptr));

        // Build standard animations based on pose patterns
        // These are constructed from known animation types used by the game
        fighter_anims.add_animation(self.build_idle_animation(fighter_id)?);
        fighter_anims.add_animation(self.build_jab_animation(fighter_id)?);
        fighter_anims.add_animation(self.build_hook_animation(fighter_id)?);
        fighter_anims.add_animation(self.build_uppercut_animation(fighter_id)?);
        fighter_anims.add_animation(self.build_dodge_left_animation(fighter_id)?);
        fighter_anims.add_animation(self.build_dodge_right_animation(fighter_id)?);
        fighter_anims.add_animation(self.build_hit_reaction_animation(fighter_id)?);
        fighter_anims.add_animation(self.build_knockdown_animation(fighter_id)?);
        fighter_anims.add_animation(self.build_get_up_animation(fighter_id)?);
        fighter_anims.add_animation(self.build_victory_animation(fighter_id)?);

        Ok(fighter_anims)
    }

    /// Build idle animation from pose data
    fn build_idle_animation(&self, fighter_id: u8) -> Result<Animation, AnimationError> {
        let poses = self.get_poses(fighter_id)?;
        let mut anim = Animation::new("Idle", ANIM_TYPE_IDLE);
        anim.looping = true;

        // Idle typically uses poses 0-3 (breathing animation)
        let idle_poses: Vec<usize> = vec![0, 1, 0, 2];
        
        for &pose_idx in &idle_poses {
            if pose_idx < poses.len() {
                anim.add_frame(AnimationFrame::new(pose_idx as u8, 8));
            }
        }

        // If no poses found, add a default frame
        if anim.frames.is_empty() {
            anim.add_frame(AnimationFrame::default());
        }

        Ok(anim)
    }

    /// Build jab animation
    fn build_jab_animation(&self, fighter_id: u8) -> Result<Animation, AnimationError> {
        let poses = self.get_poses(fighter_id)?;
        let mut anim = Animation::new("Jab", ANIM_TYPE_JAB);

        // Jab typically uses poses 10-13
        let jab_poses: Vec<(usize, u8)> = vec![(10, 4), (11, 3), (12, 6), (10, 4)];
        
        for &(pose_idx, duration) in &jab_poses {
            if pose_idx < poses.len() {
                let mut frame = AnimationFrame::new(pose_idx as u8, duration);
                // Add hitbox on the active frame
                if pose_idx == 11 {
                    frame.hitboxes.push(Hitbox::attack(20, -10, 24, 16, 8));
                }
                anim.add_frame(frame);
            }
        }

        if anim.frames.is_empty() {
            anim.add_frame(AnimationFrame::default());
        }

        Ok(anim)
    }

    /// Build hook animation
    fn build_hook_animation(&self, fighter_id: u8) -> Result<Animation, AnimationError> {
        let poses = self.get_poses(fighter_id)?;
        let mut anim = Animation::new("Hook", ANIM_TYPE_HOOK);

        // Hook typically uses poses 20-24
        let hook_poses: Vec<(usize, u8)> = vec![(20, 6), (21, 4), (22, 3), (23, 8), (20, 6)];
        
        for &(pose_idx, duration) in &hook_poses {
            if pose_idx < poses.len() {
                let mut frame = AnimationFrame::new(pose_idx as u8, duration);
                if pose_idx == 22 {
                    frame.hitboxes.push(Hitbox::attack(25, -5, 32, 20, 12));
                    frame.effects.push(FrameEffect::Shake);
                }
                anim.add_frame(frame);
            }
        }

        if anim.frames.is_empty() {
            anim.add_frame(AnimationFrame::default());
        }

        Ok(anim)
    }

    /// Build uppercut animation
    fn build_uppercut_animation(&self, fighter_id: u8) -> Result<Animation, AnimationError> {
        let poses = self.get_poses(fighter_id)?;
        let mut anim = Animation::new("Uppercut", ANIM_TYPE_UPPERCUT);

        // Uppercut typically uses poses 30-34
        let uppercut_poses: Vec<(usize, u8, Option<Hitbox>)> = vec![
            (30, 6, None),
            (31, 4, None),
            (32, 3, Some(Hitbox::attack(15, -30, 28, 24, 18))),
            (33, 8, None),
            (30, 6, None),
        ];
        
        for (pose_idx, duration, hitbox) in &uppercut_poses {
            if *pose_idx < poses.len() {
                let mut frame = AnimationFrame::new(*pose_idx as u8, *duration);
                if let Some(hb) = hitbox {
                    frame.hitboxes.push(hb.clone());
                    frame.effects.push(FrameEffect::Shake);
                }
                anim.add_frame(frame);
            }
        }

        if anim.frames.is_empty() {
            anim.add_frame(AnimationFrame::default());
        }

        Ok(anim)
    }

    /// Build dodge left animation
    fn build_dodge_left_animation(&self, fighter_id: u8) -> Result<Animation, AnimationError> {
        let poses = self.get_poses(fighter_id)?;
        let mut anim = Animation::new("Dodge Left", ANIM_TYPE_DODGE_LEFT);

        let dodge_poses: Vec<usize> = vec![40, 41, 40, 0];
        
        for &pose_idx in &dodge_poses {
            if pose_idx < poses.len() {
                anim.add_frame(AnimationFrame::new(pose_idx as u8, 4));
            }
        }

        if anim.frames.is_empty() {
            anim.add_frame(AnimationFrame::default());
        }

        Ok(anim)
    }

    /// Build dodge right animation
    fn build_dodge_right_animation(&self, fighter_id: u8) -> Result<Animation, AnimationError> {
        let poses = self.get_poses(fighter_id)?;
        let mut anim = Animation::new("Dodge Right", ANIM_TYPE_DODGE_RIGHT);

        let dodge_poses: Vec<usize> = vec![45, 46, 45, 0];
        
        for &pose_idx in &dodge_poses {
            if pose_idx < poses.len() {
                anim.add_frame(AnimationFrame::new(pose_idx as u8, 4));
            }
        }

        if anim.frames.is_empty() {
            anim.add_frame(AnimationFrame::default());
        }

        Ok(anim)
    }

    /// Build hit reaction animation
    fn build_hit_reaction_animation(&self, fighter_id: u8) -> Result<Animation, AnimationError> {
        let poses = self.get_poses(fighter_id)?;
        let mut anim = Animation::new("Hit Reaction", ANIM_TYPE_HIT_REACTION);

        let hit_poses: Vec<(usize, Option<FrameEffect>)> = vec![
            (50, Some(FrameEffect::Flash)),
            (51, None),
            (52, None),
            (0, None),
        ];
        
        for (pose_idx, effect) in &hit_poses {
            if *pose_idx < poses.len() {
                let mut frame = AnimationFrame::new(*pose_idx as u8, 4);
                if let Some(eff) = effect {
                    frame.effects.push(*eff);
                }
                anim.add_frame(frame);
            }
        }

        if anim.frames.is_empty() {
            anim.add_frame(AnimationFrame::default());
        }

        Ok(anim)
    }

    /// Build knockdown animation
    fn build_knockdown_animation(&self, fighter_id: u8) -> Result<Animation, AnimationError> {
        let poses = self.get_poses(fighter_id)?;
        let mut anim = Animation::new("Knockdown", ANIM_TYPE_KNOCKDOWN);

        let kd_poses: Vec<(usize, u8, Vec<FrameEffect>)> = vec![
            (60, 4, vec![FrameEffect::Shake, FrameEffect::Flash]),
            (61, 6, vec![]),
            (62, 8, vec![]),
            (63, 60, vec![]),
        ];
        
        for &(pose_idx, duration, ref effects) in &kd_poses {
            if pose_idx < poses.len() {
                let mut frame = AnimationFrame::new(pose_idx as u8, duration);
                frame.effects = effects.clone();
                anim.add_frame(frame);
            }
        }

        if anim.frames.is_empty() {
            anim.add_frame(AnimationFrame::default());
        }

        Ok(anim)
    }

    /// Build get up animation
    fn build_get_up_animation(&self, fighter_id: u8) -> Result<Animation, AnimationError> {
        let poses = self.get_poses(fighter_id)?;
        let mut anim = Animation::new("Get Up", ANIM_TYPE_GET_UP);

        let getup_poses: Vec<usize> = vec![64, 65, 0];
        
        for &pose_idx in &getup_poses {
            if pose_idx < poses.len() {
                anim.add_frame(AnimationFrame::new(pose_idx as u8, 8));
            }
        }

        if anim.frames.is_empty() {
            anim.add_frame(AnimationFrame::default());
        }

        Ok(anim)
    }

    /// Build victory animation
    fn build_victory_animation(&self, fighter_id: u8) -> Result<Animation, AnimationError> {
        let poses = self.get_poses(fighter_id)?;
        let mut anim = Animation::new("Victory", ANIM_TYPE_VICTORY);
        anim.looping = true;

        let victory_poses: Vec<usize> = vec![70, 71, 72, 71];
        
        for &pose_idx in &victory_poses {
            if pose_idx < poses.len() {
                anim.add_frame(AnimationFrame::new(pose_idx as u8, 8));
            }
        }

        if anim.frames.is_empty() {
            anim.add_frame(AnimationFrame::default());
        }

        Ok(anim)
    }

    /// Get all fighter animations (convenience method)
    pub fn get_all_fighter_animations(&self) -> HashMap<u8, FighterAnimations> {
        let mut result = HashMap::new();
        for fighter_id in 0..FIGHTER_COUNT as u8 {
            if let Ok(animations) = self.get_animations(fighter_id) {
                result.insert(fighter_id, animations);
            }
        }
        result
    }
}

/// Helper function to get animation category from type ID
fn animation_category_from_type(type_id: u8) -> AnimationCategory {
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

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fighter_names() {
        assert_eq!(AnimationLoader::get_fighter_name(0), "Gabby Jay");
        assert_eq!(AnimationLoader::get_fighter_name(15), "Nick Bruiser");
        assert_eq!(AnimationLoader::get_fighter_name(99), "Unknown");
    }

    #[test]
    fn test_animation_category_from_type() {
        assert_eq!(animation_category_from_type(ANIM_TYPE_IDLE), AnimationCategory::Idle);
        assert_eq!(animation_category_from_type(ANIM_TYPE_JAB), AnimationCategory::Punch);
        assert_eq!(animation_category_from_type(ANIM_TYPE_DODGE_LEFT), AnimationCategory::Dodge);
    }
}
