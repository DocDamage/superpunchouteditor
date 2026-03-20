-- Super Punch-Out!! Editor - Interactive Tutorial Plugin
-- Step-by-step interactive guides for learning the editor
-- Provides contextual help, tutorials, and tips for users

PLUGIN_INFO = {
    id = "interactive_tutorial",
    name = "Interactive Tutorial",
    version = "1.0.0",
    author = "Super Punch-Out!! Editor",
    description = "Interactive tutorials and contextual help for learning the editor",
    api_version = 1,
}

-- ============================================================================
-- Configuration and Tutorial Data
-- ============================================================================

-- Tutorial definitions
local TUTORIALS = {
    {
        id = "first_steps",
        name = "First Steps",
        description = "Learn the basics of the Super Punch-Out!! Editor",
        difficulty = "Beginner",
        estimated_time = "5 minutes",
        steps = {
            {
                id = "welcome",
                title = "Welcome!",
                content = "Welcome to the Super Punch-Out!! Editor! This tutorial will guide you through the basics.",
                action = "none",
                tip = "You can pause or exit this tutorial at any time using the buttons below."
            },
            {
                id = "load_rom",
                title = "Loading a ROM",
                content = "To begin editing, you need to load a Super Punch-Out!! ROM file. Go to File > Open ROM or press Ctrl+O.",
                action = "open_dialog",
                action_data = { type = "rom_loader" },
                tip = "Make sure you have a legally obtained ROM file of Super Punch-Out!!"
            },
            {
                id = "explore_ui",
                title = "Exploring the Interface",
                content = "The main window has several panels: the Boxer List on the left, Asset Browser in the center, and Properties on the right.",
                action = "highlight",
                action_data = { element = "main_ui" },
                tip = "You can rearrange panels by dragging their headers."
            },
            {
                id = "select_boxer",
                title = "Selecting a Boxer",
                content = "Click on a boxer in the Boxer List to view and edit their data. Try selecting Gabby Jay to start.",
                action = "require_selection",
                action_data = { type = "boxer", id = "gabby_jay" },
                tip = "Each boxer has unique stats, graphics, and AI patterns."
            },
            {
                id = "basic_editing",
                title = "Making Your First Edit",
                content = "Great! Now let's make a simple change. Try modifying Gabby Jay's Health stat in the Properties panel.",
                action = "require_edit",
                action_data = { type = "stat", field = "health" },
                tip = "Remember to save your changes! Use Ctrl+S or File > Save."
            },
            {
                id = "save_rom",
                title = "Saving Your Work",
                content = "Do not forget to save! Use File > Save or Ctrl+S to save your modified ROM.",
                action = "notification",
                action_data = { type = "info", message = "Press Ctrl+S to save" },
                tip = "It is good practice to keep backups of your original ROM."
            },
            {
                id = "congratulations",
                title = "Congratulations!",
                content = "You have completed the First Steps tutorial! You are ready to start exploring the editor on your own.",
                action = "none",
                tip = "Check out the other tutorials to learn more advanced features!"
            }
        }
    },
    {
        id = "palette_editing",
        name = "Palette Editing",
        description = "Learn how to edit boxer color palettes",
        difficulty = "Beginner",
        estimated_time = "8 minutes",
        steps = {
            {
                id = "palette_intro",
                title = "Understanding Palettes",
                content = "SNES games use 15-bit BGR555 color format. Each palette has 16 colors, with color 0 typically being transparent.",
                action = "none",
                tip = "The SNES can display up to 256 colors on screen at once using multiple palettes."
            },
            {
                id = "open_palette_editor",
                title = "Opening the Palette Editor",
                content = "Select a boxer, then click on the 'Palette' tab in the Asset Browser to open the palette editor.",
                action = "require_tab",
                action_data = { tab = "palette" },
                tip = "You can also access palettes from the boxer's context menu."
            },
            {
                id = "select_color",
                title = "Selecting a Color",
                content = "Click on any color swatch in the palette to select it. The selected color will be highlighted.",
                action = "require_action",
                action_data = { type = "select_color" },
                tip = "Color 0 is usually transparent in sprites. Be careful when modifying it!"
            },
            {
                id = "edit_color",
                title = "Editing a Color",
                content = "Use the color picker or enter RGB values directly to change the selected color. Watch the preview update in real-time!",
                action = "require_action",
                action_data = { type = "modify_color" },
                tip = "The preview shows how your boxer will look with the new colors."
            },
            {
                id = "copy_palette",
                title = "Copying Palettes",
                content = "You can copy an entire palette to use as a starting point. Right-click on a palette and select 'Copy'.",
                action = "notification",
                action_data = { type = "info", message = "Right-click a palette for more options" },
                tip = "This is useful for creating alternate costumes or special effects."
            },
            {
                id = "export_import",
                title = "Export and Import",
                content = "Palettes can be exported to various formats (ACT, JSON, GPL) and imported back. Use the buttons in the toolbar.",
                action = "none",
                tip = "Export palettes to share with others or use in other projects!"
            }
        }
    },
    {
        id = "sprite_import",
        name = "Sprite Import",
        description = "Learn how to import custom graphics",
        difficulty = "Intermediate",
        estimated_time = "12 minutes",
        steps = {
            {
                id = "sprite_formats",
                title = "Supported Formats",
                content = "The editor supports importing PNG, BMP, and GIF images. For best results, use PNG with transparency.",
                action = "none",
                tip = "Sprites should match the SNES sprite size limits for best compatibility."
            },
            {
                id = "prepare_image",
                title = "Preparing Your Image",
                content = "Before importing, ensure your image uses only 15 colors (plus transparency) and matches the required dimensions.",
                action = "notification",
                action_data = { type = "info", message = "Use an image editor to prepare your sprite" },
                tip = "Most boxer sprites are 32x64 pixels, but this varies by animation frame."
            },
            {
                id = "open_import",
                title = "Opening the Import Dialog",
                content = "Select a boxer and animation frame, then click Import > Sprite or press Ctrl+Shift+I.",
                action = "open_dialog",
                action_data = { type = "sprite_import" },
                tip = "You can also drag and drop image files directly onto the sprite view."
            },
            {
                id = "import_settings",
                title = "Import Settings",
                content = "Configure the import options: palette mapping, transparency handling, and dithering. Preview shows the result.",
                action = "require_action",
                action_data = { type = "configure_import" },
                tip = "Use 'Match to Existing Palette' to keep the original game's color scheme."
            },
            {
                id = "apply_import",
                title = "Applying the Import",
                content = "Click 'Import' to apply your sprite. The editor will automatically convert it to SNES format.",
                action = "require_action",
                action_data = { type = "confirm_import" },
                tip = "If the import does not look right, check your color count and try again."
            }
        }
    },
    {
        id = "ai_behavior",
        name = "AI Behavior",
        description = "Understanding and editing boxer AI patterns",
        difficulty = "Advanced",
        estimated_time = "15 minutes",
        steps = {
            {
                id = "ai_overview",
                title = "AI Overview",
                content = "Each boxer has unique AI that controls their behavior: attack patterns, dodging, blocking, and counter-attacks.",
                action = "none",
                tip = "AI editing can dramatically change the game's difficulty!"
            },
            {
                id = "ai_structure",
                title = "AI Data Structure",
                content = "AI is stored as a sequence of behavior states. Each state defines what the boxer does and when they transition to another state.",
                action = "none",
                tip = "Understanding hex editing basics helps with advanced AI modification."
            },
            {
                id = "open_ai_editor",
                title = "Opening the AI Editor",
                content = "Select a boxer and switch to the 'AI' tab to view their behavior patterns.",
                action = "require_tab",
                action_data = { tab = "ai" },
                tip = "AI editing is powerful but can break the game if done incorrectly. Always backup!"
            },
            {
                id = "ai_states",
                title = "Understanding States",
                content = "Each state has: Trigger (what activates it), Action (what happens), and Transition (what state comes next).",
                action = "highlight",
                action_data = { element = "ai_state_table" },
                tip = "Hover over state values to see descriptions of what they do."
            },
            {
                id = "edit_ai",
                title = "Editing AI Values",
                content = "Double-click any value to edit it. Start with small changes like attack frequency or block chance.",
                action = "require_edit",
                action_data = { type = "ai_value" },
                tip = "Test your changes frequently! Some AI values can make the game crash."
            },
            {
                id = "common_patterns",
                title = "Common AI Patterns",
                content = "Common patterns: 0x01 = Idle, 0x10 = Jab, 0x20 = Hook, 0x30 = Uppercut, 0x80 = Block, 0x90 = Dodge.",
                action = "none",
                tip = "Study the original boxers' AI to learn effective patterns."
            }
        }
    }
}

