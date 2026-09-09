//! External Tool Hooks Module
//!
//! Provides functionality to configure and launch external tools
//! for editing ROM assets (tile editors, hex editors, graphics editors, etc.)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

/// Category of external tool
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ToolCategory {
    /// Graphics editors like Photoshop, Aseprite, GIMP
    GraphicsEditor,
    /// Hex editors like HxD, 010 Editor, Hex Fiend
    HexEditor,
    /// Tile editors like Tile Layer Pro, YY-CHR, Tile Molester
    TileEditor,
    /// Emulators (already have separate support)
    Emulator,
    /// Other tools
    Other,
}

impl ToolCategory {
    /// Get the display name for this category
    pub fn display_name(&self) -> &'static str {
        match self {
            ToolCategory::GraphicsEditor => "Graphics Editor",
            ToolCategory::HexEditor => "Hex Editor",
            ToolCategory::TileEditor => "Tile Editor",
            ToolCategory::Emulator => "Emulator",
            ToolCategory::Other => "Other",
        }
    }

    /// Get an icon representation for this category
    pub fn icon(&self) -> &'static str {
        match self {
            ToolCategory::GraphicsEditor => "🎨",
            ToolCategory::HexEditor => "🔢",
            ToolCategory::TileEditor => "🧩",
            ToolCategory::Emulator => "🎮",
            ToolCategory::Other => "🔧",
        }
    }

    /// Get all supported file extensions for this category
    pub fn supported_extensions(&self) -> Vec<&'static str> {
        match self {
            ToolCategory::GraphicsEditor => {
                vec!["png", "bmp", "gif", "jpg", "jpeg", "tga", "psd"]
            }
            ToolCategory::HexEditor => {
                vec!["bin", "sfc", "smc", "ips", "bps", "dat"]
            }
            ToolCategory::TileEditor => {
                vec!["bin", "chr", "gb", "nes", "sfc", "smc"]
            }
            ToolCategory::Emulator => {
                vec!["sfc", "smc", "fig", "swc"]
            }
            ToolCategory::Other => {
                vec!["*"]
            }
        }
    }
}

impl std::str::FromStr for ToolCategory {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "graphics_editor" | "graphicseditor" | "graphics" | "image" => {
                Ok(ToolCategory::GraphicsEditor)
            }
            "hex_editor" | "hexeditor" | "hex" => Ok(ToolCategory::HexEditor),
            "tile_editor" | "tileeditor" | "tile" => Ok(ToolCategory::TileEditor),
            "emulator" | "emu" => Ok(ToolCategory::Emulator),
            "other" => Ok(ToolCategory::Other),
            _ => Err(format!("Unknown tool category: {}", s)),
        }
    }
}

/// Context information passed when launching a tool
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ToolContext {
    /// PC offset in ROM (if applicable)
    pub offset: Option<String>,
    /// Size of the data
    pub size: Option<usize>,
    /// SNES address (if applicable)
    pub snes_address: Option<String>,
    /// Asset category (e.g., "Compressed Sprite Bin")
    pub category: Option<String>,
    /// Boxer/fighter name (if applicable)
    pub boxer: Option<String>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl ToolContext {
    /// Create a new tool context with just an offset
    pub fn with_offset(offset: &str) -> Self {
        Self {
            offset: Some(offset.to_string()),
            ..Default::default()
        }
    }

    /// Create a new tool context for a specific asset
    pub fn for_asset(offset: &str, size: usize, category: &str, boxer: &str) -> Self {
        Self {
            offset: Some(offset.to_string()),
            size: Some(size),
            snes_address: None,
            category: Some(category.to_string()),
            boxer: Some(boxer.to_string()),
            metadata: HashMap::new(),
        }
    }
}

/// External tool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalTool {
    /// Unique identifier for the tool
    pub id: String,
    /// Display name
    pub name: String,
    /// Path to the executable
    pub executable_path: String,
    /// Arguments template with placeholders like {file}, {offset}, {size}
    /// Example: "{file} --offset {offset}" or "{file} -snes {snes_address}"
    pub arguments_template: String,
    /// File extensions this tool can handle (e.g., ["png", "bin", "sfc"])
    pub supported_file_types: Vec<String>,
    /// Category of tool
    pub category: ToolCategory,
    /// Whether this tool is enabled
    pub enabled: bool,
    /// Optional working directory
    pub working_directory: Option<String>,
    /// Environment variables to set
    pub env_vars: HashMap<String, String>,
}

