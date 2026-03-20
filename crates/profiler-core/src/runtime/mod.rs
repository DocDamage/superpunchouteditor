//! # Runtime Profiler
//!
//! CPU and memory performance profiler for SNES emulation.
//! Tracks function timing, memory bandwidth, CPU cycles, and VBlank usage.

use crate::{utils, ProfilerError, ProfilerTrait, Result, Severity, Bottleneck};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Performance report for runtime profiling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceReport {
    /// Total duration of the profiling session
    pub duration: Duration,
    /// Total CPU cycles executed
    pub total_cycles: u64,
    /// Cycles per routine
    pub routine_cycles: HashMap<String, u64>,
    /// Memory bandwidth statistics (bytes per second)
    pub memory_bandwidth: MemoryBandwidthStats,
    /// VBlank timing statistics
    pub vblank_stats: VBlankStats,
    /// Function timing data
    pub function_timings: Vec<FunctionTiming>,
    /// Identified bottlenecks
    pub bottlenecks: Vec<Bottleneck>,
}

/// Memory bandwidth statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryBandwidthStats {
    /// Average read bandwidth in bytes/second
    pub avg_read_bandwidth: f64,
    /// Average write bandwidth in bytes/second
    pub avg_write_bandwidth: f64,
    /// Peak read bandwidth observed
    pub peak_read_bandwidth: f64,
    /// Peak write bandwidth observed
    pub peak_write_bandwidth: f64,
    /// Read/Write ratio
    pub read_write_ratio: f64,
    /// Bandwidth samples over time
    pub samples: Vec<BandwidthSample>,
}

impl Default for MemoryBandwidthStats {
    fn default() -> Self {
        Self {
            avg_read_bandwidth: 0.0,
            avg_write_bandwidth: 0.0,
            peak_read_bandwidth: 0.0,
            peak_write_bandwidth: 0.0,
            read_write_ratio: 0.0,
            samples: Vec::new(),
        }
    }
}

/// Memory bandwidth sample at a point in time
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BandwidthSample {
    /// Timestamp in cycles
    pub timestamp: u64,
    /// Bytes read in this sample period
    pub bytes_read: u64,
    /// Bytes written in this sample period
    pub bytes_written: u64,
}

/// VBlank timing statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VBlankStats {
    /// Average VBlank period duration
    pub avg_vblank_duration: Duration,
    /// Average time spent in VBlank (hblank + visible)
    pub avg_active_time: Duration,
    /// VBlank usage percentage (0-100)
    pub vblank_usage_percent: f64,
    /// Number of VBlanks during profiling
    pub vblank_count: u64,
    /// Frame time statistics
    pub frame_time_stats: utils::Stats,
}

impl Default for VBlankStats {
    fn default() -> Self {
        Self {
            avg_vblank_duration: Duration::default(),
            avg_active_time: Duration::default(),
            vblank_usage_percent: 0.0,
            vblank_count: 0,
            frame_time_stats: utils::Stats {
                mean: 0.0,
                median: 0.0,
                std_dev: 0.0,
                min: 0.0,
                max: 0.0,
                sample_count: 0,
            },
        }
    }
}

/// Timing information for a function
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionTiming {
    /// Function name
    pub name: String,
    /// Total cycles spent in this function
    pub total_cycles: u64,
    /// Number of calls
    pub call_count: u64,
    /// Average cycles per call
    pub avg_cycles_per_call: f64,
    /// Minimum cycles for a single call
    pub min_cycles: u64,
    /// Maximum cycles for a single call
    pub max_cycles: u64,
    /// Percentage of total execution time
    pub time_percentage: f64,
}

/// Active function call tracking
#[derive(Debug)]
struct ActiveFunction {
    name: String,
    start_cycles: u64,
    start_time: Instant,
}

