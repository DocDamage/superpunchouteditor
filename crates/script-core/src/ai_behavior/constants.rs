// =============================================================================
// ROM ADDRESS CONSTANTS
// =============================================================================

/// AI Table Base Address (Bank $0B:8000)
/// Contains fighter AI headers and configuration
pub const AI_TABLE_BASE: usize = 0x058000; // Bank $0B:8000 in PC offset

/// AI Pattern Table (Bank $0B:8200)
/// Contains attack pattern sequences
pub const AI_PATTERN_TABLE: usize = 0x058200;

/// AI Defense Table (Bank $0B:8800)
/// Contains defense behavior configurations
pub const AI_DEFENSE_TABLE: usize = 0x058800;

/// AI Trigger Table (Bank $0B:9000)
/// Contains condition-action triggers
pub const AI_TRIGGER_TABLE: usize = 0x059000;

/// Fighter Header Table Base (Bank $09:8000)
pub const FIGHTER_HEADER_BASE: usize = 0x048000;

/// Size of each fighter header entry
pub const FIGHTER_HEADER_SIZE: usize = 0x20; // 32 bytes

/// Maximum number of fighters in the roster
pub const MAX_FIGHTERS: usize = 16;

/// Maximum patterns per fighter
pub const MAX_PATTERNS_PER_FIGHTER: usize = 16;

/// Maximum defense behaviors per fighter
pub const MAX_DEFENSE_PER_FIGHTER: usize = 8;

/// Maximum triggers per fighter
pub const MAX_TRIGGERS_PER_FIGHTER: usize = 8;

/// Fighter roster names (indexed by fighter_id)
pub const FIGHTER_NAMES: &[&str] = &[
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
