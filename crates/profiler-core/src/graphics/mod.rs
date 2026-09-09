//! # Graphics Profiler
//!
//! Graphics performance profiler for SNES emulation.
//! Tracks HDMA usage, VRAM access patterns, sprite/OAM usage, and Mode 7 operations.

use crate::{utils, ProfilerError, ProfilerTrait, Result, Severity, Bottleneck};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Graphics performance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphicsReport {
    /// Total duration of the profiling session
    pub duration: Duration,
    /// HDMA channel usage statistics
    pub hdma_stats: HdmaStats,
    /// VRAM access statistics
    pub vram_stats: VramStats,
    /// Sprite/OAM usage statistics
    pub sprite_stats: SpriteStats,
    /// Mode 7 performance statistics
    pub mode7_stats: Mode7Stats,
    /// Identified graphics bottlenecks
    pub bottlenecks: Vec<Bottleneck>,
}

/// HDMA (Horizontal Direct Memory Access) statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HdmaStats {
    /// Number of HDMA transfers performed
    pub transfer_count: u64,
    /// Total bytes transferred via HDMA
    pub total_bytes_transferred: u64,
    /// Channel usage counts
    pub channel_usage: HashMap<u8, ChannelUsage>,
    /// Average cycles per HDMA transfer
    pub avg_transfer_cycles: f64,
    /// HDMA overhead percentage
    pub hdma_overhead_percent: f64,
}

impl Default for HdmaStats {
    fn default() -> Self {
        Self {
            transfer_count: 0,
            total_bytes_transferred: 0,
            channel_usage: HashMap::new(),
            avg_transfer_cycles: 0.0,
            hdma_overhead_percent: 0.0,
        }
    }
}

/// HDMA channel usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelUsage {
    /// Channel number (0-7)
    pub channel: u8,
    /// Number of transfers on this channel
    pub transfer_count: u64,
    /// Total bytes transferred on this channel
    pub bytes_transferred: u64,
    /// Transfer modes used on this channel
    pub transfer_modes: Vec<String>,
    /// Destination registers accessed
    pub destination_registers: Vec<String>,
    /// Table addresses used
    pub table_addresses: Vec<u16>,
}

/// VRAM access statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VramStats {
    /// Total VRAM reads
    pub total_reads: u64,
    /// Total VRAM writes
    pub total_writes: u64,
    /// Access pattern analysis
    pub access_patterns: AccessPatterns,
    /// Bandwidth utilization percentage
    pub bandwidth_utilization: f64,
    /// Most frequently accessed VRAM regions
    pub hot_regions: Vec<VramRegion>,
}

impl Default for VramStats {
    fn default() -> Self {
        Self {
            total_reads: 0,
            total_writes: 0,
            access_patterns: AccessPatterns::default(),
            bandwidth_utilization: 0.0,
            hot_regions: Vec::new(),
        }
    }
}

/// Memory access patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessPatterns {
    /// Sequential access percentage
    pub sequential_percent: f64,
    /// Random access percentage
    pub random_percent: f64,
    /// Strided access percentage
    pub strided_percent: f64,
    /// Average stride size (in bytes)
    pub avg_stride: f64,
}

impl Default for AccessPatterns {
    fn default() -> Self {
        Self {
            sequential_percent: 0.0,
            random_percent: 0.0,
            strided_percent: 0.0,
            avg_stride: 0.0,
        }
    }
}

/// VRAM region information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VramRegion {
    /// Start address
    pub start: u16,
    /// End address
    pub end: u16,
    /// Access count
    pub access_count: u64,
    /// Access type (read/write/both)
    pub access_type: String,
}

