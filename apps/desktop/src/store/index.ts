/**
 * State Management Index
 * 
 * This is the main entry point for all state management.
 * 
 * ## New Architecture (Recommended)
 * 
 * Import individual stores for specific domains:
 * ```typescript
 * import { useRomStore } from './store/romStore';
 * import { useBoxerStore } from './store/boxerStore';
 * ```
 * 
 * ## Legacy Compatibility
 * 
 * The old `useStore` is still available for backwards compatibility:
 * ```typescript
 * import { useStore } from './store';
 * ```
 * 
 * ## Store Overview
 * 
 * - **romStore**: ROM loading, validation, pending writes
 * - **boxerStore**: Boxer/fighter data, selection, palettes
 * - **projectStore**: Project save/load, metadata, recent projects
 * - **editStore**: Undo/redo history (uses computed getters)
 * - **emulatorStore**: Emulator settings and embedded emulator
 * - **comparisonStore**: ROM comparison and diff viewing
 * - **toolsStore**: External tools configuration
 * - **uiStore**: UI state, tabs, modals, toasts, theme
 * 
 * ## Best Practices
 * 
 * 1. Use individual stores for new code
 * 2. Use selectors to prevent unnecessary re-renders
 * 3. Prefer computed getters over stored derived values
 * 4. Use persist middleware for state that should survive reloads
 */

// Export types
export * from './types';

// Export individual stores
export { useRomStore } from './romStore';
export { useBoxerStore } from './boxerStore';
export { useProjectStore } from './projectStore';
export { useEditStore } from './editStore';
export { useEmulatorStore } from './emulatorStore';
export { useComparisonStore } from './comparisonStore';
export { useToolsStore } from './toolsStore';
export { useUiStore } from './uiStore';

// Export selectors
export {
  selectRomSha1,
  selectHasRom,
  selectPendingWrites,
  selectHasPendingWrites,
  selectRomError,
} from './romStore';

export {
  selectBoxers,
  selectSelectedBoxer,
  selectCurrentPalette,
  selectBoxerError,
} from './boxerStore';

export {
  selectCurrentProject,
  selectHasProject,
  selectIsProjectModified,
  selectRecentProjects,
  selectProjectError,
} from './projectStore';

export {
  selectCanUndo,
  selectCanRedo,
  selectUndoStack,
  selectRedoStack,
} from './editStore';

export {
  selectEmulatorSettings,
  selectIsEmulatorRunning,
  selectIsEmulatorPaused,
  selectEmulatorError,
} from './emulatorStore';

export {
  selectComparison,
  selectHasComparison,
  selectTotalChanges,
  selectViewMode,
  selectSelectedAssetId,
} from './comparisonStore';

export {
  selectTools,
  selectEnabledTools,
  selectToolsByCategory,
} from './toolsStore';

export {
  selectActiveTab,
  selectIsModalOpen,
  selectActiveModalType,
  selectToasts,
  selectTheme,
} from './uiStore';

// Legacy compatibility - re-export the old useStore
// This allows existing code to continue working during migration
export { useStore } from './useStore';
