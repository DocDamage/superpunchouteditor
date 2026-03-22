# Sound/Music Editor Implementation - Version 3.0

## Overview

This implementation adds a comprehensive Sound/Music Editor for Super Punch-Out!! that allows browsing, previewing, and editing SPC700 audio data.

## Files Created

### Rust Backend (asset-core)

#### 1. `crates/asset-core/src/audio.rs`
SPC700 audio data structures:
- `Spc700Data` - Full SPC700 state (64KB RAM, DSP registers, samples, sequences)
- `Sample` - BRR sample with metadata (ID, name, loop points, ADSR envelope)
- `Sequence` - Music sequence with channels, tempo, notes
- `SoundEntry` / `MusicEntry` - UI list entries
- `TrackType` enum (Music, SoundEffect, Voice, Ambient)
- Known sound effects and music tracks for SPO

#### 2. `crates/asset-core/src/brr.rs`
BRR (Bit Rate Reduction) codec:
- `BrrDecoder` - Decodes BRR to 16-bit PCM
- `BrrEncoder` - Encodes PCM to BRR (basic implementation)
- Support for all 4 filter types
- Loop handling
- Block-level information extraction

#### 3. `crates/asset-core/src/spc.rs`
SPC file format support:
- `SpcFile` - Load/save SPC700 save states
- `Id666Tag` - Metadata (title, game, artist, duration)
- `SpcBuilder` - Builder pattern for creating SPC files
- SPC v0.30 format support

### Tauri Backend (audio_commands)

#### 4. `apps/desktop/src-tauri/src/audio_commands.rs`
Tauri commands:
- `get_sound_list` - List all sound effects
- `get_sound` / `preview_sound` - Get and preview individual sounds
- `get_music_list` - List all music tracks
- `get_music` / `get_music_sequence` - Get music details
- `export_sound_as_wav` / `export_sound_as_brr` - Export sounds
- `export_music_as_wav` / `export_music_as_spc` - Export music
- `import_sound_from_wav` - Import WAV and convert to BRR
- `decode_brr_to_pcm` / `encode_pcm_to_brr` - BRR conversion
- `load_spc` / `save_spc` / `create_new_spc` - SPC file management
- `scan_rom_for_audio` / `get_audio_engine_info` - ROM research helpers

### Frontend React Components

#### 5. `apps/desktop/src/components/AudioEditor.tsx`
Main audio editor with tabs:
- Sounds tab - Sound effects browser
- Music tab - Music tracks browser
- Samples tab - Advanced BRR editing (placeholder)
- SPC Files tab - SPC save state manager

#### 6. `apps/desktop/src/components/SoundList.tsx`
Sound effects list:
- Category filtering
- Search functionality
- Preview controls
- Export options (WAV, BRR)
- Import WAV button
- Sound detail panel with metadata

#### 7. `apps/desktop/src/components/MusicEditor.tsx`
Music editor:
- Context grouping (title, menu, match, boxer themes, etc.)
- Track metadata display
- Channel visualization (placeholder)
- Export options (SPC, WAV)
- Research TODO list

#### 8. CSS Files
- `AudioEditor.css` - Main editor styles
- `SoundList.css` - Sound list styles
- `MusicEditor.css` - Music editor styles

## Files Modified

### 1. `crates/asset-core/src/lib.rs`
Added module exports:
```rust
pub mod audio;
pub mod brr;
pub mod spc;
pub use audio::*;
pub use brr::*;
pub use spc::*;
```

### 2. `crates/asset-core/Cargo.toml`
Added dependency:
```toml
serde_bytes = "0.11"
```

### 3. `apps/desktop/src-tauri/src/lib.rs`
- Added `audio_commands` module
- Added `AudioState` to `AppState`
- Registered 25+ audio commands in `generate_handler!`

### 4. `apps/desktop/src/App.tsx`
- Added `AudioEditor` import
- Added 'audio' tab to state type
- Added Audio tab button
- Added Audio editor rendering

## Research TODOs (Documented in Code)

