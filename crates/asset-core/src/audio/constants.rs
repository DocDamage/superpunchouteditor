//! Audio constants for SPC700 emulation.

/// Size of SPC700 RAM in bytes (64KB)
pub const SPC_RAM_SIZE: usize = 65536;

/// Size of DSP register space (128 bytes)
pub const DSP_REGISTER_SIZE: usize = 128;

/// Number of audio channels
pub const CHANNEL_COUNT: usize = 8;

/// Default stack pointer value
pub const DEFAULT_STACK_POINTER: u8 = 0xEF;

/// Default SPC700 sample rate
pub const SPC_SAMPLE_RATE: u32 = 32040;

/// WAV file constants
pub const WAV_RIFF_MAGIC: &[u8] = b"RIFF";
pub const WAV_WAVE_MAGIC: &[u8] = b"WAVE";
pub const WAV_FMT_MAGIC: &[u8] = b"fmt ";
pub const WAV_DATA_MAGIC: &[u8] = b"data";

/// Known sound effects in Super Punch-Out!!
///
/// These are preliminary IDs based on typical SNES fighting game patterns.
/// Actual IDs need to be verified through ROM analysis.
pub const KNOWN_SOUNDS: &[(&str, u8, &str)] = &[
    ("Punch Hit 1", 0x01, "combat"),
    ("Punch Hit 2", 0x02, "combat"),
    ("Block Sound", 0x03, "combat"),
    ("Dodge/Whiff", 0x04, "combat"),
    ("Uppercut Swing", 0x05, "combat"),
    ("Body Blow", 0x06, "combat"),
    ("Knockdown Hit", 0x07, "combat"),
    ("Star Punch", 0x08, "special"),
    ("KO Bell", 0x09, "match"),
    ("Round Start", 0x0A, "match"),
    ("Round End", 0x0B, "match"),
    ("Victory Music", 0x0C, "music"),
    ("Defeat Music", 0x0D, "music"),
    ("Crowd Cheer", 0x10, "ambient"),
    ("Crowd Boo", 0x11, "ambient"),
    ("Referee Count", 0x12, "voice"),
    ("Referee KO", 0x13, "voice"),
    ("Referee TKO", 0x14, "voice"),
    ("Referee Decision", 0x15, "voice"),
    ("Menu Select", 0x20, "ui"),
    ("Menu Confirm", 0x21, "ui"),
    ("Menu Back", 0x22, "ui"),
    ("Pause", 0x23, "ui"),
];

/// Known music tracks in Super Punch-Out!!
pub const KNOWN_MUSIC: &[(&str, u8, u8, &str)] = &[
    ("Title Screen", 0x01, 120, "title"),
    ("Character Select", 0x02, 110, "menu"),
    ("Match Start", 0x03, 140, "match"),
    ("Gabby Jay Theme", 0x10, 125, "boxer"),
    ("Bear Hugger Theme", 0x11, 120, "boxer"),
    ("Piston Hurricane Theme", 0x12, 130, "boxer"),
    ("Bald Bull Theme", 0x13, 128, "boxer"),
    ("Bob Charlie Theme", 0x14, 125, "boxer"),
    ("Dragon Chan Theme", 0x15, 135, "boxer"),
    ("Mr. Sandman Theme", 0x16, 140, "boxer"),
    ("Aran Ryan Theme", 0x17, 130, "boxer"),
    ("Heike Kagero Theme", 0x18, 125, "boxer"),
    ("Mad Clown Theme", 0x19, 132, "boxer"),
    ("Super Macho Man Theme", 0x1A, 138, "boxer"),
    ("Narcis Prince Theme", 0x1B, 126, "boxer"),
    ("Hoy Quarlow Theme", 0x1C, 124, "boxer"),
    ("Rick Bruiser Theme", 0x1D, 145, "boxer"),
    ("Nick Bruiser Theme", 0x1E, 150, "boxer"),
    ("Training Mode", 0x20, 115, "training"),
    ("Intermission", 0x21, 100, "menu"),
    ("Victory Music", 0x30, 130, "match"),
    ("Defeat Music", 0x31, 90, "match"),
    ("Credits", 0x40, 120, "ending"),
];
