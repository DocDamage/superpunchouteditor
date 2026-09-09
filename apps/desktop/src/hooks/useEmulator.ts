/**
 * useEmulator Hook
 * 
 * Manages emulator state and provides controls for the embedded emulator.
 * Handles keyboard input, audio, save states, and rendering loop.
 * 
 * CONNECTED TO TAURI BACKEND - This hook now invokes real Tauri commands
 * to control the embedded Snes9x emulator.
 */

import { useState, useEffect, useCallback, useRef, useMemo } from 'react';
import { invoke } from '@tauri-apps/api/core';

export type EmulatorState = 'stopped' | 'running' | 'paused' | 'loading';
export type ControllerType = 'keyboard' | 'gamepad' | 'both';
export type ScalingMode = 'pixel-perfect' | 'smooth' | 'stretch';
export type SpeedMode = 0.25 | 0.5 | 1.0 | 2.0;

export interface InputMapping {
  button: string;
  key: string;
  label: string;
}

export interface EmulatorConfig {
  volume: number;
  muted: boolean;
  controllerType: ControllerType;
  scalingMode: ScalingMode;
  integerScaling: boolean;
  showFps: boolean;
  saveStateSlot: number;
  speed: SpeedMode;
}

export interface SaveState {
  slot: number;
  timestamp: number;
  exists: boolean;
  preview?: string;
}

export interface EmulatorStatus {
  fps: number;
  audioEnabled: boolean;
  romLoaded: string | null;
  frameCount: number;
}

export interface CreatorRuntimeState {
  active: boolean;
  magic: number;
  heartbeat: number;
  input_low: number;
  input_high: number;
  cursor: number;
  action: number;
  page: number;
  dirty: boolean;
  render_visible: boolean;
  render_page: number;
  render_cursor: number;
  render_rows: [number, number, number, number];
  render_revision: number;
  session_present: boolean;
  session_boxer_id: number;
  session_circuit: number;
  session_unlock_order: number;
  session_intro_text_id: number;
  session_status: number;
  session_error_code: number;
  intro_edit_active: boolean;
  intro_cursor: number;
  intro_length: number;
  intro_bytes: number[];
  name_edit_active: boolean;
  name_cursor: number;
  name_length: number;
  name_bytes: number[];
}

export interface CreatorSessionState {
  boxer_id: number;
  circuit: number;
  unlock_order: number;
  intro_text_id: number;
  status: number;
  error_code: number;
  intro_text: string;
  name_text: string;
}

export interface CreatorRuntimeActionResolution {
  runtime_state: CreatorRuntimeState;
  message: string | null;
}

// SNES button mappings
export const SNES_BUTTONS = {
  B: 0,
  Y: 1,
  SELECT: 2,
  START: 3,
  UP: 4,
  DOWN: 5,
  LEFT: 6,
  RIGHT: 7,
  A: 8,
  X: 9,
  L: 10,
  R: 11,
} as const;

// Default keyboard mappings (WASD style)
export const DEFAULT_KEY_MAPPINGS_WASD: InputMapping[] = [
  { button: 'B', key: 'KeyZ', label: 'Z' },
  { button: 'Y', key: 'KeyA', label: 'A' },
  { button: 'SELECT', key: 'ShiftRight', label: 'RShift' },
  { button: 'START', key: 'Enter', label: 'Enter' },
  { button: 'UP', key: 'KeyW', label: 'W' },
  { button: 'DOWN', key: 'KeyS', label: 'S' },
  { button: 'LEFT', key: 'KeyA', label: 'A' },
  { button: 'RIGHT', key: 'KeyD', label: 'D' },
  { button: 'A', key: 'KeyX', label: 'X' },
  { button: 'X', key: 'KeyS', label: 'S' },
  { button: 'L', key: 'KeyQ', label: 'Q' },
  { button: 'R', key: 'KeyE', label: 'E' },
];

