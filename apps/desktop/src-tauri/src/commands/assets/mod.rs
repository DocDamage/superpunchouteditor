//! Asset Commands
//!
//! Commands and helpers for working with game assets (palettes, sprites, portraits).

pub mod palettes;
pub mod portraits;
pub mod sprites;

use std::collections::HashMap;
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
use crate::utils::{format_hex, parse_offset};

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
    let rom_guard = state.rom.lock();
    let rom = rom_guard.as_ref().ok_or("No ROM loaded")?;
    rom.read_bytes(pc_offset, size)
        .map(|bytes| bytes.to_vec())
        .map_err(|e| e.to_string())
}

pub fn pending_bytes_for_offset(
    pending_writes: &HashMap<String, Vec<u8>>,
    pc_offset: usize,
) -> Option<Vec<u8>> {
    pending_writes
        .iter()
        .find_map(|(key, bytes)| match parse_offset(key) {
            Ok(offset) if offset == pc_offset => Some(bytes.clone()),
            _ => None,
        })
}

pub fn read_current_asset_bytes(
    state: &AppState,
    pc_offset: usize,
    size: usize,
) -> AssetResult<Vec<u8>> {
    if let Some(bytes) = pending_bytes_for_offset(&state.pending_writes.lock(), pc_offset) {
        return Ok(bytes);
    }

    read_original_rom_bytes(state, pc_offset, size)
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
    let mut pos = 0usize;
    let mut written = 0usize;

    while pos < input.len() {
        let ctrl = input[pos];
        pos += 1;

        if ctrl == END_OF_STREAM {
            return Ok(CompressionPassInfo {
                consumed: pos,
                written,
            });
        }

        let cmd = ctrl >> 5;
        let len = ((ctrl & 0x1F) as usize) + 1;

        match cmd {
            0 => {
                pos += len;
                written += len;
            }
            1 => {
                pos += 1;
                written += len;
            }
            2 => {
                pos += 2;
                written += len * 2;
            }
            3 => {
                pos += 1;
                written += len;
            }
            4 => {
                pos += 2;
                written += len;
            }
            _ => {
                return Err(format!("Unsupported HAL8 command {} at byte {}", cmd, pos - 1));
            }
        }

        if pos > input.len() {
            return Err("Compressed stream ends mid-command".to_string());
        }
    }

    Err("Compressed stream is missing end-of-stream marker".to_string())
}

pub fn decompress_interleaved_exact(bytes: &[u8]) -> AssetResult<Vec<u8>> {
    let pass1 = analyze_hal8_pass(bytes)?;
    let pass2 = analyze_hal8_pass(&bytes[pass1.consumed..])?;
    let expected_size = pass1.written.max(pass2.written) * 2;

    let mut decompressor = Decompressor::new(bytes);
    Ok(decompressor.decompress_interleaved(expected_size))
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
    let even = bytes
        .iter()
        .enumerate()
        .filter(|(idx, _)| idx % 2 == 0)
        .map(|(_, byte)| *byte)
        .collect::<Vec<_>>();
    let odd = bytes
        .iter()
        .enumerate()
        .filter(|(idx, _)| idx % 2 == 1)
        .map(|(_, byte)| *byte)
        .collect::<Vec<_>>();

    let mut compressed = compress_hal8_pass(&even);
    compressed.extend(compress_hal8_pass(&odd));
    compressed
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

pub fn current_tile_diff(
    original_tiles: &[Tile],
    current_tiles: &[Tile],
) -> Vec<bool> {
    let tile_count = original_tiles.len().max(current_tiles.len());
    (0..tile_count)
        .map(|idx| original_tiles.get(idx) != current_tiles.get(idx))
        .collect()
}

pub fn render_tile_strip(tiles: &[Tile], palette: &[Color], width_tiles: usize) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
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
    let tiles = decode_asset_tiles(&bytes, asset.category.contains("Compressed"))?;
    let img = render_tile_strip(&tiles, palette, width_tiles);
    png_bytes(&img)
}

pub fn encode_palette_bytes(colors: &[Color]) -> Vec<u8> {
    encode_palette(colors)
}

pub fn set_pending_write(state: &AppState, pc_offset: usize, bytes: Vec<u8>) {
    state
        .pending_writes
        .lock()
        .insert(format_hex(pc_offset), bytes);
    *state.modified.lock() = true;
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
    let palette = first_subpalette(&read_palette_colors(state.inner(), palette_pc, palette_asset.size)?);

    let icon_png = boxer
        .icon_files
        .first()
        .map(|asset| render_asset_png_bytes(state.inner(), asset, &palette, 4))
        .transpose()?;

    let portrait_png = boxer
        .large_portrait_files
        .first()
        .or_else(|| boxer.portrait_files.first())
        .map(|asset| render_asset_png_bytes(state.inner(), asset, &palette, 16))
        .transpose()?;

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
    fn hal8_roundtrip_preserves_data() {
        let data = (0..512)
            .map(|idx| ((idx * 7) & 0xFF) as u8)
            .collect::<Vec<_>>();
        let compressed = compress_interleaved(&data);
        let decompressed = decompress_interleaved_exact(&compressed).unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn first_subpalette_always_returns_16_colors() {
        let palette = vec![Color::new(255, 0, 0)];
        let subpalette = first_subpalette(&palette);
        assert_eq!(subpalette.len(), 16);
        assert_eq!(subpalette[0], Color::new(255, 0, 0));
    }
}
