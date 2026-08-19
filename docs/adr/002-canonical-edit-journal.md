# ADR 002 — Canonical ROM Edit Journal

- **Status:** Accepted
- **Date:** 2026-08-18

## Context

The legacy application had two independent edit representations: some commands mutated `Rom.data` directly while other commands staged bytes in `pending_writes`. Save, patch export, comparison, project persistence, undo/redo, and emulator launch then reconstructed state differently. This made successful edits disappear from patches/projects and made output correctness dependent on which command performed the mutation.

## Decision

The backend owns one canonical editing session:

- `BaseRom` — immutable source bytes and source identity.
- `EditOperation` — validated byte/resize operation with captured before/after state.
- `EditTransaction` — one atomic logical user action.
- `EditJournal` — ordered transactions, undo/redo cursor, revision and saved-state information.
- `RomSession` — base ROM + edit journal + source metadata.
- `MaterializedRom` — deterministic current bytes plus base/current hashes, revision and changed ranges.

All supported mutation commands must commit through `RomSession`. Existing complex ROM writers may run against a scratch materialized ROM; their resulting byte delta is committed as one transaction. They may not retain authority over mutable `Rom.data`.

The legacy `rom`, `pending_writes`, and frontend pending-offset structures are compatibility projections only during migration. They must never be used to reconstruct persistent output.

## Output invariant

ROM save, IPS/BPS export, comparison, project persistence and the stable embedded-emulator path consume the same canonical materialization/revision. A new output path that reconstructs edits independently violates this ADR.

## Failure invariant

A user-visible successful mutation must be represented in the journal. Unsupported or research-blocked writes return errors and leave the journal/revision unchanged.

## Persistence invariant

Project format v2 serializes the complete journal and source identity but never the copyrighted base ROM bytes. Loading validates the project in scratch state before replacing the active session.
