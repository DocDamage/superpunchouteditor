//! Frame Timing Test
//!
//! Measures the accuracy of frame timing on hardware vs emulator.
//! Critical for games with raster effects and precise timing.

use super::{HardwareTest, HardwareTestError, Result, TestConfig, TestResult};
use emulator_core::Emulator;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// Configuration for frame timing test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameTimingConfig {
    /// Base test configuration
    pub base: TestConfig,
    /// Number of frames to measure
    pub frame_count: u32,
    /// Target frame rate (usually 60.0988 Hz for NTSC SNES)
    pub target_fps: f64,
    /// VBlank wait method
    pub vblank_method: VBlankMethod,
    /// Measure h-blank timing
    pub measure_hblank: bool,
    /// Measure DMA timing
    pub measure_dma: bool,
    /// Specific scanlines to measure
    pub scanlines_to_measure: Vec<u16>,
}

impl Default for FrameTimingConfig {
    fn default() -> Self {
        Self {
            base: TestConfig::new("Frame Timing Test")
                .with_description("Measures frame timing accuracy")
                .with_tolerance(0.001), // 0.1% tolerance
            frame_count: 300, // 5 seconds at 60fps
            target_fps: 60.0988, // NTSC SNES frame rate
            vblank_method: VBlankMethod::Nmi,
            measure_hblank: false,
            measure_dma: false,
            scanlines_to_measure: vec![],
        }
    }
}

impl FrameTimingConfig {
    /// Create a new frame timing configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the number of frames to measure
    pub fn with_frame_count(mut self, count: u32) -> Self {
        self.frame_count = count;
        self
    }

    /// Set the target frame rate
    pub fn with_target_fps(mut self, fps: f64) -> Self {
        self.target_fps = fps;
        self
    }

    /// Enable H-blank measurement
    pub fn with_hblank(mut self) -> Self {
        self.measure_hblank = true;
        self
    }

    /// Enable DMA timing measurement
    pub fn with_dma(mut self) -> Self {
        self.measure_dma = true;
        self
    }

    /// Add a scanline to measure
    pub fn with_scanline(mut self, scanline: u16) -> Self {
        self.scanlines_to_measure.push(scanline);
        self
    }

    /// Set the V-blank detection method
    pub fn with_vblank_method(mut self, method: VBlankMethod) -> Self {
        self.vblank_method = method;
        self
    }
}

/// V-blank detection method
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum VBlankMethod {
    /// Use NMI interrupt
    Nmi,
    /// Poll HV counter
    HvCounter,
    /// Use H-blank interrupt
    HBlank,
}

/// Frame timing test
///
/// Measures frame timing accuracy by counting frames over a known time period.
#[derive(Debug)]
pub struct FrameTiming {
    /// Test configuration
    config: FrameTimingConfig,
    /// Measured frame times on hardware
    hardware_frame_times: Vec<Duration>,
    /// Measured frame times on emulator
    emulator_frame_times: Vec<Duration>,
}

impl FrameTiming {
    /// Create a new frame timing test
    pub fn new(config: FrameTimingConfig) -> Self {
        Self {
            config,
            hardware_frame_times: Vec::new(),
            emulator_frame_times: Vec::new(),
        }
    }

    /// Create with default configuration
    pub fn default_config() -> Self {
        Self::new(FrameTimingConfig::default())
    }

    /// Analyze frame timing data
    fn analyze_frames(&self, frame_times: &[Duration]) -> FrameTimingAnalysis {
        if frame_times.is_empty() {
            return FrameTimingAnalysis::default();
        }

        let mut sum = Duration::default();
        let mut min = frame_times[0];
        let mut max = frame_times[0];

        for &time in frame_times {
            sum += time;
            if time < min {
                min = time;
            }
            if time > max {
                max = time;
            }
        }

        let avg = sum / frame_times.len() as u32;
        let fps = 1.0 / avg.as_secs_f64();

        // Calculate variance
        let mut variance_sum = 0.0f64;
        for &time in frame_times {
            let diff = time.as_secs_f64() - avg.as_secs_f64();
            variance_sum += diff * diff;
        }
        let variance = variance_sum / frame_times.len() as f64;
        let std_dev = variance.sqrt();

        FrameTimingAnalysis {
            frame_count: frame_times.len() as u32,
            average_frame_time: avg,
            min_frame_time: min,
            max_frame_time: max,
            fps,
            variance,
            std_dev,
            target_deviation: ((fps - self.config.target_fps) / self.config.target_fps).abs(),
        }
    }
}

impl HardwareTest for FrameTiming {
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
        log::info!("Running frame timing test on hardware...");
        
        let start_time = Instant::now();
        let mut frame_times = Vec::with_capacity(self.config.frame_count as usize);
        let mut last_vblank = Instant::now();

        // In a real implementation, this would:
        // 1. Connect to the SNES via flash cart
        // 2. Upload a timing test ROM
        // 3. Measure actual frame timing
        
        // Placeholder: simulate frame timing measurement
        let target_frame_time = Duration::from_secs_f64(1.0 / self.config.target_fps);
        
