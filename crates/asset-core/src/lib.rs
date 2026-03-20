//! # Asset Core
//!
//! This crate provides asset processing for Super Punch-Out!! including:
//! - Palette encoding/decoding (SNES BGR555 format)
//! - 4bpp tile graphics processing
//! - HAL8-style compression/decompression
//! - Frame composition and sprite management
//! - Audio sample handling (BRR format)
//!
//! ## Example
//! ```
//! use asset_core::{decode_palette, Color};
//!
//! let palette_bytes = vec![0x00, 0x00, 0xFF, 0x03]; // 2 colors
//! let colors = decode_palette(&palette_bytes);
//! ```

pub mod animation;
pub mod audio;
pub mod brr;
pub mod compression;
pub mod fighter;
pub mod frame;
pub mod frame_renderer;
pub mod frame_tags;
pub mod gfx;
pub mod palette;
pub mod spc;

pub use animation::{
    AnimationFrame, AnimationPlayer, AnimationSequence, BlendMode, CombatHitbox, HitboxEditor,
    HitboxType, Hurtbox, EditMode, FrameMetadata, FrameFlags, FrameTrigger, InterpolatedFrame,
};
#[allow(ambiguous_glob_reexports)]
pub use audio::*;
#[allow(ambiguous_glob_reexports)]
pub use brr::*;
pub use compression::*;
pub use fighter::*;
pub use frame::*;
pub use frame_renderer::*;
pub use frame_tags::*;
pub use gfx::*;
pub use palette::*;
pub use spc::*;
