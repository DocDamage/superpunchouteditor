# Roster Metadata Editor Implementation

## Overview

This document describes the implementation of the **Roster Metadata Editor** for the Super Punch-Out!! editor. This feature allows editing of game-level roster data including boxer names, circuit assignments, unlock order, and introductory text.

## Files Created

### Rust Backend (rom-core)

- **`crates/rom-core/src/roster.rs`** - Core roster data structures and logic
  - `RosterData` - Complete roster data structure
  - `BoxerRosterEntry` - Individual boxer entry
  - `Circuit` / `CircuitType` - Circuit definitions (Minor, Major, World, Special)
  - `SpoTextEncoder` - Custom text encoding/decoding for SPO text
  - `RosterLoader` - ROM loading (placeholder for research)
  - Validation and error handling

### Rust Backend (Tauri Commands)

- **`apps/desktop/src-tauri/src/roster_commands.rs`** - Tauri command handlers
  - `get_roster_data` - Get complete roster
  - `get_boxer_roster_entry` - Get single boxer
  - `update_boxer_name` - Update boxer name with validation
  - `validate_boxer_name` - Validate name encoding/length
  - `update_boxer_circuit` - Change circuit assignment
  - `update_unlock_order` - Modify unlock progression
  - `set_champion_status` - Toggle champion flag
  - `get_intro_text` / `update_intro_text` - Intro text editing
  - `validate_roster_changes` - Full roster validation
  - `get_text_encoding_info` - Get encoding support info
  - Research/debug commands for ROM offsets

### TypeScript Types

- **`apps/desktop/src/types/roster.ts`** - Frontend type definitions

### React Components

- **`apps/desktop/src/components/RosterEditor.tsx`** - Main editor with tabs
  - Tab: Boxer Names
  - Tab: Circuits (drag-drop assignment)
  - Tab: Unlock Order
  - Tab: Intro Text

- **`apps/desktop/src/components/BoxerNameEditor.tsx`** - Name editing with validation
  - Character encoding validation
  - Byte length checking
  - Preview of encoded text
  - Supported characters info

- **`apps/desktop/src/components/CircuitEditor.tsx`** - Visual circuit editor
  - Drag-and-drop between circuits
  - Champion flag toggle
  - Circuit color coding

- **`apps/desktop/src/components/RosterEditor.css`** - Component styles

### Integration

- **`apps/desktop/src/components/index.ts`** - Added exports
- **`apps/desktop/src/App.tsx`** - Added "Roster" tab

## ROM Research Status

### Known Information

The implementation includes placeholder data based on known game information:

**Boxer List (16 total):**
1. Gabby Jay (Minor Circuit)
2. Bear Hugger (Minor Circuit)
3. Piston Hurricane (Minor Circuit)
4. Bald Bull (Minor Circuit - Champion)
5. Bob Charlie (Major Circuit)
6. Dragon Chan (Major Circuit)
7. Masked Muscle (Major Circuit)
8. Mr. Sandman (Major Circuit - Champion)
9. Aran Ryan (World Circuit)
10. Heike Kagero (World Circuit)
11. Mad Clown (World Circuit)
12. Super Macho Man (World Circuit - Champion)
13. Narcis Prince (Special Circuit)
14. Hoy Quarlow (Special Circuit)
15. Rick Bruiser (Special Circuit)
16. Nick Bruiser (Special Circuit - Champion)

### TODO: ROM Addresses to Research

The following ROM addresses need to be researched to complete the implementation:

1. **Name Table Location**
   - Where are boxer names stored?
   - Are they compressed or in a direct table?
   - What is the text encoding scheme?

2. **Circuit Assignment Table**
   - Where is circuit data stored?
   - How are circuit assignments encoded?

3. **Unlock Order Table**
   - Where is unlock progression data?
   - How is the starting boxer (Gabby Jay) marked?

4. **Intro Text Table**
   - Where are pre-match introductions stored?
   - Are they compressed?
   - Pointer table format?

5. **Text Encoding**
   - Complete character map for SPO text encoding
   - Boxing symbols (stars, etc.)
   - Accented characters for international boxers

## Text Encoding

The current implementation includes a placeholder text encoder (`SpoTextEncoder`) that supports:
- A-Z, a-z (ASCII)
- Basic punctuation: space, !, ?, ., ,, -, '
- Boxing symbols: TODO - need research

**Known Limitations:**
- Characters not in the supported list are replaced with '?'
- Actual SPO encoding may differ - needs verification

## Safety Features

1. **Name Length Limits**
   - Maximum 16 bytes per name
   - Validation before saving
   - Warning for unsupported characters

2. **Validation System**
   - Duplicate name detection
   - Circuit assignment validation
   - Unlock order gap detection
   - Champion flag consistency checking

3. **Undo/Redo Support**
   - Changes go through pending writes system
   - Can be undone via existing undo system

4. **ROM Safety**
   - Changes validated before writing
   - Placeholder addresses prevent accidental writes

## Usage

1. Open a ROM in the editor
2. Click the "Roster" tab in the sidebar
3. Select a sub-tab:
   - **Boxer Names**: Edit names with validation
   - **Circuits**: Drag boxers between circuits
   - **Unlock Order**: Adjust progression
   - **Intro Text**: Edit pre-match text
4. Changes are validated in real-time
5. Use "Reset to Defaults" to revert all changes

## Future Enhancements

1. **ROM Integration**
   - Add actual ROM address lookup once researched
   - Implement proper text encoding/decoding
   - Support for compressed text

2. **Advanced Features**
   - Import/export roster data
   - Batch name changes
   - Roster templates
   - Circuit reordering

3. **Research Tools**
   - ROM scanner for text tables
   - Hex viewer for roster data
   - Diff tool for roster changes

## Development Notes

- The editor is functional with placeholder data
- All UI components are implemented
- Tauri commands are registered
- Validation is working
- Ready for ROM address integration once research is complete

## Testing

To test the implementation:

```bash
# Run the Tauri app
cd apps/desktop
npm run tauri dev

# Or build for production
npm run tauri build
```

Navigate to the "Roster" tab to see the editor in action.
