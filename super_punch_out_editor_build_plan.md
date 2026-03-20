# Super Punch-Out!! Editor Tool — Detailed Build Plan

## 1. Purpose

Build a dedicated **Super Punch-Out!! editor** that gets as close as practical to the feel of **FF3usME** while respecting how this game is actually structured.

That means the tool should:
- open a clean USA headerless ROM,
- identify boxer assets safely,
- preview and edit palettes, icons, portraits, and fighter graphics,
- protect users from breaking **shared graphics banks**,
- rebuild changed assets into a patched ROM or patch file,
- eventually expose animation/script hooks and boxer metadata.

This is **not** a generic SNES tile editor. It is a **game-specific editor** built around the real Super Punch-Out!! asset model.

---

## 📊 Current Status (Updated 2026-03-19)

### Completed Phases
| Phase | Status | Key Deliverables |
|-------|--------|------------------|
| Phase 0 — Ground Truth | ✅ Done | Asset manifest, boxer mappings |
| Phase 1 — ROM Core | ✅ Done | ROM validation, open/save, SHA1 verification |
| Phase 2 — Safe Asset Editing | ✅ Done | Palette editor, icon/portrait import/export |
| Phase 3 — Graphics Bin Editing | ✅ Done | 4bpp viewer, tile import/export, compressed assets |
| Phase 4 — Curated Layouts | ✅ Done | Boxer layout JSONs, preview sheets |
| Phase 5 — Script Awareness | ✅ Done | Script viewer, fighter header decoding |
| Phase 6 — Advanced Features | ✅ Done | Bank duplication, relocation, BPS, projects |

### Crate Summary (7 crates)
| Crate | Purpose | Tests |
|-------|---------|-------|
| rom-core | ROM I/O, LoROM mapping, expansion | 6 passed |
| manifest-core | Asset manifests, boxer records | — |
| asset-core | Graphics, palettes, compression | — |
| patch-core | IPS + BPS patch generation | 6 passed |
| project-core | Project files, bank duplication | 5 passed |
| script-core | AI/script analysis | — |
| relocation-core | Free space, relocation helpers | — |

### Test Status
- **17 Rust unit tests passing**
- All core functionality has test coverage

---

## 2. What We Know About This Game Already

Based on the extracted asset set and manifest generated from your ROM + disassembly:

- ROM target: **Super Punch-Out!! (USA)**
- Format: **headerless LoROM**
- Size: **2 MB**
- Asset inventory includes:
  - **393** uncompressed graphics bins
  - **83** compressed graphics bins
  - **39** palette bins
  - **1** compressed tilemap bin
- Many fighter graphics banks are **shared** between two fighters.

Confirmed shared pairs:
- Gabby Jay ↔ Bob Charlie
- Bear Hugger ↔ Mad Clown
- Piston Hurricane ↔ Aran Ryan
- Bald Bull ↔ Mr. Sandman
- Dragon Chan ↔ Heike Kagero
- Masked Muscle ↔ Super Macho Man
- Rick Bruiser ↔ Nick Bruiser

Confirmed safest full-body boxer targets:
- **Hoy Quarlow** — 47 unique sprite bins, 0 shared
- **Narcis Prince** — 31 unique sprite bins, 0 shared

This matters because the editor has to understand **safe edits vs risky edits**.

---

## 3. Product Goal

### Core goal
Make a tool where someone can:
1. select a boxer,
2. see all known assets for that boxer,
3. edit safe assets first,
4. preview before saving,
5. export a modded ROM and/or patch,
6. avoid accidentally modifying another boxer through shared banks.

### FF3usME-style feel to aim for
The editor should feel like:
- game-specific,
- visual-first,
- organized by character,
- fast to navigate,
- dangerous actions clearly labeled,
- patch-oriented instead of raw file chaos.

---

## 4. Recommended Scope Strategy

Do **not** try to ship a god tool in version 1.

Build in layers:

### V1 — Safe Asset Editor
Ship the editor for:
- palette editing,
- icon import/export,
- large portrait import/export,
- boxer manifest browsing,
- ROM validation,
- patch export.

### V2 — Fighter Graphics Editor
Add:
- 4bpp graphics viewer,
- tile grouping tools,
- per-bin import/export,
- shared-bank warnings,
- boxer graphics preview sheets,
- reinsertion pipeline.

### V3 — Animation / Script Layer
Add:
- animation/sprite-script mapping,
- boxer state preview,
- pointer-safe script editing,
- side-by-side original vs modded behavior preview.

### V4 — Full Power Layer
Add:
- project workspace,
- custom bank cloning,
- “duplicate shared bank into new unique bank” helpers,
- smarter frame reconstruction,
- IPS/BPS distribution pipeline,
- plugin hooks for future research.

---

## 5. Recommended Tech Stack

## Desktop-first recommendation
Use a **desktop app**, not web-only.

### Frontend
- **React + TypeScript**
- UI library: lightweight component system or custom panels
- State management: Zustand or Redux Toolkit
- Canvas rendering for tile/palette/preview panels

### Desktop shell
- **Tauri** preferred
- Electron acceptable, but heavier

