//! Comparison Mode Commands
//!
//! Commands for comparing original vs modded ROM data.

use tauri::State;

use crate::app_state::AppState;
use crate::utils::parse_offset;
use rom_core::comparison::*;

/// Generate a full comparison between original ROM and current state
#[tauri::command]
pub fn generate_comparison(state: State<AppState>) -> Result<RomComparison, String> {
    let rom_opt = state.rom.lock();
    let rom = rom_opt.as_ref().ok_or("No ROM loaded")?;

    let original_sha1 = rom.calculate_sha1();
    let pending = state.pending_writes.lock();

    // Build modified ROM data by applying pending writes
    let mut modified_data = rom.data.clone();
    for (offset_str, bytes) in pending.iter() {
        let offset = parse_offset(offset_str)?;
        let len = bytes.len().min(modified_data.len() - offset);
        modified_data[offset..offset + len].copy_from_slice(&bytes[..len]);
    }

    // Calculate modified SHA1
    let modified_sha1 = {
        use sha1::{Digest, Sha1};
        let mut hasher = Sha1::new();
        hasher.update(&modified_data);
        format!("{:x}", hasher.finalize())
    };

    let mut comparison = RomComparison::new(original_sha1, modified_sha1);

    let manifest = state.manifest.lock();

    // Compare all assets from the manifest
    for (fighter_name, boxer) in &manifest.fighters {
        // Compare palettes
        for palette in &boxer.palette_files {
            let offset = parse_offset(&palette.start_pc)?;
            let original_bytes = rom
                .read_bytes(offset, palette.size)
                .map_err(|e| e.to_string())?;
            let modified_bytes = if let Some(edited) = pending.get(&palette.start_pc) {
                edited.as_slice()
            } else {
                original_bytes
            };

            if original_bytes != modified_bytes {
                let changed_indices =
                    ComparisonEngine::compare_bytes(original_bytes, modified_bytes);
                let original_colors: Vec<ColorDiff> = original_bytes
                    .chunks_exact(2)
                    .map(|chunk| ColorDiff::from_snes_bytes(chunk[0], chunk[1]))
                    .collect();
                let modified_colors: Vec<ColorDiff> = modified_bytes
                    .chunks_exact(2)
                    .map(|chunk| ColorDiff::from_snes_bytes(chunk[0], chunk[1]))
                    .collect();

                comparison.add_difference(Difference::Palette {
                    offset,
                    asset_id: format!("{}/{}", boxer.key, palette.filename),
                    boxer: fighter_name.clone(),
                    original_colors,
                    modified_colors,
                    changed_indices,
                });
            }
        }

        // Compare unique sprite bins
        for bin in &boxer.unique_sprite_bins {
            let offset = parse_offset(&bin.start_pc)?;
            let original_bytes = rom
                .read_bytes(offset, bin.size)
                .map_err(|e| e.to_string())?;
            let modified_bytes = if let Some(edited) = pending.get(&bin.start_pc) {
                edited.as_slice()
            } else {
                original_bytes
            };

            if original_bytes != modified_bytes {
                let changed_tiles = ComparisonEngine::compare_tiles(original_bytes, modified_bytes);
                let total_tiles = original_bytes.len() / 32;

                // Calculate per-tile change counts
                let mut tile_change_counts = std::collections::HashMap::new();
                for &tile_idx in &changed_tiles {
                    let tile_start = tile_idx * 32;
                    let mut count = 0;
                    for i in 0..32 {
                        if tile_start + i < original_bytes.len()
                            && tile_start + i < modified_bytes.len()
                            && original_bytes[tile_start + i] != modified_bytes[tile_start + i]
                        {
                            count += 1;
                        }
                    }
                    tile_change_counts.insert(tile_idx, count);
                }

                comparison.add_difference(Difference::Sprite {
                    boxer: fighter_name.clone(),
                    bin_name: bin.filename.clone(),
                    pc_offset: offset,
                    total_tiles,
                    changed_tile_indices: changed_tiles,
                    tile_change_counts,
                });
            }
        }
    }

    Ok(comparison)
}