/// Sprite/OAM (Object Attribute Memory) statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpriteStats {
    /// Average sprites per frame
    pub avg_sprites_per_frame: f64,
    /// Maximum sprites in a single frame
    pub max_sprites: u32,
    /// Minimum sprites in a single frame
    pub min_sprites: u32,
    /// Sprite size distribution
    pub size_distribution: HashMap<String, u64>,
    /// Priority level usage
    pub priority_usage: HashMap<u8, u64>,
    /// OAM bytes accessed per frame
    pub oam_bytes_per_frame: f64,
    /// Time spent processing sprites (percentage)
    pub processing_time_percent: f64,
}

impl Default for SpriteStats {
    fn default() -> Self {
        Self {
            avg_sprites_per_frame: 0.0,
            max_sprites: 0,
            min_sprites: 0,
            size_distribution: HashMap::new(),
            priority_usage: HashMap::new(),
            oam_bytes_per_frame: 0.0,
            processing_time_percent: 0.0,
        }
    }
}

/// Mode 7 graphics statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mode7Stats {
    /// Number of Mode 7 frames rendered
    pub frame_count: u64,
    /// Average matrix calculation cycles
    pub avg_matrix_cycles: f64,
    /// H-IRQ usage count
    pub hirq_count: u64,
    /// Matrix parameter distribution
    pub matrix_params: MatrixParameters,
    /// Perspective correction usage
    pub perspective_correction_percent: f64,
    /// Window clipping operations
    pub window_clipping_count: u64,
}

impl Default for Mode7Stats {
    fn default() -> Self {
        Self {
            frame_count: 0,
            avg_matrix_cycles: 0.0,
            hirq_count: 0,
            matrix_params: MatrixParameters::default(),
            perspective_correction_percent: 0.0,
            window_clipping_count: 0,
        }
    }
}

/// Mode 7 matrix parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatrixParameters {
    /// Parameter A usage distribution
    pub param_a_values: Vec<i16>,
    /// Parameter B usage distribution
    pub param_b_values: Vec<i16>,
    /// Parameter C usage distribution
    pub param_c_values: Vec<i16>,
    /// Parameter D usage distribution
    pub param_d_values: Vec<i16>,
    /// Average center X
    pub avg_center_x: f64,
    /// Average center Y
    pub avg_center_y: f64,
}

impl Default for MatrixParameters {
    fn default() -> Self {
        Self {
            param_a_values: Vec::new(),
            param_b_values: Vec::new(),
            param_c_values: Vec::new(),
            param_d_values: Vec::new(),
            avg_center_x: 0.0,
            avg_center_y: 0.0,
        }
    }
}

/// Individual HDMA transfer record
#[derive(Debug, Clone)]
struct HdmaTransfer {
    channel: u8,
    bytes: u64,
    cycles: u64,
    mode: String,
    dest_reg: String,
    table_addr: u16,
}

/// VRAM access record
#[derive(Debug, Clone, Copy)]
struct VramAccess {
    address: u16,
    is_read: bool,
    timestamp: u64,
}

/// Sprite frame record
#[derive(Debug, Clone)]
struct SpriteFrame {
    sprite_count: u32,
    sprite_sizes: Vec<String>,
    priorities: Vec<u8>,
    oam_bytes: u64,
}

/// Mode 7 frame record
#[derive(Debug, Clone)]
struct Mode7Frame {
    matrix_cycles: u64,
    params: MatrixValues,
    used_hirq: bool,
    window_clipping: u32,
}

#[derive(Debug, Clone, Copy)]
struct MatrixValues {
    a: i16,
    b: i16,
    c: i16,
    d: i16,
    center_x: u16,
    center_y: u16,
}

/// Graphics profiler for SNES graphics performance analysis
#[derive(Debug)]
pub struct GraphicsProfiler {
    /// Whether currently recording
    recording: bool,
    /// When recording started
    recording_start: Option<Instant>,
    /// HDMA transfers
    hdma_transfers: Vec<HdmaTransfer>,
    /// VRAM accesses
    vram_accesses: Vec<VramAccess>,
    /// Sprite frames
    sprite_frames: Vec<SpriteFrame>,
    /// Current frame sprite data
    current_frame_sprites: Vec<SpriteInfo>,
    /// Mode 7 frames
    mode7_frames: Vec<Mode7Frame>,
    /// Current Mode 7 data
    current_mode7: Option<Mode7FrameBuilder>,
    /// Total cycles tracked
    total_cycles: u64,
}

