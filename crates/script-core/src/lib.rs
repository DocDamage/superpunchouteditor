use rom_core::Rom;
use serde::{Deserialize, Serialize};

pub mod animation;
pub use animation::*;

pub mod ai_behavior;
pub use ai_behavior::*;
// Re-export AI-specific types for convenience
pub use ai_behavior::{
    AiAction, AiBehavior, AiBehaviorManager, AiParseError, AiParser, AiPresets, AiTrigger,
    AttackMove, AttackPattern, Condition, DefenseBehavior, DefenseType, DifficultyCurve,
    DifficultyRating, Direction, HeightZone, Hitbox, MoveType, RoundDifficulty, SimulationResult,
    AI_DEFENSE_TABLE, AI_PATTERN_TABLE, AI_TABLE_BASE, AI_TRIGGER_TABLE, FIGHTER_HEADER_BASE,
    MAX_FIGHTERS,
};

/// Known SPO animation/behavior script categories
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ScriptCategory {
    AnimationScript,
    AiScript,
    SpriteScript,
    CornerManScript,
    PlayerScript,
    Unknown,
}

impl std::fmt::Display for ScriptCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AnimationScript => write!(f, "Animation Script"),
            Self::AiScript => write!(f, "AI Script"),
            Self::SpriteScript => write!(f, "Sprite Script"),
            Self::CornerManScript => write!(f, "Corner Man Script"),
            Self::PlayerScript => write!(f, "Player Script"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Risk level for editing a particular script entry  
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

impl std::fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "Low"),
            Self::Medium => write!(f, "Medium"),
            Self::High => write!(f, "High"),
        }
    }
}

/// A single discovered script record in ROM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptRecord {
    /// Human label for this script slot
    pub label: String,
    /// SNES bank
    pub bank: u8,
    /// SNES address within bank
    pub snes_addr: u16,
    /// PC offset
    pub pc_offset: usize,
    /// Raw bytes (first 64 for preview)
    pub preview_bytes: Vec<u8>,
    /// Category
    pub category: ScriptCategory,
    /// Risk
    pub risk: RiskLevel,
    /// Short description of what this script does
    pub description: String,
    /// Is this a shared script (used by multiple boxers)?
    pub is_shared: bool,
    /// List of boxer names that reference this script
    pub owners: Vec<String>,
}

/// Known SPO Script address table entries
/// These are hardcoded from disassembly research of Super Punch-Out!! (USA)
#[derive(Debug, Clone)]
pub struct KnownScriptEntry {
    pub label: &'static str,
    pub bank: u8,
    pub addr: u16,
    pub category: ScriptCategory,
    pub risk: RiskLevel,
    pub description: &'static str,
    pub owners: &'static [&'static str],
    pub is_shared: bool,
}

