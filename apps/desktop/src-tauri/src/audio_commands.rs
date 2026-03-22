//! Audio/Sound Editor Commands for Super Punch-Out!!
//!
//! This module provides Tauri commands for the Sound/Music Editor.
//! It handles:
//! - Sound effect browsing and preview
//! - Music sequence viewing and editing
//! - BRR sample import/export
//! - SPC file handling
//!
//! ## ROM audio status
//! SPC700 audio engine addresses and sample tables in the SPO ROM have not been
//! reverse-engineered yet.  Browsing/preview/export of *in-game* sounds therefore
//! operates on catalogue metadata only until that research is complete.
//!
//! Functionality that works today:
//! - WAV → BRR import (real encode via `asset_core::brr`)
//! - BRR → WAV export for *imported* samples
//! - Preview of *imported* samples (temp WAV written to disk, path returned)
//! - SPC file load / save / info
//! - BRR ↔ PCM codec commands

use std::collections::HashMap;

use asset_core::audio::{
    export_brr_to_wav, import_wav_to_brr, MusicEntry, PlaybackState, PreviewConfig, SoundEntry,
    Spc700Data,
};
use asset_core::brr::{BrrDecoder, BrrEncodeOptions, BrrEncoder};
use asset_core::spc::{Id666Tag, SpcFile};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::AppState;

/// Audio playback state managed by Tauri
pub struct AudioState {
    /// Currently loaded SPC700 data
    current_spc: Mutex<Option<Spc700Data>>,
    /// Playback state
    playback_state: Mutex<PlaybackState>,
    /// Preview configuration
    preview_config: Mutex<PreviewConfig>,
    /// Currently playing sound ID
    current_sound_id: Mutex<Option<u8>>,
    /// Currently playing music ID
    current_music_id: Mutex<Option<u8>>,
    /// User-imported BRR data keyed by sample ID.
    /// Populated by `import_sound_from_wav`; consumed by preview and export commands.
    pub imported_samples: Mutex<HashMap<u8, Vec<u8>>>,
}

impl AudioState {
    pub fn new() -> Self {
        Self {
            current_spc: Mutex::new(None),
            playback_state: Mutex::new(PlaybackState::Stopped),
            preview_config: Mutex::new(PreviewConfig::default()),
            current_sound_id: Mutex::new(None),
            current_music_id: Mutex::new(None),
            imported_samples: Mutex::new(HashMap::new()),
        }
    }
}

impl Default for AudioState {
    fn default() -> Self {
        Self::new()
    }
}

/// Sound list response
#[derive(Debug, Clone, Serialize)]
pub struct SoundListResponse {
    pub sounds: Vec<SoundEntry>,
    pub total_count: usize,
    pub categories: Vec<String>,
}

/// Music list response
#[derive(Debug, Clone, Serialize)]
pub struct MusicListResponse {
    pub tracks: Vec<MusicEntry>,
    pub total_count: usize,
    pub categories: Vec<String>,
}

/// Sample detail response
#[derive(Debug, Clone, Serialize)]
pub struct SampleDetail {
    pub id: u8,
    pub name: String,
    pub format: String,
    pub sample_rate: u32,
    pub loop_enabled: bool,
    pub duration_ms: u32,
    pub size_bytes: usize,
    pub has_loop: bool,
    pub loop_start: u16,
    pub loop_end: u16,
}

/// Sequence detail response
#[derive(Debug, Clone, Serialize)]
pub struct SequenceDetail {
    pub id: u8,
    pub name: String,
    pub track_type: String,
    pub tempo: u8,
    pub channel_count: usize,
    pub total_ticks: u32,
    pub loop_point: Option<u32>,
    pub duration_seconds: f32,
}

/// Audio export options
#[derive(Debug, Clone, Deserialize)]
pub struct ExportOptions {
    pub format: String,
    pub sample_rate: Option<u32>,
    pub include_header: Option<bool>,
}

/// Import options for WAV to BRR
#[derive(Debug, Clone, Deserialize)]
pub struct ImportOptions {
    pub sample_id: u8,
    pub enable_loop: bool,
    pub loop_start: Option<usize>,
    pub target_sample_rate: u32,
}

// ============================================================================
// Internal helpers
// ============================================================================

