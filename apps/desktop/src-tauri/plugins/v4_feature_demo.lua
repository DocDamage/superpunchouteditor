-- ============================================================================
-- Super Punch-Out!! Editor - V4 Feature Demo Plugin
-- ============================================================================
-- This plugin serves as a comprehensive demonstration and documentation of all
-- V4 plugin system features. It showcases best practices for plugin development
-- and provides interactive examples of every API capability.
--
-- PURPOSE:
--   - Demonstrate all V4 plugin system features
--   - Serve as living documentation for plugin developers
--   - Provide test coverage for plugin API functionality
--   - Show best practices and error handling patterns
--
-- SAFETY:
--   - All ROM write operations are isolated in specific demo functions
--   - Write demos create backups before modification
--   - Read-only operations are safe to run at any time
--
-- USAGE:
--   - Call individual demo_* functions to see specific features
--   - Call run_full_demo() for an automated walkthrough of all features
--   - Check get_system_info() for current editor and ROM information
-- ============================================================================

-- ============================================================================
-- PLUGIN METADATA
-- ============================================================================
PLUGIN_INFO = {
    id = "v4_feature_demo",
    name = "V4 Feature Demo",
    version = "1.0.0",
    author = "Super Punch-Out!! Editor Team",
    description = "Comprehensive demonstration of all V4 plugin system features.",
    api_version = 4,
    category = "Development",
    tags = { "demo", "documentation", "api", "tutorial" },
}

-- ============================================================================
-- PLUGIN STATE MANAGEMENT
-- ============================================================================
local plugin_state = {
    demo_run_count = 0,
    last_results = nil,
    is_running = false,
    backup_data = {},
    timings = {},
}

-- ============================================================================
-- UTILITY FUNCTIONS
-- ============================================================================
local function format_hex_byte(value)
    return string.format("0x%02X", value & 0xFF)
end

local function format_address(pc_addr)
    local snes_addr = SPO.pc_to_snes(pc_addr)
    return string.format("PC:$%06X / SNES:$%06X", pc_addr, snes_addr)
end

local function create_progress_callback(operation_name)
    return function(percent, message)
        local status_msg = string.format("[%s] %d%%: %s", operation_name, percent, message or "")
        SPO.log_debug(status_msg)
    end
end

local function safe_execute(fn, error_prefix, ...)
    local success, result = pcall(fn, ...)
    if not success then
        local error_msg = error_prefix .. ": " .. tostring(result)
        SPO.log_error(error_msg)
        SPO.notify_error(error_msg)
        return false, error_msg
    end
    return true, result
end

