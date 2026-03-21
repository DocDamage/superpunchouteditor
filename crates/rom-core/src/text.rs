//! Text and Dialog Editor for Super Punch-Out!!
//!
//! This module handles all in-game text including:
//! - Cornerman advice texts
//! - Boxer intros (name, origin, record, rank, quote)
//! - Victory/defeat quotes
//! - Menu text
//! - Credits text
//!
//! # ROM Text Implementation
//!
//! ## Text Pointer Tables
//! Text is typically stored with a pointer table that points to each string.
//! Format: 3 bytes per pointer (bank + address)
//! - Byte 0: Bank (0x80-0xFF for LoROM)
//! - Bytes 1-2: Address within bank
//!
//! ## Text Locations (see roster.rs for actual addresses)
//! - Cornerman texts: Implemented
//! - Boxer intro data: Implemented
//! - Victory quotes: Implemented
//! - Menu text: Implemented
//! - Credits: Implemented
//!
//! ## Text Encoding
//! Uses SPO's custom encoding (see TextEncoder for implementation)
//! - A-Z: Standard uppercase letters
//! - Space, punctuation
//! - Control codes for line breaks, end of string

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::Rom;

/// Maximum lengths for different text types (in bytes)
pub const MAX_CORNERMAN_TEXT_LENGTH: usize = 40;
pub const MAX_INTRO_NAME_LENGTH: usize = 16;
pub const MAX_INTRO_ORIGIN_LENGTH: usize = 32;
pub const MAX_INTRO_RECORD_LENGTH: usize = 20;
pub const MAX_INTRO_RANK_LENGTH: usize = 24;
pub const MAX_VICTORY_QUOTE_LENGTH: usize = 50;
pub const MAX_MENU_TEXT_LENGTH: usize = 20;
pub const MAX_CREDITS_LINE_LENGTH: usize = 32;

/// Control codes for text formatting
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum TextControlCode {
    /// End of string
    EndOfString = 0x00,
    /// Line break
    LineBreak = 0x01,
    /// Wait for button press
    WaitForInput = 0x02,
    /// Clear text box
    ClearText = 0x03,
    /// Change text color (followed by color id)
    ChangeColor = 0x04,
    /// Unknown/Custom code
    Unknown(u8),
}

impl TextControlCode {
    /// Convert a byte to a control code
    pub fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0x00 => Some(Self::EndOfString),
            0x01 => Some(Self::LineBreak),
            0x02 => Some(Self::WaitForInput),
            0x03 => Some(Self::ClearText),
            0x04 => Some(Self::ChangeColor),
            0x05..=0x0F => Some(Self::Unknown(byte)), // Reserved range
            _ => None,
        }
    }

    /// Convert control code to display string
    pub fn display(&self) -> &'static str {
        match self {
            Self::EndOfString => "[END]",
            Self::LineBreak => "[BR]",
            Self::WaitForInput => "[WAIT]",
            Self::ClearText => "[CLR]",
            Self::ChangeColor => "[COLOR]",
            Self::Unknown(_) => "[?]",
        }
    }
}

// Re-export from roster.rs
pub use crate::roster::{
    BoxerIntro, CornermanCondition as TextCondition, CornermanText, VictoryQuote,
};

/// Victory condition type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VictoryCondition {
    /// Won by knockout
    Knockout,
    /// Won by decision
    Decision,
    /// Won by TKO
    Tko,
    /// Technical decision
    Technical,
}

impl VictoryCondition {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Knockout => "Knockout",
            Self::Decision => "Decision",
            Self::Tko => "TKO",
            Self::Technical => "Technical",
        }
    }

    pub fn all_conditions() -> Vec<Self> {
        vec![Self::Knockout, Self::Decision, Self::Tko, Self::Technical]
    }

    pub fn from_byte(byte: u8) -> Self {
        match byte {
            0x00 => Self::Knockout,
            0x01 => Self::Decision,
            0x02 => Self::Tko,
            _ => Self::Technical,
        }
    }

    pub fn to_byte(&self) -> u8 {
        match self {
            Self::Knockout => 0x00,
            Self::Decision => 0x01,
            Self::Tko => 0x02,
            Self::Technical => 0x03,
        }
    }
}

