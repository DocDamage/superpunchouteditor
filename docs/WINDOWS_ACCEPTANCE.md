# Windows Release Acceptance

This document is the Windows-first acceptance procedure for Super Punch-Out!! Editor. It complements automated CI without requiring copyrighted ROM data or third-party emulator binaries in the repository.

## Evidence layers

Windows release confidence is split into four layers:

1. **Source certification** — format, compile, strict Clippy, Rust tests, frontend tests, IPC contracts, repository hygiene, version checks and security audits.
2. **Package certification** — a clean Windows runner builds a real x64 NSIS installer and records its SHA-256.
3. **Installed lifecycle certification** — the NSIS installer installs silently, the installed application launches and remains alive, the bundle contains no ROM or unlicensed external emulator, uninstall succeeds, and application-data markers survive default uninstall.
4. **Local real-ROM acceptance** — a user-owned ROM is exercised through the actual editor and emulator paths on Windows.

Layers 1–3 are automated. Layer 4 is intentionally local and opt-in.

## Copyright and redistribution boundary

Never commit, upload as a CI artifact, attach to a release, or bundle:

- `.sfc` or `.smc` ROM images;
- save-state or SRAM files containing copyrighted game data;
- `SUPERZSNES.exe` or another emulator unless redistribution rights have been independently verified.

CI uses synthetic fixtures. Real-ROM acceptance uses files already owned by the tester and records metadata/hashes only.

## Automated Windows installer gate

`.github/workflows/windows-package-smoke.yml` must pass on the exact release-candidate source revision.

The gate verifies:

- Node and Rust toolchain setup on a clean Windows runner;
- frontend production build;
- optimized Tauri application build;
- x64 NSIS installer generation;
- non-empty installer and SHA-256 calculation;
- silent current-user installation;
- expected installed application and uninstaller;
- absence of `.sfc`, `.smc`, and `SUPERZSNES.exe` in the installed bundle;
- a 10-second installed-app launch smoke;
- silent uninstall;
- preservation of roaming and local app-data markers on default uninstall.

The workflow also syntax-checks every `scripts/windows/acceptance-*.ps1` helper and exercises the acceptance toolkit against synthetic ROM-like files. Equivalent saved/BPS/IPS/project-restored outputs must be accepted, manually overriding a hash-proven equivalence field must be rejected, a complete local evidence matrix must summarize as `PASS`, unsigned installer evidence must fail the signed-installer requirement, and a deliberately tampered BPS output must be rejected.

The pull-request smoke installer disables updater-artifact generation only. It is **not** evidence that release signing is configured. Tagged release artifacts must still pass the signed release workflow.

### Downloadable acceptance kit

Each successful Windows Package Smoke run uploads two short-lived artifacts:

- `super-punch-out-editor-windows-nsis-smoke` — the unsigned functional NSIS installer only;
- `super-punch-out-editor-windows-acceptance-kit` — the same installer plus this guide and all `scripts/windows/acceptance-*.ps1` helpers.

The acceptance kit is intended to make local real-ROM testing self-contained. Its installer is still an **unsigned smoke build** and must not be treated as a stable production release. A stable candidate must come from the signed tag workflow and pass the production Authenticode/updater-signature gates.

## Local acceptance prerequisites

Use a Windows 11 machine representative of the target audience.

Required:

- the exact release-candidate installer or installed build;
- a legally obtained, user-owned supported Super Punch-Out!! ROM;
- enough writable disk space for temporary ROM/project/patch outputs.

Optional external-emulator test:

- an emulator executable supplied locally by the tester.

Before testing, record SHA-256 for the installer, ROM and optional emulator. `scripts/windows/acceptance-preflight.ps1` can generate a metadata-only evidence file.

Example:

```powershell
./scripts/windows/acceptance-preflight.ps1 `
  -RomPath 'C:\Games\Super Punch-Out!!.sfc' `
  -InstallerPath 'C:\Builds\Super-Punch-Out-Editor-Setup.exe' `
  -EmulatorPath 'C:\Emulators\SUPERZSNES.exe' `
  -GitCommit '<exact-release-candidate-sha>'
```

## Automated canonical artifact evidence

After steps 3–6 below produce their output ROMs, use `scripts/windows/acceptance-verify-artifacts.ps1` to prove the canonical outputs are byte-identical and that the source ROM still matches the preflight hash. The helper records filenames, sizes and SHA-256 values only; it never copies ROM bytes into the evidence file.

