# Design System and Originality Rules

**Status:** Current. Applies to every screen in `apps/desktop`.

The editor aims for the feel of a first-party console product: bold, friendly, tactile and instantly readable by a child. It gets there with its own design, not by borrowing anyone else's artwork.

## Originality rules (copyright and trademark)

These rules are release gates, not style preferences.

1. **No game or console-maker assets in the repository or the installer.** No ripped sprites, sprite sheets, logos, fonts, sounds, box art or screenshots of the game. A ripped menu and font sheet that earlier builds bundled was removed for this reason.
2. **No imitation of protected marks.** The interface does not reproduce the game's logo lettering, the publisher's logo, or console logos. The product name is shown in the editor's own typeface.
3. **Game pictures appear only at runtime, from the user's own ROM.** Boxer faces, poses and palettes are decoded in memory from the file the user opened. They are never cached into the repository, the installer or a project file.
4. **Every bundled picture is original.** The app icon, the boxing-glove mark, the ring illustration and all menu icons were drawn for this project (`src/components/icons.tsx`, `WelcomeWorkspace.tsx`, `src-tauri/icons/app-icon.svg`).
5. **Every bundled font is openly licensed.** See [`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md).
6. **The product says what it is.** The welcome screen and sidebar state that this is an unofficial, fan-made tool that is not affiliated with or endorsed by Nintendo, and that the user must supply their own legally obtained game.

Colours, rounded shapes, a red accent and a four-colour action palette are common visual vocabulary and are not protected on their own. If a future change would add third-party artwork, a logo, or a typeface designed to imitate a protected mark, it needs legal review first.

This document is engineering guidance, not legal advice.

## Tokens

All colour, shape and type decisions live in `apps/desktop/src/styles/tokens.css`. Components must use tokens (`var(--bg-panel)`, `var(--accent)`, `var(--radius-md)`, …) rather than hard-coded values, so both themes stay correct.

| Group | Tokens | Use |
| --- | --- | --- |
| Surfaces | `--bg-primary`, `--bg-secondary`, `--bg-tertiary`, `--bg-panel`, `--bg-raised`, `--bg-input`, `--canvas-bg` | Page, sidebar, cards, inputs, picture backdrops |
| Text | `--text-primary`, `--text-secondary`, `--text-muted`, `--text-on-accent` | Body, supporting and hint text |
| Accent | `--accent`, `--accent-hover`, `--accent-depth`, `--accent-text` | Primary actions and the current selection |
| Status | `--success`, `--warning`, `--error`, `--info` and their `-bg` pairs | Readable status text and tinted panels |
| Action colours | `--pad-red`, `--pad-yellow`, `--pad-green`, `--pad-blue` | Menu badges and step markers |
| Shape | `--radius-xs` … `--radius-lg`, `--radius-pill` | Corners |
| Type | `--font-display`, `--font-body`, `--font-pixel`, `--font-mono` | Headings, text, small labels, code |

Themes are chosen with `data-theme="dark"` or `data-theme="light"` on `<html>` (set by `ThemeProvider`). The selected boxer contributes one accent, `--boxer-accent`, read from the user's ROM at runtime; it never replaces the interface colours.

Older components reference legacy names such as `--panel-bg`, `--text-dim` and `--blue`. `tokens.css` aliases every one of them to a current token.

## Buttons

- A button with no class and no inline background is a **primary** action: red, raised, and it presses down.
- `.secondary` (or `.btn-secondary`) is a neutral raised button.
- `.quiet-button` is an outlined, low-emphasis button.
- Any other button gets readable neutral colours by default, so text never disappears in either theme.

## Writing for the interface

The target reader is ten years old.

- Use everyday words: "Play Game", "Save My Game", "Use My Picture". Keep internal terms (journal, materialized, SHA-1, offset) out of the main workflow; they belong in Expert Tools and tooltips.
- One obvious next step per screen.
- Say what happens and that it is safe: "Your original game file is never touched."
- Error messages say what went wrong and what to try next.

## Helpful automation

The editor does the fiddly parts itself. Each of these has backend tests.

| Behaviour | Where |
| --- | --- |
| A recognised ROM opens immediately, with no confirmation step | `App.tsx` `beginOpenRom` |
| A ROM file can be dropped onto the window | `App.tsx` drag-and-drop listener |
| The last ROM reopens on launch | `App.tsx`, `spo-editor-last-rom-path` in local storage |
| Changes are kept automatically and restored for the same ROM | `commands/project.rs` `autosave_session`, `restore_autosave` |
| Imported pictures are fitted to the right size and matched to the palette | `commands/assets` `fit_image_to_canvas`, `import_graphic_asset_from_png` |
| A photo placed on a pose is written into the boxer's tiles, simplified if needed to fit | `commands/photo_stamp.rs` `stamp_photo_on_pose`, `utils/photoStamp.ts` |
| One-tap recolours are a single Undo step | `commands/assets/palettes.rs` `apply_palette_colors`, `utils/colorMagic.ts` |
| Colours snap to what the console can display | `utils/colorMagic.ts` `snapToConsole` |
| An installed emulator is found for Play Game; the user confirms it once | `commands/emulator.rs` `find_installed_emulators` |

## UI preview mode (development only)

`npm run dev` followed by `http://localhost:1420/?preview` shows the interface in a normal browser with made-up sample data (`src/dev/previewBackend.ts`). It exists so screens can be designed and checked without a ROM. It is loaded only in development builds and is not part of a packaged app. The sample data is synthetic and contains no game content.