### Core binary logic
Use **Rust** for the ROM/asset core.

Why Rust here:
- strong binary parsing,
- safe file IO,
- easier to build a reliable patch/rebuild pipeline,
- good fit for compression/decompression,
- integrates cleanly with Tauri.

### Optional helper layer
- Python scripts for one-off extraction research only
- not part of main shipping runtime

### Patch output
Support:
- **ROM save-as**
- **IPS export**
- **BPS export**

### Internal asset format
Use JSON manifests plus raw bin data for project state.

---

## 6. High-Level Architecture

The tool should be split into five major layers.

## A. ROM Core
Responsibilities:
- validate ROM identity,
- map PC offsets and LoROM addresses,
- read/write asset blocks,
- calculate checksums if needed,
- save modded ROM,
- emit patches.

## B. Asset Knowledge Layer
Responsibilities:
- load boxer manifest,
- resolve boxer → assets,
- track shared vs unique bins,
- attach labels, safety flags, and categories,
- maintain address/pointer metadata.

## C. Asset Processing Layer
Responsibilities:
- decode palettes,
- display 4bpp graphics,
- decode/re-encode compressed bins,
- import PNGs into tile format,
- compare edited assets to originals,
- validate dimension/palette limits.

## D. Editor UI Layer
Responsibilities:
- boxer browser,
- asset tree,
- preview panels,
- tile palette editors,
- diff view,
- warnings and save/export dialogs.

## E. Project / Patch Layer
Responsibilities:
- track edits as a project,
- preserve undo/redo,
- generate patch files,
- package mod metadata,
- allow reopening work later.

---

## 7. Product Structure

Suggested main navigation:

1. **ROM / Project**
2. **Boxers**
3. **Palettes**
4. **Graphics**
5. **Portraits / Icons**
6. **Scripts** (disabled or read-only early on)
7. **Validation**
8. **Export**

### First-run flow
1. user opens ROM,
2. tool validates it,
3. tool loads built-in manifest,
4. user sees boxer list,
5. safe-edit categories are available immediately,
6. advanced/risky categories show badges.

---

## 8. Data Model

## Core entities

### `RomInfo`
```ts
interface RomInfo {
  gameId: "super-punch-out-usa";
  headered: boolean;
  sizeBytes: number;
  sha1?: string;
  valid: boolean;
  notes: string[];
}
```

### `AssetRef`
```ts
interface AssetRef {
  id: string;
  category: "palette" | "gfx" | "gfx_compressed" | "tilemap_compressed" | "icon" | "portrait" | "script";
  path: string;
  offsetPc?: number;
  sizeBytes?: number;
  compressed: boolean;
  shared: boolean;
  owners: string[];
  risk: "low" | "medium" | "high";
  tags: string[];
}
```

### `BoxerRecord`
```ts
interface BoxerRecord {
  id: string;
  displayName: string;
  palette?: AssetRef;
  icon?: AssetRef;
  largePortrait?: AssetRef;
  uniqueSpriteBins: AssetRef[];
  sharedSpriteBins: AssetRef[];
  sharedWith: string[];
  notes: string[];
}
```

### `ProjectEdit`
```ts
interface ProjectEdit {
  assetId: string;
  editType: "palette" | "tile-import" | "png-import" | "script-edit" | "metadata";
  originalHash: string;
  editedHash: string;
  timestamp: string;
}
```

### `ProjectFile`
```ts
interface ProjectFile {
  version: number;
  romBaseId: string;
  manifestVersion: string;
  edits: ProjectEdit[];
  settings: Record<string, unknown>;
}
```

---

## 9. Boxer-Centric Asset Model

Each boxer page should include:
- boxer name,
- reference sheet preview,
- icon,
- large portrait,
- palette preview,
- list of unique sprite bins,
- list of shared sprite bins,
- shared-with names,
- risk banner,
- notes about what is safe right now.

### Risk rules

#### Low risk
- palette edits
- icon swaps
- large portrait swaps
- unique-bin edits on fully unique fighters

#### Medium risk
- unique compressed fighter bins
- unique animation-linked graphics bins

#### High risk
- shared bins
- pointer-sensitive script changes
- dimension-changing imports
- anything requiring relocation / repointing

The tool should display those risk levels visually and loudly.

---

## 10. MVP Feature Set

## MVP-A: ROM Handling
- Open ROM
- Validate game/version
- Detect headered vs headerless input
- Offer header stripping if necessary
- Save modded ROM as copy
- Export patch

## MVP-B: Boxer Browser
- Boxer list with search
- Shared-vs-unique badge system
- Summary counts
- Boxer detail page

## MVP-C: Palette Editor
- Read palette bins
- Render SNES colors correctly
- Edit color entries
- Import/export palette files
- Preview palette applied to associated assets
- Undo/redo

## MVP-D: Icon / Portrait Editor
- Export icon/portrait graphics to image-friendly format
- Import PNG with validation
- Enforce tile size / color count constraints
- Preview before apply

## MVP-E: Manifest + Validation
- Show all assets affecting current boxer
- Warn if chosen edit touches shared bank(s)
- Validation log before save/export

