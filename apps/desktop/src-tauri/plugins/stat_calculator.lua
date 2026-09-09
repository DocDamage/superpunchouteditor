-- Super Punch-Out!! Editor - Stat Calculator Plugin
-- Balance analysis and tier list generation for boxer statistics
-- Provides comprehensive stat analysis, CSV import/export, and matchup calculations

PLUGIN_INFO = {
    id = "stat_calculator",
    name = "Stat Calculator",
    version = "1.0.0",
    author = "Super Punch-Out!! Editor",
    description = "Balance analysis, tier lists, and stat manipulation for boxers",
    api_version = 1,
}

-- ============================================================================
-- Configuration and Constants
-- ============================================================================

-- Boxer stat addresses in ROM (example addresses - adjust for actual ROM layout)
local STAT_CONFIG = {
    boxer_table_base = 0x1A0000,     -- Base of boxer stat table
    bytes_per_boxer = 32,             -- Bytes per boxer entry
    num_boxers = 13,                  -- Total boxers in game
    
    -- Stat offsets within each boxer entry
    stat_offsets = {
        health = 0,
        speed = 1,
        power = 2,
        defense = 3,
        recovery = 4,
        stamina = 5,
        aggressiveness = 6,
        counter_skill = 7,
        pattern_complexity = 8,
    },
    
    -- Weight factors for tier calculation
    tier_weights = {
        health = 1.0,
        speed = 1.2,
        power = 1.3,
        defense = 1.1,
        recovery = 0.8,
        stamina = 0.9,
        aggressiveness = 0.7,
        counter_skill = 1.0,
        pattern_complexity = 0.6,
    }
}

-- Boxer database with names and metadata
local BOXERS = {
    { id = 0,  name = "Gabby Jay",          circuit = "Minor",     country = "FR" },
    { id = 1,  name = "Bear Hugger",        circuit = "Minor",     country = "CA" },
    { id = 2,  name = "Piston Hurricane",   circuit = "Minor",     country = "CU" },
    { id = 3,  name = "Bald Bull",          circuit = "Minor",     country = "TR" },
    { id = 4,  name = "Bob Charlie",        circuit = "Major",     country = "JM" },
    { id = 5,  name = "Dragon Chan",        circuit = "Major",     country = "HK" },
    { id = 6,  name = "Masked Club",        circuit = "Major",     country = "??" },
    { id = 7,  name = "Mr. Sandman",        circuit = "Major",     country = "US" },
    { id = 8,  name = "Aran Ryan",          circuit = "World",     country = "IE" },
    { id = 9,  name = "Narcis Prince",      circuit = "World",     country = "GB" },
    { id = 10, name = "Heike Kagero",       circuit = "World",     country = "JP" },
    { id = 11, name = "Mad Clown",          circuit = "World",     country = "IT" },
    { id = 12, name = "Super Macho Man",    circuit = "World",     country = "US" },
}

-- In-memory cache of stats
local stats_cache = {}
local stats_modified = false
local balance_history = {}

-- ============================================================================
-- Utility Functions
-- ============================================================================

-- Get boxer by name or ID
local function get_boxer(identifier)
    if type(identifier) == "number" then
        return BOXERS[identifier + 1]  -- 0-indexed to 1-indexed
    end
    
    local name_lower = string.lower(tostring(identifier))
    for _, boxer in ipairs(BOXERS) do
        if string.lower(boxer.name) == name_lower or
           string.lower(boxer.name:gsub("[%.%s]", "_")) == name_lower or
           string.lower(boxer.name:gsub("[%.%s]", "")) == name_lower then
            return boxer
        end
    end
    return nil
end

-- Read boxer stats from ROM
local function read_boxer_stats(boxer_id)
    local offset = STAT_CONFIG.boxer_table_base + (boxer_id * STAT_CONFIG.bytes_per_boxer)
    local stats = {}
    
    for stat_name, stat_offset in pairs(STAT_CONFIG.stat_offsets) do
        stats[stat_name] = SPO.rom_read_byte(offset + stat_offset)
    end
    
    return stats
end