/// Menu category for organization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MenuCategory {
    /// Main menu text
    MainMenu,
    /// Options menu
    Options,
    /// Pause menu
    PauseMenu,
    /// Game over screen
    GameOver,
    /// Continue/Retry prompts
    ContinuePrompt,
    /// Profile selection
    Profile,
}

impl MenuCategory {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::MainMenu => "Main Menu",
            Self::Options => "Options",
            Self::PauseMenu => "Pause Menu",
            Self::GameOver => "Game Over",
            Self::ContinuePrompt => "Continue Prompt",
            Self::Profile => "Profile",
        }
    }

    pub fn all_categories() -> Vec<Self> {
        vec![
            Self::MainMenu,
            Self::Options,
            Self::PauseMenu,
            Self::GameOver,
            Self::ContinuePrompt,
            Self::Profile,
        ]
    }
}

// Re-export from roster.rs for convenience

/// Menu text entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuText {
    /// Unique ID
    pub id: String,
    /// Menu category
    pub category: MenuCategory,
    /// The text content
    pub text: String,
    /// Maximum allowed length
    pub max_length: usize,
    /// Original text (for reset)
    pub original_text: String,
    /// ROM offset
    pub rom_offset: Option<usize>,
    /// Whether this text is shared across multiple locations
    pub is_shared: bool,
}

impl MenuText {
    pub fn new(id: impl Into<String>, category: MenuCategory, text: impl Into<String>) -> Self {
        let text = text.into();
        Self {
            id: id.into(),
            category,
            original_text: text.clone(),
            text,
            max_length: MAX_MENU_TEXT_LENGTH,
            rom_offset: None,
            is_shared: false,
        }
    }

    pub fn validate(&self, encoder: &TextEncoder) -> Result<(), TextError> {
        let encoded = encoder.encode(&self.text);

        if encoded.len() > self.max_length {
            return Err(TextError::TextTooLong {
                text: self.text.clone(),
                max_bytes: self.max_length,
                actual_bytes: encoded.len(),
            });
        }

        if !encoder.can_encode(&self.text) {
            return Err(TextError::UnsupportedCharacters(self.text.clone()));
        }

        Ok(())
    }

    /// Reset to original text
    pub fn reset(&mut self) {
        self.text = self.original_text.clone();
    }

    /// Check if text has been modified
    pub fn is_modified(&self) -> bool {
        self.text != self.original_text
    }
}

/// Credits line entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditsLine {
    /// Line number (for ordering)
    pub line_number: u16,
    /// The text content
    pub text: String,
    /// Whether this is a title/header line
    pub is_title: bool,
    /// ROM offset
    pub rom_offset: Option<usize>,
    /// Maximum length
    pub max_length: usize,
}

impl CreditsLine {
    pub fn new(line_number: u16, text: impl Into<String>) -> Self {
        Self {
            line_number,
            text: text.into(),
            is_title: false,
            rom_offset: None,
            max_length: MAX_CREDITS_LINE_LENGTH,
        }
    }

    pub fn validate(&self, encoder: &TextEncoder) -> Result<(), TextError> {
        let encoded = encoder.encode(&self.text);

        if encoded.len() > self.max_length {
            return Err(TextError::TextTooLong {
                text: self.text.clone(),
                max_bytes: self.max_length,
                actual_bytes: encoded.len(),
            });
        }

        if !encoder.can_encode(&self.text) {
            return Err(TextError::UnsupportedCharacters(self.text.clone()));
        }

        Ok(())
    }
}

/// Complete text database for the game
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextDatabase {
    /// Cornerman advice texts, grouped by boxer
    pub cornerman_texts: Vec<CornermanText>,
    /// Boxer introduction data
    pub boxer_intros: Vec<BoxerIntro>,
    /// Victory and defeat quotes
    pub victory_quotes: Vec<VictoryQuote>,
    /// Menu text entries
    pub menu_texts: Vec<MenuText>,
    /// Credits text lines
    pub credits_text: Vec<CreditsLine>,
    /// Text encoder configuration
    pub encoder: TextEncoder,
}

