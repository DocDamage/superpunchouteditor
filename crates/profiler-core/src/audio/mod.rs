//! # Audio Profiler
//!
//! Audio performance profiler for SNES S-SMP (SPC700) emulation.
//! Tracks SPC700 CPU load, BRR sample cache performance, and channel usage.

use crate::{utils, ProfilerError, ProfilerTrait, Result, Severity, Bottleneck};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Audio performance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioReport {
    /// Total duration of the profiling session
    pub duration: Duration,
    /// SPC700 CPU load statistics
    pub spc700_stats: Spc700Stats,
    /// BRR sample cache statistics
    pub brr_cache_stats: BrrCacheStats,
    /// Audio channel usage statistics
    pub channel_stats: ChannelStats,
    /// Identified audio bottlenecks
    pub bottlenecks: Vec<Bottleneck>,
}

/// SPC700 CPU statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Spc700Stats {
    /// Average CPU load percentage (0-100)
    pub avg_load_percent: f64,
    /// Peak CPU load percentage
    pub peak_load_percent: f64,
    /// Minimum CPU load percentage
    pub min_load_percent: f64,
    /// CPU load distribution
    pub load_distribution: LoadDistribution,
    /// Instructions executed per sample
    pub instructions_per_sample: f64,
    /// DSP register access frequency
    pub dsp_register_access_rate: f64,
}

impl Default for Spc700Stats {
    fn default() -> Self {
        Self {
            avg_load_percent: 0.0,
            peak_load_percent: 0.0,
            min_load_percent: 100.0,
            load_distribution: LoadDistribution::default(),
            instructions_per_sample: 0.0,
            dsp_register_access_rate: 0.0,
        }
    }
}

/// CPU load distribution across ranges
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadDistribution {
    /// Percentage of time in 0-25% load range
    pub low_percent: f64,
    /// Percentage of time in 25-50% load range
    pub medium_percent: f64,
    /// Percentage of time in 50-75% load range
    pub high_percent: f64,
    /// Percentage of time in 75-100% load range
    pub critical_percent: f64,
}

impl Default for LoadDistribution {
    fn default() -> Self {
        Self {
            low_percent: 0.0,
            medium_percent: 0.0,
            high_percent: 0.0,
            critical_percent: 0.0,
        }
    }
}

/// BRR (Bit Rate Reduction) sample cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrrCacheStats {
    /// Total cache lookups
    pub total_lookups: u64,
    /// Cache hits
    pub hits: u64,
    /// Cache misses
    pub misses: u64,
    /// Hit rate percentage (0-100)
    pub hit_rate_percent: f64,
    /// Cache evictions
    pub evictions: u64,
    /// Average sample decode time (cycles)
    pub avg_decode_cycles: f64,
    /// Most frequently accessed samples
    pub hot_samples: Vec<SampleAccess>,
    /// Cache size utilization
    pub cache_utilization_percent: f64,
}

impl Default for BrrCacheStats {
    fn default() -> Self {
        Self {
            total_lookups: 0,
            hits: 0,
            misses: 0,
            hit_rate_percent: 0.0,
            evictions: 0,
            avg_decode_cycles: 0.0,
            hot_samples: Vec::new(),
            cache_utilization_percent: 0.0,
        }
    }
}

/// Sample access information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SampleAccess {
    /// Sample address in ARAM
    pub address: u16,
    /// Number of times accessed
    pub access_count: u64,
    /// Sample size in bytes
    pub size: u16,
    /// Associated channel(s)
    pub channels: Vec<u8>,
}

/// Audio channel usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelStats {
    /// Per-channel statistics
    pub channels: HashMap<u8, ChannelInfo>,
    /// Number of channels active per sample (average)
    pub avg_active_channels: f64,
    /// Peak simultaneous channels
    pub peak_channels: u8,
    /// Channel usage distribution over time
    pub usage_over_time: Vec<ChannelUsageSnapshot>,
    /// Most common channel combinations
    pub common_combinations: Vec<ChannelCombination>,
}

impl Default for ChannelStats {
    fn default() -> Self {
        Self {
            channels: HashMap::new(),
            avg_active_channels: 0.0,
            peak_channels: 0,
            usage_over_time: Vec::new(),
            common_combinations: Vec::new(),
        }
    }
}