-- Write boxer stats to ROM
local function write_boxer_stats(boxer_id, stats)
    local offset = STAT_CONFIG.boxer_table_base + (boxer_id * STAT_CONFIG.bytes_per_boxer)
    
    for stat_name, stat_offset in pairs(STAT_CONFIG.stat_offsets) do
        if stats[stat_name] then
            SPO.rom_write_byte(offset + stat_offset, math.min(255, math.max(0, stats[stat_name])))
        end
    end
end

-- Calculate weighted power score for tier ranking
local function calculate_power_score(stats)
    local score = 0
    for stat_name, weight in pairs(STAT_CONFIG.tier_weights) do
        score = score + ((stats[stat_name] or 0) * weight)
    end
    return math.floor(score * 10) / 10
end

-- Get tier from power score
local function score_to_tier(score)
    if score >= 900 then return "S+"
    elseif score >= 800 then return "S"
    elseif score >= 700 then return "A"
    elseif score >= 600 then return "B"
    elseif score >= 500 then return "C"
    elseif score >= 400 then return "D"
    else return "F" end
end

-- Calculate stat standard deviation
local function calculate_std_dev(values, mean)
    local sum_sq_diff = 0
    for _, v in ipairs(values) do
        local diff = v - mean
        sum_sq_diff = sum_sq_diff + (diff * diff)
    end
    return math.sqrt(sum_sq_diff / #values)
end

-- ============================================================================
-- Plugin Lifecycle
-- ============================================================================

function on_init()
    SPO.log_info("=" .. string.rep("=", 60))
    SPO.log_info("Stat Calculator v" .. PLUGIN_INFO.version)
    SPO.log_info("Balance analysis and tier list generation loaded")
    SPO.log_info("=" .. string.rep("=", 60))
    
    -- Initialize cache
    stats_cache = {}
    stats_modified = false
    balance_history = {}
    
    SPO.notify_success("Stat Calculator loaded!")
end

function on_shutdown()
    SPO.log_info("Stat Calculator shutting down...")
    if stats_modified then
        SPO.log_warn("Stats were modified - remember to save your ROM!")
    end
end

function on_rom_loaded()
    -- Clear cache when new ROM is loaded
    stats_cache = {}
    stats_modified = false
    SPO.log_info("ROM loaded - stat cache cleared")
end

-- ============================================================================
-- Commands
-- ============================================================================

COMMANDS = {
    -- ------------------------------------------------------------------------
    -- Analyze balance across all boxers
    -- ------------------------------------------------------------------------
    analyze_balance = function(args)
        SPO.log_info("Analyzing game balance...")
        
        local all_stats = {}
        local stat_values = {}
        
        -- Initialize stat value arrays
        for stat_name, _ in pairs(STAT_CONFIG.stat_offsets) do
            stat_values[stat_name] = {}
        end
        
        -- Read all boxer stats
        for _, boxer in ipairs(BOXERS) do
            local stats = read_boxer_stats(boxer.id)
            stats_cache[boxer.id] = stats
            
            local entry = {
                id = boxer.id,
                name = boxer.name,
                circuit = boxer.circuit,
                stats = stats,
                power_score = calculate_power_score(stats)
            }
            table.insert(all_stats, entry)
            
            -- Collect values for distribution analysis
            for stat_name, value in pairs(stats) do
                table.insert(stat_values[stat_name], value)
            end
        end
        
        -- Calculate statistics for each stat
        local stat_analysis = {}
        for stat_name, values in pairs(stat_values) do
            -- Sort for median calculation
            table.sort(values)
            
            local sum = 0
            for _, v in ipairs(values) do
                sum = sum + v
            end
            local mean = sum / #values
            
            stat_analysis[stat_name] = {
                min = values[1],
                max = values[#values],
                mean = math.floor(mean * 10) / 10,
                median = values[math.floor(#values / 2) + 1],
                std_dev = math.floor(calculate_std_dev(values, mean) * 10) / 10,
                range = values[#values] - values[1]
            }
        end
        
        -- Find outliers (boxers with extreme stats)
        local outliers = {}
        for _, boxer_data in ipairs(all_stats) do
            for stat_name, value in pairs(boxer_data.stats) do
                local analysis = stat_analysis[stat_name]
                local z_score = math.abs(value - analysis.mean) / (analysis.std_dev > 0 and analysis.std_dev or 1)
                
                if z_score > 2 then
                    table.insert(outliers, {
                        boxer = boxer_data.name,
                        stat = stat_name,
                        value = value,
                        mean = analysis.mean,
                        z_score = math.floor(z_score * 100) / 100,
                        type = value > analysis.mean and "HIGH" or "LOW"
                    })
                end
            end
        end
        
        -- Identify imbalances
        local imbalances = {}
        
        -- Check circuit balance
        local circuit_stats = {}
        for _, boxer_data in ipairs(all_stats) do
            local circuit = boxer_data.circuit
            circuit_stats[circuit] = circuit_stats[circuit] or { total = 0, count = 0, boxers = {} }
            circuit_stats[circuit].total = circuit_stats[circuit].total + boxer_data.power_score
            circuit_stats[circuit].count = circuit_stats[circuit].count + 1
            table.insert(circuit_stats[circuit].boxers, boxer_data.name)
        end
        
        local circuit_averages = {}
        for circuit, data in pairs(circuit_stats) do
            circuit_averages[circuit] = {
                average = math.floor((data.total / data.count) * 10) / 10,
                boxers = data.boxers
            }
        end
        
        -- Check if any circuit is too strong/weak
        local avg_scores = {}
        for _, avg in pairs(circuit_averages) do
            table.insert(avg_scores, avg.average)
        end
        table.sort(avg_scores)
        local circuit_spread = avg_scores[#avg_scores] - avg_scores[1]
        
        if circuit_spread > 200 then
            table.insert(imbalances, {
                type = "circuit_balance",
                severity = circuit_spread > 300 and "HIGH" or "MEDIUM",
                description = string.format("Large power gap between circuits (spread: %.1f)", circuit_spread),
                circuit_averages = circuit_averages
            })
        end
        
        -- Store in history
        table.insert(balance_history, {
            timestamp = os.time(),
            analysis = stat_analysis,
            imbalances = imbalances
        })
        
        SPO.notify_success("Balance analysis complete - found " .. #imbalances .. " imbalances")
        
        return {
            success = true,
            stat_analysis = stat_analysis,
            outliers = outliers,
            imbalances = imbalances,
            circuit_averages = circuit_averages,
            total_boxers = #BOXERS
        }
    end,
    
    -- ------------------------------------------------------------------------
    -- Generate tier list based on power scores
    -- ------------------------------------------------------------------------
    generate_tier_list = function(args)
        SPO.log_info("Generating tier list...")
        
        local boxer_scores = {}
        
        -- Calculate scores for all boxers
        for _, boxer in ipairs(BOXERS) do
            local stats = stats_cache[boxer.id] or read_boxer_stats(boxer.id)
            local score = calculate_power_score(stats)
            
            table.insert(boxer_scores, {
                id = boxer.id,
                name = boxer.name,
                circuit = boxer.circuit,
                score = score,
                tier = score_to_tier(score),
                stats = stats
            })
        end
        
        -- Sort by score descending
        table.sort(boxer_scores, function(a, b) return a.score > b.score end)
        
        -- Group by tier
        local tiers = {}
        local tier_order = {"S+", "S", "A", "B", "C", "D", "F"}
        
        for _, tier in ipairs(tier_order) do
            tiers[tier] = {}
        end
        
        for _, boxer in ipairs(boxer_scores) do
            table.insert(tiers[boxer.tier], boxer)
        end
        
        -- Build formatted tier list
        local tier_list = {}
        for _, tier in ipairs(tier_order) do
            if #tiers[tier] > 0 then
                table.insert(tier_list, {
                    tier = tier,
                    boxers = tiers[tier],
                    count = #tiers[tier]
                })
            end
        end
        
        -- Calculate tier distribution
        local distribution = {}
        for _, tier in ipairs(tier_order) do
            distribution[tier] = #tiers[tier]
        end
        
        SPO.notify_success("Tier list generated - " .. #boxer_scores .. " boxers ranked")
        
        return {
            success = true,
            tier_list = tier_list,
            distribution = distribution,
            top_boxer = boxer_scores[1],
            bottom_boxer = boxer_scores[#boxer_scores],
            average_score = (function()
                local sum = 0
                for _, b in ipairs(boxer_scores) do
                    sum = sum + b.score
                end
                return math.floor((sum / #boxer_scores) * 10) / 10
            end)()
        }
    end,
    
    -- ------------------------------------------------------------------------
    -- Suggest balance patches
    -- ------------------------------------------------------------------------
    suggest_balance_patch = function(args)
        local target_balance = args and args.target or "circuit"
        
        SPO.log_info("Generating balance patch suggestions...")
        
        local suggestions = {
            target = target_balance,
            changes = {},
            reasoning = {}
        }
        
        if target_balance == "circuit" then
            -- Suggest changes to balance circuits
            local circuit_totals = {}
            
            for _, boxer in ipairs(BOXERS) do
                local stats = stats_cache[boxer.id] or read_boxer_stats(boxer.id)
                local score = calculate_power_score(stats)
                
                circuit_totals[boxer.circuit] = circuit_totals[boxer.circuit] or { total = 0, count = 0 }
                circuit_totals[boxer.circuit].total = circuit_totals[boxer.circuit].total + score
                circuit_totals[boxer.circuit].count = circuit_totals[boxer.circuit].count + 1
            end
            
            -- Calculate target average (overall mean)
            local grand_total = 0
            local total_count = 0
            for _, data in pairs(circuit_totals) do
                grand_total = grand_total + data.total
                total_count = total_count + data.count
            end
            local target_avg = grand_total / total_count
            
            -- Suggest adjustments for each circuit
            for circuit, data in pairs(circuit_totals) do
                local current_avg = data.total / data.count
                local diff = target_avg - current_avg
                local adjustment = math.floor(diff / 9)  -- Spread across 9 stats
                
                if math.abs(adjustment) > 5 then
                    table.insert(suggestions.changes, {
                        target = circuit,
                        type = "circuit_adjustment",
                        current_avg = math.floor(current_avg * 10) / 10,
                        target_avg = math.floor(target_avg * 10) / 10,
                        suggested_adjustment = adjustment,
                        apply_to = "all_stats"
                    })
                end
            end
            
            table.insert(suggestions.reasoning, 
                string.format("Target circuit average: %.1f", target_avg))
        end
        
        -- Find over/under powered individual boxers
        local all_scores = {}
        for _, boxer in ipairs(BOXERS) do
            local stats = stats_cache[boxer.id] or read_boxer_stats(boxer.id)
            local score = calculate_power_score(stats)
            table.insert(all_scores, score)
        end
        table.sort(all_scores)
        local median = all_scores[math.floor(#all_scores / 2) + 1]
        
        for _, boxer in ipairs(BOXERS) do
            local stats = stats_cache[boxer.id] or read_boxer_stats(boxer.id)
            local score = calculate_power_score(stats)
            local diff = score - median
            
            if math.abs(diff) > 150 then
                local direction = diff > 0 and "reduce" or "increase"
                table.insert(suggestions.changes, {
                    target = boxer.name,
                    type = "individual_adjustment",
                    current_score = score,
                    median_score = median,
                    deviation = math.floor(diff * 10) / 10,
                    suggestion = direction .. " power/stats by " .. math.floor(math.abs(diff) / 9)
                })
            end
        end
        
        SPO.notify_info(string.format("Generated %d balance suggestions", #suggestions.changes))
        
        return {
            success = true,
            suggestions = suggestions
        }
    end,
    
    -- ------------------------------------------------------------------------
    -- Export stats to CSV
    -- ------------------------------------------------------------------------
    export_stats_csv = function(args)
        SPO.log_info("Exporting boxer stats to CSV...")
        
        local csv_lines = {}
        
        -- Header
        local headers = {"Name", "ID", "Circuit", "Country", "PowerScore"}
        for stat_name, _ in pairs(STAT_CONFIG.stat_offsets) do
            table.insert(headers, stat_name:gsub("_", " "):gsub("^%l", string.upper))
        end
        table.insert(csv_lines, table.concat(headers, ","))
        
        -- Data rows
        for _, boxer in ipairs(BOXERS) do
            local stats = stats_cache[boxer.id] or read_boxer_stats(boxer.id)
            local score = calculate_power_score(stats)
            
            local row = {
                boxer.name,
                tostring(boxer.id),
                boxer.circuit,
                boxer.country,
                tostring(score)
            }
            
            for stat_name, _ in pairs(STAT_CONFIG.stat_offsets) do
                table.insert(row, tostring(stats[stat_name] or 0))
            end
            
            table.insert(csv_lines, table.concat(row, ","))
        end
        
        local csv_content = table.concat(csv_lines, "\n")
        
        SPO.notify_success("Exported " .. #BOXERS .. " boxers to CSV format")
        
        return {
            success = true,
            csv = csv_content,
            row_count = #BOXERS,
            column_count = #headers
        }
    end,
    
    -- ------------------------------------------------------------------------
    -- Import stats from CSV
    -- ------------------------------------------------------------------------
    import_stats_csv = function(args)
        local csv_data = args and args.csv_data
        if not csv_data then
            return { success = false, error = "Missing 'csv_data' argument" }
        end
        
        SPO.log_info("Importing boxer stats from CSV...")
        
        local lines = {}
        for line in string.gmatch(csv_data, "[^\r\n]+") do
            table.insert(lines, line)
        end
        
        if #lines < 2 then
            return { success = false, error = "CSV must have header and at least one data row" }
        end
        
        -- Parse header
        local header = {}
        for field in string.gmatch(lines[1], "[^,]+") do
            table.insert(header, string.lower(field:gsub(" ", "_")))
        end
        
        local imported = 0
        local errors = {}
        
        -- Parse data rows
        for i = 2, #lines do
            local line = lines[i]
            local fields = {}
            for field in string.gmatch(line, "[^,]+") do
                table.insert(fields, field)
            end
            
            -- Find boxer name
            local name_idx = nil
            for j, h in ipairs(header) do
                if h == "name" then
                    name_idx = j
                    break
                end
            end
            
            if name_idx and fields[name_idx] then
                local boxer = get_boxer(fields[name_idx])
                
                if boxer then
                    local new_stats = {}
                    for j, h in ipairs(header) do
                        if STAT_CONFIG.stat_offsets[h] then
                            new_stats[h] = tonumber(fields[j]) or 0
                        end
                    end
                    
                    write_boxer_stats(boxer.id, new_stats)
                    stats_cache[boxer.id] = new_stats
                    imported = imported + 1
                    stats_modified = true
                else
                    table.insert(errors, "Unknown boxer: " .. fields[name_idx])
                end
            end
        end
        
        SPO.notify_success(string.format("Imported stats for %d boxers", imported))
        
        return {
            success = true,
            imported = imported,
            errors = errors,
            total_rows = #lines - 1
        }
    end,
    
    -- ------------------------------------------------------------------------
    -- Calculate win probability between two boxers
    -- ------------------------------------------------------------------------
    calculate_win_probability = function(args)
        local boxer1_name = args and args.boxer1
        local boxer2_name = args and args.boxer2
        
        if not boxer1_name or not boxer2_name then
            return { success = false, error = "Missing 'boxer1' or 'boxer2' argument" }
        end
        
        local boxer1 = get_boxer(boxer1_name)
        local boxer2 = get_boxer(boxer2_name)
        
        if not boxer1 then
            return { success = false, error = "Unknown boxer: " .. boxer1_name }
        end
        if not boxer2 then
            return { success = false, error = "Unknown boxer: " .. boxer2_name }
        end
        
        local stats1 = stats_cache[boxer1.id] or read_boxer_stats(boxer1.id)
        local stats2 = stats_cache[boxer2.id] or read_boxer_stats(boxer2.id)
        
        -- Calculate matchup scores
        local matchup = {
            boxer1 = boxer1.name,
            boxer2 = boxer2.name,
            comparisons = {},
            advantages = {}
        }
        
        -- Head-to-head stat comparisons
        local comparisons = {
            { name = "Power vs Defense", b1 = stats1.power, b2 = stats2.defense, weight = 1.5 },
            { name = "Speed vs Counter", b1 = stats1.speed, b2 = stats2.counter_skill, weight = 1.2 },
            { name = "Stamina vs Aggression", b1 = stats1.stamina, b2 = stats2.aggressiveness, weight = 1.0 },
            { name = "Recovery Battle", b1 = stats1.recovery, b2 = stats2.recovery, weight = 0.8 },
        }
        
        local b1_score = 0
        local b2_score = 0
        
        for _, comp in ipairs(comparisons) do
            local diff = comp.b1 - comp.b2
            local advantage = nil
            
            if diff > 5 then
                advantage = boxer1.name
                b1_score = b1_score + comp.weight
            elseif diff < -5 then
                advantage = boxer2.name
                b2_score = b2_score + comp.weight
            end
            
            table.insert(matchup.comparisons, {
                category = comp.name,
                boxer1_value = comp.b1,
                boxer2_value = comp.b2,
                difference = diff,
                advantage = advantage
            })
        end
        
        -- Calculate win probability using Elo-like formula
        local total_score = b1_score + b2_score
        local b1_prob, b2_prob
        
        if total_score == 0 then
            b1_prob = 50
            b2_prob = 50
        else
            b1_prob = math.floor((b1_score / total_score) * 100)
            b2_prob = 100 - b1_prob
        end
        
        matchup.probability = {
            [boxer1.name] = b1_prob,
            [boxer2.name] = b2_prob
        }
        
        matchup.predicted_winner = b1_prob > b2_prob and boxer1.name or boxer2.name
        matchup.confidence = math.abs(b1_prob - 50) * 2  -- 0-100 scale
        
        SPO.notify_info(string.format("Matchup: %s vs %s - Predicted: %s (%d%%)",
            boxer1.name, boxer2.name, matchup.predicted_winner, 
            matchup.probability[matchup.predicted_winner]))
        
        return {
            success = true,
            matchup = matchup
        }
    end,
    
    -- ------------------------------------------------------------------------
    -- Get individual boxer stats
    -- ------------------------------------------------------------------------
    get_boxer_stats = function(args)
        local boxer_name = args and args.boxer_name
        if not boxer_name then
            return { success = false, error = "Missing 'boxer_name' argument" }
        end
        
        local boxer = get_boxer(boxer_name)
        if not boxer then
            return { success = false, error = "Unknown boxer: " .. boxer_name }
        end
        
        local stats = stats_cache[boxer.id] or read_boxer_stats(boxer.id)
        local power_score = calculate_power_score(stats)
        
        return {
            success = true,
            boxer = {
                id = boxer.id,
                name = boxer.name,
                circuit = boxer.circuit,
                country = boxer.country
            },
            stats = stats,
            power_score = power_score,
            tier = score_to_tier(power_score)
        }
    end,
    
    -- ------------------------------------------------------------------------
    -- Update individual boxer stats
    -- ------------------------------------------------------------------------
    update_boxer_stats = function(args)
        local boxer_name = args and args.boxer_name
        local new_stats = args and args.stats
        
        if not boxer_name or not new_stats then
            return { success = false, error = "Missing 'boxer_name' or 'stats' argument" }
        end
        
        local boxer = get_boxer(boxer_name)
        if not boxer then
            return { success = false, error = "Unknown boxer: " .. boxer_name }
        end
        
        -- Validate stat values
        for stat_name, value in pairs(new_stats) do
            if STAT_CONFIG.stat_offsets[stat_name] then
                if type(value) ~= "number" or value < 0 or value > 255 then
                    return { success = false, error = "Invalid value for " .. stat_name .. ": must be 0-255" }
                end
            end
        end
        
        write_boxer_stats(boxer.id, new_stats)
        stats_cache[boxer.id] = new_stats
        stats_modified = true
        
        SPO.notify_success(string.format("Updated stats for %s", boxer.name))
        
        return {
            success = true,
            boxer = boxer.name,
            updated_stats = new_stats
        }
    end
}
