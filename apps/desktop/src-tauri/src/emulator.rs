//! Emulator Integration Module
//!
//! Provides functionality to launch ROMs in various SNES emulators
//! for quick testing of modifications.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::{Child, Command};

/// Supported emulator types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum EmulatorType {
    /// Snes9x emulator (most popular, cross-platform)
    Snes9x,
    /// bsnes/higan emulator (accuracy-focused)
    Bsnes,
    /// Mesen-S emulator (modern, accurate)
    MesenS,
    /// Other/Generic emulator
    Other,
}

impl EmulatorType {
    /// Get the display name for this emulator type
    #[allow(dead_code)]
    pub fn display_name(&self) -> &'static str {
        match self {
            EmulatorType::Snes9x => "Snes9x",
            EmulatorType::Bsnes => "bsnes/higan",
            EmulatorType::MesenS => "Mesen-S",
            EmulatorType::Other => "Other",
        }
    }

    /// Get the default executable name for each platform
    #[allow(dead_code)]
    pub fn default_executable(&self) -> &'static str {
        match self {
            EmulatorType::Snes9x => {
                #[cfg(target_os = "windows")]
                return "snes9x-x64.exe";
                #[cfg(target_os = "macos")]
                return "Snes9x.app";
                #[cfg(target_os = "linux")]
                return "snes9x";
                #[cfg(not(any(
                    target_os = "windows",
                    target_os = "macos",
                    target_os = "linux"
                )))]
                return "snes9x";
            }
            EmulatorType::Bsnes => {
                #[cfg(target_os = "windows")]
                return "bsnes.exe";
                #[cfg(target_os = "macos")]
                return "bsnes.app";
                #[cfg(target_os = "linux")]
                return "bsnes";
                #[cfg(not(any(
                    target_os = "windows",
                    target_os = "macos",
                    target_os = "linux"
                )))]
                return "bsnes";
            }
            EmulatorType::MesenS => {
                #[cfg(target_os = "windows")]
                return "Mesen-S.exe";
                #[cfg(target_os = "macos")]
                return "Mesen-S.app";
                #[cfg(target_os = "linux")]
                return "Mesen-S";
                #[cfg(not(any(
                    target_os = "windows",
                    target_os = "macos",
                    target_os = "linux"
                )))]
                return "Mesen-S";
            }
            EmulatorType::Other => "emulator",
        }
    }

    /// Get the save state file extension for this emulator
    pub fn save_state_extension(&self) -> &'static str {
        match self {
            EmulatorType::Snes9x => ".000",
            EmulatorType::Bsnes => ".bsv",
            EmulatorType::MesenS => ".mss",
            EmulatorType::Other => ".sav",
        }
    }

    /// Get the save state directory name for this emulator
    pub fn save_state_dir(&self) -> &'static str {
        match self {
            EmulatorType::Snes9x => "Saves",
            EmulatorType::Bsnes => "States",
            EmulatorType::MesenS => "SaveStates",
            EmulatorType::Other => "",
        }
    }
}

impl std::str::FromStr for EmulatorType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "snes9x" | "snes9-x" => Ok(EmulatorType::Snes9x),
            "bsnes" | "higan" => Ok(EmulatorType::Bsnes),
            "mesen-s" | "mesens" => Ok(EmulatorType::MesenS),
            "other" => Ok(EmulatorType::Other),
            _ => Err(format!("Unknown emulator type: {}", s)),
        }
    }
}

/// Configuration for launching the emulator at a specific game state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameStateConfig {
    /// Which boxer to fight (0-15, where 0 = Gabby Jay)
    pub boxer_index: Option<u8>,
    /// Which round (1-3)
    pub round: Option<u8>,
    /// Player HP (0-128)
    pub player_health: Option<u8>,
    /// Opponent HP (0-128)
    pub opponent_health: Option<u8>,
    /// Starting time in seconds (usually 180 for 3 minutes)
    pub time_seconds: Option<u16>,
}

