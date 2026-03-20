//! Embedded Snes9x Emulator Integration
//!
//! Provides an embedded SNES emulator using the Snes9x libretro core.
//! This allows testing ROM modifications directly within the editor
//! without launching an external emulator.

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::State;

// Re-export types from emulator_core
pub use emulator_core::{
    AudioBatch, CoreConfig, Snes9xCore, VideoFrame,
};

/// State for the embedded emulator
pub struct EmbeddedEmulatorState {
    /// The Snes9x core instance (wrapped for thread safety)
    pub core: Arc<Mutex<Option<Snes9xCore>>>,
    /// Whether the emulation loop is currently running
    pub running: Arc<Mutex<bool>>,
    /// Whether emulation is paused
    pub paused: Arc<Mutex<bool>>,
    /// Current emulation speed multiplier (1.0 = normal)
    pub speed: Arc<Mutex<f32>>,
    /// Channel for sending video frames from emulation thread to frontend
    pub frame_sender: crossbeam_channel::Sender<VideoFrame>,
    /// Channel for receiving video frames in the frontend
    pub frame_receiver: crossbeam_channel::Receiver<VideoFrame>,
    /// Channel for sending audio samples from emulation thread to audio output
    pub audio_sender: crossbeam_channel::Sender<AudioBatch>,
    /// Channel for receiving audio samples
    pub audio_receiver: crossbeam_channel::Receiver<AudioBatch>,
    /// Channel for sending controller input to the emulation thread
    pub input_sender: crossbeam_channel::Sender<u16>,
    /// Channel for receiving controller input
    pub input_receiver: crossbeam_channel::Receiver<u16>,
    /// Handle to the emulation thread (if running)
    pub thread_handle: Arc<Mutex<Option<std::thread::JoinHandle<()>>>>,
    /// Path to the loaded ROM (if any)
    pub loaded_rom_path: Arc<Mutex<Option<String>>>,
    /// Current save state slot
    pub current_save_slot: Arc<Mutex<u8>>,
}

impl EmbeddedEmulatorState {
    /// Create a new embedded emulator state
    pub fn new() -> Self {
        let (frame_tx, frame_rx) = crossbeam_channel::bounded(2);
        let (audio_tx, audio_rx) = crossbeam_channel::bounded(1024);
        let (input_tx, input_rx) = crossbeam_channel::unbounded();

        Self {
            core: Arc::new(Mutex::new(None)),
            running: Arc::new(Mutex::new(false)),
            paused: Arc::new(Mutex::new(false)),
            speed: Arc::new(Mutex::new(1.0)),
            frame_sender: frame_tx,
            frame_receiver: frame_rx,
            audio_sender: audio_tx,
            audio_receiver: audio_rx,
            input_sender: input_tx,
            input_receiver: input_rx,
            thread_handle: Arc::new(Mutex::new(None)),
            loaded_rom_path: Arc::new(Mutex::new(None)),
            current_save_slot: Arc::new(Mutex::new(0)),
        }
    }

    /// Check if the emulator is initialized
    pub fn is_initialized(&self) -> bool {
        self.core.lock().is_some()
    }

    /// Check if the emulator is currently running
    pub fn is_running(&self) -> bool {
        *self.running.lock()
    }

    /// Check if emulation is paused
    pub fn is_paused(&self) -> bool {
        *self.paused.lock()
    }

    /// Get the current emulation speed
    pub fn get_speed(&self) -> f32 {
        *self.speed.lock()
    }

    /// Get the path to the default core library based on platform
    /// Checks bundled location first, then system locations
    fn get_default_core_path() -> String {
        // Check bundled location first
        let bundled_paths = [
            "binaries/snes9x_libretro.dll",   // Windows
            "binaries/snes9x_libretro.dylib", // macOS
            "binaries/snes9x_libretro.so",    // Linux
        ];

        for path in &bundled_paths {
            if std::path::Path::new(path).exists() {
                return path.to_string();
            }
        }

        // Check system locations
        #[cfg(target_os = "windows")]
        return "snes9x_libretro.dll".to_string();

        #[cfg(target_os = "macos")]
        return "snes9x_libretro.dylib".to_string();

        #[cfg(target_os = "linux")]
        return "snes9x_libretro.so".to_string();

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        return "snes9x_libretro.so".to_string();
    }
}

