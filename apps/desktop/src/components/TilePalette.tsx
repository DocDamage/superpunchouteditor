import React, { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface TilePaletteProps {
  fighterId: number | null;
  tilesetId: number;
  paletteId: number;
  selectedTileId: number | null;
  onSelectTile: (tileId: number) => void;
}

export const TilePalette: React.FC<TilePaletteProps> = ({
  fighterId,
  tilesetId,
  paletteId,
  selectedTileId,
  onSelectTile,
}) => {
  const [tileCount, setTileCount] = useState<number>(0);
  const [tilePreviews, setTilePreviews] = useState<Map<number, string>>(new Map());
  const [loading, setLoading] = useState(false);
  const [filter, setFilter] = useState('');
  const [error, setError] = useState<string | null>(null);

  const TILES_PER_ROW = 8;
  const VISIBLE_ROWS = 16;

  // Load tile count when fighter or tileset changes
  useEffect(() => {
    if (fighterId === null) return;

    const loadTiles = async () => {
      setLoading(true);
      setError(null);
      try {
        const result = await invoke<number[]>('get_fighter_tiles', {
          fighterId,
          tilesetId,
        });
        setTileCount(result[0] || 0);
        
        // Generate placeholder previews for now
        // In a full implementation, we'd render actual tile previews
        const previews = new Map<number, string>();
        for (let i = 0; i < result[0]; i++) {
          // Placeholder: just use a color based on tile index
          const hue = (i * 137) % 360;
          previews.set(i, `hsl(${hue}, 50%, 50%)`);
        }
        setTilePreviews(previews);
      } catch (e) {
        console.error('Failed to load tiles:', e);
        setError('Failed to load tiles');
      } finally {
        setLoading(false);
      }
    };

    loadTiles();
  }, [fighterId, tilesetId]);

  // Handle tile click
  const handleTileClick = (tileId: number) => {
    onSelectTile(tileId);
  };

  // Filter tiles
  const filteredTiles = React.useMemo(() => {
    if (!filter) return Array.from({ length: tileCount }, (_, i) => i);
    
    const filterNum = parseInt(filter, 10);
    if (!isNaN(filterNum)) {
      return [filterNum].filter(n => n >= 0 && n < tileCount);
    }
    
    return Array.from({ length: tileCount }, (_, i) => i);
  }, [tileCount, filter]);

  if (fighterId === null) {
    return (
      <div className="tile-palette" style={styles.container}>
        <div style={styles.emptyState}>
          Select a fighter to view tiles
        </div>
      </div>
    );
  }

  return (
    <div className="tile-palette" style={styles.container}>
      {/* Header */}
      <div style={styles.header}>
        <h3 style={styles.title}>Tile Palette</h3>
        <span style={styles.count}>{tileCount} tiles</span>
      </div>

      {/* Filter */}
      <div style={styles.filterContainer}>
        <input
          type="text"
          placeholder="Filter tile #..."
          value={filter}
          onChange={(e) => setFilter(e.target.value)}
          style={styles.filterInput}
        />
        {filter && (
          <button
            onClick={() => setFilter('')}
            style={styles.clearButton}
          >
            ×
          </button>
        )}
      </div>

      {/* Info */}
      <div style={styles.info}>
        <span>Tileset: {tilesetId}</span>
        <span>Palette: {paletteId}</span>
      </div>

      {/* Tile grid */}
      <div style={styles.gridContainer}>
        {loading ? (
          <div style={styles.loading}>Loading tiles...</div>
        ) : error ? (
          <div style={styles.error}>{error}</div>
        ) : (
          <div style={styles.grid}>
            {filteredTiles.map((tileId) => (
              <button
                key={tileId}
                onClick={() => handleTileClick(tileId)}
                style={{
                  ...styles.tile,
                  backgroundColor: tilePreviews.get(tileId) || '#333',
                  borderColor: selectedTileId === tileId ? '#00a8ff' : '#444',
                  borderWidth: selectedTileId === tileId ? 3 : 1,
                }}
                title={`Tile ${tileId}`}
              >
                <span style={styles.tileNumber}>{tileId}</span>
              </button>
            ))}
          </div>
        )}
      </div>

      {/* Selected tile info */}
      {selectedTileId !== null && (
        <div style={styles.selectedInfo}>
          <strong>Selected: Tile {selectedTileId}</strong>
          <div style={styles.hint}>
            Click on canvas to place
          </div>
        </div>
      )}
    </div>
  );
};

const styles: Record<string, React.CSSProperties> = {
  container: {
    display: 'flex',
    flexDirection: 'column',
    height: '100%',
    backgroundColor: '#1e1e2e',
    borderRadius: 8,
    overflow: 'hidden',
  },
  header: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    padding: '12px 16px',
    borderBottom: '1px solid #333',
  },
  title: {
    margin: 0,
    fontSize: '14px',
    fontWeight: 600,
    color: '#fff',
  },
  count: {
    fontSize: '12px',
    color: '#888',
  },
  filterContainer: {
    display: 'flex',
    padding: '8px 16px',
    borderBottom: '1px solid #333',
    position: 'relative',
  },
  filterInput: {
    flex: 1,
    padding: '6px 10px',
    backgroundColor: '#2a2a3e',
    border: '1px solid #444',
    borderRadius: 4,
    color: '#fff',
    fontSize: '12px',
    outline: 'none',
  },
  clearButton: {
    position: 'absolute',
    right: 24,
    top: '50%',
    transform: 'translateY(-50%)',
    background: 'none',
    border: 'none',
    color: '#888',
    fontSize: '18px',
    cursor: 'pointer',
    padding: '0 4px',
  },
  info: {
    display: 'flex',
    justifyContent: 'space-between',
    padding: '8px 16px',
    fontSize: '11px',
    color: '#888',
    borderBottom: '1px solid #333',
  },
  gridContainer: {
    flex: 1,
    overflow: 'auto',
    padding: 12,
  },
  grid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(8, 1fr)',
    gap: 4,
  },
  tile: {
    aspectRatio: '1',
    border: '1px solid #444',
    borderRadius: 4,
    cursor: 'pointer',
    display: 'flex',
    alignItems: 'flex-end',
    justifyContent: 'flex-end',
    padding: 2,
    transition: 'transform 0.1s, border-color 0.1s',
  },
  tileNumber: {
    fontSize: '9px',
    color: 'rgba(255, 255, 255, 0.7)',
    textShadow: '0 1px 2px rgba(0,0,0,0.5)',
  },
  loading: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    height: '100%',
    color: '#888',
    fontSize: '12px',
  },
  error: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    height: '100%',
    color: '#ff6b6b',
    fontSize: '12px',
  },
  emptyState: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    height: '100%',
    color: '#666',
    fontSize: '12px',
    padding: 20,
  },
  selectedInfo: {
    padding: '12px 16px',
    backgroundColor: '#2a2a3e',
    borderTop: '1px solid #333',
    fontSize: '12px',
    color: '#fff',
  },
  hint: {
    fontSize: '11px',
    color: '#888',
    marginTop: 4,
  },
};