#[derive(Debug, Clone)]
struct SpriteInfo {
    size: String,
    priority: u8,
}

#[derive(Debug, Default)]
struct Mode7FrameBuilder {
    matrix_cycles: u64,
    params: Option<MatrixValues>,
    hirq_used: bool,
    window_clipping: u32,
}

impl GraphicsProfiler {
    /// Create a new graphics profiler
    pub fn new() -> Self {
        Self {
            recording: false,
            recording_start: None,
            hdma_transfers: Vec::with_capacity(1024),
            vram_accesses: Vec::with_capacity(4096),
            sprite_frames: Vec::with_capacity(256),
            current_frame_sprites: Vec::with_capacity(128),
            mode7_frames: Vec::with_capacity(256),
            current_mode7: None,
            total_cycles: 0,
        }
    }

    /// Record an HDMA transfer
    pub fn record_hdma_transfer(
        &mut self,
        channel: u8,
        bytes: u64,
        cycles: u64,
        mode: &str,
        dest_reg: &str,
        table_addr: u16,
    ) {
        if !self.recording {
            return;
        }

        self.hdma_transfers.push(HdmaTransfer {
            channel,
            bytes,
            cycles,
            mode: mode.to_string(),
            dest_reg: dest_reg.to_string(),
            table_addr,
        });

        self.total_cycles += cycles;
    }

    /// Record VRAM access
    pub fn record_vram_access(&mut self, address: u16, is_read: bool) {
        if !self.recording {
            return;
        }

        self.vram_accesses.push(VramAccess {
            address,
            is_read,
            timestamp: self.total_cycles,
        });
    }

    /// Record sprite for current frame
    pub fn record_sprite(&mut self, size: &str, priority: u8) {
        if !self.recording {
            return;
        }

        self.current_frame_sprites.push(SpriteInfo {
            size: size.to_string(),
            priority,
        });
    }

    /// End current sprite frame
    pub fn end_sprite_frame(&mut self, oam_bytes: u64) {
        if !self.recording || self.current_frame_sprites.is_empty() {
            return;
        }

        let sprite_count = self.current_frame_sprites.len() as u32;
        let sprite_sizes: Vec<_> = self
            .current_frame_sprites
            .iter()
            .map(|s| s.size.clone())
            .collect();
        let priorities: Vec<_> = self.current_frame_sprites.iter().map(|s| s.priority).collect();

        self.sprite_frames.push(SpriteFrame {
            sprite_count,
            sprite_sizes,
            priorities,
            oam_bytes,
        });

        self.current_frame_sprites.clear();
    }

    /// Start a Mode 7 frame
    pub fn start_mode7_frame(&mut self) {
        if !self.recording {
            return;
        }

        self.current_mode7 = Some(Mode7FrameBuilder::default());
    }

    /// Record Mode 7 matrix calculation
    pub fn record_mode7_matrix(&mut self, cycles: u64, a: i16, b: i16, c: i16, d: i16) {
        if !self.recording {
            return;
        }

        if let Some(ref mut frame) = self.current_mode7 {
            frame.matrix_cycles += cycles;
            frame.params = Some(MatrixValues {
                a,
                b,
                c,
                d,
                center_x: 0,
                center_y: 0,
            });
        }
    }

    /// Record Mode 7 center coordinates
    pub fn record_mode7_center(&mut self, center_x: u16, center_y: u16) {
        if !self.recording {
            return;
        }

        if let Some(ref mut frame) = self.current_mode7 {
            if let Some(ref mut params) = frame.params {
                params.center_x = center_x;
                params.center_y = center_y;
            }
        }
    }

