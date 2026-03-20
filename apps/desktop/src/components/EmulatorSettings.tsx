import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';

export interface EmulatorSettingsData {
  emulatorPath: string;
  emulatorType: 'snes9x' | 'bsnes' | 'mesen-s' | 'other';
  autoSaveBeforeLaunch: boolean;
  commandLineArgs: string;
  jumpToSelectedBoxer: boolean;
  defaultRound: number;
  saveStateDir: string | null;
}

export interface EmulatorInfo {
  emulator_type: 'snes9x' | 'bsnes' | 'mesen-s' | 'other';
  path: string;
  version: string | null;
  is_valid: boolean;
  error_message: string | null;
}

interface EmulatorSettingsProps {
  isOpen: boolean;
  onClose: () => void;
  onSave: (settings: EmulatorSettingsData) => void;
  currentSettings?: Partial<EmulatorSettingsData>;
}

const EMULATOR_TYPES = [
  { value: 'snes9x', label: 'Snes9x', defaultExe: 'snes9x-x64.exe' },
  { value: 'bsnes', label: 'bsnes/higan', defaultExe: 'bsnes.exe' },
  { value: 'mesen-s', label: 'Mesen-S', defaultExe: 'Mesen-S.exe' },
  { value: 'other', label: 'Other', defaultExe: '' },
] as const;

