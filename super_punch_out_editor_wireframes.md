# Super Punch-Out!! Editor — Screen-by-Screen Wireframe Pack

## Purpose
This document defines the screen-by-screen wireframe, interaction model, visual hierarchy, motion language, and state behavior for a **Super Punch-Out!! editor tool that feels like it shipped as part of the original game’s internal toolchain**.

This is not a generic desktop utility spec. The product goal is a **native-feeling WVBA-style debug/editor suite** that presents ROM editing tasks through the visual, audio, and structural language of *Super Punch-Out!!*.

---

# 1. Experience Pillars

## 1.1 Core feeling
The tool must feel like:
- an unlocked internal dev mode
- a hidden test menu on original hardware
- a ROM lab operated by the game’s own production team
- a broadcast-ready fight prep system, not a spreadsheet app

## 1.2 What to avoid
Do **not** let the tool feel like:
- Electron dashboard sludge
- an IDE with boxing textures pasted on top
- a mod manager launcher
- a flat modern CRUD application
- a generic sprite editor with tabs

## 1.3 Guiding presentation rules
- Boxer-first, not file-first
- Arena/broadcast framing, not desktop window framing
- Big bold labels, compact metadata
- Immediate visual previews
- Navigation must feel snappy and arcade-like
- Warnings must feel like match official notices or corner-team alerts
- Export must feel like “official bout package generation”

---

# 2. Global UI Language

## 2.1 Visual language
Use the visual logic of the game:
- bold contrast blocks
- strong framing rectangles
- segmented panels
- punchy palette accents
- hard-edged borders
- large sprite and portrait previews
- scoreboard-style information grouping

## 2.2 Core screen regions
Most screens should use the same macro layout:

1. **Top Marquee Bar**
   - editor mode name
   - active ROM profile
   - active boxer / asset context
   - unsaved changes indicator

2. **Center Action Stage**
   - primary editor content
   - sprite preview / palette grid / portrait canvas / comparison panel

3. **Left Navigation Rail**
   - mode select
   - boxer select
   - section jumps

4. **Right Status / Inspector Rail**
   - asset metadata
   - palette usage
   - shared-bank impact
   - validation state

5. **Bottom Command Strip**
   - button legends
   - hotkeys
   - commit/apply/export hints

## 2.3 Resolution logic
Primary design baseline:
- 16:9 modern window, but visually framed like a 4:3 broadcast-safe SNES presentation

Two supported views:
- **Arcade Frame Mode**: screen centered in a faux in-game frame with thick borders and decorative status bars
- **Workbench Mode**: same visual language, but slightly expanded for practical editing

## 2.4 Pixel rules
- Integer scaling only for ROM-derived graphics
- No blurry filtering by default
- Nearest-neighbor preview for authenticity mode
- Optional analysis zoom in secondary panels only
- UI chrome may be high-resolution, but must mimic original shape language

---

# 3. Global Interaction Model

## 3.1 Input support
Support:
- keyboard
- mouse
- controller

Controller is not a joke feature. It should work convincingly enough that the editor feels like a dev cartridge menu.

## 3.2 Primary navigation mapping
Default keyboard/controller conventions:
- Arrow Keys / D-Pad: move focus
- Enter / A: confirm
- Escape / B: back
- Q / L: previous boxer / tab
- E / R: next boxer / tab
- Tab: cycle pane focus
- Shift: alternate function modifier
- Space: toggle compare/preview playback
- F5: refresh extracted asset preview
- F6: validate current context
- F9: build patch
- Ctrl+S: save project state
- Ctrl+Z / Ctrl+Y: undo / redo

## 3.3 Focus philosophy
The UI should always clearly show what has focus:
- glowing border
- pulse frame
- marquee arrow
- small animated corner bracket

Do not rely on subtle modern hover effects.

## 3.4 Audio response
Every major UI action should optionally trigger game-authentic UI sound classes:
- cursor move
- confirm
- cancel
- warning
- export success
- failed validation
- tab switch

Sound should be configurable, but **on by default** in game-authentic mode.

---

# 4. Screen Map

Primary screens:
1. Boot / Title Screen
2. ROM Load / Profile Select
3. Main Hub / Boxer Select
4. Boxer Overview
5. Palette Lab
6. Sprite Bank Browser
7. Sprite Frame Viewer
8. Portrait & Icon Studio
9. Shared Bank Impact Screen
10. Script Link Inspector
11. Validation / Diagnostics Screen
12. Patch Build / Export Screen
13. Settings / Authenticity Options
14. Recovery / Conflict Resolution Screen

