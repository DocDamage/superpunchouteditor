//! # Hardware Testing Module
//!
//! Provides tools for testing and validating hardware behavior against emulator output.
//! Useful for ensuring ROM hacks work correctly on real hardware.
//!
//! ## Test Types
//!
//! - **FrameTiming** - Measures frame timing accuracy
//! - **InputLag** - Measures controller input latency
//! - **AudioLatency** - Measures audio output latency
//! - **PowerConsumption** - Tracks power usage (placeholder)
//!
//! ## Example Usage
//!
//! ```rust,no_run
//! use console_dev_core::hardware_test::{
//!     HardwareTester, FrameTimingConfig, TestConfig
//! };
//!
//! let mut tester = HardwareTester::new();
//! tester.add_test(FrameTiming::new(FrameTimingConfig::default()));
//! let results = tester.run_all();
//! ```

use emulator_core::Emulator;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::time::Duration;
use thiserror::Error;

pub mod audio_latency;
pub mod frame_timing;
pub mod input_lag;
pub mod power_consumption;

pub use audio_latency::{AudioLatency, AudioLatencyConfig};
pub use frame_timing::{FrameTiming, FrameTimingConfig};
pub use input_lag::{InputLag, InputLagConfig};
pub use power_consumption::{PowerConsumption, PowerConsumptionConfig};

/// Errors that can occur during hardware testing
#[derive(Debug, Error)]
pub enum HardwareTestError {
    /// Test failed
    #[error("Test failed: {message}")]
    TestFailed {
        /// Test name
        test_name: String,
        /// Error message
        message: String,
    },

    /// Hardware not connected
    #[error("Hardware not connected: {0}")]
    HardwareNotConnected(String),

    /// Emulator error
    #[error("Emulator error: {0}")]
    EmulatorError(String),

    /// Comparison failed
    #[error("Comparison failed: expected {expected:?}, got {actual:?}")]
    ComparisonFailed {
        /// Expected value
        expected: String,
        /// Actual value
        actual: String,
    },

    /// Timeout
    #[error("Test timed out after {0:?}")]
    Timeout(Duration),

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type for hardware testing
pub type Result<T> = std::result::Result<T, HardwareTestError>;

/// Trait for hardware tests
///
/// Implement this trait to create custom hardware tests that can compare
/// emulator behavior against real hardware.
pub trait HardwareTest: Send + fmt::Debug {
    /// Get the test name
    fn name(&self) -> &str;

    /// Get the test description
    fn description(&self) -> &str;

    /// Run the test against hardware
    ///
    /// # Errors
    ///
    /// Returns an error if the test fails or hardware is unavailable
    fn run_hardware(&mut self) -> Result<TestResult>;

    /// Run the test against an emulator
    ///
    /// # Arguments
    ///
    /// * `emulator` - The emulator instance to test against
    ///
    /// # Errors
    ///
    /// Returns an error if the test fails
    fn run_emulator(&mut self, emulator: &mut dyn Emulator) -> Result<TestResult>;

    /// Compare hardware and emulator results
    ///
    /// # Arguments
    ///
    /// * `hardware_result` - Result from hardware test
    /// * `emulator_result` - Result from emulator test
    ///
    /// # Errors
    ///
    /// Returns an error if results don't match within tolerance
    fn compare(&self, hardware_result: &TestResult, emulator_result: &TestResult) -> Result<()>;

    /// Get the test configuration
    fn config(&self) -> &TestConfig;

    /// Get a mutable reference to the test configuration
    fn config_mut(&mut self) -> &mut TestConfig;
}

/// Configuration for hardware tests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConfig {
    /// Test name
    pub name: String,
    /// Test description
    pub description: String,
    /// Number of iterations to run
    pub iterations: u32,
    /// Timeout for the test
    pub timeout: Duration,
    /// Whether to run against emulator
    pub test_emulator: bool,
    /// Whether to run against hardware
    pub test_hardware: bool,
    /// Tolerance for comparisons (0.0 = exact match)
    pub tolerance: f64,
    /// Additional parameters
    pub parameters: HashMap<String, String>,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            name: "Unnamed Test".to_string(),
            description: String::new(),
            iterations: 1,
            timeout: Duration::from_secs(30),
            test_emulator: true,
            test_hardware: true,
            tolerance: 0.01, // 1% tolerance by default
            parameters: HashMap::new(),
        }
    }
}

impl TestConfig {
    /// Create a new test configuration
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    /// Set the description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Set the number of iterations
    pub fn with_iterations(mut self, iterations: u32) -> Self {
        self.iterations = iterations;
        self
    }

    /// Set the timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set emulator testing
    pub fn with_emulator(mut self, test: bool) -> Self {
        self.test_emulator = test;
        self
    }

    /// Set hardware testing
    pub fn with_hardware(mut self, test: bool) -> Self {
        self.test_hardware = test;
        self
    }

