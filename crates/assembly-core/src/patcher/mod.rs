//! # Patcher Module
//!
//! Code patcher for ROM modifications, including code insertion,
//! trampoline generation, checksum fixing, and jump table modification.

use crate::{AssemblyError, Result};
use rom_core::Rom;
use std::collections::HashMap;

/// A code patch
#[derive(Debug, Clone)]
pub struct Patch {
    /// Address to apply the patch
    pub address: u32,
    /// Original bytes (for verification)
    pub original_bytes: Vec<u8>,
    /// New bytes to write
    pub new_bytes: Vec<u8>,
    /// Patch description
    pub description: String,
    /// Whether this is a trampoline patch
    pub is_trampoline: bool,
}

impl Patch {
    /// Create a new patch
    pub fn new(address: u32, new_bytes: Vec<u8>, description: impl Into<String>) -> Self {
        Self {
            address,
            original_bytes: Vec::new(),
            new_bytes,
            description: description.into(),
            is_trampoline: false,
        }
    }

    /// Set original bytes for verification
    pub fn with_original_bytes(mut self, bytes: Vec<u8>) -> Self {
        self.original_bytes = bytes;
        self
    }

    /// Mark as trampoline patch
    pub fn as_trampoline(mut self) -> Self {
        self.is_trampoline = true;
        self
    }
}

/// Trampoline information
#[derive(Debug, Clone)]
pub struct Trampoline {
    /// Original address that was patched
    pub original_address: u32,
    /// Trampoline location
    pub trampoline_address: u32,
    /// Original bytes (saved before patch)
    pub saved_bytes: Vec<u8>,
    /// Code in the trampoline
    pub trampoline_code: Vec<u8>,
    /// Target address (where execution continues)
    pub target_address: u32,
}

/// Free space region in ROM
#[derive(Debug, Clone)]
pub struct FreeSpace {
    /// Start address
    pub start: u32,
    /// End address (exclusive)
    pub end: u32,
    /// Whether this space has been used
    pub used: bool,
}

/// Jump table modification
#[derive(Debug, Clone)]
pub struct JumpTableMod {
    /// Jump table address
    pub table_address: u32,
    /// Entry index to modify
    pub entry_index: usize,
    /// New target address
    pub new_target: u32,
    /// Entry size (2 or 3 bytes)
    pub entry_size: u8,
}

/// Code Patcher for ROM modifications
#[derive(Debug)]
pub struct CodePatcher {
    /// Patches to apply
    patches: Vec<Patch>,
    /// Trampolines created
    trampolines: Vec<Trampoline>,
    /// Jump table modifications
    jump_table_mods: Vec<JumpTableMod>,
    /// Free space regions
    free_space: Vec<FreeSpace>,
    /// Next free space index
    free_space_index: usize,
    /// ROM checksum info
    checksum_info: ChecksumInfo,
}

/// ROM checksum information
#[derive(Debug, Clone)]
pub struct ChecksumInfo {
    /// Checksum complement address
    pub complement_addr: u32,
    /// Checksum value address
    pub checksum_addr: u32,
}

impl Default for ChecksumInfo {
    fn default() -> Self {
        // Standard SNES header checksum locations
        Self {
            complement_addr: 0x7FDC,
            checksum_addr: 0x7FDE,
        }
    }
}

impl CodePatcher {
    /// Create a new code patcher
    pub fn new() -> Self {
        Self {
            patches: Vec::new(),
            trampolines: Vec::new(),
            jump_table_mods: Vec::new(),
            free_space: Vec::new(),
            free_space_index: 0,
            checksum_info: ChecksumInfo::default(),
        }
    }

    /// Create a new patcher with custom checksum info
    pub fn with_checksum_info(mut self, info: ChecksumInfo) -> Self {
        self.checksum_info = info;
        self
    }

    /// Add a free space region
    pub fn add_free_space(&mut self, start: u32, end: u32) {
        self.free_space.push(FreeSpace {
            start,
            end,
            used: false,
        });
        // Sort by address
        self.free_space.sort_by_key(|f| f.start);
    }

    /// Insert a patch at a specific address
    pub fn insert_patch(&mut self, address: u32, code: &[u8], description: impl Into<String>) -> Result<()> {
        if code.is_empty() {
            return Err(AssemblyError::InvalidSyntax(
                "Empty patch code".to_string(),
            ));
        }

        let patch = Patch::new(address, code.to_vec(), description);
        self.patches.push(patch);

        Ok(())
    }

