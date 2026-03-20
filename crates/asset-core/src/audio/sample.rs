//! BRR sample data structures and track types.

use crate::brr::{BrrDecoder, BrrEncodeOptions, BrrEncoder, BRR_BLOCK_SIZE, SAMPLES_PER_BLOCK};
use serde::{Deserialize, Serialize};



/// A BRR (Bit Rate Reduction) audio sample.
///
/// BRR is the native compressed format used by the SPC700.
/// Each block is 9 bytes: 1 header byte + 16 4-bit samples
///
/// # Fields
/// - `id`: Sample ID/index in the sample table
/// - `name`: Human-readable name
/// - `start_addr`: Start address in SPC700 RAM
/// - `loop_addr`: Loop start address (if looping)
/// - `sample_rate`: Playback sample rate in Hz
/// - `brr_data`: Raw BRR encoded data
/// - `adsr`: ADSR envelope settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sample {
    /// Sample ID/index in the sample table
    pub id: u8,

    /// Human-readable name (if known)
    pub name: String,

    /// Start address in SPC700 RAM
    pub start_addr: u16,

    /// Loop start address (if looping)
    pub loop_addr: Option<u16>,

    /// Number of times to loop (None = infinite)
    pub loop_count: Option<u16>,

    /// Sample rate in Hz (calculated from pitch registers)
    pub sample_rate: u32,

    /// Raw BRR encoded data
    #[serde(with = "serde_bytes")]
    pub brr_data: Vec<u8>,

    /// Duration in milliseconds
    pub duration_ms: u32,

    /// Source pitch (MIDI note number)
    pub source_note: u8,

    /// ADSR envelope settings (Attack, Decay, Sustain, Release)
    pub adsr: AdsrEnvelope,

    /// Global volume (0-127)
    pub volume: u8,

    /// Whether sample uses noise instead of BRR
    pub use_noise: bool,
}

impl Sample {
    /// Decodes this sample's BRR data to PCM.
    ///
    /// # Returns
    /// Decoded 16-bit PCM samples
    pub fn to_pcm(&self) -> Vec<i16> {
        let decoder = BrrDecoder::new();
        decoder.decode(&self.brr_data)
    }

    /// Encodes PCM data and creates a new sample.
    ///
    /// # Arguments
    /// - `id`: Sample ID
    /// - `name`: Sample name
    /// - `pcm`: PCM samples to encode
    /// - `options`: BRR encoding options
    ///
    /// # Returns
    /// A new Sample with encoded BRR data
    pub fn from_pcm(id: u8, name: &str, pcm: &[i16], options: BrrEncodeOptions) -> Self {
        let encoder = BrrEncoder::new();
        let brr_data = encoder.encode(pcm, options);

        let block_count = brr_data.len() / BRR_BLOCK_SIZE;
        let duration_ms = ((block_count * SAMPLES_PER_BLOCK) as u32 * 1000) / options.sample_rate;

        Self {
            id,
            name: name.to_string(),
            start_addr: 0,
            loop_addr: if options.looped { Some(0) } else { None },
            loop_count: if options.looped { None } else { Some(0) },
            sample_rate: options.sample_rate,
            brr_data,
            duration_ms,
            source_note: 60,
            adsr: AdsrEnvelope::default(),
            volume: 127,
            use_noise: false,
        }
    }
}

/// ADSR envelope configuration for a sample.
///
/// ADSR (Attack, Decay, Sustain, Release) controls how the volume
/// of a sound changes over time.
///
/// # Fields
/// - `attack`: Attack rate (0-15) - how quickly sound reaches full volume
/// - `decay`: Decay rate (0-7) - how quickly it drops to sustain level
/// - `sustain_level`: Sustain level (0-7) - volume during sustain phase
/// - `sustain_rate`: Sustain rate (0-31) - how volume changes during sustain
/// - `release`: Release rate (0-31) - how quickly sound fades after note off
/// - `adsr_enabled`: Whether ADSR is enabled (false = GAIN mode)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AdsrEnvelope {
    /// Attack rate (0-15)
    pub attack: u8,
    /// Decay rate (0-7)
    pub decay: u8,
    /// Sustain level (0-7)
    pub sustain_level: u8,
    /// Sustain rate (0-31)
    pub sustain_rate: u8,
    /// Release rate (0-31)
    pub release: u8,
    /// Whether ADSR is enabled (false = GAIN mode)
    pub adsr_enabled: bool,
}

impl Default for AdsrEnvelope {
    fn default() -> Self {
        Self {
            attack: 8,
            decay: 4,
            sustain_level: 4,
            sustain_rate: 16,
            release: 16,
            adsr_enabled: true,
        }
    }
}

/// Type of audio track.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrackType {
    /// Background music
    Music,
    /// Sound effect (one-shot)
    SoundEffect,
    /// Voice sample (rare in SPO)
    Voice,
    /// Ambient/atmospheric sound
    Ambient,
}

impl TrackType {
    /// Returns a short string identifier for this track type.
    pub fn as_str(&self) -> &'static str {
        match self {
            TrackType::Music => "Music",
            TrackType::SoundEffect => "SFX",
            TrackType::Voice => "Voice",
            TrackType::Ambient => "Ambient",
        }
    }

    /// Returns a human-readable display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            TrackType::Music => "Background Music",
            TrackType::SoundEffect => "Sound Effect",
            TrackType::Voice => "Voice Sample",
            TrackType::Ambient => "Ambient Sound",
        }
    }

    /// Returns an emoji icon for this track type.
    pub fn icon(&self) -> &'static str {
        match self {
            TrackType::Music => "🎵",
            TrackType::SoundEffect => "🔊",
            TrackType::Voice => "🎤",
            TrackType::Ambient => "🌊",
        }
    }
}