// Arrow keys style mapping
export const DEFAULT_KEY_MAPPINGS_ARROWS: InputMapping[] = [
  { button: 'B', key: 'KeyZ', label: 'Z' },
  { button: 'Y', key: 'KeyA', label: 'A' },
  { button: 'SELECT', key: 'ShiftRight', label: 'RShift' },
  { button: 'START', key: 'Enter', label: 'Enter' },
  { button: 'UP', key: 'ArrowUp', label: '↑' },
  { button: 'DOWN', key: 'ArrowDown', label: '↓' },
  { button: 'LEFT', key: 'ArrowLeft', label: '←' },
  { button: 'RIGHT', key: 'ArrowRight', label: '→' },
  { button: 'A', key: 'KeyX', label: 'X' },
  { button: 'X', key: 'KeyS', label: 'S' },
  { button: 'L', key: 'KeyQ', label: 'Q' },
  { button: 'R', key: 'KeyW', label: 'W' },
];

// Fight stick / Hitbox style mapping
export const DEFAULT_KEY_MAPPINGS_FIGHTSTICK: InputMapping[] = [
  { button: 'B', key: 'KeyM', label: 'M' },
  { button: 'Y', key: 'KeyJ', label: 'J' },
  { button: 'SELECT', key: 'KeyN', label: 'N' },
  { button: 'START', key: 'Enter', label: 'Enter' },
  { button: 'UP', key: 'KeyW', label: 'W' },
  { button: 'DOWN', key: 'KeyS', label: 'S' },
  { button: 'LEFT', key: 'KeyA', label: 'A' },
  { button: 'RIGHT', key: 'KeyD', label: 'D' },
  { button: 'A', key: 'Comma', label: ',' },
  { button: 'X', key: 'KeyK', label: 'K' },
  { button: 'L', key: 'KeyU', label: 'U' },
  { button: 'R', key: 'KeyI', label: 'I' },
];

export type KeyMappingPreset = 'wasd' | 'arrows' | 'fightstick' | 'custom';

export interface UseEmulatorOptions {
  canvasRef: React.RefObject<HTMLCanvasElement | null>;
  onFrame?: (imageData: ImageData) => void;
  onAudio?: (samples: Float32Array) => void;
  onError?: (error: Error) => void;
}

export interface UseEmulatorReturn {
  // State
  state: EmulatorState;
  config: EmulatorConfig;
  status: EmulatorStatus;
  saveStates: SaveState[];
  currentMappings: InputMapping[];
  currentPreset: KeyMappingPreset;
  isInitialized: boolean;
  
  // Controls
  initialize: (corePath?: string) => Promise<boolean>;
  start: () => Promise<void>;
  pause: () => Promise<void>;
  resume: () => Promise<void>;
  stop: () => Promise<void>;
  reset: (hard?: boolean) => Promise<void>;
  
  // Speed control
  setSpeed: (speed: SpeedMode) => void;
  frameAdvance: () => Promise<void>;
  
  // Save/Load states
  saveState: (slot: number) => Promise<void>;
  loadState: (slot: number) => Promise<void>;
  deleteState: (slot: number) => Promise<void>;
  setSaveStateSlot: (slot: number) => void;
  refreshSaveStates: () => Promise<void>;
  
  // Audio
  setVolume: (volume: number) => void;
  toggleMute: () => void;
  
  // Display
  setScalingMode: (mode: ScalingMode) => void;
  toggleIntegerScaling: () => void;
  toggleFps: () => void;
  takeScreenshot: () => string | null;
  
  // Input
  setControllerType: (type: ControllerType) => void;
  loadKeyMappingPreset: (preset: KeyMappingPreset) => void;
  updateKeyMapping: (button: string, key: string, label: string) => void;
  setInput: (buttons: number) => Promise<void>;
  
  // ROM
  loadRom: (romData: Uint8Array, name: string) => Promise<void>;
  loadRomFromPath: (romPath: string) => Promise<void>;
  loadRomWithEdits: (romPath: string, edits: Record<string, number[]>) => Promise<void>;
  getCreatorRuntimeState: () => Promise<CreatorRuntimeState | null>;
  setCreatorSessionState: (session: CreatorSessionState | null) => Promise<void>;
  resolveCreatorRuntimeAction: () => Promise<CreatorRuntimeActionResolution>;
  unloadRom: () => void;
  compareWithOriginal: () => void;
  
