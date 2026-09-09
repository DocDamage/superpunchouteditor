# Boxer Sprite Preview

The boxer editor intentionally provides two different graphics views. They answer different questions and should not be compared as if they were the same image.

## Complete pose view

Open a boxer in the editor's stable workflow and use **Assembled Pose Preview**. The preview shows a complete in-game pose and provides:

- a **Prev** button;
- a **Next** button; and
- a pose selector showing the pose index and ROM data address.

The preview is generated from the loaded ROM revision. It is read-only and does not change the ROM or edit journal.

## Raw tile-bank view

**Raw Tile Banks** displays the individual 8×8 graphics tiles in ROM/bank order. The game does not draw those tiles as one linear sheet. It loads selected ranges into multiple object-VRAM windows and places them with the pose's compact OAM command stream. As a result, a raw bank can look like disconnected strips, repeated pieces, or a vertically chopped boxer. That is expected for this view.

Use Raw Tile Banks when editing or comparing individual tiles. Use Assembled Pose Preview when checking how the game composes the character.

## Renderer data flow

The renderer follows the USA Super Punch-Out!! sprite layout represented by the active manifest and ROM:

1. The pose record supplies three tile-set IDs and a pose-data address.
2. The fighter graphics header resolves the source, bank, size, and VRAM configuration tables.
3. Compressed fighter graphics use the exact SPO base/flag decompressor; uncompressed banks use the normal 4bpp tile decoder.
4. The three compressed streams are associated with their runtime WRAM windows:
   - stream 1 → `$7E:8000`;
   - stream 2 → `$7F:0000`;
   - stream 3 → `$7F:8000`.
5. The configured VRAM destinations map each selected source range to the game's object tile slots.
6. The pose command stream is decoded into small/large OAM entries, including coordinates, tile numbers, palette attributes, and horizontal/vertical flips.
7. The boxer palette is applied and the result is returned as a pixelated PNG preview.

The stream order is important. A fighter's source table may list `$7F:0000`, `$7E:8000`, and `$7F:8000` in a different order than the compressed streams appear in the manifest. Mapping by filename order alone produces complete-looking fragments in the wrong places, which was the cause of the previously chopped previews.

## Troubleshooting

### The raw bank still looks chopped

That is normal. Select **Assembled Pose Preview** above the raw bank list.

### The assembled preview is blank or reports an error

Confirm that:

1. a supported ROM is loaded;
2. the ROM region matches the manifest selected by the editor;
3. the selected boxer exists in the active manifest; and
4. the ROM's graphics assets have not been truncated or replaced with unrelated data.

The preview reports a render error instead of silently presenting a misleading partial character.

### A pose is different from another pose

That is expected. Poses can use different tile-set IDs, object sizes, coordinates, palette attributes, and flips. Use the selector to compare them. A raw tile bank should not be used to judge pose composition.

## Verification

The renderer's codec and asset paths are covered by the Rust tests. On Windows, the local development checks are:

```powershell
cargo fmt --all -- --check
cargo test -p asset-core --lib
cargo test -p asset-core --test fighter_bin_edit_test
cargo test -p tauri-appsuper-punch-out-editor --lib

cd apps/desktop
npm test
npm run build
```

Visual validation may use a privately owned local ROM, but ROM files, save states, and emulator binaries must never be committed or uploaded.