### SPC700 Data Structures
- [ ] Confirm exact SPC700 RAM layout for Super Punch-Out!!
- [ ] Identify DSP register initialization values
- [ ] Map actual sound effect IDs to in-game sounds
- [ ] Determine music sequence format
- [ ] Find sample table location in ROM
- [ ] Document loop point handling for BRR samples

### BRR Decoder/Encoder
- [ ] Verify loop handling for all edge cases
- [ ] Optimize encoding quality with proper filter selection
- [ ] Test with actual SPO samples

### SPC File Format
- [ ] Verify exact SPC file locations in SPO ROM
- [ ] Support extended SPC format features (v0.31+)
- [ ] Handle multiple SPC banks in ROM

### ROM Audio Extraction
- [ ] Identify SPC engine location in ROM
- [ ] Map sample table addresses
- [ ] Locate sequence data
- [ ] Document instrument/sample mapping
- [ ] Implement full ROM audio extraction

## Known Limitations

1. **Audio Playback**: Preview is currently a stub. Actual implementation requires:
   - OS audio API integration (rodio or similar)
   - Or temp file writing + system player
   - Or Web Audio API streaming from frontend

2. **BRR Encoding**: Basic implementation only. High-quality encoding requires:
   - Proper filter/range optimization algorithms
   - Loop point-aware encoding
   - Quality level selection

3. **ROM Integration**: Audio data locations unknown:
   - Sample table address not researched
   - Sequence format not reverse-engineered
   - Music ID mapping incomplete

4. **Music Editing**: Sequence editor is placeholder. Full implementation requires:
   - Sequence format reverse engineering
   - Piano roll UI implementation
   - Note/event editing
   - Channel management

## UI Preview

The Audio Editor provides a tabbed interface:

```
┌─────────────────────────────────────────────────────────────┐
│ Sound & Music Editor                               [⏹ Stop] │
├─────────────────────────────────────────────────────────────┤
│ [Sounds] [Music] [Samples] [SPC Files]                     │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│ Sound Effects                    Preview: [▶] [⏹]          │
│ ─────────────                                                             │
│ 🔊 Punch Hit 1           ID: $01  Size: 4KB  [Edit] [Export]│
│ 🔊 Punch Hit 2           ID: $02  Size: 4KB  [Edit] [Export]│
│ 🔊 Block Sound           ID: $03  Size: 2KB  [Edit] [Export]│
│ ...                                                         │
│                                                             │
│ Selected: Punch Hit 1                                       │
│ ─────────────────────                                       │
│ Format: BRR (SNES native)                                   │
│ Sample Rate: 32000 Hz                                       │
│ Duration: 0.5s                                              │
│                                                             │
│ [Export as WAV]  [Replace from WAV]  [Preview]             │
└─────────────────────────────────────────────────────────────┘
```

## Next Steps for Full Implementation

1. **Research ROM Audio Locations**:
   - Analyze SPO ROM for SPC700 engine
   - Find sample table
   - Map music sequences

2. **Implement Audio Playback**:
   - Add rodio or cpal for audio output
   - Implement real-time BRR decoding
   - Add playback controls

3. **Complete BRR Encoder**:
   - Implement optimal filter selection
   - Add loop-aware encoding
   - Quality presets

4. **Build Sequence Editor**:
   - Reverse engineer sequence format
   - Implement piano roll UI
   - Add note editing
   - Channel solo/mute

5. **ROM Integration**:
   - Extract audio data from ROM
   - Inject modified samples
   - Update pointers
   - Test in emulator

## Technical Notes

- **SPC700**: SNES audio coprocessor with 64KB RAM, 8 channels
- **BRR Format**: 9-byte blocks, 16 samples, ADPCM compression
- **SPC Files**: Save states including full SPC700 memory
- **SNES DSP**: 8 channels, 4 filter types, ADSR envelope

## Version

This is Version 3.0 of the Sound/Music Editor implementation, providing the foundational architecture and UI with placeholders for audio processing functionality pending ROM research.
