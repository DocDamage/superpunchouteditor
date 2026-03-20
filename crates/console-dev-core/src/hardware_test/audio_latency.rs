//! Audio Latency Test
//!
//! Measures audio output latency on hardware vs emulator.
//! Important for rhythm games and audio-reactive effects.

use super::{HardwareTest, HardwareTestError, Result, TestConfig, TestResult};
use emulator_core::Emulator;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// Configuration for audio latency test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioLatencyConfig {
    /// Base test configuration
    pub base: TestConfig,
    /// Number of samples to collect
    pub sample_count: u32,
    /// Test tone frequency (Hz)
    pub tone_frequency: f64,
    /// Sample rate
    pub sample_rate: u32,
    /// Audio buffer size
    pub buffer_size: usize,
    /// Measure DSP processing time
    pub measure_dsp_time: bool,
    /// Measure DAC latency
    pub measure_dac_latency: bool,
    /// Test SPC700 communication
    pub test_spc700_comm: bool,
}

impl Default for AudioLatencyConfig {
    fn default() -> Self {
        Self {
            base: TestConfig::new("Audio Latency Test")
                .with_description("Measures audio output latency")
                .with_tolerance(0.05), // 5% tolerance (audio is more variable)
            sample_count: 50,
            tone_frequency: 1000.0, // 1kHz test tone
            sample_rate: 32000, // SPC700 output rate
            buffer_size: 512,
            measure_dsp_time: true,
            measure_dac_latency: true,
            test_spc700_comm: true,
        }
    }
}

impl AudioLatencyConfig {
    /// Create a new audio latency configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the number of samples
    pub fn with_sample_count(mut self, count: u32) -> Self {
        self.sample_count = count;
        self
    }

    /// Set the test tone frequency
    pub fn with_tone_frequency(mut self, freq: f64) -> Self {
        self.tone_frequency = freq;
        self
    }

    /// Set the sample rate
    pub fn with_sample_rate(mut self, rate: u32) -> Self {
        self.sample_rate = rate;
        self
    }

    /// Set the buffer size
    pub fn with_buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = size;
        self
    }

    /// Enable DSP processing time measurement
    pub fn with_dsp_time(mut self) -> Self {
        self.measure_dsp_time = true;
        self
    }

    /// Enable DAC latency measurement
    pub fn with_dac_latency(mut self) -> Self {
        self.measure_dac_latency = true;
        self
    }

    /// Enable SPC700 communication test
    pub fn with_spc700_comm(mut self) -> Self {
        self.test_spc700_comm = true;
        self
    }
}

/// Audio latency measurement method
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AudioLatencyMethod {
    /// Loopback via audio interface
    Loopback,
    /// Oscilloscope measurement
    Oscilloscope,
    /// Microphone pickup
    Microphone,
    /// SPC700 timer based
    Spc700Timer,
}

/// Audio latency measurement
#[derive(Debug, Clone)]
pub struct AudioLatencyMeasurement {
    /// Total latency from trigger to output
    pub total_latency: Duration,
    /// SPC700 processing latency
    pub spc700_latency: Duration,
    /// DSP processing latency
    pub dsp_latency: Duration,
    /// DAC output latency
    pub dac_latency: Duration,
    /// Buffer latency
    pub buffer_latency: Duration,
    /// Number of samples processed
    pub samples_processed: u32,
}

/// Audio latency test
#[derive(Debug)]
pub struct AudioLatency {
    /// Test configuration
    config: AudioLatencyConfig,
    /// Hardware measurements
    hardware_measurements: Vec<AudioLatencyMeasurement>,
    /// Emulator measurements
    emulator_measurements: Vec<AudioLatencyMeasurement>,
}

impl AudioLatency {
    /// Create a new audio latency test
    pub fn new(config: AudioLatencyConfig) -> Self {
        Self {
            config,
            hardware_measurements: Vec::new(),
            emulator_measurements: Vec::new(),
        }
    }

    /// Create with default configuration
    pub fn default_config() -> Self {
        Self::new(AudioLatencyConfig::default())
    }