    /// Record H-IRQ usage for Mode 7
    pub fn record_mode7_hirq(&mut self) {
        if !self.recording {
            return;
        }

        if let Some(ref mut frame) = self.current_mode7 {
            frame.hirq_used = true;
        }
    }

    /// Record window clipping for Mode 7
    pub fn record_mode7_window_clipping(&mut self) {
        if !self.recording {
            return;
        }

        if let Some(ref mut frame) = self.current_mode7 {
            frame.window_clipping += 1;
        }
    }

    /// End current Mode 7 frame
    pub fn end_mode7_frame(&mut self) {
        if !self.recording {
            return;
        }

        if let Some(frame) = self.current_mode7.take() {
            if let Some(params) = frame.params {
                self.mode7_frames.push(Mode7Frame {
                    matrix_cycles: frame.matrix_cycles,
                    params,
                    used_hirq: frame.hirq_used,
                    window_clipping: frame.window_clipping,
                });
            }
        }
    }

    /// Generate graphics performance report
    pub fn generate_report(&self) -> Result<GraphicsReport> {
        if self.recording {
            return Err(ProfilerError::AlreadyRecording);
        }

        let duration = self.recording_start.map(|_| Duration::from_secs(1)).unwrap_or_default();

        let hdma_stats = self.calculate_hdma_stats();
        let vram_stats = self.calculate_vram_stats();
        let sprite_stats = self.calculate_sprite_stats();
        let mode7_stats = self.calculate_mode7_stats();

        let bottlenecks = self.find_bottlenecks_internal(&hdma_stats, &vram_stats, &sprite_stats, &mode7_stats);

        Ok(GraphicsReport {
            duration,
            hdma_stats,
            vram_stats,
            sprite_stats,
            mode7_stats,
            bottlenecks,
        })
    }

    /// Find graphics performance bottlenecks
    pub fn find_bottlenecks(&self) -> Result<Vec<Bottleneck>> {
        if self.total_cycles == 0 && self.hdma_transfers.is_empty() {
            return Err(ProfilerError::InsufficientData(
                "No graphics data collected".to_string(),
            ));
        }

        let hdma_stats = self.calculate_hdma_stats();
        let vram_stats = self.calculate_vram_stats();
        let sprite_stats = self.calculate_sprite_stats();
        let mode7_stats = self.calculate_mode7_stats();

        Ok(self.find_bottlenecks_internal(&hdma_stats, &vram_stats, &sprite_stats, &mode7_stats))
    }

    // Private helper methods
    fn calculate_hdma_stats(&self) -> HdmaStats {
        if self.hdma_transfers.is_empty() {
            return HdmaStats::default();
        }

        let transfer_count = self.hdma_transfers.len() as u64;
        let total_bytes: u64 = self.hdma_transfers.iter().map(|t| t.bytes).sum();
        let total_cycles: u64 = self.hdma_transfers.iter().map(|t| t.cycles).sum();

        // Aggregate channel usage
        let mut channel_map: HashMap<u8, Vec<&HdmaTransfer>> = HashMap::new();
        for transfer in &self.hdma_transfers {
            channel_map.entry(transfer.channel).or_default().push(transfer);
        }

        let channel_usage: HashMap<_, _> = channel_map
            .into_iter()
            .map(|(channel, transfers)| {
                let bytes: u64 = transfers.iter().map(|t| t.bytes).sum();
                let count = transfers.len() as u64;
                let modes: Vec<_> = transfers.iter().map(|t| t.mode.clone()).collect();
                let dest_regs: Vec<_> = transfers.iter().map(|t| t.dest_reg.clone()).collect();
                let table_addrs: Vec<_> = transfers.iter().map(|t| t.table_addr).collect();

                let usage = ChannelUsage {
                    channel,
                    transfer_count: count,
                    bytes_transferred: bytes,
                    transfer_modes: modes,
                    destination_registers: dest_regs,
                    table_addresses: table_addrs,
                };

                (channel, usage)
            })
            .collect();

        HdmaStats {
            transfer_count,
            total_bytes_transferred: total_bytes,
            channel_usage,
            avg_transfer_cycles: total_cycles as f64 / transfer_count as f64,
            hdma_overhead_percent: (total_cycles as f64 / self.total_cycles.max(1) as f64) * 100.0,
        }
    }

