//! BRR Block Information

use crate::brr::constants::SAMPLES_PER_BLOCK;

/// Decoded BRR block information.
///
/// Contains metadata about a decoded BRR block for debugging
/// and analysis purposes.
///
/// # Fields
/// - `index`: Block index in the stream
/// - `range`: Range shift value (0-12)
/// - `filter`: Filter type (0-3)
/// - `loop_flag`: Whether loop flag was set
/// - `end_flag`: Whether end flag was set
/// - `samples`: The 16 decoded samples
#[derive(Debug, Clone)]
pub struct BrrBlockInfo {
    /// Block index in the stream
    pub index: usize,
    /// Range shift value (0-12)
    pub range: u8,
    /// Filter type (0-3)
    pub filter: u8,
    /// Loop flag set
    pub loop_flag: bool,
    /// End flag set
    pub end_flag: bool,
    /// Decoded samples (16 values)
    pub samples: [i16; SAMPLES_PER_BLOCK],
}
