#!/usr/bin/env python3
"""Deterministically repair the remaining remediation-branch CI blockers.

This helper is intentionally narrow and idempotent. It exists only while the remediation PR is
being certified and is removed once a clean, non-mutating CI cycle passes.
"""
from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def read(rel: str) -> tuple[Path, str]:
    path = ROOT / rel
    return path, path.read_text(encoding="utf-8")


def write(path: Path, text: str) -> None:
    path.write_text(text, encoding="utf-8")


def fix_region_serialization() -> None:
    for rel in (
        "apps/desktop/src-tauri/src/commands/preflight.rs",
        "apps/desktop/src-tauri/src/commands/project.rs",
    ):
        path, text = read(rel)
        text = text.replace(
            ".map(|region| region.as_str().to_string())",
            ".map(|region| region.code().to_string())",
        )
        write(path, text)


def demote_legacy_roster_mutation_commands() -> None:
    path, text = read("apps/desktop/src-tauri/src/roster_commands.rs")
    # The journal-backed module owns these public IPC command names. Keep the legacy functions only
    # as internal compatibility helpers; removing the Tauri attribute prevents duplicate generated
    # macro names and prevents direct-ROM mutation paths from being exposed over IPC.
    legacy_mutations = (
        "update_boxer_name",
        "commit_creator_session",
        "update_boxer_circuit",
        "update_unlock_order",
        "set_champion_status",
        "update_boxer_intro_field",
        "update_intro_text",
        "reset_roster_to_defaults",
    )
    for name in legacy_mutations:
        marker = f"#[tauri::command]\npub fn {name}("
        replacement = f"pub fn {name}("
        text = text.replace(marker, replacement, 1)
    write(path, text)


def fix_manifest_compat_lint() -> None:
    path, text = read("crates/manifest-core/src/comparison.rs")
    marker = "    pub fn from_str(s: &str) -> Option<Self> {"
    replacement = (
        "    // Kept as an Option-returning compatibility API; this intentionally differs from "
        "std::str::FromStr.\n"
        "    #[allow(clippy::should_implement_trait)]\n"
        "    pub fn from_str(s: &str) -> Option<Self> {"
    )
    if marker in text and "#[allow(clippy::should_implement_trait)]\n    pub fn from_str" not in text:
        text = text.replace(marker, replacement, 1)
    write(path, text)


def fix_libretro_safety() -> None:
    path, text = read("crates/emulator-core/src/audio.rs")
    marker = "    /// Submit a batch of samples (modern libretro callback)\n    pub fn submit_batch("
    replacement = (
        "    /// Submit a batch of samples (modern libretro callback)\n"
        "    ///\n"
        "    /// # Safety\n"
        "    /// `data` must point to at least `frames * 2` initialized `i16` samples and remain\n"
        "    /// valid for the duration of this call. This contract is provided by libretro.\n"
        "    pub unsafe fn submit_batch("
    )
    if marker in text:
        text = text.replace(marker, replacement, 1)
    write(path, text)

    path, text = read("crates/emulator-core/src/libretro_runtime.rs")
    text = text.replace(
        "        return audio.submit_batch(data, frames);",
        "        return unsafe { audio.submit_batch(data, frames) };",
        1,
    )
    write(path, text)

    path, text = read("crates/emulator-core/src/libretro.rs")
    marker = "/// Safe wrapper for converting C strings\npub unsafe fn c_str_to_string"
    replacement = (
        "/// Convert a C string pointer into an owned Rust string.\n"
        "///\n"
        "/// # Safety\n"
        "/// `ptr` must be null or point to a valid NUL-terminated C string for the duration of\n"
        "/// this call.\n"
        "pub unsafe fn c_str_to_string"
    )
    if marker in text:
        text = text.replace(marker, replacement, 1)
    write(path, text)


def fix_msrv_alignment() -> None:
    replacement = "(value / alignment + usize::from(value % alignment != 0)) * alignment"
    for rel in (
        "crates/expansion-core/src/ingame_editor.rs",
        "crates/expansion-core/src/roster_expansion.rs",
    ):
        path, text = read(rel)
        text = text.replace("value.div_ceil(alignment) * alignment", replacement)
        write(path, text)


def simplify_plugin_callback_types() -> None:
    path, text = read("crates/plugin-core/src/api.rs")
    aliases = """type RomReader = Box<dyn Fn(usize, usize) -> PluginResult<Vec<u8>> + Send + Sync>;
type RomWriter = Box<dyn Fn(usize, &[u8]) -> PluginResult<()> + Send + Sync>;
type AssetGetter = Box<dyn Fn(&str) -> PluginResult<AssetInfo> + Send + Sync>;
type PluginLogger = Box<dyn Fn(log::Level, &str) + Send + Sync>;
type PluginNotifier = Box<dyn Fn(&str, NotificationType) + Send + Sync>;

"""
    import_marker = "use std::sync::Arc;\n\n"
    if "type RomReader =" not in text:
        if import_marker not in text:
            raise SystemExit("plugin API import marker changed")
        text = text.replace(import_marker, import_marker + aliases, 1)
    text = text.replace(
        "rom_reader: Box<dyn Fn(usize, usize) -> PluginResult<Vec<u8>> + Send + Sync>,",
        "rom_reader: RomReader,",
    )
    text = text.replace(
        "rom_writer: Box<dyn Fn(usize, &[u8]) -> PluginResult<()> + Send + Sync>,",
        "rom_writer: RomWriter,",
    )
    text = text.replace(
        "asset_getter: Box<dyn Fn(&str) -> PluginResult<AssetInfo> + Send + Sync>,",
        "asset_getter: AssetGetter,",
    )
    text = text.replace(
        "logger: Box<dyn Fn(log::Level, &str) + Send + Sync>,",
        "logger: PluginLogger,",
    )
    text = text.replace(
        "notifier: Box<dyn Fn(&str, NotificationType) + Send + Sync>,",
        "notifier: PluginNotifier,",
    )
    write(path, text)


def allow_shared_test_helper_dead_code() -> None:
    markers = {
        "tests/integration/common/portrait_utils.rs": (
            "//! and verifying round-trip encoding/decoding.\n\n",
            "//! and verifying round-trip encoding/decoding.\n\n#![allow(dead_code)]\n\n",
        ),
        "tests/integration/common/sprite_utils.rs": (
            "//! Provides helper functions for creating test tiles and sprite patterns.\n\n",
            "//! Provides helper functions for creating test tiles and sprite patterns.\n\n#![allow(dead_code)]\n\n",
        ),
    }
    for rel, (marker, replacement) in markers.items():
        path, text = read(rel)
        if "#![allow(dead_code)]" not in text:
            if marker not in text:
                raise SystemExit(f"shared test helper marker changed: {rel}")
            text = text.replace(marker, replacement, 1)
        write(path, text)


def main() -> None:
    fix_region_serialization()
    demote_legacy_roster_mutation_commands()
    fix_manifest_compat_lint()
    fix_libretro_safety()
    fix_msrv_alignment()
    simplify_plugin_callback_types()
    allow_shared_test_helper_dead_code()
    print("Remaining CI blockers repaired.")


if __name__ == "__main__":
    main()