    fn calculate_vram_stats(&self) -> VramStats {
        if self.vram_accesses.is_empty() {
            return VramStats::default();
        }

        let total_reads = self.vram_accesses.iter().filter(|a| a.is_read).count() as u64;
        let total_writes = self.vram_accesses.iter().filter(|a| !a.is_read).count() as u64;

        // Analyze access patterns
        let access_patterns = self.analyze_access_patterns();

        // Find hot regions
        let hot_regions = self.find_hot_regions();

        VramStats {
            total_reads,
            total_writes,
            access_patterns,
            bandwidth_utilization: 0.0, // Would need timing info
            hot_regions,
        }
    }

    fn analyze_access_patterns(&self) -> AccessPatterns {
        if self.vram_accesses.len() < 2 {
            return AccessPatterns::default();
        }

        let mut sequential = 0u64;
        let mut strided = 0u64;
        let mut random = 0u64;
        let mut total_stride: i64 = 0;

        for window in self.vram_accesses.windows(2) {
            let diff = (window[1].address as i64) - (window[0].address as i64);
            match diff {
                1 => sequential += 1,
                2..=32 => {
                    strided += 1;
                    total_stride += diff;
                }
                _ => random += 1,
            }
        }

        let total = sequential + strided + random;
        let avg_stride = if strided > 0 {
            total_stride as f64 / strided as f64
        } else {
            0.0
        };

        AccessPatterns {
            sequential_percent: (sequential as f64 / total as f64) * 100.0,
            random_percent: (random as f64 / total as f64) * 100.0,
            strided_percent: (strided as f64 / total as f64) * 100.0,
            avg_stride,
        }
    }

    fn find_hot_regions(&self) -> Vec<VramRegion> {
        const REGION_SIZE: u16 = 256;
        let mut region_accesses: HashMap<u16, (u64, u64)> = HashMap::new();

        for access in &self.vram_accesses {
            let region_start = (access.address / REGION_SIZE) * REGION_SIZE;
            let entry = region_accesses.entry(region_start).or_insert((0, 0));
            if access.is_read {
                entry.0 += 1;
            } else {
                entry.1 += 1;
            }
        }

        let mut regions: Vec<_> = region_accesses
            .into_iter()
            .map(|(start, (reads, writes))| {
                let access_type = if reads > 0 && writes > 0 {
                    "both"
                } else if reads > 0 {
                    "read"
                } else {
                    "write"
                };

                VramRegion {
                    start,
                    end: start + REGION_SIZE - 1,
                    access_count: reads + writes,
                    access_type: access_type.to_string(),
                }
            })
            .collect();

        regions.sort_by(|a, b| b.access_count.cmp(&a.access_count));
        regions.truncate(10);
        regions
    }

