# V4 "Full Power Layer" Quick Start Guide

## 🚀 Getting Started

### Prerequisites
- Rust 1.89.0+
- Node.js 18+
- Super Punch-Out!! (USA) ROM

### Building
```bash
# Build the entire workspace
cargo build --workspace --release

# Build just the Tauri app
cargo build -p tauri-appsuper-punch-out-editor --release
```

### Running the Editor
```bash
# Run in development mode
cargo tauri dev

# Or run the built binary
./target/release/tauri-appsuper-punch-out-editor.exe
```

---

## 🔌 Using Plugins

### Loading Your First Plugin

1. **Open the Plugin Manager**
   - Press `Ctrl+7` or click the "Plugins" tab

2. **Load a Plugin**
   - Click "Load Plugin" button
   - Select one of the example plugins:
     - `example_hello.lua` - Simple demo
     - `example_rom_stats.lua` - ROM statistics
     - `v4_feature_demo.lua` - Comprehensive API demo

3. **Run the Plugin**
   - The plugin appears in the list
   - Toggle it on/off with the switch
   - Click on the plugin to see available commands
   - Execute commands with the "Execute" button

### Creating a Custom Plugin

Create a file `my_plugin.lua`:

```lua
PLUGIN_INFO = {
    id = "my_plugin",
    name = "My First Plugin",
    version = "1.0.0",
    author = "Your Name",
    description = "My custom plugin",
    api_version = 1,
}

function on_init()
    SPO.log_info("My plugin is ready!")
    SPO.notify_success("Plugin loaded successfully")
end

function on_rom_loaded()
    local size = SPO.rom_size()
    SPO.log_info("ROM loaded: " .. size .. " bytes")
end

COMMANDS = {
    -- Read a byte from ROM
    read_byte = function(args)
        local offset = tonumber(args.offset) or 0
        local byte = SPO.rom_read_byte(offset)
        return {
            success = true,
            offset = offset,
            value = byte,
            hex = string.format("%02X", byte)
        }
    end,
    
    -- Search for a pattern
    find_pattern = function(args)
        local pattern = args.pattern or {0x00, 0x00}
        local results = SPO.find_pattern(pattern)
        return {
            success = true,
            found = #results,
            locations = results
        }
    end
}
```

Load it in the Plugin Manager and try:
- `read_byte` with offset `0x100`
- `find_pattern` with pattern `[0x53, 0x4E, 0x45, 0x53]` ("SNES")

---

## 🗺️ Bank Visualization

### Viewing ROM Layout

1. **Open Bank Map**
   - Press `Ctrl+8` or click the "Bank Map" tab

2. **Understanding Colors**
   - 🟫 Dark Gray - Free space
   - 🟧 Coral - Compressed graphics
   - 🟥 Red - Uncompressed graphics
   - 🟨 Yellow - Palette data
   - 🟪 Purple - Audio data
   - 🟦 Cyan - Code
   - ⬜ Light Gray - Text

3. **Interacting with Banks**
   - Hover over banks for details
   - Click for detailed breakdown
   - Use the search to find specific regions

### Defragmentation

1. Click **"Analyze Fragmentation"**
   - View fragmentation score
   - See free space gaps
   - Check movable regions

2. Click **"Generate Plan"**
   - Review safety rating
   - Check estimated space savings

3. Click **"Execute Defragmentation"**
   - Confirm the operation
   - Wait for completion

---

## 🎬 Animation Player

### Playing Animations

1. **Open Animation Player**
   - Press `Ctrl+9` or click the "Animation Player" tab

2. **Select Animation**
   - Choose a boxer from the dropdown (e.g., "Gabby Jay")
   - Select animation type (e.g., "idle", "punch_left")
   - Click "Load"

3. **Playback Controls**
   - `Space` - Play/Pause
   - `←/→` - Previous/Next frame
   - `Home` - First frame
   - `End` - Last frame
   - Speed control (0.25x to 2x)

### Editing Hitboxes

1. **Enable Hitbox Display**
   - Click "Show Hitboxes" toggle
   - Hitboxes appear on the canvas

2. **Select a Hitbox**
   - Click on a hitbox in the list
   - Or click directly on the canvas

3. **Edit Properties**
   - Change type (jab, hook, uppercut, special)
   - Adjust position (X, Y)
   - Change size (Width, Height)
   - Set damage, hitstun, knockback

4. **Visual Editing**
   - Drag hitboxes on the canvas
   - Use resize handles
   - Press "Update" to save

5. **Add/Remove Hitboxes**
   - "Add Hitbox" creates a new one
   - "Remove" deletes the selected hitbox

---

## 📝 Running Lua Scripts

### Quick Scripts

1. **Open Plugin Manager** (Ctrl+7)
2. Go to the **Script Runner** section
3. Enter Lua code:

