use crate::text::TextEncoder;

use super::constants::*;
use super::types::*;

// ============================================================================
// ROSTER LOADER
// ============================================================================

/// Roster loader from ROM
pub struct RosterLoader<'a> {
    rom: &'a crate::Rom,
    encoder: TextEncoder,
}

impl<'a> RosterLoader<'a> {
    /// Create a new roster loader
    pub fn new(rom: &'a crate::Rom) -> Self {
        Self {
            rom,
            encoder: TextEncoder::new(),
        }
    }

    /// Load complete roster data from ROM
    pub fn load_roster(&self) -> Result<RosterData, RosterError> {
        let mut data = RosterData::new();

        // Load boxer names
        for id in 0..BOXER_COUNT as u8 {
            if let Ok(name) = self.load_boxer_name(id) {
                if let Some(boxer) = data.get_boxer_mut(id) {
                    boxer.name = name;
                }
            }
        }

        // Load circuit assignments
        for id in 0..BOXER_COUNT as u8 {
            if let Ok(circuit) = self.load_circuit_assignment(id) {
                if let Some(boxer) = data.get_boxer_mut(id) {
                    boxer.circuit = circuit;
                }
            }
        }

        // Load unlock orders
        for id in 0..BOXER_COUNT as u8 {
            if let Ok(order) = self.load_unlock_order(id) {
                if let Some(boxer) = data.get_boxer_mut(id) {
                    boxer.unlock_order = order;
                    boxer.is_unlockable = order > 0;
                }
            }
        }

        // Update circuit boxer lists
        for circuit in &mut data.circuits {
            circuit.boxers = data
                .boxers
                .iter()
                .filter(|b| b.circuit == circuit.circuit_type)
                .map(|b| b.fighter_id)
                .collect();
        }

        Ok(data)
    }

    /// Load a boxer's name from ROM
    pub fn load_boxer_name(&self, boxer_id: u8) -> Result<String, RosterError> {
        if boxer_id as usize >= BOXER_COUNT {
            return Err(RosterError::InvalidFighterId(boxer_id));
        }

        // Read pointer from pointer table
        let ptr_addr = BOXER_NAME_POINTERS + (boxer_id as usize * 2);
        let ptr_bytes = self.rom.read_bytes(ptr_addr, 2).map_err(|_| {
            RosterError::AddressNotFound(format!("Name pointer at 0x{:06X}", ptr_addr))
        })?;

        let name_offset = u16::from_le_bytes([ptr_bytes[0], ptr_bytes[1]]) as usize;
        let name_pc = self.snes_to_pc(0x0C, name_offset as u16);

        // Read name bytes until terminator (0xFF)
        let mut bytes = Vec::new();
        let mut addr = name_pc;

        loop {
            let byte = self.rom.read_bytes(addr, 1).map_err(|_| {
                RosterError::AddressNotFound(format!("Name data at 0x{:06X}", addr))
            })?[0];

            if byte == 0xFF {
                break;
            }

            bytes.push(byte);
            addr += 1;

            // Safety limit
            if bytes.len() > MAX_NAME_LENGTH {
                break;
            }
        }

        Ok(self.encoder.decode(&bytes))
    }

    /// Load a boxer's intro data from ROM
    pub fn load_boxer_intro(&self, boxer_id: u8) -> Result<BoxerIntro, RosterError> {
        if boxer_id as usize >= BOXER_COUNT {
            return Err(RosterError::InvalidFighterId(boxer_id));
        }

        let base = BOXER_INTRO_TABLE + (boxer_id as usize * INTRO_DATA_SIZE);

        let name_offset = base;
        let origin_offset = base + INTRO_FIELD_SIZE;
        let record_offset = base + INTRO_FIELD_SIZE * 2;
        let rank_offset = base + INTRO_FIELD_SIZE * 3;
        let quote_offset = base + INTRO_FIELD_SIZE * 4;

        let boxer_key = get_boxer_key(boxer_id);

        Ok(BoxerIntro {
            boxer_key: boxer_key.to_string(),
            name_text: self.read_text_at(name_offset, INTRO_FIELD_SIZE)?,
            origin_text: self.read_text_at(origin_offset, INTRO_FIELD_SIZE)?,
            record_text: self.read_text_at(record_offset, INTRO_FIELD_SIZE)?,
            rank_text: self.read_text_at(rank_offset, INTRO_FIELD_SIZE)?,
            intro_quote: self.read_text_at(quote_offset, INTRO_FIELD_SIZE)?,
            name_offset: Some(name_offset),
            origin_offset: Some(origin_offset),
            record_offset: Some(record_offset),
            rank_offset: Some(rank_offset),
            quote_offset: Some(quote_offset),
        })
    }