/// Runtime profiler for CPU and memory performance analysis
#[derive(Debug)]
pub struct Profiler {
    /// Whether currently recording
    recording: bool,
    /// When recording started
    recording_start: Option<Instant>,
    /// Total CPU cycles counter
    total_cycles: u64,
    /// Cycles per routine
    routine_cycles: HashMap<String, u64>,
    /// Active function call stack
    call_stack: Vec<ActiveFunction>,
    /// Completed function timings
    completed_functions: Vec<CompletedFunction>,
    /// Memory bandwidth samples
    bandwidth_samples: Vec<BandwidthSample>,
    /// Current bandwidth accumulator
    current_bandwidth: BandwidthAccumulator,
    /// VBlank timing data
    vblank_timings: Vec<VBlankTiming>,
    /// Current VBlank start
    current_vblank_start: Option<u64>,
    /// Frame timing data
    frame_times: Vec<f64>,
}

#[derive(Debug, Clone, Copy)]
struct BandwidthAccumulator {
    timestamp: u64,
    bytes_read: u64,
    bytes_written: u64,
}

#[derive(Debug)]
struct CompletedFunction {
    name: String,
    cycles: u64,
}

#[derive(Debug, Clone, Copy)]
struct VBlankTiming {
    start_cycle: u64,
    end_cycle: u64,
    active_cycles: u64,
}

impl Profiler {
    /// Create a new runtime profiler
    pub fn new() -> Self {
        Self {
            recording: false,
            recording_start: None,
            total_cycles: 0,
            routine_cycles: HashMap::new(),
            call_stack: Vec::new(),
            completed_functions: Vec::new(),
            bandwidth_samples: Vec::with_capacity(1024),
            current_bandwidth: BandwidthAccumulator {
                timestamp: 0,
                bytes_read: 0,
                bytes_written: 0,
            },
            vblank_timings: Vec::new(),
            current_vblank_start: None,
            frame_times: Vec::new(),
        }
    }

    /// Start measuring a function's execution time
    pub fn enter_function(&mut self, name: &str) {
        if !self.recording {
            return;
        }

        self.call_stack.push(ActiveFunction {
            name: name.to_string(),
            start_cycles: self.total_cycles,
            start_time: Instant::now(),
        });
    }

    /// End measuring a function's execution time
    pub fn exit_function(&mut self) {
        if !self.recording || self.call_stack.is_empty() {
            return;
        }

        let active = self.call_stack.pop().unwrap();
        let cycles = self.total_cycles.saturating_sub(active.start_cycles);

        self.completed_functions.push(CompletedFunction {
            name: active.name,
            cycles,
        });
    }

    /// Record CPU cycles for a routine
    pub fn record_cycles(&mut self, routine: &str, cycles: u64) {
        if !self.recording {
            return;
        }

        self.total_cycles += cycles;
        *self.routine_cycles.entry(routine.to_string()).or_insert(0) += cycles;
    }

    /// Record memory read
    pub fn record_memory_read(&mut self, bytes: u64) {
        if !self.recording {
            return;
        }

        self.current_bandwidth.bytes_read += bytes;
        self.maybe_flush_bandwidth_sample();
    }

    /// Record memory write
    pub fn record_memory_write(&mut self, bytes: u64) {
        if !self.recording {
            return;
        }

        self.current_bandwidth.bytes_written += bytes;
        self.maybe_flush_bandwidth_sample();
    }

    /// Record VBlank start
    pub fn record_vblank_start(&mut self) {
        if !self.recording {
            return;
        }

        self.current_vblank_start = Some(self.total_cycles);
    }

    /// Record VBlank end
    pub fn record_vblank_end(&mut self, active_cycles: u64) {
        if !self.recording {
            return;
        }

        if let Some(start) = self.current_vblank_start.take() {
            self.vblank_timings.push(VBlankTiming {
                start_cycle: start,
                end_cycle: self.total_cycles,
                active_cycles,
            });

            // Calculate frame time for 60 FPS target (16.67ms)
            let frame_cycles = self.total_cycles.saturating_sub(start);
            let frame_time_ms = (frame_cycles as f64) / 1789772.5 * 1000.0; // SNES master clock
            self.frame_times.push(frame_time_ms);
        }
    }

