//! Plugin manager for loading and managing plugins

use crate::api::PluginApi;
use crate::lua_runtime::LuaPlugin;
use crate::types::*;
use crate::{EditorEvent, EditorPlugin, PluginConfig, PluginContext, PluginError, PluginInfo, PluginMenuItem, PluginResult, PluginToolbarButton};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Manages all loaded plugins
pub struct PluginManager {
    /// Loaded plugins by ID
    plugins: Arc<RwLock<HashMap<String, Box<dyn EditorPlugin>>>>,
    /// Plugin configurations
    configs: Arc<RwLock<HashMap<String, PluginConfig>>>,
    /// Plugin context
    context: Arc<RwLock<PluginContext>>,
    /// API instance shared with plugins
    api: Arc<PluginApi>,
    /// Directory where plugins are stored
    plugins_dir: PathBuf,
    /// Directory for plugin configuration
    config_dir: PathBuf,
    /// Directory for plugin data
    data_dir: PathBuf,
}

impl PluginManager {
    /// Create a new plugin manager
    pub fn new(
        plugins_dir: PathBuf,
        config_dir: PathBuf,
        data_dir: PathBuf,
        api: Arc<PluginApi>,
    ) -> Self {
        let context = Arc::new(RwLock::new(PluginContext::new(
            config_dir.clone(),
            data_dir.clone(),
        )));
        
        Self {
            plugins: Arc::new(RwLock::new(HashMap::new())),
            configs: Arc::new(RwLock::new(HashMap::new())),
            context,
            api,
            plugins_dir,
            config_dir,
            data_dir,
        }
    }
    
    /// Initialize the plugin manager and load all plugins
    pub fn initialize(&self) -> PluginResult<()> {
        // Ensure directories exist
        std::fs::create_dir_all(&self.plugins_dir)?;
        std::fs::create_dir_all(&self.config_dir)?;
        std::fs::create_dir_all(&self.data_dir)?;
        
        // Load all plugins from directory
        self.load_all_plugins()?;
        
        Ok(())
    }
    
    /// Load all plugins from the plugins directory
    pub fn load_all_plugins(&self) -> PluginResult<Vec<PluginInfo>> {
        let mut loaded = Vec::new();
        
        if !self.plugins_dir.exists() {
            return Ok(loaded);
        }
        
        for entry in std::fs::read_dir(&self.plugins_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            // Check if it's a Lua file
            if path.extension().and_then(|s| s.to_str()) == Some("lua") {
                match self.load_plugin(&path) {
                    Ok(info) => {
                        log::info!("Loaded plugin: {} ({})", info.name, info.id);
                        loaded.push(info);
                    }
                    Err(e) => {
                        log::error!("Failed to load plugin {:?}: {}", path, e);
                    }
                }
            }
        }
        
        Ok(loaded)
    }
    
    /// Load a single plugin from file
    pub fn load_plugin<P: AsRef<Path>>(&self, path: P) -> PluginResult<PluginInfo> {
        let path = path.as_ref();
        
        // Load based on file extension
        let mut plugin: Box<dyn EditorPlugin> = if path.extension().and_then(|s| s.to_str()) == Some("lua") {
            Box::new(LuaPlugin::from_file(path, self.api.clone())?)
        } else {
            return Err(PluginError::ApiError("Unsupported plugin file type".into()));
        };
        
        let info = plugin.info().clone();
        
        // Check if already loaded
        {
            let plugins = self.plugins.read();
            if plugins.contains_key(&info.id) {
                return Err(PluginError::AlreadyLoaded(info.id.clone()));
            }
        }
        
        // Load config if exists
        let config = self.load_plugin_config(&info.id)?;
        
        // Initialize if enabled
        if config.enabled {
            let ctx = self.context.read().clone();
            plugin.initialize(&ctx)?;
        }
        
        // Store plugin and config
        {
            let mut plugins = self.plugins.write();
            plugins.insert(info.id.clone(), plugin);
        }
        {
            let mut configs = self.configs.write();
            configs.insert(info.id.clone(), config);
        }
        
        Ok(info)
    }
    
    /// Unload a plugin
    pub fn unload_plugin(&self, plugin_id: &str) -> PluginResult<()> {
        let mut plugins = self.plugins.write();
        
        if let Some(mut plugin) = plugins.remove(plugin_id) {
            plugin.shutdown()?;
        }
        
        Ok(())
    }
    