    /// Load circuit assignment for a boxer
    pub fn load_circuit_assignment(&self, boxer_id: u8) -> Result<CircuitType, RosterError> {
        if boxer_id as usize >= BOXER_COUNT {
            return Err(RosterError::InvalidFighterId(boxer_id));
        }

        let byte = self
            .rom
            .read_bytes(CIRCUIT_TABLE + boxer_id as usize, 1)
            .map_err(|_| RosterError::AddressNotFound("Circuit table".to_string()))?[0];

        Ok(CircuitType::from_byte(byte))
    }

    /// Load unlock order for a boxer
    pub fn load_unlock_order(&self, boxer_id: u8) -> Result<u8, RosterError> {
        if boxer_id as usize >= BOXER_COUNT {
            return Err(RosterError::InvalidFighterId(boxer_id));
        }

        let byte = self
            .rom
            .read_bytes(UNLOCK_ORDER_TABLE + boxer_id as usize, 1)
            .map_err(|_| RosterError::AddressNotFound("Unlock order table".to_string()))?[0];

        Ok(byte)
    }

    /// Load cornerman texts for a boxer
    pub fn load_cornerman_texts(&self, boxer_id: u8) -> Result<Vec<CornermanText>, RosterError> {
        if boxer_id as usize >= BOXER_COUNT {
            return Err(RosterError::InvalidFighterId(boxer_id));
        }

        // Read pointer from cornerman pointer table
        let ptr_addr = CORNERMAN_POINTER_TABLE + (boxer_id as usize * 2);
        let ptr_bytes = self
            .rom
            .read_bytes(ptr_addr, 2)
            .map_err(|_| RosterError::AddressNotFound("Cornerman pointer table".to_string()))?;

        let data_offset = u16::from_le_bytes([ptr_bytes[0], ptr_bytes[1]]) as usize;
        let data_pc = self.snes_to_pc(0x0C, data_offset as u16);

        // Read entry count
        let count = self
            .rom
            .read_bytes(data_pc, 1)
            .map_err(|_| RosterError::AddressNotFound("Cornerman data".to_string()))?[0];

        let mut texts = Vec::new();
        let boxer_key = get_boxer_key(boxer_id);

        for i in 0..count {
            let entry_addr = data_pc + 1 + (i as usize * 3);

            let condition_byte = self
                .rom
                .read_bytes(entry_addr, 1)
                .map_err(|_| RosterError::AddressNotFound("Cornerman entry".to_string()))?[0];

            let text_ptr_bytes = self
                .rom
                .read_bytes(entry_addr + 1, 2)
                .map_err(|_| RosterError::AddressNotFound("Cornerman text pointer".to_string()))?;

            let text_offset = u16::from_le_bytes([text_ptr_bytes[0], text_ptr_bytes[1]]);
            let text_pc = self.snes_to_pc(0x0C, text_offset);

            texts.push(CornermanText {
                id: i,
                boxer_key: boxer_key.to_string(),
                round: 0,
                condition: CornermanCondition::from_byte(condition_byte),
                text: self.read_null_terminated_text(text_pc)?,
                raw_bytes: Vec::new(),
                rom_offset: Some(text_pc),
                max_length: 40,
            });
        }

        Ok(texts)
    }