local function hex_dump(data, offset, length)
    offset = offset or 0
    length = length or math.min(#data, 64)
    local lines = {}
    for i = 1, length, 16 do
        local line_offset = offset + i - 1
        local hex_part = {}
        local ascii_part = {}
        for j = 0, 15 do
            local idx = i + j
            if idx <= #data then
                local byte = string.byte(data, idx)
                table.insert(hex_part, string.format("%02X", byte))
                if byte >= 32 and byte <= 126 then
                    table.insert(ascii_part, string.char(byte))
                else
                    table.insert(ascii_part, ".")
                end
            else
                table.insert(hex_part, "  ")
                table.insert(ascii_part, " ")
            end
        end
        table.insert(lines, string.format("%06X: %s | %s",
            line_offset,
            table.concat(hex_part, " "),
            table.concat(ascii_part)
        ))
    end
    return table.concat(lines, "\n")
end

-- ============================================================================
-- PLUGIN LIFECYCLE EVENTS
-- ============================================================================
function on_init()
    SPO.log_info("")
    SPO.log_info("=" .. string.rep("=", 50))
    SPO.log_info("V4 FEATURE DEMO PLUGIN")
    SPO.log_info("Version " .. PLUGIN_INFO.version .. " | API v" .. PLUGIN_INFO.api_version)
    SPO.log_info("=" .. string.rep("=", 50))
    SPO.log_info("")
    SPO.log_info("This plugin demonstrates all V4 plugin system features.")
    SPO.log_info("Available commands:")
    SPO.log_info("  run_full_demo()      - Run all demos automatically")
    SPO.log_info("  demo_rom_access()    - ROM read/write operations")
    SPO.log_info("  demo_notifications() - All notification types")
    SPO.log_info("  demo_logging()       - All log levels")
    SPO.log_info("  demo_batch_ops()     - Batch operations with progress")
    SPO.log_info("  demo_integration()   - Editor integration features")
    SPO.log_info("  demo_address_conv()  - Address conversion utilities")
    SPO.log_info("  demo_pattern_search()- Pattern searching")
    SPO.log_info("  get_system_info()    - Get editor/ROM information")
    SPO.log_info("")
    plugin_state.demo_run_count = 0
    plugin_state.last_results = nil
    plugin_state.is_running = false
    plugin_state.backup_data = {}
    plugin_state.timings = {}
    SPO.notify_success("V4 Feature Demo loaded! Try: run_full_demo()")
end

function on_shutdown()
    SPO.log_info("V4 Feature Demo plugin shutting down...")
    SPO.log_info(string.format("Demos run this session: %d", plugin_state.demo_run_count))
    plugin_state.backup_data = {}
end

function on_rom_loaded()
    SPO.log_info("ROM loaded event received")
    local rom_size = SPO.rom_size()
    SPO.log_info(string.format("  ROM size: %d bytes (%.2f KB)", rom_size, rom_size / 1024))
    plugin_state.backup_data = {}
    plugin_state.last_results = nil
    if rom_size >= 0x8000 then
        local header_data = SPO.rom_read(0x7FC0, 21)
        SPO.log_info("  ROM header: " .. header_data)
    end
end

function on_asset_modified()
    SPO.log_debug("Asset modified event received")
end


-- ============================================================================
-- DEMO: ROM ACCESS OPERATIONS
-- ============================================================================
local function demonstrate_rom_reads()
    SPO.log_info("")
    SPO.log_info("--- ROM Read Operations ---")
    local results = { read_block = nil, read_byte = nil, header_data = nil }
    
    local success, header_data = safe_execute(function()
        return SPO.rom_read(0x7FC0, 32)
    end, "ROM read failed")
    
    if success then
        results.read_block = header_data
        SPO.log_info(string.format("OK Read %d bytes from header region", #header_data))
        SPO.log_info("  Hex dump:")
        for line in hex_dump(header_data, 0x7FC0, 32):gmatch("[^\n]+") do
            SPO.log_info("  " .. line)
        end
    end
    
    local success, byte1 = safe_execute(function()
        return SPO.rom_read_byte(0x7FC0)
    end, "Byte read failed")
    
    if success then
        results.read_byte = byte1
        SPO.log_info(string.format("OK Read individual byte at $7FC0: %s", format_hex_byte(byte1)))
    end
    
    if SPO.rom_size() >= 0x10000 then
        local vectors = safe_execute(function()
            local reset_low = SPO.rom_read_byte(0x7FFC)
            local reset_high = SPO.rom_read_byte(0x7FFD)
            return reset_low + (reset_high * 256)
        end, "Vector read failed")
        if vectors then
            SPO.log_info(string.format("OK Reset vector: $%04X", vectors))
        end
    end
    return results
end

local function demonstrate_rom_writes()
    SPO.log_info("")
    SPO.log_info("--- ROM Write Operations (Safe Demo) ---")
    SPO.log_warn("This demo will modify ROM data temporarily!")
    local results = { backup_created = false, write_success = false, restored = false }
    
    local test_offset = 0x100
    local success, original_data = safe_execute(function()
        return SPO.rom_read(test_offset, 4)
    end, "Backup read failed")
    
    if not success then
        SPO.notify_error("Could not create backup - aborting write demo")
        return results
    end
    
    plugin_state.backup_data[test_offset] = original_data
    results.backup_created = true
    SPO.log_info("OK Created backup of 4 bytes at " .. format_address(test_offset))
    
    local write_success = safe_execute(function()
        SPO.rom_write_byte(test_offset, 0xDE)
        SPO.rom_write_byte(test_offset + 1, 0xAD)
        SPO.rom_write_byte(test_offset + 2, 0xBE)
        SPO.rom_write_byte(test_offset + 3, 0xEF)
        return true
    end, "Byte write failed")
    
    if write_success then
        results.write_success = true
        SPO.log_info(string.format("OK Wrote bytes: DE AD BE EF at %s", format_address(test_offset)))
        local verify_data = SPO.rom_read(test_offset, 4)
        local verify_hex = {}
        for i = 1, #verify_data do
            table.insert(verify_hex, string.format("%02X", string.byte(verify_data, i)))
        end
        SPO.log_info("  Verification: " .. table.concat(verify_hex, " "))
    end
    
    SPO.log_info("")
    SPO.log_info("Demonstrating block write...")
    local block_success = safe_execute(function()
        SPO.rom_write(test_offset, "TEST")
        return true
    end, "Block write failed")
    
    if block_success then
        SPO.log_info(string.format("OK Wrote block 'TEST' at %s", format_address(test_offset)))
        local verify = SPO.rom_read(test_offset, 4)
        SPO.log_info("  Verification: " .. verify)
    end
    
    SPO.log_info("")
    SPO.log_info("Restoring original data...")
    local restore_success = safe_execute(function()
        SPO.rom_write(test_offset, original_data)
        return true
    end, "Restore failed")
    
    if restore_success then
        results.restored = true
        SPO.log_info("OK Original data restored")
        plugin_state.backup_data[test_offset] = nil
    end
    return results
end

-- ============================================================================
-- DEMO: PATTERN SEARCHING
-- ============================================================================
local function demonstrate_pattern_search()
    SPO.log_info("")
    SPO.log_info("--- Pattern Searching ---")
    local results = { searches = {} }
    
    local patterns = {
        { name = "Mode 7 Header", pattern = {0x00, 0x00, 0x00, 0x80} },
        { name = "Empty Tile", pattern = {0x00, 0x00, 0x00, 0x00} },
        { name = "FF Fill", pattern = {0xFF, 0xFF, 0xFF, 0xFF} },
        { name = "SPC Header", pattern = {0x53, 0x50, 0x43} },
    }
    
    for _, search in ipairs(patterns) do
        SPO.log_info(string.format("Searching for: %s...", search.name))
        local success, matches = safe_execute(function()
            return SPO.find_pattern(search.pattern)
        end, "Pattern search failed")
        
        if success then
            local count = matches and #matches or 0
            SPO.log_info(string.format("  OK Found %d occurrence(s)", count))
            if matches and #matches > 0 then
                local to_show = math.min(3, #matches)
                for i = 1, to_show do
                    SPO.log_info(string.format("    - Match %d: %s", i, format_address(matches[i])))
                end
                if #matches > 3 then
                    SPO.log_info(string.format("    ... and %d more", #matches - 3))
                end
            end
            table.insert(results.searches, { name = search.name, count = count, matches = matches })
        end
    end
    return results
end

-- ============================================================================
-- DEMO: ADDRESS CONVERSION
-- ============================================================================
local function demonstrate_address_conversion()
    SPO.log_info("")
    SPO.log_info("--- Address Conversion ---")
    local results = { conversions = {} }
    
    local test_cases = {
        { pc = 0x000000, desc = "Start of ROM" },
        { pc = 0x007FC0, desc = "Header location" },
        { pc = 0x008000, desc = "Start of bank 1" },
    }
    
    SPO.log_info("PC to SNES conversion:")
    for _, test in ipairs(test_cases) do
        local success, snes = safe_execute(function()
            return SPO.pc_to_snes(test.pc)
        end, "Conversion failed")
        if success then
            SPO.log_info(string.format("  %s: PC $%06X -> SNES $%06X", test.desc, test.pc, snes))
            table.insert(results.conversions, { direction = "pc_to_snes", pc = test.pc, snes = snes })
        end
    end
    
    SPO.log_info("")
    SPO.log_info("SNES to PC conversion:")
    local snes_tests = {
        { snes = 0x808000, desc = "Bank $80 start" },
        { snes = 0x81FFC0, desc = "Bank $81 header" },
        { snes = 0xBF8000, desc = "Last bank" },
    }
    for _, test in ipairs(snes_tests) do
        local success, pc = safe_execute(function()
            return SPO.snes_to_pc(test.snes)
        end, "Conversion failed")
        if success then
            SPO.log_info(string.format("  %s: SNES $%06X -> PC $%06X", test.desc, test.snes, pc))
            table.insert(results.conversions, { direction = "snes_to_pc", snes = test.snes, pc = pc })
        end
    end
    return results
end

-- ============================================================================
-- DEMO: NOTIFICATIONS
-- ============================================================================
local function demonstrate_notifications()
    SPO.log_info("")
    SPO.log_info("--- Notification Types ---")
    SPO.log_info("Showing all notification types (check UI for visual feedback):")
    local results = { shown = {} }
    
    SPO.notify_info("This is an info notification - general purpose messaging")
    table.insert(results.shown, "info")
    
    SPO.notify_success("This is a success notification - operation completed!")
    table.insert(results.shown, "success")
    
    SPO.notify_warn("This is a warning notification - attention needed")
    table.insert(results.shown, "warn")
    
    SPO.notify_error("This is an error notification - something went wrong!")
    table.insert(results.shown, "error")
    
    SPO.log_info("OK All notification types demonstrated")
    SPO.log_info("  Note: Notifications appear in the editor UI")
    return results
end


-- ============================================================================
-- DEMO: LOGGING
-- ============================================================================
local function demonstrate_logging()
    SPO.log_info("")
    SPO.log_info("--- Logging Levels ---")
    SPO.log_info("Demonstrating all available log levels:")
    local results = { levels_used = {} }
    
    SPO.log_debug("DEBUG: Detailed diagnostic information for developers")
    table.insert(results.levels_used, "debug")
    
    SPO.log_info("INFO: General information about plugin operation")
    table.insert(results.levels_used, "info")
    
    SPO.log_warn("WARN: Warning about potential issues or deprecated features")
    table.insert(results.levels_used, "warn")
    
    SPO.log_error("ERROR: Error message - something failed!")
    table.insert(results.levels_used, "error")
    
    SPO.log_info("")
    SPO.log_info("OK All logging levels demonstrated")
    SPO.log_info("  Note: Debug logs may be filtered based on editor settings")
    return results
end

-- ============================================================================
-- DEMO: BATCH OPERATIONS WITH PROGRESS
-- ============================================================================
local function demonstrate_batch_ops(args)
    args = args or {}
    local iterations = args.iterations or 10
    
    SPO.log_info("")
    SPO.log_info("--- Batch Operations with Progress ---")
    SPO.log_info(string.format("Running batch demo with %d iterations...", iterations))
    
    local results = { iterations_completed = 0, items_processed = {}, total_time_ms = 0 }
    local start_time = os.clock()
    local progress = create_progress_callback("Batch Demo")
    
    for i = 1, iterations do
        local work_start = os.clock()
        local bank = (i - 1) % 8
        local bank_offset = bank * 0x8000
        
        local success, data = safe_execute(function()
            return SPO.rom_read(bank_offset, 16)
        end, "Batch read failed")
        
        if success then
            table.insert(results.items_processed, {
                iteration = i, bank = bank, sample_byte = string.byte(data, 1)
            })
        end
        
        local percent = math.floor((i / iterations) * 100)
        progress(percent, string.format("Processing iteration %d/%d (bank %d)", i, iterations, bank))
        results.iterations_completed = i
    end
    
    results.total_time_ms = math.floor((os.clock() - start_time) * 1000)
    
    SPO.log_info("")
    SPO.log_info(string.format("OK Batch operation complete"))
    SPO.log_info(string.format("  Iterations: %d", results.iterations_completed))
    SPO.log_info(string.format("  Time: %d ms", results.total_time_ms))
    SPO.log_info(string.format("  Avg: %.2f ms/iteration", results.total_time_ms / results.iterations_completed))
    
    return results
end

-- ============================================================================
-- DEMO: EDITOR INTEGRATION
-- ============================================================================
local function demonstrate_integration()
    SPO.log_info("")
    SPO.log_info("--- Editor Integration ---")
    local results = { features_demoed = {} }
    
    SPO.log_info("Getting system information...")
    local sys_info = COMMANDS.get_system_info()
    if sys_info.success then
        SPO.log_info(string.format("  Editor version: %s", sys_info.editor.version))
        SPO.log_info(string.format("  API version: %d", sys_info.editor.api_version))
        SPO.log_info(string.format("  ROM loaded: %s", sys_info.rom.loaded and "Yes" or "No"))
        if sys_info.rom.loaded then
            SPO.log_info(string.format("  ROM size: %d bytes", sys_info.rom.size))
        end
        table.insert(results.features_demoed, "system_info")
    end
    
    SPO.log_info("")
    SPO.log_info("Cross-plugin communication (if rom_analyzer is available):")
    local success, analyzer_result = safe_execute(function()
        return SPO.call_plugin("rom_analyzer", "get_cache_status", {})
    end, "Cross-plugin call failed (rom_analyzer may not be loaded)")
    
    if success and analyzer_result then
        SPO.log_info("  OK Successfully called rom_analyzer plugin")
        table.insert(results.features_demoed, "cross_plugin_call")
    else
        SPO.log_info("  Note: rom_analyzer plugin not available (this is OK)")
        SPO.log_info("  Cross-plugin calls work when the target plugin is loaded")
    end
    
    SPO.log_info("")
    SPO.log_info("Asset system integration:")
    SPO.log_info("  Plugins can respond to asset modifications via on_asset_modified()")
    SPO.log_info("  Plugins can trigger asset reloads and updates")
    table.insert(results.features_demoed, "asset_system")
    
    return results
end


-- ============================================================================
-- PUBLIC COMMANDS
-- ============================================================================
COMMANDS = {}

-- Get System Information
COMMANDS.get_system_info = function(args)
    local info = {
        success = true,
        plugin = {
            id = PLUGIN_INFO.id,
            name = PLUGIN_INFO.name,
            version = PLUGIN_INFO.version,
        },
        editor = {
            version = "4.0.0",
            api_version = PLUGIN_INFO.api_version,
            platform = "desktop",
        },
        rom = {
            loaded = SPO.rom_size() > 0,
            size = SPO.rom_size(),
            size_kb = math.floor(SPO.rom_size() / 1024 * 100) / 100,
        },
        timestamp = os.time(),
    }
    
    SPO.log_info("System Information:")
    SPO.log_info(string.format("  Plugin: %s v%s", info.plugin.name, info.plugin.version))
    SPO.log_info(string.format("  Editor API: v%d", info.editor.api_version))
    SPO.log_info(string.format("  ROM: %s (%d bytes)", 
        info.rom.loaded and "Loaded" or "Not loaded", 
        info.rom.size))
    
    return info
end

-- Demo: ROM Access
COMMANDS.demo_rom_access = function(args)
    args = args or {}
    
    if plugin_state.is_running then
        return { success = false, error = "Another demo is already running" }
    end
    
    plugin_state.is_running = true
    SPO.notify_info("Starting ROM Access Demo...")
    
    local results = {
        success = true,
        read_demo = nil,
        write_demo = nil,
    }
    
    results.read_demo = demonstrate_rom_reads()
    
    if args.write then
        results.write_demo = demonstrate_rom_writes()
    else
        SPO.log_info("")
        SPO.log_info("ROM write demo skipped (use {write = true} to enable)")
        SPO.log_info("Write demo temporarily modifies ROM data with automatic restore")
    end
    
    plugin_state.demo_run_count = plugin_state.demo_run_count + 1
    plugin_state.is_running = false
    
    SPO.notify_success("ROM Access Demo complete!")
    return results
end

-- Demo: Notifications
COMMANDS.demo_notifications = function(args)
    if plugin_state.is_running then
        return { success = false, error = "Another demo is already running" }
    end
    
    plugin_state.is_running = true
    SPO.notify_info("Starting Notifications Demo...")
    
    local results = demonstrate_notifications()
    results.success = true
    
    plugin_state.demo_run_count = plugin_state.demo_run_count + 1
    plugin_state.is_running = false
    
    return results
end

-- Demo: Logging
COMMANDS.demo_logging = function(args)
    if plugin_state.is_running then
        return { success = false, error = "Another demo is already running" }
    end
    
    plugin_state.is_running = true
    
    local results = demonstrate_logging()
    results.success = true
    
    plugin_state.demo_run_count = plugin_state.demo_run_count + 1
    plugin_state.is_running = false
    
    return results
end

-- Demo: Batch Operations
COMMANDS.demo_batch_ops = function(args)
    if plugin_state.is_running then
        return { success = false, error = "Another demo is already running" }
    end
    
    plugin_state.is_running = true
    SPO.notify_info("Starting Batch Operations Demo...")
    
    local results = demonstrate_batch_ops(args)
    results.success = true
    
    plugin_state.demo_run_count = plugin_state.demo_run_count + 1
    plugin_state.is_running = false
    
    SPO.notify_success("Batch Operations Demo complete!")
    return results
end

-- Demo: Integration
COMMANDS.demo_integration = function(args)
    if plugin_state.is_running then
        return { success = false, error = "Another demo is already running" }
    end
    
    plugin_state.is_running = true
    SPO.notify_info("Starting Integration Demo...")
    
    local results = demonstrate_integration()
    results.success = true
    
    plugin_state.demo_run_count = plugin_state.demo_run_count + 1
    plugin_state.is_running = false
    
    SPO.notify_success("Integration Demo complete!")
    return results
end

-- Demo: Address Conversion
COMMANDS.demo_address_conv = function(args)
    if plugin_state.is_running then
        return { success = false, error = "Another demo is already running" }
    end
    
    plugin_state.is_running = true
    SPO.notify_info("Starting Address Conversion Demo...")
    
    local results = demonstrate_address_conversion()
    results.success = true
    
    plugin_state.demo_run_count = plugin_state.demo_run_count + 1
    plugin_state.is_running = false
    
    SPO.notify_success("Address Conversion Demo complete!")
    return results
end

-- Demo: Pattern Search
COMMANDS.demo_pattern_search = function(args)
    if plugin_state.is_running then
        return { success = false, error = "Another demo is already running" }
    end
    
    plugin_state.is_running = true
    SPO.notify_info("Starting Pattern Search Demo...")
    
    local results = demonstrate_pattern_search()
    results.success = true
    
    plugin_state.demo_run_count = plugin_state.demo_run_count + 1
    plugin_state.is_running = false
    
    SPO.notify_success("Pattern Search Demo complete!")
    return results
end


-- Run Full Demo
COMMANDS.run_full_demo = function(args)
    args = args or {}
    
    if plugin_state.is_running then
        return { success = false, error = "Another demo is already running" }
    end
    
    plugin_state.is_running = true
    local start_time = os.clock()
    
    SPO.log_info("")
    SPO.log_info("=" .. string.rep("=", 50))
    SPO.log_info("STARTING FULL DEMO")
    SPO.log_info("=" .. string.rep("=", 50))
    SPO.log_info("")
    
    local results = {
        success = true,
        demos = {},
        started_at = os.time(),
    }
    
    SPO.notify_info("Step 1/8: System Information...")
    results.demos.system_info = COMMANDS.get_system_info({})
    
    SPO.notify_info("Step 2/8: Logging Levels...")
    results.demos.logging = COMMANDS.demo_logging({})
    
    SPO.notify_info("Step 3/8: Notifications...")
    results.demos.notifications = COMMANDS.demo_notifications({})
    
    SPO.notify_info("Step 4/8: Address Conversion...")
    results.demos.address_conv = COMMANDS.demo_address_conv({})
    
    SPO.notify_info("Step 5/8: Pattern Search...")
    results.demos.pattern_search = COMMANDS.demo_pattern_search({})
    
    SPO.notify_info("Step 6/8: ROM Access (read-only)...")
    results.demos.rom_access = COMMANDS.demo_rom_access({ write = false })
    
    SPO.notify_info("Step 7/8: Batch Operations...")
    results.demos.batch_ops = COMMANDS.demo_batch_ops({ iterations = 5 })
    
    SPO.notify_info("Step 8/8: Editor Integration...")
    results.demos.integration = COMMANDS.demo_integration({})
    
    local elapsed_ms = math.floor((os.clock() - start_time) * 1000)
    results.completed_at = os.time()
    results.total_time_ms = elapsed_ms
    results.demos_run = plugin_state.demo_run_count
    
    SPO.log_info("")
    SPO.log_info("=" .. string.rep("=", 50))
    SPO.log_info("FULL DEMO COMPLETE")
    SPO.log_info("=" .. string.rep("=", 50))
    SPO.log_info(string.format("  Demos run: %d", results.demos_run))
    SPO.log_info(string.format("  Total time: %d ms", elapsed_ms))
    SPO.log_info("=" .. string.rep("=", 50))
    SPO.log_info("")
    SPO.log_info("Thank you for exploring the V4 Plugin System!")
    SPO.log_info("Check the log output above for detailed information.")
    SPO.log_info("")
    
    plugin_state.is_running = false
    
    SPO.notify_success("Full Demo Complete! Check the log for details.")
    return results
end

-- Get Plugin State
COMMANDS.get_plugin_state = function(args)
    local count = 0
    for _ in pairs(plugin_state.backup_data) do count = count + 1 end
    
    return {
        success = true,
        state = {
            demo_run_count = plugin_state.demo_run_count,
            is_running = plugin_state.is_running,
            has_backup_data = next(plugin_state.backup_data) ~= nil,
            backup_count = count,
        }
    }
end

-- Reset Plugin State
COMMANDS.reset_state = function(args)
    plugin_state.demo_run_count = 0
    plugin_state.last_results = nil
    plugin_state.is_running = false
    plugin_state.backup_data = {}
    plugin_state.timings = {}
    
    SPO.notify_success("Plugin state reset")
    return { success = true, message = "State reset to initial values" }
end

-- ============================================================================
-- END OF PLUGIN
-- ============================================================================
-- This plugin demonstrates:
--   Complete PLUGIN_INFO definition
--   All lifecycle events (on_init, on_shutdown, on_rom_loaded, on_asset_modified)
--   ROM access (rom_read, rom_read_byte, rom_write, rom_write_byte)
--   Pattern searching (find_pattern)
--   Address conversion (snes_to_pc, pc_to_snes)
--   All notification types (notify_info, notify_success, notify_warn, notify_error)
--   All log levels (log_debug, log_info, log_warn, log_error)
--   Batch operations with progress reporting
--   Cross-plugin communication (call_plugin)
--   Error handling and safety patterns
--   State management and caching
--   Utility functions and code organization
--   Comprehensive documentation and comments
-- ============================================================================
