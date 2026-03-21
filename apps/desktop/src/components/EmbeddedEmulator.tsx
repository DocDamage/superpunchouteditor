/**
 * EmbeddedEmulator Component
 * 
 * The main embedded emulator UI component that integrates Snes9x directly
 * into the Super Punch-Out!! editor. Provides seamless testing of ROM edits
 * without launching an external emulator.
 * 
 * Features:
 * - Three layout modes: Tab (full panel), Split (side-by-side), Overlay (PiP)
 * - Full emulator controls: play, pause, reset, save states, frame advance
 * - Audio control with volume slider and mute
 * - Input mapping with presets (WASD, Arrows, Fight stick)
 * - Quick actions: Load edited ROM, compare with original, screenshots, GIF recording
 * - Status bar with FPS counter, ROM info, and audio status
 * - Keyboard shortcuts for common actions
 * 
 * CONNECTED TO TAURI BACKEND - This component uses the useEmulator hook which
 * invokes real Tauri commands to control the embedded Snes9x emulator.
 */

import React, { useRef, useState, useCallback, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { EmulatorCanvas } from './EmulatorCanvas';
import { EmulatorControls } from './EmulatorControls';
import { InputMapper } from './InputMapper';
import { useEmulator, type SpeedMode, type ControllerType, type CreatorRuntimeState, type CreatorSessionState } from '../hooks/useEmulator';
import { useStore } from '../store/useStore';
import type { BoxerRosterEntry, CircuitType, IntroText } from '../types/roster';
import '../styles/emulator.css';
import { showToast } from './ToastContainer';

export type EmulatorLayout = 'tab' | 'split' | 'overlay';
export type RomSource = 'edited' | 'original';

interface CreatorDraftState {
  boxerId: number;
  name: string;
  circuit: CircuitType;
  unlockOrder: number;
  introTextId: number;
  introText: string;
}

interface CreatorCommitResponse {
  boxer: BoxerRosterEntry;
  intro_text_id: number;
  intro_text: string;
}

interface CreatorSessionValidationResponse {
  valid: boolean;
  status: number;
  error_code: number;
  message: string | null;
}

const runtimeStatusLabel = (status: number): string => {
  switch (status) {
    case 1:
      return 'Seeded';
    case 2:
      return 'Draft Ready';
    case 3:
      return 'Commit In Progress';
    case 4:
      return 'Commit Succeeded';
    case 5:
      return 'Commit Failed';
    case 6:
      return 'Portrait Workflow';
    case 7:
      return 'Cancelled';
    default:
      return 'Idle';
  }
};

const circuitTypeToByte = (circuit: CircuitType): number => {
  switch (circuit) {
    case 'Minor':
      return 0;
    case 'Major':
      return 1;
    case 'World':
      return 2;
    case 'Special':
      return 3;
    default:
      return 0;
  }
};

const circuitByteToType = (circuit: number): CircuitType => {
  switch (circuit) {
    case 1:
      return 'Major';
    case 2:
      return 'World';
    case 3:
      return 'Special';
    default:
      return 'Minor';
  }
};

const decodeCreatorAscii = (bytes: number[] | undefined, length: number | undefined): string => {
  if (!Array.isArray(bytes) || bytes.length === 0) {
    return '';
  }

  const limit = Math.max(0, Math.min(length ?? bytes.length, bytes.length));
  return String.fromCharCode(...bytes.slice(0, limit)).replace(/\0+$/g, '').trimEnd();
};

export interface EmbeddedEmulatorProps {
  /** Layout mode for the emulator */
  layout?: EmulatorLayout;
  /** Current edited ROM data */
  editedRomData?: Uint8Array | null;
  /** Original unmodified ROM data */
  originalRomData?: Uint8Array | null;
  /** ROM file path for loading with Tauri */
  romPath?: string | null;
  /** ROM name for display */
  romName?: string;
  /** Whether the emulator is visible */
  isOpen?: boolean;
  /** Callback when emulator requests close (for overlay mode) */
  onClose?: () => void;
  /** Callback when layout changes */
  onLayoutChange?: (layout: EmulatorLayout) => void;
  /** Optional className */
  className?: string;
  /** Optional style */
  style?: React.CSSProperties;
  /** Incrementing token that should auto-enter creator mode when it changes */
  autoEnterCreatorToken?: number | null;
  /** Optional creator session context shown in the monitor */
  creatorSessionContext?: {
    boxerId?: number;
    boxerName?: string;
    circuit?: 'Minor' | 'Major' | 'World' | 'Special';
    unlockOrder?: number;
    introTextId?: number;
    assetOwnerKey?: string;
  } | null;
  /** Optional callback to jump to the manifest boxer that owns portrait assets */
  onOpenAssetOwner?: (boxerKey: string) => void;
}

export const EmbeddedEmulator: React.FC<EmbeddedEmulatorProps> = ({
  layout = 'tab',
  editedRomData,
  originalRomData,
  romPath,
  romName = 'Unknown ROM',
  isOpen = true,
  onClose,
  onLayoutChange,
  className = '',
  style = {},
  autoEnterCreatorToken = null,
  creatorSessionContext = null,
  onOpenAssetOwner,
}) => {
  const { boxers } = useStore((state) => ({ boxers: state.boxers }));
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const lastAutoEnterTokenRef = useRef<number | null>(null);
  const lastHandledCreatorActionRef = useRef<string | null>(null);
  const creatorNameInputRef = useRef<HTMLInputElement>(null);
  const creatorUnlockInputRef = useRef<HTMLInputElement>(null);
  const creatorIntroInputRef = useRef<HTMLTextAreaElement>(null);
  const [showInputMapper, setShowInputMapper] = useState(false);
  const [currentRomSource, setCurrentRomSource] = useState<RomSource>('edited');
  const [isRecordingGif, setIsRecordingGif] = useState(false);
  const [recordingProgress, setRecordingProgress] = useState(0);
  const [screenshots, setScreenshots] = useState<string[]>([]);
  const [creatorState, setCreatorState] = useState<CreatorRuntimeState | null>(null);
  const [creatorActionBusy, setCreatorActionBusy] = useState(false);
  const [creatorDraft, setCreatorDraft] = useState<CreatorDraftState | null>(null);
  const [creatorDraftBusy, setCreatorDraftBusy] = useState(false);
  const [creatorDraftMessage, setCreatorDraftMessage] = useState<string | null>(null);
  const [creatorDraftError, setCreatorDraftError] = useState<string | null>(null);
  const [creatorSessionStatusOverride, setCreatorSessionStatusOverride] = useState<number | null>(null);
  const [creatorSessionErrorCodeOverride, setCreatorSessionErrorCodeOverride] = useState<number | null>(null);
  const [portraitOwnerKey, setPortraitOwnerKey] = useState<string>(creatorSessionContext?.assetOwnerKey ?? '');

  // Initialize emulator hook with Tauri backend integration
  const emulator = useEmulator({
    canvasRef,
    onFrame: (imageData) => {
      // Handle frame data if needed
    },
    onError: (error) => {
      console.error('Emulator error:', error);
    },
  });

  // Initialize emulator on mount
  useEffect(() => {
    const init = async () => {
      await emulator.initialize();
    };
    init();
    
    // Cleanup on unmount
    return () => {
      emulator.shutdown();
    };
  }, []);

  // Load ROM when data changes
  useEffect(() => {
    const loadRom = async () => {
      if (editedRomData && currentRomSource === 'edited') {
        await emulator.loadRom(editedRomData, `${romName} (Edited)`);
      } else if (originalRomData && currentRomSource === 'original') {
        await emulator.loadRom(originalRomData, `${romName} (Original)`);
      } else if (romPath && currentRomSource === 'edited') {
        await emulator.loadRomFromPath(romPath);
      }
    };
    loadRom();
  }, [editedRomData, originalRomData, romPath, currentRomSource, romName]);

  useEffect(() => {
    if (!isOpen) return;

    let cancelled = false;
    const pollCreatorState = async () => {
      if (!emulator.isInitialized) {
        if (!cancelled) setCreatorState(null);
        return;
      }

      const nextState = await emulator.getCreatorRuntimeState();
      if (!cancelled) {
        setCreatorState(nextState);
      }
    };

    void pollCreatorState();
    const intervalId = window.setInterval(() => {
      void pollCreatorState();
    }, emulator.state === 'running' ? 100 : 250);

    return () => {
      cancelled = true;
      window.clearInterval(intervalId);
    };
  }, [emulator.getCreatorRuntimeState, emulator.isInitialized, emulator.state, isOpen]);

  // Keyboard input handler for SNES controls
  useEffect(() => {
    const handleKeyDown = async (e: KeyboardEvent) => {
      if (emulator.state !== 'running') return;
      
      // Map keys to SNES buttons (standard WASD layout)
      let buttons = 0;
      switch (e.key.toLowerCase()) {
        case 'z': buttons |= 0x8000; break; // B button
        case 'a': buttons |= 0x4000; break; // Y button
        case 's': buttons |= 0x2000; break; // Select
        case 'enter': buttons |= 0x1000; break; // Start
        case 'arrowup': buttons |= 0x0800; break; // Up
        case 'arrowdown': buttons |= 0x0400; break; // Down
        case 'arrowleft': buttons |= 0x0200; break; // Left
        case 'arrowright': buttons |= 0x0100; break; // Right
        case 'x': buttons |= 0x0080; break; // A button
        case 'd': buttons |= 0x0040; break; // X button
        case 'q': buttons |= 0x0020; break; // L shoulder
        case 'w': buttons |= 0x0010; break; // R shoulder
      }
      
      if (buttons !== 0) {
        e.preventDefault();
        await emulator.setInput(buttons);
      }
    };
    
    const handleKeyUp = async () => {
      // Release all buttons
      await emulator.setInput(0);
    };
    
    window.addEventListener('keydown', handleKeyDown);
    window.addEventListener('keyup', handleKeyUp);
    
    return () => {
      window.removeEventListener('keydown', handleKeyDown);
      window.removeEventListener('keyup', handleKeyUp);
    };
  }, [emulator.state, emulator.setInput]);

  // Handle layout toggle
  const handleLayoutChange = useCallback((newLayout: EmulatorLayout) => {
    onLayoutChange?.(newLayout);
  }, [onLayoutChange]);

  // Load edited ROM
  const handleLoadEditedRom = useCallback(async () => {
    if (editedRomData) {
      setCurrentRomSource('edited');
      await emulator.loadRom(editedRomData, `${romName} (Edited)`);
    } else if (romPath) {
      setCurrentRomSource('edited');
      await emulator.loadRomFromPath(romPath);
    }
  }, [editedRomData, romPath, romName, emulator]);

  // Load original ROM
  const handleLoadOriginalRom = useCallback(async () => {
    if (originalRomData) {
      setCurrentRomSource('original');
      await emulator.loadRom(originalRomData, `${romName} (Original)`);
    }
  }, [originalRomData, romName, emulator]);

  // Swap between edited and original
  const handleSwapRom = useCallback(async () => {
    const newSource = currentRomSource === 'edited' ? 'original' : 'edited';
    setCurrentRomSource(newSource);
    
    if (newSource === 'edited') {
      if (editedRomData) {
        await emulator.loadRom(editedRomData, `${romName} (Edited)`);
      } else if (romPath) {
        await emulator.loadRomFromPath(romPath);
      }
    } else if (newSource === 'original' && originalRomData) {
      await emulator.loadRom(originalRomData, `${romName} (Original)`);
    }
  }, [currentRomSource, editedRomData, originalRomData, romPath, romName, emulator]);

  // Take screenshot
  const handleScreenshot = useCallback(() => {
    const screenshot = emulator.takeScreenshot();
    if (screenshot) {
      setScreenshots(prev => [screenshot, ...prev].slice(0, 10));
      
      // Download screenshot
      const link = document.createElement('a');
      link.download = `spo-screenshot-${Date.now()}.png`;
      link.href = screenshot;
      link.click();
    }
  }, [emulator]);

  // Record GIF (simulated - actual implementation would use gif.js or similar)
  const handleRecordGif = useCallback(() => {
    if (isRecordingGif) return;
    
    setIsRecordingGif(true);
    setRecordingProgress(0);
    
    // Simulate 3 second recording
    const duration = 3000;
    const interval = 100;
    let elapsed = 0;
    
    const timer = setInterval(() => {
      elapsed += interval;
      setRecordingProgress((elapsed / duration) * 100);
      
      if (elapsed >= duration) {
        clearInterval(timer);
        setIsRecordingGif(false);
        setRecordingProgress(0);
        
        // GIF export is not yet implemented — inform the user without blocking
        showToast('GIF recording is not yet implemented.', 'info');
      }
    }, interval);
  }, [isRecordingGif]);

  // Handle fullscreen
  const handleFullscreen = useCallback(() => {
    emulator.toggleFullscreen();
  }, [emulator]);

  // Get status indicator color
  const getStatusColor = (): string => {
    switch (emulator.state) {
      case 'running':
        return 'var(--success, #22c55e)';
      case 'paused':
        return 'var(--warning, #f59e0b)';
      case 'loading':
        return 'var(--info, #3b82f6)';
      default:
        return 'var(--text-muted, #64748b)';
    }
  };

  // Get FPS badge class
  const getFpsClass = (): string => {
    if (emulator.status.fps >= 55) return 'good';
    if (emulator.status.fps >= 45) return 'warning';
    return 'bad';
  };

  const describeCreatorAction = (action: number): string => {
    switch (action) {
      case 0x11:
        return 'Name Edit';
      case 0x12:
        return 'Circuit Edit';
      case 0x13:
        return 'Portrait Edit';
      case 0x14:
        return 'Commit';
      case 0xFF:
        return 'Exit';
      case 0x00:
        return 'Idle';
      default:
        return `0x${action.toString(16).toUpperCase().padStart(2, '0')}`;
    }
  };

  const describeCreatorPage = (page: number): string => {
    switch (page) {
      case 0:
        return 'Identity';
      case 1:
        return 'Circuit';
      case 2:
        return 'Portrait';
      case 3:
        return 'Finalize';
      default:
        return `Page ${page}`;
    }
  };

  const describeCreatorSummary = (page: number): string => {
    switch (page) {
      case 0:
        return 'Edit the active boxer metadata fields.';
      case 1:
        return 'Choose a circuit directly from the highlighted row.';
      case 2:
        return 'Portrait workflow is pinned to the current boxer slot.';
      case 3:
        return 'Write the current draft back into the ROM.';
      default:
        return 'Creator contract is active.';
    }
  };

  const formatHexByte = (value: number): string =>
    `0x${value.toString(16).toUpperCase().padStart(2, '0')}`;

  const sleep = (ms: number) => new Promise((resolve) => window.setTimeout(resolve, ms));

  const loadCreatorDraftFromRom = useCallback(async (boxerId: number): Promise<CreatorDraftState> => {
    const boxer = await invoke<BoxerRosterEntry>('get_boxer_roster_entry', { fighterId: boxerId });
    const intro = await invoke<IntroText>('get_intro_text', { textId: boxer.intro_text_id });
    return {
      boxerId,
      name: boxer.name,
      circuit: boxer.circuit,
      unlockOrder: boxer.unlock_order,
      introTextId: boxer.intro_text_id,
      introText: intro.text ?? '',
    };
  }, []);

  const refreshCreatorDraft = useCallback(async (boxerId: number | null | undefined) => {
    if (typeof boxerId !== 'number') {
      setCreatorDraft(null);
      setCreatorDraftMessage(null);
      setCreatorDraftError(null);
      setCreatorSessionStatusOverride(null);
      setCreatorSessionErrorCodeOverride(null);
      return;
    }

    try {
      setCreatorDraft(await loadCreatorDraftFromRom(boxerId));
      setCreatorDraftError(null);
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      setCreatorDraftError(message);
    }
  }, [loadCreatorDraftFromRom]);

  const pulseCreatorInput = useCallback(async (buttons: number) => {
    if (!emulator.status.romLoaded || creatorActionBusy) return;

    try {
      setCreatorActionBusy(true);

      if (emulator.state === 'stopped') {
        await emulator.start();
        await sleep(140);
      }

      await emulator.setInput(buttons);

      if (emulator.state === 'paused') {
        await emulator.frameAdvance();
      } else {
        await sleep(120);
      }

      await emulator.setInput(0);

      if (emulator.state === 'paused') {
        await emulator.frameAdvance();
      } else {
        await sleep(40);
      }

      const nextState = await emulator.getCreatorRuntimeState();
      setCreatorState(nextState);
    } catch (error) {
      console.error('Failed to send creator input:', error);
    } finally {
      setCreatorActionBusy(false);
    }
  }, [
    creatorActionBusy,
    emulator,
  ]);

  const buildCreatorCommitSession = useCallback((): CreatorSessionState | null => {
    if (!creatorDraft) {
      return null;
    }

    const runtimeOwnsSession = Boolean(
      creatorState?.session_present && creatorState.session_boxer_id === creatorDraft.boxerId
    );

    return {
      boxer_id: creatorDraft.boxerId,
      circuit: runtimeOwnsSession
        ? creatorState!.session_circuit
        : circuitTypeToByte(creatorDraft.circuit),
      unlock_order: runtimeOwnsSession
        ? creatorState!.session_unlock_order
        : creatorDraft.unlockOrder,
      intro_text_id: runtimeOwnsSession
        ? creatorState!.session_intro_text_id
        : creatorDraft.introTextId,
      intro_text: runtimeOwnsSession
        ? decodeCreatorAscii(creatorState!.intro_bytes, creatorState!.intro_length)
        : creatorDraft.introText,
      name_text: runtimeOwnsSession
        ? decodeCreatorAscii(creatorState!.name_bytes, creatorState!.name_length)
        : creatorDraft.name,
      status: 3,
      error_code: 0,
    };
  }, [creatorDraft, creatorState]);

  const commitCreatorDraft = useCallback(async () => {
    const session = buildCreatorCommitSession();
    if (!session || !creatorDraft) {
      setCreatorDraftError('No creator session target is loaded.');
      return;
    }

    try {
      setCreatorDraftBusy(true);
      setCreatorDraftError(null);
      setCreatorDraftMessage('Writing creator draft back to ROM...');
      setCreatorSessionStatusOverride(3);
      setCreatorSessionErrorCodeOverride(0);

      const validation = await invoke<CreatorSessionValidationResponse>('validate_creator_session', { session });
      if (!validation.valid) {
        setCreatorDraftError(validation.message ?? 'Creator session validation failed.');
        setCreatorDraftMessage(null);
        setCreatorSessionStatusOverride(validation.status);
        setCreatorSessionErrorCodeOverride(validation.error_code);
        return;
      }

      const committed = await invoke<CreatorCommitResponse>('commit_creator_session', { session });

      const updatedRom = new Uint8Array(await invoke<number[]>('get_loaded_rom_image'));
      setCurrentRomSource('edited');
      await emulator.stop();
      await emulator.loadRom(updatedRom, `${romName} (Edited)`);
      await emulator.start();
      await sleep(140);
      await emulator.setInput(0x2000 | 0x1000 | 0x0020 | 0x0010);
      await sleep(120);
      await emulator.setInput(0);
      await sleep(40);
      setCreatorDraft({
        boxerId: committed.boxer.boxer_id ?? committed.boxer.fighter_id ?? creatorDraft.boxerId,
        name: committed.boxer.name,
        circuit: committed.boxer.circuit,
        unlockOrder: committed.boxer.unlock_order,
        introTextId: committed.intro_text_id,
        introText: committed.intro_text,
      });
      setCreatorState(await emulator.getCreatorRuntimeState());
      setCreatorSessionStatusOverride(4);
      setCreatorSessionErrorCodeOverride(0);
      await refreshCreatorDraft(creatorDraft.boxerId);
      setCreatorDraftMessage(`Committed slot #${creatorDraft.boxerId} and reloaded the edited ROM.`);
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      setCreatorDraftError(message);
      setCreatorDraftMessage(null);
      setCreatorSessionStatusOverride(5);
      setCreatorSessionErrorCodeOverride(1);
    } finally {
      setCreatorDraftBusy(false);
    }
  }, [buildCreatorCommitSession, creatorDraft, emulator, refreshCreatorDraft, romName]);

  const cancelCreatorDraft = useCallback(async () => {
    if (!creatorDraft) {
      setCreatorDraftError('No creator session target is loaded.');
      return;
    }

    try {
      setCreatorDraftBusy(true);
      setCreatorDraftError(null);
      setCreatorSessionErrorCodeOverride(0);

      const revertedDraft = await loadCreatorDraftFromRom(creatorDraft.boxerId);
      setCreatorDraft(revertedDraft);
      await emulator.setCreatorSessionState({
        boxer_id: revertedDraft.boxerId,
        circuit: circuitTypeToByte(revertedDraft.circuit),
        unlock_order: revertedDraft.unlockOrder,
        intro_text_id: revertedDraft.introTextId,
        intro_text: revertedDraft.introText,
        name_text: revertedDraft.name,
        status: 7,
        error_code: 0,
      });
      setCreatorState(await emulator.getCreatorRuntimeState());
      setCreatorSessionStatusOverride(7);
      setCreatorSessionErrorCodeOverride(0);
      setCreatorDraftMessage(`Reverted slot #${revertedDraft.boxerId} to ROM values.`);
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      setCreatorDraftError(message);
      setCreatorDraftMessage(null);
      setCreatorSessionStatusOverride(5);
      setCreatorSessionErrorCodeOverride(1);
    } finally {
      setCreatorDraftBusy(false);
    }
  }, [creatorDraft, emulator, loadCreatorDraftFromRom]);

  const resolveCreatorRuntimeAction = useCallback(async () => {
    try {
      setCreatorDraftBusy(true);
      const resolution = await emulator.resolveCreatorRuntimeAction();
      setCreatorState(resolution.runtime_state);

      if (resolution.runtime_state.session_present) {
        await refreshCreatorDraft(resolution.runtime_state.session_boxer_id);
      }

      if (resolution.message) {
        setCreatorDraftMessage(resolution.message);
      }

      setCreatorDraftError(
        resolution.runtime_state.session_error_code !== 0
          ? resolution.message ?? 'Creator runtime action failed.'
          : null
      );
      setCreatorSessionStatusOverride(resolution.runtime_state.session_status);
      setCreatorSessionErrorCodeOverride(resolution.runtime_state.session_error_code);
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      setCreatorDraftError(message);
      setCreatorDraftMessage(null);
      setCreatorSessionStatusOverride(5);
      setCreatorSessionErrorCodeOverride(1);
    } finally {
      setCreatorDraftBusy(false);
    }
  }, [emulator, refreshCreatorDraft]);

  const creatorTargetBoxerId = creatorState?.session_present
    ? creatorState.session_boxer_id
    : creatorSessionContext?.boxerId ?? null;

  useEffect(() => {
    void refreshCreatorDraft(creatorTargetBoxerId);
  }, [creatorTargetBoxerId, refreshCreatorDraft]);

  useEffect(() => {
    setCreatorSessionStatusOverride(null);
    setCreatorSessionErrorCodeOverride(null);
  }, [creatorTargetBoxerId]);

  useEffect(() => {
    if (!creatorState?.session_present) {
      return;
    }

    setCreatorDraft((current) => {
      if (!current || current.boxerId !== creatorState.session_boxer_id) {
        return current;
      }

      const nextCircuit = circuitByteToType(creatorState.session_circuit);
      if (current.circuit === nextCircuit) {
        return current;
      }

      return {
        ...current,
        circuit: nextCircuit,
      };
    });
  }, [
    creatorState?.session_present,
    creatorState?.session_boxer_id,
    creatorState?.session_circuit,
  ]);

  useEffect(() => {
    if (!creatorState?.session_present) {
      return;
    }

    const runtimeIntro = decodeCreatorAscii(creatorState.intro_bytes, creatorState.intro_length);
    setCreatorDraft((current) => {
      if (!current || current.boxerId !== creatorState.session_boxer_id) {
        return current;
      }

      if (current.introText === runtimeIntro) {
        return current;
      }

      return {
        ...current,
        introText: runtimeIntro,
      };
    });
  }, [
    creatorState?.session_present,
    creatorState?.session_boxer_id,
    creatorState?.intro_bytes,
    creatorState?.intro_length,
  ]);

  useEffect(() => {
    if (!creatorState?.session_present) {
      return;
    }

    const runtimeName = decodeCreatorAscii(creatorState.name_bytes, creatorState.name_length);
    setCreatorDraft((current) => {
      if (!current || current.boxerId !== creatorState.session_boxer_id) {
        return current;
      }

      if (current.name === runtimeName) {
        return current;
      }

      return {
        ...current,
        name: runtimeName,
      };
    });
  }, [
    creatorState?.session_present,
    creatorState?.session_boxer_id,
    creatorState?.name_bytes,
    creatorState?.name_length,
  ]);

  useEffect(() => {
    if (!creatorState?.session_present) {
      return;
    }

    setCreatorDraft((current) => {
      if (!current || current.boxerId !== creatorState.session_boxer_id) {
        return current;
      }

      if (current.unlockOrder === creatorState.session_unlock_order) {
        return current;
      }

      return {
        ...current,
        unlockOrder: creatorState.session_unlock_order,
      };
    });
  }, [
    creatorState?.session_present,
    creatorState?.session_boxer_id,
    creatorState?.session_unlock_order,
  ]);

  useEffect(() => {
    if (!creatorState?.session_present) {
      return;
    }

    setCreatorDraft((current) => {
      if (!current || current.boxerId !== creatorState.session_boxer_id) {
        return current;
      }

      if (current.introTextId === creatorState.session_intro_text_id) {
        return current;
      }

      return {
        ...current,
        introTextId: creatorState.session_intro_text_id,
      };
    });

    let cancelled = false;
    const loadRuntimeIntroText = async () => {
      try {
        const intro = await invoke<IntroText>('get_intro_text', {
          textId: creatorState.session_intro_text_id,
        });
        if (cancelled) {
          return;
        }

        setCreatorDraft((current) => {
          if (
            !current
            || current.boxerId !== creatorState.session_boxer_id
            || current.introTextId !== creatorState.session_intro_text_id
          ) {
            return current;
          }

          const nextText = intro.text ?? '';
          if (current.introText === nextText) {
            return current;
          }

          return {
            ...current,
            introText: nextText,
          };
        });
      } catch (error) {
        console.error('Failed to load runtime intro text:', error);
      }
    };

    void loadRuntimeIntroText();
    return () => {
      cancelled = true;
    };
  }, [
    creatorState?.session_present,
    creatorState?.session_boxer_id,
    creatorState?.session_intro_text_id,
  ]);

  const focusCreatorIdentityField = useCallback((cursor: number) => {
    switch (cursor) {
      case 0:
        creatorNameInputRef.current?.focus();
        setCreatorDraftMessage('Name field armed from creator runtime.');
        break;
      case 1:
        creatorIntroInputRef.current?.focus();
        setCreatorDraftMessage('Intro quote field armed from creator runtime.');
        break;
      case 2:
        creatorUnlockInputRef.current?.focus();
        setCreatorDraftMessage('Unlock order field armed from creator runtime.');
        break;
      case 3:
        setCreatorDraftMessage('Intro text slot armed from creator runtime.');
        break;
      default:
        setCreatorDraftMessage('Identity page active. Edit the draft fields below.');
        break;
    }
  }, []);

  useEffect(() => {
    if (!creatorState || creatorState.action === 0) return;

    const actionKey = [
      creatorState.action,
      creatorState.render_page,
      creatorState.render_cursor,
      creatorState.render_revision,
    ].join(':');
    if (lastHandledCreatorActionRef.current === actionKey) {
      return;
    }
    lastHandledCreatorActionRef.current = actionKey;

    if (creatorDraftBusy) {
      return;
    }

    switch (creatorState.action) {
      case 0x11:
        setCreatorSessionStatusOverride(null);
        setCreatorSessionErrorCodeOverride(null);
        if (creatorState.name_edit_active) {
          setCreatorDraftMessage(
            `Name edit active at character ${creatorState.name_cursor + 1}. Use left/right to move and up/down to change the current character.`
          );
        } else if (creatorState.render_page === 0 && creatorState.render_cursor === 2) {
          setCreatorDraftMessage(`Unlock order updated in creator runtime to #${creatorState.session_unlock_order}.`);
        } else {
          focusCreatorIdentityField(creatorState.render_cursor);
        }
        break;
      case 0x12: {
        setCreatorSessionStatusOverride(null);
        setCreatorSessionErrorCodeOverride(null);
        const nextCircuit = circuitByteToType(creatorState.session_circuit);
        setCreatorDraftMessage(`Circuit selection updated in creator runtime to ${nextCircuit}.`);
        break;
      }
      case 0x15:
        setCreatorSessionStatusOverride(null);
        setCreatorSessionErrorCodeOverride(null);
        if (creatorState.intro_edit_active) {
          setCreatorDraftMessage(
            `Intro quote edit active at character ${creatorState.intro_cursor + 1}. Use left/right to move and up/down to change the current character.`
          );
        } else if (creatorState.render_page === 0 && creatorState.render_cursor === 3) {
          setCreatorDraftMessage(`Intro text slot updated in creator runtime to #${creatorState.session_intro_text_id}.`);
        } else {
          focusCreatorIdentityField(creatorState.render_cursor);
        }
        break;
      case 0x13:
        setCreatorSessionStatusOverride(6);
        setCreatorSessionErrorCodeOverride(0);
        setCreatorDraftMessage('Portrait slot is selected. Use the dedicated asset owner below, or retarget this session if you want to borrow another manifest boxer\'s graphic assets.');
        break;
      case 0x14:
        void resolveCreatorRuntimeAction();
        break;
      case 0x16:
        void resolveCreatorRuntimeAction();
        break;
      case 0xFF:
        setCreatorDraftMessage('Creator mode exited. Draft values are still preserved here.');
        break;
      default:
        break;
    }
  }, [creatorDraftBusy, creatorState, focusCreatorIdentityField, resolveCreatorRuntimeAction]);

  const creatorButtonsDisabled = !emulator.status.romLoaded || creatorActionBusy || creatorDraftBusy;
  const creatorPageLabel = creatorState ? describeCreatorPage(creatorState.render_page) : 'Idle';
  const runtimeSessionCircuit = creatorState?.session_present
    ? circuitByteToType(creatorState.session_circuit)
    : null;
  const creatorSessionLabel = creatorDraft?.name
    ? `${creatorDraft.name}${typeof creatorDraft.boxerId === 'number' ? ` (#${creatorDraft.boxerId})` : ''}`
    : creatorState?.session_present
      ? `Runtime Session (#${creatorState.session_boxer_id})`
    : creatorSessionContext?.boxerName
      ? `${creatorSessionContext.boxerName}${typeof creatorSessionContext.boxerId === 'number' ? ` (#${creatorSessionContext.boxerId})` : ''}`
      : null;
  const creatorMenuRows = creatorState
    ? (() => {
        switch (creatorState.render_page) {
          case 0:
            return [
              {
                row: creatorState.render_rows[0],
                index: 0,
                label: 'Name',
                detail: creatorDraft
                  ? `${creatorDraft.name || '(blank)'}${creatorState.name_edit_active ? ` [char ${creatorState.name_cursor + 1}]` : ''}`
                  : 'No target boxer selected',
                selected: creatorState.render_cursor === 0,
              },
              {
                row: creatorState.render_rows[1],
                index: 1,
                label: 'Intro Quote',
                detail: creatorDraft
                  ? `${creatorDraft.introText?.trim() || 'Empty'}${creatorState.intro_edit_active ? ` [char ${creatorState.intro_cursor + 1}]` : ''}`
                  : 'No intro text loaded',
                selected: creatorState.render_cursor === 1,
              },
              {
                row: creatorState.render_rows[2],
                index: 2,
                label: 'Unlock Order',
                detail: creatorDraft ? `#${creatorState?.session_present ? creatorState.session_unlock_order : creatorDraft.unlockOrder}` : 'n/a',
                selected: creatorState.render_cursor === 2,
              },
              {
                row: creatorState.render_rows[3],
                index: 3,
                label: 'Intro Slot',
                detail: creatorDraft
                  ? `ID #${creatorState?.session_present ? creatorState.session_intro_text_id : creatorDraft.introTextId}`
                  : 'No intro slot selected',
                selected: creatorState.render_cursor === 3,
              },
            ];
          case 1: {
            const circuitOptions: CircuitType[] = ['Minor', 'Major', 'World', 'Special'];
            return creatorState.render_rows.map((row, index) => ({
              row,
              index,
              label: `${circuitOptions[index] ?? 'Unknown'} Circuit`,
              detail: (runtimeSessionCircuit ?? creatorDraft?.circuit) === circuitOptions[index]
                ? 'Current draft selection'
                : 'Press A on the highlighted row to select',
              selected: creatorState.render_cursor === index,
            }));
          }
          case 2:
            return [
              {
                row: creatorState.render_rows[0],
                index: 0,
                label: 'Portrait Assets',
                detail: portraitOwnerKey
                  ? `Import PNGs through Graphic Assets for ${boxers.find((boxer) => boxer.key === portraitOwnerKey)?.name ?? portraitOwnerKey}`
                  : 'Choose an asset owner, then import PNGs through Graphic Assets',
                selected: creatorState.render_cursor === 0,
              },
              {
                row: creatorState.render_rows[1],
                index: 1,
                label: 'Asset Owner',
                detail: portraitOwnerKey
                  ? boxers.find((boxer) => boxer.key === portraitOwnerKey)?.name ?? portraitOwnerKey
                  : 'No asset owner selected',
                selected: creatorState.render_cursor === 1,
              },
              {
                row: creatorState.render_rows[2],
                index: 2,
                label: 'Current Draft',
                detail: creatorDraft?.name || 'No draft loaded',
                selected: creatorState.render_cursor === 2,
              },
              {
                row: creatorState.render_rows[3],
                index: 3,
                label: 'Workflow',
                detail: 'Open the selected owner in the Editor tab to reach Graphic Assets',
                selected: creatorState.render_cursor === 3,
              },
            ];
          case 3:
          default:
            return [
              {
                row: creatorState.render_rows[0],
                index: 0,
                label: 'Target Slot',
                detail: creatorDraft ? `#${creatorDraft.boxerId}` : 'No active session target',
                selected: creatorState.render_cursor === 0,
              },
              {
                row: creatorState.render_rows[1],
                index: 1,
                label: 'Commit Draft',
                detail: creatorDraftBusy ? 'Writing changes...' : 'Press A to write the current draft to ROM',
                selected: creatorState.render_cursor === 1,
              },
              {
                row: creatorState.render_rows[2],
                index: 2,
                label: 'Cancel Changes',
                detail: creatorDraftBusy ? 'Reloading ROM values...' : 'Press A to restore the draft from current ROM data',
                selected: creatorState.render_cursor === 2,
              },
              {
                row: creatorState.render_rows[3],
                index: 3,
                label: 'Session Status',
                detail: creatorDraftMessage || 'Press B to leave creator mode',
                selected: creatorState.render_cursor === 3,
              },
            ];
        }
      })()
    : [];

  useEffect(() => {
    setPortraitOwnerKey(creatorSessionContext?.assetOwnerKey ?? '');
  }, [creatorSessionContext?.boxerId, creatorSessionContext?.assetOwnerKey]);

  const portraitOwner = boxers.find((boxer) => boxer.key === portraitOwnerKey) ?? null;

  useEffect(() => {
    if (!emulator.isInitialized || !emulator.status.romLoaded) {
      return;
    }

    const runtimeOwnsSession = Boolean(
      creatorDraft
      && creatorState?.session_present
      && creatorState.session_boxer_id === creatorDraft.boxerId
    );

    const nextSession: CreatorSessionState | null = creatorDraft
        ? {
            boxer_id: creatorDraft.boxerId,
          circuit: runtimeOwnsSession
            ? creatorState!.session_circuit
            : circuitTypeToByte(creatorDraft.circuit),
          unlock_order: runtimeOwnsSession
            ? creatorState!.session_unlock_order
            : creatorDraft.unlockOrder,
          intro_text_id: runtimeOwnsSession
            ? creatorState!.session_intro_text_id
            : creatorDraft.introTextId,
          intro_text: runtimeOwnsSession && creatorState!.intro_edit_active
            ? decodeCreatorAscii(creatorState!.intro_bytes, creatorState!.intro_length)
            : creatorDraft.introText,
          name_text: runtimeOwnsSession
            ? decodeCreatorAscii(creatorState!.name_bytes, creatorState!.name_length)
            : creatorDraft.name,
          status: creatorSessionStatusOverride
            ?? (creatorDraftBusy
              ? 3
              : creatorDraftError
                ? 5
                : runtimeOwnsSession
                  ? creatorState!.session_status
                  : 2),
          error_code: creatorSessionErrorCodeOverride
            ?? (creatorDraftError
              ? 1
              : runtimeOwnsSession
                ? creatorState!.session_error_code
                : 0),
        }
      : typeof creatorSessionContext?.boxerId === 'number'
        ? {
            boxer_id: creatorSessionContext.boxerId,
            circuit: circuitTypeToByte(creatorSessionContext.circuit ?? 'Minor'),
            unlock_order: creatorSessionContext.unlockOrder ?? 0,
            intro_text_id: creatorSessionContext.introTextId ?? creatorSessionContext.boxerId,
            intro_text: '',
            name_text: creatorSessionContext.boxerName ?? '',
            status: creatorSessionStatusOverride ?? 1,
            error_code: creatorSessionErrorCodeOverride ?? 0,
          }
        : null;

    void emulator.setCreatorSessionState(nextSession).catch((error) => {
      console.error('Failed to seed creator session state:', error);
    });
  }, [
    creatorState,
    creatorDraft,
    creatorDraftBusy,
    creatorDraftError,
    creatorSessionErrorCodeOverride,
    creatorSessionStatusOverride,
    creatorSessionContext,
    emulator,
  ]);

  useEffect(() => {
    if (
      autoEnterCreatorToken === null
      || autoEnterCreatorToken === lastAutoEnterTokenRef.current
      || !emulator.status.romLoaded
      || creatorActionBusy
    ) {
      return;
    }

    lastAutoEnterTokenRef.current = autoEnterCreatorToken;
    void pulseCreatorInput(0x2000 | 0x1000 | 0x0020 | 0x0010);
  }, [
    autoEnterCreatorToken,
    creatorActionBusy,
    emulator.status.romLoaded,
    pulseCreatorInput,
  ]);

  // If overlay mode and not open, don't render
  if (layout === 'overlay' && !isOpen) return null;

  return (
    <div
      className={`emulator-container mode-${layout} ${className}`.trim()}
      style={style}
    >
      {/* Header */}
      <div className="emulator-header">
        <div className="emulator-header-title">
          <span className="icon">🎮</span>
          <span>Embedded Emulator</span>
          <span
            style={{
              width: 8,
              height: 8,
              borderRadius: '50%',
              backgroundColor: getStatusColor(),
              marginLeft: 8,
            }}
          />
          {!emulator.isInitialized && (
            <span style={{ marginLeft: 8, color: 'var(--warning, #f59e0b)', fontSize: 12 }}>
              (Not Initialized)
            </span>
          )}
        </div>

        <div className="emulator-header-actions">
          {/* Layout Toggle */}
          <select
            value={layout}
            onChange={(e) => handleLayoutChange(e.target.value as EmulatorLayout)}
            className="emulator-header-btn"
            style={{ padding: '0.375rem' }}
          >
            <option value="tab">Tab Mode</option>
            <option value="split">Split Mode</option>
            <option value="overlay">Overlay</option>
          </select>

          {/* Fullscreen */}
          <button
            onClick={handleFullscreen}
            className="emulator-header-btn"
            title="Toggle Fullscreen"
          >
            ⛶
          </button>

          {/* Close button for overlay mode */}
          {layout === 'overlay' && onClose && (
            <button
              onClick={onClose}
              className="emulator-header-btn"
              title="Close"
            >
              ×
            </button>
          )}
        </div>
      </div>

      {/* Display Area */}
      <div className="emulator-display">
        {emulator.status.romLoaded ? (
          <>
            <div className="emulator-display-content">
              {/* Comparison indicator */}
              {currentRomSource === 'original' && (
                <div className="comparison-mode-indicator">
                  Viewing Original ROM
                </div>
              )}

              {/* Recording indicator */}
              {isRecordingGif && (
                <div
                  style={{
                    position: 'absolute',
                    top: '1rem',
                    right: '1rem',
                    zIndex: 10,
                  }}
                >
                  <div className="recording-indicator">
                    <span className="recording-dot" />
                    <span>Recording {Math.round(recordingProgress)}%</span>
                  </div>
                </div>
              )}

              <EmulatorCanvas
                canvasRef={canvasRef}
                width={256}
                height={224}
                integerScaling={emulator.config.integerScaling}
                scalingMode={emulator.config.scalingMode}
                onCanvasReady={(canvas) => {
                  // Canvas is ready for rendering
                }}
              />
            </div>

            {/* Status Bar */}
            <div className="emulator-status-bar">
              <div className="emulator-status-left">
                <div className="emulator-status-item">
                  <span>Status:</span>
                  <span
                    className="emulator-status-value"
                    style={{
                      textTransform: 'capitalize',
                      color: getStatusColor(),
                    }}
                  >
                    {emulator.state}
                  </span>
                </div>

                {emulator.config.showFps && (
                  <div className="emulator-status-item">
                    <span>FPS:</span>
                    <span className={`emulator-fps-badge ${getFpsClass()}`}>
                      {emulator.status.fps}
                    </span>
                  </div>
                )}

                <div className="emulator-status-item">
                  <span>ROM:</span>
                  <span className="emulator-status-value" style={{ maxWidth: 150, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
                    {emulator.status.romLoaded}
                  </span>
                </div>

                <div className="emulator-status-item">
                  <span>Slot:</span>
                  <span className="emulator-status-value">
                    {emulator.config.saveStateSlot}
                  </span>
                </div>

                <div className={`emulator-status-item ${emulator.config.muted ? 'warning' : 'active'}`}>
                  <span>Audio:</span>
                  <span className="emulator-status-value">
                    {emulator.config.muted ? 'Muted' : `${Math.round(emulator.config.volume * 100)}%`}
                  </span>
                </div>
              </div>

              {emulator.status.audioEnabled && (
                <div className="emulator-status-item">
                  <span>🎧</span>
                </div>
              )}
            </div>

            <div
              style={{
                marginTop: '0.75rem',
                border: '1px solid var(--border, rgba(255,255,255,0.12))',
                borderRadius: '10px',
                padding: '0.875rem 1rem',
                background: 'var(--bg-panel, rgba(15,23,42,0.8))',
              }}
            >
              <div
                style={{
                  display: 'flex',
                  justifyContent: 'space-between',
                  alignItems: 'center',
                  gap: '1rem',
                  marginBottom: '0.75rem',
                  flexWrap: 'wrap',
                }}
              >
                <div>
                  <div style={{ fontWeight: 700, fontSize: '0.95rem' }}>Creator Runtime Monitor</div>
                  {creatorSessionLabel && (
                    <div style={{ color: 'var(--text-muted, #94a3b8)', fontSize: '0.78rem', marginTop: '0.18rem' }}>
                      Session target: {creatorSessionLabel}
                    </div>
                  )}
                </div>
                <div
                  style={{
                    padding: '0.2rem 0.55rem',
                    borderRadius: '999px',
                    background: creatorState?.active ? 'rgba(34,197,94,0.18)' : 'rgba(148,163,184,0.14)',
                    color: creatorState?.active ? 'var(--success, #22c55e)' : 'var(--text-muted, #94a3b8)',
                    fontSize: '0.78rem',
                    fontWeight: 700,
                  }}
                >
                  {creatorState?.active ? 'Creator Active' : 'Creator Idle'}
                </div>
              </div>

              {creatorState ? (
                <>
                  {(creatorSessionContext || creatorDraft || creatorState?.session_present) && (
                    <div
                      style={{
                        display: 'grid',
                        gridTemplateColumns: 'repeat(auto-fit, minmax(150px, 1fr))',
                        gap: '0.6rem',
                        marginBottom: '0.75rem',
                        padding: '0.75rem',
                        borderRadius: '10px',
                        background: 'rgba(15,23,42,0.36)',
                        border: '1px solid var(--border, rgba(255,255,255,0.12))',
                      }}
                    >
                      {typeof (creatorDraft?.boxerId ?? creatorState?.session_boxer_id ?? creatorSessionContext?.boxerId) === 'number' && (
                        <div><strong>Slot:</strong> #{creatorDraft?.boxerId ?? creatorState?.session_boxer_id ?? creatorSessionContext?.boxerId}</div>
                      )}
                      {(creatorDraft?.circuit ?? runtimeSessionCircuit ?? creatorSessionContext?.circuit) && (
                        <div><strong>Circuit:</strong> {creatorDraft?.circuit ?? runtimeSessionCircuit ?? creatorSessionContext?.circuit}</div>
                      )}
                      {typeof (creatorDraft?.unlockOrder ?? creatorState?.session_unlock_order ?? creatorSessionContext?.unlockOrder) === 'number' && (
                        <div><strong>Unlock Order:</strong> {creatorDraft?.unlockOrder ?? creatorState?.session_unlock_order ?? creatorSessionContext?.unlockOrder}</div>
                      )}
                      {typeof (creatorDraft?.introTextId ?? creatorState?.session_intro_text_id ?? creatorSessionContext?.introTextId) === 'number' && (
                        <div><strong>Intro Text ID:</strong> {creatorDraft?.introTextId ?? creatorState?.session_intro_text_id ?? creatorSessionContext?.introTextId}</div>
                      )}
                      {portraitOwner && (
                        <div><strong>Portrait Owner:</strong> {portraitOwner.name}</div>
                      )}
                      <div><strong>Runtime Status:</strong> {runtimeStatusLabel(creatorState?.session_status ?? 0)}</div>
                    </div>
                  )}

                  <div
                    style={{
                      display: 'grid',
                      gridTemplateColumns: 'repeat(auto-fit, minmax(140px, 1fr))',
                      gap: '0.6rem',
                      marginBottom: '0.75rem',
                    }}
                  >
                    <div><strong>Page:</strong> {creatorPageLabel}</div>
                    <div><strong>Cursor:</strong> {creatorState.cursor}</div>
                    <div><strong>Action:</strong> {describeCreatorAction(creatorState.action)}</div>
                    <div><strong>Revision:</strong> {creatorState.render_revision}</div>
                    <div><strong>Dirty:</strong> {creatorState.dirty ? 'Yes' : 'No'}</div>
                    <div><strong>Visible:</strong> {creatorState.render_visible ? 'Yes' : 'No'}</div>
                    <div><strong>Session Status:</strong> {runtimeStatusLabel(creatorState.session_status)}</div>
                  </div>

                  <div
                    style={{
                      border: '1px solid var(--border, rgba(255,255,255,0.12))',
                      borderRadius: '10px',
                      padding: '0.85rem',
                      background: 'rgba(15,23,42,0.36)',
                      marginBottom: '0.85rem',
                    }}
                  >
                    <div
                      style={{
                        display: 'flex',
                        justifyContent: 'space-between',
                        alignItems: 'center',
                        gap: '0.75rem',
                        marginBottom: '0.65rem',
                        flexWrap: 'wrap',
                      }}
                    >
                      <div>
                        <div style={{ fontWeight: 700, fontSize: '0.92rem' }}>
                          {creatorPageLabel} Menu Preview
                        </div>
                        <div style={{ color: 'var(--text-muted, #94a3b8)', fontSize: '0.8rem' }}>
                          {describeCreatorSummary(creatorState.render_page)}
                        </div>
                      </div>
                      <div style={{ color: 'var(--text-muted, #94a3b8)', fontSize: '0.78rem' }}>
                        Cursor {creatorState.render_cursor + 1} of {creatorMenuRows.length || 4}
                      </div>
                    </div>

                    <div
                      style={{
                        display: 'grid',
                        gap: '0.45rem',
                      }}
                    >
                      {creatorMenuRows.map((row) => (
                        <div
                          key={`${row.index}-${row.row}`}
                          style={{
                            display: 'flex',
                            justifyContent: 'space-between',
                            alignItems: 'center',
                            gap: '0.75rem',
                            padding: '0.55rem 0.7rem',
                            borderRadius: '8px',
                            background: row.selected ? 'rgba(59,130,246,0.16)' : 'rgba(148,163,184,0.08)',
                            border: row.selected ? '1px solid rgba(59,130,246,0.45)' : '1px solid transparent',
                          }}
                        >
                          <div>
                            <div style={{ fontWeight: 600, fontSize: '0.84rem' }}>{row.label}</div>
                            <div style={{ color: 'var(--text-muted, #94a3b8)', fontSize: '0.76rem' }}>
                              {row.detail}
                            </div>
                          </div>
                          <div
                            style={{
                              color: row.selected ? 'var(--info, #60a5fa)' : 'var(--text-muted, #94a3b8)',
                              fontSize: '0.76rem',
                              fontWeight: 700,
                            }}
                          >
                            {row.selected ? 'Selected' : `Slot ${row.index + 1}`}
                          </div>
                        </div>
                      ))}
                    </div>
                  </div>

                  <div
                    style={{
                      border: '1px solid var(--border, rgba(255,255,255,0.12))',
                      borderRadius: '10px',
                      padding: '0.85rem',
                      background: 'rgba(15,23,42,0.36)',
                      marginBottom: '0.85rem',
                    }}
                  >
                    <div
                      style={{
                        display: 'flex',
                        justifyContent: 'space-between',
                        alignItems: 'center',
                        gap: '0.75rem',
                        marginBottom: '0.75rem',
                        flexWrap: 'wrap',
                      }}
                    >
                      <div>
                        <div style={{ fontWeight: 700, fontSize: '0.92rem' }}>Creator Draft Editor</div>
                        <div style={{ color: 'var(--text-muted, #94a3b8)', fontSize: '0.8rem' }}>
                          The ROM-side hook chooses the page and action. This panel owns the editable draft and commit.
                        </div>
                      </div>
                      <div style={{ color: 'var(--text-muted, #94a3b8)', fontSize: '0.78rem' }}>
                        {creatorDraftBusy ? 'Committing...' : creatorDraft ? `Target slot #${creatorDraft.boxerId}` : 'No active creator target'}
                      </div>
                    </div>

                    {creatorDraft ? (
                      <>
                        <div
                          style={{
                            display: 'grid',
                            gridTemplateColumns: 'repeat(auto-fit, minmax(220px, 1fr))',
                            gap: '0.75rem',
                            marginBottom: '0.75rem',
                          }}
                        >
                          <label style={{ display: 'grid', gap: '0.35rem' }}>
                            <span style={{ fontSize: '0.78rem', fontWeight: 700 }}>Name</span>
                            <input
                              ref={creatorNameInputRef}
                              value={creatorDraft.name}
                              disabled={creatorDraftBusy}
                              onChange={(event) => {
                                setCreatorDraft((current) => current ? { ...current, name: event.target.value } : current);
                                setCreatorDraftError(null);
                              }}
                              style={{
                                padding: '0.6rem 0.7rem',
                                borderRadius: '8px',
                                border: '1px solid var(--border, rgba(255,255,255,0.12))',
                                background: 'rgba(15,23,42,0.55)',
                                color: 'var(--text-primary, #e2e8f0)',
                              }}
                            />
                          </label>

                          <label style={{ display: 'grid', gap: '0.35rem' }}>
                            <span style={{ fontSize: '0.78rem', fontWeight: 700 }}>Unlock Order</span>
                            <input
                              ref={creatorUnlockInputRef}
                              type="number"
                              min={0}
                              max={255}
                              value={creatorDraft.unlockOrder}
                              disabled={creatorDraftBusy}
                              onChange={(event) => {
                                const parsed = Number.parseInt(event.target.value, 10);
                                setCreatorDraft((current) => current ? {
                                  ...current,
                                  unlockOrder: Number.isNaN(parsed) ? 0 : Math.max(0, Math.min(255, parsed)),
                                } : current);
                                setCreatorDraftError(null);
                              }}
                              style={{
                                padding: '0.6rem 0.7rem',
                                borderRadius: '8px',
                                border: '1px solid var(--border, rgba(255,255,255,0.12))',
                                background: 'rgba(15,23,42,0.55)',
                                color: 'var(--text-primary, #e2e8f0)',
                              }}
                            />
                          </label>
                        </div>

                        <div style={{ marginBottom: '0.75rem' }}>
                          <div style={{ fontSize: '0.78rem', fontWeight: 700, marginBottom: '0.35rem' }}>Circuit</div>
                          <div
                            style={{
                              display: 'grid',
                              gridTemplateColumns: 'repeat(auto-fit, minmax(120px, 1fr))',
                              gap: '0.5rem',
                            }}
                          >
                            {(['Minor', 'Major', 'World', 'Special'] as CircuitType[]).map((option) => (
                              <button
                                key={option}
                                className="emulator-header-btn"
                                disabled={creatorDraftBusy}
                                onClick={() => {
                                  setCreatorDraft((current) => current ? { ...current, circuit: option } : current);
                                  setCreatorDraftMessage(`${option} circuit selected in the draft.`);
                                  setCreatorDraftError(null);
                                }}
                                style={{
                                  background: creatorDraft.circuit === option ? 'rgba(59,130,246,0.18)' : undefined,
                                  borderColor: creatorDraft.circuit === option ? 'rgba(59,130,246,0.45)' : undefined,
                                }}
                              >
                                {option}
                              </button>
                            ))}
                          </div>
                        </div>

                        <div style={{ marginBottom: '0.75rem' }}>
                          <div style={{ fontSize: '0.78rem', fontWeight: 700, marginBottom: '0.35rem' }}>
                            Portrait Asset Owner
                          </div>
                          <div
                            style={{
                              display: 'grid',
                              gridTemplateColumns: onOpenAssetOwner ? 'minmax(220px, 1fr) auto' : 'minmax(220px, 1fr)',
                              gap: '0.55rem',
                              alignItems: 'center',
                            }}
                          >
                            <select
                              value={portraitOwnerKey}
                              onChange={(event) => {
                                setPortraitOwnerKey(event.target.value);
                                setCreatorDraftMessage(
                                  event.target.value
                                    ? `Portrait owner set to ${boxers.find((boxer) => boxer.key === event.target.value)?.name ?? event.target.value}.`
                                    : 'Portrait owner selection cleared.'
                                );
                              }}
                              style={{
                                padding: '0.6rem 0.7rem',
                                borderRadius: '8px',
                                border: '1px solid var(--border, rgba(255,255,255,0.12))',
                                background: 'rgba(15,23,42,0.55)',
                                color: 'var(--text-primary, #e2e8f0)',
                              }}
                            >
                              <option value="">Select portrait owner</option>
                              {boxers.map((boxer) => (
                                <option key={boxer.key} value={boxer.key}>
                                  {boxer.name}
                                </option>
                              ))}
                            </select>
                            {onOpenAssetOwner && (
                              <button
                                className="emulator-header-btn"
                                disabled={!portraitOwnerKey}
                                onClick={() => {
                                  if (!portraitOwnerKey) return;
                                  onOpenAssetOwner(portraitOwnerKey);
                                }}
                              >
                                Open In Editor
                              </button>
                            )}
                          </div>
                          <div style={{ color: 'var(--text-muted, #94a3b8)', fontSize: '0.76rem', marginTop: '0.35rem' }}>
                            Portrait PNG staging now works through Graphic Assets. Character Create now generates a dedicated owner automatically; this selector lets you inspect or retarget the current session if needed.
                          </div>
                        </div>

                        <label style={{ display: 'grid', gap: '0.35rem', marginBottom: '0.75rem' }}>
                          <span style={{ fontSize: '0.78rem', fontWeight: 700 }}>
                            Intro Quote
                          </span>
                          <textarea
                            ref={creatorIntroInputRef}
                            value={creatorDraft.introText}
                            disabled={creatorDraftBusy}
                            onChange={(event) => {
                              setCreatorDraft((current) => current ? { ...current, introText: event.target.value } : current);
                              setCreatorDraftError(null);
                            }}
                            rows={3}
                            style={{
                              resize: 'vertical',
                              minHeight: '84px',
                              padding: '0.7rem',
                              borderRadius: '8px',
                              border: '1px solid var(--border, rgba(255,255,255,0.12))',
                              background: 'rgba(15,23,42,0.55)',
                              color: 'var(--text-primary, #e2e8f0)',
                            }}
                          />
                        </label>

                        <div
                          style={{
                            display: 'flex',
                            gap: '0.55rem',
                            flexWrap: 'wrap',
                            alignItems: 'center',
                          }}
                        >
                          <button
                            className="emulator-header-btn"
                            disabled={creatorDraftBusy}
                            onClick={() => void refreshCreatorDraft(creatorDraft.boxerId)}
                          >
                            Sync From ROM
                          </button>
                          <button
                            className="emulator-header-btn"
                            disabled={creatorDraftBusy}
                            onClick={() => void cancelCreatorDraft()}
                          >
                            Reset To ROM
                          </button>
                          <button
                            className="emulator-header-btn"
                            disabled={creatorDraftBusy}
                            onClick={() => void commitCreatorDraft()}
                          >
                            Commit Draft
                          </button>
                          <button
                            className="emulator-header-btn"
                            disabled={creatorButtonsDisabled}
                            onClick={() => void pulseCreatorInput(0x2000 | 0x1000 | 0x0020 | 0x0010)}
                          >
                            Re-Enter Creator
                          </button>
                        </div>
                      </>
                    ) : (
                      <div style={{ color: 'var(--text-muted, #94a3b8)', fontSize: '0.82rem' }}>
                        Launch creator mode from a roster session to bind this draft editor to a boxer slot.
                      </div>
                    )}

                    {(creatorDraftMessage || creatorDraftError) && (
                      <div
                        style={{
                          marginTop: '0.75rem',
                          padding: '0.65rem 0.75rem',
                          borderRadius: '8px',
                          border: creatorDraftError
                            ? '1px solid rgba(248,113,113,0.35)'
                            : '1px solid rgba(59,130,246,0.3)',
                          background: creatorDraftError
                            ? 'rgba(127,29,29,0.25)'
                            : 'rgba(30,64,175,0.16)',
                          color: creatorDraftError ? '#fecaca' : '#bfdbfe',
                          fontSize: '0.8rem',
                        }}
                      >
                        {creatorDraftError || creatorDraftMessage}
                      </div>
                    )}
                  </div>

                  <div
                    style={{
                      display: 'grid',
                      gridTemplateColumns: 'repeat(auto-fit, minmax(180px, 1fr))',
                      gap: '0.6rem',
                      fontFamily: 'monospace',
                      fontSize: '0.82rem',
                      color: 'var(--text-muted, #94a3b8)',
                      marginBottom: '0.2rem',
                    }}
                  >
                    <div>Input Low: {formatHexByte(creatorState.input_low)}</div>
                    <div>Input High: {formatHexByte(creatorState.input_high)}</div>
                    <div>Magic: {formatHexByte(creatorState.magic)}</div>
                    <div>Heartbeat: {formatHexByte(creatorState.heartbeat)}</div>
                    <div>Render Page: {creatorState.render_page}</div>
                    <div>Render Cursor: {creatorState.render_cursor}</div>
                    <div>Rows: {creatorState.render_rows.map(formatHexByte).join(' ')}</div>
                  </div>

                  <div
                    style={{
                      marginTop: '0.9rem',
                      display: 'flex',
                      flexDirection: 'column',
                      gap: '0.65rem',
                    }}
                  >
                    <div
                      style={{
                        display: 'flex',
                        justifyContent: 'space-between',
                        alignItems: 'center',
                        gap: '0.75rem',
                        flexWrap: 'wrap',
                      }}
                    >
                      <div style={{ fontWeight: 600, fontSize: '0.85rem' }}>Creator Quick Controls</div>
                      <div style={{ color: 'var(--text-muted, #94a3b8)', fontSize: '0.78rem' }}>
                        {creatorActionBusy ? 'Sending input...' : 'Works while running or paused'}
                      </div>
                    </div>

                    <div
                      style={{
                        display: 'grid',
                        gridTemplateColumns: 'repeat(auto-fit, minmax(140px, 1fr))',
                        gap: '0.55rem',
                      }}
                    >
                      <button
                        className="emulator-header-btn"
                        onClick={() => void pulseCreatorInput(0x2000 | 0x1000 | 0x0020 | 0x0010)}
                        disabled={creatorButtonsDisabled}
                        title="Send Select+Start+L+R"
                      >
                        Enter Creator
                      </button>
                      <button
                        className="emulator-header-btn"
                        onClick={() => void pulseCreatorInput(0x0200)}
                        disabled={creatorButtonsDisabled}
                        title="Left"
                      >
                        Prev Page
                      </button>
                      <button
                        className="emulator-header-btn"
                        onClick={() => void pulseCreatorInput(0x0100)}
                        disabled={creatorButtonsDisabled}
                        title="Right"
                      >
                        Next Page
                      </button>
                      <button
                        className="emulator-header-btn"
                        onClick={() => void pulseCreatorInput(0x0800)}
                        disabled={creatorButtonsDisabled}
                        title="Up"
                      >
                        Cursor Up
                      </button>
                      <button
                        className="emulator-header-btn"
                        onClick={() => void pulseCreatorInput(0x0400)}
                        disabled={creatorButtonsDisabled}
                        title="Down"
                      >
                        Cursor Down
                      </button>
                      <button
                        className="emulator-header-btn"
                        onClick={() => void pulseCreatorInput(0x0080)}
                        disabled={creatorButtonsDisabled}
                        title="A button"
                      >
                        Select Action
                      </button>
                      <button
                        className="emulator-header-btn"
                        onClick={() => void pulseCreatorInput(0x8000)}
                        disabled={creatorButtonsDisabled}
                        title="B button"
                      >
                        Exit Creator
                      </button>
                    </div>

                    <div
                      style={{
                        color: 'var(--text-muted, #94a3b8)',
                        fontSize: '0.78rem',
                        lineHeight: 1.5,
                      }}
                    >
                      Keyboard equivalent: hold Select+Start+L+R to enter, then use D-pad, A to trigger, and B to exit.
                    </div>
                  </div>
                </>
              ) : (
                <div style={{ color: 'var(--text-muted, #94a3b8)', fontSize: '0.85rem' }}>
                  Initialize the emulator to inspect the in-ROM creator contract.
                </div>
              )}
            </div>
          </>
        ) : (
          /* Placeholder when no ROM loaded */
          <div className="emulator-placeholder">
            <div className="emulator-placeholder-icon">🎮</div>
            <div className="emulator-placeholder-text">
              No ROM loaded. Use the controls below to load a ROM.
            </div>
            <div className="emulator-placeholder-hint">
              You can switch between your edited ROM and the original for comparison.
            </div>
          </div>
        )}
      </div>

      {/* Controls */}
      <div className="emulator-controls-section">
        <EmulatorControls
          state={emulator.state}
          currentRom={emulator.status.romLoaded}
          volume={emulator.config.volume}
          muted={emulator.config.muted}
          controllerType={emulator.config.controllerType}
          speed={emulator.config.speed}
          saveStateSlot={emulator.config.saveStateSlot}
          saveStates={emulator.saveStates}
          isInitialized={emulator.isInitialized}
          onStart={emulator.start}
          onPause={emulator.pause}
          onResume={emulator.resume}
          onStop={emulator.stop}
          onReset={emulator.reset}
          onSpeedChange={(speed) => emulator.setSpeed(speed as SpeedMode)}
          onFrameAdvance={emulator.frameAdvance}
          onSaveState={emulator.saveState}
          onLoadState={emulator.loadState}
          onDeleteState={emulator.deleteState}
          onSlotChange={emulator.setSaveStateSlot}
          onVolumeChange={emulator.setVolume}
          onToggleMute={emulator.toggleMute}
          onControllerChange={(type) => emulator.setControllerType(type as ControllerType)}
          onShowInputMapper={() => setShowInputMapper(true)}
          onLoadEditedRom={handleLoadEditedRom}
          onLoadOriginalRom={handleLoadOriginalRom}
          onSwapRom={handleSwapRom}
          onScreenshot={handleScreenshot}
          onRecordGif={handleRecordGif}
        />
      </div>

      {/* Input Mapper Modal */}
      <InputMapper
        isOpen={showInputMapper}
        mappings={emulator.currentMappings}
        currentPreset={emulator.currentPreset}
        onUpdateMapping={emulator.updateKeyMapping}
        onLoadPreset={emulator.loadKeyMappingPreset}
        onClose={() => setShowInputMapper(false)}
      />
    </div>
  );
};

export default EmbeddedEmulator;