    /// Load victory quotes for a boxer
    pub fn load_victory_quotes(&self, boxer_id: u8) -> Result<Vec<VictoryQuote>, RosterError> {
        if boxer_id as usize >= BOXER_COUNT {
            return Err(RosterError::InvalidFighterId(boxer_id));
        }

        // Read pointer from victory quote table
        let ptr_addr = VICTORY_QUOTE_TABLE + (boxer_id as usize * 2);
        let ptr_bytes = self
            .rom
            .read_bytes(ptr_addr, 2)
            .map_err(|_| RosterError::AddressNotFound("Victory quote pointer table".to_string()))?;

        let data_offset = u16::from_le_bytes([ptr_bytes[0], ptr_bytes[1]]) as usize;
        let data_pc = self.snes_to_pc(0x0C, data_offset as u16);

        // Read entry count
        let count = self
            .rom
            .read_bytes(data_pc, 1)
            .map_err(|_| RosterError::AddressNotFound("Victory quote data".to_string()))?[0];

        let mut quotes = Vec::new();
        let boxer_key = get_boxer_key(boxer_id);

        for i in 0..count {
            let entry_addr = data_pc + 1 + (i as usize * 3);

            let flags = self
                .rom
                .read_bytes(entry_addr, 1)
                .map_err(|_| RosterError::AddressNotFound("Victory quote entry".to_string()))?[0];

            let text_ptr_bytes = self.rom.read_bytes(entry_addr + 1, 2).map_err(|_| {
                RosterError::AddressNotFound("Victory quote text pointer".to_string())
            })?;

            let text_offset = u16::from_le_bytes([text_ptr_bytes[0], text_ptr_bytes[1]]);
            let text_pc = self.snes_to_pc(0x0C, text_offset);

            quotes.push(VictoryQuote {
                id: i,
                boxer_key: boxer_key.to_string(),
                text: self.read_null_terminated_text(text_pc)?,
                is_loss_quote: (flags & 0x80) != 0,
                raw_bytes: Vec::new(),
                rom_offset: Some(text_pc),
                max_length: 50,
            });
        }

        Ok(quotes)
    }

    // Helper: Read text at a fixed offset
    fn read_text_at(&self, offset: usize, max_len: usize) -> Result<String, RosterError> {
        let bytes = self
            .rom
            .read_bytes(offset, max_len)
            .map_err(|_| RosterError::AddressNotFound(format!("Text at 0x{:06X}", offset)))?;
        Ok(self.encoder.decode_fixed(bytes))
    }

    // Helper: Read null-terminated text
    fn read_null_terminated_text(&self, start_offset: usize) -> Result<String, RosterError> {
        let mut bytes = Vec::new();
        let mut addr = start_offset;

        loop {
            let byte = self
                .rom
                .read_bytes(addr, 1)
                .map_err(|_| RosterError::AddressNotFound(format!("Text at 0x{:06X}", addr)))?[0];

            if byte == 0xFF {
                break;
            }

            bytes.push(byte);
            addr += 1;

            // Safety limit
            if bytes.len() > 256 {
                break;
            }
        }

        Ok(self.encoder.decode(&bytes))
    }

    // Helper: Convert SNES address to PC offset
    fn snes_to_pc(&self, bank: u8, addr: u16) -> usize {
        ((bank as usize & 0x7F) * 0x8000) | (addr as usize & 0x7FFF)
    }
}

// ============================================================================
// LEGACY COMPATIBILITY
// ============================================================================

/// Legacy RosterLoader impl for compatibility
impl RosterLoader<'_> {
    /// Load roster data from ROM (static method for compatibility)
    pub fn from_rom(rom: &crate::Rom) -> Result<RosterData, RosterError> {
        let loader = RosterLoader::new(rom);
        loader.load_roster()
    }

    /// Load a specific boxer's name from ROM (static method for compatibility)
    pub fn load_boxer_name_static(rom: &crate::Rom, fighter_id: u8) -> Result<String, RosterError> {
        let loader = RosterLoader::new(rom);
        loader.load_boxer_name(fighter_id)
    }

    /// Load intro text for a boxer (static method for compatibility)
    pub fn load_intro_text(rom: &crate::Rom, boxer_id: u8) -> Result<BoxerIntro, RosterError> {
        let loader = RosterLoader::new(rom);
        loader.load_boxer_intro(boxer_id)
    }
}