/// All known SPO script entry points from the disassembly.
/// Bank 00-3F: LoROM mapped. Scripts are primarily in banks $88-$9F (LoROM hiBanks).
/// The fighter header table lives in bank $09 at $8000.
pub const KNOWN_SCRIPTS: &[KnownScriptEntry] = &[
    // --- Fighter AI header table: bank $09 ---
    KnownScriptEntry {
        label: "Fighter Header Table",
        bank: 0x09, addr: 0x8000,
        category: ScriptCategory::AiScript,
        risk: RiskLevel::High,
        description: "Master table of fighter header pointers. Each entry is 0x20 bytes. Points to AI params, pose table, etc.",
        owners: &["All Fighters"],
        is_shared: true,
    },
    // --- Gabby Jay ---
    KnownScriptEntry {
        label: "Gabby Jay: Header",
        bank: 0x09, addr: 0x8000,
        category: ScriptCategory::AiScript,
        risk: RiskLevel::High,
        description: "Fighter header for Gabby Jay. Contains palette IDs, attack params, AI pointers.",
        owners: &["Gabby Jay"],
        is_shared: false,
    },
    KnownScriptEntry {
        label: "Gabby Jay: Pose Table",
        bank: 0x09, addr: 0x8006,
        category: ScriptCategory::AnimationScript,
        risk: RiskLevel::Medium,
        description: "Pointer to pose/animation table for Gabby Jay.",
        owners: &["Gabby Jay"],
        is_shared: false,
    },
    // --- Bear Hugger ---
    KnownScriptEntry {
        label: "Bear Hugger: Header",
        bank: 0x09, addr: 0x8020,
        category: ScriptCategory::AiScript,
        risk: RiskLevel::High,
        description: "Fighter header for Bear Hugger.",
        owners: &["Bear Hugger"],
        is_shared: false,
    },
    // --- Piston Hurricane ---
    KnownScriptEntry {
        label: "Piston Hurricane: Header",
        bank: 0x09, addr: 0x8040,
        category: ScriptCategory::AiScript,
        risk: RiskLevel::High,
        description: "Fighter header for Piston Hurricane.",
        owners: &["Piston Hurricane"],
        is_shared: false,
    },
    // --- Bald Bull ---
    KnownScriptEntry {
        label: "Bald Bull: Header",
        bank: 0x09, addr: 0x8060,
        category: ScriptCategory::AiScript,
        risk: RiskLevel::High,
        description: "Fighter header for Bald Bull.",
        owners: &["Bald Bull"],
        is_shared: false,
    },
    // --- Bob Charlie ---
    KnownScriptEntry {
        label: "Bob Charlie: Header",
        bank: 0x09, addr: 0x8080,
        category: ScriptCategory::AiScript,
        risk: RiskLevel::High,
        description: "Fighter header for Bob Charlie.",
        owners: &["Bob Charlie"],
        is_shared: false,
    },
    // --- Dragon Chan ---
    KnownScriptEntry {
        label: "Dragon Chan: Header",
        bank: 0x09, addr: 0x80A0,
        category: ScriptCategory::AiScript,
        risk: RiskLevel::High,
        description: "Fighter header for Dragon Chan.",
        owners: &["Dragon Chan"],
        is_shared: false,
    },
    // --- Masked Muscle ---
    KnownScriptEntry {
        label: "Masked Muscle: Header",
        bank: 0x09, addr: 0x80C0,
        category: ScriptCategory::AiScript,
        risk: RiskLevel::High,
        description: "Fighter header for Masked Muscle.",
        owners: &["Masked Muscle"],
        is_shared: false,
    },
    // --- Mr. Sandman ---
    KnownScriptEntry {
        label: "Mr. Sandman: Header",
        bank: 0x09, addr: 0x80E0,
        category: ScriptCategory::AiScript,
        risk: RiskLevel::High,
        description: "Fighter header for Mr. Sandman.",
        owners: &["Mr. Sandman"],
        is_shared: false,
    },
    // --- Aran Ryan ---
    KnownScriptEntry {
        label: "Aran Ryan: Header",
        bank: 0x09, addr: 0x8100,
        category: ScriptCategory::AiScript,
        risk: RiskLevel::High,
        description: "Fighter header for Aran Ryan.",
        owners: &["Aran Ryan"],
        is_shared: false,
    },
    // --- Heike Kagero ---
    KnownScriptEntry {
        label: "Heike Kagero: Header",
        bank: 0x09, addr: 0x8120,
        category: ScriptCategory::AiScript,
        risk: RiskLevel::High,
        description: "Fighter header for Heike Kagero.",
        owners: &["Heike Kagero"],
        is_shared: false,
    },
    // --- Mad Clown ---
    KnownScriptEntry {
        label: "Mad Clown: Header",
        bank: 0x09, addr: 0x8140,
        category: ScriptCategory::AiScript,
        risk: RiskLevel::High,
        description: "Fighter header for Mad Clown.",
        owners: &["Mad Clown"],
        is_shared: false,
    },
    // --- Super Macho Man ---
    KnownScriptEntry {
        label: "Super Macho Man: Header",
        bank: 0x09, addr: 0x8160,
        category: ScriptCategory::AiScript,
        risk: RiskLevel::High,
        description: "Fighter header for Super Macho Man.",
        owners: &["Super Macho Man"],
        is_shared: false,
    },
    // --- Narcis Prince ---
    KnownScriptEntry {
        label: "Narcis Prince: Header",
        bank: 0x09, addr: 0x8180,
        category: ScriptCategory::AiScript,
        risk: RiskLevel::High,
        description: "Fighter header for Narcis Prince.",
        owners: &["Narcis Prince"],
        is_shared: false,
    },
    // --- Hoy Quarlow ---
    KnownScriptEntry {
        label: "Hoy Quarlow: Header",
        bank: 0x09, addr: 0x81A0,
        category: ScriptCategory::AiScript,
        risk: RiskLevel::High,
        description: "Fighter header for Hoy Quarlow.",
        owners: &["Hoy Quarlow"],
        is_shared: false,
    },
    // --- Rick Bruiser ---
    KnownScriptEntry {
        label: "Rick Bruiser: Header",
        bank: 0x09, addr: 0x81C0,
        category: ScriptCategory::AiScript,
        risk: RiskLevel::High,
        description: "Fighter header for Rick Bruiser.",
        owners: &["Rick Bruiser"],
        is_shared: false,
    },
    // --- Nick Bruiser ---
    KnownScriptEntry {
        label: "Nick Bruiser: Header",
        bank: 0x09, addr: 0x81E0,
        category: ScriptCategory::AiScript,
        risk: RiskLevel::High,
        description: "Fighter header for Nick Bruiser.",
        owners: &["Nick Bruiser"],
        is_shared: false,
    },
    // --- Shared AIScript behaviors ---
    KnownScriptEntry {
        label: "Gabby Jay / Bob Charlie: Shared AI",
        bank: 0x09, addr: 0x8200,
        category: ScriptCategory::AiScript,
        risk: RiskLevel::High,
        description: "AI behavior shared between Gabby Jay and Bob Charlie. Editing affects both.",
        owners: &["Gabby Jay", "Bob Charlie"],
        is_shared: true,
    },
    KnownScriptEntry {
        label: "Bear Hugger / Mad Clown: Shared AI",
        bank: 0x09, addr: 0x8220,
        category: ScriptCategory::AiScript,
        risk: RiskLevel::High,
        description: "AI behavior shared between Bear Hugger and Mad Clown.",
        owners: &["Bear Hugger", "Mad Clown"],
        is_shared: true,
    },
    KnownScriptEntry {
        label: "Rick Bruiser / Nick Bruiser: Shared AI",
        bank: 0x09, addr: 0x8240,
        category: ScriptCategory::AiScript,
        risk: RiskLevel::High,
        description: "AI behavior shared between Rick Bruiser and Nick Bruiser.",
        owners: &["Rick Bruiser", "Nick Bruiser"],
        is_shared: true,
    },
    // --- Corner man scripts ---
    KnownScriptEntry {
        label: "Corner Man Dialog Table",
        bank: 0x09, addr: 0x9000,
        category: ScriptCategory::CornerManScript,
        risk: RiskLevel::Low,
        description: "Table of corner man message pointers for all boxers. Low risk to edit.",
        owners: &["All Fighters"],
        is_shared: true,
    },
    // --- Player ---
    KnownScriptEntry {
        label: "Little Mac: Player Script",
        bank: 0x09, addr: 0xA000,
        category: ScriptCategory::PlayerScript,
        risk: RiskLevel::High,
        description: "Little Mac player movement and attack script. High risk — global player behavior.",
        owners: &["Little Mac"],
        is_shared: false,
    },
];

