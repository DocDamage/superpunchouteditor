# UI/UX Acceptance — Windows Tester Milestone

This milestone treats usability as a release-quality requirement rather than cosmetic polish.

## Primary user journey

A first-time Windows tester should be able to complete this sequence without developer knowledge:

1. Launch the editor.
2. Understand that they must supply their own ROM.
3. Open and validate a supported ROM.
4. Choose a boxer or editing target.
5. Make one small reversible edit.
6. Undo and redo the edit.
7. Understand how to test the current revision.
8. Save/export without overwriting the source ROM unintentionally.
9. Save/reopen a project.
10. Find the Tester Checklist and produce a report.

## UI gates

- The first screen has one obvious primary action: open a ROM.
- Stable builds do not land users in hidden or experimental features after ROM load.
- Stable workflow destinations are presented before advanced/developer tools.
- Advanced tools are collapsed by default and retain an explicit disclosure.
- Buttons and interactive boxer rows are keyboard-focusable native controls.
- Undo/Redo state remains visible in the main navigation area after a ROM is loaded.
- The UI communicates the current ROM-loaded state and a shortened ROM fingerprint without exposing a local ROM path.
- The stable boxer editor presents an **Assembled Pose Preview** that shows a complete in-game pose, while the separate **Raw Tile Banks** view is labeled as an individual-tile reference.
- A raw tile bank is not treated as a complete character sheet; chopped or out-of-order pieces in that reference view are explained by nearby UI copy or documentation.
- The Test Game screen states that it consumes the current materialized revision.
- First-run and tester copy explicitly warn against uploading ROMs/save states.
- A tester can produce a structured report from inside the app without attaching copyrighted game data.
- Essential controls meet a minimum practical pointer target of roughly 42 px where the new guided shell controls them.

## Community-test gates

- A dedicated Windows tester-kit workflow runs on PRs and pushes targeting `master`.
- The kit includes a start-here guide, exact installer checksum, source/build provenance, and advanced evidence helpers.
- The kit scan fails if ROM, SRAM/save-state, or `SUPERZSNES.exe` content enters the artifact.
- The tester kit is explicitly identified as unsigned/non-production unless built by the separate production signing workflow.
- GitHub provides a dedicated community test-report issue template.

## Evidence requested from testers

Testers are asked to rate ease of use from 1–5 and report any point where they did not know what to click next. A 5 means the core journey felt extremely intuitive without requiring external instructions.

Usability feedback is actionable even when the underlying function technically succeeds. Requiring a tester to understand ROM-editor architecture, canonical materialization terminology, CI, or developer tooling to complete the basic workflow is considered a UX defect.