/// Information about a single audio channel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelInfo {
    /// Channel number (0-7)
    pub channel: u8,
    /// Percentage of time channel is active
    pub active_percent: f64,
    /// Number of key-on events
    pub key_on_count: u64,
    /// Number of key-off events
    pub key_off_count: u64,
    /// Volume level statistics
    pub volume_stats: VolumeStats,
    /// Pitch change frequency
    pub pitch_change_count: u64,
    /// Sample sources used
    pub sample_sources: Vec<u16>,
}

/// Volume statistics for a channel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeStats {
    /// Average left volume (0-127)
    pub avg_left: f64,
    /// Average right volume (0-127)
    pub avg_right: f64,
    /// Peak left volume
    pub peak_left: u8,
    /// Peak right volume
    pub peak_right: u8,
    /// Volume envelope state distribution
    pub envelope_states: HashMap<String, u64>,
}

impl Default for VolumeStats {
    fn default() -> Self {
        Self {
            avg_left: 0.0,
            avg_right: 0.0,
            peak_left: 0,
            peak_right: 0,
            envelope_states: HashMap::new(),
        }
    }
}

/// Snapshot of channel usage at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelUsageSnapshot {
    /// Timestamp (in samples)
    pub timestamp: u64,
    /// Active channels at this time
    pub active_channels: Vec<u8>,
    /// Combined volume level
    pub combined_volume: u8,
}

/// Common channel combination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelCombination {
    /// Bitmask of channels (bit 0 = channel 0, etc.)
    pub channel_mask: u8,
    /// Number of times this combination occurred
    pub occurrence_count: u64,
    /// Percentage of total samples
    pub percent: f64,
}

/// SPC700 load sample
#[derive(Debug, Clone, Copy)]
struct LoadSample {
    timestamp: u64,
    load_percent: f64,
    cycles: u64,
    instructions: u64,
}

/// BRR cache event
#[derive(Debug, Clone, Copy)]
enum BrrEvent {
    Hit { address: u16 },
    Miss { address: u16, decode_cycles: u64 },
    Eviction { address: u16 },
}

/// Channel event
#[derive(Debug, Clone)]
struct ChannelEvent {
    timestamp: u64,
    channel: u8,
    event_type: ChannelEventType,
}

#[derive(Debug, Clone)]
enum ChannelEventType {
    KeyOn { sample_addr: u16 },
    KeyOff,
    VolumeChange { left: u8, right: u8 },
    PitchChange,
}

/// Audio profiler for SPC700 and DSP performance analysis
#[derive(Debug)]
pub struct AudioProfiler {
    /// Whether currently recording
    recording: bool,
    /// When recording started
    recording_start: Option<Instant>,
    /// SPC700 load samples
    load_samples: Vec<LoadSample>,
    /// BRR cache events
    brr_events: Vec<BrrEvent>,
    /// Channel events
    channel_events: Vec<ChannelEvent>,
    /// Current active channels (bitmask)
    current_channel_mask: u8,
    /// Sample counter
    sample_counter: u64,
    /// DSP register access count
    dsp_access_count: u64,
}

impl AudioProfiler {
    /// Create a new audio profiler
    pub fn new() -> Self {
        Self {
            recording: false,
            recording_start: None,
            load_samples: Vec::with_capacity(1024),
            brr_events: Vec::with_capacity(1024),
            channel_events: Vec::with_capacity(1024),
            current_channel_mask: 0,
            sample_counter: 0,
            dsp_access_count: 0,
        }
    }

    /// Record SPC700 CPU load for a sample period
    pub fn record_spc700_load(&mut self, cycles: u64, instructions: u64) {
        if !self.recording {
            return;
        }

        // SPC700 runs at ~1.024 MHz, one sample at 32kHz = ~32 cycles
        const CYCLES_PER_SAMPLE: u64 = 32;
        let load_percent = (cycles as f64 / CYCLES_PER_SAMPLE as f64) * 100.0;

        self.load_samples.push(LoadSample {
            timestamp: self.sample_counter,
            load_percent: load_percent.min(100.0),
            cycles,
            instructions,
        });

        self.sample_counter += 1;
    }

    /// Record BRR cache hit
    pub fn record_brr_hit(&mut self, address: u16) {
        if !self.recording {
            return;
        }

        self.brr_events.push(BrrEvent::Hit { address });
    }

