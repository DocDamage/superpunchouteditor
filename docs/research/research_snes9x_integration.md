# Research Report: Embedding Snes9x into Tauri/Rust Application

## Executive Summary

This report investigates methods for embedding the Snes9x SNES emulator into a Tauri/Rust application for the Super Punch-Out!! editor. Two main approaches are evaluated: **using Snes9x as a libretro core via FFI** and **using a pure Rust SNES emulator**.

**Recommendation:** Use **Snes9x as a libretro core** for maximum compatibility and accuracy, or consider **rsnes** as a pure Rust alternative if simpler integration is prioritized over perfect accuracy.

---

## 1. Build System Analysis

### 1.1 Snes9x Build System

**Repository:** https://github.com/snes9xgit/snes9x

**Build Options:**
| Platform | Build System | Notes |
|----------|--------------|-------|
| Windows | Visual Studio (.sln) | VS2017+ recommended |
| Linux/GTK | CMake or Meson | SDL2 required |
| macOS | Xcode | Native build |
| **libretro** | **Makefile** | **Recommended for embedding** |

**Key Dependencies:**
- SDL 2.0 (for GTK port)
- zlib (optional, for compressed ROMs)
- libpng (optional, for screenshots)
- OpenGL (optional, for hardware rendering)

### 1.2 Building as Static Library

Snes9x can be built as a **libretro core** (`.dll`/`.so`/`.dylib`), which is a standardized dynamic library interface. This is the cleanest approach for embedding:

```bash
# Clone with submodules
git clone --recursive https://github.com/snes9xgit/snes9x.git
cd snes9x/libretro

# Build the libretro core
make -j$(nproc)
# Output: snes9x_libretro.so (Linux) / snes9x_libretro.dll (Windows) / snes9x_libretro.dylib (macOS)
```

**Static Library Approach:**
While Snes9x isn't designed to build as a static `.a` or `.lib` directly, the libretro core can be dynamically loaded at runtime using Rust's `libloading` crate.

### 1.3 Disabling GUI Components

The libretro core **has no GUI** - it's designed specifically for embedding. It exposes:
- Core emulation (CPU, PPU, APU, memory)
- Input handling
- Save/load states
- No windowing, rendering, or audio output code

---

## 2. C++ Interface Analysis

### 2.1 Main Entry Points

For the libretro core, the interface is defined in `libretro.h`:

**Core Functions:**
```c
// Required callbacks (set by frontend)
retro_set_environment     // Configuration
retro_set_video_refresh   // Video output callback
retro_set_audio_sample    // Audio output callback
retro_set_input_poll      // Input poll callback
retro_set_input_state     // Input state callback

// Core lifecycle
retro_init();             // Initialize emulator
retro_deinit();           // Cleanup
retro_load_game();        // Load ROM
retro_unload_game();      // Unload ROM
retro_run();              // Run one frame
retro_reset();            // Reset emulator

// Save states
retro_serialize_size();   // Get state size
retro_serialize();        // Save state
retro_unserialize();      // Load state

// Memory access (for cheats/debugging)
retro_get_memory_data();  // Get RAM/VRAM pointer
retro_get_memory_size();  // Get memory region size
```

### 2.2 Key Functions to Bind

| Function | Purpose |
|----------|---------|
| `retro_init` | Initialize emulator core |
| `retro_load_game` | Load ROM file |
| `retro_run` | Execute one frame (~16.6ms) |
| `retro_video_refresh_t` callback | Receive video buffer |
| `retro_audio_sample_batch_t` callback | Receive audio samples |
| `retro_input_state_t` callback | Provide controller input |
| `retro_serialize/unserialize` | Save/load states |

### 2.3 Existing libretro Interface

Snes9x has a **complete libretro implementation** at `libretro/libretro.cpp`:
- Video: Supports RGB565, XRGB1555, ARGB8888 pixel formats
- Audio: 16-bit stereo PCM, ~32040 Hz (configurable)
- Input: SNES controller with all buttons

---

## 3. Rust FFI Feasibility

