//! # Flash Cart Interfaces
//!
//! Provides interfaces for various flash cart devices used in SNES development:
//!
//! - **SD2SNES/FXPak Pro** - Modern USB-capable flash cart
//! - **Everdrive** - Series of flash carts from Krikzz
//! - **USB2SNES** - USB interface protocol for SNES devices
//!
//! ## Example Usage
//!
//! ```rust,no_run
//! use console_dev_core::flashcart::{AutoDetect, FlashCart};
//!
//! // Auto-detect connected device
//! if let Some(mut device) = AutoDetect::detect_first().unwrap() {
//!     println!("Detected: {}", device.device_name());
//! }
//! ```

use rom_core::RomData;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::Duration;
use thiserror::Error;

pub mod everdrive;
pub mod sd2snes;
pub mod usb2snes;

pub use everdrive::EverdriveDevice;
pub use sd2snes::Sd2snesDevice;
pub use usb2snes::Usb2snesDevice;

/// Errors that can occur during flash cart operations
#[derive(Debug, Error)]
pub enum FlashCartError {
    /// Connection failed
    #[error("Failed to connect to device: {0}")]
    ConnectionFailed(String),

    /// Device not found
    #[error("Device not found: {0}")]
    DeviceNotFound(String),

    /// Upload failed
    #[error("ROM upload failed: {message}")]
    UploadFailed {
        /// Error message
        message: String,
    },

    /// Download failed
    #[error("SRAM download failed: {message}")]
    DownloadFailed {
        /// Error message
        message: String,
    },

    /// Live patching failed
    #[error("Live patch failed: {message}")]
    PatchFailed {
        /// Error message
        message: String,
    },

    /// Device does not support operation
    #[error("Operation not supported by device: {0}")]
    NotSupported(String),

    /// USB/HID error
    #[error("USB/HID error: {0}")]
    UsbError(String),

    /// Protocol error
    #[error("Protocol error: {0}")]
    ProtocolError(String),

    /// Timeout
    #[error("Operation timed out")]
    Timeout,

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type for flash cart operations
pub type Result<T> = std::result::Result<T, FlashCartError>;

/// Trait for flash cart devices
///
/// Implement this trait for any flash cart device that supports ROM upload,
/// SRAM download, and potentially live patching.
pub trait FlashCart: Send + fmt::Debug {
    /// Get the device name
    fn device_name(&self) -> &str;

    /// Get the device firmware version
    fn firmware_version(&self) -> Option<&str>;

    /// Check if the device is connected
    fn is_connected(&self) -> bool;

    /// Connect to the device
    ///
    /// # Errors
    ///
    /// Returns an error if connection fails
    fn connect(&mut self) -> Result<()>;

    /// Disconnect from the device
    fn disconnect(&mut self);

    /// Upload a ROM to the flash cart
    ///
    /// # Arguments
    ///
    /// * `rom_data` - The ROM data to upload
    /// * `progress_callback` - Optional callback for progress updates
    ///
    /// # Errors
    ///
    /// Returns an error if the upload fails
    fn upload_rom(
        &mut self,
        rom_data: &RomData,
        progress_callback: Option<&dyn Fn(usize, usize)>,
    ) -> Result<()>;

    /// Download SRAM data from the flash cart
    ///
    /// # Arguments
    ///
    /// * `size` - Expected SRAM size in bytes
    /// * `progress_callback` - Optional callback for progress updates
    ///
    /// # Errors
    ///
    /// Returns an error if the download fails
    fn download_sram(
        &mut self,
        size: usize,
        progress_callback: Option<&dyn Fn(usize, usize)>,
    ) -> Result<Vec<u8>>;

    /// Upload SRAM data to the flash cart
    ///
    /// # Arguments
    ///
    /// * `sram_data` - The SRAM data to upload
    ///
    /// # Errors
    ///
    /// Returns an error if the upload fails
    fn upload_sram(&mut self, sram_data: &[u8]) -> Result<()>;

