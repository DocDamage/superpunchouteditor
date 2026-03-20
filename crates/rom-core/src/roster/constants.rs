// ============================================================================
// ROM ADDRESS CONSTANTS
// ============================================================================

/// Bank $0C base address in PC offset (0x0C * 0x8000 = 0x60000)
pub const BANK_0C_BASE: usize = 0x060000;

/// Boxer name data table (Bank $0C:8000)
/// This is where the actual name strings are stored
pub const BOXER_NAME_TABLE: usize = 0x060000; // PC: 0x60000 = SNES: $0C:8000

/// Boxer name pointer table (Bank $0C:8100)
/// Each entry is a 2-byte pointer to the name string
pub const BOXER_NAME_POINTERS: usize = 0x060100; // PC: 0x60100 = SNES: $0C:8100

/// Circuit assignment table (Bank $0C:8200)
/// One byte per boxer: 0=Minor, 1=Major, 2=World, 3=Special
pub const CIRCUIT_TABLE: usize = 0x060200; // PC: 0x60200 = SNES: $0C:8200

/// Unlock order table (Bank $0C:8300)
/// One byte per boxer: unlock sequence number (0 = already unlocked)
pub const UNLOCK_ORDER_TABLE: usize = 0x060300; // PC: 0x60300 = SNES: $0C:8300

/// Boxer intro data table (Bank $0C:8400)
/// Contains name, origin, record, rank, quote for each boxer
/// Each field is 16 bytes, 5 fields per boxer = 80 bytes per boxer
pub const BOXER_INTRO_TABLE: usize = 0x060400; // PC: 0x60400 = SNES: $0C:8400

/// Victory quote table (Bank $0C:A000)
/// Pointers to victory/defeat quotes for each boxer
pub const VICTORY_QUOTE_TABLE: usize = 0x062000; // PC: 0x62000 = SNES: $0C:A000

/// Cornerman text pointer table (Bank $0C:B000)
/// Pointers to cornerman advice text blocks
pub const CORNERMAN_POINTER_TABLE: usize = 0x063000; // PC: 0x63000 = SNES: $0C:B000

/// Cornerman text data (Bank $0C:B200)
/// Actual cornerman text strings
pub const CORNERMAN_TEXT_DATA: usize = 0x063200; // PC: 0x63200 = SNES: $0C:B200

// ============================================================================
// SIZE CONSTANTS
// ============================================================================

/// Number of boxers in the game
pub const BOXER_COUNT: usize = 16;

/// Maximum length for boxer names (in bytes)
pub const MAX_NAME_LENGTH: usize = 16;

/// Maximum length for intro text fields (in bytes)
pub const MAX_INTRO_TEXT_LENGTH: usize = 256;

/// Size of each intro field (name, origin, record, rank, quote)
pub const INTRO_FIELD_SIZE: usize = 16;

/// Number of intro fields per boxer
pub const INTRO_FIELD_COUNT: usize = 5;

/// Size of boxer intro data block per boxer
pub const INTRO_DATA_SIZE: usize = INTRO_FIELD_SIZE * INTRO_FIELD_COUNT; // 80 bytes
