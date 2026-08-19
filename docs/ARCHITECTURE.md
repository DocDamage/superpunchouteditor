# Architecture Overview

## State flow

The stable editor uses one backend-owned ROM session:

`immutable BaseRom -> validated EditJournal -> MaterializedRom -> previews/save/patch/comparison/project/emulator`

The frontend owns presentation state, selections and user preferences. ROM-derived edit state is a projection from the backend and must not become an independent authority.

## Core domain

`rom-core::edit` provides checked range validation, immutable base identity, edit operations/transactions, the ordered undo/redo journal, deterministic materialization, revision projections and changed-range summaries. The domain layer has no Tauri or React dependency.

`AppState::rom_session` is authoritative. `AppState::rom` and `pending_writes` are temporary compatibility projections for not-yet-stable developer surfaces; persistent output must never reconstruct state from them.

Complex legacy writers migrate through `commit_rom_transform`: a writer executes against a scratch materialized ROM, the backend computes the resulting byte delta, and that delta becomes one atomic journal transaction.

## Output pipeline

- ROM Save As writes one `MaterializedRom` snapshot through a same-filesystem temporary file, verifies length/SHA-1, preserves a backup before overwrite and then persists.
- IPS/BPS export compares immutable base bytes to the same current bytes and applies the generated patch in memory before writing it.
- Comparison reads immutable base and materialized current bytes with checked ranges.
- Project v2 serializes the complete journal and expected current hash.
- The stable embedded-emulator path loads the materialized bytes in memory and returns the loaded revision/hash.

## IPC

Tauri commands are thin adapters. Stable literal frontend invokes are mechanically compared with the registered handler. Unregistered calls are permitted only when explicitly listed as gated experimental/research-blocked commands with a reviewed reason.

Mutation success means a durable journal transaction exists. Research-blocked operations return explicit errors and do not change dirty state.

## Frontend state

`featureMaturity.ts` controls stable navigation. Production exposes stable features only. Experimental features require a development build plus `VITE_ENABLE_EXPERIMENTAL=true`; research-blocked features stay hidden.

Zustand may persist user preferences. It must not persist a second editable ROM model, pending byte map, or stale ROM-derived object graph.

## Security

The main window has a restrictive CSP and least-privilege capability set. Automatic filesystem writes are app-owned or user-selected and validated. Layout-pack/project logical paths reject traversal. Stable plugin execution is disabled until a constrained trust/capability sandbox exists.

## Release invariants

A release claim requires current evidence on the exact commit. Public CI uses synthetic fixtures. User-owned real ROM verification remains an opt-in local release-candidate gate and must not upload ROM contents.