/// Get palette diff for a specific offset
#[tauri::command]
pub fn get_palette_diff(state: State<AppState>, pc_offset: String) -> Result<PaletteDiff, String> {
    let offset = parse_offset(&pc_offset)?;

    let rom_opt = state.rom.lock();
    let rom = rom_opt.as_ref().ok_or("No ROM loaded")?;
    let pending = state.pending_writes.lock();
    let manifest = state.manifest.lock();

    // Find palette info from manifest
    let mut asset_id = String::new();
    let mut boxer_name = String::new();
    let mut palette_size = 32usize;

    for (fighter_name, boxer) in &manifest.fighters {
        for palette in &boxer.palette_files {
            if palette.start_pc == pc_offset {
                asset_id = format!("{}/{}", boxer.key, palette.filename);
                boxer_name = fighter_name.clone();
                palette_size = palette.size;
                break;
            }
        }
    }

    let original_bytes = rom
        .read_bytes(offset, palette_size)
        .map_err(|e| e.to_string())?;
    let modified_bytes = if let Some(edited) = pending.get(&pc_offset) {
        edited.clone()
    } else {
        original_bytes.to_vec()
    };

    let color_count = original_bytes.len() / 2;
    let mut colors = Vec::with_capacity(color_count);

    for i in 0..color_count {
        let byte_idx = i * 2;
        let orig = ColorDiff::from_snes_bytes(
            original_bytes.get(byte_idx).copied().unwrap_or(0),
            original_bytes.get(byte_idx + 1).copied().unwrap_or(0),
        );
        let modified = ColorDiff::from_snes_bytes(
            modified_bytes.get(byte_idx).copied().unwrap_or(0),
            modified_bytes.get(byte_idx + 1).copied().unwrap_or(0),
        );
        let changed = orig.r != modified.r || orig.g != modified.g || orig.b != modified.b;

        colors.push(ColorComparison {
            index: i,
            original: orig,
            modified,
            changed,
        });
    }

    Ok(PaletteDiff {
        offset,
        boxer: boxer_name,
        asset_id,
        colors,
    })
}

/// Get sprite bin diff for a specific offset
#[tauri::command]
pub fn get_sprite_bin_diff_comparison(
    state: State<AppState>,
    pc_offset: String,
) -> Result<SpriteDiff, String> {
    let offset = parse_offset(&pc_offset)?;

    let rom_opt = state.rom.lock();
    let rom = rom_opt.as_ref().ok_or("No ROM loaded")?;
    let pending = state.pending_writes.lock();
    let manifest = state.manifest.lock();

    // Find bin info from manifest
    let mut bin_name = String::new();
    let mut boxer_name = String::new();
    let mut bin_size = 0usize;

    for (fighter_name, boxer) in &manifest.fighters {
        for bin in boxer
            .unique_sprite_bins
            .iter()
            .chain(boxer.shared_sprite_bins.iter())
        {
            if bin.start_pc == pc_offset {
                bin_name = bin.filename.clone();
                boxer_name = fighter_name.clone();
                bin_size = bin.size;
                break;
            }
        }
    }

    let original_bytes = rom
        .read_bytes(offset, bin_size)
        .map_err(|e| e.to_string())?;
    let modified_bytes = if let Some(edited) = pending.get(&pc_offset) {
        edited.clone()
    } else {
        original_bytes.to_vec()
    };

    let total_tiles = original_bytes.len() / 32;
    let changed_indices = ComparisonEngine::compare_tiles(&original_bytes, &modified_bytes);

    let mut changed_tiles = Vec::new();
    for &tile_idx in &changed_indices {
        let tile_start = tile_idx * 32;
        let mut pixel_diffs = Vec::new();

        // Compare each byte in the tile
        for i in 0..32 {
            let idx = tile_start + i;
            let orig = original_bytes.get(idx).copied().unwrap_or(0);
            let modified = modified_bytes.get(idx).copied().unwrap_or(0);

            let orig_low = orig & 0x0F;
            let orig_high = (orig >> 4) & 0x0F;
            let mod_low = modified & 0x0F;
            let mod_high = (modified >> 4) & 0x0F;

            let row = (i / 2) % 8;
            let col1 = (i % 2) * 4;
            let col2 = col1 + 1;

            pixel_diffs.push(PixelDiff {
                x: col1,
                y: row,
                original_pixel: orig_low,
                modified_pixel: mod_low,
                changed: orig_low != mod_low,
            });
            pixel_diffs.push(PixelDiff {
                x: col2,
                y: row,
                original_pixel: orig_high,
                modified_pixel: mod_high,
                changed: orig_high != mod_high,
            });
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

/// Get binary/hex diff for a specific offset
#[tauri::command]
pub fn get_binary_diff(
    state: State<AppState>,
    pc_offset: String,
    size: usize,
) -> Result<BinaryDiff, String> {
    let offset = parse_offset(&pc_offset)?;

    let rom_opt = state.rom.lock();
    let rom = rom_opt.as_ref().ok_or("No ROM loaded")?;
    let pending = state.pending_writes.lock();

    let original_bytes = rom.read_bytes(offset, size).map_err(|e| e.to_string())?;
    let modified_bytes = if let Some(edited) = pending.get(&pc_offset) {
        edited.clone()
    } else {
        original_bytes.to_vec()
    };

    Ok(ComparisonEngine::generate_hex_diff(
        &original_bytes,
        &modified_bytes,
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