    /// Create a trampoline to redirect execution
    pub fn create_trampoline(
        &mut self,
        original_address: u32,
        new_code: &[u8],
        continue_address: Option<u32>,
        rom: &Rom,
    ) -> Result<Trampoline> {
        // Find free space for the trampoline
        let trampoline_addr = self.find_free_space(new_code.len() + 5)?;

        // Read original bytes
        let original_bytes = self.read_bytes(rom, original_address, 5)?;

        // Build trampoline code
        let mut trampoline_code = new_code.to_vec();

        // Add code to return to original flow
        if let Some(cont) = continue_address {
            // JMP to continue address
            trampoline_code.push(0x5C); // JML (long jump)
            trampoline_code.push((cont & 0xFF) as u8);
            trampoline_code.push(((cont >> 8) & 0xFF) as u8);
            trampoline_code.push(((cont >> 16) & 0xFF) as u8);
        } else {
            // Execute original instruction(s) first, then return
            trampoline_code.extend_from_slice(&original_bytes);
            
            // Calculate return address (original + overwritten bytes)
            let return_addr = original_address + original_bytes.len() as u32;
            trampoline_code.push(0x5C); // JML
            trampoline_code.push((return_addr & 0xFF) as u8);
            trampoline_code.push(((return_addr >> 8) & 0xFF) as u8);
            trampoline_code.push(((return_addr >> 16) & 0xFF) as u8);
        }

        // Find actual free space with correct size
        let actual_addr = self.find_free_space(trampoline_code.len())?;
        self.mark_space_used(actual_addr, trampoline_code.len());

        // Create patch for original location (JML to trampoline)
        let mut redirect = vec![0x5C]; // JML
        redirect.push((actual_addr & 0xFF) as u8);
        redirect.push(((actual_addr >> 8) & 0xFF) as u8);
        redirect.push(((actual_addr >> 16) & 0xFF) as u8);

        let patch = Patch::new(original_address, redirect, "Trampoline redirect")
            .with_original_bytes(original_bytes.clone())
            .as_trampoline();
        self.patches.push(patch);

        // Create trampoline info
        let trampoline = Trampoline {
            original_address,
            trampoline_address: actual_addr,
            saved_bytes: original_bytes,
            trampoline_code,
            target_address: continue_address.unwrap_or(original_address + 5),
        };

        self.trampolines.push(trampoline.clone());

        Ok(trampoline)
    }

    /// Create a trampoline with hook (call your code then continue)
    pub fn create_hook(
        &mut self,
        hook_address: u32,
        hook_code: &[u8],
        rom: &Rom,
    ) -> Result<Trampoline> {
        // JSL to hook code, then continue original flow
        let continue_addr = hook_address + 4; // Size of JSL instruction
        
        self.create_trampoline(hook_address, hook_code, Some(continue_addr), rom)
    }

    /// Modify a jump table entry
    pub fn modify_jump_table(
        &mut self,
        table_address: u32,
        entry_index: usize,
        new_target: u32,
        entry_size: u8,
    ) -> Result<()> {
        if entry_size != 2 && entry_size != 3 {
            return Err(AssemblyError::InvalidSyntax(
                "Jump table entry size must be 2 or 3".to_string(),
            ));
        }

        let mod_info = JumpTableMod {
            table_address,
            entry_index,
            new_target,
            entry_size,
        };

        self.jump_table_mods.push(mod_info);
        Ok(())
    }

    /// Replace an entire jump table
    pub fn replace_jump_table(
        &mut self,
        table_address: u32,
        new_targets: &[u32],
        entry_size: u8,
    ) -> Result<()> {
        for (i, &target) in new_targets.iter().enumerate() {
            self.modify_jump_table(table_address, i, target, entry_size)?;
        }
        Ok(())
    }

    /// Apply all patches to a ROM
    pub fn apply(&self, rom: &mut Rom) -> Result<()> {
        // Verify ROM is large enough
        for patch in &self.patches {
            let end_addr = patch.address + patch.new_bytes.len() as u32;
            if end_addr > rom.len() as u32 {
                return Err(AssemblyError::AddressOutOfRange(end_addr));
            }
        }

        // Verify original bytes match (if specified)
        for patch in &self.patches {
            if !patch.original_bytes.is_empty() {
                let current = self.read_bytes(rom, patch.address, patch.original_bytes.len())?;
                if current != patch.original_bytes {
                    return Err(AssemblyError::RomError(format!(
                        "Bytes at ${:06X} don't match expected values",
                        patch.address
                    )));
                }
            }
        }

        // Apply trampoline code first
        for trampoline in &self.trampolines {
            rom.write_bytes(trampoline.trampoline_address, &trampoline.trampoline_code)
                .map_err(|e| AssemblyError::RomError(e.to_string()))?;
        }

        // Apply main patches
        for patch in &self.patches {
            rom.write_bytes(patch.address, &patch.new_bytes)
                .map_err(|e| AssemblyError::RomError(e.to_string()))?;
        }

        // Apply jump table modifications
        for mod_info in &self.jump_table_mods {
            self.apply_jump_table_mod(rom, mod_info)?;
        }

        // Fix checksum
        self.fix_checksum(rom)?;

        Ok(())
    }

