#!/usr/bin/env python3
"""Verify literal frontend Tauri invokes are registered by the backend.

The stable contract is intentionally mechanical: adding or renaming a literal invoke without adding
its backend handler fails CI. When a missing registration is found, the diagnostic also reports any
same-named Rust function so reconciliation is deterministic rather than guesswork.
"""
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
FRONTEND = ROOT / "apps" / "desktop" / "src"
BACKEND = ROOT / "apps" / "desktop" / "src-tauri" / "src"
TAURI_LIB = BACKEND / "lib.rs"

INVOKE_RE = re.compile(r"\binvoke(?:<[^>]+>)?\s*\(\s*['\"]([^'\"]+)['\"]")
HANDLER_RE = re.compile(r"tauri::generate_handler!\s*\[(.*?)\]\s*\)", re.S)
COMMAND_RE = re.compile(r"(?:[A-Za-z_][A-Za-z0-9_]*::)*([A-Za-z_][A-Za-z0-9_]*)\s*,?")
FN_RE = re.compile(r"\b(?:pub\s+)?(?:async\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(")


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


def backend_definitions() -> dict[str, list[str]]:
    found: dict[str, list[str]] = {}
    for path in sorted(BACKEND.rglob("*.rs")):
        text = path.read_text(encoding="utf-8")
        for match in FN_RE.finditer(text):
            found.setdefault(match.group(1), []).append(str(path.relative_to(ROOT)))
    return found


def main() -> int:
    invokes = frontend_invokes()
    registered = registered_commands()
    definitions = backend_definitions()
    missing = sorted(set(invokes) - registered)

    print(f"frontend literal invokes: {len(invokes)}")
    print(f"registered backend commands: {len(registered)}")
    if missing:
        print("\nERROR: frontend invokes missing from the Tauri handler:", file=sys.stderr)
        for name in missing:
            locations = ", ".join(invokes[name])
            candidates = ", ".join(definitions.get(name, [])) or "no same-named Rust function"
            print(f"  - {name}: frontend=[{locations}] backend=[{candidates}]", file=sys.stderr)
        return 1

    print("Command contract is synchronized.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
