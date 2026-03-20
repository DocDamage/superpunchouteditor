//! SPC700 Audio Data Structures for Super Punch-Out!!
//!
//! This module provides data structures and utilities for working with SPC700
//! audio data, including sound effects, music sequences, and BRR samples.
//!
//! The SPC700 is a dedicated 8-bit audio coprocessor used in the SNES with:
//! - 64KB of dedicated RAM
//! - 8 audio channels with hardware mixing
//! - BRR (Bit Rate Reduction) compressed sample playback
//! - Built-in DSP for echo, pitch modulation, and noise generation
//!
//! ## SPC700 Architecture
//! - CPU: Sony SPC700 (8-bit, custom instruction set)
//! - RAM: 64KB shared between code and samples
//! - DSP: Digital signal processor with 128 registers
//! - Output: 16-bit stereo at 32kHz
//!
//! ## Example
//! ```
//! use asset_core::audio::{Spc700Data, Sample, TrackType};
//!
//! // Create default SPC700 state
//! let spc = Spc700Data::new();
//!
//! // Get a sound entry by ID
//! if let Some(sound) = Spc700Data::get_sound_entry(0x01) {
//!     println!("Sound: {}", sound.name);
//! }
//! ```

// Submodules
mod constants;
mod format;
mod sample;
mod sequence;
mod spc700;
mod wav;

// Re-exports from constants
pub use constants::{
    CHANNEL_COUNT, DEFAULT_STACK_POINTER, DSP_REGISTER_SIZE, KNOWN_MUSIC, KNOWN_SOUNDS,
    SPC_RAM_SIZE, SPC_SAMPLE_RATE, WAV_DATA_MAGIC, WAV_FMT_MAGIC, WAV_RIFF_MAGIC, WAV_WAVE_MAGIC,
};

// Re-exports from format
pub use format::{AudioFormat, PlaybackState, PreviewConfig};

// Re-exports from sample
pub use sample::{AdsrEnvelope, Sample, TrackType};

// Re-exports from sequence
pub use sequence::{Channel, MusicEntry, Note, NoteEffect, Sequence, SoundEntry};

// Re-exports from spc700
pub use spc700::Spc700Data;

// Re-exports from wav
pub use wav::{
    export_brr_to_wav, import_wav_to_brr, read_wav_file, resample_linear, resample_to_rate,
    write_wav_file, write_wav_stereo, WavError,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spc700_data_default() {
        let data = Spc700Data::default();
        assert_eq!(data.ram.len(), SPC_RAM_SIZE);
        assert_eq!(data.dsp_registers.len(), DSP_REGISTER_SIZE);
        assert_eq!(data.sp, DEFAULT_STACK_POINTER);
    }

    #[test]
    fn test_track_type_display() {
        assert_eq!(TrackType::Music.as_str(), "Music");
        assert_eq!(TrackType::SoundEffect.as_str(), "SFX");
        assert_eq!(TrackType::Music.display_name(), "Background Music");
    }

    #[test]
    fn test_audio_format_from_extension() {
        assert_eq!(AudioFormat::from_extension("wav"), Some(AudioFormat::Wav));
        assert_eq!(AudioFormat::from_extension("brr"), Some(AudioFormat::Brr));
        assert_eq!(AudioFormat::from_extension("spc"), Some(AudioFormat::Spc));
        assert_eq!(AudioFormat::from_extension("unknown"), None);
    }

    #[test]
    fn test_dsp_registers() {
        let mut spc = Spc700Data::new();
        spc.set_dsp_reg(0, 100);
        assert_eq!(spc.get_dsp_reg(0), 100);

        // Out of bounds returns 0
        assert_eq!(spc.get_dsp_reg(200), 0);
    }

    #[test]
    fn test_channel_volume() {
        let mut spc = Spc700Data::new();
        spc.set_channel_volume(0, 80, 60);
        let (left, right) = spc.get_channel_volume(0);
        assert_eq!(left, 80);
        assert_eq!(right, 60);
    }

    #[test]
    fn test_default_preview_config() {
        let config = PreviewConfig::default();
        assert_eq!(config.sample_rate, 48000);
        assert_eq!(config.channels, 2);
    }

    #[test]
    fn test_sample_from_pcm() {
        use crate::brr::BrrEncodeOptions;

        let pcm: Vec<i16> = (0..32)
            .map(|i| (i as i16 * 100).clamp(-16384, 16383))
            .collect();
        let options = BrrEncodeOptions::default();
        let sample = Sample::from_pcm(1, "Test Sample", &pcm, options);

        assert_eq!(sample.id, 1);
        assert_eq!(sample.name, "Test Sample");
        assert!(!sample.brr_data.is_empty());
    }

    #[test]
    fn test_sample_to_pcm() {
        use crate::brr::BrrEncodeOptions;

        let pcm: Vec<i16> = (0..32)
            .map(|i| (i as i16 * 50).clamp(-16384, 16383))
            .collect();
        let options = BrrEncodeOptions::default();
        let sample = Sample::from_pcm(1, "Test", &pcm, options);

        let decoded = sample.to_pcm();
        assert_eq!(decoded.len(), pcm.len());
    }

    #[test]
    fn test_wav_roundtrip() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        // Create test PCM data
        let original_pcm: Vec<i16> = (0..100)
            .map(|i| ((i as i16 * 200) - 10000).clamp(-16384, 16383))
            .collect();

        // Write to temp WAV file
        let mut temp = NamedTempFile::new().unwrap();
        write_wav_file(temp.path(), &original_pcm, 32040).unwrap();

        // Read it back
        let (read_pcm, rate) = read_wav_file(temp.path()).unwrap();

        assert_eq!(rate, 32040);
        assert_eq!(read_pcm.len(), original_pcm.len());
        // Allow for small rounding differences
        for (orig, read) in original_pcm.iter().zip(read_pcm.iter()) {
            assert!(
                (orig - read).abs() < 2,
                "Samples should match within tolerance"
            );
        }
    }

    #[test]
    fn test_resample() {
        let input: Vec<i16> = (0..100)
            .map(|i| ((i as i16 - 50) * 100).clamp(-16384, 16383))
            .collect();

        // Upsample
        let upsampled = resample_linear(&input, 16000, 32000);
        assert_eq!(upsampled.len(), 200);

        // Downsample
        let downsampled = resample_linear(&input, 32000, 16000);
        assert_eq!(downsampled.len(), 50);
    }
}