impl Default for GameStateConfig {
    fn default() -> Self {
        Self {
            boxer_index: None,
            round: Some(1),
            player_health: Some(128),
            opponent_health: Some(128),
            time_seconds: Some(180),
        }
    }
}

/// Information about an emulator installation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmulatorInfo {
    pub emulator_type: EmulatorType,
    pub path: String,
    pub version: Option<String>,
    pub is_valid: bool,
    pub error_message: Option<String>,
}

/// User settings for emulator integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmulatorSettings {
    /// Path to the emulator executable
    pub emulator_path: String,
    /// Type of emulator
    pub emulator_type: EmulatorType,
    /// Whether to auto-save ROM before launching
    pub auto_save_before_launch: bool,
    /// Additional command line arguments
    pub command_line_args: String,
    /// Whether to jump to selected boxer on launch
    pub jump_to_selected_boxer: bool,
    /// Default round to start in
    pub default_round: u8,
    /// Directory for save states
    pub save_state_dir: Option<String>,
}

impl Default for EmulatorSettings {
    fn default() -> Self {
        Self {
            emulator_path: String::new(),
            emulator_type: EmulatorType::Snes9x,
            auto_save_before_launch: true,
            command_line_args: String::new(),
            jump_to_selected_boxer: true,
            default_round: 1,
            save_state_dir: None,
        }
    }
}

/// Launcher for managing emulator execution
pub struct EmulatorLauncher;

impl EmulatorLauncher {
    /// Launch a ROM in the configured emulator
    ///
    /// # Arguments
    /// * `rom_path` - Path to the ROM file
    /// * `emulator_path` - Path to the emulator executable
    /// * `emulator_type` - Type of emulator being used
    /// * `args` - Additional command line arguments
    ///
    /// # Returns
    /// The spawned child process handle
    pub fn launch(
        rom_path: &Path,
        emulator_path: &Path,
        emulator_type: EmulatorType,
        args: &[String],
    ) -> Result<Child, String> {
        if !emulator_path.exists() {
            return Err(format!(
                "Emulator not found at: {}",
                emulator_path.display()
            ));
        }

        if !rom_path.exists() {
            return Err(format!("ROM not found at: {}", rom_path.display()));
        }

        let launch_args = Self::get_launch_command(emulator_type, rom_path, args);

        let child = Command::new(emulator_path)
            .args(&launch_args)
            .spawn()
            .map_err(|e| format!("Failed to launch emulator: {}", e))?;

        Ok(child)
    }

    /// Launch a ROM with a specific save state
    ///
    /// # Arguments
    /// * `rom_path` - Path to the ROM file
    /// * `emulator_path` - Path to the emulator executable
    /// * `emulator_type` - Type of emulator being used
    /// * `state_path` - Path to the save state file
    /// * `args` - Additional command line arguments
    pub fn launch_with_state(
        rom_path: &Path,
        emulator_path: &Path,
        emulator_type: EmulatorType,
        state_path: &Path,
        args: &[String],
    ) -> Result<Child, String> {
        if !state_path.exists() {
            return Err(format!("Save state not found at: {}", state_path.display()));
        }

        // Different emulators have different ways to load states
        let mut launch_args = match emulator_type {
            EmulatorType::Snes9x => {
                let mut args = vec![rom_path.to_string_lossy().to_string()];
                args.push("-state".to_string());
                args.push(state_path.to_string_lossy().to_string());
                args
            }
            EmulatorType::Bsnes => {
                // bsnes loads state from same directory as ROM with specific naming
                vec![rom_path.to_string_lossy().to_string()]
            }
            EmulatorType::MesenS => {
                let mut args = vec![rom_path.to_string_lossy().to_string()];
                args.push("--load-state".to_string());
                args.push(state_path.to_string_lossy().to_string());
                args
            }
            EmulatorType::Other => {
                vec![rom_path.to_string_lossy().to_string()]
            }
        };

        // Add any additional args
        launch_args.extend_from_slice(args);

        let child = Command::new(emulator_path)
            .args(&launch_args)
            .spawn()
            .map_err(|e| format!("Failed to launch emulator with state: {}", e))?;

        Ok(child)
    }

