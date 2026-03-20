//! # Console Development Core
//! 
//! Hardware development tools for serious ROM hackers. Provides deep hardware integration
//! for flash cart interfaces, hardware testing, and cartridge dumping capabilities.
//!
//! ## Modules
//!
//! - `flashcart` - Flash cart interfaces (SD2SNES, Everdrive, USB2SNES)
//! - `hardware_test` - Hardware testing and validation
//! - `dumper` - Cartridge dumping and analysis

#![deny(missing_docs)]
#![warn(rust_2018_idioms)]

use std::fmt;
use thiserror::Error;

pub mod dumper;
pub mod flashcart;
pub mod hardware_test;

// Re-exports from submodules
pub use dumper::{CartridgeDumper, DumperError};
pub use flashcart::{
    AutoDetect, EverdriveDevice, FlashCart, FlashCartError, Sd2snesDevice, Usb2snesDevice,
};
pub use hardware_test::{
    AudioLatency, AudioLatencyConfig, FrameTiming, FrameTimingConfig, HardwareTester,
    InputLag, InputLagConfig, PowerConsumption, PowerConsumptionConfig, TestConfig, TestResult,
};

/// Version of the console-dev-core crate
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Result type for console-dev-core operations
pub type Result<T> = std::result::Result<T, ConsoleDevError>;

/// Errors that can occur in console development operations
#[derive(Debug, Error)]
pub enum ConsoleDevError {
    /// Flash cart operation failed
    #[error("Flash cart error: {0}")]
    FlashCart(#[from] FlashCartError),

    /// Dumper operation failed
    #[error("Dumper error: {0}")]
    Dumper(#[from] DumperError),

    /// Hardware test failed
    #[error("Hardware test failed: {message}")]
    HardwareTest {
        /// The test that failed
        test_name: String,
        /// Error message
        message: String,
    },

    /// Device not found
    #[error("Device not found: {0}")]
    DeviceNotFound(String),

    /// Connection error
    #[error("Connection error: {message}")]
    Connection {
        /// Error message
        message: String,
    },

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Timeout error
    #[error("Operation timed out after {duration:?}")]
    Timeout {
        /// Duration before timeout
        duration: std::time::Duration,
    },
}

/// Main console development interface
///
/// This struct provides a unified interface for hardware development tools,
/// combining flash cart management, hardware testing, and cartridge dumping.
pub struct ConsoleDev {
    /// Current flash cart connection (if any)
    flash_cart: Option<Box<dyn FlashCart>>,
    /// Hardware tester instance
    hardware_tester: HardwareTester,
    /// Cartridge dumper instance
    dumper: Option<Box<dyn CartridgeDumper>>,
}

impl fmt::Debug for ConsoleDev {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ConsoleDev")
            .field("flash_cart_connected", &self.flash_cart.is_some())
            .field("hardware_tester", &self.hardware_tester)
            .field("dumper_connected", &self.dumper.is_some())
            .finish()
    }
}

impl Default for ConsoleDev {
    fn default() -> Self {
        Self::new()
    }
}

impl ConsoleDev {
    /// Create a new ConsoleDev instance
    pub fn new() -> Self {
        Self {
            flash_cart: None,
            hardware_tester: HardwareTester::new(),
            dumper: None,
        }
    }

    /// Create a new ConsoleDev with a connected flash cart
    ///
    /// # Errors
    ///
    /// Returns an error if auto-detection fails or no device is found
    pub fn with_flash_cart() -> Result<Self> {
        let mut dev = Self::new();
        dev.auto_connect_flash_cart()?;
        Ok(dev)
    }

    /// Get the current flash cart (if connected)
    pub fn flash_cart(&self) -> Option<&dyn FlashCart> {
        self.flash_cart.as_ref().map(|c| c.as_ref())
    }

    /// Get a mutable reference to the current flash cart (if connected)
    pub fn flash_cart_mut(&mut self) -> Option<&mut dyn FlashCart> {
        self.flash_cart.as_mut().map(|c| c.as_mut())
    }

    /// Connect to a flash cart
    pub fn connect_flash_cart(&mut self, cart: Box<dyn FlashCart>) {
        self.flash_cart = Some(cart);
    }

    /// Disconnect the current flash cart
    pub fn disconnect_flash_cart(&mut self) {
        self.flash_cart = None;
    }

    /// Auto-detect and connect to a flash cart
    ///
    /// Attempts to detect connected devices in order:
    /// 1. SD2SNES/FXPak Pro
    /// 2. Everdrive
    /// 3. USB2SNES
    ///
    /// # Errors
    ///
    /// Returns an error if no supported device is found
    pub fn auto_connect_flash_cart(&mut self) -> Result<()> {
        if let Some(device) = AutoDetect::detect_first()? {
            self.connect_flash_cart(device);
            Ok(())
        } else {
            Err(ConsoleDevError::DeviceNotFound(
                "No supported flash cart detected".to_string(),
            ))
        }
    }

    /// Get the hardware tester
    pub fn hardware_tester(&self) -> &HardwareTester {
        &self.hardware_tester
    }

    /// Get a mutable reference to the hardware tester
    pub fn hardware_tester_mut(&mut self) -> &mut HardwareTester {
        &mut self.hardware_tester
    }

    /// Get the cartridge dumper (if connected)
    pub fn dumper(&self) -> Option<&dyn CartridgeDumper> {
        self.dumper.as_ref().map(|d| d.as_ref())
    }

    /// Connect a cartridge dumper
    pub fn connect_dumper(&mut self, dumper: Box<dyn CartridgeDumper>) {
        self.dumper = Some(dumper);
    }

    /// Disconnect the cartridge dumper
    pub fn disconnect_dumper(&mut self) {
        self.dumper = None;
    }

    /// Check if a flash cart is connected
    pub fn is_flash_cart_connected(&self) -> bool {
        self.flash_cart.is_some()
    }

    /// Check if a cartridge dumper is connected
    pub fn is_dumper_connected(&self) -> bool {
        self.dumper.is_some()
    }
}

/// Builder for creating ConsoleDev instances with specific configurations
pub struct ConsoleDevBuilder {
    auto_connect_flash_cart: bool,
    auto_connect_dumper: bool,
}

impl Default for ConsoleDevBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ConsoleDevBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            auto_connect_flash_cart: false,
            auto_connect_dumper: false,
        }
    }

    /// Enable auto-connect for flash cart
    pub fn auto_connect_flash_cart(mut self) -> Self {
        self.auto_connect_flash_cart = true;
        self
    }

    /// Enable auto-connect for cartridge dumper
    pub fn auto_connect_dumper(mut self) -> Self {
        self.auto_connect_dumper = true;
        self
    }

    /// Build the ConsoleDev instance
    ///
    /// # Errors
    ///
    /// Returns an error if auto-connection is enabled but fails
    pub fn build(self) -> Result<ConsoleDev> {
        let mut dev = ConsoleDev::new();

        if self.auto_connect_flash_cart {
            dev.auto_connect_flash_cart()?;
        }

        if self.auto_connect_dumper {
            // Dumper auto-connect would be implemented here
            // For now, we leave it disconnected
        }

        Ok(dev)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_console_dev_new() {
        let dev = ConsoleDev::new();
        assert!(!dev.is_flash_cart_connected());
        assert!(!dev.is_dumper_connected());
    }

    #[test]
    fn test_console_dev_builder() {
        let dev = ConsoleDevBuilder::new().build().unwrap();
        assert!(!dev.is_flash_cart_connected());
    }

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}