impl ExternalTool {
    /// Create a new external tool
    pub fn new(id: &str, name: &str, executable_path: &str, category: ToolCategory) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            executable_path: executable_path.to_string(),
            arguments_template: "{file}".to_string(),
            supported_file_types: category
                .supported_extensions()
                .into_iter()
                .map(|s| s.to_string())
                .collect(),
            category,
            enabled: true,
            working_directory: None,
            env_vars: HashMap::new(),
        }
    }

    /// Set a custom arguments template
    pub fn with_args_template(mut self, template: &str) -> Self {
        self.arguments_template = template.to_string();
        self
    }

    /// Set supported file types
    pub fn with_file_types(mut self, types: &[&str]) -> Self {
        self.supported_file_types = types.iter().map(|&s| s.to_string()).collect();
        self
    }

    /// Check if this tool supports a given file extension
    pub fn supports_file_type(&self, extension: &str) -> bool {
        let ext_lower = extension.to_lowercase();
        self.supported_file_types
            .iter()
            .any(|t| t.to_lowercase() == ext_lower || t == "*")
    }

    /// Check if this tool can handle a given file path
    pub fn can_open(&self, file_path: &Path) -> bool {
        file_path
            .extension()
            .and_then(|e| e.to_str())
            .map(|ext| self.supports_file_type(ext))
            .unwrap_or(false)
    }

    /// Build the command arguments by replacing placeholders
    fn build_arguments(&self, file_path: &str, context: Option<&ToolContext>) -> Vec<String> {
        let mut args_str = self.arguments_template.clone();

        // Replace {file} placeholder
        args_str = args_str.replace("{file}", file_path);

        // Replace context placeholders if context is provided
        if let Some(ctx) = context {
            if let Some(offset) = &ctx.offset {
                args_str = args_str.replace("{offset}", offset);
                // Also replace {offset_dec} with decimal version
                if let Ok(dec_offset) = usize::from_str_radix(offset.trim_start_matches("0x"), 16) {
                    args_str = args_str.replace("{offset_dec}", &dec_offset.to_string());
                }
            }
            if let Some(size) = ctx.size {
                args_str = args_str.replace("{size}", &size.to_string());
            }
            if let Some(snes_addr) = &ctx.snes_address {
                args_str = args_str.replace("{snes_address}", snes_addr);
            }
            if let Some(category) = &ctx.category {
                args_str = args_str.replace("{category}", category);
            }
            if let Some(boxer) = &ctx.boxer {
                args_str = args_str.replace("{boxer}", boxer);
            }

            // Replace custom metadata placeholders
            for (key, value) in &ctx.metadata {
                args_str = args_str.replace(&format!("{{{}}}", key), value);
            }
        }

        // Split into arguments (handling quoted strings)
        Self::split_args(&args_str)
    }

    /// Split an argument string into individual arguments, respecting quotes
    fn split_args(args_str: &str) -> Vec<String> {
        let mut args = Vec::new();
        let mut current = String::new();
        let mut in_quotes = false;

        for c in args_str.chars() {
            match c {
                '"' => {
                    in_quotes = !in_quotes;
                }
                ' ' | '\t' if !in_quotes => {
                    if !current.is_empty() {
                        args.push(current.clone());
                        current.clear();
                    }
                }
                _ => {
                    current.push(c);
                }
            }
        }

        if !current.is_empty() {
            args.push(current);
        }

        args
    }

    /// Launch the tool with the given file and context
    pub fn launch(&self, file_path: &str, context: Option<&ToolContext>) -> Result<(), ToolError> {
        let exe_path = Path::new(&self.executable_path);

        if !exe_path.exists() {
            return Err(ToolError::ExecutableNotFound(self.executable_path.clone()));
        }

        let args = self.build_arguments(file_path, context);

        let mut cmd = Command::new(exe_path);
        cmd.args(&args);

        // Set working directory if specified
        if let Some(work_dir) = &self.working_directory {
            cmd.current_dir(work_dir);
        }

        // Set environment variables
        for (key, value) in &self.env_vars {
            cmd.env(key, value);
        }

        // Spawn the process (don't wait)
        let _child = cmd
            .spawn()
            .map_err(|e| ToolError::LaunchFailed(format!("Failed to spawn process: {}", e)))?;

        Ok(())
    }

    /// Verify that the tool executable exists and is accessible
    pub fn verify(&self) -> Result<(), ToolError> {
        let path = Path::new(&self.executable_path);
        if !path.exists() {
            return Err(ToolError::ExecutableNotFound(self.executable_path.clone()));
        }

        // Check if it's a file (not a directory)
        if !path.is_file() {
            return Err(ToolError::InvalidExecutable(self.executable_path.clone()));
        }

        Ok(())
    }
}

/// Configuration for all external tools
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolHooksConfig {
    /// List of configured tools
    pub tools: Vec<ExternalTool>,
    /// Default tool IDs for each file extension
    pub default_tools: HashMap<String, String>,
    /// Whether to show tool launch notifications
    pub show_notifications: bool,
}