    fn calculate_sprite_stats(&self) -> SpriteStats {
        if self.sprite_frames.is_empty() {
            return SpriteStats::default();
        }

        let frame_count = self.sprite_frames.len() as f64;
        let total_sprites: u64 = self.sprite_frames.iter().map(|f| f.sprite_count as u64).sum();
        let avg_sprites = total_sprites as f64 / frame_count;

        let max_sprites = self.sprite_frames.iter().map(|f| f.sprite_count).max().unwrap_or(0);
        let min_sprites = self.sprite_frames.iter().map(|f| f.sprite_count).min().unwrap_or(0);

        // Size distribution
        let mut size_dist: HashMap<String, u64> = HashMap::new();
        for frame in &self.sprite_frames {
            for size in &frame.sprite_sizes {
                *size_dist.entry(size.clone()).or_insert(0) += 1;
            }
        }

        // Priority usage
        let mut priority_usage: HashMap<u8, u64> = HashMap::new();
        for frame in &self.sprite_frames {
            for &priority in &frame.priorities {
                *priority_usage.entry(priority).or_insert(0) += 1;
            }
        }

        let total_oam: u64 = self.sprite_frames.iter().map(|f| f.oam_bytes).sum();
        let avg_oam = total_oam as f64 / frame_count;

        SpriteStats {
            avg_sprites_per_frame: avg_sprites,
            max_sprites,
            min_sprites,
            size_distribution: size_dist,
            priority_usage,
            oam_bytes_per_frame: avg_oam,
            processing_time_percent: 0.0, // Would need timing info
        }
    }

    fn calculate_mode7_stats(&self) -> Mode7Stats {
        if self.mode7_frames.is_empty() {
            return Mode7Stats::default();
        }

        let frame_count = self.mode7_frames.len() as u64;
        let total_cycles: u64 = self.mode7_frames.iter().map(|f| f.matrix_cycles).sum();
        let avg_cycles = total_cycles as f64 / frame_count as f64;

        let hirq_count = self.mode7_frames.iter().filter(|f| f.used_hirq).count() as u64;
        let window_clipping: u64 = self.mode7_frames.iter().map(|f| f.window_clipping as u64).sum();

        // Collect matrix parameters
        let mut params = MatrixParameters::default();
        for frame in &self.mode7_frames {
            params.param_a_values.push(frame.params.a);
            params.param_b_values.push(frame.params.b);
            params.param_c_values.push(frame.params.c);
            params.param_d_values.push(frame.params.d);
        }

        let avg_center_x = self.mode7_frames.iter().map(|f| f.params.center_x as f64).sum::<f64>() / frame_count as f64;
        let avg_center_y = self.mode7_frames.iter().map(|f| f.params.center_y as f64).sum::<f64>() / frame_count as f64;

        params.avg_center_x = avg_center_x;
        params.avg_center_y = avg_center_y;

        Mode7Stats {
            frame_count,
            avg_matrix_cycles: avg_cycles,
            hirq_count,
            matrix_params: params,
            perspective_correction_percent: (hirq_count as f64 / frame_count as f64) * 100.0,
            window_clipping_count: window_clipping,
        }
    }

    fn find_bottlenecks_internal(
        &self,
        hdma: &HdmaStats,
        vram: &VramStats,
        sprites: &SpriteStats,
        mode7: &Mode7Stats,
    ) -> Vec<Bottleneck> {
        let mut bottlenecks = Vec::new();

        // HDMA bottlenecks
        if hdma.hdma_overhead_percent > 20.0 {
            bottlenecks.push(Bottleneck {
                description: "HDMA overhead is very high".to_string(),
                category: "HDMA".to_string(),
                severity: Severity::High,
                impact: format!("{:.1}% of frame time", hdma.hdma_overhead_percent),
                suggestion: "Consolidate HDMA channels or reduce transfer frequency".to_string(),
                location: None,
            });
        }

        // VRAM bottlenecks
        if vram.access_patterns.random_percent > 50.0 {
            bottlenecks.push(Bottleneck {
                description: "High random VRAM access pattern".to_string(),
                category: "VRAM".to_string(),
                severity: Severity::Medium,
                impact: format!("{:.1}% random accesses", vram.access_patterns.random_percent),
                suggestion: "Optimize to use sequential VRAM access patterns".to_string(),
                location: None,
            });
        }

        // Sprite bottlenecks
        if sprites.avg_sprites_per_frame > 100.0 {
            bottlenecks.push(Bottleneck {
                description: "High sprite count may cause dropouts".to_string(),
                category: "Sprites".to_string(),
                severity: Severity::Medium,
                impact: format!("{:.1} sprites/frame average", sprites.avg_sprites_per_frame),
                suggestion: "Reduce sprite count or use larger sprite sizes".to_string(),
                location: None,
            });
        }

        // Mode 7 bottlenecks
        if mode7.perspective_correction_percent > 80.0 {
            bottlenecks.push(Bottleneck {
                description: "Heavy H-IRQ usage for Mode 7 perspective correction".to_string(),
                category: "Mode7".to_string(),
                severity: Severity::High,
                impact: format!("{:.1}% of frames use H-IRQ", mode7.perspective_correction_percent),
                suggestion: "Consider pre-calculated tables or simplified effects".to_string(),
                location: None,
            });
        }

        bottlenecks
    }
}

