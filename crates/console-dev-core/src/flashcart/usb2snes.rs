//! USB2SNES protocol implementation
//!
//! USB2SNES is a protocol for interfacing with SNES devices over USB.
//! It's supported by multiple devices including SD2SNES/FXPak Pro.

use super::{DeviceFeature, DeviceInfo, FlashCart, FlashCartError, Result};
use rom_core::RomData;
use std::fmt;

/// USB2SNES device
///
/// This is a protocol wrapper that can work with any USB2SNES-compatible device.
/// The SD2SNES/FXPak Pro is the primary device supporting this protocol.
#[derive(Debug)]
pub struct Usb2snesDevice {
    /// Device info
    info: DeviceInfo,
    /// Connection state
    connected: bool,
    /// WebSocket/connection handle (placeholder)
    connection: Option<()>,
    /// Server version
    server_version: Option<String>,
}

/// USB2SNES command names (as used in the protocol)
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
#[allow(clippy::upper_case_acronyms)]
pub enum Usb2snesCommand {
    /// Get device version
    DeviceVersion,
    /// List available devices
    ListDevices,
    /// Attach to a device
    Attach,
    /// Get firmware info
    Info,
    /// Send a boot command
    Boot,
    /// Send a menu command
    Menu,
    /// Reset the SNES
    Reset,
    /// Get address
    GetAddress,
    /// Put address
    PutAddress,
    /// Send data
    SendData,
    /// Receive data
    ReceiveData,
}

impl Usb2snesCommand {
    /// Get the command string for the protocol
    pub fn as_str(&self) -> &'static str {
        match self {
            Usb2snesCommand::DeviceVersion => "DeviceVersion",
            Usb2snesCommand::ListDevices => "ListDevices",
            Usb2snesCommand::Attach => "Attach",
            Usb2snesCommand::Info => "Info",
            Usb2snesCommand::Boot => "Boot",
            Usb2snesCommand::Menu => "Menu",
            Usb2snesCommand::Reset => "Reset",
            Usb2snesCommand::GetAddress => "GetAddress",
            Usb2snesCommand::PutAddress => "PutAddress",
            Usb2snesCommand::SendData => "SendData",
            Usb2snesCommand::ReceiveData => "ReceiveData",
        }
    }
}

/// USB2SNES response flags
#[derive(Debug, Clone)]
pub struct ResponseFlags {
    /// Operation success
    pub success: bool,
    /// Error message (if any)
    pub error: Option<String>,
}

impl Usb2snesDevice {
    /// Default USB2SNES server port
    pub const DEFAULT_PORT: u16 = 23074;
    /// USB2SNES WebSocket path
    pub const WS_PATH: &'static str = "/";

    /// Create a new USB2SNES device
    pub fn new() -> Self {
        let info = DeviceInfo::new("USB2SNES")
            .with_feature(DeviceFeature::Usb)
            .with_feature(DeviceFeature::LivePatch)
            .with_feature(DeviceFeature::Cheats)
            .with_feature(DeviceFeature::SaveStates);

        Self {
            info,
            connected: false,
            connection: None,
            server_version: None,
        }
    }

    /// Create a new USB2SNES device with custom server address
    pub fn with_server(host: &str, port: u16) -> Self {
        let mut device = Self::new();
        // Store server info for connection
        let _ = (host, port); // Placeholder
        device
    }

    /// Detect if a USB2SNES server is available
    ///
    /// This checks for the QUsb2Snes server or other USB2SNES-compatible servers
    ///
    /// # Errors
    ///
    /// Returns an error if detection fails
    pub fn detect() -> Result<Self> {
        let mut device = Self::new();

        // Check if server is available (placeholder)
        if device.is_server_available() {
            Ok(device)
        } else {
            Err(FlashCartError::DeviceNotFound(
                "USB2SNES server not found".to_string()
            ))
        }
    }

    /// Check if the USB2SNES server is available
    fn is_server_available(&self) -> bool {
        // Placeholder - would attempt WebSocket connection
        false
    }