```lua
-- Read ROM header
local header = SPO.rom_read(0x7FC0, 32)
local title = ""
for i = 1, #header do
    local c = string.char(header:byte(i))
    if c:match("[%w%s]") then
        title = title .. c
    end
end
SPO.log_info("ROM Title: " .. title)
return { title = title }
```

4. Click **"Run Script"**

### Batch Operations

```lua
-- Example: Analyze all banks
for bank = 0, 63 do
    local addr = bank * 0x8000
    local data = SPO.rom_read(addr, 0x8000)
    
    -- Count zero bytes
    local zeros = 0
    for i = 1, #data do
        if data:byte(i) == 0 then
            zeros = zeros + 1
        end
    end
    
    local usage = (0x8000 - zeros) / 0x8000 * 100
    SPO.log_info(string.format("Bank %02X: %.1f%% used", bank, usage))
end
```

---

## 🎓 Tutorial: Creating a Palette Editor Plugin

### Step 1: Create the File

Create `palette_editor.lua`:

```lua
PLUGIN_INFO = {
    id = "palette_editor",
    name = "Palette Editor",
    version = "1.0.0",
    author = "Your Name",
    description = "Edit SNES palettes",
    api_version = 1,
}

function on_init()
    SPO.log_info("Palette Editor loaded")
end

-- Convert BGR555 to RGB
local function bgr555_to_rgb(bgr)
    local b = (bgr >> 10) & 0x1F
    local g = (bgr >> 5) & 0x1F
    local r = bgr & 0x1F
    -- Extend 5-bit to 8-bit
    return {
        r = r << 3,
        g = g << 3,
        b = b << 3
    }
end

-- Convert RGB to BGR555
local function rgb_to_bgr555(r, g, b)
    local r5 = (r >> 3) & 0x1F
    local g5 = (g >> 3) & 0x1F
    local b5 = (b >> 3) & 0x1F
    return b5 << 10 | g5 << 5 | r5
end

COMMANDS = {
    -- Read palette at offset
    read_palette = function(args)
        local offset = tonumber(args.offset) or 0
        local count = tonumber(args.count) or 16
        
        local colors = {}
        for i = 0, count - 1 do
            local addr = offset + (i * 2)
            local lo = SPO.rom_read_byte(addr)
            local hi = SPO.rom_read_byte(addr + 1)
            local bgr = lo | (hi << 8)
            local rgb = bgr555_to_rgb(bgr)
            table.insert(colors, {
                index = i,
                bgr = bgr,
                r = rgb.r,
                g = rgb.g,
                b = rgb.b,
                hex = string.format("#%02X%02X%02X", rgb.r, rgb.g, rgb.b)
            })
        end
        
        return {
            success = true,
            offset = offset,
            count = count,
            colors = colors
        }
    end,
    
    -- Write color to palette
    write_color = function(args)
        local offset = tonumber(args.offset) or 0
        local index = tonumber(args.index) or 0
        local r = tonumber(args.r) or 0
        local g = tonumber(args.g) or 0
        local b = tonumber(args.b) or 0
        
        local bgr = rgb_to_bgr555(r, g, b)
        local addr = offset + (index * 2)
        
        -- Write low byte
        SPO.rom_write_byte(addr, bgr & 0xFF)
        -- Write high byte
        SPO.rom_write_byte(addr + 1, (bgr >> 8) & 0xFF)
        
        SPO.notify_success("Color " .. index .. " updated")
        return { success = true }
    end
}
```

### Step 2: Load and Test

1. Open Plugin Manager (Ctrl+7)
2. Load `palette_editor.lua`
3. Select the plugin to see available commands

### Step 3: Use the Commands

**Read a palette:**
- Command: `read_palette`
- Args: `{"offset": "0x1C8000", "count": 16}`

**Write a color:**
- Command: `write_color`
- Args: `{"offset": "0x1C8000", "index": 0, "r": 255, "g": 0, "b": 0}`

---

## 🔧 Troubleshooting

### Plugin Won't Load
- Check `api_version` matches (currently 1)
- Verify `PLUGIN_INFO` has all required fields
- Check the Lua syntax is valid
- Look at the log output for errors

### Animation Player Not Working
- Ensure a ROM is loaded
- Check that the boxer data exists in the manifest
- Verify the animation name is correct

### Bank Map Shows Empty
- Make sure a ROM is loaded
- Try clicking "Refresh" if available

### Script Errors
- Use `SPO.log_debug()` for debugging
- Check argument types (numbers vs strings)
- Wrap in `pcall()` for error handling:

```lua
local ok, result = pcall(function()
    -- your code here
end)
if not ok then
    SPO.log_error("Error: " .. tostring(result))
end
```

---

## 📚 Additional Resources

- **Plugin API Docs**: See `apps/desktop/src-tauri/plugins/README.md`
- **Example Plugins**: Check `apps/desktop/src-tauri/plugins/`
- **Source Code**: Browse `crates/plugin-core/src/` for implementation details

---

Happy modding! 🥊