Optional later screens:
15. Animation Timeline Viewer
16. AI / Behavior Editor
17. Asset Diff Theater
18. Tournament Preview / Attract Mode Test

---

# 5. Screen Specifications

## 5.1 Boot / Title Screen

### Intent
Sell the fantasy immediately: this is a hidden official editor suite.

### Layout
- Full-screen title composition
- Large logo treatment derived from extracted ROM style language
- Subtitle beneath: `EDITOR MODE`, `DEV TOOL`, or `WVBA ASSET LAB`
- Lower third panel with three options:
  - Load ROM Profile
  - Continue Last Project
  - Settings
- Tiny build string in bottom-right

### Motion
- subtle marquee shimmer
- occasional score-light blink
- slow pulse on selected menu item
- optional faux attract-mode transition after inactivity

### Audio
- short title sting
- menu movement ticks
- confirm thump

### Key states
- No profile loaded
- Last project available
- Missing extracted theme assets fallback

### Acceptance criteria
- User instantly understands this is a game-native tool
- Nothing on the title screen looks like default OS UI

---

## 5.2 ROM Load / Profile Select

### Intent
Bridge real tooling requirements with the fantasy shell.

### Layout
Left panel:
- ROM profile slots
- each slot shown as a boxer-card or match-card style panel
- fields: ROM name, region, extraction status, theme status

Center panel:
- ROM identity preview
- detected metadata
- status badge: Clean / Modified / Unknown / Unsupported

Right panel:
- action list
  - Load Profile
  - Create New Profile
  - Re-extract Assets
  - Import Theme From ROM

Bottom strip:
- supported ROM guidance
- checksum mismatch warnings

### Interaction details
- Loading a ROM should feel like selecting an event or save slot
- Extraction progress should present like a pre-fight system check, not a raw console log

### State variants
- Clean supported ROM
- ROM modified but still loadable
- checksum mismatch
- missing extracted cache
- failed extraction

### Acceptance criteria
- Technical ROM checks are visible without breaking immersion
- Error handling remains readable and not theatrical to the point of confusion

---

## 5.3 Main Hub / Boxer Select

### Intent
This is the home screen of the editor. It must feel like a roster selection + production control room.

### Layout
Top marquee:
- current ROM
- project name
- modification state

Left column:
- circuit filters
  - Minor Circuit
  - Major Circuit
  - World Circuit
  - Special Circuit
  - Player / UI / Global

Center stage:
- large boxer cards laid out like a selectable roster wall
- selected boxer shown in big portrait card
- card includes name, circuit, edit status, risk class

Right inspector:
- summary of boxer assets
  - sprite bins
  - portraits
  - icons
  - palettes
  - shared banks count
  - scripts linked

Bottom command strip:
- Open Boxer
- Preview Assets
- View Shared Impact
- Validate
- Back

### Motion
- roster card snap transitions
- selected card bounce/flash inspired by menu selection energy
- optional crowd-light sweep across active card

### Risk badges
Every boxer card should show:
- Safe
- Mixed
- Shared
- Advanced

### Acceptance criteria
- A modder can understand who is safest to edit within seconds
- The screen remains hype, readable, and fast

---

## 5.4 Boxer Overview

### Intent
One-stop command screen for a selected boxer.

### Layout
Large left portrait area:
- boxer portrait
- optional alt portrait / icon strip below

Center info board:
- boxer metadata
- linked assets summary
- change history
- quick stats on unique vs shared content

Right action menu:
- Edit Palette
- Browse Sprite Banks
- Open Portrait Studio
- Inspect Shared Assets
- View Script Links
- Run Validation

Bottom preview ribbon:
- thumbnails of major assets
- icon, portrait, ring marker, selected sprite frame samples

### Key information blocks
- **Unique Assets**
- **Shared Assets**
- **Known Risks**
- **Recommended Safe Edits**

### Acceptance criteria
- User understands what they can safely change before touching any editor sub-screen

---

## 5.5 Palette Lab

### Intent
Fast, tactile, confidence-building color editing with strong authenticity.

### Layout
Left panel:
- palette list for selected boxer
- row-per-subpalette with usage label

Center panel:
- large color chip grid
- editable palette slots
- SNES color value display
- RGB breakdown

Right panel:
- live preview targets
  - portrait preview
  - icon preview
  - sprite sample preview
- “usage heat” or palette occupancy hints

