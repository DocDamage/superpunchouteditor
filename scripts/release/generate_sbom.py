#!/usr/bin/env python3
"""Generate a dependency SBOM from committed Rust and npm lock state."""
from __future__ import annotations

import argparse
import json
import subprocess
import tomllib
from pathlib import Path
from urllib.parse import quote

ROOT = Path(__file__).resolve().parents[2]


def cargo_components() -> list[dict[str, object]]:
    proc = subprocess.run(
        ["cargo", "metadata", "--format-version", "1", "--locked"],
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    )
    metadata = json.loads(proc.stdout)
    components: list[dict[str, object]] = []
    for package in metadata["packages"]:
        name = package["name"]
        version = package["version"]
        source = package.get("source")
        component: dict[str, object] = {
            "type": "library",
            "name": name,
            "version": version,
            "bom-ref": f"pkg:cargo/{quote(name, safe='')}@{version}",
            "purl": f"pkg:cargo/{quote(name, safe='')}@{version}",
            "properties": [
                {"name": "superpunchouteditor:ecosystem", "value": "cargo"},
                {
                    "name": "superpunchouteditor:dependency-source",
                    "value": source or "workspace/path",
                },
            ],
        }
        license_expression = package.get("license")
        if license_expression:
            component["licenses"] = [{"expression": license_expression}]
        components.append(component)
    return components


def npm_name_from_path(path: str) -> str:
    tail = path.rsplit("node_modules/", 1)[-1].strip("/")
    if not tail:
        raise ValueError(f"Cannot derive npm package name from {path!r}")
    return tail


def npm_components() -> list[dict[str, object]]:
    lock_path = ROOT / "apps" / "desktop" / "package-lock.json"
    lock = json.loads(lock_path.read_text(encoding="utf-8"))
    components: list[dict[str, object]] = []
    for path, package in lock.get("packages", {}).items():
        if not path or "node_modules/" not in path:
            continue
        version = package.get("version")
        if not version:
            continue
        name = package.get("name") or npm_name_from_path(path)
        encoded = quote(name, safe="/")
        component: dict[str, object] = {
            "type": "library",
            "name": name,
            "version": version,
            "bom-ref": f"pkg:npm/{encoded}@{version}",
            "purl": f"pkg:npm/{encoded}@{version}",
            "properties": [
                {"name": "superpunchouteditor:ecosystem", "value": "npm"},
                {
                    "name": "superpunchouteditor:development-only",
                    "value": str(bool(package.get("dev"))).lower(),
                },
            ],
        }
        license_name = package.get("license")
        if isinstance(license_name, str) and license_name:
            component["licenses"] = [{"license": {"name": license_name}}]
        components.append(component)
    return components


def application_version() -> str:
    workspace = tomllib.loads((ROOT / "Cargo.toml").read_text(encoding="utf-8"))
    return workspace["workspace"]["package"]["version"]


def deduplicate(components: list[dict[str, object]]) -> list[dict[str, object]]:
    by_ref = {str(component["bom-ref"]): component for component in components}
    return [by_ref[key] for key in sorted(by_ref)]


def build_sbom() -> dict[str, object]:
    version = application_version()
    components = deduplicate(cargo_components() + npm_components())
    return {
        "bomFormat": "CycloneDX",
        "specVersion": "1.6",
        "version": 1,
        "metadata": {
            "component": {
                "type": "application",
                "name": "Super Punch-Out!! Editor",
                "version": version,
                "bom-ref": f"pkg:github/DocDamage/superpunchouteditor@{version}",
                "purl": f"pkg:github/DocDamage/superpunchouteditor@{version}",
            },
            "properties": [
                {
                    "name": "superpunchouteditor:source",
                    "value": "Cargo.lock + apps/desktop/package-lock.json",
                }
            ],
        },
        "components": components,
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()

    output = args.output.resolve()
    output.parent.mkdir(parents=True, exist_ok=True)
    sbom = build_sbom()
    output.write_text(json.dumps(sbom, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"Wrote CycloneDX SBOM with {len(sbom['components'])} components: {output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
