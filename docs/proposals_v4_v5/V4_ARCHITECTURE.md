# V4 "Full Power Layer" Architecture

## System Overview

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         SUPER PUNCH-OUT!! EDITOR                        │
│                              V4 "Full Power Layer"                       │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐         │
│  │   REACT / TS    │  │   REACT / TS    │  │   REACT / TS    │         │
│  │  PluginManager  │  │BankVisualization│  │ AnimationPlayer │         │
│  │    (41 KB)      │  │    (42 KB)      │  │    (47 KB)      │         │
│  └────────┬────────┘  └────────┬────────┘  └────────┬────────┘         │
│           │                    │                    │                  │
│  ┌────────▼────────┐  ┌────────▼────────┐  ┌────────▼────────┐         │
│  │   usePlugins    │  │ useBankManage   │  │  useAnimation   │         │
│  │    (15 KB)      │  │    (10 KB)      │  │    (23 KB)      │         │
│  └────────┬────────┘  └────────┬────────┘  └────────┬────────┘         │
│           │                    │                    │                  │
│           └────────────────────┼────────────────────┘                  │
│                                │                                       │
│  ╔═════════════════════════════╧═══════════════════════════════╗       │
│  ║                     TAURI BRIDGE                             ║       │
│  ║                   (invoke commands)                          ║       │
│  ╚═════════════════════════════╤═══════════════════════════════╝       │
│                                │                                       │
├────────────────────────────────┼───────────────────────────────────────┤
│           RUST BACKEND         │                                       │
│                                │                                       │
│  ┌─────────────────────────────┼───────────────────────────────┐       │
│  │     TAURI COMMANDS          │                               │       │
│  │  ┌───────────────────────┐  │  ┌───────────────────────┐    │       │
│  │  │      plugins.rs       │  │  │   bank_management.rs  │    │       │
│  │  │  - 14 plugin commands │  │  │  - 7 bank commands    │    │       │
│  │  │  - Script execution   │  │  │  - Visualization      │    │       │
│  │  │  - Batch jobs         │  │  │  - Defragmentation    │    │       │
│  │  └───────────┬───────────┘  │  └───────────┬───────────┘    │       │
│  │              │              │              │                │       │
│  │  ┌───────────▼───────────┐  │  ┌───────────▼───────────┐    │       │
│  │  │     animation.rs      │  │  │      Other cmds       │    │       │
│  │  │  - 13 anim commands   │  │  │  - ROM operations     │    │       │
│  │  │  - Hitbox editing     │  │  │  - Project mgmt       │    │       │
│  │  │  - Frame control      │  │  │  - Export/import      │    │       │
│  │  └───────────────────────┘  │  └───────────────────────┘    │       │
│  └─────────────────────────────┼───────────────────────────────┘       │
│                                │                                       │
│  ┌─────────────────────────────▼───────────────────────────────┐       │
│  │                    APP STATE                                 │       │
│  │  ┌───────────────────────────────────────────────────────┐  │       │
│  │  │  plugin_manager: Mutex<PluginManager>                 │  │       │
│  │  │  script_runner: Mutex<ScriptRunner>                   │  │       │
│  │  │  rom: Mutex<Option<Rom>>                              │  │       │
│  │  │  manifest: Mutex<Manifest>                            │  │       │
│  │  │  ...                                                  │  │       │
│  │  └───────────────────────────────────────────────────────┘  │       │
│  └─────────────────────────────────────────────────────────────┘       │
│                                │                                       │
├────────────────────────────────┼───────────────────────────────────────┤
│           CORE CRATES          │                                       │
│                                │                                       │
│  ┌─────────────────────────────▼───────────────────────────────┐       │
│  │  plugin-core (NEW)                                          │       │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │       │
│  │  │   types     │  │   api       │  │   manager           │  │       │
│  │  │  - Plugin   │  │  - PluginApi│  │  - PluginManager    │  │       │
│  │  │  - Events   │  │  - Safe ROM │  │  - Load/unload      │  │       │
│  │  │  - Commands │  │    access   │  │  - Enable/disable   │  │       │
│  │  └─────────────┘  └─────────────┘  └─────────────────────┘  │       │
│  │                                                             │       │
│  │  ┌───────────────────────────────────────────────────────┐  │       │
│  │  │                 lua_runtime                           │  │       │
│  │  │  ┌─────────────────┐  ┌─────────────────────────────┐ │  │       │
│  │  │  │   LuaPlugin     │  │      LuaWorker              │ │  │       │
│  │  │  │  - Thread-safe  │──│  - Dedicated thread         │ │  │       │
│  │  │  │    wrapper      │  │  - Runs Lua state           │ │  │       │
│  │  │  │  - Channel comm │  │  - Handles events/commands  │ │  │       │
│  │  │  └─────────────────┘  └─────────────────────────────┘ │  │       │
│  │  └───────────────────────────────────────────────────────┘  │       │
│  └─────────────────────────────────────────────────────────────┘       │
│                                │                                       │
│  ┌─────────────────────────────┼───────────────────────────────┐       │
│  │  asset-core                                                 │       │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │       │
│  │  │  animation  │  │   audio/    │  │   brr/              │  │       │
│  │  │  - Sequence │  │  ├spc700.rs │  │  ├decoder.rs        │  │       │
│  │  │  - Player   │  │  ├sample.rs │  │  ├encoder.rs        │  │       │
│  │  │  - Hitbox   │  │  ├wav.rs    │  │  └block.rs          │  │       │
│  │  │  - Interp   │  │  └...       │  │                     │  │       │
│  │  └─────────────┘  └─────────────┘  └─────────────────────┘  │       │
│  └─────────────────────────────────────────────────────────────┘       │
│                                │                                       │
│  ┌─────────────────────────────┼───────────────────────────────┐       │
│  │  script-core                                                │       │
│  │  ┌───────────────────────────────────────────────────────┐  │       │
│  │  │  ai_behavior/                                         │  │       │
│  │  │  ├types.rs    - AttackPattern, MoveType, etc.        │  │       │
│  │  │  ├parser.rs   - AiParser                             │  │       │
│  │  │  ├manager.rs  - AiBehaviorManager                    │  │       │
│  │  │  ├presets.rs  - AiPresets                            │  │       │
│  │  │  └...                                                 │  │       │
│  │  └───────────────────────────────────────────────────────┘  │       │
│  └─────────────────────────────────────────────────────────────┘       │
│                                │                                       │
│  ┌─────────────────────────────┼───────────────────────────────┐       │
│  │  relocation-core                                            │       │
│  │  ┌───────────────────────────────────────────────────────┐  │       │
│  │  │  bank_manager/                                        │  │       │
│  │  │  ├types.rs    - BankMap, RegionType                  │  │       │
│  │  │  ├analysis.rs - FragmentationAnalysis                │  │       │
│  │  │  └planner.rs  - DefragmentationPlanner               │  │       │
│  │  └───────────────────────────────────────────────────────┘  │       │
│  └─────────────────────────────────────────────────────────────┘       │
│                                │                                       │
│  ┌─────────────────────────────┼───────────────────────────────┐       │
│  │  Other crates: rom-core, manifest-core, patch-core,        │       │
│  │  project-core, emulator-core                               │       │
│  └─────────────────────────────┴───────────────────────────────┘       │
│                                                                         │
├────────────────────────────────┼───────────────────────────────────────┤
│        LUA PLUGIN RUNTIME      │                                       │
│                                │                                       │
│  ┌─────────────────────────────▼───────────────────────────────┐       │
│  │  Lua 5.4 VM (per plugin)                                    │       │
│  │                                                             │       │
│  │  Global: SPO table with API functions                       │       │
│  │  ├── rom_read(offset, len) -> string                        │       │
│  │  ├── rom_write(offset, data)                                │       │
│  │  ├── rom_read_byte(offset) -> number                        │       │
│  │  ├── snes_to_pc(bank, addr) -> offset                       │       │
│  │  ├── pc_to_snes(offset) -> {bank, addr}                     │       │
│  │  ├── find_pattern(pattern) -> locations                     │       │
│  │  ├── log_info/debug/warn/error(msg)                         │       │
│  │  ├── notify_info/success/warn/error(msg)                    │       │
│  │  └── call_plugin(id, command, args)                        │       │
│  │                                                             │       │
│  │  Lifecycle:                                                 │       │
│  │  ├── on_init()          - Called on load                    │       │
│  │  ├── on_shutdown()      - Called on unload                  │       │
│  │  ├── on_rom_loaded()    - ROM loaded event                  │       │
│  │  └── on_asset_modified()- Asset changed event               │       │
│  │                                                             │       │
│  │  Commands:                                                  │       │
│  │  └── COMMANDS table with callable functions                 │       │
│  │                                                             │       │
│  └─────────────────────────────────────────────────────────────┘       │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘


## Data Flow

### Plugin Execution Flow
```
┌──────────┐    invoke     ┌─────────────┐    mpsc    ┌─────────────┐
│  React   │──────────────▶│PluginManager│───────────▶│ LuaWorker   │
│   UI     │               │  (Rust)     │   channel  │   Thread    │
└──────────┘               └─────────────┘            └──────┬──────┘
                                                             │
                                                             ▼
                                                      ┌─────────────┐
                                                      │  Lua 5.4 VM │
                                                      │  - Execute  │
                                                      │  - Return   │
                                                      └──────┬──────┘
                                                             │
                                                             │ result
                                                             ▼
                                                      ┌─────────────┐
                                                      │ JSON Result │
                                                      └─────────────┘
```

### Animation Playback Flow
```
┌──────────┐    invoke     ┌─────────────┐    use      ┌─────────────┐
│  React   │──────────────▶│  Animation  │───────────▶│  Animation  │
│   UI     │               │   Player    │            │   Player    │
│          │◀──────────────│  (Rust)     │            │   (Rust)    │
└──────────┘    frame data └─────────────┘            └─────────────┘
                                                              │
                                                              ▼
                                                       ┌─────────────┐
                                                       │   Canvas    │
                                                       │   Render    │
                                                       └─────────────┘
```

