#!/usr/bin/env python3
"""Verify literal frontend Tauri invokes are registered by the backend.

The stable contract is intentionally mechanical: adding or renaming a literal invoke without adding
its backend handler fails CI. Dynamic invokes are not allowed outside the central command adapter.
"""
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
FRONTEND = ROOT / "apps" / "desktop" / "src"
TAURI_LIB = ROOT / "apps" / "desktop" / "src-tauri" / "src" / "lib.rs"

INVOKE_RE = re.compile(r"\binvoke(?:<[^>]+>)?\s*\(\s*['\"]([^'\"]+)['\"]")
HANDLER_RE = re.compile(r"tauri::generate_handler!\s*\[(.*?)\]\s*\)", re.S)
COMMAND_RE = re.compile(r"(?:[A-Za-z_][A-Za-z0-9_]*::)*([A-Za-z_][A-Za-z0-9_]*)\s*,?")


def frontend_invokes() -> dict[str, list[str]]:
    found: dict[str, list[str]] = {}
    for path in sorted(FRONTEND.rglob("*.ts*")):
        text = path.read_text(encoding="utf-8")
        for match in INVOKE_RE.finditer(text):
            found.setdefault(match.group(1), []).append(str(path.relative_to(ROOT)))
    return found


def registered_commands() -> set[str]:
    text = TAURI_LIB.read_text(encoding="utf-8")
    match = HANDLER_RE.search(text)
    if not match:
        raise SystemExit("Unable to find tauri::generate_handler![...] in backend lib.rs")
    block = re.sub(r"//.*", "", match.group(1))
    names: set[str] = set()
    for item in block.split(","):
        item = item.strip()
        if not item:
            continue
        command = COMMAND_RE.fullmatch(item)
        if command:
            names.add(command.group(1))
    return names


def main() -> int:
    invokes = frontend_invokes()
    registered = registered_commands()
    missing = sorted(set(invokes) - registered)

    print(f"frontend literal invokes: {len(invokes)}")
    print(f"registered backend commands: {len(registered)}")
    if missing:
        print("\nERROR: frontend invokes missing from the Tauri handler:", file=sys.stderr)
        for name in missing:
            locations = ", ".join(invokes[name])
            print(f"  - {name}: {locations}", file=sys.stderr)
        return 1

    print("Command contract is synchronized.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
