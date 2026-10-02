//! Asset Commands
//!
//! Commands and helpers for working with game assets (palettes, sprites, portraits).

pub mod palettes;
pub mod portraits;
pub mod sprites;

use std::io::Cursor;

use asset_core::{
    encode_4bpp_sheet, encode_palette, image_to_tiles, tiles_to_image, Color, Decompressor, Tile,
    END_OF_STREAM,
};
use image::{ImageBuffer, Rgba};
use manifest_core::{AssetFile, BoxerRecord, Manifest};
use serde::Serialize;
use tauri::State;

use crate::app_state::AppState;
use crate::utils::parse_offset;

#[allow(ambiguous_glob_reexports)]
pub use palettes::*;
#[allow(ambiguous_glob_reexports)]
pub use portraits::*;
#[allow(ambiguous_glob_reexports)]
pub use sprites::*;

pub type AssetResult<T> = Result<T, String>;
pub const DEFAULT_TILE_STRIP_WIDTH: usize = 16;
pub const DEFAULT_SUBPALETTE_SIZE: usize = 16;

#[derive(Debug, Clone, Serialize)]
pub struct RuntimeThemeAssets {
    pub boxer_key: String,
    pub boxer_name: String,
    pub palette: Vec<Color>,
    pub icon_png: Option<Vec<u8>>,
    pub portrait_png: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Copy)]
pub struct CompressionPassInfo {
    pub consumed: usize,
    pub written: usize,
}

pub fn read_original_rom_bytes(
    state: &AppState,
    pc_offset: usize,
    size: usize,
) -> AssetResult<Vec<u8>> {
    let session_guard = state.rom_session.lock();
    let session = session_guard.as_ref().ok_or("No ROM loaded")?;
    let range = rom_core::validate_range(pc_offset, size, session.base().len())
        .map_err(|error| error.to_string())?;
    Ok(session.base().bytes()[range].to_vec())
}

pub fn read_current_asset_bytes(
    state: &AppState,
    pc_offset: usize,
    size: usize,
) -> AssetResult<Vec<u8>> {
    let materialized = state.materialize_current_rom()?;
    let range = rom_core::validate_range(pc_offset, size, materialized.bytes.len())
        .map_err(|error| error.to_string())?;
    Ok(materialized.bytes[range].to_vec())
}

pub fn read_palette_colors(
    state: &AppState,
    palette_pc_offset: usize,
    palette_size: usize,
) -> AssetResult<Vec<Color>> {
    let bytes = read_current_asset_bytes(state, palette_pc_offset, palette_size)?;
    Ok(asset_core::decode_palette(&bytes))
}

pub fn first_subpalette(colors: &[Color]) -> Vec<Color> {
    let mut palette = colors
        .iter()
        .copied()
        .take(DEFAULT_SUBPALETTE_SIZE)
        .collect::<Vec<_>>();

    if palette.is_empty() {
        palette.push(Color::new(0, 0, 0));
    }

    while palette.len() < DEFAULT_SUBPALETTE_SIZE {
        palette.push(Color::new(0, 0, 0));
    }

    palette
}

pub fn save_png(img: &ImageBuffer<Rgba<u8>, Vec<u8>>, output_path: &str) -> AssetResult<()> {
    img.save(output_path).map_err(|e| e.to_string())
}

