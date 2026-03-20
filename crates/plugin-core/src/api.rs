//! Plugin API - Safe interface for plugins to interact with the editor

use crate::types::*;
use crate::{PluginContext, PluginError, PluginResult};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// API exposed to plugins
/// 
/// This provides a controlled interface for plugins to access and modify
/// ROM data, assets, and editor state.
pub struct PluginApi {
    context: Arc<RwLock<PluginContext>>,
    /// Callback to read ROM bytes
    rom_reader: Box<dyn Fn(usize, usize) -> PluginResult<Vec<u8>> + Send + Sync>,
    /// Callback to write ROM bytes
    rom_writer: Box<dyn Fn(usize, &[u8]) -> PluginResult<()> + Send + Sync>,
    /// Callback to get asset info
    asset_getter: Box<dyn Fn(&str) -> PluginResult<AssetInfo> + Send + Sync>,
    /// Callback to log messages
    logger: Box<dyn Fn(log::Level, &str) + Send + Sync>,
    /// Callback to show notifications
    notifier: Box<dyn Fn(&str, NotificationType) + Send + Sync>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationType {
    Info,
    Success,
    Warning,
    Error,
}

impl PluginApi {
    pub fn new(context: Arc<RwLock<PluginContext>>) -> Self {
        Self {
            context,
            rom_reader: Box::new(|_, _| Err(PluginError::ApiError("ROM reader not initialized".into()))),
            rom_writer: Box::new(|_, _| Err(PluginError::ApiError("ROM writer not initialized".into()))),
            asset_getter: Box::new(|_| Err(PluginError::ApiError("Asset getter not initialized".into()))),
            logger: Box::new(|level, msg| println!("[{:?}] {}", level, msg)),
            notifier: Box::new(|msg, _| println!("[Notification] {}", msg)),
        }
    }
    
    pub fn with_rom_reader<F>(mut self, reader: F) -> Self
    where
        F: Fn(usize, usize) -> PluginResult<Vec<u8>> + Send + Sync + 'static,
    {
        self.rom_reader = Box::new(reader);
        self
    }
    
    pub fn with_rom_writer<F>(mut self, writer: F) -> Self
    where
        F: Fn(usize, &[u8]) -> PluginResult<()> + Send + Sync + 'static,
    {
        self.rom_writer = Box::new(writer);
        self
    }
    
    pub fn with_asset_getter<F>(mut self, getter: F) -> Self
    where
        F: Fn(&str) -> PluginResult<AssetInfo> + Send + Sync + 'static,
    {
        self.asset_getter = Box::new(getter);
        self
    }
    
    pub fn with_logger<F>(mut self, logger: F) -> Self
    where
        F: Fn(log::Level, &str) + Send + Sync + 'static,
    {
        self.logger = Box::new(logger);
        self
    }
    
    pub fn with_notifier<F>(mut self, notifier: F) -> Self
    where
        F: Fn(&str, NotificationType) + Send + Sync + 'static,
    {
        self.notifier = Box::new(notifier);
        self
    }
    
    // ============================================================================
    // ROM Operations
    // ============================================================================
    
    /// Read bytes from ROM at given offset
    pub fn rom_read(&self, offset: usize, length: usize) -> PluginResult<Vec<u8>> {
        (self.rom_reader)(offset, length)
    }
    
    /// Read a single byte from ROM
    pub fn rom_read_byte(&self, offset: usize) -> PluginResult<u8> {
        let bytes = self.rom_read(offset, 1)?;
        Ok(bytes[0])
    }
    
    /// Read a u16 value from ROM (little-endian)
    pub fn rom_read_u16(&self, offset: usize) -> PluginResult<u16> {
        let bytes = self.rom_read(offset, 2)?;
        Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
    }
    
