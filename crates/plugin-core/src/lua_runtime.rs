//! Lua scripting runtime for plugins
//!
//! Uses a dedicated thread for Lua operations since mlua::Lua is not Send/Sync.

use crate::api::PluginApi;
use crate::types::*;
use crate::{EditorEvent, EditorPlugin, PluginCommand, PluginContext, PluginError, PluginInfo, PluginResult, PluginType};
use std::path::Path;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

/// Messages sent to the Lua worker thread
enum LuaMessage {
    Initialize,
    Shutdown,
    OnEvent(EditorEvent),
    ExecuteCommand(String, serde_json::Value, Sender<PluginResult<serde_json::Value>>),
    GetCommands(Sender<Vec<PluginCommand>>),
}

/// Lua plugin that runs scripts in a dedicated thread
pub struct LuaPlugin {
    info: PluginInfo,
    tx: Sender<LuaMessage>,
    _worker: JoinHandle<()>,
    commands: Vec<PluginCommand>,
}

impl LuaPlugin {
    /// Load a Lua plugin from file
    pub fn from_file<P: AsRef<Path>>(
        path: P,
        api: Arc<PluginApi>,
    ) -> PluginResult<Self> {
        let path = path.as_ref().to_path_buf();
        let path_for_worker = path.clone();
        
        // Create channels for communication
        let (tx, rx): (Sender<LuaMessage>, Receiver<LuaMessage>) = channel();
        
        // Spawn worker thread
        let worker = thread::spawn(move || {
            LuaWorker::new(path_for_worker, api, rx).run();
        });
        
        // Create initial plugin with placeholder info
        // The worker will extract the real info from the script
        let info = PluginInfo {
            id: "loading".into(),
            name: "Loading...".into(),
            version: "0.0.0".into(),
            author: "Unknown".into(),
            description: "Loading plugin...".into(),
            api_version: 1,
            plugin_type: PluginType::Lua,
            enabled: false,
            path: path.clone(),
            loaded_at: chrono::Utc::now(),
        };
        
        Ok(Self {
            info,
            tx,
            _worker: worker,
            commands: Vec::new(),
        })
    }
    
    /// Update plugin info (called after worker extracts it from script)
    fn update_info(&mut self, info: PluginInfo) {
        self.info = info;
    }
}

impl EditorPlugin for LuaPlugin {
    fn info(&self) -> &PluginInfo {
        &self.info
    }
    
    fn initialize(&mut self, _ctx: &PluginContext) -> PluginResult<()> {
        self.tx.send(LuaMessage::Initialize)
            .map_err(|_| PluginError::PluginCrashed("Worker thread died".into()))?;
        
        // Get commands from worker
        let (tx, rx) = channel();
        self.tx.send(LuaMessage::GetCommands(tx))
            .map_err(|_| PluginError::PluginCrashed("Worker thread died".into()))?;
        
        self.commands = rx.recv()
            .map_err(|_| PluginError::PluginCrashed("Worker thread died".into()))?;
        
        Ok(())
    }
    
    fn shutdown(&mut self) -> PluginResult<()> {
        let _ = self.tx.send(LuaMessage::Shutdown);
        Ok(())
    }
    
    fn on_event(&mut self, event: &EditorEvent, _ctx: &PluginContext) -> PluginResult<()> {
        self.tx.send(LuaMessage::OnEvent(event.clone()))
            .map_err(|_| PluginError::PluginCrashed("Worker thread died".into()))?;
        Ok(())
    }
    
    fn execute_command(&mut self, command: &str, args: &serde_json::Value) -> PluginResult<serde_json::Value> {
        let (tx, rx) = channel();
        self.tx.send(LuaMessage::ExecuteCommand(command.into(), args.clone(), tx))
            .map_err(|_| PluginError::PluginCrashed("Worker thread died".into()))?;
        
        rx.recv()
            .map_err(|_| PluginError::PluginCrashed("Worker thread died".into()))?
    }
    
    fn available_commands(&self) -> Vec<PluginCommand> {
        self.commands.clone()
    }
}

/// Worker thread that owns the Lua state
struct LuaWorker {
    lua: mlua::Lua,
    api: Arc<PluginApi>,
    rx: Receiver<LuaMessage>,
    info: Option<PluginInfo>,
}