### 3.1 Binding Approaches

**Option A: `libretro-sys` crate + `libloading`** (Recommended)
```rust
// Use existing libretro bindings
use libretro_sys::CoreAPI;
use libloading::{Library, Symbol};

// Load the dynamic library
unsafe {
    let lib = Library::new("snes9x_libretro.dll")?;
    let core: Symbol<fn()> = lib.get(b"retro_init")?;
}
```

**Option B: `bindgen` for C headers**
```rust
// build.rs
use std::env;
use std::path::PathBuf;

fn main() {
    let bindings = bindgen::Builder::default()
        .header("libretro.h")
        .generate()
        .expect("Unable to generate bindings");
    
    bindings.write_to_file(PathBuf::from(env::var("OUT_DIR").unwrap()).join("bindings.rs"))
        .expect("Couldn't write bindings");
}
```

**Option C: `cxx` crate for C++**
Not recommended for Snes9x - the libretro interface is pure C, making `cxx` unnecessary.

### 3.2 Recommended FFI Stack

```toml
[dependencies]
libretro-sys = "0.1"
libloading = "0.8"

[build-dependencies]
bindgen = "0.70"  # Only if custom bindings needed
```

### 3.3 Pure Rust Alternative: `rsnes`

**Repository:** https://github.com/nat-rix/rsnes

**Status:** Work in progress - many games playable, some graphics/sound issues

**Pros:**
- Pure Rust, no FFI complexity
- Better integration with Rust ecosystem
- Easier to modify for specific needs

**Cons:**
- Less accurate than Snes9x
- May not run all Super Punch-Out!! features perfectly
- Limited documentation

**When to use:** If FFI complexity outweighs accuracy requirements for your use case.

---

## 4. Video Output Options

### 4.1 Snes9x Video Buffer

**From `gfx.h`:**
```cpp
struct SGFX {
    uint16 *Screen;           // Main screen buffer (16-bit)
    uint32 PPL;               // Pixels per line (pitch)
    uint32 ScreenSize;        // Total buffer size
    // ...
};
```

**Pixel Formats Supported (via libretro):**
| Format | Bits | Description |
|--------|------|-------------|
| RETRO_PIXEL_FORMAT_RGB565 | 16 | Most common |
| RETRO_PIXEL_FORMAT_0RGB1555 | 16 | Legacy |
| RETRO_PIXEL_FORMAT_XRGB8888 | 32 | Best quality |

**From `pixform.h`:**
- Default is **RGB565** (5-6-5 bits for R-G-B)
- Can request **XRGB8888** via libretro environment callback

### 4.2 Native Resolution

- **Base:** 256×224 pixels (NTSC SNES)
- **Max:** 512×478 (with interlace modes)
- **Aspect Ratio:** 4:3 (or 8:7 for pixel-perfect)

### 4.3 Rendering in Tauri/React

**Option A: Canvas 2D with `putImageData`** (Recommended for simplicity)
```typescript
// React component
const canvasRef = useRef<HTMLCanvasElement>(null);

useEffect(() => {
  const canvas = canvasRef.current;
  const ctx = canvas.getContext('2d');
  
  // Receive frame data from Rust
  listen('frame', (event) => {
    const imageData = new ImageData(
      new Uint8ClampedArray(event.payload),
      256, 224
    );
    ctx.putImageData(imageData, 0, 0);
  });
}, []);
```

**Option B: WebGL Texture** (Better performance for scaling/filters)
```typescript
// Upload frame as WebGL texture
const texture = gl.createTexture();
gl.bindTexture(gl.TEXTURE_2D, texture);
gl.texImage2D(
  gl.TEXTURE_2D, 0, gl.RGB565,
  256, 224, 0,
  gl.RGB, gl.UNSIGNED_SHORT_5_6_5,
  frameBuffer
);
```

**Performance Comparison:**
| Method | FPS (256×224) | CPU Usage | Notes |
|--------|---------------|-----------|-------|
| Canvas putImageData | 60+ | Low | Simplest implementation |
| WebGL Texture | 60+ | Very Low | Best for shaders/filters |
| WebSocket transfer | <30 | High | Too slow, don't use |

