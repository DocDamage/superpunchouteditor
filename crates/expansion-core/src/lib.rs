//! Expansion support for in-ROM tooling.
//!
//! This crate provides the first step of "editor inside the ROM":
//! - Dynamic roster table expansion (beyond the stock 16 boxers)
//! - In-ROM editor bootstrap metadata + optional hook patching
//!
//! The bootstrap is intentionally conservative. It writes structured data and an
//! entry stub the game can jump to, but it does not yet replace all gameplay
//! code paths that assume fixed-size vanilla tables.

mod ingame_editor;
mod roster_expansion;
mod types;

pub use ingame_editor::*;
pub use roster_expansion::*;
pub use types::*;
