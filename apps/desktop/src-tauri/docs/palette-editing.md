# Palette Editing

Palettes define the colors used throughout Super Punch-Out!!. Understanding how to edit them is essential for customizing the game's appearance.

## Understanding SNES Palettes

Super Punch-Out!! uses the SNES color format:

### Color Format
- **15-bit color depth** (5 bits per RGB channel)
- **32,768 possible colors**
- Colors are stored in **little-endian 16-bit words**

### Color Encoding

Each color is encoded as:
```
Bit 0-4:   Blue (0-31)
Bit 5-9:   Green (0-31)
Bit 10-14: Red (0-31)
Bit 15:    Unused (always 0)
```

Example color values:
- `0x0000` = Black
- `0x7FFF` = White
- `0x001F` = Pure blue
- `0x03E0` = Pure green
- `0x7C00` = Pure red

## The Palette Editor

### Interface Overview

The Palette Editor displays:
- **Color grid**: All colors in the selected palette
- **Color picker**: RGB sliders and hex input
- **Preview**: Live preview of how colors look
- **Import/Export**: Save and load palette files

### Selecting Colors

1. Click any color in the grid to select it
2. Use the color picker to adjust:
   - **R (Red)**: 0-255 (scaled to 0-31 internally)
   - **G (Green)**: 0-255 (scaled to 0-31 internally)
   - **B (Blue)**: 0-255 (scaled to 0-31 internally)
3. Or enter a hex color code directly

### SNES Color Limitations

Important constraints when choosing colors:

1. **Color depth**: Only 32 levels per channel (vs. 256 on modern systems)
2. **Color snapping**: Colors will be rounded to nearest SNES value
3. **Transparency**: Color 0 in any palette is transparent for sprites

## Palette Organization

### Fighter Palettes

Each fighter uses multiple palettes:

| Palette Type | Colors | Usage |
|--------------|--------|-------|
| Skin | 16 | Fighter's skin tones |
| Shorts | 16 | Boxing shorts |
| Gloves | 16 | Boxing gloves |
| Hair | 16 | Hair color |
| Shoes | 16 | Footwear |

### Special Palettes

- **UI Palettes**: Menu screens, HUD elements
- **Background Palettes**: Ring backgrounds, crowd
- **Effect Palettes**: Hit effects, special moves

## Editing Techniques

### Creating Gradient Palettes

For smooth color transitions:

1. Select the starting color
2. Hold **Shift** and click the ending color
3. Use the "Create Gradient" button to auto-fill intermediate colors

### Copying Palettes

To copy colors between palettes:

1. Select the source color
2. Press **Ctrl+C** to copy
3. Select the destination
4. Press **Ctrl+V** to paste

### Importing External Palettes

Supported formats:
- **.pal**: Standard 768-byte RGB palette
- **.act**: Adobe Photoshop palette
- **.gpl**: GIMP palette
- **JSON**: Custom format with metadata

## Color Theory Tips

### Fighter Design

When recoloring fighters, consider:

1. **Contrast**: Ensure fighters are visible against backgrounds
2. **Readability**: Colors should clearly distinguish body parts
3. **Theme**: Colors should match the fighter's personality/nationality

### Skin Tones

The editor includes a preset library of skin tones:
- Light (Type 1-3)
- Medium (Type 4-6)
- Dark (Type 7-9)
- Special (zombie, alien, etc.)

### National Colors

Common palette themes by nationality:
- **USA**: Red, white, blue
- **Japan**: Red, white
- **Mexico**: Green, white, red
- **Russia**: White, blue, red

## Advanced Features

### Palette Animation

Some palettes animate (flashing effects):

1. Select an animated palette
2. The editor shows all animation frames
3. Edit each frame individually
4. Preview the animation with the play button

### Shared Palettes

Some palettes are shared between fighters:

- Shared palettes show a warning indicator
- Changes affect all fighters using that palette
- Use "Duplicate Palette" to create a unique copy

### Color Cycling

The SNES supports color cycling for water/lava effects:

1. Select a color range
2. Enable "Color Cycling"
3. Set cycle speed and direction
4. Preview in real-time

## Common Issues

### "Colors look different in-game"
- SNES has limited color precision
- Emulators may apply filters
- Test on real hardware if possible

### "Transparency not working"
- Color 0 must be used for transparency
- Ensure you're editing the correct palette
- Check sprite properties

### "Palette changes don't show"
- Make sure you've clicked "Apply"
- Check if palette is loaded into VRAM
- Verify you're editing the correct offset

## Import/Export

### Exporting Palettes

Save your palette for use in other projects:

1. Click **"Export Palette"**
2. Choose format (PAL, ACT, GPL, or JSON)
3. Select destination
4. Add optional metadata

### Importing Palettes

Load palettes from external sources:

1. Click **"Import Palette"**
2. Select palette file
3. Choose import options:
   - **Merge**: Combine with existing colors
   - **Replace**: Overwrite entire palette
   - **Remap**: Match colors to available slots

## Best Practices

1. **Save frequently**: Palette edits are small but important
2. **Document changes**: Use the notes feature
3. **Test on target hardware**: Colors may vary between emulators
4. **Keep backups**: Export palettes before major changes

## Related Topics

- [Sprite Editing](./sprite-editing.md)
- [Animation Editor](./animation-editor.md)
- [Frame Reconstructor](./frame-reconstructor.md)