/// Safe-to-edit fighter parameters with validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditableFighterParams {
    pub palette_id: u8,     // 0-255
    pub attack_power: u8,   // 0-255
    pub defense_rating: u8, // 0-255
    pub speed_rating: u8,   // 0-255
}

/// Validation warnings for extreme parameter values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamValidationResult {
    pub valid: bool,
    pub warnings: Vec<String>,
    pub is_extreme: bool,
}

impl EditableFighterParams {
    /// Create from a FighterHeader
    #[allow(deprecated)]
    pub fn from_header(header: &FighterHeader) -> Self {
        Self {
            palette_id: header.palette_id,
            attack_power: header.attack_power,
            defense_rating: header.defense_rating,
            speed_rating: header.speed_rating,
        }
    }

    /// Validate parameters are in safe ranges
    pub fn validate(&self) -> Result<(), String> {
        // All fields are u8 so they're already bounded 0-255
        // Just check for any game-specific constraints
        if self.attack_power > 250 {
            return Err("Attack power cannot exceed 250".to_string());
        }
        if self.defense_rating > 250 {
            return Err("Defense rating cannot exceed 250".to_string());
        }
        if self.speed_rating > 250 {
            return Err("Speed rating cannot exceed 250".to_string());
        }
        Ok(())
    }

