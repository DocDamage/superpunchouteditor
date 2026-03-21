use serde::{Deserialize, Serialize};
use crate::text::TextEncoder;
use thiserror::Error;

use super::constants::*;

// ============================================================================
// CIRCUIT TYPE
// ============================================================================

/// Circuit types in Super Punch-Out!!
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CircuitType {
    /// Minor Circuit - first circuit
    Minor = 0,
    /// Major Circuit - second circuit
    Major = 1,
    /// World Circuit - third circuit
    World = 2,
    /// Special Circuit - dream fights
    Special = 3,
}

impl CircuitType {
    /// Get the display name for the circuit
    pub fn display_name(&self) -> &'static str {
        match self {
            CircuitType::Minor => "Minor Circuit",
            CircuitType::Major => "Major Circuit",
            CircuitType::World => "World Circuit",
            CircuitType::Special => "Special Circuit",
        }
    }

    /// Get the circuit number (1-4)
    pub fn number(&self) -> u8 {
        *self as u8 + 1
    }

    /// Get circuit from number
    pub fn from_number(n: u8) -> Option<Self> {
        match n {
            1 => Some(CircuitType::Minor),
            2 => Some(CircuitType::Major),
            3 => Some(CircuitType::World),
            4 => Some(CircuitType::Special),
            _ => None,
        }
    }

    /// Get circuit from raw byte value
    pub fn from_byte(byte: u8) -> Self {
        match byte {
            0 => CircuitType::Minor,
            1 => CircuitType::Major,
            2 => CircuitType::World,
            3 => CircuitType::Special,
            _ => CircuitType::Minor,
        }
    }

    /// Convert to byte value
    pub fn to_byte(&self) -> u8 {
        *self as u8
    }
}

impl Default for CircuitType {
    fn default() -> Self {
        CircuitType::Minor
    }
}

// ============================================================================
// BOXER ROSTER ENTRY
// ============================================================================

/// A single boxer entry in the roster
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoxerRosterEntry {
    /// Fighter ID (0-15)
    #[serde(rename = "boxer_id", alias = "fighter_id")]
    pub fighter_id: u8,
    /// Display name (decoded)
    pub name: String,
    /// Original ROM bytes for the name
    pub name_raw: Vec<u8>,
    /// Circuit assignment
    pub circuit: CircuitType,
    /// Order in which this boxer is unlocked (0 = starting)
    pub unlock_order: u8,
    /// ID for intro text lookup
    pub intro_text_id: u8,
    /// Whether this boxer needs to be unlocked
    pub is_unlockable: bool,
    /// Whether this boxer is a circuit champion
    pub is_champion: bool,
    /// PC offset where name data is stored
    pub name_offset: Option<usize>,
}

impl BoxerRosterEntry {
    /// Create a new boxer entry with defaults
    pub fn new(fighter_id: u8, name: impl Into<String>) -> Self {
        Self {
            fighter_id,
            name: name.into(),
            name_raw: Vec::new(),
            circuit: CircuitType::Minor,
            unlock_order: fighter_id,
            intro_text_id: fighter_id,
            is_unlockable: fighter_id > 0,
            is_champion: false,
            name_offset: None,
        }
    }

    /// Check if this is a champion (last boxer in circuit)
    pub fn is_circuit_champion(&self) -> bool {
        self.is_champion
    }
}

// ============================================================================
// CIRCUIT
// ============================================================================

/// Circuit information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Circuit {
    /// Circuit type
    pub circuit_type: CircuitType,
    /// Display name
    pub name: String,
    /// Fighter IDs in this circuit (in order)
    pub boxers: Vec<u8>,
    /// Number of wins required to challenge this circuit
    pub required_wins: u8,
}

impl Circuit {
    /// Create a new circuit
    pub fn new(circuit_type: CircuitType) -> Self {
        Self {
            name: circuit_type.display_name().to_string(),
            circuit_type,
            boxers: Vec::new(),
            required_wins: 0,
        }
    }

    /// Get the champion (last boxer) of this circuit
    pub fn champion(&self) -> Option<u8> {
        self.boxers.last().copied()
    }