impl TextDatabase {
    /// Create an empty text database
    pub fn new() -> Self {
        Self {
            cornerman_texts: Vec::new(),
            boxer_intros: Vec::new(),
            victory_quotes: Vec::new(),
            menu_texts: Vec::new(),
            credits_text: Vec::new(),
            encoder: TextEncoder::default(),
        }
    }

    /// Create with default placeholder data (for development)
    pub fn with_defaults() -> Self {
        let mut db = Self::new();
        db.load_defaults();
        db
    }

    /// Load default placeholder data
    fn load_defaults(&mut self) {
        // Add some sample cornerman texts
        self.cornerman_texts = vec![
            CornermanText::new(0, "gabby_jay", "Watch his left hook!"),
            CornermanText::new(1, "gabby_jay", "You can do it, Mac!"),
            CornermanText::new(2, "gabby_jay", "Keep your guard up!"),
        ];

        // Set conditions
        self.cornerman_texts[0].condition = TextCondition::StartOfRound;
        self.cornerman_texts[1].condition = TextCondition::PlayerLowHealth;
        self.cornerman_texts[2].condition = TextCondition::Random;

        // Add sample boxer intro
        let mut intro = BoxerIntro::new("gabby_jay");
        intro.name_text = "GABBY JAY".to_string();
        intro.origin_text = "From: Paris, France".to_string();
        intro.record_text = "Record: 1-99".to_string();
        intro.rank_text = "Rank: #1 Contender".to_string();
        intro.intro_quote = "Let me show you my technique!".to_string();
        self.boxer_intros.push(intro);

        // Add sample victory quotes
        self.victory_quotes = vec![
            VictoryQuote::new(0, "gabby_jay", "I did it! I won!"),
            VictoryQuote::new(1, "gabby_jay", "Finally, a victory!"),
        ];

        // Add sample menu texts
        self.menu_texts = vec![
            MenuText::new("main_start", MenuCategory::MainMenu, "START"),
            MenuText::new("main_options", MenuCategory::MainMenu, "OPTIONS"),
            MenuText::new("pause_continue", MenuCategory::PauseMenu, "CONTINUE"),
            MenuText::new("pause_quit", MenuCategory::PauseMenu, "QUIT"),
        ];
    }

    /// Get cornerman texts for a specific boxer
    pub fn get_cornerman_texts(&self, boxer_key: &str) -> Vec<&CornermanText> {
        self.cornerman_texts
            .iter()
            .filter(|t| t.boxer_key == boxer_key)
            .collect()
    }

    /// Get cornerman text by ID
    pub fn get_cornerman_text(&self, id: u8) -> Option<&CornermanText> {
        self.cornerman_texts.iter().find(|t| t.id == id)
    }

    /// Get mutable cornerman text by ID
    pub fn get_cornerman_text_mut(&mut self, id: u8) -> Option<&mut CornermanText> {
        self.cornerman_texts.iter_mut().find(|t| t.id == id)
    }

    /// Get boxer intro
    pub fn get_boxer_intro(&self, boxer_key: &str) -> Option<&BoxerIntro> {
        self.boxer_intros.iter().find(|i| i.boxer_key == boxer_key)
    }

    /// Get mutable boxer intro
    pub fn get_boxer_intro_mut(&mut self, boxer_key: &str) -> Option<&mut BoxerIntro> {
        self.boxer_intros
            .iter_mut()
            .find(|i| i.boxer_key == boxer_key)
    }

    /// Get victory quotes for a boxer
    pub fn get_victory_quotes(&self, boxer_key: &str) -> Vec<&VictoryQuote> {
        self.victory_quotes
            .iter()
            .filter(|q| q.boxer_key == boxer_key)
            .collect()
    }

    /// Get menu texts by category
    pub fn get_menu_texts(&self, category: MenuCategory) -> Vec<&MenuText> {
        self.menu_texts
            .iter()
            .filter(|t| t.category == category)
            .collect()
    }

