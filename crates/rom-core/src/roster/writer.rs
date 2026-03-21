use crate::text::TextEncoder;

use super::constants::*;
use super::layout::{detect_roster_layout, RosterLayout};
use super::types::*;

// ============================================================================
// ROSTER WRITER
// ============================================================================

/// Roster writer to ROM
pub struct RosterWriter<'a> {
    rom: &'a mut crate::Rom,
    encoder: TextEncoder,
    layout: RosterLayout,
}

impl<'a> RosterWriter<'a> {
    /// Create a new roster writer
    pub fn new(rom: &'a mut crate::Rom) -> Self {
        let layout = detect_roster_layout(rom);
        Self {
            rom,
            encoder: TextEncoder::new(),
            layout,
        }
    }

    /// Write a boxer's name to ROM
    pub fn write_boxer_name(&mut self, boxer_id: u8, new_name: &str) -> Result<(), RosterError> {
        if boxer_id as usize >= self.layout.boxer_count {
            return Err(RosterError::InvalidFighterId(boxer_id));
        }

        // Validate characters
        self.encoder.validate(new_name).map_err(|invalid| {
            RosterError::EncodingError(format!("Invalid characters: {:?}", invalid))
        })?;

        // Encode with terminator so fit checks and padding preserve the full name.
        let encoded = self.encoder.encode_with_terminator(new_name);

        let name_pc = self.resolve_name_pc(boxer_id)?;

        // Find current length
        let mut current_len = 0usize;
        let mut addr = name_pc;

        loop {
            let byte = self
                .rom
                .read_bytes(addr, 1)
                .map_err(|_| RosterError::AddressNotFound("Name data".to_string()))?[0];
            if byte == 0xFF {
                break;
            }
            current_len += 1;
            addr += 1;

            if current_len > MAX_NAME_LENGTH * 2 {
                return Err(RosterError::EncodingError(
                    "Name field too long".to_string(),
                ));
            }
        }

        // Check if new name fits
        if encoded.len() > current_len + 1 {
            // +1 for terminator
            return Err(RosterError::NameTooLong {
                name: new_name.to_string(),
                max_bytes: current_len,
                actual_bytes: encoded.len(),
            });
        }

        // Write the variable-length name plus terminator. Any bytes after the
        // terminator are ignored by the loader, so no pre-terminator padding is needed.
        self.rom
            .write_bytes(name_pc, &encoded)
            .map_err(|_| RosterError::AddressNotFound("Name write".to_string()))?;

        Ok(())
    }

    /// Write circuit assignment
    pub fn write_circuit_assignment(
        &mut self,
        boxer_id: u8,
        circuit: CircuitType,
    ) -> Result<(), RosterError> {
        if boxer_id as usize >= self.layout.boxer_count {
            return Err(RosterError::InvalidFighterId(boxer_id));
        }

        self.rom
            .write_bytes(self.layout.circuit_table_pc + boxer_id as usize, &[circuit.to_byte()])
            .map_err(|_| RosterError::AddressNotFound("Circuit table".to_string()))?;

        Ok(())
    }

    /// Write unlock order
    pub fn write_unlock_order(&mut self, boxer_id: u8, order: u8) -> Result<(), RosterError> {
        if boxer_id as usize >= self.layout.boxer_count {
            return Err(RosterError::InvalidFighterId(boxer_id));
        }

        self.rom
            .write_bytes(self.layout.unlock_table_pc + boxer_id as usize, &[order])
            .map_err(|_| RosterError::AddressNotFound("Unlock order table".to_string()))?;

        Ok(())
    }

    /// Write boxer intro field
    pub fn write_boxer_intro_field(
        &mut self,
        boxer_id: u8,
        field_index: u8,
        text: &str,
    ) -> Result<(), RosterError> {
        if boxer_id as usize >= self.layout.boxer_count {
            return Err(RosterError::InvalidFighterId(boxer_id));
        }
        if field_index as usize >= INTRO_FIELD_COUNT {
            return Err(RosterError::EncodingError(
                "Invalid field index".to_string(),
            ));
        }

        let base = self.layout.intro_table_pc + (boxer_id as usize * INTRO_DATA_SIZE);
        let field_offset = base + (field_index as usize * INTRO_FIELD_SIZE);

        // Validate and encode
        self.encoder.validate(text).map_err(|invalid| {
            RosterError::EncodingError(format!("Invalid characters: {:?}", invalid))
        })?;

        let encoded = self.encoder.encode_fixed(text, INTRO_FIELD_SIZE);

        self.rom
            .write_bytes(field_offset, &encoded)
            .map_err(|_| RosterError::AddressNotFound("Intro field".to_string()))?;

        Ok(())
    }