    /// Apply a single jump table modification
    fn apply_jump_table_mod(&self, rom: &mut Rom, mod_info: &JumpTableMod) -> Result<()> {
        let entry_addr = mod_info.table_address + (mod_info.entry_index as u32 * mod_info.entry_size as u32);
        
        let bytes = if mod_info.entry_size == 2 {
            vec![
                (mod_info.new_target & 0xFF) as u8,
                ((mod_info.new_target >> 8) & 0xFF) as u8,
            ]
        } else {
            vec![
                (mod_info.new_target & 0xFF) as u8,
                ((mod_info.new_target >> 8) & 0xFF) as u8,
                ((mod_info.new_target >> 16) & 0xFF) as u8,
            ]
        };

        rom.write_bytes(entry_addr, &bytes)
            .map_err(|e| AssemblyError::RomError(e.to_string()))?;

        Ok(())
    }

    /// Fix ROM checksum
    pub fn fix_checksum(&self, rom: &mut Rom) -> Result<()> {
        let checksum = self.calculate_checksum(rom);
        let complement = checksum ^ 0xFFFF;

        // Write checksum complement
        rom.write_byte(self.checksum_info.complement_addr, (complement & 0xFF) as u8)
            .map_err(|e| AssemblyError::RomError(e.to_string()))?;
        rom.write_byte(self.checksum_info.complement_addr + 1, ((complement >> 8) & 0xFF) as u8)
            .map_err(|e| AssemblyError::RomError(e.to_string()))?;

        // Write checksum
        rom.write_byte(self.checksum_info.checksum_addr, (checksum & 0xFF) as u8)
            .map_err(|e| AssemblyError::RomError(e.to_string()))?;
        rom.write_byte(self.checksum_info.checksum_addr + 1, ((checksum >> 8) & 0xFF) as u8)
            .map_err(|e| AssemblyError::RomError(e.to_string()))?;

        Ok(())
    }

    /// Calculate ROM checksum
    fn calculate_checksum(&self, rom: &Rom) -> u16 {
        let mut sum: u32 = 0;
        
        // Sum all bytes except checksum bytes themselves
        for i in 0..rom.len() {
            let addr = i as u32;
            // Skip checksum bytes
            if addr == self.checksum_info.complement_addr 
                || addr == self.checksum_info.complement_addr + 1
                || addr == self.checksum_info.checksum_addr
                || addr == self.checksum_info.checksum_addr + 1 {
                continue;
            }
            
            if let Ok(byte) = rom.read_byte(addr) {
                sum = sum.wrapping_add(byte as u32);
            }
        }

        (sum & 0xFFFF) as u16
    }

    /// Get all patches
    pub fn patches(&self) -> &[Patch] {
        &self.patches
    }

    /// Get all trampolines
    pub fn trampolines(&self) -> &[Trampoline] {
        &self.trampolines
    }

    /// Get all jump table modifications
    pub fn jump_table_mods(&self) -> &[JumpTableMod] {
        &self.jump_table_mods
    }

    /// Get free space regions
    pub fn free_space(&self) -> &[FreeSpace] {
        &self.free_space
    }

