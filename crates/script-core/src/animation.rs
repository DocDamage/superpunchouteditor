use rom_core::Rom;
use serde::{Deserialize, Serialize};

/// A single animation frame
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnimationFrame {
    pub pose_id: u8,
    pub duration: u8, // frames to hold (60fps)
    pub tileset_id: u8,
    pub effects: Vec<FrameEffect>,
}

impl Default for AnimationFrame {
    fn default() -> Self {
        Self {
            pose_id: 0,
            duration: 4,
            tileset_id: 0,
            effects: Vec::new(),
        }
    }
}

/// Effects that can be triggered on a specific frame
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "data")]
pub enum FrameEffect {
    Shake,                                 // Screen shake
    Flash,                                 // Flash effect
    Sound(u8),                             // Sound effect ID
    Hitbox { x: i8, y: i8, w: u8, h: u8 }, // Hit detection
}

impl FrameEffect {
    pub fn name(&self) -> &'static str {
        match self {
            FrameEffect::Shake => "Shake",
            FrameEffect::Flash => "Flash",
            FrameEffect::Sound(_) => "Sound",
            FrameEffect::Hitbox { .. } => "Hitbox",
        }
    }

    pub fn description(&self) -> String {
        match self {
            FrameEffect::Shake => "Screen shake effect".to_string(),
            FrameEffect::Flash => "Flash effect".to_string(),
            FrameEffect::Sound(id) => format!("Sound effect #{}", id),
            FrameEffect::Hitbox { x, y, w, h } => {
                format!("Hitbox at ({}, {}) size {}x{}", x, y, w, h)
            }
        }
    }
}

/// Categories of animations for organization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AnimationCategory {
    Idle,
    PunchLeft,
    PunchRight,
    Dodge,
    Hit,
    Knockdown,
    Special,
    Custom(String),
}

impl AnimationCategory {
    pub fn as_str(&self) -> &str {
        match self {
            AnimationCategory::Idle => "Idle",
            AnimationCategory::PunchLeft => "PunchLeft",
            AnimationCategory::PunchRight => "PunchRight",
            AnimationCategory::Dodge => "Dodge",
            AnimationCategory::Hit => "Hit",
            AnimationCategory::Knockdown => "Knockdown",
            AnimationCategory::Special => "Special",
            AnimationCategory::Custom(s) => s.as_str(),
        }
    }

    pub fn display_name(&self) -> String {
        match self {
            AnimationCategory::Idle => "Idle".to_string(),
            AnimationCategory::PunchLeft => "Punch (Left)".to_string(),
            AnimationCategory::PunchRight => "Punch (Right)".to_string(),
            AnimationCategory::Dodge => "Dodge".to_string(),
            AnimationCategory::Hit => "Hit Reaction".to_string(),
            AnimationCategory::Knockdown => "Knockdown".to_string(),
            AnimationCategory::Special => "Special Move".to_string(),
            AnimationCategory::Custom(s) => s.clone(),
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            AnimationCategory::Idle => "⏸️",
            AnimationCategory::PunchLeft => "👊",
            AnimationCategory::PunchRight => "🥊",
            AnimationCategory::Dodge => "💨",
            AnimationCategory::Hit => "😵",
            AnimationCategory::Knockdown => "💫",
            AnimationCategory::Special => "⭐",
            AnimationCategory::Custom(_) => "📦",
        }
    }
}

impl Default for AnimationCategory {
    fn default() -> Self {
        AnimationCategory::Idle
    }
}

/// Complete animation sequence
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Animation {
    pub name: String,
    pub frames: Vec<AnimationFrame>,
    pub looping: bool,
    pub category: AnimationCategory,
}

impl Animation {
    pub fn new(name: impl Into<String>, category: AnimationCategory) -> Self {
        Self {
            name: name.into(),
            frames: Vec::new(),
            looping: false,
            category,
        }
    }