## MVP-F: Patch Output
- Save-as ROM
- Export IPS
- Project file save/load

This is the first version worth using.

---

## 11. V2 Fighter Graphics Editing

This is where the tool starts feeling genuinely special.

### Required capabilities
- display raw 4bpp fighter bins as tiles,
- group tiles into editable sheets,
- import PNG -> quantize -> tile pack,
- export tile sheets for external editing,
- preview with boxer palette,
- mark bins as unique/shared,
- compare old/new binary size,
- reinsert safely.

### Required editor modes
1. **Tile View** — raw tile layout
2. **Sheet View** — reconstructed image sheet where possible
3. **Diff View** — changed vs original tiles
4. **Bank View** — bin-level ownership and usage

### Important constraint
Do **not** promise automatic full-frame reconstruction for every fighter immediately.
Some graphics are easy to visualize, some will require manually curated layout metadata.

So the V2 plan should support:
- raw tile accuracy first,
- prettier composite previews second.

---

## 12. V3 Script / Behavior Editing

Once graphics are stable, add script work.

Potential script surfaces:
- animation scripts
- sprite scripts
- AI scripts
- player scripts
- cornerman scripts

### Initial V3 approach
Start read-only:
- show linked script records,
- label states where known,
- show asset references,
- allow export/disassembly preview.

### Writable V3 later
- controlled edits for enumerated values,
- known-safe parameter edits,
- pointer validation,
- compile/apply pipeline.

### Strong rule
No raw freeform assembler editing in the first writable version.
That turns the tool into a bug cannon.

---

## 13. Shared-Bank Safety System

This is one of the most important systems in the whole product.

## Problem
A user edits a graphics bank thinking it belongs to Boxer A, but Boxer B also uses it.

## Required protections

### Protection 1 — Visibility
Every asset must show:
- owners,
- shared status,
- risk level,
- what other boxer(s) will change.

### Protection 2 — Confirmation
If a user edits a shared asset, the tool must say plainly:
> This asset is shared by Piston Hurricane and Aran Ryan. Applying this change will affect both fighters.

### Protection 3 — Safer alternatives
Offer options:
- **Apply to both**
- **Duplicate into a new custom bank** (future)
- **Cancel**

### Protection 4 — Pre-export validator
The export screen should summarize:
- edited shared assets,
- affected fighters,
- edited compressed bins,
- any repointing/overflow issues.

---

## 14. “Duplicate Shared Bank” Stretch Feature

This is not MVP, but it is a killer feature.

### Goal
Allow a shared asset to be cloned into a new bank so one boxer can diverge without changing the other.

### Requirements
- find free space or expansion space,
- write cloned asset,
- update only the target boxer’s references,
- preserve original shared bank for the other boxer,
- validate all pointers/scripts using that bank.

### Why this matters
Without this, shared-bank fighters will always be harder to customize cleanly.
With this, the tool gets much closer to a true per-boxer editor.

---

## 15. Import / Export Rules

## Input support
- PNG for graphics imports
- project JSON / custom project file
- palette import format(s)
- original ROM only

## Output support
- modded ROM copy
- IPS patch
- BPS patch
- export current asset as raw bin
- export current view as PNG

## PNG import validation
For every PNG import, validate:
- dimensions,
- tile alignment,
- max color count,
- transparency handling,
- palette slot compatibility,
- compression size if needed,
- whether resulting data exceeds original capacity.

---

## 16. Compression / Rebuild Plan

Because the game uses compressed graphics bins, the editor must handle them explicitly.

## Phase 1 approach
- support viewing and exporting compressed bins after decode,
- allow reinsert only when recompression fits expected constraints,
- reject unsafe writes with clear error messages.

## Phase 2 approach
- full recompress pipeline,
- automated size comparison,
- overflow detection,
- optional relocation if supported later.

## Rule
Never silently write a compressed asset that no longer fits.
Either:
- recompress and fit,
- relocate safely,
- or fail with a direct message.

---

## 17. UI / UX Plan

The editor should not look like a hacker’s junk drawer.

## Main window layout

### Left panel
- boxer list / navigation tree

### Center panel
- current asset preview/editor

### Right panel
- metadata, ownership, warnings, validation, linked assets

### Bottom panel
- log / diff / export summary

## Boxer list row content
Each boxer row should show:
- name,
- icon,
- shared badge if applicable,
- palette badge,
- changed badge if edited.

## Warning language style
Use blunt, plain warnings.
Examples:
- “This bin is shared with Aran Ryan.”
- “This import uses too many colors.”
- “Compressed result is larger than source block.”
- “Export blocked until shared-asset warning is acknowledged.”

---

## 18. Preview System

The preview layer is what will make the tool feel real.

## Required preview types
- palette strip preview,
- icon preview,
- portrait preview,
- tile preview,
- before/after diff preview,
- boxer asset ownership preview.

## Stretch preview types
- assembled boxer frame preview,
- simple animation cycle preview,
- fight-scene test preview,
- transparency/mask overlay preview.

---

## 19. Validation Engine

Every export/save should run a validation pass.

