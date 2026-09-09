-- Super Punch-Out!! Editor - ROM Analyzer Plugin
-- Comprehensive ROM analysis and documentation generation
-- Finds compression patterns, unused space, generates memory maps, and more

PLUGIN_INFO = {
    id = "rom_analyzer",
    name = "ROM Analyzer",
    version = "1.0.0",
    author = "Super Punch-Out!! Editor",
    description = "Deep ROM analysis: compression detection, free space, memory mapping, corruption detection",
    api_version = 1,
}

-- ============================================================================
-- Configuration and Constants
-- ============================================================================

local ANALYSIS_CONFIG = {
    -- Common SNES compression signatures
    compression_signatures = {
        { name = "LZSS variant", pattern = {0x01}, min_size = 4 },
        { name = "RLE", pattern = {0x00, 0x00}, min_size = 3 },
        { name = "Huffman", pattern = {0xFF, 0xFF}, min_size = 8 },
    },
    
    -- Graphics format signatures
    graphics_signatures = {
        { name = "Mode 7 Tilemap", pattern = {0x00, 0x00, 0x00, 0x80}, description = "Mode 7 header" },
        { name = "SNES Tile", pattern = {0x00, 0x00, 0x00, 0x00}, description = "Empty 2bpp tile" },
        { name = "SPC700", pattern = {0x53, 0x50, 0x43}, description = "SPC700 header" },
    },
    
    -- Free space detection
    free_space_byte = 0xFF,
    min_free_space = 256,  -- Minimum bytes to report as free
    
    -- Corruption check
    corruption_patterns = {
        { name = "Repeated bytes", pattern = "corruption_repeated" },
        { name = "Unusual bank boundary", pattern = "bank_boundary" },
    }
}

-- Analysis cache
local analysis_cache = {
    compression_blocks = nil,
    free_regions = nil,
    asset_list = nil,
    last_rom_size = 0
}

-- ============================================================================
-- Utility Functions
-- ============================================================================

-- Convert PC offset to SNES address
local function pc_to_snes_addr(pc_offset)
    local bank = math.floor(pc_offset / 0x8000)
    local addr = 0x8000 + (pc_offset % 0x8000)
    return 0x800000 + (bank * 0x10000) + (addr - 0x8000)
end

-- Format address as string
local function format_addr(pc_offset)
    return string.format("$%06X (SNES: $%06X)", pc_offset, pc_to_snes_addr(pc_offset))
end