-- Contextual tips database
local CONTEXTUAL_TIPS = {
    rom_loader = {
        "Tip: Make sure your ROM is the USA version for best compatibility.",
        "Tip: ROM files should have .sfc or .smc extension.",
        "Tip: Keep a backup of your original ROM before editing!"
    },
    boxer_editor = {
        "Tip: Click on a stat name to see its description.",
        "Tip: Use Ctrl+Z to undo your last change.",
        "Tip: Right-click a boxer to export their data."
    },
    palette_editor = {
        "Tip: Color 0 is usually transparent - be careful when changing it!",
        "Tip: Use the color picker to match colors from reference images.",
        "Tip: You can copy palettes between boxers using copy/paste."
    },
    sprite_editor = {
        "Tip: Sprites use SNES native format - 4bpp planar.",
        "Tip: Zoom with mouse wheel or Ctrl++ / Ctrl+-.",
        "Tip: Use the grid overlay to align tiles properly."
    },
    ai_editor = {
        "Tip: AI editing is advanced - make small changes and test often!",
        "Tip: Save different AI versions to compare behaviors.",
        "Tip: Some AI values can crash the game - always keep backups."
    },
    general = {
        "Tip: Press F1 to access help at any time.",
        "Tip: Use plugins to extend the editor's functionality.",
        "Tip: Check the log panel for detailed operation information.",
        "Tip: Export your work frequently to avoid losing progress.",
        "Tip: Join the community discord for help and to share your hacks!"
    }
}

