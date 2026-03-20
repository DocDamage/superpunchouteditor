# Sprite Editing

Sprite editing allows you to modify the graphics of fighters, UI elements, and other visual assets in Super Punch-Out!!.

## Understanding SNES Graphics

### 4BPP Graphics Mode

Super Punch-Out!! uses 4-bit per pixel (4BPP) graphics:

- **16 colors per tile** (4 bits = 16 values)
- **8x8 pixel tiles**
- **32 bytes per tile** (8×8 pixels × 4 bits = 256 bits = 32 bytes)

### Tile Format

Each 8x8 tile is stored as:
```
Bitplanes 0-1: Low bits of color index
Bitplanes 2-3: High bits of color index
```

The format is **planar**, meaning each bit of the color index is stored separately.

## The Sprite Editor

### Tile View

The editor displays tiles in a grid:
- **Grid size**: Configurable (default 16 tiles wide)
- **Tile size**: 8×8 pixels
- **Zoom**: Adjustable from 1x to 8x

### Tools

Available editing tools:

| Tool | Shortcut | Description |
|------|----------|-------------|
| Pencil | P | Draw individual pixels |
| Line | L | Draw straight lines |
| Rectangle | R | Draw filled/empty rectangles |
| Circle | C | Draw circles/ellipses |
| Fill | F | Flood fill an area |
| Eyedropper | I | Pick a color |
| Select | S | Select a region |

### Color Selection

- **Left click**: Draw with primary color
- **Right click**: Draw with secondary color
- **Number keys**: Quick select colors 1-0

## Importing Graphics

### From PNG

The recommended workflow:

1. Create/edit your sprite in an image editor (Photoshop, GIMP, Aseprite)
2. Save as PNG with transparency
3. Use **"Import PNG"** in the editor
4. Map colors to the fighter's palette
5. Apply changes

### Import Options

When importing, you can:
- **Auto-detect palette**: Try to match colors automatically
- **Manual mapping**: Specify exact color mappings
- **Dithering**: Apply Floyd-Steinberg dithering for smoother results
- **Quantize**: Reduce colors to fit 16-color limit

### Best Practices for PNG Creation

1. **Use exact palette colors**: Sample from the game when possible
2. **Keep within 16 colors**: Including transparency
3. **Proper sizing**: Dimensions should be multiples of 8
4. **Transparency**: Use true transparency, not a solid color

## Exporting Graphics

### To PNG

Export sprites for editing:

1. Select the sprite bin to export
2. Click **"Export to PNG"**
3. Choose layout (width in tiles)
4. Select destination

### Export Options

- **Include grid**: Add visible tile boundaries
- **Color 0 as transparent**: Make background transparent
- **Scale**: Export at 1x, 2x, 4x, etc.
- **Sprite boundaries**: Highlight individual sprites

## Working with Sprite Bins

### What is a Sprite Bin?

A sprite bin is a collection of tiles stored together in ROM:
- Contains multiple tiles (typically 64-256)
- May be compressed
- Shared between multiple fighters

### Unique vs Shared Bins

| Type | Description | Warning Level |
|------|-------------|---------------|
| Unique | Only one fighter uses this | Safe to edit |
| Shared | Multiple fighters reference this | Changes affect all |

### The Sprite Bin Editor

Features:
- **Tile grid**: View all tiles in the bin
- **Diff view**: See which tiles have been modified
- **Size indicator**: Shows current vs original size
- **Fit warning**: Alerts if tiles won't fit in original space

## Tile Editing Techniques

### Basic Drawing

1. Select the Pencil tool
2. Choose a color
3. Click on the tile to draw pixels
4. Use zoom for precision

### Copy and Paste

1. Select a tile or region
2. Press **Ctrl+C** to copy
3. Click destination tile
4. Press **Ctrl+V** to paste

### Mirror and Flip

Tools for symmetrical editing:
- **Horizontal flip**: Mirror left-right
- **Vertical flip**: Mirror top-bottom
- **Auto-mirror**: Draw on both sides simultaneously

### Tile Rotation

Rotate tiles in 90° increments:
- **Rotate 90° CW**: Clockwise
- **Rotate 90° CCW**: Counter-clockwise
- **Rotate 180°**: Upside down

## Advanced Features

### Tile Mapping

View how tiles are assembled into sprites:
- **OAM viewer**: See Object Attribute Memory data
- **Sprite assembly**: View composed sprites
- **Animation preview**: See tiles in motion

### Compression

Some sprite bins are compressed:
- **Automatic decompression**: Transparent to user
- **Size checking**: Warns if compressed data is too large
- **Recompression**: Automatic on save

### Batch Operations

Apply operations to multiple tiles:
1. Select multiple tiles (Ctrl+Click or drag selection)
2. Choose operation:
   - Shift colors
   - Apply filter
   - Copy to another bin

## Common Workflows

### Creating a New Fighter Sprite

1. Export existing fighter as reference
2. Create new sprites in image editor
3. Import PNG into sprite bin
4. Adjust palette as needed
5. Test in frame reconstructor

### Recoloring Existing Sprites

1. Export sprite bin to PNG
2. Open in image editor
3. Shift hues/adjust colors
4. Import back
5. Update palette if needed

### Fixing a Single Sprite

1. Locate the sprite in the tile viewer
2. Use pencil tool for small fixes
3. Or export single tile, edit, re-import
4. Verify in preview

## Troubleshooting

### "Import too large"
- Reduce number of colors
- Remove unnecessary tiles
- Check if bin is compressed

### "Colors look wrong"
- Verify palette is correct
- Check color 0 is transparent
- Ensure PNG uses correct palette

### "Changes not showing in preview"
- Click "Apply" after editing
- Check if viewing correct pose
- Verify tile IDs in frame data

### "Tile corruption"
- Don't exceed original bin size
- Check for compressed data issues
- Revert and try again

## Performance Tips

1. **Work at 2x zoom**: Good balance of speed and precision
2. **Use keyboard shortcuts**: Faster than mouse-only
3. **Batch similar edits**: Edit all tiles needing same change together
4. **Export frequently**: Save progress as PNG backups

## Related Topics

- [Palette Editing](./palette-editing.md)
- [Frame Reconstructor](./frame-reconstructor.md)
- [Animation Editor](./animation-editor.md)