    /// Generate a performance report from collected data
    pub fn generate_report(&self) -> Result<PerformanceReport> {
        if self.recording {
            return Err(ProfilerError::AlreadyRecording);
        }

        if self.total_cycles == 0 {
            return Err(ProfilerError::InsufficientData(
                "No CPU cycles recorded".to_string(),
            ));
        }

        let duration = self.recording_start.map(|_| Duration::from_secs(1)).unwrap_or_default();

        // Aggregate function timings
        let function_timings = self.aggregate_function_timings();

        // Calculate memory bandwidth
        let memory_bandwidth = self.calculate_bandwidth_stats();

        // Calculate VBlank stats
        let vblank_stats = self.calculate_vblank_stats();

        // Find bottlenecks
        let bottlenecks = self.find_bottlenecks_internal(&function_timings, &memory_bandwidth, &vblank_stats);

        Ok(PerformanceReport {
            duration,
            total_cycles: self.total_cycles,
            routine_cycles: self.routine_cycles.clone(),
            memory_bandwidth,
            vblank_stats,
            function_timings,
            bottlenecks,
        })
    }

    /// Find performance bottlenecks and optimization opportunities
    pub fn find_bottlenecks(&self) -> Result<Vec<Bottleneck>> {
        if self.total_cycles == 0 {
            return Err(ProfilerError::InsufficientData(
                "No data collected".to_string(),
            ));
        }

        let function_timings = self.aggregate_function_timings();
        let memory_bandwidth = self.calculate_bandwidth_stats();
        let vblank_stats = self.calculate_vblank_stats();

        Ok(self.find_bottlenecks_internal(&function_timings, &memory_bandwidth, &vblank_stats))
    }

    // Private helper methods
    fn maybe_flush_bandwidth_sample(&mut self) {
        // Flush sample every 10000 cycles
        if self.total_cycles.saturating_sub(self.current_bandwidth.timestamp) >= 10000 {
            self.bandwidth_samples.push(BandwidthSample {
                timestamp: self.total_cycles,
                bytes_read: self.current_bandwidth.bytes_read,
                bytes_written: self.current_bandwidth.bytes_written,
            });

            self.current_bandwidth = BandwidthAccumulator {
                timestamp: self.total_cycles,
                bytes_read: 0,
                bytes_written: 0,
            };
        }
    }

    fn aggregate_function_timings(&self) -> Vec<FunctionTiming> {
        let mut timing_map: HashMap<String, Vec<u64>> = HashMap::new();

        for func in &self.completed_functions {
            timing_map
                .entry(func.name.clone())
                .or_default()
                .push(func.cycles);
        }

        timing_map
            .into_iter()
            .map(|(name, cycles)| {
                let total_cycles: u64 = cycles.iter().sum();
                let call_count = cycles.len() as u64;
                let avg_cycles = total_cycles as f64 / call_count as f64;
                let min_cycles = *cycles.iter().min().unwrap_or(&0);
                let max_cycles = *cycles.iter().max().unwrap_or(&0);
                let time_percentage = (total_cycles as f64 / self.total_cycles as f64) * 100.0;

                FunctionTiming {
                    name,
                    total_cycles,
                    call_count,
                    avg_cycles_per_call: avg_cycles,
                    min_cycles,
                    max_cycles,
                    time_percentage,
                }
            })
            .collect()
    }

    fn calculate_bandwidth_stats(&self) -> MemoryBandwidthStats {
        if self.bandwidth_samples.is_empty() {
            return MemoryBandwidthStats::default();
        }

        let total_read: u64 = self.bandwidth_samples.iter().map(|s| s.bytes_read).sum();
        let total_write: u64 = self.bandwidth_samples.iter().map(|s| s.bytes_written).sum();
        let sample_count = self.bandwidth_samples.len() as f64;

        let avg_read = total_read as f64 / sample_count;
        let avg_write = total_write as f64 / sample_count;

        let peak_read = self
            .bandwidth_samples
            .iter()
            .map(|s| s.bytes_read)
            .max()
            .unwrap_or(0) as f64;
        let peak_write = self
            .bandwidth_samples
            .iter()
            .map(|s| s.bytes_written)
            .max()
            .unwrap_or(0) as f64;

        let read_write_ratio = if total_write > 0 {
            total_read as f64 / total_write as f64
        } else {
            0.0
        };

        MemoryBandwidthStats {
            avg_read_bandwidth: avg_read,
            avg_write_bandwidth: avg_write,
            peak_read_bandwidth: peak_read,
            peak_write_bandwidth: peak_write,
            read_write_ratio,
            samples: self.bandwidth_samples.clone(),
        }
    }