    /// Generate a patch report
    pub fn generate_report(&self) -> String {
        let mut report = String::new();

        report.push_str("CODE PATCH REPORT\n");
        report.push_str("=================\n\n");

        report.push_str(&format!("Patches: {}\n", self.patches.len()));
        report.push_str(&format!("Trampolines: {}\n", self.trampolines.len()));
        report.push_str(&format!("Jump Table Modifications: {}\n\n", self.jump_table_mods.len()));

        report.push_str("PATCHES:\n");
        for (i, patch) in self.patches.iter().enumerate() {
            report.push_str(&format!("\n[Patch {}]\n", i + 1));
            report.push_str(&format!("  Address: ${:06X}\n", patch.address));
            report.push_str(&format!("  Description: {}\n", patch.description));
            report.push_str(&format!("  Size: {} bytes\n", patch.new_bytes.len()));
            report.push_str(&format!(
                "  Type: {}\n",
                if patch.is_trampoline {
                    "Trampoline"
                } else {
                    "Direct"
                }
            ));

            let hex_bytes: Vec<String> = patch
                .new_bytes
                .iter()
                .map(|b| format!("{:02X}", b))
                .collect();
            report.push_str(&format!("  Bytes: {}\n", hex_bytes.join(" ")));
        }

        if !self.trampolines.is_empty() {
            report.push_str("\nTRAMPOLINES:\n");
            for (i, trampoline) in self.trampolines.iter().enumerate() {
                report.push_str(&format!("\n[Trampoline {}]\n", i + 1));
                report.push_str(&format!(
                    "  Original Address: ${:06X}\n",
                    trampoline.original_address
                ));
                report.push_str(&format!(
                    "  Trampoline Address: ${:06X}\n",
                    trampoline.trampoline_address
                ));
                report.push_str(&format!(
                    "  Size: {} bytes\n",
                    trampoline.trampoline_code.len()
                ));
            }
        }

        if !self.jump_table_mods.is_empty() {
            report.push_str("\nJUMP TABLE MODIFICATIONS:\n");
            for (i, mod_info) in self.jump_table_mods.iter().enumerate() {
                report.push_str(&format!("\n[Modification {}]\n", i + 1));
                report.push_str(&format!("  Table Address: ${:06X}\n", mod_info.table_address));
                report.push_str(&format!("  Entry Index: {}\n", mod_info.entry_index));
                report.push_str(&format!(
                    "  New Target: ${:06X}\n",
                    mod_info.new_target
                ));
            }
        }

        report
    }

    /// Read bytes from ROM
    fn read_bytes(&self, rom: &Rom, address: u32, len: usize) -> Result<Vec<u8>> {
        let mut bytes = Vec::with_capacity(len);
        for i in 0..len {
            let byte = rom
                .read_byte(address + i as u32)
                .map_err(|e| AssemblyError::RomError(e.to_string()))?;
            bytes.push(byte);
        }
        Ok(bytes)
    }

    /// Find free space of at least the given size
    fn find_free_space(&self, size: usize) -> Result<u32> {
        for space in &self.free_space {
            if !space.used && (space.end - space.start) >= size as u32 {
                return Ok(space.start);
            }
        }

        Err(AssemblyError::NoTrampolineSpace)
    }

    /// Mark free space as used
    fn mark_space_used(&mut self, addr: u32, size: usize) {
        for space in &mut self.free_space {
            if space.start == addr {
                space.used = true;
                // If there's leftover space, we could split it
                // For now, just mark the whole region used
                break;
            }
        }
    }
}

impl Default for CodePatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper to create a simple byte patch
pub fn create_byte_patch(address: u32, bytes: &[u8]) -> Patch {
    Patch::new(address, bytes.to_vec(), "Byte patch")
}

/// Helper to create a single byte patch
pub fn create_single_byte_patch(address: u32, byte: u8) -> Patch {
    Patch::new(address, vec![byte], "Single byte patch")
}

/// Builder for complex patches
#[derive(Debug)]
pub struct PatchBuilder {
    address: u32,
    bytes: Vec<u8>,
    description: String,
}

impl PatchBuilder {
    /// Create a new patch builder
    pub fn new(address: u32) -> Self {
        Self {
            address,
            bytes: Vec::new(),
            description: String::new(),
        }
    }

    /// Add NOP instructions
    pub fn nops(mut self, count: usize) -> Self {
        for _ in 0..count {
            self.bytes.push(0xEA); // NOP
        }
        self
    }

    /// Add a JMP instruction
    pub fn jmp(mut self, target: u32) -> Self {
        self.bytes.push(0x4C); // JMP absolute
        self.bytes.push((target & 0xFF) as u8);
        self.bytes.push(((target >> 8) & 0xFF) as u8);
        self
    }

    /// Add a JML (long jump) instruction
    pub fn jml(mut self, target: u32) -> Self {
        self.bytes.push(0x5C); // JML absolute long
        self.bytes.push((target & 0xFF) as u8);
        self.bytes.push(((target >> 8) & 0xFF) as u8);
        self.bytes.push(((target >> 16) & 0xFF) as u8);
        self
    }

    /// Add a JSR instruction
    pub fn jsr(mut self, target: u32) -> Self {
        self.bytes.push(0x20); // JSR absolute
        self.bytes.push((target & 0xFF) as u8);
        self.bytes.push(((target >> 8) & 0xFF) as u8);
        self
    }

