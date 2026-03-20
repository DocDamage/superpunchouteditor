-- Super Punch-Out!! Editor - Advanced Palette Manager Plugin
-- Sophisticated palette manipulation and analysis plugin
-- Provides color analysis, optimization, conversion, and export functionality

PLUGIN_INFO = {
    id = "advanced_palette_manager",
    name = "Advanced Palette Manager",
    version = "1.0.0",
    author = "Super Punch-Out!! Editor",
    description = "Advanced palette manipulation with analysis, optimization, and export features",
    api_version = 1,
}

-- ============================================================================
-- Configuration and Constants
-- ============================================================================

-- SNES palette format: BGR555 (15-bit color, 2 bytes per color)
local PALETTE_CONFIG = {
    colors_per_palette = 16,      -- Standard SNES palette size
    bytes_per_color = 2,          -- 2 bytes per BGR555 color
    palette_size_bytes = 32,      -- 16 colors * 2 bytes
    boxer_palette_base = 0x180000, -- Base address for boxer palettes (example)
    max_palettes = 256,           -- Maximum palettes to scan
}

-- Track modified palettes for logging
local palette_change_log = {}
local plugin_initialized = false

-- ============================================================================
-- Utility Functions
-- ============================================================================

-- Convert BGR555 (SNES format) to RGB table
local function bgr555_to_rgb(bgr_value)
    -- BGR555 format: 0bbbbbgggggrrrrr
    local r = (bgr_value & 0x1F) << 3
    local g = (bgr_value >> 5) & 0x1F
    local b = (bgr_value >> 10) & 0x1F
    
    -- Scale 5-bit to 8-bit
    g = g << 3
    b = b << 3
    
    return { r = r, g = g, b = b }
end

-- Convert RGB to BGR555
local function rgb_to_bgr555(r, g, b)
    -- Scale 8-bit to 5-bit and pack
    local r5 = (r >> 3) & 0x1F
    local g5 = (g >> 3) & 0x1F
    local b5 = (b >> 3) & 0x1F
    
    return (b5 << 10) | (g5 << 5) | r5
end

-- Convert RGB to grayscale luminance
local function rgb_to_luminance(r, g, b)
    -- Standard luminance formula: 0.299*R + 0.587*G + 0.114*B
    return math.floor(0.299 * r + 0.587 * g + 0.114 * b)
end

-- Calculate color distance (simple Euclidean)
local function color_distance(c1, c2)
    local dr = c1.r - c2.r
    local dg = c1.g - c2.g
    local db = c1.b - c2.b
    return math.sqrt(dr * dr + dg * dg + db * db)
end

-- Read palette from ROM at given offset
local function read_palette_from_rom(offset, num_colors)
    num_colors = num_colors or PALETTE_CONFIG.colors_per_palette
    local palette = {}
    
    for i = 0, num_colors - 1 do
        local color_offset = offset + (i * 2)
        local low_byte = SPO.rom_read_byte(color_offset)
        local high_byte = SPO.rom_read_byte(color_offset + 1)
        local bgr_value = (high_byte << 8) | low_byte
        
        table.insert(palette, {
            index = i,
            bgr555 = bgr_value,
            rgb = bgr555_to_rgb(bgr_value)
        })
    end
    
    return palette
end

-- Write palette to ROM at given offset
local function write_palette_to_rom(offset, palette)
    for i, color in ipairs(palette) do
        local color_offset = offset + ((i - 1) * 2)
        local bgr_value = color.bgr555
        local low_byte = bgr_value & 0xFF
        local high_byte = (bgr_value >> 8) & 0xFF
        
        SPO.rom_write_byte(color_offset, low_byte)
        SPO.rom_write_byte(color_offset + 1, high_byte)
    end
end

-- Get boxer palette address (simplified - would use actual ROM map)
local function get_boxer_palette_address(boxer_name)
    local boxer_ids = {
        ["gabby_jay"] = 0,
        ["bear_hugger"] = 1,
        ["piston_hurricane"] = 2,
        ["bald_bull"] = 3,
        ["bob_charlie"] = 4,
        ["dragon_chan"] = 5,
        ["masked_club"] = 6,
        ["mr_sandman"] = 7,
        ["ardo"] = 8,
        ["narcis_prince"] = 9,
        ["heike_kagero"] = 10,
        ["mad_clown"] = 11,
        ["super_macho_man"] = 12,
    }
    
    local id = boxer_ids[string.lower(boxer_name)]
    if not id then
        return nil
    end
    
    -- Each boxer has multiple palettes (normal, hit, special, etc.)
    return PALETTE_CONFIG.boxer_palette_base + (id * 0x100)
