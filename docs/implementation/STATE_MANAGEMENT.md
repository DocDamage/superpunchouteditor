# State Management

This document describes the state management architecture for the Super Punch-Out!! Editor.

## Overview

The editor uses a dual-layer state management approach:

1. **Rust Backend (AppState)**: Thread-safe state using `parking_lot::Mutex`
2. **TypeScript Frontend (Zustand)**: Split into domain-specific stores

## Rust Backend (AppState)

### Thread-Safe State

All fields in `AppState` are wrapped in `parking_lot::Mutex` for safe concurrent access:

```rust
pub struct AppState {
    pub rom: Mutex<Option<Rom>>,
    pub manifest: Mutex<Manifest>,
    pub pending_writes: Mutex<HashMap<String, Vec<u8>>>,
    pub current_project: Mutex<Option<Project>>,
    pub rom_path: Mutex<Option<String>>,
    pub edit_history: Mutex<EditHistory>,
    pub emulator_settings: Mutex<EmulatorSettings>,
    pub frame_tag_manager: Mutex<FrameTagManager>,
    pub external_tools: Mutex<ToolHooksConfig>,
    pub audio_state: Mutex<AudioState>,
    pub embedded_emulator: Mutex<EmbeddedEmulatorState>,
}
```

### Why parking_lot?

We use `parking_lot::Mutex` instead of `std::sync::Mutex` because:

- **No poisoning**: Locks never become "poisoned" after a panic
- **Better performance**: Faster lock/unlock operations
- **Simpler API**: No need for `.unwrap()` or error handling on lock

### Access Pattern

```rust
// Simple lock - never fails
let rom = state.rom.lock();
// Use rom...
drop(rom); // Explicit drop for clarity (optional)
```

### Converting from std::sync::Mutex

Before (with std::sync::Mutex):
```rust
let rom = state.rom.lock().map_err(|_| "Lock poisoned")?;
```

After (with parking_lot::Mutex):
```rust
let rom = state.rom.lock();
```

## TypeScript Frontend (Zustand)

### Store Architecture

State is split into domain-specific stores:

| Store | Purpose |
|-------|---------|
| `romStore` | ROM loading, validation, pending writes |
| `boxerStore` | Boxer data, selection, palettes |
| `projectStore` | Project save/load, recent projects |
| `editStore` | Undo/redo history |
| `emulatorStore` | Emulator settings and embedded emulator |
| `comparisonStore` | ROM comparison and diff viewing |
| `toolsStore` | External tools configuration |
| `uiStore` | UI state, tabs, modals, theme |

### Usage

#### Import Individual Stores (Recommended)

```typescript
import { useRomStore } from '../store/romStore';
import { useBoxerStore } from '../store/boxerStore';

function MyComponent() {
  const romSha1 = useRomStore(selectRomSha1);
  const boxers = useBoxerStore(selectBoxers);
  
  return (
    <div>
      <p>ROM: {romSha1}</p>
      <BoxerList boxers={boxers} />
    </div>
  );
}
```

#### Using Selectors for Performance

Use selectors to prevent unnecessary re-renders:

```typescript
// Good: Component only re-renders when romSha1 changes
const romSha1 = useRomStore(selectRomSha1);

// Bad: Component re-renders on any store change
const { romSha1 } = useRomStore();
```

### Computed Values

Instead of storing derived values in state, use computed getters:

```typescript
export const useEditStore = create<EditStore>((set, get) => ({
  undoStack: [],
  redoStack: [],
  
  // Computed getter
  get canUndo() {
    return get().undoStack.length > 0;
  },
  
  get canRedo() {
    return get().redoStack.length > 0;
  },
}));
```

### State Persistence

Stores can persist state to localStorage using the `persist` middleware:

```typescript
export const useUiStore = create<UiStore>()(
  persist(
    (set, get) => ({
      // ... store implementation
    }),
    {
      name: 'spo-ui-storage',
      partialize: (state) => ({
        // Only persist these fields
        theme: state.theme,
        layout: state.layout,
      }),
    }
  )
);
```

### State Validation

Validate state when loading from external sources:

```typescript
import { validateProjectFile, safeValidateProject } from '../store/validation';

// Strict validation with error details
const result = validateProjectFile(storedData);
if (result.valid) {
  useProjectStore.setState({ currentProject: result.data });
} else {
  console.error('Invalid project:', result.errors);
}

// Safe validation with fallback
const project = safeValidateProject(storedData);
if (project) {
  useProjectStore.setState({ currentProject: project });
}
```

## State Flow

The typical data flow for a user action:

```
1. User action → Zustand action
   User clicks "Save Palette"
   ↓
2. Frontend validation
   Validate color values
   ↓
3. Tauri command invoked
   invoke('update_palette', { ... })
   ↓
4. Rust updates AppState
   state.pending_writes.lock().insert(...)
   ↓
5. Results returned to frontend
   Return success/error
   ↓
6. Zustand updates with response
   set({ pendingWrites: newSet })
   ↓
7. UI re-renders
   Components using pendingWrites update
```

## Migration Guide

### From Old useStore to New Stores

Before:
```typescript
import { useStore } from '../store/useStore';

function Component() {
  const { romSha1, boxers, loadBoxers } = useStore();
  // ...
}
```

After:
```typescript
import { useRomStore, useBoxerStore } from '../store';

function Component() {
  const romSha1 = useRomStore(selectRomSha1);
  const boxers = useBoxerStore(selectBoxers);
  // ...
}
```

## Best Practices

1. **Use individual stores** for new code instead of the monolithic `useStore`
2. **Use selectors** to prevent unnecessary re-renders
3. **Prefer computed getters** over storing derived values
4. **Use persist middleware** for state that should survive page reloads
5. **Validate state** when loading from external sources
6. **Keep state normalized** to avoid duplication
7. **Use type-safe actions** instead of directly setting state

## Error Handling

### Frontend

```typescript
const loadBoxers = async () => {
  set({ isLoading: true, error: null });
  try {
    const boxers = await invoke<BoxerRecord[]>('get_boxers');
    set({ boxers, isLoading: false });
  } catch (e) {
    console.error('Failed to load boxers:', e);
    set({ error: (e as Error).message, isLoading: false });
  }
};
```

### Backend

```rust
#[tauri::command]
fn get_boxers(state: State<AppState>) -> Result<Vec<BoxerRecord>, String> {
    let manifest = state.manifest.lock();
    Ok(manifest.fighters.values().cloned().collect())
}
```

## Testing

### Testing Stores

```typescript
import { useRomStore } from '../store/romStore';

describe('romStore', () => {
  beforeEach(() => {
    // Reset store state
    useRomStore.setState({
      romSha1: null,
      pendingWrites: new Set(),
      error: null,
    });
  });
  
  it('should track pending writes', () => {
    useRomStore.getState().setPendingWrite('0x1234');
    expect(useRomStore.getState().hasPendingWrites).toBe(true);
  });
});
```

### Testing Validation

```typescript
import { validateColor, ValidationError } from '../store/validation';

describe('validation', () => {
  it('should validate color', () => {
    const color = validateColor({ r: 255, g: 128, b: 0 });
    expect(color).toEqual({ r: 255, g: 128, b: 0 });
  });
  
  it('should throw on invalid color', () => {
    expect(() => validateColor({ r: 'invalid' })).toThrow(ValidationError);
  });
});
```