pub fn png_bytes(img: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> AssetResult<Vec<u8>> {
    let mut bytes = Vec::new();
    let mut cursor = Cursor::new(&mut bytes);
    img.write_to(&mut cursor, image::ImageFormat::Png)
        .map_err(|e| e.to_string())?;
    Ok(bytes)
}

/// Number of 8x8 tiles in a boxer's small face icon (32x32 pixels).
pub const ICON_TILE_COUNT: usize = 16;
/// Tiles per row when an icon is shown on screen.
pub const ICON_WIDTH_TILES: usize = 4;

/// Convert a small face icon between the order its tiles are stored in the ROM
/// and the order they appear on screen.
///
/// The game keeps an icon as four 16x16 sprites laid out for video memory:
/// the first eight tiles are the top halves of the four sprites and the next
/// eight are the bottom halves. Read in file order as a 4-tile-wide picture,
/// the second and third rows are therefore swapped. Swapping them back gives
/// the real picture. The swap is its own inverse, so the same function
/// converts in both directions. Anything that is not a 16-tile icon is
/// returned unchanged.
pub fn icon_tiles_swap_layout(tiles: &[Tile]) -> Vec<Tile> {
    if tiles.len() != ICON_TILE_COUNT {
        return tiles.to_vec();
    }
    let row = ICON_WIDTH_TILES;
    let mut out = Vec::with_capacity(ICON_TILE_COUNT);
    out.extend_from_slice(&tiles[..row]);
    out.extend_from_slice(&tiles[2 * row..3 * row]);
    out.extend_from_slice(&tiles[row..2 * row]);
    out.extend_from_slice(&tiles[3 * row..]);
    out
}

/// True when the asset at `pc_offset` is a boxer's small face icon.
pub fn is_icon_asset(state: &AppState, pc_offset: usize) -> bool {
    let manifest = state.manifest.lock();
    find_asset_by_offset(&manifest, pc_offset).is_some_and(|asset| asset.subtype == "icon")
}

/// Largest picture (per side, in pixels) accepted for import. Anything bigger is
/// almost certainly a wrong file, and decoding it would only waste memory.
pub const MAX_IMPORT_DIMENSION: u32 = 4096;

/// How an imported picture was fitted to the size the game expects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ImageFit {
    /// True when the picture had to be scaled or padded to fit.
    pub resized: bool,
    pub source_width: u32,
    pub source_height: u32,
    pub target_width: u32,
    pub target_height: u32,
}

/// Result returned to the frontend after importing a picture into an asset slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ImportOutcome {
    pub new_size: usize,
    pub original_size: usize,
    pub fits: bool,
    pub fit: ImageFit,
}

/// Tile-strip width used when the caller does not name one. Icons are 4 tiles
/// (32 px) wide; every other boxer graphic uses the 16-tile sheet width.
pub fn default_width_tiles(tile_count: usize) -> usize {
    if tile_count <= 16 {
        4
    } else {
        DEFAULT_TILE_STRIP_WIDTH
    }
}

/// Pixel size of a sheet holding `tile_count` 8x8 tiles laid out `width_tiles` wide.
pub fn asset_pixel_size(tile_count: usize, width_tiles: usize) -> AssetResult<(u32, u32)> {
    if tile_count == 0 {
        return Err("This picture slot is empty, so there is nothing to replace".to_string());
    }
    let width_tiles = width_tiles.clamp(1, tile_count);
    let rows = tile_count.div_ceil(width_tiles);
    let width = u32::try_from(width_tiles * 8).map_err(|_| "Picture is too wide".to_string())?;
    let height = u32::try_from(rows * 8).map_err(|_| "Picture is too tall".to_string())?;
    Ok((width, height))
}

