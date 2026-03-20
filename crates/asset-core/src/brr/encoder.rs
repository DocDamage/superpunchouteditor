//! BRR Encoder

use crate::brr::constants::{BRR_BLOCK_SIZE, BRR_FLAG_END, BRR_FLAG_LOOP, SAMPLES_PER_BLOCK};
use crate::brr::decoder::apply_filter;

/// BRR encoder implementation.
///
/// Encodes 16-bit PCM audio data to BRR format.
/// The encoder determines optimal range and filter values
/// for each block to minimize quantization error.
///
/// # Example
/// ```
/// use asset_core::brr::{BrrEncoder, BrrEncodeOptions};
///
/// let encoder = BrrEncoder::new();
/// let pcm: Vec<i16> = (0..16).map(|i| i as i16 * 50).collect();
/// let options = BrrEncodeOptions::default();
/// let brr = encoder.encode(&pcm, options);
/// ```
pub struct BrrEncoder;

/// BRR encoding options.
///
/// Controls the encoding process including looping and quality settings.
///
/// # Fields
/// - `looped`: Enable looping
/// - `loop_start`: Block index where loop starts
/// - `sample_rate`: Target sample rate (affects playback pitch)
/// - `quality`: Quality level (1-5, higher = better but slower)
#[derive(Debug, Clone, Copy)]
pub struct BrrEncodeOptions {
    /// Enable looping
    pub looped: bool,
    /// Loop start block index
    pub loop_start: usize,
    /// Target sample rate in Hz
    pub sample_rate: u32,
    /// Quality level (1-5, higher = better but slower)
    pub quality: u8,
}

impl Default for BrrEncodeOptions {
    fn default() -> Self {
        Self {
            looped: false,
            loop_start: 0,
            sample_rate: 32000,
            quality: 3,
        }
    }
}

impl BrrEncodeOptions {
    /// Creates new encoding options with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the loop option.
    pub fn with_loop(mut self, looped: bool) -> Self {
        self.looped = looped;
        self
    }

    /// Sets the loop start block.
    pub fn with_loop_start(mut self, start: usize) -> Self {
        self.loop_start = start;
        self
    }

    /// Sets the sample rate.
    pub fn with_sample_rate(mut self, rate: u32) -> Self {
        self.sample_rate = rate;
        self
    }

    /// Sets the quality level.
    pub fn with_quality(mut self, quality: u8) -> Self {
        self.quality = quality.min(5).max(1);
        self
    }
}

impl BrrEncoder {
    /// Creates a new encoder.
    pub fn new() -> Self {
        Self
    }

    /// Encodes 16-bit PCM to BRR format with optimal filter/range selection.
    ///
    /// # Arguments
    /// - `pcm_data`: 16-bit PCM samples
    /// - `options`: Encoding options
    ///
    /// # Returns
    /// A vector of BRR encoded bytes
    pub fn encode(&self, pcm_data: &[i16], options: BrrEncodeOptions) -> Vec<u8> {
        let mut output = Vec::new();
        let mut old = 0i16;
        let mut older = 0i16;

        // Process in chunks of 16 samples
        let chunks: Vec<&[i16]> = pcm_data.chunks(SAMPLES_PER_BLOCK).collect();
        let total_chunks = chunks.len();

        for (chunk_idx, chunk) in chunks.iter().enumerate() {
            let is_last = chunk_idx == total_chunks - 1;
            let is_loop_point = options.looped && chunk_idx == options.loop_start;

            // Pad chunk to 16 samples if needed
            let mut samples = [0i16; SAMPLES_PER_BLOCK];
            for (i, &sample) in chunk.iter().enumerate().take(SAMPLES_PER_BLOCK) {
                samples[i] = sample.clamp(-16384, 16383);
            }

            // Find optimal range and filter using brute force search
            let (range, filter, encoded) =
                self.find_best_encoding(&samples, old, older, options.quality);

            // Build header byte
            let mut header = (range << 4) | (filter << 2);
            if is_loop_point {
                header |= BRR_FLAG_LOOP;
            }
            if is_last && !options.looped {
                header |= BRR_FLAG_END;
            }

            output.push(header);

            // Pack nibbles into bytes
            for n in 0..8 {
                let s1 = (encoded[n * 2] & 0x0F) as u8;
                let s2 = (encoded[n * 2 + 1] & 0x0F) as u8;
                output.push((s1 << 4) | s2);

                // Update filter history for next encoding
                let decoded1 = decode_sample(s1, range, filter, old, older);
                older = old;
                old = decoded1;

                let decoded2 = decode_sample(s2, range, filter, old, older);
                older = old;
                old = decoded2;
            }
        }

        output
    }