    /// Verify that an emulator is installed and accessible
    ///
    /// # Arguments
    /// * `emulator_path` - Path to the emulator executable
    ///
    /// # Returns
    /// Information about the emulator including validity
    pub fn verify_emulator(emulator_path: &Path) -> Result<EmulatorInfo, String> {
        if !emulator_path.exists() {
            return Err(format!(
                "Emulator not found at: {}",
                emulator_path.display()
            ));
        }

        // Try to determine emulator type from path
        let path_str = emulator_path.to_string_lossy().to_lowercase();
        let emulator_type = if path_str.contains("snes9x") {
            EmulatorType::Snes9x
        } else if path_str.contains("bsnes") || path_str.contains("higan") {
            EmulatorType::Bsnes
        } else if path_str.contains("mesen-s") || path_str.contains("mesens") {
            EmulatorType::MesenS
        } else {
            EmulatorType::Other
        };

        // Try to get version (platform-specific)
        let version = Self::try_get_version(emulator_path, emulator_type);

        Ok(EmulatorInfo {
            emulator_type,
            path: emulator_path.to_string_lossy().to_string(),
            version,
            is_valid: true,
            error_message: None,
        })
    }

    /// Get the default save state path for the given emulator
    ///
    /// # Arguments
    /// * `emulator_type` - Type of emulator
    /// * `rom_path` - Path to the ROM (for naming the state file)
    ///
    /// # Returns
    /// Path to the default save state location
    pub fn get_save_state_path(
        emulator_type: EmulatorType,
        rom_path: &Path,
        slot: Option<u8>,
    ) -> PathBuf {
        let slot_num = slot.unwrap_or(0);
        let rom_name = rom_path
            .file_stem()
            .map(|s| s.to_string_lossy())
            .unwrap_or_else(|| "rom".into());

        let ext = emulator_type.save_state_extension();
        let slot_ext = if slot_num > 0 {
            format!(".{:03}", slot_num)
        } else {
            ext.to_string()
        };

        // Platform-specific save state directories
        #[cfg(target_os = "windows")]
        let base_dir = dirs::data_local_dir()
            .map(|d| d.join(emulator_type.save_state_dir()))
            .unwrap_or_else(|| rom_path.parent().unwrap_or(Path::new(".")).to_path_buf());

        #[cfg(target_os = "macos")]
        let base_dir = dirs::home_dir()
            .map(|d| {
                d.join("Library/Application Support")
                    .join(emulator_type.save_state_dir())
            })
            .unwrap_or_else(|| rom_path.parent().unwrap_or(Path::new(".")).to_path_buf());

        #[cfg(target_os = "linux")]
        let base_dir = dirs::config_dir()
            .map(|d| d.join(emulator_type.save_state_dir()))
            .unwrap_or_else(|| rom_path.parent().unwrap_or(Path::new(".")).to_path_buf());

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        let base_dir = rom_path.parent().unwrap_or(Path::new(".")).to_path_buf();

        base_dir.join(format!("{}{}", rom_name, slot_ext))
    }