/// Fit any picture to exactly `target_width` x `target_height`.
///
/// The picture keeps its proportions and fills the whole area; whatever hangs
/// over the edges is trimmed equally from both sides, so there are never empty
/// bars. A picture that already has the right size is returned untouched.
/// Pixel art is enlarged with hard edges; larger pictures are smoothed while
/// shrinking.
pub fn fit_image_to_canvas(
    img: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    target_width: u32,
    target_height: u32,
) -> (ImageBuffer<Rgba<u8>, Vec<u8>>, ImageFit) {
    let fit = ImageFit {
        resized: img.width() != target_width || img.height() != target_height,
        source_width: img.width(),
        source_height: img.height(),
        target_width,
        target_height,
    };

    if !fit.resized || img.width() == 0 || img.height() == 0 {
        return (img.clone(), fit);
    }

    let scale = f64::max(
        f64::from(target_width) / f64::from(img.width()),
        f64::from(target_height) / f64::from(img.height()),
    );
    // Rounding can land one pixel short; never go below the target size.
    let scaled_width = ((f64::from(img.width()) * scale).round() as u32).max(target_width);
    let scaled_height = ((f64::from(img.height()) * scale).round() as u32).max(target_height);
    let filter = if scale >= 1.0 {
        image::imageops::FilterType::Nearest
    } else {
        image::imageops::FilterType::Triangle
    };
    let scaled = image::imageops::resize(img, scaled_width, scaled_height, filter);

    let left = (scaled_width - target_width) / 2;
    let top = (scaled_height - target_height) / 2;
    let cropped =
        image::imageops::crop_imm(&scaled, left, top, target_width, target_height).to_image();
    (cropped, fit)
}

/// Open a picture and turn it into exactly `tile_count` tiles, resizing and
/// recolouring it to the nearest palette colours as needed.
pub fn load_png_fitted_as_tiles(
    png_path: &str,
    palette: &[Color],
    tile_count: usize,
    width_tiles: usize,
) -> AssetResult<(Vec<Tile>, ImageFit)> {
    let img = image::open(png_path)
        .map_err(|e| format!("Could not open the picture '{}': {}", png_path, e))?
        .to_rgba8();

    if img.width() == 0 || img.height() == 0 {
        return Err("That picture is empty".to_string());
    }
    if img.width() > MAX_IMPORT_DIMENSION || img.height() > MAX_IMPORT_DIMENSION {
        return Err(format!(
            "That picture is too big ({}x{}). Use one that is at most {MAX_IMPORT_DIMENSION} pixels on each side",
            img.width(),
            img.height()
        ));
    }

    let (target_width, target_height) = asset_pixel_size(tile_count, width_tiles)?;
    let (fitted, fit) = fit_image_to_canvas(&img, target_width, target_height);
    let mut tiles = image_to_tiles(&fitted, palette);
    tiles.truncate(tile_count);
    Ok((tiles, fit))
}

pub fn load_png_as_tiles(png_path: &str, palette: &[Color]) -> AssetResult<Vec<Tile>> {
    let img = image::open(png_path)
        .map_err(|e| format!("Failed to open PNG '{}': {}", png_path, e))?
        .to_rgba8();

    if img.width() % 8 != 0 || img.height() % 8 != 0 {
        return Err(format!(
            "PNG dimensions must be multiples of 8 pixels, got {}x{}",
            img.width(),
            img.height()
        ));
    }

    Ok(image_to_tiles(&img, palette))
}

pub fn encode_tiles_to_snes_bytes(tiles: &[Tile]) -> Vec<u8> {
    encode_4bpp_sheet(tiles)
}

pub fn find_asset_by_offset(manifest: &Manifest, pc_offset: usize) -> Option<AssetFile> {
    manifest
        .fighters
        .values()
        .flat_map(all_boxer_assets)
        .find(|asset| parse_offset(&asset.start_pc).ok() == Some(pc_offset))
        .cloned()
}

pub fn find_boxer_by_key(manifest: &Manifest, boxer_key: &str) -> Option<BoxerRecord> {
    manifest
        .fighters
        .values()
        .find(|boxer| boxer.key == boxer_key)
        .cloned()
}

pub fn all_boxer_assets(boxer: &BoxerRecord) -> impl Iterator<Item = &AssetFile> {
    boxer
        .palette_files
        .iter()
        .chain(boxer.icon_files.iter())
        .chain(boxer.portrait_files.iter())
        .chain(boxer.large_portrait_files.iter())
        .chain(boxer.unique_sprite_bins.iter())
        .chain(boxer.shared_sprite_bins.iter())
        .chain(boxer.other_files.iter())
}

