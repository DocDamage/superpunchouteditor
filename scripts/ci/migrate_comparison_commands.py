#!/usr/bin/env python3
"""Move comparison commands to immutable-base versus materialized-current state."""
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[2]
PATH = ROOT / "apps/desktop/src-tauri/src/commands/comparison.rs"
text = PATH.read_text(encoding="utf-8")

canonical_generate = r'''/// Generate a full comparison between the immutable base ROM and the exact current revision.
#[tauri::command]
pub fn generate_comparison(state: State<AppState>) -> Result<RomComparison, String> {
    let session_guard = state.rom_session.lock();
    let session = session_guard.as_ref().ok_or("No ROM loaded")?;
    let materialized = session.materialize().map_err(|error| error.to_string())?;
    let base = session.base();
    let mut comparison = RomComparison::new(
        base.sha1().to_string(),
        materialized.current_sha1.clone(),
    );

    for range in &materialized.change_ranges {
        let original = &base.bytes()[range.start..range.end.min(base.len())];
        let modified_end = range.end.min(materialized.bytes.len());
        let modified = &materialized.bytes[range.start..modified_end];
        let common = original.len().min(modified.len());
        let mut changed = (0..common)
            .filter(|index| original[*index] != modified[*index])
            .count();
        changed += original.len().abs_diff(modified.len());
        if changed == 0 && range.end <= base.len() {
            continue;
        }
        comparison.add_difference(Difference::Binary {
            offset: range.start,
            size: range.end.saturating_sub(range.start),
            bytes_changed: changed.max(range.end.saturating_sub(base.len())),
            description: format!(
                "Canonical journal change at 0x{:X} (revision {})",
                range.start, materialized.revision
            ),
        });
    }

    Ok(comparison)
}
'''
text, count = re.subn(
    r'/// Generate a full comparison between original ROM and current state\n#\[tauri::command\]\npub fn generate_comparison\(.*?\n}\n\n/// Get palette diff for a specific offset',
    canonical_generate + '\n/// Get palette diff for a specific offset',
    text,
    count=1,
    flags=re.S,
)
if count == 0 and "Canonical journal change at" not in text:
    raise SystemExit("generate_comparison source shape changed")

palette_fn = r'''/// Get palette diff for a specific offset.
#[tauri::command]
pub fn get_palette_diff(state: State<AppState>, pc_offset: String) -> Result<PaletteDiff, String> {
    let offset = parse_offset(&pc_offset)?;
    let manifest = state.manifest.lock();
    let mut asset_id = String::new();
    let mut boxer_name = String::new();
    let mut palette_size = 32usize;
    for (fighter_name, boxer) in &manifest.fighters {
        if let Some(palette) = boxer.palette_files.iter().find(|palette| palette.start_pc == pc_offset) {
            asset_id = format!("{}/{}", boxer.key, palette.filename);
            boxer_name = fighter_name.clone();
            palette_size = palette.size;
            break;
        }
    }
    drop(manifest);

    let session_guard = state.rom_session.lock();
    let session = session_guard.as_ref().ok_or("No ROM loaded")?;
    let current = session.materialize().map_err(|error| error.to_string())?;
    let base_range = rom_core::validate_range(offset, palette_size, session.base().len())
        .map_err(|error| error.to_string())?;
    let current_range = rom_core::validate_range(offset, palette_size, current.bytes.len())
        .map_err(|error| error.to_string())?;
    let original_bytes = &session.base().bytes()[base_range];
    let modified_bytes = &current.bytes[current_range];

    let color_count = original_bytes.len().min(modified_bytes.len()) / 2;
    let colors = (0..color_count)
        .map(|index| {
            let byte_index = index * 2;
            let original = ColorDiff::from_snes_bytes(
                original_bytes[byte_index],
                original_bytes[byte_index + 1],
            );
            let modified = ColorDiff::from_snes_bytes(
                modified_bytes[byte_index],
                modified_bytes[byte_index + 1],
            );
            ColorComparison {
                index,
                changed: original.r != modified.r
                    || original.g != modified.g
                    || original.b != modified.b,
                original,
                modified,
            }
        })
        .collect();

    Ok(PaletteDiff { offset, boxer: boxer_name, asset_id, colors })
}
'''
text, count = re.subn(
    r'/// Get palette diff for a specific offset\n#\[tauri::command\]\npub fn get_palette_diff\(.*?\n}\n\n/// Get sprite bin diff for a specific offset',
    palette_fn + '\n/// Get sprite bin diff for a specific offset',
    text,
    count=1,
    flags=re.S,
)
if count == 0 and "let current = session.materialize()" not in text:
    raise SystemExit("get_palette_diff source shape changed")