/// Decode `brr_data` to PCM, write a temporary WAV file, and return its path.
///
/// The file is placed in the OS temp directory with a name derived from `sound_id`.
/// The frontend can convert this path to a playable URL via Tauri's `convertFileSrc`.
fn write_preview_wav(brr_data: &[u8], sound_id: u8) -> Result<String, String> {
    use asset_core::audio::write_wav_file;

    let decoder = BrrDecoder::new();
    let pcm = decoder.decode(brr_data);

    let temp_path = std::env::temp_dir().join(format!("spo_preview_{}.wav", sound_id));

    write_wav_file(&temp_path, &pcm, 32000).map_err(|e| {
        format!("Failed to write preview WAV: {}", e)
    })?;

    temp_path
        .to_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "Preview WAV path contains non-UTF8 characters".to_string())
}

// ============================================================================
// Sound/SFX Commands
// ============================================================================

/// Get list of all sound effects
#[tauri::command]
pub fn get_sound_list() -> Result<SoundListResponse, String> {
    let sounds = Spc700Data::get_all_sounds();

    // Collect unique categories
    let mut categories: Vec<String> = sounds
        .iter()
        .map(|s| s.category.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    categories.sort();

    let total_count = sounds.len();

    Ok(SoundListResponse {
        sounds,
        total_count,
        categories,
    })
}

/// Get a specific sound entry
#[tauri::command]
pub fn get_sound(sound_id: u8) -> Result<SoundEntry, String> {
    Spc700Data::get_sound_entry(sound_id).ok_or_else(|| format!("Sound ID {} not found", sound_id))
}

/// Preview a sound effect.
///
/// For samples imported via `import_sound_from_wav`, BRR data is decoded to PCM,
/// written to a temporary WAV file, and the file path is returned so the frontend
/// can play it via an HTML `<audio>` element + `convertFileSrc`.
///
/// In-game sounds whose ROM addresses have not been researched return an explanatory
/// error rather than silently doing nothing.
#[tauri::command]
pub fn preview_sound(state: State<AppState>, sound_id: u8) -> Result<String, String> {
    let _sound = Spc700Data::get_sound_entry(sound_id)
        .ok_or_else(|| format!("Sound ID {} not found", sound_id))?;

    let brr = {
        let audio = state.audio_state.lock();
        audio.imported_samples.lock().get(&sound_id).cloned()
    }
    .ok_or_else(|| {
        format!(
            "Sound {} has no imported audio data. \
             Import a WAV file first, or note that in-game ROM audio \
             addresses have not yet been reverse-engineered.",
            sound_id
        )
    })?;

    let wav_path = write_preview_wav(&brr, sound_id)?;

    {
        let audio = state.audio_state.lock();
        *audio.playback_state.lock() = PlaybackState::Playing;
        *audio.current_sound_id.lock() = Some(sound_id);
    }

    Ok(wav_path)
}

/// Stop any playing preview and clear playback state.
#[tauri::command]
pub fn stop_preview(state: State<AppState>) -> Result<(), String> {
    let audio = state.audio_state.lock();
    *audio.playback_state.lock() = PlaybackState::Stopped;
    *audio.current_sound_id.lock() = None;
    *audio.current_music_id.lock() = None;
    Ok(())
}

/// Get current playback state as a string (`"playing"`, `"paused"`, or `"stopped"`).
#[tauri::command]
pub fn get_playback_state(state: State<AppState>) -> Result<String, String> {
    let audio = state.audio_state.lock();
    let s = match *audio.playback_state.lock() {
        PlaybackState::Playing => "playing",
        PlaybackState::Paused => "paused",
        PlaybackState::Stopped => "stopped",
    };
    Ok(s.to_string())
}

/// Export an imported sample as a WAV file.
///
/// Requires the sample to have been imported via `import_sound_from_wav`.
/// Returns an error for in-game sounds (ROM audio addresses not yet researched).
#[tauri::command]
pub fn export_sound_as_wav(
    state: State<AppState>,
    sound_id: u8,
    output_path: String,
    options: Option<ExportOptions>,
) -> Result<(), String> {
    let _sound = Spc700Data::get_sound_entry(sound_id)
        .ok_or_else(|| format!("Sound ID {} not found", sound_id))?;

    let brr = {
        let audio = state.audio_state.lock();
        audio.imported_samples.lock().get(&sound_id).cloned()
    }
    .ok_or_else(|| {
        format!(
            "Sound {} has no imported audio data. \
             WAV export requires a prior WAV import. \
             In-game ROM audio addresses have not yet been reverse-engineered.",
            sound_id
        )
    })?;

    let target_rate = options.as_ref().and_then(|o| o.sample_rate).unwrap_or(32000);
    export_brr_to_wav(&brr, &output_path, target_rate).map_err(|e| e.to_string())
}

/// Export an imported sample as a BRR file.
///
/// Requires the sample to have been imported via `import_sound_from_wav`.
#[tauri::command]
pub fn export_sound_as_brr(
    state: State<AppState>,
    sound_id: u8,
    output_path: String,
) -> Result<usize, String> {
    let _sound = Spc700Data::get_sound_entry(sound_id)
        .ok_or_else(|| format!("Sound ID {} not found", sound_id))?;

    let brr = {
        let audio = state.audio_state.lock();
        audio.imported_samples.lock().get(&sound_id).cloned()
    }
    .ok_or_else(|| {
        format!(
            "Sound {} has no imported audio data. \
             BRR export requires a prior WAV import.",
            sound_id
        )
    })?;

    let size = brr.len();
    std::fs::write(&output_path, &brr).map_err(|e| format!("Failed to write BRR file: {}", e))?;
    Ok(size)
}

// ============================================================================
// Music/Sequence Commands
// ============================================================================

/// Get list of all music tracks
#[tauri::command]
pub fn get_music_list() -> Result<MusicListResponse, String> {
    let tracks = Spc700Data::get_all_music();

    // Collect categories
    let mut categories = vec!["All".to_string()];
    let track_types: std::collections::HashSet<_> = tracks
        .iter()
        .map(|t| t.track_type.as_str().to_string())
        .collect();
    categories.extend(track_types);

    let total_count = tracks.len();

    Ok(MusicListResponse {
        tracks,
        total_count,
        categories,
    })
}

/// Get a specific music track
#[tauri::command]
pub fn get_music(music_id: u8) -> Result<MusicEntry, String> {
    Spc700Data::get_music_entry(music_id).ok_or_else(|| format!("Music ID {} not found", music_id))
}

/// Get music sequence details
#[tauri::command]
pub fn get_music_sequence(music_id: u8) -> Result<SequenceDetail, String> {
    let entry = Spc700Data::get_music_entry(music_id)
        .ok_or_else(|| format!("Music ID {} not found", music_id))?;

    // Calculate approximate duration
    let duration_seconds = 60.0; // Placeholder

    Ok(SequenceDetail {
        id: entry.id,
        name: entry.name,
        track_type: entry.track_type.as_str().to_string(),
        tempo: entry.tempo,
        channel_count: entry.channel_count,
        total_ticks: 0,   // TODO: Get from sequence data
        loop_point: None, // TODO: Get from sequence data
        duration_seconds,
    })
}

/// Preview a music track.
///
/// Music playback requires SPC700 emulation which is not yet implemented.
/// This command returns an error so the frontend can show an informative message.
#[tauri::command]
pub fn preview_music(_state: State<AppState>, music_id: u8) -> Result<(), String> {
    let _music = Spc700Data::get_music_entry(music_id)
        .ok_or_else(|| format!("Music ID {} not found", music_id))?;

    Err(
        "Music preview requires SPC700 emulation which is not yet implemented. \
         Export the track as an SPC file and open it in a standalone SPC player."
            .to_string(),
    )
}

/// Update music sequence
#[tauri::command]
pub fn update_music_sequence(_music_id: u8, _updates: serde_json::Value) -> Result<(), String> {
    // TODO: Implement sequence editing
    Err("Music sequence editing not yet implemented".to_string())
}

/// Export music as WAV
#[tauri::command]
pub fn export_music_as_wav(
    music_id: u8,
    _output_path: String,
    duration_seconds: Option<u32>,
) -> Result<(), String> {
    let _music = Spc700Data::get_music_entry(music_id)
        .ok_or_else(|| format!("Music ID {} not found", music_id))?;

    let _duration = duration_seconds.unwrap_or(60);

    // TODO: Render sequence to WAV
    // Requires SPC700 emulation or prerendered samples

    Err("Music export not yet implemented".to_string())
}

/// Export music as SPC file
#[tauri::command]
pub fn export_music_as_spc(music_id: u8, output_path: String) -> Result<(), String> {
    let music = Spc700Data::get_music_entry(music_id)
        .ok_or_else(|| format!("Music ID {} not found", music_id))?;

    // Create SPC data
    let spc_data = Spc700Data::default();

    // Build ID666 tag
    let tag = Id666Tag {
        song_title: music.name.clone(),
        game_title: "Super Punch-Out!!".to_string(),
        seconds_to_play: 60,
        artist: "Nintendo".to_string(),
        ..Default::default()
    };

    // Save SPC file
    SpcFile::save(&spc_data, output_path, Some(&tag)).map_err(|e| e.to_string())?;

    Ok(())
}

// ============================================================================
// Sample Import/Export Commands
// ============================================================================

/// Get sample details
#[tauri::command]
pub fn get_sample(sample_id: u8) -> Result<SampleDetail, String> {
    Err(format!(
        "Sample {} metadata not available: SPO sample table addresses are not yet researched",
        sample_id
    ))
}

/// Import a WAV file, encode it to BRR, and store it in `AudioState::imported_samples`.
///
/// The encoded BRR data can then be previewed via `preview_sound` and exported
/// via `export_sound_as_wav` / `export_sound_as_brr`.
#[tauri::command]
pub fn import_sound_from_wav(
    state: State<AppState>,
    wav_path: String,
    options: ImportOptions,
) -> Result<serde_json::Value, String> {
    let encode_options = BrrEncodeOptions {
        looped: options.enable_loop,
        loop_start: options.loop_start.unwrap_or(0),
        sample_rate: options.target_sample_rate,
        quality: 3,
    };

    // Read WAV → PCM, resample if needed, encode to BRR
    let brr_data =
        import_wav_to_brr(&wav_path, encode_options).map_err(|e| e.to_string())?;

    let brr_size = brr_data.len();
    let sample_id = options.sample_id;

    // Store in AudioState so preview/export commands can access it
    {
        let audio = state.audio_state.lock();
        audio.imported_samples.lock().insert(sample_id, brr_data);
    }

    // Compute approximate duration from BRR size
    // Each BRR block is 9 bytes → 16 samples at the target rate
    let brr_blocks = brr_size / 9;
    let duration_ms = (brr_blocks as u64 * 16 * 1000)
        / options.target_sample_rate.max(1) as u64;

    Ok(serde_json::json!({
        "sample_id": sample_id,
        "wav_path": wav_path,
        "brr_size": brr_size,
        "sample_rate": options.target_sample_rate,
        "loop_enabled": options.enable_loop,
        "duration_ms": duration_ms,
        "ready_for_preview": true,
    }))
}

/// Decode BRR to PCM for preview
#[tauri::command]
pub fn decode_brr_to_pcm(brr_data: Vec<u8>) -> Result<Vec<i16>, String> {
    let decoder = BrrDecoder::new();
    let pcm = decoder.decode(&brr_data);
    Ok(pcm)
}

/// Encode PCM to BRR
#[tauri::command]
pub fn encode_pcm_to_brr(
    pcm_data: Vec<i16>,
    looped: bool,
    loop_start: Option<usize>,
) -> Result<Vec<u8>, String> {
    let encoder = BrrEncoder::new();
    let options = BrrEncodeOptions {
        looped,
        loop_start: loop_start.unwrap_or(0),
        sample_rate: 32000,
        quality: 3,
    };
    let brr = encoder.encode(&pcm_data, options);
    Ok(brr)
}

// ============================================================================
// SPC File Commands
// ============================================================================

/// Load SPC file for editing
#[tauri::command]
pub fn load_spc(state: State<AppState>, path: String) -> Result<serde_json::Value, String> {
    let spc_data = SpcFile::load(&path).map_err(|e| e.to_string())?;

    // Store in state
    {
        let audio = state.audio_state.lock();
        let mut current = audio.current_spc.lock();
        *current = Some(spc_data.clone());
    }

    // Read ID666 tag
    let tag_info = SpcFile::read_id666(&path).map_err(|e| e.to_string())?;

    Ok(serde_json::json!({
        "loaded": true,
        "pc": spc_data.pc,
        "a": spc_data.a,
        "x": spc_data.x,
        "y": spc_data.y,
        "sp": spc_data.sp,
        "psw": spc_data.psw,
        "has_tag": tag_info.is_some(),
        "tag": tag_info.map(|t| serde_json::json!({
            "song_title": t.song_title,
            "game_title": t.game_title,
            "artist": t.artist,
            "seconds_to_play": t.seconds_to_play,
        })),
    }))
}

/// Save current SPC data to file
#[tauri::command]
pub fn save_spc(
    state: State<AppState>,
    path: String,
    metadata: Option<serde_json::Value>,
) -> Result<(), String> {
    let audio = state.audio_state.lock();
    let current = audio.current_spc.lock();
    let spc_data = current.as_ref().ok_or("No SPC data loaded")?;

    // Build ID666 tag from metadata
    let tag = metadata.map(|m| Id666Tag {
        song_title: m["song_title"].as_str().unwrap_or("").to_string(),
        game_title: m["game_title"].as_str().unwrap_or("").to_string(),
        artist: m["artist"].as_str().unwrap_or("").to_string(),
        ..Default::default()
    });

    SpcFile::save(spc_data, path, tag.as_ref()).map_err(|e| e.to_string())?;

    Ok(())
}

/// Create new empty SPC
#[tauri::command]
pub fn create_new_spc(state: State<AudioState>) -> Result<(), String> {
    let new_spc = Spc700Data::default();

    let mut current = state.current_spc.lock();
    *current = Some(new_spc);

    Ok(())
}

/// Get SPC file info without loading
#[tauri::command]
pub fn get_spc_info(path: String) -> Result<serde_json::Value, String> {
    let info = SpcFile::get_info(&path).map_err(|e| e.to_string())?;

    Ok(serde_json::json!({
        "path": info.path.to_string_lossy().to_string(),
        "size": info.size,
        "has_id666": info.has_id666,
        "tag": info.tag.map(|t| serde_json::json!({
            "song_title": t.song_title,
            "game_title": t.game_title,
            "artist": t.artist,
            "dumper": t.dumper,
            "dump_date": t.dump_date,
            "seconds_to_play": t.seconds_to_play,
            "fade_length_ms": t.fade_length_ms,
        })),
    }))
}

// ============================================================================
// Audio Settings Commands
// ============================================================================

/// Get preview configuration
#[tauri::command]
pub fn get_preview_config(state: State<AudioState>) -> Result<PreviewConfig, String> {
    Ok(state.preview_config.lock().clone())
}

/// Set preview configuration
#[tauri::command]
pub fn set_preview_config(state: State<AppState>, config: PreviewConfig) -> Result<(), String> {
    let audio = state.audio_state.lock();
    let mut current = audio.preview_config.lock();
    *current = config;
    Ok(())
}

/// Get audio editor settings
#[tauri::command]
pub fn get_audio_settings() -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "supported_import_formats": ["wav", "aiff", "raw"],
        "supported_export_formats": ["wav", "brr", "spc"],
        "max_sample_rate": 48000,
        "min_sample_rate": 8000,
        "default_sample_rate": 32000,
        "max_brr_size": 65536,
        "spc_ram_size": 65536,
        "dsp_register_count": 128,
        "audio_channels": 8,
    }))
}

