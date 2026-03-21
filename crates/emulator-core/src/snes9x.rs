//! # Snes9x Core Implementation
//!
//! Snes9x emulator wrapper for the Super Punch-Out!! editor.

use crate::audio::{AudioBatch, AudioBuffer};
use crate::input::{InputManager, SnesController};
use crate::libretro;
use crate::libretro_runtime::{clear_callback_targets, LibretroCore};
use crate::video::{PixelFormat, VideoBuffer, VideoFrame};
use crate::{EmulatorError, Result};
use rom_core::{
    SpoTextEncoder,
    roster::{CircuitType, INTRO_FIELD_SIZE, MAX_NAME_LENGTH, RosterLoader, RosterWriter},
};
use serde::{Deserialize, Serialize};

use std::path::PathBuf;
use std::sync::Arc;

/// Core configuration options
#[derive(Debug, Clone)]
pub struct CoreConfig {
    pub core_path: String,
    pub enable_audio: bool,
    pub target_fps: f64,
    pub pixel_format: PixelFormat,
    pub frame_limit: bool,
    pub enable_states: bool,
}

impl Default for CoreConfig {
    fn default() -> Self {
        Self {
            core_path: default_core_path(),
            enable_audio: true,
            target_fps: 60.098,
            pixel_format: PixelFormat::FormatXRGB8888,
            frame_limit: true,
            enable_states: true,
        }
    }
}

fn default_core_path() -> String {
    #[cfg(target_os = "windows")]
    return "snes9x_libretro.dll".to_string();
    #[cfg(target_os = "linux")]
    return "snes9x_libretro.so".to_string();
    #[cfg(target_os = "macos")]
    return "snes9x_libretro.dylib".to_string();
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    return "snes9x_libretro".to_string();
}

#[cfg(test)]
const WRAM_SIZE: usize = 0x20_000;
const CREATOR_MODE_FLAG: usize = 0x1FFF;
const CREATOR_MODE_MAGIC: usize = 0x1FFE;
const CREATOR_MODE_HEARTBEAT: usize = 0x1FFD;
const CREATOR_MODE_INPUT_LOW: usize = 0x1FFC;
const CREATOR_MODE_INPUT_HIGH: usize = 0x1FFB;
const CREATOR_MODE_CURSOR: usize = 0x1FFA;
const CREATOR_MODE_ACTION: usize = 0x1FF9;
const CREATOR_MODE_PAGE: usize = 0x1FF8;
const CREATOR_MODE_DIRTY: usize = 0x1FF7;
const CREATOR_RENDER_VISIBLE: usize = 0x1FF6;
const CREATOR_RENDER_PAGE: usize = 0x1FF5;
const CREATOR_RENDER_CURSOR: usize = 0x1FF4;
const CREATOR_RENDER_ROW0: usize = 0x1FF3;
const CREATOR_RENDER_ROW1: usize = 0x1FF2;
const CREATOR_RENDER_ROW2: usize = 0x1FF1;
const CREATOR_RENDER_ROW3: usize = 0x1FF0;
const CREATOR_RENDER_REVISION: usize = 0x1FEF;
const CREATOR_SESSION_PRESENT: usize = 0x1FEE;
const CREATOR_SESSION_BOXER_ID: usize = 0x1FED;
const CREATOR_SESSION_CIRCUIT: usize = 0x1FEC;
const CREATOR_SESSION_UNLOCK_ORDER: usize = 0x1FEB;
const CREATOR_SESSION_INTRO_TEXT_ID: usize = 0x1FEA;
const CREATOR_SESSION_STATUS: usize = 0x1FE9;
const CREATOR_SESSION_ERROR_CODE: usize = 0x1FE8;
const CREATOR_NAME_EDIT_ACTIVE: usize = 0x1FE7;
const CREATOR_NAME_CURSOR: usize = 0x1FE6;
const CREATOR_NAME_LENGTH: usize = 0x1FE5;
const CREATOR_INTRO_EDIT_ACTIVE: usize = 0x1FE4;
const CREATOR_INTRO_CURSOR: usize = 0x1FE3;
const CREATOR_INTRO_LENGTH: usize = 0x1FE2;
const CREATOR_INTRO_BUFFER_START: usize = 0x1FD0;
const CREATOR_INTRO_BUFFER_LEN: usize = 16;
const CREATOR_NAME_BUFFER_START: usize = 0x1FC0;
const CREATOR_NAME_BUFFER_LEN: usize = 16;

const CREATOR_MODE_LOW_MASK: u8 = 0x0C; // Select + Start
const CREATOR_MODE_HIGH_MASK: u8 = 0x0C; // L + R
const CREATOR_MODE_MAGIC_VALUE: u8 = 0x43; // 'C'
const CREATOR_MODE_HEARTBEAT_VALUE: u8 = 0xA5;
const CREATOR_MODE_PAGE_MAX: u8 = 0x03;
const CREATOR_MODE_CURSOR_MAX: u8 = 0x03;
const CREATOR_ACTION_NAME_EDIT: u8 = 0x11;
const CREATOR_ACTION_CIRCUIT_EDIT: u8 = 0x12;
const CREATOR_ACTION_PORTRAIT_EDIT: u8 = 0x13;
const CREATOR_ACTION_COMMIT: u8 = 0x14;
const CREATOR_ACTION_INTRO_EDIT: u8 = 0x15;
const CREATOR_ACTION_CANCEL: u8 = 0x16;
const CREATOR_ACTION_EXIT: u8 = 0xFF;
const CREATOR_COMBO_INPUT: u16 = 0x2000 | 0x1000 | 0x0020 | 0x0010;
const CREATOR_SESSION_STATUS_DRAFT_READY: u8 = 0x02;
const CREATOR_SESSION_STATUS_COMMIT_PENDING: u8 = 0x03;
const CREATOR_SESSION_STATUS_COMMIT_SUCCEEDED: u8 = 0x04;
const CREATOR_SESSION_STATUS_COMMIT_FAILED: u8 = 0x05;
const CREATOR_SESSION_STATUS_CANCELLED: u8 = 0x07;
const CREATOR_ERROR_GENERIC: u8 = 0x01;
const CREATOR_ERROR_BOXER_NOT_FOUND: u8 = 0x02;
const CREATOR_ERROR_INVALID_NAME: u8 = 0x03;
const CREATOR_ERROR_INVALID_INTRO_TEXT: u8 = 0x04;
const CREATOR_ERROR_INVALID_INTRO_SLOT: u8 = 0x05;