    fn calculate_vblank_stats(&self) -> VBlankStats {
        if self.vblank_timings.is_empty() {
            return VBlankStats::default();
        }

        let vblank_count = self.vblank_timings.len() as u64;
        let total_active: u64 = self.vblank_timings.iter().map(|v| v.active_cycles).sum();
        let avg_active = total_active / vblank_count;

        // Convert cycles to duration (SNES master clock ~1.79 MHz)
        let avg_active_duration = Duration::from_secs_f64(avg_active as f64 / 1789772.5);
        let avg_vblank_duration = Duration::from_secs_f64(1.0 / 60.0); // ~16.67ms for 60 FPS

        // VBlank usage percentage
        let total_cycles: u64 = self.vblank_timings.iter()
            .map(|v| v.end_cycle.saturating_sub(v.start_cycle))
            .sum();
        let vblank_usage = (total_active as f64 / total_cycles as f64) * 100.0;

        let frame_time_stats = utils::calculate_stats(&self.frame_times).unwrap_or(utils::Stats {
            mean: 0.0,
            median: 0.0,
            std_dev: 0.0,
            min: 0.0,
            max: 0.0,
            sample_count: self.frame_times.len(),
        });

        VBlankStats {
            avg_vblank_duration,
            avg_active_time: avg_active_duration,
            vblank_usage_percent: vblank_usage,
            vblank_count,
            frame_time_stats,
        }
    }

    fn find_bottlenecks_internal(
        &self,
        function_timings: &[FunctionTiming],
        bandwidth: &MemoryBandwidthStats,
        vblank: &VBlankStats,
    ) -> Vec<Bottleneck> {
        let mut bottlenecks = Vec::new();

        // Check for slow functions
        for func in function_timings {
            if func.time_percentage > 30.0 {
                bottlenecks.push(Bottleneck {
                    description: format!("Function '{}' consumes {:.1}% of CPU time", func.name, func.time_percentage),
                    category: "CPU".to_string(),
                    severity: Severity::Critical,
                    impact: format!("{:.1}% of total cycles", func.time_percentage),
                    suggestion: format!("Optimize '{}' function or reduce call frequency", func.name),
                    location: Some(func.name.clone()),
                });
            } else if func.time_percentage > 15.0 {
                bottlenecks.push(Bottleneck {
                    description: format!("Function '{}' is a significant CPU consumer", func.name),
                    category: "CPU".to_string(),
                    severity: Severity::High,
                    impact: format!("{:.1}% of total cycles", func.time_percentage),
                    suggestion: format!("Review '{}' for optimization opportunities", func.name),
                    location: Some(func.name.clone()),
                });
            }
        }

        // Check memory bandwidth
        if bandwidth.peak_read_bandwidth > bandwidth.avg_read_bandwidth * 10.0 {
            bottlenecks.push(Bottleneck {
                description: "Large variance in memory read bandwidth detected".to_string(),
                category: "Memory".to_string(),
                severity: Severity::Medium,
                impact: "Potential cache thrashing".to_string(),
                suggestion: "Review memory access patterns for cache optimization".to_string(),
                location: None,
            });
        }

        // Check VBlank usage
        if vblank.vblank_usage_percent > 90.0 {
            bottlenecks.push(Bottleneck {
                description: "VBlank period nearly saturated".to_string(),
                category: "Timing".to_string(),
                severity: Severity::Critical,
                impact: format!("{:.1}% VBlank usage", vblank.vblank_usage_percent),
                suggestion: "Reduce processing in VBlank or optimize DMA transfers".to_string(),
                location: None,
            });
        } else if vblank.vblank_usage_percent > 70.0 {
            bottlenecks.push(Bottleneck {
                description: "High VBlank usage detected".to_string(),
                category: "Timing".to_string(),
                severity: Severity::Medium,
                impact: format!("{:.1}% VBlank usage", vblank.vblank_usage_percent),
                suggestion: "Monitor VBlank processing to prevent overflow".to_string(),
                location: None,
            });
        }

        // Check frame timing
        if let Some(stats) = utils::calculate_stats(&self.frame_times) {
            if stats.mean > 17.0 {
                bottlenecks.push(Bottleneck {
                    description: "Frame time exceeds 60 FPS budget".to_string(),
                    category: "Timing".to_string(),
                    severity: Severity::High,
                    impact: format!("{:.2}ms average frame time", stats.mean),
                    suggestion: "Optimize to maintain 60 FPS (16.67ms budget)".to_string(),
                    location: None,
                });
            }
        }

        bottlenecks
    }
}

