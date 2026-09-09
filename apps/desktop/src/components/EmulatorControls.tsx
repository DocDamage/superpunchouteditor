/**
 * EmulatorControls Component
 * 
 * Provides control buttons and UI for the embedded emulator.
 * Includes play/pause, reset, save/load states, volume, and quick actions.
 */

import React from 'react';
import type { 
  EmulatorState, 
  ControllerType, 
  SpeedMode,
  SaveState,
} from '../hooks/useEmulator';

export interface EmulatorControlsProps {
  // State
  state: EmulatorState;
  currentRom: string | null;
  volume: number;
  muted: boolean;
  controllerType: ControllerType;
  speed: SpeedMode;
  saveStateSlot: number;
  saveStates: SaveState[];
  isInitialized?: boolean;
  
  // Play controls
  onStart: () => void;
  onPause: () => void;
  onResume: () => void;
  onStop: () => void;
  onReset: (hard?: boolean) => void;
  
  // Speed controls
  onSpeedChange: (speed: SpeedMode) => void;
  onFrameAdvance: () => void;
  
  // Save/Load states
  onSaveState: (slot: number) => void;
  onLoadState: (slot: number) => void;
  onDeleteState: (slot: number) => void;
  onSlotChange: (slot: number) => void;
  
  // Audio
  onVolumeChange: (volume: number) => void;
  onToggleMute: () => void;
  
  // Input
  onControllerChange: (type: ControllerType) => void;
  onShowInputMapper: () => void;
  
  // ROM actions
  onLoadEditedRom: () => void;
  onLoadOriginalRom: () => void;
  onSwapRom: () => void;
  onScreenshot: () => void;
  onRecordGif: () => void;
  
  // Optional className
  className?: string;
}

// Speed options
const SPEED_OPTIONS: { value: SpeedMode; label: string }[] = [
  { value: 0.25, label: '0.25x' },
  { value: 0.5, label: '0.5x' },
  { value: 1.0, label: '1x' },
  { value: 2.0, label: '2x' },
];