    /// Add a new cornerman text
    pub fn add_cornerman_text(&mut self, text: CornermanText) {
        self.cornerman_texts.push(text);
    }

    /// Remove a cornerman text
    pub fn remove_cornerman_text(&mut self, id: u8) -> Option<CornermanText> {
        if let Some(pos) = self.cornerman_texts.iter().position(|t| t.id == id) {
            Some(self.cornerman_texts.remove(pos))
        } else {
            None
        }
    }

    /// Validate the entire database
    pub fn validate(&self) -> Vec<(String, TextError)> {
        let mut errors = Vec::new();
        let spo_encoder = TextEncoder::new();

        for text in &self.cornerman_texts {
            if let Err(e) = text.validate(&spo_encoder) {
                errors.push((
                    format!("cornerman_{}", text.id),
                    TextError::EncodingError(e),
                ));
            }
        }

        for intro in &self.boxer_intros {
            for (field, err) in intro.validate(&spo_encoder) {
                errors.push((
                    format!("intro_{}_{}", intro.boxer_key, field),
                    TextError::EncodingError(err),
                ));
            }
        }

        for quote in &self.victory_quotes {
            if let Err(e) = quote.validate(&spo_encoder) {
                errors.push((format!("quote_{}", quote.id), TextError::EncodingError(e)));
            }
        }

        for menu in &self.menu_texts {
            if let Err(e) = menu.validate(&self.encoder) {
                errors.push((format!("menu_{}", menu.id), e));
            }
        }

        errors
    }
}

impl Default for TextDatabase {
    fn default() -> Self {
        Self::new()
    }
}

/// Text encoder/decoder for SPO
/// This uses the same encoding as roster.rs but provides a struct interface
///
/// # SPO Character Encoding
/// - 0x00-0x19: A-Z
/// - 0x1A-0x23: 0-9
/// - 0x24: Space
/// - 0x25-0x3A: Special characters
/// - 0xFF: End of string marker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextEncoder {
    /// Character mappings (byte -> char)
    #[serde(skip)]
    char_map: std::collections::HashMap<char, u8>,
    /// Reverse mappings (byte -> char)  
    #[serde(skip)]
    reverse_map: std::collections::HashMap<u8, char>,
}

/// Type alias for backward compatibility with code using SpoTextEncoder
pub type SpoTextEncoder = TextEncoder;

impl TextEncoder {
    /// Create a new encoder with the complete SPO character mapping
    pub fn new() -> Self {
        use std::collections::HashMap;

        let mut char_map = HashMap::new();

        // A-Z: 0x00-0x19
        for (i, c) in ('A'..='Z').enumerate() {
            char_map.insert(c, i as u8);
        }

        // 0-9: 0x1A-0x23
        for (i, c) in ('0'..='9').enumerate() {
            char_map.insert(c, 0x1A + i as u8);
        }

        // Special characters
        char_map.insert(' ', 0x24); // Space
        char_map.insert('!', 0x25); // Exclamation
        char_map.insert('?', 0x26); // Question mark
        char_map.insert('.', 0x27); // Period
        char_map.insert(',', 0x28); // Comma
        char_map.insert('\'', 0x29); // Apostrophe
        char_map.insert('-', 0x2A); // Hyphen
        char_map.insert('&', 0x2B); // Ampersand
        char_map.insert('(', 0x2C); // Open parenthesis
        char_map.insert(')', 0x2D); // Close parenthesis
        char_map.insert(':', 0x2E); // Colon
        char_map.insert(';', 0x2F); // Semicolon
        char_map.insert('*', 0x30); // Asterisk (star)
        char_map.insert('/', 0x31); // Forward slash
        char_map.insert('#', 0x32); // Hash/Number sign
        char_map.insert('%', 0x33); // Percent
        char_map.insert('@', 0x34); // At sign
        char_map.insert('+', 0x35); // Plus
        char_map.insert('=', 0x36); // Equals
        char_map.insert('<', 0x37); // Less than
        char_map.insert('>', 0x38); // Greater than
        char_map.insert('[', 0x39); // Open bracket
        char_map.insert(']', 0x3A); // Close bracket

        // Build reverse map
        let reverse_map = char_map.iter().map(|(&k, &v)| (v, k)).collect();

        Self {
            char_map,
            reverse_map,
        }
    }

