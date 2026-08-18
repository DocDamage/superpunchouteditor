#!/usr/bin/env python3
"""Fail when application version consumers drift apart."""
from __future__ import annotations

import json
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def read_json(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def main() -> int:
    workspace = tomllib.loads((ROOT / "Cargo.toml").read_text(encoding="utf-8"))
    frontend = read_json(ROOT / "apps/desktop/package.json")
    tauri = read_json(ROOT / "apps/desktop/src-tauri/tauri.conf.json")

    versions = {
        "workspace": workspace["workspace"]["package"]["version"],
        "frontend": frontend["version"],
        "tauri": tauri["version"],
    }
    expected = versions["workspace"]
    mismatches = {name: value for name, value in versions.items() if value != expected}

    for name, value in versions.items():
        print(f"{name}: {value}")

    if mismatches:
        print(f"ERROR: application version consumers must all equal {expected}", file=sys.stderr)
        return 1

    updater = tauri.get("plugins", {}).get("updater", {})
    endpoints = updater.get("endpoints", [])
    expected_endpoint = (
        "https://github.com/DocDamage/superpunchouteditor/releases/latest/download/latest.json"
    )
    if endpoints != [expected_endpoint]:
        print("ERROR: updater endpoint is not the canonical signed static feed", file=sys.stderr)
        return 1

    if tauri.get("bundle", {}).get("createUpdaterArtifacts") is not True:
        print("ERROR: updater artifact generation must remain enabled", file=sys.stderr)
        return 1

    print("Application versions and updater configuration are consistent.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