    /// Set tolerance for comparisons
    pub fn with_tolerance(mut self, tolerance: f64) -> Self {
        self.tolerance = tolerance;
        self
    }

    /// Add a parameter
    pub fn with_parameter(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.parameters.insert(key.into(), value.into());
        self
    }
}

/// Result of a hardware test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    /// Test name
    pub test_name: String,
    /// Whether the test passed
    pub passed: bool,
    /// Duration of the test
    pub duration: Duration,
    /// Metrics collected during the test
    pub metrics: HashMap<String, f64>,
    /// Raw data from the test
    pub raw_data: Vec<u8>,
    /// Error message (if failed)
    pub error_message: Option<String>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl TestResult {
    /// Create a new test result
    pub fn new(test_name: impl Into<String>) -> Self {
        Self {
            test_name: test_name.into(),
            passed: false,
            duration: Duration::default(),
            metrics: HashMap::new(),
            raw_data: Vec::new(),
            error_message: None,
            metadata: HashMap::new(),
        }
    }

    /// Mark as passed
    pub fn pass(mut self) -> Self {
        self.passed = true;
        self
    }

    /// Mark as failed
    pub fn fail(mut self, message: impl Into<String>) -> Self {
        self.passed = false;
        self.error_message = Some(message.into());
        self
    }

    /// Set duration
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    /// Add a metric
    pub fn with_metric(mut self, name: impl Into<String>, value: f64) -> Self {
        self.metrics.insert(name.into(), value);
        self
    }

    /// Add raw data
    pub fn with_raw_data(mut self, data: Vec<u8>) -> Self {
        self.raw_data = data;
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Get a metric value
    pub fn get_metric(&self, name: &str) -> Option<f64> {
        self.metrics.get(name).copied()
    }

    /// Compare this result with another within tolerance
    pub fn compare_with(&self, other: &TestResult, tolerance: f64) -> bool {
        if self.metrics.len() != other.metrics.len() {
            return false;
        }

        for (key, value1) in &self.metrics {
            if let Some(value2) = other.metrics.get(key) {
                let diff = (value1 - value2).abs();
                let avg = (value1 + value2) / 2.0;
                if avg > 0.0 && diff / avg > tolerance {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }
}

/// Main hardware tester
///
/// Manages multiple hardware tests and runs them against both
/// emulator and real hardware for comparison.
#[derive(Debug, Default)]
pub struct HardwareTester {
    /// Tests to run
    tests: Vec<Box<dyn HardwareTest>>,
    /// Last results
    last_results: Vec<TestResult>,
    /// Whether hardware is connected
    hardware_connected: bool,
    /// Default emulator instance (optional)
    emulator: Option<Box<dyn Emulator>>,
}

impl HardwareTester {
    /// Create a new hardware tester
    pub fn new() -> Self {
        Self {
            tests: Vec::new(),
            last_results: Vec::new(),
            hardware_connected: false,
            emulator: None,
        }
    }

    /// Create a new hardware tester with an emulator
    pub fn with_emulator(emulator: Box<dyn Emulator>) -> Self {
        Self {
            tests: Vec::new(),
            last_results: Vec::new(),
            hardware_connected: false,
            emulator: Some(emulator),
        }
    }

    /// Add a test
    pub fn add_test(&mut self, test: Box<dyn HardwareTest>) {
        self.tests.push(test);
    }

    /// Remove all tests
    pub fn clear_tests(&mut self) {
        self.tests.clear();
    }

    /// Get the number of tests
    pub fn test_count(&self) -> usize {
        self.tests.len()
    }

    /// Check if hardware is connected
    pub fn is_hardware_connected(&self) -> bool {
        self.hardware_connected
    }

    /// Set hardware connection state
    pub fn set_hardware_connected(&mut self, connected: bool) {
        self.hardware_connected = connected;
    }

    /// Set the emulator
    pub fn set_emulator(&mut self, emulator: Box<dyn Emulator>) {
        self.emulator = Some(emulator);
    }

    /// Run all tests
    ///
    /// Runs each test against both hardware and emulator (if available),
    /// then compares the results.
    ///
    /// # Errors
    ///
    /// Returns an error if any test fails critically
    pub fn run_all(&mut self) -> Result<Vec<TestComparison>> {
        let mut comparisons = Vec::new();

        for test in &mut self.tests {
            let comparison = self.run_test(test)?;
            comparisons.push(comparison);
        }

        Ok(comparisons)
    }

    /// Run a specific test
    ///
    /// # Arguments
    ///
    /// * `test` - The test to run
    ///
    /// # Errors
    ///
    /// Returns an error if the test fails critically
    pub fn run_test(&mut self, test: &mut dyn HardwareTest) -> Result<TestComparison> {
        let config = test.config().clone();
        
        log::info!("Running test: {}", config.name);

        let hardware_result = if config.test_hardware && self.hardware_connected {
            Some(test.run_hardware()?)
        } else {
            None
        };

        let emulator_result = if config.test_emulator {
            if let Some(ref mut emulator) = self.emulator {
                Some(test.run_emulator(emulator.as_mut())?)
            } else {
                None
            }
        } else {
            None
        };

        // Compare results if both are available
        let comparison = if let (Some(ref hw), Some(ref emu)) = (&hardware_result, &emulator_result) {
            let match_result = test.compare(hw, emu);
            
            TestComparison {
                test_name: config.name.clone(),
                hardware_result: Some(hw.clone()),
                emulator_result: Some(emu.clone()),
                matched: match_result.is_ok(),
                error: match_result.err().map(|e| e.to_string()),
            }
        } else {
            TestComparison {
                test_name: config.name.clone(),
                hardware_result,
                emulator_result,
                matched: false,
                error: None,
            }
        };

        Ok(comparison)
    }

    /// Run only hardware tests
    pub fn run_hardware_only(&mut self) -> Result<Vec<TestResult>> {
        let mut results = Vec::new();

        for test in &mut self.tests {
            if test.config().test_hardware {
                results.push(test.run_hardware()?);
            }
        }

        Ok(results)
    }

    /// Run only emulator tests
    pub fn run_emulator_only(&mut self) -> Result<Vec<TestResult>> {
        let mut results = Vec::new();

        if let Some(ref mut emulator) = self.emulator {
            for test in &mut self.tests {
                if test.config().test_emulator {
                    results.push(test.run_emulator(emulator.as_mut())?);
                }
            }
        }

        Ok(results)
    }

    /// Get the last results
    pub fn last_results(&self) -> &[TestResult] {
        &self.last_results
    }
}

/// Comparison of hardware vs emulator test results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestComparison {
    /// Test name
    pub test_name: String,
    /// Hardware test result (if run)
    pub hardware_result: Option<TestResult>,
    /// Emulator test result (if run)
    pub emulator_result: Option<TestResult>,
    /// Whether results matched within tolerance
    pub matched: bool,
    /// Error message (if comparison failed)
    pub error: Option<String>,
}

impl TestComparison {
    /// Check if both tests passed
    pub fn both_passed(&self) -> bool {
        let hw_passed = self.hardware_result.as_ref().map(|r| r.passed).unwrap_or(true);
        let emu_passed = self.emulator_result.as_ref().map(|r| r.passed).unwrap_or(true);
        hw_passed && emu_passed
    }

    /// Check if this is a complete comparison (both results available)
    pub fn is_complete(&self) -> bool {
        self.hardware_result.is_some() && self.emulator_result.is_some()
    }

    /// Generate a summary report
    pub fn summary(&self) -> String {
        let mut report = format!("Test: {}\n", self.test_name);
        
        if let Some(ref hw) = self.hardware_result {
            report.push_str(&format!("  Hardware: {}\n", 
                if hw.passed { "PASSED" } else { "FAILED" }));
        } else {
            report.push_str("  Hardware: NOT RUN\n");
        }

        if let Some(ref emu) = self.emulator_result {
            report.push_str(&format!("  Emulator: {}\n", 
                if emu.passed { "PASSED" } else { "FAILED" }));
        } else {
            report.push_str("  Emulator: NOT RUN\n");
        }

        if self.is_complete() {
            report.push_str(&format!("  Match: {}\n", 
                if self.matched { "YES" } else { "NO" }));
        }

        if let Some(ref error) = self.error {
            report.push_str(&format!("  Error: {}\n", error));
        }

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_test_config_default() {
        let config = TestConfig::default();
        assert_eq!(config.name, "Unnamed Test");
        assert_eq!(config.iterations, 1);
        assert!(config.test_emulator);
        assert!(config.test_hardware);
    }

    #[test]
    fn test_test_config_builder() {
        let config = TestConfig::new("My Test")
            .with_description("A test")
            .with_iterations(10)
            .with_tolerance(0.05);

        assert_eq!(config.name, "My Test");
        assert_eq!(config.description, "A test");
        assert_eq!(config.iterations, 10);
        assert_eq!(config.tolerance, 0.05);
    }

    #[test]
    fn test_test_result() {
        let result = TestResult::new("Test")
            .pass()
            .with_duration(Duration::from_secs(1))
            .with_metric("fps", 60.0);

        assert!(result.passed);
        assert_eq!(result.get_metric("fps"), Some(60.0));
    }

    #[test]
    fn test_hardware_tester_new() {
        let tester = HardwareTester::new();
        assert_eq!(tester.test_count(), 0);
        assert!(!tester.is_hardware_connected());
    }

    #[test]
    fn test_test_comparison() {
        let hw_result = TestResult::new("Test").pass();
        let emu_result = TestResult::new("Test").pass();

        let comparison = TestComparison {
            test_name: "Test".to_string(),
            hardware_result: Some(hw_result),
            emulator_result: Some(emu_result),
            matched: true,
            error: None,
        };

        assert!(comparison.both_passed());
        assert!(comparison.is_complete());
        assert!(comparison.matched);
    }
}
