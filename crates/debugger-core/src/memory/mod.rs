//! Memory Watcher and Heatmap
//!
//! Provides tracking of memory accesses for debugging and analysis.
//! Includes:
//! - Memory access tracking (read/write/execute)
//! - Memory heatmap for visualizing access patterns
//! - Watchpoints for specific memory addresses

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Type of memory access
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AccessType {
    /// Memory read operation
    Read,
    /// Memory write operation
    Write,
    /// Instruction execution (read from executable memory)
    Execute,
}

impl std::fmt::Display for AccessType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccessType::Read => write!(f, "R"),
            AccessType::Write => write!(f, "W"),
            AccessType::Execute => write!(f, "X"),
        }
    }
}

/// Single memory access entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryAccess {
    /// Address that was accessed
    pub address: u32,
    /// Type of access
    pub access_type: AccessType,
    /// Value that was read or written
    pub value: u8,
    /// Program counter at time of access
    pub pc: u32,
    /// Cycle count at time of access
    pub cycle: u64,
    /// Timestamp (can be frame number or system tick)
    pub timestamp: u64,
}

/// Memory access statistics for a single address
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AddressStats {
    /// Number of read operations
    pub read_count: u64,
    /// Number of write operations
    pub write_count: u64,
    /// Number of execute operations
    pub execute_count: u64,
    /// First access timestamp
    pub first_access: Option<u64>,
    /// Last access timestamp
    pub last_access: Option<u64>,
    /// Values written to this address (for tracking changes)
    pub last_value: Option<u8>,
    /// Number of times the value changed
    pub value_changes: u64,
}

impl AddressStats {
    /// Get total access count
    pub fn total_accesses(&self) -> u64 {
        self.read_count + self.write_count + self.execute_count
    }

    /// Record a memory access
    pub fn record_access(&mut self, access_type: AccessType, value: u8, timestamp: u64) {
        match access_type {
            AccessType::Read => self.read_count += 1,
            AccessType::Write => {
                self.write_count += 1;
                if let Some(last) = self.last_value {
                    if last != value {
                        self.value_changes += 1;
                    }
                }
                self.last_value = Some(value);
            }
            AccessType::Execute => self.execute_count += 1,
        }

        if self.first_access.is_none() {
            self.first_access = Some(timestamp);
        }
        self.last_access = Some(timestamp);
    }
}

/// Watchpoint for monitoring specific memory addresses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Watchpoint {
    pub id: u64,
    pub address: u32,
    pub access_type: WatchCondition,
    pub enabled: bool,
    pub hit_count: u64,
    pub description: Option<String>,
}

/// Watchpoint condition
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WatchCondition {
    /// Trigger on any access
    Any,
    /// Trigger only on reads
    Read,
    /// Trigger only on writes
    Write,
    /// Trigger only on execute
    Execute,
    /// Trigger when a specific value is written
    WriteValue(u8),
    /// Trigger when value changes
    Change,
}

/// Memory watcher that tracks all accesses
#[derive(Debug)]
pub struct MemoryWatcher {
    /// Access history (circular buffer of recent accesses)
    history: Vec<MemoryAccess>,
    /// Maximum history size
    max_history: usize,
    /// Statistics per address
    address_stats: HashMap<u32, AddressStats>,
    /// Active watchpoints
    watchpoints: HashMap<u64, Watchpoint>,
    /// Next watchpoint ID
    next_watchpoint_id: u64,
    /// Current cycle counter
    cycle: u64,
    /// Current timestamp
    timestamp: u64,
    /// Memory backing store (for the watcher)
    memory: Vec<u8>,
    /// Memory size
    memory_size: usize,
}

impl Default for MemoryWatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryWatcher {
    /// Create a new memory watcher with default 16MB SNES address space
    pub fn new() -> Self {
        Self::with_size(0x100_0000) // 16MB
    }

    /// Create a memory watcher with a specific memory size
    pub fn with_size(size: usize) -> Self {
        Self {
            history: Vec::with_capacity(10000),
            max_history: 10000,
            address_stats: HashMap::new(),
            watchpoints: HashMap::new(),
            next_watchpoint_id: 1,
            cycle: 0,
            timestamp: 0,
            memory: vec![0; size],
            memory_size: size,
        }
    }