    /// Record BRR cache miss
    pub fn record_brr_miss(&mut self, address: u16, decode_cycles: u64) {
        if !self.recording {
            return;
        }

        self.brr_events.push(BrrEvent::Miss {
            address,
            decode_cycles,
        });
    }

    /// Record BRR cache eviction
    pub fn record_brr_eviction(&mut self, address: u16) {
        if !self.recording {
            return;
        }

        self.brr_events.push(BrrEvent::Eviction { address });
    }

    /// Record channel key-on
    pub fn record_channel_keyon(&mut self, channel: u8, sample_addr: u16) {
        if !self.recording || channel > 7 {
            return;
        }

        self.current_channel_mask |= 1 << channel;
        self.channel_events.push(ChannelEvent {
            timestamp: self.sample_counter,
            channel,
            event_type: ChannelEventType::KeyOn { sample_addr },
        });
    }

    /// Record channel key-off
    pub fn record_channel_keyoff(&mut self, channel: u8) {
        if !self.recording || channel > 7 {
            return;
        }

        self.current_channel_mask &= !(1 << channel);
        self.channel_events.push(ChannelEvent {
            timestamp: self.sample_counter,
            channel,
            event_type: ChannelEventType::KeyOff,
        });
    }

    /// Record channel volume change
    pub fn record_channel_volume(&mut self, channel: u8, left: u8, right: u8) {
        if !self.recording || channel > 7 {
            return;
        }

        self.channel_events.push(ChannelEvent {
            timestamp: self.sample_counter,
            channel,
            event_type: ChannelEventType::VolumeChange { left, right },
        });
    }

    /// Record channel pitch change
    pub fn record_channel_pitch_change(&mut self, channel: u8) {
        if !self.recording || channel > 7 {
            return;
        }

        self.channel_events.push(ChannelEvent {
            timestamp: self.sample_counter,
            channel,
            event_type: ChannelEventType::PitchChange,
        });
    }

    /// Record DSP register access
    pub fn record_dsp_access(&mut self) {
        if !self.recording {
            return;
        }

        self.dsp_access_count += 1;
    }

    /// Generate audio performance report
    pub fn generate_report(&self) -> Result<AudioReport> {
        if self.recording {
            return Err(ProfilerError::AlreadyRecording);
        }

        let duration = self.recording_start.map(|_| Duration::from_secs(1)).unwrap_or_default();

        let spc700_stats = self.calculate_spc700_stats();
        let brr_cache_stats = self.calculate_brr_stats();
        let channel_stats = self.calculate_channel_stats();

        let bottlenecks = self.find_bottlenecks_internal(&spc700_stats, &brr_cache_stats, &channel_stats);

        Ok(AudioReport {
            duration,
            spc700_stats,
            brr_cache_stats,
            channel_stats,
            bottlenecks,
        })
    }

    /// Find audio performance bottlenecks
    pub fn find_bottlenecks(&self) -> Result<Vec<Bottleneck>> {
        if self.load_samples.is_empty() {
            return Err(ProfilerError::InsufficientData(
                "No audio data collected".to_string(),
            ));
        }

        let spc700_stats = self.calculate_spc700_stats();
        let brr_cache_stats = self.calculate_brr_stats();
        let channel_stats = self.calculate_channel_stats();

        Ok(self.find_bottlenecks_internal(&spc700_stats, &brr_cache_stats, &channel_stats))
    }

    // Private helper methods
    fn calculate_spc700_stats(&self) -> Spc700Stats {
        if self.load_samples.is_empty() {
            return Spc700Stats::default();
        }

        let loads: Vec<f64> = self.load_samples.iter().map(|s| s.load_percent).collect();
        
        let avg_load = loads.iter().sum::<f64>() / loads.len() as f64;
        let peak_load = loads.iter().cloned().fold(0.0, f64::max);
        let min_load = loads.iter().cloned().fold(100.0, f64::min);

        // Calculate distribution
        let total = loads.len() as f64;
        let low = loads.iter().filter(|&&l| l < 25.0).count() as f64 / total * 100.0;
        let medium = loads.iter().filter(|&&l| l >= 25.0 && l < 50.0).count() as f64 / total * 100.0;
        let high = loads.iter().filter(|&&l| l >= 50.0 && l < 75.0).count() as f64 / total * 100.0;
        let critical = loads.iter().filter(|&&l| l >= 75.0).count() as f64 / total * 100.0;

        let total_cycles: u64 = self.load_samples.iter().map(|s| s.cycles).sum();
        let total_instructions: u64 = self.load_samples.iter().map(|s| s.instructions).sum();

        Spc700Stats {
            avg_load_percent: avg_load,
            peak_load_percent: peak_load,
            min_load_percent: min_load,
            load_distribution: LoadDistribution {
                low_percent: low,
                medium_percent: medium,
                high_percent: high,
                critical_percent: critical,
            },
            instructions_per_sample: total_instructions as f64 / self.load_samples.len() as f64,
            dsp_register_access_rate: self.dsp_access_count as f64 / self.load_samples.len() as f64,
        }
    }

