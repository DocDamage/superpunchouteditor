//! Super Punch-Out!! Editor - Tauri Backend
//!
//! Main entry point for the Tauri desktop application.
//! Commands are organized into modules under `commands/`.

// ============================================================================
// Module Declarations
// ============================================================================

mod app_state;
mod commands;
mod types;
mod utils;

// Existing modules - kept in place for now
mod audio_commands;
#[allow(hidden_glob_reexports)]
mod emulator;
mod emulator_embedded;
mod help_system;
mod roster_commands;
mod settings_commands;
mod text_commands;
mod tools_commands;
mod undo;
mod update_commands;

// ============================================================================
// Public Re-exports
// ============================================================================

pub use app_state::AppState;
pub use commands::*;
pub use types::*;
pub use utils::*;

// Re-export from existing modules
pub use audio_commands::AudioState;
pub use emulator::{EmulatorLauncher, EmulatorSettings, EmulatorType};
pub use emulator_embedded::{
    ControllerInput, CreatorRuntimeState, CreatorSessionState, EmbeddedEmulatorState, EmulatorFrameData, EmulatorStatus,
};
pub use help_system::{HelpArticle, HelpArticleSummary, HelpCategory, HelpSystem, SearchResult};
pub use settings_commands::{
    export_settings, get_app_settings, import_settings, load_theme_settings,
    preview_settings_import, reset_settings_to_defaults, save_settings, save_theme_settings,
    update_settings, validate_settings_file, AppSettings, ImportReport, SettingsChangePreview,
};
pub use tools_commands::{
    add_external_tool, get_compatible_tools, get_default_tool, get_external_tools,
    get_preset_tools, get_tool_categories, launch_with_tool, load_external_tools_config,
    remove_external_tool, set_default_tool, update_external_tool, verify_tool,
};
pub use undo::{EditHistory, EditSummary};
pub use update_commands::{
    check_for_updates, clear_skipped_versions, download_and_install_update, get_current_version,
    get_download_progress, get_manual_download_url, get_update_settings, set_update_settings,
    should_auto_check, skip_version, UpdateState,
};

// ============================================================================
// Imports
// ============================================================================

