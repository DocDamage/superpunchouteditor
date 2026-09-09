# Repository instructions

## Authoritative branch

`main` is the authoritative development and pull-request target after the September 9, 2026 branch consolidation. This explicitly supersedes earlier handoffs directing work to `master` or describing `main` as obsolete.

Create new feature branches from current `origin/main` and open pull requests against `main`. Do not continue development on the retired `master`, `agent/*`, or `codex/project-audit-cleanup-plan` milestone branches. Their historical work is preserved by the consolidation; they are not parallel development tracks.

## Preservation and validation

Preserve the canonical BaseRom -> EditJournal -> WorkingRom model and all production signing, release, and artifact-boundary checks. Never commit or bundle ROMs, SRAM/save states, third-party emulator binaries, private keys, or signing credentials.

Windows remains the release-priority platform. Do not claim local real-ROM acceptance, installed updater acceptance, or signed production readiness without the corresponding evidence. Follow CONTRIBUTING.md and docs/RELEASE_PROCESS.md for validation.

See docs/BRANCH_CONSOLIDATION_2026-09-09.md for the source snapshot and history-preserving merge decisions.