impl LuaWorker {
    fn new(path: std::path::PathBuf, api: Arc<PluginApi>, rx: Receiver<LuaMessage>) -> Self {
        let lua = mlua::Lua::new();
        
        let mut worker = Self {
            lua,
            api,
            rx,
            info: None,
        };
        
        // Load and setup the script
        if let Err(e) = worker.load_script(&path) {
            log::error!("Failed to load Lua script: {}", e);
        }
        
        worker
    }
    
    fn load_script(&mut self, path: &Path) -> PluginResult<()> {
        // Read the script
        let script = std::fs::read_to_string(path)
            .map_err(|e| PluginError::Io(e))?;
        
        // Load and execute the script
        self.lua.load(&script)
            .set_name(path.file_name().unwrap_or_default().to_string_lossy())
            .exec()
            .map_err(|e| PluginError::LuaError(e.to_string()))?;
        
        // Extract plugin info - do all Lua operations first
        let (id, name, version, author, description, api_version) = {
            let info_table: mlua::Table = self.lua.globals()
                .get("PLUGIN_INFO")
                .map_err(|_| PluginError::LuaError("Missing PLUGIN_INFO table".into()))?;
            
            let id: String = info_table.get("id")
                .map_err(|_| PluginError::LuaError("Missing PLUGIN_INFO.id".into()))?;
            let name: String = info_table.get("name")
                .unwrap_or_else(|_| "Unnamed Plugin".into());
            let version: String = info_table.get("version")
                .unwrap_or_else(|_| "0.1.0".into());
            let author: String = info_table.get("author")
                .unwrap_or_else(|_| "Unknown".into());
            let description: String = info_table.get("description")
                .unwrap_or_else(|_| "".into());
            let api_version: u32 = info_table.get("api_version")
                .unwrap_or(1);
            
            (id, name, version, author, description, api_version)
        }; // Table is dropped here, releasing the borrow
        
        let info = PluginInfo {
            id,
            name,
            version,
            author,
            description,
            api_version,
            plugin_type: PluginType::Lua,
            enabled: true,
            path: path.to_path_buf(),
            loaded_at: chrono::Utc::now(),
        };
        
        self.info = Some(info);
        
        // Setup API bindings
        self.setup_api_bindings()?;
        
        Ok(())
    }
    
