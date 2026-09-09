/**
 * Boxer Store
 * 
 * Manages boxer/fighter data, selection, and related operations.
 */

import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import { subscribeWithSelector } from 'zustand/middleware';
import type { BoxerRecord, FighterMetadata, PoseInfo, Color, AssetFile } from './types';

// ============================================================================
// Types
// ============================================================================

export interface BoxerState {
  // Data
  boxers: BoxerRecord[];
  selectedBoxerKey: string | null;
  fighters: FighterMetadata[];
  selectedFighterId: number | null;
  poses: PoseInfo[];
  
  // Palette state
  currentPalette: Color[] | null;
  currentPaletteOffset: string | null;
  
  // Loading state
  isLoading: boolean;
  error: string | null;
  
  // Computed
  readonly selectedBoxer: BoxerRecord | null;
  readonly hasSelectedBoxer: boolean;
  readonly selectedPaletteFile: AssetFile | null;
}

export interface BoxerActions {
  // Data loading
  loadBoxers: () => Promise<void>;
  loadFighterList: () => Promise<void>;
  clearBoxers: () => void;
  
  // Selection
  selectBoxer: (key: string) => Promise<void>;
  selectFighter: (id: number) => Promise<void>;
  clearSelection: () => void;
  
  // Palette
  loadPalette: (pcOffset: string, size: number) => Promise<void>;
  updatePaletteColor: (index: number, color: Color) => void;
  selectPaletteFile: (index: number) => Promise<void>;
  
  // Error handling
  setError: (error: string | null) => void;
  clearError: () => void;
}

export type BoxerStore = BoxerState & BoxerActions;

// ============================================================================
// Store Implementation
// ============================================================================

export const useBoxerStore = create<BoxerStore>()(
  subscribeWithSelector((set, get) => ({
    // Initial state
    boxers: [],
    selectedBoxerKey: null,
    fighters: [],
    selectedFighterId: null,
    poses: [],
    currentPalette: null,
    currentPaletteOffset: null,
    isLoading: false,
    error: null,
    
    // Computed getters
    get selectedBoxer() {
      const { boxers, selectedBoxerKey } = get();
      if (!selectedBoxerKey) return null;
      return boxers.find(b => b.key === selectedBoxerKey) || null;
    },
    get hasSelectedBoxer() {
      return get().selectedBoxerKey !== null;
    },
    get selectedPaletteFile() {
      const boxer = get().selectedBoxer;
      const offset = get().currentPaletteOffset;
      if (!boxer || !offset) return null;
      
      return boxer.palette_files.find(p => p.start_pc === offset) || null;
    },
    
    // Actions
    loadBoxers: async () => {
      set({ isLoading: true, error: null });
      try {
        const boxers = await invoke<BoxerRecord[]>('get_boxers');
        set({ boxers, isLoading: false });
      } catch (e) {
        console.error('Failed to load boxers:', e);
        set({ error: (e as Error).message, isLoading: false });
      }
    },
    
    loadFighterList: async () => {
      try {
        const fighters = await invoke<FighterMetadata[]>('get_fighter_list');
        set({ fighters });
      } catch (e) {
        console.error('Failed to load fighter list:', e);
        set({ error: (e as Error).message });
      }
    },
    
    clearBoxers: () => {
      set({
        boxers: [],
        selectedBoxerKey: null,
        fighters: [],
        selectedFighterId: null,
        poses: [],
        currentPalette: null,
        currentPaletteOffset: null,
      });
    },
    
    selectBoxer: async (key: string) => {
      set({ isLoading: true, error: null });
      try {
        const boxer = await invoke<BoxerRecord | null>('get_boxer', { key });
        
        if (boxer) {
          set({ 
            selectedBoxerKey: key, 
            currentPalette: null,
            currentPaletteOffset: null 
          });
          
          // Auto-load first palette if available
          if (boxer.palette_files.length > 0) {
            const firstPalette = boxer.palette_files[0];
            await get().loadPalette(firstPalette.start_pc, firstPalette.size);
          }
        } else {
          set({ error: `Boxer '${key}' not found`, isLoading: false });
        }
      } catch (e) {
        console.error('Failed to select boxer:', e);
        set({ error: (e as Error).message, isLoading: false });
      }
    },
    
    selectFighter: async (id: number) => {
      try {
        const poses = await invoke<PoseInfo[]>('get_fighter_poses', { fighterId: id });
        set({ selectedFighterId: id, poses });
      } catch (e) {
        console.error('Failed to select fighter:', e);
        set({ error: (e as Error).message });
      }
    },
    
    clearSelection: () => {
      set({
        selectedBoxerKey: null,
        selectedFighterId: null,
        poses: [],
        currentPalette: null,
        currentPaletteOffset: null,
      });
    },
    
    loadPalette: async (pcOffset: string, size: number) => {
      try {
        const palette = await invoke<Color[]>('get_palette', { pcOffset, size });
        set({ 
          currentPalette: palette,
          currentPaletteOffset: pcOffset,
          isLoading: false
        });
      } catch (e) {
        console.error('Failed to load palette:', e);
        set({ error: (e as Error).message, isLoading: false });
      }
    },
    
    updatePaletteColor: (index: number, color: Color) => {
      set((state) => {
        if (!state.currentPalette) return state;
        const next = [...state.currentPalette];
        next[index] = color;
        return { currentPalette: next };
      });
    },
    
    selectPaletteFile: async (index: number) => {
      const boxer = get().selectedBoxer;
      if (!boxer || index >= boxer.palette_files.length) return;
      
      const paletteFile = boxer.palette_files[index];
      await get().loadPalette(paletteFile.start_pc, paletteFile.size);
    },
    
    setError: (error: string | null) => set({ error }),
    clearError: () => set({ error: null }),
  }))
);

// ============================================================================
// Selectors
// ============================================================================

export const selectBoxers = (state: BoxerStore) => state.boxers;
export const selectSelectedBoxer = (state: BoxerStore) => state.selectedBoxer;
export const selectCurrentPalette = (state: BoxerStore) => state.currentPalette;
export const selectBoxerError = (state: BoxerStore) => state.error;
