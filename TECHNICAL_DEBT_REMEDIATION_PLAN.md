# Technical Debt Remediation Plan

## Purpose

This document turns the current technical debt audit into an execution plan. The goal is to reduce defect risk in the ROM loading path, remove architectural drift in frontend state management, make manifest loading portable, and raise the baseline for test coverage and warning hygiene.

This plan is intentionally detailed enough to be used as an implementation checklist.

## Scope

The plan covers the following debt areas:

1. Frontend state not refreshing correctly after ROM or region changes
2. Non-portable backend manifest path resolution
3. Region selector UX and API contract mismatch
4. Duplicate ROM state implementations in the frontend
5. Polling-based undo/redo state synchronization
6. Import-time DOM side effects in the region selector
7. Excessive warning volume that reduces build signal
8. Missing tests around the new multi-region ROM flow

## Current Risk Summary

### Highest-risk issues

1. ROM and manifest changes can leave stale boxer and palette data in memory
2. Manifest loading depends on path guessing, including a hard-coded machine-specific Windows path
3. The frontend region selection flow is inconsistent and can trigger duplicate or misleading operations
4. Two different frontend ROM stores already behave differently and can drift further

### Secondary issues

1. Undo state is synchronized through background polling instead of explicit state transitions
2. Region selector styling is implemented with module-scope DOM mutation
3. Backend warnings are numerous enough to hide real regressions

## Guiding Principles

1. Fix correctness before cosmetics
2. Remove duplicate sources of truth before adding features
3. Replace environment-specific assumptions with Tauri-native resource resolution
4. Add tests at each seam where state crosses process boundaries
5. Prefer explicit state transitions over periodic polling

## Execution Order

Implementation should proceed in this order:

1. Stabilize ROM-load state transitions
2. Make manifest loading portable
3. Simplify and correct region-selection behavior
4. Consolidate frontend store architecture
5. Replace undo polling with event-driven or action-driven synchronization
6. Remove import-time side effects
7. Reduce warning volume
8. Backfill tests and CI guardrails

## Workstream 1: Stabilize ROM Load and Region Switch State

### Problem

The backend now loads manifests based on detected ROM region, but the frontend does not fully reset or reload dependent state after `open_rom`. This can leave stale boxer lists, selected boxer data, palettes, frame state, or other derived UI state from a previous ROM or region.

### Objective

Make ROM loading a single explicit state transition that resets dependent state and reloads the correct region-specific data every time.

### Required changes

1. Define the full set of frontend state that is ROM-bound
2. Reset that state immediately before or immediately after a successful ROM load
3. Reload all dependent data from the backend after ROM load
4. Ensure the first usable screen after load reflects the new manifest, not cached frontend state

### State to review and likely reset

1. `boxers`
2. `selectedBoxer`
3. `currentPalette`
4. `currentPaletteOffset`
5. `fighters`
6. `selectedFighterId`
7. `poses`
8. `frames`
9. `currentFrame`
10. `currentFrameIndex`
11. `pendingWrites`
12. `error`
13. Any tab-local selection state that assumes the previous manifest

### Concrete implementation steps

1. Audit `apps/desktop/src/store/useStore.ts` and identify all state fields derived from the currently loaded ROM
2. Introduce a dedicated helper such as `resetRomBoundState()`
3. Update `openRom(path)` to:
   - clear stale ROM-bound state
   - call `invoke('open_rom', { path })`
   - set `romSha1`
   - set `romPath`
   - clear pending writes locally
   - reload boxers
   - reload other required manifest-derived data
4. Decide whether the first boxer should auto-select or the UI should return to an empty state
5. If empty state is preferred, make that explicit and consistent across tabs
6. Ensure any palette load only occurs after boxer selection from the new manifest
7. Verify the region switch case:
   - USA ROM loaded
   - JPN ROM loaded immediately after
   - PAL ROM loaded immediately after

### Acceptance criteria

1. Loading a different ROM never leaves the previous boxer selected
2. Boxer list matches the newly loaded manifest every time
3. Palette state never references offsets from a previous ROM
4. Frame or animation editors do not render stale content after ROM switch
5. Re-loading the same ROM remains stable and idempotent

