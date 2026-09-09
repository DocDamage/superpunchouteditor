//! Comparison Mode Commands
//!
//! Commands for comparing original vs modded ROM data.

use tauri::State;

use crate::app_state::AppState;
use crate::utils::parse_offset;
use image::{ImageBuffer, Rgba};
use rom_core::comparison::*;

#[allow(clippy::too_many_arguments)]
#[tauri::command]
pub fn render_comparison_view(
    state: State<AppState>,
    boxer_key: String,
    view_type: String,
    show_original: bool,
    show_modified: bool,
    asset_offset: Option<String>,
    palette_offset: Option<String>,
    mode: Option<String>,
) -> Result<Vec<u8>, String> {
    use super::assets::{
        all_boxer_assets, decode_asset_tiles, first_subpalette, png_bytes, render_tile_strip,
    };
    if !matches!(
        view_type.as_str(),
        "palette" | "sprite" | "portrait" | "icon"
    ) {
        return Err("Frame/animation comparison requires reconstructed frames".into());
    }
    let manifest = state.manifest.lock();
    let boxer = manifest
        .fighters
        .iter()
        .find(|(name, boxer)| {
            boxer.key == boxer_key || name.to_lowercase().replace(' ', "_") == boxer_key
        })
        .map(|(_, boxer)| boxer)
        .ok_or("Unknown comparison boxer")?;
    let requested_asset = asset_offset.as_deref().map(parse_offset).transpose()?;
    let requested_palette = palette_offset.as_deref().map(parse_offset).transpose()?;
    let palette = boxer
        .palette_files
        .iter()
        .find(|asset| {
            requested_palette.is_none() || parse_offset(&asset.start_pc).ok() == requested_palette
        })
        .cloned()
        .ok_or("Comparison palette not found")?;
    let asset = if view_type == "palette" {
        None
    } else {
        Some(
            all_boxer_assets(boxer)
                .find(|asset| {
                    requested_asset.is_some()
                        && parse_offset(&asset.start_pc).ok() == requested_asset
                })
                .cloned()
                .ok_or("Select a comparison graphic asset")?,
        )
    };
    drop(manifest);
    let guard = state.rom_session.lock();
    let session = guard.as_ref().ok_or("No ROM loaded")?;
    let current = session.materialize().map_err(|error| error.to_string())?;
    let render = |bytes: &[u8]| -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, String> {
        let palette_pc = parse_offset(&palette.start_pc)?;
        let range = rom_core::validate_range(palette_pc, palette.size, bytes.len())
            .map_err(|error| error.to_string())?;
        if palette.size == 0 || palette.size % 2 != 0 {
            return Err("Invalid comparison palette length".into());
        }
        if palette.size > 512 {
            return Err("Comparison palette exceeds 256 colors".into());
        }
        let colors = asset_core::decode_palette(&bytes[range]);
        if let Some(asset) = &asset {
            let pc = parse_offset(&asset.start_pc)?;
            let range = rom_core::validate_range(pc, asset.size, bytes.len())
                .map_err(|error| error.to_string())?;
            let tiles = decode_asset_tiles(
                &bytes[range],
                asset.category.contains("Compressed") || asset.subtype == "compressed_sprite_bin",
            )?;
            if tiles.len() > 65_536 {
                return Err("Comparison tile sheet is too large".into());
            }
            Ok(render_tile_strip(&tiles, &first_subpalette(&colors), 16))
        } else {
            let mut image = ImageBuffer::new(16 * 16, (colors.len().div_ceil(16) * 16) as u32);
            for (index, color) in colors.iter().enumerate() {
                for y in 0..16 {
                    for x in 0..16 {
                        image.put_pixel(
                            ((index % 16) * 16 + x) as u32,
                            ((index / 16) * 16 + y) as u32,
                            Rgba([color.r, color.g, color.b, 255]),
                        );
                    }
                }
            }
            Ok(image)
        }
    };
    let original = render(session.base().bytes())?;
    let modified = render(&current.bytes)?;
    png_bytes(&compose_comparison(
        &original,
        &modified,
        mode.as_deref().unwrap_or("side-by-side"),
        show_original,
        show_modified,
    )?)
}

