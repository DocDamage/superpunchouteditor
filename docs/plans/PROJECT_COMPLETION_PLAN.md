# Project Completion Plan

Last updated: 2026-03-21

## Purpose

This document defines what must be finished for Super Punch-Out!! Editor to be considered complete for its intended scope.

It is broader than the technical debt plan and broader than the roster/creator status document. It covers:

1. feature completion
2. architectural cleanup
3. testing and release readiness
4. packaging and portability
5. documentation and support expectations

This is the project-level source of truth for "what is left."

## Definition of Complete

The project should only be called 100% complete when all of the following are true:

1. All user-visible editor areas are either fully implemented or intentionally removed from the product surface.
2. There are no placeholder UIs presented as active features.
3. Core ROM-editing flows are stable across supported ROM regions and supported expansion states.
4. Save, export, patch generation, project save/load, and embedded test flows all operate on one coherent ROM state model.
5. The app can be built, packaged, launched, and used on a clean machine without machine-specific path assumptions.
6. Critical workflows have regression coverage and the warning volume is low enough that real regressions are visible.
7. Documentation reflects the actual shipped behavior.

## Scope Lock

To finish the project, the team must explicitly lock what "complete" means.

These capabilities should be treated as in scope:

1. ROM loading, validation, region handling, save/export, patch export
2. roster editing, expanded roster support, and in-ROM creator-assisted creation
3. asset editing and import for supported boxer graphics
4. text editing for all exposed text categories
5. audio browsing, preview, and export for supported audio assets
6. embedded emulator-based testing
7. project save/load
8. plugin management if plugins remain visible in the shipped UI
9. frame tags/annotation if that surface remains visible in the shipped UI

Anything still visible in the UI but not actually complete must be resolved one of two ways:

1. finish it
2. hide/remove it from the shipped product

## Current Baseline

Already substantially complete:

1. ROM-backed roster editing
2. expanded roster support
3. creator runtime hook and embedded emulator monitor
4. emulator-core-owned creator commit/cancel resolution
5. live edited-ROM reload into embedded emulator
6. portrait import staging and asset-owner workflow
7. ROM-load state reset on openRom() — all stale state is cleared before each load
8. Manifest path resolution is portable — hard-coded developer paths removed
9. Undo polling removed — undo state is event-driven
10. Rust backend compiles with 0 warnings
11. Plugin manager wired to real Tauri commands — stub hooks replaced
12. Frame reconstructor commands registered in lib.rs — Frames tab is now backend-connected
13. Audio placeholder tabs removed — only Sounds and Music tabs remain
14. GIF recording simulation removed — replaced with honest "not implemented" notice
15. alert() calls replaced across all components with toast notifications (ToastContainer)

Still clearly incomplete:

1. ~~text editor persistence gaps (victory quotes, menu text, credits still mock)~~ — resolved: all three text write-back paths are real ROM persistence (victory quotes, boxer intros, cornerman texts); menu text and credits removed from UI
2. ~~frame tag persistence (FrameTagger UI has no backend persistence; gracefully degrades)~~ — resolved: full frame tag backend implemented in frame_tags.rs using FrameTagManager in AppState
3. ~~frame annotations backend (get_fighter_annotations not yet registered)~~ — resolved: registered; get_fighter_annotations returns empty HashMap, degrades gracefully
4. ~~store consolidation — romStore.ts deprecated but modular stores still exported from index.ts~~ — resolved: store/index.ts now exports only useStore and useUiStore
5. project save/load state consistency: saves all pending_writes per-edit with MD5 hash; validated against ROM SHA1 on load — **confirmed correct**
6. packaging/bundled resource smoke verification — pending
7. ~~test coverage~~ — 48 rom-core unit tests passing including new intro field and cornerman write-back round-trip tests; asset-core sprite bin tests also pass
8. ~~documentation drift — ROSTER_EDITOR_IMPLEMENTATION.md still describes old architecture~~ — section 8 added covering text editor ROM write-back and API changes

## Workstream 1: Product Surface Audit and Scope Cleanup

### Goal

Remove ambiguity about what ships and what does not.

### Tasks