pub fn is_compressed_category(category: &str) -> bool {
    category.contains("Compressed")
}

pub fn analyze_hal8_pass(input: &[u8]) -> AssetResult<CompressionPassInfo> {
    let mut decompressor = Decompressor::new(input);
    let bytes = decompressor.decompress_sprite_graphics_exact()?;
    Ok(CompressionPassInfo {
        consumed: decompressor.position(),
        written: bytes.len(),
    })
}

pub fn decompress_interleaved_exact(bytes: &[u8]) -> AssetResult<Vec<u8>> {
    let mut decompressor = Decompressor::new(bytes);
    decompressor
        .decompress_interleaved_exact()
        .map_err(|error| format!("SPO graphics decompression failed: {error}"))
}

fn byte_rle_len(data: &[u8], start: usize) -> usize {
    let byte = data[start];
    let mut len = 1usize;
    while start + len < data.len() && len < 32 && data[start + len] == byte {
        len += 1;
    }
    len
}

fn word_rle_len(data: &[u8], start: usize) -> usize {
    if start + 3 >= data.len() {
        return 0;
    }

    let b1 = data[start];
    let b2 = data[start + 1];
    let mut pairs = 1usize;
    let mut pos = start + 2;

    while pos + 1 < data.len() && pairs < 32 && data[pos] == b1 && data[pos + 1] == b2 {
        pairs += 1;
        pos += 2;
    }

    pairs
}

fn incremental_len(data: &[u8], start: usize) -> usize {
    let mut len = 1usize;
    let mut expected = data[start].wrapping_add(1);
    while start + len < data.len() && len < 32 && data[start + len] == expected {
        len += 1;
        expected = expected.wrapping_add(1);
    }
    len
}

pub fn compress_hal8_pass(data: &[u8]) -> Vec<u8> {
    let mut output = Vec::new();
    let mut i = 0usize;

    while i < data.len() {
        let byte_run = byte_rle_len(data, i);
        if byte_run >= 4 {
            output.push(0x20 | ((byte_run - 1) as u8));
            output.push(data[i]);
            i += byte_run;
            continue;
        }

        let word_run = word_rle_len(data, i);
        if word_run >= 3 {
            output.push(0x40 | ((word_run - 1) as u8));
            output.push(data[i]);
            output.push(data[i + 1]);
            i += word_run * 2;
            continue;
        }

        let inc_run = incremental_len(data, i);
        if inc_run >= 4 {
            output.push(0x60 | ((inc_run - 1) as u8));
            output.push(data[i]);
            i += inc_run;
            continue;
        }

        let literal_start = i;
        let mut literal_len = 0usize;
        while i < data.len() && literal_len < 32 {
            let next_byte_run = byte_rle_len(data, i);
            let next_word_run = word_rle_len(data, i);
            let next_inc_run = incremental_len(data, i);
            if literal_len > 0 && (next_byte_run >= 4 || next_word_run >= 3 || next_inc_run >= 4) {
                break;
            }
            i += 1;
            literal_len += 1;
        }

        output.push((literal_len - 1) as u8);
        output.extend_from_slice(&data[literal_start..literal_start + literal_len]);
    }

    output.push(END_OF_STREAM);
    output
}

pub fn compress_interleaved(bytes: &[u8]) -> Vec<u8> {
    asset_core::compress_sprite_graphics(bytes)
}

pub fn decode_asset_tiles(bytes: &[u8], compressed: bool) -> AssetResult<Vec<Tile>> {
    let raw = if compressed {
        decompress_interleaved_exact(bytes)?
    } else {
        bytes.to_vec()
    };

    if raw.len() % 32 != 0 {
        return Err(format!(
            "Asset data is not aligned to 32-byte 4bpp tiles ({} bytes)",
            raw.len()
        ));
    }

    Ok(asset_core::decode_4bpp_sheet(&raw))
}