### Testing requirements

1. Add frontend store tests for `openRom()` reset behavior
2. Add integration coverage for sequential ROM loads across multiple regions
3. Add regression coverage for stale boxer and stale palette scenarios

## Workstream 2: Replace Manifest Path Guessing With Portable Resource Resolution

### Problem

Manifest resolution currently uses path guessing and includes a hard-coded path to a specific Windows development machine. That is not acceptable for release packaging, other developer environments, or long-term maintenance.

### Objective

Use Tauri-native path and resource mechanisms so manifest loading works consistently in development and packaged builds without machine-specific fallbacks.

### Required changes

1. Remove the hard-coded Windows path
2. Stop relying on fragile current-working-directory traversal
3. Resolve manifests through Tauri app resources or a single supported development fallback

### Concrete implementation steps

1. Review how `data/manifests` is bundled into the Tauri app
2. Confirm `tauri.conf.json` resource configuration includes the manifest files
3. Replace `manifest_search_paths()` with a resolution approach that prefers:
   - Tauri resource directory in packaged builds
   - repository-relative development path only when running in dev mode
4. If the command layer does not currently have access to Tauri path resolution, introduce it through application setup or app handle access
5. Log resolved manifest paths in debug builds only
6. Return actionable error messages when a manifest cannot be found

### Design constraints

1. No absolute user-specific paths
2. No more than one explicit dev-mode fallback
3. Error messages must identify the missing region and the resolved lookup base

### Acceptance criteria

1. Packaged Windows build resolves manifests without depending on current working directory
2. Development build resolves manifests from the repo without hard-coded machine paths
3. USA, JPN, and PAL manifests all resolve through the same mechanism

### Testing requirements

1. Add unit tests for region-to-manifest filename mapping
2. Add tests for manifest resolution behavior where feasible
3. Add one smoke test or manual verification checklist for packaged build resource loading

## Workstream 3: Simplify Region Selector Behavior

### Problem

The region selector currently mixes automatic selection behavior with an explicit confirmation button. It also exposes a region argument through the callback, but the app shell ignores that argument and simply re-opens the ROM path.

### Objective

Make region handling explicit, predictable, and aligned with the backend contract.

### Decision required

Choose one of these models and implement it consistently:

1. Auto-load on selection, with no separate submit button
2. Select first, then load only when the user presses a confirmation button

The current UI shows model 2 while behaving partly like model 1. That should be corrected.

### Recommended approach

Use explicit confirmation. Region selection is a meaningful operation and should not fire immediately from a radio change.

### Concrete implementation steps

1. Update `RegionSelector` so radio change only updates local selection state
2. Make the primary action button the only control that triggers `onRegionSelected`
3. Rename the button if needed:
   - `Load ROM`
   - `Open ROM`
   - `Continue`
4. Remove duplicate callback paths
5. Decide whether the frontend region choice should:
   - simply confirm backend auto-detection, or
   - override backend region selection
6. If backend auto-detection remains authoritative, update the UI copy to say so clearly
7. If user override is supported, pass the chosen region all the way to the backend and validate it there

### API cleanup

1. Align `RegionSelectorProps.onRegionSelected`
2. Align `App.tsx.handleRegionSelected`
3. Remove unused parameters or wire them fully through the backend

### Acceptance criteria

1. Selecting a radio option never triggers an unexpected ROM load
2. Exactly one control performs the load action
3. The callback signature matches actual behavior
4. Unsupported-region force-load behavior is explicit and testable

### Testing requirements

1. Add component tests for selection vs confirmation behavior
2. Add regression coverage for unsupported-region force-load flow
3. Add tests proving callback invocation count is exactly one per confirmed action

## Workstream 4: Consolidate Frontend ROM State Into a Single Store

### Problem

There are two ROM state implementations in the frontend:

1. The monolithic `useStore.ts`
2. The modular `romStore.ts`

They already diverge in behavior, especially around post-load data refresh. This is architectural debt with direct bug potential.

### Objective

Reduce the app to one authoritative ROM state flow.

### Recommended approach

Choose one store architecture and finish the migration. Do not maintain both.

