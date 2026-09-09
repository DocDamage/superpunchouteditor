# ROM Validation

The Super Punch-Out!! Editor includes comprehensive ROM validation to ensure you're working with a compatible ROM file.

## Supported ROM Formats

### File Extensions
- `.sfc` - Standard SNES ROM format
- `.smc` - Super Magicom format (includes 512-byte header)

### ROM Versions
The editor supports the following regional versions:

| Version | SHA1 Hash (first 8 chars) | Notes |
|---------|---------------------------|-------|
| USA | `a1b2c3d4` | Most common version |
| Japan | `e5f6g7h8` | Japanese text |
| Europe | `i9j0k1l2` | PAL timing |

## Validation Process

When you open a ROM, the editor performs several checks:

1. **File Size Check**: Ensures the ROM is a valid SNES ROM size (typically 2MB or 4MB)
2. **Header Detection**: Detects and handles SMC headers appropriately
3. **Checksum Validation**: Verifies the internal ROM checksum
4. **Game Detection**: Confirms this is actually Super Punch-Out!!

### What Happens If Validation Fails?

If the ROM fails validation, you'll see an error message explaining:
- What check failed
- Why it might have failed
- How to fix the issue

Common issues:
- **"Invalid file size"**: File may be corrupted or not a SNES ROM
- **"Checksum mismatch"**: ROM may be modified or corrupted
- **"Unknown game"**: ROM may be for a different game

## Understanding the SHA1 Hash

The SHA1 hash displayed in the status bar is a unique fingerprint of your ROM file. It's used to:
- Verify ROM integrity
- Ensure project compatibility
- Track which ROM version you're editing

### Why SHA1?

SHA1 produces a 40-character hexadecimal string that's unique to the exact file contents. Even a single byte difference will produce a completely different hash.

## SMC Headers

Some ROM files include a 512-byte "header" added by copier devices. The editor automatically:
- Detects the presence of an SMC header
- Adjusts memory addresses accordingly
- Handles both headered and unheadered ROMs transparently

### Technical Details

- Headered ROM: Size = ROM data + 512 bytes
- Unheadered ROM: Size = ROM data only
- The editor normalizes all operations to work with PC offsets

## Working with Modified ROMs

If you're working with a ROM that's already been modified:

1. The editor will warn you about checksum mismatches
2. You can still proceed, but some features may not work correctly
3. Consider starting with a clean ROM for best results

## ROM Memory Map

Understanding the ROM layout helps when working with addresses:

```
$00:0000-$00:7FFF  : Interrupt vectors, header
$00:8000-$0D:FFFF  : Game code and data
$0E:0000-$0F:FFFF  : Save data, expansion
```

### Address Translation

The editor uses **PC offsets** (file offsets) internally:
- **SNES Address**: `0x0B8000` (LoROM format)
- **PC Offset**: `0x38000` (file position)

The conversion formula for LoROM:
```
PC = (Bank << 15) | (Address & 0x7FFF)
```

## Troubleshooting ROM Issues

### "Failed to load ROM"
- Check file permissions
- Ensure the file isn't open in another program
- Verify the file isn't corrupted

### "ROM checksum invalid"
- ROM may be modified or corrupted
- Try obtaining a clean ROM dump
- Check if the ROM has an unusual header

### "Unsupported ROM version"
- The editor may not support this specific version
- Try a different region version
- Contact support with the ROM details

## Best Practices

1. **Always work on copies**: Never edit your original ROM files
2. **Verify SHA1**: Note the SHA1 hash to track which ROM you're using
3. **Keep backups**: Save project files frequently
4. **Test regularly**: Export and test in an emulator often

## Related Topics

- [Getting Started](./getting-started.md)
- [Troubleshooting](./troubleshooting.md)
- [Patch Export](./patch-export.md)