    /// Check if a boxer is in this circuit
    pub fn contains(&self, fighter_id: u8) -> bool {
        self.boxers.contains(&fighter_id)
    }
}

// ============================================================================
// ROSTER DATA
// ============================================================================

/// Complete roster data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RosterData {
    /// All boxers in the game
    pub boxers: Vec<BoxerRosterEntry>,
    /// Circuits
    pub circuits: Vec<Circuit>,
    /// ROM offsets for name table
    pub name_table_offset: Option<usize>,
    /// ROM offsets for circuit table
    pub circuit_table_offset: Option<usize>,
    /// ROM offsets for unlock order table
    pub unlock_table_offset: Option<usize>,
    /// ROM offsets for intro text pointers
    pub intro_text_table_offset: Option<usize>,
}

impl Default for RosterData {
    fn default() -> Self {
        Self::new()
    }
}

impl RosterData {
    /// Create a new roster with default data
    pub fn new() -> Self {
        let boxers = get_default_boxers();
        let circuits = get_default_circuits();

        Self {
            boxers,
            circuits,
            name_table_offset: Some(BOXER_NAME_TABLE),
            circuit_table_offset: Some(CIRCUIT_TABLE),
            unlock_table_offset: Some(UNLOCK_ORDER_TABLE),
            intro_text_table_offset: Some(BOXER_INTRO_TABLE),
        }
    }

    /// Get a boxer by fighter ID
    pub fn get_boxer(&self, fighter_id: u8) -> Option<&BoxerRosterEntry> {
        self.boxers.iter().find(|b| b.fighter_id == fighter_id)
    }

    /// Get a mutable boxer by fighter ID
    pub fn get_boxer_mut(&mut self, fighter_id: u8) -> Option<&mut BoxerRosterEntry> {
        self.boxers.iter_mut().find(|b| b.fighter_id == fighter_id)
    }

    /// Get boxers in a specific circuit
    pub fn get_boxers_in_circuit(&self, circuit: CircuitType) -> Vec<&BoxerRosterEntry> {
        self.boxers
            .iter()
            .filter(|b| b.circuit == circuit)
            .collect()
    }

    /// Get boxers in unlock order
    pub fn get_boxers_by_unlock_order(&self) -> Vec<&BoxerRosterEntry> {
        let mut result: Vec<_> = self.boxers.iter().collect();
        result.sort_by_key(|b| b.unlock_order);
        result
    }

    /// Get the circuit for a boxer
    pub fn get_boxer_circuit(&self, fighter_id: u8) -> Option<&Circuit> {
        let boxer = self.get_boxer(fighter_id)?;
        self.circuits
            .iter()
            .find(|c| c.circuit_type == boxer.circuit)
    }

    /// Update a boxer's circuit assignment
    pub fn update_boxer_circuit(
        &mut self,
        fighter_id: u8,
        new_circuit: CircuitType,
    ) -> Result<(), RosterError> {
        let boxer = self
            .get_boxer_mut(fighter_id)
            .ok_or(RosterError::InvalidFighterId(fighter_id))?;

        let old_circuit = boxer.circuit;
        boxer.circuit = new_circuit;

        // Update circuit boxer lists
        if let Some(circuit) = self
            .circuits
            .iter_mut()
            .find(|c| c.circuit_type == old_circuit)
        {
            circuit.boxers.retain(|&id| id != fighter_id);
        }
        if let Some(circuit) = self
            .circuits
            .iter_mut()
            .find(|c| c.circuit_type == new_circuit)
        {
            if !circuit.boxers.contains(&fighter_id) {
                circuit.boxers.push(fighter_id);
            }
        }

        Ok(())
    }

    /// Update a boxer's name
    pub fn update_boxer_name(
        &mut self,
        fighter_id: u8,
        new_name: impl Into<String>,
    ) -> Result<(), RosterError> {
        let boxer = self
            .get_boxer_mut(fighter_id)
            .ok_or(RosterError::InvalidFighterId(fighter_id))?;

        let name = new_name.into();

        // Validate name length
        let encoder = TextEncoder::new();
        let encoded = encoder.encode(&name);
        if encoded.len() > MAX_NAME_LENGTH {
            return Err(RosterError::NameTooLong {
                name: name.clone(),
                max_bytes: MAX_NAME_LENGTH,
                actual_bytes: encoded.len(),
            });
        }

        boxer.name = name;
        boxer.name_raw = encoded;

        Ok(())
    }