    /// Get the server version
    pub fn server_version(&self) -> Option<&str> {
        self.server_version.as_deref()
    }

    /// Send a USB2SNES command
    fn send_command(
        &mut self,
        command: Usb2snesCommand,
        operands: Option<Vec<String>>,
    ) -> Result<Vec<u8>> {
        if !self.connected {
            return Err(FlashCartError::ConnectionFailed(
                "Not connected to USB2SNES server".to_string()
            ));
        }

        log::debug!("Sending USB2SNES command: {:?}", command);

        // Placeholder: actual WebSocket send would go here
        let _ = operands;

        // Simulate response
        match command {
            Usb2snesCommand::DeviceVersion => Ok(b"11.0".to_vec()),
            Usb2snesCommand::Info => Ok(b"SD2SNES".to_vec()),
            _ => Ok(vec![]),
        }
    }

    /// Get the device info
    pub fn info(&self) -> &DeviceInfo {
        &self.info
    }

    /// List available devices from the server
    pub fn list_devices(&mut self) -> Result<Vec<String>> {
        let response = self.send_command(Usb2snesCommand::ListDevices, None)?;
        // Parse response as JSON array of device names
        let devices: Vec<String> = serde_json::from_slice(&response)
            .map_err(|e| FlashCartError::ProtocolError(e.to_string()))?;
        Ok(devices)
    }

    /// Attach to a specific device
    pub fn attach(&mut self, device_name: &str) -> Result<()> {
        self.send_command(
            Usb2snesCommand::Attach,
            Some(vec![device_name.to_string()]),
        )?;
        log::info!("Attached to USB2SNES device: {}", device_name);
        Ok(())
    }

    /// Read memory from the SNES
    pub fn read_memory(&mut self, address: u32, size: usize) -> Result<Vec<u8>> {
        let space = "SNES".to_string();
        let flags = if size > 255 { "1" } else { "0" }.to_string();
        let addr_hex = format!("{:06X}", address);
        let size_hex = format!("{:06X}", size);

        self.send_command(
            Usb2snesCommand::GetAddress,
            Some(vec![space, flags, addr_hex, size_hex]),
        )?;

        // Receive data
        let mut data = Vec::with_capacity(size);
        let mut remaining = size;

        while remaining > 0 {
            let chunk_size = remaining.min(1024);
            let chunk = self.send_command(
                Usb2snesCommand::ReceiveData,
                None,
            )?;
            data.extend_from_slice(&chunk);
            remaining -= chunk.len();
        }

        Ok(data)
    }

    /// Write memory to the SNES (for live patching)
    pub fn write_memory(&mut self, address: u32, data: &[u8]) -> Result<()> {
        let space = "SNES".to_string();
        let flags = "0".to_string();
        let addr_hex = format!("{:06X}", address);
        let size_hex = format!("{:06X}", data.len());

        self.send_command(
            Usb2snesCommand::PutAddress,
            Some(vec![space, flags, addr_hex, size_hex]),
        )?;

        // Send data in chunks
        for chunk in data.chunks(1024) {
            self.send_command(
                Usb2snesCommand::SendData,
                None,
            )?;
        }

        Ok(())
    }
}

impl Default for Usb2snesDevice {
    fn default() -> Self {
        Self::new()
    }
}

impl FlashCart for Usb2snesDevice {
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

        log::info!("Connecting to USB2SNES server...");

        // Placeholder: actual WebSocket connection would go here
        self.connected = true;
        self.connection = Some(());

        // Query server version
        let version = self.send_command(Usb2snesCommand::DeviceVersion, None)?;
        self.server_version = Some(String::from_utf8_lossy(&version).to_string());

        // Query device info
        let info = self.send_command(Usb2snesCommand::Info, None)?;
        let device_name = String::from_utf8_lossy(&info);
        self.info.name = format!("USB2SNES ({})", device_name);

