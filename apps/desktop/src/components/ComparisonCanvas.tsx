import React, { useEffect, useRef, useState, useCallback } from 'react';
import { useStore } from '../store/useStore';

interface ComparisonCanvasProps {
  viewMode: 'side-by-side' | 'overlay' | 'difference' | 'split' | 'blink';
  selectedAsset: string | null;
}

export const ComparisonCanvas: React.FC<ComparisonCanvasProps> = ({
  viewMode,
  selectedAsset,
}) => {
  const {
    comparison,
    selectedBoxer,
    renderComparisonView,
    getPaletteDiff,
    getSpriteBinDiff,
  } = useStore();

  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [imageSrc, setImageSrc] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [splitPosition, setSplitPosition] = useState(0.5);
  const [showOriginal, setShowOriginal] = useState(true);
  const [showModified, setShowModified] = useState(true);
  const [blinkState, setBlinkState] = useState(false);

  // Find the selected difference
  const selectedDiff = React.useMemo(() => {
    if (!comparison || !selectedAsset) return null;
    const [type, indexStr] = selectedAsset.split('-');
    const index = parseInt(indexStr);
    const diffs = comparison.differences.filter(d => d.type === type);
    return diffs[index] || null;
  }, [comparison, selectedAsset]);

  // Handle blink animation
  useEffect(() => {
    if (viewMode !== 'blink') return;
    const interval = setInterval(() => {
      setBlinkState(prev => !prev);
    }, 500);
    return () => clearInterval(interval);
  }, [viewMode]);

  // Render the comparison view
  useEffect(() => {
    const render = async () => {
      if (!selectedDiff) {
        setImageSrc(null);
        return;
      }

      setLoading(true);
      setError(null);

      try {
        let boxerKey = '';
        let viewType: 'sprite' | 'frame' | 'animation' | 'palette' | 'portrait' | 'icon' = 'sprite';
        let assetOffset: string | undefined;
        let paletteOffset: string | undefined;

        if (selectedDiff.type === 'Sprite') {
          const diff = selectedDiff as any;
          boxerKey = diff.boxer?.toLowerCase().replace(/\s+/g, '_') || '';
          viewType = 'sprite';
          assetOffset = `0x${diff.pc_offset?.toString(16)}`;
        } else if (selectedDiff.type === 'Palette') {
          const diff = selectedDiff as any;
          boxerKey = diff.boxer?.toLowerCase().replace(/\s+/g, '_') || '';
          viewType = 'palette';
          paletteOffset = `0x${diff.offset?.toString(16)}`;
        }

        // If we can't determine the boxer key, show placeholder
        if (!boxerKey) {
          setError('Unable to determine asset details');
          setLoading(false);
          return;
        }

        const bytes = await renderComparisonView({
          boxer_key: boxerKey,
          view_type: viewType,
          show_original: viewMode === 'blink' ? !blinkState : showOriginal,
          show_modified: viewMode === 'blink' ? blinkState : showModified,
          asset_offset: assetOffset,
          palette_offset: paletteOffset,
          mode: viewMode,
        });

        if (bytes) {
          const blob = new Blob([bytes], { type: 'image/png' });
          const url = URL.createObjectURL(blob);
          setImageSrc(url);
        } else {
          setError('Failed to render comparison');
        }
      } catch (e) {
        console.error('Render error:', e);
        setError('Error rendering comparison view');
      } finally {
        setLoading(false);
      }
    };

    render();
  }, [selectedDiff, viewMode, showOriginal, showModified, blinkState, renderComparisonView]);

  // Handle split drag
  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    if (viewMode !== 'split') return;
    
    const handleMouseMove = (e: MouseEvent) => {
      const rect = canvasRef.current?.getBoundingClientRect();
      if (!rect) return;
      const x = e.clientX - rect.left;
      const newSplit = Math.max(0.1, Math.min(0.9, x / rect.width));
      setSplitPosition(newSplit);
    };

    const handleMouseUp = () => {
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
    };

    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);
  }, [viewMode]);

  if (!selectedDiff) {
    return (
      <div style={{
        flex: 1,
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        color: 'var(--text-dim)',
        flexDirection: 'column',
        gap: '1rem'
      }}>
        <div style={{ fontSize: '3rem', opacity: 0.5 }}>◫</div>
        <p>Select an asset from the list to compare</p>
      </div>
    );
  }

  return (
    <div style={{
      flex: 1,
      display: 'flex',
      flexDirection: 'column',
      overflow: 'hidden'
    }}>
      {/* Toolbar */}
      <div style={{
        padding: '0.75rem 1rem',
        borderBottom: '1px solid var(--border)',
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        gap: '1rem'
      }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
          <label style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', cursor: 'pointer' }}>
            <input
              type="checkbox"
              checked={showOriginal}
              onChange={(e) => setShowOriginal(e.target.checked)}
              disabled={viewMode === 'blink'}
            />
            <span style={{ fontSize: '0.875rem' }}>Original</span>
          </label>
          <label style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', cursor: 'pointer' }}>
            <input
              type="checkbox"
              checked={showModified}
              onChange={(e) => setShowModified(e.target.checked)}
              disabled={viewMode === 'blink'}
            />
            <span style={{ 
              fontSize: '0.875rem',
              color: 'var(--accent)'
            }}>Modified</span>
          </label>
        </div>

        {viewMode === 'split' && (
          <div style={{ fontSize: '0.75rem', color: 'var(--text-dim)' }}>
            Drag the white line to adjust split
          </div>
        )}

        {viewMode === 'blink' && (
          <div style={{ 
            fontSize: '0.75rem', 
            color: 'var(--text-dim)',
            display: 'flex',
            alignItems: 'center',
            gap: '0.5rem'
          }}>
            <span style={{
              width: '8px',
              height: '8px',
              borderRadius: '50%',
              backgroundColor: blinkState ? 'var(--accent)' : '#666'
            }} />
            Blinking {blinkState ? '(Modified)' : '(Original)'}
          </div>
        )}
      </div>

      {/* Canvas Container */}
      <div style={{
        flex: 1,
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        padding: '1rem',
        backgroundColor: 'var(--primary-bg)',
        position: 'relative'
      }}>
        {loading && (
          <div style={{
            position: 'absolute',
            inset: 0,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            backgroundColor: 'rgba(0,0,0,0.5)',
            zIndex: 10
          }}>
            <div style={{
              width: '40px',
              height: '40px',
              border: '3px solid var(--border)',
              borderTopColor: 'var(--accent)',
              borderRadius: '50%',
              animation: 'spin 1s linear infinite'
            }} />
          </div>
        )}

        {error ? (
          <div style={{ color: 'var(--accent)', textAlign: 'center' }}>
            <p>{error}</p>
          </div>
        ) : imageSrc ? (
          <div style={{ position: 'relative' }}>
            <img
              ref={(img) => {
                if (img && canvasRef.current !== img as any) {
                  (canvasRef as any).current = img;
                }
              }}
              src={imageSrc}
              alt="Comparison"
              style={{
                maxWidth: '100%',
                maxHeight: 'calc(100vh - 300px)',
                imageRendering: 'pixelated',
                cursor: viewMode === 'split' ? 'col-resize' : 'default'
              }}
              onMouseDown={handleMouseDown}
            />
            
            {viewMode === 'split' && (
              <div style={{
                position: 'absolute',
                top: 0,
                bottom: 0,
                left: `${splitPosition * 100}%`,
                width: '2px',
                backgroundColor: 'white',
                cursor: 'col-resize',
                transform: 'translateX(-1px)'
              }} />
            )}
          </div>
        ) : (
          <div style={{ color: 'var(--text-dim)' }}>
            No preview available
          </div>
        )}
      </div>

      {/* Asset Info */}
      {selectedDiff && (
        <div style={{
          padding: '0.75rem 1rem',
          borderTop: '1px solid var(--border)',
          backgroundColor: 'var(--glass)',
          fontSize: '0.875rem'
        }}>
          <AssetInfo diff={selectedDiff} />
        </div>
      )}

      <style>{`
        @keyframes spin {
          to { transform: rotate(360deg); }
        }
      `}</style>
    </div>
  );
};