    fn setup_api_bindings(&mut self) -> PluginResult<()> {
        let api = self.api.clone();
        
        // Create API table
        let api_table = self.lua.create_table()
            .map_err(|e| PluginError::LuaError(e.to_string()))?;
        
        // ROM read function
        {
            let api = api.clone();
            api_table.set(
                "rom_read",
                self.lua.create_function(move |_, (offset, length): (usize, usize)| {
                    api.rom_read(offset, length)
                        .map_err(|e| mlua::Error::runtime(e.to_string()))
                }).map_err(|e| PluginError::LuaError(e.to_string()))?,
            ).map_err(|e| PluginError::LuaError(e.to_string()))?;
        }
        
        // ROM write function
        {
            let api = api.clone();
            api_table.set(
                "rom_write",
                self.lua.create_function(move |_, (offset, data): (usize, mlua::String)| {
                    api.rom_write(offset, data.as_bytes())
                        .map_err(|e| mlua::Error::runtime(e.to_string()))
                }).map_err(|e| PluginError::LuaError(e.to_string()))?,
            ).map_err(|e| PluginError::LuaError(e.to_string()))?;
        }
        
        // ROM read byte
        {
            let api = api.clone();
            api_table.set(
                "rom_read_byte",
                self.lua.create_function(move |_, offset: usize| {
                    api.rom_read_byte(offset)
                        .map_err(|e| mlua::Error::runtime(e.to_string()))
                }).map_err(|e| PluginError::LuaError(e.to_string()))?,
            ).map_err(|e| PluginError::LuaError(e.to_string()))?;
        }
        
        // ROM write byte
        {
            let api = api.clone();
            api_table.set(
                "rom_write_byte",
                self.lua.create_function(move |_, (offset, value): (usize, u8)| {
                    api.rom_write_byte(offset, value)
                        .map_err(|e| mlua::Error::runtime(e.to_string()))
                }).map_err(|e| PluginError::LuaError(e.to_string()))?,
            ).map_err(|e| PluginError::LuaError(e.to_string()))?;
        }
        
        // ROM size
        {
            let api = api.clone();
            api_table.set(
                "rom_size",
                self.lua.create_function(move |_, ()| {
                    api.rom_size()
                        .map_err(|e| mlua::Error::runtime(e.to_string()))
                }).map_err(|e| PluginError::LuaError(e.to_string()))?,
            ).map_err(|e| PluginError::LuaError(e.to_string()))?;
        }
        
        // Logging functions
        {
            let api = api.clone();
            api_table.set(
                "log_info",
                self.lua.create_function(move |_, msg: String| {
                    api.log_info(&msg);
                    Ok(())
                }).map_err(|e| PluginError::LuaError(e.to_string()))?,
            ).map_err(|e| PluginError::LuaError(e.to_string()))?;
        }
        
        {
            let api = api.clone();
            api_table.set(
                "log_debug",
                self.lua.create_function(move |_, msg: String| {
                    api.log_debug(&msg);
                    Ok(())
                }).map_err(|e| PluginError::LuaError(e.to_string()))?,
            ).map_err(|e| PluginError::LuaError(e.to_string()))?;
        }
        
        {
            let api = api.clone();
            api_table.set(
                "log_warn",
                self.lua.create_function(move |_, msg: String| {
                    api.log_warn(&msg);
                    Ok(())
                }).map_err(|e| PluginError::LuaError(e.to_string()))?,
            ).map_err(|e| PluginError::LuaError(e.to_string()))?;
        }
        
        {
            let api = api.clone();
            api_table.set(
                "log_error",
                self.lua.create_function(move |_, msg: String| {
                    api.log_error(&msg);
                    Ok(())
                }).map_err(|e| PluginError::LuaError(e.to_string()))?,
            ).map_err(|e| PluginError::LuaError(e.to_string()))?;
        }
        
        // Notifications
        {
            let api = api.clone();
            api_table.set(
                "notify_info",
                self.lua.create_function(move |_, msg: String| {
                    api.notify_info(&msg);
                    Ok(())
                }).map_err(|e| PluginError::LuaError(e.to_string()))?,
            ).map_err(|e| PluginError::LuaError(e.to_string()))?;
        }
        
        {
            let api = api.clone();
            api_table.set(
                "notify_success",
                self.lua.create_function(move |_, msg: String| {
                    api.notify_success(&msg);
                    Ok(())
                }).map_err(|e| PluginError::LuaError(e.to_string()))?,
            ).map_err(|e| PluginError::LuaError(e.to_string()))?;
        }
        
        {
            let api = api.clone();
            api_table.set(
                "notify_error",
                self.lua.create_function(move |_, msg: String| {
                    api.notify_error(&msg);
                    Ok(())
                }).map_err(|e| PluginError::LuaError(e.to_string()))?,
            ).map_err(|e| PluginError::LuaError(e.to_string()))?;
        }
        
        // Address conversion
        {
            let api = api.clone();
            api_table.set(
                "snes_to_pc",
                self.lua.create_function(move |_, (bank, addr): (u8, u16)| {
                    Ok(api.snes_to_pc(bank, addr))
                }).map_err(|e| PluginError::LuaError(e.to_string()))?,
            ).map_err(|e| PluginError::LuaError(e.to_string()))?;
        }
        
        {
            let api = api.clone();
            api_table.set(
                "pc_to_snes",
                self.lua.create_function(move |_, pc: usize| {
                    Ok(api.pc_to_snes(pc))
                }).map_err(|e| PluginError::LuaError(e.to_string()))?,
            ).map_err(|e| PluginError::LuaError(e.to_string()))?;
        }
        
        // Pattern search
        {
            let api = api.clone();
            api_table.set(
                "find_pattern",
                self.lua.create_function(move |_, pattern: mlua::String| {
                    api.find_pattern(pattern.as_bytes())
                        .map_err(|e| mlua::Error::runtime(e.to_string()))
                }).map_err(|e| PluginError::LuaError(e.to_string()))?,
            ).map_err(|e| PluginError::LuaError(e.to_string()))?;
        }
        
        // Store API table in globals
        self.lua.globals().set("SPO", api_table)
            .map_err(|e| PluginError::LuaError(e.to_string()))?;
        
        Ok(())
    }
    