    /// Calculate statistics from measurements
    fn calculate_stats(&self, measurements: &[AudioLatencyMeasurement]) -> AudioLatencyStats {
        if measurements.is_empty() {
            return AudioLatencyStats::default();
        }

        let count = measurements.len() as f64;

        let total_sum: f64 = measurements.iter()
            .map(|m| m.total_latency.as_secs_f64())
            .sum();
        let spc700_sum: f64 = measurements.iter()
            .map(|m| m.spc700_latency.as_secs_f64())
            .sum();
        let dsp_sum: f64 = measurements.iter()
            .map(|m| m.dsp_latency.as_secs_f64())
            .sum();
        let dac_sum: f64 = measurements.iter()
            .map(|m| m.dac_latency.as_secs_f64())
            .sum();

        let avg_total = Duration::from_secs_f64(total_sum / count);
        let min_total = measurements.iter().map(|m| m.total_latency).min().unwrap();
        let max_total = measurements.iter().map(|m| m.total_latency).max().unwrap();

        // Calculate variance
        let avg_total_secs = total_sum / count;
        let variance: f64 = measurements.iter()
            .map(|m| {
                let diff = m.total_latency.as_secs_f64() - avg_total_secs;
                diff * diff
            })
            .sum::<f64>() / count;

        AudioLatencyStats {
            sample_count: measurements.len() as u32,
            avg_total_latency: avg_total,
            min_total_latency: min_total,
            max_total_latency: max_total,
            avg_spc700_latency: Duration::from_secs_f64(spc700_sum / count),
            avg_dsp_latency: Duration::from_secs_f64(dsp_sum / count),
            avg_dac_latency: Duration::from_secs_f64(dac_sum / count),
            variance,
            std_dev: Duration::from_secs_f64(variance.sqrt()),
        }
    }

    /// Generate a test SPC file or ROM
    fn generate_test_audio(&self) -> Vec<u8> {
        // In a real implementation, this would generate:
        // - An SPC file with a test tone
        // - Or a minimal ROM that plays a tone on trigger
        // Placeholder: return empty vector
        vec![]
    }

    /// Calculate expected buffer latency
    fn calculate_buffer_latency(&self) -> Duration {
        let samples_per_buffer = self.config.buffer_size as f64;
        let sample_rate = self.config.sample_rate as f64;
        Duration::from_secs_f64(samples_per_buffer / sample_rate)
    }
}

impl HardwareTest for AudioLatency {
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
        log::info!("Running audio latency test on hardware...");
        
        let start_time = Instant::now();
        let mut measurements = Vec::new();

        // Generate test audio data
        let test_audio = self.generate_test_audio();
        log::debug!("Generated test audio ({} bytes)", test_audio.len());

        // In a real implementation:
        // 1. Upload test ROM or SPC to flash cart
        // 2. Connect audio capture device
        // 3. Trigger audio and measure latency
        
        // Placeholder: simulate measurements
        let buffer_latency = self.calculate_buffer_latency();
        
        for _ in 0..self.config.sample_count {
            let measurement = AudioLatencyMeasurement {
                total_latency: Duration::from_millis(20 + fastrand::u64(0..10)),
                spc700_latency: Duration::from_micros(100),
                dsp_latency: Duration::from_micros(200),
                dac_latency: Duration::from_micros(500),
                buffer_latency,
                samples_processed: self.config.buffer_size as u32,
            };
            measurements.push(measurement);
        }

        let duration = start_time.elapsed();
        let stats = self.calculate_stats(&measurements);

        log::info!("Hardware audio latency: {:?} avg, {:?} min, {:?} max",
            stats.avg_total_latency, stats.min_total_latency, stats.max_total_latency);

        let result = TestResult::new(self.name())
            .pass()
            .with_duration(duration)
            .with_metric("avg_latency_ms", stats.avg_total_latency.as_secs_f64() * 1000.0)
            .with_metric("min_latency_ms", stats.min_total_latency.as_secs_f64() * 1000.0)
            .with_metric("max_latency_ms", stats.max_total_latency.as_secs_f64() * 1000.0)
            .with_metric("std_dev_ms", stats.std_dev.as_secs_f64() * 1000.0)
            .with_metric("spc700_latency_ms", stats.avg_spc700_latency.as_secs_f64() * 1000.0)
            .with_metric("dsp_latency_ms", stats.avg_dsp_latency.as_secs_f64() * 1000.0)
            .with_metric("dac_latency_ms", stats.avg_dac_latency.as_secs_f64() * 1000.0)
            .with_metric("sample_count", stats.sample_count as f64);

        self.hardware_measurements = measurements;