    /// Create a quick save state at a specific game state
    ///
    /// Note: This is a placeholder implementation. Creating actual save states
    /// requires emulator-specific memory manipulation which is complex.
    /// In practice, users would create save states manually in the emulator.
    ///
    /// # Arguments
    /// * `rom_path` - Path to the ROM file
    /// * `emulator_type` - Type of emulator
    /// * `state_config` - Configuration for the game state
    ///
    /// # Returns
    /// Path to the created state file (or error)
    #[allow(dead_code)]
    pub fn create_quick_save(
        _rom_path: &Path,
        _emulator_type: EmulatorType,
        _state_config: &GameStateConfig,
    ) -> Result<PathBuf, String> {
        // Note: Creating save states programmatically requires:
        // 1. Running the emulator in a controlled way
        // 2. Manipulating emulator memory to set game state
        // 3. Triggering the save state functionality
        //
        // This is highly emulator-specific and complex.
        // For now, we return an error indicating manual creation is needed.
        Err("Automatic save state creation is not implemented. \
             Please create save states manually in the emulator. \
             Use slot 0 for quick test launches."
            .to_string())
    }

    /// Get the platform-specific launch command for an emulator
    fn get_launch_command(
        emulator_type: EmulatorType,
        rom_path: &Path,
        extra_args: &[String],
    ) -> Vec<String> {
        let mut args = match emulator_type {
            EmulatorType::Snes9x => {
                vec![
                    rom_path.to_string_lossy().to_string(),
                    "-fullscreen".to_string(),
                ]
            }
            EmulatorType::Bsnes => {
                vec![rom_path.to_string_lossy().to_string()]
            }
            EmulatorType::MesenS => {
                vec![
                    rom_path.to_string_lossy().to_string(),
                    "--fullscreen".to_string(),
                ]
            }
            EmulatorType::Other => {
                vec![rom_path.to_string_lossy().to_string()]
            }
        };

        // Add any extra arguments
        args.extend_from_slice(extra_args);

        args
    }

    /// Try to get the emulator version by running it with a version flag
    fn try_get_version(emulator_path: &Path, emulator_type: EmulatorType) -> Option<String> {
        let version_flag = match emulator_type {
            EmulatorType::Snes9x => "-h",
            EmulatorType::Bsnes => "--version",
            EmulatorType::MesenS => "--version",
            EmulatorType::Other => "--version",
        };

        // Run the emulator briefly to get version info
        // This is best-effort and may not work for all emulators
        let output = Command::new(emulator_path)
            .arg(version_flag)
            .output()
            .ok()?;

        let output_str = String::from_utf8_lossy(&output.stdout);

        // Try to extract version number
        output_str
            .lines()
            .next()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    }

    /// Get a list of common emulator installation paths for the current platform
    #[allow(dead_code)]
    pub fn get_common_install_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();

        #[cfg(target_os = "windows")]
        {
            // Common Windows installation directories
            if let Some(program_files) = std::env::var("ProgramFiles").ok() {
                paths.push(PathBuf::from(&program_files).join("Snes9x"));
                paths.push(PathBuf::from(&program_files).join("bsnes"));
                paths.push(PathBuf::from(&program_files).join("Mesen-S"));
            }
            if let Some(program_files_x86) = std::env::var("ProgramFiles(x86)").ok() {
                paths.push(PathBuf::from(&program_files_x86).join("Snes9x"));
                paths.push(PathBuf::from(&program_files_x86).join("bsnes"));
                paths.push(PathBuf::from(&program_files_x86).join("Mesen-S"));
            }
            if let Some(local_app_data) = dirs::data_local_dir() {
                paths.push(local_app_data.join("Snes9x"));
                paths.push(local_app_data.join("Mesen-S"));
            }
        }

        #[cfg(target_os = "macos")]
        {
            // macOS application directories
            paths.push(PathBuf::from("/Applications"));
            if let Some(home) = dirs::home_dir() {
                paths.push(home.join("Applications"));
            }
        }

        #[cfg(target_os = "linux")]
        {
            // Linux executable paths
            paths.push(PathBuf::from("/usr/bin"));
            paths.push(PathBuf::from("/usr/local/bin"));
            if let Some(home) = dirs::home_dir() {
                paths.push(home.join(".local/bin"));
                paths.push(home.join("Applications"));
            }
        }

