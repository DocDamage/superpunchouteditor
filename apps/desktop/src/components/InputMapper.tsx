/**
 * InputMapper Component
 * 
 * Allows users to view and remap SNES controller buttons to keyboard keys.
 * Supports multiple presets and custom mapping configuration.
 */

import React, { useState, useEffect, useCallback } from 'react';
import type { 
  InputMapping, 
  KeyMappingPreset,
  DEFAULT_KEY_MAPPINGS_WASD,
  DEFAULT_KEY_MAPPINGS_ARROWS,
  DEFAULT_KEY_MAPPINGS_FIGHTSTICK,
} from '../hooks/useEmulator';

export interface InputMapperProps {
  /** Current key mappings */
  mappings: InputMapping[];
  /** Current preset */
  currentPreset: KeyMappingPreset;
  /** Called when a mapping is updated */
  onUpdateMapping: (button: string, key: string, label: string) => void;
  /** Called when a preset is loaded */
  onLoadPreset: (preset: KeyMappingPreset) => void;
  /** Called when the modal should close */
  onClose: () => void;
  /** Whether the mapper is visible */
  isOpen: boolean;
}

// Button display info
const BUTTON_INFO: Record<string, { label: string; color: string; icon: string }> = {
  B: { label: 'B Button', color: '#ef4444', icon: '🅱' },
  Y: { label: 'Y Button', color: '#eab308', icon: 'Ⓨ' },
  SELECT: { label: 'Select', color: '#64748b', icon: '⊖' },
  START: { label: 'Start', color: '#64748b', icon: '⊕' },
  UP: { label: 'D-Pad Up', color: '#3b82f6', icon: '▲' },
  DOWN: { label: 'D-Pad Down', color: '#3b82f6', icon: '▼' },
  LEFT: { label: 'D-Pad Left', color: '#3b82f6', icon: '◀' },
  RIGHT: { label: 'D-Pad Right', color: '#3b82f6', icon: '▶' },
  A: { label: 'A Button', color: '#22c55e', icon: '🅰' },
  X: { label: 'X Button', color: '#3b82f6', icon: 'Ⓧ' },
  L: { label: 'L Shoulder', color: '#8b5cf6', icon: 'L' },
  R: { label: 'R Shoulder', color: '#8b5cf6', icon: 'R' },
};

// Preset options
const PRESETS: { value: KeyMappingPreset; label: string; description: string }[] = [
  { 
    value: 'wasd', 
    label: 'WASD', 
    description: 'WASD for movement, Z/X for A/B buttons' 
  },
  { 
    value: 'arrows', 
    label: 'Arrow Keys', 
    description: 'Arrow keys for movement, Z/X for A/B buttons' 
  },
  { 
    value: 'fightstick', 
    label: 'Fight Stick', 
    description: 'Layout mimicking a fight stick / hitbox controller' 
  },
  { 
    value: 'custom', 
    label: 'Custom', 
    description: 'Your custom key configuration' 
  },
];