### Decision criteria

Use the modular store architecture if:

1. The project is already moving toward separated Zustand slices
2. Cross-domain state dependencies can be clearly expressed
3. The team wants easier isolated testing

Keep the monolithic store only if:

1. The modular migration is incomplete and blocking current delivery
2. There is no real usage of the modular exports outside experiments

### Concrete implementation steps

1. Inventory all imports of:
   - `useStore`
   - `useRomStore`
   - other slice stores
2. Determine which architecture is actually active in production paths
3. Freeze the non-authoritative store:
   - mark it deprecated
   - stop exporting it publicly if possible
4. Migrate active consumers to the chosen store
5. Remove duplicate ROM-loading logic
6. Centralize:
   - ROM load
   - ROM close
   - pending writes
   - error handling
   - post-load refresh sequence

### Acceptance criteria

1. There is one authoritative `openRom` implementation
2. Public exports do not expose redundant ROM stores
3. ROM load behavior is identical everywhere in the app
4. Store tests cover the chosen architecture

### Testing requirements

1. Add unit tests for the canonical ROM store
2. Add lint or review guidance against reintroducing duplicate ROM state

## Workstream 5: Replace Undo Polling With Explicit Synchronization

### Problem

Undo/redo state is refreshed through a one-second polling loop. That adds background IPC traffic and indicates state ownership is not modeled cleanly.

### Objective

Update undo and redo capability only when relevant actions occur.

### Potential approaches

1. Refresh undo state after every edit, undo, redo, and history clear action
2. Emit backend events whenever history changes and subscribe in the frontend

### Recommended approach

Use action-driven refresh first. It is lower risk and easier to verify than introducing event plumbing immediately.

### Concrete implementation steps

1. Inventory every frontend action that mutates edit history
2. Ensure `refreshUndoState()` is called after those actions only
3. Remove the interval from `App.tsx`
4. Verify history-clearing actions also refresh state
5. If some edits originate only from backend-side flows, add event emission for those cases

### Acceptance criteria

1. No interval-based polling remains for undo state
2. Undo/redo button state updates immediately after edits
3. Loading a new ROM clears undo/redo state without delay

### Testing requirements

1. Add store-level tests for history state transitions
2. Add a UI regression test for button enabled/disabled behavior

## Workstream 6: Remove Import-Time DOM Side Effects

### Problem

`RegionSelector.tsx` injects a style tag into `document.head` at module import time. This is fragile under hot reload and makes the component harder to test.

### Objective

Move styling into normal CSS or controlled component lifecycle code.

### Concrete implementation steps

1. Move the spinner keyframes into:
   - `App.css`, or
   - a dedicated component CSS file
2. Remove `document.createElement('style')` usage from module scope
3. Confirm the component still renders correctly after HMR and fresh load

### Acceptance criteria

1. No DOM mutation occurs at module import time
2. HMR does not duplicate style tags
3. Component styling remains unchanged functionally

## Workstream 7: Reduce Warning Volume

### Problem

`cargo check` currently passes with a large warning count. The biggest issue is not any single warning but the total noise floor.

### Objective

Cut warning volume enough that new warnings become visible and meaningful.

### Strategy

Do not try to make the whole repository warning-free in one pass. Triage by production relevance.

### Priority order

1. Warnings in active desktop app paths
2. Warnings in backend command modules
3. Warnings in shared crates used by the desktop app
4. Dead code or placeholder warnings in less active systems

### Concrete implementation steps

1. Classify warnings into buckets:
   - real bug indicators
   - placeholder stubs
   - dead code
   - intentional unused parameters
2. Fix or explicitly underscore intentionally unused parameters
3. Remove dead code where clearly safe
4. Add `#[allow(...)]` only where there is a justified temporary reason
5. Create a warning budget target for the desktop app crate

### Acceptance criteria

1. Warning count in the desktop app backend is materially reduced
2. Remaining warnings are intentional and documented
3. New warning regressions are easy to spot in local builds and CI

## Workstream 8: Test and CI Backfill

### Problem

The new multi-region load path and region selector behavior do not appear to have enough direct test coverage.

### Objective

