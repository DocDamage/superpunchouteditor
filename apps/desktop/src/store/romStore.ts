/**
 * ROM Store
 * 
 * Manages ROM loading, validation, and core ROM state.
 * This is the foundational store that other stores depend on.
 */

import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import { invoke } from '@tauri-apps/api/core';
import { useBoxerStore } from './boxerStore';
import { useProjectStore } from './projectStore';
import { useEditStore } from './editStore';

// ============================================================================
// Types
// ============================================================================

export interface RomState {
  // State
  romSha1: string | null;
  romPath: string | null;
  isLoading: boolean;
  error: string | null;
  pendingWrites: Set<string>;
  
  // Computed (derived) getters
  readonly hasRom: boolean;
  readonly hasPendingWrites: boolean;
  readonly pendingWriteCount: number;
}

export interface RomActions {
  // Actions
  openRom: (path: string) => Promise<void>;
  closeRom: () => void;
  setPendingWrite: (pcOffset: string) => void;
  removePendingWrite: (pcOffset: string) => void;
  clearPendingWrites: () => void;
  saveRom: (outputPath: string) => Promise<void>;
  exportIpsPatch: (outputPath: string) => Promise<number>;
  setError: (error: string | null) => void;
  clearError: () => void;
}

export type RomStore = RomState & RomActions;

// ============================================================================
// Store Implementation
// ============================================================================

export const useRomStore = create<RomStore>()(
  persist(
    (set, get) => ({
      // Initial state
      romSha1: null,
      romPath: null,
      isLoading: false,
      error: null,
      pendingWrites: new Set(),
      
      // Computed getters
      get hasRom() {
        return this.romSha1 !== null;
      },
      get hasPendingWrites() {
        return this.pendingWrites.size > 0;
      },
      get pendingWriteCount() {
        return this.pendingWrites.size;
      },
      
      // Actions
      openRom: async (path: string) => {
        set({ isLoading: true, error: null });
        try {
          const sha1 = await invoke<string>('open_rom', { path });
          set({ 
            romSha1: sha1, 
            romPath: path,
            pendingWrites: new Set(),
            isLoading: false 
          });
          
          // Load related data
          await useBoxerStore.getState().loadBoxers();
          await useProjectStore.getState().loadCurrentProject();
        } catch (e) {
          console.error('Failed to open ROM:', e);
          set({ 
            error: (e as Error).message || 'Failed to open ROM',
            isLoading: false 
          });
        }
      },
      
      closeRom: () => {
        set({
          romSha1: null,
          romPath: null,
          pendingWrites: new Set(),
          error: null
        });
        // Clear dependent stores
        useBoxerStore.getState().clearBoxers();
        useEditStore.getState().clearHistory();
      },
      
      setPendingWrite: (pcOffset: string) => {
        set((state) => ({
          pendingWrites: new Set([...state.pendingWrites, pcOffset])
        }));
      },
      
      removePendingWrite: (pcOffset: string) => {
        set((state) => {
          const next = new Set(state.pendingWrites);
          next.delete(pcOffset);
          return { pendingWrites: next };
        });
      },
      
      clearPendingWrites: () => {
        set({ pendingWrites: new Set() });
      },
      
      saveRom: async (outputPath: string) => {
        try {
          await invoke('save_rom_as', { outputPath });
          // Clear pending writes after successful save
          set({ pendingWrites: new Set() });
        } catch (e) {
          console.error('Failed to save ROM:', e);
          throw e;
        }
      },
      
      exportIpsPatch: async (outputPath: string) => {
        try {
          const count = await invoke<number>('export_ips_patch', { outputPath });
          return count;
        } catch (e) {
          console.error('Failed to export IPS patch:', e);
          throw e;
        }
      },
      
      setError: (error: string | null) => set({ error }),
      clearError: () => set({ error: null }),
    }),
    {
      name: 'spo-rom-storage',
      // Only persist certain fields
      partialize: (state) => ({
        romPath: state.romPath,
        // Don't persist pending writes - they should be saved to project or lost
      }),
    }
  )
);

// ============================================================================
// Selectors (for efficient re-renders)
// ============================================================================

export const selectRomSha1 = (state: RomStore) => state.romSha1;
export const selectHasRom = (state: RomStore) => state.hasRom;
export const selectPendingWrites = (state: RomStore) => state.pendingWrites;
export const selectHasPendingWrites = (state: RomStore) => state.hasPendingWrites;
export const selectRomError = (state: RomStore) => state.error;
