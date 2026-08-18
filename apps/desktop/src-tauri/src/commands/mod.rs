//! Tauri Command Modules
//!
//! This module contains all Tauri command handlers organized by category.
//! Each submodule handles a specific domain of functionality.

// Allow ambiguous glob re-exports for the placeholder functions
#![allow(ambiguous_glob_reexports)]

// Re-export all command modules
pub mod ai_behavior;
pub mod animation;
pub mod assets;
pub mod audio;
pub mod bank_management;
pub mod boxer;
pub mod comparison;
pub mod emulator;
pub mod emulator_current;
pub mod expansion;
pub mod frame_reconstructor;
pub mod frame_tags;
pub mod help;
pub mod history;
pub mod layout_pack;
pub mod patches;
pub mod plugins;
pub mod project;
pub mod project_thumbnail;
pub mod region;
pub mod relocation;
pub mod rom;
pub mod scripts;
pub mod settings;
pub mod text;
pub mod tools;

// Re-export commonly used types from submodules
pub use ai_behavior::*;
pub use animation::*;
pub use assets::*;
pub use audio::*;
pub use bank_management::*;
pub use boxer::*;
pub use comparison::*;
pub use emulator::*;
pub use emulator_current::*;
pub use expansion::*;
pub use frame_reconstructor::*;
pub use frame_tags::*;
pub use help::*;
pub use history::*;
pub use layout_pack::*;
pub use patches::*;
pub use plugins::*;
pub use project::*;
pub use project_thumbnail::*;
pub use region::*;
pub use relocation::*;
pub use rom::*;
pub use scripts::*;
pub use settings::*;
pub use text::*;
