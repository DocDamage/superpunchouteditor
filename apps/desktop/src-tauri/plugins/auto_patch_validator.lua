-- Super Punch-Out!! Editor - Auto Patch Validator Plugin
-- Quality assurance plugin for validating patches and edits
-- Provides patch validation, conflict detection, stability tests, and changelog generation

PLUGIN_INFO = {
    id = "auto_patch_validator",
    name = "Auto Patch Validator",
    version = "1.0.0",
    author = "Super Punch-Out!! Editor",
    description = "Quality assurance: validate patches, detect conflicts, test stability, generate changelogs",
    api_version = 1,
}

-- ============================================================================
-- Configuration and Constants
-- ============================================================================

local VALIDATOR_CONFIG = {
    -- Known good ROM checksums
    known_checksums = {
        -- Standard Super Punch-Out!! (USA) ROM
        ["A0B1C2D3"] = { region = "USA", version = "1.0", name = "Super Punch-Out!! (U)" },
        -- Other known versions would go here
    },
    
    -- Patch validation rules
    validation_rules = {
        max_patch_size = 4 * 1024 * 1024,  -- 4MB max patch
        min_patch_size = 1,                  -- At least 1 byte
        forbidden_regions = {                -- Regions that shouldn't be patched
            { start = 0x7FDC, size = 4, reason = "ROM header checksum" },
        },
        max_overwrites_per_bank = 100,       -- Warn if too many changes in one bank
    },
    
    -- Stability test thresholds
    stability_thresholds = {
        max_entropy_delta = 2.0,             -- Max allowed entropy change
        min_free_space = 1024,               -- Minimum free space required
        max_bank_usage = 95,                 -- Max bank usage percentage
    }
}

-- Tracking data
local validation_state = {
    rom_loaded = false,
    original_checksum = nil,
    edits = {},
    patches_applied = {},
    validation_history = {},
    stability_tests_run = 0
}

-- ============================================================================
-- Utility Functions
-- ============================================================================

-- Calculate simple checksum of ROM region
local function calculate_checksum(offset, length)
    local data = SPO.rom_read(offset, length)
    local sum = 0
    for i = 1, #data do
        sum = (sum + string.byte(data, i)) & 0xFFFFFFFF
    end
    return string.format("%08X", sum)
end

-- Calculate CRC32 (simplified)
local function calculate_crc(offset, length)
    -- Simplified CRC calculation
    local data = SPO.rom_read(offset, length)
    local crc = 0xFFFFFFFF
    
    for i = 1, #data do
        local byte = string.byte(data, i)
        crc = crc ~ byte
        for _ = 1, 8 do
            if (crc & 1) == 1 then
                crc = (crc >> 1) ~ 0xEDB88320
            else
                crc = crc >> 1
            end
        end
    end
    
    return string.format("%08X", (~crc) & 0xFFFFFFFF)
end

-- Calculate entropy of data
local function calculate_entropy(data)
    if #data == 0 then return 0 end
    
    local freq = {}
    for i = 0, 255 do freq[i] = 0 end
    
    for i = 1, #data do
        local byte = string.byte(data, i)
        freq[byte] = freq[byte] + 1
    end
    
    local entropy = 0
    local n = #data
    
    for i = 0, 255 do
        if freq[i] > 0 then
            local p = freq[i] / n
            entropy = entropy - (p * math.log(p, 2))
        end
    end
    
    return entropy
end

-- Check if two ranges overlap
local function ranges_overlap(start1, size1, start2, size2)
    return start1 < (start2 + size2) and start2 < (start1 + size1)
end

-- Format bytes for display
local function format_bytes(bytes)
    if bytes >= 1024 * 1024 then
        return string.format("%.2f MB", bytes / (1024 * 1024))
    elseif bytes >= 1024 then
        return string.format("%.2f KB", bytes / 1024)
    else
        return string.format("%d bytes", bytes)
    end
end

-- ============================================================================
-- Plugin Lifecycle
-- ============================================================================

function on_init()
    SPO.log_info("=" .. string.rep("=", 60))
    SPO.log_info("Auto Patch Validator v" .. PLUGIN_INFO.version)
    SPO.log_info("Quality assurance tools loaded")
    SPO.log_info("=" .. string.rep("=", 60))
    
    validation_state = {
        rom_loaded = false,
        original_checksum = nil,
        edits = {},
        patches_applied = {},
        validation_history = {},
        stability_tests_run = 0
    }
    
    SPO.notify_success("Auto Patch Validator loaded!")