    /// Read memory without recording access
    pub fn read_silent(&self, addr: u32, size: usize) -> Vec<u8> {
        let mut result = Vec::with_capacity(size);
        for i in 0..size {
            let idx = ((addr as usize) + i) % self.memory_size;
            result.push(self.memory[idx]);
        }
        result
    }

    /// Read a single byte without recording access
    pub fn read_byte_silent(&self, addr: u32) -> u8 {
        self.memory[(addr as usize) % self.memory_size]
    }

    /// Read memory and record access
    pub fn read(&mut self, addr: u32, size: usize) -> Vec<u8> {
        let mut result = Vec::with_capacity(size);
        for i in 0..size {
            let address = addr + i as u32;
            let idx = (address as usize) % self.memory_size;
            let value = self.memory[idx];
            result.push(value);
            self.record_access(address, AccessType::Read, value, 0);
        }
        result
    }

    /// Write memory and record access
    pub fn write(&mut self, addr: u32, data: &[u8]) {
        for (i, &value) in data.iter().enumerate() {
            let address = addr + i as u32;
            let idx = (address as usize) % self.memory_size;
            self.memory[idx] = value;
            self.record_access(address, AccessType::Write, value, 0);
        }
    }

    /// Record an instruction execution at the given address
    pub fn record_execute(&mut self, addr: u32, pc: u32) {
        let value = self.read_byte_silent(addr);
        self.record_access(addr, AccessType::Execute, value, pc);
    }

    /// Record a memory access
    fn record_access(&mut self, address: u32, access_type: AccessType, value: u8, pc: u32) {
        let timestamp = self.timestamp;

        // Add to history
        let access = MemoryAccess {
            address,
            access_type,
            value,
            pc,
            cycle: self.cycle,
            timestamp,
        };

        if self.history.len() >= self.max_history {
            self.history.remove(0);
        }
        self.history.push(access);

        // Update address statistics
        let stats = self
            .address_stats
            .entry(address)
            .or_insert_with(AddressStats::default);
        stats.record_access(access_type, value, timestamp);

        // Check watchpoints
        self.check_watchpoints(address, access_type, value);
    }

    /// Increment cycle counter
    pub fn tick_cycle(&mut self) {
        self.cycle += 1;
    }

    /// Increment timestamp (e.g., per frame)
    pub fn tick_timestamp(&mut self) {
        self.timestamp += 1;
    }

    /// Add a watchpoint
    pub fn add_watchpoint(&mut self, address: u32, condition: WatchCondition) -> u64 {
        let id = self.next_watchpoint_id;
        self.next_watchpoint_id += 1;

        let watchpoint = Watchpoint {
            id,
            address,
            access_type: condition,
            enabled: true,
            hit_count: 0,
            description: None,
        };

        self.watchpoints.insert(id, watchpoint);
        id
    }

    /// Remove a watchpoint
    pub fn remove_watchpoint(&mut self, id: u64) -> bool {
        self.watchpoints.remove(&id).is_some()
    }

    /// Enable/disable a watchpoint
    pub fn toggle_watchpoint(&mut self, id: u64, enabled: bool) -> bool {
        if let Some(wp) = self.watchpoints.get_mut(&id) {
            wp.enabled = enabled;
            true
        } else {
            false
        }
    }

    /// Get all watchpoints
    pub fn watchpoints(&self) -> Vec<&Watchpoint> {
        self.watchpoints.values().collect()
    }

    /// Check if any watchpoints match
    fn check_watchpoints(&mut self, address: u32, access_type: AccessType, value: u8) -> Vec<u64> {
        let mut triggered = Vec::new();

        for wp in self.watchpoints.values_mut() {
            if !wp.enabled || wp.address != address {
                continue;
            }

            let matches = match (wp.access_type, access_type) {
                (WatchCondition::Any, _) => true,
                (WatchCondition::Read, AccessType::Read) => true,
                (WatchCondition::Write, AccessType::Write) => true,
                (WatchCondition::Execute, AccessType::Execute) => true,
                (WatchCondition::WriteValue(v), AccessType::Write) => v == value,
                (WatchCondition::Change, AccessType::Write) => {
                    if let Some(stats) = self.address_stats.get(&address) {
                        stats.value_changes > 0
                    } else {
                        false
                    }
                }
                _ => false,
            };

            if matches {
                wp.hit_count += 1;
                triggered.push(wp.id);
            }
        }

        triggered
    }