pub fn encode_tiles_for_asset(tiles: &[Tile], compressed: bool) -> Vec<u8> {
    let raw = encode_tiles_to_snes_bytes(tiles);
    if compressed {
        compress_interleaved(&raw)
    } else {
        raw
    }
}

pub fn current_tile_diff(original_tiles: &[Tile], current_tiles: &[Tile]) -> Vec<bool> {
    let tile_count = original_tiles.len().max(current_tiles.len());
    (0..tile_count)
        .map(|idx| original_tiles.get(idx) != current_tiles.get(idx))
        .collect()
}

pub fn render_tile_strip(
    tiles: &[Tile],
    palette: &[Color],
    width_tiles: usize,
) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    tiles_to_image(tiles, width_tiles.max(1), palette)
}

pub fn render_asset_png_bytes(
    state: &AppState,
    asset: &AssetFile,
    palette: &[Color],
    width_tiles: usize,
) -> AssetResult<Vec<u8>> {
    let asset_pc_offset = parse_offset(&asset.start_pc)?;
    let bytes = read_current_asset_bytes(state, asset_pc_offset, asset.size)?;
    let mut tiles = decode_asset_tiles(&bytes, asset.category.contains("Compressed"))?;
    if asset.subtype == "icon" {
        tiles = icon_tiles_swap_layout(&tiles);
    }
    let img = render_tile_strip(&tiles, palette, width_tiles);
    png_bytes(&img)
}

pub fn encode_palette_bytes(colors: &[Color]) -> Vec<u8> {
    encode_palette(colors)
}

pub fn set_pending_write(state: &AppState, pc_offset: usize, bytes: Vec<u8>) -> AssetResult<()> {
    state
        .commit_rom_write(
            format!("Import asset at 0x{pc_offset:X}"),
            pc_offset,
            bytes,
            Some(format!("asset@0x{pc_offset:X}")),
            Some("Imported graphic/sprite asset".to_string()),
        )
        .map(|_| ())
}

fn preferred_theme_boxer(manifest: &Manifest) -> Option<BoxerRecord> {
    ["hoy_quarlow", "narcis_prince"]
        .iter()
        .find_map(|key| find_boxer_by_key(manifest, key))
        .or_else(|| {
            manifest
                .fighters
                .values()
                .cloned()
                .collect::<Vec<_>>()
                .into_iter()
                .min_by(|a, b| a.name.cmp(&b.name))
        })
}