end

function on_shutdown()
    SPO.log_info("Auto Patch Validator shutting down...")
    SPO.log_info(string.format("Total edits tracked: %d", #validation_state.edits))
end

function on_rom_loaded()
    validation_state.rom_loaded = true
    validation_state.edits = {}
    validation_state.patches_applied = {}
    
    -- Calculate and store original checksum
    local size = SPO.rom_size()
    validation_state.original_checksum = calculate_checksum(0, size)
    
    SPO.log_info(string.format("ROM loaded - Original checksum: %s", validation_state.original_checksum))
    
    -- Validate checksum against known good ROMs
    local known = VALIDATOR_CONFIG.known_checksums[validation_state.original_checksum]
    if known then
        SPO.notify_success(string.format("ROM validated: %s %s", known.name, known.version))
    else
        SPO.notify_info("ROM checksum not in database - may be modified or unknown version")
    end
end

function on_asset_modified()
    -- Track the edit
    local edit = {
        timestamp = os.time(),
        type = "asset_modification",
        details = "Asset was modified"
    }
    table.insert(validation_state.edits, edit)
    
    -- Run quick validation
    local quick_check = COMMANDS._quick_validate()
    if not quick_check.passed then
        SPO.notify_warn("Validation warning: " .. quick_check.message)
    end
end

-- ============================================================================
-- Internal Functions
-- ============================================================================

-- Quick validation check (used internally)
COMMANDS._quick_validate = function()
    local size = SPO.rom_size()
    
    -- Check if ROM size is valid
    if size == 0 then
        return { passed = false, message = "ROM appears to be empty" }
    end
    
    -- Check if ROM is too large (unlikely for SNES)
    if size > 8 * 1024 * 1024 then
        return { passed = false, message = "ROM size exceeds expected maximum" }
    end
    
    return { passed = true, message = "Quick validation passed" }
end

-- ============================================================================
-- Public Commands
-- ============================================================================

COMMANDS = {
    -- ------------------------------------------------------------------------
    -- Validate a patch before applying
    -- ------------------------------------------------------------------------
    validate_patch = function(args)
        local patch_data = args and args.patch_data
        if not patch_data then
            return { success = false, error = "Missing 'patch_data' argument" }
        end
        
        SPO.log_info("Validating patch...")
        
        local validation_result = {
            valid = true,
            warnings = {},
            errors = {},
            info = {},
            patch_analysis = {}
        }
        
        -- Check 1: Patch size
        local patch_size = #patch_data
        if patch_size > VALIDATOR_CONFIG.validation_rules.max_patch_size then
            table.insert(validation_result.errors, {
                code = "PATCH_TOO_LARGE",
                message = string.format("Patch size (%s) exceeds maximum (%s)",
                    format_bytes(patch_size),
                    format_bytes(VALIDATOR_CONFIG.validation_rules.max_patch_size))
            })
            validation_result.valid = false
        elseif patch_size < VALIDATOR_CONFIG.validation_rules.min_patch_size then
            table.insert(validation_result.errors, {
                code = "PATCH_TOO_SMALL",
                message = "Patch appears to be empty"
            })
            validation_result.valid = false
        else
            table.insert(validation_result.info, {
                code = "SIZE_OK",
                message = string.format("Patch size: %s", format_bytes(patch_size))
            })
        end
        
        -- Check 2: Patch format (assume IPS format for this example)
        local is_ips = string.sub(patch_data, 1, 5) == "PATCH"
        local is_ups = string.sub(patch_data, 1, 4) == "UPS1"
        
        if is_ips then
            table.insert(validation_result.info, {
                code = "FORMAT_IPS",
                message = "Detected IPS patch format"
            })
            
            -- Parse IPS header
            local offset = 6  -- Skip "PATCH" header
            local records = 0
            
            while offset < #patch_data - 3 do
                local record_offset = 0
                for i = 0, 2 do
                    record_offset = (record_offset << 8) | string.byte(patch_data, offset + i)
                end
                
                if record_offset == 0x454F46 then  -- "EOF" marker
                    break
                end
                
                local size = (string.byte(patch_data, offset + 3) << 8) | 
                             string.byte(patch_data, offset + 4)
                
                records = records + 1
                
                -- Check for RLE encoding
                if size == 0 then
                    -- RLE record
                    size = (string.byte(patch_data, offset + 5) << 8) | 
                           string.byte(patch_data, offset + 6)
                    table.insert(validation_result.info, {
                        code = "RLE_RECORD",
                        message = string.format("RLE record at $%06X, %d bytes", record_offset, size)
                    })
                    offset = offset + 8
                else
                    offset = offset + 5 + size
                end
                
                -- Check for forbidden regions
                for _, forbidden in ipairs(VALIDATOR_CONFIG.validation_rules.forbidden_regions) do
                    if record_offset >= forbidden.start and 
                       record_offset < (forbidden.start + forbidden.size) then
                        table.insert(validation_result.warnings, {
                            code = "FORBIDDEN_REGION",
                            message = string.format("Patch modifies %s at $%06X", 
                                forbidden.reason, record_offset)
                        })
                    end
                end
            end
            
            table.insert(validation_result.patch_analysis, {
                format = "IPS",
                records = records
            })
            
        elseif is_ups then
            table.insert(validation_result.info, {
                code = "FORMAT_UPS",
                message = "Detected UPS patch format"
            })
        else
            table.insert(validation_result.warnings, {
                code = "UNKNOWN_FORMAT",
                message = "Could not identify patch format"
            })
        end
        
        -- Store validation in history
        table.insert(validation_state.validation_history, {
            timestamp = os.time(),
            result = validation_result
        })
        
        local status = validation_result.valid and "VALID" or "INVALID"
        if #validation_result.warnings > 0 then
            status = validation_result.valid and "VALID_WITH_WARNINGS" or "INVALID"
        end
        
        SPO.notify_info(string.format("Patch validation: %s (%d warnings, %d errors)",
            status, #validation_result.warnings, #validation_result.errors))
        
        return {
            success = true,
            valid = validation_result.valid,
            status = status,
            validation = validation_result
        }
    end,
    
    -- ------------------------------------------------------------------------
    -- Check if multiple patches conflict
    -- ------------------------------------------------------------------------
    check_conflicts = function(args)
        local patches = args and args.patches
        if not patches or #patches < 2 then
            return { success = false, error = "Need at least 2 patches to check conflicts" }
        end
        
        SPO.log_info(string.format("Checking %d patches for conflicts...", #patches))
        
        local conflict_report = {
            patches_analyzed = #patches,
            conflicts_found = 0,
            conflicts = {},
            safe_patches = {},
            recommendations = {}
        }
        
        -- Extract modification ranges from each patch
        local patch_ranges = {}
        
        for i, patch in ipairs(patches) do
            local ranges = {}
            
            -- Parse patch and extract ranges
            -- (Simplified - real implementation would parse actual patch format)
            if patch.modifications then
                for _, mod in ipairs(patch.modifications) do
                    table.insert(ranges, {
                        start = mod.offset,
                        size = mod.size or 1,
                        description = mod.description or "Unknown"
                    })
                end
            end
            
            patch_ranges[i] = {
                name = patch.name or ("Patch " .. i),
                ranges = ranges
            }
        end
        
        -- Check for overlaps between all pairs
        for i = 1, #patch_ranges do
            for j = i + 1, #patch_ranges do
                local patch1 = patch_ranges[i]
                local patch2 = patch_ranges[j]
                
                for _, range1 in ipairs(patch1.ranges) do
                    for _, range2 in ipairs(patch2.ranges) do
                        if ranges_overlap(range1.start, range1.size, range2.start, range2.size) then
                            table.insert(conflict_report.conflicts, {
                                patch1 = patch1.name,
                                patch2 = patch2.name,
                                offset = range1.start,
                                size = math.min(range1.start + range1.size, range2.start + range2.size) - range1.start,
                                severity = "HIGH"
                            })
                            conflict_report.conflicts_found = conflict_report.conflicts_found + 1
                        end
                    end
                end
            end
        end
        
        -- Identify safe patches (no conflicts with others)
        if conflict_report.conflicts_found == 0 then
            for _, patch_range in ipairs(patch_ranges) do
                table.insert(conflict_report.safe_patches, patch_range.name)
            end
            table.insert(conflict_report.recommendations, "All patches are compatible and can be applied safely")
        else
            table.insert(conflict_report.recommendations, "Resolve conflicts before applying patches")
            table.insert(conflict_report.recommendations, "Consider applying patches in order of importance")
        end
        
        SPO.notify_info(string.format("Conflict check: %d conflicts found", conflict_report.conflicts_found))
        
        return {
            success = true,
            has_conflicts = conflict_report.conflicts_found > 0,
            report = conflict_report
        }
    end,
    
    -- ------------------------------------------------------------------------
    -- Run stability tests on current edits
    -- ------------------------------------------------------------------------
    test_patch_stability = function(args)
        local test_level = args and args.level or "standard"  -- quick, standard, thorough
        
        SPO.log_info(string.format("Running %s stability tests...", test_level))
        
        local test_results = {
            tests_run = 0,
            tests_passed = 0,
            tests_failed = 0,
            details = {},
            overall_status = "UNKNOWN"
        }
        
        local size = SPO.rom_size()
        
        -- Test 1: ROM Size Validation
        test_results.tests_run = test_results.tests_run + 1
        if size > 0 and size <= 8 * 1024 * 1024 then
            table.insert(test_results.details, {
                name = "ROM Size",
                passed = true,
                message = string.format("ROM size valid: %s", format_bytes(size))
            })
            test_results.tests_passed = test_results.tests_passed + 1
        else
            table.insert(test_results.details, {
                name = "ROM Size",
                passed = false,
                message = "ROM size out of expected range",
                severity = "CRITICAL"
            })
            test_results.tests_failed = test_results.tests_failed + 1
        end
        
        -- Test 2: Bank Boundary Alignment (important for SNES)
        test_results.tests_run = test_results.tests_run + 1
        local bank_aligned = true
        local misaligned_banks = {}
        
        -- Check key data structures align to bank boundaries
        for bank = 0, math.floor(size / 0x8000) - 1 do
            local bank_start = bank * 0x8000
            local first_bytes = SPO.rom_read(bank_start, 4)
            
            -- Simple heuristic: banks starting with unusual patterns
            -- might indicate misaligned data
            local all_same = true
            local first_byte = string.byte(first_bytes, 1)
            for i = 2, 4 do
                if string.byte(first_bytes, i) ~= first_byte then
                    all_same = false
                    break
                end
            end
            
            if all_same and first_byte ~= 0xFF and first_byte ~= 0x00 then
                table.insert(misaligned_banks, bank)
                bank_aligned = false
            end
        end
        
        if bank_aligned or #misaligned_banks < 3 then
            table.insert(test_results.details, {
                name = "Bank Alignment",
                passed = true,
                message = string.format("%d potential alignment issues", #misaligned_banks)
            })
            test_results.tests_passed = test_results.tests_passed + 1
        else
            table.insert(test_results.details, {
                name = "Bank Alignment",
                passed = false,
                message = string.format("Found %d potentially misaligned banks", #misaligned_banks),
                severity = "WARNING"
            })
            test_results.tests_failed = test_results.tests_failed + 1
        end
        
        -- Test 3: Free Space Availability
        test_results.tests_run = test_results.tests_run + 1
        local free_space_test = COMMANDS._check_free_space()
        
        if free_space_test.total_free >= VALIDATOR_CONFIG.stability_thresholds.min_free_space then
            table.insert(test_results.details, {
                name = "Free Space",
                passed = true,
                message = string.format("%s available for expansion", format_bytes(free_space_test.total_free))
            })
            test_results.tests_passed = test_results.tests_passed + 1
        else
            table.insert(test_results.details, {
                name = "Free Space",
                passed = false,
                message = "Insufficient free space for safe expansion",
                severity = "WARNING"
            })
            test_results.tests_failed = test_results.tests_failed + 1
        end
        
        -- Test 4: Checksum Validation (if we have original)
        if validation_state.original_checksum then
            test_results.tests_run = test_results.tests_run + 1
            local current_checksum = calculate_checksum(0, size)
            
            if current_checksum ~= validation_state.original_checksum then
                table.insert(test_results.details, {
                    name = "ROM Modified",
                    passed = true,
                    message = "ROM has been modified from original"
                })
                test_results.tests_passed = test_results.tests_passed + 1
            else
                table.insert(test_results.details, {
                    name = "ROM Modified",
                    passed = true,
                    message = "ROM matches original checksum"
                })
                test_results.tests_passed = test_results.tests_passed + 1
            end
        end
        
        -- Test 5: Entropy Analysis (thorough only)
        if test_level == "thorough" then
            test_results.tests_run = test_results.tests_run + 1
            local entropy_test = COMMANDS._check_entropy_stability()
            
            if entropy_test.stable then
                table.insert(test_results.details, {
                    name = "Entropy Stability",
                    passed = true,
                    message = "ROM entropy within expected bounds"
                })
                test_results.tests_passed = test_results.tests_passed + 1
            else
                table.insert(test_results.details, {
                    name = "Entropy Stability",
                    passed = false,
                    message = entropy_test.message,
                    severity = "WARNING"
                })
                test_results.tests_failed = test_results.tests_failed + 1
            end
        end
        
        -- Determine overall status
        if test_results.tests_failed == 0 then
            test_results.overall_status = "STABLE"
        elseif test_results.tests_failed <= 1 then
            test_results.overall_status = "MOSTLY_STABLE"
        else
            test_results.overall_status = "UNSTABLE"
        end
        
        validation_state.stability_tests_run = validation_state.stability_tests_run + 1
        
        SPO.notify_info(string.format("Stability tests: %d/%d passed (%s)",
            test_results.tests_passed, test_results.tests_run, test_results.overall_status))
        
        return {
            success = true,
            status = test_results.overall_status,
            results = test_results
        }
    end,
    
    -- ------------------------------------------------------------------------
    -- Generate changelog from edits
    -- ------------------------------------------------------------------------
    generate_changelog = function(args)
        local format = args and args.format or "markdown"
        local include_timestamps = args and args.include_timestamps
        
        SPO.log_info("Generating changelog...")
        
        local changelog = {
            title = "Super Punch-Out!! Editor Changelog",
            generated_at = os.date("%Y-%m-%d %H:%M:%S"),
            entries = {},
            summary = {
                total_edits = #validation_state.edits,
                total_patches = #validation_state.patches_applied,
                validation_tests = #validation_state.validation_history
            }
        }
        
        -- Build changelog entries from edit history
        for i, edit in ipairs(validation_state.edits) do
            local entry = {
                number = i,
                type = edit.type,
                timestamp = edit.timestamp and os.date("%Y-%m-%d %H:%M:%S", edit.timestamp) or "Unknown"
            }
            
            if edit.type == "asset_modification" then
                entry.description = "Modified game asset"
            elseif edit.type == "patch_applied" then
                entry.description = "Applied patch: " .. (edit.patch_name or "Unknown")
            elseif edit.type == "stat_change" then
                entry.description = string.format("Changed %s: %d -> %d", 
                    edit.stat_name or "stat", 
                    edit.old_value or 0, 
                    edit.new_value or 0)
            end
            
            table.insert(changelog.entries, entry)
        end
        
        -- Format output
        local formatted = ""
        
        if format == "markdown" then
            local lines = {}
            table.insert(lines, "# " .. changelog.title)
            table.insert(lines, "")
            table.insert(lines, "Generated: " .. changelog.generated_at)
            table.insert(lines, "")
            table.insert(lines, "## Summary")
            table.insert(lines, "- Total edits: " .. changelog.summary.total_edits)
            table.insert(lines, "- Patches applied: " .. changelog.summary.total_patches)
            table.insert(lines, "- Validation tests: " .. changelog.summary.validation_tests)
            table.insert(lines, "")
            table.insert(lines, "## Changes")
            table.insert(lines, "")
            
            for _, entry in ipairs(changelog.entries) do
                local prefix = "- "
                if include_timestamps then
                    prefix = string.format("- [%s] ", entry.timestamp)
                end
                table.insert(lines, prefix .. entry.description)
            end
            
            formatted = table.concat(lines, "\n")
            
        elseif format == "json" then
            -- JSON format handled by return value
            formatted = "JSON format - see return data"
            
        elseif format == "bbcode" then
            local lines = {}
            table.insert(lines, "[b]" .. changelog.title .. "[/b]")
            table.insert(lines, "Generated: " .. changelog.generated_at)
            table.insert(lines, "")
            table.insert(lines, "[b]Summary[/b]")
            table.insert(lines, "Total edits: " .. changelog.summary.total_edits)
            table.insert(lines, "Patches applied: " .. changelog.summary.total_patches)
            table.insert(lines, "")
            table.insert(lines, "[b]Changes[/b]")
            table.insert(lines, "[list]")
            
            for _, entry in ipairs(changelog.entries) do
                table.insert(lines, "[*]" .. entry.description)
            end
            
            table.insert(lines, "[/list]")
            formatted = table.concat(lines, "\n")
        end
        
        SPO.notify_success(string.format("Changelog generated with %d entries", #changelog.entries))
        
        return {
            success = true,
            format = format,
            changelog = changelog,
            formatted = formatted
        }
    end,
    
    -- ------------------------------------------------------------------------
    -- Estimate final patch size
    -- ------------------------------------------------------------------------
    estimate_patch_size = function(args)
        local format = args and args.format or "ips"  -- ips, ups, bps
        
        SPO.log_info("Estimating patch size...")
        
        -- Count modified bytes
        local modified_bytes = 0
        local modified_regions = {}
        
        for _, edit in ipairs(validation_state.edits) do
            if edit.size then
                modified_bytes = modified_bytes + edit.size
                table.insert(modified_regions, {
                    offset = edit.offset,
                    size = edit.size
                })
            end
        end
        
        -- Estimate patch size based on format
        local estimated_size = 0
        local overhead = 0
        
        if format == "ips" then
            -- IPS: 5 byte header + 5 bytes per record + data
            overhead = 5 + (#modified_regions * 5)
            estimated_size = overhead + modified_bytes
            
        elseif format == "ups" then
            -- UPS: variable encoding, typically smaller than IPS
            overhead = 18  -- Header + footer
            estimated_size = overhead + math.floor(modified_bytes * 1.1)
            
        elseif format == "bps" then
            -- BPS: delta-based, usually smallest
            overhead = 16
            estimated_size = overhead + math.floor(modified_bytes * 0.8)
        end
        
        SPO.notify_info(string.format("Estimated %s patch size: %s", 
            format:upper(), format_bytes(estimated_size)))
        
        return {
            success = true,
            format = format,
            estimated_size = estimated_size,
            estimated_size_formatted = format_bytes(estimated_size),
            modified_bytes = modified_bytes,
            modified_regions = #modified_regions,
            overhead_bytes = overhead,
            compression_potential = format == "bps" and "High" or 
                                   (format == "ups" and "Medium" or "Low")
        }
    end,
    
    -- ------------------------------------------------------------------------
    -- Internal: Check free space
    -- ------------------------------------------------------------------------
    _check_free_space = function()
        local size = SPO.rom_size()
        local bank_size = 0x8000
        local total_free = 0
        
        for bank = 0, math.floor(size / bank_size) - 1 do
            local bank_data = SPO.rom_read(bank * bank_size, bank_size)
            local ff_count = 0
            
            for i = 1, #bank_data do
                if string.byte(bank_data, i) == 0xFF then
                    ff_count = ff_count + 1
                end
            end
            
            total_free = total_free + ff_count
        end
        
        return { total_free = total_free }
    end,
    
    -- ------------------------------------------------------------------------
    -- Internal: Check entropy stability
    -- ------------------------------------------------------------------------
    _check_entropy_stability = function()
        local size = SPO.rom_size()
        local bank_size = 0x8000
        local entropies = {}
        
        for bank = 0, math.floor(size / bank_size) - 1 do
            local bank_data = SPO.rom_read(bank * bank_size, bank_size)
            local entropy = calculate_entropy(bank_data)
            table.insert(entropies, entropy)
        end
        
        -- Check for anomalies
        local avg_entropy = 0
        for _, e in ipairs(entropies) do
            avg_entropy = avg_entropy + e
        end
        avg_entropy = avg_entropy / #entropies
        
        local anomalies = 0
        for _, e in ipairs(entropies) do
            if math.abs(e - avg_entropy) > VALIDATOR_CONFIG.stability_thresholds.max_entropy_delta then
                anomalies = anomalies + 1
            end
        end
        
        return {
            stable = anomalies < 3,
            message = string.format("%d banks with unusual entropy detected", anomalies),
            average_entropy = math.floor(avg_entropy * 100) / 100
        }
    end,
    
    -- ------------------------------------------------------------------------
    -- Get validation state
    -- ------------------------------------------------------------------------
    get_validation_state = function(args)
        return {
            success = true,
            state = {
                rom_loaded = validation_state.rom_loaded,
                original_checksum = validation_state.original_checksum,
                total_edits = #validation_state.edits,
                total_patches = #validation_state.patches_applied,
                stability_tests_run = validation_state.stability_tests_run,
                validation_tests_run = #validation_state.validation_history
            }
        }
    end,
    
    -- ------------------------------------------------------------------------
    -- Clear validation history
    -- ------------------------------------------------------------------------
    clear_history = function(args)
        validation_state.edits = {}
        validation_state.validation_history = {}
        validation_state.patches_applied = {}
        
        SPO.notify_success("Validation history cleared")
        
        return { success = true }
    end
}
