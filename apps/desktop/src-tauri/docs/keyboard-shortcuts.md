# Keyboard Shortcuts

Master these keyboard shortcuts to work more efficiently in the Super Punch-Out!! Editor.

## Global Shortcuts

| Shortcut | Action |
|----------|--------|
| `F1` | Open Help System |
| `F5` | Test in Emulator |
| `Ctrl + O` | Open ROM |
| `Ctrl + S` | Save Project |
| `Ctrl + Shift + S` | Save Project As |
| `Ctrl + P` | Export Patch |
| `Ctrl + Q` | Quit Application |
| `Ctrl + ,` | Open Settings |

## Undo/Redo

| Shortcut | Action |
|----------|--------|
| `Ctrl + Z` | Undo |
| `Ctrl + Shift + Z` | Redo |
| `Ctrl + Y` | Redo (alternative) |

## Navigation

| Shortcut | Action |
|----------|--------|
| `Tab` | Next field |
| `Shift + Tab` | Previous field |
| `Ctrl + Tab` | Next tab |
| `Ctrl + Shift + Tab` | Previous tab |
| `Ctrl + 1-9` | Switch to tab 1-9 |
| `Esc` | Close modal/cancel action |

## Palette Editor

| Shortcut | Action |
|----------|--------|
| `Click` | Select color |
| `Ctrl + Click` | Multi-select colors |
| `Shift + Click` | Select color range |
| `Ctrl + C` | Copy selected color(s) |
| `Ctrl + V` | Paste color(s) |
| `Ctrl + D` | Duplicate palette |
| `Delete` | Reset color to black |
| `1-9, 0` | Quick-select color index |

### Color Picker Shortcuts

| Shortcut | Action |
|----------|--------|
| `↑/↓` | Adjust R by 1 |
| `←/→` | Adjust G by 1 |
| `Shift + ↑/↓` | Adjust B by 1 |
| `Page Up/Down` | Increase/decrease by 10 |
| `Enter` | Apply color |
| `Escape` | Cancel |

## Sprite Editor

| Shortcut | Action |
|----------|--------|
| `P` | Pencil tool |
| `L` | Line tool |
| `R` | Rectangle tool |
| `C` | Circle tool |
| `F` | Fill tool |
| `I` | Eyedropper tool |
| `S` | Select tool |
| `Z` | Zoom tool |
| `Ctrl + Z` | Undo (when canvas focused) |

### Canvas Navigation

| Shortcut | Action |
|----------|--------|
| `Space + Drag` | Pan canvas |
| `Ctrl + +` | Zoom in |
| `Ctrl + -` | Zoom out |
| `Ctrl + 0` | Reset zoom |
| `1-4` | Set zoom level (1x-4x) |
| `Home` | Go to top-left |
| `End` | Go to bottom-right |

### Drawing Shortcuts

| Shortcut | Action |
|----------|--------|
| `Left Click` | Draw with primary color |
| `Right Click` | Draw with secondary color |
| `Shift + Drag` | Constrain to straight line |
| `Alt + Click` | Pick color (eyedropper) |
| `X` | Swap primary/secondary color |
| `[` | Decrease brush size |
| `]` | Increase brush size |

### Tile Operations

| Shortcut | Action |
|----------|--------|
| `Ctrl + X` | Cut tile(s) |
| `Ctrl + C` | Copy tile(s) |
| `Ctrl + V` | Paste tile(s) |
| `Ctrl + D` | Duplicate tile(s) |
| `Delete` | Delete tile(s) |
| `H` | Flip horizontally |
| `V` | Flip vertically |
| `R` | Rotate 90° CW (when not in rect mode) |
| `Shift + R` | Rotate 90° CCW |

## Frame Reconstructor

| Shortcut | Action |
|----------|--------|
| `A` | Add sprite mode |
| `S` | Select mode |
| `M` | Move mode |
| `Delete` | Delete selected sprite(s) |
| `Ctrl + A` | Select all sprites |
| `Ctrl + D` | Deselect all |

### Sprite Positioning