impl Default for EmbeddedEmulatorState {
    fn default() -> Self {
        Self::new()
    }
}

/// Emulator status information for the frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmulatorStatus {
    pub initialized: bool,
    pub running: bool,
    pub paused: bool,
    pub speed: f32,
    pub has_rom: bool,
    pub rom_path: Option<String>,
    pub current_slot: u8,
}

/// Video frame data sent to the frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmulatorFrameData {
    /// Raw RGBA pixel data
    pub pixels: Vec<u8>,
    /// Frame width
    pub width: u32,
    /// Frame height
    pub height: u32,
}

/// Input state for SNES controller
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControllerInput {
    pub b: bool,
    pub y: bool,
    pub select: bool,
    pub start: bool,
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub a: bool,
    pub x: bool,
    pub l: bool,
    pub r: bool,
}

impl ControllerInput {
    /// Convert to SNES controller button bitmask
    pub fn to_buttons(&self) -> u16 {
        let mut buttons = 0u16;
        if self.b {
            buttons |= 0x8000;
        }
        if self.y {
            buttons |= 0x4000;
        }
        if self.select {
            buttons |= 0x2000;
        }
        if self.start {
            buttons |= 0x1000;
        }
        if self.up {
            buttons |= 0x0800;
        }
        if self.down {
            buttons |= 0x0400;
        }
        if self.left {
            buttons |= 0x0200;
        }
        if self.right {
            buttons |= 0x0100;
        }
        if self.a {
            buttons |= 0x0080;
        }
        if self.x {
            buttons |= 0x0040;
        }
        if self.l {
            buttons |= 0x0020;
        }
        if self.r {
            buttons |= 0x0010;
        }
        buttons
    }
}

/// Initialize the emulator with Snes9x core
#[tauri::command]
pub fn init_emulator(
    state: State<'_, EmbeddedEmulatorState>,
    core_path: Option<String>,
) -> Result<(), String> {
    let core_path = core_path.unwrap_or_else(|| EmbeddedEmulatorState::get_default_core_path());

    // Check if core file exists
    if !std::path::Path::new(&core_path).exists() {
        // Try to find in common locations
        let common_paths = vec![
            "./cores/snes9x_libretro.dll",
            "./cores/snes9x_libretro.so",
            "./cores/snes9x_libretro.dylib",
            "../cores/snes9x_libretro.dll",
            "../cores/snes9x_libretro.so",
            "../cores/snes9x_libretro.dylib",
        ];

        let found_path = common_paths
            .iter()
            .find(|p| std::path::Path::new(p).exists());

        if let Some(path) = found_path {
            println!("Found core at: {}", path);
        } else {
            return Err(format!(
                "Core library not found at: {}. Please provide the correct path.",
                core_path
            ));
        }
    }

    // Initialize the core with default config
    let config = CoreConfig::default();
    let core =
        Snes9xCore::with_config(config).map_err(|e| format!("Failed to initialize core: {}", e))?;

    *state.core.lock() = Some(core);
    println!("Snes9x core initialized successfully");

    Ok(())
}

/// Load ROM into emulator from file path
#[tauri::command]
pub fn emulator_load_rom(
    state: State<'_, EmbeddedEmulatorState>,
    rom_path: String,
) -> Result<(), String> {
    let rom_data = std::fs::read(&rom_path).map_err(|e| format!("Failed to read ROM: {}", e))?;
    let rom_size = rom_data.len();

    // Validate ROM header
    if rom_size < 0x8000 {
        return Err("ROM file too small".to_string());
    }

    let mut core_guard = state.core.lock();
    if let Some(ref mut core) = *core_guard {
        core.load_rom(rom_data)
            .map_err(|e| format!("Failed to load ROM into core: {}", e))?;
        *state.loaded_rom_path.lock() = Some(rom_path);
        println!("ROM loaded successfully: {} bytes", rom_size);
        Ok(())
    } else {
        Err("Emulator not initialized. Call init_emulator first.".to_string())
    }
}

