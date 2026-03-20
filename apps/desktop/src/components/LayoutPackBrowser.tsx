import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useStore } from '../store/useStore';
import { 
  LayoutPackInfo, 
  LayoutPack, 
  PackBoxerLayout,
  BoxerLayoutComparison,
  PackPreviewData 
} from '../types/layoutPack';

interface LayoutPackBrowserProps {
  initialPack?: LayoutPackInfo | null;
  onClose?: () => void;
}

export const LayoutPackBrowser = ({ initialPack, onClose }: LayoutPackBrowserProps) => {
  const { boxers } = useStore();
  const [packs, setPacks] = useState<LayoutPackInfo[]>([]);
  const [selectedPack, setSelectedPack] = useState<LayoutPackInfo | null>(initialPack || null);
  const [packData, setPackData] = useState<LayoutPack | null>(null);
  const [previewData, setPreviewData] = useState<PackPreviewData | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [selectedBoxerKey, setSelectedBoxerKey] = useState<string | null>(null);
  const [comparisonMode, setComparisonMode] = useState<'side-by-side' | 'overlay'>('side-by-side');

  const loadPacks = useCallback(async () => {
    try {
      const availablePacks = await invoke<LayoutPackInfo[]>('get_available_layout_packs');
      setPacks(availablePacks);
    } catch (e) {
      setError(`Failed to load packs: ${e}`);
    }
  }, []);

  useEffect(() => {
    loadPacks();
  }, [loadPacks]);

  useEffect(() => {
    if (selectedPack) {
      loadPackDetails(selectedPack);
    } else {
      setPackData(null);
      setPreviewData(null);
    }
  }, [selectedPack]);

  const loadPackDetails = async (packInfo: LayoutPackInfo) => {
    setLoading(true);
    setError(null);
    try {
      const pack = await invoke<LayoutPack>('import_layout_pack', {
        packPath: `data/boxer-layouts/community/${packInfo.filename}`,
      });
      setPackData(pack);

      // Generate comparison data
      const comparisons: BoxerLayoutComparison[] = pack.layouts.map(layout => {
        const currentBoxer = boxers.find(b => b.key === layout.boxer_key);
        if (!currentBoxer) {
          return {
            boxer_key: layout.boxer_key,
            current_bins: 0,
            pack_bins: layout.bins.length,
            matching_bins: 0,
            conflicts: ['Boxer not found in current manifest'],
          };
        }

        const currentBins = [...currentBoxer.unique_sprite_bins, ...currentBoxer.shared_sprite_bins];
        let matching = 0;
        const conflicts: string[] = [];

        for (const packBin of layout.bins) {
          const currentBin = currentBins.find(b => b.filename === packBin.filename);
          if (currentBin) {
            if (currentBin.start_pc === packBin.pc_offset && currentBin.size === packBin.size) {
              matching++;
            } else {
              conflicts.push(`${packBin.filename}: offset/size mismatch`);
            }
          } else {
            conflicts.push(`${packBin.filename}: not found in current boxer`);
          }
        }

        return {
          boxer_key: layout.boxer_key,
          current_bins: currentBins.length,
          pack_bins: layout.bins.length,
          matching_bins: matching,
          conflicts,
        };
      });

      setPreviewData({
        pack,
        comparisons,
        overall_compatible: comparisons.every(c => c.conflicts.length === 0),
      });

      // Select first boxer by default
      if (pack.layouts.length > 0) {
        setSelectedBoxerKey(pack.layouts[0].boxer_key);
      }
    } catch (e) {
      setError(`Failed to load pack details: ${e}`);
    } finally {
      setLoading(false);
    }
  };

  const handleApplyPack = async () => {
    if (!selectedPack || !packData) return;

    const boxerKeys = packData.layouts.map(l => l.boxer_key);
    
    if (!confirm(`Apply "${selectedPack.name}" to ${boxerKeys.length} boxer(s)?`)) {
      return;
    }

    try {
      await invoke('apply_layout_pack', {
        packPath: `data/boxer-layouts/community/${selectedPack.filename}`,
        boxerKeys,
      });
      alert('Layout pack applied successfully!');
    } catch (e) {
      setError(`Failed to apply pack: ${e}`);
    }
  };

  const handleSelectiveApply = async (boxerKey: string) => {
    if (!selectedPack) return;

    if (!confirm(`Apply layout for "${boxerKey}" from "${selectedPack.name}"?`)) {
      return;
    }

    try {
      await invoke('apply_layout_pack', {
        packPath: `data/boxer-layouts/community/${selectedPack.filename}`,
        boxerKeys: [boxerKey],
      });
      alert(`Layout for ${boxerKey} applied successfully!`);
    } catch (e) {
      setError(`Failed to apply layout: ${e}`);
    }
  };

  const getComparisonColor = (comparison: BoxerLayoutComparison) => {
    if (comparison.conflicts.length === 0) return '#6bdb7d';
    if (comparison.matching_bins === 0) return '#ff6666';
    return '#ffcc88';
  };

  const selectedLayout = packData?.layouts.find(l => l.boxer_key === selectedBoxerKey);
  const selectedComparison = previewData?.comparisons.find(c => c.boxer_key === selectedBoxerKey);
  const currentBoxer = boxers.find(b => b.key === selectedBoxerKey);

  return (
    <div style={{ 
      display: 'flex', 
      flexDirection: 'column',
      height: '100%',
      gap: '1rem',
    }}>
      {/* Header */}
      <div style={{ 
        display: 'flex', 
        justifyContent: 'space-between',
        alignItems: 'center',
        flexWrap: 'wrap',
        gap: '1rem',
      }}>
        <div>
          <h2 style={{ margin: '0 0 0.25rem' }}>Layout Pack Browser</h2>
          <p style={{ color: 'var(--text-dim)', margin: 0, fontSize: '0.9rem' }}>
            Preview pack contents and compare with current layouts
          </p>
        </div>
        {onClose && (
          <button onClick={onClose} style={{ background: 'var(--glass)' }}>
            ✕ Close
          </button>
        )}
      </div>

      {/* Error */}
      {error && (
        <div style={{ 
          padding: '12px 16px', 
          background: 'rgba(255, 100, 100, 0.1)', 
          border: '1px solid rgba(255, 100, 100, 0.3)',
          borderRadius: '8px',
          color: '#ff8888',
        }}>
          {error}
          <button onClick={() => setError(null)} style={{ marginLeft: '12px', fontSize: '0.8rem' }}>
            Dismiss
          </button>
        </div>
      )}

      {/* Main Content */}
      <div style={{ display: 'flex', gap: '1rem', flex: 1, minHeight: 0 }}>
        {/* Pack List */}
        <div style={{ 
          width: '280px',
          flexShrink: 0,
          background: 'var(--panel-bg)',
          borderRadius: '12px',
          border: '1px solid var(--border)',
          overflow: 'hidden',
          display: 'flex',
          flexDirection: 'column',
        }}>
          <div style={{ 
            padding: '12px 16px',
            borderBottom: '1px solid var(--border)',
            fontWeight: 600,
          }}>
            Available Packs ({packs.length})
          </div>
          <div style={{ flex: 1, overflow: 'auto' }}>
            {packs.length === 0 ? (
              <div style={{ padding: '2rem 1rem', textAlign: 'center', color: 'var(--text-dim)', fontSize: '0.9rem' }}>
                No packs available
              </div>
            ) : (
              packs.map(pack => (
                <div
                  key={pack.filename}
                  onClick={() => setSelectedPack(pack)}
                  style={{
                    padding: '12px 16px',
                    borderBottom: '1px solid var(--border)',
                    cursor: 'pointer',
                    background: selectedPack?.filename === pack.filename 
                      ? 'rgba(100, 150, 255, 0.15)' 
                      : 'transparent',
                    transition: 'background 0.15s',
                  }}
                >
                  <div style={{ fontWeight: 500, fontSize: '0.95rem', marginBottom: '2px' }}>
                    {pack.name}
                  </div>
                  <div style={{ fontSize: '0.8rem', color: 'var(--text-dim)' }}>
                    {pack.boxer_count} boxer{pack.boxer_count !== 1 ? 's' : ''}
                  </div>
                </div>
              ))
            )}
          </div>
        </div>

        {/* Pack Details */}
        <div style={{ flex: 1, display: 'flex', flexDirection: 'column', gap: '1rem', minWidth: 0 }}>
          {loading ? (
            <div style={{ 
              flex: 1,
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              color: 'var(--text-dim)',
            }}>
              <div>⏳ Loading pack details...</div>
            </div>
          ) : !packData || !previewData ? (
            <div style={{ 
              flex: 1,
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              color: 'var(--text-dim)',
            }}>
              <div>Select a pack to preview</div>
            </div>
          ) : (
            <>
              {/* Pack Header */}
              <div style={{ 
                padding: '20px',
                background: 'var(--panel-bg)',
                borderRadius: '12px',
                border: '1px solid var(--border)',
              }}>
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', marginBottom: '12px' }}>
                  <div>
                    <h3 style={{ margin: '0 0 4px' }}>{packData.name}</h3>
                    <div style={{ color: 'var(--text-dim)', fontSize: '0.9rem' }}>
                      by {packData.author || 'Unknown'} • {new Date(packData.created_at).toLocaleDateString()}
                    </div>
                  </div>
                  <div style={{ 
                    padding: '6px 12px',
                    borderRadius: '6px',
                    fontSize: '0.85rem',
                    fontWeight: 500,
                    background: previewData.overall_compatible 
                      ? 'rgba(100, 200, 100, 0.15)' 
                      : 'rgba(255, 200, 100, 0.15)',
                    color: previewData.overall_compatible ? '#6bdb7d' : '#ffcc88',
                  }}>
                    {previewData.overall_compatible ? '✓ Compatible' : '⚠ Conflicts'}
                  </div>
                </div>
                <p style={{ margin: '0 0 16px', fontSize: '0.95rem' }}>{packData.description}</p>
                <div style={{ display: 'flex', gap: '12px' }}>
                  <button 
                    onClick={handleApplyPack}
                    style={{ background: 'var(--green)' }}
                  >
                    Apply All ({packData.layouts.length} boxers)
                  </button>
                  <button 
                    onClick={() => loadPackDetails(selectedPack!)}
                    style={{ background: 'var(--glass)' }}
                  >
                    🔄 Refresh
                  </button>
                </div>
              </div>

              {/* Comparison Summary */}
              <div style={{ 
                padding: '16px 20px',
                background: 'var(--panel-bg)',
                borderRadius: '12px',
                border: '1px solid var(--border)',
              }}>
                <h4 style={{ margin: '0 0 12px', fontSize: '1rem' }}>Compatibility Summary</h4>
                <div style={{ 
                  display: 'grid',
                  gridTemplateColumns: 'repeat(auto-fill, minmax(200px, 1fr))',
                  gap: '12px',
                }}>
                  {previewData.comparisons.map(comp => (
                    <div
                      key={comp.boxer_key}
                      onClick={() => setSelectedBoxerKey(comp.boxer_key)}
                      style={{
                        padding: '12px',
                        borderRadius: '8px',
                        border: `2px solid ${getComparisonColor(comp)}`,
                        background: selectedBoxerKey === comp.boxer_key 
                          ? 'rgba(100, 150, 255, 0.1)' 
                          : 'var(--glass)',
                        cursor: 'pointer',
                        transition: 'all 0.15s',
                      }}
                    >
                      <div style={{ 
                        display: 'flex', 
                        justifyContent: 'space-between',
                        alignItems: 'center',
                        marginBottom: '6px',
                      }}>
                        <span style={{ fontWeight: 600 }}>{comp.boxer_key}</span>
                        <span style={{ 
                          fontSize: '0.75rem',
                          color: getComparisonColor(comp),
                        }}>
                          {comp.conflicts.length === 0 ? '✓' : comp.conflicts.length + ' issues'}
                        </span>
                      </div>
                      <div style={{ fontSize: '0.8rem', color: 'var(--text-dim)' }}>
                        {comp.matching_bins}/{comp.pack_bins} bins match
                      </div>
                    </div>
                  ))}
                </div>
              </div>

              {/* Selected Boxer Details */}
              {selectedLayout && selectedComparison && (
                <div style={{ 
                  flex: 1,
                  padding: '20px',
                  background: 'var(--panel-bg)',
                  borderRadius: '12px',
                  border: '1px solid var(--border)',
                  overflow: 'auto',
                  minHeight: 0,
                }}>
                  <div style={{ 
                    display: 'flex', 
                    justifyContent: 'space-between',
                    alignItems: 'center',
                    marginBottom: '16px',
                  }}>
                    <h4 style={{ margin: 0 }}>{selectedLayout.boxer_key} Details</h4>
                    <div style={{ display: 'flex', gap: '8px' }}>
                      <button
                        onClick={() => setComparisonMode('side-by-side')}
                        style={{
                          padding: '6px 12px',
                          fontSize: '0.8rem',
                          background: comparisonMode === 'side-by-side' ? 'var(--blue)' : 'var(--glass)',
                        }}
                      >
                        Side by Side
                      </button>
                      <button
                        onClick={() => setComparisonMode('overlay')}
                        style={{
                          padding: '6px 12px',
                          fontSize: '0.8rem',
                          background: comparisonMode === 'overlay' ? 'var(--blue)' : 'var(--glass)',
                        }}
                      >
                        Overlay
                      </button>
                      <button
                        onClick={() => handleSelectiveApply(selectedLayout.boxer_key)}
                        style={{
                          padding: '6px 12px',
                          fontSize: '0.8rem',
                          background: 'var(--green)',
                        }}
                      >
                        Apply This
                      </button>
                    </div>
                  </div>

                  {/* Comparison Stats */}
                  <div style={{ 
                    display: 'grid',
                    gridTemplateColumns: 'repeat(3, 1fr)',
                    gap: '12px',
                    marginBottom: '20px',
                  }}>
                    <div style={{ 
                      padding: '12px',
                      background: 'var(--glass)',
                      borderRadius: '8px',
                      textAlign: 'center',
                    }}>
                      <div style={{ fontSize: '1.5rem', fontWeight: 600, color: 'var(--blue)' }}>
                        {selectedComparison.current_bins}
                      </div>
                      <div style={{ fontSize: '0.8rem', color: 'var(--text-dim)' }}>
                        Current Bins
                      </div>
                    </div>
                    <div style={{ 
                      padding: '12px',
                      background: 'var(--glass)',
                      borderRadius: '8px',
                      textAlign: 'center',
                    }}>
                      <div style={{ fontSize: '1.5rem', fontWeight: 600, color: 'var(--green)' }}>
                        {selectedComparison.pack_bins}
                      </div>
                      <div style={{ fontSize: '0.8rem', color: 'var(--text-dim)' }}>
                        Pack Bins
                      </div>
                    </div>
                    <div style={{ 
                      padding: '12px',
                      background: 'var(--glass)',
                      borderRadius: '8px',
                      textAlign: 'center',
                    }}>
                      <div style={{ 
                        fontSize: '1.5rem', 
                        fontWeight: 600, 
                        color: selectedComparison.matching_bins === selectedComparison.pack_bins 
                          ? '#6bdb7d' 
                          : '#ffcc88'
                      }}>
                        {selectedComparison.matching_bins}
                      </div>
                      <div style={{ fontSize: '0.8rem', color: 'var(--text-dim)' }}>
                        Matching
                      </div>
                    </div>
                  </div>

                  {/* Conflicts */}
                  {selectedComparison.conflicts.length > 0 && (
                    <div style={{ 
                      padding: '12px 16px',
                      background: 'rgba(255, 100, 100, 0.08)',
                      border: '1px solid rgba(255, 100, 100, 0.2)',
                      borderRadius: '8px',
                      marginBottom: '20px',
                    }}>
                      <div style={{ fontSize: '0.85rem', color: '#ff8888', marginBottom: '8px' }}>
                        ⚠ Issues ({selectedComparison.conflicts.length}):
                      </div>
                      <ul style={{ margin: 0, paddingLeft: '1.25rem', fontSize: '0.85rem', color: '#ffaaaa' }}>
                        {selectedComparison.conflicts.slice(0, 5).map((conflict, i) => (
                          <li key={i}>{conflict}</li>
                        ))}
                        {selectedComparison.conflicts.length > 5 && (
                          <li>...and {selectedComparison.conflicts.length - 5} more</li>
                        )}
                      </ul>
                    </div>
                  )}

                  {/* Bin Comparison */}
                  <h5 style={{ margin: '0 0 12px', fontSize: '0.95rem' }}>
                    Bin Comparison ({selectedLayout.bins.length} bins)
                  </h5>
                  <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
                    {selectedLayout.bins.map((bin, index) => {
                      const currentBin = currentBoxer 
                        ? [...currentBoxer.unique_sprite_bins, ...currentBoxer.shared_sprite_bins]
                            .find(b => b.filename === bin.filename)
                        : null;
                      
                      const matches = currentBin && 
                        currentBin.start_pc === bin.pc_offset && 
                        currentBin.size === bin.size;
                      
                      return (
                        <div
                          key={bin.filename}
                          style={{
                            padding: '12px 16px',
                            background: 'var(--glass)',
                            borderRadius: '8px',
                            borderLeft: `4px solid ${matches ? '#6bdb7d' : currentBin ? '#ffcc88' : '#ff6666'}`,
                          }}
                        >
                          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                            <div>
                              <div style={{ fontWeight: 500, fontSize: '0.9rem' }}>{bin.filename}</div>
                              <div style={{ fontSize: '0.8rem', color: 'var(--text-dim)', marginTop: '2px' }}>
                                Pack: {bin.pc_offset} • {bin.size} bytes • {bin.category}
                              </div>
                              {currentBin && (
                                <div style={{ 
                                  fontSize: '0.8rem', 
                                  color: matches ? '#6bdb7d' : '#ffcc88',
                                  marginTop: '2px',
                                }}>
                                  Current: {currentBin.start_pc} • {currentBin.size} bytes
                                  {matches ? ' ✓' : ' (differs)'}
                                </div>
                              )}
                            </div>
                            <div style={{ 
                              fontSize: '0.75rem',
                              padding: '4px 10px',
                              borderRadius: '4px',
                              background: matches ? 'rgba(100, 200, 100, 0.15)' : currentBin ? 'rgba(255, 200, 100, 0.15)' : 'rgba(255, 100, 100, 0.15)',
                              color: matches ? '#6bdb7d' : currentBin ? '#ffcc88' : '#ff6666',
                            }}>
                              {matches ? 'Match' : currentBin ? 'Mismatch' : 'Missing'}
                            </div>
                          </div>
                        </div>
                      );
                    })}
                  </div>
                </div>
              )}
            </>
          )}
        </div>
      </div>
    </div>
  );
};

export default LayoutPackBrowser;
