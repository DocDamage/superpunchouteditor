//! Input Lag Test
//!
//! Measures controller input latency on hardware vs emulator.
//! Important for games requiring precise timing like fighting games.

use super::{HardwareTest, HardwareTestError, Result, TestConfig, TestResult};
use emulator_core::Emulator;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// Configuration for input lag test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputLagConfig {
    /// Base test configuration
    pub base: TestConfig,
    /// Number of input samples to collect
    pub sample_count: u32,
    /// Buttons to test
    pub buttons_to_test: Vec<Button>,
    /// Test method
    pub method: InputLagMethod,
    /// Display response mode
    pub display_mode: DisplayResponseMode,
    /// Include USB/polling latency
    pub include_polling_latency: bool,
}

impl Default for InputLagConfig {
    fn default() -> Self {
        Self {
            base: TestConfig::new("Input Lag Test")
                .with_description("Measures controller input latency")
                .with_tolerance(0.02), // 2% tolerance
            sample_count: 100,
            buttons_to_test: vec![Button::A, Button::B, Button::Start],
            method: InputLagMethod::Photodiode,
            display_mode: DisplayResponseMode::Instant,
            include_polling_latency: true,
        }
    }
}

impl InputLagConfig {
    /// Create a new input lag configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the number of samples
    pub fn with_sample_count(mut self, count: u32) -> Self {
        self.sample_count = count;
        self
    }

    /// Add a button to test
    pub fn with_button(mut self, button: Button) -> Self {
        self.buttons_to_test.push(button);
        self
    }

    /// Set the test method
    pub fn with_method(mut self, method: InputLagMethod) -> Self {
        self.method = method;
        self
    }

    /// Set the display response mode
    pub fn with_display_mode(mut self, mode: DisplayResponseMode) -> Self {
        self.display_mode = mode;
        self
    }

    /// Enable polling latency measurement
    pub fn with_polling_latency(mut self) -> Self {
        self.include_polling_latency = true;
        self
    }
}

/// SNES controller buttons
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Button {
    /// B button
    B,
    /// Y button
    Y,
    /// Select button
    Select,
    /// Start button
    Start,
    /// Up on D-pad
    Up,
    /// Down on D-pad
    Down,
    /// Left on D-pad
    Left,
    /// Right on D-pad
    Right,
    /// A button
    A,
    /// X button
    X,
    /// L shoulder
    L,
    /// R shoulder
    R,
}

impl Button {
    /// Get the button mask for SNES register $4218
    pub fn mask(&self) -> u16 {
        match self {
            Button::B => 0x8000,
            Button::Y => 0x4000,
            Button::Select => 0x2000,
            Button::Start => 0x1000,
            Button::Up => 0x0800,
            Button::Down => 0x0400,
            Button::Left => 0x0200,
            Button::Right => 0x0100,
            Button::A => 0x0080,
            Button::X => 0x0040,
            Button::L => 0x0020,
            Button::R => 0x0010,
        }
    }
}

/// Input lag measurement method
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum InputLagMethod {
    /// Use photodiode on screen
    Photodiode,
    /// Use high-speed camera
    HighSpeedCamera,
    /// Use electrical signal on controller port
    Electrical,
    /// Use frame counter in ROM
    FrameCounter,
}

/// Display response mode
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DisplayResponseMode {
    /// Instant response (change color immediately)
    Instant,
    /// On next V-blank
    Vblank,
    /// On specific scanline
    Scanline(u16),
}

/// Input lag measurement
#[derive(Debug, Clone)]
pub struct LagMeasurement {
    /// The button tested
    pub button: Button,
    /// Time from button press to display response
    pub total_lag: Duration,
    /// Polling latency
    pub polling_latency: Duration,
    /// Processing latency (CPU)
    pub processing_latency: Duration,
    /// Display latency
    pub display_latency: Duration,
    /// Frame number when input was registered
    pub frame_number: u32,
}

/// Input lag test
#[derive(Debug)]
pub struct InputLag {
    /// Test configuration
    config: InputLagConfig,
    /// Hardware measurements
    hardware_measurements: Vec<LagMeasurement>,
    /// Emulator measurements
    emulator_measurements: Vec<LagMeasurement>,
}

impl InputLag {
    /// Create a new input lag test
    pub fn new(config: InputLagConfig) -> Self {
        Self {
            config,
            hardware_measurements: Vec::new(),
            emulator_measurements: Vec::new(),
        }
    }

    /// Create with default configuration
    pub fn default_config() -> Self {
        Self::new(InputLagConfig::default())
    }