    /// Encode text to ROM bytes
    ///
    /// Does NOT add terminator - use `encode_with_terminator` for that
    pub fn encode(&self, text: &str) -> Vec<u8> {
        text.chars()
            .map(|c| *self.char_map.get(&c).unwrap_or(&0x24)) // Default to space
            .collect()
    }

    /// Encode text with 0xFF terminator
    pub fn encode_with_terminator(&self, text: &str) -> Vec<u8> {
        let mut result = self.encode(text);
        result.push(0xFF);
        result
    }

    /// Encode to fixed-length field (padded with spaces)
    pub fn encode_fixed(&self, text: &str, max_len: usize) -> Vec<u8> {
        let mut result: Vec<u8> = text
            .chars()
            .map(|c| *self.char_map.get(&c).unwrap_or(&0x24))
            .take(max_len)
            .collect();

        // Pad with spaces if needed
        while result.len() < max_len {
            result.push(0x24);
        }

        result
    }

    /// Decode ROM bytes to text (stops at 0xFF terminator)
    pub fn decode(&self, bytes: &[u8]) -> String {
        bytes
            .iter()
            .take_while(|&&b| b != 0xFF)
            .map(|&b| *self.reverse_map.get(&b).unwrap_or(&'?'))
            .collect()
    }

    /// Decode fixed-length field (no terminator expected)
    pub fn decode_fixed(&self, bytes: &[u8]) -> String {
        bytes
            .iter()
            .map(|&b| *self.reverse_map.get(&b).unwrap_or(&'?'))
            .collect::<String>()
            .trim_end()
            .to_string()
    }

    /// Check if text can be encoded
    pub fn can_encode(&self, text: &str) -> bool {
        text.chars().all(|c| self.char_map.contains_key(&c))
    }

    /// Validate text encoding
    pub fn validate(&self, text: &str) -> Result<(), Vec<char>> {
        let invalid: Vec<char> = text
            .chars()
            .filter(|c| !self.char_map.contains_key(c))
            .collect();

        if invalid.is_empty() {
            Ok(())
        } else {
            Err(invalid)
        }
    }

    /// Get supported characters in text (instance method)
    pub fn supported_chars_in_text(&self, text: &str) -> Vec<char> {
        text.chars()
            .filter(|ch| self.char_map.contains_key(ch))
            .collect()
    }

    /// Get unsupported characters in text
    pub fn get_unsupported_chars(&self, text: &str) -> Vec<char> {
        text.chars()
            .filter(|ch| !self.char_map.contains_key(ch))
            .collect()
    }

    /// Get byte value for character
    pub fn char_to_byte(&self, ch: char) -> Option<u8> {
        self.char_map.get(&ch).copied()
    }

    /// Get character for byte value
    pub fn byte_to_char(&self, byte: u8) -> Option<char> {
        self.reverse_map.get(&byte).copied()
    }
}

impl TextEncoder {
    /// Get the list of all supported characters (instance method)
    pub fn supported_chars(&self) -> Vec<char> {
        let mut chars: Vec<char> = self.char_map.keys().copied().collect();
        chars.sort();
        chars
    }
    
    /// Get the list of all supported characters (static method for convenience)
    pub fn all_supported_chars() -> Vec<char> {
        Self::new().supported_chars()
    }
}

impl Default for TextEncoder {
    fn default() -> Self {
        Self::new()
    }
}

/// Text pointer table entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextPointer {
    /// Entry ID
    pub id: u8,
    /// Bank number
    pub bank: u8,
    /// Address within bank
    pub address: u16,
    /// Computed PC offset
    pub pc_offset: Option<usize>,
}