        Ok(result)
    }

    fn run_emulator(&mut self, emulator: &mut dyn Emulator) -> Result<TestResult> {
        log::info!("Running audio latency test on emulator...");

        let start_time = Instant::now();
        let mut measurements = Vec::new();

        // Load test audio in emulator
        // emulator.load_audio(&test_audio)?;

        // Simulate measurements
        let buffer_latency = self.calculate_buffer_latency();
        
        for _ in 0..self.config.sample_count {
            let measurement = AudioLatencyMeasurement {
                total_latency: Duration::from_millis(15 + fastrand::u64(0..5)),
                spc700_latency: Duration::from_micros(50),
                dsp_latency: Duration::from_micros(100),
                dac_latency: buffer_latency / 2, // Emulated DAC is instant
                buffer_latency,
                samples_processed: self.config.buffer_size as u32,
            };
            measurements.push(measurement);
        }

        let duration = start_time.elapsed();
        let stats = self.calculate_stats(&measurements);

        log::info!("Emulator audio latency: {:?} avg, {:?} min, {:?} max",
            stats.avg_total_latency, stats.min_total_latency, stats.max_total_latency);

        let result = TestResult::new(self.name())
            .pass()
            .with_duration(duration)
            .with_metric("avg_latency_ms", stats.avg_total_latency.as_secs_f64() * 1000.0)
            .with_metric("min_latency_ms", stats.min_total_latency.as_secs_f64() * 1000.0)
            .with_metric("max_latency_ms", stats.max_total_latency.as_secs_f64() * 1000.0)
            .with_metric("std_dev_ms", stats.std_dev.as_secs_f64() * 1000.0)
            .with_metric("spc700_latency_ms", stats.avg_spc700_latency.as_secs_f64() * 1000.0)
            .with_metric("dsp_latency_ms", stats.avg_dsp_latency.as_secs_f64() * 1000.0)
            .with_metric("dac_latency_ms", stats.avg_dac_latency.as_secs_f64() * 1000.0)
            .with_metric("sample_count", stats.sample_count as f64);

        self.emulator_measurements = measurements;

        Ok(result)
    }

    fn compare(&self, hardware_result: &TestResult, emulator_result: &TestResult) -> Result<()> {
        let tolerance = self.config.base.tolerance;

        let hw_latency = hardware_result.get_metric("avg_latency_ms").unwrap_or(0.0);
        let emu_latency = emulator_result.get_metric("avg_latency_ms").unwrap_or(0.0);

        let latency_diff = (hw_latency - emu_latency).abs();
        let avg_latency = (hw_latency + emu_latency) / 2.0;
        let relative_diff = if avg_latency > 0.0 { latency_diff / avg_latency } else { 0.0 };

        log::info!("Audio latency comparison:");
        log::info!("  Hardware: {:.2} ms avg", hw_latency);
        log::info!("  Emulator: {:.2} ms avg", emu_latency);
        log::info!("  Difference: {:.2}%", relative_diff * 100.0);

        if relative_diff > tolerance {
            return Err(HardwareTestError::ComparisonFailed {
                expected: format!("{:.2} ms (hardware)", hw_latency),
                actual: format!("{:.2} ms (emulator)", emu_latency),
            });
        }

        Ok(())
    }
}

/// Statistics for audio latency measurements
#[derive(Debug, Clone, Default)]
pub struct AudioLatencyStats {
    /// Number of samples
    pub sample_count: u32,
    /// Average total latency
    pub avg_total_latency: Duration,
    /// Minimum total latency
    pub min_total_latency: Duration,
    /// Maximum total latency
    pub max_total_latency: Duration,
    /// Average SPC700 latency
    pub avg_spc700_latency: Duration,
    /// Average DSP latency
    pub avg_dsp_latency: Duration,
    /// Average DAC latency
    pub avg_dac_latency: Duration,
    /// Variance
    pub variance: f64,
    /// Standard deviation
    pub std_dev: Duration,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_latency_config_default() {
        let config = AudioLatencyConfig::default();
        assert_eq!(config.sample_count, 50);
        assert!((config.tone_frequency - 1000.0).abs() < 0.01);
        assert_eq!(config.sample_rate, 32000);
    }

    #[test]
    fn test_audio_latency_config_builder() {
        let config = AudioLatencyConfig::new()
            .with_sample_count(100)
            .with_tone_frequency(440.0)
            .with_sample_rate(48000)
            .with_buffer_size(1024);

        assert_eq!(config.sample_count, 100);
        assert!((config.tone_frequency - 440.0).abs() < 0.01);
        assert_eq!(config.sample_rate, 48000);
        assert_eq!(config.buffer_size, 1024);
    }

    #[test]
    fn test_buffer_latency_calculation() {
        let test = AudioLatency::new(AudioLatencyConfig::default());
        let latency = test.calculate_buffer_latency();
        // 512 samples at 32000 Hz = 16ms
        assert!(latency.as_millis() >= 15 && latency.as_millis() <= 17);
    }

    #[test]
    fn test_audio_latency_new() {
        let test = AudioLatency::default_config();
        assert_eq!(test.name(), "Audio Latency Test");
    }

    #[test]
    fn test_audio_latency_stats_default() {
        let stats = AudioLatencyStats::default();
        assert_eq!(stats.sample_count, 0);
    }
}
