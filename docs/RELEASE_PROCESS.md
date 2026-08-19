# Release Process

## Channels

- **alpha** — incomplete/internal validation; breaking migration changes allowed.
- **beta** — feature scope mostly fixed; known experimental areas remain gated.
- **release candidate** — no planned stable-feature changes; full certification required.
- **stable** — all stable claims have current evidence and no open P0/P1 defects.

## Version authority

The workspace package version is authoritative. CI verifies that the frontend package and Tauri bundle version match it. Project schema versions are independent of application versions.

## Required pre-release gates

1. Clean checkout/dependency install using committed lockfiles.
2. Rust format/check/Clippy/tests on supported platform matrix.
3. Frontend `npm ci`, type check, production build and tests.
4. IPC contract, repository hygiene and version-consistency checks.
5. RustSec and npm high-severity audits.
6. Synthetic canonical-output integration test: journal materialization = IPS result = BPS result = project-v2 restoration.
7. Installed application smoke tests on each supported platform.
8. Opt-in local real-ROM RC suite using user-owned files and known hashes only; no ROM data uploaded.
9. Updater staging test from the previous supported stable release.
10. Final feature-maturity matrix and known-limitations review.

## Windows-first acceptance

Windows is the active release-priority platform for the current milestone. The Windows gate is defined in [WINDOWS_ACCEPTANCE.md](WINDOWS_ACCEPTANCE.md) and has two required evidence layers before a Windows release candidate can be promoted:

- automated source/package/install/launch/uninstall certification on the exact candidate revision;
- local, metadata-only real-ROM acceptance using a user-owned ROM.

The automated package gate must build a real x64 NSIS installer, verify its hash, install it on a clean Windows runner, launch the installed application, reject accidentally bundled ROM/emulator content, uninstall it, and verify default uninstall preserves application-data markers.

The local real-ROM gate must demonstrate that edit → undo/redo → saved ROM → IPS/BPS → comparison → project-v2 reopen → embedded emulator → optional external emulator all consume the same canonical edited revision. `scripts/windows/acceptance-preflight.ps1` records installer/ROM/emulator hashes and Windows metadata without copying ROM bytes.

macOS and Linux can be certified on their own schedule while Windows is the active milestone. Shared-code defects discovered on those platforms still require triage before a stable multi-platform claim.

## Signing and updater

Release installers and updater artifacts must be signed. The updater feed is a signed static `latest.json` release asset under the official repository. Private signing keys/certificates are never stored in source control. Release automation consumes protected signing material only in the release environment.

Before stable publication verify:

- updater public key matches the signing key used for artifacts;
- invalid signatures are rejected;
- downgrade policy behaves as documented;
- missing/interrupted assets fail without damaging the installed application;
- checksums and SBOM are published with the release.

## Rollback

If a release or updater manifest is defective:

1. stop/replace the affected release metadata;
2. retain the previous known-good artifacts;
3. publish a corrected signed manifest or newer fixed release rather than reusing an already-published version number;
4. document any manual recovery required;
5. verify project/app-data directories remain untouched by application rollback.

## Release evidence

The release PR/tag notes must identify the exact commit, CI runs, installed-smoke results, real-ROM metadata-only verification status, updater/signing verification, checksums/SBOM, stable feature list and accepted P2 issues. A historical plan or prior green commit is not sufficient evidence for the release candidate being shipped.
