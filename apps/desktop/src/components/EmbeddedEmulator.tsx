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
import { EmulatorCanvas } from './EmulatorCanvas';
import { EmulatorControls } from './EmulatorControls';
import { InputMapper } from './InputMapper';
import { useEmulator, type SpeedMode, type ControllerType } from '../hooks/useEmulator';
import '../styles/emulator.css';

export type EmulatorLayout = 'tab' | 'split' | 'overlay';
export type RomSource = 'edited' | 'original';

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
}) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [showInputMapper, setShowInputMapper] = useState(false);
  const [currentRomSource, setCurrentRomSource] = useState<RomSource>('edited');
  const [isRecordingGif, setIsRecordingGif] = useState(false);
  const [recordingProgress, setRecordingProgress] = useState(0);
  const [screenshots, setScreenshots] = useState<string[]>([]);

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
        
        // In real implementation, this would compile and download the GIF
        alert('GIF recording complete! (Feature simulation)');
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
                ref={canvasRef}
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