    fn calculate_brr_stats(&self) -> BrrCacheStats {
        let hits = self.brr_events.iter().filter(|e| matches!(e, BrrEvent::Hit { .. })).count() as u64;
        let misses = self.brr_events.iter().filter(|e| matches!(e, BrrEvent::Miss { .. })).count() as u64;
        let evictions = self.brr_events.iter().filter(|e| matches!(e, BrrEvent::Eviction { .. })).count() as u64;

        let total = hits + misses;
        let hit_rate = if total > 0 { (hits as f64 / total as f64) * 100.0 } else { 0.0 };

        // Calculate average decode cycles from misses
        let total_decode_cycles: u64 = self.brr_events
            .iter()
            .filter_map(|e| match e {
                BrrEvent::Miss { decode_cycles, .. } => Some(*decode_cycles),
                _ => None,
            })
            .sum();

        let avg_decode_cycles = if misses > 0 {
            total_decode_cycles as f64 / misses as f64
        } else {
            0.0
        };

        // Find hot samples
        let mut sample_counts: HashMap<u16, (u64, Vec<u8>)> = HashMap::new();
        for event in &self.brr_events {
            match event {
                BrrEvent::Hit { address } | BrrEvent::Miss { address, .. } => {
                    let entry = sample_counts.entry(*address).or_insert((0, Vec::new()));
                    entry.0 += 1;
                }
                _ => {}
            }
        }

        let mut hot_samples: Vec<_> = sample_counts
            .into_iter()
            .map(|(address, (count, channels))| SampleAccess {
                address,
                access_count: count,
                size: 9, // BRR block size
                channels,
            })
            .collect();
        
        hot_samples.sort_by(|a, b| b.access_count.cmp(&a.access_count));
        hot_samples.truncate(10);

        BrrCacheStats {
            total_lookups: total,
            hits,
            misses,
            hit_rate_percent: hit_rate,
            evictions,
            avg_decode_cycles,
            hot_samples,
            cache_utilization_percent: 0.0, // Would need cache size info
        }
    }