    /// Update a boxer's unlock order
    pub fn update_unlock_order(
        &mut self,
        fighter_id: u8,
        new_order: u8,
    ) -> Result<(), RosterError> {
        let boxer = self
            .get_boxer_mut(fighter_id)
            .ok_or(RosterError::InvalidFighterId(fighter_id))?;

        boxer.unlock_order = new_order;

        // Update unlockable status based on order
        boxer.is_unlockable = new_order > 0;

        Ok(())
    }

    /// Validate the entire roster for consistency
    pub fn validate(&self) -> ValidationReport {
        let mut report = ValidationReport::default();

        // Check for duplicate names
        let mut name_counts: std::collections::HashMap<&str, Vec<u8>> =
            std::collections::HashMap::new();
        for boxer in &self.boxers {
            name_counts
                .entry(&boxer.name)
                .or_default()
                .push(boxer.fighter_id);
        }
        for (name, ids) in name_counts {
            if ids.len() > 1 {
                report.warnings.push(ValidationIssue::DuplicateName {
                    name: name.to_string(),
                    fighter_ids: ids,
                });
            }
        }

        // Check for gaps in unlock order
        let mut unlock_orders: Vec<u8> = self.boxers.iter().map(|b| b.unlock_order).collect();
        unlock_orders.sort_unstable();
        for window in unlock_orders.windows(2) {
            if window[1] - window[0] > 1 {
                report.warnings.push(ValidationIssue::GapInUnlockOrder {
                    from: window[0],
                    to: window[1],
                });
            }
        }

        // Check circuit champions
        for circuit in &self.circuits {
            if let Some(champion_id) = circuit.champion() {
                if let Some(boxer) = self.get_boxer(champion_id) {
                    if !boxer.is_champion {
                        report.warnings.push(ValidationIssue::MissingChampionFlag {
                            fighter_id: champion_id,
                            circuit: circuit.circuit_type,
                        });
                    }
                }
            }
        }

        // Check that all boxers are assigned to circuits
        for boxer in &self.boxers {
            if !self.circuits.iter().any(|c| c.contains(boxer.fighter_id)) {
                report.errors.push(ValidationIssue::BoxerNotInAnyCircuit {
                    fighter_id: boxer.fighter_id,
                    name: boxer.name.clone(),
                });
            }
        }

        report.is_valid = report.errors.is_empty();
        report
    }
}

// ============================================================================
// DEFAULT DATA
// ============================================================================

/// Default boxer data (based on known game data)
fn get_default_boxers() -> Vec<BoxerRosterEntry> {
    vec![
        BoxerRosterEntry::new(0, "Gabby Jay"),
        BoxerRosterEntry::new(1, "Bear Hugger"),
        BoxerRosterEntry::new(2, "Piston Hurricane"),
        BoxerRosterEntry::new(3, "Bald Bull"),
        BoxerRosterEntry::new(4, "Bob Charlie"),
        BoxerRosterEntry::new(5, "Dragon Chan"),
        BoxerRosterEntry::new(6, "Masked Muscle"),
        BoxerRosterEntry::new(7, "Mr. Sandman"),
        BoxerRosterEntry::new(8, "Aran Ryan"),
        BoxerRosterEntry::new(9, "Heike Kagero"),
        BoxerRosterEntry::new(10, "Mad Clown"),
        BoxerRosterEntry::new(11, "Super Macho Man"),
        BoxerRosterEntry::new(12, "Narcis Prince"),
        BoxerRosterEntry::new(13, "Hoy Quarlow"),
        BoxerRosterEntry::new(14, "Rick Bruiser"),
        BoxerRosterEntry::new(15, "Nick Bruiser"),
    ]
}

