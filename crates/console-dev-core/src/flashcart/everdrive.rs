//! Everdrive flash cart implementation
//!
//! Supports various Everdrive models including:
//! - Everdrive X5
//! - Everdrive X6
//! - Everdrive X7
//! - Everdrive Pro

use super::{DeviceFeature, DeviceInfo, FlashCart, FlashCartError, Result};
use rom_core::RomData;
use std::fmt;

/// Everdrive device
///
/// Supports USB connectivity on newer models (X7, Pro)
#[derive(Debug)]
pub struct EverdriveDevice {
    /// Device info
    info: DeviceInfo,
    /// Connection state
    connected: bool,
    /// USB device handle (placeholder)
    usb_handle: Option<()>,
    /// Model type
    model: EverdriveModel,
}

/// Everdrive model types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EverdriveModel {
    /// Everdrive X5 (basic, no USB)
    X5,
    /// Everdrive X6 (RTC support)
    X6,
    /// Everdrive X7 (USB support)
    X7,
    /// Everdrive Pro (USB + advanced features)
    Pro,
    /// Unknown/generic Everdrive
    Unknown,
}

impl EverdriveModel {
    /// Get the model name
    pub fn name(&self) -> &'static str {
        match self {
            EverdriveModel::X5 => "Everdrive X5",
            EverdriveModel::X6 => "Everdrive X6",
            EverdriveModel::X7 => "Everdrive X7",
            EverdriveModel::Pro => "Everdrive Pro",
            EverdriveModel::Unknown => "Everdrive (Unknown)",
        }
    }

    /// Check if this model supports USB
    pub fn supports_usb(&self) -> bool {
        matches!(self, EverdriveModel::X7 | EverdriveModel::Pro)
    }

    /// Check if this model supports live patching
    pub fn supports_live_patch(&self) -> bool {
        matches!(self, EverdriveModel::X7 | EverdriveModel::Pro)
    }

    /// Check if this model supports RTC
    pub fn supports_rtc(&self) -> bool {
        matches!(self, EverdriveModel::X6 | EverdriveModel::X7 | EverdriveModel::Pro)
    }
}

impl EverdriveDevice {
    /// USB Vendor ID for Everdrive
    pub const VID: u16 = 0x0483;
    /// USB Product ID for Everdrive
    pub const PID: u16 = 0x5740;

    /// Create a new Everdrive device
    pub fn new(model: EverdriveModel) -> Self {
        let mut info = DeviceInfo::new(model.name())
            .with_feature(DeviceFeature::Dsp)
            .with_feature(DeviceFeature::Cx4)
            .with_feature(DeviceFeature::Srtc);

        if model.supports_usb() {
            info = info.with_feature(DeviceFeature::Usb);
        }

        if model.supports_live_patch() {
            info = info.with_feature(DeviceFeature::LivePatch);
        }

        if model == EverdriveModel::Pro {
            info = info
                .with_feature(DeviceFeature::Cheats)
                .with_feature(DeviceFeature::SuperFx)
                .with_feature(DeviceFeature::Sa1)
                .with_feature(DeviceFeature::Obc1)
                .with_feature(DeviceFeature::Sdd1);
        }

        Self {
            info,
            connected: false,
            usb_handle: None,
            model,
        }
    }

    /// Create a new Everdrive with auto-detected model
    pub fn new_auto() -> Self {
        Self::new(EverdriveModel::Unknown)
    }

    /// Detect if an Everdrive device is connected
    ///
    /// # Errors
    ///
    /// Returns an error if USB enumeration fails
    pub fn detect() -> Result<Self> {
        // In a real implementation, this would enumerate USB devices
        // and query the device for its model
        let mut device = Self::new_auto();
        
        if device.is_present() {
            // Try to detect model
            device.detect_model()?;
            Ok(device)
        } else {
            Err(FlashCartError::DeviceNotFound(
                "Everdrive not found".to_string()
            ))
        }
    }

    /// Check if the device is present on USB
    fn is_present(&self) -> bool {
        // Placeholder - would enumerate USB devices
        false
    }

    /// Detect the specific Everdrive model
    fn detect_model(&mut self) -> Result<()> {
        // In a real implementation, this would query the device
        // For now, default to X7 as most common USB-capable model
        self.model = EverdriveModel::X7;
        self.info.name = self.model.name().to_string();
        Ok(())
    }

    /// Get the device model
    pub fn model(&self) -> EverdriveModel {
        self.model
    }

    /// Get the device info
    pub fn info(&self) -> &DeviceInfo {
        &self.info
    }

    /// Send a command to the device
    fn send_command(&mut self, command: EverdriveCommand) -> Result<Vec<u8>> {
        if !self.connected {
            return Err(FlashCartError::ConnectionFailed(
                "Device not connected".to_string()
            ));
        }

        if !self.model.supports_usb() {
            return Err(FlashCartError::NotSupported(
                "USB operations not supported on this model".to_string()
            ));
        }

        // Placeholder implementation
        match command {
            EverdriveCommand::GetVersion => Ok(b"3.06".to_vec()),
            EverdriveCommand::Reset => Ok(vec![]),
            _ => Ok(vec![]),
        }
    }
}

impl Default for EverdriveDevice {
    fn default() -> Self {
        Self::new_auto()
    }
}

impl FlashCart for EverdriveDevice {
    fn device_name(&self) -> &str {
        &self.info.name
    }

