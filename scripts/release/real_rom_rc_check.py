#!/usr/bin/env python3
"""Local-only release-candidate ROM identity gate.

This script intentionally never reads ROM content beyond local hashing/size/header-byte inspection and
never uploads or copies a ROM. It requires an expected SHA-1 for every supplied path so an accidental
or unknown file cannot silently become release evidence.

Environment variables (set one or more region pairs):
  SPO_ROM_USA / SPO_ROM_USA_SHA1
  SPO_ROM_JPN / SPO_ROM_JPN_SHA1
  SPO_ROM_PAL / SPO_ROM_PAL_SHA1

The output is metadata-only JSON suitable for attaching to local release evidence.
"""
from __future__ import annotations

import hashlib
import json
import os
import sys
from pathlib import Path

REGIONS = ("USA", "JPN", "PAL")
MAX_REASONABLE_ROM = 16 * 1024 * 1024
MIN_REASONABLE_ROM = 256 * 1024


def sha1_file(path: Path) -> str:
    digest = hashlib.sha1()
    with path.open("rb") as handle:
        while chunk := handle.read(1024 * 1024):
            digest.update(chunk)
    return digest.hexdigest()


def main() -> int:
    evidence: list[dict[str, object]] = []
    configured = 0

    for region in REGIONS:
        path_value = os.environ.get(f"SPO_ROM_{region}")
        expected = os.environ.get(f"SPO_ROM_{region}_SHA1")
        if not path_value and not expected:
            continue
        configured += 1
        if not path_value or not expected:
            print(
                f"ERROR: SPO_ROM_{region} and SPO_ROM_{region}_SHA1 must be supplied together",
                file=sys.stderr,
            )
            return 2

        path = Path(path_value).expanduser().resolve()
        if not path.is_file():
            print(f"ERROR: {region} ROM path is not a file: {path}", file=sys.stderr)
            return 2
        size = path.stat().st_size
        if not (MIN_REASONABLE_ROM <= size <= MAX_REASONABLE_ROM):
            print(f"ERROR: {region} ROM size {size} is outside the safety range", file=sys.stderr)
            return 2

        expected = expected.strip().lower()
        if len(expected) != 40 or any(ch not in "0123456789abcdef" for ch in expected):
            print(f"ERROR: {region} expected SHA-1 must be exactly 40 hex characters", file=sys.stderr)
            return 2

        actual = sha1_file(path)
        if actual != expected:
            print(
                f"ERROR: {region} SHA-1 mismatch: expected {expected}, actual {actual}",
                file=sys.stderr,
            )
            return 3

        evidence.append(
            {
                "region_label": region,
                "sha1": actual,
                "size": size,
                "filename": path.name,
                "verified": True,
            }
        )

    if configured == 0:
        print(
            "ERROR: no ROMs configured. Set at least one SPO_ROM_<REGION> and matching SHA-1.",
            file=sys.stderr,
        )
        return 2

    print(json.dumps({"schema": 1, "rom_identity_checks": evidence}, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
