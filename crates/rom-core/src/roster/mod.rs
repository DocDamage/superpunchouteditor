//! Roster metadata management for Super Punch-Out!!
//!
//! This module handles game-level roster data including:
//! - Boxer names (with custom text encoding)
//! - Circuit assignments (Minor, Major, World, Special)
//! - Unlock order progression
//! - Introductory text
//!
//! # ROM Structure
//!
//! Based on Super Punch-Out!! (USA) ROM research:
//!
//! ## Boxer Name Table (Bank $0C)
//! - Boxer names are stored in a compressed/encoded format
//! - Each name is null-terminated (0xFF)
//! - Names are accessed via a pointer table
//!
//! ## Address Constants
//! These addresses are in PC offset format (after removing SMC header if present)

mod constants;
mod layout;
mod loader;
mod types;
mod writer;

// Re-export all public types from submodules
pub use constants::*;
pub use layout::*;
pub use loader::RosterLoader;
pub use types::*;
pub use writer::RosterWriter;

/// Intro text entry (legacy alias for compatibility)
pub type IntroText = BoxerIntro;

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Rom, TextEncoder};

    fn build_expanded_roster_rom() -> Rom {
        let mut rom = Rom::new(vec![0; 0x20_0000]);
        let header_pc = 0x1F0000usize;
        let boxer_count = 24usize;
        let name_ptr = 0x1F0100usize;
        let long_ptr = 0x1F0140usize;
        let name_blob = 0x1F0200usize;
        let circuit = 0x1F0600usize;
        let unlock = 0x1F0620usize;
        let intro = 0x1F0640usize;

        let mut header = vec![0u8; 46];
        header[..8].copy_from_slice(b"SPOEDITR");
        header[8] = 2;
        header[10..12].copy_from_slice(&(boxer_count as u16).to_le_bytes());
        header[12..16].copy_from_slice(&(name_ptr as u32).to_le_bytes());
        header[16..20].copy_from_slice(&(long_ptr as u32).to_le_bytes());
        header[20..24].copy_from_slice(&(name_blob as u32).to_le_bytes());
        header[24..28].copy_from_slice(&(circuit as u32).to_le_bytes());
        header[28..32].copy_from_slice(&(unlock as u32).to_le_bytes());
        header[32..36].copy_from_slice(&(intro as u32).to_le_bytes());
        rom.write_bytes(header_pc, &header).expect("write expansion header");

        let encoder = TextEncoder::new();
        for boxer_id in 0..boxer_count {
            let slot_pc = name_blob + boxer_id * 16;
            let default_name = format!("BOXER {}", boxer_id + 1);
            let encoded = encoder.encode_with_terminator(&default_name);
            rom.write_bytes(slot_pc, &encoded).expect("write boxer name");

            let (bank, addr) = rom.pc_to_snes(slot_pc);
            let [lo, hi] = addr.to_le_bytes();
            rom.write_bytes(name_ptr + boxer_id * 2, &[lo, hi])
                .expect("write short pointer");
            rom.write_bytes(long_ptr + boxer_id * 3, &[bank, lo, hi])
                .expect("write long pointer");
        }

        let special_name_pc = name_blob + 20 * 16;
        let special_name = encoder.encode_with_terminator("ROBOT ACE");
        rom.write_bytes(special_name_pc, &special_name)
            .expect("write special boxer name");

        let circuit_bytes = vec![CircuitType::Minor.to_byte(); boxer_count];
        rom.write_bytes(circuit, &circuit_bytes)
            .expect("write circuit table");
        rom.write_bytes(circuit + 20, &[CircuitType::World.to_byte()])
            .expect("write expanded boxer circuit");

        let unlock_bytes: Vec<u8> = (0..boxer_count as u8).collect();
        rom.write_bytes(unlock, &unlock_bytes)
            .expect("write unlock table");
        rom.write_bytes(unlock + 20, &[21])
            .expect("write expanded boxer unlock");

        rom
    }

    #[test]
    fn test_circuit_type_display() {
        assert_eq!(CircuitType::Minor.display_name(), "Minor Circuit");
        assert_eq!(CircuitType::Special.display_name(), "Special Circuit");
    }

    #[test]
    fn test_circuit_type_number() {
        assert_eq!(CircuitType::Minor.number(), 1);
        assert_eq!(CircuitType::Special.number(), 4);
    }

    #[test]
    fn test_circuit_type_from_byte() {
        assert_eq!(CircuitType::from_byte(0), CircuitType::Minor);
        assert_eq!(CircuitType::from_byte(1), CircuitType::Major);
        assert_eq!(CircuitType::from_byte(2), CircuitType::World);
        assert_eq!(CircuitType::from_byte(3), CircuitType::Special);
    }

    #[test]
    fn test_text_encoder_a_z() {
        let encoder = TextEncoder::new();

        // Test A-Z encoding
        let text = "ABC";
        let encoded = encoder.encode_with_terminator(text);
        assert_eq!(encoded[0], 0x00); // A
        assert_eq!(encoded[1], 0x01); // B
        assert_eq!(encoded[2], 0x02); // C
        assert_eq!(encoded[3], 0xFF); // Terminator

        let decoded = encoder.decode(&encoded[..encoded.len() - 1]);
        assert_eq!(decoded, text);
    }

    #[test]
    fn test_text_encoder_numbers() {
        let encoder = TextEncoder::new();

        let text = "012";
        let encoded = encoder.encode(text);
        assert_eq!(encoded[0], 0x1A); // 0
        assert_eq!(encoded[1], 0x1B); // 1
        assert_eq!(encoded[2], 0x1C); // 2
    }

    #[test]
    fn test_text_encoder_special_chars() {
        let encoder = TextEncoder::new();

        let text = "!?.";
        let encoded = encoder.encode(text);
        assert_eq!(encoded[0], 0x25); // !
        assert_eq!(encoded[1], 0x26); // ?
        assert_eq!(encoded[2], 0x27); // .
    }

    #[test]
    fn test_text_encoder_validation() {
        let encoder = TextEncoder::new();

        // Valid text
        assert!(encoder.validate("HELLO WORLD").is_ok());

        // Invalid text with lowercase (not supported)
        assert!(encoder.validate("hello").is_err());

        // Invalid text with unsupported character
        assert!(encoder.validate("HELLO@WORLD").is_ok()); // @ is supported
        assert!(encoder.validate("HELLO~WORLD").is_err()); // ~ is not supported
    }

    #[test]
    fn test_text_encoder_fixed_length() {
        let encoder = TextEncoder::new();

        let text = "HI";
        let encoded = encoder.encode_fixed(text, 5);
        assert_eq!(encoded.len(), 5);
        assert_eq!(encoded[0], 0x07); // H
        assert_eq!(encoded[1], 0x08); // I
        assert_eq!(encoded[2], 0x24); // Space
        assert_eq!(encoded[3], 0x24); // Space
        assert_eq!(encoded[4], 0x24); // Space
    }

    #[test]
    fn test_roster_default_data() {
        let roster = RosterData::new();
        assert_eq!(roster.boxers.len(), BOXER_COUNT);
        assert_eq!(roster.circuits.len(), 4);
    }

    #[test]
    fn test_roster_get_boxer() {
        let roster = RosterData::new();
        let boxer = roster.get_boxer(0);
        assert!(boxer.is_some());
        assert_eq!(boxer.unwrap().name, "Gabby Jay");
    }

    #[test]
    fn test_roster_update_circuit() {
        let mut roster = RosterData::new();
        roster.update_boxer_circuit(0, CircuitType::Major).unwrap();

        let boxer = roster.get_boxer(0).unwrap();
        assert_eq!(boxer.circuit, CircuitType::Major);
    }

    #[test]
    fn test_roster_validation() {
        let roster = RosterData::new();
        let report = roster.validate();

        // Should have warnings about missing champion flags
        assert!(!report.warnings.is_empty());
        // But no errors
        assert!(report.errors.is_empty());
    }

    #[test]
    fn test_name_too_long() {
        let mut roster = RosterData::new();
        let long_name = "A".repeat(MAX_NAME_LENGTH + 10);
        let result = roster.update_boxer_name(0, &long_name);

        assert!(result.is_err());
        match result {
            Err(RosterError::NameTooLong { .. }) => (),
            _ => panic!("Expected NameTooLong error"),
        }
    }

    #[test]
    fn test_cornerman_condition_roundtrip() {
        for i in 0..8u8 {
            let condition = CornermanCondition::from_byte(i);
            assert_eq!(condition.to_byte(), i);
        }
    }

    #[test]
    fn test_get_boxer_key() {
        assert_eq!(get_boxer_key(0), "gabby_jay");
        assert_eq!(get_boxer_key(15), "nick_bruiser");
        assert_eq!(get_boxer_key(255), "unknown");
    }

    #[test]
    fn test_get_boxer_id_from_key() {
        assert_eq!(get_boxer_id_from_key("gabby_jay"), Some(0));
        assert_eq!(get_boxer_id_from_key("nick_bruiser"), Some(15));
        assert_eq!(get_boxer_id_from_key("unknown"), None);
    }

    #[test]
    fn test_address_constants() {
        // Verify address calculations
        assert_eq!(BANK_0C_BASE, 0x060000);
        assert_eq!(BOXER_NAME_TABLE, 0x060000);
        assert_eq!(BOXER_NAME_POINTERS, 0x060100);
        assert_eq!(CIRCUIT_TABLE, 0x060200);
        assert_eq!(UNLOCK_ORDER_TABLE, 0x060300);
        assert_eq!(BOXER_INTRO_TABLE, 0x060400);
    }

    #[test]
    fn test_loader_reads_expanded_roster_layout() {
        let rom = build_expanded_roster_rom();
        let loader = RosterLoader::new(&rom);
        let roster = loader.load_roster().expect("expanded roster should load");

        assert_eq!(roster.boxers.len(), 24);
        let boxer = roster.get_boxer(20).expect("expanded boxer entry should exist");
        assert_eq!(boxer.name, "ROBOT ACE");
        assert_eq!(boxer.circuit, CircuitType::World);
        assert_eq!(boxer.unlock_order, 21);
    }

    #[test]
    fn test_writer_updates_expanded_roster_tables() {
        let mut rom = build_expanded_roster_rom();
        {
            let mut writer = RosterWriter::new(&mut rom);
            writer
                .write_boxer_name(20, "IRONBOT")
                .expect("expanded boxer name should write");
            writer
                .write_circuit_assignment(20, CircuitType::Special)
                .expect("expanded boxer circuit should write");
            writer
                .write_unlock_order(20, 30)
                .expect("expanded boxer unlock should write");
        }

        let loader = RosterLoader::new(&rom);
        let roster = loader.load_roster().expect("expanded roster should reload");
        let boxer = roster.get_boxer(20).expect("expanded boxer entry should exist");
        assert_eq!(boxer.name, "IRONBOT");
        assert_eq!(boxer.circuit, CircuitType::Special);
        assert_eq!(boxer.unlock_order, 30);
    }

    /// Build a minimal vanilla ROM with intro data initialized for all 16 boxers.
    fn build_intro_rom() -> crate::Rom {
        // Cover BOXER_INTRO_TABLE + all boxer intros
        let size = BOXER_INTRO_TABLE + INTRO_DATA_SIZE * BOXER_COUNT + 1024;
        let mut rom = crate::Rom::new(vec![0u8; size]);
        let encoder = TextEncoder::new();
        for boxer_id in 0..BOXER_COUNT {
            let base = BOXER_INTRO_TABLE + boxer_id * INTRO_DATA_SIZE;
            let name = encoder.encode_fixed(&format!("BOXER {}", boxer_id + 1), INTRO_FIELD_SIZE);
            rom.write_bytes(base, &name).expect("write intro name");
            let pad = encoder.encode_fixed("USA", INTRO_FIELD_SIZE);
            rom.write_bytes(base + INTRO_FIELD_SIZE, &pad).expect("write intro origin");
            let pad = encoder.encode_fixed("10-0-0", INTRO_FIELD_SIZE);
            rom.write_bytes(base + INTRO_FIELD_SIZE * 2, &pad).expect("write intro record");
            let pad = encoder.encode_fixed("1", INTRO_FIELD_SIZE);
            rom.write_bytes(base + INTRO_FIELD_SIZE * 3, &pad).expect("write intro rank");
            let pad = encoder.encode_fixed("HI", INTRO_FIELD_SIZE);
            rom.write_bytes(base + INTRO_FIELD_SIZE * 4, &pad).expect("write intro quote");
        }
        rom
    }

    #[test]
    fn test_write_boxer_intro_field_round_trip() {
        let mut rom = build_intro_rom();
        {
            let mut writer = RosterWriter::new(&mut rom);
            writer.write_boxer_intro_field(0, 0, "GARY JAY").expect("name write");
            writer.write_boxer_intro_field(0, 1, "CANADA").expect("origin write");
        }
        let loader = RosterLoader::new(&rom);
        let intro = loader.load_boxer_intro(0).expect("intro should load");
        assert_eq!(intro.name_text.trim(), "GARY JAY");
        assert_eq!(intro.origin_text.trim(), "CANADA");
    }

    #[test]
    fn test_write_boxer_intro_field_long_text_truncates() {
        // write_boxer_intro_field uses encode_fixed which silently truncates
        // to INTRO_FIELD_SIZE bytes. Text longer than 16 chars is accepted.
        let mut rom = build_intro_rom();
        {
            let mut writer = RosterWriter::new(&mut rom);
            let long = "A".repeat(INTRO_FIELD_SIZE + 4);
            writer.write_boxer_intro_field(0, 0, &long).expect("should succeed via truncation");
        }
        let loader = RosterLoader::new(&rom);
        let intro = loader.load_boxer_intro(0).expect("intro should load");
        // The decoded field should not exceed the field size
        assert!(intro.name_text.trim().len() <= INTRO_FIELD_SIZE);
    }

    /// Build a minimal ROM with one cornerman text entry for boxer 0.
    fn build_cornerman_test_rom() -> crate::Rom {
        let size = 0x070000;
        let mut rom = crate::Rom::new(vec![0u8; size]);

        let data_pc: usize = 0x063200;
        let text_pc: usize = 0x063230;

        // data_snes_offset: snes_to_pc(0x0C, addr) = data_pc
        // => addr = (data_pc - 0x60000) | 0x8000
        let data_snes: u16 = ((data_pc - BANK_0C_BASE) as u16) | 0x8000;
        let [dlo, dhi] = data_snes.to_le_bytes();
        rom.write_bytes(CORNERMAN_POINTER_TABLE, &[dlo, dhi]).expect("pointer");

        let text_snes: u16 = ((text_pc - BANK_0C_BASE) as u16) | 0x8000;
        let [tlo, thi] = text_snes.to_le_bytes();
        // data block: count=1, entry: condition=0, text_ptr
        rom.write_bytes(data_pc, &[1, 0, tlo, thi]).expect("data block");

        let encoder = TextEncoder::new();
        let mut text_bytes = encoder.encode("GOOD LUCK");
        text_bytes.push(0xFF);
        rom.write_bytes(text_pc, &text_bytes).expect("text");

        rom
    }

    #[test]
    fn test_write_cornerman_text_round_trip() {
        let mut rom = build_cornerman_test_rom();
        {
            let mut writer = RosterWriter::new(&mut rom);
            writer
                .write_cornerman_text(0, 0, "NICE JOB", None)
                .expect("cornerman write should succeed");
        }
        let loader = RosterLoader::new(&rom);
        let texts = loader
            .load_cornerman_texts(0)
            .expect("cornerman load should succeed");
        assert_eq!(texts.len(), 1);
        assert_eq!(texts[0].text, "NICE JOB");
    }

    #[test]
    fn test_write_cornerman_text_too_long_is_err() {
        let mut rom = build_cornerman_test_rom();
        let mut writer = RosterWriter::new(&mut rom);
        // Original text "GOOD LUCK" is 9 bytes. This 20-byte text must fail.
        let result = writer.write_cornerman_text(0, 0, "THIS TEXT IS TOO LONG", None);
        assert!(result.is_err());
    }
}