impl TextPointer {
    /// Calculate PC offset from SNES address
    pub fn to_pc_offset(&self) -> Option<usize> {
        // LoROM: PC = (Bank & 0x7F) * 0x8000 + (Addr & 0x7FFF)
        let bank = self.bank as usize;
        let addr = self.address as usize;
        Some(((bank & 0x7F) * 0x8000) | (addr & 0x7FFF))
    }

    /// Create from PC offset
    pub fn from_pc_offset(id: u8, pc_offset: usize) -> Self {
        // LoROM: Bank = (PC / 0x8000) | 0x80, Addr = (PC % 0x8000) | 0x8000
        let bank = ((pc_offset / 0x8000) | 0x80) as u8;
        let addr = ((pc_offset % 0x8000) | 0x8000) as u16;

        Self {
            id,
            bank,
            address: addr,
            pc_offset: Some(pc_offset),
        }
    }
}

/// Text pointer table
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextPointerTable {
    /// Base address of the pointer table
    pub base_address: usize,
    /// Table entries
    pub entries: Vec<TextPointer>,
    /// Table type/name
    pub table_type: TextTableType,
}

/// Type of text table
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextTableType {
    Cornerman,
    VictoryQuotes,
    MenuText,
    Credits,
    Custom(String),
}

impl TextTableType {
    pub fn display_name(&self) -> String {
        match self {
            Self::Cornerman => "Cornerman Texts".to_string(),
            Self::VictoryQuotes => "Victory Quotes".to_string(),
            Self::MenuText => "Menu Text".to_string(),
            Self::Credits => "Credits".to_string(),
            Self::Custom(s) => s.to_string(),
        }
    }
}

impl TextPointerTable {
    /// Create a new empty table
    pub fn new(base_address: usize, table_type: TextTableType) -> Self {
        Self {
            base_address,
            entries: Vec::new(),
            table_type,
        }
    }

    /// Load pointer table from ROM
    ///
    /// # Arguments
    /// * `rom` - The ROM data
    /// * `base_address` - PC offset where pointer table starts
    /// * `entry_count` - Number of entries in the table
    /// * `entry_size` - Size of each entry in bytes (usually 3: bank + addr)
    ///
    /// # Returns
    /// The loaded pointer table or an error
    pub fn load_from_rom(
        rom: &Rom,
        base_address: usize,
        entry_count: usize,
        table_type: TextTableType,
    ) -> Result<Self, TextError> {
        let mut entries = Vec::with_capacity(entry_count);

        for i in 0..entry_count {
            let offset = base_address + (i * 3); // 3 bytes per entry

            let bytes = rom
                .read_bytes(offset, 3)
                .map_err(|_| TextError::InvalidPointerOffset(offset))?;

            let bank = bytes[0];
            let address = u16::from_le_bytes([bytes[1], bytes[2]]);

            entries.push(TextPointer {
                id: i as u8,
                bank,
                address,
                pc_offset: None, // Will be calculated on demand
            });
        }

        Ok(Self {
            base_address,
            entries,
            table_type,
        })
    }

    /// Get pointer by ID
    pub fn get(&self, id: u8) -> Option<&TextPointer> {
        self.entries.iter().find(|e| e.id == id)
    }

    /// Get mutable pointer by ID
    pub fn get_mut(&mut self, id: u8) -> Option<&mut TextPointer> {
        self.entries.iter_mut().find(|e| e.id == id)
    }

    /// Add a new entry
    pub fn add_entry(&mut self, bank: u8, address: u16) -> u8 {
        let id = self.entries.len() as u8;
        self.entries.push(TextPointer {
            id,
            bank,
            address,
            pc_offset: None,
        });
        id
    }

    /// Convert table to bytes for ROM writing
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(self.entries.len() * 3);

        for entry in &self.entries {
            result.push(entry.bank);
            result.extend_from_slice(&entry.address.to_le_bytes());
        }

        result
    }
}

/// Errors that can occur when working with text data
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
pub enum TextError {
    #[error("Text too long: {actual_bytes} bytes (max {max_bytes})")]
    TextTooLong {
        text: String,
        max_bytes: usize,
        actual_bytes: usize,
    },