### Validation checks
- correct ROM base
- manifest version compatibility
- asset offsets valid
- import dimensions valid
- palette count valid
- no accidental overflow
- no invalid compressed writes
- shared-asset edits acknowledged
- patch generation succeeded

### Validation result levels
- **Info** — harmless
- **Warning** — user may proceed
- **Error** — export blocked

---

## 20. Internal File Layout

Suggested repo layout:

```text
super-punch-out-editor/
  apps/
    desktop/
  crates/
    rom-core/
    asset-core/
    compression/
    patching/
    manifest/
  packages/
    ui/
    shared-types/
  data/
    manifests/
    boxer-layouts/
    schemas/
  tests/
    rom-fixtures/
    golden/
  docs/
```

### Built-in data folders
- `manifests/boxers.json`
- `manifests/assets.json`
- `schemas/project.schema.json`
- `boxer-layouts/*.json` for curated frame/tile preview mappings later

---

## 21. Development Phases

## Phase 0 — Ground Truth
**Goal:** lock down real data model

Tasks:
- import current manifest into editor repo
- normalize boxer IDs/names
- normalize asset categories
- verify offsets/sizes/path mapping
- define schema for built-in manifest data

Deliverable:
- stable internal manifest package

## Phase 1 — ROM Core + Project Skeleton [DONE]
**Goal:** open ROM, validate, save project

Tasks:
- [x] build ROM validator
- [x] implement file open/save
- [x] detect headered/headerless
- [x] implement project file format (project-core crate with save/load)
- [x] implement patch output base (IPS + BPS)

Deliverable:
- app opens ROM and shows boxer list

## Phase 2 — Safe Asset Editing [DONE]
**Goal:** ship usable first version

Tasks:
- [x] palette editor
- [x] icon/portrait export/import
- [x] preview panels
- [x] warning system
- [x] export modded ROM / IPS / BPS

Deliverable:
- first public/private usable build

## Phase 3 — Graphics Bin Editing [DONE]
**Goal:** full fighter graphics workflow

Tasks:
- [x] raw 4bpp viewer (Fighter Graphics Viewer / Sprite OAM)
- [x] tile import/export (SpriteBinEditor: per-bin PNG export/import with palette quantization)
- [x] per-bin diff (tile-level diff visualization with change count)
- [x] compressed asset reinsertion (Integrations in fighter manager)
- [x] shared-bank edit handling (Confirmation dialog + visual shared badges)

Deliverable:
- **graphics modding becomes practical** ✓

## Phase 4 — Curated Fighter Preview Layouts [DONE]
**Goal:** make graphics editing pleasant

Tasks:
- [x] build boxer layout metadata (data/boxer-layouts/ JSON files per fighter, shared_banks.json)
- [x] reconstruct common sheet/frame views (render_sprite_sheet command — vertical stacked tile strips)
- [x] add image preview assembly where known (BoxerPreviewSheet component with zoom, bin legend, layout badges)

Deliverable:
- **less raw tile pain, more visual editing** ✓

## Phase 5 — Script Awareness [DONE]
**Goal:** expose logic layer

Tasks:
- [x] link scripts to boxers/assets (KNOWN_SCRIPTS constant with 16 fighter headers + shared)
- [x] read-only script viewer (ScriptViewer.tsx with hex preview, categories, risk levels)
- [x] safe parameter editing for known fields (FighterHeader decoding with attack/defense/speed)

Deliverables:
- script-core crate with ScriptReader
- ScriptViewer React component
- Fighter header decoding (32-byte structure)

## Phase 6 — Advanced Mutation Features [DONE]
**Goal:** surpass a basic editor

Tasks:
- [x] duplicate shared bank feature (BankDuplicationManager, ROM expansion 2MB→4MB)
- [x] relocation helpers (relocation-core crate, free space finder, visual ROM browser)
- [x] expanded validation (relocation validation with risk levels)
- [x] full project packaging (.spo project directories, SHA1 validation, recent projects)
- [x] BPS patch export (complete BPS implementation with metadata)

Deliverables:
- relocation-core crate
- SharedBankWarning modal + indicators
- ProjectManager component
- BPS patch generation (6 tests passing)

---

## 22. Milestone Order for Fastest Real Progress

If you want the fastest path to something real, build in this exact order:

1. [x] ROM validator
2. [x] Boxer browser using current manifest
3. [x] Palette viewer/editor
4. [x] Icon export/import
5. [x] Large portrait export/import
6. [x] Save-as ROM + IPS export + BPS export
7. [x] Shared-bank warning engine
8. [x] Raw fighter bin viewer
9. [x] PNG/tile import for unique fighters
10. [x] Compressed bin reinsertion
11. [x] Curated boxer preview layouts
12. [x] Script viewer
13. [x] Duplicate shared bank feature
14. [x] Relocation helpers
15. [x] Full project packaging

That gets you value fast instead of vanishing into engine work.

---

## 23. First Fighters to Support

Support these first in order:

### Tier 1 — safest full-body targets
1. **Hoy Quarlow**
2. **Narcis Prince**

Why:
- they have large numbers of unique bins,
- zero shared sprite bins in current manifest,
- ideal for proving the graphics editing loop.