    /// Reload a plugin
    pub fn reload_plugin(&self, plugin_id: &str) -> PluginResult<PluginInfo> {
        let path = {
            let plugins = self.plugins.read();
            if let Some(plugin) = plugins.get(plugin_id) {
                plugin.info().path.clone()
            } else {
                return Err(PluginError::NotFound(plugin_id.into()));
            }
        };
        
        self.unload_plugin(plugin_id)?;
        self.load_plugin(&path)
    }
    
    /// Get information about a loaded plugin
    pub fn get_plugin_info(&self, plugin_id: &str) -> Option<PluginInfo> {
        let plugins = self.plugins.read();
        plugins.get(plugin_id).map(|p| p.info().clone())
    }
    
    /// Get information about all loaded plugins
    pub fn get_all_plugins(&self) -> Vec<PluginInfo> {
        let plugins = self.plugins.read();
        plugins.values().map(|p| p.info().clone()).collect()
    }
    
    /// Enable a plugin
    pub fn enable_plugin(&self, plugin_id: &str) -> PluginResult<()> {
        {
            let mut configs = self.configs.write();
            if let Some(config) = configs.get_mut(plugin_id) {
                config.enabled = true;
            }
        }
        
        let mut plugins = self.plugins.write();
        if let Some(plugin) = plugins.get_mut(plugin_id) {
            let ctx = self.context.read().clone();
            plugin.initialize(&ctx)?;
        }
        
        Ok(())
    }
    
    /// Disable a plugin
    pub fn disable_plugin(&self, plugin_id: &str) -> PluginResult<()> {
        {
            let mut plugins = self.plugins.write();
            if let Some(plugin) = plugins.get_mut(plugin_id) {
                plugin.shutdown()?;
            }
        }
        
        let mut configs = self.configs.write();
        if let Some(config) = configs.get_mut(plugin_id) {
            config.enabled = false;
        }
        
        Ok(())
    }
    
    /// Execute a plugin command
    pub fn execute_command(
        &self,
        plugin_id: &str,
        command: &str,
        args: &serde_json::Value,
    ) -> PluginResult<serde_json::Value> {
        let mut plugins = self.plugins.write();
        
        if let Some(plugin) = plugins.get_mut(plugin_id) {
            plugin.execute_command(command, args)
        } else {
            Err(PluginError::NotFound(plugin_id.into()))
        }
    }
    
    /// Broadcast an event to all enabled plugins
    pub fn broadcast_event(&self, event: &EditorEvent) -> PluginResult<()> {
        let plugins_to_notify: Vec<String> = {
            let plugins = self.plugins.read();
            let configs = self.configs.read();
            
            plugins
                .iter()
                .filter(|(id, _)| {
                    configs.get(*id).map(|c| c.enabled).unwrap_or(false)
                })
                .map(|(id, _)| id.clone())
                .collect()
        };
        
        for plugin_id in plugins_to_notify {
            let mut plugins = self.plugins.write();
            if let Some(plugin) = plugins.get_mut(&plugin_id) {
                let ctx = self.context.read().clone();
                if let Err(e) = plugin.on_event(event, &ctx) {
                    log::error!("Plugin {} failed to handle event: {}", plugin_id, e);
                }
            }
        }
        
        Ok(())
    }
    
    /// Get all menu items contributed by plugins
    pub fn get_menu_items(&self) -> Vec<(String, PluginMenuItem)> {
        // This would collect menu items from all plugins
        // For now, return empty
        Vec::new()
    }
    
    /// Get all toolbar buttons contributed by plugins
    pub fn get_toolbar_buttons(&self) -> Vec<(String, PluginToolbarButton)> {
        // This would collect toolbar buttons from all plugins
        // For now, return empty
        Vec::new()
    }
    
    /// Get all hotkeys from all plugins
    pub fn get_hotkeys(&self) -> Vec<(String, PluginHotkey)> {
        let mut hotkeys = Vec::new();
        
        let configs = self.configs.read();
        for (plugin_id, config) in configs.iter() {
            for hotkey in &config.hotkeys {
                hotkeys.push((plugin_id.clone(), hotkey.clone()));
            }
        }
        
        hotkeys
    }
    
    /// Update plugin configuration
    pub fn update_config(&self, plugin_id: &str, config: PluginConfig) -> PluginResult<()> {
        let mut configs = self.configs.write();
        configs.insert(plugin_id.into(), config);
        
        // Save to disk
        self.save_plugin_config(plugin_id)?;
        
        Ok(())
    }
    
    /// Get plugin configuration
    pub fn get_config(&self, plugin_id: &str) -> Option<PluginConfig> {
        let configs = self.configs.read();
        configs.get(plugin_id).cloned()
    }
    
