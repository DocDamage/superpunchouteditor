# Community Layout Packs

This directory contains user-imported layout packs for the Super Punch-Out!! Editor.

## What are Layout Packs?

Layout Packs are JSON files that contain curated boxer layout configurations. They allow users to:

- **Export** their current layout configurations to share with others
- **Import** layout packs created by the community
- **Validate** layout packs before applying them
- **Apply** layouts selectively to specific boxers

## Directory Structure

```
data/boxer-layouts/
├── default/              # Built-in layouts (managed by editor)
│   ├── gabby_jay.json
│   ├── bear_hugger.json
│   └── ...
└── community/            # User-imported packs (this directory)
    ├── hd_sprites_pack.json
    ├── accurate_colors_pack.json
    └── README.md
```

## Layout Pack Format

```json
{
  "version": "1.0",
  "name": "HD Sprite Layouts",
  "author": "Your Name",
  "description": "Description of this layout pack",
  "created_at": "2026-03-19T12:00:00Z",
  "layouts": [
    {
      "boxer_key": "Hoy Quarlow",
      "version": "1.0",
      "layout_type": "reference",
      "bins": [
        {
          "filename": "GFX_Sprite_HoyQuarlow1.bin",
          "pc_offset": "0x180000",
          "size": 2048,
          "category": "Compressed Sprite",
          "label": "Optional description"
        }
      ],
      "notes": "Optional notes about this boxer's layout"
    }
  ]
}
```

### Field Descriptions

**LayoutPack:**
- `version`: Pack format version (currently "1.0")
- `name`: Display name for the pack
- `author`: Creator's name
- `description`: Brief description of the pack's purpose
- `created_at`: ISO 8601 timestamp
- `layouts`: Array of boxer layouts

**BoxerLayout:**
- `boxer_key`: Key identifying the boxer (e.g., "Hoy Quarlow")
- `version`: Layout version
- `layout_type`: Either "reference" or "custom"
- `bins`: Array of bin configurations
- `notes`: Optional notes

**LayoutBin:**
- `filename`: Name of the bin file
- `pc_offset`: PC offset in the ROM (hex string)
- `size`: Size in bytes
- `category`: Category (e.g., "Compressed Sprite", "Uncompressed Sprite")
- `label`: Optional descriptive label

## Using Layout Packs

### In the Editor

1. **Open the Packs Tab**: Click on "Packs" in the sidebar navigation
2. **Import a Pack**: Click "Import Pack" and select a JSON file
3. **Preview**: Click "Preview" to see pack contents and compatibility
4. **Apply**: Click "Apply" to apply the layout to boxers

### From Boxer Preview Sheet

When viewing a boxer's preview sheet:
- **Export Layout**: Export just that boxer's layout configuration
- **Apply Pack**: Apply an installed layout pack to the current boxer

## Creating Layout Packs

### Method 1: Export from Editor

1. Navigate to the boxer you want to export
2. Click "Export Layout" in the Boxer Preview Sheet
3. Fill in pack metadata (name, author, description)
4. Save the JSON file

### Method 2: Manual Creation

1. Copy the example template from `example_hd_layouts.json`
2. Modify the fields to match your layout configuration
3. Validate the JSON structure
4. Import into the editor

## Sharing Layout Packs

To share your layout pack with the community:

1. Export your layout pack from the editor
2. Share the JSON file on forums, Discord, or other community platforms
3. Include a description of what your pack does
4. Note any ROM requirements or dependencies

## Validation

When importing a layout pack, the editor performs these validations:

- **Version Check**: Ensures pack version compatibility
- **Boxer Verification**: Confirms boxers exist in the manifest
- **Bin Validation**: Checks that bins exist at expected offsets
- **Size Check**: Warns if bin sizes differ from current ROM

## Safety Notes

- Always back up your ROM before applying layout packs
- Layout packs modify pending writes - save your project after applying
- Some packs may be designed for specific ROM versions
- Shared banks affect multiple boxers - be aware of dependencies

## Troubleshooting

**Pack import fails validation:**
- Check that the pack version matches the editor version
- Verify boxers in the pack exist in your manifest
- Check that bin offsets and sizes match your ROM

**Layout doesn't apply correctly:**
- Ensure the ROM is loaded before applying packs
- Check the boxer key matches exactly
- Look for validation warnings in the preview dialog

**Pack doesn't appear in list:**
- Verify the JSON file is in the `community/` directory
- Check that the file has a `.json` extension
- Ensure the JSON is valid (no syntax errors)