  // Layout
  toggleFullscreen: () => void;
  
  // Cleanup
  shutdown: () => Promise<void>;
}

const DEFAULT_CONFIG: EmulatorConfig = {
  volume: 0.7,
  muted: false,
  controllerType: 'keyboard',
  scalingMode: 'pixel-perfect',
  integerScaling: true,
  showFps: true,
  saveStateSlot: 0,
  speed: 1.0,
};

export function useEmulator(options: UseEmulatorOptions): UseEmulatorReturn {
  const { canvasRef, onFrame, onAudio, onError } = options;
  
  // Core state
  const [state, setState] = useState<EmulatorState>('stopped');
  const [config, setConfig] = useState<EmulatorConfig>(DEFAULT_CONFIG);
  const [status, setStatus] = useState<EmulatorStatus>({
    fps: 0,
    audioEnabled: true,
    romLoaded: null,
    frameCount: 0,
  });
  const [saveStates, setSaveStates] = useState<SaveState[]>(
    Array.from({ length: 10 }, (_, i) => ({
      slot: i,
      timestamp: 0,
      exists: false,
    }))
  );
  const [currentMappings, setCurrentMappings] = useState<InputMapping[]>(DEFAULT_KEY_MAPPINGS_WASD);
  const [currentPreset, setCurrentPreset] = useState<KeyMappingPreset>('wasd');
  const [isInitialized, setIsInitialized] = useState(false);
  
  // Refs for animation loop and state
  const animationFrameRef = useRef<number | null>(null);
  const lastTimeRef = useRef<number>(0);
  const frameCountRef = useRef<number>(0);
  const fpsUpdateTimeRef = useRef<number>(0);
  const inputStateRef = useRef<Record<string, boolean>>({});
  const romDataRef = useRef<Uint8Array | null>(null);
  const originalRomDataRef = useRef<Uint8Array | null>(null);
  const isComparingRef = useRef<boolean>(false);
  
  // Audio context
  const audioContextRef = useRef<AudioContext | null>(null);
  const audioWorkletRef = useRef<AudioWorkletNode | null>(null);
  
  // ============================================================================
  // TAURI BACKEND INTEGRATION
  // ============================================================================
  
  // Initialize emulator with Snes9x core
  const initialize = useCallback(async (corePath?: string): Promise<boolean> => {
    try {
      await invoke('init_emulator', { corePath });
      setIsInitialized(true);
      console.log('Snes9x emulator initialized successfully');
      return true;
    } catch (error) {
      console.error('Failed to initialize emulator:', error);
      onError?.(error as Error);
      return false;
    }
  }, [onError]);
  
  // Load ROM from file path
  const loadRomFromPath = useCallback(async (romPath: string): Promise<void> => {
    try {
      await invoke('emulator_load_rom', { romPath });
      setStatus(prev => ({ ...prev, romLoaded: romPath }));
      setState('stopped');
    } catch (error) {
      console.error('Failed to load ROM:', error);
      onError?.(error as Error);
      throw error;
    }
  }, [onError]);
  
  // Load ROM from memory buffer
  const loadRom = useCallback(async (romData: Uint8Array, name: string): Promise<void> => {
    try {
      // Convert Uint8Array to number array for Tauri
      const romArray = Array.from(romData);
      await invoke('emulator_load_rom_from_memory', { romData: romArray });
      romDataRef.current = romData;
      if (!originalRomDataRef.current) {
        originalRomDataRef.current = new Uint8Array(romData);
      }
      setStatus(prev => ({ ...prev, romLoaded: name }));
      setState('stopped');
    } catch (error) {
      console.error('Failed to load ROM:', error);
      onError?.(error as Error);
      throw error;
    }
  }, [onError]);
  
  // Load ROM with pending edits applied
  const loadRomWithEdits = useCallback(async (romPath: string, edits: Record<string, number[]>): Promise<void> => {
    try {
      await invoke('emulator_load_rom_with_edits', { romPath, edits });
      setStatus(prev => ({ ...prev, romLoaded: `${romPath} (with edits)` }));
      setState('stopped');
    } catch (error) {
      console.error('Failed to load ROM with edits:', error);
      onError?.(error as Error);
      throw error;
    }
  }, [onError]);

  const getCreatorRuntimeState = useCallback(async (): Promise<CreatorRuntimeState | null> => {
    try {
      return await invoke<CreatorRuntimeState>('emulator_get_creator_runtime_state');
    } catch (error) {
      console.error('Failed to fetch creator runtime state:', error);
      return null;
    }
  }, []);

  const setCreatorSessionState = useCallback(async (session: CreatorSessionState | null): Promise<void> => {
    try {
      await invoke('emulator_set_creator_session_state', { session });
    } catch (error) {
      console.error('Failed to update creator session state:', error);
      throw error;
    }
  }, []);

  const resolveCreatorRuntimeAction = useCallback(async (): Promise<CreatorRuntimeActionResolution> => {
    try {
      return await invoke<CreatorRuntimeActionResolution>('emulator_resolve_creator_runtime_action');
    } catch (error) {
      console.error('Failed to resolve creator runtime action:', error);
      throw error;
    }
  }, []);
  
  // Start emulation
  const start = useCallback(async (): Promise<void> => {
    if (!isInitialized) {
      const success = await initialize();
      if (!success) return;
    }
    
    try {
      await invoke('emulator_start');
      setState('running');
      
      // Start frame polling loop
      const pollFrame = async () => {
        if (state !== 'running') return;
        
        try {
          const frameData = await invoke<{
            pixels: number[];
            width: number;
            height: number;
          } | null>('emulator_get_frame');
          
          if (frameData && canvasRef.current) {
            const canvas = canvasRef.current;
            const ctx = canvas.getContext('2d');
            if (ctx) {
              // Convert number array to Uint8ClampedArray
              const pixelData = new Uint8ClampedArray(frameData.pixels);
              const imageData = new ImageData(pixelData, frameData.width, frameData.height);
              ctx.putImageData(imageData, 0, 0);
              onFrame?.(imageData);
              
              // Update frame count
              frameCountRef.current++;
            }
          }
          
          // Update FPS counter
          const now = performance.now();
          if (now - fpsUpdateTimeRef.current >= 1000) {
            setStatus(prev => ({
              ...prev,
              fps: Math.round(frameCountRef.current * 1000 / (now - fpsUpdateTimeRef.current)),
              frameCount: prev.frameCount + frameCountRef.current,
            }));
            frameCountRef.current = 0;
            fpsUpdateTimeRef.current = now;
          }
        } catch (error) {
          console.error('Frame error:', error);
        }
        
        animationFrameRef.current = requestAnimationFrame(pollFrame);
      };
      
      animationFrameRef.current = requestAnimationFrame(pollFrame);
    } catch (error) {
      console.error('Failed to start emulator:', error);
      onError?.(error as Error);
    }
  }, [isInitialized, initialize, state, canvasRef, onFrame]);
  
  // Stop emulation
  const stop = useCallback(async (): Promise<void> => {
    try {
      await invoke('emulator_stop');
      setState('stopped');
      setStatus(prev => ({ ...prev, fps: 0 }));
      
      if (animationFrameRef.current) {
        cancelAnimationFrame(animationFrameRef.current);
        animationFrameRef.current = null;
      }
    } catch (error) {
      console.error('Failed to stop emulator:', error);
    }
  }, []);
  
  // Pause emulation
  const pause = useCallback(async (): Promise<void> => {
    try {
      await invoke('emulator_set_paused', { paused: true });
      setState('paused');
      
      if (animationFrameRef.current) {
        cancelAnimationFrame(animationFrameRef.current);
        animationFrameRef.current = null;
      }
    } catch (error) {
      console.error('Failed to pause emulator:', error);
    }
  }, []);
  
  // Resume emulation
  const resume = useCallback(async (): Promise<void> => {
    try {
      await invoke('emulator_set_paused', { paused: false });
      setState('running');
      
      // Restart frame polling
      const pollFrame = async () => {
        if (state !== 'running') return;
        
        try {
          const frameData = await invoke<{
            pixels: number[];
            width: number;
            height: number;
          } | null>('emulator_get_frame');
          
          if (frameData && canvasRef.current) {
            const canvas = canvasRef.current;
            const ctx = canvas.getContext('2d');
            if (ctx) {
              const pixelData = new Uint8ClampedArray(frameData.pixels);
              const imageData = new ImageData(pixelData, frameData.width, frameData.height);
              ctx.putImageData(imageData, 0, 0);
              onFrame?.(imageData);
              frameCountRef.current++;
            }
          }
          
          const now = performance.now();
          if (now - fpsUpdateTimeRef.current >= 1000) {
            setStatus(prev => ({
              ...prev,
              fps: Math.round(frameCountRef.current * 1000 / (now - fpsUpdateTimeRef.current)),
              frameCount: prev.frameCount + frameCountRef.current,
            }));
            frameCountRef.current = 0;
            fpsUpdateTimeRef.current = now;
          }
        } catch (error) {
          console.error('Frame error:', error);
        }
        
        animationFrameRef.current = requestAnimationFrame(pollFrame);
      };
      
      animationFrameRef.current = requestAnimationFrame(pollFrame);
    } catch (error) {
      console.error('Failed to resume emulator:', error);
    }
  }, [state, canvasRef, onFrame]);
  
  // Reset emulator
  const reset = useCallback(async (hard = false): Promise<void> => {
    try {
      await invoke('emulator_reset', { hard });
      if (hard) {
        setState('stopped');
      }
    } catch (error) {
      console.error('Failed to reset emulator:', error);
    }
  }, []);
  
  // Set emulation speed
  const setSpeed = useCallback((speed: SpeedMode): void => {
    setConfig(prev => ({ ...prev, speed }));
    invoke('emulator_set_speed', { speed }).catch(error => {
      console.error('Failed to set speed:', error);
    });
  }, []);
  
  // Advance one frame (frame stepping)
  const frameAdvance = useCallback(async (): Promise<void> => {
    try {
      await invoke('emulator_advance_frame');
      
      // Get and display the frame
      const frameData = await invoke<{
        pixels: number[];
        width: number;
        height: number;
      } | null>('emulator_get_frame');
      
      if (frameData && canvasRef.current) {
        const canvas = canvasRef.current;
        const ctx = canvas.getContext('2d');
        if (ctx) {
          const pixelData = new Uint8ClampedArray(frameData.pixels);
          const imageData = new ImageData(pixelData, frameData.width, frameData.height);
          ctx.putImageData(imageData, 0, 0);
          onFrame?.(imageData);
        }
      }
      
      setStatus(prev => ({ ...prev, frameCount: prev.frameCount + 1 }));
    } catch (error) {
      console.error('Failed to advance frame:', error);
    }
  }, [canvasRef, onFrame]);
  
  // Save state to slot
  const saveState = useCallback(async (slot: number): Promise<void> => {
    try {
      await invoke('emulator_save_state', { slot });
      await refreshSaveStates();
    } catch (error) {
      console.error('Failed to save state:', error);
    }
  }, []);
  
  // Load state from slot
  const loadState = useCallback(async (slot: number): Promise<void> => {
    try {
      await invoke('emulator_load_state', { slot });
      setConfig(prev => ({ ...prev, saveStateSlot: slot }));
    } catch (error) {
      console.error('Failed to load state:', error);
    }
  }, []);
  
  // Delete save state
  const deleteState = useCallback(async (slot: number): Promise<void> => {
    try {
      await invoke('emulator_delete_save_state', { slot });
      await refreshSaveStates();
    } catch (error) {
      console.error('Failed to delete state:', error);
    }
  }, []);
  
  // Refresh save states list
  const refreshSaveStates = useCallback(async (): Promise<void> => {
    try {
      const states = await invoke<Array<[number, string]>>('emulator_get_save_states');
      setSaveStates(prev => prev.map(s => {
        const found = states.find(([slot]) => slot === s.slot);
        return {
          ...s,
          exists: !!found,
          timestamp: found ? new Date(found[1]).getTime() : 0,
        };
      }));
    } catch (error) {
      console.error('Failed to refresh save states:', error);
    }
  }, []);
  
  // Set save state slot
  const setSaveStateSlot = useCallback((slot: number): void => {
    setConfig(prev => ({ ...prev, saveStateSlot: slot }));
  }, []);
  
  // Set controller input (button bitmask)
  const setInput = useCallback(async (buttons: number): Promise<void> => {
    try {
      await invoke('emulator_set_input', { buttons });
    } catch (error) {
      console.error('Failed to set input:', error);
    }
  }, []);
  
  // Shutdown emulator
  const shutdown = useCallback(async (): Promise<void> => {
    try {
      if (animationFrameRef.current) {
        cancelAnimationFrame(animationFrameRef.current);
      }
      await invoke('emulator_shutdown');
      setIsInitialized(false);
      setState('stopped');
    } catch (error) {
      console.error('Failed to shutdown emulator:', error);
    }
  }, []);
  
  // ============================================================================
  // LOCAL UI STATE MANAGEMENT
  // ============================================================================
  
  // Initialize audio context
  useEffect(() => {
    const initAudio = async () => {
      try {
        const audioContextCtor = window.AudioContext
          || (window as unknown as { webkitAudioContext?: typeof window.AudioContext }).webkitAudioContext;
        if (audioContextCtor) {
          audioContextRef.current = new audioContextCtor({ sampleRate: 32040 });
          setStatus(prev => ({ ...prev, audioEnabled: true }));
        }
      } catch (e) {
        console.error('Failed to initialize audio:', e);
        setStatus(prev => ({ ...prev, audioEnabled: false }));
      }
    };
    
    initAudio();
    
    return () => {
      if (audioContextRef.current?.state !== 'closed') {
        audioContextRef.current?.close();
      }
    };
  }, []);
  
  // Keyboard input handling
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (state !== 'running') return;
      
      const mapping = currentMappings.find(m => m.key === e.code);
      if (mapping) {
        e.preventDefault();
        inputStateRef.current[mapping.button] = true;
        
        // Send input to emulator
        const buttons = getInputBitmask();
        setInput(buttons);
      }
      
      // Hotkeys
      if (e.code === 'F5' && !e.repeat) {
        e.preventDefault();
        saveState(config.saveStateSlot);
      } else if (e.code === 'F7' && !e.repeat) {
        e.preventDefault();
        loadState(config.saveStateSlot);
      } else if (e.code === 'F9' && !e.repeat) {
        e.preventDefault();
        frameAdvance();
      }
    };
    
    const handleKeyUp = (e: KeyboardEvent) => {
      const mapping = currentMappings.find(m => m.key === e.code);
      if (mapping) {
        inputStateRef.current[mapping.button] = false;
        
        // Send input to emulator
        const buttons = getInputBitmask();
        setInput(buttons);
      }
    };
    
    // Helper to convert input state to SNES button bitmask
    function getInputBitmask(): number {
      let buttons = 0;
      if (inputStateRef.current['B']) buttons |= 0x8000;
      if (inputStateRef.current['Y']) buttons |= 0x4000;
      if (inputStateRef.current['SELECT']) buttons |= 0x2000;
      if (inputStateRef.current['START']) buttons |= 0x1000;
      if (inputStateRef.current['UP']) buttons |= 0x0800;
      if (inputStateRef.current['DOWN']) buttons |= 0x0400;
      if (inputStateRef.current['LEFT']) buttons |= 0x0200;
      if (inputStateRef.current['RIGHT']) buttons |= 0x0100;
      if (inputStateRef.current['A']) buttons |= 0x0080;
      if (inputStateRef.current['X']) buttons |= 0x0040;
      if (inputStateRef.current['L']) buttons |= 0x0020;
      if (inputStateRef.current['R']) buttons |= 0x0010;
      return buttons;
    }
    
    window.addEventListener('keydown', handleKeyDown);
    window.addEventListener('keyup', handleKeyUp);
    
    return () => {
      window.removeEventListener('keydown', handleKeyDown);
      window.removeEventListener('keyup', handleKeyUp);
    };
  }, [state, currentMappings, config.saveStateSlot, setInput]);
  
  // Gamepad input handling
  useEffect(() => {
    if (config.controllerType === 'keyboard') return;
    
    let gamepadIndex: number | null = null;
    
    const handleGamepadConnected = (e: GamepadEvent) => {
      gamepadIndex = e.gamepad.index;
    };
    
    const handleGamepadDisconnected = (e: GamepadEvent) => {
      if (gamepadIndex === e.gamepad.index) {
        gamepadIndex = null;
      }
    };
    
    window.addEventListener('gamepadconnected', handleGamepadConnected);
    window.addEventListener('gamepaddisconnected', handleGamepadDisconnected);
    
    // Poll gamepad state
    const pollGamepad = () => {
      if (gamepadIndex !== null && state === 'running') {
        const gamepad = navigator.getGamepads()[gamepadIndex];
        if (gamepad) {
          // Standard gamepad mapping
          inputStateRef.current['B'] = gamepad.buttons[0].pressed;
          inputStateRef.current['A'] = gamepad.buttons[1].pressed;
          inputStateRef.current['Y'] = gamepad.buttons[2].pressed;
          inputStateRef.current['X'] = gamepad.buttons[3].pressed;
          inputStateRef.current['L'] = gamepad.buttons[4].pressed;
          inputStateRef.current['R'] = gamepad.buttons[5].pressed;
          inputStateRef.current['SELECT'] = gamepad.buttons[8].pressed;
          inputStateRef.current['START'] = gamepad.buttons[9].pressed;
          inputStateRef.current['UP'] = gamepad.buttons[12].pressed || gamepad.axes[1] < -0.5;
          inputStateRef.current['DOWN'] = gamepad.buttons[13].pressed || gamepad.axes[1] > 0.5;
          inputStateRef.current['LEFT'] = gamepad.buttons[14].pressed || gamepad.axes[0] < -0.5;
          inputStateRef.current['RIGHT'] = gamepad.buttons[15].pressed || gamepad.axes[0] > 0.5;
          
          // Send input to emulator
          let buttons = 0;
          if (inputStateRef.current['B']) buttons |= 0x8000;
          if (inputStateRef.current['Y']) buttons |= 0x4000;
          if (inputStateRef.current['SELECT']) buttons |= 0x2000;
          if (inputStateRef.current['START']) buttons |= 0x1000;
          if (inputStateRef.current['UP']) buttons |= 0x0800;
          if (inputStateRef.current['DOWN']) buttons |= 0x0400;
          if (inputStateRef.current['LEFT']) buttons |= 0x0200;
          if (inputStateRef.current['RIGHT']) buttons |= 0x0100;
          if (inputStateRef.current['A']) buttons |= 0x0080;
          if (inputStateRef.current['X']) buttons |= 0x0040;
          if (inputStateRef.current['L']) buttons |= 0x0020;
          if (inputStateRef.current['R']) buttons |= 0x0010;
          
          setInput(buttons);
        }
      }
      requestAnimationFrame(pollGamepad);
    };
    
    const rafId = requestAnimationFrame(pollGamepad);
    
    return () => {
      window.removeEventListener('gamepadconnected', handleGamepadConnected);
      window.removeEventListener('gamepaddisconnected', handleGamepadDisconnected);
      cancelAnimationFrame(rafId);
    };
  }, [config.controllerType, state, setInput]);
  
  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (animationFrameRef.current) {
        cancelAnimationFrame(animationFrameRef.current);
      }
      shutdown();
    };
  }, [shutdown]);
  
  // ============================================================================
  // UI CONTROL FUNCTIONS
  // ============================================================================
  
  const setVolume = useCallback((volume: number) => {
    setConfig(prev => ({ ...prev, volume: Math.max(0, Math.min(1, volume)) }));
    // Note: Volume control would need to be implemented in audio output
  }, []);
  
  const toggleMute = useCallback(() => {
    setConfig(prev => ({ ...prev, muted: !prev.muted }));
  }, []);
  
  const setScalingMode = useCallback((mode: ScalingMode) => {
    setConfig(prev => ({ ...prev, scalingMode: mode }));
  }, []);
  
  const toggleIntegerScaling = useCallback(() => {
    setConfig(prev => ({ ...prev, integerScaling: !prev.integerScaling }));
  }, []);
  
  const toggleFps = useCallback(() => {
    setConfig(prev => ({ ...prev, showFps: !prev.showFps }));
  }, []);
  
  const takeScreenshot = useCallback((): string | null => {
    const canvas = canvasRef.current;
    if (canvas) {
      return canvas.toDataURL('image/png');
    }
    return null;
  }, [canvasRef]);
  
  const setControllerType = useCallback((type: ControllerType) => {
    setConfig(prev => ({ ...prev, controllerType: type }));
  }, []);
  
  const loadKeyMappingPreset = useCallback((preset: KeyMappingPreset) => {
    setCurrentPreset(preset);
    switch (preset) {
      case 'wasd':
        setCurrentMappings(DEFAULT_KEY_MAPPINGS_WASD);
        break;
      case 'arrows':
        setCurrentMappings(DEFAULT_KEY_MAPPINGS_ARROWS);
        break;
      case 'fightstick':
        setCurrentMappings(DEFAULT_KEY_MAPPINGS_FIGHTSTICK);
        break;
      // 'custom' keeps current mappings
    }
  }, []);
  
  const updateKeyMapping = useCallback((button: string, key: string, label: string) => {
    setCurrentPreset('custom');
    setCurrentMappings(prev => 
      prev.map(m => m.button === button ? { ...m, key, label } : m)
    );
  }, []);
  
  const unloadRom = useCallback(() => {
    romDataRef.current = null;
    originalRomDataRef.current = null;
    setStatus(prev => ({ ...prev, romLoaded: null }));
    setState('stopped');
  }, []);
  
  const compareWithOriginal = useCallback(() => {
    if (originalRomDataRef.current && romDataRef.current) {
      isComparingRef.current = !isComparingRef.current;
      // Toggle between edited and original ROM
      if (isComparingRef.current) {
        // Load original for comparison
      } else {
        // Return to edited ROM
      }
    }
  }, []);
  
  const toggleFullscreen = useCallback(() => {
    const canvas = canvasRef.current;
    if (canvas) {
      if (!document.fullscreenElement) {
        canvas.requestFullscreen();
      } else {
        document.exitFullscreen();
      }
    }
  }, [canvasRef]);
  
  return useMemo(() => ({
    // State
    state,
    config,
    status,
    saveStates,
    currentMappings,
    currentPreset,
    isInitialized,
    
    // Controls
    initialize,
    start,
    pause,
    resume,
    stop,
    reset,
    setSpeed,
    frameAdvance,
    saveState,
    loadState,
    deleteState,
    setSaveStateSlot,
    refreshSaveStates,
    
    // Audio
    setVolume,
    toggleMute,
    
    // Display
    setScalingMode,
    toggleIntegerScaling,
    toggleFps,
    takeScreenshot,
    
    // Input
    setControllerType,
    loadKeyMappingPreset,
    updateKeyMapping,
    setInput,
    
    // ROM
    loadRom,
    loadRomFromPath,
    loadRomWithEdits,
    getCreatorRuntimeState,
    setCreatorSessionState,
    resolveCreatorRuntimeAction,
    unloadRom,
    compareWithOriginal,
    
    // Layout
    toggleFullscreen,
    
    // Cleanup
    shutdown,
  }), [
    state, config, status, saveStates, currentMappings, currentPreset, isInitialized,
    initialize, start, pause, resume, stop, reset, setSpeed, frameAdvance,
    saveState, loadState, deleteState, setSaveStateSlot, refreshSaveStates,
    setVolume, toggleMute, setScalingMode, toggleIntegerScaling, toggleFps, takeScreenshot,
    setControllerType, loadKeyMappingPreset, updateKeyMapping, setInput,
    loadRom, loadRomFromPath, loadRomWithEdits, getCreatorRuntimeState, setCreatorSessionState, resolveCreatorRuntimeAction, unloadRom, compareWithOriginal, toggleFullscreen,
    shutdown,
  ]);
}

export default useEmulator;