        log::info!("Connected to USB2SNES server v{}", 
            self.server_version.as_deref().unwrap_or("unknown"));

        Ok(())
    }

    fn disconnect(&mut self) {
        if self.connected {
            log::info!("Disconnecting from USB2SNES server...");
            self.connection = None;
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
                "Not connected".to_string()
            ));
        }

        // USB2SNES doesn't directly support ROM upload
        // It works with the device's SD card
        log::warn!("USB2SNES does not support direct ROM upload");
        log::info!("Use the device's SD card to load ROMs");

        // Simulate progress for compatibility
        if let Some(cb) = progress_callback {
            cb(0, rom_data.data().len());
        }

        Err(FlashCartError::NotSupported(
            "Direct ROM upload not supported via USB2SNES".to_string()
        ))
    }

    fn download_sram(
        &mut self,
        size: usize,
        progress_callback: Option<&dyn Fn(usize, usize)>,
    ) -> Result<Vec<u8>> {
        if !self.connected {
            return Err(FlashCartError::ConnectionFailed(
                "Not connected".to_string()
            ));
        }

        log::info!("Downloading SRAM via USB2SNES ({} bytes)...", size);

        // SRAM is at $700000-$800000 in SNES address space
        let sram_address = 0x700000;
        
        let mut data = Vec::with_capacity(size);
        let chunk_size = 1024;
        let mut downloaded = 0;

        while downloaded < size {
            let current_chunk = chunk_size.min(size - downloaded);
            let chunk = self.read_memory(sram_address + downloaded as u32, current_chunk)?;
            data.extend_from_slice(&chunk);
            downloaded += chunk.len();

            if let Some(cb) = progress_callback {
                cb(downloaded, size);
            }
        }

        Ok(data)
    }

    fn upload_sram(&mut self, sram_data: &[u8]) -> Result<()> {
        if !self.connected {
            return Err(FlashCartError::ConnectionFailed(
                "Not connected".to_string()
            ));
        }

        log::info!("Uploading SRAM via USB2SNES ({} bytes)...", sram_data.len());

        // SRAM is at $700000-$800000
        let sram_address = 0x700000;
        self.write_memory(sram_address, sram_data)?;

        Ok(())
    }

    fn patch_live(&mut self, address: u32, data: &[u8]) -> Result<()> {
        if !self.connected {
            return Err(FlashCartError::ConnectionFailed(
                "Not connected".to_string()
            ));
        }

        log::info!("Applying live patch via USB2SNES at ${:06X} ({} bytes)", 
            address, data.len());

        self.write_memory(address, data)?;
        Ok(())
    }

    fn supports_live_patch(&self) -> bool {
        true
    }

    fn reset(&mut self) -> Result<()> {
        log::info!("Resetting SNES via USB2SNES...");
        self.send_command(Usb2snesCommand::Reset, None)?;
        Ok(())
    }

    fn device_info(&self) -> DeviceInfo {
        self.info.clone()
    }
}

impl Drop for Usb2snesDevice {
    fn drop(&mut self) {
        self.disconnect();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_usb2snes_new() {
        let device = Usb2snesDevice::new();
        assert_eq!(device.device_name(), "USB2SNES");
        assert!(!device.is_connected());
    }

    #[test]
    fn test_usb2snes_commands() {
        assert_eq!(Usb2snesCommand::Reset.as_str(), "Reset");
        assert_eq!(Usb2snesCommand::GetAddress.as_str(), "GetAddress");
        assert_eq!(Usb2snesCommand::PutAddress.as_str(), "PutAddress");
    }

    #[test]
    fn test_usb2snes_default_port() {
        assert_eq!(Usb2snesDevice::DEFAULT_PORT, 23074);
    }

    #[test]
    fn test_usb2snes_features() {
        let device = Usb2snesDevice::new();
        let info = device.device_info();
        assert!(info.features.contains(&DeviceFeature::Usb));
        assert!(info.features.contains(&DeviceFeature::LivePatch));
    }
}