Bottom strip:
- Copy Palette
- Paste Palette
- Revert Slot
- Import Indexed PNG Palette
- Apply to Preview
- Save

### Interaction rules
- palette selection should be instant
- live preview must update immediately
- out-of-range or invalid edits must be prevented, not merely warned later

### Game-authentic touches
- chips framed like status lamps or scoreboard bulbs
- selection cursor behaves like in-game menu selector
- warning state flashes like a referee/official notice

### State variants
- clean palette
- modified unsaved palette
- illegal import mapping
- preview mismatch

### Acceptance criteria
- This should be the first feature shipped and it must already feel premium
- User can make safe boxer recolors with minimal fear

---

## 5.6 Sprite Bank Browser

### Intent
Expose graphics bins without overwhelming the user.

### Layout
Left rail:
- asset groups
  - Unique Sprite Bins
  - Shared Sprite Bins
  - Uncompressed
  - Compressed
  - Related Global Assets

Center panel:
- tile/bin thumbnail grid
- each block shown as a card
- cards display asset name, compression type, ownership class, size

Right inspector:
- selected bin metadata
  - source pointer
  - size
  - extracted file path
  - linked fighters
  - validation status

Bottom strip:
- Open Viewer
- Mark Favorite
- Compare Original
- Show Linked Boxers
- Export Raw

### Critical UX rule
Shared bins must never look visually equivalent to unique bins.

Use persistent visual distinctions:
- shared = split-color banner or chain-link icon
- unique = solid owner-color banner
- dangerous = warning trim / hazard stripe

### Acceptance criteria
- A novice can spot the difference between safe and risky asset bins without reading documentation

---

## 5.7 Sprite Frame Viewer

### Intent
This is the close-up lab. It must let the user inspect and edit frame assets while still staying inside the game’s presentation language.

### Layout
Main center stage:
- large pixel-perfect sprite view
- optional transparency checker or matte backdrop
- zoom levels: 1x, 2x, 4x, 8x, integer only

Left sidebar:
- frame/bin list
- tags such as idle, punch, hit, knockdown, unknown
- quick navigation among linked assets

Right sidebar:
- metadata and tools
  - palette selector
  - original vs modified toggle
  - raw tile metrics
  - linked boxers
  - import/export buttons

Bottom strip:
- Prev Frame
- Next Frame
- Compare
- Onion Skin
- Tile Grid
- Save Variant

### Editing modes
Mode A: View only
Mode B: Raw graphics import/export
Mode C: lightweight pixel touch-up mode
Mode D: external-editor roundtrip mode

### Core design warning
Do not try to make the first version a full Aseprite clone.
This screen should prioritize:
- inspection
- import pipeline confidence
- bank ownership awareness
- compare workflow

### Acceptance criteria
- User can verify whether a modified frame actually reads correctly in-context
- Shared-bin impact is always visible before commit

---

## 5.8 Portrait & Icon Studio

### Intent
Fast wins. This screen should make portrait and icon swaps feel satisfying and low-risk.

### Layout
Left panel:
- asset type selector
  - Small Icon
  - Large Portrait
  - Banner / Label if available

Center panel:
- large canvas preview
- crop frame overlay if import mode is active

Right panel:
- import settings
  - indexed mode
  - palette target
  - quantization preview
  - dimension checks

Bottom strip:
- Import PNG
- Extract Current
- Restore Original
- Compare
- Apply

### Suggested polish
- portrait presentation framed like tale-of-the-tape or boxer spotlight card
- icon row feels like pre-fight HUD asset selection

### Acceptance criteria
- First-time users should be able to complete a successful portrait or icon swap here with almost no documentation

---

## 5.9 Shared Bank Impact Screen

### Intent
This is one of the most important screens in the whole editor. It prevents bad edits and teaches the asset structure.

### Layout
Top warning banner:
- `SHARED ASSET DETECTED`
- clear severity label

Center split panel:
Left side:
- current boxer
- currently edited asset
- proposed changes summary

Right side:
- all linked boxers impacted
- preview thumbnails for each linked boxer
- expected effect classification

Lower detail panel:
- recommended options
  - Continue and affect all linked boxers
  - Duplicate and repoint if supported
  - Cancel and return
  - Export report

### Presentation tone
This should feel like an official ringside warning or production alert, not an OS pop-up.

### Motion
- subtle alert pulse
- hazard trim shimmer
- no cheesy flashing that hurts readability

