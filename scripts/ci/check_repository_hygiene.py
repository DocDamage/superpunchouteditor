#!/usr/bin/env python3
"""Guard application-repository hygiene and reproducibility invariants."""
from __future__ import annotations

import re
import subprocess
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
FORBIDDEN_SUFFIXES = {".pyc", ".pyo", ".sfc", ".smc", ".ips", ".bps", ".srm", ".sav"}
TARGET_RE = re.compile(r"(^|/)target(?:_codex|\d+)?(/|$)")
EXPERIMENTAL_CRATES = {
    "crates/assembly-core",
    "crates/console-dev-core",
    "crates/profiler-core",
}


def tracked_files() -> list[str]:
    result = subprocess.run(
        ["git", "ls-files"], cwd=ROOT, text=True, capture_output=True, check=True
    )
    return [line for line in result.stdout.splitlines() if line]


def main() -> int:
    errors: list[str] = []
    files = tracked_files()

    for required in ("Cargo.lock", "apps/desktop/package-lock.json"):
        if required not in files:
            errors.append(f"required lockfile is not tracked: {required}")

    for path in files:
        if TARGET_RE.search(path):
            errors.append(f"generated Rust build output is tracked: {path}")
        if Path(path).suffix.lower() in FORBIDDEN_SUFFIXES:
            errors.append(f"generated/copyright-sensitive binary is tracked: {path}")
        if "/__pycache__/" in f"/{path}/" or "/.pytest_cache/" in f"/{path}/":
            errors.append(f"Python cache is tracked: {path}")

    workspace = tomllib.loads((ROOT / "Cargo.toml").read_text(encoding="utf-8"))["workspace"]
    excluded = set(workspace.get("exclude", []))
    if excluded != EXPERIMENTAL_CRATES:
        errors.append(
            "experimental crate classification drifted; expected explicit workspace exclusions: "
            + ", ".join(sorted(EXPERIMENTAL_CRATES))
        )

    if errors:
        print("Repository hygiene check failed:", file=sys.stderr)
        for error in errors:
            print(f"  - {error}", file=sys.stderr)
        return 1

    print(f"Repository hygiene check passed across {len(files)} tracked paths.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