1. Audit every top-level tab, panel, and tool entry point in the desktop app.
2. Mark each surface as:
   - complete
   - incomplete but must ship
   - incomplete and should be removed from shipped UI
3. Remove or hide any placeholder-only surface that will not be finished.
4. Remove "coming soon", "placeholder", "research TODO", and stub messaging from user-facing areas that remain visible.
5. Make one explicit list of supported workflows in README and release docs.

### Exit criteria

1. No user-visible feature is mislabeled as available when it is not.
2. The shipped navigation contains only supported workflows.

## Workstream 2: Core ROM State Architecture

### Goal

Ensure the app has one coherent model of the current edited ROM.

### Required work

1. Finish the ROM-load and region-switch reset work described in [TECHNICAL_DEBT_REMEDIATION_PLAN.md](/c:/Users/Doc/Desktop/Projects/SuperPunchOutEditor/TECHNICAL_DEBT_REMEDIATION_PLAN.md).
2. Consolidate frontend ROM state into one authoritative store flow.
3. Define one authoritative rule for how `state.rom`, `pending_writes`, project saves, export, patch generation, and embedded emulator sync relate.
4. Review all commands that currently read directly from `state.rom` and confirm they behave correctly after creator-driven ROM sync.
5. Confirm all commands that rely on `pending_writes` still produce correct output after the new emulator-core commit path.
6. Ensure project save/load captures the real edited state consistently.

### Specific checks

1. Opening a ROM resets boxer selection, palette selection, animation/frame state, and any region-bound derived state.
2. Re-opening the same ROM is idempotent.
3. Loading USA, then JPN, then PAL does not leave stale state behind.
4. Saving ROM, exporting IPS/BPS, and generating comparison data all agree on the edited image.

### Exit criteria

1. There is exactly one authoritative current-ROM state model.
2. Sequential ROM loads across regions are stable.
3. Save/export/project flows produce the same edited data.

## Workstream 3: Packaging and Resource Portability

### Goal

Make the app runnable on clean development and packaged environments.

### Tasks

1. Remove any remaining machine-specific manifest/resource path assumptions.
2. Finish the manifest/resource resolution work from the technical debt plan.
3. Validate bundled libretro core loading for packaged builds on supported platforms.
4. Confirm all required app resources are included in Tauri bundle metadata.
5. Add packaged-build smoke verification steps.

### Exit criteria

1. No hard-coded developer machine paths remain.
2. Packaged builds can resolve manifests and emulator binaries correctly.

## Workstream 4: Roster Editor and Creator Completion

### Goal

Finish the boxer-creation workflow as a polished end-to-end experience.

### Tasks

1. Update [ROSTER_EDITOR_IMPLEMENTATION.md](/c:/Users/Doc/Desktop/Projects/SuperPunchOutEditor/ROSTER_EDITOR_IMPLEMENTATION.md) so it matches the current architecture:
   - creator commit/cancel resolved in `emulator-core`
   - desktop sync only mirrors edited ROM state
2. Decide and document the final supported creator architecture.
3. Finish portrait workflow UX:
   - make portrait owner selection understandable
   - auto-route users to the right asset workflow
   - remove ambiguity around borrowed owner vs dedicated owner
4. Decide whether portrait workflow must be first-class inside creator or whether current asset-manager handoff is the finished design.
5. Polish creator session lifecycle:
   - success/error messaging
   - reseed behavior
   - creator re-entry behavior
   - cancel/reset behavior
6. Add end-to-end regression tests for:
   - create boxer
   - enter creator
   - edit metadata
   - commit
   - save/export
   - reload and verify persistence

### Important scope note

The project should not be blocked on impossible emulator-core behavior that the current libretro binary does not expose. For this repository, creator persistence is complete when:

1. `emulator-core` owns creator runtime validation and ROM mutation
2. the embedded emulator reloads its own edited ROM image
3. the desktop app mirrors that edited image back into application state

### Exit criteria

1. New boxer creation is smooth, understandable, and testable end to end.
2. The creator workflow no longer feels like a diagnostic surface stitched to a draft editor.
3. Docs accurately describe the final architecture.

## Workstream 5: Text Editor Completion

### Goal