    /// Add a JSL (long subroutine call) instruction
    pub fn jsl(mut self, target: u32) -> Self {
        self.bytes.push(0x22); // JSL absolute long
        self.bytes.push((target & 0xFF) as u8);
        self.bytes.push(((target >> 8) & 0xFF) as u8);
        self.bytes.push(((target >> 16) & 0xFF) as u8);
        self
    }

    /// Add RTS instruction
    pub fn rts(mut self) -> Self {
        self.bytes.push(0x60); // RTS
        self
    }

    /// Add RTL instruction
    pub fn rtl(mut self) -> Self {
        self.bytes.push(0x6B); // RTL
        self
    }

    /// Add raw bytes
    pub fn bytes(mut self, bytes: &[u8]) -> Self {
        self.bytes.extend_from_slice(bytes);
        self
    }

    /// Add a single byte
    pub fn byte(mut self, byte: u8) -> Self {
        self.bytes.push(byte);
        self
    }

    /// Set description
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Build the patch
    pub fn build(self) -> Patch {
        Patch::new(self.address, self.bytes, self.description)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_rom() -> Rom {
        let data = vec![0u8; 0x10000];
        Rom::from_bytes(&data).unwrap()
    }

    #[test]
    fn test_patch_new() {
        let patch = Patch::new(0x8000, vec![0xEA, 0x60], "Test patch");
        assert_eq!(patch.address, 0x8000);
        assert_eq!(patch.new_bytes, vec![0xEA, 0x60]);
        assert_eq!(patch.description, "Test patch");
    }

    #[test]
    fn test_patch_builder() {
        let patch = PatchBuilder::new(0x8000)
            .nops(1)
            .rts()
            .description("NOP and return")
            .build();
        
        assert_eq!(patch.address, 0x8000);
        assert_eq!(patch.new_bytes, vec![0xEA, 0x60]);
    }

    #[test]
    fn test_patch_builder_jmp() {
        let patch = PatchBuilder::new(0x8000)
            .jmp(0x9000)
            .build();
        
        assert_eq!(patch.new_bytes, vec![0x4C, 0x00, 0x90]);
    }

    #[test]
    fn test_patch_builder_jml() {
        let patch = PatchBuilder::new(0x8000)
            .jml(0x123456)
            .build();
        
        assert_eq!(patch.new_bytes, vec![0x5C, 0x56, 0x34, 0x12]);
    }

    #[test]
    fn test_code_patcher_add_free_space() {
        let mut patcher = CodePatcher::new();
        patcher.add_free_space(0x10000, 0x11000);
        
        assert_eq!(patcher.free_space().len(), 1);
        assert_eq!(patcher.free_space()[0].start, 0x10000);
        assert_eq!(patcher.free_space()[0].end, 0x11000);
    }

    #[test]
    fn test_insert_patch() {
        let mut patcher = CodePatcher::new();
        patcher.insert_patch(0x8000, &[0xEA, 0x60], "Test").unwrap();
        
        assert_eq!(patcher.patches().len(), 1);
        assert_eq!(patcher.patches()[0].address, 0x8000);
    }

    #[test]
    fn test_modify_jump_table() {
        let mut patcher = CodePatcher::new();
        patcher.modify_jump_table(0x9000, 0, 0x8000, 2).unwrap();
        
        assert_eq!(patcher.jump_table_mods().len(), 1);
        assert_eq!(patcher.jump_table_mods()[0].entry_index, 0);
        assert_eq!(patcher.jump_table_mods()[0].new_target, 0x8000);
    }

    #[test]
    fn test_jump_table_invalid_size() {
        let mut patcher = CodePatcher::new();
        let result = patcher.modify_jump_table(0x9000, 0, 0x8000, 4);
        assert!(result.is_err());
    }

    #[test]
    fn test_calculate_checksum() {
        let patcher = CodePatcher::new();
        let data = vec![0u8; 0x8000];
        let rom = Rom::from_bytes(&data).unwrap();
        
        let checksum = patcher.calculate_checksum(&rom);
        assert_eq!(checksum, 0); // All zeros
    }

    #[test]
    fn test_generate_report() {
        let mut patcher = CodePatcher::new();
        patcher.insert_patch(0x8000, &[0xEA], "NOP patch").unwrap();
        
        let report = patcher.generate_report();
        assert!(report.contains("CODE PATCH REPORT"));
        assert!(report.contains("NOP patch"));
        assert!(report.contains("$008000"));
    }
}
