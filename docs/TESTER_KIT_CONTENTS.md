# Community Tester Kit Contents

The automated `super-punch-out-editor-community-tester-kit` artifact is intended to be shared with Windows testers as one ZIP.

## Included

- Unsigned Windows x64 NSIS tester installer.
- `README_FIRST.txt` — build identity, source commit, checksum, and unsigned-test warning.
- `START_HERE.md` — short community smoke-test instructions.
- `CHECKSUMS.txt` — SHA-256 for the included installer.
- `BUILD_INFO.json` — machine-readable source/build provenance.
- `advanced-evidence/WINDOWS_ACCEPTANCE.md` — full canonical Windows acceptance procedure.
- `advanced-evidence/acceptance-*.ps1` — metadata/hash-only evidence helpers.

The installed application also provides a **Tester Checklist** that can copy or download a Markdown report without including ROM bytes or ROM paths.

## Explicitly excluded

The workflow scans the assembled kit and fails if it finds:

- `.sfc` or `.smc` ROM images;
- `.srm` SRAM files;
- `.state` or `.savestate` emulator save states;
- `SUPERZSNES.exe`.

The kit also must not contain private signing keys, certificates, passwords, or production signing secrets.

## Release meaning

A successful community tester kit proves the source revision can build a Windows NSIS tester installer and that the shareable package observes the ROM/emulator boundary. It does **not** prove production Authenticode signing, Tauri updater signing, real-ROM acceptance, or installed-update lifecycle acceptance.

Those remain separate release gates.
