//! Audio format and playback configuration types.

use serde::{Deserialize, Serialize};

/// Audio format for import/export.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AudioFormat {
    /// Wave format (PCM)
    Wav,
    /// BRR format (native SNES)
    Brr,
    /// SPC700 save state
    Spc,
    /// Nintendo SSEQ (if applicable)
    Sseq,
    /// VGM format
    Vgm,
}

impl AudioFormat {
    /// Detects the format from a file extension.
    ///
    /// # Arguments
    /// - `ext`: File extension (without dot)
    ///
    /// # Returns
    /// `Some(AudioFormat)` if recognized, `None` otherwise
    ///
    /// # Example
    /// ```
    /// use asset_core::audio::AudioFormat;
    ///
    /// assert_eq!(AudioFormat::from_extension("wav"), Some(AudioFormat::Wav));
    /// assert_eq!(AudioFormat::from_extension("brr"), Some(AudioFormat::Brr));
    /// assert_eq!(AudioFormat::from_extension("unknown"), None);
    /// ```
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "wav" => Some(AudioFormat::Wav),
            "brr" => Some(AudioFormat::Brr),
            "spc" => Some(AudioFormat::Spc),
            "sseq" => Some(AudioFormat::Sseq),
            "vgm" => Some(AudioFormat::Vgm),
            _ => None,
        }
    }

    /// Returns the file extension for this format.
    pub fn extension(&self) -> &'static str {
        match self {
            AudioFormat::Wav => "wav",
            AudioFormat::Brr => "brr",
            AudioFormat::Spc => "spc",
            AudioFormat::Sseq => "sseq",
            AudioFormat::Vgm => "vgm",
        }
    }
}

/// Audio playback state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackState {
    Stopped,
    Playing,
    Paused,
}

/// Audio preview configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewConfig {
    /// Output sample rate
    pub sample_rate: u32,
    /// Output channels (1 or 2)
    pub channels: u8,
    /// Bits per sample (8, 16, or 32)
    pub bits_per_sample: u8,
    /// Buffer size in samples
    pub buffer_size: usize,
}

impl Default for PreviewConfig {
    fn default() -> Self {
        Self {
            sample_rate: 48000,
            channels: 2,
            bits_per_sample: 16,
            buffer_size: 1024,
        }
    }
}