```powershell
./scripts/windows/acceptance-verify-artifacts.ps1 `
  -EvidencePath '.\windows-acceptance-evidence.json' `
  -RomPath 'C:\Games\Super Punch-Out!!.sfc' `
  -SavedRomPath '.\outputs\edited.sfc' `
  -BpsPatchedRomPath '.\outputs\from-bps.sfc' `
  -IpsPatchedRomPath '.\outputs\from-ips.sfc' `
  -ProjectRestoredRomPath '.\outputs\from-reopened-project.sfc'
```

If IPS is explicitly unsupported for the chosen edit, use `-IpsUnsupported` instead of `-IpsPatchedRomPath`.

The verifier fails closed if:

- the source ROM size/hash changed since preflight;
- BPS output differs from the saved materialized ROM;
- supplied IPS output differs from the saved materialized ROM;
- supplied project-restored output differs from the saved materialized ROM;
- both `-IpsPatchedRomPath` and `-IpsUnsupported` are supplied.

On success it writes a sibling `*.verified.json` evidence file by default, adds an `artifactVerification` metadata section, and records `PASS` for the hash-proven equivalence fields.

## Record manual/visual gate results

Use `scripts/windows/acceptance-record-status.ps1` for observations that require the installed editor or emulator. The recorder deliberately cannot change hash-proven fields such as saved-ROM/BPS/IPS/project equivalence; those fields belong only to `acceptance-verify-artifacts.ps1`.

Examples:

```powershell
./scripts/windows/acceptance-record-status.ps1 `
  -EvidencePath '.\windows-acceptance-evidence.verified.json' `
  -Field loadValidation `
  -Status PASS `
  -Note 'Supported USA ROM loaded without mutation.'

./scripts/windows/acceptance-record-status.ps1 `
  -EvidencePath '.\windows-acceptance-evidence.verified.json' `
  -Field embeddedEmulatorCurrentRevision `
  -Status PASS `
  -Note 'Edited palette was visible after stop/restart.'
```

Use the same helper for the automated-gate fields when transcribing the exact candidate's already-green CI/package evidence. External-emulator acceptance may be `N/A` when no locally supplied external emulator is part of the release claim.

## Generate the release evidence matrix

`scripts/windows/acceptance-summary.ps1` turns the evidence JSON into a Markdown matrix suitable for release notes or a release issue.

During functional acceptance with an unsigned smoke installer:

```powershell
./scripts/windows/acceptance-summary.ps1 `
  -EvidencePath '.\windows-acceptance-evidence.verified.json' `
  -RequireCompleteLocalAcceptance
```

For final stable-release evidence using the signed production candidate:

```powershell
./scripts/windows/acceptance-summary.ps1 `
  -EvidencePath '.\windows-acceptance-evidence.verified.json' `
  -RequireCompleteLocalAcceptance `
  -RequireSignedInstaller