    /// Validate and return detailed result with warnings
    pub fn validate_with_warnings(&self) -> ParamValidationResult {
        let mut warnings = Vec::new();
        let mut is_extreme = false;

        // Check for extreme values that might make the game unbalanced
        if self.attack_power > 200 {
            warnings.push(format!(
                "Attack power ({}) is extremely high. This may make the boxer unfairly difficult.",
                self.attack_power
            ));
            is_extreme = true;
        } else if self.attack_power > 150 {
            warnings.push(format!(
                "Attack power ({}) is very high.",
                self.attack_power
            ));
        }

        if self.defense_rating > 200 {
            warnings.push(format!(
                "Defense rating ({}) is extremely high. The boxer may take very little damage.",
                self.defense_rating
            ));
            is_extreme = true;
        }

        if self.speed_rating > 200 {
            warnings.push(format!(
                "Speed rating ({}) is extremely high. The boxer may be too fast to hit.",
                self.speed_rating
            ));
            is_extreme = true;
        } else if self.speed_rating > 150 {
            warnings.push(format!(
                "Speed rating ({}) is very high.",
                self.speed_rating
            ));
        }

        // Check for very low values (may make boxer too easy)
        if self.attack_power < 10 {
            warnings.push(format!(
                "Attack power ({}) is very low. The boxer may be too easy to defeat.",
                self.attack_power
            ));
        }

        let valid = self.validate().is_ok();

        ParamValidationResult {
            valid,
            warnings,
            is_extreme,
        }
    }
}

pub struct ScriptReader<'a> {
    rom: &'a Rom,
}

impl<'a> ScriptReader<'a> {
    pub fn new(rom: &'a Rom) -> Self {
        Self { rom }
    }

    /// Return all known script records, reading preview bytes from the ROM.
    pub fn get_all_scripts(&self) -> Vec<ScriptRecord> {
        KNOWN_SCRIPTS
            .iter()
            .map(|entry| {
                let pc = self.rom.snes_to_pc(entry.bank, entry.addr);
                let preview = self
                    .rom
                    .read_bytes(pc, 64.min(self.rom.data.len().saturating_sub(pc)))
                    .unwrap_or(&[])
                    .to_vec();
                ScriptRecord {
                    label: entry.label.to_string(),
                    bank: entry.bank,
                    snes_addr: entry.addr,
                    pc_offset: pc,
                    preview_bytes: preview,
                    category: entry.category.clone(),
                    risk: entry.risk.clone(),
                    description: entry.description.to_string(),
                    is_shared: entry.is_shared,
                    owners: entry.owners.iter().map(|s| s.to_string()).collect(),
                }
            })
            .collect()
    }

    /// Return only scripts relevant to a specific fighter by name.
    pub fn get_scripts_for_fighter(&self, fighter_name: &str) -> Vec<ScriptRecord> {
        self.get_all_scripts()
            .into_iter()
            .filter(|r| {
                r.owners
                    .iter()
                    .any(|o| o == fighter_name || o == "All Fighters")
            })
            .collect()
    }

