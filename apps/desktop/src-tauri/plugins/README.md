# Super Punch-Out!! Editor Plugins

This directory contains Lua plugins for the Super Punch-Out!! Editor.

## Installing Plugins

1. Copy `.lua` plugin files to this directory
2. Restart the editor, or use "Reload Plugins" from the Tools menu
3. Plugins will be automatically loaded and enabled

## Creating Plugins

Create a new `.lua` file with the following structure:

```lua
-- PLUGIN_INFO table (required)
PLUGIN_INFO = {
    id = "my_plugin",           -- Unique identifier (required)
    name = "My Plugin",         -- Display name
    version = "1.0.0",          -- Version string
    author = "Your Name",       -- Author name
    description = "Does cool stuff",
    api_version = 1,            -- API version (currently 1)
}

-- Called when the plugin is loaded
function on_init()
    SPO.log_info("My Plugin loaded!")
end

-- Called when the plugin is unloaded
function on_shutdown()
    SPO.log_info("My Plugin shutting down...")
end

-- Event handlers (optional)
function on_rom_loaded()
    SPO.log_info("ROM was loaded!")
end

function on_asset_modified()
    SPO.notify_info("Asset was modified")
end

-- Custom commands (optional)
COMMANDS = {
    my_command = function(args)
        SPO.log_info("Running my_command with args: " .. tostring(args))
        return { success = true }
    end
}
```

## API Reference

### ROM Operations

- `SPO.rom_read(offset, length)` - Read bytes from ROM
- `SPO.rom_write(offset, data)` - Write bytes to ROM
- `SPO.rom_read_byte(offset)` - Read single byte
- `SPO.rom_write_byte(offset, value)` - Write single byte
- `SPO.rom_size()` - Get ROM size in bytes

### Address Conversion

- `SPO.snes_to_pc(bank, addr)` - Convert SNES LoROM address to PC offset
- `SPO.pc_to_snes(pc)` - Convert PC offset to SNES LoROM address

### Utility

- `SPO.find_pattern(pattern)` - Search for byte pattern in ROM

### Logging

- `SPO.log_info(message)` - Log info message
- `SPO.log_debug(message)` - Log debug message
- `SPO.log_warn(message)` - Log warning message
- `SPO.log_error(message)` - Log error message

### Notifications

- `SPO.notify_info(message)` - Show info notification
- `SPO.notify_success(message)` - Show success notification
- `SPO.notify_error(message)` - Show error notification

## Example Plugins

See the included example plugins:
- `example_hello.lua` - Basic plugin structure
- `example_rom_stats.lua` - ROM analysis plugin
- `example_batch_export.lua` - Batch export functionality
