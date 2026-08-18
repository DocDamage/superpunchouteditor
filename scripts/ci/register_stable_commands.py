#!/usr/bin/env python3
"""Register stable command modules introduced during remediation."""
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
LIB = ROOT / "apps/desktop/src-tauri/src/lib.rs"

text = LIB.read_text(encoding="utf-8")


def insert_after(marker: str, block: str, sentinel: str) -> None:
    global text
    if sentinel in text:
        return
    if marker not in text:
        raise SystemExit(f"command registry marker missing: {marker!r}")
    text = text.replace(marker, marker + block, 1)


insert_after(
    "            commands::project::save_patch_notes,\n",
    """            commands::project_thumbnail::capture_project_thumbnail,
            commands::project_thumbnail::save_project_thumbnail,
            commands::project_thumbnail::get_project_thumbnail,
            commands::project_thumbnail::clear_project_thumbnail,
            commands::project_thumbnail::load_project_thumbnail_from_path,
""",
    "commands::project_thumbnail::capture_project_thumbnail,",
)

insert_after(
    "            commands::emulator::get_emulator_presets,\n",
    "            commands::emulator_current::emulator_load_current_rom,\n",
    "commands::emulator_current::emulator_load_current_rom,",
)

LIB.write_text(text, encoding="utf-8")
print("Stable command registrations reconciled.")