export const InputMapper: React.FC<InputMapperProps> = ({
  mappings,
  currentPreset,
  onUpdateMapping,
  onLoadPreset,
  onClose,
  isOpen,
}) => {
  const [listeningButton, setListeningButton] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<'mapping' | 'help'>('mapping');

  // Handle key capture when listening for input
  useEffect(() => {
    if (!listeningButton) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      e.preventDefault();
      
      // Get the key label
      let label = e.code;
      if (e.code.startsWith('Key')) {
        label = e.code.replace('Key', '');
      } else if (e.code.startsWith('Arrow')) {
        label = e.code.replace('Arrow', '');
        switch (label) {
          case 'Up': label = '↑'; break;
          case 'Down': label = '↓'; break;
          case 'Left': label = '←'; break;
          case 'Right': label = '→'; break;
        }
      } else if (e.code.startsWith('Digit')) {
        label = e.code.replace('Digit', '');
      } else if (e.code === 'Space') {
        label = 'Space';
      } else if (e.code === 'Enter') {
        label = 'Enter';
      } else if (e.code === 'ShiftLeft' || e.code === 'ShiftRight') {
        label = e.code === 'ShiftLeft' ? 'LShift' : 'RShift';
      } else if (e.code === 'ControlLeft' || e.code === 'ControlRight') {
        label = e.code === 'ControlLeft' ? 'LCtrl' : 'RCtrl';
      } else if (e.code === 'AltLeft' || e.code === 'AltRight') {
        label = e.code === 'AltLeft' ? 'LAlt' : 'RAlt';
      }

      onUpdateMapping(listeningButton, e.code, label);
      setListeningButton(null);
    };

    const handleKeyUp = (e: KeyboardEvent) => {
      // Prevent key repeat
      e.preventDefault();
    };

    window.addEventListener('keydown', handleKeyDown);
    window.addEventListener('keyup', handleKeyUp);

    return () => {
      window.removeEventListener('keydown', handleKeyDown);
      window.removeEventListener('keyup', handleKeyUp);
    };
  }, [listeningButton, onUpdateMapping]);

  // Reset to default preset
  const handleReset = useCallback(() => {
    onLoadPreset('wasd');
  }, [onLoadPreset]);

  // Clear all mappings
  const handleClearAll = useCallback(() => {
    // This would require a bulk update function
    // For now, individual mappings can be cleared
  }, []);

  if (!isOpen) return null;

  return (
    <div style={styles.overlay} onClick={onClose}>
      <div style={styles.modal} onClick={(e) => e.stopPropagation()}>
        {/* Header */}
        <div style={styles.header}>
          <h2 style={styles.title}>Input Configuration</h2>
          <button onClick={onClose} style={styles.closeButton}>×</button>
        </div>

        {/* Tabs */}
        <div style={styles.tabs}>
          <button
            onClick={() => setActiveTab('mapping')}
            style={{
              ...styles.tab,
              ...(activeTab === 'mapping' ? styles.tabActive : {}),
            }}
          >
            Key Mapping
          </button>
          <button
            onClick={() => setActiveTab('help')}
            style={{
              ...styles.tab,
              ...(activeTab === 'help' ? styles.tabActive : {}),
            }}
          >
            Quick Reference
          </button>
        </div>

        {/* Content */}
        <div style={styles.content}>
          {activeTab === 'mapping' ? (
            <>
              {/* Preset Selector */}
              <div style={styles.presetSection}>
                <label style={styles.label}>Preset:</label>
                <select
                  value={currentPreset}
                  onChange={(e) => onLoadPreset(e.target.value as KeyMappingPreset)}
                  style={styles.select}
                >
                  {PRESETS.map(preset => (
                    <option key={preset.value} value={preset.value}>
                      {preset.label}
                    </option>
                  ))}
                </select>
                <span style={styles.presetDescription}>
                  {PRESETS.find(p => p.value === currentPreset)?.description}
                </span>
              </div>

              {/* Mapping Grid */}
              <div style={styles.mappingGrid}>
                {mappings.map((mapping) => {
                  const info = BUTTON_INFO[mapping.button] || { 
                    label: mapping.button, 
                    color: '#64748b',
                    icon: mapping.button,
                  };
                  const isListening = listeningButton === mapping.button;

                  return (
                    <div
                      key={mapping.button}
                      style={{
                        ...styles.mappingRow,
                        ...(isListening ? styles.mappingRowListening : {}),
                      }}
                    >
                      <div style={styles.buttonInfo}>
                        <span
                          style={{
                            ...styles.buttonIcon,
                            backgroundColor: info.color,
                          }}
                        >
                          {info.icon}
                        </span>
                        <span style={styles.buttonLabel}>{info.label}</span>
                      </div>
                      <button
                        onClick={() => setListeningButton(isListening ? null : mapping.button)}
                        style={{
                          ...styles.keyButton,
                          ...(isListening ? styles.keyButtonListening : {}),
                        }}
                      >
                        {isListening ? 'Press key...' : mapping.label || 'Unmapped'}
                      </button>
                    </div>
                  );
                })}
              </div>

              {/* Action Buttons */}
              <div style={styles.actions}>
                <button onClick={handleReset} style={styles.actionButton}>
                  Reset to Default
                </button>
                <button onClick={() => setListeningButton(null)} style={styles.actionButton}>
                  Cancel Listening
                </button>
              </div>
            </>
          ) : (
            /* Quick Reference Tab */
            <div style={styles.helpContent}>
              <h3 style={styles.helpTitle}>Default Keyboard Shortcuts</h3>
              
              <div style={styles.helpSection}>
                <h4 style={styles.helpSubtitle}>Emulator Controls</h4>
                <ul style={styles.helpList}>
                  <li><kbd>F5</kbd> - Save State</li>
                  <li><kbd>F7</kbd> - Load State</li>
                  <li><kbd>F9</kbd> - Frame Advance</li>
                  <li><kbd>Space</kbd> - Pause/Resume</li>
                  <li><kbd>Ctrl+R</kbd> - Soft Reset</li>
                </ul>
              </div>

              <div style={styles.helpSection}>
                <h4 style={styles.helpSubtitle}>SNES Controller Layout</h4>
                <div style={styles.controllerVisual}>
                  <div style={styles.dpadVisual}>
                    <div style={styles.dpadRow}>
                      <div style={styles.dpadEmpty} />
                      <kbd style={styles.dpadKey}>↑</kbd>
                      <div style={styles.dpadEmpty} />
                    </div>
                    <div style={styles.dpadRow}>
                      <kbd style={styles.dpadKey}>←</kbd>
                      <kbd style={styles.dpadKey}>↓</kbd>
                      <kbd style={styles.dpadKey}>→</kbd>
                    </div>
                  </div>
                  <div style={styles.actionButtonsVisual}>
                    <div style={styles.actionRow}>
                      <kbd style={{ ...styles.actionKey, backgroundColor: '#eab308' }}>Y</kbd>
                      <kbd style={{ ...styles.actionKey, backgroundColor: '#3b82f6' }}>X</kbd>
                    </div>
                    <div style={styles.actionRow}>
                      <kbd style={{ ...styles.actionKey, backgroundColor: '#ef4444' }}>B</kbd>
                      <kbd style={{ ...styles.actionKey, backgroundColor: '#22c55e' }}>A</kbd>
                    </div>
                  </div>
                </div>
              </div>

              <div style={styles.helpSection}>
                <h4 style={styles.helpSubtitle}>Tips</h4>
                <ul style={styles.tipsList}>
                  <li>Click on a key binding to change it</li>
                  <li>Use unique keys for each button to avoid conflicts</li>
                  <li>Modifier keys (Shift, Ctrl, Alt) can be used</li>
                  <li>Gamepad is automatically detected when connected</li>
                </ul>
              </div>
            </div>
          )}
        </div>

        {/* Footer */}
        <div style={styles.footer}>
          <button onClick={onClose} style={styles.doneButton}>
            Done
          </button>
        </div>
      </div>
    </div>
  );
};