### 4.4 Recommended Video Architecture

```
┌─────────────────────────────────────┐
│  Tauri WebView (React frontend)     │
│  ┌─────────────────────────────┐    │
│  │  Canvas/WebGL display       │    │
│  │  (256×224 or scaled)        │    │
│  └─────────────────────────────┘    │
└─────────────────────────────────────┘
                   ↑
                   │ IPC (binary frame data)
                   ↓
┌─────────────────────────────────────┐
│  Rust Backend                       │
│  ┌─────────────────────────────┐    │
│  │  Frame buffer conversion    │    │
│  │  (RGB565 → RGBA8888)        │    │
│  └─────────────────────────────┘    │
│                   ↑                 │
│  ┌────────────────┴────────────────┐│
│  │  Snes9x libretro core (.dll)    ││
│  │  - retro_run()                  ││
│  │  - video_refresh callback       ││
│  └─────────────────────────────────┘│
└─────────────────────────────────────┘
```

---

## 5. Audio Output

### 5.1 Snes9x Audio Format

**From `apu.h` and libretro interface:**
- **Format:** 16-bit signed PCM (stereo interleaved)
- **Sample Rate:** 32040 Hz (default, NTSC accurate)
- **Channels:** 2 (stereo)
- **Buffer:** ~735 samples per frame (at 60fps)

**Audio Callback Signature:**
```c
typedef size_t (*retro_audio_sample_batch_t)(
    const int16_t *data,  // Interleaved stereo samples
    size_t frames         // Number of stereo frames
);
```

### 5.2 Audio Playback Options

**Option A: Rust `cpal` crate** (Recommended)
```rust
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

fn setup_audio() {
    let host = cpal::default_host();
    let device = host.default_output_device().expect("no output device");
    let config = device.default_output_config().unwrap();
    
    let stream = device.build_output_stream(
        &config.into(),
        move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
            // Fill buffer from Snes9x audio samples
            fill_audio_buffer(data);
        },
        |err| eprintln!("audio error: {}", err),
        None,
    ).unwrap();
    
    stream.play().unwrap();
}
```

**Option B: Web Audio API** (via Tauri)
```typescript
const audioContext = new AudioContext({ sampleRate: 32040 });

// Receive audio data from Rust
listen('audio', (event) => {
  const samples = new Int16Array(event.payload);
  const buffer = audioContext.createBuffer(2, samples.length / 2, 32040);
  
  // Convert Int16 to Float32
  const left = buffer.getChannelData(0);
  const right = buffer.getChannelData(1);
  for (let i = 0; i < samples.length / 2; i++) {
    left[i] = samples[i * 2] / 32768;
    right[i] = samples[i * 2 + 1] / 32768;
  }
  
  const source = audioContext.createBufferSource();
  source.buffer = buffer;
  source.connect(audioContext.destination);
  source.start();
});
```

**Recommendation:** Use `cpal` in Rust for lower latency and better synchronization with video.

---

## 6. Input Handling

### 6.1 Snes9x Input Interface

**libretro input callback:**
```c
typedef int16_t (*retro_input_state_t)(
    unsigned port,      // Controller port (0 = player 1)
    unsigned device,    // RETRO_DEVICE_JOYPAD
    unsigned index,     // Analog index (unused for SNES)
    unsigned id         // Button ID (see below)
);
```

**SNES Button Mappings (from libretro.h):**
```c
RETRO_DEVICE_ID_JOYPAD_B      // SNES B
RETRO_DEVICE_ID_JOYPAD_Y      // SNES Y
RETRO_DEVICE_ID_JOYPAD_SELECT // Select
RETRO_DEVICE_ID_JOYPAD_START  // Start
RETRO_DEVICE_ID_JOYPAD_UP     // D-Pad Up
RETRO_DEVICE_ID_JOYPAD_DOWN   // D-Pad Down
RETRO_DEVICE_ID_JOYPAD_LEFT   // D-Pad Left
RETRO_DEVICE_ID_JOYPAD_RIGHT  // D-Pad Right
RETRO_DEVICE_ID_JOYPAD_A      // SNES A
RETRO_DEVICE_ID_JOYPAD_X      // SNES X
RETRO_DEVICE_ID_JOYPAD_L      // L Button
RETRO_DEVICE_ID_JOYPAD_R      // R Button
```