// Asset info component
const AssetInfo: React.FC<{ diff: any }> = ({ diff }) => {
  switch (diff.type) {
    case 'Palette':
      return (
        <div style={{ display: 'flex', gap: '2rem', flexWrap: 'wrap' }}>
          <span><strong>Boxer:</strong> {diff.boxer}</span>
          <span><strong>Asset:</strong> {diff.asset_id}</span>
          <span><strong>Offset:</strong> 0x{diff.offset?.toString(16).toUpperCase()}</span>
          <span><strong>Colors Changed:</strong> {diff.changed_indices?.length || 0}</span>
        </div>
      );
    case 'Sprite':
      return (
        <div style={{ display: 'flex', gap: '2rem', flexWrap: 'wrap' }}>
          <span><strong>Boxer:</strong> {diff.boxer}</span>
          <span><strong>Bin:</strong> {diff.bin_name}</span>
          <span><strong>Offset:</strong> 0x{diff.pc_offset?.toString(16).toUpperCase()}</span>
          <span><strong>Tiles Changed:</strong> {diff.changed_tile_indices?.length || 0} / {diff.total_tiles}</span>
        </div>
      );
    case 'Header':
      return (
        <div style={{ display: 'flex', gap: '2rem', flexWrap: 'wrap' }}>
          <span><strong>Boxer:</strong> {diff.boxer}</span>
          <span><strong>Fields Changed:</strong> {diff.changed_fields?.length || 0}</span>
        </div>
      );
    default:
      return (
        <div>
          <span><strong>Type:</strong> {diff.type}</span>
        </div>
      );
  }
};

export default ComparisonCanvas;