    /// Shutdown all plugins
    pub fn shutdown(&self) -> PluginResult<()> {
        let mut plugins = self.plugins.write();
        
        for (id, plugin) in plugins.iter_mut() {
            if let Err(e) = plugin.shutdown() {
                log::error!("Failed to shutdown plugin {}: {}", id, e);
            }
        }
        
        plugins.clear();
        Ok(())
    }
    
    // ============================================================================
    // Configuration Management
    // ============================================================================
    
    fn load_plugin_config(&self, plugin_id: &str) -> PluginResult<PluginConfig> {
        let config_path = self.config_dir.join(format!("{}.json", plugin_id));
        
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config: PluginConfig = serde_json::from_str(&content)?;
            Ok(config)
        } else {
            Ok(PluginConfig::default())
        }
    }
    
    fn save_plugin_config(&self, plugin_id: &str) -> PluginResult<()> {
        let config_path = self.config_dir.join(format!("{}.json", plugin_id));
        
        let configs = self.configs.read();
        if let Some(config) = configs.get(plugin_id) {
            let content = serde_json::to_string_pretty(config)?;
            std::fs::write(&config_path, content)?;
        }
        
        Ok(())
    }
    
    /// Save all plugin configurations
    pub fn save_all_configs(&self) -> PluginResult<()> {
        let configs = self.configs.read();
        
        for plugin_id in configs.keys() {
            self.save_plugin_config(plugin_id)?;
        }
        
        Ok(())
    }
    
    // ============================================================================
    // Context Updates
    // ============================================================================
    
    /// Update the ROM data in context
    pub fn set_rom_data(&self, rom_data: Option<Arc<RwLock<Vec<u8>>>>) {
        self.context.write().rom_data = rom_data;
    }
    
    /// Update the project path in context
    pub fn set_project_path(&self, path: Option<PathBuf>) {
        self.context.write().project_path = path;
    }
    
    /// Update the selected boxer in context
    pub fn set_selected_boxer(&self, boxer: Option<String>) {
        self.context.write().selected_boxer = boxer;
    }
}

/// Plugin script runner for executing one-off scripts
pub struct ScriptRunner {
    api: Arc<PluginApi>,
}

impl ScriptRunner {
    pub fn new(api: Arc<PluginApi>) -> Self {
        Self { api }
    }
    
    /// Run a Lua script file
    pub fn run_file<P: AsRef<Path>>(&self, path: P) -> PluginResult<ScriptResult> {
        let path_ref = path.as_ref();
        let script = std::fs::read_to_string(path_ref)
            .map_err(|e| PluginError::Io(e))?;
        
        self.run_string(&script, Some(path_ref.to_string_lossy().as_ref()))
    }
    
    /// Run a Lua script from string
    pub fn run_string(&self, script: &str, name: Option<&str>) -> PluginResult<ScriptResult> {
        let start_time = std::time::Instant::now();
        
        let lua = mlua::Lua::new();
        
        // Setup API bindings (simplified version)
        let api_table = lua.create_table().map_err(|e| PluginError::LuaError(e.to_string()))?;
        
        // Add basic API functions
        {
            let api = self.api.clone();
            api_table.set(
                "rom_read",
                lua.create_function(move |_, (offset, length): (usize, usize)| {
                    api.rom_read(offset, length)
                        .map_err(|e| mlua::Error::runtime(e.to_string()))
                }).map_err(|e| PluginError::LuaError(e.to_string()))?,
            ).map_err(|e| PluginError::LuaError(e.to_string()))?;
        }
        
        {
            let api = self.api.clone();
            api_table.set(
                "log_info",
                lua.create_function(move |_, msg: String| {
                    api.log_info(&msg);
                    Ok(())
                }).map_err(|e| PluginError::LuaError(e.to_string()))?,
            ).map_err(|e| PluginError::LuaError(e.to_string()))?;
        }
        
        lua.globals().set("SPO", api_table).map_err(|e| PluginError::LuaError(e.to_string()))?;
        
        // Execute the script
        let result = lua.load(script)
            .set_name(name.unwrap_or("script"))
            .exec();
        
        let execution_time = start_time.elapsed().as_millis() as u64;
        
        match result {
            Ok(_) => Ok(ScriptResult {
                success: true,
                output: "Script executed successfully".into(),
                error: None,
                return_value: None,
                execution_time_ms: execution_time,
            }),
            Err(e) => Ok(ScriptResult {
                success: false,
                output: String::new(),
                error: Some(e.to_string()),
                return_value: None,
                execution_time_ms: execution_time,
            }),
        }
    }
}