/// Load ROM from memory buffer (for testing pending edits)
#[tauri::command]
pub fn emulator_load_rom_from_memory(
    state: State<'_, EmbeddedEmulatorState>,
    rom_data: Vec<u8>,
) -> Result<(), String> {
    let rom_size = rom_data.len();
    if rom_size < 0x8000 {
        return Err("ROM data too small".to_string());
    }

    let mut core_guard = state.core.lock();
    if let Some(ref mut core) = *core_guard {
        core.load_rom(rom_data)
            .map_err(|e| format!("Failed to load ROM: {}", e))?;
        *state.loaded_rom_path.lock() = None; // No file path for memory-loaded ROM
        println!("ROM loaded from memory: {} bytes", rom_size);
        Ok(())
    } else {
        Err("Emulator not initialized".to_string())
    }
}

/// Load ROM with pending edits applied
#[tauri::command]
pub fn emulator_load_rom_with_edits(
    state: State<'_, EmbeddedEmulatorState>,
    rom_path: String,
    edits: std::collections::HashMap<String, Vec<u8>>,
) -> Result<(), String> {
    // Read original ROM
    let mut rom_data =
        std::fs::read(&rom_path).map_err(|e| format!("Failed to read ROM: {}", e))?;

    // Apply pending edits
    for (offset_str, bytes) in edits {
        let offset = parse_offset(&offset_str)?;
        let len = bytes.len().min(rom_data.len() - offset);
        rom_data[offset..offset + len].copy_from_slice(&bytes[..len]);
    }

    let rom_size = rom_data.len();

    // Load modified ROM
    let mut core_guard = state.core.lock();
    if let Some(ref mut core) = *core_guard {
        core.load_rom(rom_data)
            .map_err(|e| format!("Failed to load modified ROM: {}", e))?;
        *state.loaded_rom_path.lock() = Some(rom_path);
        println!("ROM with edits loaded: {} bytes", rom_size);
        Ok(())
    } else {
        Err("Emulator not initialized".to_string())
    }
}

/// Start the emulation loop
#[tauri::command]
pub fn emulator_start(state: State<'_, EmbeddedEmulatorState>) -> Result<(), String> {
    // Check if already running
    if *state.running.lock() {
        return Err("Emulator already running".to_string());
    }

    // Check if core is initialized
    {
        let core_guard = state.core.lock();
        if core_guard.is_none() {
            return Err("Emulator not initialized".to_string());
        }
    }

    // Set running flag
    *state.running.lock() = true;
    *state.paused.lock() = false;

    // Clone Arc references for the thread
    let core = state.core.clone();
    let running = state.running.clone();
    let paused = state.paused.clone();
    let speed = state.speed.clone();
    let frame_sender = state.frame_sender.clone();
    let audio_sender = state.audio_sender.clone();
    let input_receiver = state.input_receiver.clone();

    // Start emulation thread
    let handle = std::thread::spawn(move || {
        let target_fps = 60.0;
        let _frame_duration = Duration::from_secs_f64(1.0 / target_fps);

        while *running.lock() {
            let frame_start = Instant::now();

            // Check if paused
            if *paused.lock() {
                std::thread::sleep(Duration::from_millis(16));
                continue;
            }

            // Get current speed
            let current_speed = *speed.lock() as f64;
            let adjusted_frame_duration =
                Duration::from_secs_f64(1.0 / (target_fps * current_speed));

            // Process input
            if let Ok(buttons) = input_receiver.try_recv() {
                let mut core_guard = core.lock();
                if let Some(ref mut core) = *core_guard {
                    core.set_input(0, buttons); // Controller port 0
                }
            }

            // Run one frame
            let mut core_guard = core.lock();
            if let Some(ref mut core) = *core_guard {
                core.run_frame();

                // Get video frame
                if let Some(frame) = core.get_frame_buffer() {
                    let _ = frame_sender.try_send(frame);
                }

                // Get audio samples
                if let Some(audio) = core.get_audio_samples() {
                    let _ = audio_sender.try_send(audio);
                }
            }

            // Frame pacing
            let elapsed = frame_start.elapsed();
            if elapsed < adjusted_frame_duration {
                spin_sleep::sleep(adjusted_frame_duration - elapsed);
            }
        }

        println!("Emulation thread stopped");
    });

    *state.thread_handle.lock() = Some(handle);
    println!("Emulation started");

    Ok(())
}

