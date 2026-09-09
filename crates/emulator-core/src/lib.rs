//! # emulator-core
//!
//! Snes9x libretro core integration for the Super Punch-Out!! editor.
//!
//! This crate provides a Rust wrapper around the Snes9x libretro core,
//! enabling SNES emulation with video, audio, input, and save state support.

pub mod audio;
pub mod input;
pub mod libretro;
mod libretro_runtime;
pub mod snes9x;
pub mod state;
pub mod video;

pub use audio::{AudioBatch, AudioBuffer, AudioConfig};
pub use input::{SnesButton, SnesController};
pub use snes9x::{
    CoreConfig, CreatorRuntimeActionResolution, CreatorRuntimeState, CreatorSessionState,
    EmulationThread, Snes9xCore,
};
pub use state::{SaveState, StateManager};
pub use video::{PixelFormat, VideoBuffer, VideoFrame};

use thiserror::Error;

/// Errors that can occur in the emulator core
#[derive(Error, Debug)]
pub enum EmulatorError {
    #[error("Failed to load core library: {0}")]
    LibraryLoadError(String),

    #[error("Failed to initialize core: {0}")]
    InitializationError(String),

    #[error("Failed to load ROM: {0}")]
    RomLoadError(String),

    #[error("Failed to save/load state: {0}")]
    StateError(String),

    #[error("Invalid ROM data")]
    InvalidRomData,

    #[error("Core not initialized")]
    NotInitialized,

    #[error("Core already initialized")]
    AlreadyInitialized,

    #[error("Audio error: {0}")]
    AudioError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Result type for emulator operations
pub type Result<T> = std::result::Result<T, EmulatorError>;

/// Standard SNES resolution constants
pub const SNES_WIDTH: u32 = 256;
pub const SNES_HEIGHT: u32 = 224;
pub const SNES_MAX_WIDTH: u32 = 512;
pub const SNES_MAX_HEIGHT: u32 = 448;

/// Standard SNES audio constants
pub const SNES_SAMPLE_RATE: f64 = 32040.5;
pub const SNES_AUDIO_CHANNELS: usize = 2;

/// Core version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert_eq!(VERSION, env!("CARGO_PKG_VERSION"));
    }
}
