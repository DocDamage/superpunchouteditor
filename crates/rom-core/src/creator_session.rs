//! Shared creator-session protocol constants.
//!
//! These values are written to / read from WRAM by the in-ROM creator hook and
//! are used by every crate that participates in the creator session protocol:
//! - `emulator-core` (Stub runtime and Libretro monitor)
//! - `expansion-core` (ingame editor state machine)
//! - `tauri-appsuper-punch-out-editor` (desktop roster commands)
//!
//! All values must stay in sync with the hook assembly embedded in the ROM
//! expansion code.

// ----------------------------------------------------------------------------
// Session status bytes (CREATOR_SESSION_STATUS WRAM slot)
// ----------------------------------------------------------------------------

/// Creator is active and the draft has been initialised.
pub const CREATOR_SESSION_STATUS_DRAFT_READY: u8 = 0x02;

/// The player has confirmed; emulator-core is performing the ROM mutation.
pub const CREATOR_SESSION_STATUS_COMMIT_PENDING: u8 = 0x03;

/// Commit completed successfully; new boxer written to ROM.
pub const CREATOR_SESSION_STATUS_COMMIT_SUCCEEDED: u8 = 0x04;

/// Commit failed; see `CREATOR_SESSION_ERROR_CODE` for reason.
pub const CREATOR_SESSION_STATUS_COMMIT_FAILED: u8 = 0x05;

/// Player cancelled; draft discarded.
pub const CREATOR_SESSION_STATUS_CANCELLED: u8 = 0x07;

// ----------------------------------------------------------------------------
// Error codes (CREATOR_SESSION_ERROR_CODE WRAM slot)
// ----------------------------------------------------------------------------

/// Unspecified error.
pub const CREATOR_ERROR_GENERIC: u8 = 0x01;

/// Boxer slot referenced in the draft was not found in the roster.
pub const CREATOR_ERROR_BOXER_NOT_FOUND: u8 = 0x02;

/// Draft name field contains invalid characters or exceeds maximum length.
pub const CREATOR_ERROR_INVALID_NAME: u8 = 0x03;

/// Draft intro-text field contains invalid characters or exceeds maximum length.
pub const CREATOR_ERROR_INVALID_INTRO_TEXT: u8 = 0x04;

/// Intro slot index is out of range for the current ROM layout.
pub const CREATOR_ERROR_INVALID_INTRO_SLOT: u8 = 0x05;
