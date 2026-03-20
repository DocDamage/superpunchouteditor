//! # Snes9x Core Implementation
//!
//! Stub implementation of the Snes9x emulator core for the Super Punch-Out!! editor.

use crate::audio::{AudioBatch, AudioBuffer};
use crate::input::{InputManager, SnesController};
use crate::video::{PixelFormat, VideoBuffer, VideoFrame};
use crate::{EmulatorError, Result};

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

/// The main Snes9x emulator core
pub struct Snes9xCore {
    config: CoreConfig,
    video_buffer: Arc<VideoBuffer>,
    audio_buffer: Arc<AudioBuffer>,
    input_manager: Arc<InputManager>,
    rom_loaded: bool,
    rom_data: Option<Vec<u8>>,
    frame_count: u64,
    #[allow(dead_code)]
    save_dir: PathBuf,
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

        Ok(Self {
            config,
            video_buffer,
            audio_buffer,
            input_manager,
            rom_loaded: false,
            rom_data: None,
            frame_count: 0,
            save_dir,
        })
    }

    pub fn load_rom(&mut self, rom_data: impl Into<Vec<u8>>) -> Result<()> {
        let data: Vec<u8> = rom_data.into();
        if data.len() < 0x8000 {
            return Err(EmulatorError::InvalidRomData);
        }
        self.rom_data = Some(data);
        self.rom_loaded = true;
        Ok(())
    }

    pub fn run_frame(&mut self) {
        if self.rom_loaded {
            self.frame_count += 1;
        }
    }

    pub fn get_frame(&self) -> Option<VideoFrame> {
        self.video_buffer.get_frame()
    }

    pub fn get_frame_buffer(&self) -> Option<VideoFrame> {
        self.get_frame()
    }

    pub fn get_audio_samples(&self) -> Option<AudioBatch> {
        None
    }

    pub fn get_audio_samples_vec(&self) -> Vec<i16> {
        Vec::new()
    }

    pub fn set_input(&mut self, port: u32, buttons: u16) {
        let controller = SnesController::from_mask(buttons);
        self.input_manager
            .set_controller_state(port as usize, controller);
    }

    pub fn reset(&mut self) {
        self.reset_soft();
    }

    pub fn reset_soft(&mut self) {
        self.frame_count = 0;
    }

    pub fn reset_hard(&mut self) {
        self.frame_count = 0;
        if let Some(ref data) = self.rom_data.clone() {
            let _ = self.load_rom(data.clone());
        }
    }

    pub fn save_state(&self) -> Result<Vec<u8>> {
        if !self.rom_loaded {
            return Err(EmulatorError::NotInitialized);
        }
        Ok(vec![0u8; 1024])
    }

    pub fn load_state(&mut self, _state_data: &[u8]) -> Result<()> {
        if !self.rom_loaded {
            return Err(EmulatorError::NotInitialized);
        }
        Ok(())
    }

    pub fn save_state_size(&self) -> Result<usize> {
        if !self.rom_loaded {
            return Err(EmulatorError::NotInitialized);
        }
        Ok(512 * 1024)
    }

    pub fn is_rom_loaded(&self) -> bool {
        self.rom_loaded
    }

    pub fn config(&self) -> CoreConfig {
        self.config.clone()
    }

    pub fn set_config(&mut self, config: CoreConfig) {
        self.config = config;
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

    pub fn get_frame_dimensions(&self) -> (u32, u32) {
        self.video_buffer.get_dimensions()
    }

    pub fn unload_rom(&mut self) {
        self.rom_loaded = false;
        self.rom_data = None;
    }
}

impl Drop for Snes9xCore {
    fn drop(&mut self) {
        self.unload_rom();
    }
}

unsafe impl Send for Snes9xCore {}
unsafe impl Sync for Snes9xCore {}

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
        let core = Snes9xCore::new("dummy.dll");
        assert!(core.is_ok());
    }
}