        paths
    }

    /// Auto-detect an emulator installation
    #[allow(dead_code)]
    pub fn auto_detect_emulator(emulator_type: EmulatorType) -> Option<PathBuf> {
        let common_paths = Self::get_common_install_paths();
        let exe_name = emulator_type.default_executable();

        for base_path in common_paths {
            let exe_path = base_path.join(exe_name);
            if exe_path.exists() {
                return Some(exe_path);
            }

            // On macOS, check inside .app bundles
            #[cfg(target_os = "macos")]
            {
                let app_path = base_path.join(format!("{}.app", exe_name.trim_end_matches(".app")));
                let macos_exe = app_path
                    .join("Contents/MacOS")
                    .join(exe_name.trim_end_matches(".app"));
                if macos_exe.exists() {
                    return Some(macos_exe);
                }
            }
        }

        None
    }
}

/// Quick save state presets for testing
pub mod presets {
    use super::GameStateConfig;

    /// Get preset for testing a specific boxer
    #[allow(dead_code)]
    pub fn boxer_preset(boxer_index: u8, round: u8) -> GameStateConfig {
        GameStateConfig {
            boxer_index: Some(boxer_index),
            round: Some(round),
            player_health: Some(128),
            opponent_health: Some(128),
            time_seconds: Some(180),
        }
    }

    /// Get preset for testing knockdown scenarios
    #[allow(dead_code)]
    pub fn knockdown_preset(boxer_index: u8) -> GameStateConfig {
        GameStateConfig {
            boxer_index: Some(boxer_index),
            round: Some(1),
            player_health: Some(128),
            opponent_health: Some(10), // Low health for easy KO
            time_seconds: Some(180),
        }
    }

    /// Get preset for testing low health scenarios
    #[allow(dead_code)]
    pub fn low_health_preset(boxer_index: u8) -> GameStateConfig {
        GameStateConfig {
            boxer_index: Some(boxer_index),
            round: Some(2),
            player_health: Some(20), // Low player health
            opponent_health: Some(128),
            time_seconds: Some(60),
        }
    }

    /// Get preset for testing cornerman dialog
    #[allow(dead_code)]
    pub fn cornerman_preset(boxer_index: u8) -> GameStateConfig {
        GameStateConfig {
            boxer_index: Some(boxer_index),
            round: Some(2),
            player_health: Some(30),
            opponent_health: Some(80),
            time_seconds: Some(10), // Low time to trigger cornerman
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emulator_type_from_str() {
        assert!(matches!(
            "snes9x".parse::<EmulatorType>(),
            Ok(EmulatorType::Snes9x)
        ));
        assert!(matches!(
            "bsnes".parse::<EmulatorType>(),
            Ok(EmulatorType::Bsnes)
        ));
        assert!(matches!(
            "Mesen-S".parse::<EmulatorType>(),
            Ok(EmulatorType::MesenS)
        ));
    }

    #[test]
    fn test_save_state_extension() {
        assert_eq!(EmulatorType::Snes9x.save_state_extension(), ".000");
        assert_eq!(EmulatorType::Bsnes.save_state_extension(), ".bsv");
        assert_eq!(EmulatorType::MesenS.save_state_extension(), ".mss");
    }

    #[test]
    fn test_game_state_config_default() {
        let config = GameStateConfig::default();
        assert_eq!(config.round, Some(1));
        assert_eq!(config.player_health, Some(128));
        assert_eq!(config.opponent_health, Some(128));
    }

    #[test]
    fn test_presets() {
        let boxer = presets::boxer_preset(5, 2);
        assert_eq!(boxer.boxer_index, Some(5));
        assert_eq!(boxer.round, Some(2));

        let knockdown = presets::knockdown_preset(3);
        assert_eq!(knockdown.opponent_health, Some(10));

        let low_health = presets::low_health_preset(7);
        assert_eq!(low_health.player_health, Some(20));
    }
}