sprite_fn = r'''/// Get sprite bin diff for a specific offset.
#[tauri::command]
pub fn get_sprite_bin_diff_comparison(
    state: State<AppState>,
    pc_offset: String,
) -> Result<SpriteDiff, String> {
    let offset = parse_offset(&pc_offset)?;
    let manifest = state.manifest.lock();
    let mut bin_name = String::new();
    let mut boxer_name = String::new();
    let mut bin_size = 0usize;
    for (fighter_name, boxer) in &manifest.fighters {
        if let Some(bin) = boxer
            .unique_sprite_bins
            .iter()
            .chain(boxer.shared_sprite_bins.iter())
            .find(|bin| bin.start_pc == pc_offset)
        {
            bin_name = bin.filename.clone();
            boxer_name = fighter_name.clone();
            bin_size = bin.size;
            break;
        }
    }
    drop(manifest);
    if bin_size == 0 {
        return Err(format!("Sprite bin {pc_offset} was not found in the manifest"));
    }

    let session_guard = state.rom_session.lock();
    let session = session_guard.as_ref().ok_or("No ROM loaded")?;
    let current = session.materialize().map_err(|error| error.to_string())?;
    let base_range = rom_core::validate_range(offset, bin_size, session.base().len())
        .map_err(|error| error.to_string())?;
    let current_range = rom_core::validate_range(offset, bin_size, current.bytes.len())
        .map_err(|error| error.to_string())?;
    let original_bytes = &session.base().bytes()[base_range];
    let modified_bytes = &current.bytes[current_range];

    let total_tiles = original_bytes.len() / 32;
    let changed_indices = ComparisonEngine::compare_tiles(original_bytes, modified_bytes);
    let mut changed_tiles = Vec::new();
    for tile_idx in changed_indices {
        let tile_start = tile_idx * 32;
        let mut pixel_diffs = Vec::new();
        for i in 0..32 {
            let idx = tile_start + i;
            let original = original_bytes.get(idx).copied().unwrap_or(0);
            let modified = modified_bytes.get(idx).copied().unwrap_or(0);
            let row = (i / 2) % 8;
            let col1 = (i % 2) * 4;
            let col2 = col1 + 1;
            for (x, original_pixel, modified_pixel) in [
                (col1, original & 0x0f, modified & 0x0f),
                (col2, (original >> 4) & 0x0f, (modified >> 4) & 0x0f),
            ] {
                pixel_diffs.push(PixelDiff {
                    x,
                    y: row,
                    original_pixel,
                    modified_pixel,
                    changed: original_pixel != modified_pixel,
                });
            }
        }
        changed_tiles.push(TileDiff { tile_index: tile_idx, pixel_diffs, has_changes: true });
    }

    Ok(SpriteDiff { pc_offset: offset, boxer: boxer_name, bin_name, total_tiles, changed_tiles })
}
'''
text, count = re.subn(
    r'/// Get sprite bin diff for a specific offset\n#\[tauri::command\]\npub fn get_sprite_bin_diff_comparison\(.*?\n}\n\n/// Get binary/hex diff for a specific offset',
    sprite_fn + '\n/// Get binary/hex diff for a specific offset',
    text,
    count=1,
    flags=re.S,
)
if count == 0 and "Sprite bin {pc_offset} was not found" not in text:
    raise SystemExit("get_sprite_bin_diff_comparison source shape changed")

binary_fn = r'''/// Get binary/hex diff for a specific offset.
#[tauri::command]
pub fn get_binary_diff(
    state: State<AppState>,
    pc_offset: String,
    size: usize,
) -> Result<BinaryDiff, String> {
    let offset = parse_offset(&pc_offset)?;
    let session_guard = state.rom_session.lock();
    let session = session_guard.as_ref().ok_or("No ROM loaded")?;
    let current = session.materialize().map_err(|error| error.to_string())?;
    let base_range = rom_core::validate_range(offset, size, session.base().len())
        .map_err(|error| error.to_string())?;
    let current_range = rom_core::validate_range(offset, size, current.bytes.len())
        .map_err(|error| error.to_string())?;
    Ok(ComparisonEngine::generate_hex_diff(
        &session.base().bytes()[base_range],
        &current.bytes[current_range],
        offset,
    ))
}
'''
text, count = re.subn(
    r'/// Get binary/hex diff for a specific offset\n#\[tauri::command\]\npub fn get_binary_diff\(.*?\n}\n\n/// Export comparison report to file',
    binary_fn + '\n/// Export comparison report to file',
    text,
    count=1,
    flags=re.S,
)
if count == 0 and "session.base().bytes()[base_range]" not in text:
    raise SystemExit("get_binary_diff source shape changed")

PATH.write_text(text, encoding="utf-8")
print("Comparison commands now use one canonical base/current snapshot.")
