/**
 * Edit History Store
 * 
 * Manages undo/redo state and edit history tracking.
 * Uses computed properties for canUndo/canRedo instead of storing them.
 */

import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import type { EditSummary } from './types';

// ============================================================================
// Types
// ============================================================================

export interface EditState {
  // Use computed getters instead of storing derived values
  readonly canUndo: boolean;
  readonly canRedo: boolean;
  readonly historyLength: number;
  
  // Raw state (normalized)
  undoStack: EditSummary[];
  redoStack: EditSummary[];
  
  // Loading state
  isProcessing: boolean;
  error: string | null;
}

export interface EditActions {
  // History operations
  undo: () => Promise<void>;
  redo: () => Promise<void>;
  clearHistory: () => Promise<void>;
  refreshState: () => Promise<void>;
  
  // Recording edits
  recordPaletteEdit: (pcOffset: string, colorIndex: number, oldColor: number[], newColor: number[]) => Promise<void>;
  recordSpriteBinEdit: (pcOffset: string, oldBytes: number[], newBytes: number[]) => Promise<void>;
  recordAssetImport: (pcOffset: string, oldBytes: number[], newBytes: number[], sourcePath: string) => Promise<void>;
  
  // Error handling
  setError: (error: string | null) => void;
}

export type EditStore = EditState & EditActions;

// ============================================================================
// Store Implementation
// ============================================================================

export const useEditStore = create<EditStore>((set, get) => ({
  // Initial state - don't store computed values
  undoStack: [],
  redoStack: [],
  isProcessing: false,
  error: null,
  
  // Computed getters
  get canUndo() {
    return get().undoStack.length > 0;
  },
  get canRedo() {
    return get().redoStack.length > 0;
  },
  get historyLength() {
    return get().undoStack.length + get().redoStack.length;
  },
  
  // Actions
  undo: async () => {
    if (!get().canUndo) return;
    
    set({ isProcessing: true });
    try {
      await invoke('undo');
      await get().refreshState();
    } catch (e) {
      console.error('Undo failed:', e);
      set({ error: (e as Error).message });
    } finally {
      set({ isProcessing: false });
    }
  },
  
  redo: async () => {
    if (!get().canRedo) return;
    
    set({ isProcessing: true });
    try {
      await invoke('redo');
      await get().refreshState();
    } catch (e) {
      console.error('Redo failed:', e);
      set({ error: (e as Error).message });
    } finally {
      set({ isProcessing: false });
    }
  },
  
  clearHistory: async () => {
    try {
      await invoke('clear_history');
      set({ undoStack: [], redoStack: [] });
    } catch (e) {
      console.error('Failed to clear history:', e);
      set({ error: (e as Error).message });
    }
  },
  
  refreshState: async () => {
    try {
      const [undoStack, redoStack] = await Promise.all([
        invoke<EditSummary[]>('get_undo_stack'),
        invoke<EditSummary[]>('get_redo_stack'),
      ]);
      set({ undoStack, redoStack });
    } catch (e) {
      console.error('Failed to refresh edit state:', e);
    }
  },
  
  recordPaletteEdit: async (pcOffset, colorIndex, oldColor, newColor) => {
    try {
      await invoke('record_palette_edit', {
        pcOffset,
        colorIndex,
        oldColor,
        newColor,
      });
      await get().refreshState();
    } catch (e) {
      console.error('Failed to record palette edit:', e);
    }
  },
  
  recordSpriteBinEdit: async (pcOffset, oldBytes, newBytes) => {
    try {
      await invoke('record_sprite_bin_edit', {
        pcOffset,
        oldBytes,
        newBytes,
      });
      await get().refreshState();
    } catch (e) {
      console.error('Failed to record sprite bin edit:', e);
    }
  },
  
  recordAssetImport: async (pcOffset, oldBytes, newBytes, sourcePath) => {
    try {
      await invoke('record_asset_import', {
        pcOffset,
        oldBytes,
        newBytes,
        sourcePath,
      });
      await get().refreshState();
    } catch (e) {
      console.error('Failed to record asset import:', e);
    }
  },
  
  setError: (error: string | null) => set({ error }),
}));

// ============================================================================
// Selectors
// ============================================================================

export const selectCanUndo = (state: EditStore) => state.canUndo;
export const selectCanRedo = (state: EditStore) => state.canRedo;
export const selectUndoStack = (state: EditStore) => state.undoStack;
export const selectRedoStack = (state: EditStore) => state.redoStack;
