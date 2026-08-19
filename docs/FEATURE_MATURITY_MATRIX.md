# Product Feature Maturity Matrix

**Status:** Current release-surface source of truth  
**Application line:** 2.0.x remediation/modernization

The runtime registry in `apps/desktop/src/featureMaturity.ts` is the mechanically enforced UI source of truth. This document records the release rationale and ownership decision for every top-level product area.

| Feature | Status | Stable-build decision | Persistence / backend authority |
| --- | --- | --- | --- |
| Editor | stable | Visible | Asset and palette writes must commit to `RomSession` / `EditJournal` immediately. |
| Viewer | stable | Visible | Read-only projection of the current materialized revision. |
| Project | stable | Visible | Project format v2 stores the complete edit journal and source identity, never the base ROM bytes. |
| Test / embedded emulator | stable | Visible | Loads the exact materialized editor revision in memory and reports revision + SHA-1. |
| Settings | stable | Visible | Preferences/update settings only; no ROM-derived state persisted in browser storage. |
| Roster / creator | experimental | Hidden by default | Proven name/circuit/unlock/intro writes are journal-backed. Champion toggle and reset-to-defaults remain explicitly unsupported. |
| Compare | experimental | Hidden by default | Binary/palette/sprite comparison reads immutable base vs materialized current image. Visual renderer remains incomplete. |
| Scripts | experimental | Hidden by default | Developer/research tooling; not part of stable release claims. |
| Layout packs | experimental | Hidden by default | Import/list/install/delete are hardened. Apply is disabled until pack payload semantics are real. |
| AI behavior | experimental | Hidden by default | Hidden until every mutation is journal-backed and validated. |
| Bank / relocation tools | experimental | Hidden by default | Hidden until all relocation mutations use one canonical transaction path. |
| Animation player | experimental | Hidden by default | Inspection/playback only. |
| Audio | experimental | Hidden by default | Browse/import/export work is separated from research-blocked ROM sequence editing. |
| Text editor | experimental | Hidden by default | Hidden until all text mutation commands are journal-backed end-to-end. |
| Animation / frame / hitbox / hurtbox mutation | research-blocked | Not visible | Backend mutation helper returns an explicit unsupported error and cannot dirty ROM state. |
| Plugins | research-blocked | Not visible | Stable IPC execution is disabled. No plugin code is executed during startup/discovery. |

## Status definitions

- **stable** — supported in the normal production surface and covered by release gates.
- **experimental** — available only to an explicit development/experimental build and excluded from stable release claims.
- **research-blocked** — implementation depends on unproven ROM behavior or an incomplete security model; not exposed as a successful user action.
- **deprecated** — retained only for migration/compatibility and scheduled for removal.
- **removed** — intentionally absent.

## Experimental mode

The frontend only exposes `experimental` features when running a development build with `VITE_ENABLE_EXPERIMENTAL=true`. `research-blocked`, `deprecated`, and `removed` features do not become visible through that flag.

## Completion rule

A feature may move to `stable` only when its backend command surface is registered, its successful mutations are durable in the canonical edit journal, its failure behavior is explicit, its persistence/output paths agree on the materialized hash, and current automated or recorded manual evidence exists.