end

-- Log palette modification
local function log_palette_change(palette_id, operation, details)
    local entry = {
        timestamp = os.time(),
        palette_id = palette_id,
        operation = operation,
        details = details
    }
    table.insert(palette_change_log, entry)
    SPO.log_info(string.format("[Palette Change] %s on palette %s: %s", 
        operation, tostring(palette_id), tostring(details)))
end

-- ============================================================================
-- Plugin Lifecycle
-- ============================================================================

function on_init()
    SPO.log_info("=" .. string.rep("=", 60))
    SPO.log_info("Advanced Palette Manager v" .. PLUGIN_INFO.version)
    SPO.log_info("Loaded with support for BGR555 color manipulation")
    SPO.log_info("=" .. string.rep("=", 60))
    
    palette_change_log = {}
    plugin_initialized = true
    
    SPO.notify_success("Advanced Palette Manager loaded!")
end

function on_shutdown()
    plugin_initialized = false
    SPO.log_info("Advanced Palette Manager shutting down...")
    SPO.log_info(string.format("Total palette operations logged: %d", #palette_change_log))
end

function on_asset_modified()
    -- Called when any asset is modified - we can check if it's a palette
    SPO.log_debug("Asset modified - checking if palette-related...")
end

-- ============================================================================
-- Commands
-- ============================================================================

COMMANDS = {
    -- ------------------------------------------------------------------------
    -- Analyze all palettes for a boxer, show color distribution
    -- ------------------------------------------------------------------------
    analyze_palettes = function(args)
        local boxer_name = args and args.boxer_name
        if not boxer_name then
            return { success = false, error = "Missing 'boxer_name' argument" }
        end
        
        local base_addr = get_boxer_palette_address(boxer_name)
        if not base_addr then
            return { success = false, error = "Unknown boxer: " .. boxer_name }
        end
        
        SPO.log_info(string.format("Analyzing palettes for %s at offset 0x%06X", boxer_name, base_addr))
        
        local analysis = {
            boxer = boxer_name,
            base_address = base_addr,
            palettes = {},
            color_distribution = {},
            brightness_stats = {},
            unique_colors = {}
        }
        
        local unique_color_set = {}
        local all_colors = {}
        
        -- Analyze multiple palettes for this boxer (normal, hit, special, etc.)
        for palette_idx = 0, 3 do
            local palette_offset = base_addr + (palette_idx * PALETTE_CONFIG.palette_size_bytes)
            local palette = read_palette_from_rom(palette_offset)
            
            local palette_analysis = {
                index = palette_idx,
                offset = palette_offset,
                colors = {},
                average_brightness = 0,
                saturation = 0
            }
            
            local total_brightness = 0
            local color_count = 0
            
            for _, color in ipairs(palette) do
                -- Skip color 0 (usually transparent)
                if color.index > 0 then
                    local brightness = math.floor((color.rgb.r + color.rgb.g + color.rgb.b) / 3)
                    total_brightness = total_brightness + brightness
                    color_count = color_count + 1
                    
                    -- Track unique colors
                    local color_key = string.format("%02X%02X%02X", color.rgb.r, color.rgb.g, color.rgb.b)
                    unique_color_set[color_key] = {
                        r = color.rgb.r,
                        g = color.rgb.g,
                        b = color.rgb.b,
                        bgr555 = color.bgr555
                    }
                    
                    -- Track color distribution by hue
                    local hue = "neutral"
                    if color.rgb.r > color.rgb.g and color.rgb.r > color.rgb.b then
                        hue = "red"
                    elseif color.rgb.g > color.rgb.r and color.rgb.g > color.rgb.b then
                        hue = "green"
                    elseif color.rgb.b > color.rgb.r and color.rgb.b > color.rgb.g then
                        hue = "blue"
                    elseif color.rgb.r > 200 and color.rgb.g > 200 and color.rgb.b > 200 then
                        hue = "white"
                    elseif color.rgb.r < 50 and color.rgb.g < 50 and color.rgb.b < 50 then
                        hue = "black"
                    end
                    
                    analysis.color_distribution[hue] = (analysis.color_distribution[hue] or 0) + 1
                    table.insert(all_colors, { hue = hue, brightness = brightness, color = color })
                end
                
                table.insert(palette_analysis.colors, {
                    index = color.index,
                    hex = string.format("#%02X%02X%02X", color.rgb.r, color.rgb.g, color.rgb.b),
                    bgr555 = string.format("0x%04X", color.bgr555)
                })
            end
            
            palette_analysis.average_brightness = color_count > 0 and math.floor(total_brightness / color_count) or 0
            table.insert(analysis.palettes, palette_analysis)
        end
        
        -- Compile unique colors list
        for _, color in pairs(unique_color_set) do
            table.insert(analysis.unique_colors, color)
        end
        
        -- Calculate overall brightness statistics
        if #all_colors > 0 then
            table.sort(all_colors, function(a, b) return a.brightness < b.brightness end)
            analysis.brightness_stats = {
                min = all_colors[1].brightness,
                max = all_colors[#all_colors].brightness,
                median = all_colors[math.floor(#all_colors / 2)].brightness,
                unique_count = #analysis.unique_colors
            }
        end
        
        SPO.notify_success(string.format("Analyzed %d palettes for %s", #analysis.palettes, boxer_name))
        
        return {
            success = true,
            analysis = analysis
        }
    end,
    
    -- ------------------------------------------------------------------------
    -- Optimize palette to reduce colors and minimize space
    -- ------------------------------------------------------------------------
    optimize_palette = function(args)
        local palette_id = args and args.palette_id
        if not palette_id then
            return { success = false, error = "Missing 'palette_id' argument" }
        end
        
        -- Parse palette_id (format: "boxer_name:idx" or just offset)
        local offset = tonumber(palette_id)
        if not offset then
            -- Try to parse boxer format
            local boxer_name, idx = string.match(palette_id, "([^:]+):(%d+)")
            if boxer_name then
                local base = get_boxer_palette_address(boxer_name)
                if base then
                    offset = base + (tonumber(idx) * PALETTE_CONFIG.palette_size_bytes)
                end
            end
        end
        
        if not offset then
            return { success = false, error = "Invalid palette_id format" }
        end
        
        SPO.log_info(string.format("Optimizing palette at offset 0x%06X", offset))
        
        local palette = read_palette_from_rom(offset)
        local original_colors = {}
        local color_usage = {}
        
        -- Count similar colors (within threshold)
        local similarity_threshold = args and args.threshold or 30
        local colors_to_merge = {}
        
        for i = 1, #palette do
            local c1 = palette[i]
            original_colors[i] = c1.bgr555
            
            for j = i + 1, #palette do
                local c2 = palette[j]
                local dist = color_distance(c1.rgb, c2.rgb)
                
                if dist < similarity_threshold then
                    table.insert(colors_to_merge, {
                        idx1 = i,
                        idx2 = j,
                        distance = dist,
                        c1 = c1,
                        c2 = c2
                    })
                end
            end
        end
        
        -- Sort by distance (closest first)
        table.sort(colors_to_merge, function(a, b) return a.distance < b.distance end)
        
        local optimization_result = {
            palette_offset = offset,
            original_unique_colors = #palette,
            similar_pairs_found = #colors_to_merge,
            suggestions = {},
            space_savings_estimate = 0
        }
        
        -- Generate suggestions for merging
        for i, merge in ipairs(colors_to_merge) do
            if i <= 5 then  -- Top 5 suggestions
                table.insert(optimization_result.suggestions, {
                    color1_idx = merge.idx1 - 1,  -- 0-indexed
                    color2_idx = merge.idx2 - 1,
                    color1_hex = string.format("#%02X%02X%02X", merge.c1.rgb.r, merge.c1.rgb.g, merge.c1.rgb.b),
                    color2_hex = string.format("#%02X%02X%02X", merge.c2.rgb.r, merge.c2.rgb.g, merge.c2.rgb.b),
                    distance = math.floor(merge.distance * 100) / 100,
                    recommendation = "Consider merging these similar colors"
                })
            end
        end
        
        -- Calculate potential space savings
        local potential_merges = math.min(#colors_to_merge, 8)
        optimization_result.space_savings_estimate = potential_merges * 2  -- 2 bytes per color
        
        SPO.notify_info(string.format("Found %d similar color pairs in palette", #colors_to_merge))
        
        return {
            success = true,
            result = optimization_result
        }
    end,
    
    -- ------------------------------------------------------------------------
    -- Convert all boxer graphics to grayscale
    -- ------------------------------------------------------------------------
    convert_to_grayscale = function(args)
        local boxer_name = args and args.boxer_name
        if not boxer_name then
            return { success = false, error = "Missing 'boxer_name' argument" }
        end
        
        local base_addr = get_boxer_palette_address(boxer_name)
        if not base_addr then
            return { success = false, error = "Unknown boxer: " .. boxer_name }
        end
        
        SPO.log_info(string.format("Converting %s palettes to grayscale", boxer_name))
        
        local conversion_results = {
            boxer = boxer_name,
            palettes_converted = 0,
            colors_converted = 0,
            palettes = {}
        }
        
        -- Convert all palettes for this boxer
        for palette_idx = 0, 3 do
            local palette_offset = base_addr + (palette_idx * PALETTE_CONFIG.palette_size_bytes)
            local palette = read_palette_from_rom(palette_offset)
            
            local converted = {}
            local changes_made = false
            
            for _, color in ipairs(palette) do
                if color.index > 0 then  -- Skip color 0 (transparent)
                    local lum = rgb_to_luminance(color.rgb.r, color.rgb.g, color.rgb.b)
                    local new_bgr = rgb_to_bgr555(lum, lum, lum)
                    
                    if new_bgr ~= color.bgr555 then
                        changes_made = true
                        conversion_results.colors_converted = conversion_results.colors_converted + 1
                    end
                    
                    table.insert(converted, {
                        index = color.index,
                        bgr555 = new_bgr,
                        original_bgr555 = color.bgr555,
                        grayscale_value = lum
                    })
                else
                    table.insert(converted, color)  -- Keep transparent color
                end
            end
            
            if changes_made then
                write_palette_to_rom(palette_offset, converted)
                conversion_results.palettes_converted = conversion_results.palettes_converted + 1
                table.insert(conversion_results.palettes, {
                    index = palette_idx,
                    offset = palette_offset,
                    colors_changed = #converted - 1  -- Excluding color 0
                })
                
                log_palette_change(string.format("%s:%d", boxer_name, palette_idx), 
                    "CONVERT_TO_GRAYSCALE", "Converted to grayscale")
            end
        end
        
        SPO.notify_success(string.format("Converted %d palettes to grayscale for %s", 
            conversion_results.palettes_converted, boxer_name))
        
        return {
            success = true,
            result = conversion_results
        }
    end,
    
    -- ------------------------------------------------------------------------
    -- Swap two colors in a palette
    -- ------------------------------------------------------------------------
    swap_colors = function(args)
        local palette_id = args and args.palette_id
        local color1_idx = args and args.color1_idx
        local color2_idx = args and args.color2_idx
        
        if not palette_id or color1_idx == nil or color2_idx == nil then
            return { success = false, error = "Missing required arguments: palette_id, color1_idx, color2_idx" }
        end
        
        -- Parse palette offset
        local offset = tonumber(palette_id)
        if not offset then
            local boxer_name, idx = string.match(palette_id, "([^:]+):(%d+)")
            if boxer_name then
                local base = get_boxer_palette_address(boxer_name)
                if base then
                    offset = base + (tonumber(idx) * PALETTE_CONFIG.palette_size_bytes)
                end
            end
        end
        
        if not offset then
            return { success = false, error = "Invalid palette_id format" }
        end
        
        -- Validate color indices
        if color1_idx < 0 or color1_idx >= PALETTE_CONFIG.colors_per_palette or
           color2_idx < 0 or color2_idx >= PALETTE_CONFIG.colors_per_palette then
            return { success = false, error = "Color indices must be 0-15" }
        end
        
        if color1_idx == color2_idx then
            return { success = false, error = "Color indices must be different" }
        end
        
        -- Read current palette
        local palette = read_palette_from_rom(offset)
        local color1 = palette[color1_idx + 1]  -- Lua is 1-indexed
        local color2 = palette[color2_idx + 1]
        
        -- Swap the colors
        palette[color1_idx + 1] = color2
        palette[color2_idx + 1] = color1
        
        -- Update indices
        palette[color1_idx + 1].index = color1_idx
        palette[color2_idx + 1].index = color2_idx
        
        -- Write back to ROM
        write_palette_to_rom(offset, palette)
        
        local swap_info = {
            palette_offset = offset,
            color1 = {
                index = color1_idx,
                original_bgr555 = string.format("0x%04X", color1.bgr555),
                new_bgr555 = string.format("0x%04X", color2.bgr555),
                hex = string.format("#%02X%02X%02X", color2.rgb.r, color2.rgb.g, color2.rgb.b)
            },
            color2 = {
                index = color2_idx,
                original_bgr555 = string.format("0x%04X", color2.bgr555),
                new_bgr555 = string.format("0x%04X", color1.bgr555),
                hex = string.format("#%02X%02X%02X", color1.rgb.r, color1.rgb.g, color1.rgb.b)
            }
        }
        
        log_palette_change(palette_id, "SWAP_COLORS", 
            string.format("Swapped colors %d and %d", color1_idx, color2_idx))
        
        SPO.notify_success(string.format("Swapped colors %d and %d in palette", color1_idx, color2_idx))
        
        return {
            success = true,
            swap = swap_info
        }
    end,
    
    -- ------------------------------------------------------------------------
    -- Export palette to various formats
    -- ------------------------------------------------------------------------
    export_palette = function(args)
        local palette_id = args and args.palette_id
        local format = args and args.format or "json"
        
        if not palette_id then
            return { success = false, error = "Missing 'palette_id' argument" }
        end
        
        -- Parse palette offset
        local offset = tonumber(palette_id)
        local boxer_name = nil
        local palette_idx = 0
        
        if not offset then
            boxer_name, palette_idx = string.match(palette_id, "([^:]+):(%d+)")
            if boxer_name then
                palette_idx = tonumber(palette_idx)
                local base = get_boxer_palette_address(boxer_name)
                if base then
                    offset = base + (palette_idx * PALETTE_CONFIG.palette_size_bytes)
                end
            end
        end
        
        if not offset then
            return { success = false, error = "Invalid palette_id format" }
        end
        
        local palette = read_palette_from_rom(offset)
        local export_data = {
            format = format,
            palette_id = palette_id,
            source_offset = string.format("0x%06X", offset),
            colors = {}
        }
        
        if format == "json" then
            -- JSON format with full metadata
            for _, color in ipairs(palette) do
                table.insert(export_data.colors, {
                    index = color.index,
                    r = color.rgb.r,
                    g = color.rgb.g,
                    b = color.rgb.b,
                    hex = string.format("#%02X%02X%02X", color.rgb.r, color.rgb.g, color.rgb.b),
                    bgr555 = color.bgr555
                })
            end
            
        elseif format == "act" then
            -- Adobe Color Table format (raw RGB values)
            local act_data = {}
            for _, color in ipairs(palette) do
                table.insert(act_data, color.rgb.r)
                table.insert(act_data, color.rgb.g)
                table.insert(act_data, color.rgb.b)
            end
            -- Pad to 256 colors if needed
            while #act_data < 768 do
                table.insert(act_data, 0)
            end
            export_data.raw_bytes = act_data
            export_data.note = "Adobe Color Table format - 768 bytes (256 colors * 3 RGB)"
            
        elseif format == "png_palette" then
            -- PNG PLTE chunk format (similar to ACT but no padding)
            local plte_data = {}
            for _, color in ipairs(palette) do
                table.insert(plte_data, color.rgb.r)
                table.insert(plte_data, color.rgb.g)
                table.insert(plte_data, color.rgb.b)
            end
            export_data.raw_bytes = plte_data
            export_data.note = "PNG PLTE format - 48 bytes (16 colors * 3 RGB)"
            
        elseif format == "csv" then
            -- CSV format
            export_data.csv_header = "index,r,g,b,hex,bgr555"
            export_data.csv_rows = {}
            for _, color in ipairs(palette) do
                table.insert(export_data.csv_rows, string.format("%d,%d,%d,%d,#%02X%02X%02X,0x%04X",
                    color.index, color.rgb.r, color.rgb.g, color.rgb.b,
                    color.rgb.r, color.rgb.g, color.rgb.b, color.bgr555))
            end
            
        elseif format == "gpl" then
            -- GIMP Palette format
            export_data.gpl_header = string.format("GIMP Palette\nName: %s\nColumns: 16\n#",
                boxer_name and (boxer_name .. "_" .. palette_idx) or ("palette_" .. offset))
            export_data.gpl_colors = {}
            for _, color in ipairs(palette) do
                table.insert(export_data.gpl_colors, 
                    string.format("%3d %3d %3d\tColor %d", 
                        color.rgb.r, color.rgb.g, color.rgb.b, color.index))
            end
            
        else
            return { success = false, error = "Unknown format: " .. format .. ". Supported: json, act, png_palette, csv, gpl" }
        end
        
        SPO.notify_success(string.format("Exported palette to %s format", format))
        
        return {
            success = true,
            export = export_data
        }
    end,
    
    -- ------------------------------------------------------------------------
    -- Get palette change log
    -- ------------------------------------------------------------------------
    get_change_log = function(args)
        local limit = args and args.limit or 50
        local recent_changes = {}
        
        for i = math.max(1, #palette_change_log - limit + 1), #palette_change_log do
            table.insert(recent_changes, palette_change_log[i])
        end
        
        return {
            success = true,
            total_changes = #palette_change_log,
            changes = recent_changes
        }
    end,
    
    -- ------------------------------------------------------------------------
    -- Import palette from various formats
    -- ------------------------------------------------------------------------
    import_palette = function(args)
        local palette_id = args and args.palette_id
        local format = args and args.format or "json"
        local data = args and args.data
        
        if not palette_id or not data then
            return { success = false, error = "Missing 'palette_id' or 'data' argument" }
        end
        
        -- Parse palette offset
        local offset = tonumber(palette_id)
        if not offset then
            local boxer_name, idx = string.match(palette_id, "([^:]+):(%d+)")
            if boxer_name then
                local base = get_boxer_palette_address(boxer_name)
                if base then
                    offset = base + (tonumber(idx) * PALETTE_CONFIG.palette_size_bytes)
                end
            end
        end
        
        if not offset then
            return { success = false, error = "Invalid palette_id format" }
        end
        
        local imported_colors = {}
        local colors_imported = 0
        
        if format == "json" then
            -- Expect array of {r, g, b} or {bgr555}
            for i, color_data in ipairs(data.colors or {}) do
                if i > PALETTE_CONFIG.colors_per_palette then break end
                
                local bgr_value
                if color_data.bgr555 then
                    bgr_value = color_data.bgr555
                else
                    bgr_value = rgb_to_bgr555(color_data.r or 0, color_data.g or 0, color_data.b or 0)
                end
                
                table.insert(imported_colors, {
                    index = i - 1,
                    bgr555 = bgr_value
                })
                colors_imported = colors_imported + 1
            end
            
        elseif format == "rgb_array" then
            -- Flat array of R,G,B values
            local data_array = data
            for i = 1, math.min(#data_array / 3, PALETTE_CONFIG.colors_per_palette) do
                local r = data_array[(i - 1) * 3 + 1] or 0
                local g = data_array[(i - 1) * 3 + 2] or 0
                local b = data_array[(i - 1) * 3 + 3] or 0
                
                table.insert(imported_colors, {
                    index = i - 1,
                    bgr555 = rgb_to_bgr555(r, g, b)
                })
                colors_imported = colors_imported + 1
            end
        end
        
        if colors_imported > 0 then
            write_palette_to_rom(offset, imported_colors)
            
            log_palette_change(palette_id, "IMPORT", 
                string.format("Imported %d colors from %s format", colors_imported, format))
            
            SPO.notify_success(string.format("Imported %d colors into palette", colors_imported))
        end
        
        return {
            success = true,
            colors_imported = colors_imported,
            format = format
        }
    end
}