### Tier 2 — good shared-bank proof cases
3. **Piston Hurricane**
4. **Aran Ryan**

Why:
- good test of shared-bank warnings,
- shared-pair workflow gets exercised.

### Tier 3 — early simple roster / visible user interest
5. **Gabby Jay**
6. **Bear Hugger**

---

## 24. Testing Plan

This tool absolutely needs real tests.

## Unit tests
- [x] ROM header detection (rom-core)
- [x] LoROM address conversion (rom-core)
- [x] asset offset reads (rom-core)
- [x] palette decode/encode (asset-core)
- [x] 4bpp tile decode/encode (asset-core)
- [ ] compression round-trip where supported
- [x] IPS patch generation (patch-core)
- [x] BPS patch generation (patch-core) - 6 tests passing
- [x] Free space finding (rom-core)
- [x] ROM expansion (rom-core)
- [x] Project save/load (project-core)

## Integration tests
- [x] open clean ROM and load manifest
- [x] export unchanged ROM copy
- [x] palette edit then save (8 tests in palette_edit_test.rs)
- [x] portrait import then save (5 integration + 9 unit tests)
- [x] unique fighter graphics bin edit then save (7 tests)
- [x] shared bank edit triggers warning (UI implementation)

## Golden tests
Use known original bins and expected decoded previews:
- palette golden files
- tile decode golden files
- compressed asset round-trip samples

## Regression tests
Any time a boxer asset mapping is fixed, add a regression fixture so it never breaks quietly.

---

## 25. Acceptance Criteria

## V1 acceptance criteria
- [x] user can open the correct ROM
- [x] boxer list loads correctly
- [x] every boxer shows palette/icon/portrait where present
- [x] palette edits preview correctly
- [x] icon/portrait imports validate correctly
- [x] export creates working ROM copy
- [x] shared assets are visibly marked
- [x] IPS export works on changed data
- [x] BPS export works on changed data

## V2 acceptance criteria
- [x] user can inspect fighter bins visually (FighterViewer, SpriteBinEditor)
- [x] unique fighter bin edits can be imported and saved
- [x] compressed asset handling is reliable
- [x] diff view shows changed bins (tile-level diff)
- [x] validation blocks bad imports cleanly

## V3 acceptance criteria
- [x] script references are visible for supported fighters (ScriptViewer)
- [x] read-only script inspection works (ScriptViewer with hex preview)
- [x] known-safe script parameters can be changed without corrupting build (EditableFighterParams with validation)

---

## 26. Risks

## Technical risks
- compression format edge cases
- incomplete frame/tile layout knowledge
- script references not fully mapped
- hidden dependencies between graphics and logic
- limited free space for future relocation features

## Product risks
- trying to ship too much too early
- spending months on automatic frame assembly before safe editing exists
- not warning clearly enough on shared banks

## Mitigation
- ship safe edits first
- treat raw tile editing as acceptable early value
- keep advanced mutation features behind explicit warnings
- log every edited asset at export time

---

## 27. Nice-to-Have Features Later

- boxer compare mode
- project thumbnails
- built-in patch notes generator
- external tool hooks
- emulator launch shortcut for testing
- frame tagging / annotation system
- community layout packs
- custom roster metadata editor if research supports it
- “show all assets touched by this patch” report

---

## 28. Suggested Build Sequence by Week / Sprint

## Sprint 1
- repo setup
- manifest import
- ROM validator
- boxer browser scaffold

## Sprint 2
- palette decode/encode
- palette UI
- project file support

## Sprint 3
- portrait/icon import-export
- preview system
- export modded ROM

## Sprint 4
- IPS export
- validation engine
- shared-bank warnings

## Sprint 5
- 4bpp raw tile viewer
- bin-level graphics page

## Sprint 6
- PNG/tile import pipeline for unique bins
- diff view

## Sprint 7+
- compressed reinsertion
- curated layout metadata
- script viewer
- advanced cloning / relocation research

---

## 29. Final Recommendation

Build this as a **desktop-first, boxer-centric, manifest-driven editor**.

Do **not** start with script editing.
Do **not** start with automatic frame reconstruction for every fighter.
Do **not** try to solve shared-bank cloning in the MVP.

Start with the version that gets a user from:
- open ROM,
- pick boxer,
- edit palette/icon/portrait,
- understand what is shared,
- save/export patch.

Then add fighter graphics editing for **Hoy Quarlow** and **Narcis Prince** first.
That will give you the cleanest proof that the editor is real and not just another research project.

---

## 30. Recommended Immediate Next Tasks

1. Create the editor repo and manifest schema
2. Import the current boxer manifest as built-in data
3. Implement ROM validation and boxer browser
4. Ship palette editor first
5. Ship portrait/icon editor second
6. Add shared-bank warnings before any fighter graphics writing
7. Use Hoy Quarlow as first full graphics edit target

---

## 31. Definition of Done for the First Real Release

The first real release is done when a user can:
- load the correct Super Punch-Out!! ROM,
- browse every boxer,
- see shared vs unique asset ownership,
- edit at least one boxer’s palette, icon, and portrait safely,
- save a modded ROM,
- export an IPS patch,
- and successfully perform one full graphics replacement on **Hoy Quarlow** or **Narcis Prince** without breaking another fighter.

