/**
 * Settings Import/Export Integration Example
 * 
 * This file shows how to integrate the SettingsManager component
 * into your application.
 */

import { useState } from 'react';
import { SettingsManager, SettingsImportDialog } from '../components';

// Example 1: Basic Settings Manager Usage
export function SettingsExample() {
  const [showSettings, setShowSettings] = useState(false);

  return (
    <div>
      <h1>Super Punch-Out!! Editor</h1>
      
      {/* Settings Button */}
      <button 
        onClick={() => setShowSettings(true)}
        style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}
      >
        ⚙️ Settings
      </button>

      {/* Settings Manager Dialog */}
      <SettingsManager
        isOpen={showSettings}
        onClose={() => setShowSettings(false)}
      />
    </div>
  );
}

// Example 2: Advanced Usage with Import Dialog
export function AdvancedSettingsExample() {
  const [showSettings, setShowSettings] = useState(false);
  const [showImportDialog, setShowImportDialog] = useState(false);

  const handleImport = (filePath: string, merge: boolean) => {
    console.log('Importing settings from:', filePath, 'Merge:', merge);
    setShowImportDialog(false);
    // The actual import is handled by the SettingsImportDialog component
  };

  return (
    <div>
      <h1>Super Punch-Out!! Editor</h1>
      
      <div style={{ display: 'flex', gap: '0.5rem' }}>
        <button onClick={() => setShowSettings(true)}>
          ⚙️ Settings
        </button>
        <button onClick={() => setShowImportDialog(true)}>
          📥 Import Settings
        </button>
      </div>

      <SettingsManager
        isOpen={showSettings}
        onClose={() => setShowSettings(false)}
      />

      <SettingsImportDialog
        isOpen={showImportDialog}
        onClose={() => setShowImportDialog(false)}
        onImport={handleImport}
      />
    </div>
  );
}

// Example 3: Menu Integration
export function MenuBarExample() {
  const [showSettings, setShowSettings] = useState(false);
  const [showImportDialog, setShowImportDialog] = useState(false);

  return (
    <>
      <nav style={{ 
        display: 'flex', 
        gap: '1rem', 
        padding: '0.5rem 1rem',
        backgroundColor: 'var(--panel-bg)',
        borderBottom: '1px solid var(--border)'
      }}>
        <div style={{ position: 'relative' }}>
          <button>File</button>
          {/* Dropdown menu */}
          <div style={{
            position: 'absolute',
            top: '100%',
            left: 0,
            backgroundColor: 'var(--panel-bg)',
            border: '1px solid var(--border)',
            borderRadius: '4px',
            padding: '0.5rem',
            minWidth: '200px',
          }}>
            <button 
              onClick={() => setShowSettings(true)}
              style={{ 
                width: '100%', 
                textAlign: 'left',
                padding: '0.5rem',
                backgroundColor: 'transparent',
                border: 'none',
                cursor: 'pointer'
              }}
            >
              ⚙️ Settings...
            </button>
            <button 
              onClick={() => setShowImportDialog(true)}
              style={{ 
                width: '100%', 
                textAlign: 'left',
                padding: '0.5rem',
                backgroundColor: 'transparent',
                border: 'none',
                cursor: 'pointer'
              }}
            >
              📥 Import Settings...
            </button>
            <div style={{ height: '1px', backgroundColor: 'var(--border)', margin: '0.5rem 0' }} />
            <button 
              onClick={() => {}}
              style={{ 
                width: '100%', 
                textAlign: 'left',
                padding: '0.5rem',
                backgroundColor: 'transparent',
                border: 'none',
                cursor: 'pointer'
              }}
            >
              📤 Export Settings...
            </button>
          </div>
        </div>
      </nav>

      <SettingsManager
        isOpen={showSettings}
        onClose={() => setShowSettings(false)}
      />

      <SettingsImportDialog
        isOpen={showImportDialog}
        onClose={() => setShowImportDialog(false)}
        onImport={(file, merge) => console.log('Import:', file, merge)}
      />
    </>
  );
}

// Example 4: Using the settings API directly
import { invoke } from '@tauri-apps/api/core';

export async function settingsApiExamples() {
  // Export settings
  try {
    await invoke('export_settings', { 
      outputPath: '/path/to/export.json' 
    });
    console.log('Settings exported successfully');
  } catch (e) {
    console.error('Export failed:', e);
  }

  // Import settings with merge
  try {
    const report = await invoke('import_settings', {
      settingsPath: '/path/to/import.json',
      merge: true
    });
    console.log('Import report:', report);
  } catch (e) {
    console.error('Import failed:', e);
  }

  // Preview import changes
  try {
    const preview = await invoke('preview_settings_import', {
      settingsPath: '/path/to/import.json'
    });
    console.log('Import preview:', preview);
  } catch (e) {
    console.error('Preview failed:', e);
  }

  // Validate settings file
  try {
    const validation = await invoke('validate_settings_file', {
      settingsPath: '/path/to/file.json'
    });
    console.log('Validation result:', validation);
  } catch (e) {
    console.error('Validation failed:', e);
  }

  // Reset to defaults
  try {
    await invoke('reset_settings_to_defaults');
    console.log('Settings reset to defaults');
  } catch (e) {
    console.error('Reset failed:', e);
  }
}

// Example 5: Settings Context Provider
import { createContext, useContext, useState, useEffect, ReactNode } from 'react';
import { AppSettings, DEFAULT_SETTINGS } from './settings';

interface SettingsContextType {
  settings: AppSettings;
  updateSettings: (updates: Partial<AppSettings>) => Promise<void>;
  exportSettings: (path: string) => Promise<void>;
  importSettings: (path: string, merge: boolean) => Promise<void>;
  resetSettings: () => Promise<void>;
}

const SettingsContext = createContext<SettingsContextType | undefined>(undefined);

export function SettingsProvider({ children }: { children: ReactNode }) {
  const [settings, setSettings] = useState<AppSettings>(DEFAULT_SETTINGS);

  useEffect(() => {
    // Load settings on mount
    loadSettings();
  }, []);

  const loadSettings = async () => {
    try {
      const loaded = await invoke<AppSettings>('get_app_settings');
      setSettings(loaded);
    } catch (e) {
      console.error('Failed to load settings:', e);
    }
  };

  const updateSettings = async (updates: Partial<AppSettings>) => {
    try {
      const updated = await invoke<AppSettings>('update_settings', { updates });
      setSettings(updated);
    } catch (e) {
      console.error('Failed to update settings:', e);
      throw e;
    }
  };

  const exportSettings = async (path: string) => {
    await invoke('export_settings', { outputPath: path });
  };

  const importSettings = async (path: string, merge: boolean) => {
    const report = await invoke('import_settings', {
      settingsPath: path,
      merge
    });
    console.log('Import report:', report);
    await loadSettings(); // Reload after import
  };

  const resetSettings = async () => {
    await invoke('reset_settings_to_defaults');
    setSettings(DEFAULT_SETTINGS);
  };

  return (
    <SettingsContext.Provider value={{
      settings,
      updateSettings,
      exportSettings,
      importSettings,
      resetSettings
    }}>
      {children}
    </SettingsContext.Provider>
  );
}

export function useSettings() {
  const context = useContext(SettingsContext);
  if (!context) {
    throw new Error('useSettings must be used within a SettingsProvider');
  }
  return context;
}