    /// Finds the best range and filter for a block by trying all combinations.
    fn find_best_encoding(
        &self,
        samples: &[i16; 16],
        old: i16,
        older: i16,
        _quality: u8,
    ) -> (u8, u8, [i8; 16]) {
        let mut best_range = 0u8;
        let mut best_filter = 0u8;
        let mut best_error = i64::MAX;
        let mut best_encoded = [0i8; 16];

        // Try all filter and range combinations
        for filter in 0u8..=3 {
            for range in 0u8..=12 {
                let (encoded, error) = self.try_encoding(samples, old, older, range, filter);
                if error < best_error {
                    best_error = error;
                    best_range = range;
                    best_filter = filter;
                    best_encoded = encoded;
                }
            }
        }

        (best_range, best_filter, best_encoded)
    }

    /// Tries encoding a block with specific range and filter, returning encoded samples and error.
    fn try_encoding(
        &self,
        samples: &[i16; 16],
        initial_old: i16,
        initial_older: i16,
        range: u8,
        filter: u8,
    ) -> ([i8; 16], i64) {
        let mut encoded = [0i8; 16];
        let mut old = initial_old;
        let mut older = initial_older;
        let mut total_error: i64 = 0;

        for (i, &target) in samples.iter().enumerate() {
            // Reverse the filter to get the unfiltered target
            let unfiltered = match filter {
                0 => target,
                1 => target.wrapping_sub(old).wrapping_add(old >> 4),
                2 => {
                    let predicted = old
                        .wrapping_mul(2)
                        .wrapping_sub(old >> 4)
                        .wrapping_sub(old >> 3)
                        .wrapping_sub(older)
                        .wrapping_add(older >> 1);
                    target.wrapping_sub(predicted)
                }
                3 => {
                    let predicted = old
                        .wrapping_mul(2)
                        .wrapping_sub(old >> 3)
                        .wrapping_sub(old >> 2)
                        .wrapping_sub(older)
                        .wrapping_add(older >> 1);
                    target.wrapping_sub(predicted)
                }
                _ => target,
            };

            // Apply range shift and quantize to 4 bits
            let shifted = if range <= 12 {
                unfiltered >> range
            } else {
                unfiltered >> 12
            };

            // Clamp to 4-bit signed range (-8 to 7)
            let clamped = shifted.max(-8).min(7);
            encoded[i] = clamped as i8;

            // Calculate error by decoding and comparing
            let decoded = decode_sample((clamped & 0x0F) as u8, range, filter, old, older);
            let error = (target as i32 - decoded as i32).abs() as i64;
            total_error += error;

            // Update filter history
            older = old;
            old = decoded;
        }

        (encoded, total_error)
    }

    /// Calculates the BRR size for a given PCM size.
    ///
    /// # Arguments
    /// - `pcm_sample_count`: Number of PCM samples
    ///
    /// # Returns
    /// The size of the BRR data in bytes
    pub fn calculate_brr_size(&self, pcm_sample_count: usize) -> usize {
        let block_count = (pcm_sample_count + SAMPLES_PER_BLOCK - 1) / SAMPLES_PER_BLOCK;
        block_count * BRR_BLOCK_SIZE
    }
}

impl Default for BrrEncoder {
    fn default() -> Self {
        Self::new()
    }
}

/// Decodes a sample (for filter history update during encoding).
pub(crate) fn decode_sample(nybble: u8, range: u8, filter: u8, old: i16, older: i16) -> i16 {
    let sample_4bit = if nybble & 0x08 != 0 {
        (nybble as i8) | -16i8
    } else {
        nybble as i8
    };

    let sample = if range <= 12 {
        (sample_4bit as i16) << range
    } else {
        (sample_4bit as i16) << 12
    };

    apply_filter(sample, filter, old, older)
        .max(-16384)
        .min(16383)
}