/// Default circuit data
fn get_default_circuits() -> Vec<Circuit> {
    vec![
        Circuit {
            name: "Minor Circuit".to_string(),
            circuit_type: CircuitType::Minor,
            boxers: vec![0, 1, 2, 3],
            required_wins: 0,
        },
        Circuit {
            name: "Major Circuit".to_string(),
            circuit_type: CircuitType::Major,
            boxers: vec![4, 5, 6, 7],
            required_wins: 4,
        },
        Circuit {
            name: "World Circuit".to_string(),
            circuit_type: CircuitType::World,
            boxers: vec![8, 9, 10, 11],
            required_wins: 8,
        },
        Circuit {
            name: "Special Circuit".to_string(),
            circuit_type: CircuitType::Special,
            boxers: vec![12, 13, 14, 15],
            required_wins: 12,
        },
    ]
}

// ============================================================================
// BOXER INTRO DATA
// ============================================================================

/// Boxer introduction data (shown before fight)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoxerIntro {
    /// Boxer key (e.g., "gabby_jay")
    pub boxer_key: String,
    /// Display name (e.g., "GABBY JAY")
    pub name_text: String,
    /// Origin text (e.g., "From: Paris, France")
    pub origin_text: String,
    /// Record text (e.g., "Record: 1-99")
    pub record_text: String,
    /// Rank text (e.g., "Rank: #1 Contender")
    pub rank_text: String,
    /// Introductory quote
    pub intro_quote: String,
    /// ROM offsets for each field
    pub name_offset: Option<usize>,
    pub origin_offset: Option<usize>,
    pub record_offset: Option<usize>,
    pub rank_offset: Option<usize>,
    pub quote_offset: Option<usize>,
}

impl BoxerIntro {
    /// Create a new boxer intro with defaults
    pub fn new(boxer_key: impl Into<String>) -> Self {
        Self {
            boxer_key: boxer_key.into(),
            name_text: String::new(),
            origin_text: String::new(),
            record_text: String::new(),
            rank_text: String::new(),
            intro_quote: String::new(),
            name_offset: None,
            origin_offset: None,
            record_offset: None,
            rank_offset: None,
            quote_offset: None,
        }
    }

    /// Validate all fields
    pub fn validate(&self, encoder: &TextEncoder) -> Vec<(String, String)> {
        let mut errors = Vec::new();

        let fields = [
            ("name", &self.name_text, 16usize),
            ("origin", &self.origin_text, 32usize),
            ("record", &self.record_text, 20usize),
            ("rank", &self.rank_text, 24usize),
            ("quote", &self.intro_quote, 50usize),
        ];

        for (name, text, max_len) in fields {
            let encoded = encoder.encode(text);
            if encoded.len() > max_len {
                errors.push((
                    name.to_string(),
                    format!("Text too long: {} bytes (max {})", encoded.len(), max_len),
                ));
            }
            if let Err(invalid) = encoder.validate(text) {
                errors.push((
                    name.to_string(),
                    format!("Invalid characters: {:?}", invalid),
                ));
            }
        }

        errors
    }

    /// Check if all fields are valid
    pub fn is_valid(&self, encoder: &TextEncoder) -> bool {
        self.validate(encoder).is_empty()
    }
}

// ============================================================================
// CORNERMAN TEXT
// ============================================================================

/// Condition when cornerman text appears
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum CornermanCondition {
    /// Beginning of a round
    StartOfRound = 0x00,
    /// Player has low health
    PlayerLowHealth = 0x01,
    /// Opponent has low health
    OpponentLowHealth = 0x02,
    /// Player has been knocked down
    PlayerKnockedDown = 0x03,
    /// Opponent has been knocked down
    OpponentKnockedDown = 0x04,
    /// Between rounds (corner advice)
    BetweenRounds = 0x05,
    /// Time is running low
    TimeRunningOut = 0x06,
    /// Random selection from pool
    Random = 0x07,
    /// Custom condition
    Custom(u8),
}