Make the text editor a real editor rather than a mixed real/mock surface.

### Known gaps

1. victory quote updates are mock-only
2. menu text updates are mock-only
3. reset-to-default functionality is stubbed
4. credits editor is still placeholder-only

### Tasks

1. Implement real persistence for victory quote editing in [text_commands.rs](/c:/Users/Doc/Desktop/Projects/SuperPunchOutEditor/apps/desktop/src-tauri/src/text_commands.rs).
2. Implement real persistence for menu text editing.
3. Implement text reset/default restore behavior against ROM data.
4. Decide whether credits editing is in scope:
   - if yes, implement it fully
   - if no, remove/hide it from the UI
5. Add robust validation and byte-length enforcement across all text types.
6. Add tests for each text category and reset behavior.

### Exit criteria

1. Every visible text editor tab persists real data.
2. No text tab is placeholder-only.
3. Reset/default behavior works and is tested.

## Workstream 6: Audio Editor Completion

### Goal

Either finish the audio editor properly or narrow the surface so it no longer overpromises.

### Known gaps

1. sound preview is stubbed
2. playback state is not real
3. WAV export is not implemented
4. BRR export is placeholder-only
5. music playback/editing is partially research-stage
6. sample editing UI is explicitly placeholder

### Tasks

1. Decide the final audio scope for version 1.
2. For the selected scope, implement:
   - real preview/playback
   - stop/playback-state control
   - export for supported audio data
3. If sample editing remains out of scope, hide that tab or relabel the product honestly.
4. If SPC file management is intended to ship, make sure load/create/export flows are complete and documented.
5. Replace research TODO messaging in the visible UI.
6. Add tests and manual verification notes for supported audio flows.

### Exit criteria

1. The shipped audio UI only contains features that actually work.
2. Preview/export behavior is real for all advertised supported audio workflows.

## Workstream 7: Plugin System Completion

### Goal

Resolve the mismatch between visible plugin UI and stubbed frontend behavior.

### Tasks

1. Replace the stubbed hooks in [PluginManager.tsx](/c:/Users/Doc/Desktop/Projects/SuperPunchOutEditor/apps/desktop/src/components/PluginManager.tsx) with real command wiring.
2. Validate plugin load/unload/enable/disable flows.
3. Validate plugin command execution and batch job handling.
4. Add failure handling and user-visible status/error reporting.
5. If plugins are not ready for release, remove the plugin UI from the shipped build.

### Exit criteria

1. Plugins either work end to end or are not exposed as a feature.

## Workstream 8: Frame Tags and Annotation Completion

### Goal

Finish the backend for frame tag workflows if the UI remains visible.

### Tasks

1. Implement real frame tag commands in [frame_tags.rs](/c:/Users/Doc/Desktop/Projects/SuperPunchOutEditor/apps/desktop/src-tauri/src/commands/frame_tags.rs).
2. Wire them to the existing frame tag UI/state.
3. Confirm persistence model for annotations and tags.
4. Add tests for create/update/delete/query flows.

### Exit criteria

1. Frame tagging either works end to end or is removed from release scope.

## Workstream 9: Embedded Emulator and External Emulator Quality

### Goal

Make the test workflow reliable and predictable.

### Tasks

1. Finish or explicitly de-scope automatic save-state creation in [settings.rs](/c:/Users/Doc/Desktop/Projects/SuperPunchOutEditor/apps/desktop/src-tauri/src/commands/settings.rs).
2. Audit embedded emulator controls for accuracy and consistency.
3. Verify pause, frame advance, reset, ROM load, save-state, and creator-mode flows together.
4. Remove simulation-style placeholder UX such as fake GIF completion behavior if not implemented for real.
5. Ensure external-emulator launch and embedded-emulator behavior are documented clearly.

### Exit criteria

1. Testing workflows are reliable and there are no fake-complete controls in the emulator UI.

## Workstream 10: UX and Interaction Polish

### Goal

Bring the app to release quality rather than "dev tool with working internals."

### Tasks