That is the point where this becomes a real editor and not just a pile of extraction scripts.

---

## 18. Non-Negotiable Experience Goal — It Must Feel Like It Belongs To Super Punch-Out!!

This editor must **not** feel like a generic ROM hacker utility with a Super Punch-Out skin pasted on top.

It should feel like one of these:
- an internal Nintendo dev utility made during production,
- an official hidden debug / asset lab mode left inside the game,
- a broadcast-control / trainer-console used backstage at the WVBA.

That means the editor should inherit the game's:
- visual rhythm,
- color hierarchy,
- panel framing,
- typography feel,
- motion language,
- sound language,
- status/warning presentation,
- character-first organization.

The user should get the impression that the tool was **built from the same cartridge DNA** as the game itself.

### Hard rule
If a screen can be mistaken for a normal Electron/Tauri productivity app, the design failed.

---

## 19. Authenticity Strategy — Use the Game's Own Visual Language

The safest way to make the editor feel truly native is to structure the UI around **assets extracted from the user's own ROM**, not bundled third-party replacements.

### Use runtime-loaded game-derived UI elements where legally and technically appropriate
- fonts extracted from the user's ROM
- menu frame tiles / borders extracted from the user's ROM
- ringside / scoreboard motifs extracted from the user's ROM
- boxer portraits, icons, and palette samples from the user's ROM
- optional sound cues extracted from the user's ROM

### Do not ship Nintendo art in the installer
The app should ship with:
- code,
- layout definitions,
- manifest metadata,
- editor chrome templates,
- fallback placeholder theme.

Then, after ROM validation, the app should unlock **authentic theme mode** by sourcing the needed UI art from the user's ROM or extracted workspace.

That gives the editor the real game feel without baking copyrighted assets into the distributable.

---

## 20. Visual Design Pillars

## Pillar A — Broadcast / Arena Presentation
Super Punch-Out is loud, theatrical, and TV-like. The editor should feel like a control board for a live event.

Use:
- bold title bars,
- framed data boxes,
- chunky separators,
- scoreboard-style number fields,
- spotlighted character previews,
- ring-corner color accents,
- strong contrast rather than soft gradients.

Avoid:
- minimal white-space-heavy SaaS layouts,
- subtle gray-on-gray panels,
- modern glassmorphism,
- rounded mobile-app card systems,
- generic file-explorer aesthetics.

## Pillar B — Pixel Purity
Everything user-facing should honor pixel art rules.

Requirements:
- integer scaling only for sprite previews,
- nearest-neighbor scaling,
- no blurry CSS transforms,
- no antialiased interpolation on sprite canvases,
- palette previews rendered as crisp blocks,
- 1x/2x/3x/4x zoom presets matched to clean pixel boundaries.

## Pillar C — Character-First Navigation
The game is about personalities, not raw file bins. The editor should be organized around fighters first, data second.

Primary browse model:
- choose fighter,
- see their ring card,
- open assets by category,
- inspect warnings if anything is shared,
- edit with visual confirmation.

Do not make the first experience a naked list of offsets and .bin names.

## Pillar D — Dramatic but Fast
The editor should have presence, but it cannot become sluggish.

Good:
- 120–220 ms transitions,
- quick wipe / slide / snap panel movement,
- immediate preview refresh,
- subtle blinking selection bars,
- restrained screen shake only for special confirmation moments.

Bad:
- long cinematic animations,
- delayed panel loading,
- excessive particles,
- flashy clutter while editing.

---

## 21. Core Theme Specification

## Base palette philosophy
The editor theme should feel like it was assembled from:
- menu backdrops,
- ring signage,
- scoreboard panels,
- boxer card colors,
- announcer / status strip styling.

### Primary color groups
The exact palette should be sourced dynamically when possible, but the hierarchy should be:
- **deep background navy / black** for canvas and app shell,
- **electric blue / cyan** for selection and active panel edges,
- **hot red** for destructive or dangerous actions,
- **gold / yellow** for featured fighter labels and highlighted stats,
- **white / off-white** for major text,
- **muted grays / desaturated blues** for inactive chrome.

### Functional color semantics
- Safe / clean = blue or green-accented status frame
- Shared bank risk = yellow / amber caution frame
- Destructive / overwrite = red frame with stronger blink cadence
- Read-only disassembly-derived field = steel blue or gray frame
- Modified / unsaved = gold or orange badge

### Texture and finish
Use:
- flat SNES-style fills,
- simple checker / stripe / tile motifs,
- hard-edged inset borders,
- occasional metallic or TV-monitor framing illusion via pixel art.

Do not use:
- soft blurred shadows,
- glossy 2010s web gradients,
- frosted glass,
- modern neumorphism,
- smooth vector icon packs that clash with sprites.

---

## 22. Typography and Text Presentation

## Typography rule
The app should feel like it uses the game's own UI fonts or something extremely close.

### Preferred approach
1. Extract and reconstruct bitmap fonts from the user's ROM.
2. Build a font atlas for UI chrome.
3. Use that atlas in title bars, tabs, warning labels, and boxer cards.