impl Default for Profiler {
    fn default() -> Self {
        Self::new()
    }
}

impl ProfilerTrait for Profiler {
    fn start_recording(&mut self) -> Result<()> {
        if self.recording {
            return Err(ProfilerError::AlreadyRecording);
        }

        self.recording = true;
        self.recording_start = Some(Instant::now());
        self.clear();

        Ok(())
    }

    fn stop_recording(&mut self) -> Result<()> {
        if !self.recording {
            return Err(ProfilerError::NotRecording);
        }

        self.recording = false;
        // Flush any pending bandwidth data
        self.bandwidth_samples.push(BandwidthSample {
            timestamp: self.total_cycles,
            bytes_read: self.current_bandwidth.bytes_read,
            bytes_written: self.current_bandwidth.bytes_written,
        });

        Ok(())
    }

    fn is_recording(&self) -> bool {
        self.recording
    }

    fn clear(&mut self) {
        self.total_cycles = 0;
        self.routine_cycles.clear();
        self.call_stack.clear();
        self.completed_functions.clear();
        self.bandwidth_samples.clear();
        self.current_bandwidth = BandwidthAccumulator {
            timestamp: 0,
            bytes_read: 0,
            bytes_written: 0,
        };
        self.vblank_timings.clear();
        self.current_vblank_start = None;
        self.frame_times.clear();
    }

    fn recording_duration(&self) -> Option<Duration> {
        self.recording_start.map(|start| start.elapsed())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profiler_start_stop() {
        let mut profiler = Profiler::new();
        assert!(!profiler.is_recording());

        profiler.start_recording().unwrap();
        assert!(profiler.is_recording());

        profiler.stop_recording().unwrap();
        assert!(!profiler.is_recording());
    }

    #[test]
    fn test_function_timing() {
        let mut profiler = Profiler::new();
        profiler.start_recording().unwrap();

        profiler.enter_function("test_func");
        profiler.record_cycles("test_func", 100);
        profiler.exit_function();

        profiler.stop_recording().unwrap();

        let report = profiler.generate_report().unwrap();
        assert_eq!(report.total_cycles, 100);
        assert_eq!(report.function_timings.len(), 1);
        assert_eq!(report.function_timings[0].name, "test_func");
    }

    #[test]
    fn test_memory_bandwidth() {
        let mut profiler = Profiler::new();
        profiler.start_recording().unwrap();

        profiler.record_memory_read(1024);
        profiler.record_memory_write(512);
        profiler.record_cycles("test", 15000); // Trigger sample flush

        profiler.stop_recording().unwrap();

        let report = profiler.generate_report().unwrap();
        assert!(!report.memory_bandwidth.samples.is_empty());
    }

    #[test]
    fn test_bottleneck_detection() {
        let mut profiler = Profiler::new();
        profiler.start_recording().unwrap();

        // Simulate a heavy function
        for _ in 0..100 {
            profiler.enter_function("heavy_func");
            profiler.record_cycles("heavy_func", 1000);
            profiler.exit_function();
        }

        profiler.stop_recording().unwrap();

        let bottlenecks = profiler.find_bottlenecks().unwrap();
        assert!(!bottlenecks.is_empty());
    }
}
