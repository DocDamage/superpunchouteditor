//! SPC700 audio data structures and implementation.

use serde::{Deserialize, Serialize};

use crate::brr::{BrrDecoder, BRR_BLOCK_SIZE, SAMPLES_PER_BLOCK};

use super::constants::{
    CHANNEL_COUNT, DEFAULT_STACK_POINTER, DSP_REGISTER_SIZE, KNOWN_MUSIC, KNOWN_SOUNDS,
    SPC_RAM_SIZE, SPC_SAMPLE_RATE,
};
use super::sample::{AdsrEnvelope, Sample, TrackType};
use super::sequence::{MusicEntry, Sequence, SoundEntry};

/// SPC700 data representing the full audio subsystem state.
///
/// The SPC700 is a dedicated audio coprocessor with 64KB of RAM
/// that handles all sound generation for the SNES.
///
/// # Fields
/// - `ram`: 64KB SPC700 memory (0x0000 - 0xFFFF)
/// - `dsp_registers`: 128 bytes of DSP register values
/// - `sample_table`: Extracted BRR sample information
/// - `sequences`: Music/sound effect sequences
/// - `pc`, `a`, `x`, `y`, `sp`, `psw`: CPU registers
///
/// # Example
/// ```
/// use asset_core::audio::Spc700Data;
///
/// let spc = Spc700Data::new();
/// assert_eq!(spc.ram.len(), 65536);
/// assert_eq!(spc.sp, 0xEF);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Spc700Data {
    /// 64KB SPC700 memory (0x0000 - 0xFFFF)
    /// Stored as Vec for serialization compatibility
    #[serde(with = "serde_bytes")]
    pub ram: Vec<u8>,

    /// DSP registers (128 bytes at SPC700 addresses 0xF0F0-0xF0FF, 0xF1F0-0xF1FF, etc.)
    /// These control the 8 audio channels
    #[serde(with = "serde_bytes")]
    pub dsp_registers: Vec<u8>,

    /// Extracted sample table entries
    pub sample_table: Vec<Sample>,

    /// Music/sound effect sequences
    pub sequences: Vec<Sequence>,

    /// SPC700 internal registers
    /// Program counter
    pub pc: u16,
    /// Accumulator (8-bit)
    pub a: u8,
    /// X index register (8-bit)
    pub x: u8,
    /// Y index register (8-bit)
    pub y: u8,
    /// Stack pointer (points to page 1: 0x0100-0x01FF)
    pub sp: u8,
    /// Processor status word (flags)
    pub psw: u8,
}

impl Default for Spc700Data {
    fn default() -> Self {
        Self {
            ram: vec![0; SPC_RAM_SIZE],
            dsp_registers: vec![0; DSP_REGISTER_SIZE],
            sample_table: Vec::new(),
            sequences: Vec::new(),
            pc: 0,
            a: 0,
            x: 0,
            y: 0,
            sp: DEFAULT_STACK_POINTER,
            psw: 0,
        }
    }
}

impl Spc700Data {
    /// Creates empty SPC700 data with default initialization.
    ///
    /// # Example
    /// ```
    /// use asset_core::audio::Spc700Data;
    ///
    /// let spc = Spc700Data::new();
    /// assert_eq!(spc.ram.len(), 65536);
    /// assert_eq!(spc.dsp_registers.len(), 128);
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Gets a DSP register value by address.
    ///
    /// # Arguments
    /// - `addr`: DSP register address (0-127)
    ///
    /// # Returns
    /// The register value, or 0 if address is out of range
    ///
    /// # Example
    /// ```
    /// use asset_core::audio::Spc700Data;
    ///
    /// let spc = Spc700Data::new();
    /// let vol = spc.get_dsp_reg(0); // Channel 0 left volume
    /// ```
    pub fn get_dsp_reg(&self, addr: u8) -> u8 {
        if addr < DSP_REGISTER_SIZE as u8 {
            self.dsp_registers[addr as usize]
        } else {
            0
        }
    }

    /// Sets a DSP register value by address.
    ///
    /// # Arguments
    /// - `addr`: DSP register address (0-127)
    /// - `value`: The value to set
    ///
    /// # Example
    /// ```
    /// use asset_core::audio::Spc700Data;
    ///
    /// let mut spc = Spc700Data::new();
    /// spc.set_dsp_reg(0, 127); // Set channel 0 left volume to max
    /// ```
    pub fn set_dsp_reg(&mut self, addr: u8, value: u8) {
        if addr < DSP_REGISTER_SIZE as u8 {
            self.dsp_registers[addr as usize] = value;
        }
    }

    /// Gets the left and right volume for a channel.
    ///
    /// # Arguments
    /// - `channel`: Channel number (0-7)
    ///
    /// # Returns
    /// A tuple of (left_volume, right_volume), each 0-127
    ///
    /// # Example
    /// ```
    /// use asset_core::audio::Spc700Data;
    ///
    /// let spc = Spc700Data::new();
    /// let (left, right) = spc.get_channel_volume(0);
    /// ```
    pub fn get_channel_volume(&self, channel: u8) -> (u8, u8) {
        let base = (channel & 0x07) * 16;
        let left = self.get_dsp_reg(base);
        let right = self.get_dsp_reg(base + 1);
        (left, right)
    }

    /// Sets the volume for a channel.
    ///
    /// # Arguments
    /// - `channel`: Channel number (0-7)
    /// - `left`: Left volume (0-127)
    /// - `right`: Right volume (0-127)
    ///
    /// # Example
    /// ```
    /// use asset_core::audio::Spc700Data;
    ///
    /// let mut spc = Spc700Data::new();
    /// spc.set_channel_volume(0, 100, 100); // Full volume on both channels
    /// ```
    pub fn set_channel_volume(&mut self, channel: u8, left: u8, right: u8) {
        let base = (channel & 0x07) * 16;
        self.set_dsp_reg(base, left.min(127));
        self.set_dsp_reg(base + 1, right.min(127));
    }

