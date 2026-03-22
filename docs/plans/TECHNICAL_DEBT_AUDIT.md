# Technical Debt Audit
_Generated 2026-03-21 · Updated 2026-03-21 (session 7 re-audit)_

---

## First: That summary file

`ANIMATION_EDITOR_implementation_summary.md` is not real documentation. It is corrupted/hallucinated AI output — broken Rust syntax, invalid identifiers (`FIGHTer_COUNT`, `POse_table_ptr_offset`), malformed byte arrays, recursive `Display` impls that call `self`, and code fragments that would never compile. It describes work that **was not actually done**. The file should be deleted.

---

## Critical (feature-breaking stubs)

| # | File | Issue |
|---|---|---|
| 1 | `commands/plugins.rs` ~170 | `list_batch_jobs()` returns empty Vec — "not yet implemented" |
| 2 | `commands/plugins.rs` ~176 | `create_batch_job()` always returns Err — "not yet fully implemented" |
| 3 | `commands/plugins.rs` ~188 | `cancel_batch_job()` always returns Err |

Batch jobs appear in the UI but the entire system is non-functional.

---

## High

**Debug output left in production:**
- `emulator_embedded.rs` — ~15 `println!()` calls scattered throughout the emulator loop. These should be behind a `log::debug!` gate or removed entirely.
- `emulator_embedded.rs` ~858 — `eprintln!()` for audio errors

**Panic-prone `.unwrap()` on ROM state:**
- `roster_commands.rs` — 6 instances of `rom_guard.as_ref().unwrap()` scattered through distinct commands. If any roster command fires before a ROM is loaded (possible on startup race), it panics instead of returning a proper error.
- `text_commands.rs` ~348, ~512 — same pattern

**Weak TypeScript typing:**
- `ComparisonTable.tsx` — 15+ `as any` casts on diff objects
- `ComparisonCanvas.tsx`, `ComparisonView.tsx` — additional `as any` casts
These suppress type errors rather than modeling the data correctly.

---

## Medium

**Duplicate ROM-loading boilerplate:**
`let loader = RosterLoader::new(rom_guard.as_ref().unwrap());` is copy-pasted in at least 6 places in `roster_commands.rs`. Should be a single `with_loader(state, |loader| { ... })` helper — which would also fix the unwrap issue above.

**`#[allow(dead_code)]` attributes:**
- `roster_commands.rs` — 3 struct fields on request types marked dead
- `undo.rs` — ~14 enum variants/fields (`PaletteReplace`, `Relocation`, `source_path`, `color_index`) marked dead — suggests undo branches that were stubbed but never wired up
- `update_commands.rs` — 1 function marked dead

**Unused `_state` parameters:**
- `commands/plugins.rs` — `run_script_file`, `run_script`, and batch functions all accept `_state: State<AppState>` but ignore it entirely

**Example code in production tree:**
- `config/settings-integration-example.tsx` — 265 lines of demo code with `console.log` throughout, sitting in the production component tree

**`lib.rs` ~194:** Commented-out help system commands with a TODO — partial feature disable left in place

---

## Low

**`pub mod` re-export stubs** — `commands/help.rs`, `commands/text.rs`, `commands/tools.rs` are pure re-export shims. Fine organizationally, just worth knowing they exist.

**`lib.rs` ~76:** `.unwrap_or_default()` on config load silently swallows corrupted settings files.

---

## Unverified command surface (maintenance risk)

The backend registers 100+ Tauri commands. The audit found ~50 with no corresponding `invoke()` call in the frontend, including: `add_cornerman_text`, `analyze_fragmentation`, `apply_ai_preset`, `apply_in_game_expansion`, `compare_ai_behavior`, `create_new_spc`, `decode_brr_to_pcm`, `execute_defrag_plan`, `extract_all_rom_audio`, `find_free_regions`, and many more. Some may be intentionally backend-only or reserved for future work, but the list is large enough to warrant a deliberate prune pass.

---

## Priority order

1. ~~Delete `ANIMATION_EDITOR_implementation_summary.md` — misleading garbage~~ ✅ Done 2026-03-21
2. ~~Implement batch job system (all 3 functions)~~ ✅ Done 2026-03-21
3. ~~Replace `roster_commands.rs` / `text_commands.rs` unwraps with safe `ok_or`~~ ✅ Done 2026-03-21
4. ~~Strip `println!` / `eprintln!` from `emulator_embedded.rs`~~ ✅ Done 2026-03-21
5. ~~Audit unregistered commands; wire undo/redo, fix emulatorStore naming~~ ✅ Done 2026-03-21
6. ~~Fix `as any` casts in `ComparisonTable.tsx`, `ComparisonCanvas.tsx`, `ComparisonView.tsx`~~ ✅ Done 2026-03-21
7. ~~Remove `config/settings-integration-example.tsx` from production tree~~ ✅ Done 2026-03-21
8. ~~Fix `rom-core` animation crate (broken `types.rs`, `constants.rs`, `loader.rs`, `writer.rs`)~~ ✅ Done 2026-03-21
9. ~~Rewrite `commands/animation.rs` (invalid syntax throughout, duplicate function names, broken locks)~~ ✅ Done 2026-03-21

10. ~~Fix audio stubs: `get_sample` now returns honest error; `get_fighter_header` now calls `decode_boxer_header`~~ ✅ Done 2026-03-21
11. ~~Remove 3 dead request structs from `roster_commands.rs`~~ ✅ Done 2026-03-21
12. ~~Remove dead `UpdateState::new()` from `update_commands.rs`~~ ✅ Done 2026-03-21
13. ~~Clean `undo.rs`: removed `PaletteReplace`/`Relocation`/`BatchEdit` variants + dead fields/helpers; collapsed `history.rs` match~~ ✅ Done 2026-03-21

---

## Remaining

**Research-blocked stubs (return honest errors — no action needed):**

| File | Function | Note |
| --- | --- | --- |
| `audio_commands.rs` | `update_music_sequence` | Returns Err — SPC700 sequence editing unresearched |
| `audio_commands.rs` | `export_music_as_wav` | Returns Err — SPC700 render unresearched |
| `audio_commands.rs` | `scan_rom_for_audio` | Returns JSON with `scanned: false` — audio engine unresearched |
| `audio_commands.rs` | `extract_all_rom_audio` | Returns Err — requires audio research |

**Low — `lib.rs` ~194:** Commented-out help system commands; deliberate disable, clean up when help system is implemented.
