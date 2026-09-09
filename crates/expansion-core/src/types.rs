use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Original SPO roster size.
pub const VANILLA_BOXER_COUNT: usize = 16;

/// Header signature written into the ROM for in-game editor discovery.
pub const EDITOR_HEADER_MAGIC: [u8; 8] = *b"SPOEDITR";

/// Target configuration for in-ROM editor expansion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpansionOptions {
    /// Desired number of boxers in the expanded roster.
    pub target_boxer_count: usize,
    /// Whether to patch a JML hook into game code.
    pub patch_editor_hook: bool,
    /// PC offset where a 4-byte JML should be written, when hook patching is enabled.
    pub editor_hook_pc_offset: Option<usize>,
    /// Optional exact number of bytes to preserve at the hook site.
    ///
    /// When not provided, the patcher auto-selects an instruction-aligned length
    /// that is at least 4 bytes.
    pub editor_hook_overwrite_len: Option<usize>,
}

impl Default for ExpansionOptions {
    fn default() -> Self {
        Self {
            target_boxer_count: VANILLA_BOXER_COUNT,
            patch_editor_hook: false,
            editor_hook_pc_offset: None,
            editor_hook_overwrite_len: None,
        }
    }
}

/// Concrete expanded roster table locations (PC offsets).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpandedRosterLayout {
    pub boxer_count: usize,
    pub name_pointer_table_pc: usize,
    pub name_long_pointer_table_pc: usize,
    pub name_blob_pc: usize,
    pub circuit_table_pc: usize,
    pub unlock_table_pc: usize,
    pub intro_table_pc: usize,
}

/// ROM write range record for diagnostics/reports.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriteRange {
    pub start_pc: usize,
    pub size: usize,
    pub description: String,
}

/// Expansion result returned to callers/UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpansionReport {
    pub layout: ExpandedRosterLayout,
    pub header_pc: usize,
    pub editor_stub_pc: usize,
    pub editor_hook_patched: bool,
    pub editor_hook_overwrite_len: usize,
    pub write_ranges: Vec<WriteRange>,
    pub notes: Vec<String>,
}

/// Candidate hook location discovered by static ROM scan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookSiteCandidate {
    pub hook_pc: usize,
    pub overwrite_len: usize,
    pub return_pc: usize,
    pub first_instruction: String,
    pub preview_bytes: Vec<u8>,
}

#[derive(Debug, Error)]
pub enum ExpansionError {
    #[error("target boxer count must be between {min} and {max}, got {actual}")]
    InvalidTargetCount {
        min: usize,
        max: usize,
        actual: usize,
    },
    #[error("hook overwrite length must be between {min} and {max}, got {actual}")]
    InvalidHookOverwriteLen {
        min: usize,
        max: usize,
        actual: usize,
    },
    #[error("hook patching requested but no hook PC offset was provided")]
    MissingHookOffset,
    #[error("unable to decode instruction at hook PC 0x{pc:06X}")]
    HookDecodeFailed { pc: usize },
    #[error("hook overwrite length at PC 0x{pc:06X} splits an instruction boundary")]
    HookSplitInstruction { pc: usize },
    #[error("unsafe hook instruction at PC 0x{pc:06X}: {mnemonic}")]
    UnsafeHookInstruction { pc: usize, mnemonic: String },
    #[error("unable to allocate free space for {0}")]
    FreeSpaceNotFound(&'static str),
    #[error("rom operation failed: {0}")]
    Rom(String),
}

pub type ExpansionResult<T> = Result<T, ExpansionError>;
