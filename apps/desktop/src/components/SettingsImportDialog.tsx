import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';

interface SettingsChangePreview {
  category: string;
  key: string;
  display_name: string;
  current_value: unknown;
  new_value: unknown;
  will_change: boolean;
  has_conflict: boolean;
}

interface SettingsImportDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onImport: (filePath: string, merge: boolean) => void;
}

export function SettingsImportDialog({ isOpen, onClose, onImport }: SettingsImportDialogProps) {
  const [selectedFile, setSelectedFile] = useState<string | null>(null);
  const [preview, setPreview] = useState<SettingsChangePreview[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [mergeMode, setMergeMode] = useState(true);
  const [selectedChanges, setSelectedChanges] = useState<Set<string>>(new Set());
  const [showAll, setShowAll] = useState(false);

  // Reset state when dialog opens
  useEffect(() => {
    if (isOpen) {
      setSelectedFile(null);
      setPreview([]);
      setError(null);
      setSelectedChanges(new Set());
      setShowAll(false);
      setMergeMode(true);
    }
  }, [isOpen]);

  const handleBrowse = async () => {
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
        setLoading(true);
        setError(null);

        try {
          const previewData = await invoke<SettingsChangePreview[]>('preview_settings_import', {
            settingsPath: selected,
          });
          setPreview(previewData);
          
          // Auto-select all changes by default
          const allKeys = new Set(previewData.filter(p => p.will_change).map(p => p.key));
          setSelectedChanges(allKeys);
        } catch (e) {
          setError(`Failed to preview settings: ${e}`);
        } finally {
          setLoading(false);
        }
      }
    } catch (e) {
      setError(`Failed to browse for file: ${e}`);
    }
  };

  const toggleChange = (key: string) => {
    const newSelected = new Set(selectedChanges);
    if (newSelected.has(key)) {
      newSelected.delete(key);
    } else {
      newSelected.add(key);
    }
    setSelectedChanges(newSelected);
  };

  const selectAll = () => {
    const allKeys = new Set(preview.filter(p => p.will_change).map(p => p.key));
    setSelectedChanges(allKeys);
  };

  const deselectAll = () => {
    setSelectedChanges(new Set());
  };

  const handleImport = () => {
    if (selectedFile) {
      onImport(selectedFile, mergeMode);
    }
  };

  const formatValue = (value: unknown): string => {
    if (value === null || value === undefined) return 'Not set';
    if (typeof value === 'boolean') return value ? 'Yes' : 'No';
    if (typeof value === 'number') return value.toString();
    if (typeof value === 'string') {
      if (value.length > 50) return value.substring(0, 50) + '...';
      return value || '(empty)';
    }
    if (Array.isArray(value)) {
      if (value.length === 0) return 'None';
      return `${value.length} items`;
    }
    if (typeof value === 'object') {
      const keys = Object.keys(value as object);
      if (keys.length === 0) return 'None';
      return `${keys.length} entries`;
    }
    return String(value);
  };

  const getCategoryIcon = (category: string): string => {
    switch (category) {
      case 'Appearance': return '🎨';
      case 'Editor': return '✏️';
      case 'Emulator': return '🎮';
      case 'Paths': return '📁';
      case 'External Tools': return '🛠️';
      case 'Keyboard Shortcuts': return '⌨️';
      case 'Layout': return '📐';
      default: return '⚙️';
    }
  };

  const getChangeType = (item: SettingsChangePreview): { icon: string; color: string; label: string } => {
    if (item.has_conflict) {
      return { icon: '⚠️', color: '#fbbf24', label: 'Conflict' };
    }
    if (!item.current_value && item.new_value) {
      return { icon: '➕', color: '#22c55e', label: 'New' };
    }
    if (item.current_value && !item.new_value) {
      return { icon: '🗑️', color: '#ef4444', label: 'Remove' };
    }
    return { icon: '↻', color: '#3b82f6', label: 'Update' };
  };

  // Group changes by category
  const groupedChanges = preview.reduce((acc, item) => {
    if (!acc[item.category]) {
      acc[item.category] = [];
    }
    acc[item.category].push(item);
    return acc;
  }, {} as Record<string, SettingsChangePreview[]>);

  const categories = Object.keys(groupedChanges).sort();

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
          padding: '1.5rem',
          width: '100%',
          maxWidth: '800px',
          maxHeight: '90vh',
          overflow: 'auto',
          display: 'flex',
          flexDirection: 'column',
        }}
        onClick={(e) => e.stopPropagation()}
      >
        <h2 style={{ marginTop: 0, marginBottom: '1rem', display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
          📥 Import Settings
        </h2>

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

        {/* File Selection */}
        <div style={{ marginBottom: '1.5rem' }}>
          <label style={{ display: 'block', marginBottom: '0.5rem', fontWeight: 500 }}>
            Settings File
          </label>
          <div style={{ display: 'flex', gap: '0.5rem' }}>
            <input
              type="text"
              value={selectedFile || ''}
              readOnly
              placeholder="Select a settings file..."
              style={{
                flex: 1,
                padding: '0.5rem',
                borderRadius: '6px',
                border: '1px solid var(--border)',
                backgroundColor: 'var(--glass)',
                color: 'var(--text)',
              }}
            />
            <button onClick={handleBrowse} disabled={loading}>
              {loading ? '⏳' : '📂 Browse...'}
            </button>
          </div>
        </div>

        {/* Merge Mode */}
        <div style={{ marginBottom: '1.5rem' }}>
          <label
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: '0.5rem',
              cursor: 'pointer',
              padding: '0.75rem',
              backgroundColor: 'var(--glass)',
              borderRadius: '6px',
            }}
          >
            <input
              type="checkbox"
              checked={mergeMode}
              onChange={(e) => setMergeMode(e.target.checked)}
            />
            <span style={{ fontWeight: 500 }}>Merge with existing settings</span>
            <span style={{ fontSize: '0.8rem', color: 'var(--text-dim)', marginLeft: 'auto' }}>
              {mergeMode ? 'Preserves current settings' : 'Replaces all settings'}
            </span>
          </label>
        </div>

        {/* Changes Preview */}
        {preview.length > 0 && (
          <div style={{ flex: 1, overflow: 'auto', marginBottom: '1.5rem' }}>
            <div
              style={{
                display: 'flex',
                justifyContent: 'space-between',
                alignItems: 'center',
                marginBottom: '0.75rem',
              }}
            >
              <h3 style={{ margin: 0, fontSize: '1rem' }}>
                Changes Preview
                <span style={{ fontSize: '0.85rem', color: 'var(--text-dim)', marginLeft: '0.5rem' }}>
                  ({selectedChanges.size} of {preview.filter(p => p.will_change).length} selected)
                </span>
              </h3>
              <div style={{ display: 'flex', gap: '0.5rem' }}>
                <button onClick={selectAll} style={{ padding: '0.25rem 0.5rem', fontSize: '0.8rem' }}>
                  Select All
                </button>
                <button onClick={deselectAll} style={{ padding: '0.25rem 0.5rem', fontSize: '0.8rem' }}>
                  Deselect All
                </button>
              </div>
            </div>

            {/* Legend */}
            <div
              style={{
                display: 'flex',
                gap: '1rem',
                fontSize: '0.8rem',
                color: 'var(--text-dim)',
                marginBottom: '0.75rem',
                padding: '0.5rem',
                backgroundColor: 'var(--glass)',
                borderRadius: '6px',
              }}
            >
              <span>➕ New</span>
              <span>↻ Update</span>
              <span>⚠️ Conflict</span>
              <span>🗑️ Remove</span>
              <span>○ No change</span>
            </div>

            {/* Changes List */}
            <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem' }}>
              {categories.map((category) => {
                const items = groupedChanges[category];
                const visibleItems = showAll ? items : items.filter((i) => i.will_change);
                const hasHidden = items.some((i) => !i.will_change);

                if (visibleItems.length === 0) return null;

                return (
                  <div key={category} style={{ marginBottom: '0.75rem' }}>
                    <div
                      style={{
                        display: 'flex',
                        alignItems: 'center',
                        gap: '0.5rem',
                        padding: '0.5rem',
                        backgroundColor: 'var(--glass)',
                        borderRadius: '6px 6px 0 0',
                        fontWeight: 500,
                        fontSize: '0.9rem',
                      }}
                    >
                      <span>{getCategoryIcon(category)}</span>
                      <span>{category}</span>
                      <span style={{ fontSize: '0.8rem', color: 'var(--text-dim)', marginLeft: 'auto' }}>
                        {visibleItems.length} change{visibleItems.length !== 1 ? 's' : ''}
                      </span>
                    </div>
                    <div
                      style={{
                        border: '1px solid var(--border)',
                        borderTop: 'none',
                        borderRadius: '0 0 6px 6px',
                        overflow: 'hidden',
                      }}
                    >
                      {visibleItems.map((item, index) => {
                        const changeType = getChangeType(item);
                        const isSelected = selectedChanges.has(item.key);
                        const showCheckbox = item.will_change;

                        return (
                          <div
                            key={item.key}
                            style={{
                              display: 'flex',
                              alignItems: 'center',
                              gap: '0.75rem',
                              padding: '0.75rem',
                              backgroundColor: index % 2 === 0 ? 'transparent' : 'rgba(0,0,0,0.1)',
                              borderBottom: index < visibleItems.length - 1 ? '1px solid var(--border)' : 'none',
                              opacity: showCheckbox && !isSelected ? 0.5 : 1,
                            }}
                          >
                            {showCheckbox && (
                              <input
                                type="checkbox"
                                checked={isSelected}
                                onChange={() => toggleChange(item.key)}
                                style={{ cursor: 'pointer' }}
                              />
                            )}
                            {!showCheckbox && <span style={{ width: '16px' }}>○</span>}

                            <span style={{ fontSize: '1rem' }}>{changeType.icon}</span>

                            <div style={{ flex: 1, minWidth: 0 }}>
                              <div style={{ fontWeight: 500, marginBottom: '0.25rem' }}>
                                {item.display_name}
                              </div>
                              <div
                                style={{
                                  display: 'grid',
                                  gridTemplateColumns: '1fr auto 1fr',
                                  gap: '0.5rem',
                                  alignItems: 'center',
                                  fontSize: '0.8rem',
                                  color: 'var(--text-dim)',
                                }}
                              >
                                <span
                                  style={{
                                    textDecoration: item.will_change ? 'line-through' : 'none',
                                    opacity: item.will_change ? 0.6 : 1,
                                  }}
                                >
                                  {formatValue(item.current_value)}
                                </span>
                                {item.will_change && (
                                  <span style={{ color: changeType.color }}>→</span>
                                )}
                                {item.will_change && (
                                  <span style={{ color: changeType.color, fontWeight: 500 }}>
                                    {formatValue(item.new_value)}
                                  </span>
                                )}
                              </div>
                            </div>

                            <span
                              style={{
                                fontSize: '0.75rem',
                                padding: '0.125rem 0.375rem',
                                borderRadius: '4px',
                                backgroundColor: changeType.color + '20',
                                color: changeType.color,
                              }}
                            >
                              {changeType.label}
                            </span>
                          </div>
                        );
                      })}
                    </div>
                  </div>
                );
              })}
            </div>

            {/* Show All Toggle */}
            {preview.some((p) => !p.will_change) && (
              <button
                onClick={() => setShowAll(!showAll)}
                style={{
                  marginTop: '0.5rem',
                  padding: '0.5rem',
                  fontSize: '0.85rem',
                  backgroundColor: 'transparent',
                  border: '1px dashed var(--border)',
                  width: '100%',
                }}
              >
                {showAll ? 'Hide unchanged settings' : 'Show all settings'}
              </button>
            )}
          </div>
        )}

        {/* Empty State */}
        {preview.length === 0 && selectedFile && !loading && !error && (
          <div
            style={{
              padding: '2rem',
              textAlign: 'center',
              color: 'var(--text-dim)',
              backgroundColor: 'var(--glass)',
              borderRadius: '8px',
              marginBottom: '1.5rem',
            }}
          >
            No changes detected in this settings file.
          </div>
        )}

        {/* Initial State */}
        {!selectedFile && !loading && (
          <div
            style={{
              padding: '3rem',
              textAlign: 'center',
              color: 'var(--text-dim)',
              backgroundColor: 'var(--glass)',
              borderRadius: '8px',
              marginBottom: '1.5rem',
            }}
          >
            <div style={{ fontSize: '3rem', marginBottom: '1rem' }}>📂</div>
            <div>Select a settings file to see a preview of changes</div>
          </div>
        )}

        {/* Actions */}
        <div style={{ display: 'flex', justifyContent: 'flex-end', gap: '0.75rem' }}>
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
            onClick={handleImport}
            disabled={!selectedFile || selectedChanges.size === 0 || loading}
            style={{
              padding: '0.75rem 1.5rem',
              opacity: !selectedFile || selectedChanges.size === 0 ? 0.5 : 1,
            }}
          >
            {loading ? '⏳ Importing...' : `Import Selected (${selectedChanges.size})`}
          </button>
        </div>
      </div>
    </div>
  );
}

export default SettingsImportDialog;
