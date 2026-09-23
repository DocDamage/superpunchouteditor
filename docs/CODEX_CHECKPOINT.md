# Advanced Feature Completion Checkpoint

Updated: 2026-09-22. Local status and development checkpoint, not release certification.

## Objective
Finish text, audio, AI editing, relocation, layout packs, scripts, comparison,
animation/frame/hitbox editing and plugins. The full objective remains active.
The advanced-feature implementation below is committed as `1f2a260` on
`origin/main`. Local `main` remains at that baseline. Local `work` contains the
reconciliation cleanup: lint fixes in sprite compression, fighter lookup,
comparison, layout-pack export and an AI parser test, plus a scoped permission
fix for the CI Security Audit check. Three lint fixes from closed PR #38 were
recovered into `work`. The retired local `master` ref was removed after
confirming its tip is an ancestor of `main`.

Dependabot PRs #40–#53 were closed without merging because their checks failed.
Their head refs were removed; the remote now contains only `main` and `work`.
PR #54 carries the reconciliation commit from `work` to `main` and is awaiting
CI. PR #38 (`Creative Corner`) remains closed without merge as a recovery
record. Its verified UI work and later Appearance Studio / Game Extras source
handoff are separate from the committed advanced-feature work; the handoff
source has not been recovered in this checkout and remains a follow-up
decision.

Preserve BaseRom -> EditJournal -> WorkingRom and user edits. Never commit ROMs,
SRAM, emulator binaries or private signing material.

## Implemented in the current main baseline
- Text updates/reset use the journal and immutable base; search/statistics read
  current bytes. Intro overflow rejects. Cornerman deletion handles private tables.
- AI decoder covers 16 primary USA intervals. Operand edits preserve sizes,
  validate changed branch targets and full preimages, and commit atomically.
- Animation decoder handles extended instructions. Duration editing targets real
  display-pose instructions in 17 catalog intervals covering all 16 fighters plus
  the upstream reference. Scripts UI selects intervals; commands enforce catalog
  IDs, source identity and stale-byte checks. History changes discard drafts.
  Batch staging applies several durations in one journal transaction. Instructions,
  cross-catalog references and partial timing traces share one source-checked ROM
  snapshot. Reference navigation never treats unknown destinations as instructions.
  Runtime-verified timing handles pose/delay, jump and loop counter operations,
  stopping on unknown state/opcodes/targets or a step limit. Zero delay wraps to
  256 uninterrupted animation updates, not zero time. This is not full playback.
  Latest local-ROM trace coverage: 16 of 17 stock catalog intervals produce one
  or two timing events after modeling $36/$80 and conditional $28 state writes.
  Piston/Narcis reach the end of their selected blocks, NOT proven animation ends.
  None are complete playback. Dominant remaining stops include $1E and $44;
  the reference stops at $76 before any event. This is a substantial remaining
  implementation gap, despite valid decode/edit/Undo results. Run the opt-in
  usa_reference_animation_round_trip test with --nocapture to reproduce counts.
  Runtime evidence for next steps: $36/CODE_018F0C writes $81 to direct-page $76;
  $80/CODE_019085 writes its operand to $0324 and sets bit $20 at $0326. Both
  continue immediately. $28/CODE_018DFF sets four state bytes $68..$6B with ordering
  conditional on direct-page $0E. Model these effects, do not silently skip them.
- Layout packs v2 carry sparse expected/replacement bytes and base SHA1.
  File export/import/application uses the journal. Validation shares Apply checks,
  supports boxer selection and rejects conflicts/overlaps/already-applied edits.
  Installed paths are backend-supplied. Shared graphics export is explicit opt-in.
- Comparison handles exact binary totals and decoded raw/compressed sprite/palette
  changes; PNG modes include split/overlay/difference. Split clips locally; stale
  responses and blob URLs are cleaned up.
- Audio import retains sample rate for preview/export and correct resampling.
- Relocation has checked pointer encoding and real bounded 24-bit LoROM candidate
  scanning with preimages/bank aliases. Unsafe copy/erase execution now rejects.

## Remaining Work
1. Replace legacy structured AI serializer: 13-byte writes versus 12-byte parsing,
   overlapping fields and scattered-versus-contiguous mappings remain unverified.
   Journal backing does not make these writes valid game data.
2. Finish animation discovery, nested pointer-table traversal, control flow,
   playback, pose mapping and hitbox/hurtbox editing. The 17 intervals are selected
   bytecode blocks, not every animation. Legacy loader synthesizes sequences and
   core writer contains no-op methods; these are not persistence evidence.
3. Implement verified live relocation references, short-pointer bank context,
   allocation provenance, atomic writes and journal-projected asset locations.
   Manifest gaps and scanner candidates are not proof of safe relocation.
4. Finish text menu mapping, cornerman insertion, shared-table copy-on-write,
   relocation-aware capacity/reset and game verification of legacy text offsets.
5. Implement actual audio ROM sample mapping/replacement, loops, pointers and
   music sequence editing. Imports currently remain session-only.
6. Finish resized/relocated pack/comparison assets, shared-impact previews and
   reconstructed frame/animation comparison.
7. Implement plugin permissions, isolation, limits and journal writes. Execution
   remains disabled; disabling is not completion.
8. Replace misleading legacy script header/stat labels. Invalid header mutation
   is rejected to avoid overwriting pointers, not implemented.
9. Complete native desktop/game workflows, save/project/patch round-trips, full
   quality/CI gates, documentation reconciliation and signed release acceptance.

## Evidence
Format authority: Yoshifanatic1/Super-Punch-Out-Disassembly commit
3a1bd913e5ff6aefe7c7bcdb2c797919bcea7cba, Routine_Macros_SPO.asm and AI/animation
disassembler scripts. Source catalog blocks contain only corresponding macros;
they are not allocation or complete animation ownership proofs.

USA SHA1: 3604c855790f37db567e9b425252625045f86697. Opt-in tests use SPO_USA_ROM
pointing to the user-supplied local ROM; copyrighted bytes are not in fixtures.

Completed checks at their respective implementation boundaries:
- Desktop library suite after pack/animation integration: 68 passed, 3 ignored
  (the opt-in USA-ROM tests, verified separately at their implementation gates).
- Full configured frontend Vitest suite: 8 files, 31 tests passed.
- Pack backend: 8 passed; pack-manager UI: 3 passed.
- Animation core: 4 passed including all catalog ROM round-trips.
- Desktop catalog test: all 17 edits changed one intended byte, Undo restored full
  original ROM. Reference test also checked stale rejection and Redo.
- Animation UI: 3 passed (payload bounds, stale/history reads, source rejection).
- Earlier targeted suites: 28 roster, 4 desktop text, 23 relocation, 7 comparison
  backend, 3 comparison UI and 4 AI UI tests passed.
- TypeScript passed after catalog UI integration; latest diff check passed.
- Reconciliation checks: `cargo fmt --all -- --check` and
  `cargo clippy --workspace --all-targets -- -D warnings` passed. Targeted
  `asset-core` and `script-core` test suites passed.
- CI workflow hygiene passed after scoping `checks: write` and `contents: read`
  to the Security Audit job.

No test process remains running. Tests above are not gameplay or release proof.
Next substantive milestone: source-backed animation traversal/coverage and
replacement of unsafe legacy AI mappings, not more isolated UI.
