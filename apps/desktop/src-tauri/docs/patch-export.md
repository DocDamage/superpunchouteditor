# Patch Export

Patch export allows you to share your modifications with others without distributing copyrighted ROM files.

## Understanding Patches

### What is a Patch?

A patch file contains only the differences between:
- **Original ROM**: Unmodified game
- **Modified ROM**: Your edited version

### Why Use Patches?

Legal and practical benefits:
- **Legal**: Don't distribute copyrighted ROMs
- **Small size**: Only changed bytes (KB vs MB)
- **Universal**: Works with any compatible ROM
- **Reversible**: Can be removed/applied at will

## Supported Patch Formats

### IPS (International Patching System)

The most common format:
- **File extension**: `.ips`
- **Limitations**: 16MB max ROM size, no checksum
- **Compatibility**: Supported by most emulators

### IPS Creation

1. Make your edits in the editor
2. Go to **Export** section
3. Click **"Export IPS Patch"**
4. Choose save location
5. Name your patch

### BPS (Binary Patch System)

Modern alternative:
- **File extension**: `.bps`
- **Advantages**: Checksum validation, better compression
- **Support**: Growing emulator support

## Export Options

### Full Export

Export all pending changes:
- Includes every modification
- Largest file size
- Complete mod package

### Selective Export

Choose specific changes:
1. Open **Export Manager**
2. Review list of changes
3. Check/uncheck items
4. Export selected only

### Fighter-Specific

Export changes for one fighter:
1. Select fighter
2. Click **"Export Fighter Patch"**
3. Choose format
4. Save patch file

## The Export Panel

### Pending Changes List

Review before export:
- **Asset type**: Palette, sprite, script, etc.
- **Offset**: Memory location
- **Size**: Number of bytes changed
- **Description**: Human-readable summary

### Change Details

Inspect individual changes:
- **Before/After**: Hex comparison
- **Visual diff**: For graphics
- **Impact**: What this affects

### Batch Operations

Manage multiple changes:
- **Select All**: Include everything
- **Deselect All**: Start fresh
- **Invert**: Toggle selections
- **Filter**: By type or fighter

## Patch Metadata

### Adding Information

Include with your patch:
- **Author name**: Your credit
- **Version**: Patch version number
- **Description**: What the patch does
- **Requirements**: Base ROM needed
- **Changelog**: Version history

### README Generation

Auto-generate documentation:
1. Fill in metadata fields
2. Click **"Generate README"**
3. Review and edit
4. Save with patch

## Distribution

### Packaging

Create a distribution package:
```
MyMod_v1.0/
  ├── mymod.ips
  ├── README.txt
  ├── screenshots/
  │   ├── before.png
  │   └── after.png
  └── instructions.md
```

### Sharing Platforms

Where to share patches:
- **ROM hacking forums**: Romhacking.net
- **Discord communities**: SPO!! modding servers
- **GitHub**: Version control and releases
- **Personal websites**: Direct download

### Patch Hosting

Upload to archives:
1. Create account on Romhacking.net
2. Submit patch for review
3. Include screenshots
4. Add detailed description

## Applying Patches

### Using the Editor

Apply patches within the editor:
1. Go to **Tools** menu
2. Click **"Apply Patch"**
3. Select patch file
4. Choose target ROM
5. Verify result

### External Tools

Recommended patchers:
- **FLIPS**: Floating IPS (Windows/Linux)
- **Lunar IPS**: Classic IPS tool (Windows)
- **Beat**: BPS patcher (multi-platform)
- **MultiPatch**: macOS patcher

### Emulator Support

Many emulators support patches:
- **snes9x**: Auto-load `.ips` files
- **bsnes/higan**: IPS and BPS support
- **Mesen-S**: Full patch support

## Verification

### Checksum Validation

Verify patch integrity:
- **Source ROM**: Ensure correct original
- **Patched ROM**: Confirm success
- **SHA1 display**: Compare hashes

### Testing Checklist

Before releasing:
- [ ] Apply to clean ROM
- [ ] Test in emulator
- [ ] Verify all changes present
- [ ] Check no corruption
- [ ] Test on real hardware (if possible)

## Version Management

### Semantic Versioning

Use standard versioning:
- **MAJOR**: Significant changes, incompatibilities
- **MINOR**: New features, backwards compatible
- **PATCH**: Bug fixes only

Example: `v1.2.3`

### Update Patches

Create update patches:
1. Export changes since last version
2. Name appropriately (`MyMod_v1.0_to_v1.1.ips`)
3. Include upgrade instructions
4. Test update path

## Troubleshooting

### "Patch doesn't apply"
- Check ROM version matches
- Verify ROM isn't already modified
- Try different patcher tool

### "Game crashes after patch"
- ROM may be wrong version
- Patch may be corrupted
- Re-apply to clean ROM

### "Changes not appearing"
- Patch may not include those changes
- Check if emulator has patch caching
- Verify correct file loaded

## Best Practices

### Creation

1. **Work from clean ROM**: Always start fresh
2. **Document changes**: Track what you modify
3. **Test thoroughly**: Before releasing
4. **Version properly**: Use semantic versioning

### Distribution

1. **Clear instructions**: How to apply
2. **Screenshots**: Show changes
3. **Requirements**: List dependencies
4. **Support info**: Where to get help

## Related Topics

- [Getting Started](./getting-started.md)
- [Troubleshooting](./troubleshooting.md)
- [Project Management](./getting-started.md#projects)
