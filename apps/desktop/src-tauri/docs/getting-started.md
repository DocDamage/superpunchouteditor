# Getting Started with Super Punch-Out!! Editor

Welcome to the Super Punch-Out!! Editor! This guide will help you get up and running with editing your favorite boxing game.

## What You Can Do

This editor allows you to modify various aspects of Super Punch-Out!! for the SNES:

- **Edit Palettes**: Change the colors used by fighters, UI elements, and backgrounds
- **Modify Sprites**: Import and export sprite graphics using standard image formats
- **Edit Fighter Stats**: Modify punch power, speed, defense, and other attributes
- **Create Animations**: Build custom animations using the animation editor
- **Frame Reconstruction**: Edit how sprites are assembled into fighter poses
- **Manage Projects**: Save your work and export as IPS patches

## Prerequisites

Before you begin, you'll need:

1. A legally obtained copy of the Super Punch-Out!! ROM file (`.sfc` or `.smc` format)
2. The editor supports the following ROM versions:
   - USA version (most common)
   - Japanese version
   - European version

## Opening Your First ROM

1. Launch the Super Punch-Out!! Editor
2. Click the **"Open ROM"** button in the sidebar
3. Navigate to your Super Punch-Out!! ROM file and select it
4. The editor will validate the ROM and load it into memory

You'll see a green status badge showing "ROM OK" with the first 8 characters of the ROM's SHA1 hash. This helps ensure you're working with a valid ROM.

## Understanding the Interface

### Sidebar

The left sidebar contains:
- **ROM controls**: Open/Switch ROM buttons
- **Undo/Redo**: Navigation through your edit history
- **Tab navigation**: Switch between different editing modes
- **Boxer list**: Select which fighter to edit

### Main Content Area

The main area displays different content based on your selected tab:
- **Editor**: Main editing interface for palettes and sprites
- **Viewer**: Visual preview of fighters and their poses
- **Scripts**: Fighter behavior and stat editing
- **Animations**: Animation editor and preview
- **Frames**: Frame reconstruction tools
- **Project**: Project management and export

## Your First Edit

Let's make a simple palette change:

1. Select a boxer from the list (try "Gabby Jay" for a simple example)
2. Navigate to the **Editor** tab
3. Scroll down to the **Palette Editor** section
4. Click on any color square to open the color picker
5. Choose a new color and click **Apply**
6. See your changes reflected in the preview

## Saving Your Work

### Projects

Projects allow you to save your entire editing session:

1. Go to the **Project** tab
2. Click **"Create New Project"**
3. Give your project a name and optional description
4. Your edits are automatically saved to the project

### Exporting as IPS Patch

To share your modifications:

1. Make your desired edits
2. Go to the **Export** section at the bottom of the boxer detail page
3. Click **"Export IPS Patch"**
4. Choose a location to save the `.ips` file
5. Others can apply this patch to their ROM using any IPS patcher

## Getting Help

- Press **F1** to open the help system
- Click the **?** button next to any section for context-sensitive help
- Use the search bar in the help window to find specific topics

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| Ctrl+Z | Undo |
| Ctrl+Shift+Z | Redo |
| Ctrl+Y | Redo (alternative) |
| F5 | Test in emulator |
| F1 | Open help |

## Next Steps

- Learn about [Palette Editing](./palette-editing.md)
- Explore [Sprite Editing](./sprite-editing.md)
- Understand [ROM Validation](./rom-validation.md)
- Read about [Troubleshooting](./troubleshooting.md)

---

**Tip**: Always work on a copy of your ROM file, not the original!
