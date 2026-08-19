# Troubleshooting

## Clean install fails

Use the committed toolchain/lockfiles. Rust uses the pinned `rust-toolchain.toml`; the frontend must be installed with `npm ci`. If a manifest changed without a matching lockfile, regenerate the appropriate lockfile and include it in the same change rather than falling back to an unpinned install.

## Frontend says a command is missing

Run `python scripts/ci/check_command_contract.py`. Stable/unclassified invokes must be registered in the Tauri handler. Experimental/research-only calls require an explicit reason in `scripts/ci/experimental_frontend_commands.json` and must remain hidden from stable navigation.

## An edit appears in the UI but not in output

This is a correctness defect. Stable mutations must create a canonical journal transaction. Save, patch, project, comparison and embedded-emulator paths must not reconstruct state from a frontend cache or `pending_writes`. Record the exact command/action and current revision, then verify the mutation response changed the backend journal.

## Patch export fails

- If ROM size changed, use BPS; the stable IPS writer rejects expansion.
- A verification failure means the generated patch did not reproduce the current materialized image and the patch is intentionally not written.
- Range/overflow errors indicate an invalid edit or incompatible base ROM and must not be bypassed.

## Project will not load

Check the selected base ROM. Project v2 verifies source SHA-1/size, manifest integrity, every journal operation and the expected current hash before state replacement. A legacy v1 project with edit metadata but no replacement payload cannot restore those edits; metadata-only migration is the honest supported fallback.

## Animation edits return unsupported

This is intentional. Animation/frame/hitbox/hurtbox write-back remains research-blocked until round-trip persistence is proven. Inspection/playback may remain available in experimental builds, but mutation cannot report success.

## Plugins return disabled

This is intentional in stable builds. The previous runtime did not meet the required sandbox/trust model. See `docs/PLUGIN_SECURITY_MODEL.md`.

## Layout pack apply returns unsupported

The current pack format describes layout references and does not contain a complete replacement-byte payload. Import, validation and install are available experimentally; applying a metadata-only pack would be a false success, so the command fails explicitly.

## Embedded emulator tests an old revision

Use the stable current-ROM load path. It loads the backend materialized bytes directly and returns the revision/SHA-1. Do not rely on the legacy external-emulator disk-path flow for release evidence.

## RustSec/npm audit fails

Do not lower the audit threshold to make CI green. Determine whether the vulnerable dependency is used. Remove dead dependencies when possible; otherwise update or document a narrowly scoped, reviewed exception with impact and a removal/update target.

## Release certification needs a real ROM

Public CI cannot contain copyrighted ROM fixtures. Use `scripts/release/real_rom_rc_check.py` locally with a user-owned ROM path and expected SHA-1. Share only the metadata-only JSON result; never commit/upload the ROM.
