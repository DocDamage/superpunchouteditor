# Contributing

## Supported development baseline

Use the committed Rust toolchain file and frontend engine constraints; do not substitute newer toolchains merely to make a local warning disappear. Install dependencies from committed lockfiles with `cargo` and `npm ci`.

## Before opening a pull request

From the repository root run:

```text
cargo fmt --all -- --check
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
python scripts/ci/check_command_contract.py
python scripts/ci/check_versions.py
python scripts/ci/check_repository_hygiene.py
```

Then from `apps/desktop` run:

```text
npm ci
npx tsc --noEmit
npm test
npm run build
npm audit --audit-level=high
```

Public tests must use synthetic fixtures. Do not commit ROMs, patches, save states, emulator binaries, private signing material, or generated build trees.

## Mutation rule

A successful stable mutation must commit through the backend canonical `RomSession` / `EditJournal`. Do not add a second pending-write/edit representation. Complex existing writers should run against a scratch materialized ROM and commit the resulting delta as one atomic transaction.

Every new or renamed frontend Tauri invoke must be registered in the same change. Research/experimental calls may remain unregistered only when they are hidden from the stable product and are explicitly documented in `scripts/ci/experimental_frontend_commands.json`.

## Persistent formats

Released persistent formats require migration handling before schema changes. Project format v2 is the current write format; the copyrighted base ROM must never be embedded in a project.

## Feature maturity

Update `apps/desktop/src/featureMaturity.ts` and `docs/FEATURE_MATURITY_MATRIX.md` when changing release status. A control is not stable merely because its UI exists.

## Security

Treat project files, layout packs, images, scripts and ROM-like inputs as untrusted. Validate ranges with checked arithmetic, bound file sizes/dimensions, reject traversal, and keep automatic writes inside app-owned or user-selected locations. Plugins remain disabled in stable builds until the trust model in `docs/PLUGIN_SECURITY_MODEL.md` is implemented and reviewed.

## Definition of done

A task is complete when its success/failure behavior is implemented, current tests pass, required IPC/persistence migrations are included, user-facing limitations are documented, and verification evidence corresponds to the exact commit under review. Do not hide follow-up correctness/security work in an untracked TODO.
