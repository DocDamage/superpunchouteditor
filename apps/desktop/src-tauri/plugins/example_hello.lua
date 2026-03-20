-- Super Punch-Out!! Editor - Hello World Example Plugin
-- Demonstrates basic plugin structure and event handling

PLUGIN_INFO = {
    id = "example_hello",
    name = "Hello World",
    version = "1.0.0",
    author = "Super Punch-Out!! Editor",
    description = "A simple example plugin that demonstrates the plugin system",
    api_version = 1,
}

function on_init()
    SPO.log_info("=" .. string.rep("=", 50))
    SPO.log_info("Hello World plugin loaded!")
    SPO.log_info("This is an example of how to create plugins.")
    SPO.log_info("=" .. string.rep("=", 50))
    
    SPO.notify_success("Hello World plugin is ready!")
end

function on_shutdown()
    SPO.log_info("Hello World plugin shutting down...")
end

function on_rom_loaded()
    local size = SPO.rom_size()
    SPO.log_info("ROM loaded! Size: " .. size .. " bytes")
    SPO.notify_info("Hello World detected a ROM with " .. (size / 1024) .. " KB")
end

function on_asset_modified()
    SPO.log_debug("An asset was modified")
end

-- Define a custom command that can be called from the UI
COMMANDS = {
    say_hello = function(args)
        local name = args and args.name or "World"
        local message = "Hello, " .. name .. "!"
        
        SPO.log_info(message)
        SPO.notify_success(message)
        
        return {
            success = true,
            message = message,
            timestamp = os.time()
        }
    end,
    
    get_rom_info = function(args)
        local size = SPO.rom_size()
        
        return {
            success = true,
            rom_size = size,
            rom_size_kb = math.floor(size / 1024 * 100) / 100,
            plugin_version = PLUGIN_INFO.version
        }
    end
}
