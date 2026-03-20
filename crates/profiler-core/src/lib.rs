//! # profiler-core
//!
//! Performance profiling crate for SNES emulation.
//! Provides CPU, graphics, and audio performance analysis tools.
//!
//! ## Features
//!
//! - **Runtime Profiling**: CPU cycle counting, memory bandwidth, VBlank tracking
//! - **Graphics Profiling**: HDMA analysis, VRAM patterns, sprite tracking, Mode 7 profiling
//! - **Audio Profiling**: SPC700 load, BRR cache analysis, channel usage tracking
//!
//! ## Example
//!
//! ```rust,no_run
//! use profiler_core::runtime::Profiler;
//! use profiler_core::graphics::GraphicsProfiler;
//! use profiler_core::audio::AudioProfiler;
//!
//! let mut profiler = Profiler::new();
//! profiler.start_recording();
//! // ... run emulation ...
//! let report = profiler.generate_report();
//! ```

pub mod audio;
pub mod graphics;
pub mod runtime;

use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;

/// Common error type for profiler operations
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum ProfilerError {
    #[error("Profiler is not currently recording")]
    NotRecording,
    #[error("Profiler is already recording")]
    AlreadyRecording,
    #[error("Invalid sample rate: {0}")]
    InvalidSampleRate(u32),
    #[error("Insufficient data for analysis: {0}")]
    InsufficientData(String),
    #[error("Hardware access error: {0}")]
    HardwareAccessError(String),
}

/// Result type for profiler operations
pub type Result<T> = std::result::Result<T, ProfilerError>;

/// Severity level for performance issues
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    /// Informational, not a problem
    Info,
    /// Minor optimization opportunity
    Low,
    /// Moderate performance impact
    Medium,
    /// Significant performance bottleneck
    High,
    /// Critical performance issue
    Critical,
}

/// A performance bottleneck or optimization opportunity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bottleneck {
    /// Human-readable description of the issue
    pub description: String,
    /// Category of the bottleneck
    pub category: String,
    /// Severity level
    pub severity: Severity,
    /// Estimated impact (percentage or cycles)
    pub impact: String,
    /// Suggested optimization
    pub suggestion: String,
    /// Location in code (if known)
    pub location: Option<String>,
}

/// Base trait for all profilers
pub trait ProfilerTrait {
    /// Start recording performance data
    fn start_recording(&mut self) -> Result<()>;
    
    /// Stop recording performance data
    fn stop_recording(&mut self) -> Result<()>;
    
    /// Check if currently recording
    fn is_recording(&self) -> bool;
    
    /// Clear all collected data
    fn clear(&mut self);
    
    /// Get the duration of the current/profiling session
    fn recording_duration(&self) -> Option<Duration>;
}

/// Common utility functions for profilers
pub mod utils {
    use super::*;
    use std::collections::HashMap;

    /// Calculate statistical metrics for a series of samples
    pub fn calculate_stats(samples: &[f64]) -> Option<Stats> {
        if samples.is_empty() {
            return None;
        }

        let sum: f64 = samples.iter().sum();
        let count = samples.len() as f64;
        let mean = sum / count;

        let variance = samples.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / count;
        let std_dev = variance.sqrt();

        let mut sorted = samples.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let min = sorted[0];
        let max = sorted[sorted.len() - 1];
        let median = if sorted.len() % 2 == 0 {
            (sorted[sorted.len() / 2 - 1] + sorted[sorted.len() / 2]) / 2.0
        } else {
            sorted[sorted.len() / 2]
        };

        Some(Stats {
            mean,
            median,
            std_dev,
            min,
            max,
            sample_count: samples.len(),
        })
    }

    /// Find the most frequent values in a collection
    pub fn find_frequent<T: std::hash::Hash + Eq + Clone>(
        items: &[T],
        top_n: usize,
    ) -> Vec<(T, usize)> {
        let mut counts: HashMap<T, usize> = HashMap::new();
        for item in items {
            *counts.entry(item.clone()).or_insert(0) += 1;
        }

        let mut sorted: Vec<_> = counts.into_iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));
        sorted.truncate(top_n);
        sorted
    }

    /// Statistical metrics for a data series
    #[derive(Debug, Clone, Copy, Serialize, Deserialize)]
    pub struct Stats {
        pub mean: f64,
        pub median: f64,
        pub std_dev: f64,
        pub min: f64,
        pub max: f64,
        pub sample_count: usize,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_stats() {
        let samples = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let stats = utils::calculate_stats(&samples).unwrap();
        
        assert_eq!(stats.mean, 3.0);
        assert_eq!(stats.median, 3.0);
        assert_eq!(stats.min, 1.0);
        assert_eq!(stats.max, 5.0);
        assert_eq!(stats.sample_count, 5);
    }

    #[test]
    fn test_find_frequent() {
        let items = vec!["a", "b", "a", "c", "a", "b"];
        let frequent = utils::find_frequent(&items, 2);
        
        assert_eq!(frequent.len(), 2);
        assert_eq!(frequent[0], ("a", 3));
        assert_eq!(frequent[1], ("b", 2));
    }
}
