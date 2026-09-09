import React from 'react';
import { FrameData, SpriteEntry } from '../types/frame';

interface SpritePropertiesProps {
  frame: FrameData | null;
  selectedSprites: number[];
  onUpdateSprite: (index: number, updates: Partial<SpriteEntry>) => void;
  onRemoveSprite: (index: number) => void;
  onDuplicateSprite: (index: number) => void;
}

export const SpriteProperties: React.FC<SpritePropertiesProps> = ({
  frame,
  selectedSprites,
  onUpdateSprite,
  onRemoveSprite,
  onDuplicateSprite,
}) => {
  if (!frame || selectedSprites.length === 0) {
    return (
      <div style={styles.container}>
        <div style={styles.emptyState}>
          <div style={styles.emptyIcon}>🖱️</div>
          <div style={styles.emptyText}>
            Select a sprite to edit its properties
          </div>
          <div style={styles.emptyHint}>
            Click on a sprite in the canvas to select it
          </div>
        </div>
      </div>
    );
  }

  // If multiple sprites selected, show simplified view
  if (selectedSprites.length > 1) {
    return (
      <div style={styles.container}>
        <div style={styles.header}>
          <h3 style={styles.title}>{selectedSprites.length} Sprites Selected</h3>
        </div>
        <div style={styles.multiSelectInfo}>
          <p>Use arrow keys to nudge all selected sprites</p>
          <p style={styles.hint}>Hold Shift for 8px steps</p>
        </div>
      </div>
    );
  }

  const spriteIndex = selectedSprites[0];
  const sprite = frame.sprites[spriteIndex];

  if (!sprite) {
    return (
      <div style={styles.container}>
        <div style={styles.emptyState}>
          <div style={styles.emptyText}>Sprite not found</div>
        </div>
      </div>
    );
  }

  const handleChange = (field: keyof SpriteEntry, value: number | boolean) => {
    onUpdateSprite(spriteIndex, { [field]: value });
  };

  return (
    <div style={styles.container}>
      {/* Header */}
      <div style={styles.header}>
        <h3 style={styles.title}>Sprite #{spriteIndex}</h3>
        <div style={styles.actions}>
          <button
            onClick={() => onDuplicateSprite(spriteIndex)}
            style={styles.actionButton}
            title="Duplicate sprite (Ctrl+D)"
          >
            📋
          </button>
          <button
            onClick={() => onRemoveSprite(spriteIndex)}
            style={{ ...styles.actionButton, ...styles.dangerButton }}
            title="Remove sprite (Delete)"
          >
            🗑️
          </button>
        </div>
      </div>

      {/* Position */}
      <div style={styles.section}>
        <h4 style={styles.sectionTitle}>Position</h4>
        <div style={styles.row}>
          <div style={styles.field}>
            <label style={styles.label}>X</label>
            <input
              type="number"
              value={sprite.x}
              onChange={(e) => handleChange('x', parseInt(e.target.value, 10) || 0)}
              style={styles.input}
              min={-128}
              max={127}
            />
          </div>
          <div style={styles.field}>
            <label style={styles.label}>Y</label>
            <input
              type="number"
              value={sprite.y}
              onChange={(e) => handleChange('y', parseInt(e.target.value, 10) || 0)}
              style={styles.input}
              min={-128}
              max={127}
            />
          </div>
        </div>
      </div>

      {/* Tile */}
      <div style={styles.section}>
        <h4 style={styles.sectionTitle}>Tile</h4>
        <div style={styles.field}>
          <label style={styles.label}>Tile ID</label>
          <input
            type="number"
            value={sprite.tile_id}
            onChange={(e) => handleChange('tile_id', parseInt(e.target.value, 10) || 0)}
            style={styles.input}
            min={0}
            max={255}
          />
        </div>
      </div>

      {/* Attributes */}
      <div style={styles.section}>
        <h4 style={styles.sectionTitle}>Attributes</h4>
        
        <div style={styles.field}>
          <label style={styles.label}>Palette</label>
          <select
            value={sprite.palette}
            onChange={(e) => handleChange('palette', parseInt(e.target.value, 10))}
            style={styles.select}
          >
            {Array.from({ length: 8 }, (_, i) => (
              <option key={i} value={i}>
                Palette {i}
              </option>
            ))}
          </select>
        </div>

        <div style={styles.field}>
          <label style={styles.label}>Priority</label>
          <select
            value={sprite.priority}
            onChange={(e) => handleChange('priority', parseInt(e.target.value, 10))}
            style={styles.select}
          >
            <option value={0}>0 (Back)</option>
            <option value={1}>1</option>
            <option value={2}>2</option>
            <option value={3}>3 (Front)</option>
          </select>
        </div>

        <div style={styles.checkboxRow}>
          <label style={styles.checkbox}>
            <input
              type="checkbox"
              checked={sprite.h_flip}
              onChange={(e) => handleChange('h_flip', e.target.checked)}
            />
            <span>Flip Horizontal</span>
          </label>
        </div>

        <div style={styles.checkboxRow}>
          <label style={styles.checkbox}>
            <input
              type="checkbox"
              checked={sprite.v_flip}
              onChange={(e) => handleChange('v_flip', e.target.checked)}
            />
            <span>Flip Vertical</span>
          </label>
        </div>
      </div>

      {/* Raw bytes */}
      <div style={styles.section}>
        <h4 style={styles.sectionTitle}>Raw Data</h4>
        <div style={styles.rawData}>
          <code style={styles.code}>
            X: {sprite.x >= 0 ? ` $${sprite.x.toString(16).padStart(2, '0')}` : `-$${Math.abs(sprite.x).toString(16).padStart(2, '0')}`}
            <br />
            Y: {sprite.y >= 0 ? ` $${sprite.y.toString(16).padStart(2, '0')}` : `-$${Math.abs(sprite.y).toString(16).padStart(2, '0')}`}
            <br />
            Tile: ${sprite.tile_id.toString(16).padStart(2, '0')}
            <br />
            Attr: ${((sprite.palette & 0x07) << 1 | (sprite.priority & 0x03) << 4 | (sprite.h_flip ? 0x40 : 0) | (sprite.v_flip ? 0x80 : 0)).toString(16).padStart(2, '0')}
          </code>
        </div>
      </div>

      {/* Keyboard shortcuts hint */}
      <div style={styles.hintSection}>
        <div style={styles.hintTitle}>Keyboard Shortcuts</div>
        <div style={styles.hintText}>
          ↑↓←→ Nudge 1px
          <br />
          Shift + Arrow Nudge 8px
          <br />
          Ctrl+D Duplicate
          <br />
          Delete Remove
        </div>
      </div>
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
  actions: {
    display: 'flex',
    gap: 8,
  },
  actionButton: {
    padding: '6px 8px',
    backgroundColor: '#2a2a3e',
    border: '1px solid #444',
    borderRadius: 4,
    cursor: 'pointer',
    fontSize: '14px',
    transition: 'background-color 0.2s',
  },
  dangerButton: {
    backgroundColor: 'rgba(255, 107, 107, 0.2)',
    borderColor: '#ff6b6b',
  },
  section: {
    padding: '12px 16px',
    borderBottom: '1px solid #333',
  },
  sectionTitle: {
    margin: '0 0 12px 0',
    fontSize: '12px',
    fontWeight: 600,
    color: '#888',
    textTransform: 'uppercase',
    letterSpacing: '0.5px',
  },
  row: {
    display: 'flex',
    gap: 12,
  },
  field: {
    flex: 1,
    marginBottom: 8,
  },
  label: {
    display: 'block',
    fontSize: '11px',
    color: '#888',
    marginBottom: 4,
  },
  input: {
    width: '100%',
    padding: '6px 8px',
    backgroundColor: '#2a2a3e',
    border: '1px solid #444',
    borderRadius: 4,
    color: '#fff',
    fontSize: '13px',
    fontFamily: 'monospace',
    boxSizing: 'border-box',
    outline: 'none',
  },
  select: {
    width: '100%',
    padding: '6px 8px',
    backgroundColor: '#2a2a3e',
    border: '1px solid #444',
    borderRadius: 4,
    color: '#fff',
    fontSize: '13px',
    outline: 'none',
    cursor: 'pointer',
  },
  checkboxRow: {
    marginTop: 8,
  },
  checkbox: {
    display: 'flex',
    alignItems: 'center',
    gap: 8,
    fontSize: '13px',
    color: '#ccc',
    cursor: 'pointer',
  },
  rawData: {
    padding: '8px 12px',
    backgroundColor: '#16161e',
    borderRadius: 4,
    fontFamily: 'monospace',
  },
  code: {
    fontSize: '11px',
    color: '#888',
    lineHeight: 1.6,
  },
  emptyState: {
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    justifyContent: 'center',
    height: '100%',
    padding: 24,
    textAlign: 'center',
  },
  emptyIcon: {
    fontSize: '32px',
    marginBottom: 12,
    opacity: 0.5,
  },
  emptyText: {
    fontSize: '14px',
    color: '#888',
    marginBottom: 8,
  },
  emptyHint: {
    fontSize: '12px',
    color: '#666',
  },
  multiSelectInfo: {
    padding: 24,
    textAlign: 'center',
    color: '#888',
    fontSize: '13px',
  },
  hint: {
    fontSize: '11px',
    color: '#666',
    marginTop: 8,
  },
  hintSection: {
    marginTop: 'auto',
    padding: '12px 16px',
    backgroundColor: '#16161e',
    borderTop: '1px solid #333',
  },
  hintTitle: {
    fontSize: '11px',
    fontWeight: 600,
    color: '#666',
    marginBottom: 8,
    textTransform: 'uppercase',
  },
  hintText: {
    fontSize: '11px',
    color: '#888',
    lineHeight: 1.8,
    fontFamily: 'monospace',
  },
};