/// Stop emulation
#[tauri::command]
pub fn emulator_stop(state: State<'_, EmbeddedEmulatorState>) {
    *state.running.lock() = false;
    *state.paused.lock() = false;

    // Wait for thread to finish
    if let Some(handle) = state.thread_handle.lock().take() {
        let _ = handle.join();
    }

    println!("Emulation stopped");
}

/// Pause/resume emulation
#[tauri::command]
pub fn emulator_set_paused(state: State<'_, EmbeddedEmulatorState>, paused: bool) {
    *state.paused.lock() = paused;
    println!("Emulation {}", if paused { "paused" } else { "resumed" });
}

/// Toggle pause state
#[tauri::command]
pub fn emulator_toggle_pause(state: State<'_, EmbeddedEmulatorState>) -> bool {
    let new_state = !*state.paused.lock();
    *state.paused.lock() = new_state;
    new_state
}

/// Reset emulator
#[tauri::command]
pub fn emulator_reset(state: State<'_, EmbeddedEmulatorState>, hard: bool) {
    let mut core_guard = state.core.lock();
    if let Some(ref mut core) = *core_guard {
        if hard {
            core.reset_hard();
            println!("Hard reset performed");
        } else {
            core.reset_soft();
            println!("Soft reset performed");
        }
    }
}

/// Get the next video frame (called repeatedly by frontend)
#[tauri::command]
pub fn emulator_get_frame(state: State<'_, EmbeddedEmulatorState>) -> Option<EmulatorFrameData> {
    state
        .frame_receiver
        .try_recv()
        .ok()
        .map(|frame| EmulatorFrameData {
            pixels: frame.to_rgba(),
            width: frame.width,
            height: frame.height,
        })
}

/// Send controller input
#[tauri::command]
pub fn emulator_set_input(state: State<'_, EmbeddedEmulatorState>, buttons: u16) {
    let _ = state.input_sender.try_send(buttons);
}

/// Send controller input from structured data
#[tauri::command]
pub fn emulator_set_controller_input(
    state: State<'_, EmbeddedEmulatorState>,
    input: ControllerInput,
) {
    let buttons = input.to_buttons();
    let _ = state.input_sender.try_send(buttons);
}

/// Save state to a slot
#[tauri::command]
pub fn emulator_save_state(
    state: State<'_, EmbeddedEmulatorState>,
    slot: u8,
) -> Result<(), String> {
    let core_guard = state.core.lock();
    if let Some(ref core) = *core_guard {
        let state_data = core
            .save_state()
            .map_err(|e| format!("Failed to save state: {}", e))?;

        let path = get_save_state_path(slot)?;
        std::fs::write(&path, state_data)
            .map_err(|e| format!("Failed to write state file: {}", e))?;

        *state.current_save_slot.lock() = slot;
        println!("State saved to slot {}: {:?}", slot, path);
        Ok(())
    } else {
        Err("Emulator not initialized".to_string())
    }
}

