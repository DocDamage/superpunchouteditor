# AI Behavior Editor - Super Punch-Out!! Editor v3.0

## Overview

The AI Behavior Editor provides a visual interface for editing boxer AI attack patterns, defense behaviors, and difficulty scaling in Super Punch-Out!!.

⚠️ **WARNING**: This feature uses placeholder data structures. The actual ROM addresses and byte formats require further research (see TODOs below).

## Features

### 1. Attack Patterns Editor
- Visual node-based pattern creation
- Move sequence editing with timing controls
- Frame-accurate windup/active/recovery timing
- Hitbox configuration (position, size, height zone)
- Condition-based availability (round, difficulty, health)
- Frequency and weight controls for pattern selection

### 2. Defense Behaviors
- Configuration of dodge, block, duck, and counter behaviors
- Success rate and frequency tuning
- Counter-attack linking to patterns
- Recovery frame settings

### 3. Difficulty Curve Editor
- Round-by-round stat adjustment
- Visual difficulty progression chart
- Quick presets (Steady, Difficulty Spike, Reverse)
- Base aggression/defense/speed multipliers

### 4. AI Triggers
- Condition-action rule system
- Health-based triggers
- Round-specific behaviors
- Random chance events
- Combo counters and player state detection

### 5. Simulation Preview
- 100-fight simulation for balance testing
- Pattern usage statistics
- Difficulty rating calculation
- Damage estimates and fight duration predictions
- Warning system for potential issues

## Architecture

### Backend (Rust)

#### File: `crates/script-core/src/ai_behavior.rs`

Core data structures:

```rust
pub struct AiBehavior {
    pub fighter_id: usize,
    pub fighter_name: String,
    pub attack_patterns: Vec<AttackPattern>,
    pub defense_behaviors: Vec<DefenseBehavior>,
    pub difficulty_curve: DifficultyCurve,
    pub triggers: Vec<AiTrigger>,
}

pub struct AttackPattern {
    pub id: String,
    pub name: String,
    pub sequence: Vec<AttackMove>,
    pub frequency: u8,        // 0-255 chance per frame
    pub conditions: Vec<Condition>,
    pub difficulty_min: u8,
    pub difficulty_max: u8,
}
```

#### Tauri Commands (`apps/desktop/src-tauri/src/lib.rs`)

| Command | Description |
|---------|-------------|
| `get_ai_behavior` | Load AI data for a fighter |
| `update_attack_pattern` | Save pattern changes |
| `add_attack_pattern` | Create new pattern |
| `remove_attack_pattern` | Delete pattern |
| `update_difficulty_curve` | Save difficulty settings |
| `update_defense_behaviors` | Save defense config |
| `update_ai_triggers` | Save trigger rules |
| `test_ai_behavior` | Run simulation |
| `validate_ai_behavior` | Check for issues |
| `get_ai_presets` | Load templates |
| `get_move_types` | Available moves |
| `get_defense_types` | Available defenses |
| `get_condition_types` | Available conditions |

### Frontend (React + TypeScript)

#### Components

| Component | Purpose |
|-----------|---------|
| `AIEditor.tsx` | Main editor container, tab management |
| `AttackPatternEditor.tsx` | Pattern creation/editing |
| `DifficultyCurveEditor.tsx` | Round difficulty settings |
| `SimulationPreview.tsx` | Simulation results display |

#### Type Definitions

File: `apps/desktop/src/types/aiBehavior.ts`

## Research TODOs

### Critical - Blocking ROM Integration

1. **AI Table Location**
   - [ ] Confirm bank $09 contains AI data
   - [ ] Find base address for AI scripts
   - [ ] Document per-fighter offset calculation
   - [ ] Identify shared AI vs unique AI

2. **Attack Pattern Format**
   - [ ] Byte structure for move sequences
   - [ ] Frame timing encoding (4-bit? 8-bit?)
   - [ ] Damage/stun value storage
   - [ ] Hitbox data format
   - [ ] Animation pose references

3. **Defense Behavior Format**
   - [ ] Defense type encoding
   - [ ] Success rate calculation formula
   - [ ] Counter-attack linking mechanism

4. **Difficulty Scaling**
   - [ ] Round modifier table location
   - [ ] Stat multiplier encoding
   - [ ] Pattern availability flags

