import { useState, useEffect, useRef, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { save, open } from '@tauri-apps/plugin-dialog';
import { useStore, BoxerRecord } from '../store/useStore';
import { SharedBankSummary } from './SharedBankIndicator';
import { LayoutPackInfo } from '../types/layoutPack';

interface BinLayoutInfo {
  region?: string;
  description?: string;
  priority?: number;
}

interface BoxerLayout {
  fighter?: string;
  tier?: number;
  notes?: string;
  bin_labels?: Record<string, BinLayoutInfo>;
}

interface BoxerPreviewSheetProps {
  boxer: BoxerRecord;
}

interface SharedBankPair {
  fighters: string[];
  note: string;
}

export const BoxerPreviewSheet = ({ boxer }: BoxerPreviewSheetProps) => {
  const { romSha1 } = useStore();
  const [imageSrc, setImageSrc] = useState<string | null>(null);
  const [layout, setLayout] = useState<BoxerLayout | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [includeShared, setIncludeShared] = useState(false);
  const [zoom, setZoom] = useState(2);
  const [sharedPairs, setSharedPairs] = useState<SharedBankPair[]>([]);
  const [availablePacks, setAvailablePacks] = useState<LayoutPackInfo[]>([]);
  const [showPackMenu, setShowPackMenu] = useState(false);
  const blobUrlRef = useRef<string | null>(null);
  const packMenuRef = useRef<HTMLDivElement>(null);

  const totalBins = boxer.unique_sprite_bins.length + (includeShared ? boxer.shared_sprite_bins.length : 0);
  
  // Close pack menu when clicking outside
  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (packMenuRef.current && !packMenuRef.current.contains(e.target as Node)) {
        setShowPackMenu(false);
      }
    };
    
    if (showPackMenu) {
      document.addEventListener('mousedown', handleClickOutside);
      return () => document.removeEventListener('mousedown', handleClickOutside);
    }
  }, [showPackMenu]);

  // Load curated layout metadata and shared pairs
  useEffect(() => {
    invoke<BoxerLayout | null>('get_boxer_layout', { boxerKey: boxer.key })
      .then(l => setLayout(l))
      .catch(() => setLayout(null));

    // Load shared pairs info
    loadSharedPairs();
    
    // Load available layout packs
    loadAvailablePacks();
  }, [boxer.key]);
  
  const loadAvailablePacks = async () => {
    try {
      const packs = await invoke<LayoutPackInfo[]>('get_available_layout_packs');
      setAvailablePacks(packs);
    } catch (e) {
      console.error('Failed to load layout packs:', e);
    }
  };

  const loadSharedPairs = async () => {
    try {
      const layouts = await invoke<{ shared_pairs?: SharedBankPair[] }>('get_all_layouts');
      setSharedPairs(layouts.shared_pairs || []);
    } catch (e) {
      console.error('Failed to load shared pairs:', e);
    }
  };

  const renderSheet = useCallback(async () => {
    if (!romSha1) { setError('Load a ROM first'); return; }
    setLoading(true);
    setError(null);
    try {
      const bytes = await invoke<number[]>('render_sprite_sheet', {
        boxerKey: boxer.key,
        includeShared,
      });
      // Revoke old URL
      if (blobUrlRef.current) URL.revokeObjectURL(blobUrlRef.current);
      const blob = new Blob([new Uint8Array(bytes)], { type: 'image/png' });
      const url = URL.createObjectURL(blob);
      blobUrlRef.current = url;
      setImageSrc(url);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, [boxer.key, romSha1, includeShared]);

  // Auto-render when boxer changes and ROM is loaded
  useEffect(() => {
    if (romSha1 && boxer.unique_sprite_bins.length > 0) {
      renderSheet();
    } else {
      setImageSrc(null);
    }
  }, [boxer.key, romSha1]);

  const handleExportSheet = async () => {
    if (!imageSrc) return;
    const path = await save({
      filters: [{ name: 'PNG Image', extensions: ['png'] }],
      defaultPath: `${boxer.name}_sprite_sheet.png`,
    });
    if (!path) return;
    // Fetch the blob and save manually via Tauri
    try {
      const bytes = await invoke<number[]>('render_sprite_sheet', {
        boxerKey: boxer.key,
        includeShared,
      });
      // Write via CLI workaround: invoke a generic "save bytes" — or re-use export_sprite_bin approach
      // For now, instruct user to right-click → save on the rendered image
      // This is a known limitation; proper save is a follow-up
      alert(`Sheet rendered. Right-click the sheet image and "Save image as…" to save to disk.\nPath: ${path}`);
    } catch (e) {
      setError(String(e));
    }
  };
  
  // Export layout for this boxer as a layout pack
  const handleExportLayout = async () => {
    const path = await save({
      filters: [{ name: 'Layout Pack', extensions: ['json'] }],
      defaultPath: `${boxer.name.replace(/\s+/g, '_')}_layout.json`,
    });
    if (!path) return;
    
    try {
      await invoke('export_layout_pack', {
        boxerKeys: [boxer.key],
        metadata: {
          name: `${boxer.name} Layout`,
          author: '',
          description: `Layout pack for ${boxer.name} exported from SPO!! Editor`,
        },
        outputPath: path,
      });
      alert(`Layout for ${boxer.name} exported successfully!`);
    } catch (e) {
      setError(`Failed to export layout: ${e}`);
    }
  };
  
  // Apply a layout pack to this boxer
  const handleApplyPack = async (pack: LayoutPackInfo) => {
    try {
      await invoke('apply_layout_pack', {
        packPath: `data/boxer-layouts/community/${pack.filename}`,
        boxerKeys: [boxer.key],
      });
      alert(`Applied layout from "${pack.name}" to ${boxer.name}`);
      setShowPackMenu(false);
    } catch (e) {
      setError(`Failed to apply layout pack: ${e}`);
    }
  };
  
  // Import and apply a layout pack
  const handleImportAndApply = async () => {
    try {
      const selected = await open({
        multiple: false,
        filters: [{ name: 'Layout Pack', extensions: ['json'] }],
      });
      
      if (typeof selected !== 'string') return;
      
      // Validate first
      const validation = await invoke<{
        valid: boolean;
        warnings: string[];
        boxer_validations: Array<{ boxer_key: string; exists_in_manifest: boolean }>;
      }>('validate_layout_pack', { packPath: selected });
      
      if (!validation.valid && !confirm('This pack has validation issues. Apply anyway?')) {
        return;
      }
      
      // Check if this boxer is in the pack
      const boxerInPack = validation.boxer_validations.find(b => b.boxer_key === boxer.key);
      if (!boxerInPack) {
        alert(`This pack doesn't contain layout for ${boxer.name}`);
        return;
      }
      
      await invoke('apply_layout_pack', {
        packPath: selected,
        boxerKeys: [boxer.key],
      });
      
      alert(`Layout pack applied to ${boxer.name}`);
    } catch (e) {
      setError(`Failed to import and apply layout: ${e}`);
    }
  };

  const hasLayout = layout !== null;

  // Get pair info for this boxer
  const pairInfo = sharedPairs.find(pair =>
    pair.fighters.some(f => f.toLowerCase() === boxer.name.toLowerCase())
  );

  // Get all shared fighters for this boxer
  const allSharedFighters = new Set<string>();
  boxer.shared_sprite_bins.forEach(bin => {
    bin.shared_with.forEach(f => {
      if (f.toLowerCase() !== boxer.name.toLowerCase()) {
        allSharedFighters.add(f);
      }
    });
  });
  const sharedFighterList = Array.from(allSharedFighters);

  return (
    <div className="boxer-preview-sheet">
      {/* Header */}
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', marginBottom: '1rem', flexWrap: 'wrap', gap: '0.5rem' }}>
        <div>
          <h3 style={{ margin: 0 }}>Reference Sheet</h3>
          <p style={{ margin: '4px 0 0', color: 'var(--text-dim)', fontSize: '0.85rem' }}>
            Assembled tile view · {boxer.unique_sprite_bins.length} unique
            {boxer.shared_sprite_bins.length > 0 && ` + ${boxer.shared_sprite_bins.length} shared`} bins
          </p>
        </div>
        <div style={{ display: 'flex', gap: '0.5rem', flexWrap: 'wrap' }}>
          <button
            id={`btn-render-sheet-${boxer.key.replace(/\s+/g, '-')}`}
            onClick={renderSheet}
            disabled={loading || !romSha1}
            style={{ padding: '6px 14px', fontSize: '0.85rem', opacity: loading ? 0.7 : 1 }}
          >
            {loading ? '⏳ Rendering…' : '🖼 Render Sheet'}
          </button>
          {imageSrc && (
            <button
              id="btn-export-sheet"
              onClick={handleExportSheet}
              style={{ padding: '6px 14px', fontSize: '0.85rem', background: 'var(--border)' }}
            >
              ↓ Export PNG
            </button>
          )}
          <button
            onClick={handleExportLayout}
            style={{ padding: '6px 14px', fontSize: '0.85rem', background: 'var(--blue)' }}
            title="Export this boxer's layout configuration"
          >
            📤 Export Layout
          </button>
          <div ref={packMenuRef} style={{ position: 'relative' }}>
            <button
              onClick={() => setShowPackMenu(!showPackMenu)}
              style={{ padding: '6px 14px', fontSize: '0.85rem', background: 'var(--glass)' }}
            >
              📦 Apply Pack ▼
            </button>
            {showPackMenu && (
              <div style={{
                position: 'absolute',
                top: '100%',
                right: 0,
                marginTop: '4px',
                background: 'var(--panel-bg)',
                border: '1px solid var(--border)',
                borderRadius: '8px',
                minWidth: '220px',
                zIndex: 100,
                boxShadow: '0 4px 20px rgba(0,0,0,0.3)',
              }}>
                <div style={{ 
                  padding: '8px 12px',
                  borderBottom: '1px solid var(--border)',
                  fontSize: '0.8rem',
                  color: 'var(--text-dim)',
                }}>
                  Available Packs
                </div>
                {availablePacks.length === 0 ? (
                  <div style={{ padding: '12px', fontSize: '0.85rem', color: 'var(--text-dim)' }}>
                    No packs installed
                  </div>
                ) : (
                  availablePacks.map(pack => (
                    <button
                      key={pack.filename}
                      onClick={() => handleApplyPack(pack)}
                      style={{
                        display: 'block',
                        width: '100%',
                        padding: '10px 12px',
                        textAlign: 'left',
                        background: 'transparent',
                        border: 'none',
                        borderBottom: '1px solid var(--border)',
                        color: 'inherit',
                        fontSize: '0.85rem',
                        cursor: 'pointer',
                      }}
                    >
                      <div style={{ fontWeight: 500 }}>{pack.name}</div>
                      <div style={{ fontSize: '0.75rem', color: 'var(--text-dim)' }}>
                        {pack.boxer_count} boxer{pack.boxer_count !== 1 ? 's' : ''}
                      </div>
                    </button>
                  ))
                )}
                <button
                  onClick={handleImportAndApply}
                  style={{
                    display: 'block',
                    width: '100%',
                    padding: '10px 12px',
                    textAlign: 'left',
                    background: 'transparent',
                    border: 'none',
                    color: 'var(--blue)',
                    fontSize: '0.85rem',
                    cursor: 'pointer',
                    borderTop: availablePacks.length > 0 ? '1px solid var(--border)' : 'none',
                  }}
                >
                  + Import Pack...
                </button>
              </div>
            )}
          </div>
        </div>
      </div>

      {/* Layout metadata badge */}
      {hasLayout && (
        <div style={{
          marginBottom: '0.75rem',
          padding: '10px 14px',
          borderRadius: '8px',
          background: 'rgba(107,219,125,0.08)',
          border: '1px solid rgba(107,219,125,0.25)',
          fontSize: '0.82rem',
          color: '#6bdb7d',
        }}>
          ✓ Curated layout available
          {layout?.tier && <span style={{ marginLeft: '8px', opacity: 0.7 }}>Tier {layout.tier}</span>}
          {layout?.notes && <span style={{ marginLeft: '8px', color: 'var(--text-dim)' }}>— {layout.notes}</span>}
        </div>
      )}

      {/* Shared bank summary */}
      {boxer.shared_sprite_bins.length > 0 && (
        <div style={{ marginBottom: '0.75rem' }}>
          <SharedBankSummary
            uniqueCount={boxer.unique_sprite_bins.length}
            sharedCount={boxer.shared_sprite_bins.length}
            sharedBins={boxer.shared_sprite_bins}
            currentBoxer={boxer.name}
          />
        </div>
      )}

      {/* Detailed shared bank info */}
      {sharedFighterList.length > 0 && pairInfo && (
        <div style={{
          marginBottom: '0.75rem',
          padding: '12px 14px',
          borderRadius: '8px',
          background: 'rgba(255, 200, 100, 0.05)',
          border: '1px solid rgba(255, 200, 100, 0.2)',
          fontSize: '0.82rem',
        }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px', marginBottom: '8px' }}>
            <span style={{ fontSize: '1.1rem' }}>👥</span>
            <span style={{ fontWeight: 600, color: '#ffcc88' }}>
              Shared Bank Pair
            </span>
          </div>
          <div style={{ color: '#ffccaa', marginBottom: '6px' }}>
            {boxer.name} ↔ {sharedFighterList.join(', ')}
          </div>
          {pairInfo.note && (
            <div style={{ color: 'var(--text-dim)', fontSize: '0.78rem', fontStyle: 'italic' }}>
              {pairInfo.note}
            </div>
          )}
        </div>
      )}

      {/* Controls */}
      <div style={{ display: 'flex', alignItems: 'center', gap: '1rem', marginBottom: '0.75rem', flexWrap: 'wrap', fontSize: '0.85rem' }}>
        {boxer.shared_sprite_bins.length > 0 && (
          <label style={{ display: 'flex', alignItems: 'center', gap: '6px', cursor: 'pointer' }}>
            <input
              type="checkbox"
              checked={includeShared}
              onChange={e => setIncludeShared(e.target.checked)}
            />
            <span style={{ color: includeShared ? '#ff8888' : 'inherit' }}>
              Include shared bins in sheet
            </span>
          </label>
        )}
        <label style={{ display: 'flex', alignItems: 'center', gap: '6px', cursor: 'pointer' }}>
          Zoom:
          <select
            value={zoom}
            onChange={e => setZoom(Number(e.target.value))}
            style={{ padding: '2px 6px', borderRadius: '4px', background: 'var(--glass)' }}
          >
            <option value={1}>1×</option>
            <option value={2}>2×</option>
            <option value={3}>3×</option>
            <option value={4}>4×</option>
          </select>
        </label>
        {imageSrc && (
          <span style={{ color: 'var(--text-dim)' }}>
            {totalBins} bin{totalBins !== 1 ? 's' : ''} · 16 tiles wide
          </span>
        )}
      </div>

      {/* Sheet viewer */}
      <div style={{
        background: '#0c0d14',
        border: '1px solid var(--border)',
        borderRadius: '10px',
        overflow: 'auto',
        maxHeight: '600px',
        minHeight: '200px',
        display: 'flex',
        alignItems: imageSrc ? 'flex-start' : 'center',
        justifyContent: imageSrc ? 'flex-start' : 'center',
        padding: '12px',
        position: 'relative',
      }}>
        {loading && (
          <div style={{
            position: 'absolute', inset: 0, display: 'flex', alignItems: 'center', justifyContent: 'center',
            background: 'rgba(12,13,20,0.7)', zIndex: 10, borderRadius: '10px'
          }}>
            <div style={{ textAlign: 'center' }}>
              <div style={{ fontSize: '2rem', marginBottom: '0.5rem' }}>⏳</div>
              <div style={{ color: 'var(--text-dim)', fontSize: '0.85rem' }}>Decoding tiles…</div>
            </div>
          </div>
        )}

        {error && !loading && (
          <div style={{ textAlign: 'center', color: '#ff6666', fontSize: '0.875rem' }}>
            <div style={{ fontSize: '1.5rem', marginBottom: '0.5rem' }}>✗</div>
            {error}
          </div>
        )}

        {!imageSrc && !loading && !error && (
          <div style={{ textAlign: 'center', color: 'var(--text-dim)', fontSize: '0.875rem' }}>
            {!romSha1 ? 'Load a ROM to render the sprite sheet.' :
             boxer.unique_sprite_bins.length === 0 ? 'No unique sprite bins for this fighter.' :
             'Click "Render Sheet" to decode and display all sprite bins.'}
          </div>
        )}

        {imageSrc && !loading && (
          <img
            src={imageSrc}
            alt={`${boxer.name} sprite sheet`}
            style={{
              imageRendering: 'pixelated',
              transform: `scale(${zoom})`,
              transformOrigin: 'top left',
              display: 'block',
            }}
          />
        )}
      </div>

      {/* Bin legend from layout metadata */}
      {hasLayout && layout?.bin_labels && Object.keys(layout.bin_labels).length > 0 && (
        <div style={{ marginTop: '1rem' }}>
          <h4 style={{ margin: '0 0 0.5rem', fontSize: '0.875rem', color: 'var(--text-dim)', textTransform: 'uppercase', letterSpacing: '0.05em' }}>
            Bin Regions
          </h4>
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(250px, 1fr))', gap: '0.5rem' }}>
            {Object.entries(layout.bin_labels)
              .sort((a, b) => (a[1].priority ?? 99) - (b[1].priority ?? 99))
              .map(([filename, info]) => {
                // Check if this bin is shared
                const isSharedBin = boxer.shared_sprite_bins.some(b => b.filename === filename);
                
                return (
                  <div 
                    key={filename} 
                    style={{
                      padding: '8px 12px',
                      borderRadius: '6px',
                      background: isSharedBin ? 'rgba(255, 80, 80, 0.08)' : 'var(--glass)',
                      border: `1px solid ${isSharedBin ? 'rgba(255, 80, 80, 0.25)' : 'var(--border)'}`,
                      fontSize: '0.82rem',
                    }}
                  >
                    <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                      <span style={{ fontWeight: 600, color: isSharedBin ? '#ff9999' : '#8ab4ff' }}>
                        {info.region}
                      </span>
                      {isSharedBin && (
                        <span style={{ 
                          fontSize: '0.6rem', 
                          padding: '1px 4px', 
                          borderRadius: '3px',
                          background: 'rgba(255, 80, 80, 0.2)',
                          color: '#ff8888',
                        }}>
                          SHARED
                        </span>
                      )}
                    </div>
                    <div style={{ color: 'var(--text-dim)', marginTop: '2px' }}>{info.description}</div>
                    <div style={{ color: 'var(--text-dim)', opacity: 0.5, marginTop: '2px', fontFamily: 'monospace', fontSize: '0.75rem' }}>
                      {filename}
                    </div>
                  </div>
                );
              })}
          </div>
        </div>
      )}
    </div>
  );
};

export default BoxerPreviewSheet;