    fn run(mut self) {
        while let Ok(msg) = self.rx.recv() {
            match msg {
                LuaMessage::Initialize => {
                    // Call on_init if exists
                    if let Ok(on_init) = self.lua.globals().get::<_, mlua::Function>("on_init") {
                        let _ = on_init.call::<_, ()>(());
                    }
                }
                LuaMessage::Shutdown => {
                    // Call on_shutdown if exists
                    if let Ok(on_shutdown) = self.lua.globals().get::<_, mlua::Function>("on_shutdown") {
                        let _ = on_shutdown.call::<_, ()>(());
                    }
                    break;
                }
                LuaMessage::OnEvent(event) => {
                    match event {
                        EditorEvent::RomLoaded => {
                            if let Ok(handler) = self.lua.globals().get::<_, mlua::Function>("on_rom_loaded") {
                                let _ = handler.call::<_, ()>(());
                            }
                        }
                        EditorEvent::RomSaving => {
                            if let Ok(handler) = self.lua.globals().get::<_, mlua::Function>("on_rom_saving") {
                                let _ = handler.call::<_, ()>(());
                            }
                        }
                        EditorEvent::AssetModified => {
                            if let Ok(handler) = self.lua.globals().get::<_, mlua::Function>("on_asset_modified") {
                                let _ = handler.call::<_, ()>(());
                            }
                        }
                        EditorEvent::PaletteEdited => {
                            if let Ok(handler) = self.lua.globals().get::<_, mlua::Function>("on_palette_edited") {
                                let _ = handler.call::<_, ()>(());
                            }
                        }
                        EditorEvent::SpriteEdited => {
                            if let Ok(handler) = self.lua.globals().get::<_, mlua::Function>("on_sprite_edited") {
                                let _ = handler.call::<_, ()>(());
                            }
                        }
                        EditorEvent::AnimationEdited => {
                            if let Ok(handler) = self.lua.globals().get::<_, mlua::Function>("on_animation_edited") {
                                let _ = handler.call::<_, ()>(());
                            }
                        }
                        EditorEvent::ProjectCreated => {
                            if let Ok(handler) = self.lua.globals().get::<_, mlua::Function>("on_project_created") {
                                let _ = handler.call::<_, ()>(());
                            }
                        }
                        EditorEvent::ProjectOpened => {
                            if let Ok(handler) = self.lua.globals().get::<_, mlua::Function>("on_project_opened") {
                                let _ = handler.call::<_, ()>(());
                            }
                        }
                        EditorEvent::Custom(name) => {
                            let name_copy = name.clone();
                            if let Ok(handler) = self.lua.globals().get::<_, mlua::Function>(name_copy) {
                                let _ = handler.call::<_, ()>(());
                            }
                        }
                    }
                }
                LuaMessage::ExecuteCommand(command, args, tx) => {
                    let result = self.execute_command(&command, &args);
                    let _ = tx.send(result);
                }
                LuaMessage::GetCommands(tx) => {
                    let commands = self.get_commands();
                    let _ = tx.send(commands);
                }
            }
        }
    }
    
    fn execute_command(&mut self, command: &str, args: &serde_json::Value) -> PluginResult<serde_json::Value> {
        // Check COMMANDS table first
        if let Ok(commands) = self.lua.globals().get::<_, mlua::Table>("COMMANDS") {
            if let Ok(func) = commands.get::<_, mlua::Function>(command) {
                let lua_args = json_to_lua(&self.lua, args)
                    .map_err(|e| PluginError::LuaError(e.to_string()))?;
                let result: mlua::Value = func.call(lua_args)
                    .map_err(|e| PluginError::LuaError(e.to_string()))?;
                return lua_to_json(&result)
                    .map_err(|e| PluginError::LuaError(e.to_string()));
            }
        }
        
        // Try global function
        if let Ok(func) = self.lua.globals().get::<_, mlua::Function>(command) {
            let lua_args = json_to_lua(&self.lua, args)
                .map_err(|e| PluginError::LuaError(e.to_string()))?;
            let result: mlua::Value = func.call(lua_args)
                .map_err(|e| PluginError::LuaError(e.to_string()))?;
            return lua_to_json(&result)
                .map_err(|e| PluginError::LuaError(e.to_string()));
        }
        
        Err(PluginError::ApiError(format!("Command '{}' not found", command)))
    }
    
    fn get_commands(&self) -> Vec<PluginCommand> {
        let mut commands = Vec::new();
        
        if let Ok(commands_table) = self.lua.globals().get::<_, mlua::Table>("COMMANDS") {
            for pair in commands_table.pairs::<mlua::String, mlua::Function>() {
                if let Ok((name, _)) = pair {
                    commands.push(PluginCommand::new(
                        name.to_string_lossy(),
                        format!("Lua command: {}", name.to_string_lossy()),
                    ));
                }
            }
        }
        
        commands
    }
}