/// Load state from a slot
#[tauri::command]
pub fn emulator_load_state(
    state: State<'_, EmbeddedEmulatorState>,
    slot: u8,
) -> Result<(), String> {
    let path = get_save_state_path(slot)?;

    if !path.exists() {
        return Err(format!("No save state in slot {}", slot));
    }

    let state_data =
        std::fs::read(&path).map_err(|e| format!("Failed to read state file: {}", e))?;

    let mut core_guard = state.core.lock();
    if let Some(ref mut core) = *core_guard {
        core.load_state(&state_data)
            .map_err(|e| format!("Failed to load state: {}", e))?;

        *state.current_save_slot.lock() = slot;
        println!("State loaded from slot {}: {:?}", slot, path);
        Ok(())
    } else {
        Err("Emulator not initialized".to_string())
    }
}

/// Set emulation speed
#[tauri::command]
pub fn emulator_set_speed(state: State<'_, EmbeddedEmulatorState>, speed: f32) {
    let clamped_speed = speed.clamp(0.25, 4.0);
    *state.speed.lock() = clamped_speed;
    println!("Emulation speed set to {}x", clamped_speed);
}

/// Advance one frame (for debugging/frame stepping)
#[tauri::command]
pub fn emulator_advance_frame(state: State<'_, EmbeddedEmulatorState>) -> Result<(), String> {
    let mut core_guard = state.core.lock();
    if let Some(ref mut core) = *core_guard {
        core.run_frame();

        // Send frame to frontend
        if let Some(frame) = core.get_frame_buffer() {
            let _ = state.frame_sender.try_send(frame);
        }

        Ok(())
    } else {
        Err("Emulator not initialized".to_string())
    }
}

/// Get current emulator status
#[tauri::command]
pub fn emulator_get_status(state: State<'_, EmbeddedEmulatorState>) -> EmulatorStatus {
    EmulatorStatus {
        initialized: state.is_initialized(),
        running: state.is_running(),
        paused: state.is_paused(),
        speed: state.get_speed(),
        has_rom: state.loaded_rom_path.lock().is_some(),
        rom_path: state.loaded_rom_path.lock().clone(),
        current_slot: *state.current_save_slot.lock(),
    }
}

/// Shutdown the emulator and release resources
#[tauri::command]
pub fn emulator_shutdown(state: State<'_, EmbeddedEmulatorState>) {
    // Stop emulation if running
    emulator_stop(state.clone());

    // Clear the core
    *state.core.lock() = None;
    *state.loaded_rom_path.lock() = None;

    println!("Emulator shutdown complete");
}

/// Get list of available save states
#[tauri::command]
pub fn emulator_get_save_states() -> Result<Vec<(u8, String)>, String> {
    let mut states = Vec::new();

    for slot in 0..10 {
        if let Ok(path) = get_save_state_path(slot) {
            if path.exists() {
                if let Ok(metadata) = std::fs::metadata(&path) {
                    if let Ok(modified) = metadata.modified() {
                        let datetime: chrono::DateTime<chrono::Local> = modified.into();
                        states.push((slot, datetime.format("%Y-%m-%d %H:%M:%S").to_string()));
                    }
                }
            }
        }
    }

    Ok(states)
}

/// Delete a save state
#[tauri::command]
pub fn emulator_delete_save_state(slot: u8) -> Result<(), String> {
    let path = get_save_state_path(slot)?;
    if path.exists() {
        std::fs::remove_file(&path).map_err(|e| format!("Failed to delete state: {}", e))?;
    }
    Ok(())
}

/// Helper: Parse offset string (hex or decimal)
fn parse_offset(s: &str) -> Result<usize, String> {
    if s.starts_with("0x") || s.starts_with("0X") {
        usize::from_str_radix(&s[2..], 16).map_err(|e| e.to_string())
    } else {
        s.parse::<usize>().map_err(|e| e.to_string())
    }
}

/// Helper: Get save state path for a slot
fn get_save_state_path(slot: u8) -> Result<PathBuf, String> {
    let config_dir = dirs::config_dir()
        .ok_or("Could not find config directory")?
        .join("super-punch-out-editor")
        .join("savestates");

    std::fs::create_dir_all(&config_dir)
        .map_err(|e| format!("Failed to create save state directory: {}", e))?;

    Ok(config_dir.join(format!("slot_{}.s9x", slot)))
}