| Shortcut | Action |
|----------|--------|
| `Arrow Keys` | Nudge selected sprite 1px |
| `Shift + Arrows` | Nudge selected sprite 8px |
| `Ctrl + Arrows` | Nudge selected sprite 0.5px (subpixel) |
| `[` | Send to back |
| `]` | Bring to front |

### View Options

| Shortcut | Action |
|----------|--------|
| `G` | Toggle grid |
| `H` | Toggle hitboxes |
| `B` | Toggle background |
| `T` | Toggle transparency |
| `D` | Toggle debug info |

## Animation Editor

| Shortcut | Action |
|----------|--------|
| `Space` | Play/Pause animation |
| `←` | Previous frame |
| `→` | Next frame |
| `Shift + ←` | Previous keyframe |
| `Shift + →` | Next keyframe |
| `Home` | Go to first frame |
| `End` | Go to last frame |
| `Ctrl + C` | Copy frame(s) |
| `Ctrl + V` | Paste frame(s) |
| `Delete` | Delete selected frame(s) |

### Timeline

| Shortcut | Action |
|----------|--------|
| `+` | Increase frame duration |
| `-` | Decrease frame duration |
| `K` | Toggle keyframe |
| `L` | Toggle loop point |

## Script Editor

| Shortcut | Action |
|----------|--------|
| `Ctrl + F` | Find |
| `Ctrl + H` | Replace |
| `Ctrl + G` | Go to address |
| `Ctrl + B` | Add bookmark |
| `F2` | Next bookmark |
| `Shift + F2` | Previous bookmark |
| `Tab` | Indent selection |
| `Shift + Tab` | Unindent selection |

## Help System

| Shortcut | Action |
|----------|--------|
| `F1` | Open Help |
| `/` or `Ctrl + F` | Search help |
| `↑/↓` | Navigate search results |
| `Enter` | Open selected result |
| `Alt + ←` | Back |
| `Alt + →` | Forward |
| `Escape` | Close help |

## Project Management

| Shortcut | Action |
|----------|--------|
| `Ctrl + N` | New Project |
| `Ctrl + O` | Open Project |
| `Ctrl + Shift + O` | Open Recent |
| `Ctrl + S` | Save Project |
| `Ctrl + Shift + S` | Save Project As |
| `Ctrl + E` | Export |
| `Ctrl + I` | Import |

## Boxer/Asset Selection

| Shortcut | Action |
|----------|--------|
| `↑/↓` | Navigate boxer list |
| `Page Up/Down` | Scroll list faster |
| `Enter` | Select boxer |
| `Ctrl + F` | Search boxers |
| `1-9` | Quick-select boxer by position |

## Tour Shortcuts

When a guided tour is active:

| Shortcut | Action |
|----------|--------|
| `→` or `Space` | Next step |
| `←` | Previous step |
| `Escape` | Skip tour |
| `Home` | Restart tour |

## Creating Custom Shortcuts

### Configuration File

Customize shortcuts in `shortcuts.json`:
```json
{
  "custom": {
    "Ctrl+Shift+P": "export_palette",
    "F12": "screenshot"
  }
}
```

### Reset to Defaults

Restore default shortcuts:
1. Go to **Settings > Keyboard**
2. Click **"Reset to Defaults"**
3. Confirm action

## Printable Cheat Sheet

Generate a printable reference:
1. Go to **Help > Keyboard Shortcuts**
2. Click **"Print Cheat Sheet"**
3. Select categories to include
4. Print or save as PDF

## macOS Equivalents

On macOS, use `Cmd` instead of `Ctrl`:

| Windows/Linux | macOS |
|---------------|-------|
| `Ctrl + O` | `Cmd + O` |
| `Ctrl + S` | `Cmd + S` |
| `Ctrl + Z` | `Cmd + Z` |
| `Alt` | `Option` |

## Accessibility

For users with mobility impairments:

- All shortcuts have menu equivalents
- Sticky keys supported
- Voice control compatible
- Full keyboard navigation

## Related Topics

- [Getting Started](./getting-started.md)
- [Help System Overview](./getting-started.md#help)