fn compose_comparison(
    original: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    modified: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    mode: &str,
    show_original: bool,
    show_modified: bool,
) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, String> {
    if !matches!(
        mode,
        "side-by-side" | "split" | "overlay" | "difference" | "blink"
    ) {
        return Err("Unknown comparison mode".into());
    }
    if !show_original && !show_modified {
        return Err("Select at least one comparison image".into());
    }
    if mode == "split" && show_original != show_modified {
        let width = original.width().max(modified.width());
        let height = original.height().max(modified.height());
        if u64::from(width) * u64::from(height) > 16_777_216 {
            return Err("Comparison image is too large".into());
        }
        let source = if show_original { original } else { modified };
        return Ok(ImageBuffer::from_fn(width, height, |x, y| {
            source
                .get_pixel_checked(x, y)
                .copied()
                .unwrap_or(Rgba([0, 0, 0, 0]))
        }));
    }
    if !show_modified {
        return Ok(original.clone());
    }
    if !show_original {
        return Ok(modified.clone());
    }
    let width = original.width().max(modified.width());
    let height = original.height().max(modified.height());
    let side_by_side = mode == "side-by-side";
    let output_width = if side_by_side {
        width.checked_mul(2).ok_or("Comparison width overflow")?
    } else {
        width
    };
    if u64::from(output_width) * u64::from(height) > 16_777_216 {
        return Err("Comparison image is too large".into());
    }
    let mut output = ImageBuffer::new(output_width, height);
    let pixel = |image: &ImageBuffer<Rgba<u8>, Vec<u8>>, x, y| {
        image
            .get_pixel_checked(x, y)
            .copied()
            .unwrap_or(Rgba([0, 0, 0, 0]))
    };
    for y in 0..height {
        for x in 0..width {
            let a = pixel(original, x, y);
            let b = pixel(modified, x, y);
            if side_by_side {
                output.put_pixel(x, y, a);
                output.put_pixel(x + width, y, b);
            } else {
                let result = match mode {
                    "split" => {
                        if x < width / 2 {
                            a
                        } else {
                            b
                        }
                    }
                    "overlay" => Rgba(std::array::from_fn(|i| {
                        ((u16::from(a[i]) + u16::from(b[i])) / 2) as u8
                    })),
                    "difference" => {
                        if a == b {
                            Rgba([0, 0, 0, 255])
                        } else {
                            Rgba([
                                a[0].abs_diff(b[0]).max(a[3].abs_diff(b[3])),
                                a[1].abs_diff(b[1]),
                                a[2].abs_diff(b[2]),
                                255,
                            ])
                        }
                    }
                    _ => b,
                };
                output.put_pixel(x, y, result);
            }
        }
    }
    Ok(output)
}

/// Generate a full comparison between the immutable base ROM and the exact current revision.
#[tauri::command]
pub fn generate_comparison(state: State<AppState>) -> Result<RomComparison, String> {
    generate_comparison_internal(&state)
}

fn generate_comparison_internal(state: &AppState) -> Result<RomComparison, String> {
    let manifest = state.manifest.lock().clone();
    let session_guard = state.rom_session.lock();
    let session = session_guard.as_ref().ok_or("No ROM loaded")?;
    let materialized = session.materialize().map_err(|error| error.to_string())?;
    let base = session.base();
    let mut comparison =
        RomComparison::new(base.sha1().to_string(), materialized.current_sha1.clone());

    for range in &materialized.change_ranges {
        let original = &base.bytes()[range.start.min(base.len())..range.end.min(base.len())];
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
            bytes_changed: changed,
            description: format!(
                "Canonical journal change at 0x{:X} (revision {})",
                range.start, materialized.revision
            ),
        });
    }

    let exact_bytes_changed = comparison.summary.total_bytes_changed;
    let mut fighters: Vec<_> = manifest.fighters.iter().collect();
    fighters.sort_by(|a, b| a.0.cmp(b.0));
    let mut seen = std::collections::HashSet::new();
    for (name, boxer) in fighters {
        for (asset, palette) in boxer.palette_files.iter().map(|asset| (asset, true)).chain(
            boxer
                .unique_sprite_bins
                .iter()
                .chain(&boxer.shared_sprite_bins)
                .map(|asset| (asset, false)),
        ) {
            let Ok(offset) = parse_offset(&asset.start_pc) else {
                continue;
            };
            if !seen.insert((offset, asset.size, palette)) {
                continue;
            }
            let Some(end) = offset.checked_add(asset.size) else {
                continue;
            };
            let (Some(before), Some(after)) = (
                base.bytes().get(offset..end),
                materialized.bytes.get(offset..end),
            ) else {
                continue;
            };
            if before == after {
                continue;
            }
            if let Some(diff) =
                asset_difference(asset, name, &boxer.key, offset, palette, before, after)
            {
                comparison.add_difference(diff);
            }
        }
    }
    // Semantic entries are additional views of the same bytes, not extra edits.
    comparison.summary.total_bytes_changed = exact_bytes_changed;
    Ok(comparison)
}