    /// Calculate statistics from measurements
    fn calculate_stats(&self, measurements: &[LagMeasurement]) -> LagStats {
        if measurements.is_empty() {
            return LagStats::default();
        }

        let count = measurements.len() as f64;
        
        let total_lag_sum: f64 = measurements.iter()
            .map(|m| m.total_lag.as_secs_f64())
            .sum();
        let polling_sum: f64 = measurements.iter()
            .map(|m| m.polling_latency.as_secs_f64())
            .sum();
        let processing_sum: f64 = measurements.iter()
            .map(|m| m.processing_latency.as_secs_f64())
            .sum();
        let display_sum: f64 = measurements.iter()
            .map(|m| m.display_latency.as_secs_f64())
            .sum();

        let avg_total = Duration::from_secs_f64(total_lag_sum / count);
        let min_total = measurements.iter().map(|m| m.total_lag).min().unwrap();
        let max_total = measurements.iter().map(|m| m.total_lag).max().unwrap();

        // Calculate variance
        let avg_total_secs = total_lag_sum / count;
        let variance: f64 = measurements.iter()
            .map(|m| {
                let diff = m.total_lag.as_secs_f64() - avg_total_secs;
                diff * diff
            })
            .sum::<f64>() / count;

        LagStats {
            sample_count: measurements.len() as u32,
            avg_total_lag: avg_total,
            min_total_lag: min_total,
            max_total_lag: max_total,
            avg_polling_latency: Duration::from_secs_f64(polling_sum / count),
            avg_processing_latency: Duration::from_secs_f64(processing_sum / count),
            avg_display_latency: Duration::from_secs_f64(display_sum / count),
            variance,
            std_dev: Duration::from_secs_f64(variance.sqrt()),
        }
    }

    /// Generate a test ROM for the specified buttons
    fn generate_test_rom(&self) -> Vec<u8> {
        // In a real implementation, this would generate a minimal SNES ROM
        // that displays a visual response when buttons are pressed
        // Placeholder: return empty vector
        vec![]
    }
}

impl HardwareTest for InputLag {
    fn name(&self) -> &str {
        &self.config.base.name
    }

    fn description(&self) -> &str {
        &self.config.base.description
    }

    fn config(&self) -> &TestConfig {
        &self.config.base
    }

    fn config_mut(&mut self) -> &mut TestConfig {
        &mut self.config.base
    }

    fn run_hardware(&mut self) -> Result<TestResult> {
        log::info!("Running input lag test on hardware...");
        
        let start_time = Instant::now();
        let mut measurements = Vec::new();

        // Generate and upload test ROM
        let test_rom = self.generate_test_rom();
        log::debug!("Generated test ROM ({} bytes)", test_rom.len());

        // In a real implementation:
        // 1. Upload test ROM to flash cart
        // 2. Connect automated controller or use manual input
        // 3. Measure response time using photodiode/high-speed camera
        
        // Placeholder: simulate measurements
        for button in &self.config.buttons_to_test {
            let samples_per_button = self.config.sample_count / self.config.buttons_to_test.len() as u32;
            for _ in 0..samples_per_button {
                let measurement = LagMeasurement {
                    button: *button,
                    total_lag: Duration::from_millis(16 + fastrand::u64(0..5)),
                    polling_latency: Duration::from_millis(1),
                    processing_latency: Duration::from_millis(2),
                    display_latency: Duration::from_millis(13),
                    frame_number: 0,
                };
                measurements.push(measurement);
            }
        }

        let duration = start_time.elapsed();
        let stats = self.calculate_stats(&measurements);

        log::info!("Hardware input lag: {:?} avg, {:?} min, {:?} max",
            stats.avg_total_lag, stats.min_total_lag, stats.max_total_lag);

        let result = TestResult::new(self.name())
            .pass()
            .with_duration(duration)
            .with_metric("avg_lag_ms", stats.avg_total_lag.as_secs_f64() * 1000.0)
            .with_metric("min_lag_ms", stats.min_total_lag.as_secs_f64() * 1000.0)
            .with_metric("max_lag_ms", stats.max_total_lag.as_secs_f64() * 1000.0)
            .with_metric("std_dev_ms", stats.std_dev.as_secs_f64() * 1000.0)
            .with_metric("sample_count", stats.sample_count as f64)
            .with_metric("avg_polling_ms", stats.avg_polling_latency.as_secs_f64() * 1000.0)
            .with_metric("avg_processing_ms", stats.avg_processing_latency.as_secs_f64() * 1000.0)
            .with_metric("avg_display_ms", stats.avg_display_latency.as_secs_f64() * 1000.0);

        self.hardware_measurements = measurements;

        Ok(result)
    }

