#!/usr/bin/env python3
"""Migrate stable asset reads/writes onto immutable base/current journal state."""
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
MOD = ROOT / "apps/desktop/src-tauri/src/commands/assets/mod.rs"
SPRITES = ROOT / "apps/desktop/src-tauri/src/commands/assets/sprites.rs"
PORTRAITS = ROOT / "apps/desktop/src-tauri/src/commands/assets/portraits.rs"
HISTORY = ROOT / "apps/desktop/src-tauri/src/commands/history.rs"
MATURITY = ROOT / "apps/desktop/src/featureMaturity.ts"

text = MOD.read_text(encoding="utf-8")
text = text.replace("use std::collections::HashMap;\n", "")
text = text.replace("use crate::utils::{format_hex, parse_offset};", "use crate::utils::parse_offset;")
old = '''pub fn read_original_rom_bytes(
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
'''
new = '''pub fn read_original_rom_bytes(
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
'''
if old in text:
    text = text.replace(old, new, 1)
elif "session.base().bytes()[range]" not in text:
    raise SystemExit("asset read helpers changed unexpectedly")
old = '''pub fn set_pending_write(state: &AppState, pc_offset: usize, bytes: Vec<u8>) {
    state
        .pending_writes
        .lock()
        .insert(format_hex(pc_offset), bytes);
    *state.modified.lock() = true;
}
'''
new = '''pub fn set_pending_write(state: &AppState, pc_offset: usize, bytes: Vec<u8>) -> AssetResult<()> {
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
'''
if old in text:
    text = text.replace(old, new, 1)
elif "commit_rom_write(" not in text:
    raise SystemExit("asset write helper changed unexpectedly")
MOD.write_text(text, encoding="utf-8")

for path in (SPRITES, PORTRAITS):
    text = path.read_text(encoding="utf-8")
    text = text.replace(
        "    set_pending_write(state.inner(), asset_pc_offset, new_bytes.clone());\n\n    Ok((new_bytes.len(), original_size, fits))",
        '''    if !fits {
        return Err(format!(
            "Imported asset is {} bytes but the original slot is {} bytes; relocation is experimental, so no ROM change was committed",
            new_bytes.len(), original_size
        ));
    }
    set_pending_write(state.inner(), asset_pc_offset, new_bytes.clone())?;

    Ok((new_bytes.len(), original_size, true))''',
    )
    path.write_text(text, encoding="utf-8")

text = HISTORY.read_text(encoding="utf-8")
old = '''    record_compat_edit(
        &state,
        format!("Palette color {color_index} at {pc_offset}"),
        pc_offset,
        old_color,
        new_color,
    )
'''
new = '''    let base_offset = parse_hex_offset(&pc_offset)?;
    let byte_offset = base_offset
        .checked_add(color_index.saturating_mul(2))
        .ok_or("Palette color offset overflow")?;
    record_compat_edit(
        &state,
        format!("Palette color {color_index} at {pc_offset}"),
        format!("0x{byte_offset:X}"),
        old_color,
        new_color,
    )
'''
if old in text:
    text = text.replace(old, new, 1)
elif "color_index.saturating_mul(2)" not in text:
    raise SystemExit("palette history adapter changed unexpectedly")
HISTORY.write_text(text, encoding="utf-8")

text = MATURITY.read_text(encoding="utf-8")
text = text.replace(
    '''  roster: {
    status: "stable",
    releaseDecision: "Stable after roster writers are journal-backed.",
  },''',
    '''  roster: {
    status: "experimental",
    releaseDecision: "Proven roster writes are journal-backed; champion/reset controls remain intentionally unsupported.",
  },''',
)
MATURITY.write_text(text, encoding="utf-8")
print("Asset and palette journal migration applied.")
