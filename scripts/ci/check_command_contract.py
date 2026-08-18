#!/usr/bin/env python3
"""Verify literal frontend Tauri invokes are registered or explicitly gated.

Stable calls must have a backend handler. A frontend call may remain unregistered only when it is
listed in `experimental_frontend_commands.json` with a reviewed reason and the corresponding product
surface is hidden from stable navigation. This keeps research code auditable without pretending it is
part of the released command contract.
"""
from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
FRONTEND = ROOT / "apps" / "desktop" / "src"
BACKEND = ROOT / "apps" / "desktop" / "src-tauri" / "src"
TAURI_LIB = BACKEND / "lib.rs"
EXPERIMENTAL_MANIFEST = ROOT / "scripts" / "ci" / "experimental_frontend_commands.json"

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


def experimental_commands() -> dict[str, str]:
    raw = json.loads(EXPERIMENTAL_MANIFEST.read_text(encoding="utf-8"))
    if not isinstance(raw, dict) or any(not isinstance(k, str) or not isinstance(v, str) for k, v in raw.items()):
        raise SystemExit("experimental command manifest must be an object of command -> reason strings")
    if any(not reason.strip() for reason in raw.values()):
        raise SystemExit("every experimental command exception requires a non-empty reason")
    return raw


def main() -> int:
    invokes = frontend_invokes()
    registered = registered_commands()
    definitions = backend_definitions()
    experimental = experimental_commands()

    stale_exceptions = sorted(set(experimental) - set(invokes))
    registered_exceptions = sorted(set(experimental) & registered)
    missing = sorted(set(invokes) - registered - set(experimental))

    print(f"frontend literal invokes: {len(invokes)}")
    print(f"registered backend commands: {len(registered)}")
    print(f"explicitly gated frontend commands: {len(experimental)}")

    errors = False
    if missing:
        errors = True
        print("\nERROR: stable/unclassified frontend invokes missing from the Tauri handler:", file=sys.stderr)
        for name in missing:
            locations = ", ".join(invokes[name])
            candidates = ", ".join(definitions.get(name, [])) or "no same-named Rust function"
            print(f"  - {name}: frontend=[{locations}] backend=[{candidates}]", file=sys.stderr)

    if stale_exceptions:
        errors = True
        print("\nERROR: stale experimental command exceptions (remove them):", file=sys.stderr)
        for name in stale_exceptions:
            print(f"  - {name}", file=sys.stderr)

    if registered_exceptions:
        errors = True
        print("\nERROR: experimental exceptions are now registered; reclassify/remove the exception:", file=sys.stderr)
        for name in registered_exceptions:
            print(f"  - {name}", file=sys.stderr)

    if errors:
        return 1

    if experimental:
        print("\nExplicitly gated commands:")
        for name, reason in sorted(experimental.items()):
            print(f"  - {name}: {reason}")
    print("Stable command contract is synchronized.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
