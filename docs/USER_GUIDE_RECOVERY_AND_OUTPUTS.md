# User Guide — Projects, Recovery, ROM Save, Patches and Testing

## Before editing

Keep your original ROM separately. The editor treats the loaded source as an immutable base and records supported changes in an edit journal. Stable Save As, patch export, project persistence, comparison and embedded-emulator testing are designed to reference that same current journal materialization.

## View a complete boxer sprite

When a boxer is selected, the editor shows **Assembled Pose Preview** above **Raw Tile Banks**. Use the pose selector or **Prev/Next** buttons to inspect complete in-game poses.

Raw Tile Banks are intentionally different: they show individual 8×8 graphics tiles in ROM/bank order for editing and inspection. Because the game places those tiles into object VRAM and composes them with pose/OAM data, a raw bank can look chopped or out of sequence. That does not mean the ROM is missing a sprite. See [`SPRITE_PREVIEW.md`](SPRITE_PREVIEW.md) for the renderer details and troubleshooting steps.

## Projects

Use project format v2 for current work. A project stores source-ROM identity and your edit journal; it does **not** contain the base ROM itself. When reopening a project, load/select the matching base ROM. If the base hash/size does not match, the project is rejected before the active session is replaced.

Legacy v1 projects may contain edit descriptions without replacement bytes. If those edits cannot be reconstructed, the editor reports that limitation rather than claiming restoration succeeded.

## Save As

Prefer a new destination instead of overwriting your source ROM. The backend validates/materializes the current revision, writes a temporary file in the destination filesystem, flushes and reopens it for verification, and preserves a backup before overwriting an existing destination.

The output preflight API reports:

- source and current SHA-1;
- detected region;
- current revision;
- logical transaction count;
- changed-byte count and ranges;
- destination path/overwrite state;
- backup behavior and validation warnings.

## IPS and BPS

Patch export compares the immutable base ROM to the exact materialized current image. The generated patch is applied back to the base in memory before it is written; export fails if the result does not reproduce the current image.

IPS export is intentionally rejected when source and target ROM lengths differ. Use BPS for expansion-capable changes.

## Undo / redo

Undo and redo operate on logical journal transactions, not on a separate frontend history. A new edit after undo discards the redo branch. Selective removal of an arbitrary middle edit is intentionally unsupported because it can invalidate later before-bytes; use Undo or start from a known saved project state.

## Play Game

**Play Game** writes the current edited game (every change, saved or not) to a temporary file and opens it in your emulator, so what you play always matches what the editor shows. The first time, the editor looks for an emulator in the usual download, desktop, documents and program folders by file name only, and asks you to confirm the one to use. You can also pick the program yourself.

**Play inside the editor** is an advanced option that needs a separate Snes9x libretro core. It loads the current edited game in memory.

## Automatic safekeeping

Shortly after every change, Undo or Redo, the editor keeps your edit journal in its own data folder (`super-punch-out-editor/autosave/<ROM SHA-1>` under the local app-data directory). Like a project, this copy never contains the ROM itself. The next time the same ROM is opened, the changes come back and are still individual Undo steps. The copy is tied to the exact ROM by its SHA-1 and is never applied to a different file; a damaged copy is rejected and the session is left untouched. **Start Over** on the Save & Share tab undoes every change.

The editor also remembers the path of the last ROM you opened, on this computer only, so it can reopen it at startup.

## Recovery

Project-v2 writes preserve the previous valid manifest as a recovery file while the new manifest is verified. If a project manifest is malformed, has an integrity mismatch, does not match the selected base ROM, or its journal cannot materialize safely, the existing editor session is preserved.

Never delete the original ROM or your last explicit project save merely because a recovery snapshot exists.

## Experimental and research-blocked features

Stable builds hide experimental tooling. Development builds may expose selected experimental features when explicitly enabled. Animation/frame mutation and plugins remain research-blocked: those operations return errors instead of pretending a change was persisted or executing untrusted code.
