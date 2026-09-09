/**
 * Comparison Store
 * 
 * Manages ROM comparison state and diff viewing.
 */

import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import type { RomComparison, PaletteDiff, SpriteDiff, BinaryDiff, ViewMode } from './types';

// ============================================================================
// Types
// ============================================================================

export interface ComparisonState {
  // Data
  comparison: RomComparison | null;
  selectedAssetId: string | null;
  viewMode: ViewMode;
  
  // Loading state
  isLoading: boolean;
  error: string | null;
  
  // Computed
  readonly hasComparison: boolean;
  readonly totalChanges: number;
  readonly hasSelectedAsset: boolean;
}

export interface ComparisonActions {
  // Comparison lifecycle
  generateComparison: () => Promise<void>;
  clearComparison: () => void;
  
  // Asset selection
  selectAsset: (assetId: string | null) => void;
  
  // View mode
  setViewMode: (mode: ViewMode) => void;
  
  // Diff queries
  getPaletteDiff: (pcOffset: string) => Promise<PaletteDiff | null>;
  getSpriteBinDiff: (pcOffset: string) => Promise<SpriteDiff | null>;
  getBinaryDiff: (pcOffset: string, size: number) => Promise<BinaryDiff | null>;
  
  // Rendering
  renderComparisonView: (params: {
    boxerKey: string;
    viewType: 'sprite' | 'frame' | 'animation' | 'palette' | 'portrait' | 'icon';
    showOriginal: boolean;
    showModified: boolean;
    assetOffset?: string;
    paletteOffset?: string;
  }) => Promise<Uint8Array | null>;
  
  // Export
  exportReport: (outputPath: string, format: 'json' | 'html' | 'text') => Promise<void>;
  
  // Error handling
  setError: (error: string | null) => void;
  clearError: () => void;
}

export type ComparisonStore = ComparisonState & ComparisonActions;

// ============================================================================
// Store Implementation
// ============================================================================

export const useComparisonStore = create<ComparisonStore>((set, get) => ({
  // Initial state
  comparison: null,
  selectedAssetId: null,
  viewMode: 'side-by-side',
  isLoading: false,
  error: null,
  
  // Computed getters
  get hasComparison() {
    return get().comparison !== null;
  },
  get totalChanges() {
    return get().comparison?.summary.total_changes ?? 0;
  },
  get hasSelectedAsset() {
    return get().selectedAssetId !== null;
  },
  
  // Actions
  generateComparison: async () => {
    set({ isLoading: true, error: null });
    try {
      const result = await invoke<RomComparison>('generate_comparison');
      set({ comparison: result, isLoading: false });
    } catch (e) {
      console.error('Failed to generate comparison:', e);
      set({ error: (e as Error).message, isLoading: false });
    }
  },
  
  clearComparison: () => {
    set({
      comparison: null,
      selectedAssetId: null,
      viewMode: 'side-by-side',
    });
  },
  
  selectAsset: (assetId: string | null) => {
    set({ selectedAssetId: assetId });
  },
  
  setViewMode: (mode: ViewMode) => {
    set({ viewMode: mode });
  },
  
  getPaletteDiff: async (pcOffset: string) => {
    try {
      return await invoke<PaletteDiff>('get_palette_diff', { pcOffset });
    } catch (e) {
      console.error('Failed to get palette diff:', e);
      return null;
    }
  },
  
  getSpriteBinDiff: async (pcOffset: string) => {
    try {
      return await invoke<SpriteDiff>('get_sprite_bin_diff_comparison', { pcOffset });
    } catch (e) {
      console.error('Failed to get sprite bin diff:', e);
      return null;
    }
  },
  
  getBinaryDiff: async (pcOffset: string, size: number) => {
    try {
      return await invoke<BinaryDiff>('get_binary_diff', { pcOffset, size });
    } catch (e) {
      console.error('Failed to get binary diff:', e);
      return null;
    }
  },
  
  renderComparisonView: async (params) => {
    try {
      const bytes = await invoke<number[]>('render_comparison_view', {
        boxerKey: params.boxerKey,
        viewType: params.viewType,
        showOriginal: params.showOriginal,
        showModified: params.showModified,
        assetOffset: params.assetOffset,
        paletteOffset: params.paletteOffset,
        mode: get().viewMode,
      });
      return new Uint8Array(bytes);
    } catch (e) {
      console.error('Failed to render comparison view:', e);
      return null;
    }
  },
  
  exportReport: async (outputPath: string, format: 'json' | 'html' | 'text') => {
    try {
      await invoke('export_comparison_report', { outputPath, format });
    } catch (e) {
      console.error('Failed to export comparison report:', e);
      throw e;
    }
  },
  
  setError: (error: string | null) => set({ error }),
  clearError: () => set({ error: null }),
}));

// ============================================================================
// Selectors
// ============================================================================

export const selectComparison = (state: ComparisonStore) => state.comparison;
export const selectHasComparison = (state: ComparisonStore) => state.hasComparison;
export const selectTotalChanges = (state: ComparisonStore) => state.totalChanges;
export const selectViewMode = (state: ComparisonStore) => state.viewMode;
export const selectSelectedAssetId = (state: ComparisonStore) => state.selectedAssetId;