impl CornermanCondition {
    /// Get display name for the condition
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::StartOfRound => "Start of Round",
            Self::PlayerLowHealth => "Player Low Health",
            Self::OpponentLowHealth => "Opponent Low Health",
            Self::PlayerKnockedDown => "Player Knocked Down",
            Self::OpponentKnockedDown => "Opponent Knocked Down",
            Self::BetweenRounds => "Between Rounds",
            Self::TimeRunningOut => "Time Running Out",
            Self::Random => "Random",
            Self::Custom(_) => "Custom",
        }
    }

    /// Get all standard conditions (for UI dropdowns)
    pub fn all_conditions() -> Vec<Self> {
        vec![
            Self::StartOfRound,
            Self::PlayerLowHealth,
            Self::OpponentLowHealth,
            Self::PlayerKnockedDown,
            Self::OpponentKnockedDown,
            Self::BetweenRounds,
            Self::TimeRunningOut,
            Self::Random,
        ]
    }

    /// Convert from byte value
    pub fn from_byte(byte: u8) -> Self {
        match byte {
            0x00 => Self::StartOfRound,
            0x01 => Self::PlayerLowHealth,
            0x02 => Self::OpponentLowHealth,
            0x03 => Self::PlayerKnockedDown,
            0x04 => Self::OpponentKnockedDown,
            0x05 => Self::BetweenRounds,
            0x06 => Self::TimeRunningOut,
            0x07 => Self::Random,
            n => Self::Custom(n),
        }
    }

    /// Convert to byte value
    pub fn to_byte(&self) -> u8 {
        match self {
            Self::StartOfRound => 0x00,
            Self::PlayerLowHealth => 0x01,
            Self::OpponentLowHealth => 0x02,
            Self::PlayerKnockedDown => 0x03,
            Self::OpponentKnockedDown => 0x04,
            Self::BetweenRounds => 0x05,
            Self::TimeRunningOut => 0x06,
            Self::Random => 0x07,
            Self::Custom(b) => *b,
        }
    }
}

/// Cornerman advice text entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CornermanText {
    /// Unique ID for this text entry
    pub id: u8,
    /// Which boxer this advice is for (key like "gabby_jay")
    pub boxer_key: String,
    /// Which round this applies to (0 = any round)
    pub round: u8,
    /// Condition that triggers this text
    pub condition: CornermanCondition,
    /// The actual text content
    pub text: String,
    /// Original ROM bytes (for reference)
    pub raw_bytes: Vec<u8>,
    /// PC offset in ROM where this is stored
    pub rom_offset: Option<usize>,
    /// Maximum allowed length
    pub max_length: usize,
}

impl CornermanText {
    /// Create a new cornerman text entry
    pub fn new(id: u8, boxer_key: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            id,
            boxer_key: boxer_key.into(),
            round: 0,
            condition: CornermanCondition::Random,
            text: text.into(),
            raw_bytes: Vec::new(),
            rom_offset: None,
            max_length: 40,
        }
    }

    /// Validate the text for ROM constraints
    pub fn validate(&self, encoder: &TextEncoder) -> Result<(), String> {
        let encoded = encoder.encode(&self.text);

        if encoded.len() > self.max_length {
            return Err(format!(
                "Text too long: {} bytes (max {})",
                encoded.len(),
                self.max_length
            ));
        }

        if let Err(invalid) = encoder.validate(&self.text) {
            return Err(format!("Invalid characters: {:?}", invalid));
        }

        Ok(())
    }

    /// Get the byte length when encoded
    pub fn encoded_length(&self, encoder: &TextEncoder) -> usize {
        encoder.encode(&self.text).len()
    }

    /// Check if this text will fit in ROM
    pub fn fits(&self, encoder: &TextEncoder) -> bool {
        self.encoded_length(encoder) <= self.max_length && encoder.can_encode(&self.text)
    }
}

// ============================================================================
// VICTORY QUOTE
// ============================================================================

/// Victory/defeat quote
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VictoryQuote {
    /// Unique ID
    pub id: u8,
    /// Boxer key
    pub boxer_key: String,
    /// The quote text
    pub text: String,
    /// Whether this is a loss quote (instead of victory)
    pub is_loss_quote: bool,
    /// Original ROM bytes
    pub raw_bytes: Vec<u8>,
    /// ROM offset
    pub rom_offset: Option<usize>,
    /// Maximum length
    pub max_length: usize,
}

