# Main branch consolidation — September 9, 2026

## Current authority

`main` is the default and authoritative branch. New feature branches start from `origin/main`; new pull requests target `main`. This replaces the previous instruction to use `master` and leave `main` untouched. Old handoffs and archived plans are historical context, not current branch policy.

## Problem and source selection

Before consolidation, `main` at `d4ee3531f87e2550d701aa7dfb7e7079c73099ff` held only five planning/data files and shared no ancestor with the application on `master`. Nine completed feature branches still appeared unmerged by ancestry because their PRs had been squash-merged.

The application snapshot is `master` commit `8d1d7db1ed5fcd1adfee316845d6e2b713a90215` (August 20, 2026, "Render complete assembled boxer poses"). This includes the Windows remediation, signing-verification, lifecycle, acceptance, guided UX, community tester kit, and newer assembled-pose work. No old feature branch is replayed over this newer application tree.

## History-preserving integration

The reconciliation commit has the original `main` tip as its first parent and the other 12 original branch tips as additional parents. This is an explicit reconciliation of already-integrated or intentionally retired histories, not a claim that every old workflow remains active. The original `main` files are preserved byte-for-byte at `docs/archive/main-before-2026-09-09/`.

| Historical branch | Original tip | Disposition |
| --- | --- | --- |
| `master` | `8d1d7db1ed5fcd1adfee316845d6e2b713a90215` | Current application baseline, including assembled boxer poses |
| `agent/project-remediation-modernization` | `9b357bfacda9125ccd80808af7b85c4e23a9b17b` | PR #1 already merged; exact tree verified |
| `agent/windows-release-readiness` | `00709fe2166d3ef65f7c0be5eb1fbd8f2e6ebe03` | PR #2 already merged; exact tree verified |
| `agent/windows-install-lifecycle` | `7d757437670635125fa1c1e811754cecba39ab28` | PR #3 already merged; exact tree verified |
| `agent/windows-release-pipeline` | `b1429cf4840a7531754db23911d352de2229ce38` | PR #4 already merged; exact tree verified |
| `agent/updater-signature-verification` | `9680d055ab36982a328c32a95e24b2b6cbe7d5b7` | PR #5 already merged; exact tree verified |
| `agent/updater-verifier-test-vectors` | `753fe69ea2a1e818d3d10b731bf38b42aabe9a7b` | PR #6 already merged; exact tree verified |
| `agent/windows-real-rom-evidence-automation` | `fcd5e6ea3c3580b73cd9d863f6774788630a2a75` | PR #7 already merged; exact tree verified |
| `agent/windows-acceptance-kit` | `c28f82e7c0fa80386aa35abeeb0b943b286d83ea` | PR #8 already merged; exact tree verified |
| `agent/usability-community-testing` | `27c9eddd8e24928be28bbdda17086432fa07330d` | PR #9 already merged; exact tree verified |
| `codex/project-audit-cleanup-plan` | `3338eeaf2fd68888af59b9e7c77bb19471becfee` | Roadmap/archive match baseline; cleanup deletions retained; newer documentation retained |
| `agent/codex-handoff-export` | `b2e79f8dc1d078f5300867201d5c3830a99f4842` | PR #11 intentionally closed unmerged; obsolete one-shot export workflow not activated |

Existing branch refs are retained as historical references, not force-updated or deleted. Their original tips are ancestors of the consolidation, so they no longer contain commits ahead of consolidated `main`. Do not push new development to them.

## Checks and boundaries

`Consolidation Verification` checks ancestry for all 13 original tips, exact tree equality for the nine original squash-merged milestones, byte-identical archival of the old `main`, retention of the audit payload/deletions, and exclusion of the retired export workflow. Initial verification passed in Actions run `34316222392` on reconciliation commit `bde78ce303174ebc037c299b7cbbba39df2ae7bf`.

CI, Lint, Windows Package Smoke, and Community Windows Tester Kit are retargeted from repository branch `master` to `main`; their test, security, signing, artifact, and lifecycle steps are not weakened. Third-party action refs such as `dtolnay/rust-toolchain@master` are unrelated to this repository's branch policy and remain unchanged.

This branch repair does not certify local real-ROM behavior, signed production release readiness, or installed updater behavior. Those remain subject to the existing acceptance and release process.