Add coverage at the layers where regressions are most likely:

1. backend region detection and manifest selection
2. frontend store reset logic
3. frontend selector behavior

### Test matrix

#### Backend

1. USA ROM -> USA manifest
2. JPN ROM -> JPN manifest
3. PAL ROM -> PAL manifest
4. Unknown ROM -> clear error path
5. Missing manifest -> actionable failure

#### Frontend store

1. Open ROM clears stale boxer selection
2. Open ROM clears palette state
3. Open ROM reloads boxer list
4. Sequential ROM loads remain stable

#### Frontend component

1. Region selection does not auto-submit if confirmation model is chosen
2. Confirm button invokes callback once
3. Unsupported-region force-load path is explicit

### CI recommendations

1. Add a frontend test target if one is not currently enforced
2. Ensure `cargo test` for the desktop backend runs in CI
3. Add `vite build` and `cargo check` as baseline gates if not already present
4. Consider a warning budget gate for the desktop backend crate after the first cleanup pass

## Suggested Delivery Phases

### Phase 1: Correctness and portability

1. Workstream 1
2. Workstream 2
3. Workstream 3

### Phase 2: Architectural cleanup

1. Workstream 4
2. Workstream 5
3. Workstream 6

### Phase 3: Maintainability hardening

1. Workstream 7
2. Workstream 8

## Detailed Task Breakdown

### Phase 1 tasks

1. Refactor `openRom()` to perform complete ROM-bound state reset
2. Reload boxer data after successful ROM load
3. Validate region-switch behavior manually across USA, JPN, and PAL ROMs
4. Remove hard-coded Windows manifest path
5. Implement portable manifest resolution
6. Simplify region selector flow to one submission path
7. Align region-selection callback signature and semantics

### Phase 2 tasks

1. Map all current consumers of ROM state
2. Select the canonical store architecture
3. Remove or deprecate duplicate ROM store logic
4. Delete undo polling interval
5. Refresh undo state through explicit actions
6. Move spinner styling into CSS

### Phase 3 tasks

1. Triage warnings by module and severity
2. Eliminate obviously safe unused and dead-code warnings
3. Add missing backend and frontend regression tests
4. Add CI checks or tighten existing ones

## Definition of Done

This debt plan is considered complete only when all of the following are true:

1. Switching ROMs across regions does not leave stale frontend state
2. Manifest loading works without machine-specific paths
3. Region selection behavior is internally consistent
4. Only one ROM store implementation is authoritative
5. Undo/redo state is no longer polled every second
6. No import-time DOM mutation remains in the region selector
7. Warning volume is materially lower and more intentional
8. Multi-region load flows have direct regression coverage

## Recommended First Implementation Slice

If this work needs to be broken into the smallest useful first PR, do this:

1. Fix `openRom()` state reset and boxer reload
2. Remove the hard-coded manifest path
3. Simplify region selector to confirmation-only behavior
4. Add regression tests for sequential ROM loads

That slice addresses the highest-risk defects without forcing the full store architecture cleanup immediately.

## File Areas Likely To Change

### Frontend

1. `apps/desktop/src/App.tsx`
2. `apps/desktop/src/App.css`
3. `apps/desktop/src/components/RegionSelector.tsx`
4. `apps/desktop/src/store/useStore.ts`
5. `apps/desktop/src/store/romStore.ts`
6. `apps/desktop/src/store/index.ts`

### Backend

1. `apps/desktop/src-tauri/src/commands/rom.rs`
2. `apps/desktop/src-tauri/src/commands/region.rs`
3. `apps/desktop/src-tauri/src/utils/mod.rs`
4. `apps/desktop/src-tauri/src/lib.rs`
5. `apps/desktop/tauri.conf.json` or equivalent Tauri config if resource bundling needs adjustment

### Tests

1. Desktop backend command tests
2. Frontend store tests
3. Frontend component tests

## Notes

This plan intentionally prioritizes correctness and architecture over visual polish. The UI quality issue is real, but the faster way to make the interface trustworthy is to first fix the state model and interaction flow underneath it. Once those seams are stable, visual cleanup can happen on top of predictable behavior instead of masking deeper defects.