    /// Get access history
    pub fn history(&self) -> &[MemoryAccess] {
        &self.history
    }

    /// Get statistics for a specific address
    pub fn address_stats(&self, address: u32) -> Option<&AddressStats> {
        self.address_stats.get(&address)
    }

    /// Get all address statistics
    pub fn all_stats(&self) -> &HashMap<u32, AddressStats> {
        &self.address_stats
    }

    /// Clear all history and statistics
    pub fn clear(&mut self) {
        self.history.clear();
        self.address_stats.clear();
    }

    /// Generate a heatmap for a memory range
    pub fn heatmap(&self, range: std::ops::Range<u32>) -> MemoryHeatmap {
        let mut heatmap_data = Vec::new();

        for addr in range.clone() {
            if let Some(stats) = self.address_stats.get(&addr) {
                heatmap_data.push((addr, stats.clone()));
            }
        }

        MemoryHeatmap {
            range,
            data: heatmap_data,
        }
    }

    /// Get the most frequently accessed addresses
    pub fn hottest_addresses(&self, count: usize) -> Vec<(u32, u64)> {
        let mut addresses: Vec<(u32, u64)> = self
            .address_stats
            .iter()
            .map(|(&addr, stats)| (addr, stats.total_accesses()))
            .collect();

        addresses.sort_by(|a, b| b.1.cmp(&a.1));
        addresses.into_iter().take(count).collect()
    }

    /// Get addresses that have been written to most frequently
    pub fn most_written(&self, count: usize) -> Vec<(u32, u64)> {
        let mut addresses: Vec<(u32, u64)> = self
            .address_stats
            .iter()
            .map(|(&addr, stats)| (addr, stats.write_count))
            .collect();

        addresses.sort_by(|a, b| b.1.cmp(&a.1));
        addresses.into_iter().take(count).collect()
    }

    /// Find addresses that changed value most frequently
    pub fn most_volatile(&self, count: usize) -> Vec<(u32, u64)> {
        let mut addresses: Vec<(u32, u64)> = self
            .address_stats
            .iter()
            .map(|(&addr, stats)| (addr, stats.value_changes))
            .collect();

        addresses.sort_by(|a, b| b.1.cmp(&a.1));
        addresses.into_iter().take(count).collect()
    }
}

/// Memory heatmap for visualizing access patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryHeatmap {
    /// Address range covered
    pub range: std::ops::Range<u32>,
    /// Access statistics for addresses with activity
    pub data: Vec<(u32, AddressStats)>,
}

impl MemoryHeatmap {
    /// Create an empty heatmap
    pub fn empty() -> Self {
        Self {
            range: 0..0,
            data: Vec::new(),
        }
    }

    /// Get the maximum access count in this heatmap
    pub fn max_accesses(&self) -> u64 {
        self.data
            .iter()
            .map(|(_, stats)| stats.total_accesses())
            .max()
            .unwrap_or(0)
    }

    /// Get access intensity for an address (0.0 - 1.0)
    pub fn intensity(&self, address: u32) -> f32 {
        if let Some((_, stats)) = self.data.iter().find(|(addr, _)| *addr == address) {
            let max = self.max_accesses().max(1);
            stats.total_accesses() as f32 / max as f32
        } else {
            0.0
        }
    }

    /// Get read intensity for an address (0.0 - 1.0)
    pub fn read_intensity(&self, address: u32) -> f32 {
        if let Some((_, stats)) = self.data.iter().find(|(addr, _)| *addr == address) {
            let max_reads = self
                .data
                .iter()
                .map(|(_, s)| s.read_count)
                .max()
                .unwrap_or(0)
                .max(1);
            stats.read_count as f32 / max_reads as f32
        } else {
            0.0
        }
    }

    /// Get write intensity for an address (0.0 - 1.0)
    pub fn write_intensity(&self, address: u32) -> f32 {
        if let Some((_, stats)) = self.data.iter().find(|(addr, _)| *addr == address) {
            let max_writes = self
                .data
                .iter()
                .map(|(_, s)| s.write_count)
                .max()
                .unwrap_or(0)
                .max(1);
            stats.write_count as f32 / max_writes as f32
        } else {
            0.0
        }
    }