const styles: Record<string, React.CSSProperties> = {
  overlay: {
    position: 'fixed',
    top: 0,
    left: 0,
    right: 0,
    bottom: 0,
    backgroundColor: 'rgba(0, 0, 0, 0.7)',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    zIndex: 1000,
    padding: '1rem',
  },
  modal: {
    backgroundColor: 'var(--bg-panel, #1e293b)',
    borderRadius: '12px',
    border: '1px solid var(--border, #334155)',
    width: '100%',
    maxWidth: '500px',
    maxHeight: '90vh',
    display: 'flex',
    flexDirection: 'column',
    overflow: 'hidden',
    boxShadow: '0 25px 50px -12px rgba(0, 0, 0, 0.5)',
  },
  header: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    padding: '1rem 1.25rem',
    borderBottom: '1px solid var(--border, #334155)',
  },
  title: {
    margin: 0,
    fontSize: '1.25rem',
    fontWeight: 600,
    color: 'var(--text-primary, #f8fafc)',
  },
  closeButton: {
    background: 'none',
    border: 'none',
    fontSize: '1.5rem',
    color: 'var(--text-muted, #64748b)',
    cursor: 'pointer',
    padding: '0.25rem',
    lineHeight: 1,
  },
  tabs: {
    display: 'flex',
    borderBottom: '1px solid var(--border, #334155)',
  },
  tab: {
    flex: 1,
    padding: '0.75rem',
    backgroundColor: 'transparent',
    border: 'none',
    color: 'var(--text-muted, #64748b)',
    fontSize: '0.875rem',
    cursor: 'pointer',
    transition: 'all 0.15s ease',
  },
  tabActive: {
    color: 'var(--accent, #e74c3c)',
    borderBottom: '2px solid var(--accent, #e74c3c)',
    marginBottom: '-1px',
  },
  content: {
    flex: 1,
    overflow: 'auto',
    padding: '1.25rem',
  },
  presetSection: {
    display: 'flex',
    flexWrap: 'wrap',
    alignItems: 'center',
    gap: '0.75rem',
    marginBottom: '1.25rem',
    padding: '1rem',
    backgroundColor: 'var(--bg-secondary, #0f172a)',
    borderRadius: '8px',
  },
  label: {
    fontSize: '0.875rem',
    fontWeight: 500,
    color: 'var(--text-secondary, #cbd5e1)',
  },
  select: {
    padding: '0.5rem',
    backgroundColor: 'var(--bg-tertiary, #334155)',
    border: '1px solid var(--border, #334155)',
    borderRadius: '4px',
    color: 'var(--text-primary, #f8fafc)',
    fontSize: '0.875rem',
  },
  presetDescription: {
    fontSize: '0.75rem',
    color: 'var(--text-muted, #64748b)',
    flex: 1,
  },
  mappingGrid: {
    display: 'flex',
    flexDirection: 'column',
    gap: '0.5rem',
  },
  mappingRow: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    padding: '0.625rem',
    backgroundColor: 'var(--bg-secondary, #0f172a)',
    borderRadius: '6px',
    border: '1px solid transparent',
    transition: 'all 0.15s ease',
  },
  mappingRowListening: {
    borderColor: 'var(--accent, #e74c3c)',
    backgroundColor: 'var(--accent-muted, #1e3a5f)',
  },
  buttonInfo: {
    display: 'flex',
    alignItems: 'center',
    gap: '0.75rem',
  },
  buttonIcon: {
    width: '28px',
    height: '28px',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    borderRadius: '50%',
    fontSize: '0.75rem',
    fontWeight: 700,
    color: 'white',
  },
  buttonLabel: {
    fontSize: '0.875rem',
    color: 'var(--text-primary, #f8fafc)',
  },
  keyButton: {
    padding: '0.5rem 1rem',
    minWidth: '80px',
    backgroundColor: 'var(--bg-tertiary, #334155)',
    border: '1px solid var(--border, #334155)',
    borderRadius: '4px',
    color: 'var(--text-primary, #f8fafc)',
    fontSize: '0.8125rem',
    fontFamily: 'monospace',
    cursor: 'pointer',
    transition: 'all 0.15s ease',
  },
  keyButtonListening: {
    backgroundColor: 'var(--accent, #e74c3c)',
    borderColor: 'var(--accent-hover, #c0392b)',
    color: 'white',
  },
  actions: {
    display: 'flex',
    gap: '0.5rem',
    marginTop: '1rem',
    paddingTop: '1rem',
    borderTop: '1px solid var(--border, #334155)',
  },
  actionButton: {
    padding: '0.5rem 1rem',
    backgroundColor: 'var(--bg-tertiary, #334155)',
    border: '1px solid var(--border, #334155)',
    borderRadius: '4px',
    color: 'var(--text-secondary, #cbd5e1)',
    fontSize: '0.8125rem',
    cursor: 'pointer',
  },
  footer: {
    padding: '1rem 1.25rem',
    borderTop: '1px solid var(--border, #334155)',
    display: 'flex',
    justifyContent: 'flex-end',
  },
  doneButton: {
    padding: '0.625rem 1.5rem',
    backgroundColor: 'var(--accent, #e74c3c)',
    border: 'none',
    borderRadius: '6px',
    color: 'white',
    fontSize: '0.875rem',
    fontWeight: 600,
    cursor: 'pointer',
  },
  // Help tab styles
  helpContent: {
    display: 'flex',
    flexDirection: 'column',
    gap: '1rem',
  },
  helpTitle: {
    margin: 0,
    fontSize: '1rem',
    fontWeight: 600,
    color: 'var(--text-primary, #f8fafc)',
  },
  helpSection: {
    padding: '1rem',
    backgroundColor: 'var(--bg-secondary, #0f172a)',
    borderRadius: '8px',
  },
  helpSubtitle: {
    margin: '0 0 0.75rem 0',
    fontSize: '0.875rem',
    fontWeight: 600,
    color: 'var(--text-secondary, #cbd5e1)',
  },
  helpList: {
    margin: 0,
    padding: '0 0 0 1.25rem',
    fontSize: '0.8125rem',
    color: 'var(--text-muted, #64748b)',
    lineHeight: 1.8,
  },
  tipsList: {
    margin: 0,
    padding: '0 0 0 1.25rem',
    fontSize: '0.8125rem',
    color: 'var(--text-muted, #64748b)',
    lineHeight: 1.8,
  },
  controllerVisual: {
    display: 'flex',
    justifyContent: 'space-around',
    alignItems: 'center',
    padding: '1rem',
  },
  dpadVisual: {
    display: 'flex',
    flexDirection: 'column',
    gap: '0.25rem',
  },
  dpadRow: {
    display: 'flex',
    justifyContent: 'center',
    gap: '0.25rem',
  },
  dpadEmpty: {
    width: '32px',
    height: '32px',
  },
  dpadKey: {
    width: '32px',
    height: '32px',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    backgroundColor: 'var(--bg-tertiary, #334155)',
    border: '1px solid var(--border, #334155)',
    borderRadius: '4px',
    fontSize: '0.75rem',
    color: 'var(--text-primary, #f8fafc)',
  },
  actionButtonsVisual: {
    display: 'flex',
    flexDirection: 'column',
    gap: '0.5rem',
  },
  actionRow: {
    display: 'flex',
    justifyContent: 'center',
    gap: '0.5rem',
  },
  actionKey: {
    width: '32px',
    height: '32px',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    border: 'none',
    borderRadius: '50%',
    fontSize: '0.75rem',
    fontWeight: 700,
    color: 'white',
  },
};

export default InputMapper;
