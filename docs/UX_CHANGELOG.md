# Usability Milestone Summary

This milestone simplifies the Windows editing experience and prepares shareable community tester builds.

Key changes:

- guided first-run workspace with one obvious ROM-opening action;
- stable post-ROM landing that cannot enter a hidden experimental tab;
- short main workflow with advanced tools visually secondary;
- clearer ROM/session/edit state and Undo/Redo controls;
- improved keyboard/accessibility semantics for boxer selection and navigation;
- in-app persistent Tester Checklist with privacy-safe Markdown export;
- community test guide and GitHub issue template;
- automated Windows community tester kit with checksum/provenance metadata;
- fail-closed tester-kit scan for ROM, SRAM/save-state, and SuperZSNES content;
- documentation identifies Windows x64 as the active release target. The original milestone used `master`; the September 9, 2026 consolidation supersedes that branch policy with `main` (see `BRANCH_CONSOLIDATION_2026-09-09.md`).
- complete assembled boxer-pose preview with pose navigation;
- raw tile-bank view relabeled as an individual-8×8-tile editing reference so chopped bank sheets are not mistaken for missing sprites;
- user, architecture, troubleshooting, and Windows acceptance guidance added for the two preview modes.

## Look, ease of use and automation (October 2026)

- New original visual design in light and dark themes; see [`DESIGN_SYSTEM.md`](DESIGN_SYSTEM.md).
- Removed a bundled sheet of ripped game artwork and two screenshots that showed it. The app icon and all bundled pictures are now original.
- Plain-language menu: Edit Boxers, Look Around, Play Game, My Projects.
- Boxer page reorganised into Colors, Photo Face, Pictures, Save & Share and Expert Tools, with the boxer picker at the top.
- One-tap recolouring (Magic Paint, Special Looks) with a live preview; each is one Undo step.
- Photo Face: place a photo on a boxer's in-ring picture; it is written into the boxer's graphics in that boxer's colors.
- Any picture can replace a boxer's small face; it is fitted and colour-matched automatically.
- Recognised ROMs open without a confirmation step; ROM files can be dropped on the window; the last ROM reopens on launch.
- Changes are kept automatically and restored for the same ROM.
- Play Game finds an installed emulator and launches the current edited game with one button.
- Look Around was rebuilt (it was previously unstyled) and can flip through poses.

Fixes found along the way:

- Small boxer faces were drawn with two rows of tiles swapped. Drawing, saving-as-file and importing now use the game's real layout.
- The graphics packer was about 20% less efficient than the game's own data, so writing edited body graphics back always failed for lack of room. It now matches the original size exactly for unmodified data.
- Emulator and external-tool settings were loaded at startup and then discarded, so they never survived a restart.
- Reloading the window lost the open ROM even though the backend still held it.
- Several buttons had invisible text in the light theme.
