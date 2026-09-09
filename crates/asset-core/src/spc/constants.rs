/// SPC file header signature
pub const SPC_SIGNATURE: &[u8] = b"SNES-SPC700 Sound File Data v0.30";

/// Standard SPC file header size (256 bytes)
pub const SPC_HEADER_SIZE: usize = 0x100;

/// SPC RAM offset in file
pub const SPC_RAM_OFFSET: usize = 0x100;

/// SPC DSP registers offset in file
pub const SPC_DSP_OFFSET: usize = 0x10100;

/// SPC extra RAM offset
pub const SPC_EXTRA_RAM_OFFSET: usize = 0x10180;

/// Total SPC file size
pub const SPC_FILE_SIZE: usize = 0x10200;
