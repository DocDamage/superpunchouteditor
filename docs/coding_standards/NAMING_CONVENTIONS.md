# Naming Conventions

This document defines the naming standards for the Super Punch-Out!! Editor codebase.

## Rust

### General
- **Structs/Enums**: `PascalCase` (e.g., `BoxerRecord`, `AssetType`)
- **Functions/Variables**: `snake_case` (e.g., `get_boxer`, `pc_offset`)
- **Constants**: `SCREAMING_SNAKE_CASE` (e.g., `MAX_ROM_SIZE`)
- **Modules**: `snake_case` (e.g., `rom_commands`)

### Domain Terms
- Use **"boxer"** not "fighter" for consistency with product name
- Use **"pc_offset"** for PC file offsets
- Use **"snes_addr"** for SNES bus addresses only
- Use **"asset"** for graphics/palette resources

### Type Names
| Type | Naming | Example |
|------|--------|---------|
| Metadata | `{Noun}Metadata` | `BoxerMetadata` |
| Manager | `{Noun}Manager` | `BoxerManager` |
| Header | `{Noun}Header` | `BoxerHeader` |
| Params | `{Adjective}{Noun}Params` | `EditableBoxerParams` |
| Annotations | `{Noun}Annotations` | `BoxerAnnotations` |
| Animations | `{Noun}Animations` | `BoxerAnimations` |

## TypeScript

### General
- **Components**: `PascalCase` (e.g., `BoxerEditor.tsx`)
- **Interfaces/Types**: `PascalCase` (e.g., `BoxerRecord`)
- **Functions/Variables**: `camelCase` (e.g., `getBoxer`, `pcOffset`)
- **Props**: `camelCase` (e.g., `boxerId`, `poseIndex`)

### Property Names
- Use `name` for display names (not `fighter` or other terms)
- Use `id` for identifiers (not `idx` or abbreviations)
- Use `pcOffset` for PC offsets in camelCase contexts

## Tauri Commands

### Pattern: `{action}_{noun}`

#### Queries (get_*, list_*)
```rust
get_boxer_list          // Get list of all boxers
get_boxer               // Get single boxer by key
get_boxer_poses         // Get poses for a boxer
get_boxer_header        // Get boxer header data
get_palette             // Get palette data
get_rom_info            // Get ROM information
list_assets             // List available assets
```

#### Mutations (update_*, create_*, delete_*)
```rust
update_palette
update_sprite
update_boxer_stats
create_project
delete_project
```

#### Operations (export_*, import_*, render_*)
```rust
export_palette_as_png
import_palette_from_png
export_patch_ips
render_boxer_pose
render_sprite_sheet
```

#### Special Cases
```rust
open_rom
close_rom
save_project
load_project
```

## Address/Offset Naming

### PC Offsets
- **Rust**: `pc_offset` (parameter/variable), `start_pc` (field in AssetFile)
- **TypeScript**: `pcOffset` (camelCase), `startPc` (if needed)

### SNES Addresses
- **Rust**: `snes_addr` (when specifically SNES bus address)
- **TypeScript**: `snesAddr` (camelCase)

### Functions
- `parse_offset()` - Parse hex string to usize (handles 0x prefix)
- Avoid `parse_pc_offset` (redundant with `parse_offset`)

## Migration Guide

### Deprecated Names (to be removed)
| Old | New | Status |
|-----|-----|--------|
| `FighterMetadata` | `BoxerMetadata` | ✅ Migrated |
| `FighterManager` | `BoxerManager` | ✅ Migrated |
| `FighterHeader` | `BoxerHeader` | ✅ Migrated |
| `FighterAnnotations` | `BoxerAnnotations` | ✅ Migrated |
| `FighterAnimations` | `BoxerAnimations` | ✅ Migrated |
| `EditableFighterParams` | `EditableBoxerParams` | ✅ Migrated |
| `fighter_id` | `boxer_id` | ✅ Migrated |
| `get_fighter_list` | `get_boxer_list` | ✅ Migrated |
| `get_fighter_poses` | `get_boxer_poses` | ✅ Migrated |
| `render_fighter_pose` | `render_boxer_pose` | ✅ Migrated |
| `BoxerRecord.fighter` | `BoxerRecord.name` | ✅ Migrated |

## File Naming

- **Rust files**: `snake_case.rs` (e.g., `boxer_commands.rs`)
- **TypeScript/React files**: `PascalCase.tsx` for components (e.g., `BoxerEditor.tsx`)
- **CSS files**: `kebab-case.css` (e.g., `boxer-editor.css`)
- **Documentation**: `UPPER_SNAKE_CASE.md` (e.g., `NAMING_CONVENTIONS.md`)
