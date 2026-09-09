# Layout Pack Format

## Version 2.0

Packs describe selected boxer sprite-bin edits relative to an immutable source
ROM. `source_sha1` identifies that base, not the current edited revision. Export
stores only changed byte runs; it does not embed an unchanged base ROM.

Existing metadata fields (`name`, `author`, `description`, `created_at`, and
`layouts`) remain. Each bin adds an `edits` array:

```json
{
  "filename": "example.bin",
  "pc_offset": "0x48000",
  "size": 32,
  "category": "Raw",
  "label": null,
  "edits": [
    { "offset": 3, "expected": [0, 0], "replacement": [1, 2] }
  ]
}
```

The example is illustrative, not a game-address recommendation. Edit offsets
are relative to the bin. Byte values are integers from 0 to 255. Expected and
replacement arrays must have identical, nonzero lengths and fit inside the bin.

## Application

- Select at least one boxer contained in both the pack and current manifest.
- Bin filename, numeric PC offset, and size must match the manifest.
- The source SHA-1 must match under the same lock used to commit the journal edit.
- Current bytes must equal their expected or already-applied replacement values.
- Conflicting overlapping edits fail; identical shared-bin edits are deduplicated.
- All selected edits commit as one transaction. Any conflict leaves ROM state unchanged.
- Undo/Redo and project persistence use the canonical journal, not browser state.

No-change application returns an error rather than creating an empty transaction.
Validation uses the same payload planning and current-byte checks as application,
without creating a journal entry. Single-boxer import validates only that selected
boxer; whole-pack inspection validates all layouts. Application rechecks under the
journal lock because a validation report is not a reservation of ROM state.
Pack files are limited to 2 MiB. Installed pack paths are supplied by the backend
and live under the application's data directory, not the repository checkout.

## Compatibility and Limits

Version 1 metadata-only packs remain parseable for inspection but cannot be
applied as edits. Version 2 currently supports same-size patches within existing
manifest sprite bins. It does not relocate bins, resize them, or certify edited
game behavior. Shared assets can affect boxers outside the selected list.
Export excludes shared graphics by default. In the pack manager, explicitly
enable "Include shared graphics" for each selected fighter whose shared edits
should be included. Single-boxer quick export includes unique graphics only.

Packs can contain user-authored data or small fragments of changed game data.
Review contents and distribution rights before sharing. Never distribute a base
ROM inside a pack.