fn asset_difference(
    asset: &manifest_core::AssetFile,
    boxer: &str,
    boxer_key: &str,
    offset: usize,
    palette: bool,
    before: &[u8],
    after: &[u8],
) -> Option<Difference> {
    if palette {
        if before.len() % 2 != 0 || before.len() != after.len() {
            return None;
        }
        let colors = |bytes: &[u8]| {
            bytes
                .chunks_exact(2)
                .map(|pair| ColorDiff::from_snes_bytes(pair[0], pair[1]))
                .collect::<Vec<_>>()
        };
        let original_colors = colors(before);
        let modified_colors = colors(after);
        let changed_indices: Vec<_> = original_colors
            .iter()
            .zip(&modified_colors)
            .enumerate()
            .filter(|(_, (a, b))| (a.r, a.g, a.b) != (b.r, b.g, b.b))
            .map(|(i, _)| i)
            .collect();
        if changed_indices.is_empty() {
            return None;
        }
        Some(Difference::Palette {
            offset,
            asset_id: format!("{boxer_key}/{}", asset.filename),
            boxer: boxer.into(),
            original_colors,
            modified_colors,
            changed_indices,
        })
    } else {
        let compressed =
            asset.category.contains("Compressed") || asset.subtype == "compressed_sprite_bin";
        let (total_tiles, tiles) = compare_sprite_payloads(before, after, compressed).ok()?;
        if tiles.is_empty() {
            return None;
        }
        Some(Difference::Sprite {
            boxer: boxer.into(),
            bin_name: asset.filename.clone(),
            pc_offset: offset,
            total_tiles,
            changed_tile_indices: tiles.iter().map(|tile| tile.tile_index).collect(),
            tile_change_counts: tiles
                .iter()
                .map(|tile| {
                    (
                        tile.tile_index,
                        tile.pixel_diffs
                            .iter()
                            .filter(|pixel| pixel.changed)
                            .count(),
                    )
                })
                .collect(),
        })
    }
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
            .find(|palette| parse_offset(&palette.start_pc).ok() == Some(offset))
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
    let mut compressed = false;
    for (fighter_name, boxer) in &manifest.fighters {
        if let Some(bin) = boxer
            .unique_sprite_bins
            .iter()
            .chain(boxer.shared_sprite_bins.iter())
            .find(|bin| parse_offset(&bin.start_pc).ok() == Some(offset))
        {
            bin_name = bin.filename.clone();
            boxer_name = fighter_name.clone();
            bin_size = bin.size;
            compressed =
                bin.category.contains("Compressed") || bin.subtype == "compressed_sprite_bin";
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

    let (total_tiles, changed_tiles) =
        compare_sprite_payloads(original_bytes, modified_bytes, compressed)?;

    Ok(SpriteDiff {
        pc_offset: offset,
        boxer: boxer_name,
        bin_name,
        total_tiles,
        changed_tiles,
    })
}

fn compare_sprite_payloads(
    original: &[u8],
    modified: &[u8],
    compressed: bool,
) -> Result<(usize, Vec<TileDiff>), String> {
    let original = super::assets::decode_asset_tiles(original, compressed)?;
    let modified = super::assets::decode_asset_tiles(modified, compressed)?;
    let total = original.len().max(modified.len());
    let changed = (0..total)
        .filter(|&index| original.get(index) != modified.get(index))
        .map(|index| {
            let before = original.get(index);
            let after = modified.get(index);
            let pixel_diffs = (0..64)
                .map(|pixel| {
                    let a = before.map(|tile| tile.pixels[pixel]);
                    let b = after.map(|tile| tile.pixels[pixel]);
                    PixelDiff {
                        x: pixel % 8,
                        y: pixel / 8,
                        original_pixel: a.unwrap_or(0),
                        modified_pixel: b.unwrap_or(0),
                        changed: a != b,
                    }
                })
                .collect();
            TileDiff {
                tile_index: index,
                pixel_diffs,
                has_changes: true,
            }
        })
        .collect();
    Ok((total, changed))
}

#[cfg(test)]
fn tile_pixel_diffs(original: &[u8], modified: &[u8]) -> Vec<PixelDiff> {
    let original = asset_core::decode_4bpp_tile(original);
    let modified = asset_core::decode_4bpp_tile(modified);
    original
        .pixels
        .iter()
        .zip(&modified.pixels)
        .enumerate()
        .map(|(index, (&a, &b))| PixelDiff {
            x: index % 8,
            y: index / 8,
            original_pixel: a,
            modified_pixel: b,
            changed: a != b,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn separate_split_layers_have_matching_canvas_sizes_without_scaling() {
        let original = ImageBuffer::from_pixel(1, 1, Rgba([20, 40, 60, 255]));
        let modified = ImageBuffer::from_pixel(2, 3, Rgba([100, 80, 60, 255]));
        let a = compose_comparison(&original, &modified, "split", true, false).unwrap();
        let b = compose_comparison(&original, &modified, "split", false, true).unwrap();
        assert_eq!(a.dimensions(), b.dimensions());
        assert_eq!(a.dimensions(), (2, 3));
        assert_eq!(a.get_pixel(0, 0), original.get_pixel(0, 0));
        assert_eq!(a.get_pixel(1, 2).0, [0, 0, 0, 0]);
        assert_eq!(b, modified);
    }

    #[test]
    fn journal_comparison_exposes_visual_assets_without_double_counting_bytes() {
        let asset = |name: &str, offset: &str, size| manifest_core::AssetFile {
            file: name.into(),
            filename: name.into(),
            category: "Raw".into(),
            subtype: String::new(),
            size,
            start_snes: String::new(),
            end_snes: String::new(),
            start_pc: offset.into(),
            end_pc: String::new(),
            shared_with: vec![],
        };
        let boxer = manifest_core::BoxerRecord {
            name: "Gabby Jay".into(),
            key: "gabby_jay".into(),
            reference_sheet: String::new(),
            palette_files: vec![asset("palette", "0x0", 32)],
            unique_sprite_bins: vec![asset("tiles", "0x20", 32)],
            shared_sprite_bins: vec![],
            icon_files: vec![],
            portrait_files: vec![],
            large_portrait_files: vec![],
            other_files: vec![],
        };
        let mut manifest = manifest_core::Manifest::empty();
        manifest.fighters.insert("Gabby Jay".into(), boxer.clone());
        manifest.fighters.insert("Shared Owner".into(), boxer);
        let state = AppState::new(manifest);
        state.install_rom_session(rom_core::Rom::new(vec![0; 64]), "synthetic.sfc".into());
        state
            .commit_rom_transform("Palette and tile", |rom| {
                rom.data[0] = 31;
                rom.data[32] = 128;
                Ok(())
            })
            .unwrap();
        let comparison = generate_comparison_internal(&state).unwrap();
        assert_eq!(comparison.summary.total_bytes_changed, 2);
        assert_eq!(comparison.summary.palettes_modified, 1);
        assert_eq!(comparison.summary.sprite_bins_changed, 1);
        assert_eq!(comparison.summary.tiles_changed, 1);
        assert!(comparison.differences.iter().any(|diff| matches!(diff, Difference::Palette { changed_indices, .. } if changed_indices == &[0])));
        assert!(comparison.differences.iter().any(|diff| matches!(diff, Difference::Sprite { tile_change_counts, .. } if tile_change_counts.get(&0) == Some(&1))));
        state.undo_journal().unwrap();
        assert!(generate_comparison_internal(&state)
            .unwrap()
            .differences
            .is_empty());
    }

    #[test]
    fn compressed_sprite_comparison_reports_decoded_pixel_changes() {
        let before = vec![asset_core::Tile::filled(0); 2];
        let mut after = before.clone();
        after[1].pixels[63] = 15;
        let a = super::super::assets::encode_tiles_for_asset(&before, true);
        let b = super::super::assets::encode_tiles_for_asset(&after, true);
        let (total, differences) = compare_sprite_payloads(&a, &b, true).unwrap();
        assert_eq!(total, 2);
        assert_eq!(differences.len(), 1);
        assert_eq!(differences[0].tile_index, 1);
        let pixels: Vec<_> = differences[0]
            .pixel_diffs
            .iter()
            .filter(|pixel| pixel.changed)
            .collect();
        assert_eq!(pixels.len(), 1);
        assert_eq!(
            (pixels[0].x, pixels[0].y, pixels[0].modified_pixel),
            (7, 7, 15)
        );
        assert!(compare_sprite_payloads(&a, &a, true).unwrap().1.is_empty());
    }

    #[test]
    fn sprite_comparison_detects_added_and_removed_blank_tiles() {
        for (before, after) in [(vec![0; 32], vec![0; 64]), (vec![0; 64], vec![0; 32])] {
            let (total, differences) = compare_sprite_payloads(&before, &after, false).unwrap();
            assert_eq!(total, 2);
            assert_eq!(differences.len(), 1);
            assert_eq!(differences[0].tile_index, 1);
            assert!(differences[0].pixel_diffs.iter().all(|pixel| pixel.changed));
        }
        assert!(compare_sprite_payloads(&[0; 31], &[0; 32], false).is_err());
    }

    #[test]
    fn visual_composition_modes_preserve_expected_pixels() {
        let original = ImageBuffer::from_pixel(2, 1, Rgba([20, 40, 60, 255]));
        let modified = ImageBuffer::from_pixel(2, 1, Rgba([100, 80, 60, 255]));
        let side = compose_comparison(&original, &modified, "side-by-side", true, true).unwrap();
        assert_eq!(side.dimensions(), (4, 1));
        assert_eq!(side.get_pixel(0, 0), original.get_pixel(0, 0));
        assert_eq!(side.get_pixel(2, 0), modified.get_pixel(0, 0));
        let overlay = compose_comparison(&original, &modified, "overlay", true, true).unwrap();
        assert_eq!(overlay.get_pixel(0, 0).0, [60, 60, 60, 255]);
        let difference =
            compose_comparison(&original, &modified, "difference", true, true).unwrap();
        assert_eq!(difference.get_pixel(0, 0).0, [80, 40, 0, 255]);
        let split = compose_comparison(&original, &modified, "split", true, true).unwrap();
        assert_eq!(split.get_pixel(0, 0), original.get_pixel(0, 0));
        assert_eq!(split.get_pixel(1, 0), modified.get_pixel(1, 0));
        assert_eq!(
            compose_comparison(&original, &modified, "blink", true, false).unwrap(),
            original
        );
        assert_eq!(
            compose_comparison(&original, &modified, "blink", false, true).unwrap(),
            modified
        );
        assert!(compose_comparison(&original, &modified, "unknown", true, true).is_err());
        assert!(compose_comparison(&original, &modified, "split", false, false).is_err());
        let png = super::super::assets::png_bytes(&side).unwrap();
        assert_eq!(image::load_from_memory(&png).unwrap().to_rgba8(), side);
    }

    #[test]
    fn sprite_diff_decodes_all_bitplanes_and_unique_pixel_coordinates() {
        let original = [0u8; 32];
        let mut modified = original;
        modified[0] = 0x80;
        modified[1] = 0x80;
        modified[16] = 0x80;
        modified[17] = 0x80;
        modified[31] = 1;
        let pixels = tile_pixel_diffs(&original, &modified);
        assert_eq!(pixels.len(), 64);
        assert_eq!(pixels.iter().filter(|pixel| pixel.changed).count(), 2);
        assert_eq!(
            (pixels[0].x, pixels[0].y, pixels[0].modified_pixel),
            (0, 0, 15)
        );
        assert_eq!(
            (pixels[63].x, pixels[63].y, pixels[63].modified_pixel),
            (7, 7, 8)
        );
        for (index, pixel) in pixels.iter().enumerate() {
            assert_eq!((pixel.x, pixel.y), (index % 8, index / 8));
        }
    }

    #[test]
    fn comparison_handles_journal_expansion_and_undo() {
        let state = AppState::new(manifest_core::Manifest::empty());
        state.install_rom_session(rom_core::Rom::new(vec![0; 32]), "synthetic.sfc".into());
        state
            .commit_rom_transform("Expand", |rom| {
                rom.data.resize(64, 0);
                rom.data[60] = 1;
                Ok(())
            })
            .unwrap();
        let comparison = generate_comparison_internal(&state).unwrap();
        assert_eq!(comparison.summary.total_bytes_changed, 32);
        state.undo_journal().unwrap();
        assert_eq!(
            generate_comparison_internal(&state)
                .unwrap()
                .summary
                .total_bytes_changed,
            0
        );
    }
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