    fn run_emulator(&mut self, emulator: &mut dyn Emulator) -> Result<TestResult> {
        log::info!("Running input lag test on emulator...");

        let start_time = Instant::now();
        let mut measurements = Vec::new();

        // Load test ROM in emulator
        // emulator.load_rom(&test_rom)?;

        // Simulate measurements
        for button in &self.config.buttons_to_test {
            let samples_per_button = self.config.sample_count / self.config.buttons_to_test.len() as u32;
            for _ in 0..samples_per_button {
                // Simulate button press and measure response
                let measurement = LagMeasurement {
                    button: *button,
                    total_lag: Duration::from_millis(16 + fastrand::u64(0..3)),
                    polling_latency: Duration::from_millis(0), // No USB polling in emulator
                    processing_latency: Duration::from_millis(1),
                    display_latency: Duration::from_millis(15),
                    frame_number: 0,
                };
                measurements.push(measurement);
            }
        }

        let duration = start_time.elapsed();
        let stats = self.calculate_stats(&measurements);

        log::info!("Emulator input lag: {:?} avg, {:?} min, {:?} max",
            stats.avg_total_lag, stats.min_total_lag, stats.max_total_lag);

        let result = TestResult::new(self.name())
            .pass()
            .with_duration(duration)
            .with_metric("avg_lag_ms", stats.avg_total_lag.as_secs_f64() * 1000.0)
            .with_metric("min_lag_ms", stats.min_total_lag.as_secs_f64() * 1000.0)
            .with_metric("max_lag_ms", stats.max_total_lag.as_secs_f64() * 1000.0)
            .with_metric("std_dev_ms", stats.std_dev.as_secs_f64() * 1000.0)
            .with_metric("sample_count", stats.sample_count as f64)
            .with_metric("avg_polling_ms", stats.avg_polling_latency.as_secs_f64() * 1000.0)
            .with_metric("avg_processing_ms", stats.avg_processing_latency.as_secs_f64() * 1000.0)
            .with_metric("avg_display_ms", stats.avg_display_latency.as_secs_f64() * 1000.0);

        self.emulator_measurements = measurements;

        Ok(result)
    }

    fn compare(&self, hardware_result: &TestResult, emulator_result: &TestResult) -> Result<()> {
        let tolerance = self.config.base.tolerance;

        let hw_lag = hardware_result.get_metric("avg_lag_ms").unwrap_or(0.0);
        let emu_lag = emulator_result.get_metric("avg_lag_ms").unwrap_or(0.0);

        let lag_diff = (hw_lag - emu_lag).abs();
        let avg_lag = (hw_lag + emu_lag) / 2.0;
        let relative_diff = if avg_lag > 0.0 { lag_diff / avg_lag } else { 0.0 };

        log::info!("Input lag comparison:");
        log::info!("  Hardware: {:.2} ms avg", hw_lag);
        log::info!("  Emulator: {:.2} ms avg", emu_lag);
        log::info!("  Difference: {:.2}%", relative_diff * 100.0);

        if relative_diff > tolerance {
            return Err(HardwareTestError::ComparisonFailed {
                expected: format!("{:.2} ms (hardware)", hw_lag),
                actual: format!("{:.2} ms (emulator)", emu_lag),
            });
        }

        Ok(())
    }
}

/// Statistics for lag measurements
#[derive(Debug, Clone, Default)]
pub struct LagStats {
    /// Number of samples
    pub sample_count: u32,
    /// Average total lag
    pub avg_total_lag: Duration,
    /// Minimum total lag
    pub min_total_lag: Duration,
    /// Maximum total lag
    pub max_total_lag: Duration,
    /// Average polling latency
    pub avg_polling_latency: Duration,
    /// Average processing latency
    pub avg_processing_latency: Duration,
    /// Average display latency
    pub avg_display_latency: Duration,
    /// Variance
    pub variance: f64,
    /// Standard deviation
    pub std_dev: Duration,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_lag_config_default() {
        let config = InputLagConfig::default();
        assert_eq!(config.sample_count, 100);
        assert!(config.buttons_to_test.contains(&Button::A));
    }

    #[test]
    fn test_input_lag_config_builder() {
        let config = InputLagConfig::new()
            .with_sample_count(200)
            .with_button(Button::L)
            .with_button(Button::R)
            .with_method(InputLagMethod::Electrical);

        assert_eq!(config.sample_count, 200);
        assert!(config.buttons_to_test.contains(&Button::L));
        assert!(config.buttons_to_test.contains(&Button::R));
        assert!(matches!(config.method, InputLagMethod::Electrical));
    }

    #[test]
    fn test_button_masks() {
        assert_eq!(Button::B.mask(), 0x8000);
        assert_eq!(Button::Y.mask(), 0x4000);
        assert_eq!(Button::A.mask(), 0x0080);
        assert_eq!(Button::Start.mask(), 0x1000);
    }

    #[test]
    fn test_input_lag_new() {
        let test = InputLag::default_config();
        assert_eq!(test.name(), "Input Lag Test");
    }

    #[test]
    fn test_lag_stats_default() {
        let stats = LagStats::default();
        assert_eq!(stats.sample_count, 0);
    }
}