    /// Write cornerman text for a specific entry
    ///
    /// Updates the text string in ROM at the entry's text pointer.
    /// Optionally updates the condition byte.
    ///
    /// The new text must not be longer than the original allocated text (in-place write).
    pub fn write_cornerman_text(
        &mut self,
        boxer_id: u8,
        text_id: u8,
        new_text: &str,
        new_condition: Option<u8>,
    ) -> Result<(), RosterError> {
        if boxer_id as usize >= self.layout.boxer_count {
            return Err(RosterError::InvalidFighterId(boxer_id));
        }

        // Follow the boxer's cornerman pointer to find the data block
        let ptr_addr = CORNERMAN_POINTER_TABLE + (boxer_id as usize * 2);
        let ptr_bytes = self
            .rom
            .read_bytes(ptr_addr, 2)
            .map_err(|_| RosterError::AddressNotFound("Cornerman pointer table".to_string()))?;
        let data_offset = u16::from_le_bytes([ptr_bytes[0], ptr_bytes[1]]);
        let data_pc = self.snes_to_pc(0x0C, data_offset);

        // Verify text_id is within the stored count
        let count = self
            .rom
            .read_bytes(data_pc, 1)
            .map_err(|_| RosterError::AddressNotFound("Cornerman count byte".to_string()))?[0];

        if text_id >= count {
            return Err(RosterError::EncodingError(format!(
                "Cornerman text ID {} out of range (count: {})",
                text_id, count
            )));
        }

        let entry_addr = data_pc + 1 + (text_id as usize * 3);

        // Optionally update the condition byte (first byte of the 3-byte entry)
        if let Some(condition) = new_condition {
            self.rom
                .write_bytes(entry_addr, &[condition])
                .map_err(|_| RosterError::AddressNotFound("Cornerman condition byte".to_string()))?;
        }

        // Read the text pointer (bytes 1-2 of the entry)
        let text_ptr_bytes = self
            .rom
            .read_bytes(entry_addr + 1, 2)
            .map_err(|_| RosterError::AddressNotFound("Cornerman text pointer".to_string()))?;
        let text_offset = u16::from_le_bytes([text_ptr_bytes[0], text_ptr_bytes[1]]);
        let text_pc = self.snes_to_pc(0x0C, text_offset);

        // Validate the new text encoding
        self.encoder.validate(new_text).map_err(|invalid| {
            RosterError::EncodingError(format!("Invalid characters: {:?}", invalid))
        })?;

        let encoded = self.encoder.encode(new_text);

        // Measure the current allocated space (bytes until 0xFF terminator)
        let mut orig_len = 0usize;
        let mut scan = text_pc;
        loop {
            let byte = self
                .rom
                .read_bytes(scan, 1)
                .map_err(|_| RosterError::AddressNotFound("Cornerman text scan".to_string()))?[0];
            if byte == 0xFF {
                break;
            }
            orig_len += 1;
            scan += 1;
            if orig_len > MAX_CORNERMAN_SCAN_LEN {
                return Err(RosterError::EncodingError(
                    "Cornerman text field overrun during scan".to_string(),
                ));
            }
        }

        // New text cannot exceed the original allocated space
        if encoded.len() > orig_len {
            return Err(RosterError::EncodingError(format!(
                "New text is {} bytes but original is {} bytes; cannot extend in-place",
                encoded.len(),
                orig_len
            )));
        }

        // Write encoded text + 0xFF null terminator
        let mut write_bytes = encoded;
        write_bytes.push(0xFF);
        self.rom
            .write_bytes(text_pc, &write_bytes)
            .map_err(|_| RosterError::AddressNotFound("Cornerman text write".to_string()))?;

        Ok(())
    }

    // Helper: Convert SNES address to PC offset
    fn snes_to_pc(&self, bank: u8, addr: u16) -> usize {
        ((bank as usize & 0x7F) * 0x8000) | (addr as usize & 0x7FFF)
    }

    fn resolve_name_pc(&self, boxer_id: u8) -> Result<usize, RosterError> {
        let boxer_index = boxer_id as usize;

        if self.layout.expanded {
            let ptr_addr = self.layout.name_long_pointer_table_pc + (boxer_index * 3);
            let ptr_bytes = self.rom.read_bytes(ptr_addr, 3).map_err(|_| {
                RosterError::AddressNotFound("Long name pointer".to_string())
            })?;
            let bank = ptr_bytes[0];
            let addr = u16::from_le_bytes([ptr_bytes[1], ptr_bytes[2]]);
            if bank != 0 && addr >= 0x8000 {
                return Ok(self.rom.snes_to_pc(bank, addr));
            }
        }

        let ptr_addr = self.layout.name_pointer_table_pc + (boxer_index * 2);
        let ptr_bytes = self
            .rom
            .read_bytes(ptr_addr, 2)
            .map_err(|_| RosterError::AddressNotFound("Name pointer".to_string()))?;
        let addr = u16::from_le_bytes([ptr_bytes[0], ptr_bytes[1]]);

        let bank = if self.layout.expanded {
            self.rom.pc_to_snes(self.layout.name_blob_pc).0
        } else {
            0x0C
        };
        Ok(self.snes_to_pc(bank, addr))
    }
}