    fn calculate_channel_stats(&self) -> ChannelStats {
        if self.channel_events.is_empty() {
            return ChannelStats::default();
        }

        // Build per-channel info
        let mut channel_info: HashMap<u8, ChannelAccumulator> = HashMap::new();
        let mut usage_snapshots: Vec<ChannelUsageSnapshot> = Vec::new();
        let mut combination_counts: HashMap<u8, u64> = HashMap::new();
        let mut current_mask = 0u8;
        let mut last_sample = 0u64;

        for event in &self.channel_events {
            // Track combinations per sample
            if event.timestamp != last_sample {
                if last_sample > 0 {
                    *combination_counts.entry(current_mask).or_insert(0) += event.timestamp - last_sample;
                }
                
                let active_channels: Vec<u8> = (0..8)
                    .filter(|&c| current_mask & (1 << c) != 0)
                    .collect();
                
                if active_channels.len() > 0 {
                    usage_snapshots.push(ChannelUsageSnapshot {
                        timestamp: last_sample,
                        active_channels: active_channels.clone(),
                        combined_volume: 0, // Would need to track
                    });
                }
                
                last_sample = event.timestamp;
            }

            let info = channel_info.entry(event.channel).or_default();

            match &event.event_type {
                ChannelEventType::KeyOn { sample_addr } => {
                    info.key_on_count += 1;
                    info.sample_sources.push(*sample_addr);
                    current_mask |= 1 << event.channel;
                }
                ChannelEventType::KeyOff => {
                    info.key_off_count += 1;
                    current_mask &= !(1 << event.channel);
                }
                ChannelEventType::VolumeChange { left, right } => {
                    info.volume_samples.push((*left, *right));
                }
                ChannelEventType::PitchChange => {
                    info.pitch_changes += 1;
                }
            }
        }

        // Calculate final combination count
        if last_sample < self.sample_counter {
            *combination_counts.entry(current_mask).or_insert(0) += self.sample_counter - last_sample;
        }

        // Convert to ChannelInfo
        let total_samples = self.sample_counter as f64;
        let channels: HashMap<_, _> = channel_info
            .into_iter()
            .map(|(ch, acc)| {
                let active_samples = combination_counts
                    .iter()
                    .filter(|(&mask, _)| mask & (1 << ch) != 0)
                    .map(|(_, count)| count)
                    .sum::<u64>();

                let avg_left = if !acc.volume_samples.is_empty() {
                    acc.volume_samples.iter().map(|(l, _)| *l as f64).sum::<f64>() / acc.volume_samples.len() as f64
                } else {
                    0.0
                };

                let avg_right = if !acc.volume_samples.is_empty() {
                    acc.volume_samples.iter().map(|(_, r)| *r as f64).sum::<f64>() / acc.volume_samples.len() as f64
                } else {
                    0.0
                };

                let (peak_left, peak_right) = acc.volume_samples.iter().fold((0u8, 0u8), |(pl, pr), (l, r)| {
                    (pl.max(*l), pr.max(*r))
                });

                let info = ChannelInfo {
                    channel: ch,
                    active_percent: (active_samples as f64 / total_samples) * 100.0,
                    key_on_count: acc.key_on_count,
                    key_off_count: acc.key_off_count,
                    volume_stats: VolumeStats {
                        avg_left,
                        avg_right,
                        peak_left,
                        peak_right,
                        envelope_states: HashMap::new(),
                    },
                    pitch_change_count: acc.pitch_changes,
                    sample_sources: acc.sample_sources.clone(),
                };

                (ch, info)
            })
            .collect();

        // Calculate average active channels
        let avg_active: f64 = combination_counts
            .iter()
            .map(|(mask, count)| {
                let active = mask.count_ones() as f64;
                active * (*count as f64 / total_samples)
            })
            .sum();

        // Find peak channels
        let peak_channels = combination_counts
            .keys()
            .map(|mask| mask.count_ones() as u8)
            .max()
            .unwrap_or(0);

        // Common combinations
        let mut common_combinations: Vec<_> = combination_counts
            .into_iter()
            .map(|(mask, count)| ChannelCombination {
                channel_mask: mask,
                occurrence_count: count,
                percent: (count as f64 / total_samples) * 100.0,
            })
            .collect();
        common_combinations.sort_by(|a, b| b.occurrence_count.cmp(&a.occurrence_count));
        common_combinations.truncate(10);

        ChannelStats {
            channels,
            avg_active_channels: avg_active,
            peak_channels,
            usage_over_time: usage_snapshots,
            common_combinations,
        }
    }

    fn find_bottlenecks_internal(
        &self,
        spc700: &Spc700Stats,
        brr: &BrrCacheStats,
        channels: &ChannelStats,
    ) -> Vec<Bottleneck> {
        let mut bottlenecks = Vec::new();

        // SPC700 load bottlenecks
        if spc700.peak_load_percent > 95.0 {
            bottlenecks.push(Bottleneck {
                description: "SPC700 CPU overloaded - risk of audio dropouts".to_string(),
                category: "SPC700".to_string(),
                severity: Severity::Critical,
                impact: format!("Peak load {:.1}%", spc700.peak_load_percent),
                suggestion: "Reduce number of active channels or simplify BRR samples".to_string(),
                location: None,
            });
        } else if spc700.avg_load_percent > 80.0 {
            bottlenecks.push(Bottleneck {
                description: "High average SPC700 CPU load".to_string(),
                category: "SPC700".to_string(),
                severity: Severity::Medium,
                impact: format!("Average load {:.1}%", spc700.avg_load_percent),
                suggestion: "Monitor for potential overload scenarios".to_string(),
                location: None,
            });
        }

        // BRR cache bottlenecks
        if brr.hit_rate_percent < 70.0 && brr.total_lookups > 100 {
            bottlenecks.push(Bottleneck {
                description: "Low BRR cache hit rate".to_string(),
                category: "BRR".to_string(),
                severity: Severity::High,
                impact: format!("{:.1}% hit rate", brr.hit_rate_percent),
                suggestion: "Increase cache size or reduce unique samples".to_string(),
                location: None,
            });
        }

        // Channel usage bottlenecks
        if channels.avg_active_channels > 6.0 {
            bottlenecks.push(Bottleneck {
                description: "High channel utilization".to_string(),
                category: "Channels".to_string(),
                severity: Severity::Medium,
                impact: format!("{:.1} channels average", channels.avg_active_channels),
                suggestion: "Consider voice stealing or sample merging".to_string(),
                location: None,
            });
        }

        bottlenecks
    }
}