    /// Extracts a sample from RAM at the given address.
    ///
    /// BRR samples in SPC700 RAM are typically stored as a sequence
    /// of 9-byte blocks. The end of a sample is marked by a block
    /// with the end flag set.
    ///
    /// # Arguments
    /// - `id`: Sample ID/index
    /// - `addr`: Start address in SPC700 RAM
    /// - `max_size`: Maximum bytes to read (safety limit)
    ///
    /// # Returns
    /// A Sample struct with extracted information, or None if extraction fails
    pub fn extract_sample(&self, id: u8, addr: u16, max_size: usize) -> Option<Sample> {
        let addr = addr as usize;
        if addr >= SPC_RAM_SIZE {
            return None;
        }

        // Scan for end flag in BRR blocks
        let mut brr_data = Vec::new();
        let mut pos = addr;
        let max_pos = (addr + max_size).min(SPC_RAM_SIZE);

        while pos + BRR_BLOCK_SIZE <= max_pos {
            let header = self.ram[pos];
            let end_flag = (header & 0x01) != 0;

            // Copy this block
            brr_data.extend_from_slice(&self.ram[pos..pos + BRR_BLOCK_SIZE]);
            pos += BRR_BLOCK_SIZE;

            if end_flag {
                break;
            }
        }

        // Calculate duration from BRR size
        let block_count = brr_data.len() / BRR_BLOCK_SIZE;
        let duration_ms = ((block_count * SAMPLES_PER_BLOCK) as u32 * 1000) / SPC_SAMPLE_RATE;

        Some(Sample {
            id,
            name: format!("Sample_{:02X}", id),
            start_addr: addr as u16,
            loop_addr: None,
            loop_count: None,
            sample_rate: SPC_SAMPLE_RATE,
            brr_data,
            duration_ms,
            source_note: 60, // Middle C
            adsr: AdsrEnvelope::default(),
            volume: 127,
            use_noise: false,
        })
    }

    /// Decodes a sample's BRR data to PCM.
    ///
    /// # Arguments
    /// - `sample_id`: ID of the sample to decode
    ///
    /// # Returns
    /// Decoded PCM samples if found
    pub fn decode_sample_to_pcm(&self, sample_id: u8) -> Option<Vec<i16>> {
        self.sample_table
            .iter()
            .find(|s| s.id == sample_id)
            .map(|sample| {
                let decoder = BrrDecoder::new();
                decoder.decode(&sample.brr_data)
            })
    }

    /// Gets a sound entry by ID.
    ///
    /// Looks up the sound in the `KNOWN_SOUNDS` table.
    ///
    /// # Arguments
    /// - `id`: Sound effect ID
    ///
    /// # Returns
    /// `Some(SoundEntry)` if found, `None` otherwise
    ///
    /// # Example
    /// ```
    /// use asset_core::audio::Spc700Data;
    ///
    /// if let Some(sound) = Spc700Data::get_sound_entry(0x01) {
    ///     println!("Found sound: {}", sound.name);
    /// }
    /// ```
    pub fn get_sound_entry(id: u8) -> Option<SoundEntry> {
        KNOWN_SOUNDS
            .iter()
            .find(|(_, sid, _)| *sid == id)
            .map(|(name, sid, category)| {
                SoundEntry {
                    id: *sid,
                    name: name.to_string(),
                    category: category.to_string(),
                    sample_id: *sid,
                    size_bytes: 0,    // TODO: Get actual size
                    duration_ms: 500, // TODO: Get actual duration
                    associated_music: None,
                }
            })
    }

    /// Gets a music entry by ID.
    ///
    /// Looks up the track in the `KNOWN_MUSIC` table.
    ///
    /// # Arguments
    /// - `id`: Music track ID
    ///
    /// # Returns
    /// `Some(MusicEntry)` if found, `None` otherwise
    pub fn get_music_entry(id: u8) -> Option<MusicEntry> {
        KNOWN_MUSIC
            .iter()
            .find(|(_, mid, _, _)| *mid == id)
            .map(|(name, mid, tempo, context)| MusicEntry {
                id: *mid,
                name: name.to_string(),
                track_type: TrackType::Music,
                tempo: *tempo,
                channel_count: CHANNEL_COUNT,
                associated_boxer: if *context == "boxer" {
                    Some(name.replace(" Theme", "").to_string())
                } else {
                    None
                },
                play_context: context.to_string(),
            })
    }

    /// Gets all known sound effects.
    ///
    /// # Returns
    /// A vector of all sounds in the `KNOWN_SOUNDS` table
    pub fn get_all_sounds() -> Vec<SoundEntry> {
        KNOWN_SOUNDS
            .iter()
            .map(|(name, id, category)| SoundEntry {
                id: *id,
                name: name.to_string(),
                category: category.to_string(),
                sample_id: *id,
                size_bytes: 0,
                duration_ms: 500,
                associated_music: None,
            })
            .collect()
    }

    /// Gets all known music tracks.
    ///
    /// # Returns
    /// A vector of all tracks in the `KNOWN_MUSIC` table
    pub fn get_all_music() -> Vec<MusicEntry> {
        KNOWN_MUSIC
            .iter()
            .map(|(name, id, tempo, context)| MusicEntry {
                id: *id,
                name: name.to_string(),
                track_type: TrackType::Music,
                tempo: *tempo,
                channel_count: CHANNEL_COUNT,
                associated_boxer: if *context == "boxer" {
                    Some(name.replace(" Theme", "").to_string())
                } else {
                    None
                },
                play_context: context.to_string(),
            })
            .collect()
    }
}