// ============================================================================
// ROM Audio Extraction Commands (Research TODOs)
// ============================================================================

/// Scan ROM for audio data
///
/// # Research TODO
/// - Identify SPC engine location
/// - Map sample table addresses
/// - Locate sequence data
#[tauri::command]
pub fn scan_rom_for_audio(_rom_path: String) -> Result<serde_json::Value, String> {
    // TODO: Implement ROM audio scanning
    // This would:
    // 1. Search for known SPC engine signatures
    // 2. Identify sample table locations
    // 3. Find sequence data
    // 4. Build a map of audio assets

    Ok(serde_json::json!({
        "scanned": false,
        "note": "ROM audio scanning requires reverse engineering of SPO audio engine",
        "research_needed": [
            "SPC700 engine location in ROM",
            "Sample table format and location",
            "Sequence data format",
            "Instrument/sample mapping",
        ],
    }))
}

/// Extract all audio from ROM
#[tauri::command]
pub fn extract_all_rom_audio(
    _rom_path: String,
    _output_dir: String,
) -> Result<serde_json::Value, String> {
    // TODO: Implement full ROM audio extraction
    Err("ROM audio extraction requires SPO audio engine research".to_string())
}

/// Get audio engine info
#[tauri::command]
pub fn get_audio_engine_info() -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "engine": "Unknown - Research Required",
        "spc700_present": true,
        "likely_engine": "Nintendo S-SMP (custom variant)",
        "known_info": {
            "sample_format": "BRR (Bit Rate Reduction)",
            "max_samples": 256,
            "max_sequences": 256,
            "channels": 8,
            "sample_rates": "Variable, typically 32000 Hz",
        },
        "research_status": "Not started",
        "needed_research": [
            "Confirm SPC700 IPL ROM location",
            "Identify sample table address",
            "Document sequence format",
            "Map music IDs to sequences",
            "Map SFX IDs to samples",
        ],
    }))
}