    /// Get total duration of animation in frames
    pub fn total_duration(&self) -> u32 {
        self.frames.iter().map(|f| f.duration as u32).sum()
    }

    /// Get duration in seconds at 60fps
    pub fn duration_seconds(&self) -> f32 {
        self.total_duration() as f32 / 60.0
    }

    /// Add a frame at the end
    pub fn add_frame(&mut self, frame: AnimationFrame) {
        self.frames.push(frame);
    }

    /// Insert a frame at specific index
    pub fn insert_frame(&mut self, index: usize, frame: AnimationFrame) {
        if index <= self.frames.len() {
            self.frames.insert(index, frame);
        }
    }

    /// Remove a frame at specific index
    pub fn remove_frame(&mut self, index: usize) -> Option<AnimationFrame> {
        if index < self.frames.len() {
            Some(self.frames.remove(index))
        } else {
            None
        }
    }

    /// Move frame from one index to another
    pub fn move_frame(&mut self, from_index: usize, to_index: usize) -> Result<(), String> {
        if from_index >= self.frames.len() || to_index > self.frames.len() {
            return Err("Invalid frame index".to_string());
        }

        let frame = self.frames.remove(from_index);
        let insert_idx = if to_index > from_index {
            to_index - 1
        } else {
            to_index
        };
        self.frames.insert(insert_idx, frame);
        Ok(())
    }
}

/// Animation data for a specific fighter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FighterAnimations {
    pub fighter_id: usize,
    pub fighter_name: String,
    pub animations: Vec<Animation>,
}

/// Manager for reading and writing animation data
pub struct AnimationManager<'a> {
    #[allow(dead_code)]
    rom: &'a Rom,
}

impl<'a> AnimationManager<'a> {
    pub fn new(rom: &'a Rom) -> Self {
        Self { rom }
    }

