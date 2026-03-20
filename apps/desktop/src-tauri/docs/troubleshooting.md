# Troubleshooting

This guide helps resolve common issues when using the Super Punch-Out!! Editor.

## ROM Issues

### "Failed to load ROM"

**Symptoms:**
- Error dialog when opening ROM
- ROM doesn't appear in list
- Application freezes

**Solutions:**
1. Check file permissions - ensure you have read access
2. Verify file isn't open in another program
3. Try a different ROM file
4. Check file extension (.sfc or .smc)

### "ROM validation failed"

**Symptoms:**
- SHA1 mismatch error
- Checksum error
- "Unsupported ROM version"

**Solutions:**
1. Ensure you have a clean, unmodified ROM
2. Try a different region version (USA recommended)
3. Remove any existing patches
4. Check ROM isn't corrupted (compare SHA1)

### "ROM appears corrupted"

**Symptoms:**
- Garbled graphics
- Incorrect colors
- Game crashes when played

**Solutions:**
1. Restore from original backup
2. Re-apply patches one by one
3. Check for SMC header issues
4. Verify file wasn't truncated

## Graphics Issues

### "Palette changes not showing"

**Symptoms:**
- Colors look the same after editing
- Preview doesn't update
- In-game colors unchanged

**Solutions:**
1. Click **"Apply"** after making changes
2. Check you're editing the correct palette
3. Verify palette is loaded in current pose
4. Reload fighter data

### "Sprites appear garbled"

**Symptoms:**
- Random pixels instead of sprites
- Tiles in wrong order
- Graphics corruption

**Solutions:**
1. Check tile ID references
2. Verify correct sprite bin loaded
3. Ensure decompression worked (for compressed bins)
4. Check for edit overflow (too many tiles)

### "Import fails or looks wrong"

**Symptoms:**
- PNG import produces garbage
- Colors don't match
- Wrong tiles imported

**Solutions:**
1. Ensure PNG uses correct palette
2. Check dimensions are multiples of 8
3. Verify color 0 is transparent
4. Try quantizing to 16 colors first

## Performance Issues

### "Editor is slow/laggy"

**Symptoms:**
- Delayed response to clicks
- Slow preview updates
- High CPU usage

**Solutions:**
1. Close other applications
2. Reduce preview quality in settings
3. Work with fewer tiles visible
4. Restart the editor

### "Out of memory error"

**Symptoms:**
- Crash during large operations
- "Memory allocation failed" error
- System becomes unresponsive

**Solutions:**
1. Close other programs
2. Work with smaller batches
3. Save and restart editor
4. Increase system virtual memory

### "Long load times"

**Symptoms:**
- ROM takes forever to load
- Fighter selection is slow
- Preview generation delayed

**Solutions:**
1. Move ROM to faster storage (SSD)
2. Disable real-time preview
3. Reduce animation complexity
4. Clear cache directory

## Save/Export Issues

### "Failed to save project"

**Symptoms:**
- Save operation fails
- Project file not created
- Error writing to disk

**Solutions:**
1. Check disk space
2. Verify write permissions
3. Try different save location
4. Check for invalid characters in filename

### "Patch export fails"

**Symptoms:**
- IPS/BPS creation error
- Empty or invalid patch file
- Export hangs

**Solutions:**
1. Ensure original ROM is still available
2. Check pending writes exist
3. Verify output directory is writable
4. Try smaller export scope

### "Changes lost after closing"

**Symptoms:**
- Edits disappear
- Project file empty
- ROM unchanged

**Solutions:**
1. Always use **"Save Project"** before closing
2. Check auto-save is enabled in settings
3. Verify project saved to expected location
4. Keep backups of important work

## Editor-Specific Issues

### "Frame reconstructor crashes"

**Symptoms:**
- Editor closes unexpectedly
- White/blank canvas
- Can't add sprites

**Solutions:**
1. Reduce number of sprites
2. Check for invalid tile IDs
3. Reload fighter data
4. Update graphics drivers

### "Animation won't play"

**Symptoms:**
- Preview stuck on first frame
- No motion visible
- Timeline not advancing

**Solutions:**
1. Check frame durations (not zero)
2. Verify animation not paused
3. Ensure frames have valid poses
4. Try different playback speed

### "Script changes not applied"

**Symptoms:**
- Stats unchanged in-game
- AI behaves same as before
- Header modifications lost

**Solutions:**
1. Click **"Apply Changes"**
2. Check pending writes list
3. Verify editing correct fighter
4. Reload ROM and reapply

## Compatibility Issues

### "Patch doesn't work on [emulator]"

**Symptoms:**
- Works in one emulator, not another
- Graphics glitches in specific emu
- Audio issues

**Solutions:**
1. Test on multiple emulators
2. Use accurate emulators (bsnes, Mesen-S)
3. Check emulator-specific settings
4. Report to emulator developers

### "Works in emulator but not on real hardware"

**Symptoms:**
- Functions on emulator
- Fails on SNES/flash cart
- Graphical corruption on hardware

**Solutions:**
1. Test on accurate emulators first
2. Check for out-of-bounds memory access
3. Verify proper SNES timing
4. Consult SNES development resources

### "Incompatible with other patches"

**Symptoms:**
- Patches work individually
- Combined patches cause issues
- Conflicts with other mods

**Solutions:**
1. Apply patches in different order
2. Check for overlapping changes
3. Use conflict resolution tools
4. Manually merge changes

## Getting Help

### Before Asking

Do this first:
1. Search this documentation
2. Check the FAQ
3. Try the troubleshooting steps above
4. Update to latest editor version

### Information to Provide

When seeking help, include:
- Editor version
- Operating system
- ROM version (first 8 chars of SHA1)
- Exact error message
- Steps to reproduce
- Screenshots if applicable

### Where to Ask

Support channels:
- **GitHub Issues**: Bug reports
- **Discord**: Real-time help
- **Romhacking.net Forums**: General discussion
- **Reddit r/romhacking**: Community support

## Diagnostic Tools

### Built-in Diagnostics

Access via **Help > Diagnostics**:
- ROM information
- System information
- Log viewer
- Performance metrics

### Log Files

Find logs at:
- Windows: `%APPDATA%\SuperPunchOutEditor\logs`
- macOS: `~/Library/Logs/SuperPunchOutEditor`
- Linux: `~/.local/share/SuperPunchOutEditor/logs`

### Debug Mode

Enable advanced logging:
1. Go to **Settings**
2. Enable **"Debug Mode"**
3. Restart editor
4. Reproduce issue
5. Check logs

## Recovery

### Restoring from Backup

If corruption occurs:
1. Close editor
2. Locate backup (`.spo.backup`)
3. Rename to remove `.backup`
4. Reopen

### Starting Fresh

Nuclear option:
1. Export any salvageable work
2. Clear editor cache
3. Reinstall editor
4. Start new project

## Related Topics

- [Getting Started](./getting-started.md)
- [ROM Validation](./rom-validation.md)
- [Patch Export](./patch-export.md)