### Acceptance criteria
- The user should never accidentally alter a shared bin without being explicitly shown downstream effects

---

## 5.10 Script Link Inspector

### Intent
Show relationships without forcing all users into raw assembly or script tables.

### Layout
Left tree:
- linked systems
  - animation scripts
  - sprite scripts
  - AI scripts
  - player/cornerman/global links

Center graph panel:
- node-link view or structured dependency list
- selected node highlighted

Right detail panel:
- script name
- address
- summary
- boxer usage
- open raw source option

Bottom strip:
- Open Source
- Copy Address
- Trace Usage
- Jump to Asset

### Core rule
This is an inspector first, editor second, at least in early versions.

### Acceptance criteria
- Users can understand why a sprite or palette exists in multiple contexts
- Debug-oriented users can reach technical details without overwhelming casual modders

---

## 5.11 Validation / Diagnostics Screen

### Intent
A full-screen confidence dashboard before export.

### Layout
Top summary bar:
- overall project health meter
- counts for pass, warning, fail

Left column:
- validation categories
  - asset integrity
  - palette legality
  - pointer safety
  - shared-bank impact
  - script linkage
  - export readiness

Center panel:
- results list in fight-card style rows
- each row clickable for drill-down

Right panel:
- selected issue detail
- fix suggestion
- jump action

Bottom strip:
- Re-run Validation
- Ignore Warning
- Export Report
- Jump to Issue

### Game-authentic presentation
This screen should feel like an official pre-match inspection board.

### Acceptance criteria
- Validation must be readable to beginners and still precise enough for advanced users

---

## 5.12 Patch Build / Export Screen

### Intent
Make export feel ceremonial and trustworthy.

### Layout
Top panel:
- selected ROM/profile
- patch target type
- project name

Center panel:
- export checklist
- generated artifacts list
- notes field
- preview of changed boxer cards/assets

Right panel:
- build output summary
  - BPS/IPS package
  - manifest
  - backup snapshot
  - validation report

Bottom strip:
- Build Patch
- Build Full Package
- Save Project Only
- Cancel

### Build flow language
Preferred verbs:
- Build Package
- Generate Bout Patch
- Finalize Mod Set

Avoid boring labels like `Run` or `Submit`.

### Success state
- celebratory but not gaudy
- subtle victory-card vibe
- clear file locations and next steps

### Acceptance criteria
- Export must feel reliable enough that users trust the output before testing it in an emulator

---

## 5.13 Settings / Authenticity Options

### Intent
Let advanced users tune practicality vs immersion without collapsing the identity of the tool.

### Layout
Sections:
- Theme Source
- Audio Feedback
- Input Mapping
- Workspace Density
- Authenticity Level
- External Tools
- Safety Prompts

### Authenticity presets
- Broadcast Authentic
- Dev Cart Authentic
- Hybrid Workbench
- Minimal Utility

### Acceptance criteria
- Users can tone down effects if needed, but the default remains game-authentic

---

## 5.14 Recovery / Conflict Resolution Screen

### Intent
Handle broken imports, conflicting edits, and invalid repoints without panicking the user.

### Layout
Left panel:
- issue list

Center panel:
- selected conflict explanation
- before/after details

Right panel:
- resolution options
  - keep original
  - keep modified
  - duplicate branch
  - revert session changes

### Tone
Firm, clear, sober. This is one of the few screens that should reduce theatrics.

### Acceptance criteria
- Recovery flows are safer than forcing users into manual file surgery

---

# 6. Overlay / Modal Specifications

## 6.1 Confirm Apply Modal
Use for:
- destructive imports
- shared-bank writes
- bulk palette applications

Must include:
- what changes
- who is affected
- whether undo is available

## 6.2 Quick Compare Overlay
Triggered by holding a key/button.
- original vs modified flipbook style compare
- no page navigation interruption

## 6.3 Toast / Notification Style
Use short, punchy messages:
- `PALETTE UPDATED`
- `VALIDATION PASSED`
- `SHARED BIN WARNING`
- `PATCH BUILT`

Do not use generic system-toast language.

---

# 7. Navigation Flow Recommendations

## 7.1 New user happy path
1. Boot screen
2. Load ROM profile
3. Main hub
4. Select boxer
5. Boxer overview
6. Portrait/Icon studio or Palette lab
7. Validation
8. Export

## 7.2 Advanced user path
1. Boot
2. Continue project
3. Main hub
4. Boxer overview
5. Sprite bank browser
6. Frame viewer
7. Shared impact screen
8. Validation
9. Export