    /// Get fighter names list
    pub fn get_fighter_names(&self) -> Vec<&'static str> {
        vec![
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
        ]
    }

    /// Parse animation data from ROM for a specific fighter
    ///
    /// NOTE: This is a placeholder implementation. The actual animation table
    /// structure in SPO ROM needs to be reverse-engineered for full support.
    /// Currently returns sample animations based on available pose data.
    pub fn get_animations_for_fighter(&self, fighter_id: usize) -> FighterAnimations {
        let fighter_names = self.get_fighter_names();
        let fighter_name = fighter_names
            .get(fighter_id)
            .unwrap_or(&"Unknown")
            .to_string();

        // Sample animation sequences based on common fighter patterns
        // In a full implementation, this would parse actual animation tables from ROM
        let animations = vec![
            Animation {
                name: "Idle".to_string(),
                frames: vec![
                    AnimationFrame {
                        pose_id: 0,
                        duration: 8,
                        tileset_id: 0,
                        effects: vec![],
                    },
                    AnimationFrame {
                        pose_id: 1,
                        duration: 8,
                        tileset_id: 0,
                        effects: vec![],
                    },
                    AnimationFrame {
                        pose_id: 0,
                        duration: 8,
                        tileset_id: 0,
                        effects: vec![],
                    },
                    AnimationFrame {
                        pose_id: 2,
                        duration: 8,
                        tileset_id: 0,
                        effects: vec![],
                    },
                ],
                looping: true,
                category: AnimationCategory::Idle,
            },
            Animation {
                name: "Left Jab".to_string(),
                frames: vec![
                    AnimationFrame {
                        pose_id: 10,
                        duration: 4,
                        tileset_id: 1,
                        effects: vec![],
                    },
                    AnimationFrame {
                        pose_id: 11,
                        duration: 3,
                        tileset_id: 1,
                        effects: vec![FrameEffect::Hitbox {
                            x: 20,
                            y: -10,
                            w: 30,
                            h: 20,
                        }],
                    },
                    AnimationFrame {
                        pose_id: 12,
                        duration: 6,
                        tileset_id: 1,
                        effects: vec![],
                    },
                    AnimationFrame {
                        pose_id: 10,
                        duration: 4,
                        tileset_id: 1,
                        effects: vec![],
                    },
                ],
                looping: false,
                category: AnimationCategory::PunchLeft,
            },
            Animation {
                name: "Right Hook".to_string(),
                frames: vec![
                    AnimationFrame {
                        pose_id: 20,
                        duration: 6,
                        tileset_id: 2,
                        effects: vec![],
                    },
                    AnimationFrame {
                        pose_id: 21,
                        duration: 4,
                        tileset_id: 2,
                        effects: vec![FrameEffect::Shake],
                    },
                    AnimationFrame {
                        pose_id: 22,
                        duration: 3,
                        tileset_id: 2,
                        effects: vec![FrameEffect::Hitbox {
                            x: 25,
                            y: -5,
                            w: 35,
                            h: 25,
                        }],
                    },
                    AnimationFrame {
                        pose_id: 23,
                        duration: 8,
                        tileset_id: 2,
                        effects: vec![],
                    },
                    AnimationFrame {
                        pose_id: 20,
                        duration: 6,
                        tileset_id: 2,
                        effects: vec![],
                    },
                ],
                looping: false,
                category: AnimationCategory::PunchRight,
            },
            Animation {
                name: "Dodge Left".to_string(),
                frames: vec![
                    AnimationFrame {
                        pose_id: 30,
                        duration: 4,
                        tileset_id: 0,
                        effects: vec![],
                    },
                    AnimationFrame {
                        pose_id: 31,
                        duration: 12,
                        tileset_id: 0,
                        effects: vec![],
                    },
                    AnimationFrame {
                        pose_id: 30,
                        duration: 4,
                        tileset_id: 0,
                        effects: vec![],
                    },
                    AnimationFrame {
                        pose_id: 0,
                        duration: 4,
                        tileset_id: 0,
                        effects: vec![],
                    },
                ],
                looping: false,
                category: AnimationCategory::Dodge,
            },
            Animation {
                name: "Dodge Right".to_string(),
                frames: vec![
                    AnimationFrame {
                        pose_id: 35,
                        duration: 4,
                        tileset_id: 0,
                        effects: vec![],
                    },
                    AnimationFrame {
                        pose_id: 36,
                        duration: 12,
                        tileset_id: 0,
                        effects: vec![],
                    },
                    AnimationFrame {
                        pose_id: 35,
                        duration: 4,
                        tileset_id: 0,
                        effects: vec![],
                    },
                    AnimationFrame {
                        pose_id: 0,
                        duration: 4,
                        tileset_id: 0,
                        effects: vec![],
                    },
                ],
                looping: false,
                category: AnimationCategory::Dodge,
            },
            Animation {
                name: "Hit Stun".to_string(),
                frames: vec![
                    AnimationFrame {
                        pose_id: 40,
                        duration: 4,
                        tileset_id: 0,
                        effects: vec![FrameEffect::Flash],
                    },
                    AnimationFrame {
                        pose_id: 41,
                        duration: 16,
                        tileset_id: 0,
                        effects: vec![],
                    },
                    AnimationFrame {
                        pose_id: 42,
                        duration: 8,
                        tileset_id: 0,
                        effects: vec![],
                    },
                    AnimationFrame {
                        pose_id: 0,
                        duration: 4,
                        tileset_id: 0,
                        effects: vec![],
                    },
                ],
                looping: false,
                category: AnimationCategory::Hit,
            },
            Animation {
                name: "Knockdown".to_string(),
                frames: vec![
                    AnimationFrame {
                        pose_id: 50,
                        duration: 4,
                        tileset_id: 0,
                        effects: vec![FrameEffect::Shake, FrameEffect::Flash],
                    },
                    AnimationFrame {
                        pose_id: 51,
                        duration: 6,
                        tileset_id: 0,
                        effects: vec![],
                    },
                    AnimationFrame {
                        pose_id: 52,
                        duration: 8,
                        tileset_id: 0,
                        effects: vec![],
                    },
                    AnimationFrame {
                        pose_id: 53,
                        duration: 60,
                        tileset_id: 0,
                        effects: vec![],
                    },
                ],
                looping: false,
                category: AnimationCategory::Knockdown,
            },
            Animation {
                name: "Get Up".to_string(),
                frames: vec![
                    AnimationFrame {
                        pose_id: 54,
                        duration: 20,
                        tileset_id: 0,
                        effects: vec![],
                    },
                    AnimationFrame {
                        pose_id: 55,
                        duration: 8,
                        tileset_id: 0,
                        effects: vec![],
                    },
                    AnimationFrame {
                        pose_id: 0,
                        duration: 8,
                        tileset_id: 0,
                        effects: vec![],
                    },
                ],
                looping: false,
                category: AnimationCategory::Special,
            },
        ];

        FighterAnimations {
            fighter_id,
            fighter_name,
            animations,
        }
    }

    /// Serialize animation back to ROM format
    ///
    /// NOTE: This is a placeholder. Full implementation requires
    /// understanding the ROM's animation table structure.
    pub fn serialize_animation(&self, _animation: &Animation) -> Vec<u8> {
        // Placeholder: Return empty bytes
        // In full implementation, this would convert Animation struct
        // back to the ROM's native format
        Vec::new()
    }

    /// Get all animation categories
    pub fn get_categories() -> Vec<AnimationCategory> {
        vec![
            AnimationCategory::Idle,
            AnimationCategory::PunchLeft,
            AnimationCategory::PunchRight,
            AnimationCategory::Dodge,
            AnimationCategory::Hit,
            AnimationCategory::Knockdown,
            AnimationCategory::Special,
        ]
    }
}

