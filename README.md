# Super Punch-Out!! Editor

A Windows-first desktop ROM editor for *Super Punch-Out!!* (SNES), built with Tauri, Rust, React, and TypeScript.

> **ROM boundary:** this project does not ship, bundle, or upload copyrighted game ROMs. Supply your own legally obtained *Super Punch-Out!!* ROM (`.sfc` or `.smc`) locally. Do not attach ROMs, SRAM, or emulator save states to issues or release artifacts.

## Current release focus

**Windows x64 is the active release target.** Linux and macOS CI remain useful shared-code signals, but they are not the current packaging/release gate.

The authoritative development branch is **`master`**.

The GitHub repository default branch is still `main` for historical/admin reasons. That `main` branch is obsolete starter history and is **not** the real application. Do not develop from, merge into, force-update, or replace `main`.

## Stable user workflow

The stable Windows UI is intentionally organized around a small guided path:

1. **Open ROM** — choose and validate your own local ROM.
2. **Edit & Export** — choose a boxer, make journal-backed edits, Undo/Redo, save a materialized ROM, and export supported patch/output formats.
3. **Inspect** — safely inspect supported ROM/asset data.
4. **Test Game** — run the current materialized revision rather than a stale or immutable base image.
5. **Projects** — save and reopen project-v2 editing sessions.
6. **Settings** — application/update preferences and emulator/tool configuration.

The first-run screen and left sidebar are designed to make the next safe action obvious. Development builds may expose additional experimental tools behind **Advanced tools**; stable builds do not expose research-blocked surfaces as normal release features.

## Community Windows testing

A dedicated GitHub Actions workflow builds:

`super-punch-out-editor-community-tester-kit`

The kit contains:

- the unsigned Windows NSIS tester installer;
- `README_FIRST.txt`;
- `START_HERE.md` with the short community test procedure;
- `CHECKSUMS.txt` with the exact installer SHA-256;
- `BUILD_INFO.json` with source commit/build provenance;
- an `advanced-evidence/` folder containing the metadata/hash-only Windows acceptance helpers.

The workflow fails if ROM, SRAM/save-state, or `SUPERZSNES.exe` content enters the tester kit.

Start with [`docs/COMMUNITY_TESTING.md`](docs/COMMUNITY_TESTING.md). The application also includes **Tester Checklist**, which stores progress locally and can copy/download a privacy-safe Markdown test report.

An unsigned tester installer is **not a production stable release** and is not evidence that Authenticode or Tauri updater signing is configured. Production signing remains fail-closed in the separate release workflow.

## Supported platform status

| Platform | Current status |
| --- | --- |
| Windows x64 | **Primary release target / active acceptance** |
| Linux x64 | Shared-code CI signal; not current release gate |
| macOS | Shared-code CI signal; not current release gate |

## Developer setup

### Prerequisites

- Rust toolchain used by CI (currently pinned in workflows)
- Node.js 22 recommended for parity with Windows package CI
- npm 10+
- Optional embedded-emulator core supplied locally by the developer/tester

### Run the desktop app

```sh
cd apps/desktop
npm ci
npm run tauri dev
```

### Frontend verification

```sh
cd apps/desktop
npm ci
npm test
npm run build
```

### Rust verification

```sh
cargo fmt --all -- --check
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

CI also runs repository hygiene, security, package, release-contract, updater/version, SBOM, and Windows lifecycle checks.

## Embedded and external emulation

Test launches must consume the exact current materialized ROM revision.

The project does not redistribute third-party emulator binaries without verified redistribution rights. A compatible embedded libretro core or external emulator may be supplied locally by the tester. External emulator configuration must point to the tester's existing executable; the editor must not copy that executable into the application or project.

## Windows acceptance and release engineering

See [`docs/WINDOWS_ACCEPTANCE.md`](docs/WINDOWS_ACCEPTANCE.md) for the full Windows canonical-output acceptance process.

The acceptance tooling verifies metadata/hashes for:

- source-ROM immutability;
- saved materialized ROM equivalence;
- BPS equivalence;
- IPS equivalence when supported;
- project-v2 restored output equivalence;
- manual/visual editor and emulator gates;
- signed-installer requirements for production release evidence.

Real ROM bytes are never required in GitHub Actions artifacts.

## Project structure

```text
apps/desktop/          Tauri desktop application
  src/                 React/TypeScript frontend
  src-tauri/           Rust backend and Tauri commands
crates/                 Rust editor/core libraries
data/                   manifests and editor data
scripts/windows/        Windows acceptance/release helpers
docs/                   architecture, release, recovery, testing, and acceptance docs
```

The desktop application does not require Python at runtime. Remaining Python files are optional research/build utilities.

## Contributing

1. Create work from the current authoritative **`master`** branch.
2. Keep changes on a feature branch and target pull requests to `master`.
3. Run the relevant frontend/Rust checks before merge.
4. Do not commit ROM files, save states, copyrighted ROM extracts, private keys, certificates, or signing secrets.
5. Keep stable mutations on the canonical `BaseRom → EditJournal → WorkingRom` materialization path.

## License

MIT. See `LICENSE`. The license applies to the editor code only and grants no rights to the *Super Punch-Out!!* ROM, game assets, or third-party emulator binaries.