impl VictoryQuote {
    /// Create a new victory quote
    pub fn new(id: u8, boxer_key: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            id,
            boxer_key: boxer_key.into(),
            text: text.into(),
            is_loss_quote: false,
            raw_bytes: Vec::new(),
            rom_offset: None,
            max_length: 50,
        }
    }

    /// Validate the quote for ROM constraints
    pub fn validate(&self, encoder: &TextEncoder) -> Result<(), String> {
        let encoded = encoder.encode(&self.text);

        if encoded.len() > self.max_length {
            return Err(format!(
                "Text too long: {} bytes (max {})",
                encoded.len(),
                self.max_length
            ));
        }

        if let Err(invalid) = encoder.validate(&self.text) {
            return Err(format!("Invalid characters: {:?}", invalid));
        }

        Ok(())
    }
}

// ============================================================================
// ERRORS AND VALIDATION
// ============================================================================

/// Errors that can occur when working with roster data
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
pub enum RosterError {
    #[error("Invalid fighter ID: {0}")]
    InvalidFighterId(u8),

    #[error("Name too long: {actual_bytes} bytes (max {max_bytes})")]
    NameTooLong {
        name: String,
        max_bytes: usize,
        actual_bytes: usize,
    },

    #[error("Name encoding failed: {0}")]
    EncodingError(String),

    #[error("ROM address not found: {0}")]
    AddressNotFound(String),

    #[error("Text too long: {0}")]
    TextTooLong(String),
}

/// Validation issue types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationIssue {
    DuplicateName {
        name: String,
        #[serde(rename = "boxer_ids", alias = "fighter_ids")]
        fighter_ids: Vec<u8>,
    },
    GapInUnlockOrder {
        from: u8,
        to: u8,
    },
    MissingChampionFlag {
        #[serde(rename = "boxer_id", alias = "fighter_id")]
        fighter_id: u8,
        circuit: CircuitType,
    },
    BoxerNotInAnyCircuit {
        #[serde(rename = "boxer_id", alias = "fighter_id")]
        fighter_id: u8,
        name: String,
    },
}

/// Validation report for roster data
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ValidationReport {
    pub is_valid: bool,
    pub errors: Vec<ValidationIssue>,
    pub warnings: Vec<ValidationIssue>,
}

impl ValidationReport {
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    /// Get a human-readable summary
    pub fn summary(&self) -> String {
        let mut parts = Vec::new();

        if self.errors.is_empty() && self.warnings.is_empty() {
            return "Roster data is valid.".to_string();
        }

        if !self.errors.is_empty() {
            parts.push(format!("{} error(s)", self.errors.len()));
        }
        if !self.warnings.is_empty() {
            parts.push(format!("{} warning(s)", self.warnings.len()));
        }

        parts.join(", ")
    }
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Get boxer key from ID
pub fn get_boxer_key(boxer_id: u8) -> &'static str {
    match boxer_id {
        0 => "gabby_jay",
        1 => "bear_hugger",
        2 => "piston_hurricane",
        3 => "bald_bull",
        4 => "bob_charlie",
        5 => "dragon_chan",
        6 => "masked_muscle",
        7 => "mr_sandman",
        8 => "aran_ryan",
        9 => "heike_kagero",
        10 => "mad_clown",
        11 => "super_macho_man",
        12 => "narcis_prince",
        13 => "hoy_quarlow",
        14 => "rick_bruiser",
        15 => "nick_bruiser",
        _ => "unknown",
    }
}

/// Get boxer ID from key
#[allow(dead_code)]
pub fn get_boxer_id_from_key(key: &str) -> Option<u8> {
    match key {
        "gabby_jay" => Some(0),
        "bear_hugger" => Some(1),
        "piston_hurricane" => Some(2),
        "bald_bull" => Some(3),
        "bob_charlie" => Some(4),
        "dragon_chan" => Some(5),
        "masked_muscle" => Some(6),
        "mr_sandman" => Some(7),
        "aran_ryan" => Some(8),
        "heike_kagero" => Some(9),
        "mad_clown" => Some(10),
        "super_macho_man" => Some(11),
        "narcis_prince" => Some(12),
        "hoy_quarlow" => Some(13),
        "rick_bruiser" => Some(14),
        "nick_bruiser" => Some(15),
        _ => None,
    }
}

/// Re-export TextEncoder as RosterTextEncoder for roster-specific usage
pub type RosterTextEncoder = crate::text::TextEncoder;
