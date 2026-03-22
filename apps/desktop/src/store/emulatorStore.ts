/**
 * Emulator Store
 * 
 * Manages emulator settings and embedded emulator state.
 */

import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import { invoke } from '@tauri-apps/api/core';

// ============================================================================
// Types
// ============================================================================

export type EmulatorType = 'snes9x' | 'bsnes' | 'mesen-s' | 'other';

export interface EmulatorSettings {
  emulatorPath: string;
  emulatorType: EmulatorType;
  autoSaveBeforeLaunch: boolean;
  commandLineArgs: string;
  jumpToSelectedBoxer: boolean;
  defaultRound: number;
  saveStateDir: string | null;
}

export interface EmulatorState {
  // Settings
  settings: EmulatorSettings;
  
  // Embedded emulator state
  isRunning: boolean;
  isPaused: boolean;
  currentSlot: number;
  speed: number;
  hasRom: boolean;
  
  // Loading state
  isLoading: boolean;
  error: string | null;
}

export interface EmulatorActions {
  // Settings
  loadSettings: () => Promise<void>;
  saveSettings: (settings: EmulatorSettings) => Promise<void>;
  updateSettings: (partial: Partial<EmulatorSettings>) => Promise<void>;
  
  // External emulator
  launchExternal: () => Promise<void>;
  
  // Embedded emulator
  initEmbedded: () => Promise<void>;
  loadRomInEmulator: (romPath: string) => Promise<void>;
  startEmulation: () => Promise<void>;
  stopEmulation: () => Promise<void>;
  pauseEmulation: () => Promise<void>;
  resumeEmulation: () => Promise<void>;
  togglePause: () => Promise<void>;
  setSpeed: (speed: number) => Promise<void>;
  saveState: (slot?: number) => Promise<void>;
  loadState: (slot?: number) => Promise<void>;
  resetEmulator: () => Promise<void>;
  shutdownEmulator: () => Promise<void>;
  
  // Error handling
  setError: (error: string | null) => void;
  clearError: () => void;
}

export type EmulatorStore = EmulatorState & EmulatorActions;

// ============================================================================
// Constants
// ============================================================================

const DEFAULT_SETTINGS: EmulatorSettings = {
  emulatorPath: '',
  emulatorType: 'snes9x',
  autoSaveBeforeLaunch: true,
  commandLineArgs: '',
  jumpToSelectedBoxer: true,
  defaultRound: 1,
  saveStateDir: null,
};

// ============================================================================
// Store Implementation
// ============================================================================

export const useEmulatorStore = create<EmulatorStore>()(
  persist(
    (set, get) => ({
      // Initial state
      settings: { ...DEFAULT_SETTINGS },
      isRunning: false,
      isPaused: false,
      currentSlot: 0,
      speed: 1.0,
      hasRom: false,
      isLoading: false,
      error: null,
      
      // Actions
      loadSettings: async () => {
        try {
          const settings = await invoke<EmulatorSettings>('get_emulator_settings');
          set({ settings });
        } catch (e) {
          console.error('Failed to load emulator settings:', e);
        }
      },
      
      saveSettings: async (settings: EmulatorSettings) => {
        try {
          await invoke('set_emulator_settings', { settings });
          set({ settings });
        } catch (e) {
          console.error('Failed to save emulator settings:', e);
          throw e;
        }
      },
      
      updateSettings: async (partial: Partial<EmulatorSettings>) => {
        const updated = { ...get().settings, ...partial };
        await get().saveSettings(updated);
      },
      
      launchExternal: async () => {
        const { settings } = get();
        if (!settings.emulatorPath) {
          throw new Error('No emulator configured');
        }
        // Launch via Tauri command
        try {
          await invoke('launch_external_emulator');
        } catch (e) {
          console.error('Failed to launch emulator:', e);
          throw e;
        }
      },
      
      initEmbedded: async () => {
        set({ isLoading: true });
        try {
          await invoke('init_emulator');
          set({ isLoading: false });
        } catch (e) {
          console.error('Failed to init embedded emulator:', e);
          set({ error: (e as Error).message, isLoading: false });
        }
      },
      
      loadRomInEmulator: async (romPath: string) => {
        try {
          await invoke('emulator_load_rom', { romPath });
          set({ hasRom: true });
        } catch (e) {
          console.error('Failed to load ROM in emulator:', e);
          throw e;
        }
      },
      
      startEmulation: async () => {
        try {
          await invoke('emulator_start');
          set({ isRunning: true, isPaused: false });
        } catch (e) {
          console.error('Failed to start emulation:', e);
          throw e;
        }
      },
      
      stopEmulation: async () => {
        try {
          await invoke('emulator_stop');
          set({ isRunning: false, isPaused: false });
        } catch (e) {
          console.error('Failed to stop emulation:', e);
        }
      },
      
      pauseEmulation: async () => {
        try {
          await invoke('emulator_set_paused', { paused: true });
          set({ isPaused: true });
        } catch (e) {
          console.error('Failed to pause emulation:', e);
        }
      },
      
      resumeEmulation: async () => {
        try {
          await invoke('emulator_set_paused', { paused: false });
          set({ isPaused: false });
        } catch (e) {
          console.error('Failed to resume emulation:', e);
        }
      },
      
      togglePause: async () => {
        const { isPaused } = get();
        if (isPaused) {
          await get().resumeEmulation();
        } else {
          await get().pauseEmulation();
        }
      },
      
      setSpeed: async (speed: number) => {
        try {
          await invoke('emulator_set_speed', { speed });
          set({ speed });
        } catch (e) {
          console.error('Failed to set speed:', e);
        }
      },
      
      saveState: async (slot?: number) => {
        const targetSlot = slot ?? get().currentSlot;
        try {
          await invoke('emulator_save_state', { slot: targetSlot });
        } catch (e) {
          console.error('Failed to save state:', e);
          throw e;
        }
      },
      
      loadState: async (slot?: number) => {
        const targetSlot = slot ?? get().currentSlot;
        try {
          await invoke('emulator_load_state', { slot: targetSlot });
          set({ currentSlot: targetSlot });
        } catch (e) {
          console.error('Failed to load state:', e);
          throw e;
        }
      },
      
      resetEmulator: async () => {
        try {
          await invoke('emulator_reset');
          set({ isPaused: false });
        } catch (e) {
          console.error('Failed to reset emulator:', e);
        }
      },
      
      shutdownEmulator: async () => {
        try {
          await invoke('emulator_shutdown');
          set({ isRunning: false, isPaused: false, hasRom: false });
        } catch (e) {
          console.error('Failed to shutdown emulator:', e);
        }
      },
      
      setError: (error: string | null) => set({ error }),
      clearError: () => set({ error: null }),
    }),
    {
      name: 'spo-emulator-storage',
      partialize: (state) => ({
        settings: state.settings,
      }),
    }
  )
);

// ============================================================================
// Selectors
// ============================================================================

export const selectEmulatorSettings = (state: EmulatorStore) => state.settings;
export const selectIsEmulatorRunning = (state: EmulatorStore) => state.isRunning;
export const selectIsEmulatorPaused = (state: EmulatorStore) => state.isPaused;
export const selectEmulatorError = (state: EmulatorStore) => state.error;
