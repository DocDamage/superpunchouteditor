//! Music sequence, channel, and note data structures.

use serde::{Deserialize, Serialize};

use super::sample::TrackType;

/// A music sequence/track.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sequence {
    /// Sequence ID
    pub id: u8,

    /// Human-readable name
    pub name: String,

    /// Type of track
    pub track_type: TrackType,

    /// Tempo in BPM
    pub tempo: u8,

    /// Time signature numerator
    pub time_signature_num: u8,

    /// Time signature denominator
    pub time_signature_den: u8,

    /// Audio channels in this sequence
    pub channels: Vec<Channel>,

    /// Total length in ticks
    pub total_ticks: u32,

    /// Loop point (tick to loop back to)
    pub loop_point: Option<u32>,

    /// Associated sound effects for this track
    pub associated_sfx: Vec<u8>,
}

/// A channel within a sequence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    /// Channel ID (0-7 for SPC700)
    pub channel_id: u8,

    /// Notes/events in this channel
    pub notes: Vec<Note>,

    /// Instrument/sample ID
    pub instrument: u8,

    /// Channel volume (0-127)
    pub volume: u8,

    /// Pan position (-64 to +64, 0 = center)
    pub pan: i8,

    /// Pitch bend amount (-8192 to +8191)
    pub pitch_bend: i16,

    /// Whether channel is muted
    pub muted: bool,

    /// Echo enable
    pub echo: bool,
}

/// A single note/event.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Note {
    /// Tick position in the sequence
    pub tick: u32,

    /// MIDI note number (0-127)
    pub pitch: u8,

    /// Velocity (0-127)
    pub velocity: u8,

    /// Duration in ticks
    pub duration: u16,

    /// Special effect flag
    pub effect: Option<NoteEffect>,
}

/// Special note effects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NoteEffect {
    /// Slide to next note
    Slide,
    /// Vibrato
    Vibrato,
    /// Tremolo
    Tremolo,
    /// Portamento
    Portamento,
    /// Key off (note end)
    KeyOff,
}

/// Entry for a sound in the sound list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoundEntry {
    /// Sound ID
    pub id: u8,

    /// Sound name
    pub name: String,

    /// Sound category
    pub category: String,

    /// Sample ID used
    pub sample_id: u8,

    /// Size in bytes (BRR data)
    pub size_bytes: usize,

    /// Duration in milliseconds
    pub duration_ms: u32,

    /// Associated music track (if any)
    pub associated_music: Option<u8>,
}

/// Entry for a music track in the music list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MusicEntry {
    /// Music ID
    pub id: u8,

    /// Track name
    pub name: String,

    /// Track type
    pub track_type: TrackType,

    /// Tempo in BPM
    pub tempo: u8,

    /// Number of channels
    pub channel_count: usize,

    /// Associated boxer/character (if character theme)
    pub associated_boxer: Option<String>,

    /// When this track plays in game
    pub play_context: String,
}