## 7.3 Script/debug path
1. Main hub
2. Boxer overview
3. Script link inspector
4. Validation
5. Export report only

---

# 8. Visual Identity Notes by Screen

## 8.1 Title and Hub screens
- highest theatrical energy
- strongest marquee visuals
- big character presentation

## 8.2 Editing screens
- slightly calmer
- clearer utility focus
- still framed with game-native elements

## 8.3 Warning/validation screens
- broadcast-official tone
- caution colors and structured rows
- reduced decorative motion

## 8.4 Export/success screens
- victory-card tone
- strong completion feedback
- concise artifact summary

---

# 9. Motion Language

## 9.1 Allowed motion
- panel slides
- focus pulses
- card snap transitions
- marquee sweeps
- subtle numeric tick-ups
- sprite preview cycling

## 9.2 Disallowed motion
- floaty modern easing everywhere
- material-design bounce nonsense
- big modal zoom explosions
- overactive particles

## 9.3 Timing guidance
- navigation response: immediate
- panel transition: short and punchy
- warnings: pulse only, no spam

---

# 10. Sound Language

## 10.1 Sound classes
- menu tick
- heavy confirm thump
- cancel blip
- warning buzz
- success stinger
- export completion sting

## 10.2 Rules
- never overwhelm repetitive navigation
- allow per-screen mute
- allow global mute
- keep sound responsive and arcade-tight

---

# 11. Accessibility and Practicality Rules

Even in full authenticity mode:
- text must remain legible on modern displays
- warning colors must not be the only signal
- keyboard-only use must be fully supported
- controller-only use must be practical
- motion reduction option required
- audio-off option required

The tool can feel native **without** becoming unusable.

---

# 12. Screen-by-Screen Acceptance Checklist

## Title Screen
- Feels like a hidden game mode
- No visible default OS chrome in primary composition

## ROM Load
- Technical requirements clear
- Extraction flow understandable

## Main Hub
- Roster selection is fast
- Risk level visible at a glance

## Boxer Overview
- Safe edits obvious
- Risky edits clearly labeled

## Palette Lab
- Live preview works instantly
- Illegal values prevented

## Sprite Bank Browser
- Shared bins unmistakable
- Metadata easy to inspect

## Sprite Frame Viewer
- Comparison fast
- Ownership context always visible

## Portrait/Icon Studio
- First successful swap easy
- Import validation readable

## Shared Bank Impact
- User cannot miss downstream effects
- Cancel path obvious

## Script Link Inspector
- Relationship model readable
- Technical detail available on demand

## Validation Screen
- Actionable next steps shown
- Warning vs failure distinction clear

## Export Screen
- Output trusted
- Artifacts clearly listed

---

# 13. MVP Screen Order

Ship in this order:
1. Boot / Title Screen
2. ROM Load / Profile Select
3. Main Hub / Boxer Select
4. Boxer Overview
5. Palette Lab
6. Portrait & Icon Studio
7. Validation / Diagnostics
8. Patch Build / Export

Then add:
9. Sprite Bank Browser
10. Sprite Frame Viewer
11. Shared Bank Impact Screen
12. Script Link Inspector

This order gets a usable, authentic-feeling editor in people’s hands sooner while preserving the long-term architecture.

---

# 14. Implementation Notes for the UI Team

## 14.1 Recommended stack behavior
Whatever framework is used, the UI layer should support:
- integer-scaled sprite rendering
- controller focus management
- layered animation states
- theme asset loading from extracted ROM resources
- custom window chrome or borderless mode

## 14.2 Theme system
The authentic shell should be driven by theme assets extracted from the user’s ROM/profile:
- font samples where legal and practical
- border patterns
- palette ramps
- iconography shapes
- menu accent colors

## 14.3 Data bindings
Every screen should bind cleanly to core domain objects:
- `RomProfile`
- `BoxerManifest`
- `AssetBank`
- `PaletteAsset`
- `ValidationIssue`
- `ExportPackage`

---

# 15. Final Creative Direction Summary

If this editor is successful, a user should say:

> “This feels like Super Punch-Out had a secret built-in editor the whole time.”

Not:
- “nice web app”
- “good mod manager”
- “cool sprite utility”

That is the bar.

The product wins when utility and fantasy meet in the same place:
- technically trustworthy
- visually native
- fast to use
- respectful of shared-bank complexity
- fun enough that opening the editor feels like entering the game

