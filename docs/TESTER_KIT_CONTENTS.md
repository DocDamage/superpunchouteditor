# Community Tester Kit Contents

The automated `super-punch-out-editor-community-tester-kit` artifact is intended to be shared with Windows testers as one ZIP.

## Included

- Unsigned Windows x64 NSIS tester installer.
- `README_FIRST.txt` — build identity, source commit, checksum, unsigned-test warning, and lifecycle result.
- `START_HERE.md` — short community smoke-test instructions.
- `CHECKSUMS.txt` — SHA-256 for the included installer.
- `BUILD_INFO.json` — machine-readable source/build provenance and Windows lifecycle status.
- `advanced-evidence/WINDOWS_ACCEPTANCE.md` — full canonical Windows acceptance procedure.
- `advanced-evidence/acceptance-*.ps1` — metadata/hash-only evidence helpers.

The installed application also provides a **Tester Checklist** that can copy or download a Markdown report without including ROM bytes or ROM paths.

## Exact-installer Windows lifecycle

Before upload, the workflow exercises the exact installer copied into the tester kit. The artifact is not published unless that installer passes:

- SHA-256 re-verification immediately before installation;
- silent NSIS install on a clean Windows runner;
- expected installed application/uninstaller discovery;
- installed-content ROM/emulator/save-state boundary scan;
- installed application launch and survival smoke;
- silent uninstall;
- preservation of roaming and local application-data markers.

`BUILD_INFO.json` records `windowsLifecycleStatus: PASS` only after those checks complete.

## Explicitly excluded

The workflow scans the assembled kit and fails if it finds:

- `.sfc` or `.smc` ROM images;
- `.srm` SRAM files;
- `.state` or `.savestate` emulator save states;
- `SUPERZSNES.exe`.

The kit also must not contain private signing keys, certificates, passwords, or production signing secrets.

## Release meaning

A successful community tester kit proves the exact packaged Windows tester installer builds, passes its automated install/launch/uninstall/data-preservation lifecycle, and respects the ROM/emulator/save-state boundary. It does **not** prove production Authenticode signing, Tauri updater signing, real-ROM functional acceptance, or installed-update-from-predecessor acceptance.

Those remain separate production release gates.