#[derive(Debug, Default)]
struct ChannelAccumulator {
    key_on_count: u64,
    key_off_count: u64,
    volume_samples: Vec<(u8, u8)>,
    pitch_changes: u64,
    sample_sources: Vec<u16>,
}

impl Default for AudioProfiler {
    fn default() -> Self {
        Self::new()
    }
}

impl ProfilerTrait for AudioProfiler {
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
        Ok(())
    }

    fn is_recording(&self) -> bool {
        self.recording
    }

    fn clear(&mut self) {
        self.load_samples.clear();
        self.brr_events.clear();
        self.channel_events.clear();
        self.current_channel_mask = 0;
        self.sample_counter = 0;
        self.dsp_access_count = 0;
    }

    fn recording_duration(&self) -> Option<Duration> {
        self.recording_start.map(|start| start.elapsed())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_profiler_start_stop() {
        let mut profiler = AudioProfiler::new();
        assert!(!profiler.is_recording());

        profiler.start_recording().unwrap();
        assert!(profiler.is_recording());

        profiler.stop_recording().unwrap();
        assert!(!profiler.is_recording());
    }

    #[test]
    fn test_spc700_load_recording() {
        let mut profiler = AudioProfiler::new();
        profiler.start_recording().unwrap();

        // Simulate load samples
        for i in 0..100 {
            let cycles = 20 + (i % 20);
            profiler.record_spc700_load(cycles, cycles * 2);
        }

        profiler.stop_recording().unwrap();

        let report = profiler.generate_report().unwrap();
        assert!(report.spc700_stats.avg_load_percent > 0.0);
        assert!(report.spc700_stats.instructions_per_sample > 0.0);
    }

    #[test]
    fn test_brr_cache_recording() {
        let mut profiler = AudioProfiler::new();
        profiler.start_recording().unwrap();

        // Simulate cache hits and misses
        for i in 0..100 {
            if i % 4 == 0 {
                profiler.record_brr_miss(0x1000 + i as u16, 50);
            } else {
                profiler.record_brr_hit(0x1000 + (i % 20) as u16);
            }
        }

        profiler.stop_recording().unwrap();

        let report = profiler.generate_report().unwrap();
        assert!(report.brr_cache_stats.hit_rate_percent > 0.0);
        assert!(report.brr_cache_stats.hit_rate_percent < 100.0);
    }

    #[test]
    fn test_channel_recording() {
        let mut profiler = AudioProfiler::new();
        profiler.start_recording().unwrap();

        // Simulate channel activity
        profiler.record_channel_keyon(0, 0x2000);
        profiler.record_channel_keyon(1, 0x2100);
        profiler.record_channel_volume(0, 100, 100);
        profiler.record_channel_pitch_change(0);
        profiler.record_channel_keyoff(0);

        // Advance sample counter
        for _ in 0..10 {
            profiler.record_spc700_load(30, 60);
        }

        profiler.record_channel_keyoff(1);

        profiler.stop_recording().unwrap();

        let report = profiler.generate_report().unwrap();
        assert!(!report.channel_stats.channels.is_empty());
    }

    #[test]
    fn test_bottleneck_detection() {
        let mut profiler = AudioProfiler::new();
        profiler.start_recording().unwrap();

        // Simulate overload condition
        for _ in 0..100 {
            profiler.record_spc700_load(50, 100); // Very high load
        }

        profiler.stop_recording().unwrap();

        let bottlenecks = profiler.find_bottlenecks().unwrap();
        assert!(!bottlenecks.is_empty());
    }
}
