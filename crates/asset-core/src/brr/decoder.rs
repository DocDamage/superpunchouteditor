//! BRR Decoder

use crate::brr::block::BrrBlockInfo;
use crate::brr::constants::{BRR_BLOCK_SIZE, BRR_FLAG_END, BRR_FLAG_LOOP, SAMPLES_PER_BLOCK};

/// BRR decoder implementation.
///
/// Decodes BRR-compressed audio data to 16-bit PCM samples.
/// The decoder maintains filter state across blocks for proper
/// reconstruction of the audio signal.
///
/// # Example
/// ```
/// use asset_core::brr::BrrDecoder;
///
/// let decoder = BrrDecoder::new();
/// let brr_data = vec![0xB0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
/// let pcm = decoder.decode(&brr_data);
/// assert_eq!(pcm.len(), 16);
/// ```
pub struct BrrDecoder;

impl BrrDecoder {
    /// Creates a new decoder.
    ///
    /// # Example
    /// ```
    /// use asset_core::brr::BrrDecoder;
    ///
    /// let decoder = BrrDecoder::new();
    /// ```
    pub fn new() -> Self {
        Self
    }

    /// Decodes BRR data to 16-bit PCM.
    ///
    /// # Arguments
    /// - `brr_data`: Raw BRR encoded bytes
    ///
    /// # Returns
    /// A vector of 16-bit PCM samples
    ///
    /// # Example
    /// ```
    /// use asset_core::brr::BrrDecoder;
    ///
    /// let decoder = BrrDecoder::new();
    /// let brr_data = vec![0xB0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    /// let pcm = decoder.decode(&brr_data);
    /// assert_eq!(pcm.len(), 16);
    /// ```
    pub fn decode(&self, brr_data: &[u8]) -> Vec<i16> {
        let mut output = Vec::new();
        let mut old = 0i16;
        let mut older = 0i16;

        // Process 9-byte blocks
        for chunk in brr_data.chunks(BRR_BLOCK_SIZE) {
            if chunk.len() < BRR_BLOCK_SIZE {
                break; // Incomplete block
            }

            let header = chunk[0];
            let range = (header >> 4) & 0x0F;
            let filter = (header >> 2) & 0x03;
            let _loop_flag = (header & BRR_FLAG_LOOP) != 0;
            let end_flag = (header & BRR_FLAG_END) != 0;

            // Decode samples
            for n in 0..SAMPLES_PER_BLOCK {
                // Get 4-bit sample
                let byte = chunk[1 + n / 2];
                let nybble = if n % 2 == 0 {
                    (byte >> 4) & 0x0F
                } else {
                    byte & 0x0F
                };

                // Sign extend to 8 bits
                let sample_4bit = if nybble & 0x08 != 0 {
                    (nybble as i8) | -16i8
                } else {
                    nybble as i8
                };

                // Apply range shift
                let mut sample = if range <= 12 {
                    (sample_4bit as i16) << range
                } else {
                    (sample_4bit as i16) << 12 // Clamp to 12 bits max
                };

                // Apply filter
                sample = apply_filter(sample, filter, old, older);

                // Clamp to 15-bit signed range (-16384 to 16383)
                sample = sample.max(-16384).min(16383);

                // Store and update history
                older = old;
                old = sample;

                output.push(sample);
            }

            if end_flag {
                break;
            }
        }

        output
    }

    /// Decodes with per-block information.
    ///
    /// Returns both the decoded samples and detailed information
    /// about each block for debugging purposes.
    ///
    /// # Arguments
    /// - `brr_data`: Raw BRR encoded bytes
    ///
    /// # Returns
    /// A tuple of (samples, block_info)
    pub fn decode_with_info(&self, brr_data: &[u8]) -> (Vec<i16>, Vec<BrrBlockInfo>) {
        let mut output = Vec::new();
        let mut block_info = Vec::new();
        let mut old = 0i16;
        let mut older = 0i16;

        for (block_idx, chunk) in brr_data.chunks(BRR_BLOCK_SIZE).enumerate() {
            if chunk.len() < BRR_BLOCK_SIZE {
                break;
            }

            let header = chunk[0];
            let range = (header >> 4) & 0x0F;
            let filter = (header >> 2) & 0x03;
            let loop_flag = (header & BRR_FLAG_LOOP) != 0;
            let end_flag = (header & BRR_FLAG_END) != 0;

            let mut block_samples = [0i16; SAMPLES_PER_BLOCK];

            for n in 0..SAMPLES_PER_BLOCK {
                let byte = chunk[1 + n / 2];
                let nybble = if n % 2 == 0 {
                    (byte >> 4) & 0x0F
                } else {
                    byte & 0x0F
                };

                let sample_4bit = if nybble & 0x08 != 0 {
                    (nybble as i8) | -16i8
                } else {
                    nybble as i8
                };

                let mut sample = if range <= 12 {
                    (sample_4bit as i16) << range
                } else {
                    (sample_4bit as i16) << 12
                };

                sample = apply_filter(sample, filter, old, older);
                sample = sample.max(-16384).min(16383);
                older = old;
                old = sample;

                block_samples[n] = sample;
                output.push(sample);
            }

            block_info.push(BrrBlockInfo {
                index: block_idx,
                range,
                filter,
                loop_flag,
                end_flag,
                samples: block_samples,
            });

            if end_flag {
                break;
            }
        }

        (output, block_info)
    }

    /// Calculates the number of PCM samples that will be produced.
    ///
    /// # Arguments
    /// - `brr_data`: Raw BRR encoded bytes
    ///
    /// # Returns
    /// The expected number of PCM samples after decoding
    pub fn calculate_output_size(&self, brr_data: &[u8]) -> usize {
        let block_count = brr_data.len() / BRR_BLOCK_SIZE;
        block_count * SAMPLES_PER_BLOCK
    }
}

impl Default for BrrDecoder {
    fn default() -> Self {
        Self::new()
    }
}

/// Applies the BRR filter to a sample.
pub(crate) fn apply_filter(sample: i16, filter: u8, old: i16, older: i16) -> i16 {
    match filter {
        0 => sample, // No filter
        1 => sample.wrapping_add(old).wrapping_sub(old >> 4),
        2 => sample
            .wrapping_add(old.wrapping_mul(2))
            .wrapping_sub(old >> 4)
            .wrapping_sub(old >> 3)
            .wrapping_sub(older)
            .wrapping_add(older >> 1),
        3 => sample
            .wrapping_add(old.wrapping_mul(2))
            .wrapping_sub(old >> 3)
            .wrapping_sub(old >> 2)
            .wrapping_sub(older)
            .wrapping_add(older >> 1),
        _ => sample,
    }
}