5. **Trigger System**
   - [ ] Condition bytecode format
   - [ ] Action pointer structure
   - [ ] Priority/cooldown storage

### Secondary

- [ ] Sound effect ID mapping
- [ ] Special move uniqueness per fighter
- [ ] AI vs player-specific behavior flags

## Known Addresses (Placeholder)

These are educated guesses based on the fighter header table structure:

| Fighter | Estimated AI Offset | Notes |
|---------|---------------------|-------|
| Gabby Jay | $09:8800 | Entry-level patterns |
| Bear Hugger | $09:8A00 | Heavy, slow attacks |
| Piston Hurricane | $09:8C00 | Fast combos |
| Bald Bull | $09:8E00 | Bull charge special |
| Bob Charlie | $09:9000 | Dance patterns |
| Dragon Chan | $09:9200 | Kick moves |
| Masked Muscle | $09:9400 | Spit attack |
| Mr. Sandman | $09:9600 | Dreamland express |
| Aran Ryan | $09:9800 | Headbutt |
| Heike Kagero | $09:9A00 | Hair attacks |
| Mad Clown | $09:9C00 | Ball juggling |
| Super Macho Man | $09:9E00 | Spin punch |
| Narcis Prince | $09:A000 | Mirror check |
| Hoy Quarlow | $09:A200 | Stick attacks |
| Rick Bruiser | $09:A400 | Charge attacks |
| Nick Bruiser | $09:A600 | Counter focus |

## Usage Guide

### Creating a New Attack Pattern

1. Select the "Attack Patterns" tab
2. Click "+ Add Pattern"
3. Enter a descriptive name
4. Click on the pattern to edit
5. Use "+ Add Move" to build the sequence
6. Configure timing, damage, and hitbox for each move
7. Set availability conditions (rounds, difficulty)
8. Adjust frequency and weight
9. Click "Save Pattern"

### Balancing Difficulty

1. Select the "Difficulty" tab
2. Adjust base stats for the fighter
3. Modify per-round multipliers
4. Use the visual chart to see progression
5. Try quick presets as starting points
6. Run simulation to verify

### Testing Changes

1. Click "🎮 Test AI" button
2. Wait for simulation to complete (100 fights)
3. Review difficulty rating and statistics
4. Check warnings for potential issues
5. Adjust and re-test as needed

## File Structure

```
crates/script-core/src/
├── ai_behavior.rs          # Core Rust module
└── lib.rs                  # Module export

apps/desktop/src/
├── components/
│   ├── AIEditor.tsx        # Main editor
│   ├── AIEditor.css        # Styles
│   ├── AttackPatternEditor.tsx
│   ├── AttackPatternEditor.css
│   ├── DifficultyCurveEditor.tsx
│   ├── DifficultyCurveEditor.css
│   ├── SimulationPreview.tsx
│   └── SimulationPreview.css
├── types/
│   └── aiBehavior.ts       # TypeScript types
└── App.tsx                 # Tab integration

apps/desktop/src-tauri/src/
└── lib.rs                  # Tauri commands

docs/
└── AI_BEHAVIOR_EDITOR.md   # This file
```

## Future Enhancements

1. **ROM Integration**: Once addresses are confirmed, implement real parsing/serialization
2. **Visual Hitbox Editor**: Drag-to-position hitbox overlay on fighter sprites
3. **Animation Preview**: Play pattern sequences with actual game graphics
4. **AI Comparison**: Side-by-side compare two fighters' AI configurations
5. **Import/Export**: Share AI configurations as JSON files
6. **Training Mode Integration**: Test AI directly in emulator from editor

## Contributing

To help with ROM research:

1. Use an SNES debugger (bsnes-plus, Mesen-S)
2. Set breakpoints at suspected AI addresses
3. Document byte patterns for different behaviors
4. Compare across fighters to find shared vs unique code
5. Submit findings via GitHub issues with:
   - ROM address (SNES format: $BB:AAAA)
   - Byte dump (hex)
   - Observed behavior in-game
   - Fighter being tested

## References

- [Super Punch-Out!! Disassembly](https://github.com/etchkode/super-punch-out-disasm)
- [SNES CPU Memory Map](https://en.wikibooks.org/wiki/Super_NES_Programming/CPU_Memory_Map)
- [SMC ROM Format](https://snes.nesdev.org/wiki/ROM_header)