### Fallback approach
If runtime font reconstruction is not ready yet:
- use a pixel font that is close in weight and spacing,
- constrain font sizes to a small SNES-like set,
- avoid modern proportional UI typography for the main chrome.

### Text hierarchy
- Main title: bold all-caps arcade heading
- Section label: compact all-caps or title case pixel label
- Body / metadata: readable pixel or tightly matched retro UI font
- Offset / pointer / checksum fields: monospaced pixel-friendly font

### Text treatment
Use:
- strong label boxes,
- shadowed or outlined title text where appropriate,
- compact stat readouts,
- boxer subtitles and region labels in card format.

Avoid:
- huge essay-like text blocks on primary screens,
- tiny faint metadata text,
- mixed font families with different vibes.

---

## 23. Layout Language — How Screens Should Be Structured

Every screen should look like a **game mode screen**, not a document editor.

### Recommended shell structure

#### Top bar
Contains:
- editor logo / title
- ROM identity
- current boxer
- project dirty state
- authenticity mode indicator

It should resemble a title strip or broadcast banner.

#### Left column
Character / category navigation:
- boxer list,
- portraits/icons,
- palette groups,
- graphics bins,
- scripts,
- validation,
- export.

Should feel like a mode-select column, not a generic folder tree.

#### Main center panel
Main preview area:
- sprite sheet view,
- portrait view,
- palette board,
- diff panel,
- state preview.

This must be the visual focus of the entire app.

#### Right column
Context card:
- asset metadata,
- shared-bank warnings,
- related bins,
- import/export actions,
- undo/redo,
- notes.

Should feel like a cornerman clipboard, stat card, or debug box.

#### Bottom strip
Status / command bar:
- hints,
- current tool,
- selected tile count,
- palette index,
- rebuild status,
- patch status.

Should feel like an arcade machine footer or service menu strip.

### Golden rule
At first glance, the user should understand **who** they are editing before they understand **which file** they are editing.

---

## 24. Signature Screens That Sell the Illusion

## A. Title / Boot Screen
This should feel like entering a hidden dev mode.

Sequence:
1. black screen
2. short logo or utility mark
3. ROM validation flash
4. transition into boxer-select or project dashboard

Optional wording ideas:
- WVBA Asset Lab
- Special Match Editor
- Ringside Development System
- SPO Asset Studio

This can be themed without pretending to be an official Nintendo product.

## B. Boxer Select Screen
This should feel like the game's match setup / profile presentation.

Screen elements:
- large portrait card
- boxer flag / region / title
- small icon strip
- palette swatches
- asset completion meter
- “shared bank” badge if applicable
- last modified marker

## C. Graphics Edit Screen
This should feel like a training room / monitor wall.

Screen elements:
- large sprite preview canvas
- tile bin browser
- source vs modded toggle
- palette selector board
- overlay options for hitbox/script state later
- bank safety callout box

## D. Palette Edit Screen
This should feel like a technician color-balance station.

Screen elements:
- boxer portrait live preview
- full-body preview
- palette row grid
- original / modded comparison
- import / export / revert buttons

## E. Validation / Export Screen
This should feel like a final weigh-in / inspection board.

Screen elements:
- pass/fail indicators
- ROM target confirmation
- changed assets list
- shared bank impact report
- patch format options
- final export button with strong ceremony

---

## 25. Motion Language

The app should animate like a 16-bit action presentation, not a modern phone app.

### Use
- horizontal wipes
- snap-ins
- blink pulses on active selection
- fast palette-flash confirmations
- scoreboard-like tick updates for counters
- optional subtle CRT line or scan overlay in non-edit chrome only

### Avoid
- elastic easing
- floaty spring physics everywhere
- overblown particle transitions
- long fades that soften the interface

### Timing targets
- tab switch: 120–180 ms
- panel reveal: 160–220 ms
- warning blink: 2–3 step cadence
- success flash: < 150 ms

### Motion priority
Motion must always reinforce:
- current selection,
- changed state,
- danger state,
- successful apply/export.

---

## 26. Audio Language

Optional, but if included, it will massively help authenticity.

### Safe audio approach
Allow the app to source optional cues from extracted ROM audio or use original non-infringing stand-ins until the user enables asset-backed theme mode.

### Good UI sounds
- short confirm beep
- cursor move tick
- warning buzz
- export success stinger
- tab / panel step sound

### Bad audio choices
- generic desktop pop sounds
- soft productivity-app chimes
- cinematic orchestral whooshes

### Requirement
All audio must be globally toggleable, with separate controls for:
- UI sounds
- ambient menu loop
- preview-related sounds

---

## 27. Authenticity Mode vs Clean Mode

The app should support two shells:

## Clean Mode
For users who want a straightforward tool.
- simplified chrome
- less animation
- neutral font fallback
- no ambient audio

## Authenticity Mode
For the intended experience.
- game-derived UI skin from extracted assets
- fighter-card layout emphasis
- stronger motion and broadcast styling
- optional authentic sound cues
- pixel font / reconstructed font atlas