    /// Generate a color for an address based on access type
    /// Returns RGB values
    pub fn color_for_address(&self, address: u32) -> (u8, u8, u8) {
        if let Some((_, stats)) = self.data.iter().find(|(addr, _)| *addr == address) {
            let total = stats.total_accesses() as f32;
            let read_ratio = stats.read_count as f32 / total;
            let write_ratio = stats.write_count as f32 / total;
            let exec_ratio = stats.execute_count as f32 / total;

            // Blend colors: red for writes, green for reads, blue for execute
            let r = (write_ratio * 255.0) as u8;
            let g = (read_ratio * 255.0) as u8;
            let b = (exec_ratio * 255.0) as u8;

            (r, g, b)
        } else {
            (0, 0, 0)
        }
    }

    /// Get summary statistics
    pub fn summary(&self) -> HeatmapSummary {
        let total_addresses = self.data.len();
        let total_reads: u64 = self.data.iter().map(|(_, s)| s.read_count).sum();
        let total_writes: u64 = self.data.iter().map(|(_, s)| s.write_count).sum();
        let total_executes: u64 = self.data.iter().map(|(_, s)| s.execute_count).sum();

        HeatmapSummary {
            total_addresses,
            total_reads,
            total_writes,
            total_executes,
            total_accesses: total_reads + total_writes + total_executes,
        }
    }
}

/// Summary statistics for a heatmap
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct HeatmapSummary {
    pub total_addresses: usize,
    pub total_reads: u64,
    pub total_writes: u64,
    pub total_executes: u64,
    pub total_accesses: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_watcher_basic() {
        let mut watcher = MemoryWatcher::new();

        // Write and read
        watcher.write(0x1000, &[0x42, 0x43, 0x44]);
        let data = watcher.read(0x1000, 3);

        assert_eq!(data, vec![0x42, 0x43, 0x44]);

        // Check stats
        let stats = watcher.address_stats(0x1000).unwrap();
        assert_eq!(stats.write_count, 1);
        assert_eq!(stats.read_count, 1);
    }

    #[test]
    fn test_address_stats() {
        let mut stats = AddressStats::default();

        stats.record_access(AccessType::Read, 0x42, 100);
        stats.record_access(AccessType::Read, 0x42, 200);
        stats.record_access(AccessType::Write, 0x43, 300);

        assert_eq!(stats.read_count, 2);
        assert_eq!(stats.write_count, 1);
        assert_eq!(stats.total_accesses(), 3);
        assert_eq!(stats.first_access, Some(100));
        assert_eq!(stats.last_access, Some(300));
    }

    #[test]
    fn test_watchpoint() {
        let mut watcher = MemoryWatcher::new();

        let wp_id = watcher.add_watchpoint(0x2000, WatchCondition::Write);
        assert_eq!(watcher.watchpoints().len(), 1);

        // Write to trigger watchpoint
        watcher.write(0x2000, &[0x42]);

        let watchpoints = watcher.watchpoints();
        let wp = watchpoints.first().unwrap();
        assert_eq!(wp.hit_count, 1);

        // Remove watchpoint
        assert!(watcher.remove_watchpoint(wp_id));
        assert!(watcher.watchpoints().is_empty());
    }

    #[test]
    fn test_heatmap() {
        let mut watcher = MemoryWatcher::new();

        // Generate some accesses
        for i in 0..100 {
            watcher.write(0x1000 + (i % 10) as u32, &[i]);
        }

        let heatmap = watcher.heatmap(0x1000..0x1010);
        assert_eq!(heatmap.data.len(), 10);

        let summary = heatmap.summary();
        assert_eq!(summary.total_addresses, 10);
        assert_eq!(summary.total_writes, 100);
    }

    #[test]
    fn test_hottest_addresses() {
        let mut watcher = MemoryWatcher::new();

        // Generate accesses with varying frequency
        for _ in 0..10 {
            watcher.read(0x1000, 1);
        }
        for _ in 0..5 {
            watcher.read(0x1001, 1);
        }
        watcher.read(0x1002, 1);

        let hottest = watcher.hottest_addresses(2);
        assert_eq!(hottest.len(), 2);
        assert_eq!(hottest[0].0, 0x1000);
        assert_eq!(hottest[0].1, 10);
    }
}