-- Active tutorial state
local active_tutorial = nil
local current_step = 0
local tutorial_history = {}

-- ============================================================================
-- Utility Functions
-- ============================================================================

-- Get tutorial by ID
local function get_tutorial(tutorial_id)
    for _, tutorial in ipairs(TUTORIALS) do
        if tutorial.id == tutorial_id then
            return tutorial
        end
    end
    return nil
end

-- Send notification with proper formatting
local function send_tutorial_notification(step, step_num, total_steps)
    local progress = string.format("[%d/%d] ", step_num, total_steps)
    local message = progress .. step.title .. "\n\n" .. step.content
    
    SPO.notify_info(message)
    
    if step.tip then
        SPO.log_info("TIP: " .. step.tip)
    end
end

-- ============================================================================
-- Plugin Lifecycle
-- ============================================================================

function on_init()
    SPO.log_info("=" .. string.rep("=", 60))
    SPO.log_info("Interactive Tutorial v" .. PLUGIN_INFO.version)
    SPO.log_info("Interactive guides and contextual help loaded")
    SPO.log_info("=" .. string.rep("=", 60))
    
    active_tutorial = nil
    current_step = 0
    tutorial_history = {}
    
    SPO.notify_success("Interactive Tutorial loaded! Type 'list_tutorials' to see available tutorials.")
end

function on_shutdown()
    if active_tutorial then
        SPO.log_info(string.format("Tutorial '%s' was in progress at step %d", 
            active_tutorial.id, current_step))
    end
    SPO.log_info("Interactive Tutorial shutting down...")