    /// Decode a 32-byte boxer header into labeled fields.
    pub fn decode_boxer_header(&self, boxer_index: usize) -> BoxerHeader {
        let base_snes = 0x8000u16 + (boxer_index as u16 * 0x20);
        let pc = self.rom.snes_to_pc(0x09, base_snes);
        let raw = self
            .rom
            .read_bytes(pc, 32.min(self.rom.data.len().saturating_sub(pc)))
            .unwrap_or(&[])
            .to_vec();

        let get_u8 = |bytes: &[u8], i: usize| bytes.get(i).copied().unwrap_or(0);
        let get_u16 = |bytes: &[u8], i: usize| {
            let lo = bytes.get(i).copied().unwrap_or(0) as u16;
            let hi = bytes.get(i + 1).copied().unwrap_or(0) as u16;
            (hi << 8) | lo
        };

        BoxerHeader {
            pc_offset: pc,
            snes_bank: 0x09,
            snes_addr: base_snes,
            palette_id: get_u8(&raw, 0),
            attack_power: get_u8(&raw, 1),
            defense_rating: get_u8(&raw, 2),
            speed_rating: get_u8(&raw, 3),
            pose_table_ptr: get_u16(&raw, 6),
            ai_script_ptr: get_u16(&raw, 8),
            corner_man_ptr: get_u16(&raw, 10),
            raw_bytes: raw,
        }
    }

    /// Get editable parameters for a boxer
    pub fn get_editable_params(&self, boxer_index: usize) -> EditableFighterParams {
        let header = self.decode_boxer_header(boxer_index);
        EditableFighterParams::from_header(&header)
    }

    /// Generate the new 32-byte header with updated parameters
    /// Returns the modified header bytes and the PC offset for writing
    pub fn generate_header_with_params(
        &self,
        boxer_index: usize,
        params: &EditableFighterParams,
    ) -> Result<(Vec<u8>, usize), String> {
        self._generate_header_with_params(boxer_index, params)
    }

    fn _generate_header_with_params(
        &self,
        fighter_index: usize,
        params: &EditableFighterParams,
    ) -> Result<(Vec<u8>, usize), String> {
        // Validate params first
        params.validate()?;

        let base_snes = 0x8000u16 + (fighter_index as u16 * 0x20);
        let pc = self.rom.snes_to_pc(0x09, base_snes);

        // Read current header
        let mut raw = self
            .rom
            .read_bytes(pc, 32.min(self.rom.data.len().saturating_sub(pc)))
            .unwrap_or(&[])
            .to_vec();

        if raw.len() < 32 {
            return Err(format!(
                "Insufficient header data for fighter {}. Got {} bytes, expected 32.",
                fighter_index,
                raw.len()
            ));
        }

        // Update the editable fields in place
        raw[0] = params.palette_id;
        raw[1] = params.attack_power;
        raw[2] = params.defense_rating;
        raw[3] = params.speed_rating;

        Ok((raw, pc))
    }
}

/// Decoded boxer header fields (32 bytes per boxer, bank $09 starting at $8000)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoxerHeader {
    pub pc_offset: usize,
    pub snes_bank: u8,
    pub snes_addr: u16,
    pub raw_bytes: Vec<u8>,
    // Known fields:
    pub palette_id: u8,
    pub attack_power: u8,
    pub defense_rating: u8,
    pub speed_rating: u8,
    pub pose_table_ptr: u16,
    pub ai_script_ptr: u16,
    pub corner_man_ptr: u16,
}

/// Deprecated: Use `BoxerHeader` instead
#[deprecated(since = "0.1.0", note = "Use BoxerHeader instead")]
pub type FighterHeader = BoxerHeader;

impl BoxerHeader {
    /// Get editable parameters from this header
    pub fn to_editable_params(&self) -> EditableFighterParams {
        EditableFighterParams {
            palette_id: self.palette_id,
            attack_power: self.attack_power,
            defense_rating: self.defense_rating,
            speed_rating: self.speed_rating,
        }
    }
}