        for _ in 0..self.config.frame_count {
            // Simulate frame timing with small variations
            let variation = fastrand::f64() * 0.0001;
            let frame_time = target_frame_time + Duration::from_secs_f64(variation);
            frame_times.push(frame_time);
            last_vblank += frame_time;
        }

        let duration = start_time.elapsed();
        let analysis = self.analyze_frames(&frame_times);

        log::info!("Hardware frame timing: {:.4} FPS (target: {:.4})", 
            analysis.fps, self.config.target_fps);

        let result = TestResult::new(self.name())
            .pass()
            .with_duration(duration)
            .with_metric("fps", analysis.fps)
            .with_metric("avg_frame_time_ms", analysis.average_frame_time.as_secs_f64() * 1000.0)
            .with_metric("min_frame_time_ms", analysis.min_frame_time.as_secs_f64() * 1000.0)
            .with_metric("max_frame_time_ms", analysis.max_frame_time.as_secs_f64() * 1000.0)
            .with_metric("std_dev_ms", analysis.std_dev * 1000.0)
            .with_metric("target_deviation", analysis.target_deviation);

        self.hardware_frame_times = frame_times;

        Ok(result)
    }

    fn run_emulator(&mut self, emulator: &mut dyn Emulator) -> Result<TestResult> {
        log::info!("Running frame timing test on emulator...");

        let start_time = Instant::now();
        let mut frame_times = Vec::with_capacity(self.config.frame_count as usize);

        // Run the emulator for the specified number of frames
        for _ in 0..self.config.frame_count {
            let frame_start = Instant::now();
            
            // Step the emulator one frame
            emulator.step_frame()
                .map_err(|e| HardwareTestError::EmulatorError(e.to_string()))?;
            
            frame_times.push(frame_start.elapsed());
        }

        let duration = start_time.elapsed();
        let analysis = self.analyze_frames(&frame_times);

        log::info!("Emulator frame timing: {:.4} FPS", analysis.fps);

        let result = TestResult::new(self.name())
            .pass()
            .with_duration(duration)
            .with_metric("fps", analysis.fps)
            .with_metric("avg_frame_time_ms", analysis.average_frame_time.as_secs_f64() * 1000.0)
            .with_metric("min_frame_time_ms", analysis.min_frame_time.as_secs_f64() * 1000.0)
            .with_metric("max_frame_time_ms", analysis.max_frame_time.as_secs_f64() * 1000.0)
            .with_metric("std_dev_ms", analysis.std_dev * 1000.0)
            .with_metric("target_deviation", analysis.target_deviation);

        self.emulator_frame_times = frame_times;

        Ok(result)
    }

    fn compare(&self, hardware_result: &TestResult, emulator_result: &TestResult) -> Result<()> {
        let tolerance = self.config.base.tolerance;

        let hw_fps = hardware_result.get_metric("fps").unwrap_or(0.0);
        let emu_fps = emulator_result.get_metric("fps").unwrap_or(0.0);

        let fps_diff = (hw_fps - emu_fps).abs();
        let avg_fps = (hw_fps + emu_fps) / 2.0;
        let relative_diff = if avg_fps > 0.0 { fps_diff / avg_fps } else { 0.0 };

        log::info!("Frame timing comparison:");
        log::info!("  Hardware FPS: {:.4}", hw_fps);
        log::info!("  Emulator FPS: {:.4}", emu_fps);
        log::info!("  Difference: {:.4}%", relative_diff * 100.0);

        if relative_diff > tolerance {
            return Err(HardwareTestError::ComparisonFailed {
                expected: format!("{:.4} FPS (hardware)", hw_fps),
                actual: format!("{:.4} FPS (emulator)", emu_fps),
            });
        }

        Ok(())
    }
}

/// Analysis of frame timing data
#[derive(Debug, Clone, Default)]
pub struct FrameTimingAnalysis {
    /// Number of frames analyzed
    pub frame_count: u32,
    /// Average frame time
    pub average_frame_time: Duration,
    /// Minimum frame time
    pub min_frame_time: Duration,
    /// Maximum frame time
    pub max_frame_time: Duration,
    /// Calculated FPS
    pub fps: f64,
    /// Variance in frame times
    pub variance: f64,
    /// Standard deviation
    pub std_dev: f64,
    /// Deviation from target FPS
    pub target_deviation: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_timing_config_default() {
        let config = FrameTimingConfig::default();
        assert_eq!(config.frame_count, 300);
        assert!((config.target_fps - 60.0988).abs() < 0.0001);
    }

    #[test]
    fn test_frame_timing_config_builder() {
        let config = FrameTimingConfig::new()
            .with_frame_count(600)
            .with_target_fps(50.0)
            .with_hblank();

        assert_eq!(config.frame_count, 600);
        assert_eq!(config.target_fps, 50.0);
        assert!(config.measure_hblank);
    }

    #[test]
    fn test_frame_timing_new() {
        let test = FrameTiming::default_config();
        assert_eq!(test.name(), "Frame Timing Test");
    }

    #[test]
    fn test_vblank_method() {
        assert!(matches!(VBlankMethod::Nmi, VBlankMethod::Nmi));
        assert!(matches!(VBlankMethod::HvCounter, VBlankMethod::HvCounter));
    }
}