// ============================================================================
// Main Entry Point
// ============================================================================

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // The manifest is loaded per-region when a ROM is opened (see open_rom command).
    // Start with an empty default so startup never fails due to missing manifest files.
    let manifest = manifest_core::Manifest::empty();

    // Load saved settings
    let _emulator_settings = commands::settings::load_emulator_settings().unwrap_or_default();
    let _external_tools = load_external_tools_config().unwrap_or_default();

    tauri::Builder::default()
        .manage(AppState::new(manifest))
        .manage(UpdateState::default())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            // ROM Commands
            commands::rom::open_rom,
            commands::rom::get_rom_sha1,
            commands::rom::get_rom_path,
            commands::rom::save_rom_as,
            commands::rom::get_pending_writes,
            commands::rom::get_pending_bytes,
            commands::rom::get_rom_bytes,
            commands::rom::get_loaded_rom_image,
            commands::rom::discard_bin_edit,
            commands::rom::is_rom_loaded,
            // Expansion Commands
            commands::expansion::apply_in_game_expansion,
            commands::expansion::analyze_in_game_hook_sites,
            commands::expansion::verify_in_game_hook_site,
            commands::expansion::get_in_game_hook_presets,
            // Boxer Commands
            commands::boxer::get_boxers,
            commands::boxer::get_boxer,
            commands::boxer::create_boxer_asset_owner,
            commands::boxer::get_fighter_list,
            commands::boxer::get_fighter_poses,
            commands::boxer::render_fighter_pose,
            // Asset Commands
            commands::assets::get_palette,
            commands::assets::get_runtime_theme_assets,
            commands::assets::export_asset_to_png,
            commands::assets::import_asset_from_png,
            commands::assets::import_graphic_asset_from_png,
            commands::assets::export_sprite_bin_to_png,
            commands::assets::import_sprite_bin_from_png,
            commands::assets::get_bin_original_bytes,
            commands::assets::get_sprite_bin_diff,
            commands::assets::get_fighter_tiles,
            commands::assets::render_sprite_sheet,
            commands::boxer::get_boxer_layout,
            commands::boxer::get_all_layouts,
            commands::boxer::compare_boxers,
            commands::boxer::get_similar_boxers,
            commands::boxer::copy_boxer_stat,
            commands::boxer::copy_all_boxer_stats,
            // Project Commands
            commands::project::create_project,
            commands::project::save_project,
            commands::project::load_project,
            commands::project::validate_project,
            commands::project::get_current_project,
            commands::project::get_current_project_path,
            commands::project::close_project,
            commands::project::generate_patch_notes,
            commands::project::get_change_summary,
            commands::project::save_patch_notes,
            // Patch Commands
            commands::patches::export_ips_patch,
            commands::patches::export_bps_patch,
            commands::patches::export_patch_notes_with_patch,
            // Settings Commands
            commands::settings::get_emulator_settings,
            commands::settings::set_emulator_settings,
            commands::settings::verify_emulator,
            commands::settings::get_save_state_slots,
            // External Emulator Commands
            commands::emulator::test_in_emulator,
            commands::emulator::get_emulator_presets,
            // Comparison Commands
            commands::comparison::generate_comparison,
            commands::comparison::get_palette_diff,
            commands::comparison::get_sprite_bin_diff_comparison,
            commands::comparison::get_binary_diff,
            commands::comparison::export_comparison_report,
            // Region Commands
            commands::region::get_supported_regions,
            commands::region::detect_rom_region,
            commands::region::validate_region_manifest,
            // Settings Import/Export
            export_settings,
            import_settings,
            preview_settings_import,
            reset_settings_to_defaults,
            validate_settings_file,
            get_app_settings,
            save_settings,
            update_settings,
            load_theme_settings,
            save_theme_settings,
            // Auto-updater
            get_current_version,
            get_update_settings,
            set_update_settings,
            check_for_updates,
            skip_version,
            download_and_install_update,
            get_download_progress,
            get_manual_download_url,
            clear_skipped_versions,
            should_auto_check,
            // External Tools
            get_external_tools,
            add_external_tool,
            remove_external_tool,
            update_external_tool,
            launch_with_tool,
            get_compatible_tools,
            get_tool_categories,
            get_preset_tools,
            set_default_tool,
            get_default_tool,
            verify_tool,
            // Help System (currently disabled - TODO: implement in help_system.rs)
            // get_help_articles,
            // get_help_article,
            // search_help,
            // get_context_help,
            // get_help_categories,
            // submit_help_feedback,
            // Roster Commands
            roster_commands::get_roster_data,
            roster_commands::get_boxer_roster_entry,
            roster_commands::get_boxers_by_circuit,
            roster_commands::get_boxers_by_unlock_order,
            roster_commands::update_boxer_name,
            roster_commands::commit_creator_session,
            roster_commands::validate_creator_session,
            roster_commands::validate_boxer_name,
            roster_commands::preview_name_encoding,
            roster_commands::get_text_encoding_info,
            roster_commands::update_boxer_circuit,
            roster_commands::get_circuits,
            roster_commands::get_circuit_types,
            roster_commands::update_unlock_order,
            roster_commands::set_champion_status,
            roster_commands::get_intro_text,
            roster_commands::update_intro_text,
            roster_commands::validate_intro_text,
            roster_commands::validate_roster_changes,
            roster_commands::reset_roster_to_defaults,
            roster_commands::get_roster_offsets,
            roster_commands::scan_for_text_tables,
            // Text Commands
            roster_commands::get_cornerman_texts,  // From roster_commands
            text_commands::get_cornerman_text,
            text_commands::update_cornerman_text,
            text_commands::add_cornerman_text,
            text_commands::delete_cornerman_text,
            text_commands::get_text_conditions,
            roster_commands::get_boxer_intro,  // From roster_commands
            text_commands::update_boxer_intro,
            roster_commands::get_victory_quotes,  // From roster_commands
            text_commands::update_victory_quote,
            text_commands::get_victory_conditions,
            text_commands::get_menu_texts,
            text_commands::update_menu_text,
            text_commands::get_menu_categories,
            text_commands::preview_text_render,
            text_commands::validate_text,
            text_commands::get_text_editor_encoding_info,
            text_commands::encode_text,
            text_commands::decode_text,
            text_commands::validate_all_texts,
            text_commands::search_texts,
            text_commands::reset_text_to_defaults,
            text_commands::get_text_statistics,
            // Audio Commands
            audio_commands::get_sound_list,
            audio_commands::get_sound,
            audio_commands::preview_sound,
            audio_commands::stop_preview,
            audio_commands::get_playback_state,
            audio_commands::export_sound_as_wav,
            audio_commands::export_sound_as_brr,
            audio_commands::get_music_list,
            audio_commands::get_music,
            audio_commands::get_music_sequence,
            audio_commands::preview_music,
            audio_commands::update_music_sequence,
            audio_commands::export_music_as_wav,
            audio_commands::export_music_as_spc,
            audio_commands::get_sample,
            audio_commands::import_sound_from_wav,
            audio_commands::decode_brr_to_pcm,
            audio_commands::encode_pcm_to_brr,
            audio_commands::load_spc,
            audio_commands::save_spc,
            audio_commands::create_new_spc,
            audio_commands::get_spc_info,
            audio_commands::get_preview_config,
            audio_commands::set_preview_config,
            audio_commands::get_audio_settings,
            audio_commands::scan_rom_for_audio,
            audio_commands::extract_all_rom_audio,
            audio_commands::get_audio_engine_info,
            // Embedded Emulator
            emulator_embedded::init_emulator,
            emulator_embedded::emulator_load_rom,
            emulator_embedded::emulator_load_rom_from_memory,
            emulator_embedded::emulator_load_rom_with_edits,
            emulator_embedded::emulator_start,
            emulator_embedded::emulator_stop,
            emulator_embedded::emulator_set_paused,
            emulator_embedded::emulator_toggle_pause,
            emulator_embedded::emulator_reset,
            emulator_embedded::emulator_get_frame,
            emulator_embedded::emulator_get_creator_runtime_state,
            emulator_embedded::emulator_set_creator_session_state,
            emulator_embedded::emulator_resolve_creator_runtime_action,
            emulator_embedded::emulator_set_input,
            emulator_embedded::emulator_set_controller_input,
            emulator_embedded::emulator_save_state,
            emulator_embedded::emulator_load_state,
            emulator_embedded::emulator_set_speed,
            emulator_embedded::emulator_advance_frame,
            emulator_embedded::emulator_get_status,
            emulator_embedded::emulator_shutdown,
            emulator_embedded::emulator_get_save_states,
            emulator_embedded::emulator_delete_save_state,
            // AI Behavior Commands
            commands::ai_behavior::get_ai_behavior,
            commands::ai_behavior::get_all_ai_behaviors,
            commands::ai_behavior::update_attack_pattern,
            commands::ai_behavior::update_defense_behavior,
            commands::ai_behavior::update_difficulty_curve,
            commands::ai_behavior::update_triggers,
            commands::ai_behavior::test_ai_behavior,
            commands::ai_behavior::validate_ai_behavior,
            commands::ai_behavior::compare_ai_behavior,
            commands::ai_behavior::get_ai_presets,
            commands::ai_behavior::apply_ai_preset,
            commands::ai_behavior::get_move_types,
            commands::ai_behavior::get_defense_types,
            commands::ai_behavior::get_condition_types,
            commands::ai_behavior::reset_ai_to_defaults,
            commands::ai_behavior::get_ai_table_addresses,
            // Plugin Commands
            commands::plugins::list_plugins,
            commands::plugins::load_plugin,
            commands::plugins::unload_plugin,
            commands::plugins::enable_plugin,
            commands::plugins::disable_plugin,
            commands::plugins::execute_plugin_command,
            commands::plugins::run_script_file,
            commands::plugins::run_script,
            commands::plugins::list_batch_jobs,
            commands::plugins::create_batch_job,
            commands::plugins::cancel_batch_job,
            commands::plugins::get_plugins_directory,
            commands::plugins::open_plugins_directory,
            commands::plugins::reload_all_plugins,
            // Edit History Commands
            commands::history::undo,
            commands::history::redo,
            commands::history::clear_history,
            // Bank Management Commands
            commands::bank_management::get_bank_visualization,
            commands::bank_management::find_free_regions,
            commands::bank_management::analyze_fragmentation,
            commands::bank_management::generate_defrag_plan,
            commands::bank_management::execute_defrag_plan,
            commands::bank_management::mark_bank_region,
            commands::bank_management::get_rom_statistics,
            // Frame Reconstructor Commands
            commands::frame_reconstructor::get_fighter_frames,
            commands::frame_reconstructor::get_frame_detail,
            commands::frame_reconstructor::render_frame_preview,
            commands::frame_reconstructor::get_fighter_annotations,
            // Frame Tag Commands
            commands::frame_tags::get_frame_tags,
            commands::frame_tags::add_frame_tag,
            commands::frame_tags::remove_frame_tag,
            commands::frame_tags::get_frame_annotation,
            commands::frame_tags::update_frame_annotation,
            commands::frame_tags::add_tag_to_frame,
            commands::frame_tags::remove_tag_from_frame,
            commands::frame_tags::get_boxer_annotations,
            // Animation Commands
            commands::animation::get_boxer_animation,
            commands::animation::play_animation,
            commands::animation::pause_animation,
            commands::animation::stop_animation,
            commands::animation::seek_animation_frame,
            commands::animation::update_animation,
            commands::animation::get_interpolated_frame,
            commands::animation::get_hitbox_editor_state,
            commands::animation::create_hitbox,
            commands::animation::create_hurtbox,
            commands::animation::update_hitbox,
            commands::animation::delete_hitbox,
            commands::animation::set_hitbox_editor_option,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// ============================================================================
// Legacy Command Placeholders (to be moved to modules)
// ============================================================================

// These commands need to be extracted from the original lib.rs:
// - get_palette
// - export_asset_to_png
// - import_asset_from_png
// - export_sprite_bin_to_png
// - import_sprite_bin_from_png
// - get_sprite_bin_diff (sprite bin version)
// - get_fighter_list
// - get_fighter_poses
// - render_fighter_pose
// - render_sprite_sheet
// - get_all_scripts
// - get_scripts_for_fighter
// - get_fighter_header
// - get_editable_fighter_params
// - validate_fighter_params
// - update_fighter_params
// - Project thumbnail commands
// - Relocation commands
// - Animation commands
// - Frame reconstructor commands
// - Patch notes generator
// - Detailed asset report
// - Layout pack commands
// - Frame tagging commands
// - ROM Region commands
// - AI Behavior commands
