import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { open, save } from '@tauri-apps/plugin-dialog';
import { useStore } from '../store/useStore';
import { showToast } from './ToastContainer';
import { 
  LayoutPackInfo, 
  LayoutPack, 
  ValidationReport,
  ExportSelection 
} from '../types/layoutPack';

interface LayoutPackManagerProps {
  onBrowsePack?: (pack: LayoutPackInfo) => void;
}

export const LayoutPackManager = ({ onBrowsePack }: LayoutPackManagerProps) => {
  const { boxers, selectedBoxer } = useStore();
  const [packs, setPacks] = useState<LayoutPackInfo[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [selectedPack, setSelectedPack] = useState<LayoutPackInfo | null>(null);
  const [validationReport, setValidationReport] = useState<ValidationReport | null>(null);
  const [importDialogOpen, setImportDialogOpen] = useState(false);
  const [exportDialogOpen, setExportDialogOpen] = useState(false);
  const [packToImport, setPackToImport] = useState<LayoutPack | null>(null);
  const [importPath, setImportPath] = useState<string>('');
  
  // Export dialog state
  const [exportSelections, setExportSelections] = useState<ExportSelection[]>([]);
  const [exportMetadata, setExportMetadata] = useState({
    name: '',
    author: '',
    description: '',
  });

  const loadPacks = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const availablePacks = await invoke<LayoutPackInfo[]>('get_available_layout_packs');
      setPacks(availablePacks);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadPacks();
  }, [loadPacks]);

  const handleImport = async () => {
    try {
      const selected = await open({
        multiple: false,
        filters: [{ name: 'Layout Pack', extensions: ['json'] }],
      });
      
      if (typeof selected !== 'string') return;
      
      setImportPath(selected);
      const pack = await invoke<LayoutPack>('import_layout_pack', { packPath: selected });
      setPackToImport(pack);
      
      // Validate the pack
      const validation = await invoke<ValidationReport>('validate_layout_pack', { packPath: selected });
      setValidationReport(validation);
      
      setImportDialogOpen(true);
    } catch (e) {
      setError(`Failed to import pack: ${e}`);
    }
  };

  const handleConfirmImport = async () => {
    if (!packToImport || !importPath) return;
    
    try {
      await invoke('install_layout_pack', { sourcePath: importPath });
      await loadPacks();
      setImportDialogOpen(false);
      setPackToImport(null);
      setValidationReport(null);
      setImportPath('');
    } catch (e) {
      setError(`Failed to install pack: ${e}`);
    }
  };

  const handleDelete = async (pack: LayoutPackInfo) => {
    if (!confirm(`Delete layout pack "${pack.name}"?`)) return;
    
    try {
      await invoke('delete_layout_pack', { filename: pack.filename });
      await loadPacks();
      if (selectedPack?.filename === pack.filename) {
        setSelectedPack(null);
      }
    } catch (e) {
      setError(`Failed to delete pack: ${e}`);
    }
  };

  const handleApplyPack = async (pack: LayoutPackInfo) => {
    try {
      const dir = 'data/boxer-layouts/community';
      const packPath = `${dir}/${pack.filename}`;
      
      // Get full pack to see which boxers are included
      const fullPack = await invoke<LayoutPack>('import_layout_pack', { packPath });
      
      const boxerKeys = fullPack.layouts.map(l => l.boxer_key);
      
      if (boxerKeys.length === 0) {
        setError('No boxers in this pack');
        return;
      }
      
      await invoke('apply_layout_pack', { packPath, boxerKeys });
      showToast(`Applied layout pack "${pack.name}" to ${boxerKeys.length} boxer(s).`, 'success');
    } catch (e) {
      setError(`Failed to apply pack: ${e}`);
    }
  };

  const openExportDialog = () => {
    const selections: ExportSelection[] = boxers.map(b => ({
      boxer_key: b.key,
      selected: b.key === selectedBoxer?.key,
      include_shared: false,
    }));
    setExportSelections(selections);
    setExportMetadata({ name: '', author: '', description: '' });
    setExportDialogOpen(true);
  };

  const handleExport = async () => {
    const selectedKeys = exportSelections
      .filter(s => s.selected)
      .map(s => s.boxer_key);
    
    if (selectedKeys.length === 0) {
      setError('Select at least one boxer to export');
      return;
    }
    
    if (!exportMetadata.name.trim()) {
      setError('Pack name is required');
      return;
    }
    
    try {
      const outputPath = await save({
        filters: [{ name: 'Layout Pack', extensions: ['json'] }],
        defaultPath: `${exportMetadata.name.replace(/\s+/g, '_')}.json`,
      });
      
      if (!outputPath) return;
      
      await invoke('export_layout_pack', {
        boxerKeys: selectedKeys,
        metadata: exportMetadata,
        outputPath,
      });
      
      setExportDialogOpen(false);
      showToast('Layout pack exported.', 'success');
    } catch (e) {
      setError(`Failed to export pack: ${e}`);
    }
  };

  const toggleBoxerSelection = (key: string) => {
    setExportSelections(prev => prev.map(s => 
      s.boxer_key === key ? { ...s, selected: !s.selected } : s
    ));
  };

  const formatDate = (dateStr: string) => {
    try {
      return new Date(dateStr).toLocaleDateString();
    } catch {
      return dateStr;
    }
  };

  return (
    <div className="layout-pack-manager">
      {/* Header */}
      <div style={{ marginBottom: '1.5rem' }}>
        <h2 style={{ margin: '0 0 0.5rem' }}>Community Layout Packs</h2>
        <p style={{ color: 'var(--text-dim)', margin: 0 }}>
          Import and export boxer layout configurations to share with the community.
        </p>
      </div>

      {/* Actions */}
      <div style={{ display: 'flex', gap: '0.75rem', marginBottom: '1.5rem', flexWrap: 'wrap' }}>
        <button onClick={handleImport} style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
          <span>📥</span> Import Pack
        </button>
        <button 
          onClick={openExportDialog} 
          style={{ 
            display: 'flex', 
            alignItems: 'center', 
            gap: '6px',
            background: 'var(--blue)',
          }}
        >
          <span>📤</span> Export Layouts
        </button>
        <button 
          onClick={loadPacks} 
          disabled={loading}
          style={{ 
            display: 'flex', 
            alignItems: 'center', 
            gap: '6px',
            background: 'var(--glass)',
          }}
        >
          <span>{loading ? '⏳' : '🔄'}</span> Refresh
        </button>
      </div>

      {/* Error */}
      {error && (
        <div style={{ 
          padding: '12px 16px', 
          background: 'rgba(255, 100, 100, 0.1)', 
          border: '1px solid rgba(255, 100, 100, 0.3)',
          borderRadius: '8px',
          marginBottom: '1rem',
          color: '#ff8888',
        }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
            <span>⚠️</span>
            <span>{error}</span>
          </div>
          <button 
            onClick={() => setError(null)}
            style={{ 
              marginTop: '8px', 
              fontSize: '0.8rem',
              padding: '4px 12px',
            }}
          >
            Dismiss
          </button>
        </div>
      )}

      {/* Pack List */}
      <div style={{ 
        background: 'var(--panel-bg)', 
        borderRadius: '12px',
        border: '1px solid var(--border)',
        overflow: 'hidden',
      }}>
        {packs.length === 0 ? (
          <div style={{ 
            padding: '3rem 2rem', 
            textAlign: 'center',
            color: 'var(--text-dim)',
          }}>
            <div style={{ fontSize: '3rem', marginBottom: '1rem' }}>📦</div>
            <h3 style={{ margin: '0 0 0.5rem', color: 'var(--text)' }}>No Layout Packs</h3>
            <p style={{ margin: 0 }}>
              Import a layout pack or export your current layouts to get started.
            </p>
          </div>
        ) : (
          <div style={{ maxHeight: '400px', overflow: 'auto' }}>
            {packs.map(pack => (
              <div
                key={pack.filename}
                onClick={() => setSelectedPack(selectedPack?.filename === pack.filename ? null : pack)}
                style={{
                  padding: '16px 20px',
                  borderBottom: '1px solid var(--border)',
                  cursor: 'pointer',
                  background: selectedPack?.filename === pack.filename ? 'rgba(100, 150, 255, 0.1)' : 'transparent',
                  transition: 'background 0.15s',
                }}
              >
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start' }}>
                  <div style={{ flex: 1 }}>
                    <h4 style={{ margin: '0 0 4px', fontSize: '1rem' }}>{pack.name}</h4>
                    <div style={{ 
                      fontSize: '0.85rem', 
                      color: 'var(--text-dim)',
                      display: 'flex',
                      gap: '12px',
                      flexWrap: 'wrap',
                    }}>
                      <span>by {pack.author || 'Unknown'}</span>
                      <span>•</span>
                      <span>{pack.boxer_count} boxer{pack.boxer_count !== 1 ? 's' : ''}</span>
                      <span>•</span>
                      <span>{formatDate(pack.created_at)}</span>
                    </div>
                    {pack.description && (
                      <p style={{ 
                        margin: '8px 0 0', 
                        fontSize: '0.85rem', 
                        color: 'var(--text-dim)',
                        lineHeight: 1.4,
                      }}>
                        {pack.description}
                      </p>
                    )}
                  </div>
                  <div style={{ display: 'flex', gap: '8px' }}>
                    <button
                      onClick={(e) => { e.stopPropagation(); handleApplyPack(pack); }}
                      style={{ 
                        padding: '6px 12px', 
                        fontSize: '0.8rem',
                        background: 'var(--green)',
                      }}
                      title="Apply this pack to boxers"
                    >
                      Apply
                    </button>
                    {onBrowsePack && (
                      <button
                        onClick={(e) => { e.stopPropagation(); onBrowsePack(pack); }}
                        style={{ 
                          padding: '6px 12px', 
                          fontSize: '0.8rem',
                          background: 'var(--glass)',
                        }}
                        title="Preview pack contents"
                      >
                        Preview
                      </button>
                    )}
                    <button
                      onClick={(e) => { e.stopPropagation(); handleDelete(pack); }}
                      style={{ 
                        padding: '6px 12px', 
                        fontSize: '0.8rem',
                        background: 'var(--accent)',
                      }}
                      title="Delete this pack"
                    >
                      🗑
                    </button>
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Import Dialog */}
      {importDialogOpen && packToImport && (
        <div style={{
          position: 'fixed',
          inset: 0,
          background: 'rgba(0, 0, 0, 0.7)',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          zIndex: 1000,
          padding: '20px',
        }}>
          <div style={{
            background: 'var(--panel-bg)',
            borderRadius: '16px',
            maxWidth: '600px',
            width: '100%',
            maxHeight: '80vh',
            overflow: 'auto',
            border: '1px solid var(--border)',
          }}>
            <div style={{ padding: '24px' }}>
              <h3 style={{ margin: '0 0 1rem' }}>Import Layout Pack</h3>
              
              <div style={{ marginBottom: '1.5rem' }}>
                <h4 style={{ margin: '0 0 0.5rem', fontSize: '1rem' }}>{packToImport.name}</h4>
                <p style={{ margin: '0 0 0.5rem', color: 'var(--text-dim)', fontSize: '0.9rem' }}>
                  by {packToImport.author || 'Unknown'}
                </p>
                <p style={{ margin: 0, fontSize: '0.9rem' }}>{packToImport.description}</p>
                <div style={{ 
                  marginTop: '12px',
                  padding: '10px 14px',
                  background: 'var(--glass)',
                  borderRadius: '8px',
                  fontSize: '0.85rem',
                }}>
                  <div>Version: {packToImport.version}</div>
                  <div>Created: {formatDate(packToImport.created_at)}</div>
                  <div>Boxers: {packToImport.layouts.length}</div>
                </div>
              </div>

              {/* Validation Results */}
              {validationReport && (
                <div style={{ marginBottom: '1.5rem' }}>
                  <h4 style={{ margin: '0 0 0.75rem', fontSize: '0.9rem' }}>Validation Results</h4>
                  
                  <div style={{
                    padding: '12px 16px',
                    borderRadius: '8px',
                    marginBottom: '12px',
                    background: validationReport.valid ? 'rgba(100, 200, 100, 0.1)' : 'rgba(255, 200, 100, 0.1)',
                    border: `1px solid ${validationReport.valid ? 'rgba(100, 200, 100, 0.3)' : 'rgba(255, 200, 100, 0.3)'}`,
                  }}>
                    <div style={{ 
                      fontWeight: 600,
                      color: validationReport.valid ? '#6bdb7d' : '#ffcc88',
                    }}>
                      {validationReport.valid ? '✓ Pack is valid' : '⚠ Pack has issues'}
                    </div>
                    {!validationReport.version_compatible && (
                      <div style={{ fontSize: '0.85rem', marginTop: '4px', color: '#ffcc88' }}>
                        Version mismatch detected
                      </div>
                    )}
                  </div>

                  {validationReport.warnings.length > 0 && (
                    <div style={{ marginBottom: '12px' }}>
                      <div style={{ fontSize: '0.85rem', color: '#ffcc88', marginBottom: '4px' }}>
                        Warnings ({validationReport.warnings.length}):
                      </div>
                      <ul style={{ margin: 0, paddingLeft: '1.5rem', fontSize: '0.85rem', color: 'var(--text-dim)' }}>
                        {validationReport.warnings.slice(0, 3).map((w, i) => (
                          <li key={i}>{w}</li>
                        ))}
                        {validationReport.warnings.length > 3 && (
                          <li>...and {validationReport.warnings.length - 3} more</li>
                        )}
                      </ul>
                    </div>
                  )}

                  {validationReport.errors.length > 0 && (
                    <div>
                      <div style={{ fontSize: '0.85rem', color: '#ff6666', marginBottom: '4px' }}>
                        Errors ({validationReport.errors.length}):
                      </div>
                      <ul style={{ margin: 0, paddingLeft: '1.5rem', fontSize: '0.85rem', color: '#ff6666' }}>
                        {validationReport.errors.map((e, i) => (
                          <li key={i}>{e}</li>
                        ))}
                      </ul>
                    </div>
                  )}

                  {/* Boxer Validations */}
                  {validationReport.boxer_validations.length > 0 && (
                    <div style={{ marginTop: '12px' }}>
                      <div style={{ fontSize: '0.85rem', color: 'var(--text-dim)', marginBottom: '8px' }}>
                        Boxer Validations:
                      </div>
                      <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
                        {validationReport.boxer_validations.map(bv => (
                          <div 
                            key={bv.boxer_key}
                            style={{
                              padding: '8px 12px',
                              background: 'var(--glass)',
                              borderRadius: '6px',
                              fontSize: '0.85rem',
                              display: 'flex',
                              alignItems: 'center',
                              gap: '8px',
                            }}
                          >
                            <span>{bv.exists_in_manifest ? '✓' : '✗'}</span>
                            <span style={{ flex: 1 }}>{bv.boxer_key}</span>
                            {!bv.bins_valid && <span style={{ color: '#ff6666' }}>bin mismatch</span>}
                            {!bv.size_matches && <span style={{ color: '#ffcc88' }}>size diff</span>}
                          </div>
                        ))}
                      </div>
                    </div>
                  )}
                </div>
              )}

              <div style={{ display: 'flex', gap: '12px', justifyContent: 'flex-end' }}>
                <button 
                  onClick={() => {
                    setImportDialogOpen(false);
                    setPackToImport(null);
                    setValidationReport(null);
                  }}
                  style={{ background: 'var(--glass)' }}
                >
                  Cancel
                </button>
                <button 
                  onClick={handleConfirmImport}
                  disabled={!validationReport?.valid}
                  style={{ 
                    background: validationReport?.valid ? 'var(--green)' : 'var(--border)',
                    opacity: validationReport?.valid ? 1 : 0.5,
                  }}
                >
                  Import Pack
                </button>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Export Dialog */}
      {exportDialogOpen && (
        <div style={{
          position: 'fixed',
          inset: 0,
          background: 'rgba(0, 0, 0, 0.7)',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          zIndex: 1000,
          padding: '20px',
        }}>
          <div style={{
            background: 'var(--panel-bg)',
            borderRadius: '16px',
            maxWidth: '600px',
            width: '100%',
            maxHeight: '80vh',
            overflow: 'auto',
            border: '1px solid var(--border)',
          }}>
            <div style={{ padding: '24px' }}>
              <h3 style={{ margin: '0 0 1rem' }}>Export Layout Pack</h3>
              
              {/* Metadata */}
              <div style={{ marginBottom: '1.5rem' }}>
                <div style={{ marginBottom: '12px' }}>
                  <label style={{ display: 'block', fontSize: '0.85rem', marginBottom: '4px' }}>
                    Pack Name *
                  </label>
                  <input
                    type="text"
                    value={exportMetadata.name}
                    onChange={e => setExportMetadata(prev => ({ ...prev, name: e.target.value }))}
                    placeholder="e.g., HD Sprite Layouts"
                    style={{
                      width: '100%',
                      padding: '10px 14px',
                      borderRadius: '8px',
                      border: '1px solid var(--border)',
                      background: 'var(--glass)',
                      color: 'inherit',
                      fontSize: '0.95rem',
                    }}
                  />
                </div>
                <div style={{ marginBottom: '12px' }}>
                  <label style={{ display: 'block', fontSize: '0.85rem', marginBottom: '4px' }}>
                    Author
                  </label>
                  <input
                    type="text"
                    value={exportMetadata.author}
                    onChange={e => setExportMetadata(prev => ({ ...prev, author: e.target.value }))}
                    placeholder="Your name"
                    style={{
                      width: '100%',
                      padding: '10px 14px',
                      borderRadius: '8px',
                      border: '1px solid var(--border)',
                      background: 'var(--glass)',
                      color: 'inherit',
                      fontSize: '0.95rem',
                    }}
                  />
                </div>
                <div>
                  <label style={{ display: 'block', fontSize: '0.85rem', marginBottom: '4px' }}>
                    Description
                  </label>
                  <textarea
                    value={exportMetadata.description}
                    onChange={e => setExportMetadata(prev => ({ ...prev, description: e.target.value }))}
                    placeholder="Describe your layout pack..."
                    rows={3}
                    style={{
                      width: '100%',
                      padding: '10px 14px',
                      borderRadius: '8px',
                      border: '1px solid var(--border)',
                      background: 'var(--glass)',
                      color: 'inherit',
                      fontSize: '0.95rem',
                      resize: 'vertical',
                    }}
                  />
                </div>
              </div>

              {/* Boxer Selection */}
              <div style={{ marginBottom: '1.5rem' }}>
                <h4 style={{ margin: '0 0 0.75rem', fontSize: '0.9rem' }}>
                  Select Boxers to Export ({exportSelections.filter(s => s.selected).length} selected)
                </h4>
                <div style={{ 
                  maxHeight: '200px', 
                  overflow: 'auto',
                  border: '1px solid var(--border)',
                  borderRadius: '8px',
                }}>
                  {exportSelections.map(selection => {
                    const boxer = boxers.find(b => b.key === selection.boxer_key);
                    if (!boxer) return null;
                    return (
                      <label
                        key={selection.boxer_key}
                        style={{
                          display: 'flex',
                          alignItems: 'center',
                          padding: '10px 14px',
                          borderBottom: '1px solid var(--border)',
                          cursor: 'pointer',
                          background: selection.selected ? 'rgba(100, 150, 255, 0.1)' : 'transparent',
                        }}
                      >
                        <input
                          type="checkbox"
                          checked={selection.selected}
                          onChange={() => toggleBoxerSelection(selection.boxer_key)}
                          style={{ marginRight: '10px' }}
                        />
                        <span style={{ flex: 1 }}>{boxer.name}</span>
                        <span style={{ 
                          fontSize: '0.8rem', 
                          color: 'var(--text-dim)',
                        }}>
                          {boxer.unique_sprite_bins.length} unique
                          {boxer.shared_sprite_bins.length > 0 && ` + ${boxer.shared_sprite_bins.length} shared`}
                        </span>
                      </label>
                    );
                  })}
                </div>
              </div>

              <div style={{ display: 'flex', gap: '12px', justifyContent: 'flex-end' }}>
                <button 
                  onClick={() => setExportDialogOpen(false)}
                  style={{ background: 'var(--glass)' }}
                >
                  Cancel
                </button>
                <button 
                  onClick={handleExport}
                  disabled={exportSelections.filter(s => s.selected).length === 0}
                  style={{ 
                    background: exportSelections.filter(s => s.selected).length > 0 ? 'var(--blue)' : 'var(--border)',
                    opacity: exportSelections.filter(s => s.selected).length > 0 ? 1 : 0.5,
                  }}
                >
                  Export Pack
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default LayoutPackManager;