### 6.2 Keyboard to SNES Mapping

**Recommended Mapping for Super Punch-Out!!:**
| Keyboard | SNES Button | Function |
|----------|-------------|----------|
| Arrow Keys | D-Pad | Dodge left/right/duck |
| Z | B | Left punch |
| X | A | Right punch |
| A | Y | Left hook |
| S | X | Right hook |
| Enter | Start | Pause/menu |
| Shift | Select | - |
| Q | L | Special left |
| W | R | Special right |

### 6.3 Implementation

```rust
// Rust side - track input state
static INPUT_STATE: AtomicU16 = AtomicU16::new(0);

// Called by Tauri command when key event occurs
#[tauri::command]
fn set_button(button: u8, pressed: bool) {
    let mask = 1u16 << button;
    if pressed {
        INPUT_STATE.fetch_or(mask, Ordering::Relaxed);
    } else {
        INPUT_STATE.fetch_and(!mask, Ordering::Relaxed);
    }
}

// Called by libretro core
extern "C" fn input_state(port: u32, device: u32, index: u32, id: u32) -> i16 {
    if port != 0 || device != RETRO_DEVICE_JOYPAD {
        return 0;
    }
    let state = INPUT_STATE.load(Ordering::Relaxed);
    ((state >> id) & 1) as i16
}
```

### 6.4 Gamepad Support

Use the Gamepad API in the frontend:
```typescript
window.addEventListener('gamepadconnected', (e) => {
  const gamepad = e.gamepad;
  pollGamepad(gamepad);
});

function pollGamepad(gamepad: Gamepad) {
  // Map gamepad buttons to SNES
  const buttons = {
    b: gamepad.buttons[0].pressed,      // A on Xbox, X on PlayStation
    a: gamepad.buttons[1].pressed,      // B on Xbox, O on PlayStation
    // ... etc
  };
  invoke('set_gamepad_state', buttons);
}
```

---

## 7. Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                    SUPER PUNCH-OUT!! EDITOR                      │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│  FRONTEND (Tauri WebView - React/TypeScript)                    │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │
│  │  ROM Editor │  │  Emulator   │  │  Memory/State Inspector │  │
│  │  (Hex edit) │  │  Controls   │  │  (for debugging)        │  │
│  └─────────────┘  └─────────────┘  └─────────────────────────┘  │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │                    Canvas Display                          │ │
│  │              (256×224 scaled to viewport)                  │ │
│  └────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
                              ↑↓ IPC (Tauri Commands/Events)