impl Default for ToolHooksConfig {
    fn default() -> Self {
        Self {
            tools: Vec::new(),
            default_tools: HashMap::new(),
            show_notifications: true,
        }
    }
}

impl ToolHooksConfig {
    /// Create a new empty configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a tool to the configuration
    pub fn add_tool(&mut self, tool: ExternalTool) {
        // Remove existing tool with same ID
        self.tools.retain(|t| t.id != tool.id);
        self.tools.push(tool);
    }

    /// Remove a tool by ID
    pub fn remove_tool(&mut self, tool_id: &str) -> bool {
        let original_len = self.tools.len();
        self.tools.retain(|t| t.id != tool_id);

        // Also remove from default tools if present
        self.default_tools.retain(|_, id| id != tool_id);

        self.tools.len() < original_len
    }

    /// Get a tool by ID
    pub fn get_tool(&self, tool_id: &str) -> Option<&ExternalTool> {
        self.tools.iter().find(|t| t.id == tool_id)
    }

    /// Get a mutable reference to a tool by ID
    pub fn get_tool_mut(&mut self, tool_id: &str) -> Option<&mut ExternalTool> {
        self.tools.iter_mut().find(|t| t.id == tool_id)
    }

    /// Set the default tool for a file extension
    pub fn set_default_tool(&mut self, file_extension: &str, tool_id: &str) {
        self.default_tools
            .insert(file_extension.to_lowercase(), tool_id.to_string());
    }

    /// Get the default tool for a file extension
    pub fn get_default_tool(&self, file_extension: &str) -> Option<&ExternalTool> {
        let ext_lower = file_extension.to_lowercase();
        self.default_tools
            .get(&ext_lower)
            .and_then(|id| self.get_tool(id))
    }

    /// Get all tools that can handle a given file extension
    pub fn get_compatible_tools(&self, file_extension: &str) -> Vec<&ExternalTool> {
        let ext_lower = file_extension.to_lowercase();
        self.tools
            .iter()
            .filter(|t| t.enabled && t.supports_file_type(&ext_lower))
            .collect()
    }

    /// Get all tools in a specific category
    pub fn get_tools_by_category(&self, category: ToolCategory) -> Vec<&ExternalTool> {
        self.tools
            .iter()
            .filter(|t| t.enabled && t.category == category)
            .collect()
    }

    /// Launch a file with a specific tool
    pub fn launch_with_tool(
        &self,
        tool_id: &str,
        file_path: &str,
        context: Option<&ToolContext>,
    ) -> Result<(), ToolError> {
        let tool = self
            .get_tool(tool_id)
            .ok_or_else(|| ToolError::ToolNotFound(tool_id.to_string()))?;

        if !tool.enabled {
            return Err(ToolError::ToolDisabled(tool_id.to_string()));
        }

        tool.launch(file_path, context)
    }

    /// Launch a file with the default tool for its extension
    pub fn launch_with_default(
        &self,
        file_path: &str,
        file_extension: &str,
        context: Option<&ToolContext>,
    ) -> Result<(), ToolError> {
        if let Some(tool) = self.get_default_tool(file_extension) {
            tool.launch(file_path, context)
        } else {
            Err(ToolError::NoDefaultTool(file_extension.to_string()))
        }
    }

    /// Load configuration from a file
    pub fn load(path: &Path) -> Result<Self, ToolError> {
        if !path.exists() {
            return Ok(Self::default());
        }

        let content =
            std::fs::read_to_string(path).map_err(|e| ToolError::IoError(e.to_string()))?;

        let config: ToolHooksConfig =
            serde_json::from_str(&content).map_err(|e| ToolError::ParseError(e.to_string()))?;

        Ok(config)
    }

    /// Save configuration to a file
    pub fn save(&self, path: &Path) -> Result<(), ToolError> {
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| ToolError::SerializeError(e.to_string()))?;

        std::fs::write(path, content).map_err(|e| ToolError::IoError(e.to_string()))?;