    fn firmware_version(&self) -> Option<&str> {
        self.info.firmware_version.as_deref()
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    fn connect(&mut self) -> Result<()> {
        if self.connected {
            return Ok(());
        }

        if !self.model.supports_usb() {
            return Err(FlashCartError::NotSupported(
                "USB not supported on this Everdrive model".to_string()
            ));
        }

        log::info!("Connecting to {}...", self.model.name());
        
        // Placeholder: simulate connection
        self.connected = true;
        self.usb_handle = Some(());

        // Query firmware version
        let response = self.send_command(EverdriveCommand::GetVersion)?;
        let version = String::from_utf8_lossy(&response);
        self.info.firmware_version = Some(version.to_string());

        log::info!("Connected to {}, firmware: {}", self.model.name(), version);
        Ok(())
    }

    fn disconnect(&mut self) {
        if self.connected {
            log::info!("Disconnecting from {}...", self.model.name());
            self.usb_handle = None;
            self.connected = false;
        }
    }

    fn upload_rom(
        &mut self,
        rom_data: &RomData,
        progress_callback: Option<&dyn Fn(usize, usize)>,
    ) -> Result<()> {
        if !self.connected {
            return Err(FlashCartError::ConnectionFailed(
                "Device not connected".to_string()
            ));
        }

        log::info!("Uploading ROM to {} ({} bytes)...", 
            self.model.name(), 
            rom_data.data().len()
        );

        // Placeholder: simulate upload with progress
        let total = rom_data.data().len();
        let chunk_size = 4096;
        let mut uploaded = 0;

        while uploaded < total {
            let end = (uploaded + chunk_size).min(total);
            uploaded = end;

            if let Some(cb) = progress_callback {
                cb(uploaded, total);
            }
        }

        log::info!("ROM upload complete");
        Ok(())
    }

    fn download_sram(
        &mut self,
        size: usize,
        progress_callback: Option<&dyn Fn(usize, usize)>,
    ) -> Result<Vec<u8>> {
        if !self.connected {
            return Err(FlashCartError::ConnectionFailed(
                "Device not connected".to_string()
            ));
        }

        log::info!("Downloading SRAM from {} ({} bytes)...", 
            self.model.name(), 
            size
        );

        // Placeholder: simulate download
        let mut data = vec![0u8; size];
        let chunk_size = 4096;
        let mut downloaded = 0;

        while downloaded < size {
            let end = (downloaded + chunk_size).min(size);
            downloaded = end;

            if let Some(cb) = progress_callback {
                cb(downloaded, size);
            }
        }

        Ok(data)
    }

    fn upload_sram(&mut self, sram_data: &[u8]) -> Result<()> {
        if !self.connected {
            return Err(FlashCartError::ConnectionFailed(
                "Device not connected".to_string()
            ));
        }

        log::info!("Uploading SRAM to {} ({} bytes)...", 
            self.model.name(), 
            sram_data.len()
        );
        Ok(())
    }

    fn patch_live(&mut self, address: u32, data: &[u8]) -> Result<()> {
        if !self.supports_live_patch() {
            return Err(FlashCartError::NotSupported(
                "Live patching not supported on this model".to_string()
            ));
        }

        if !self.connected {
            return Err(FlashCartError::ConnectionFailed(
                "Device not connected".to_string()
            ));
        }

        log::info!("Applying live patch at ${:06X} ({} bytes)", address, data.len());
        
        // Send command for live patching
        self.send_command(EverdriveCommand::WriteRam {
            address,
            data: data.to_vec(),
        })?;

        Ok(())
    }

    fn supports_live_patch(&self) -> bool {
        self.model.supports_live_patch()
    }

    fn reset(&mut self) -> Result<()> {
        log::info!("Resetting SNES via Everdrive...");
        self.send_command(EverdriveCommand::Reset)?;
        Ok(())
    }

    fn device_info(&self) -> DeviceInfo {
        self.info.clone()
    }
}

impl Drop for EverdriveDevice {
    fn drop(&mut self) {
        self.disconnect();
    }
}

/// Everdrive USB commands
#[derive(Debug)]
#[allow(dead_code)]
enum EverdriveCommand {
    /// Get firmware version
    GetVersion,
    /// Reset SNES
    Reset,
    /// Write ROM data
    WriteRom {
        /// Address
        address: u32,
        /// Data
        data: Vec<u8>,
    },
    /// Read SRAM
    ReadSram {
        /// Address
        address: u32,
        /// Size
        size: usize,
    },
    /// Write SRAM
    WriteSram {
        /// Address
        address: u32,
        /// Data
        data: Vec<u8>,
    },
    /// Write to RAM (live patch)
    WriteRam {
        /// Address
        address: u32,
        /// Data
        data: Vec<u8>,
    },
    /// Get status
    GetStatus,
    /// Set features
    SetFeatures(u32),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_everdrive_models() {
        assert!(!EverdriveModel::X5.supports_usb());
        assert!(!EverdriveModel::X6.supports_usb());
        assert!(EverdriveModel::X7.supports_usb());
        assert!(EverdriveModel::Pro.supports_usb());

        assert!(!EverdriveModel::X5.supports_live_patch());
        assert!(EverdriveModel::X7.supports_live_patch());
        assert!(EverdriveModel::Pro.supports_live_patch());

        assert!(!EverdriveModel::X5.supports_rtc());
        assert!(EverdriveModel::X6.supports_rtc());
    }

    #[test]
    fn test_everdrive_names() {
        assert_eq!(EverdriveModel::X5.name(), "Everdrive X5");
        assert_eq!(EverdriveModel::X7.name(), "Everdrive X7");
        assert_eq!(EverdriveModel::Pro.name(), "Everdrive Pro");
    }

    #[test]
    fn test_everdrive_device() {
        let device = EverdriveDevice::new(EverdriveModel::X7);
        assert_eq!(device.device_name(), "Everdrive X7");
        assert!(device.supports_live_patch());
    }

    #[test]
    fn test_everdrive_usb_ids() {
        assert_eq!(EverdriveDevice::VID, 0x0483);
        assert_eq!(EverdriveDevice::PID, 0x5740);
    }
}