    /// Apply a live patch to the running ROM
    ///
    /// This allows modifying RAM or ROM values without restarting.
    ///
    /// # Arguments
    ///
    /// * `address` - The address to patch
    /// * `data` - The data to write
    ///
    /// # Errors
    ///
    /// Returns an error if patching is not supported or fails
    fn patch_live(&mut self, address: u32, data: &[u8]) -> Result<()>;

    /// Check if live patching is supported
    fn supports_live_patch(&self) -> bool;

    /// Reset the SNES
    ///
    /// # Errors
    ///
    /// Returns an error if the reset command fails
    fn reset(&mut self) -> Result<()>;

    /// Get the device info
    fn device_info(&self) -> DeviceInfo;
}

/// Information about a flash cart device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    /// Device name
    pub name: String,
    /// Firmware version
    pub firmware_version: Option<String>,
    /// Hardware version
    pub hardware_version: Option<String>,
    /// Supported features
    pub features: Vec<DeviceFeature>,
    /// Maximum ROM size supported
    pub max_rom_size: Option<usize>,
    /// Maximum SRAM size supported
    pub max_sram_size: Option<usize>,
    /// USB vendor ID
    pub usb_vid: Option<u16>,
    /// USB product ID
    pub usb_pid: Option<u16>,
}

impl DeviceInfo {
    /// Create new device info
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            firmware_version: None,
            hardware_version: None,
            features: Vec::new(),
            max_rom_size: None,
            max_sram_size: None,
            usb_vid: None,
            usb_pid: None,
        }
    }

    /// Set firmware version
    pub fn with_firmware(mut self, version: impl Into<String>) -> Self {
        self.firmware_version = Some(version.into());
        self
    }

    /// Set hardware version
    pub fn with_hardware(mut self, version: impl Into<String>) -> Self {
        self.hardware_version = Some(version.into());
        self
    }

    /// Add a supported feature
    pub fn with_feature(mut self, feature: DeviceFeature) -> Self {
        self.features.push(feature);
        self
    }
}

/// Features supported by a flash cart device
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceFeature {
    /// USB connectivity
    Usb,
    /// Live ROM patching
    LivePatch,
    /// Cheat codes
    Cheats,
    /// Save states
    SaveStates,
    /// MSU-1 audio expansion
    Msu1,
    /// SuperFX chip support
    SuperFx,
    /// SA-1 chip support
    Sa1,
    /// DSP chip support
    Dsp,
    /// CX4 chip support
    Cx4,
    /// SRTC support
    Srtc,
    /// OBC1 support
    Obc1,
    /// SDD1 support
    Sdd1,
    /// SPC7110 support
    Spc7110,
    /// BS-X support
    BsX,
}

impl fmt::Display for DeviceFeature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeviceFeature::Usb => write!(f, "USB"),
            DeviceFeature::LivePatch => write!(f, "Live Patch"),
            DeviceFeature::Cheats => write!(f, "Cheats"),
            DeviceFeature::SaveStates => write!(f, "Save States"),
            DeviceFeature::Msu1 => write!(f, "MSU-1"),
            DeviceFeature::SuperFx => write!(f, "SuperFX"),
            DeviceFeature::Sa1 => write!(f, "SA-1"),
            DeviceFeature::Dsp => write!(f, "DSP"),
            DeviceFeature::Cx4 => write!(f, "CX4"),
            DeviceFeature::Srtc => write!(f, "SRTC"),
            DeviceFeature::Obc1 => write!(f, "OBC1"),
            DeviceFeature::Sdd1 => write!(f, "SDD1"),
            DeviceFeature::Spc7110 => write!(f, "SPC7110"),
            DeviceFeature::BsX => write!(f, "BS-X"),
        }
    }
}

/// Auto-detection of connected flash cart devices
pub struct AutoDetect;