#[tauri::command]
pub fn get_runtime_theme_assets(
    state: State<AppState>,
    boxer_key: Option<String>,
) -> AssetResult<RuntimeThemeAssets> {
    let manifest = state.manifest.lock();
    let boxer = boxer_key
        .as_deref()
        .and_then(|key| find_boxer_by_key(&manifest, key))
        .or_else(|| preferred_theme_boxer(&manifest))
        .ok_or_else(|| "No boxer data available to build runtime theme".to_string())?;
    drop(manifest);

    let palette_asset = boxer
        .palette_files
        .first()
        .ok_or_else(|| format!("Boxer '{}' has no palette assets", boxer.name))?;
    let palette_pc = parse_offset(&palette_asset.start_pc)?;
    let palette = first_subpalette(&read_palette_colors(
        state.inner(),
        palette_pc,
        palette_asset.size,
    )?);

    // Each picture is optional on its own: one that cannot be drawn must not
    // hide the other.
    let icon_png = boxer
        .icon_files
        .first()
        .and_then(|asset| render_asset_png_bytes(state.inner(), asset, &palette, 4).ok());

    let portrait_png = boxer
        .large_portrait_files
        .first()
        .or_else(|| boxer.portrait_files.first())
        .and_then(|asset| render_asset_png_bytes(state.inner(), asset, &palette, 16).ok());

    Ok(RuntimeThemeAssets {
        boxer_key: boxer.key,
        boxer_name: boxer.name,
        palette,
        icon_png,
        portrait_png,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sprite_graphics_roundtrip_preserves_data() {
        let data = (0..512)
            .map(|idx| ((idx * 7) & 0xFF) as u8)
            .collect::<Vec<_>>();
        let compressed = compress_interleaved(&data);
        let decompressed = decompress_interleaved_exact(&compressed).unwrap();
        assert_eq!(decompressed, data);
    }

    fn numbered_tiles(count: usize) -> Vec<Tile> {
        (0..count)
            .map(|index| Tile {
                pixels: vec![index as u8; 64],
            })
            .collect()
    }

    #[test]
    fn icon_layout_swaps_the_two_middle_rows() {
        let stored = numbered_tiles(ICON_TILE_COUNT);
        let shown = icon_tiles_swap_layout(&stored);
        let order: Vec<u8> = shown.iter().map(|tile| tile.pixels[0]).collect();
        assert_eq!(
            order,
            vec![0, 1, 2, 3, 8, 9, 10, 11, 4, 5, 6, 7, 12, 13, 14, 15]
        );
    }

    /// How abruptly the picture changes across the three seams between its
    /// four rows of tiles. A face drawn in the right order is smooth there.
    fn seam_roughness(img: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> u64 {
        let mut total = 0u64;
        for seam in [8u32, 16, 24] {
            for x in 0..img.width() {
                let above = img.get_pixel(x, seam - 1);
                let below = img.get_pixel(x, seam);
                for channel in 0..3 {
                    total += u64::from(above[channel].abs_diff(below[channel]));
                }
            }
        }
        total
    }

    /// Opt-in check against a user-supplied USA ROM (`SPO_USA_ROM`): every
    /// boxer's small face must be smoother across tile rows in the corrected
    /// layout than in raw file order. Set `SPO_PROBE_DIR` to also write the
    /// drawn faces there for a visual check. Copyrighted bytes never enter the
    /// repository.
    #[test]
    #[ignore]
    fn usa_small_faces_are_drawn_in_the_right_tile_order() {
        let rom_path = std::env::var("SPO_USA_ROM").expect("set SPO_USA_ROM to a local USA ROM");
        let rom = rom_core::Rom::load(&rom_path).unwrap();
        let region = rom.detect_region().expect("recognised region");
        let manifest = crate::utils::load_manifest_for_region(region, None).unwrap();
        let state = AppState::new(manifest.clone());
        state.install_rom_session(rom, rom_path);

        let mut checked = 0;
        let mut smoother = 0;
        for boxer in manifest.fighters.values() {
            let (Some(icon), Some(palette_asset)) =
                (boxer.icon_files.first(), boxer.palette_files.first())
            else {
                continue;
            };
            let palette = first_subpalette(
                &read_palette_colors(
                    &state,
                    parse_offset(&palette_asset.start_pc).unwrap(),
                    palette_asset.size,
                )
                .unwrap(),
            );

            let png = render_asset_png_bytes(&state, icon, &palette, ICON_WIDTH_TILES).unwrap();
            let drawn = image::load_from_memory(&png).unwrap().to_rgba8();
            assert_eq!((drawn.width(), drawn.height()), (32, 32));

            let raw_bytes =
                read_current_asset_bytes(&state, parse_offset(&icon.start_pc).unwrap(), icon.size)
                    .unwrap();
            let raw = render_tile_strip(
                &decode_asset_tiles(&raw_bytes, false).unwrap(),
                &palette,
                ICON_WIDTH_TILES,
            );

            checked += 1;
            if seam_roughness(&drawn) < seam_roughness(&raw) {
                smoother += 1;
            }

            if let Ok(dir) = std::env::var("SPO_PROBE_DIR") {
                drawn
                    .save(std::path::Path::new(&dir).join(format!("{}.png", boxer.key)))
                    .unwrap();
            }
        }

        assert!(checked >= 16, "expected every boxer icon, found {checked}");
        assert_eq!(
            smoother, checked,
            "every small face should be smoother in the corrected layout"
        );
    }

    #[test]
    fn icon_layout_round_trips_and_leaves_other_sizes_alone() {
        let stored = numbered_tiles(ICON_TILE_COUNT);
        assert_eq!(
            icon_tiles_swap_layout(&icon_tiles_swap_layout(&stored)),
            stored
        );

        let sheet = numbered_tiles(64);
        assert_eq!(icon_tiles_swap_layout(&sheet), sheet);
    }

    #[test]
    fn right_sized_picture_is_left_untouched() {
        let img = ImageBuffer::from_pixel(32, 32, Rgba([10, 20, 30, 255]));
        let (fitted, fit) = fit_image_to_canvas(&img, 32, 32);
        assert!(!fit.resized);
        assert_eq!(fitted, img);
    }

    #[test]
    fn small_picture_is_enlarged_to_fill_the_slot() {
        let img = ImageBuffer::from_pixel(16, 16, Rgba([200, 0, 0, 255]));
        let (fitted, fit) = fit_image_to_canvas(&img, 32, 32);
        assert!(fit.resized);
        assert_eq!((fit.source_width, fit.source_height), (16, 16));
        assert_eq!((fitted.width(), fitted.height()), (32, 32));
        assert!(fitted
            .pixels()
            .all(|pixel| *pixel == Rgba([200, 0, 0, 255])));
    }

    #[test]
    fn wide_picture_fills_the_slot_and_trims_its_sides_equally() {
        // Left third red, middle third green, right third blue.
        let img = ImageBuffer::from_fn(96, 32, |x, _| match x / 32 {
            0 => Rgba([200u8, 0, 0, 255]),
            1 => Rgba([0, 200, 0, 255]),
            _ => Rgba([0, 0, 200, 255]),
        });
        let (fitted, fit) = fit_image_to_canvas(&img, 32, 32);
        assert!(fit.resized);
        assert_eq!((fitted.width(), fitted.height()), (32, 32));
        // Only the green middle survives; no pixel is left empty.
        assert!(fitted
            .pixels()
            .all(|pixel| *pixel == Rgba([0, 200, 0, 255])));
    }

    #[test]
    fn odd_sized_picture_never_overflows_the_canvas() {
        let img = ImageBuffer::from_pixel(37, 91, Rgba([1, 2, 3, 255]));
        let (fitted, _) = fit_image_to_canvas(&img, 128, 96);
        assert_eq!((fitted.width(), fitted.height()), (128, 96));
    }

    #[test]
    fn asset_pixel_size_matches_icon_and_sheet_layouts() {
        assert_eq!(default_width_tiles(16), 4);
        assert_eq!(default_width_tiles(64), 16);
        assert_eq!(asset_pixel_size(16, 4).unwrap(), (32, 32));
        assert_eq!(asset_pixel_size(40, 16).unwrap(), (128, 24));
        assert!(asset_pixel_size(0, 4).is_err());
    }

    #[test]
    fn fitted_import_produces_exactly_the_slot_tile_count() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("big.png");
        ImageBuffer::from_pixel(100, 60, Rgba([255u8, 255, 255, 255]))
            .save(&path)
            .unwrap();
        let palette = first_subpalette(&[Color::new(0, 0, 0), Color::new(255, 255, 255)]);
        let (tiles, fit) =
            load_png_fitted_as_tiles(path.to_str().unwrap(), &palette, 16, 4).unwrap();
        assert_eq!(tiles.len(), 16);
        assert!(fit.resized);
        assert_eq!((fit.target_width, fit.target_height), (32, 32));
    }

    #[test]
    fn first_subpalette_always_returns_16_colors() {
        let palette = vec![Color::new(255, 0, 0)];
        let subpalette = first_subpalette(&palette);
        assert_eq!(subpalette.len(), 16);
        assert_eq!(subpalette[0], Color::new(255, 0, 0));
    }
}
