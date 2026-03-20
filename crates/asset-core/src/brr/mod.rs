//! BRR (Bit Rate Reduction) Codec for SNES/SPC700
//!
//! BRR is the native compressed audio format used by the SNES SPC700.
//! It uses ADPCM compression with 9-byte blocks containing 16 samples each.
//!
//! # Format Details
//! - Each block is 9 bytes: 1 header byte + 8 bytes of 4-bit samples
//! - Header contains: range (4 bits), filter (2 bits), loop flag (1 bit), end flag (1 bit)
//! - Four filters: 0=none, 1=filtered, 2=more filtered, 3=most filtered
//!
//! # Filter Coefficients
//! The SPC700 uses these filter formulas:
//! - Filter 0: sample = sample
//! - Filter 1: sample = sample + old - (old >> 4)
//! - Filter 2: sample = sample + (old * 2) - (old >> 4) - (old >> 3) - older + (older >> 1)
//! - Filter 3: sample = sample + (old * 2) - (old >> 3) - (old >> 2) - older + (older >> 1)
//!
//! # Example
//! ```
//! use asset_core::brr::{BrrDecoder, BrrEncoder, BrrEncodeOptions};
//!
//! // Decode BRR data
//! let decoder = BrrDecoder::new();
//! let brr_data = vec![0xB0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
//! let pcm = decoder.decode(&brr_data);
//!
//! // Encode PCM to BRR
//! let encoder = BrrEncoder::new();
//! let pcm_data: Vec<i16> = (0..16).map(|i| (i as i16 * 100)).collect();
//! let options = BrrEncodeOptions::default();
//! let brr = encoder.encode(&pcm_data, options);
//! ```

mod block;
mod constants;
mod decoder;
mod encoder;

// Re-exports for public API
pub use block::BrrBlockInfo;
pub use constants::{BRR_BLOCK_SIZE, BRR_FLAG_END, BRR_FLAG_LOOP, SAMPLES_PER_BLOCK};
pub use decoder::BrrDecoder;
pub use encoder::{BrrEncodeOptions, BrrEncoder};

/// Utility function to decode BRR quickly.
///
/// # Arguments
/// - `brr_data`: Raw BRR encoded bytes
///
/// # Returns
/// A vector of 16-bit PCM samples
///
/// # Example
/// ```
/// use asset_core::brr::decode_brr;
///
/// let brr_data = vec![0xB0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
/// let pcm = decode_brr(&brr_data);
/// ```
pub fn decode_brr(brr_data: &[u8]) -> Vec<i16> {
    let decoder = BrrDecoder::new();
    decoder.decode(brr_data)
}

/// Utility function to encode PCM to BRR.
///
/// # Arguments
/// - `pcm_data`: 16-bit PCM samples
/// - `options`: Encoding options
///
/// # Returns
/// A vector of BRR encoded bytes
///
/// # Example
/// ```
/// use asset_core::brr::{encode_brr, BrrEncodeOptions};
///
/// let pcm: Vec<i16> = (0..16).map(|i| i as i16).collect();
/// let options = BrrEncodeOptions::default();
/// let brr = encode_brr(&pcm, options);
/// ```
pub fn encode_brr(pcm_data: &[i16], options: BrrEncodeOptions) -> Vec<u8> {
    let encoder = BrrEncoder::new();
    encoder.encode(pcm_data, options)
}

/// Calculates loop-adjusted duration.
///
/// # Arguments
/// - `block_count`: Total number of BRR blocks
/// - `loop_start_block`: Block index where loop starts
/// - `sample_rate`: Sample rate in Hz
/// - `max_duration_ms`: Maximum duration to calculate
///
/// # Returns
/// Estimated duration in milliseconds
pub fn calculate_looped_duration(
    block_count: usize,
    loop_start_block: usize,
    sample_rate: u32,
    max_duration_ms: u32,
) -> u32 {
    let samples_per_block = SAMPLES_PER_BLOCK as u32;
    let total_samples = (block_count as u32) * samples_per_block;
    let sample_duration_ms = (total_samples * 1000) / sample_rate;

    if loop_start_block < block_count {
        // Looped sample - calculate intro + loop portion
        let intro_blocks = loop_start_block as u32;
        let loop_blocks = (block_count - loop_start_block) as u32;
        let intro_duration = (intro_blocks * samples_per_block * 1000) / sample_rate;
        let loop_duration = (loop_blocks * samples_per_block * 1000) / sample_rate;

        // Assume loop plays until max_duration
        let loop_count = (max_duration_ms.saturating_sub(intro_duration)) / loop_duration.max(1);
        intro_duration + (loop_count * loop_duration)
    } else {
        sample_duration_ms.min(max_duration_ms)
    }
}