### Bank Visualization Flow
```
┌──────────┐    invoke     ┌─────────────┐             ┌─────────────┐
│  React   │──────────────▶│   BankMap   │────────────▶│     Rom     │
│   UI     │               │  (Rust)     │   analyze   │   (Bytes)   │
│          │◀──────────────│             │◀────────────│             │
└──────────┘   BankMapData └─────────────┘             └─────────────┘
      │
      ▼
┌──────────┐
│  Color   │
│  Blocks  │
└──────────┘
```


## Module Dependencies

```
plugin-core
    ├── api (RomAccess via Arc<Mutex<>>)
    ├── lua_runtime (mlua crate, channels)
    ├── manager (HashMap of plugins)
    └── types (PluginInfo, EditorEvent)

asset-core
    ├── animation (CombatHitbox, AnimationPlayer)
    ├── audio/spc700 (Spc700Data, Sample)
    └── brr (BrrDecoder, BrrEncoder)

script-core
    └── ai_behavior/
        ├── types (AiBehavior, AttackPattern)
        ├── parser (AiParser)
        ├── manager (AiBehaviorManager)
        └── presets (AiPresets)

relocation-core
    └── bank_manager/
        ├── types (BankMap, RegionType)
        ├── analysis (FragmentationAnalysis)
        └── planner (DefragmentationPlanner)
```


## Thread Safety Architecture

### Plugin System Thread Model
```
Main Thread:
    - PluginManager (Arc<RwLock<>>)
    - PluginApi (Arc<>>)
    - AppState
         │
         ├── Spawns ──▶ LuaWorker Thread (per plugin)
         │                    │
         │                    ├── Runs: Lua VM (NOT Send)
         │                    ├── Uses: mpsc channels
         │                    └── Returns: results via channels
         │
         └── AppState.api (Arc<Mutex<Option<Rom>>>>)
                    ▲
                    │ Safe concurrent access
LuaWorker Thread: ──┘
    - Receives: Commands via channel
    - Executes: Lua code
    - Sends: Results back via channel
```

