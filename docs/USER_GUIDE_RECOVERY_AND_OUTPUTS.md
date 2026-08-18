# User Guide — Projects, Recovery, ROM Save, Patches and Testing

## Before editing

Keep your original ROM separately. The editor treats the loaded source as an immutable base and records supported changes in an edit journal. Stable Save As, patch export, project persistence, comparison and embedded-emulator testing are designed to reference that same current journal materialization.

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

## Embedded emulator testing

The stable test path loads the current materialized ROM bytes in memory and records the editor revision/SHA-1 loaded by the emulator. Legacy external-emulator launching is experimental because a disk-path workflow can test an older image than the editor currently represents.

## Recovery

Project-v2 writes preserve the previous valid manifest as a recovery file while the new manifest is verified. If a project manifest is malformed, has an integrity mismatch, does not match the selected base ROM, or its journal cannot materialize safely, the existing editor session is preserved.

Never delete the original ROM or your last explicit project save merely because a recovery snapshot exists.

## Experimental and research-blocked features

Stable builds hide experimental tooling. Development builds may expose selected experimental features when explicitly enabled. Animation/frame mutation and plugins remain research-blocked: those operations return errors instead of pretending a change was persisted or executing untrusted code.
