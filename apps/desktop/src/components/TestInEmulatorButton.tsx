import { useState, useCallback, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { save } from '@tauri-apps/plugin-dialog';

interface TestInEmulatorButtonProps {
  romSha1: string | null;
  selectedBoxerKey: string | null;
  selectedBoxerName: string | null;
  pendingWritesCount: number;
  disabled?: boolean;
}

interface LaunchOptions {
  autoSave: boolean;
  quickLoadSlot: number | null;
  boxerKey: string | null;
  round: number;
}

export function TestInEmulatorButton({
  romSha1,
  selectedBoxerKey,
  selectedBoxerName,
  pendingWritesCount,
  disabled = false,
}: TestInEmulatorButtonProps) {
  const [isLaunching, setIsLaunching] = useState(false);
  const [showDropdown, setShowDropdown] = useState(false);
  const [showUnsavedDialog, setShowUnsavedDialog] = useState(false);
  const [launchError, setLaunchError] = useState<string | null>(null);
  const [emulatorConfigured, setEmulatorConfigured] = useState(false);
  const dropdownRef = useRef<HTMLDivElement>(null);

  // Check if emulator is configured
  useEffect(() => {
    const checkEmulator = async () => {
      try {
        const settings = await invoke<{
          emulatorPath: string;
        } | null>('get_emulator_settings');
        setEmulatorConfigured(!!settings?.emulatorPath);
      } catch (e) {
        setEmulatorConfigured(false);
      }
    };
    checkEmulator();
  }, []);

  // Close dropdown when clicking outside
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
        setShowDropdown(false);
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  // Keyboard shortcut: F5
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'F5' && !e.repeat) {
        e.preventDefault();
        if (romSha1 && emulatorConfigured && !disabled) {
          handleQuickTest();
        }
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [romSha1, emulatorConfigured, disabled, selectedBoxerKey, pendingWritesCount]);

  const handleQuickTest = useCallback(async () => {
    if (!romSha1) return;

    // Check for unsaved changes
    if (pendingWritesCount > 0) {
      setShowUnsavedDialog(true);
      return;
    }

    await launchEmulator({
      autoSave: false,
      quickLoadSlot: null,
      boxerKey: selectedBoxerKey,
      round: 1,
    });
  }, [romSha1, pendingWritesCount, selectedBoxerKey]);

  const launchEmulator = async (options: LaunchOptions) => {
    if (!romSha1) return;

    setIsLaunching(true);
    setLaunchError(null);
    setShowUnsavedDialog(false);
    setShowDropdown(false);

    try {
      await invoke('test_in_emulator', {
        autoSave: options.autoSave,
        quickLoadSlot: options.quickLoadSlot,
        boxerKey: options.boxerKey,
        round: options.round,
      });
    } catch (e) {
      const errorMsg = (e as Error).message;
      setLaunchError(errorMsg);
      console.error('Failed to launch emulator:', e);
    } finally {
      setIsLaunching(false);
    }
  };

  const handleSaveAndLaunch = async () => {
    // First save the ROM to a temp file
    try {
      const tempPath = await save({
        defaultPath: 'temp/testing_rom.sfc',
        filters: [
          {
            name: 'SFC ROM',
            extensions: ['sfc', 'smc'],
          },
        ],
      });

      if (tempPath) {
        await invoke('save_rom_as', { outputPath: tempPath });
        await launchEmulator({
          autoSave: true,
          quickLoadSlot: null,
          boxerKey: selectedBoxerKey,
          round: 1,
        });
      }
    } catch (e) {
      setLaunchError((e as Error).message);
      setIsLaunching(false);
    }
  };

  const handleLaunchWithoutSaving = () => {
    launchEmulator({
      autoSave: false,
      quickLoadSlot: null,
      boxerKey: selectedBoxerKey,
      round: 1,
    });
  };

  // Don't render if no ROM is loaded
  if (!romSha1) return null;

  // Show configuration hint if emulator not configured
  if (!emulatorConfigured) {
    return (
      <div style={{ position: 'relative' }} ref={dropdownRef}>
        <button
          onClick={() => setShowDropdown(!showDropdown)}
          disabled={disabled}
          title="Test in Emulator (F5) - Not Configured"
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: '0.5rem',
            padding: '0.5rem 1rem',
            opacity: disabled ? 0.5 : 0.7,
            cursor: disabled ? 'not-allowed' : 'pointer',
            backgroundColor: 'var(--glass)',
            border: '1px solid var(--border)',
          }}
        >
          <span>▶ Test in Emulator</span>
          <span style={{ fontSize: '0.75rem' }}>⚙️</span>
        </button>

        {showDropdown && (
          <div
            style={{
              position: 'absolute',
              top: '100%',
              right: 0,
              marginTop: '0.5rem',
              backgroundColor: 'var(--panel-bg)',
              border: '1px solid var(--border)',
              borderRadius: '8px',
              padding: '1rem',
              minWidth: '280px',
              zIndex: 100,
              boxShadow: '0 4px 12px rgba(0,0,0,0.3)',
            }}
          >
            <p style={{ margin: '0 0 1rem 0', color: 'var(--text-dim)' }}>
              Emulator not configured. Please configure an emulator in settings first.
            </p>
            <button
              onClick={() => {
                setShowDropdown(false);
                // Dispatch custom event to open settings
                window.dispatchEvent(new CustomEvent('open-emulator-settings'));
              }}
              style={{ width: '100%' }}
            >
              Configure Emulator
            </button>
          </div>
        )}
      </div>
    );
  }

  return (
    <>
      <div style={{ position: 'relative' }} ref={dropdownRef}>
        <div style={{ display: 'flex' }}>
          <button
            onClick={handleQuickTest}
            disabled={isLaunching || disabled}
            title="Test in Emulator (F5)"
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: '0.5rem',
              padding: '0.5rem 1rem',
              borderRadius: '4px 0 0 4px',
              opacity: isLaunching || disabled ? 0.5 : 1,
              cursor: isLaunching || disabled ? 'not-allowed' : 'pointer',
            }}
          >
            {isLaunching ? (
              <span>Launching...</span>
            ) : (
              <>
                <span>▶</span>
                <span>Test in Emulator</span>
              </>
            )}
          </button>
          <button
            onClick={() => setShowDropdown(!showDropdown)}
            disabled={isLaunching || disabled}
            style={{
              padding: '0.5rem',
              borderRadius: '0 4px 4px 0',
              borderLeft: '1px solid rgba(0,0,0,0.2)',
              opacity: isLaunching || disabled ? 0.5 : 1,
              cursor: isLaunching || disabled ? 'not-allowed' : 'pointer',
            }}
          >
            ▼
          </button>
        </div>

        {/* Dropdown Menu */}
        {showDropdown && (
          <div
            style={{
              position: 'absolute',
              top: '100%',
              right: 0,
              marginTop: '0.5rem',
              backgroundColor: 'var(--panel-bg)',
              border: '1px solid var(--border)',
              borderRadius: '8px',
              padding: '0.5rem 0',
              minWidth: '220px',
              zIndex: 100,
              boxShadow: '0 4px 12px rgba(0,0,0,0.3)',
            }}
          >
            {selectedBoxerKey && (
              <button
                onClick={() =>
                  launchEmulator({
                    autoSave: pendingWritesCount > 0,
                    quickLoadSlot: null,
                    boxerKey: selectedBoxerKey,
                    round: 1,
                  })
                }
                style={{
                  width: '100%',
                  textAlign: 'left',
                  padding: '0.75rem 1rem',
                  backgroundColor: 'transparent',
                  border: 'none',
                  color: 'var(--text)',
                  cursor: 'pointer',
                }}
                onMouseEnter={(e) =>
                  (e.currentTarget.style.backgroundColor = 'var(--glass)')
                }
                onMouseLeave={(e) =>
                  (e.currentTarget.style.backgroundColor = 'transparent')
                }
              >
                <div style={{ fontWeight: 500 }}>Test vs {selectedBoxerName || selectedBoxerKey}</div>
                <div style={{ fontSize: '0.8rem', color: 'var(--text-dim)' }}>
                  Jump to selected boxer
                </div>
              </button>
            )}

            <button
              onClick={() =>
                launchEmulator({
                  autoSave: pendingWritesCount > 0,
                  quickLoadSlot: null,
                  boxerKey: null,
                  round: 1,
                })
              }
              style={{
                width: '100%',
                textAlign: 'left',
                padding: '0.75rem 1rem',
                backgroundColor: 'transparent',
                border: 'none',
                color: 'var(--text)',
                cursor: 'pointer',
              }}
              onMouseEnter={(e) =>
                (e.currentTarget.style.backgroundColor = 'var(--glass)')
              }
              onMouseLeave={(e) =>
                (e.currentTarget.style.backgroundColor = 'transparent')
              }
            >
              <div style={{ fontWeight: 500 }}>Test from Start</div>
              <div style={{ fontSize: '0.8rem', color: 'var(--text-dim)' }}>
                Begin at title screen
              </div>
            </button>

            <div
              style={{
                margin: '0.5rem 0',
                borderTop: '1px solid var(--border)',
              }}
            />

            <button
              onClick={() => {
                setShowDropdown(false);
                window.dispatchEvent(new CustomEvent('open-emulator-settings'));
              }}
              style={{
                width: '100%',
                textAlign: 'left',
                padding: '0.75rem 1rem',
                backgroundColor: 'transparent',
                border: 'none',
                color: 'var(--text-dim)',
                cursor: 'pointer',
                fontSize: '0.9rem',
              }}
              onMouseEnter={(e) =>
                (e.currentTarget.style.backgroundColor = 'var(--glass)')
              }
              onMouseLeave={(e) =>
                (e.currentTarget.style.backgroundColor = 'transparent')
              }
            >
              Configure Emulator...
            </button>
          </div>
        )}
      </div>

      {/* Unsaved Changes Dialog */}
      {showUnsavedDialog && (
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
          onClick={() => setShowUnsavedDialog(false)}
        >
          <div
            style={{
              backgroundColor: 'var(--panel-bg)',
              borderRadius: '12px',
              border: '1px solid var(--border)',
              padding: '2rem',
              maxWidth: '400px',
              textAlign: 'center',
            }}
            onClick={(e) => e.stopPropagation()}
          >
            <h3 style={{ marginTop: 0 }}>Unsaved Changes</h3>
            <p style={{ color: 'var(--text-dim)', marginBottom: '1.5rem' }}>
              You have {pendingWritesCount} pending edit
              {pendingWritesCount === 1 ? '' : 's'}. Save changes before testing?
            </p>
            <div style={{ display: 'flex', gap: '1rem', justifyContent: 'center' }}>
              <button
                onClick={() => setShowUnsavedDialog(false)}
                style={{
                  padding: '0.75rem 1.5rem',
                  backgroundColor: 'transparent',
                  border: '1px solid var(--border)',
                }}
              >
                Cancel
              </button>
              <button
                onClick={handleLaunchWithoutSaving}
                style={{
                  padding: '0.75rem 1.5rem',
                  backgroundColor: 'var(--glass)',
                }}
              >
                Launch Without Saving
              </button>
              <button
                onClick={handleSaveAndLaunch}
                style={{
                  padding: '0.75rem 1.5rem',
                }}
              >
                Save & Launch
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Error Dialog */}
      {launchError && (
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
          onClick={() => setLaunchError(null)}
        >
          <div
            style={{
              backgroundColor: 'var(--panel-bg)',
              borderRadius: '12px',
              border: '1px solid var(--accent)',
              padding: '2rem',
              maxWidth: '400px',
              textAlign: 'center',
            }}
            onClick={(e) => e.stopPropagation()}
          >
            <h3 style={{ marginTop: 0, color: 'var(--accent)' }}>Launch Failed</h3>
            <p
              style={{
                color: 'var(--text-dim)',
                marginBottom: '1.5rem',
                wordBreak: 'break-word',
              }}
            >
              {launchError}
            </p>
            <button
              onClick={() => setLaunchError(null)}
              style={{ padding: '0.75rem 1.5rem' }}
            >
              OK
            </button>
          </div>
        </div>
      )}
    </>
  );
}

export default TestInEmulatorButton;
