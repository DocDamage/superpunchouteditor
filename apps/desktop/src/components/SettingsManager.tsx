import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { open, save } from '@tauri-apps/plugin-dialog';

interface SettingsManagerProps {
  isOpen: boolean;
  onClose: () => void;
}

interface ImportReport {
  success: boolean;
  imported: string[];
  merged: string[];
  skipped: string[];
  errors: string[];
  warnings: string[];
}

interface ValidationResult {
  valid: boolean;
  version_compatible: boolean;
  version: string;
  exported_at: string;
  errors: string[];
  warnings: string[];
}

export function SettingsManager({ isOpen, onClose }: SettingsManagerProps) {
  const [isImporting, setIsImporting] = useState(false);
  const [isExporting, setIsExporting] = useState(false);
  const [importReport, setImportReport] = useState<ImportReport | null>(null);
  const [validationResult, setValidationResult] = useState<ValidationResult | null>(null);
  const [selectedFile, setSelectedFile] = useState<string | null>(null);
  const [mergeMode, setMergeMode] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);
  const [showResetConfirm, setShowResetConfirm] = useState(false);

  // Clear state when dialog opens/closes
  useEffect(() => {
    if (isOpen) {
      setImportReport(null);
      setValidationResult(null);
      setSelectedFile(null);
      setError(null);
      setSuccessMessage(null);
      setShowResetConfirm(false);
      setMergeMode(true);
    }
  }, [isOpen]);

  const handleExport = async () => {
    try {
      setIsExporting(true);
      setError(null);

      const filePath = await save({
        filters: [
          { name: 'Settings Files', extensions: ['json'] },
          { name: 'All Files', extensions: ['*'] },
        ],
        defaultPath: 'spo-editor-settings.json',
      });

      if (filePath) {
        await invoke('export_settings', { outputPath: filePath });
        setSuccessMessage('Settings exported successfully!');
        setTimeout(() => setSuccessMessage(null), 3000);
      }
    } catch (e) {
      console.error('Failed to export settings:', e);
      setError(`Failed to export settings: ${e}`);
    } finally {
      setIsExporting(false);
    }
  };

  const handleBrowseImport = async () => {
    try {
      const selected = await open({
        multiple: false,
        filters: [
          { name: 'Settings Files', extensions: ['json'] },
          { name: 'All Files', extensions: ['*'] },
        ],
      });

      if (typeof selected === 'string') {
        setSelectedFile(selected);
        setError(null);
        
        // Validate the file
        const result = await invoke<ValidationResult>('validate_settings_file', {
          settingsPath: selected,
        });
        setValidationResult(result);
      }
    } catch (e) {
      console.error('Failed to browse for settings file:', e);
      setError(`Failed to load settings file: ${e}`);
    }
  };

  const handleImport = async () => {
    if (!selectedFile) return;

    try {
      setIsImporting(true);
      setError(null);

      const report = await invoke<ImportReport>('import_settings', {
        settingsPath: selectedFile,
        merge: mergeMode,
      });

      setImportReport(report);
      
      if (report.success && report.errors.length === 0) {
        setSuccessMessage('Settings imported successfully!');
        setTimeout(() => {
          setSuccessMessage(null);
          setImportReport(null);
          setSelectedFile(null);
          setValidationResult(null);
        }, 3000);
      }
    } catch (e) {
      console.error('Failed to import settings:', e);
      setError(`Failed to import settings: ${e}`);
    } finally {
      setIsImporting(false);
    }
  };

  const handleReset = async () => {
    try {
      await invoke('reset_settings_to_defaults');
      setShowResetConfirm(false);
      setSuccessMessage('Settings reset to defaults successfully!');
      setTimeout(() => setSuccessMessage(null), 3000);
    } catch (e) {
      console.error('Failed to reset settings:', e);
      setError(`Failed to reset settings: ${e}`);
    }
  };

  const clearSelection = () => {
    setSelectedFile(null);
    setValidationResult(null);
    setImportReport(null);
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
        <h2 style={{ marginTop: 0, marginBottom: '1.5rem', display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
          ⚙️ Settings Management
        </h2>

        {/* Error Message */}
        {error && (
          <div
            style={{
              padding: '1rem',
              backgroundColor: 'rgba(220, 38, 38, 0.2)',
              borderRadius: '8px',
              color: 'var(--accent)',
              marginBottom: '1rem',
            }}
          >
            ⚠️ {error}
          </div>
        )}

        {/* Success Message */}
        {successMessage && (
          <div
            style={{
              padding: '1rem',
              backgroundColor: 'rgba(34, 197, 94, 0.2)',
              borderRadius: '8px',
              color: '#22c55e',
              marginBottom: '1rem',
            }}
          >
            ✓ {successMessage}
          </div>
        )}

        {/* Import Report */}
        {importReport && (
          <div
            style={{
              padding: '1rem',
              backgroundColor: importReport.success && importReport.errors.length === 0 
                ? 'rgba(34, 197, 94, 0.1)' 
                : 'rgba(251, 191, 36, 0.1)',
              borderRadius: '8px',
              marginBottom: '1rem',
              border: '1px solid var(--border)',
            }}
          >
            <h4 style={{ marginTop: 0, marginBottom: '0.75rem' }}>
              Import Report
            </h4>
            
            {importReport.imported.length > 0 && (
              <div style={{ marginBottom: '0.5rem' }}>
                <strong style={{ color: '#22c55e' }}>✓ Imported:</strong>
                <div style={{ fontSize: '0.85rem', color: 'var(--text-dim)', marginLeft: '1rem' }}>
                  {importReport.imported.join(', ')}
                </div>
              </div>
            )}
            
            {importReport.merged.length > 0 && (
              <div style={{ marginBottom: '0.5rem' }}>
                <strong style={{ color: '#3b82f6' }}>↻ Merged:</strong>
                <div style={{ fontSize: '0.85rem', color: 'var(--text-dim)', marginLeft: '1rem' }}>
                  {importReport.merged.join(', ')}
                </div>
              </div>
            )}
            
            {importReport.skipped.length > 0 && (
              <div style={{ marginBottom: '0.5rem' }}>
                <strong style={{ color: 'var(--text-dim)' }}>○ Skipped (no change):</strong>
                <div style={{ fontSize: '0.85rem', color: 'var(--text-dim)', marginLeft: '1rem' }}>
                  {importReport.skipped.join(', ')}
                </div>
              </div>
            )}
            
            {importReport.warnings.length > 0 && (
              <div style={{ marginBottom: '0.5rem' }}>
                <strong style={{ color: '#fbbf24' }}>⚠ Warnings:</strong>
                <ul style={{ fontSize: '0.85rem', color: 'var(--text-dim)', margin: '0.25rem 0', paddingLeft: '1.5rem' }}>
                  {importReport.warnings.map((w, i) => (
                    <li key={i}>{w}</li>
                  ))}
                </ul>
              </div>
            )}
            
            {importReport.errors.length > 0 && (
              <div style={{ marginBottom: '0.5rem' }}>
                <strong style={{ color: '#ef4444' }}>✗ Errors:</strong>
                <ul style={{ fontSize: '0.85rem', color: 'var(--text-dim)', margin: '0.25rem 0', paddingLeft: '1.5rem' }}>
                  {importReport.errors.map((e, i) => (
                    <li key={i}>{e}</li>
                  ))}
                </ul>
              </div>
            )}

            <div style={{ marginTop: '1rem', display: 'flex', gap: '0.5rem' }}>
              <button
                onClick={() => {
                  setImportReport(null);
                  setSelectedFile(null);
                  setValidationResult(null);
                }}
                style={{ padding: '0.5rem 1rem', fontSize: '0.85rem' }}
              >
                Import Another File
              </button>
            </div>
          </div>
        )}

        {/* Export Section */}
        {!importReport && (
          <>
            <div style={{ marginBottom: '2rem' }}>
              <h3 style={{ marginTop: 0, marginBottom: '0.75rem', fontSize: '1.1rem', display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
                📤 Export
              </h3>
              <p style={{ marginBottom: '1rem', color: 'var(--text-dim)', fontSize: '0.9rem' }}>
                Save your complete configuration to a file. You can use this to backup your settings or share them with others.
              </p>
              <button
                onClick={handleExport}
                disabled={isExporting}
                style={{
                  padding: '0.75rem 1.5rem',
                  display: 'flex',
                  alignItems: 'center',
                  gap: '0.5rem',
                }}
              >
                {isExporting ? '⏳ Exporting...' : '💾 Export All Settings...'}
              </button>
            </div>

            <div style={{ height: '1px', backgroundColor: 'var(--border)', margin: '1.5rem 0' }} />

            {/* Import Section */}
            <div style={{ marginBottom: '2rem' }}>
              <h3 style={{ marginTop: 0, marginBottom: '0.75rem', fontSize: '1.1rem', display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
                📥 Import
              </h3>
              <p style={{ marginBottom: '1rem', color: 'var(--text-dim)', fontSize: '0.9rem' }}>
                Load settings from a previously exported file.
              </p>

              {/* Merge Mode Toggle */}
              <label
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: '0.5rem',
                  marginBottom: '1rem',
                  cursor: 'pointer',
                  padding: '0.5rem',
                  backgroundColor: 'var(--glass)',
                  borderRadius: '6px',
                }}
              >
                <input
                  type="checkbox"
                  checked={mergeMode}
                  onChange={(e) => setMergeMode(e.target.checked)}
                  disabled={isImporting || !!selectedFile}
                />
                <span>Merge with existing settings</span>
                <span style={{ fontSize: '0.8rem', color: 'var(--text-dim)', marginLeft: 'auto' }}>
                  {mergeMode ? 'Preserves current settings where not specified' : 'Replaces all settings'}
                </span>
              </label>

              {/* File Selection */}
              {!selectedFile ? (
                <button
                  onClick={handleBrowseImport}
                  disabled={isImporting}
                  style={{
                    padding: '0.75rem 1.5rem',
                    display: 'flex',
                    alignItems: 'center',
                    gap: '0.5rem',
                  }}
                >
                  📂 Import Settings...
                </button>
              ) : (
                <div
                  style={{
                    padding: '1rem',
                    backgroundColor: 'var(--glass)',
                    borderRadius: '8px',
                    border: '1px solid var(--border)',
                  }}
                >
                  <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', marginBottom: '0.5rem' }}>
                    <span style={{ fontSize: '1.5rem' }}>📄</span>
                    <span style={{ fontWeight: 500, flex: 1, wordBreak: 'break-all' }}>
                      {selectedFile.split(/[\\/]/).pop()}
                    </span>
                    <button
                      onClick={clearSelection}
                      disabled={isImporting}
                      style={{
                        padding: '0.25rem 0.5rem',
                        fontSize: '0.75rem',
                        backgroundColor: 'transparent',
                        border: '1px solid var(--border)',
                      }}
                    >
                      Change
                    </button>
                  </div>
                  
                  {validationResult && (
                    <div style={{ fontSize: '0.85rem', color: 'var(--text-dim)', marginTop: '0.5rem' }}>
                      <div>Version: {validationResult.version}</div>
                      <div>Exported: {new Date(validationResult.exported_at).toLocaleString()}</div>
                      
                      {!validationResult.version_compatible && (
                        <div style={{ color: '#fbbf24', marginTop: '0.5rem' }}>
                          ⚠️ Version mismatch - some settings may not be compatible
                        </div>
                      )}
                      
                      {validationResult.warnings.length > 0 && (
                        <div style={{ marginTop: '0.5rem' }}>
                          <strong>Warnings:</strong>
                          <ul style={{ margin: '0.25rem 0', paddingLeft: '1rem' }}>
                            {validationResult.warnings.slice(0, 3).map((w, i) => (
                              <li key={i}>{w}</li>
                            ))}
                            {validationResult.warnings.length > 3 && (
                              <li>...and {validationResult.warnings.length - 3} more</li>
                            )}
                          </ul>
                        </div>
                      )}
                    </div>
                  )}
                  
                  <button
                    onClick={handleImport}
                    disabled={isImporting || (validationResult && !validationResult.valid)}
                    style={{
                      marginTop: '1rem',
                      padding: '0.75rem 1.5rem',
                      width: '100%',
                    }}
                  >
                    {isImporting ? '⏳ Importing...' : `Import Settings (${mergeMode ? 'Merge' : 'Replace'})`}
                  </button>
                </div>
              )}
            </div>

            <div style={{ height: '1px', backgroundColor: 'var(--border)', margin: '1.5rem 0' }} />

            {/* Reset Section */}
            <div>
              <h3 style={{ marginTop: 0, marginBottom: '0.75rem', fontSize: '1.1rem', display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
                🔄 Reset
              </h3>
              
              {!showResetConfirm ? (
                <>
                  <p style={{ marginBottom: '1rem', color: 'var(--text-dim)', fontSize: '0.9rem' }}>
                    Restore all settings to their default values. This will erase all custom settings.
                  </p>
                  <button
                    onClick={() => setShowResetConfirm(true)}
                    style={{
                      padding: '0.75rem 1.5rem',
                      backgroundColor: 'var(--accent)',
                      display: 'flex',
                      alignItems: 'center',
                      gap: '0.5rem',
                    }}
                  >
                    🔄 Reset to Defaults...
                  </button>
                </>
              ) : (
                <div
                  style={{
                    padding: '1rem',
                    backgroundColor: 'rgba(220, 38, 38, 0.1)',
                    borderRadius: '8px',
                    border: '1px solid var(--accent)',
                  }}
                >
                  <p style={{ marginTop: 0, color: 'var(--accent)', fontWeight: 500 }}>
                    ⚠️ Are you sure?
                  </p>
                  <p style={{ fontSize: '0.9rem', color: 'var(--text-dim)' }}>
                    This will erase all your custom settings and restore defaults. This cannot be undone.
                  </p>
                  <div style={{ display: 'flex', gap: '0.5rem', marginTop: '1rem' }}>
                    <button
                      onClick={() => setShowResetConfirm(false)}
                      style={{
                        padding: '0.5rem 1rem',
                        backgroundColor: 'transparent',
                        border: '1px solid var(--border)',
                      }}
                    >
                      Cancel
                    </button>
                    <button
                      onClick={handleReset}
                      style={{
                        padding: '0.5rem 1rem',
                        backgroundColor: 'var(--accent)',
                      }}
                    >
                      Yes, Reset Everything
                    </button>
                  </div>
                </div>
              )}
            </div>
          </>
        )}

        {/* Close Button */}
        <div style={{ marginTop: '2rem', display: 'flex', justifyContent: 'flex-end' }}>
          <button
            onClick={onClose}
            style={{
              padding: '0.75rem 1.5rem',
              backgroundColor: 'transparent',
              border: '1px solid var(--border)',
            }}
          >
            Close
          </button>
        </div>
      </div>
    </div>
  );
}

export default SettingsManager;