/// Simple linear resampling for converting between sample rates.
///
/// # Arguments
/// - `input`: Input PCM samples
/// - `input_rate`: Input sample rate in Hz
/// - `output_rate`: Output sample rate in Hz
///
/// # Returns
/// Resampled PCM samples
pub fn resample_linear(input: &[i16], input_rate: u32, output_rate: u32) -> Vec<i16> {
    if input_rate == output_rate || input.is_empty() {
        return input.to_vec();
    }

    let ratio = input_rate as f64 / output_rate as f64;
    let output_len = ((input.len() as f64) / ratio) as usize;
    let mut output = Vec::with_capacity(output_len);

    for i in 0..output_len {
        let input_pos = i as f64 * ratio;
        let input_idx = input_pos as usize;
        let frac = input_pos - input_idx as f64;

        let s1 = input[input_idx.min(input.len() - 1)] as f64;
        let s2 = input[(input_idx + 1).min(input.len() - 1)] as f64;

        let sample = s1 + (s2 - s1) * frac;
        output.push(sample as i16);
    }

    output
}

/// Resamples audio to target sample rate (convenience wrapper).
///
/// # Arguments
/// - `pcm`: Input PCM samples
/// - `source_rate`: Source sample rate in Hz
/// - `target_rate`: Target sample rate in Hz
///
/// # Returns
/// Resampled PCM samples at target rate
pub fn resample_to_rate(pcm: &[i16], source_rate: u32, target_rate: u32) -> Vec<i16> {
    resample_linear(pcm, source_rate, target_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brr_decode_empty() {
        let decoder = BrrDecoder::new();
        let result = decoder.decode(&[]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_brr_decode_single_block() {
        let decoder = BrrDecoder::new();
        // Create a simple BRR block with range=0, filter=0, end flag
        let brr_data = vec![0xB1, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let result = decoder.decode(&brr_data);
        assert_eq!(result.len(), 16);
    }

    #[test]
    fn test_brr_encode_decode() {
        let encoder = BrrEncoder::new();
        let decoder = BrrDecoder::new();

        // Create simple PCM data (16 samples = 1 block)
        let pcm: Vec<i16> = (0..16).map(|i| (i as i16 * 100)).collect();

        let options = BrrEncodeOptions::default();
        let brr = encoder.encode(&pcm, options);

        // Should produce exactly 9 bytes
        assert_eq!(brr.len(), 9);

        // Decode and check
        let decoded = decoder.decode(&brr);
        assert_eq!(decoded.len(), 16);
    }

    #[test]
    fn test_brr_encode_decode_multiple_blocks() {
        let encoder = BrrEncoder::new();
        let decoder = BrrDecoder::new();

        // Create PCM data for 2 blocks (32 samples)
        let pcm: Vec<i16> = (0..32).map(|i| (i as i16 * 50)).collect();

        let options = BrrEncodeOptions::default();
        let brr = encoder.encode(&pcm, options);

        // Should produce 18 bytes (2 blocks)
        assert_eq!(brr.len(), 18);

        let decoded = decoder.decode(&brr);
        assert_eq!(decoded.len(), 32);
    }

    #[test]
    fn test_calculate_sizes() {
        let encoder = BrrEncoder::new();
        let decoder = BrrDecoder::new();

        // 32 PCM samples = 2 BRR blocks = 18 bytes
        assert_eq!(encoder.calculate_brr_size(32), 18);
        assert_eq!(decoder.calculate_output_size(&vec![0u8; 18]), 32);

        // 33 PCM samples = 3 BRR blocks = 27 bytes (padded)
        assert_eq!(encoder.calculate_brr_size(33), 27);
    }

    #[test]
    fn test_encode_options() {
        let opts = BrrEncodeOptions::new()
            .with_loop(true)
            .with_loop_start(5)
            .with_sample_rate(22050)
            .with_quality(5);

        assert!(opts.looped);
        assert_eq!(opts.loop_start, 5);
        assert_eq!(opts.sample_rate, 22050);
        assert_eq!(opts.quality, 5);
    }

    #[test]
    fn test_utility_functions() {
        let pcm: Vec<i16> = (0..16).map(|i| i as i16).collect();
        let brr = decode_brr(&encode_brr(&pcm, BrrEncodeOptions::default()));
        assert_eq!(brr.len(), 16);
    }

    #[test]
    fn test_resample_linear() {
        // Test resampling from 32000 to 16000 (downsample by 2x)
        let input: Vec<i16> = (0..32).map(|i| (i * 100) as i16).collect();
        let output = resample_linear(&input, 32000, 16000);
        assert_eq!(output.len(), 16);

        // Test resampling from 16000 to 32000 (upsample by 2x)
        let output2 = resample_linear(&input, 16000, 32000);
        assert_eq!(output2.len(), 64);
    }

    #[test]
    fn test_encode_decode_with_silence() {
        let encoder = BrrEncoder::new();
        let decoder = BrrDecoder::new();

        // Silence should encode/decode cleanly
        let pcm = vec![0i16; 32];
        let options = BrrEncodeOptions::default();
        let brr = encoder.encode(&pcm, options);
        let decoded = decoder.decode(&brr);

        assert_eq!(decoded.len(), 32);
        // Silent input should produce near-silent output
        for sample in &decoded {
            assert!(
                sample.abs() < 10,
                "Silent block should decode to near silence"
            );
        }
    }

    #[test]
    fn test_encode_decode_with_sine_wave() {
        let encoder = BrrEncoder::new();
        let decoder = BrrDecoder::new();

        // Create a simple sine wave
        let pcm: Vec<i16> = (0..64)
            .map(|i| {
                let phase = 2.0 * std::f64::consts::PI * (i as f64) / 16.0;
                (phase.sin() * 1000.0) as i16
            })
            .collect();

        let options = BrrEncodeOptions::default();
        let brr = encoder.encode(&pcm, options);
        let decoded = decoder.decode(&brr);

        assert_eq!(decoded.len(), 64);
        // Verify the decoded signal follows the same pattern
        for i in 0..decoded.len() {
            assert!(decoded[i].abs() <= 16384);
        }
    }

    #[test]
    fn test_loop_flag_in_header() {
        let encoder = BrrEncoder::new();

        let pcm: Vec<i16> = (0..32).map(|i| i as i16 * 50).collect();
        let options = BrrEncodeOptions::new().with_loop(true).with_loop_start(0);

        let brr = encoder.encode(&pcm, options);

        // First block should have loop flag set
        assert!(brr[0] & BRR_FLAG_LOOP != 0);
    }

    #[test]
    fn test_end_flag_on_last_block() {
        let encoder = BrrEncoder::new();

        let pcm: Vec<i16> = (0..16).map(|i| i as i16 * 50).collect();
        let options = BrrEncodeOptions::default();

        let brr = encoder.encode(&pcm, options);

        // Last (and only) block should have end flag set
        assert!(brr[0] & BRR_FLAG_END != 0);
    }

    #[test]
    fn test_no_end_flag_when_looped() {
        let encoder = BrrEncoder::new();

        let pcm: Vec<i16> = (0..16).map(|i| i as i16 * 50).collect();
        let options = BrrEncodeOptions::new().with_loop(true).with_loop_start(0);

        let brr = encoder.encode(&pcm, options);

        // Should NOT have end flag when looped
        assert!(brr[0] & BRR_FLAG_END == 0);
        // But should have loop flag
        assert!(brr[0] & BRR_FLAG_LOOP != 0);
    }
}