┌─────────────────────────────────────────────────────────────────┐
│  BACKEND (Rust)                                                 │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │              Emulator Manager (emulator.rs)                │ │
│  │  - Load/unload ROM                                         │ │
│  │  - Frame pacing (60 FPS)                                   │ │
│  │  - State management (save/load)                            │ │
│  │  - Memory editing hooks                                    │ │
│  └────────────────────────────────────────────────────────────┘ │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐  │
│  │   Video Bridge  │  │   Audio Bridge  │  │   Input Bridge  │  │
│  │  (RGB565→RGBA)  │  │  (16-bit PCM)   │  │ (Keyboard/Game  │  │
│  │                 │  │                 │  │  → SNES mapping)│  │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘  │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │         Snes9x libretro Core (snes9x_libretro.dll)         │ │
│  │  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐        │ │
│  │  │  65816  │  │   PPU   │  │   APU   │  │  SMP    │        │ │
│  │  │  CPU    │  │(graphics│  │ (audio) │  │(sound)  │        │ │
│  │  └─────────┘  └─────────┘  └─────────┘  └─────────┘        │ │
│  │  ┌─────────┐  ┌─────────┐  ┌─────────┐                     │ │
│  │  │  Memory │  │ Cartridge│  │  DMA    │                     │ │
│  │  │  (RAM)  │  │  (ROM)   │  │ Controller│                   │ │
│  │  └─────────┘  └─────────┘  └─────────┘                     │ │
│  └────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

---

## 8. Potential Challenges

### 8.1 Technical Challenges

| Challenge | Severity | Solution |
|-----------|----------|----------|
| **Thread safety** | High | Run emulator in dedicated thread; use channels for communication |
| **Frame pacing** | Medium | Use `spin_sleep` or `winit` event loop for 60Hz timing |
| **Audio sync** | Medium | Use `cpal` with callback-based audio; maintain small buffer |
| **Cross-platform builds** | Medium | Build libretro core for each target platform in CI |
| **Memory editing while running** | Medium | Pause emulation, edit memory, resume; or use Snes9x's cheat interface |

### 8.2 Build Challenges

| Challenge | Solution |
|-----------|----------|
| Snes9x compilation | Use pre-built libretro cores from libretro buildbot |
| Windows dependencies | Static link where possible; bundle DLLs |
| macOS code signing | Sign the libretro dylib with your cert |

### 8.3 Performance Considerations

- **Video:** At 256×224 @ 60fps, raw data is ~8.5 MB/s - easily handled by modern hardware
- **Audio:** ~128 KB/s stereo 16-bit @ 32kHz - negligible
- **Overall:** Snes9x runs at <5% CPU on modern hardware; Tauri overhead is minimal

---

## 9. Implementation Roadmap

### Phase 1: Proof of Concept (1-2 weeks)
1. Build Snes9x libretro core
2. Create minimal Rust libretro frontend
3. Load ROM and display frames via simple window

### Phase 2: Tauri Integration (1-2 weeks)
1. Add Tauri with canvas display
2. Implement input handling (keyboard)
3. Add audio via `cpal`

### Phase 3: Editor Features (2-3 weeks)
1. Memory inspection/editing
2. Save state management
3. Breakpoint support (if needed)

### Phase 4: Polish (1 week)
1. Gamepad support
2. Video filters/shaders
3. Performance optimization

---

## 10. Dependencies Summary

### Required Crates
```toml
[dependencies]
# Core
tauri = { version = "2", features = [] }
serde = { version = "1", features = ["derive"] }

# FFI
libloading = "0.8"
libretro-sys = "0.1"

# Audio
cpal = "0.15"

# Utilities
spin_sleep = "1"  # For frame pacing
crossbeam-channel = "0.5"  # Thread communication
```

### Optional Crates
```toml
# For rsnes alternative
# rsnes = { git = "https://github.com/nat-rix/rsnes" }

# For better error handling
anyhow = "1"
thiserror = "1"

# For logging
tracing = "0.1"
```

---

## 11. References

### Snes9x Resources
- Repository: https://github.com/snes9xgit/snes9x
- libretro header: https://github.com/snes9xgit/snes9x/blob/master/libretro/libretro.h
- Porting guide: https://github.com/snes9xgit/snes9x/blob/master/docs/porting.html

### libretro Resources
- API documentation: https://www.libretro.com/index.php/api/
- Rust libretro tutorial: https://www.retroreversing.com/CreateALibRetroFrontEndInRust
- libretro-sys crate: https://crates.io/crates/libretro-sys

### Alternative: Pure Rust
- rsnes: https://github.com/nat-rix/rsnes

---

## 12. Conclusion

Embedding Snes9x into a Tauri application is **feasible and recommended** for a Super Punch-Out!! editor. The libretro interface provides a clean, well-documented API that abstracts away the complexity of the emulator core.

**Key takeaways:**
1. Use Snes9x as a libretro core for best compatibility
2. Build a Rust libretro frontend using `libloading`
3. Use `cpal` for audio, Canvas/WebGL for video
4. Consider `rsnes` only if FFI complexity is a blocker

**Estimated effort:** 4-6 weeks for a fully functional integrated emulator.
