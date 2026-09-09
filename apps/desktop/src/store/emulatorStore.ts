/**
 * Emulator Store
 *
 * Settings are persisted as user preferences. Runtime ROM state is always projected from the
 * backend and the embedded emulator loads the canonical materialized editor revision in memory.
 */

import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import { invoke } from '@tauri-apps/api/core';

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

interface EmulatorLoadReceipt {
  revision: number;
  current_sha1: string;
  byte_length: number;
}

export interface EmulatorState {
  settings: EmulatorSettings;
  isRunning: boolean;
  isPaused: boolean;
  currentSlot: number;
  speed: number;
  hasRom: boolean;
  loadedRevision: number | null;
  loadedSha1: string | null;
  isLoading: boolean;
  error: string | null;
}

export interface EmulatorActions {
  loadSettings: () => Promise<void>;
  saveSettings: (settings: EmulatorSettings) => Promise<void>;
  updateSettings: (partial: Partial<EmulatorSettings>) => Promise<void>;
  launchExternal: () => Promise<void>;
  initEmbedded: () => Promise<void>;
  loadRomInEmulator: (romPath?: string) => Promise<void>;
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
  setError: (error: string | null) => void;
  clearError: () => void;
}

export type EmulatorStore = EmulatorState & EmulatorActions;

const DEFAULT_SETTINGS: EmulatorSettings = {
  emulatorPath: '',
  emulatorType: 'snes9x',
  autoSaveBeforeLaunch: true,
  commandLineArgs: '',
  jumpToSelectedBoxer: true,
  defaultRound: 1,
  saveStateDir: null,
};

export const useEmulatorStore = create<EmulatorStore>()(
  persist(
    (set, get) => ({
      settings: { ...DEFAULT_SETTINGS },
      isRunning: false,
      isPaused: false,
      currentSlot: 0,
      speed: 1.0,
      hasRom: false,
      loadedRevision: null,
      loadedSha1: null,
      isLoading: false,
      error: null,

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
        await get().saveSettings({ ...get().settings, ...partial });
      },

      launchExternal: async () => {
        throw new Error(
          'Legacy external-emulator launch is experimental. Use the embedded emulator so the exact current revision is tested.',
        );
      },

      initEmbedded: async () => {
        set({ isLoading: true, error: null });
        try {
          await invoke('init_emulator');
          set({ isLoading: false });
        } catch (e) {
          const message = e instanceof Error ? e.message : String(e);
          console.error('Failed to init embedded emulator:', e);
          set({ error: message, isLoading: false });
          throw e;
        }
      },

      loadRomInEmulator: async (_romPath?: string) => {
        try {
          const receipt = await invoke<EmulatorLoadReceipt>('emulator_load_current_rom');
          set({
            hasRom: true,
            loadedRevision: receipt.revision,
            loadedSha1: receipt.current_sha1,
          });
        } catch (e) {
          console.error('Failed to load current ROM revision in emulator:', e);
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
        if (get().isPaused) {
          await get().resumeEmulation();
        } else {
          await get().pauseEmulation();
        }
      },

      setSpeed: async (speed: number) => {
        if (!Number.isFinite(speed) || speed <= 0 || speed > 8) {
          throw new Error('Emulator speed must be greater than 0 and no more than 8x');
        }
        await invoke('emulator_set_speed', { speed });
        set({ speed });
      },

      saveState: async (slot?: number) => {
        const targetSlot = slot ?? get().currentSlot;
        await invoke('emulator_save_state', { slot: targetSlot });
      },

      loadState: async (slot?: number) => {
        const targetSlot = slot ?? get().currentSlot;
        await invoke('emulator_load_state', { slot: targetSlot });
        set({ currentSlot: targetSlot });
      },

      resetEmulator: async () => {
        await invoke('emulator_reset');
        set({ isPaused: false });
      },

      shutdownEmulator: async () => {
        try {
          await invoke('emulator_shutdown');
        } finally {
          set({
            isRunning: false,
            isPaused: false,
            hasRom: false,
            loadedRevision: null,
            loadedSha1: null,
          });
        }
      },

      setError: (error: string | null) => set({ error }),
      clearError: () => set({ error: null }),
    }),
    {
      name: 'spo-emulator-storage',
      partialize: (state) => ({ settings: state.settings }),
    },
  ),
);

export const selectEmulatorSettings = (state: EmulatorStore) => state.settings;
export const selectIsEmulatorRunning = (state: EmulatorStore) => state.isRunning;
export const selectIsEmulatorPaused = (state: EmulatorStore) => state.isPaused;
export const selectEmulatorError = (state: EmulatorStore) => state.error;