### Why This Architecture?
1. **Lua is !Send**: Lua state cannot be moved between threads
2. **Solution**: Dedicated worker thread per plugin with channels
3. **Benefit**: Thread-safe plugin system while maintaining Lua compatibility
4. **Trade-off**: Slight overhead of channel communication


## File Organization

```
editor/
├── apps/
│   └── desktop/
│       ├── src/
│       │   ├── components/
│       │   │   ├── PluginManager.tsx
│       │   │   ├── BankVisualization.tsx
│       │   │   └── AnimationPlayer.tsx
│       │   ├── hooks/
│       │   │   ├── usePlugins.ts
│       │   │   ├── useBankManagement.ts
│       │   │   └── useAnimation.ts
│       │   ├── types/
│       │   │   └── api.ts
│       │   └── App.tsx
│       └── src-tauri/
│           ├── src/
│           │   ├── commands/
│           │   │   ├── plugins.rs
│           │   │   ├── bank_management.rs
│           │   │   └── animation.rs
│           │   ├── app_state.rs
│           │   └── lib.rs
│           └── plugins/
│               ├── README.md
│               ├── v4_feature_demo.lua
│               ├── advanced_palette_manager.lua
│               ├── stat_calculator.lua
│               ├── rom_analyzer.lua
│               ├── auto_patch_validator.lua
│               ├── interactive_tutorial.lua
│               ├── example_hello.lua
│               ├── example_rom_stats.lua
│               └── example_batch_export.lua
│
├── crates/
│   ├── plugin-core/          # NEW
│   ├── asset-core/
│   ├── script-core/
│   ├── relocation-core/
│   └── ... (other crates)
│
└── V4_*.md                   # Documentation
```


## Key Design Decisions

### 1. Lua for Scripting
- **Why**: Lightweight, embeddable, familiar to modders
- **Implementation**: mlua crate with Luau 5.4
- **Safety**: Sandboxed, no filesystem/network access (only through API)

### 2. Thread-per-Plugin
- **Why**: Solves Lua's !Send constraint
- **Implementation**: mpsc channels for communication
- **Benefit**: True parallelism for multiple plugins

### 3. Color-Coded Bank Map
- **Why**: Visual ROM layout understanding
- **Implementation**: RegionType enum with associated colors
- **Benefit**: Immediate visual feedback on ROM usage

### 4. Canvas-Based Hitbox Editor
- **Why**: Precise visual positioning
- **Implementation**: HTML5 Canvas with mouse interaction
- **Benefit**: Intuitive hitbox editing


## Performance Considerations

| Component | Strategy |
|-----------|----------|
| Plugin Loading | Lazy initialization, on-demand Lua VM creation |
| Animation Playback | RequestAnimationFrame, efficient re-renders |
| Bank Map | Virtual scrolling for large ROMs |
| ROM Access | Arc<Mutex<>> for thread-safe shared access |
| Pattern Search | Efficient byte scanning with early exit |


## Security Model

### Plugin Sandboxing
- ✅ No direct filesystem access
- ✅ No network access
- ✅ No process spawning
- ✅ Memory-safe (Rust backend)
- ✅ API-controlled ROM access only

### ROM Protection
- Checksums verified before/after operations
- Backup creation before writes
- Validation of all offsets


## Future Extensibility

### Planned Features
1. **WASM Plugins**: For performance-critical extensions
2. **Plugin Marketplace**: Discovery and sharing
3. **Real-time Collaboration**: Multi-user editing
4. **Cloud Saves**: Project backup and sync

### Extension Points
- Plugin API versioning
- Event system for all editor actions
- Custom UI components from plugins
- Plugin-to-plugin communication


---

*Architecture designed for modularity, safety, and extensibility.*