export const EmulatorControls: React.FC<EmulatorControlsProps> = ({
  state,
  currentRom,
  volume,
  muted,
  controllerType,
  speed,
  saveStateSlot,
  saveStates,
  onStart,
  onPause,
  onResume,
  onStop,
  onReset,
  onSpeedChange,
  onFrameAdvance,
  onSaveState,
  onLoadState,
  onDeleteState,
  onSlotChange,
  onVolumeChange,
  onToggleMute,
  onControllerChange,
  onShowInputMapper,
  onLoadEditedRom,
  onLoadOriginalRom,
  onSwapRom,
  onScreenshot,
  onRecordGif,
  className = '',
  isInitialized = true,
}) => {
  const isRunning = state === 'running';
  const isPaused = state === 'paused';
  const isStopped = state === 'stopped';
  const isLoading = state === 'loading';

  // Format timestamp for save states
  const formatTimestamp = (timestamp: number): string => {
    if (timestamp === 0) return 'Empty';
    const date = new Date(timestamp);
    return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  };

  return (
    <div className={`emulator-controls ${className}`} style={styles.container}>
      {/* Main Control Bar */}
      <div style={styles.controlBar}>
        {/* Playback Controls */}
        <div style={styles.controlGroup}>
          {isStopped ? (
            <button
              onClick={onStart}
              disabled={!currentRom || isLoading || !isInitialized}
              style={styles.primaryButton}
              title={isInitialized ? "Start (Space)" : "Emulator not initialized"}
            >
              <span style={styles.icon}>▶</span> Play
            </button>
          ) : isRunning ? (
            <button
              onClick={onPause}
              style={styles.primaryButton}
              title="Pause (Space)"
            >
              <span style={styles.icon}>⏸</span> Pause
            </button>
          ) : (
            <button
              onClick={onResume}
              style={styles.primaryButton}
              title="Resume (Space)"
            >
              <span style={styles.icon}>▶</span> Resume
            </button>
          )}

          <button
            onClick={() => onReset(false)}
            disabled={isStopped}
            style={styles.button}
            title="Soft Reset (Ctrl+R)"
          >
            <span style={styles.icon}>↺</span>
          </button>

          <button
            onClick={() => onReset(true)}
            disabled={isStopped}
            style={styles.button}
            title="Hard Reset"
          >
            <span style={styles.icon}>⟲</span>
          </button>

          <button
            onClick={onStop}
            disabled={isStopped}
            style={styles.button}
            title="Stop"
          >
            <span style={styles.icon}>⏹</span>
          </button>
        </div>

        <div style={styles.divider} />

        {/* Speed Controls */}
        <div style={styles.controlGroup}>
          <select
            value={speed}
            onChange={(e) => onSpeedChange(Number(e.target.value) as SpeedMode)}
            disabled={isStopped}
            style={styles.select}
            title="Emulation Speed"
          >
            {SPEED_OPTIONS.map(opt => (
              <option key={opt.value} value={opt.value}>
                {opt.label}
              </option>
            ))}
          </select>

          <button
            onClick={onFrameAdvance}
            disabled={!isPaused}
            style={styles.button}
            title="Frame Advance (F9)"
          >
            <span style={styles.icon}>⏭</span>
          </button>
        </div>

        <div style={styles.divider} />

        {/* Save State Controls */}
        <div style={styles.controlGroup}>
          <select
            value={saveStateSlot}
            onChange={(e) => onSlotChange(Number(e.target.value))}
            style={styles.select}
            title="Save State Slot"
          >
            {saveStates.map(s => (
              <option key={s.slot} value={s.slot}>
                Slot {s.slot}{s.exists ? ' ✓' : ''}
              </option>
            ))}
          </select>

          <button
            onClick={() => onSaveState(saveStateSlot)}
            disabled={isStopped}
            style={styles.button}
            title={`Save State (F5) Slot ${saveStateSlot}`}
          >
            <span style={styles.icon}>💾</span>
          </button>

          <button
            onClick={() => onLoadState(saveStateSlot)}
            disabled={isStopped || !saveStates[saveStateSlot]?.exists}
            style={styles.button}
            title={`Load State (F7) Slot ${saveStateSlot}`}
          >
            <span style={styles.icon}>📂</span>
          </button>
        </div>

        <div style={styles.divider} />

        {/* Volume Control */}
        <div style={styles.controlGroup}>
          <button
            onClick={onToggleMute}
            style={styles.button}
            title={muted ? 'Unmute' : 'Mute'}
          >
            <span style={styles.icon}>{muted ? '🔇' : volume > 0.5 ? '🔊' : volume > 0 ? '🔉' : '🔇'}</span>
          </button>
          <input
            type="range"
            min={0}
            max={1}
            step={0.1}
            value={muted ? 0 : volume}
            onChange={(e) => onVolumeChange(Number(e.target.value))}
            style={styles.volumeSlider}
            title={`Volume: ${Math.round((muted ? 0 : volume) * 100)}%`}
          />
        </div>

        <div style={styles.divider} />

        {/* Controller Selection */}
        <div style={styles.controlGroup}>
          <select
            value={controllerType}
            onChange={(e) => onControllerChange(e.target.value as ControllerType)}
            style={styles.select}
            title="Controller Type"
          >
            <option value="keyboard">Keyboard</option>
            <option value="gamepad">Gamepad</option>
            <option value="both">Both</option>
          </select>

          <button
            onClick={onShowInputMapper}
            style={styles.button}
            title="Configure Input"
          >
            <span style={styles.icon}>⌨</span>
          </button>
        </div>
      </div>

      {/* Quick Actions Bar */}
      <div style={styles.quickActions}>
        <span style={styles.sectionLabel}>Quick Actions:</span>

        <button
          onClick={onLoadEditedRom}
          disabled={isLoading}
          style={styles.actionButton}
          title="Load current edited ROM"
        >
          Load Edited ROM
        </button>

        <button
          onClick={onLoadOriginalRom}
          disabled={isLoading}
          style={styles.actionButton}
          title="Load original unmodified ROM"
        >
          Load Original
        </button>

        <button
          onClick={onSwapRom}
          disabled={isLoading || !currentRom}
          style={styles.actionButton}
          title="Swap between edited and original"
        >
          Swap ROM
        </button>

        <div style={styles.divider} />

        <button
          onClick={onScreenshot}
          disabled={isStopped}
          style={styles.actionButton}
          title="Take Screenshot"
        >
          📷 Screenshot
        </button>

        <button
          onClick={onRecordGif}
          disabled={isStopped}
          style={styles.actionButton}
          title="Record GIF (3 seconds)"
        >
          🎬 Record GIF
        </button>
      </div>

      {/* Save State Grid (Collapsible) */}
      <div style={styles.saveStateGrid}>
        {saveStates.slice(0, 10).map((s) => (
          <button
            key={s.slot}
            onClick={() => onSlotChange(s.slot)}
            onDoubleClick={() => s.exists && onLoadState(s.slot)}
            style={{
              ...styles.saveStateSlot,
              ...(saveStateSlot === s.slot ? styles.saveStateSlotActive : {}),
              ...(s.exists ? styles.saveStateSlotExists : {}),
            }}
            title={s.exists ? `Slot ${s.slot} - ${formatTimestamp(s.timestamp)}` : `Empty Slot ${s.slot}`}
          >
            <span style={styles.slotNumber}>{s.slot}</span>
            {s.exists && <span style={styles.slotIndicator}>●</span>}
          </button>
        ))}
      </div>
    </div>
  );
};

