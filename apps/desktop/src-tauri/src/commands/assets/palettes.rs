//! Palette Asset Commands
//!
//! Commands for reading and encoding palette data.

use tauri::State;

use asset_core::Color;

use crate::app_state::AppState;
use crate::utils::parse_offset;

use super::{encode_palette_bytes, read_original_rom_bytes, read_palette_colors, AssetResult};

#[tauri::command]
pub fn get_palette(
    state: State<AppState>,
    pc_offset: String,
    size: usize,
) -> AssetResult<Vec<Color>> {
    let palette_pc_offset = parse_offset(&pc_offset)?;
    read_palette_colors(state.inner(), palette_pc_offset, size)
}

#[tauri::command]
pub fn encode_palette_for_preview(colors: Vec<Color>) -> AssetResult<Vec<u8>> {
    Ok(encode_palette_bytes(&colors))
}

/// Most colours a single recolour may write. A boxer palette file is far smaller.
pub const MAX_PALETTE_COLORS: usize = 256;

/// Write a whole palette as ONE journal transaction, so a recolour of many
/// colours is undone with a single Undo.
///
/// Returns `false` when the colours already match and nothing was written.
pub fn apply_palette_colors_to_state(
    state: &AppState,
    pc_offset: usize,
    colors: &[Color],
    label: Option<String>,
) -> AssetResult<bool> {
    if colors.is_empty() {
        return Err("No colors were given".to_string());
    }
    if colors.len() > MAX_PALETTE_COLORS {
        return Err(format!(
            "Too many colors ({}); a palette holds at most {MAX_PALETTE_COLORS}",
            colors.len()
        ));
    }

    let mut new_bytes = encode_palette_bytes(colors);
    let current = super::read_current_asset_bytes(state, pc_offset, new_bytes.len())?;
    // A colour only uses 15 bits. Keep whatever the ROM stores in the spare top
    // bit so a recolour never changes bytes it does not understand.
    for (new_pair, current_pair) in new_bytes.chunks_exact_mut(2).zip(current.chunks_exact(2)) {
        new_pair[1] = (new_pair[1] & 0x7F) | (current_pair[1] & 0x80);
    }
    if current == new_bytes {
        return Ok(false);
    }

    let label = label
        .map(|text| text.trim().chars().take(80).collect::<String>())
        .filter(|text| !text.is_empty())
        .unwrap_or_else(|| format!("Recolor palette at 0x{pc_offset:X}"));

    state.commit_rom_write(
        label,
        pc_offset,
        new_bytes,
        Some(format!("palette@0x{pc_offset:X}")),
        Some("Palette recolor".to_string()),
    )?;
    Ok(true)
}

/// Replace every colour of a palette in one undoable step.
#[tauri::command]
pub fn apply_palette_colors(
    state: State<AppState>,
    pc_offset: String,
    colors: Vec<Color>,
    label: Option<String>,
) -> AssetResult<bool> {
    let palette_pc_offset = parse_offset(&pc_offset)?;
    apply_palette_colors_to_state(state.inner(), palette_pc_offset, &colors, label)
}

/// Read a palette as it is in the untouched original ROM, ignoring all edits.
#[tauri::command]
pub fn get_original_palette(
    state: State<AppState>,
    pc_offset: String,
    size: usize,
) -> AssetResult<Vec<Color>> {
    let palette_pc_offset = parse_offset(&pc_offset)?;
    let bytes = read_original_rom_bytes(state.inner(), palette_pc_offset, size)?;
    Ok(asset_core::decode_palette(&bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use manifest_core::Manifest;
    use rom_core::Rom;

    fn state_with_rom(bytes: Vec<u8>) -> AppState {
        let state = AppState::new(Manifest::empty());
        state.install_rom_session(Rom::new(bytes), "synthetic.sfc".into());
        state
    }

    #[test]
    fn recolor_is_one_undoable_transaction() {
        let state = state_with_rom(vec![0; 16]);
        let colors = vec![
            Color::new(248, 0, 0),
            Color::new(0, 248, 0),
            Color::new(0, 0, 248),
        ];

        let changed = apply_palette_colors_to_state(&state, 4, &colors, None).unwrap();
        assert!(changed);

        let after = state.materialize_current_rom().unwrap().bytes;
        assert_eq!(&after[4..10], encode_palette_bytes(&colors).as_slice());
        assert_eq!(&after[..4], &[0, 0, 0, 0]);
        assert_eq!(&after[10..], &[0; 6]);

        // One Undo restores every colour at once.
        state.undo_journal().unwrap().unwrap();
        assert_eq!(state.materialize_current_rom().unwrap().bytes, vec![0; 16]);
        assert!(state.undo_journal().unwrap().is_none());
    }

    #[test]
    fn unchanged_colors_do_not_create_history() {
        let state = state_with_rom(vec![0; 16]);
        let colors = vec![Color::new(0, 0, 0); 2];
        assert!(!apply_palette_colors_to_state(&state, 0, &colors, None).unwrap());
        assert!(state.undo_journal().unwrap().is_none());
    }

    #[test]
    fn out_of_range_and_oversized_recolors_are_rejected_without_writing() {
        let state = state_with_rom(vec![0; 8]);
        let colors = vec![Color::new(248, 248, 248); 3];
        assert!(apply_palette_colors_to_state(&state, 6, &colors, None).is_err());
        assert!(apply_palette_colors_to_state(&state, 0, &[], None).is_err());
        let too_many = vec![Color::new(8, 8, 8); MAX_PALETTE_COLORS + 1];
        assert!(apply_palette_colors_to_state(&state, 0, &too_many, None).is_err());
        assert_eq!(state.materialize_current_rom().unwrap().bytes, vec![0; 8]);
    }

    #[test]
    fn spare_top_bit_is_preserved() {
        let state = state_with_rom(vec![0x00, 0x80, 0x00, 0x00]);
        let colors = vec![Color::new(248, 0, 0), Color::new(248, 0, 0)];
        apply_palette_colors_to_state(&state, 0, &colors, None).unwrap();
        assert_eq!(
            state.materialize_current_rom().unwrap().bytes,
            vec![0x1F, 0x80, 0x1F, 0x00]
        );
    }

    #[test]
    fn original_palette_ignores_edits() {
        let state = state_with_rom(vec![0; 8]);
        apply_palette_colors_to_state(&state, 0, &[Color::new(248, 0, 0)], None).unwrap();
        let original = read_original_rom_bytes(&state, 0, 2).unwrap();
        assert_eq!(original, vec![0, 0]);
    }
}