    #[error("Text contains unsupported characters: {0}")]
    UnsupportedCharacters(String),

    #[error("Invalid character in text: {0}")]
    InvalidCharacter(char),

    #[error("ROM address not found: {0}")]
    AddressNotFound(String),

    #[error("Invalid pointer offset: 0x{0:X}")]
    InvalidPointerOffset(usize),

    #[error("Text entry not found: ID {0}")]
    EntryNotFound(u8),

    #[error("Boxer not found: {0}")]
    BoxerNotFound(String),

    #[error("Encoding error: {0}")]
    EncodingError(String),
}

/// Text search query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextSearchQuery {
    pub search_term: String,
    pub boxer_filter: Option<String>,
    pub category_filter: Option<MenuCategory>,
    pub condition_filter: Option<TextCondition>,
}

/// Search results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextSearchResult {
    pub result_type: TextResultType,
    pub id: String,
    pub boxer_key: Option<String>,
    pub text_preview: String,
    pub match_positions: Vec<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextResultType {
    Cornerman,
    Intro,
    VictoryQuote,
    Menu,
    Credits,
}

/// Text preview renderer (for in-game appearance)
pub struct TextPreviewRenderer;

impl TextPreviewRenderer {
    /// Render text as it would appear in-game
    ///
    /// This is a simplified preview - actual rendering would use
    /// the game's font tiles and palette
    pub fn render_preview(text: &str, max_width: usize) -> String {
        let mut result = String::new();
        let mut current_line = String::new();

        for word in text.split_whitespace() {
            if current_line.len() + word.len() + 1 > max_width {
                if !result.is_empty() {
                    result.push('\n');
                }
                result.push_str(&current_line);
                current_line = word.to_string();
            } else {
                if !current_line.is_empty() {
                    current_line.push(' ');
                }
                current_line.push_str(word);
            }
        }

        if !current_line.is_empty() {
            if !result.is_empty() {
                result.push('\n');
            }
            result.push_str(&current_line);
        }

        result
    }

    /// Get estimated display width of text
    ///
    /// Assumes each character is roughly the same width
    /// in the game's font (which uses fixed-width tiles)
    pub fn estimate_display_width(text: &str) -> usize {
        text.chars().count()
    }

    /// Check if text would fit on one line
    pub fn fits_on_line(text: &str, max_chars: usize) -> bool {
        text.chars().count() <= max_chars
    }
}

/// Validation summary for text editing
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TextValidationSummary {
    pub total_entries: usize,
    pub valid_entries: usize,
    pub invalid_entries: usize,
    pub warnings: Vec<String>,
    pub errors: Vec<(String, TextError)>,
}

impl TextValidationSummary {
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_encoder() {
        let encoder = TextEncoder::new();

        let text = "HELLO WORLD";
        let encoded = encoder.encode(text);
        let decoded = encoder.decode(&encoded);

        assert_eq!(text, decoded);
    }

    #[test]
    fn test_text_condition_roundtrip() {
        for condition in TextCondition::all_conditions() {
            let byte = condition.to_byte();
            let decoded = TextCondition::from_byte(byte);
            assert_eq!(condition, decoded);
        }
    }

    #[test]
    fn test_cornerman_validation() {
        let encoder = TextEncoder::new();
        let text = CornermanText::new(0, "test", "SHORT TEXT");

        assert!(text.validate(&encoder).is_ok());

        let long_text = CornermanText::new(1, "test", "A".repeat(100));
        assert!(long_text.validate(&encoder).is_err());
    }

    #[test]
    fn test_text_preview_renderer() {
        let text = "This is a long text that should be wrapped";
        let preview = TextPreviewRenderer::render_preview(text, 15);

        assert!(preview.contains('\n'));
        assert!(preview.lines().all(|line| line.len() <= 15));
    }

    #[test]
    fn test_pointer_conversion() {
        let pc_offset = 0x12345;
        let pointer = TextPointer::from_pc_offset(0, pc_offset);

        assert_eq!(pointer.to_pc_offset(), Some(pc_offset));
    }
}
