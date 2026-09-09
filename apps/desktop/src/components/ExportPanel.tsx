import { useState } from 'react';
import { save } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';
import { useStore } from '../store/useStore';

type PatchFormat = 'ips' | 'bps';

export const ExportPanel = () => {
  const { romSha1, pendingWrites } = useStore();
  const [status, setStatus] = useState<string | null>(null);
  const [isExporting, setIsExporting] = useState(false);
  const [selectedFormat, setSelectedFormat] = useState<PatchFormat>('ips');
  const [showMetadata, setShowMetadata] = useState(false);
  const [author, setAuthor] = useState('');
  const [description, setDescription] = useState('');

  const handleSaveRomAs = async () => {
    if (!romSha1) return;
    const path = await save({
      filters: [{ name: 'SNES ROM', extensions: ['sfc', 'smc'] }],
      defaultPath: 'super_punch_out_modded.sfc',
    });
    if (!path) return;
    setIsExporting(true);
    setStatus(null);
    try {
      await invoke('save_rom_as', { outputPath: path });
      setStatus(`✓ ROM saved to: ${path.split(/[\\/]/).pop()}`);
    } catch (e) {
      setStatus(`✗ Error: ${e}`);
    } finally {
      setIsExporting(false);
    }
  };

  const handleExportIps = async () => {
    if (!romSha1) return;
    const path = await save({
      filters: [{ name: 'IPS Patch', extensions: ['ips'] }],
      defaultPath: 'super_punch_out_mod.ips',
    });
    if (!path) return;
    setIsExporting(true);
    setStatus(null);
    try {
      const patchCount = await invoke<number>('export_ips_patch', { outputPath: path });
      setStatus(`✓ IPS patch exported (${patchCount} edited bin${patchCount !== 1 ? 's' : ''}) → ${path.split(/[\\/]/).pop()}`);
    } catch (e) {
      setStatus(`✗ Error: ${e}`);
    } finally {
      setIsExporting(false);
    }
  };

  const handleExportBps = async () => {
    if (!romSha1) return;
    const path = await save({
      filters: [{ name: 'BPS Patch', extensions: ['bps'] }],
      defaultPath: 'super_punch_out_mod.bps',
    });
    if (!path) return;
    setIsExporting(true);
    setStatus(null);
    try {
      const patchCount = await invoke<number>('export_bps_patch', {
        outputPath: path,
        author: author.trim() || null,
        description: description.trim() || null,
      });
      setStatus(`✓ BPS patch exported (${patchCount} edited bin${patchCount !== 1 ? 's' : ''}) → ${path.split(/[\\/]/).pop()}`);
    } catch (e) {
      setStatus(`✗ Error: ${e}`);
    } finally {
      setIsExporting(false);
    }
  };

  const handleExportPatch = async () => {
    if (selectedFormat === 'ips') {
      await handleExportIps();
    } else {
      await handleExportBps();
    }
  };

  const canExport = !!romSha1;
  const canExportPatch = canExport && pendingWrites.size > 0;

  return (
    <div className="export-panel" style={{
      backgroundColor: 'var(--panel-bg)',
      border: '1px solid var(--border)',
      borderRadius: '12px',
      padding: '1.5rem',
    }}>
      <h3 style={{ margin: '0 0 0.5rem' }}>Export</h3>

      {/* Summary */}
      <div style={{ marginBottom: '1rem', padding: '12px', borderRadius: '8px', background: 'var(--glass)', fontSize: '0.875rem' }}>
        {!romSha1 ? (
          <span style={{ color: 'var(--text-dim)' }}>No ROM loaded. Open a ROM to enable export.</span>
        ) : pendingWrites.size === 0 ? (
          <span style={{ color: 'var(--text-dim)' }}>No pending edits. Import sprite bins or edit palettes to create changes.</span>
        ) : (
          <div>
            <div style={{ fontWeight: 600, marginBottom: '4px', color: '#ffd700' }}>
              ✏️ {pendingWrites.size} edited bin{pendingWrites.size !== 1 ? 's' : ''} staged for export
            </div>
            <div style={{ color: 'var(--text-dim)', fontSize: '0.8rem' }}>
              Pending: {Array.from(pendingWrites).join(', ')}
            </div>
          </div>
        )}
      </div>

      {/* Validation checklist */}
      <div style={{ marginBottom: '1rem', fontSize: '0.8rem', display: 'flex', flexDirection: 'column', gap: '4px' }}>
        <div style={{ color: romSha1 ? '#6bdb7d' : 'var(--text-dim)' }}>
          {romSha1 ? '✓' : '○'} ROM loaded and validated
        </div>
        <div style={{ color: pendingWrites.size > 0 ? '#ffd700' : 'var(--text-dim)' }}>
          {pendingWrites.size > 0 ? '✏' : '○'} {pendingWrites.size} pending write(s)
        </div>
      </div>

      <div style={{ display: 'flex', gap: '0.75rem', flexWrap: 'wrap' }}>
        <button
          id="btn-save-rom-as"
          onClick={handleSaveRomAs}
          disabled={!canExport || isExporting}
          style={{
            padding: '10px 20px',
            fontWeight: 600,
            background: canExport ? 'var(--blue)' : 'var(--border)',
            opacity: isExporting ? 0.6 : 1,
            cursor: canExport ? 'pointer' : 'not-allowed',
          }}
        >
          {isExporting ? '⏳ Saving…' : '💾 Save ROM As…'}
        </button>
      </div>

      {/* Patch Export Section */}
      <div style={{ 
        marginTop: '1.5rem', 
        padding: '1rem', 
        borderRadius: '8px', 
        background: 'var(--glass)',
        border: '1px solid var(--border)'
      }}>
        <div style={{ fontWeight: 600, marginBottom: '0.75rem', fontSize: '0.9rem' }}>
          📦 Export Patch
        </div>

        {/* Format Selection */}
        <div style={{ display: 'flex', gap: '0.5rem', marginBottom: '1rem' }}>
          <button
            onClick={() => setSelectedFormat('ips')}
            disabled={!canExportPatch || isExporting}
            style={{
              flex: 1,
              padding: '8px 16px',
              fontWeight: 600,
              fontSize: '0.85rem',
              background: selectedFormat === 'ips' ? 'var(--blue)' : 'var(--border)',
              opacity: (!canExportPatch || isExporting) ? 0.5 : 1,
              cursor: canExportPatch ? 'pointer' : 'not-allowed',
              borderRadius: '6px',
            }}
          >
            IPS Format
          </button>
          <button
            onClick={() => setSelectedFormat('bps')}
            disabled={!canExportPatch || isExporting}
            style={{
              flex: 1,
              padding: '8px 16px',
              fontWeight: 600,
              fontSize: '0.85rem',
              background: selectedFormat === 'bps' ? 'var(--blue)' : 'var(--border)',
              opacity: (!canExportPatch || isExporting) ? 0.5 : 1,
              cursor: canExportPatch ? 'pointer' : 'not-allowed',
              borderRadius: '6px',
            }}
          >
            BPS Format
          </button>
        </div>

        {/* Format Description */}
        <div style={{ 
          marginBottom: '1rem', 
          fontSize: '0.78rem', 
          color: 'var(--text-dim)',
          padding: '8px 12px',
          background: 'rgba(0,0,0,0.2)',
          borderRadius: '6px',
        }}>
          {selectedFormat === 'ips' ? (
            <span>
              <strong>IPS</strong> — Classic format, widely supported. No metadata.
            </span>
          ) : (
            <span>
              <strong>BPS</strong> — Modern format with better compression and metadata support.
            </span>
          )}
        </div>

        {/* BPS Metadata Toggle */}
        {selectedFormat === 'bps' && canExportPatch && (
          <div style={{ marginBottom: '1rem' }}>
            <button
              onClick={() => setShowMetadata(!showMetadata)}
              style={{
                padding: '6px 12px',
                fontSize: '0.8rem',
                background: 'transparent',
                border: '1px solid var(--border)',
                borderRadius: '4px',
                cursor: 'pointer',
                color: 'var(--text-dim)',
              }}
            >
              {showMetadata ? '▼' : '▶'} Metadata (optional)
            </button>
            
            {showMetadata && (
              <div style={{ 
                marginTop: '0.75rem',
                padding: '0.75rem',
                background: 'rgba(0,0,0,0.2)',
                borderRadius: '6px',
                display: 'flex',
                flexDirection: 'column',
                gap: '0.5rem',
              }}>
                <div>
                  <label style={{ 
                    display: 'block', 
                    fontSize: '0.75rem', 
                    color: 'var(--text-dim)',
                    marginBottom: '0.25rem',
                  }}>
                    Author
                  </label>
                  <input
                    type="text"
                    value={author}
                    onChange={(e) => setAuthor(e.target.value)}
                    placeholder="Your name"
                    maxLength={64}
                    style={{
                      width: '100%',
                      padding: '6px 10px',
                      fontSize: '0.85rem',
                      background: 'var(--panel-bg)',
                      border: '1px solid var(--border)',
                      borderRadius: '4px',
                      color: 'var(--text)',
                      boxSizing: 'border-box',
                    }}
                  />
                </div>
                <div>
                  <label style={{ 
                    display: 'block', 
                    fontSize: '0.75rem', 
                    color: 'var(--text-dim)',
                    marginBottom: '0.25rem',
                  }}>
                    Description
                  </label>
                  <textarea
                    value={description}
                    onChange={(e) => setDescription(e.target.value)}
                    placeholder="Brief description of changes..."
                    maxLength={256}
                    rows={2}
                    style={{
                      width: '100%',
                      padding: '6px 10px',
                      fontSize: '0.85rem',
                      background: 'var(--panel-bg)',
                      border: '1px solid var(--border)',
                      borderRadius: '4px',
                      color: 'var(--text)',
                      boxSizing: 'border-box',
                      resize: 'vertical',
                      fontFamily: 'inherit',
                    }}
                  />
                </div>
              </div>
            )}
          </div>
        )}

        {/* Export Button */}
        <button
          id={selectedFormat === 'ips' ? 'btn-export-ips' : 'btn-export-bps'}
          onClick={handleExportPatch}
          disabled={!canExportPatch || isExporting}
          style={{
            width: '100%',
            padding: '10px 20px',
            fontWeight: 600,
            background: canExportPatch ? 'var(--blue)' : 'var(--border)',
            opacity: (!canExportPatch || isExporting) ? 0.5 : 1,
            cursor: canExportPatch ? 'pointer' : 'not-allowed',
            borderRadius: '6px',
          }}
          title={pendingWrites.size === 0 ? 'Make some edits first to export a patch' : `Export ${selectedFormat.toUpperCase()} patch of all staged changes`}
        >
          {isExporting 
            ? '⏳ Exporting…' 
            : `Export ${selectedFormat.toUpperCase()} Patch`
          }
        </button>
      </div>

      {status && (
        <div style={{
          marginTop: '1rem',
          padding: '10px 14px',
          borderRadius: '8px',
          background: status.startsWith('✓') ? 'rgba(107,219,125,0.1)' : 'rgba(255,80,80,0.1)',
          border: `1px solid ${status.startsWith('✓') ? 'rgba(107,219,125,0.3)' : 'rgba(255,80,80,0.3)'}`,
          color: status.startsWith('✓') ? '#6bdb7d' : '#ff6666',
          fontSize: '0.85rem',
          fontFamily: 'monospace',
        }}>
          {status}
        </div>
      )}

      <div style={{ marginTop: '1rem', fontSize: '0.78rem', color: 'var(--text-dim)' }}>
        Both IPS and BPS patches require the same unmodified base ROM to apply. 
        BPS is recommended for better compatibility and metadata support.
      </div>
    </div>
  );
};
