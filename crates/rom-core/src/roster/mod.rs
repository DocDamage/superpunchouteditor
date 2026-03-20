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
mod loader;
mod types;
mod writer;

// Re-export all public types from submodules
pub use constants::*;
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
        let encoded = encoder.encode(text);
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

        let text = "123";
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
}
