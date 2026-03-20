-- Super Punch-Out!! Editor - Batch Export Plugin
-- Provides batch export functionality for assets

PLUGIN_INFO = {
    id = "example_batch_export",
    name = "Batch Exporter",
    version = "1.0.0",
    author = "Super Punch-Out!! Editor",
    description = "Batch export assets with customizable options",
    api_version = 1,
}

-- Export state tracking
local export_state = {
    in_progress = false,
    completed = 0,
    total = 0,
    errors = {},
    start_time = nil
}

function on_init()
    SPO.log_info("Batch Exporter plugin loaded")
end

function on_shutdown()
    export_state.in_progress = false
end

function on_rom_loaded()
    -- Reset state when a new ROM is loaded
    export_state = {
        in_progress = false,
        completed = 0,
        total = 0,
        errors = {},
        start_time = nil
    }
end

-- Simulated export functions (in a real plugin, these would use actual asset data)
local function export_graphics(id, format, output_dir)
    -- Simulate export time
    SPO.log_debug("Exporting graphics " .. id .. " to " .. format)
    return { success = true, file = output_dir .. "/gfx_" .. id .. "." .. string.lower(format) }
end

local function export_palette(id, format, output_dir)
    SPO.log_debug("Exporting palette " .. id .. " to " .. format)
    return { success = true, file = output_dir .. "/pal_" .. id .. "." .. string.lower(format) }
end

local function export_animation(id, format, output_dir)
    SPO.log_debug("Exporting animation " .. id .. " to " .. format)
    return { success = true, file = output_dir .. "/anim_" .. id .. "." .. string.lower(format) }
end

COMMANDS = {
    -- Start a batch export job
    start_export = function(args)
        if export_state.in_progress then
            return {
                success = false,
                error = "Export already in progress. Use get_status to check progress or cancel_export to abort."
            }
        end
        
        local asset_types = args and args.asset_types or { "graphics", "palette", "animation" }
        local format = args and args.format or "PNG"
        local output_dir = args and args.output_dir or "./exports"
        
        -- In a real implementation, you'd query the actual asset list from the manifest
        -- Here we simulate with some example IDs
        local assets_to_export = {}
        
        if table.contains(asset_types, "graphics") then
            for i = 1, 10 do
                table.insert(assets_to_export, { type = "graphics", id = "gfx_" .. i })
            end
        end
        
        if table.contains(asset_types, "palette") then
            for i = 1, 5 do
                table.insert(assets_to_export, { type = "palette", id = "pal_" .. i })
            end
        end
        
        if table.contains(asset_types, "animation") then
            for i = 1, 3 do
                table.insert(assets_to_export, { type = "animation", id = "anim_" .. i })
            end
        end
        
        export_state = {
            in_progress = true,
            completed = 0,
            total = #assets_to_export,
            errors = {},
            start_time = os.time(),
            assets = assets_to_export,
            format = format,
            output_dir = output_dir
        }
        
        SPO.log_info("Starting batch export of " .. #assets_to_export .. " assets")
        SPO.notify_info("Batch export started: " .. #assets_to_export .. " items")
        
        return {
            success = true,
            total = export_state.total,
            message = "Export job started"
        }
    end,
    
    -- Process next batch of exports (called repeatedly by UI)
    process_batch = function(args)
        if not export_state.in_progress then
            return {
                success = false,
                error = "No export in progress"
            }
        end
        
        local batch_size = args and args.batch_size or 5
        local processed = 0
        local results = {}
        
        while processed < batch_size and export_state.completed < export_state.total do
            export_state.completed = export_state.completed + 1
            local asset = export_state.assets[export_state.completed]
            
            local result
            if asset.type == "graphics" then
                result = export_graphics(asset.id, export_state.format, export_state.output_dir)
            elseif asset.type == "palette" then
                result = export_palette(asset.id, export_state.format, export_state.output_dir)
            elseif asset.type == "animation" then
                result = export_animation(asset.id, export_state.format, export_state.output_dir)
            end
            
            if result.success then
                table.insert(results, { id = asset.id, status = "success", file = result.file })
            else
                table.insert(export_state.errors, { id = asset.id, error = result.error })
                table.insert(results, { id = asset.id, status = "error", error = result.error })
            end
            
            processed = processed + 1
        end
        
        -- Check if complete
        local is_complete = export_state.completed >= export_state.total
        if is_complete then
            export_state.in_progress = false
            local elapsed = os.time() - (export_state.start_time or os.time())
            SPO.notify_success("Batch export complete! " .. export_state.completed .. " items in " .. elapsed .. "s")
        end
        
        return {
            success = true,
            completed = export_state.completed,
            total = export_state.total,
            percent = math.floor(export_state.completed / export_state.total * 100),
            is_complete = is_complete,
            results = results,
            errors = export_state.errors
        }
    end,
    
    -- Get current export status
    get_status = function(args)
        return {
            success = true,
            in_progress = export_state.in_progress,
            completed = export_state.completed,
            total = export_state.total,
            percent = export_state.total > 0 and math.floor(export_state.completed / export_state.total * 100) or 0,
            error_count = #export_state.errors
        }
    end,
    
    -- Cancel current export
    cancel_export = function(args)
        if not export_state.in_progress then
            return {
                success = false,
                error = "No export in progress"
            }
        end
        
        export_state.in_progress = false
        SPO.log_warn("Export cancelled by user")
        SPO.notify_info("Export cancelled")
        
        return {
            success = true,
            completed = export_state.completed,
            total = export_state.total,
            message = "Export cancelled"
        }
    end,
    
    -- Get export history/summary
    get_summary = function(args)
        return {
            success = true,
            last_export = export_state.start_time and os.date("%Y-%m-%d %H:%M:%S", export_state.start_time) or nil,
            total_exported = export_state.completed,
            errors = export_state.errors,
            output_directory = export_state.output_dir
        }
    end
}

-- Helper function for table containment
function table.contains(table, element)
    for _, value in pairs(table) do
        if value == element then
            return true
        end
    end
    return false
end
