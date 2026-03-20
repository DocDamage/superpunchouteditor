//! BRR Constants

/// Size of a BRR block in bytes (9 bytes)
pub const BRR_BLOCK_SIZE: usize = 9;

/// Number of samples per BRR block (16 samples)
pub const SAMPLES_PER_BLOCK: usize = 16;

/// BRR block header flags

/// Loop flag - indicates this block is a loop point
pub const BRR_FLAG_LOOP: u8 = 0x02;

/// End flag - indicates this is the last block
pub const BRR_FLAG_END: u8 = 0x01;