        Ok(())
    }

    /// Get preset tools for common applications
    pub fn get_preset_tools() -> Vec<ExternalTool> {
        vec![
            // Tile Layer Pro
            ExternalTool::new(
                "tile_layer_pro",
                "Tile Layer Pro",
                "C:/Program Files (x86)/Tile Layer Pro/TLP.exe",
                ToolCategory::TileEditor,
            )
            .with_file_types(&["bin", "chr", "gb", "nes", "sfc", "smc", "vra", "bmp"]),
            // YY-CHR
            ExternalTool::new(
                "yy_chr",
                "YY-CHR",
                "C:/Program Files/YY-CHR/yy-chr.exe",
                ToolCategory::TileEditor,
            )
            .with_file_types(&["bin", "chr", "nes", "sfc", "smc", "gb", "gbc"]),
            // HxD
            ExternalTool::new(
                "hxd",
                "HxD",
                "C:/Program Files/HxD/HxD.exe",
                ToolCategory::HexEditor,
            )
            .with_file_types(&["bin", "sfc", "smc", "nes", "dat", "ips", "bps"])
            .with_args_template("{file}"),
            // 010 Editor
            ExternalTool::new(
                "010_editor",
                "010 Editor",
                "C:/Program Files/010 Editor/010Editor.exe",
                ToolCategory::HexEditor,
            )
            .with_file_types(&["bin", "sfc", "smc", "nes", "dat", "hex", "1sc", "1pk"])
            .with_args_template("{file}"),
            // Aseprite
            ExternalTool::new(
                "aseprite",
                "Aseprite",
                "C:/Program Files/Aseprite/aseprite.exe",
                ToolCategory::GraphicsEditor,
            )
            .with_file_types(&["png", "ase", "aseprite", "gif", "jpg", "bmp", "tga"]),
            // GIMP
            ExternalTool::new(
                "gimp",
                "GIMP",
                "C:/Program Files/GIMP 2/bin/gimp-2.10.exe",
                ToolCategory::GraphicsEditor,
            )
            .with_file_types(&["png", "jpg", "jpeg", "gif", "bmp", "tga", "xcf"]),
            // Photoshop
            ExternalTool::new(
                "photoshop",
                "Adobe Photoshop",
                "C:/Program Files/Adobe/Adobe Photoshop/Photoshop.exe",
                ToolCategory::GraphicsEditor,
            )
            .with_file_types(&[
                "psd", "png", "jpg", "jpeg", "gif", "bmp", "tga", "tif", "tiff",
            ]),
        ]
    }
}

/// Error type for tool operations
#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    #[error("Executable not found: {0}")]
    ExecutableNotFound(String),

    #[error("Invalid executable: {0}")]
    InvalidExecutable(String),

    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    #[error("Tool is disabled: {0}")]
    ToolDisabled(String),

    #[error("No default tool configured for: {0}")]
    NoDefaultTool(String),

    #[error("Launch failed: {0}")]
    LaunchFailed(String),

    #[error("IO error: {0}")]
    IoError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Serialize error: {0}")]
    SerializeError(String),
}

/// Result type for tool operations
pub type ToolResult<T> = Result<T, ToolError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_category_from_str() {
        assert!(matches!(
            "graphics_editor".parse::<ToolCategory>(),
            Ok(ToolCategory::GraphicsEditor)
        ));
        assert!(matches!(
            "hex".parse::<ToolCategory>(),
            Ok(ToolCategory::HexEditor)
        ));
        assert!(matches!(
            "tile".parse::<ToolCategory>(),
            Ok(ToolCategory::TileEditor)
        ));
    }

    #[test]
    fn test_external_tool_supports_file_type() {
        let tool = ExternalTool::new(
            "test",
            "Test Tool",
            "/path/to/tool",
            ToolCategory::HexEditor,
        );

        assert!(tool.supports_file_type("bin"));
        assert!(tool.supports_file_type("BIN")); // Case insensitive
        assert!(!tool.supports_file_type("png"));
    }

    #[test]
    fn test_build_arguments() {
        let tool = ExternalTool::new(
            "test",
            "Test Tool",
            "/path/to/tool",
            ToolCategory::HexEditor,
        )
        .with_args_template("{file} --offset {offset} --size {size}");

        let context = ToolContext {
            offset: Some("0x1234".to_string()),
            size: Some(100),
            ..Default::default()
        };

        let args = tool.build_arguments("test.bin", Some(&context));
        assert_eq!(
            args,
            vec!["test.bin", "--offset", "0x1234", "--size", "100"]
        );
    }

    #[test]
    fn test_split_args() {
        let args = ExternalTool::split_args("file.bin --offset 0x100");
        assert_eq!(args, vec!["file.bin", "--offset", "0x100"]);

        let args = ExternalTool::split_args("file.bin \"path with spaces\" --opt");
        assert_eq!(args, vec!["file.bin", "path with spaces", "--opt"]);
    }

    #[test]
    fn test_tool_hooks_config() {
        let mut config = ToolHooksConfig::new();

        let tool = ExternalTool::new("test", "Test", "/path", ToolCategory::HexEditor);

        config.add_tool(tool.clone());
        assert_eq!(config.tools.len(), 1);

        config.set_default_tool("bin", "test");
        assert!(config.get_default_tool("bin").is_some());

        let compatible = config.get_compatible_tools("bin");
        assert_eq!(compatible.len(), 1);
    }
}
