//! Comparison Mode Commands
//!
//! Commands for comparing original vs modded ROM data.

use tauri::State;

use crate::app_state::AppState;
use crate::utils::parse_offset;
use rom_core::comparison::*;

/// Generate a full comparison between the immutable base ROM and the exact current revision.
#[tauri::command]
pub fn generate_comparison(state: State<AppState>) -> Result<RomComparison, String> {
    let session_guard = state.rom_session.lock();
    let session = session_guard.as_ref().ok_or("No ROM loaded")?;
    let materialized = session.materialize().map_err(|error| error.to_string())?;
    let base = session.base();
    let mut comparison =
        RomComparison::new(base.sha1().to_string(), materialized.current_sha1.clone());

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

/// Get palette diff for a specific offset.
#[tauri::command]
pub fn get_palette_diff(state: State<AppState>, pc_offset: String) -> Result<PaletteDiff, String> {
    let offset = parse_offset(&pc_offset)?;
    let manifest = state.manifest.lock();
    let mut asset_id = String::new();
    let mut boxer_name = String::new();
    let mut palette_size = 32usize;
    for (fighter_name, boxer) in &manifest.fighters {
        if let Some(palette) = boxer
            .palette_files
            .iter()
            .find(|palette| palette.start_pc == pc_offset)
        {
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

    Ok(PaletteDiff {
        offset,
        boxer: boxer_name,
        asset_id,
        colors,
    })
}

/// Get sprite bin diff for a specific offset.
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
        return Err(format!(
            "Sprite bin {pc_offset} was not found in the manifest"
        ));
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
        changed_tiles.push(TileDiff {
            tile_index: tile_idx,
            pixel_diffs,
            has_changes: true,
        });
    }

    Ok(SpriteDiff {
        pc_offset: offset,
        boxer: boxer_name,
        bin_name,
        total_tiles,
        changed_tiles,
    })
}

/// Get binary/hex diff for a specific offset.
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

/// Export comparison report to file
#[tauri::command]
pub fn export_comparison_report(
    state: State<AppState>,
    output_path: String,
    format: String,
) -> Result<(), String> {
    let comparison = generate_comparison(state)?;

    match format.as_str() {
        "json" => {
            let json = serde_json::to_string_pretty(&comparison).map_err(|e| e.to_string())?;
            std::fs::write(&output_path, json).map_err(|e| e.to_string())?;
        }
        "html" => {
            let html = generate_html_report(&comparison);
            std::fs::write(&output_path, html).map_err(|e| e.to_string())?;
        }
        "text" => {
            let text = generate_text_report(&comparison);
            std::fs::write(&output_path, text).map_err(|e| e.to_string())?;
        }
        _ => return Err(format!("Unknown format: {}", format)),
    }

    Ok(())
}

/// Generate HTML comparison report
fn generate_html_report(comparison: &RomComparison) -> String {
    let mut html = String::new();
    html.push_str("<!DOCTYPE html><html><head>");
    html.push_str("<title>SPO Editor - ROM Comparison Report</title>");
    html.push_str("<style>");
    html.push_str("body{font-family:sans-serif;max-width:1200px;margin:0 auto;padding:20px;background:#1a1a2e;color:#eee}");
    html.push_str("h1,h2{color:#e74c3c}");
    html.push_str(".summary{background:#16213e;padding:15px;border-radius:8px;margin-bottom:20px}");
    html.push_str(".diff-item{background:#0f3460;padding:10px;margin:10px 0;border-radius:4px}");
    html.push_str(".changed{color:#4ade80}");
    html.push_str(".unchanged{color:#666}");
    html.push_str("</style></head><body>");

    html.push_str("<h1>ROM Comparison Report</h1>");
    html.push_str(&format!(
        "<p>Original SHA1: {}</p>",
        comparison.original_sha1
    ));
    html.push_str(&format!(
        "<p>Modified SHA1: {}</p>",
        comparison.modified_sha1
    ));

    html.push_str("<div class=\"summary\">");
    html.push_str("<h2>Summary</h2>");
    html.push_str(&format!(
        "<p>Total Changes: {}</p>",
        comparison.summary.total_changes
    ));
    html.push_str(&format!(
        "<p>Palettes Modified: {}</p>",
        comparison.summary.palettes_modified
    ));
    html.push_str(&format!(
        "<p>Sprite Bins Changed: {}</p>",
        comparison.summary.sprite_bins_changed
    ));
    html.push_str(&format!(
        "<p>Tiles Changed: {}</p>",
        comparison.summary.tiles_changed
    ));
    html.push_str(&format!(
        "<p>Total Bytes Changed: {}</p>",
        comparison.summary.total_bytes_changed
    ));
    html.push_str("</div>");

    html.push_str("<h2>Differences</h2>");
    for diff in &comparison.differences {
        html.push_str("<div class=\"diff-item\">");
        match diff {
            Difference::Palette {
                boxer,
                asset_id,
                changed_indices,
                ..
            } => {
                html.push_str(&format!(
                    "<strong>Palette:</strong> {} - {} ({} colors changed)",
                    boxer,
                    asset_id,
                    changed_indices.len()
                ));
            }
            Difference::Sprite {
                boxer,
                bin_name,
                changed_tile_indices,
                ..
            } => {
                html.push_str(&format!(
                    "<strong>Sprite Bin:</strong> {} - {} ({} tiles changed)",
                    boxer,
                    bin_name,
                    changed_tile_indices.len()
                ));
            }
            Difference::Header {
                boxer,
                changed_fields,
                ..
            } => {
                html.push_str(&format!(
                    "<strong>Header:</strong> {} ({} fields changed)",
                    boxer,
                    changed_fields.len()
                ));
            }
            Difference::Animation {
                boxer, anim_name, ..
            } => {
                html.push_str(&format!(
                    "<strong>Animation:</strong> {} - {}",
                    boxer, anim_name
                ));
            }
            Difference::Binary {
                description,
                bytes_changed,
                ..
            } => {
                html.push_str(&format!(
                    "<strong>Binary:</strong> {} ({} bytes)",
                    description, bytes_changed
                ));
            }
        }
        html.push_str("</div>");
    }

    html.push_str("</body></html>");
    html
}

/// Generate text comparison report
fn generate_text_report(comparison: &RomComparison) -> String {
    let mut text = String::new();
    text.push_str("ROM Comparison Report\n");
    text.push_str("=====================\n\n");
    text.push_str(&format!("Original SHA1: {}\n", comparison.original_sha1));
    text.push_str(&format!("Modified SHA1: {}\n\n", comparison.modified_sha1));

    text.push_str("Summary\n");
    text.push_str("-------\n");
    text.push_str(&format!(
        "Total Changes: {}\n",
        comparison.summary.total_changes
    ));
    text.push_str(&format!(
        "Palettes Modified: {}\n",
        comparison.summary.palettes_modified
    ));
    text.push_str(&format!(
        "Sprite Bins Changed: {}\n",
        comparison.summary.sprite_bins_changed
    ));
    text.push_str(&format!(
        "Tiles Changed: {}\n",
        comparison.summary.tiles_changed
    ));
    text.push_str(&format!(
        "Total Bytes Changed: {}\n\n",
        comparison.summary.total_bytes_changed
    ));

    text.push_str("Differences\n");
    text.push_str("-----------\n");
    for diff in &comparison.differences {
        match diff {
            Difference::Palette {
                boxer,
                asset_id,
                changed_indices,
                ..
            } => {
                text.push_str(&format!(
                    "[PALETTE] {} - {}: {} colors changed\n",
                    boxer,
                    asset_id,
                    changed_indices.len()
                ));
            }
            Difference::Sprite {
                boxer,
                bin_name,
                changed_tile_indices,
                ..
            } => {
                text.push_str(&format!(
                    "[SPRITE] {} - {}: {} tiles changed\n",
                    boxer,
                    bin_name,
                    changed_tile_indices.len()
                ));
            }
            Difference::Header {
                boxer,
                changed_fields,
                ..
            } => {
                text.push_str(&format!(
                    "[HEADER] {}: {} fields changed\n",
                    boxer,
                    changed_fields.len()
                ));
                for field in changed_fields {
                    text.push_str(&format!(
                        "  - {}: {} -> {}\n",
                        field.field_name, field.original_value, field.modified_value
                    ));
                }
            }
            Difference::Animation {
                boxer, anim_name, ..
            } => {
                text.push_str(&format!("[ANIMATION] {} - {}\n", boxer, anim_name));
            }
            Difference::Binary {
                description,
                bytes_changed,
                ..
            } => {
                text.push_str(&format!(
                    "[BINARY] {} ({} bytes)\n",
                    description, bytes_changed
                ));
            }
        }
    }

    text
}