impl AutoDetect {
    /// Detect all connected flash cart devices
    ///
    /// Returns a vector of detected devices (boxed trait objects)
    ///
    /// # Errors
    ///
    /// Returns an error if detection fails
    pub fn detect_all() -> Result<Vec<Box<dyn FlashCart>>> {
        let mut devices = Vec::new();

        // Try SD2SNES/FXPak Pro
        if let Ok(device) = Sd2snesDevice::detect() {
            log::info!("Detected SD2SNES/FXPak Pro device");
            devices.push(Box::new(device) as Box<dyn FlashCart>);
        }

        // Try Everdrive
        if let Ok(device) = EverdriveDevice::detect() {
            log::info!("Detected Everdrive device");
            devices.push(Box::new(device) as Box<dyn FlashCart>);
        }

        // Try USB2SNES
        if let Ok(device) = Usb2snesDevice::detect() {
            log::info!("Detected USB2SNES device");
            devices.push(Box::new(device) as Box<dyn FlashCart>);
        }

        Ok(devices)
    }

    /// Detect the first available flash cart device
    ///
    /// Checks devices in priority order:
    /// 1. SD2SNES/FXPak Pro (most features)
    /// 2. Everdrive
    /// 3. USB2SNES
    ///
    /// # Errors
    ///
    /// Returns an error if detection fails
    pub fn detect_first() -> Result<Option<Box<dyn FlashCart>>> {
        // Try SD2SNES first (most feature-rich)
        if let Ok(device) = Sd2snesDevice::detect() {
            log::info!("Auto-detected SD2SNES/FXPak Pro");
            return Ok(Some(Box::new(device)));
        }

        // Try Everdrive
        if let Ok(device) = EverdriveDevice::detect() {
            log::info!("Auto-detected Everdrive");
            return Ok(Some(Box::new(device)));
        }

        // Try USB2SNES
        if let Ok(device) = Usb2snesDevice::detect() {
            log::info!("Auto-detected USB2SNES");
            return Ok(Some(Box::new(device)));
        }

        log::info!("No flash cart devices detected");
        Ok(None)
    }

    /// Check if any flash cart device is connected
    pub fn is_any_connected() -> bool {
        Self::detect_first().map(|d| d.is_some()).unwrap_or(false)
    }

    /// Wait for a device to be connected
    ///
    /// # Arguments
    ///
    /// * `timeout` - Maximum time to wait
    ///
    /// # Errors
    ///
    /// Returns an error if timeout is reached
    pub fn wait_for_device(timeout: Duration) -> Result<Box<dyn FlashCart>> {
        let start = std::time::Instant::now();

        loop {
            if let Some(device) = Self::detect_first()? {
                return Ok(device);
            }

            if start.elapsed() >= timeout {
                return Err(FlashCartError::Timeout);
            }

            std::thread::sleep(Duration::from_millis(100));
        }
    }
}

/// USB device identifiers for known flash carts
pub mod usb_ids {
    /// SD2SNES/FXPak Pro Vendor ID
    pub const SD2SNES_VID: u16 = 0x1209;
    /// SD2SNES/FXPak Pro Product ID
    pub const SD2SNES_PID: u16 = 0x8F63;

    /// Everdrive USB Vendor ID (varies by model)
    pub const EVERDRIVE_VID: u16 = 0x0483;
    /// Everdrive USB Product ID (varies by model)
    pub const EVERDRIVE_PID: u16 = 0x5740;
}

/// Progress callback for long-running operations
pub type ProgressCallback<'a> = &'a dyn Fn(usize, usize);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_info() {
        let info = DeviceInfo::new("Test Device")
            .with_firmware("1.0.0")
            .with_hardware("Rev A")
            .with_feature(DeviceFeature::Usb)
            .with_feature(DeviceFeature::LivePatch);

        assert_eq!(info.name, "Test Device");
        assert_eq!(info.firmware_version, Some("1.0.0".to_string()));
        assert_eq!(info.features.len(), 2);
    }

    #[test]
    fn test_device_feature_display() {
        assert_eq!(format!("{}", DeviceFeature::Usb), "USB");
        assert_eq!(format!("{}", DeviceFeature::LivePatch), "Live Patch");
    }
}
