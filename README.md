# Super Punch-Out!! Editor

A Windows-first desktop ROM editor for *Super Punch-Out!!* (SNES), built with Tauri, Rust, React, and TypeScript.

> **ROM boundary:** this project does not ship, bundle, or upload copyrighted game ROMs. Supply your own legally obtained *Super Punch-Out!!* ROM (`.sfc` or `.smc`) locally. Do not attach ROMs, SRAM, or emulator save states to issues or release artifacts.

## Current release focus

**Windows x64 is the active release target.** Linux and macOS CI remain useful shared-code signals, but they are not the current packaging/release gate.

The authoritative development and default branch is **`main`**.

The September 9, 2026 consolidation brought the current application, all merged milestones, and the assembled boxer-pose update onto `main` while preserving the original branch histories. Earlier instructions calling `main` obsolete or directing development to `master` are superseded. Start new work from `origin/main` and target pull requests to `main`; the old milestone branches are historical references only. See [`docs/BRANCH_CONSOLIDATION_2026-09-09.md`](docs/BRANCH_CONSOLIDATION_2026-09-09.md).

## Unofficial fan project

This is an unofficial, fan-made tool. It is not affiliated with, sponsored by, or endorsed by Nintendo. *Super Punch-Out!!* is a trademark of Nintendo. The editor contains no game graphics, logos, fonts, sound or text: every picture it bundles is original, every font is openly licensed, and game pictures are only ever drawn at runtime from the ROM you supply. See [`docs/DESIGN_SYSTEM.md`](docs/DESIGN_SYSTEM.md) and [`docs/THIRD_PARTY_NOTICES.md`](docs/THIRD_PARTY_NOTICES.md).

## Stable user workflow

The interface is written so a ten-year-old can use it without instructions. The menu has four places:

1. **Edit Boxers** — pick a boxer, then use the tabs:
   - **Colors** — one-tap Magic Paint and Special Looks with a live preview, or change one color at a time.
   - **Photo Face** — put a photo on the boxer as they appear in the ring.
   - **Pictures** — swap the small menu face for any picture.
   - **Save & Share** — save a new game file, or make a patch file with only your changes.
   - **Expert Tools** — the raw tile and sprite tools for experienced editors.
2. **Look Around** — browse every boxer and pose. Nothing here changes the game.
3. **Play Game** — play the current edited game in an emulator with one button.
4. **My Projects** — save your work and open it again later.

### Things the editor does for you

- A recognised ROM opens immediately. You can also drop a ROM file onto the window.
- The last ROM reopens when the editor starts.
- Changes are kept automatically and come back the next time the same ROM is opened.
- Any picture can be used for a boxer's small face: it is fitted to the right size and matched to the boxer's colors.
- A photo placed on a boxer is written into every pose that shares that head, and simplified automatically if it has more detail than the game has room for.
- Colors always snap to what the console can display.
- An emulator already on the computer is found for you; you confirm it once.
- Every one of these is a single Undo step, and the original ROM file is never changed.

### Known limits

- A boxer uses several different head pictures. One photo stamp covers the poses that share a head; other poses need their own stamp.
- The large portrait shown before a fight uses a storage format the editor cannot read yet, so it cannot be changed.
- Boxer body graphics are stored packed with no spare room, so a very detailed photo may not fit even after simplifying.
- The Photo Face and automatic safekeeping features were verified with the editor's own renderer and automated tests against a real USA ROM; in-game appearance should still be checked with Play Game.

### Boxer graphics preview

The boxer editor shows a live **Preview** on the Colors tab (the Assembled Pose Preview). It reconstructs a complete in-game pose from the ROM's pose/OAM data, compressed graphics streams, VRAM tile destinations, and palette. Use the pose selector or **Prev/Next** controls to inspect the available poses.

The **Raw Tile Banks** section under **Expert Tools** is an editing reference: it shows individual 8×8 tiles in ROM/bank order. Those tiles are expected to look chopped or out of sequence when viewed by themselves. See [`docs/SPRITE_PREVIEW.md`](docs/SPRITE_PREVIEW.md) for the data flow, limitations, and troubleshooting guidance.

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

1. Create work from the current authoritative **`main`** branch.
2. Keep changes on a feature branch and target pull requests to `main`.
3. Run the relevant frontend/Rust checks before merge.
4. Do not commit ROM files, save states, copyrighted ROM extracts, private keys, certificates, or signing secrets.
5. Keep stable mutations on the canonical `BaseRom → EditJournal → WorkingRom` materialization path.

## License

MIT. See `LICENSE`. The license applies to the editor code only and grants no rights to the *Super Punch-Out!!* ROM, game assets, or third-party emulator binaries.
