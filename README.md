# Super Punch-Out!! Editor

A desktop ROM editor for *Super Punch-Out!!* (SNES). Built with Tauri (Rust backend, React/TypeScript frontend).

> **Disclaimer**: This editor does not ship with any copyrighted game data. You must supply your own *Super Punch-Out!!* ROM (`.sfc` or `.smc` format). Use of ROM images is subject to your local laws and the rights of the copyright holder.

## Supported workflows

- **ROM management** — load, validate, and detect region (USA / JPN / PAL); save modified ROM; export IPS or BPS patch; compare against original
- **Roster editing** — edit all boxer attributes (name, stats, AI config, palette); expanded-roster support beyond the original nine boxers
- **In-ROM boxer creation** — launch the game's built-in creator mode inside the embedded emulator; commit or cancel the created boxer back into the ROM image
- **Asset editing** — import and stage sprite graphics and portrait images for any boxer slot
- **Text editing** — boxer names, victory quotes, cornerman text, and boxer intro text; enforces byte-length limits per field
- **Audio** — browse and preview sound effects and music; export supported audio assets
- **Embedded emulator** — run the edited ROM in an integrated Snes9x (libretro) instance; pause, step frames, reset, and save/restore states
- **Project save / load** — save editor state (including all pending writes) to a project file and reload it later
- **Plugin management** — load and run third-party editor plugins
- **Frame tags and annotations** — tag and annotate individual animation frames

### Intentionally out of scope (v1)

- Menu text ROM persistence (offsets not researched; tab removed from UI)
- Credits text editing (removed from UI)
- Full sample/SPC music editing beyond browse and export

## Supported platforms

| Platform | Status |
| --- | --- |
| Windows (x64) | Supported |
| macOS (x64 / Apple Silicon) | Supported |
| Linux (x64) | Supported |

## Getting started

### Prerequisites

- [Rust](https://rustup.rs/) 1.70 or newer
- [Node.js](https://nodejs.org/) 20 or newer and npm
- The Snes9x libretro core binary placed in `apps/desktop/src-tauri/binaries/` (see [Embedded emulator](#embedded-emulator) below)
- **Linux only**: system libraries for WebKit/GTK

```sh
# Ubuntu / Debian
sudo apt-get install -y \
  libwebkit2gtk-4.1-dev libgtk-3-dev \
  libayatana-appindicator3-dev librsvg2-dev patchelf
```

### Run in development mode

```sh
cd apps/desktop
npm install
npm run tauri dev
```

### Build a release package

```sh
cd apps/desktop
npm install
npm run tauri build
```

The packaged installer appears in `apps/desktop/src-tauri/target/release/bundle/`.

## Embedded emulator

The embedded emulator requires a compiled Snes9x libretro core. Place the platform-appropriate binary in `apps/desktop/src-tauri/binaries/`:

| Platform | File |
| --- | --- |
| Windows | `snes9x_libretro.dll` |
| macOS | `snes9x_libretro.dylib` |
| Linux | `snes9x_libretro.so` |

Pre-built cores are available from the [Libretro buildbot](https://buildbot.libretro.com/nightly/). The editor degrades gracefully if the core is absent — all non-emulator features remain available.

## Project structure

```text
apps/desktop/          Tauri desktop application
  src/                 React/TypeScript frontend
  src-tauri/           Rust backend (Tauri commands, state)
crates/
  rom-core/            ROM parsing, roster read/write, text, patches
  asset-core/          Sprite and portrait asset handling
  emulator-core/       Libretro wrapper and creator-mode runtime
  expansion-core/      Expanded roster support
  manifest-core/       Boxer layout manifests
  patch-core/          IPS/BPS patch generation
  plugin-core/         Plugin loader and command execution
  project-core/        Project save/load
  script-core/         Scripting support
  debugger-core/       Disassembler and debug utilities
data/
  manifests/           Boxer layout manifest JSON files (bundled at build time)
  boxer-layouts/       Community-contributed layout packs
```

## Building from source: verification checklist

Run these before opening a pull request or cutting a release:

1. `cargo fmt --all -- --check`
2. `cargo check --workspace`
3. `cargo clippy --workspace --all-targets -- -D warnings`
4. `cargo test --workspace --exclude tauri-appsuper-punch-out-editor`
5. `cd apps/desktop && npx tsc --noEmit`
6. `cd apps/desktop && npm run build`

CI (`.github/workflows/ci.yml`) runs all of the above automatically on push and pull request.

## Contributing

1. Fork the repository and create your branch from `main`.
2. Follow the verification checklist above before opening a pull request.
3. Do not commit ROM files or copyrighted game assets.

## License

MIT. See `LICENSE`. This license applies only to the editor code — it does not grant any rights to the *Super Punch-Out!!* ROM or game assets.