    /// Read a u24 value from ROM (3 bytes, little-endian)
    pub fn rom_read_u24(&self, offset: usize) -> PluginResult<u32> {
        let bytes = self.rom_read(offset, 3)?;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], 0]))
    }
    
    /// Write bytes to ROM at given offset
    pub fn rom_write(&self, offset: usize, data: &[u8]) -> PluginResult<()> {
        (self.rom_writer)(offset, data)
    }
    
    /// Write a single byte to ROM
    pub fn rom_write_byte(&self, offset: usize, value: u8) -> PluginResult<()> {
        self.rom_write(offset, &[value])
    }
    
    /// Get ROM size
    pub fn rom_size(&self) -> PluginResult<usize> {
        let ctx = self.context.read();
        match &ctx.rom_data {
            Some(data) => Ok(data.read().len()),
            None => Err(PluginError::ApiError("No ROM loaded".into())),
        }
    }
    
    /// Calculate ROM checksum
    pub fn rom_checksum(&self) -> PluginResult<u32> {
        let data = self.rom_read(0, self.rom_size()?)?;
        let sum: u32 = data.iter().map(|&b| b as u32).sum();
        Ok(sum)
    }
    
    // ============================================================================
    // Asset Operations
    // ============================================================================
    
    /// Get information about an asset
    pub fn get_asset(&self, asset_id: &str) -> PluginResult<AssetInfo> {
        (self.asset_getter)(asset_id)
    }
    
    /// List all assets of a given type
    pub fn list_assets(&self, asset_type: AssetType) -> PluginResult<Vec<AssetInfo>> {
        // This would query the manifest or asset database
        // For now, return empty
        Ok(Vec::new())
    }
    
    /// Check if an asset exists
    pub fn asset_exists(&self, asset_id: &str) -> PluginResult<bool> {
        match self.get_asset(asset_id) {
            Ok(_) => Ok(true),
            Err(PluginError::ApiError(_)) => Ok(false),
            Err(e) => Err(e),
        }
    }
    
    // ============================================================================
    // Logging & Notifications
    // ============================================================================
    
    /// Log a debug message
    pub fn log_debug(&self, message: &str) {
        (self.logger)(log::Level::Debug, message);
    }
    
    /// Log an info message
    pub fn log_info(&self, message: &str) {
        (self.logger)(log::Level::Info, message);
    }
    
    /// Log a warning message
    pub fn log_warn(&self, message: &str) {
        (self.logger)(log::Level::Warn, message);
    }
    
    /// Log an error message
    pub fn log_error(&self, message: &str) {
        (self.logger)(log::Level::Error, message);
    }
    
    /// Show an info notification
    pub fn notify_info(&self, message: &str) {
        (self.notifier)(message, NotificationType::Info);
    }
    
    /// Show a success notification
    pub fn notify_success(&self, message: &str) {
        (self.notifier)(message, NotificationType::Success);
    }
    
    /// Show a warning notification
    pub fn notify_warning(&self, message: &str) {
        (self.notifier)(message, NotificationType::Warning);
    }
    
    /// Show an error notification
    pub fn notify_error(&self, message: &str) {
        (self.notifier)(message, NotificationType::Error);
    }
    
    // ============================================================================
    // Editor State
    // ============================================================================
    
    /// Get the currently selected boxer
    pub fn get_selected_boxer(&self) -> Option<String> {
        self.context.read().selected_boxer.clone()
    }
    
    /// Set the currently selected boxer
    pub fn set_selected_boxer(&self, boxer: Option<String>) {
        self.context.write().selected_boxer = boxer;
    }
    
    /// Get the current project path
    pub fn get_project_path(&self) -> Option<std::path::PathBuf> {
        self.context.read().project_path.clone()
    }
    
    /// Check if a ROM is loaded
    pub fn is_rom_loaded(&self) -> bool {
        self.context.read().rom_data.is_some()
    }
    
    // ============================================================================
    // Utility Functions
    // ============================================================================
    
    /// Convert SNES LoROM address to PC offset
    pub fn snes_to_pc(&self, bank: u8, addr: u16) -> usize {
        ((bank as usize & 0x7F) * 0x8000) | (addr as usize & 0x7FFF)
    }
    
    /// Convert PC offset to SNES LoROM address
    pub fn pc_to_snes(&self, pc: usize) -> (u8, u16) {
        let bank = ((pc / 0x8000) | 0x80) as u8;
        let addr = ((pc % 0x8000) | 0x8000) as u16;
        (bank, addr)
    }
    
    /// Search for a byte pattern in ROM
    pub fn find_pattern(&self, pattern: &[u8]) -> PluginResult<Vec<usize>> {
        let rom_size = self.rom_size()?;
        let rom_data = self.rom_read(0, rom_size)?;
        
        let mut matches = Vec::new();
        for i in 0..=rom_data.len().saturating_sub(pattern.len()) {
            if &rom_data[i..i + pattern.len()] == pattern {
                matches.push(i);
            }
        }
        Ok(matches)
    }
    
    /// Compare two regions of ROM
    pub fn compare_regions(&self, offset1: usize, offset2: usize, length: usize) -> PluginResult<Vec<usize>> {
        let data1 = self.rom_read(offset1, length)?;
        let data2 = self.rom_read(offset2, length)?;
        
        let mut diffs = Vec::new();
        for i in 0..length {
            if data1[i] != data2[i] {
                diffs.push(i);
            }
        }
        Ok(diffs)
    }
}

/// Statistics about ROM usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RomStats {
    pub total_size: usize,
    pub used_bytes: usize,
    pub free_bytes: usize,
    pub asset_breakdown: HashMap<AssetType, usize>,
}

/// Batch operation helper
pub struct BatchOperation<'a> {
    api: &'a PluginApi,
    operations: Vec<Box<dyn FnOnce() -> PluginResult<()> + Send>>,
}

impl<'a> BatchOperation<'a> {
    pub fn new(api: &'a PluginApi) -> Self {
        Self {
            api,
            operations: Vec::new(),
        }
    }
    
    pub fn add<F>(&mut self, op: F)
    where
        F: FnOnce() -> PluginResult<()> + Send + 'static,
    {
        self.operations.push(Box::new(op));
    }
    
    pub fn execute(self) -> BatchResult {
        let mut success_count = 0;
        let mut failure_count = 0;
        let mut errors = Vec::new();
        let mut results = Vec::new();
        
        for op in self.operations {
            match op() {
                Ok(_) => {
                    success_count += 1;
                    results.push(serde_json::json!({"success": true}));
                }
                Err(e) => {
                    failure_count += 1;
                    errors.push(e.to_string());
                    results.push(serde_json::json!({"success": false, "error": e.to_string()}));
                }
            }
        }
        
        BatchResult {
            success_count,
            failure_count,
            errors,
            results,
        }
    }
}