export function EmulatorSettings({ isOpen, onClose, onSave, currentSettings }: EmulatorSettingsProps) {
  const [settings, setSettings] = useState<EmulatorSettingsData>({
    emulatorPath: '',
    emulatorType: 'snes9x',
    autoSaveBeforeLaunch: true,
    commandLineArgs: '',
    jumpToSelectedBoxer: true,
    defaultRound: 1,
    saveStateDir: null,
    ...currentSettings,
  });

  const [verificationStatus, setVerificationStatus] = useState<{
    verifying: boolean;
    info: EmulatorInfo | null;
    error: string | null;
  }>({
    verifying: false,
    info: null,
    error: null,
  });

  const [isModified, setIsModified] = useState(false);

  // Load saved settings on mount
  useEffect(() => {
    const loadSettings = async () => {
      try {
        const saved = await invoke<EmulatorSettingsData | null>('get_emulator_settings');
        if (saved) {
          setSettings(saved);
        }
      } catch (e) {
        console.log('No saved emulator settings found');
      }
    };
    if (isOpen) {
      loadSettings();
    }
  }, [isOpen]);

  const handleBrowseEmulator = async () => {
    const selected = await open({
      multiple: false,
      filters: [
        {
          name: 'Emulator Executable',
          extensions: ['exe', 'app', ''],
        },
      ],
    });

    if (typeof selected === 'string') {
      setSettings((prev) => ({ ...prev, emulatorPath: selected }));
      setIsModified(true);
      // Auto-verify when path is selected
      verifyEmulator(selected);
    }
  };

  const verifyEmulator = async (path: string) => {
    if (!path) return;

    setVerificationStatus({ verifying: true, info: null, error: null });

    try {
      const info = await invoke<EmulatorInfo>('verify_emulator', { emulatorPath: path });
      setVerificationStatus({
        verifying: false,
        info,
        error: info.is_valid ? null : info.error_message,
      });
      // Update detected emulator type
      if (info.is_valid) {
        setSettings((prev) => ({
          ...prev,
          emulatorType: info.emulator_type,
        }));
      }
    } catch (e) {
      setVerificationStatus({
        verifying: false,
        info: null,
        error: (e as Error).message,
      });
    }
  };

  const handleSave = async () => {
    try {
      await invoke('set_emulator_settings', { settings });
      onSave(settings);
      onClose();
    } catch (e) {
      console.error('Failed to save emulator settings:', e);
      setVerificationStatus((prev) => ({
        ...prev,
        error: 'Failed to save settings: ' + (e as Error).message,
      }));
    }
  };

  const handleChange = <K extends keyof EmulatorSettingsData>(
    key: K,
    value: EmulatorSettingsData[K]
  ) => {
    setSettings((prev) => ({ ...prev, [key]: value }));
    setIsModified(true);
  };

  if (!isOpen) return null;

  return (
    <div
      style={{
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
      }}
      onClick={onClose}
    >
      <div
        style={{
          backgroundColor: 'var(--panel-bg)',
          borderRadius: '12px',
          border: '1px solid var(--border)',
          padding: '2rem',
          width: '100%',
          maxWidth: '600px',
          maxHeight: '90vh',
          overflow: 'auto',
        }}
        onClick={(e) => e.stopPropagation()}
      >
        <h2 style={{ marginTop: 0, marginBottom: '1.5rem' }}>Emulator Settings</h2>

        {/* Emulator Type Selection */}
        <div style={{ marginBottom: '1.5rem' }}>
          <label
            style={{
              display: 'block',
              marginBottom: '0.5rem',
              fontWeight: 500,
              color: 'var(--text-dim)',
            }}
          >
            Emulator Type
          </label>
          <select
            value={settings.emulatorType}
            onChange={(e) => handleChange('emulatorType', e.target.value as EmulatorSettingsData['emulatorType'])}
            style={{
              width: '100%',
              padding: '0.5rem',
              borderRadius: '4px',
              border: '1px solid var(--border)',
              backgroundColor: 'var(--glass)',
              color: 'var(--text)',
            }}
          >
            {EMULATOR_TYPES.map((type) => (
              <option key={type.value} value={type.value}>
                {type.label}
              </option>
            ))}
          </select>
        </div>

        {/* Emulator Path */}
        <div style={{ marginBottom: '1.5rem' }}>
          <label
            style={{
              display: 'block',
              marginBottom: '0.5rem',
              fontWeight: 500,
              color: 'var(--text-dim)',
            }}
          >
            Emulator Path
          </label>
          <div style={{ display: 'flex', gap: '0.5rem' }}>
            <input
              type="text"
              value={settings.emulatorPath}
              readOnly
              placeholder="Click Browse to select emulator..."
              style={{
                flex: 1,
                padding: '0.5rem',
                borderRadius: '4px',
                border: '1px solid var(--border)',
                backgroundColor: 'var(--glass)',
                color: 'var(--text)',
              }}
            />
            <button
              onClick={handleBrowseEmulator}
              style={{
                padding: '0.5rem 1rem',
                whiteSpace: 'nowrap',
              }}
            >
              Browse...
            </button>
          </div>
        </div>

        {/* Verification Status */}
        {settings.emulatorPath && (
          <div
            style={{
              marginBottom: '1.5rem',
              padding: '1rem',
              borderRadius: '8px',
              backgroundColor: verificationStatus.error
                ? 'rgba(220, 38, 38, 0.2)'
                : verificationStatus.info?.is_valid
                ? 'rgba(74, 222, 128, 0.1)'
                : 'var(--glass)',
              border: `1px solid ${
                verificationStatus.error
                  ? 'var(--accent)'
                  : verificationStatus.info?.is_valid
                  ? '#4ade80'
                  : 'var(--border)'
              }`,
            }}
          >
            {verificationStatus.verifying ? (
              <span style={{ color: 'var(--text-dim)' }}>Verifying emulator...</span>
            ) : verificationStatus.error ? (
              <div>
                <strong style={{ color: 'var(--accent)' }}>❌ Error</strong>
                <p style={{ margin: '0.5rem 0 0 0', color: 'var(--text-dim)' }}>
                  {verificationStatus.error}
                </p>
              </div>
            ) : verificationStatus.info?.is_valid ? (
              <div>
                <strong style={{ color: '#4ade80' }}>✓ Verified</strong>
                {verificationStatus.info.version && (
                  <p style={{ margin: '0.5rem 0 0 0', color: 'var(--text-dim)' }}>
                    Version: {verificationStatus.info.version}
                  </p>
                )}
              </div>
            ) : null}
          </div>
        )}

        {/* Auto-save Option */}
        <div
          style={{
            marginBottom: '1.5rem',
            padding: '1rem',
            backgroundColor: 'var(--glass)',
            borderRadius: '8px',
          }}
        >
          <label
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: '0.5rem',
              cursor: 'pointer',
            }}
          >
            <input
              type="checkbox"
              checked={settings.autoSaveBeforeLaunch}
              onChange={(e) => handleChange('autoSaveBeforeLaunch', e.target.checked)}
            />
            <span>Auto-save ROM before launching emulator</span>
          </label>
          <p
            style={{
              margin: '0.5rem 0 0 1.5rem',
              fontSize: '0.85rem',
              color: 'var(--text-dim)',
            }}
          >
            Saves your current edits to a temporary file before testing
          </p>
        </div>

        {/* Quick Launch Options */}
        <div
          style={{
            marginBottom: '1.5rem',
            padding: '1rem',
            backgroundColor: 'var(--glass)',
            borderRadius: '8px',
          }}
        >
          <label
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: '0.5rem',
              cursor: 'pointer',
              marginBottom: '1rem',
            }}
          >
            <input
              type="checkbox"
              checked={settings.jumpToSelectedBoxer}
              onChange={(e) => handleChange('jumpToSelectedBoxer', e.target.checked)}
            />
            <span>Jump to selected boxer on launch</span>
          </label>

          {settings.jumpToSelectedBoxer && (
            <div style={{ marginLeft: '1.5rem' }}>
              <label
                style={{
                  display: 'block',
                  marginBottom: '0.5rem',
                  fontSize: '0.9rem',
                  color: 'var(--text-dim)',
                }}
              >
                Default Round
              </label>
              <select
                value={settings.defaultRound}
                onChange={(e) => handleChange('defaultRound', parseInt(e.target.value))}
                style={{
                  padding: '0.5rem',
                  borderRadius: '4px',
                  border: '1px solid var(--border)',
                  backgroundColor: 'var(--glass)',
                  color: 'var(--text)',
                }}
              >
                <option value={1}>Round 1</option>
                <option value={2}>Round 2</option>
                <option value={3}>Round 3</option>
              </select>
            </div>
          )}
        </div>

        {/* Command Line Arguments */}
        <div style={{ marginBottom: '1.5rem' }}>
          <label
            style={{
              display: 'block',
              marginBottom: '0.5rem',
              fontWeight: 500,
              color: 'var(--text-dim)',
            }}
          >
            Additional Command Line Arguments (Optional)
          </label>
          <input
            type="text"
            value={settings.commandLineArgs}
            onChange={(e) => handleChange('commandLineArgs', e.target.value)}
            placeholder="e.g., -fullscreen -nospeedhacks"
            style={{
              width: '100%',
              padding: '0.5rem',
              borderRadius: '4px',
              border: '1px solid var(--border)',
              backgroundColor: 'var(--glass)',
              color: 'var(--text)',
            }}
          />
          <p
            style={{
              margin: '0.5rem 0 0 0',
              fontSize: '0.85rem',
              color: 'var(--text-dim)',
            }}
          >
            These arguments will be passed to the emulator when launching
          </p>
        </div>

        {/* Keyboard Shortcut Info */}
        <div
          style={{
            marginBottom: '1.5rem',
            padding: '1rem',
            backgroundColor: 'var(--glass)',
            borderRadius: '8px',
            fontSize: '0.9rem',
          }}
        >
          <strong style={{ color: 'var(--blue)' }}>Keyboard Shortcut</strong>
          <p style={{ margin: '0.5rem 0 0 0', color: 'var(--text-dim)' }}>
            Press <kbd>F5</kbd> to quickly test in emulator
          </p>
        </div>

        {/* Action Buttons */}
        <div style={{ display: 'flex', gap: '1rem', justifyContent: 'flex-end' }}>
          <button
            onClick={onClose}
            style={{
              padding: '0.75rem 1.5rem',
              backgroundColor: 'transparent',
              border: '1px solid var(--border)',
            }}
          >
            Cancel
          </button>
          <button
            onClick={handleSave}
            disabled={!settings.emulatorPath || verificationStatus.verifying}
            style={{
              padding: '0.75rem 1.5rem',
              opacity: !settings.emulatorPath || verificationStatus.verifying ? 0.5 : 1,
              cursor: !settings.emulatorPath || verificationStatus.verifying ? 'not-allowed' : 'pointer',
            }}
          >
            {isModified ? 'Save Settings' : 'Close'}
          </button>
        </div>
      </div>
    </div>
  );
}

export default EmulatorSettings;
