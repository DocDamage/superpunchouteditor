# Roster Metadata Editor and In-ROM Creator Status

Last updated: 2026-03-21 (session 3)

## Overview

The roster editor is no longer a placeholder-only surface. It now supports real ROM-backed roster editing, expanded roster layouts, in-ROM creator hook installation, embedded emulator inspection, and a draft-to-ROM creator workflow for live metadata testing.

This document reflects the current implementation status, not the original proposal state.

## Current Capability Summary

### Completed and working

1. Real ROM-backed roster editing for boxer names, circuits, unlock order, and intro text
2. Expanded roster table support beyond the stock 16-boxer layout
3. In-ROM creator hook installation and WRAM contract publishing
4. Embedded emulator integration using the real Snes9x libretro core
5. Live creator runtime inspection from WRAM
6. Creator session launch from the Character Create flow into the embedded emulator
7. Creator draft editing and commit back into the loaded ROM for:
   - boxer name
   - circuit
   - unlock order
   - intro quote
8. Live edited-ROM reload into the embedded emulator after creator commits

### Still incomplete

1. Portrait reassignment is not yet a true creator-side write path
2. The ROM-side SNES menu is still a runtime scaffold, not a fully self-contained standalone editor UI
3. The creator hook publishes menu state and accepts actions, but desktop-side draft/commit logic still owns the actual metadata writes

## Major Progress Made

## 1. Roster data is real and layout-aware

The backend roster loader and writer now read and write the active roster layout from the ROM, including expanded layouts written by the in-game expansion path.

Key completed work:

1. Boxer ID serialization was normalized so expanded entries can be addressed consistently by the frontend
2. Expanded roster load/write tests were added
3. The boxer-name writer bug was fixed so names terminate correctly and no longer lose the last character or reload with trailing padding
4. The frontend roster editor now works against the real expanded roster contract instead of assuming the original 16-boxer tables

Relevant code:

1. `crates/rom-core/src/roster/types.rs`
2. `crates/rom-core/src/roster/writer.rs`
3. `crates/rom-core/src/roster/mod.rs`
4. `apps/desktop/src/types/roster.ts`
5. `apps/desktop/src/components/RosterEditor.tsx`
6. `apps/desktop/src/components/BoxerNameEditor.tsx`
7. `apps/desktop/src/components/CircuitEditor.tsx`

## 2. Character creation expands the roster and seeds a test session

The Character Create tab now does more than collect inputs. It can expand the roster, patch the in-ROM creator hook, create the new boxer slot, and hand that session off to the embedded emulator.

Completed flow:

1. Expand the roster to add a new boxer slot
2. Patch the in-ROM creator hook
3. Write initial metadata for the created boxer
4. Store the created boxer session context
5. Open the Test tab and auto-enter creator mode

The session handoff currently carries:

1. boxer slot
2. boxer name
3. circuit
4. unlock order
5. intro text id

Relevant code:

1. `apps/desktop/src/components/RosterEditor.tsx`
2. `apps/desktop/src/App.tsx`

## 3. In-ROM creator hook and WRAM contract are installed

The ROM expansion path now installs a creator bootstrap header plus a 65816 stub that:

1. enters creator mode on `Select + Start + L + R`
2. mirrors controller state into WRAM
3. tracks creator page, cursor, dirty flag, and action latch
4. publishes a render contract into WRAM
5. exposes exit and commit-style action latches

This work established the ROM-side contract consumed by the emulator monitor and desktop creator workflow.

Important limitation:

The hook still does not perform direct ROM metadata edits by itself. It is currently a control and render-state scaffold.

Relevant code:

1. `crates/expansion-core/src/ingame_editor.rs`
2. `crates/expansion-core/src/roster_expansion.rs`

## 4. Embedded emulator uses the real libretro runtime

The embedded emulator no longer relies on a pure simulation path during normal runtime use. It now loads the real Snes9x libretro core and reads creator WRAM state from `RETRO_MEMORY_SYSTEM_RAM`.

Completed work:

1. dynamic libretro loading
2. video/audio/input callback wiring
3. frame execution through `retro_run`
4. creator-state reads from system RAM
5. Tauri exposure of creator runtime state

The old stub path remains only for tests.

Relevant code:

1. `crates/emulator-core/src/libretro_runtime.rs`
2. `crates/emulator-core/src/snes9x.rs`
3. `apps/desktop/src-tauri/src/emulator_embedded.rs`
4. `apps/desktop/src/hooks/useEmulator.ts`

## 5. Creator runtime monitor and menu preview are in place

The embedded emulator UI now exposes the creator runtime as a usable debugging and test surface.

Completed monitor features:

