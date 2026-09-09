# Creative Corner usability milestone — September 9, 2026

Based on authoritative `main` commit `071aab1e2e8ef6278d1ff4196d69ef6e1455e0bc`.

## User-facing changes

The stable editor now opens on a character-selection workspace rather than an asset-summary wall. Search by boxer name, choose a card, and work in four tool areas: Colors, Sprites, Assets, and Export. Tool areas mount on first use and remain mounted while switching between them, preserving local component drafts. Changing boxer or leaving the entire editor still follows the existing component/session lifecycle; this is not a new autosave system.

A sticky action bar puts Undo, Redo, project management, and Test Game next to the workspace. Technical asset counts and patch notes are secondary disclosures. The welcome screen explains the difference between an editable project and a playable ROM export. Navigation disables ROM-only destinations before a ROM is opened and retains visible Experimental labels.

The original console-inspired styling uses chunky cards, four colored tool markers, selected-state outlines and text, stronger readable secondary text, and restrained press feedback. Existing theme choices and runtime-derived boxer portraits remain. No Nintendo logos, new third-party artwork, external fonts, sound assets, or ROM data were added. Reduced-motion and forced-color preferences have explicit styles.

## Correctness repairs

- App shortcuts leave text controls, editable content, dialogs, handled events, IME composition, repeated key events, and the embedded game alone. Uppercase Ctrl+Shift+Z works, along with Ctrl+Y and Command-based history shortcuts.
- Test Game tracks changed-range and history projection references rather than only the number of changed ranges. Repeated edits to one range trigger a new canonical image read. Old asynchronous replies are discarded. A stale image is withheld during loading or failure, and failures have an explicit Retry action.
- Selecting a candidate ROM no longer changes the active test path or active region. Cancel leaves the active selection intact. Only a successful open commits the new path. Failed opens remain in the confirmation UI with a visible explanation.
- Switching away from a session with edits requires confirmation. This is deliberately conservative: it does not infer that every pending range is unsaved.
- ROM confirmation uses a native HTML modal dialog with Escape cancellation, focus restoration, and disabled controls during loading. Boxer selection and file dialogs are guarded against overlapping requests.
- Panel display errors offer a local retry, and handled sidebar errors can be dismissed.

## Preserved boundaries

No backend commands, edit representations, persistent schemas, dependencies, feature maturity classifications, production signing, updater trust, or release packaging requirements changed. All mutations still flow through the existing store and canonical backend journal. The bundled-emulator/core requirements are unchanged. The CI browser-preview artifact is for reviewing the web UI only; it is not an installer and does not prove desktop/runtime acceptance.

## Automated coverage

Added tests cover shortcut ownership and uppercase redo; same-range image refresh and out-of-order reads; stale-image withholding and retry; candidate ROM cancellation and failed opens; edit-discard confirmation; duplicate file-picker prevention; character search and selection states; lazy-mounted tabs and draft retention; sidebar prerequisites and Experimental labels; and modal semantics/focus/cancellation.

Local validation in the authoring environment is syntax transpilation only. Full TypeScript checking, Vitest, production frontend build, Rust checks, command contracts, dependency audits, and repository hygiene are executed by the existing PR CI. Refer to the PR checks for the exact tested commit and results. Do not interpret this document as a claim that Windows real-ROM, installed updater, signed production, or complete accessibility acceptance has passed.

## Windows acceptance checklist

1. Open a supported local ROM. Confirm its region, choose a boxer, and search/clear the boxer list.
2. Edit one color twice at the same address. Open Test Game after each edit and confirm the second revision is used. Repeat after Undo and Redo.
3. Enter text in a project field and use Ctrl+Z: only that text should undo. With workspace focus, test Ctrl+Z, Ctrl+Y, and Ctrl+Shift+Z.
4. Open Sprites or Assets, change a local field, switch internal tool tabs and return; the field should retain its value. Verify canvas layout after revisiting Sprites.
5. Start Switch ROM with existing edits and choose Keep editing. Then choose another file but cancel region confirmation. Verify the original active ROM and region remain selected.
6. Exercise a failed ROM open and a failed current-image read. Verify visible error/retry and no old game image substitution.
7. Navigate tools using Tab, arrows, Home/End, and Enter/Space. Open ROM confirmation, cancel with Escape, and verify focus returns to its opener.
8. Review dark, light, and authentic themes at 1440×900, 1024×768, and Windows 125%/150% scaling. Check reduced-motion settings. Confirm no clipped primary actions or unreadable controls.
9. Save/reopen a project and export to a NEW ROM path. Verify exported bytes through the existing canonical artifact acceptance workflow before sharing a release.