end

function on_rom_loaded()
    SPO.log_info("ROM loaded - use 'start_tutorial' command to begin a guided tour")
end

-- ============================================================================
-- Commands
-- ============================================================================

COMMANDS = {
    -- ------------------------------------------------------------------------
    -- Start a tutorial
    -- ------------------------------------------------------------------------
    start_tutorial = function(args)
        local topic = args and args.topic
        
        if not topic then
            return { 
                success = false, 
                error = "Missing 'topic' argument. Use 'list_tutorials' to see available topics.",
                available_topics = (function()
                    local topics = {}
                    for _, t in ipairs(TUTORIALS) do
                        table.insert(topics, { id = t.id, name = t.name })
                    end
                    return topics
                end)()
            }
        end
        
        local tutorial = get_tutorial(topic)
        if not tutorial then
            return { 
                success = false, 
                error = "Unknown tutorial: " .. topic .. ". Use 'list_tutorials' to see available topics."
            }
        end
        
        -- Check if already in a tutorial
        if active_tutorial then
            SPO.notify_info(string.format("Switching from '%s' to '%s'", 
                active_tutorial.name, tutorial.name))
        end
        
        -- Start the tutorial
        active_tutorial = tutorial
        current_step = 1
        
        -- Log to history
        table.insert(tutorial_history, {
            tutorial_id = tutorial.id,
            started_at = os.time(),
            completed = false
        })
        
        SPO.log_info(string.format("Starting tutorial: %s", tutorial.name))
        SPO.log_info(string.format("Difficulty: %s | Estimated time: %s", 
            tutorial.difficulty, tutorial.estimated_time))
        
        -- Display first step
        local step = tutorial.steps[1]
        send_tutorial_notification(step, 1, #tutorial.steps)
        
        return {
            success = true,
            tutorial = {
                id = tutorial.id,
                name = tutorial.name,
                description = tutorial.description,
                total_steps = #tutorial.steps,
                current_step = 1,
                difficulty = tutorial.difficulty
            },
            current_step = step
        }
    end,
    
    -- ------------------------------------------------------------------------
    -- Get next step in current tutorial
    -- ------------------------------------------------------------------------
    next_step = function(args)
        if not active_tutorial then
            return { 
                success = false, 
                error = "No tutorial in progress. Use 'start_tutorial' to begin.",
                available_commands = {"start_tutorial", "list_tutorials", "get_tip"}
            }
        end
        
        current_step = current_step + 1
        
        if current_step > #active_tutorial.steps then
            -- Tutorial complete
            SPO.notify_success(string.format("Congratulations! You completed '%s'!", active_tutorial.name))
            SPO.log_info(string.format("Tutorial '%s' completed", active_tutorial.name))
            
            -- Mark as completed in history
            if #tutorial_history > 0 then
                tutorial_history[#tutorial_history].completed = true
                tutorial_history[#tutorial_history].completed_at = os.time()
            end
            
            local completed_tutorial = active_tutorial
            active_tutorial = nil
            current_step = 0
            
            return {
                success = true,
                completed = true,
                tutorial_name = completed_tutorial.name,
                message = "Tutorial completed! Try another one with 'list_tutorials'."
            }
        end
        
        -- Show next step
        local step = active_tutorial.steps[current_step]
        send_tutorial_notification(step, current_step, #active_tutorial.steps)
        
        return {
            success = true,
            tutorial = {
                id = active_tutorial.id,
                name = active_tutorial.name,
                current_step = current_step,
                total_steps = #active_tutorial.steps
            },
            step = step,
            progress = string.format("%d/%d", current_step, #active_tutorial.steps)
        }
    end,
    
    -- ------------------------------------------------------------------------
    -- Get previous step in current tutorial
    -- ------------------------------------------------------------------------
    previous_step = function(args)
        if not active_tutorial then
            return { success = false, error = "No tutorial in progress." }
        end
        
        if current_step <= 1 then
            SPO.notify_info("Already at the first step!")
            return {
                success = true,
                at_beginning = true,
                step = active_tutorial.steps[1]
            }
        end
        
        current_step = current_step - 1
        local step = active_tutorial.steps[current_step]
        
        SPO.notify_info("Previous step: " .. step.title)
        
        return {
            success = true,
            tutorial = {
                id = active_tutorial.id,
                name = active_tutorial.name,
                current_step = current_step,
                total_steps = #active_tutorial.steps
            },
            step = step
        }
    end,
    
    -- ------------------------------------------------------------------------
    -- Get current step info
    -- ------------------------------------------------------------------------
    get_current_step = function(args)
        if not active_tutorial then
            return { 
                success = false, 
                in_progress = false,
                error = "No tutorial in progress."
            }
        end
        
        local step = active_tutorial.steps[current_step]
        
        return {
            success = true,
            in_progress = true,
            tutorial = {
                id = active_tutorial.id,
                name = active_tutorial.name,
                current_step = current_step,
                total_steps = #active_tutorial.steps,
                progress_percent = math.floor((current_step / #active_tutorial.steps) * 100)
            },
            step = step
        }
    end,
    
    -- ------------------------------------------------------------------------
    -- Stop current tutorial
    -- ------------------------------------------------------------------------
    stop_tutorial = function(args)
        if not active_tutorial then
            return { success = false, error = "No tutorial in progress." }
        end
        
        local stopped_name = active_tutorial.name
        local stopped_at = current_step
        local total_steps = #active_tutorial.steps
        
        active_tutorial = nil
        current_step = 0
        
        SPO.notify_info(string.format("Tutorial '%s' stopped at step %d/%d", stopped_name, stopped_at, total_steps))
        SPO.log_info(string.format("Tutorial stopped at step %d of %d", stopped_at, total_steps))
        
        return {
            success = true,
            stopped = true,
            tutorial_name = stopped_name,
            stopped_at = stopped_at,
            total_steps = total_steps,
            can_resume = false
        }
    end,
    
    -- ------------------------------------------------------------------------
    -- List available tutorials
    -- ------------------------------------------------------------------------
    list_tutorials = function(args)
        local list = {}
        
        for _, tutorial in ipairs(TUTORIALS) do
            table.insert(list, {
                id = tutorial.id,
                name = tutorial.name,
                description = tutorial.description,
                difficulty = tutorial.difficulty,
                estimated_time = tutorial.estimated_time,
                steps_count = #tutorial.steps
            })
        end
        
        -- Calculate completion stats
        local completed_count = 0
        for _, history in ipairs(tutorial_history) do
            if history.completed then
                completed_count = completed_count + 1
            end
        end
        
        SPO.log_info(string.format("Available tutorials: %d", #list))
        
        return {
            success = true,
            tutorials = list,
            total_available = #list,
            completed_count = completed_count,
            in_progress = active_tutorial and {
                id = active_tutorial.id,
                name = active_tutorial.name,
                current_step = current_step,
                total_steps = #active_tutorial.steps
            } or nil
        }
    end,
    
    -- ------------------------------------------------------------------------
    -- Get contextual tip
    -- ------------------------------------------------------------------------
    get_tip = function(args)
        local context = args and args.context
        
        if not context then
            -- Return a random general tip
            local general_tips = CONTEXTUAL_TIPS.general
            local tip = general_tips[math.random(1, #general_tips)]
            
            SPO.notify_info(tip)
            
            return {
                success = true,
                context = "general",
                tip = tip
            }
        end
        
        local tips = CONTEXTUAL_TIPS[context]
        if not tips then
            -- Unknown context, return general tip
            tips = CONTEXTUAL_TIPS.general
            context = "general"
        end
        
        local tip = tips[math.random(1, #tips)]
        
        SPO.notify_info(tip)
        
        return {
            success = true,
            context = context,
            tip = tip,
            available_tips = #tips
        }
    end,
    
    -- ------------------------------------------------------------------------
    -- Get all tips for a context
    -- ------------------------------------------------------------------------
    get_all_tips = function(args)
        local context = args and args.context
        
        if context then
            local tips = CONTEXTUAL_TIPS[context]
            if tips then
                return {
                    success = true,
                    context = context,
                    tips = tips
                }
            else
                return {
                    success = false,
                    error = "Unknown context: " .. context,
                    available_contexts = (function()
                        local ctx = {}
                        for k, _ in pairs(CONTEXTUAL_TIPS) do
                            table.insert(ctx, k)
                        end
                        return ctx
                    end)()
                }
            end
        else
            -- Return all tips
            return {
                success = true,
                all_tips = CONTEXTUAL_TIPS
            }
        end
    end,
    
    -- ------------------------------------------------------------------------
    -- Get tutorial history
    -- ------------------------------------------------------------------------
    get_history = function(args)
        local include_details = args and args.details
        
        local history_summary = {}
        for _, entry in ipairs(tutorial_history) do
            local summary = {
                tutorial_id = entry.tutorial_id,
                started_at = entry.started_at,
                completed = entry.completed
            }
            
            if include_details then
                summary.started_at_formatted = os.date("%Y-%m-%d %H:%M:%S", entry.started_at)
                if entry.completed_at then
                    summary.completed_at = entry.completed_at
                    summary.completed_at_formatted = os.date("%Y-%m-%d %H:%M:%S", entry.completed_at)
                    summary.duration_seconds = entry.completed_at - entry.started_at
                end
            end
            
            table.insert(history_summary, summary)
        end
        
        return {
            success = true,
            history = history_summary,
            total_started = #tutorial_history,
            total_completed = (function()
                local count = 0
                for _, e in ipairs(tutorial_history) do
                    if e.completed then count = count + 1 end
                end
                return count
            end)()
        }
    end,
    
    -- ------------------------------------------------------------------------
    -- Reset tutorial progress
    -- ------------------------------------------------------------------------
    reset_progress = function(args)
        local confirm = args and args.confirm
        
        if not confirm then
            return {
                success = false,
                error = "Must pass confirm=true to reset progress",
                warning = "This will clear all tutorial history!"
            }
        end
        
        local cleared_count = #tutorial_history
        tutorial_history = {}
        active_tutorial = nil
        current_step = 0
        
        SPO.notify_success("Tutorial progress has been reset")
        SPO.log_info(string.format("Cleared %d tutorial history entries", cleared_count))
        
        return {
            success = true,
            cleared_entries = cleared_count,
            message = "All tutorial progress has been reset"
        }
    end,
    
    -- ------------------------------------------------------------------------
    -- Show help message
    -- ------------------------------------------------------------------------
    help = function(args)
        local help_text = [[
Interactive Tutorial Plugin - Available Commands:

  start_tutorial {topic="first_steps"}  - Start a tutorial
  next_step()                           - Go to next tutorial step
  previous_step()                       - Go to previous tutorial step
  stop_tutorial()                       - Stop current tutorial
  get_current_step()                    - Get current tutorial progress
  list_tutorials()                      - List all available tutorials
  get_tip {context="palette_editor"}    - Get a contextual tip
  get_all_tips {context="ai_editor"}    - Get all tips for a context
  get_history {details=true}            - View your tutorial history
  reset_progress {confirm=true}         - Clear all tutorial progress

Available Tutorials:
  - first_steps    : Beginner - Basic editor usage
  - palette_editing: Beginner - How to edit palettes
  - sprite_import  : Intermediate - How to import sprites
  - ai_behavior    : Advanced - Understanding AI editing
        ]]
        
        SPO.log_info("Interactive Tutorial Help:")
        for line in string.gmatch(help_text, "[^\r\n]+") do
            SPO.log_info(line)
        end
        
        return {
            success = true,
            help_text = help_text
        }
    end
}
