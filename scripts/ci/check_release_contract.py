#!/usr/bin/env python3
"""Protect the Windows-first production release contract from silent regression."""
from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
RELEASE = ROOT / ".github" / "workflows" / "release.yml"


def main() -> int:
    text = RELEASE.read_text(encoding="utf-8")
    required = {
        "Windows runner": "runs-on: windows-latest",
        "pinned Windows target": "x86_64-pc-windows-msvc",
        "NSIS-only release bundle": "--bundles nsis",
        "pinned Tauri release action": "tauri-apps/tauri-action@v0.6.2",
        "draft release": "releaseDraft: true",
        "updater private-key gate": "TAURI_SIGNING_PRIVATE_KEY",
        "Windows certificate gate": "WINDOWS_CERTIFICATE",
        "timestamp authority gate": "WINDOWS_TIMESTAMP_URL",
        "Windows certificate thumbprint override": "certificateThumbprint",
        "Authenticode verification": "Get-AuthenticodeSignature",
        "updater signature artifact": "updater_sig",
        "configured updater key parse": "verify_updater_signature -- check-key",
        "updater signature/public-key verification": "verify_updater_signature -- verify",
        "CycloneDX generation": "generate_sbom.py",
        "Windows checksums": "SHA256SUMS-windows.txt",
        "static updater feed": "latest.json",
    }
    missing = [label for label, token in required.items() if token not in text]

    forbidden = {
        "Linux tagged release runner": "ubuntu-latest",
        "macOS tagged release runner": "macos-latest",
        "moving Tauri v0 action alias": "tauri-apps/tauri-action@v0\n",
    }
    present_forbidden = [label for label, token in forbidden.items() if token in text]

    if missing:
        print("ERROR: release workflow is missing required Windows release controls:", file=sys.stderr)
        for label in missing:
            print(f"  - {label}", file=sys.stderr)
    if present_forbidden:
        print("ERROR: release workflow contains forbidden/deferred configuration:", file=sys.stderr)
        for label in present_forbidden:
            print(f"  - {label}", file=sys.stderr)

    if missing or present_forbidden:
        return 1

    print("Windows release workflow contract is intact.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
