# Duplicate Shared Bank Feature

This document describes the "Duplicate Shared Bank" feature implemented for the Super Punch-Out!! ROM Editor.

## Overview

The duplicate shared bank feature allows users to clone a shared graphics bank so they can modify it without affecting the other boxer that shares it. This is an advanced feature for Phase 6 of the build plan.

## Key Components

### 1. project-core Crate (`crates/project-core/`)

**New File: `src/bank_duplication.rs`**
- `BankDuplication` struct: Tracks a single bank duplication operation
- `BankDuplicationManager`: Manages all duplications in a project
- `DuplicationResult`: Result type for duplication operations
- `DuplicationStrategy`: Configuration for where to place duplicated banks

**Modified: `src/lib.rs`**
- Added `DuplicatedBankInfo` struct for project file serialization
- Added `duplicated_banks` field to `ProjectFile`
- Added methods to `ProjectFile`: `add_duplicated_bank()`, `get_boxer_duplicated_banks()`, `has_duplicated_bank()`, `get_duplicated_bank()`, `remove_duplicated_bank()`

### 2. rom-core Crate (`crates/rom-core/`)

**Modified: `src/lib.rs`**
- Added `SIZE_2MB`, `SIZE_2_5MB`, `SIZE_4MB` constants for ROM sizes
- Added `FreeSpaceInfo` and `FreeSpaceRegion` types
- Added `KNOWN_FREE_REGIONS` constant for known free space areas
- Added methods to `Rom`:
  - `size()`: Get ROM size
  - `is_expanded()`: Check if ROM is expanded
  - `expand_to()`, `expand_to_2_5mb()`, `expand_to_4mb()`: ROM expansion
  - `pc_to_snes()`: Convert PC offset to SNES address
  - `find_free_space()`: Find free space in ROM
  - `find_or_expand_free_space()`: Find space or expand ROM
  - `align_up()`: Helper for alignment
  - `get_bank()`: Get bank number for PC offset
  - `is_region_empty()`: Check if region is empty/free

### 3. Tauri App (`apps/desktop/src-tauri/`)

**Modified: `src/lib.rs`**
- Added `bank_duplications: Mutex<BankDuplicationManager>` to `AppState`
- New commands:
  - `duplicate_shared_bank()`: Main duplication command
  - `get_boxer_bank_duplications()`: Get duplications for a boxer
  - `has_duplicated_bank()`: Check if boxer has duplicated a bank
  - `get_effective_bank_offset()`: Get effective offset (original or duplicated)
  - `get_all_bank_duplications()`: Get all duplications
  - `remove_bank_duplication()`: Remove a duplication

### 4. React Frontend (`apps/desktop/src/`)

**Modified: `src/store/useStore.ts`**
- Added `BankDuplication` and `DuplicationResult` interfaces
- Added `bankDuplications` state
- Added actions: `duplicateSharedBank()`, `loadBankDuplications()`, `hasDuplicatedBank()`, `getEffectiveBankOffset()`

**Modified: `src/components/SpriteBinEditor.tsx`**
- Updated `BinState` to track duplication status
- Added `duplicatingKey` state
- Added effect hooks to load and track duplications
- Updated `handleWarningConfirm()` to support actual duplication
- Updated UI to show duplicated status (green badge)
- Updated import button to show different states based on duplication status

**Modified: `src/components/SharedBankWarning.tsx`**
- Updated duplicate option description to reflect implemented feature

## How It Works

### User Flow

1. User selects a boxer with shared banks (e.g., Gabby Jay)
2. User clicks "Import" on a shared bank
3. Warning modal appears showing the shared status
4. User can choose to:
   - **Edit Shared Bank**: Changes affect all fighters using this bank
   - **Duplicate & Edit**: Creates a unique copy for this boxer only

### Technical Flow (Duplicate)

1. Frontend calls `duplicate_shared_bank()` Tauri command
2. Backend checks if boxer already has a duplication for this bank
3. Backend reads original bank data from ROM
4. Backend calls `find_or_expand_free_space()` to find space:
   - First checks known free regions
   - Then checks end-of-bank gaps
   - If needed, expands ROM to 2.5MB or 4MB
5. Backend writes duplicated data to new location
6. Backend creates `BankDuplication` record and registers it
7. Backend returns `DuplicationResult` with success info
8. Frontend updates state to show bank is now "UNIQUE (DUPLICATED)"
9. User can now safely edit without affecting other fighters

## ROM Expansion

The feature supports automatic ROM expansion:
- **Original**: 2MB (16Mbit)
- **Intermediate**: 2.5MB (20Mbit) - common expansion size
- **Maximum**: 4MB (32Mbit) - maximum LoROM size

When a ROM is expanded, the new space is marked as `FreeSpaceRegion::Expanded`.

## Safety Considerations

- **Warnings**: Clear warnings are shown when editing shared banks
- **Visual Indicators**: 
  - Red "SHARED" badge for shared banks
  - Green "UNIQUE (DUPLICATED)" badge for duplicated banks
- **Undo**: Duplications can be removed via `remove_bank_duplication()`
- **Project Tracking**: Duplications are tracked in the project file

## Testing

Run tests with:
```bash
cd crates/project-core && cargo test
cd crates/rom-core && cargo test
```

## Future Enhancements

1. **Pointer Updates**: Automatically update fighter pointer tables to reference duplicated banks
2. **Compression**: Support recompression of duplicated banks if needed
3. **Visualization**: Show shared bank relationships in a graph view
4. **Batch Duplicate**: Allow duplicating all shared banks for a boxer at once
