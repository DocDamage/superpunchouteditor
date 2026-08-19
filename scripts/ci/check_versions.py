#!/usr/bin/env python3
"""Fail when application version and updater consumers drift apart."""
from __future__ import annotations

import base64
import binascii
import json
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def read_json(path: Path):
    return json.loads(path.read_text(encoding="utf-8"))


def validate_updater_pubkey(value: object) -> str | None:
    if not isinstance(value, str) or not value.strip():
        return "updater public key must be a non-empty base64 string"
    try:
        decoded = base64.b64decode(value, validate=True).decode("utf-8")
    except (binascii.Error, UnicodeDecodeError):
        return "updater public key must decode to UTF-8 minisign public-key text"

    lines = [line.strip() for line in decoded.splitlines() if line.strip()]
    if len(lines) < 2 or not lines[0].startswith("untrusted comment: minisign public key:"):
        return "updater public key is missing the expected minisign comment"
    if not lines[1].startswith("RW"):
        return "updater public key is missing the expected minisign key line"
    return None


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

    key_error = validate_updater_pubkey(updater.get("pubkey"))
    if key_error:
        print(f"ERROR: {key_error}", file=sys.stderr)
        return 1

    print("Application versions and updater configuration are consistent.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
