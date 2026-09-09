# Frame Reconstructor

The Frame Reconstructor allows you to edit how individual sprites are assembled to create fighter poses and frames.

## Understanding Frame Structure

### What is a Frame?

In Super Punch-Out!!, a frame is composed of:
- **Multiple sprites**: 8x8 or 16x16 pixel tiles
- **Position data**: X,Y coordinates for each sprite
- **Attributes**: Flip, palette, priority bits

### OAM (Object Attribute Memory)

The SNES stores sprite data in OAM:
- **Low OAM**: X position, Y position, tile ID, attributes
- **High OAM**: Size and X position MSB
- **512 bytes total**: Up to 128 sprites

### Sprite Attributes

Each sprite has attribute flags:
```
Bits 0-2: Palette selection (0-7)
Bit 3: Priority (0=low, 1=high)
Bit 4: X flip
Bit 5: Y flip
Bits 6-7: Sprite size (0=small, 1=large, 2=custom)
```

## The Frame Reconstructor Interface

### Canvas View

The main editing area:
- **Grid overlay**: 8x8 pixel grid
- **Sprite selection**: Click to select individual sprites
- **Multi-select**: Shift+click or drag box
- **Zoom**: Mouse wheel or slider

### Sprite List

Detailed sprite information:
- **Index**: Sprite number in OAM
- **Tile ID**: Graphics reference
- **Position**: X,Y coordinates
- **Size**: 8x8, 16x16, etc.
- **Attributes**: Visual indicator flags

### Properties Panel

Edit selected sprite(s):
- **Position**: X/Y numeric inputs
- **Tile ID**: Direct entry or picker
- **Size**: Dropdown selection
- **Flags**: Checkboxes for flip, palette, priority

## Working with Sprites

### Adding Sprites

1. Click **"Add Sprite"** button
2. Select from tile picker
3. Click on canvas to place
4. Adjust position as needed

### Moving Sprites

Methods to move sprites:
- **Drag**: Click and drag on canvas
- **Arrow keys**: Nudge 1 pixel
- **Shift+arrows**: Nudge 8 pixels
- **Properties**: Enter exact coordinates

### Removing Sprites

- **Delete key**: Remove selected sprite(s)
- **Backspace**: Same as delete
- **Context menu**: Right-click > Delete

### Modifying Sprites

Change sprite properties:
1. Select sprite(s)
2. Edit in Properties panel:
   - Change tile ID
   - Toggle flip flags
   - Switch palette
   - Adjust priority

## Coordinate System

### Screen Coordinates

SNES sprite coordinates:
- **X: 0-255**: Horizontal position
- **Y: 0-223**: Vertical position (NTSC)
- **Origin**: Top-left of screen

### Relative Positioning

Fighter coordinates are relative:
- Center point is typically middle of body
- Negative X = left of center
- Positive X = right of center
- Negative Y = above center
- Positive Y = below center

### Coordinate Display

The editor shows:
- **Absolute**: Raw SNES coordinates
- **Relative**: From fighter center
- **Screen**: Actual screen position

## Tile Management

### Tile Picker

Browse available tiles:
- **Filter by bin**: Show only specific sprite bin
- **Search**: Find by tile ID
- **Favorites**: Star frequently used tiles
- **Recent**: Recently selected tiles

### Tile Swapping

Replace tile references:
1. Select sprite(s)
2. Open tile picker
3. Click new tile
4. Preview updates immediately

### Batch Tile Operations

Apply to multiple sprites:
1. Multi-select sprites
2. Right-click menu:
   - Shift tile IDs (+1, +2, etc.)
   - Mirror X coordinates
   - Flip all horizontally

## Frame Templates

### Using Templates

Pre-configured frame layouts:
1. Click **"Templates"**
2. Browse categories (idle, punch, hit)
3. Select template
4. Apply to current frame

### Creating Templates

Save your own templates:
1. Configure frame as desired
2. Click **"Save Template"**
3. Name and categorize
4. Add optional description

### Template Library

Organize templates:
- **Fighter-specific**: For specific fighters
- **Generic**: Reusable across fighters
- **Import/Export**: Share with community

## Advanced Features

### Hitbox Visualization

Display collision areas:
- **Hit boxes**: Red overlay (attack area)
- **Hurt boxes**: Green overlay (vulnerable area)
- **Block boxes**: Blue overlay (defensive area)

### Frame Comparison

Compare with other frames:
1. Enable "Compare Mode"
2. Select reference frame
3. Ghost overlay appears
4. Highlight differences

### Animation Preview

Test in animation context:
1. Select animation sequence
2. Current frame highlighted
3. See how it flows
4. Adjust timing

## Working with Palettes

### Palette Assignment

Each sprite uses a palette:
- **0-7**: Available palettes
- **Varies by frame**: Different frames may use different palettes
- **Fighter-dependent**: Each fighter has unique palettes

### Palette Preview

Test different palettes:
1. Select sprite(s)
2. Change palette number
3. Preview updates
4. Find optimal appearance

## Import/Export

### Frame Data Export

Export frame configuration:
```json
{
  "frame_id": 12,
  "sprites": [
    {"tile": 64, "x": 120, "y": 80, "attr": 0x20},
    {"tile": 65, "x": 128, "y": 80, "attr": 0x20}
  ]
}
```

### Image Export

Export as PNG:
- **Current frame**: Single image
- **All frames**: Sprite sheet
- **With/without grid**: Toggle overlay
- **Transparent background**: Option

## Best Practices

### Organization

1. **Use consistent naming**: For templates and saved frames
2. **Document changes**: Add notes for significant edits
3. **Save iterations**: Don't overwrite original frames
4. **Test frequently**: Verify in-game appearance

### Performance

1. **Minimize sprites**: Fewer sprites = better performance
2. **Reuse tiles**: Efficient tile usage
3. **Check bounds**: Keep sprites on-screen
4. **Priority ordering**: Proper layering

### Visual Quality

1. **Pixel alignment**: Keep to 8-pixel boundaries when possible
2. **Consistent style**: Match original game aesthetic
3. **Color consistency**: Use appropriate palettes
4. **Proportions**: Maintain fighter proportions

## Troubleshooting

### "Sprites not showing"
- Check tile ID is valid
- Verify palette assignment
- Confirm sprite is on-screen

### "Wrong tiles displayed"
- Check tile ID calculation
- Verify correct sprite bin loaded
- Reload fighter data

### "Flickering sprites"
- Too many sprites per scanline (SNES limit: 32)
- Reduce number of overlapping sprites
- Check priority settings

### "Can't select sprite"
- Zoom in for precision
- Check layer visibility
- Verify not locked

## Related Topics

- [Animation Editor](./animation-editor.md)
- [Sprite Editing](./sprite-editing.md)
- [Palette Editing](./palette-editing.md)