```

`-RequireCompleteLocalAcceptance` fails if any required local gate is unrecorded/failed or the source-ROM immutability proof is missing. `-RequireSignedInstaller` additionally requires the preflight Authenticode status to be `Valid`. Tauri updater signature/public-key verification remains authoritative in the tagged production release workflow.

## Real-ROM canonical-output acceptance

Use a fresh temporary working directory. Do not modify the source ROM in place.

### 1. Load and validate

- Launch the installed editor.
- Open the user-owned ROM.
- Confirm region/title detection succeeds and no unsupported-ROM warning is hidden.
- Confirm the source ROM hash shown/recorded by the acceptance evidence matches the file being tested.

**Pass:** the ROM loads without mutation and the editor reports a usable session.

### 1a. Verify assembled boxer graphics

- Open a boxer in the stable editor workflow.
- Confirm **Assembled Pose Preview** shows a complete pose.
- Optionally use the pose selector or **Prev/Next** controls to inspect another pose.
- Confirm the separate **Raw Tile Banks** view is labeled as an individual-tile reference; it may look chopped or out of order by design.

**Pass:** the assembled preview is complete and the UI makes the distinction between a game-facing pose and raw tile-bank editing clear. Record the boxer, pose index, and error text if rendering fails.

### 2. Make one obvious reversible edit

Choose a small, visually verifiable stable edit such as a palette/color or supported roster/text value.

- Record the original value.
- Apply the edit.
- Confirm the editor becomes dirty.
- Undo and confirm the original value returns.
- Redo and confirm the edited value returns.

**Pass:** edit, undo and redo all describe the same logical transaction and the dirty state is correct.

### 3. Save a materialized ROM

- Save to a new `.sfc` output path.
- Never overwrite the original test ROM.
- Record the saved ROM SHA-256.
- Reopen the saved ROM in a separate editor session if supported by the tested workflow.

**Pass:** the saved image contains the redone edit and the source ROM remains byte-for-byte unchanged. The artifact verifier rechecks source-ROM size/hash against the preflight record.

### 4. Export IPS and BPS

From the same edited revision:

- export IPS where the edit is representable without unsupported expansion;
- export BPS;
- apply each patch to a fresh copy of the original ROM using the editor/test tooling;
- hash the resulting images.

**Pass:** patched output bytes equal the editor's materialized saved-ROM bytes for the same revision. If IPS is explicitly unsupported for the edit, the operation must fail clearly rather than silently produce a different image. Use the artifact verifier to prove BPS/IPS equality rather than comparing hashes manually.

### 5. Comparison/report path

- Open the comparison view/report for the same edited revision.
- Confirm the changed region includes the edit made in step 2.
- Confirm comparison is against the immutable base ROM rather than an already-mutated working buffer.

**Pass:** comparison describes the same materialized bytes saved/exported above.

### 6. Project v2 persistence

- Save the project/session.
- Close the editor completely.
- Reopen the project using the original source ROM when prompted/required.
- Confirm the edit journal, undo/redo cursor and dirty/saved state restore correctly.
- Materialize/save again and hash the result.

**Pass:** the reopened project produces bytes identical to the earlier materialized edited ROM. Supply that reopened output to `-ProjectRestoredRomPath` so the artifact verifier records the equivalence automatically.

### 7. Embedded emulator

- Launch the embedded emulator from the edited session.
- Navigate far enough to verify the chosen edit in-game where practical.
- Stop/restart emulation once.

**Pass:** the emulator consumes the current materialized revision rather than the immutable base ROM or stale saved output.

### 8. External emulator

If an external emulator is available locally:

- configure its executable path in the editor;
- launch it through the editor;
- verify the temporary ROM it receives reflects the current edited revision;
- close the emulator and repeat after one additional reversible edit.

**Pass:** external launch follows the current materialized edit journal each time. No emulator binary is copied into the editor installation or project.

### 9. Recovery and non-destructive behavior

- Trigger a safe-save conflict or choose an existing output path where the UI offers overwrite protection.
- Cancel once and confirm no target file changes.
- Complete a valid save and confirm expected recovery/backup behavior.
- Uninstall the application without selecting any explicit app-data deletion option.

**Pass:** cancellation is non-destructive, safe-save behavior is visible, and projects/preferences remain after default uninstall.

## Acceptance evidence record

Record this metadata in the release issue/notes. Do not attach ROM bytes.

```text
Application version:
Git commit:
Windows version:
Installer filename:
Installer SHA-256:
Installer signed: yes/no
ROM filename:
ROM size:
ROM SHA-256:
External emulator name/version (optional):
External emulator SHA-256 (optional):

Automated Windows source gate: PASS/FAIL
Automated NSIS package gate: PASS/FAIL
Automated install/launch/uninstall gate: PASS/FAIL
Load/validation: PASS/FAIL
Edit + undo/redo: PASS/FAIL
Saved-ROM equivalence: PASS/FAIL
IPS equivalence or explicit unsupported result: PASS/FAIL/N/A
BPS equivalence: PASS/FAIL
Comparison path: PASS/FAIL
Project-v2 restore equivalence: PASS/FAIL
Embedded emulator current-revision test: PASS/FAIL
External emulator current-revision test: PASS/FAIL/N/A
Safe-save/recovery: PASS/FAIL
Default-uninstall data preservation: PASS/FAIL

Known issues / notes:
Tester:
Date:
```

## Windows release decision

A Windows release candidate may advance when:

- Windows source/lint/security gates are green on the exact candidate revision;
- the Windows NSIS package/lifecycle smoke is green on that revision;
- the local real-ROM canonical-output acceptance is recorded as passing;
- production signing/updater credentials are configured and a signed candidate is verified;
- no open Windows P0/P1 defect remains.

macOS and Linux certification can be tracked separately while Windows is the active release priority; shared-code failures discovered there should still be investigated before a stable multi-platform claim.