/// Convert JSON value to Lua value
fn json_to_lua<'a>(lua: &'a mlua::Lua, value: &serde_json::Value) -> Result<mlua::Value<'a>, mlua::Error> {
    let lua_value = match value {
        serde_json::Value::Null => mlua::Value::Nil,
        serde_json::Value::Bool(b) => mlua::Value::Boolean(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                mlua::Value::Integer(i)
            } else {
                mlua::Value::Number(n.as_f64().unwrap_or(0.0))
            }
        }
        serde_json::Value::String(s) => mlua::Value::String(lua.create_string(s)?),
        serde_json::Value::Array(arr) => {
            let table = lua.create_table()?;
            for (i, v) in arr.iter().enumerate() {
                table.set(i + 1, json_to_lua(lua, v)?)?;
            }
            mlua::Value::Table(table)
        }
        serde_json::Value::Object(obj) => {
            let table = lua.create_table()?;
            for (k, v) in obj.iter() {
                table.set(k.clone(), json_to_lua(lua, v)?)?;
            }
            mlua::Value::Table(table)
        }
    };
    Ok(lua_value)
}

/// Convert Lua value to JSON value
fn lua_to_json(value: &mlua::Value) -> Result<serde_json::Value, mlua::Error> {
    let json_value = match value {
        mlua::Value::Nil => serde_json::Value::Null,
        mlua::Value::Boolean(b) => serde_json::Value::Bool(*b),
        mlua::Value::Integer(i) => serde_json::Value::Number((*i).into()),
        mlua::Value::Number(n) => serde_json::json!(n),
        mlua::Value::String(s) => serde_json::Value::String(s.to_string_lossy().into_owned()),
        mlua::Value::Table(t) => {
            // Check if it's an array
            let len = t.raw_len();
            if len > 0 {
                let mut arr = Vec::new();
                for i in 1..=len {
                    let v: mlua::Value = t.raw_get(i).unwrap_or(mlua::Value::Nil);
                    arr.push(lua_to_json(&v)?);
                }
                serde_json::Value::Array(arr)
            } else {
                let mut map = serde_json::Map::new();
                for pair in t.clone().pairs::<mlua::String, mlua::Value>() {
                    let (k, v) = pair?;
                    map.insert(k.to_string_lossy().into_owned(), lua_to_json(&v)?);
                }
                serde_json::Value::Object(map)
            }
        }
        _ => serde_json::Value::Null,
    };
    Ok(json_value)
}

/// Script runner for one-off Lua scripts
pub struct ScriptRunner;

impl ScriptRunner {
    pub fn new() -> Self {
        Self
    }
    
    /// Run a Lua script file
    pub fn run_file<P: AsRef<Path>>(&self, path: P, api: Arc<PluginApi>) -> PluginResult<ScriptResult> {
        let script = std::fs::read_to_string(path)
            .map_err(|e| PluginError::Io(e))?;
        
        self.run_string(&script, api)
    }
    
    /// Run a Lua script from string
    pub fn run_string(&self, script: &str, api: Arc<PluginApi>) -> PluginResult<ScriptResult> {
        let start_time = std::time::Instant::now();
        
        let lua = mlua::Lua::new();
        
        // Setup basic API
        let api_table = lua.create_table()
            .map_err(|e| PluginError::LuaError(e.to_string()))?;
        
        // Add ROM read
        {
            let api = api.clone();
            api_table.set(
                "rom_read",
                lua.create_function(move |_, (offset, length): (usize, usize)| {
                    api.rom_read(offset, length)
                        .map_err(|e| mlua::Error::runtime(e.to_string()))
                }).map_err(|e| PluginError::LuaError(e.to_string()))?,
            ).map_err(|e| PluginError::LuaError(e.to_string()))?;
        }
        
        // Add logging
        {
            let api = api.clone();
            api_table.set(
                "log_info",
                lua.create_function(move |_, msg: String| {
                    api.log_info(&msg);
                    Ok(())
                }).map_err(|e| PluginError::LuaError(e.to_string()))?,
            ).map_err(|e| PluginError::LuaError(e.to_string()))?;
        }
        
        lua.globals().set("SPO", api_table)
            .map_err(|e| PluginError::LuaError(e.to_string()))?;
        
        // Execute
        let result = lua.load(script).exec();
        
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