impl Default for GraphicsProfiler {
    fn default() -> Self {
        Self::new()
    }
}

impl ProfilerTrait for GraphicsProfiler {
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
        self.hdma_transfers.clear();
        self.vram_accesses.clear();
        self.sprite_frames.clear();
        self.current_frame_sprites.clear();
        self.mode7_frames.clear();
        self.current_mode7 = None;
        self.total_cycles = 0;
    }

    fn recording_duration(&self) -> Option<Duration> {
        self.recording_start.map(|start| start.elapsed())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graphics_profiler_start_stop() {
        let mut profiler = GraphicsProfiler::new();
        assert!(!profiler.is_recording());

        profiler.start_recording().unwrap();
        assert!(profiler.is_recording());

        profiler.stop_recording().unwrap();
        assert!(!profiler.is_recording());
    }

    #[test]
    fn test_hdma_recording() {
        let mut profiler = GraphicsProfiler::new();
        profiler.start_recording().unwrap();

        profiler.record_hdma_transfer(0, 256, 100, "1-reg", "$2105", 0x1000);
        profiler.record_hdma_transfer(1, 512, 150, "2-reg", "$2106", 0x2000);

        profiler.stop_recording().unwrap();

        let report = profiler.generate_report().unwrap();
        assert_eq!(report.hdma_stats.transfer_count, 2);
        assert_eq!(report.hdma_stats.total_bytes_transferred, 768);
    }

    #[test]
    fn test_vram_access_recording() {
        let mut profiler = GraphicsProfiler::new();
        profiler.start_recording().unwrap();

        for i in 0..100 {
            profiler.record_vram_access(i * 2, true);
        }

        profiler.stop_recording().unwrap();

        let report = profiler.generate_report().unwrap();
        assert_eq!(report.vram_stats.total_reads, 100);
        assert!(report.vram_stats.access_patterns.sequential_percent > 50.0);
    }

    #[test]
    fn test_sprite_recording() {
        let mut profiler = GraphicsProfiler::new();
        profiler.start_recording().unwrap();

        for _ in 0..10 {
            profiler.record_sprite("8x8", 2);
            profiler.record_sprite("16x16", 1);
        }
        profiler.end_sprite_frame(512);

        profiler.stop_recording().unwrap();

        let report = profiler.generate_report().unwrap();
        assert_eq!(report.sprite_stats.avg_sprites_per_frame, 20.0);
    }

    #[test]
    fn test_mode7_recording() {
        let mut profiler = GraphicsProfiler::new();
        profiler.start_recording().unwrap();

        profiler.start_mode7_frame();
        profiler.record_mode7_matrix(500, 256, 0, 0, 256);
        profiler.record_mode7_center(128, 112);
        profiler.record_mode7_hirq();
        profiler.end_mode7_frame();

        profiler.stop_recording().unwrap();

        let report = profiler.generate_report().unwrap();
        assert_eq!(report.mode7_stats.frame_count, 1);
        assert_eq!(report.mode7_stats.hirq_count, 1);
    }
}