// ============================================================================
// Audio Output (Optional - can be enabled with cpal feature)
// ============================================================================

#[cfg(feature = "audio-output")]
pub mod audio_output {
    use super::*;
    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

    /// Initialize audio output for the emulator
    pub fn init_audio_output(state: &EmbeddedEmulatorState) -> Result<cpal::Stream, String> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or("No audio output device available")?;

        let config = device
            .default_output_config()
            .map_err(|e| format!("Failed to get default output config: {}", e))?;

        let audio_receiver = state.audio_receiver.clone();

        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => {
                build_stream::<f32>(&device, &config.config(), audio_receiver)?
            }
            cpal::SampleFormat::I16 => {
                build_stream::<i16>(&device, &config.config(), audio_receiver)?
            }
            cpal::SampleFormat::U16 => {
                build_stream::<u16>(&device, &config.config(), audio_receiver)?
            }
            _ => return Err("Unsupported sample format".to_string()),
        };

        stream
            .play()
            .map_err(|e| format!("Failed to start audio stream: {}", e))?;

        Ok(stream)
    }

    fn build_stream<T: cpal::SizedSample + From<i16>>(
        device: &cpal::Device,
        config: &cpal::StreamConfig,
        audio_receiver: crossbeam_channel::Receiver<AudioBuffer>,
    ) -> Result<cpal::Stream, String> {
        let channels = config.channels as usize;

        let stream = device
            .build_output_stream(
                config,
                move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                    // Fill buffer with audio from emulator
                    let mut sample_idx = 0;

                    while sample_idx < data.len() && !audio_receiver.is_empty() {
                        if let Ok(buffer) = audio_receiver.try_recv() {
                            for sample in buffer.samples {
                                if sample_idx >= data.len() {
                                    break;
                                }
                                // Convert i16 to target format
                                data[sample_idx] = T::from(sample);
                                sample_idx += 1;

                                // Duplicate for stereo if needed
                                if channels > 1 && sample_idx < data.len() {
                                    data[sample_idx] = T::from(sample);
                                    sample_idx += 1;
                                }
                            }
                        }
                    }

                    // Fill remaining with silence
                    for i in sample_idx..data.len() {
                        data[i] = T::from(0);
                    }
                },
                move |err| {
                    eprintln!("Audio error: {}", err);
                },
                None,
            )
            .map_err(|e| format!("Failed to build audio stream: {}", e))?;

        Ok(stream)
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_controller_input_to_buttons() {
        let input = ControllerInput {
            b: true,
            y: true,
            select: false,
            start: true,
            up: false,
            down: true,
            left: false,
            right: true,
            a: false,
            x: true,
            l: false,
            r: true,
        };

        let buttons = input.to_buttons();

        // Verify individual bits
        assert!(buttons & 0x8000 != 0, "B button");
        assert!(buttons & 0x4000 != 0, "Y button");
        assert!(buttons & 0x2000 == 0, "Select button should be off");
        assert!(buttons & 0x1000 != 0, "Start button");
        assert!(buttons & 0x0400 != 0, "Down button");
        assert!(buttons & 0x0100 != 0, "Right button");
        assert!(buttons & 0x0040 != 0, "X button");
        assert!(buttons & 0x0010 != 0, "R button");
    }

    #[test]
    fn test_parse_offset() {
        assert_eq!(parse_offset("0x1234").unwrap(), 0x1234);
        assert_eq!(parse_offset("4660").unwrap(), 4660); // 0x1234 in decimal
        assert_eq!(parse_offset("0xABC").unwrap(), 0xABC);
    }

    #[test]
    fn test_emulator_status_serialization() {
        let status = EmulatorStatus {
            initialized: true,
            running: true,
            paused: false,
            speed: 1.5,
            has_rom: true,
            rom_path: Some("test.sfc".to_string()),
            current_slot: 3,
        };

        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("initialized"));
        assert!(json.contains("true"));
    }
}