// Styles using CSS variables for theme integration
const styles: Record<string, React.CSSProperties> = {
  container: {
    display: 'flex',
    flexDirection: 'column',
    gap: '0.75rem',
    padding: '1rem',
    backgroundColor: 'var(--bg-secondary, #1e293b)',
    borderRadius: '8px',
    border: '1px solid var(--border, #334155)',
  },
  controlBar: {
    display: 'flex',
    flexWrap: 'wrap',
    alignItems: 'center',
    gap: '0.75rem',
  },
  controlGroup: {
    display: 'flex',
    alignItems: 'center',
    gap: '0.5rem',
  },
  divider: {
    width: '1px',
    height: '24px',
    backgroundColor: 'var(--border, #334155)',
  },
  button: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    padding: '0.5rem 0.75rem',
    backgroundColor: 'var(--bg-tertiary, #334155)',
    border: '1px solid var(--border, #334155)',
    borderRadius: '4px',
    color: 'var(--text-primary, #f8fafc)',
    fontSize: '0.875rem',
    cursor: 'pointer',
    transition: 'all 0.15s ease',
  },
  primaryButton: {
    display: 'flex',
    alignItems: 'center',
    gap: '0.375rem',
    padding: '0.5rem 1rem',
    backgroundColor: 'var(--accent, #e74c3c)',
    border: 'none',
    borderRadius: '4px',
    color: 'white',
    fontSize: '0.875rem',
    fontWeight: 600,
    cursor: 'pointer',
    transition: 'all 0.15s ease',
  },
  select: {
    padding: '0.5rem',
    backgroundColor: 'var(--bg-tertiary, #334155)',
    border: '1px solid var(--border, #334155)',
    borderRadius: '4px',
    color: 'var(--text-primary, #f8fafc)',
    fontSize: '0.875rem',
    cursor: 'pointer',
  },
  volumeSlider: {
    width: '80px',
    accentColor: 'var(--accent, #e74c3c)',
  },
  icon: {
    fontSize: '1rem',
    lineHeight: 1,
  },
  quickActions: {
    display: 'flex',
    flexWrap: 'wrap',
    alignItems: 'center',
    gap: '0.5rem',
    padding: '0.75rem',
    backgroundColor: 'var(--bg-primary, #0f172a)',
    borderRadius: '6px',
  },
  sectionLabel: {
    fontSize: '0.75rem',
    fontWeight: 600,
    color: 'var(--text-muted, #64748b)',
    textTransform: 'uppercase',
    letterSpacing: '0.05em',
    marginRight: '0.5rem',
  },
  actionButton: {
    padding: '0.375rem 0.75rem',
    backgroundColor: 'var(--glass, rgba(255, 255, 255, 0.05))',
    border: '1px solid var(--border, #334155)',
    borderRadius: '4px',
    color: 'var(--text-secondary, #cbd5e1)',
    fontSize: '0.8125rem',
    cursor: 'pointer',
    transition: 'all 0.15s ease',
  },
  saveStateGrid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(10, 1fr)',
    gap: '0.25rem',
  },
  saveStateSlot: {
    position: 'relative',
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    justifyContent: 'center',
    aspectRatio: '1',
    padding: '0.25rem',
    backgroundColor: 'var(--bg-tertiary, #334155)',
    border: '1px solid var(--border, #334155)',
    borderRadius: '4px',
    cursor: 'pointer',
    transition: 'all 0.15s ease',
  },
  saveStateSlotActive: {
    borderColor: 'var(--accent, #e74c3c)',
    boxShadow: '0 0 0 2px var(--accent-muted, #1e3a5f)',
  },
  saveStateSlotExists: {
    backgroundColor: 'var(--success-muted, rgba(34, 197, 94, 0.1))',
  },
  slotNumber: {
    fontSize: '0.75rem',
    fontWeight: 600,
    color: 'var(--text-muted, #64748b)',
  },
  slotIndicator: {
    position: 'absolute',
    top: '2px',
    right: '2px',
    fontSize: '0.5rem',
    color: 'var(--success, #22c55e)',
  },
};

export default EmulatorControls;
