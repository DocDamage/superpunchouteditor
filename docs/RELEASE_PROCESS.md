# Release Process

## Channels

- **alpha** — incomplete/internal validation; breaking migration changes allowed.
- **beta** — feature scope mostly fixed; known experimental areas remain gated.
- **release candidate** — no planned stable-feature changes; full certification required.
- **stable** — all stable claims have current evidence and no open P0/P1 defects.

## Version authority

The workspace package version is authoritative. CI verifies that the frontend package and Tauri bundle version match it. Project schema versions are independent of application versions. A production tag must be exactly `v<workspace-version>`; the release workflow rejects version/tag drift before building.

## Windows release gates

Windows is the active release-priority platform for this milestone. A Windows release candidate requires:

1. Clean checkout/dependency install using committed lockfiles.
2. Windows Rust format/check/strict-Clippy/tests on the exact candidate revision.
3. Frontend `npm ci`, type check, production build, tests and high-severity npm audit.
4. IPC contract, repository hygiene, release-contract and version/updater checks.
5. RustSec audit.
6. Synthetic canonical-output integration evidence: journal materialization = IPS result = BPS result = project-v2 restoration.
7. The automated Windows NSIS install/launch/uninstall lifecycle gate.
8. Opt-in local real-ROM acceptance using a user-owned ROM and metadata/hashes only.
9. Production Windows code signing and Tauri updater signing, including cryptographic verification that the generated updater signature matches the public key committed in `tauri.conf.json`.
10. Published CycloneDX SBOM and SHA-256 release checksums.
11. Updater staging from the previous supported stable release once such a release exists.
12. Final feature-maturity matrix and known-limitations review.

macOS and Linux certification is intentionally separate while Windows is the active milestone. Those platforms may continue to run in source CI as useful shared-code signals, but they are not part of the tagged production release job and do not block a Windows-only release claim.

## Windows-first acceptance

The detailed Windows gate is defined in [WINDOWS_ACCEPTANCE.md](WINDOWS_ACCEPTANCE.md). It has two evidence layers before release promotion:

- automated source/package/install/launch/uninstall certification on the exact candidate revision;
- local, metadata-only real-ROM acceptance using a user-owned ROM.

The automated package gate builds a real x64 NSIS installer, verifies its hash, installs it on a clean Windows runner, launches the installed application, rejects accidentally bundled ROM/emulator content, uninstalls it, and verifies default uninstall preserves application-data markers.

The local real-ROM gate must demonstrate that edit → undo/redo → saved ROM → IPS/BPS → comparison → project-v2 reopen → embedded emulator → optional external emulator all consume the same canonical edited revision. `scripts/windows/acceptance-preflight.ps1` records installer/ROM/emulator hashes and Windows metadata without copying ROM bytes.

## Production signing and updater

`.github/workflows/release.yml` is the production, tag-triggered Windows release path. It is fail-closed: no unsigned stable fallback exists.

Required GitHub Actions secrets for the currently implemented PFX certificate path:

- `TAURI_SIGNING_PRIVATE_KEY` — Tauri updater private key content or path.
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` — optional updater key password.
- `WINDOWS_CERTIFICATE` — base64-encoded PFX code-signing certificate payload.
- `WINDOWS_CERTIFICATE_PASSWORD` — optional PFX export password.
- `WINDOWS_TIMESTAMP_URL` — timestamp authority URL supplied by the certificate provider.

The workflow imports the PFX into the current-user certificate store, creates a temporary Tauri signing override, builds only the Windows x64 NSIS bundle, and then requires Windows Authenticode validation to report `Valid` for both the application executable and installer.

Updater signing is verified independently from Windows Authenticode. `testing-core` contains the `verify_updater_signature` release utility. Before a tag build it parses the updater public key from `tauri.conf.json` with the same locked minisign-verification dependency used by the updater stack. After Tauri creates the updater `.sig`, the workflow verifies the installer bytes against that signature and the committed public key with legacy signatures disabled. A missing signature, malformed public key, wrong updater private key, or invalid signature therefore fails the production release before release metadata is accepted.

The repository never stores private signing material. If the selected modern code-signing provider uses a hardware/cloud identity that cannot be exported as a PFX, adapt the release workflow to that provider through Tauri's supported `bundle.windows.signCommand` mechanism before creating a production tag. Do not weaken the signature checks to accommodate a different provider.

Tauri updater configuration remains in `tauri.conf.json`: updater artifact generation must stay enabled, the public key must be a valid minisign public key, and the canonical static feed remains the repository `latest.json` release asset. The private key is supplied only to the production release environment.

## Release metadata

Every production Windows draft release must contain at least:

- the Authenticode-signed x64 NSIS installer;
- the corresponding cryptographically verified Tauri updater `.sig` file;
- `latest.json` for the static updater feed;
- `SBOM.cdx.json` generated from committed Rust/npm dependency state;
- `SHA256SUMS-windows.txt` covering the installer, updater signature and SBOM.

CI runs `scripts/ci/check_release_contract.py` so these controls cannot be removed silently. `scripts/release/generate_sbom.py` and the updater public-key parser are exercised before merge rather than being first-run on a production tag.

## Updater acceptance still required before stable publication

Cryptographically verifying the generated signature closes the private-key/public-key mismatch failure mode, but it does not replace an end-to-end updater staging test. Before publishing a stable release, verify with an installed prior release that:

- a normal signed update is accepted and installed through the application updater;
- a tampered/invalid signature is rejected by the installed application;
- normal version comparison does not downgrade users unexpectedly;
- missing or interrupted update assets fail without damaging the installed application;
- project and application-data directories survive failed update/recovery paths.

If there is no previous stable release yet, record updater staging as **blocked by missing predecessor** rather than claiming it passed.

## Rollback

If a release or updater manifest is defective:

1. stop/replace the affected release metadata;
2. retain the previous known-good artifacts;
3. publish a corrected signed manifest or newer fixed release rather than reusing an already-published version number;
4. document any manual recovery required;
5. verify project/app-data directories remain untouched by application rollback.

## Release evidence

The release PR/tag notes must identify the exact commit, CI runs, installed-smoke results, real-ROM metadata-only verification status, updater/signing verification, checksums/SBOM, stable feature list and accepted P2 issues. A historical plan or prior green commit is not sufficient evidence for the release candidate being shipped.