1. Replace `alert(...)` usage with proper app notifications, banners, dialogs, or toasts.
2. Standardize destructive/confirm flows.
3. Standardize busy/loading/error states across major editors.
4. Improve wording in panels that still read like internal tooling.
5. Audit keyboard shortcuts, focus behavior, and accessibility for heavy-use surfaces.
6. Audit typography/encoding issues in components that currently show broken glyphs or mojibake.

### Exit criteria

1. The app no longer feels held together by debugging UI.
2. Major interactions follow a consistent pattern.

## Workstream 11: Warning Reduction and Code Health

### Goal

Raise signal quality for ongoing maintenance and release confidence.

### Tasks

1. Work through the warning-reduction track in the technical debt plan.
2. Remove dead code created by feature refactors.
3. Resolve stale docs/comments that describe old architecture.
4. Reduce warning volume in Rust and TypeScript builds to near-zero or a consciously accepted minimum.
5. Remove or justify placeholder comments that are no longer actionable.

### Exit criteria

1. Build output is clean enough that new regressions are obvious.

## Workstream 12: Test Coverage and CI

### Goal

Give the project a credible regression net for release.

### Tasks

1. Add focused tests around ROM-load state reset and region switching.
2. Add coverage for project save/load and pending-write consistency.
3. Add text editor persistence tests.
4. Add audio tests for whatever final supported audio flows ship.
5. Add creator end-to-end tests at the highest feasible layer.
6. Add smoke tests for packaged resource resolution.
7. Define a standard verification matrix for PRs and releases.
8. Add CI guardrails for build/test/doc health if not already present.

### Minimum release verification matrix

1. `cargo check -p tauri-appsuper-punch-out-editor --lib`
2. targeted Rust crate tests for changed subsystems
3. `npx tsc --noEmit`
4. `npm run build`
5. sequential ROM-load smoke test across supported regions
6. create/edit/save/export/reload smoke test

### Exit criteria

1. Core workflows are covered by repeatable automated checks.
2. Release verification is documented and realistic.

## Workstream 13: Documentation and Release Readiness

### Goal

Make the codebase supportable by someone who did not personally build each subsystem.

### Tasks

1. Update README to reflect actual current capabilities.
2. Update or retire outdated proposal/summary docs that conflict with current behavior.
3. Keep the following docs aligned:
   - [PROJECT_COMPLETION_PLAN.md](/c:/Users/Doc/Desktop/Projects/SuperPunchOutEditor/PROJECT_COMPLETION_PLAN.md)
   - [TECHNICAL_DEBT_REMEDIATION_PLAN.md](/c:/Users/Doc/Desktop/Projects/SuperPunchOutEditor/TECHNICAL_DEBT_REMEDIATION_PLAN.md)
   - [ROSTER_EDITOR_IMPLEMENTATION.md](/c:/Users/Doc/Desktop/Projects/SuperPunchOutEditor/ROSTER_EDITOR_IMPLEMENTATION.md)
4. Add a release checklist covering:
   - build/package verification
   - supported ROM versions
   - known limitations
   - migration/project compatibility notes
5. Document what is intentionally out of scope.

### Exit criteria

1. Docs no longer contradict the code.
2. A new maintainer can understand what ships, what does not, and how to verify it.

## Recommended Execution Order

To actually finish the project, the work should be sequenced like this:

1. Scope cleanup and product-surface audit
2. Core ROM state architecture and technical debt remediation
3. Packaging/resource portability
4. Text editor completion
5. Audio editor completion or scope reduction
6. Plugin/frame-tag completion or removal from release scope
7. Creator/roster polish and end-to-end coverage
8. UX polish and warning cleanup
9. Documentation refresh and release checklist
10. Final verification and packaging pass

## Completion Checklist

The project can be marked complete only when every line below is true:

1. No visible feature is placeholder-only.
2. No critical path depends on machine-specific paths or manual state repair.
3. The current ROM state model is unified and coherent.
4. Text editing is fully real for all visible text categories.
5. Audio features exposed in the UI are genuinely implemented.
6. Plugin and frame-tag surfaces are either complete or removed.
7. The boxer-creation workflow is polished and documented.
8. Build/test/package verification passes on a clean run.
9. Documentation matches shipped behavior.
10. Remaining limitations are intentional, documented scope choices rather than unfinished work.