const CREATOR_PAGE0_ROWS: [u8; 4] = [0x21, 0x22, 0x23, 0x24];
const CREATOR_PAGE1_ROWS: [u8; 4] = [0x31, 0x32, 0x33, 0x34];
const CREATOR_PAGE2_ROWS: [u8; 4] = [0x41, 0x42, 0x43, 0x44];
const CREATOR_PAGE3_ROWS: [u8; 4] = [0x51, 0x52, 0x53, 0x54];

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct CreatorRuntimeState {
    pub active: bool,
    pub magic: u8,
    pub heartbeat: u8,
    pub input_low: u8,
    pub input_high: u8,
    pub cursor: u8,
    pub action: u8,
    pub page: u8,
    pub dirty: bool,
    pub render_visible: bool,
    pub render_page: u8,
    pub render_cursor: u8,
    pub render_rows: [u8; 4],
    pub render_revision: u8,
    pub session_present: bool,
    pub session_boxer_id: u8,
    pub session_circuit: u8,
    pub session_unlock_order: u8,
    pub session_intro_text_id: u8,
    pub session_status: u8,
    pub session_error_code: u8,
    pub intro_edit_active: bool,
    pub intro_cursor: u8,
    pub intro_length: u8,
    pub intro_bytes: Vec<u8>,
    pub name_edit_active: bool,
    pub name_cursor: u8,
    pub name_length: u8,
    pub name_bytes: Vec<u8>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct CreatorSessionState {
    pub boxer_id: u8,
    pub circuit: u8,
    pub unlock_order: u8,
    pub intro_text_id: u8,
    pub status: u8,
    pub error_code: u8,
    pub intro_text: String,
    pub name_text: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct CreatorRuntimeActionResolution {
    pub runtime_state: CreatorRuntimeState,
    pub message: Option<String>,
    pub rom_updated: bool,
}

#[derive(Debug, Clone)]
struct CreatorSessionValidation {
    valid: bool,
    status: u8,
    error_code: u8,
    message: Option<String>,
}

/// The main Snes9x emulator core
pub struct Snes9xCore {
    config: CoreConfig,
    video_buffer: Arc<VideoBuffer>,
    audio_buffer: Arc<AudioBuffer>,
    input_manager: Arc<InputManager>,
    rom_loaded: bool,
    rom_data: Option<Vec<u8>>,
    frame_count: u64,
    runtime_backend: RuntimeBackend,
    current_input: u16,
    #[allow(dead_code)]
    save_dir: PathBuf,
}

enum RuntimeBackend {
    Libretro(LibretroCore),
    #[allow(dead_code)]
    Stub { system_ram: Vec<u8> },
}

/// Handle to the emulation thread
pub struct EmulationThread {
    running: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl EmulationThread {
    pub fn stop(&self) {
        self.running
            .store(false, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn is_running(&self) -> bool {
        self.running.load(std::sync::atomic::Ordering::SeqCst)
    }
}

impl Snes9xCore {
    pub fn new(core_path: &str) -> Result<Self> {
        let config = CoreConfig {
            core_path: core_path.to_string(),
            ..Default::default()
        };
        Self::with_config(config)
    }

    pub fn with_config(config: CoreConfig) -> Result<Self> {
        let video_buffer = Arc::new(VideoBuffer::new());
        let audio_buffer = Arc::new(AudioBuffer::new());
        let input_manager = Arc::new(InputManager::new());

        let save_dir = dirs::config_dir()
            .map(|d| d.join("super-punch-out-editor").join("states"))
            .unwrap_or_else(|| PathBuf::from("./states"));

        let _ = std::fs::create_dir_all(&save_dir);

        video_buffer.set_format(config.pixel_format);
        if !config.enable_audio {
            audio_buffer.disable();
        }

        let runtime_backend =
            match LibretroCore::load(&config.core_path, &video_buffer, &audio_buffer, &input_manager)
            {
                Ok(core) => RuntimeBackend::Libretro(core),
                Err(error) => {
                    clear_callback_targets();
                    return Err(error);
                }
            };

        Ok(Self {
            config,
            video_buffer,
            audio_buffer,
            input_manager,
            rom_loaded: false,
            rom_data: None,
            frame_count: 0,
            runtime_backend,
            current_input: 0,
            save_dir,
        })
    }

    #[cfg(test)]
    fn new_stub() -> Self {
        let config = CoreConfig {
            core_path: "__stub__".to_string(),
            ..Default::default()
        };
        let video_buffer = Arc::new(VideoBuffer::new());
        let audio_buffer = Arc::new(AudioBuffer::new());
        let input_manager = Arc::new(InputManager::new());
        let save_dir = PathBuf::from("./states");

        Self {
            config,
            video_buffer,
            audio_buffer,
            input_manager,
            rom_loaded: false,
            rom_data: None,
            frame_count: 0,
            runtime_backend: RuntimeBackend::Stub {
                system_ram: vec![0; WRAM_SIZE],
            },
            current_input: 0,
            save_dir,
        }
    }

    pub fn load_rom(&mut self, rom_data: impl Into<Vec<u8>>) -> Result<()> {
        let data: Vec<u8> = rom_data.into();
        if data.len() < 0x8000 {
            return Err(EmulatorError::InvalidRomData);
        }

        if self.rom_loaded {
            self.unload_rom();
        }

        self.rom_data = Some(data);
        self.input_manager.clear_all();
        self.current_input = 0;
        self.audio_buffer.clear();
        self.video_buffer.clear();

        if let RuntimeBackend::Libretro(core) = &mut self.runtime_backend {
            let rom = self
                .rom_data
                .as_ref()
                .ok_or_else(|| EmulatorError::RomLoadError("ROM payload missing".to_string()))?;
            core.load_rom(rom, &self.video_buffer, &self.audio_buffer)?;
        } else {
            self.reset_creator_runtime();
        }

        self.rom_loaded = true;
        Ok(())
    }

    pub fn run_frame(&mut self) {
        if !self.rom_loaded {
            return;
        }

        self.frame_count += 1;
        if let RuntimeBackend::Libretro(core) = &mut self.runtime_backend {
            core.run_frame();
        } else {
            self.update_creator_runtime();
        }
    }

    pub fn get_frame(&self) -> Option<VideoFrame> {
        self.video_buffer.get_frame()
    }

    pub fn get_frame_buffer(&self) -> Option<VideoFrame> {
        self.get_frame()
    }

    pub fn get_audio_samples(&self) -> Option<AudioBatch> {
        let samples = self.audio_buffer.drain_samples();
        if samples.is_empty() {
            return None;
        }

        let config = self.audio_buffer.get_config();
        Some(AudioBatch::new(samples, config.sample_rate, config.channels))
    }

    pub fn get_audio_samples_vec(&self) -> Vec<i16> {
        self.audio_buffer.drain_samples()
    }

    pub fn set_input(&mut self, port: u32, buttons: u16) {
        let controller = SnesController::from_mask(buttons);
        self.input_manager
            .set_controller_state(port as usize, controller);
        if port == 0 {
            self.current_input = buttons;
        }
    }

    pub fn reset(&mut self) {
        self.reset_soft();
    }

    pub fn reset_soft(&mut self) {
        self.frame_count = 0;
        self.current_input = 0;
        self.input_manager.clear_all();
        self.audio_buffer.clear();

        if let RuntimeBackend::Libretro(core) = &mut self.runtime_backend {
            if self.rom_loaded {
                core.reset();
            }
        } else {
            self.reset_creator_runtime();
        }
    }

    pub fn reset_hard(&mut self) {
        self.frame_count = 0;
        if let Some(data) = self.rom_data.clone() {
            let _ = self.load_rom(data);
        } else {
            self.reset_soft();
        }
    }

    pub fn save_state(&self) -> Result<Vec<u8>> {
        if !self.rom_loaded {
            return Err(EmulatorError::NotInitialized);
        }

        match &self.runtime_backend {
            RuntimeBackend::Libretro(core) => core.save_state(),
            RuntimeBackend::Stub { .. } => Ok(vec![0u8; 1024]),
        }
    }

    pub fn load_state(&mut self, state_data: &[u8]) -> Result<()> {
        if !self.rom_loaded {
            return Err(EmulatorError::NotInitialized);
        }

        match &mut self.runtime_backend {
            RuntimeBackend::Libretro(core) => core.load_state(state_data),
            RuntimeBackend::Stub { .. } => Ok(()),
        }
    }

    pub fn save_state_size(&self) -> Result<usize> {
        if !self.rom_loaded {
            return Err(EmulatorError::NotInitialized);
        }

        match &self.runtime_backend {
            RuntimeBackend::Libretro(core) => Ok(core.save_state_size()),
            RuntimeBackend::Stub { .. } => Ok(512 * 1024),
        }
    }

    pub fn is_rom_loaded(&self) -> bool {
        self.rom_loaded
    }

    pub fn config(&self) -> CoreConfig {
        self.config.clone()
    }

    pub fn set_config(&mut self, config: CoreConfig) {
        self.config = config.clone();
        self.video_buffer.set_format(config.pixel_format);
        if config.enable_audio {
            self.audio_buffer.enable();
        } else {
            self.audio_buffer.disable();
        }
    }

    pub fn video_buffer(&self) -> &Arc<VideoBuffer> {
        &self.video_buffer
    }

    pub fn audio_buffer(&self) -> &Arc<AudioBuffer> {
        &self.audio_buffer
    }

    pub fn input_manager(&self) -> &Arc<InputManager> {
        &self.input_manager
    }

    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }

    pub fn creator_runtime_state(&self) -> CreatorRuntimeState {
        CreatorRuntimeState {
            active: self.read_wram_byte(CREATOR_MODE_FLAG) != 0,
            magic: self.read_wram_byte(CREATOR_MODE_MAGIC),
            heartbeat: self.read_wram_byte(CREATOR_MODE_HEARTBEAT),
            input_low: self.read_wram_byte(CREATOR_MODE_INPUT_LOW),
            input_high: self.read_wram_byte(CREATOR_MODE_INPUT_HIGH),
            cursor: self.read_wram_byte(CREATOR_MODE_CURSOR),
            action: self.read_wram_byte(CREATOR_MODE_ACTION),
            page: self.read_wram_byte(CREATOR_MODE_PAGE),
            dirty: self.read_wram_byte(CREATOR_MODE_DIRTY) != 0,
            render_visible: self.read_wram_byte(CREATOR_RENDER_VISIBLE) != 0,
            render_page: self.read_wram_byte(CREATOR_RENDER_PAGE),
            render_cursor: self.read_wram_byte(CREATOR_RENDER_CURSOR),
            render_rows: [
                self.read_wram_byte(CREATOR_RENDER_ROW0),
                self.read_wram_byte(CREATOR_RENDER_ROW1),
                self.read_wram_byte(CREATOR_RENDER_ROW2),
                self.read_wram_byte(CREATOR_RENDER_ROW3),
            ],
            render_revision: self.read_wram_byte(CREATOR_RENDER_REVISION),
            session_present: self.read_wram_byte(CREATOR_SESSION_PRESENT) != 0,
            session_boxer_id: self.read_wram_byte(CREATOR_SESSION_BOXER_ID),
            session_circuit: self.read_wram_byte(CREATOR_SESSION_CIRCUIT),
            session_unlock_order: self.read_wram_byte(CREATOR_SESSION_UNLOCK_ORDER),
            session_intro_text_id: self.read_wram_byte(CREATOR_SESSION_INTRO_TEXT_ID),
            session_status: self.read_wram_byte(CREATOR_SESSION_STATUS),
            session_error_code: self.read_wram_byte(CREATOR_SESSION_ERROR_CODE),
            intro_edit_active: self.read_wram_byte(CREATOR_INTRO_EDIT_ACTIVE) != 0,
            intro_cursor: self.read_wram_byte(CREATOR_INTRO_CURSOR),
            intro_length: self.read_wram_byte(CREATOR_INTRO_LENGTH),
            intro_bytes: self
                .read_system_ram(CREATOR_INTRO_BUFFER_START, CREATOR_INTRO_BUFFER_LEN)
                .unwrap_or_else(|| vec![0; CREATOR_INTRO_BUFFER_LEN]),
            name_edit_active: self.read_wram_byte(CREATOR_NAME_EDIT_ACTIVE) != 0,
            name_cursor: self.read_wram_byte(CREATOR_NAME_CURSOR),
            name_length: self.read_wram_byte(CREATOR_NAME_LENGTH),
            name_bytes: self
                .read_system_ram(CREATOR_NAME_BUFFER_START, CREATOR_NAME_BUFFER_LEN)
                .unwrap_or_else(|| vec![0; CREATOR_NAME_BUFFER_LEN]),
        }
    }

    pub fn set_creator_session_state(&mut self, session: &CreatorSessionState) -> bool {
        let mut intro_bytes = vec![b' '; CREATOR_INTRO_BUFFER_LEN];
        let intro_source_bytes = session.intro_text.as_bytes();
        let intro_copy_len = intro_source_bytes.len().min(CREATOR_INTRO_BUFFER_LEN);
        intro_bytes[..intro_copy_len].copy_from_slice(&intro_source_bytes[..intro_copy_len]);

        let mut name_bytes = vec![b' '; CREATOR_NAME_BUFFER_LEN];
        let source_bytes = session.name_text.as_bytes();
        let copy_len = source_bytes.len().min(CREATOR_NAME_BUFFER_LEN);
        name_bytes[..copy_len].copy_from_slice(&source_bytes[..copy_len]);

        self.write_wram(CREATOR_SESSION_PRESENT, &[1])
            && self.write_wram(CREATOR_SESSION_BOXER_ID, &[session.boxer_id])
            && self.write_wram(CREATOR_SESSION_CIRCUIT, &[session.circuit])
            && self.write_wram(CREATOR_SESSION_UNLOCK_ORDER, &[session.unlock_order])
            && self.write_wram(CREATOR_SESSION_INTRO_TEXT_ID, &[session.intro_text_id])
            && self.write_wram(CREATOR_SESSION_STATUS, &[session.status])
            && self.write_wram(CREATOR_SESSION_ERROR_CODE, &[session.error_code])
            && self.write_wram(CREATOR_INTRO_EDIT_ACTIVE, &[0])
            && self.write_wram(CREATOR_INTRO_CURSOR, &[0])
            && self.write_wram(CREATOR_INTRO_LENGTH, &[intro_copy_len as u8])
            && self.write_wram(CREATOR_INTRO_BUFFER_START, &intro_bytes)
            && self.write_wram(CREATOR_NAME_EDIT_ACTIVE, &[0])
            && self.write_wram(CREATOR_NAME_CURSOR, &[0])
            && self.write_wram(CREATOR_NAME_LENGTH, &[copy_len as u8])
            && self.write_wram(CREATOR_NAME_BUFFER_START, &name_bytes)
    }

    pub fn clear_creator_session_state(&mut self) -> bool {
        self.write_wram(CREATOR_SESSION_PRESENT, &[0])
            && self.write_wram(CREATOR_SESSION_BOXER_ID, &[0])
            && self.write_wram(CREATOR_SESSION_CIRCUIT, &[0])
            && self.write_wram(CREATOR_SESSION_UNLOCK_ORDER, &[0])
            && self.write_wram(CREATOR_SESSION_INTRO_TEXT_ID, &[0])
            && self.write_wram(CREATOR_SESSION_STATUS, &[0])
            && self.write_wram(CREATOR_SESSION_ERROR_CODE, &[0])
            && self.write_wram(CREATOR_INTRO_EDIT_ACTIVE, &[0])
            && self.write_wram(CREATOR_INTRO_CURSOR, &[0])
            && self.write_wram(CREATOR_INTRO_LENGTH, &[0])
            && self.write_wram(CREATOR_INTRO_BUFFER_START, &[0; CREATOR_INTRO_BUFFER_LEN])
            && self.write_wram(CREATOR_NAME_EDIT_ACTIVE, &[0])
            && self.write_wram(CREATOR_NAME_CURSOR, &[0])
            && self.write_wram(CREATOR_NAME_LENGTH, &[0])
            && self.write_wram(CREATOR_NAME_BUFFER_START, &[0; CREATOR_NAME_BUFFER_LEN])
    }

    pub fn write_creator_action(&mut self, action: u8) -> bool {
        self.write_wram(CREATOR_MODE_ACTION, &[action])
    }

    pub fn current_rom_image(&self) -> Option<Vec<u8>> {
        self.rom_data.clone()
    }

    pub fn resolve_creator_runtime_action(&mut self) -> Result<CreatorRuntimeActionResolution> {
        let runtime = self.creator_runtime_state();
        if !runtime.session_present {
            return Err(EmulatorError::StateError(
                "No creator session present in emulator runtime".to_string(),
            ));
        }

        let session = creator_session_from_runtime(&runtime);

        match runtime.action {
            CREATOR_ACTION_COMMIT => {
                let rom_bytes = self
                    .rom_data
                    .clone()
                    .ok_or(EmulatorError::NotInitialized)?;
                let mut rom = rom_core::Rom::new(rom_bytes);
                let validation = validate_creator_session_payload(&rom, &session);

                if !validation.valid {
                    let failed_session = CreatorSessionState {
                        status: validation.status,
                        error_code: validation.error_code,
                        ..session
                    };
                    if !self.set_creator_session_state(&failed_session) || !self.write_creator_action(0)
                    {
                        return Err(EmulatorError::StateError(
                            "Failed to update creator runtime after validation failure".to_string(),
                        ));
                    }

                    return Ok(CreatorRuntimeActionResolution {
                        runtime_state: self.creator_runtime_state(),
                        message: validation.message,
                        rom_updated: false,
                    });
                }

                commit_creator_session_to_rom(&mut rom, &session).map_err(EmulatorError::RomLoadError)?;
                let refreshed_session = load_creator_session_from_rom(
                    &rom,
                    session.boxer_id,
                    CREATOR_SESSION_STATUS_COMMIT_SUCCEEDED,
                    0,
                )
                .map_err(EmulatorError::RomLoadError)?;
                let updated_rom = rom.data.clone();

                self.load_rom(updated_rom)?;
                if !self.set_creator_session_state(&refreshed_session) {
                    return Err(EmulatorError::StateError(
                        "Failed to reseed creator session after commit".to_string(),
                    ));
                }

                self.set_input(0, CREATOR_COMBO_INPUT);
                self.run_frame();
                self.set_input(0, 0);
                self.run_frame();
                let _ = self.write_creator_action(0);

                Ok(CreatorRuntimeActionResolution {
                    runtime_state: self.creator_runtime_state(),
                    message: Some(format!(
                        "Committed slot #{} and reloaded the edited ROM.",
                        session.boxer_id
                    )),
                    rom_updated: true,
                })
            }
            CREATOR_ACTION_CANCEL => {
                let rom_bytes = self
                    .rom_data
                    .clone()
                    .ok_or(EmulatorError::NotInitialized)?;
                let rom = rom_core::Rom::new(rom_bytes);
                let refreshed_session = load_creator_session_from_rom(
                    &rom,
                    session.boxer_id,
                    CREATOR_SESSION_STATUS_CANCELLED,
                    0,
                )
                .map_err(EmulatorError::RomLoadError)?;
                if !self.set_creator_session_state(&refreshed_session) || !self.write_creator_action(0)
                {
                    return Err(EmulatorError::StateError(
                        "Failed to restore creator session from ROM".to_string(),
                    ));
                }

                Ok(CreatorRuntimeActionResolution {
                    runtime_state: self.creator_runtime_state(),
                    message: Some(format!("Reverted slot #{} to ROM values.", session.boxer_id)),
                    rom_updated: false,
                })
            }
            _ => Ok(CreatorRuntimeActionResolution {
                runtime_state: runtime,
                message: None,
                rom_updated: false,
            }),
        }
    }

    pub fn read_system_ram(&self, offset: usize, len: usize) -> Option<Vec<u8>> {
        match &self.runtime_backend {
            RuntimeBackend::Libretro(core) => {
                core.read_memory(libretro::RETRO_MEMORY_SYSTEM_RAM, offset, len)
            }
            RuntimeBackend::Stub { system_ram } => {
                let end = offset.checked_add(len)?;
                system_ram.get(offset..end).map(|slice| slice.to_vec())
            }
        }
    }

    pub fn get_frame_dimensions(&self) -> (u32, u32) {
        self.video_buffer.get_dimensions()
    }

    pub fn unload_rom(&mut self) {
        self.rom_loaded = false;
        self.rom_data = None;
        self.current_input = 0;
        self.input_manager.clear_all();
        self.audio_buffer.clear();
        self.video_buffer.clear();

        if let RuntimeBackend::Libretro(core) = &mut self.runtime_backend {
            core.unload_game();
        } else {
            self.reset_creator_runtime();
        }
    }

    fn read_wram_byte(&self, offset: usize) -> u8 {
        self.read_system_ram(offset, 1)
            .and_then(|bytes| bytes.first().copied())
            .unwrap_or(0)
    }

    fn write_wram(&mut self, offset: usize, bytes: &[u8]) -> bool {
        match &mut self.runtime_backend {
            RuntimeBackend::Libretro(core) => {
                core.write_memory(libretro::RETRO_MEMORY_SYSTEM_RAM, offset, bytes)
            }
            RuntimeBackend::Stub { system_ram } => {
                let end = match offset.checked_add(bytes.len()) {
                    Some(end) => end,
                    None => return false,
                };
                let Some(slice) = system_ram.get_mut(offset..end) else {
                    return false;
                };
                slice.copy_from_slice(bytes);
                true
            }
        }
    }

    fn reset_creator_runtime(&mut self) {
        if let RuntimeBackend::Stub { system_ram } = &mut self.runtime_backend {
            system_ram.fill(0);
        }
        self.current_input = 0;
    }

    fn update_creator_runtime(&mut self) {
        let RuntimeBackend::Stub { .. } = &self.runtime_backend else {
            return;
        };

        let (input_low, input_high) = creator_input_bytes(self.current_input);
        self.set_wram(CREATOR_MODE_INPUT_LOW, input_low);
        self.set_wram(CREATOR_MODE_INPUT_HIGH, input_high);

        if (input_low & CREATOR_MODE_LOW_MASK) == CREATOR_MODE_LOW_MASK
            && (input_high & CREATOR_MODE_HIGH_MASK) == CREATOR_MODE_HIGH_MASK
        {
            self.set_wram(CREATOR_MODE_FLAG, 1);
            self.set_wram(CREATOR_MODE_MAGIC, CREATOR_MODE_MAGIC_VALUE);
            self.set_wram(CREATOR_MODE_CURSOR, 0);
            self.set_wram(CREATOR_MODE_ACTION, 0);
            self.set_wram(CREATOR_MODE_PAGE, 0);
            self.set_wram(CREATOR_MODE_DIRTY, 1);
        }

        if self.get_wram(CREATOR_MODE_FLAG) == 0 {
            self.set_wram(CREATOR_RENDER_VISIBLE, 0);
            return;
        }

        self.set_wram(CREATOR_MODE_HEARTBEAT, CREATOR_MODE_HEARTBEAT_VALUE);
        self.set_wram(CREATOR_MODE_ACTION, 0);

        let intro_editing = self.get_wram(CREATOR_INTRO_EDIT_ACTIVE) != 0;
        let name_editing = self.get_wram(CREATOR_NAME_EDIT_ACTIVE) != 0;

        if intro_editing {
            if (input_low & 0x40) != 0 {
                let cursor = self.get_wram(CREATOR_INTRO_CURSOR);
                if cursor > 0 {
                    self.set_wram(CREATOR_INTRO_CURSOR, cursor - 1);
                    self.set_wram(CREATOR_MODE_DIRTY, 1);
                }
            }

            if (input_low & 0x80) != 0 {
                let cursor = self.get_wram(CREATOR_INTRO_CURSOR);
                if cursor < (CREATOR_INTRO_BUFFER_LEN as u8).saturating_sub(1) {
                    self.set_wram(CREATOR_INTRO_CURSOR, cursor + 1);
                    self.set_wram(CREATOR_MODE_DIRTY, 1);
                }
            }

            if (input_low & 0x10) != 0 || (input_low & 0x20) != 0 {
                let cursor = self.get_wram(CREATOR_INTRO_CURSOR) as usize;
                let current = self.read_wram_byte(CREATOR_INTRO_BUFFER_START + cursor);
                let next = if (input_low & 0x10) != 0 {
                    cycle_creator_name_char(current, 1)
                } else {
                    cycle_creator_name_char(current, -1)
                };
                self.set_wram(CREATOR_INTRO_BUFFER_START + cursor, next);
                self.set_wram(CREATOR_SESSION_STATUS, CREATOR_SESSION_STATUS_DRAFT_READY);
                self.set_wram(CREATOR_SESSION_ERROR_CODE, 0);
                self.set_wram(CREATOR_MODE_ACTION, CREATOR_ACTION_INTRO_EDIT);
                self.set_wram(CREATOR_MODE_DIRTY, 1);
            }

            if (input_high & 0x01) != 0 {
                let cursor = self.get_wram(CREATOR_INTRO_CURSOR);
                if cursor < (CREATOR_INTRO_BUFFER_LEN as u8).saturating_sub(1) {
                    self.set_wram(CREATOR_INTRO_CURSOR, cursor + 1);
                } else {
                    self.set_wram(CREATOR_INTRO_EDIT_ACTIVE, 0);
                }
                self.set_wram(CREATOR_MODE_ACTION, CREATOR_ACTION_INTRO_EDIT);
                self.set_wram(CREATOR_MODE_DIRTY, 1);
            }

            if (input_low & 0x01) != 0 {
                self.set_wram(CREATOR_INTRO_EDIT_ACTIVE, 0);
                self.set_wram(CREATOR_MODE_DIRTY, 1);
            }
        } else if name_editing {
            if (input_low & 0x40) != 0 {
                let cursor = self.get_wram(CREATOR_NAME_CURSOR);
                if cursor > 0 {
                    self.set_wram(CREATOR_NAME_CURSOR, cursor - 1);
                    self.set_wram(CREATOR_MODE_DIRTY, 1);
                }
            }

            if (input_low & 0x80) != 0 {
                let cursor = self.get_wram(CREATOR_NAME_CURSOR);
                if cursor < (CREATOR_NAME_BUFFER_LEN as u8).saturating_sub(1) {
                    self.set_wram(CREATOR_NAME_CURSOR, cursor + 1);
                    self.set_wram(CREATOR_MODE_DIRTY, 1);
                }
            }

            if (input_low & 0x10) != 0 || (input_low & 0x20) != 0 {
                let cursor = self.get_wram(CREATOR_NAME_CURSOR) as usize;
                let current = self.read_wram_byte(CREATOR_NAME_BUFFER_START + cursor);
                let next = if (input_low & 0x10) != 0 {
                    cycle_creator_name_char(current, 1)
                } else {
                    cycle_creator_name_char(current, -1)
                };
                self.set_wram(CREATOR_NAME_BUFFER_START + cursor, next);
                self.set_wram(CREATOR_SESSION_STATUS, CREATOR_SESSION_STATUS_DRAFT_READY);
                self.set_wram(CREATOR_SESSION_ERROR_CODE, 0);
                self.set_wram(CREATOR_MODE_ACTION, CREATOR_ACTION_NAME_EDIT);
                self.set_wram(CREATOR_MODE_DIRTY, 1);
            }

            if (input_high & 0x01) != 0 {
                let cursor = self.get_wram(CREATOR_NAME_CURSOR);
                if cursor < (CREATOR_NAME_BUFFER_LEN as u8).saturating_sub(1) {
                    self.set_wram(CREATOR_NAME_CURSOR, cursor + 1);
                } else {
                    self.set_wram(CREATOR_NAME_EDIT_ACTIVE, 0);
                }
                self.set_wram(CREATOR_MODE_ACTION, CREATOR_ACTION_NAME_EDIT);
                self.set_wram(CREATOR_MODE_DIRTY, 1);
            }

            if (input_low & 0x01) != 0 {
                self.set_wram(CREATOR_NAME_EDIT_ACTIVE, 0);
                self.set_wram(CREATOR_MODE_DIRTY, 1);
            }
        } else {

            if (input_low & 0x40) != 0 {
                let page = self.get_wram(CREATOR_MODE_PAGE);
                let next_page = if page == 0 {
                    CREATOR_MODE_PAGE_MAX
                } else {
                    page.saturating_sub(1)
                };
                self.set_wram(CREATOR_MODE_PAGE, next_page);
                let next_cursor = if next_page == 1 {
                    self.get_wram(CREATOR_SESSION_CIRCUIT).min(CREATOR_MODE_CURSOR_MAX)
                } else {
                    0
                };
                self.set_wram(CREATOR_MODE_CURSOR, next_cursor);
                self.set_wram(CREATOR_MODE_DIRTY, 1);
            }

            if (input_low & 0x80) != 0 {
                let page = self.get_wram(CREATOR_MODE_PAGE);
                let next_page = if page >= CREATOR_MODE_PAGE_MAX {
                    0
                } else {
                    page + 1
                };
                self.set_wram(CREATOR_MODE_PAGE, next_page);
                let next_cursor = if next_page == 1 {
                    self.get_wram(CREATOR_SESSION_CIRCUIT).min(CREATOR_MODE_CURSOR_MAX)
                } else {
                    0
                };
                self.set_wram(CREATOR_MODE_CURSOR, next_cursor);
                self.set_wram(CREATOR_MODE_DIRTY, 1);
            }

            if (input_low & 0x10) != 0 {
                let cursor = self.get_wram(CREATOR_MODE_CURSOR);
                if cursor > 0 {
                    self.set_wram(CREATOR_MODE_CURSOR, cursor - 1);
                    self.set_wram(CREATOR_MODE_DIRTY, 1);
                }
            }

            if (input_low & 0x20) != 0 {
                let cursor = self.get_wram(CREATOR_MODE_CURSOR);
                if cursor < CREATOR_MODE_CURSOR_MAX {
                    self.set_wram(CREATOR_MODE_CURSOR, cursor + 1);
                    self.set_wram(CREATOR_MODE_DIRTY, 1);
                }
            }

            if (input_high & 0x01) != 0 {
                let action = match self.get_wram(CREATOR_MODE_PAGE) {
                    0 => match self.get_wram(CREATOR_MODE_CURSOR) {
                        0 => {
                            self.set_wram(CREATOR_INTRO_EDIT_ACTIVE, 0);
                            self.set_wram(CREATOR_NAME_EDIT_ACTIVE, 1);
                            self.set_wram(CREATOR_NAME_CURSOR, 0);
                            CREATOR_ACTION_NAME_EDIT
                        }
                        1 => {
                            self.set_wram(CREATOR_NAME_EDIT_ACTIVE, 0);
                            self.set_wram(CREATOR_INTRO_EDIT_ACTIVE, 1);
                            self.set_wram(CREATOR_INTRO_CURSOR, 0);
                            CREATOR_ACTION_INTRO_EDIT
                        }
                        2 => {
                            let next_unlock = self.get_wram(CREATOR_SESSION_UNLOCK_ORDER).wrapping_add(1);
                            self.set_wram(CREATOR_SESSION_UNLOCK_ORDER, next_unlock);
                            self.set_wram(CREATOR_SESSION_STATUS, CREATOR_SESSION_STATUS_DRAFT_READY);
                            self.set_wram(CREATOR_SESSION_ERROR_CODE, 0);
                            CREATOR_ACTION_NAME_EDIT
                        }
                        3 => {
                            let next_intro = self.get_wram(CREATOR_SESSION_INTRO_TEXT_ID).wrapping_add(1);
                            self.set_wram(CREATOR_SESSION_INTRO_TEXT_ID, next_intro);
                            self.set_wram(CREATOR_SESSION_STATUS, CREATOR_SESSION_STATUS_DRAFT_READY);
                            self.set_wram(CREATOR_SESSION_ERROR_CODE, 0);
                            CREATOR_ACTION_INTRO_EDIT
                        }
                        _ => CREATOR_ACTION_NAME_EDIT,
                    },
                    1 => {
                        let selected_circuit = self.get_wram(CREATOR_MODE_CURSOR).min(CREATOR_MODE_CURSOR_MAX);
                        self.set_wram(CREATOR_SESSION_CIRCUIT, selected_circuit);
                        self.set_wram(CREATOR_SESSION_STATUS, CREATOR_SESSION_STATUS_DRAFT_READY);
                        self.set_wram(CREATOR_SESSION_ERROR_CODE, 0);
                        CREATOR_ACTION_CIRCUIT_EDIT
                    }
                    2 => CREATOR_ACTION_PORTRAIT_EDIT,
                    3 => match self.get_wram(CREATOR_MODE_CURSOR) {
                        1 => {
                            self.set_wram(CREATOR_SESSION_STATUS, CREATOR_SESSION_STATUS_COMMIT_PENDING);
                            self.set_wram(CREATOR_SESSION_ERROR_CODE, 0);
                            CREATOR_ACTION_COMMIT
                        }
                        2 => {
                            self.set_wram(CREATOR_SESSION_STATUS, CREATOR_SESSION_STATUS_CANCELLED);
                            self.set_wram(CREATOR_SESSION_ERROR_CODE, 0);
                            CREATOR_ACTION_CANCEL
                        }
                        _ => 0,
                    },
                    _ => 0,
                };
                self.set_wram(CREATOR_MODE_ACTION, action);
                self.set_wram(CREATOR_MODE_DIRTY, 1);
            }
        }

        if (input_low & 0x01) != 0 && !name_editing && !intro_editing {
            self.set_wram(CREATOR_MODE_ACTION, CREATOR_ACTION_EXIT);
            self.set_wram(CREATOR_MODE_FLAG, 0);
            self.set_wram(CREATOR_RENDER_VISIBLE, 0);
            self.set_wram(CREATOR_RENDER_PAGE, 0);
            self.set_wram(CREATOR_RENDER_CURSOR, 0);
            self.write_render_rows([0; 4]);
            self.set_wram(CREATOR_MODE_DIRTY, 0);
            self.bump_render_revision();
            return;
        }

        if self.get_wram(CREATOR_MODE_DIRTY) == 0 {
            return;
        }

        let page = self.get_wram(CREATOR_MODE_PAGE);
        let cursor = self.get_wram(CREATOR_MODE_CURSOR);
        self.set_wram(CREATOR_RENDER_VISIBLE, 1);
        self.set_wram(CREATOR_RENDER_PAGE, page);
        self.set_wram(CREATOR_RENDER_CURSOR, cursor);
        self.write_render_rows(render_rows_for_page(page));
        self.bump_render_revision();
        self.set_wram(CREATOR_MODE_DIRTY, 0);
    }

    fn write_render_rows(&mut self, rows: [u8; 4]) {
        self.set_wram(CREATOR_RENDER_ROW0, rows[0]);
        self.set_wram(CREATOR_RENDER_ROW1, rows[1]);
        self.set_wram(CREATOR_RENDER_ROW2, rows[2]);
        self.set_wram(CREATOR_RENDER_ROW3, rows[3]);
    }

    fn bump_render_revision(&mut self) {
        let next = self.get_wram(CREATOR_RENDER_REVISION).wrapping_add(1);
        self.set_wram(CREATOR_RENDER_REVISION, next);
    }

    fn set_wram(&mut self, offset: usize, value: u8) {
        let _ = self.write_wram(offset, &[value]);
    }

    fn get_wram(&self, offset: usize) -> u8 {
        match &self.runtime_backend {
            RuntimeBackend::Stub { system_ram } => system_ram.get(offset).copied().unwrap_or(0),
            RuntimeBackend::Libretro(_) => 0,
        }
    }
}

impl Drop for Snes9xCore {
    fn drop(&mut self) {
        self.unload_rom();
        clear_callback_targets();
    }
}

unsafe impl Send for Snes9xCore {}
unsafe impl Sync for Snes9xCore {}

fn creator_session_from_runtime(runtime: &CreatorRuntimeState) -> CreatorSessionState {
    CreatorSessionState {
        boxer_id: runtime.session_boxer_id,
        circuit: runtime.session_circuit,
        unlock_order: runtime.session_unlock_order,
        intro_text_id: runtime.session_intro_text_id,
        status: runtime.session_status,
        error_code: runtime.session_error_code,
        intro_text: decode_creator_ascii(&runtime.intro_bytes, runtime.intro_length),
        name_text: decode_creator_ascii(&runtime.name_bytes, runtime.name_length),
    }
}

fn decode_creator_ascii(bytes: &[u8], length: u8) -> String {
    let limit = usize::from(length).min(bytes.len());
    String::from_utf8_lossy(&bytes[..limit])
        .trim_end_matches('\0')
        .trim_end()
        .to_string()
}

fn validate_creator_session_payload(
    rom: &rom_core::Rom,
    session: &CreatorSessionState,
) -> CreatorSessionValidation {
    let loader = RosterLoader::new(rom);
    let roster = match loader.load_roster() {
        Ok(roster) => roster,
        Err(error) => {
            return CreatorSessionValidation {
                valid: false,
                status: CREATOR_SESSION_STATUS_COMMIT_FAILED,
                error_code: CREATOR_ERROR_GENERIC,
                message: Some(error.to_string()),
            };
        }
    };

    if roster.get_boxer(session.boxer_id).is_none() {
        return CreatorSessionValidation {
            valid: false,
            status: CREATOR_SESSION_STATUS_COMMIT_FAILED,
            error_code: CREATOR_ERROR_BOXER_NOT_FOUND,
            message: Some(format!("Boxer with ID {} not found", session.boxer_id)),
        };
    }

    if loader.load_boxer_intro(session.intro_text_id).is_err() {
        return CreatorSessionValidation {
            valid: false,
            status: CREATOR_SESSION_STATUS_COMMIT_FAILED,
            error_code: CREATOR_ERROR_INVALID_INTRO_SLOT,
            message: Some(format!("Intro text slot {} not found", session.intro_text_id)),
        };
    }

    let encoder = SpoTextEncoder::new();
    if let Err(invalid) = encoder.validate(&session.name_text) {
        return CreatorSessionValidation {
            valid: false,
            status: CREATOR_SESSION_STATUS_COMMIT_FAILED,
            error_code: CREATOR_ERROR_INVALID_NAME,
            message: Some(format!("Invalid name characters: {:?}", invalid)),
        };
    }

    let encoded_name = encoder.encode(&session.name_text);
    if encoded_name.len() > MAX_NAME_LENGTH {
        return CreatorSessionValidation {
            valid: false,
            status: CREATOR_SESSION_STATUS_COMMIT_FAILED,
            error_code: CREATOR_ERROR_INVALID_NAME,
            message: Some(format!(
                "Name too long: {} bytes (max {})",
                encoded_name.len(),
                MAX_NAME_LENGTH
            )),
        };
    }

    if !encoder.can_encode(&session.intro_text) {
        let unsupported: Vec<char> = session
            .intro_text
            .chars()
            .filter(|c| !encoder.can_encode(&c.to_string()))
            .collect();
        return CreatorSessionValidation {
            valid: false,
            status: CREATOR_SESSION_STATUS_COMMIT_FAILED,
            error_code: CREATOR_ERROR_INVALID_INTRO_TEXT,
            message: Some(format!("Invalid intro text characters: {:?}", unsupported)),
        };
    }

    let encoded_intro = encoder.encode(&session.intro_text);
    if encoded_intro.len() > INTRO_FIELD_SIZE {
        return CreatorSessionValidation {
            valid: false,
            status: CREATOR_SESSION_STATUS_COMMIT_FAILED,
            error_code: CREATOR_ERROR_INVALID_INTRO_TEXT,
            message: Some(format!(
                "Intro text too long: {} bytes (max {})",
                encoded_intro.len(),
                INTRO_FIELD_SIZE
            )),
        };
    }

    CreatorSessionValidation {
        valid: true,
        status: CREATOR_SESSION_STATUS_DRAFT_READY,
        error_code: 0,
        message: None,
    }
}

fn load_creator_session_from_rom(
    rom: &rom_core::Rom,
    boxer_id: u8,
    status: u8,
    error_code: u8,
) -> std::result::Result<CreatorSessionState, String> {
    let loader = RosterLoader::new(rom);
    let roster = loader.load_roster().map_err(|e| e.to_string())?;
    let boxer = roster
        .get_boxer(boxer_id)
        .cloned()
        .ok_or_else(|| format!("Boxer with ID {} not found", boxer_id))?;
    let intro = loader
        .load_boxer_intro(boxer.intro_text_id)
        .map_err(|e| e.to_string())?;

    Ok(CreatorSessionState {
        boxer_id,
        circuit: boxer.circuit.to_byte(),
        unlock_order: boxer.unlock_order,
        intro_text_id: boxer.intro_text_id,
        status,
        error_code,
        intro_text: intro.intro_quote,
        name_text: boxer.name,
    })
}

fn commit_creator_session_to_rom(
    rom: &mut rom_core::Rom,
    session: &CreatorSessionState,
) -> std::result::Result<(), String> {
    let normalized_name = session.name_text.trim().to_string();
    let normalized_intro = session.intro_text.trim().to_string();
    let circuit = CircuitType::from_byte(session.circuit);
    let mut writer = RosterWriter::new(rom);
    writer
        .write_boxer_name(session.boxer_id, &normalized_name)
        .map_err(|e| e.to_string())?;
    writer
        .write_circuit_assignment(session.boxer_id, circuit)
        .map_err(|e| e.to_string())?;
    writer
        .write_unlock_order(session.boxer_id, session.unlock_order)
        .map_err(|e| e.to_string())?;
    writer
        .write_boxer_intro_field(session.intro_text_id, 4, &normalized_intro)
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn creator_input_bytes(buttons: u16) -> (u8, u8) {
    let mut low = 0u8;
    let mut high = 0u8;

    if buttons & 0x8000 != 0 {
        low |= 0x01;
    }
    if buttons & 0x4000 != 0 {
        low |= 0x02;
    }
    if buttons & 0x2000 != 0 {
        low |= 0x04;
    }
    if buttons & 0x1000 != 0 {
        low |= 0x08;
    }
    if buttons & 0x0800 != 0 {
        low |= 0x10;
    }
    if buttons & 0x0400 != 0 {
        low |= 0x20;
    }
    if buttons & 0x0200 != 0 {
        low |= 0x40;
    }
    if buttons & 0x0100 != 0 {
        low |= 0x80;
    }

    if buttons & 0x0080 != 0 {
        high |= 0x01;
    }
    if buttons & 0x0040 != 0 {
        high |= 0x02;
    }
    if buttons & 0x0020 != 0 {
        high |= 0x04;
    }
    if buttons & 0x0010 != 0 {
        high |= 0x08;
    }

    (low, high)
}

fn cycle_creator_name_char(current: u8, direction: i8) -> u8 {
    match direction.cmp(&0) {
        std::cmp::Ordering::Greater => {
            if current < b' ' || current >= b'Z' {
                b' '
            } else {
                current + 1
            }
        }
        std::cmp::Ordering::Less => {
            if current <= b' ' || current > b'Z' {
                b'Z'
            } else {
                current - 1
            }
        }
        std::cmp::Ordering::Equal => current,
    }
}

fn render_rows_for_page(page: u8) -> [u8; 4] {
    match page {
        0 => CREATOR_PAGE0_ROWS,
        1 => CREATOR_PAGE1_ROWS,
        2 => CREATOR_PAGE2_ROWS,
        _ => CREATOR_PAGE3_ROWS,
    }
}

/// Commands sent to the emulation thread
#[derive(Debug, Clone)]
pub enum EmulationCommand {
    StepFrame,
    RunToFrame(u64),
    Pause,
    Resume,
    Reset,
    SaveState(u8),
    LoadState(u8),
    SetInput { port: usize, buttons: u16 },
    Stop,
}

/// Events sent from the emulation thread
#[derive(Debug, Clone)]
pub enum EmulationEvent {
    FrameComplete(u64),
    StateSaved(u8),
    StateLoaded(u8),
    Error(String),
    Paused,
    Resumed,
    Stopped,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rom_core::SpoTextEncoder;

    fn build_expanded_creator_rom() -> Vec<u8> {
        let mut rom = rom_core::Rom::new(vec![0; 0x20_0000]);
        let header_pc = 0x1F0000usize;
        let boxer_count = 24usize;
        let name_ptr = 0x1F0100usize;
        let long_ptr = 0x1F0140usize;
        let name_blob = 0x1F0200usize;
        let circuit = 0x1F0600usize;
        let unlock = 0x1F0620usize;
        let intro = 0x1F0640usize;

        let mut header = vec![0u8; 46];
        header[..8].copy_from_slice(b"SPOEDITR");
        header[8] = 2;
        header[10..12].copy_from_slice(&(boxer_count as u16).to_le_bytes());
        header[12..16].copy_from_slice(&(name_ptr as u32).to_le_bytes());
        header[16..20].copy_from_slice(&(long_ptr as u32).to_le_bytes());
        header[20..24].copy_from_slice(&(name_blob as u32).to_le_bytes());
        header[24..28].copy_from_slice(&(circuit as u32).to_le_bytes());
        header[28..32].copy_from_slice(&(unlock as u32).to_le_bytes());
        header[32..36].copy_from_slice(&(intro as u32).to_le_bytes());
        rom.write_bytes(header_pc, &header).unwrap();

        let encoder = SpoTextEncoder::new();
        for boxer_id in 0..boxer_count {
            let slot_pc = name_blob + boxer_id * 16;
            let default_name = format!("BOXER {}", boxer_id + 1);
            let encoded = encoder.encode_with_terminator(&default_name);
            rom.write_bytes(slot_pc, &encoded).unwrap();

            let (bank, addr) = rom.pc_to_snes(slot_pc);
            let [lo, hi] = addr.to_le_bytes();
            rom.write_bytes(name_ptr + boxer_id * 2, &[lo, hi]).unwrap();
            rom.write_bytes(long_ptr + boxer_id * 3, &[bank, lo, hi]).unwrap();

            let intro_base = intro + boxer_id * 80;
            let intro_fields = [
                format!("NAME {}", boxer_id + 1),
                "USA".to_string(),
                "1-0".to_string(),
                "RANKED".to_string(),
                format!("QUOTE {}", boxer_id + 1),
            ];
            for (field_index, field) in intro_fields.iter().enumerate() {
                rom.write_bytes(
                    intro_base + field_index * 16,
                    &encoder.encode_fixed(field, 16),
                )
                .unwrap();
            }
        }

        let circuit_bytes = vec![CircuitType::Minor.to_byte(); boxer_count];
        rom.write_bytes(circuit, &circuit_bytes).unwrap();
        rom.write_bytes(circuit + 20, &[CircuitType::World.to_byte()])
            .unwrap();

        let unlock_bytes: Vec<u8> = (0..boxer_count as u8).collect();
        rom.write_bytes(unlock, &unlock_bytes).unwrap();
        rom.write_bytes(unlock + 20, &[21]).unwrap();

        rom.data
    }

    #[test]
    fn test_core_config_default() {
        let config = CoreConfig::default();
        assert!(config.enable_audio);
        assert!(config.frame_limit);
        assert!(config.enable_states);
    }

    #[test]
    fn test_emulation_commands() {
        let cmd = EmulationCommand::StepFrame;
        assert!(matches!(cmd, EmulationCommand::StepFrame));

        let cmd = EmulationCommand::SetInput {
            port: 0,
            buttons: 0xFF,
        };
        if let EmulationCommand::SetInput { port, buttons } = cmd {
            assert_eq!(port, 0);
            assert_eq!(buttons, 0xFF);
        }
    }

    #[test]
    fn test_snes9x_core_creation() {
        let core = Snes9xCore::new_stub();
        assert!(!core.is_rom_loaded());
    }

    #[test]
    fn test_creator_runtime_activates_on_combo() {
        let mut core = Snes9xCore::new_stub();
        core.load_rom(vec![0u8; 0x80_000]).unwrap();

        // Select + Start + L + R
        core.set_input(0, 0x2000 | 0x1000 | 0x0020 | 0x0010);
        core.run_frame();

        let state = core.creator_runtime_state();
        assert!(state.active);
        assert_eq!(state.magic, CREATOR_MODE_MAGIC_VALUE);
        assert_eq!(state.heartbeat, CREATOR_MODE_HEARTBEAT_VALUE);
        assert!(state.render_visible);
        assert_eq!(state.page, 0);
        assert_eq!(state.render_rows, CREATOR_PAGE0_ROWS);
    }

    #[test]
    fn test_creator_runtime_navigation_and_action() {
        let mut core = Snes9xCore::new_stub();
        core.load_rom(vec![0u8; 0x80_000]).unwrap();

        core.set_input(0, 0x2000 | 0x1000 | 0x0020 | 0x0010);
        core.run_frame();

        // Right to next page.
        core.set_input(0, 0x0100);
        core.run_frame();
        let state = core.creator_runtime_state();
        assert_eq!(state.page, 1);
        assert_eq!(state.render_page, 1);
        assert_eq!(state.render_rows, CREATOR_PAGE1_ROWS);

        // Down then A to select page-specific action.
        core.set_input(0, 0x0400);
        core.run_frame();
        core.set_input(0, 0x0080);
        core.run_frame();

        let state = core.creator_runtime_state();
        assert_eq!(state.cursor, 1);
        assert_eq!(state.action, CREATOR_ACTION_CIRCUIT_EDIT);
        assert_eq!(state.session_circuit, 1);
        assert_eq!(state.session_status, CREATOR_SESSION_STATUS_DRAFT_READY);
    }

    #[test]
    fn test_creator_runtime_unlock_order_advances_from_identity_page() {
        let mut core = Snes9xCore::new_stub();
        core.load_rom(vec![0u8; 0x80_000]).unwrap();
        assert!(core.set_creator_session_state(&CreatorSessionState {
            boxer_id: 20,
            circuit: 1,
            unlock_order: 7,
            intro_text_id: 20,
            status: 2,
            error_code: 0,
            intro_text: "HELLO".to_string(),
            name_text: "MAX".to_string(),
        }));

        core.set_input(0, 0x2000 | 0x1000 | 0x0020 | 0x0010);
        core.run_frame();

        core.set_input(0, 0x0400);
        core.run_frame();
        core.set_input(0, 0x0400);
        core.run_frame();
        core.set_input(0, 0x0080);
        core.run_frame();

        let state = core.creator_runtime_state();
        assert_eq!(state.page, 0);
        assert_eq!(state.cursor, 2);
        assert_eq!(state.action, CREATOR_ACTION_NAME_EDIT);
        assert_eq!(state.session_unlock_order, 8);
        assert_eq!(state.session_status, CREATOR_SESSION_STATUS_DRAFT_READY);
    }

    #[test]
    fn test_creator_runtime_intro_text_id_advances_from_identity_page() {
        let mut core = Snes9xCore::new_stub();
        core.load_rom(vec![0u8; 0x80_000]).unwrap();
        assert!(core.set_creator_session_state(&CreatorSessionState {
            boxer_id: 22,
            circuit: 1,
            unlock_order: 7,
            intro_text_id: 33,
            status: 2,
            error_code: 0,
            intro_text: "QUOTE".to_string(),
            name_text: "MAX".to_string(),
        }));

        core.set_input(0, 0x2000 | 0x1000 | 0x0020 | 0x0010);
        core.run_frame();

        core.set_input(0, 0x0400);
        core.run_frame();
        core.set_input(0, 0x0400);
        core.run_frame();
        core.set_input(0, 0x0400);
        core.run_frame();
        core.set_input(0, 0x0080);
        core.run_frame();

        let state = core.creator_runtime_state();
        assert_eq!(state.page, 0);
        assert_eq!(state.cursor, 3);
        assert_eq!(state.action, CREATOR_ACTION_INTRO_EDIT);
        assert_eq!(state.session_intro_text_id, 34);
        assert_eq!(state.session_status, CREATOR_SESSION_STATUS_DRAFT_READY);
    }

    #[test]
    fn test_creator_runtime_intro_edit_updates_buffer_and_cursor() {
        let mut core = Snes9xCore::new_stub();
        core.load_rom(vec![0u8; 0x80_000]).unwrap();
        assert!(core.set_creator_session_state(&CreatorSessionState {
            boxer_id: 23,
            circuit: 0,
            unlock_order: 4,
            intro_text_id: 23,
            status: 2,
            error_code: 0,
            intro_text: "AB".to_string(),
            name_text: "AB".to_string(),
        }));

        core.set_input(0, 0x2000 | 0x1000 | 0x0020 | 0x0010);
        core.run_frame();

        core.set_input(0, 0x0400);
        core.run_frame();
        core.set_input(0, 0x0080);
        core.run_frame();
        let state = core.creator_runtime_state();
        assert!(state.intro_edit_active);
        assert_eq!(state.action, CREATOR_ACTION_INTRO_EDIT);
        assert_eq!(state.intro_cursor, 0);

        core.set_input(0, 0x0800);
        core.run_frame();
        let state = core.creator_runtime_state();
        assert_eq!(state.intro_bytes[0], b'B');

        core.set_input(0, 0x0080);
        core.run_frame();
        let state = core.creator_runtime_state();
        assert_eq!(state.intro_cursor, 1);

        core.set_input(0, 0x8000);
        core.run_frame();
        let state = core.creator_runtime_state();
        assert!(!state.intro_edit_active);
        assert!(state.active);
    }

    #[test]
    fn test_creator_runtime_name_edit_updates_buffer_and_cursor() {
        let mut core = Snes9xCore::new_stub();
        core.load_rom(vec![0u8; 0x80_000]).unwrap();
        assert!(core.set_creator_session_state(&CreatorSessionState {
            boxer_id: 21,
            circuit: 0,
            unlock_order: 4,
            intro_text_id: 21,
            status: 2,
            error_code: 0,
            intro_text: "AB".to_string(),
            name_text: "AB".to_string(),
        }));

        core.set_input(0, 0x2000 | 0x1000 | 0x0020 | 0x0010);
        core.run_frame();

        core.set_input(0, 0x0080);
        core.run_frame();
        let state = core.creator_runtime_state();
        assert!(state.name_edit_active);
        assert_eq!(state.action, CREATOR_ACTION_NAME_EDIT);
        assert_eq!(state.name_cursor, 0);

        core.set_input(0, 0x0800);
        core.run_frame();
        let state = core.creator_runtime_state();
        assert_eq!(state.name_bytes[0], b'B');

        core.set_input(0, 0x0080);
        core.run_frame();
        let state = core.creator_runtime_state();
        assert_eq!(state.name_cursor, 1);

        core.set_input(0, 0x8000);
        core.run_frame();
        let state = core.creator_runtime_state();
        assert!(!state.name_edit_active);
        assert!(state.active);
    }

    #[test]
    fn test_creator_runtime_restores_circuit_cursor_from_runtime_session() {
        let mut core = Snes9xCore::new_stub();
        core.load_rom(vec![0u8; 0x80_000]).unwrap();
        assert!(core.set_creator_session_state(&CreatorSessionState {
            boxer_id: 18,
            circuit: 3,
            unlock_order: 9,
            intro_text_id: 18,
            status: 2,
            error_code: 0,
            intro_text: "RANKED".to_string(),
            name_text: "BOB".to_string(),
        }));

        core.set_input(0, 0x2000 | 0x1000 | 0x0020 | 0x0010);
        core.run_frame();

        core.set_input(0, 0x0100);
        core.run_frame();

        let state = core.creator_runtime_state();
        assert_eq!(state.page, 1);
        assert_eq!(state.cursor, 3);
        assert_eq!(state.render_cursor, 3);
    }

    #[test]
    fn test_creator_runtime_exit_clears_render_contract() {
        let mut core = Snes9xCore::new_stub();
        core.load_rom(vec![0u8; 0x80_000]).unwrap();

        core.set_input(0, 0x2000 | 0x1000 | 0x0020 | 0x0010);
        core.run_frame();
        let initial_revision = core.creator_runtime_state().render_revision;

        // B exits creator mode.
        core.set_input(0, 0x8000);
        core.run_frame();

        let state = core.creator_runtime_state();
        assert!(!state.active);
        assert_eq!(state.action, CREATOR_ACTION_EXIT);
        assert!(!state.render_visible);
        assert_eq!(state.render_rows, [0; 4]);
        assert_eq!(state.render_revision, initial_revision.wrapping_add(1));
    }

    #[test]
    fn test_creator_runtime_commit_requires_finalize_commit_row() {
        let mut core = Snes9xCore::new_stub();
        core.load_rom(vec![0u8; 0x80_000]).unwrap();

        core.set_input(0, 0x2000 | 0x1000 | 0x0020 | 0x0010);
        core.run_frame();

        core.set_input(0, 0x0100);
        core.run_frame();
        core.set_input(0, 0x0100);
        core.run_frame();
        core.set_input(0, 0x0100);
        core.run_frame();
        core.set_input(0, 0x0400);
        core.run_frame();
        core.set_input(0, 0x0080);
        core.run_frame();

        let state = core.creator_runtime_state();
        assert_eq!(state.page, 3);
        assert_eq!(state.cursor, 1);
        assert_eq!(state.action, CREATOR_ACTION_COMMIT);
        assert_eq!(state.session_status, CREATOR_SESSION_STATUS_COMMIT_PENDING);
    }

    #[test]
    fn test_creator_runtime_cancel_from_finalize_row() {
        let mut core = Snes9xCore::new_stub();
        core.load_rom(vec![0u8; 0x80_000]).unwrap();

        core.set_input(0, 0x2000 | 0x1000 | 0x0020 | 0x0010);
        core.run_frame();

        core.set_input(0, 0x0100);
        core.run_frame();
        core.set_input(0, 0x0100);
        core.run_frame();
        core.set_input(0, 0x0100);
        core.run_frame();
        core.set_input(0, 0x0400);
        core.run_frame();
        core.set_input(0, 0x0400);
        core.run_frame();
        core.set_input(0, 0x0080);
        core.run_frame();

        let state = core.creator_runtime_state();
        assert_eq!(state.page, 3);
        assert_eq!(state.cursor, 2);
        assert_eq!(state.action, CREATOR_ACTION_CANCEL);
        assert_eq!(state.session_status, CREATOR_SESSION_STATUS_CANCELLED);
    }

    #[test]
    fn test_read_system_ram_reads_stub_contract_bytes() {
        let mut core = Snes9xCore::new_stub();
        core.load_rom(vec![0u8; 0x80_000]).unwrap();
        core.set_input(0, 0x2000 | 0x1000 | 0x0020 | 0x0010);
        core.run_frame();

        assert_eq!(core.read_system_ram(CREATOR_RENDER_ROW0, 1).unwrap(), vec![CREATOR_PAGE0_ROWS[0]]);
        assert_eq!(core.read_system_ram(CREATOR_RENDER_ROW1, 1).unwrap(), vec![CREATOR_PAGE0_ROWS[1]]);
        assert_eq!(core.read_system_ram(CREATOR_RENDER_ROW2, 1).unwrap(), vec![CREATOR_PAGE0_ROWS[2]]);
        assert_eq!(core.read_system_ram(CREATOR_RENDER_ROW3, 1).unwrap(), vec![CREATOR_PAGE0_ROWS[3]]);
    }

    #[test]
    fn test_creator_session_state_round_trips_through_stub_wram() {
        let mut core = Snes9xCore::new_stub();
        core.load_rom(vec![0u8; 0x80_000]).unwrap();

        let session = CreatorSessionState {
            boxer_id: 20,
            circuit: 2,
            unlock_order: 30,
            intro_text_id: 20,
            status: 4,
            error_code: 0,
            intro_text: "WIN BIG".to_string(),
            name_text: "RICK".to_string(),
        };

        assert!(core.set_creator_session_state(&session));
        let state = core.creator_runtime_state();
        assert!(state.session_present);
        assert_eq!(state.session_boxer_id, 20);
        assert_eq!(state.session_circuit, 2);
        assert_eq!(state.session_unlock_order, 30);
        assert_eq!(state.session_intro_text_id, 20);
        assert_eq!(state.session_status, 4);
        assert_eq!(state.session_error_code, 0);
        assert_eq!(state.intro_length, 7);
        assert_eq!(&state.intro_bytes[..7], b"WIN BIG");
        assert_eq!(state.name_length, 4);
        assert_eq!(&state.name_bytes[..4], b"RICK");

        assert!(core.clear_creator_session_state());
        let state = core.creator_runtime_state();
        assert!(!state.session_present);
        assert_eq!(state.session_boxer_id, 0);
    }

    #[test]
    fn test_resolve_creator_runtime_commit_updates_internal_rom() {
        let mut core = Snes9xCore::new_stub();
        core.load_rom(build_expanded_creator_rom()).unwrap();
        assert!(core.set_creator_session_state(&CreatorSessionState {
            boxer_id: 20,
            circuit: CircuitType::Special.to_byte(),
            unlock_order: 30,
            intro_text_id: 20,
            status: CREATOR_SESSION_STATUS_COMMIT_PENDING,
            error_code: 0,
            intro_text: "NEW QUOTE".to_string(),
            name_text: "ACE".to_string(),
        }));
        assert!(core.write_creator_action(CREATOR_ACTION_COMMIT));

        let resolution = core.resolve_creator_runtime_action().unwrap();
        assert!(resolution.rom_updated);
        assert_eq!(
            resolution.runtime_state.session_status,
            CREATOR_SESSION_STATUS_COMMIT_SUCCEEDED
        );

        let rom = rom_core::Rom::new(core.current_rom_image().unwrap());
        let loader = RosterLoader::new(&rom);
        let roster = loader.load_roster().unwrap();
        let boxer = roster.get_boxer(20).unwrap();
        assert_eq!(boxer.name, "ACE");
        assert_eq!(boxer.circuit, CircuitType::Special);
        assert_eq!(boxer.unlock_order, 30);

        let intro = loader.load_boxer_intro(20).unwrap();
        assert_eq!(intro.intro_quote, "NEW QUOTE");
    }

    #[test]
    fn test_resolve_creator_runtime_cancel_restores_session_from_rom() {
        let mut core = Snes9xCore::new_stub();
        core.load_rom(build_expanded_creator_rom()).unwrap();
        assert!(core.set_creator_session_state(&CreatorSessionState {
            boxer_id: 20,
            circuit: CircuitType::Special.to_byte(),
            unlock_order: 30,
            intro_text_id: 20,
            status: CREATOR_SESSION_STATUS_DRAFT_READY,
            error_code: 0,
            intro_text: "NEW QUOTE".to_string(),
            name_text: "ACE".to_string(),
        }));
        assert!(core.write_creator_action(CREATOR_ACTION_CANCEL));

        let resolution = core.resolve_creator_runtime_action().unwrap();
        assert!(!resolution.rom_updated);
        assert_eq!(
            resolution.runtime_state.session_status,
            CREATOR_SESSION_STATUS_CANCELLED
        );

        let restored = creator_session_from_runtime(&resolution.runtime_state);
        assert_eq!(restored.name_text, "BOXER 21");
        assert_eq!(restored.circuit, CircuitType::World.to_byte());
        assert_eq!(restored.unlock_order, 21);
        assert_eq!(restored.intro_text_id, 20);
        assert_eq!(restored.intro_text, "QUOTE 21");
    }
}
