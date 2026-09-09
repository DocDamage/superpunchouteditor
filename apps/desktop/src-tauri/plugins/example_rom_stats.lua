-- Super Punch-Out!! Editor - ROM Statistics Plugin
-- Analyzes ROM contents and provides statistics

PLUGIN_INFO = {
    id = "example_rom_stats",
    name = "ROM Statistics",
    version = "1.0.0",
    author = "Super Punch-Out!! Editor",
    description = "Analyzes ROM contents and provides detailed statistics",
    api_version = 1,
}

function on_init()
    SPO.log_info("ROM Statistics plugin loaded")
end

function on_shutdown()
    SPO.log_info("ROM Statistics plugin shutting down")
end

-- Helper function to analyze bank usage
local function analyze_bank(bank_num)
    local bank_start = bank_num * 0x8000
    local bank_data = SPO.rom_read(bank_start, 0x8000)
    
    local zero_count = 0
    local ff_count = 0
    local other_count = 0
    
    for i = 1, #bank_data do
        local byte = string.byte(bank_data, i)
        if byte == 0 then
            zero_count = zero_count + 1
        elseif byte == 0xFF then
            ff_count = ff_count + 1
        else
            other_count = other_count + 1
        end
    end
    
    return {
        bank = bank_num,
        zero_bytes = zero_count,
        ff_bytes = ff_count,
        used_bytes = other_count,
        usage_percent = math.floor(other_count / 0x8000 * 1000) / 10
    }
end

-- Detect likely graphics data by looking for common patterns
local function detect_graphics_regions()
    local patterns_found = {}
    
    -- Look for Mode 7 graphics header pattern
    local mode7_results = SPO.find_pattern({0x00, 0x00, 0x00, 0x80})
    if mode7_results and #mode7_results > 0 then
        table.insert(patterns_found, {
            name = "Possible Mode 7 Headers",
            count = #mode7_results,
            locations = mode7_results
        })
    end
    
    return patterns_found
end

COMMANDS = {
    -- Get overall ROM statistics
    get_stats = function(args)
        local size = SPO.rom_size()
        local num_banks = math.floor(size / 0x8000)
        
        -- Sample a few banks to estimate usage
        local sampled_banks = {}
        local sample_interval = math.max(1, math.floor(num_banks / 8))
        
        for i = 0, num_banks - 1, sample_interval do
            table.insert(sampled_banks, analyze_bank(i))
        end
        
        -- Calculate average usage
        local total_usage = 0
        for _, bank in ipairs(sampled_banks) do
            total_usage = total_usage + bank.usage_percent
        end
        local avg_usage = math.floor(total_usage / #sampled_banks * 10) / 10
        
        return {
            success = true,
            rom_size = size,
            total_banks = num_banks,
            banks_sampled = #sampled_banks,
            estimated_usage_percent = avg_usage,
            sample_details = sampled_banks
        }
    end,
    
    -- Analyze a specific bank in detail
    analyze_bank = function(args)
        local bank_num = args and args.bank
        if not bank_num then
            return {
                success = false,
                error = "Missing 'bank' argument"
            }
        end
        
        bank_num = tonumber(bank_num)
        local num_banks = math.floor(SPO.rom_size() / 0x8000)
        
        if bank_num < 0 or bank_num >= num_banks then
            return {
                success = false,
                error = "Invalid bank number. ROM has " .. num_banks .. " banks (0-" .. (num_banks-1) .. ")"
            }
        end
        
        local stats = analyze_bank(bank_num)
        local graphics = detect_graphics_regions()
        
        return {
            success = true,
            bank_stats = stats,
            detected_patterns = graphics
        }
    end,
    
    -- Find potential free space
    find_free_space = function(args)
        local min_size = args and args.min_size or 256
        local results = {}
        local num_banks = math.floor(SPO.rom_size() / 0x8000)
        
        for bank = 0, num_banks - 1 do
            local bank_start = bank * 0x8000
            local bank_data = SPO.rom_read(bank_start, 0x8000)
            
            local consecutive_ff = 0
            local ff_start = 0
            
            for i = 1, #bank_data do
                local byte = string.byte(bank_data, i)
                
                if byte == 0xFF then
                    if consecutive_ff == 0 then
                        ff_start = i - 1
                    end
                    consecutive_ff = consecutive_ff + 1
                else
                    if consecutive_ff >= min_size then
                        table.insert(results, {
                            bank = bank,
                            offset = bank_start + ff_start,
                            size = consecutive_ff,
                            snes_addr = { bank = 0x80 + bank, addr = 0x8000 + ff_start }
                        })
                    end
                    consecutive_ff = 0
                end
            end
            
            -- Check if we ended with free space
            if consecutive_ff >= min_size then
                table.insert(results, {
                    bank = bank,
                    offset = bank_start + ff_start,
                    size = consecutive_ff,
                    snes_addr = { bank = 0x80 + bank, addr = 0x8000 + ff_start }
                })
            end
        end
        
        return {
            success = true,
            free_regions = results,
            total_free = (function()
                local total = 0
                for _, r in ipairs(results) do
                    total = total + r.size
                end
                return total
            end)()
        }
    end
}