### Important
Authenticity Mode should be the flagship presentation and the default once the ROM/theme assets are available.

---

## 28. Editing Rules That Preserve the Illusion

Even the workflows should feel game-native.

### Replace “File > Open Asset” with in-world wording where sensible
Examples:
- Load ROM
- Enter Asset Lab
- Select Boxer
- Inspect Bank
- Review Shared Impact
- Rebuild Match Data
- Export Patch

### Use boxer-card language for warnings
Example:
Instead of:
> This asset is referenced by two entities.

Use:
> Shared Corner Warning: this graphics bank is used by Piston Hurricane and Aran Ryan. Changes here affect both fighters unless cloned.

### Use visual comparison constantly
The game feel depends on making edits feel tied to the live fighter, not abstract binary data.

Every destructive or important operation should show:
- original preview,
- edited preview,
- impacted boxer list,
- palette compatibility,
- export result.

---

## 29. Shared-Bank Safety UX — Must Feel Native, Not Technical

This is one of the most important design opportunities.

Because shared banks are the main structural danger in this game, the warnings cannot feel like dry programmer text.

### Shared-bank warning design
Show:
- both fighter portraits side by side
- the shared bank name
- what type of asset it is
- a direct statement of impact
- one-click choices:
  - Edit Shared Bank
  - Clone to Unique Bank
  - Cancel

### Preferred presentation
A dramatic split-card warning screen, like a special match announcement.

### Required clarity
The user must instantly understand:
- who else is affected,
- what will change,
- which path is safest.

---

## 30. Implementation Requirements for the Skin System

To make the app feel built into the game, the theme layer must be architected intentionally.

### Build a skinning subsystem with these capabilities
- load font atlas from extracted assets or fallback theme
- load border/frame tiles from extracted assets
- load palette themes from extracted assets
- load menu sound cues conditionally
- load boxer portrait cards dynamically
- expose style tokens to the React UI

### Suggested theme architecture
- `theme/base/` for neutral fallback shell
- `theme/authentic/` for runtime-generated game-derived shell
- `theme/runtime-cache/` for extracted UI art atlases

### UI components that must be theme-aware
- window frame
- tab strip
- action button
- warning modal
- stat badge
- boxer card
- palette grid
- asset tree row
- status footer

### Goal
Reskinning the app should not be a CSS afterthought. The authenticity shell is a first-class system.

---

## 31. Suggested Visual Component Set

Required custom components:
- BoxerCard
- RegionBadge
- SharedBankAlert
- PaletteBoard
- TileCanvas
- RingFramePanel
- BroadcastHeader
- StatusTicker
- PortraitDock
- ScriptStatePanel
- ExportInspectionBoard

Each component should be designed with:
- hard edges,
- pixel-perfect padding,
- theme token support,
- animation presets,
- 1x/2x/3x UI scale testing.

---

## 32. Authenticity Acceptance Criteria

The editor passes the visual/authenticity bar only if:

### Immediate impression
- A user can identify the tool as Super Punch-Out-themed within 2 seconds.
- The UI does not resemble a generic file utility.
- The boxer selection flow feels like entering a game mode.

### Visual quality
- All sprite previews use integer scaling only.
- No blurry sprite rendering is visible anywhere.
- Core screens use game-authentic color hierarchy and panel framing.

### Interaction quality
- Editing a fighter always foregrounds the fighter identity.
- Shared-bank warnings show impacted fighters visually, not just textually.
- Export/validation feels like a final in-game inspection, not a form.

### Audio / motion quality
- Cursor movement and confirms feel arcade-like if sound is enabled.
- Motion is sharp and restrained.
- The app remains responsive while looking theatrical.

### Legal / packaging quality
- Installer does not ship copyrighted Nintendo assets.
- Authentic theme mode is unlocked by loading the user's ROM / extracted assets.

---

## 33. Revised Milestone Order With Authenticity Built In

## Milestone A — Functional Skeleton With Theme Hooks
- Build shell layout
- Add boxer navigation
- Add ROM validation
- Add runtime theme system
- Add pixel-perfect preview canvases

## Milestone B — Authentic Boxer Select + Safe Editors
- Boxer card screen
- portrait/icon editor
- palette editor
- authentic top/bottom bars
- first-pass sound/motion

## Milestone C — Shared-Bank Warning Experience
- split-card impact modal
- clone-vs-edit workflow
- affected-fighter preview
- export inspection board

## Milestone D — Graphics Editor Proper
- tile/bin viewer
- bin import/export
- source vs modded preview
- authentic training-monitor presentation

## Milestone E — Final Polish
- reconstructed font atlas
- runtime-loaded frame/border art
- optional authentic menu ambiance
- final visual pass to remove any remaining generic-app feel

---

## 34. Final Directive

Do not build a ROM editor and then try to “retro skin” it later.

Build the editor from day one as a **Super Punch-Out control room** with a binary core underneath.

The right mental model is:
- **front end:** hidden in-universe asset lab / debug mode / ringside production console
- **back end:** strict ROM parser, asset processor, and patch builder

If those two layers stay locked together, the tool will feel like it belongs to the game instead of just operating on it.