-- Calculate entropy of data block (0-8, higher = more random/compressed)
local function calculate_entropy(data, offset, length)
    offset = offset or 1
    length = length or #data
    
    local freq = {}
    for i = 0, 255 do
        freq[i] = 0
    end
    
    for i = offset, math.min(offset + length - 1, #data) do
        local byte = string.byte(data, i)
        freq[byte] = freq[byte] + 1
    end
    
    local entropy = 0
    local n = length
    
    for i = 0, 255 do
        if freq[i] > 0 then
            local p = freq[i] / n
            entropy = entropy - (p * math.log(p, 2))
        end
    end
    
    return entropy
end

-- Find runs of identical bytes
local function find_byte_runs(data, target_byte, min_length)
    min_length = min_length or 256
    local runs = {}
    local current_run_start = nil
    local current_run_length = 0
    
    for i = 1, #data do
        local byte = string.byte(data, i)
        if byte == target_byte then
            if not current_run_start then
                current_run_start = i
                current_run_length = 1
            else
                current_run_length = current_run_length + 1
            end
        else
            if current_run_start and current_run_length >= min_length then
                table.insert(runs, {
                    start = current_run_start,
                    length = current_run_length,
                    pc_offset = current_run_start - 1,  -- 0-indexed
                })
            end
            current_run_start = nil
            current_run_length = 0
        end
    end
    
    -- Check for run at end
    if current_run_start and current_run_length >= min_length then
        table.insert(runs, {
            start = current_run_start,
            length = current_run_length,
            pc_offset = current_run_start - 1,
        })
    end
    
    return runs
end

-- ============================================================================
-- Plugin Lifecycle
-- ============================================================================

function on_init()
    SPO.log_info("=" .. string.rep("=", 60))
    SPO.log_info("ROM Analyzer v" .. PLUGIN_INFO.version)
    SPO.log_info("Deep ROM analysis and documentation tools loaded")
    SPO.log_info("=" .. string.rep("=", 60))
    
    analysis_cache = {
        compression_blocks = nil,
        free_regions = nil,
        asset_list = nil,
        last_rom_size = 0
    }
    
    SPO.notify_success("ROM Analyzer loaded!")
end

function on_shutdown()
    SPO.log_info("ROM Analyzer shutting down...")
end

function on_rom_loaded()
    -- Clear cache when ROM changes
    analysis_cache = {
        compression_blocks = nil,
        free_regions = nil,
        asset_list = nil,
        last_rom_size = SPO.rom_size()
    }
    SPO.log_info("ROM loaded - analysis cache cleared")
end

-- ============================================================================
-- Commands
-- ============================================================================

COMMANDS = {
    -- ------------------------------------------------------------------------
    -- Analyze compression in ROM
    -- ------------------------------------------------------------------------
    analyze_compression = function(args)
        SPO.log_info("Analyzing ROM compression patterns...")
        
        local rom_size = SPO.rom_size()
        local bank_size = 0x8000
        local num_banks = math.floor(rom_size / bank_size)
        
        local compression_blocks = {}
        local total_compressed = 0
        local total_uncompressed = 0
        
        -- Analyze each bank
        for bank = 0, num_banks - 1 do
            local bank_offset = bank * bank_size
            local bank_data = SPO.rom_read(bank_offset, bank_size)
            
            -- Calculate entropy for this bank
            local entropy = calculate_entropy(bank_data)
            
            -- Entropy > 7 suggests compressed/encrypted data
            -- Entropy < 5 suggests structured/uncompressed data
            local is_likely_compressed = entropy > 6.5
            local is_likely_empty = entropy < 0.5
            
            if is_likely_compressed then
                total_compressed = total_compressed + bank_size
            elseif not is_likely_empty then
                total_uncompressed = total_uncompressed + bank_size
            end
            
            -- Find specific compression signatures
            local signatures_found = {}
            for _, sig in ipairs(ANALYSIS_CONFIG.compression_signatures) do
                local results = SPO.find_pattern(sig.pattern)
                if results and #results > 0 then
                    table.insert(signatures_found, {
                        name = sig.name,
                        count = #results,
                        example_offset = results[1]
                    })
                end
            end
            
            table.insert(compression_blocks, {
                bank = bank,
                pc_offset = bank_offset,
                snes_bank = 0x80 + bank,
                entropy = math.floor(entropy * 100) / 100,
                likely_compressed = is_likely_compressed,
                likely_empty = is_likely_empty,
                signatures_found = signatures_found
            })
        end
        
        -- Store in cache
        analysis_cache.compression_blocks = compression_blocks
        
        -- Calculate overall statistics
        local compression_ratio = total_uncompressed > 0 and 
            math.floor((total_compressed / (total_compressed + total_uncompressed)) * 100) or 0
        
        SPO.notify_success(string.format("Analyzed %d banks - %d%% appear compressed", num_banks, compression_ratio))
        
        return {
            success = true,
            banks_analyzed = num_banks,
            total_rom_size = rom_size,
            likely_compressed_bytes = total_compressed,
            likely_uncompressed_bytes = total_uncompressed,
            estimated_compression_ratio = compression_ratio .. "%",
            bank_analysis = compression_blocks,
            high_entropy_banks = (function()
                local high = {}
                for _, block in ipairs(compression_blocks) do
                    if block.entropy > 7 then
                        table.insert(high, block)
                    end
                end
                return high
            end)()
        }
    end,
    
    -- ------------------------------------------------------------------------
    -- Find unused/free space in ROM
    -- ------------------------------------------------------------------------
    find_unused_space = function(args)
        local min_size = args and args.min_size or ANALYSIS_CONFIG.min_free_space
        
        SPO.log_info(string.format("Scanning for unused space (min %d bytes)...", min_size))
        
        local rom_size = SPO.rom_size()
        local bank_size = 0x8000
        local num_banks = math.floor(rom_size / bank_size)
        
        local free_regions = {}
        local total_free = 0
        local free_by_bank = {}
        
        for bank = 0, num_banks - 1 do
            local bank_offset = bank * bank_size
            local bank_data = SPO.rom_read(bank_offset, bank_size)
            
            local runs = find_byte_runs(bank_data, ANALYSIS_CONFIG.free_space_byte, min_size)
            local bank_free = 0
            
            for _, run in ipairs(runs) do
                table.insert(free_regions, {
                    bank = bank,
                    pc_offset = bank_offset + run.pc_offset,
                    snes_addr = pc_to_snes_addr(bank_offset + run.pc_offset),
                    size = run.length,
                    end_offset = bank_offset + run.pc_offset + run.length - 1,
                    suitable_for = run.length >= 1024 and "code/data" or "small tables"
                })
                bank_free = bank_free + run.length
                total_free = total_free + run.length
            end
            
            free_by_bank[bank] = bank_free
        end
        
        -- Sort by size (largest first)
        table.sort(free_regions, function(a, b) return a.size > b.size end)
        
        -- Store in cache
        analysis_cache.free_regions = free_regions
        
        -- Calculate statistics
        local free_percentage = math.floor((total_free / rom_size) * 1000) / 10
        local largest_free = free_regions[1] and free_regions[1].size or 0
        
        SPO.notify_success(string.format("Found %d free regions totaling %d KB (%.1f%%)", 
            #free_regions, math.floor(total_free / 1024), free_percentage))
        
        return {
            success = true,
            total_free_bytes = total_free,
            total_free_kb = math.floor(total_free / 1024 * 10) / 10,
            free_percentage = free_percentage .. "%",
            largest_contiguous = largest_free,
            num_regions = #free_regions,
            regions = free_regions,
            free_by_bank = free_by_bank,
            rom_size = rom_size
        }
    end,
    
    -- ------------------------------------------------------------------------
    -- Generate visual/text ROM memory map
    -- ------------------------------------------------------------------------
    generate_rom_map = function(args)
        local detail_level = args and args.detail or "medium"  -- low, medium, high
        
        SPO.log_info("Generating ROM memory map...")
        
        local rom_size = SPO.rom_size()
        local bank_size = 0x8000
        local num_banks = math.floor(rom_size / bank_size)
        
        local memory_map = {
            rom_size = rom_size,
            num_banks = num_banks,
            banks = {},
            visual_map = {},
            summary = {}
        }
        
        -- Use cached data if available
        local compression_data = analysis_cache.compression_blocks
        local free_data = {}
        
        if analysis_cache.free_regions then
            for _, region in ipairs(analysis_cache.free_regions) do
                free_data[region.bank] = free_data[region.bank] or {}
                table.insert(free_data[region.bank], region)
            end
        end
        
        for bank = 0, num_banks - 1 do
            local bank_offset = bank * bank_size
            local bank_info = {
                bank_num = bank,
                snes_bank = string.format("$%02X", 0x80 + bank),
                pc_start = bank_offset,
                pc_end = bank_offset + bank_size - 1,
                regions = {}
            }
            
            -- Analyze bank content
            if compression_data and compression_data[bank + 1] then
                local comp = compression_data[bank + 1]
                bank_info.entropy = comp.entropy
                bank_info.likely_compressed = comp.likely_compressed
            end
            
            -- Mark free regions
            if free_data[bank] then
                bank_info.free_regions = free_data[bank]
            end
            
            -- Simple classification
            if bank_info.likely_compressed then
                bank_info.classification = "COMPRESSED_DATA"
                bank_info.description = "Likely compressed graphics or tilemap data"
            elseif bank_info.free_regions and #bank_info.free_regions > 0 then
                local total_free = 0
                for _, r in ipairs(bank_info.free_regions) do
                    total_free = total_free + r.size
                end
                if total_free > bank_size * 0.8 then
                    bank_info.classification = "MOSTLY_FREE"
                    bank_info.description = "Mostly free space"
                else
                    bank_info.classification = "MIXED"
                    bank_info.description = "Mixed used/free space"
                end
            else
                bank_info.classification = "CODE_DATA"
                bank_info.description = "Likely code or uncompressed data"
            end
            
            table.insert(memory_map.banks, bank_info)
        end
        
        -- Generate visual ASCII map
        local visual_lines = {}
        table.insert(visual_lines, "ROM Memory Map (32 KB per character = 1 bank)")
        table.insert(visual_lines, string.rep("=", 60))
        
        local legend = {
            ["COMPRESSED_DATA"] = "#",
            ["CODE_DATA"] = "X",
            ["MOSTLY_FREE"] = ".",
            ["MIXED"] = "+",
        }
        
        local map_row = ""
        for i, bank in ipairs(memory_map.banks) do
            map_row = map_row .. (legend[bank.classification] or "?")
            if i % 32 == 0 then
                table.insert(visual_lines, string.format("$%02X: ", 0x80 + i - 32) .. map_row)
                map_row = ""
            end
        end
        if #map_row > 0 then
            table.insert(visual_lines, map_row)
        end
        
        table.insert(visual_lines, "")
        table.insert(visual_lines, "Legend: # = Compressed, X = Code/Data, . = Free, + = Mixed")
        
        memory_map.visual_map = table.concat(visual_lines, "\n")
        
        -- Generate summary
        local classifications = {}
        for _, bank in ipairs(memory_map.banks) do
            classifications[bank.classification] = (classifications[bank.classification] or 0) + 1
        end
        memory_map.summary = classifications
        
        SPO.notify_success(string.format("Generated memory map for %d banks", num_banks))
        
        return {
            success = true,
            memory_map = memory_map,
            visual_map = memory_map.visual_map
        }
    end,
    
    -- ------------------------------------------------------------------------
    -- Detect potential corruption issues
    -- ------------------------------------------------------------------------
    detect_corruption = function(args)
        SPO.log_info("Scanning for potential corruption...")
        
        local rom_size = SPO.rom_size()
        local issues = {}
        
        -- Check 1: Look for suspicious repeated byte patterns (corruption marker)
        local bank_size = 0x8000
        local num_banks = math.floor(rom_size / bank_size)
        
        for bank = 0, num_banks - 1 do
            local bank_offset = bank * bank_size
            local bank_data = SPO.rom_read(bank_offset, bank_size)
            
            -- Check for excessive repetition (possible corruption)
            local byte_freq = {}
            for i = 0, 255 do byte_freq[i] = 0 end
            
            for i = 1, #bank_data do
                local byte = string.byte(bank_data, i)
                byte_freq[byte] = byte_freq[byte] + 1
            end
            
            -- Find dominant byte
            local max_freq = 0
            local max_byte = 0
            for i = 0, 255 do
                if byte_freq[i] > max_freq then
                    max_freq = byte_freq[i]
                    max_byte = i
                end
            end
            
            local dominance = max_freq / bank_size
            if dominance > 0.5 and max_byte ~= 0xFF then
                table.insert(issues, {
                    type = "suspicious_repetition",
                    severity = dominance > 0.8 and "HIGH" or "MEDIUM",
                    bank = bank,
                    description = string.format("Byte $%02X dominates %.1f%% of bank", max_byte, dominance * 100),
                    recommendation = "Verify this bank is not corrupted"
                })
            end
        end
        
        -- Check 2: Look for truncated data at bank boundaries
        for bank = 1, num_banks - 1 do
            local boundary = bank * bank_size
            local bytes_before = SPO.rom_read(boundary - 4, 4)
            local bytes_after = SPO.rom_read(boundary, 4)
            
            -- Check if data looks cut off (non-FF followed by sudden FF)
            local has_data_before = false
            local has_ff_after = true
            
            for i = 1, 4 do
                if string.byte(bytes_before, i) ~= 0xFF then
                    has_data_before = true
                end
                if string.byte(bytes_after, i) ~= 0xFF then
                    has_ff_after = false
                end
            end
            
            if has_data_before and has_ff_after then
                table.insert(issues, {
                    type = "potential_truncate",
                    severity = "LOW",
                    bank = bank,
                    boundary = boundary,
                    description = "Data may be truncated at bank boundary",
                    recommendation = "Check if data continues in next bank"
                })
            end
        end
        
        -- Check 3: Header validation (if ROM has standard header)
        local header_offset = 0x7FB0  -- Typical SNES header location
        if rom_size > header_offset + 32 then
            local header_data = SPO.rom_read(header_offset, 32)
            local maker_code = string.sub(header_data, 1, 2)
            local game_code = string.sub(header_data, 3, 6)
            
            -- Check for valid maker code (should be alphanumeric)
            local valid_header = true
            for i = 1, #maker_code do
                local byte = string.byte(maker_code, i)
                if byte < 0x20 or byte > 0x7E then
                    valid_header = false
                    break
                end
            end
            
            if not valid_header then
                table.insert(issues, {
                    type = "invalid_header",
                    severity = "MEDIUM",
                    description = "ROM header appears invalid or non-standard",
                    recommendation = "Verify ROM is not corrupted or is a valid ROM hack"
                })
            end
        end
        
        -- Summary
        local severity_count = { HIGH = 0, MEDIUM = 0, LOW = 0 }
        for _, issue in ipairs(issues) do
            severity_count[issue.severity] = severity_count[issue.severity] + 1
        end
        
        local status = #issues == 0 and "CLEAN" or 
                       (severity_count.HIGH > 0 and "CRITICAL" or 
                        (severity_count.MEDIUM > 0 and "WARNING" or "OK"))
        
        SPO.notify_info(string.format("Corruption scan: %d issues found (%d HIGH, %d MEDIUM)",
            #issues, severity_count.HIGH, severity_count.MEDIUM))
        
        return {
            success = true,
            status = status,
            issues_found = #issues,
            severity_breakdown = severity_count,
            issues = issues,
            recommendations = #issues > 0 and {
                "Review HIGH severity issues immediately",
                "Compare with known good ROM if available",
                "Check backup copies for differences"
            } or {"No corruption detected - ROM appears healthy"}
        }
    end,
    
    -- ------------------------------------------------------------------------
    -- Export asset list
    -- ------------------------------------------------------------------------
    export_asset_list = function(args)
        local format = args and args.format or "json"
        
        SPO.log_info("Generating asset list...")
        
        local assets = {
            graphics = {},
            palettes = {},
            audio = {},
            code = {},
            unknown = {}
        }
        
        local rom_size = SPO.rom_size()
        local bank_size = 0x8000
        
        -- Find graphics patterns
        for _, sig in ipairs(ANALYSIS_CONFIG.graphics_signatures) do
            local results = SPO.find_pattern(sig.pattern)
            if results then
                for _, offset in ipairs(results) do
                    table.insert(assets.graphics, {
                        type = sig.name,
                        description = sig.description,
                        pc_offset = offset,
                        snes_addr = pc_to_snes_addr(offset)
                    })
                end
            end
        end
        
        -- Find palette data (look for BGR555 patterns)
        -- Palettes often have specific byte patterns
        for bank = 0, math.floor(rom_size / bank_size) - 1 do
            local bank_offset = bank * bank_size
            local bank_data = SPO.rom_read(bank_offset, bank_size)
            
            -- Simple heuristic: scan for potential palette data
            -- Real SNES palettes often have low/high byte patterns
            for i = 1, #bank_data - 32, 32 do
                local potential_palette = true
                local color_count = 0
                
                for j = 0, 15 do
                    local low = string.byte(bank_data, i + j * 2)
                    local high = string.byte(bank_data, i + j * 2 + 1)
                    local color = low + (high * 256)
                    
                    -- Valid BGR555 colors should have reasonable values
                    if high > 0x7F then  -- Upper bit should be 0 in BGR555
                        potential_palette = false
                        break
                    end
                end
                
                if potential_palette then
                    table.insert(assets.palettes, {
                        type = "BGR555_Palette",
                        pc_offset = bank_offset + i - 1,
                        snes_addr = pc_to_snes_addr(bank_offset + i - 1),
                        colors = 16
                    })
                end
            end
        end
        
        -- Summary
        local summary = {
            total_assets = #assets.graphics + #assets.palettes + #assets.audio + #assets.code,
            graphics_count = #assets.graphics,
            palette_count = #assets.palettes,
            audio_count = #assets.audio,
            code_entries = #assets.code
        }
        
        SPO.notify_success(string.format("Found %d total assets", summary.total_assets))
        
        return {
            success = true,
            format = format,
            assets = assets,
            summary = summary
        }
    end,
    
    -- ------------------------------------------------------------------------
    -- Compare two ROM versions
    -- ------------------------------------------------------------------------
    compare_versions = function(args)
        local rom1_path = args and args.rom1_path
        local rom2_path = args and args.rom2_path
        
        if not rom1_path or not rom2_path then
            return { success = false, error = "Missing 'rom1_path' or 'rom2_path' argument" }
        end
        
        -- Note: In a real implementation, this would load and compare two ROM files
        -- For this plugin, we simulate the comparison structure
        
        SPO.log_info(string.format("Comparing ROMs: %s vs %s", rom1_path, rom2_path))
        
        -- This is a simulated comparison - in reality, you'd load both ROMs
        local comparison = {
            rom1 = rom1_path,
            rom2 = rom2_path,
            differences = {},
            summary = {
                total_differences = 0,
                bytes_changed = 0,
                new_regions = {},
                removed_regions = {}
            }
        }
        
        -- Simulate finding some differences
        -- In real implementation, this would byte-compare the two ROMs
        SPO.notify_info("ROM comparison would load and compare both files")
        
        return {
            success = true,
            note = "This command requires multi-ROM support. Structure provided for future implementation.",
            comparison = comparison
        }
    end,
    
    -- ------------------------------------------------------------------------
    -- Get analysis cache status
    -- ------------------------------------------------------------------------
    get_cache_status = function(args)
        return {
            success = true,
            cache = {
                compression_blocks_cached = analysis_cache.compression_blocks ~= nil,
                free_regions_cached = analysis_cache.free_regions ~= nil,
                asset_list_cached = analysis_cache.asset_list ~= nil,
                last_rom_size = analysis_cache.last_rom_size
            }
        }
    end,
    
    -- ------------------------------------------------------------------------
    -- Clear analysis cache
    -- ------------------------------------------------------------------------
    clear_cache = function(args)
        analysis_cache = {
            compression_blocks = nil,
            free_regions = nil,
            asset_list = nil,
            last_rom_size = SPO.rom_size()
        }
        
        SPO.notify_success("Analysis cache cleared")
        
        return { success = true }
    end
}
