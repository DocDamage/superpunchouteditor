//! Core types for patch notes

use serde::{Deserialize, Serialize};

/// Target ROM SHA1 for Super Punch-Out!! (USA)
pub const SPO_ROM_SHA1: &str = "3604c855790f37db567e9b425252625045f86697";

/// Summary statistics of changes
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChangeSummary {
    pub total_boxers_modified: usize,
    pub total_palettes_changed: usize,
    pub total_sprites_edited: usize,
    pub total_animations_modified: usize,
    pub total_headers_edited: usize,
    pub total_changes: usize,
}

/// A color in RGB format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

/// A single change entry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Change {
    Palette {
        name: String,
        colors_changed: usize,
        description: String,
    },
    Sprite {
        bin_name: String,
        tiles_modified: usize,
        description: String,
    },
    Stats {
        field: String,
        before: String,
        after: String,
        significant: bool,
    },
    Animation {
        name: String,
        frames_changed: usize,
        description: String,
    },
    Other {
        description: String,
    },
}

/// Changes for a specific boxer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoxerChangeSet {
    pub boxer_name: String,
    pub boxer_key: String,
    pub changes: Vec<Change>,
}

/// System-level changes not tied to a specific boxer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemChange {
    pub category: String,
    pub description: String,
}

/// Output format for patch notes
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputFormat {
    Markdown,
    Html,
    PlainText,
    Json,
    Bbcode,
}

impl OutputFormat {
    pub fn from_string(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "markdown" | "md" => Some(Self::Markdown),
            "html" | "htm" => Some(Self::Html),
            "text" | "txt" | "plaintext" => Some(Self::PlainText),
            "json" => Some(Self::Json),
            "bbcode" | "bb" => Some(Self::Bbcode),
            _ => None,
        }
    }

    pub fn file_extension(&self) -> &str {
        match self {
            Self::Markdown => "md",
            Self::Html => "html",
            Self::PlainText => "txt",
            Self::Json => "json",
            Self::Bbcode => "bb",
        }
    }
}