1. creator active/idle status
2. page, cursor, action, dirty, revision, and render visibility
3. higher-level page and row preview instead of only raw bytes
4. quick controls for:
   - enter creator
   - previous/next page
   - cursor up/down
   - select action
   - exit creator
5. creator session metadata card

Relevant code:

1. `apps/desktop/src/components/EmbeddedEmulator.tsx`

## 6. The embedded emulator now boots the live edited ROM image

The Test tab previously depended on file-path loading, which meant the embedded emulator could drift from the actual edited ROM state held in memory by the app.

That gap is now closed.

Completed work:

1. backend command to export the current loaded ROM image with pending writes applied
2. Test tab wiring to feed that live ROM image into the embedded emulator
3. creator commit flow that reloads the updated edited ROM immediately

Relevant code:

1. `apps/desktop/src-tauri/src/commands/rom.rs`
2. `apps/desktop/src-tauri/src/lib.rs`
3. `apps/desktop/src/App.tsx`

## 7. Creator draft editing and commit are functional

The creator monitor is now backed by a desktop-side draft editor. The ROM-side creator hook still provides page, cursor, and action intent, but the app now translates those signals into a real editable draft and can commit the result back into the loaded ROM.

Completed draft behavior:

1. load the target boxer slot from the current ROM
2. load intro quote text for that boxer
3. focus draft fields based on creator page/cursor/action
4. update circuit selection from the creator circuit page
5. commit current draft values back into the ROM
6. reload the edited ROM into the emulator
7. re-enter creator mode automatically after commit

Committed fields:

1. boxer name
2. circuit
3. unlock order
4. intro quote

Relevant code:

1. `apps/desktop/src/components/EmbeddedEmulator.tsx`

## Verification Completed

The following checks have been run successfully during this implementation sequence:

```powershell
$env:CARGO_TARGET_DIR='target_codex'; cargo test -p rom-core roster -- --nocapture
$env:CARGO_TARGET_DIR='target_codex'; cargo test -p expansion-core -- --nocapture
$env:CARGO_TARGET_DIR='target_codex'; cargo test -p emulator-core -- --nocapture
$env:CARGO_TARGET_DIR='target_codex'; cargo check -p tauri-appsuper-punch-out-editor --lib
npx tsc --noEmit
npm run build
```

## Remaining Gaps

### Portrait workflow

Portrait selection is still not an end-to-end creator write path. The current state is:

1. the creator runtime exposes a portrait page
2. the embedded creator UI explains the portrait workflow
3. actual portrait asset replacement still lives in the asset pipeline, not in the creator draft commit path

### Fully standalone ROM-side editor logic

The ROM patch currently provides:

1. entry combo
2. action/page/cursor state
3. render rows
4. dirty/revision/visibility contract

What it does not yet provide:

1. direct in-ROM text entry UI
2. direct in-ROM commit-to-ROM logic
3. full standalone portrait management

## 8. Text editor is now ROM-backed for victory quotes and boxer intros

The Text editor tab is now exposed in the app navigation and the following
categories write real data to the ROM:

1. **Victory quotes** — `update_victory_quote` reads the existing quote from ROM
   to find its offset, encodes the new text, and writes it back in-place.
   New text must not exceed the original allocated byte length.

2. **Boxer intros** — `update_boxer_intro` uses `RosterWriter::write_boxer_intro_field`
   to write each field that was changed. All 5 fixed-size intro fields (name,
   origin, record, rank, intro_quote) are 16 bytes each in the ROM.

3. **Cornerman text** — Loaded from ROM via `load_cornerman_texts`. Writes
   currently use the in-memory TextDatabase pending a `write_cornerman_text`
   implementation in RosterWriter.

The menus tab and credits tab have been removed from the text editor surface.
Menu text ROM offsets require research; credits editing was out of scope.

### API changes from this session

- `get_victory_quotes(boxer_key: String)` — was `fighter_id: u8`
- `get_boxer_intro(boxer_key: String)` — was `fighter_id: u8`
- `get_cornerman_texts(boxer_key: String)` — was `fighter_id: u8`
- `BoxerIntroResponse` now includes a `validation` field with per-field
  byte lengths, validity flags, and unsupported character lists.

### Relevant code

1. `apps/desktop/src-tauri/src/text_commands.rs` — victory quote and intro write-back
2. `apps/desktop/src-tauri/src/roster_commands.rs` — API fixes, intro validation
3. `apps/desktop/src/components/TextEditor.tsx` — tab removed, menus/credits gone
4. `apps/desktop/src/App.tsx` — "Text" tab added to navigation

## Practical Status

If the goal is "create a new boxer slot, test it in the embedded emulator, edit core roster metadata from the creator flow, and immediately verify the result," that path is implemented.

If the goal is "the ROM alone contains a complete standalone creator UI that performs all edits without desktop assistance," that is not finished yet.