/// Animation playback state for preview
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationPlayback {
    pub current_frame: usize,
    pub frame_counter: u8,
    pub is_playing: bool,
    pub speed: f32,
}

impl AnimationPlayback {
    pub fn new() -> Self {
        Self {
            current_frame: 0,
            frame_counter: 0,
            is_playing: false,
            speed: 1.0,
        }
    }

    pub fn reset(&mut self) {
        self.current_frame = 0;
        self.frame_counter = 0;
        self.is_playing = false;
    }

    pub fn play(&mut self) {
        self.is_playing = true;
    }

    pub fn pause(&mut self) {
        self.is_playing = false;
    }

    pub fn stop(&mut self) {
        self.reset();
    }

    /// Advance frame, returns true if wrapped around (for looping)
    pub fn advance(&mut self, animation: &Animation) -> bool {
        if !self.is_playing || animation.frames.is_empty() {
            return false;
        }

        let frame_duration =
            (animation.frames[self.current_frame].duration as f32 / self.speed) as u8;
        self.frame_counter += 1;

        if self.frame_counter >= frame_duration.max(1) {
            self.frame_counter = 0;
            self.current_frame += 1;

            if self.current_frame >= animation.frames.len() {
                if animation.looping {
                    self.current_frame = 0;
                    return true;
                } else {
                    self.current_frame = animation.frames.len() - 1;
                    self.is_playing = false;
                }
            }
        }

        false
    }

    pub fn set_frame(&mut self, frame_index: usize) {
        self.current_frame = frame_index;
        self.frame_counter = 0;
    }

    pub fn next_frame(&mut self, animation: &Animation) {
        if animation.frames.is_empty() {
            return;
        }
        self.current_frame = (self.current_frame + 1) % animation.frames.len();
        self.frame_counter = 0;
    }

    pub fn prev_frame(&mut self, animation: &Animation) {
        if animation.frames.is_empty() {
            return;
        }
        if self.current_frame == 0 {
            self.current_frame = animation.frames.len() - 1;
        } else {
            self.current_frame -= 1;
        }
        self.frame_counter = 0;
    }
}

impl Default for AnimationPlayback {
    fn default() -> Self {
        Self::new()
    }
}
