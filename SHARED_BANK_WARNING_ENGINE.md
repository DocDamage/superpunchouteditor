# Shared-Bank Warning Engine Implementation

## Overview
This document describes the implementation of the Shared-Bank Warning Engine for the Super Punch-Out!! ROM editor.

## Architecture

### Rust Backend (`apps/desktop/src-tauri/src/lib.rs`)

#### New Commands Added:

1. **`get_shared_bank_info`**
   - Takes a PC offset string
   - Returns detailed information about a shared bank including:
     - Filename, category, size
     - List of fighters sharing the bank
     - Whether the bank is shared or unique

2. **`get_fighter_shared_banks`**
   - Takes a fighter key
   - Returns comprehensive information about a fighter's shared banks:
     - Count of unique vs shared bins
     - List of all shared bins with details
     - List of all fighters this fighter shares with
     - Safety status (is_safe_target)

#### Supporting Types Added:

- `BankDuplication` - Record for duplicated banks
- `DuplicationResult` - Result of duplication operations
- `SpaceInfo` - Space allocation information
- `BankDuplicationManager` - Manager for tracking duplications
- `compute_hash` - Simple hash function for data integrity

### React Components

#### 1. `SharedBankWarning.tsx` (NEW)
Modal dialog component for shared bank warnings.

**Features:**
- Shows which fighters share the bank
- Displays implications of editing
- Option to duplicate the bank (placeholder for future feature)
- Visual warnings with fighter badges
- Pair info from shared_banks.json

**Props:**
- `isOpen`: boolean
- `onClose`: () => void
- `onConfirm`: (duplicate: boolean) => void
- `bankInfo`: SharedBankInfo | null
- `currentBoxer`: string

#### 2. `SharedBankIndicator.tsx` (NEW)
Visual indicator components for shared status.

**Components:**
- `SharedBankIndicator`: Badge showing "SHARED" status with tooltip
- `SharedBankSummary`: Expandable summary of shared bank status

**Features:**
- Size variants (small, medium, large)
- Hover tooltips with shared fighter info
- Expandable/collapsible summary
- Color-coded indicators (red for shared, green for unique)

#### 3. `SpriteBinEditor.tsx` (UPDATED)
Enhanced with shared bank warning integration.

**Changes:**
- Added `SharedBankWarning` modal integration
- Added `SharedBankSummary` component
- Import button now shows warning modal for shared bins
- Import button styling changes to red for shared bins
- Enhanced tooltips showing affected fighters

#### 4. `AssetManager.tsx` (UPDATED)
Added shared indicators for all asset types.

**Changes:**
- Shows shared badge for shared assets
- Warning confirmation on import of shared assets
- Summary banner for shared asset count
- Visual distinction between shared and unique assets

#### 5. `BoxerPreviewSheet.tsx` (UPDATED)
Enhanced shared bank information display.

**Changes:**
- Integrated `SharedBankSummary` component
- Shows shared pair information from shared_banks.json
- Enhanced shared bank warning with pair notes
- Bin legend shows shared indicators for shared bins

#### 6. `useStore.ts` (UPDATED)
Added shared bank utility functions.

**New Methods:**
- `getSharedBankInfo(pcOffset: string)`: Get info for a specific bank
- `getFighterSharedBanks(fighterKey: string)`: Get all shared banks for a fighter

### Data Flow

```
User clicks Import on shared bin
  ↓
SpriteBinEditor detects isShared=true
  ↓
SharedBankWarning modal opens
  ↓
Modal loads shared pair info from get_all_layouts
  ↓
User sees affected fighters and implications
  ↓
User chooses to proceed or cancel
  ↓
If proceed: Import continues
If duplicate selected: Shows "coming soon" message
```

## Shared Bank Pairs

The engine is aware of these shared bank pairs (from `data/boxer-layouts/shared_banks.json`):

1. Gabby Jay ↔ Bob Charlie
2. Bear Hugger ↔ Mad Clown
3. Piston Hurricane ↔ Aran Ryan
4. Bald Bull ↔ Mr. Sandman
5. Dragon Chan ↔ Heike Kagero
6. Masked Muscle ↔ Super Macho Man
7. Rick Bruiser ↔ Nick Bruiser

Safe solo fighters (no shared banks):
- Hoy Quarlow
- Narcis Prince
- Little Mac

## Visual Design

### Color Scheme
- **Shared indicators**: Red (#ff8888) with red borders/backgrounds
- **Unique/Safe**: Green (#6bdb7d) with green accents
- **Warnings**: Amber/Yellow (#ffd700) for edit indicators

### Icons
- ⚠️ Warning icon for shared banks
- ✓ Checkmark for unique/safe banks
- ✏️ Edit indicator for modified banks

## Future Enhancements

### Bank Duplication Feature (Placeholder)
The modal includes a checkbox for "Duplicate this bank first" which currently shows a "coming soon" message. This feature will:

1. Find or create free space in the ROM
2. Copy the original bank data to the new location
3. Update pointers to use the new location for the target boxer
4. Preserve the original shared bank for other fighters

This requires:
- ROM expansion capability
- Pointer tracking and updates
- Manifest updates
- Space management

## Files Modified/Created

### New Files:
- `apps/desktop/src/components/SharedBankWarning.tsx`
- `apps/desktop/src/components/SharedBankIndicator.tsx`
- `apps/desktop/src/components/index.ts` (barrel exports)

### Modified Files:
- `apps/desktop/src-tauri/src/lib.rs` (Rust backend)
- `apps/desktop/src/components/SpriteBinEditor.tsx`
- `apps/desktop/src/components/AssetManager.tsx`
- `apps/desktop/src/components/BoxerPreviewSheet.tsx`
- `apps/desktop/src/store/useStore.ts`

## Testing

To test the Shared-Bank Warning Engine:

1. Load a ROM
2. Select a fighter with shared banks (e.g., Gabby Jay, Bear Hugger)
3. Go to Sprite Bins section
4. Try to import a PNG into a shared bin
5. Verify the warning modal appears
6. Check that affected fighters are listed correctly
7. Verify the pair note is displayed (if available)
8. Test cancel and proceed buttons

## Notes

- The manifest already has `shared_with` fields on AssetFile - these are used throughout
- The shared_banks.json file provides additional pair metadata (notes)
- The warning engine uses the existing `isShared` field derived from `shared_sprite_bins`
- All shared bank information is derived from the manifest data, not hardcoded
