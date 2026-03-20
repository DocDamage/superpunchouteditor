use crate::text::TextEncoder;

use super::constants::*;
use super::types::*;

// ============================================================================
// ROSTER WRITER
// ============================================================================

/// Roster writer to ROM
pub struct RosterWriter<'a> {
    rom: &'a mut crate::Rom,
    encoder: TextEncoder,
}

impl<'a> RosterWriter<'a> {
    /// Create a new roster writer
    pub fn new(rom: &'a mut crate::Rom) -> Self {
        Self {
            rom,
            encoder: TextEncoder::new(),
        }
    }

    /// Write a boxer's name to ROM
    pub fn write_boxer_name(&mut self, boxer_id: u8, new_name: &str) -> Result<(), RosterError> {
        if boxer_id as usize >= BOXER_COUNT {
            return Err(RosterError::InvalidFighterId(boxer_id));
        }

        // Validate characters
        self.encoder.validate(new_name).map_err(|invalid| {
            RosterError::EncodingError(format!("Invalid characters: {:?}", invalid))
        })?;

        // Encode name
        let encoded = self.encoder.encode(new_name);

        // Read current pointer to find name location
        let ptr_addr = BOXER_NAME_POINTERS + (boxer_id as usize * 2);
        let ptr_bytes = self
            .rom
            .read_bytes(ptr_addr, 2)
            .map_err(|_| RosterError::AddressNotFound("Name pointer".to_string()))?;

        let name_offset = u16::from_le_bytes([ptr_bytes[0], ptr_bytes[1]]) as usize;
        let name_pc = self.snes_to_pc(0x0C, name_offset as u16);

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

        // Write encoded name
        self.rom
            .write_bytes(name_pc, &encoded)
            .map_err(|_| RosterError::AddressNotFound("Name write".to_string()))?;

        // Pad with spaces if shorter (up to but not including the terminator we just wrote)
        let new_name_len = encoded.len() - 1; // Exclude terminator
        if new_name_len < current_len {
            for i in new_name_len..current_len {
                self.rom
                    .write_bytes(name_pc + i, &[0x24]) // Space
                    .map_err(|_| RosterError::AddressNotFound("Name padding".to_string()))?;
            }
            // Write terminator after padding
            self.rom
                .write_bytes(name_pc + current_len, &[0xFF])
                .map_err(|_| RosterError::AddressNotFound("Name terminator".to_string()))?;
        }

        Ok(())
    }

    /// Write circuit assignment
    pub fn write_circuit_assignment(
        &mut self,
        boxer_id: u8,
        circuit: CircuitType,
    ) -> Result<(), RosterError> {
        if boxer_id as usize >= BOXER_COUNT {
            return Err(RosterError::InvalidFighterId(boxer_id));
        }

        self.rom
            .write_bytes(CIRCUIT_TABLE + boxer_id as usize, &[circuit.to_byte()])
            .map_err(|_| RosterError::AddressNotFound("Circuit table".to_string()))?;

        Ok(())
    }

    /// Write unlock order
    pub fn write_unlock_order(&mut self, boxer_id: u8, order: u8) -> Result<(), RosterError> {
        if boxer_id as usize >= BOXER_COUNT {
            return Err(RosterError::InvalidFighterId(boxer_id));
        }

        self.rom
            .write_bytes(UNLOCK_ORDER_TABLE + boxer_id as usize, &[order])
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
        if boxer_id as usize >= BOXER_COUNT {
            return Err(RosterError::InvalidFighterId(boxer_id));
        }
        if field_index as usize >= INTRO_FIELD_COUNT {
            return Err(RosterError::EncodingError(
                "Invalid field index".to_string(),
            ));
        }

        let base = BOXER_INTRO_TABLE + (boxer_id as usize * INTRO_DATA_SIZE);
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

    // Helper: Convert SNES address to PC offset
    fn snes_to_pc(&self, bank: u8, addr: u16) -> usize {
        ((bank as usize & 0x7F) * 0x8000) | (addr as usize & 0x7FFF)
    }
}
