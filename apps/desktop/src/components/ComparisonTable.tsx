import React, { useState } from 'react';
import type { Difference } from '../store/useStore';

interface ComparisonTableProps {
  differences: Difference[];
  selectedAsset: string | null;
}

export const ComparisonTable: React.FC<ComparisonTableProps> = ({
  differences,
  selectedAsset,
}) => {
  const [expandedRows, setExpandedRows] = useState<Set<string>>(new Set());

  const toggleRow = (key: string) => {
    const newSet = new Set(expandedRows);
    if (newSet.has(key)) {
      newSet.delete(key);
    } else {
      newSet.add(key);
    }
    setExpandedRows(newSet);
  };

  const getChangeIndicator = (original: number, modified: number) => {
    if (original === modified) return <span style={{ color: 'var(--text-dim)' }}>-</span>;
    const delta = modified - original;
    const isIncrease = delta > 0;
    return (
      <span style={{ 
        color: isIncrease ? '#4ade80' : '#f87171',
        display: 'flex',
        alignItems: 'center',
        gap: '0.25rem'
      }}>
        {isIncrease ? '↑' : '↓'} {Math.abs(delta)}
      </span>
    );
  };

  return (
    <div style={{ 
      flex: 1, 
      overflow: 'auto',
      padding: '1rem'
    }}>
      {differences.length === 0 ? (
        <div style={{ 
          textAlign: 'center', 
          padding: '3rem 1rem',
          color: 'var(--text-dim)'
        }}>
          No differences to display
        </div>
      ) : (
        <table style={{ 
          width: '100%', 
          borderCollapse: 'collapse',
          fontSize: '0.875rem'
        }}>
          <thead>
            <tr style={{ 
              borderBottom: '2px solid var(--border)',
              textAlign: 'left'
            }}>
              <th style={{ padding: '0.75rem' }}>Asset</th>
              <th style={{ padding: '0.75rem' }}>Original</th>
              <th style={{ padding: '0.75rem' }}>Modified</th>
              <th style={{ padding: '0.75rem' }}>Change</th>
            </tr>
          </thead>
          <tbody>
            {differences.map((diff, index) => {
              const key = `${diff.type}-${index}`;
              const isSelected = selectedAsset === key;
              const isExpanded = expandedRows.has(key);

              return (
                <React.Fragment key={key}>
                  <tr
                    onClick={() => toggleRow(key)}
                    style={{
                      borderBottom: '1px solid var(--border)',
                      backgroundColor: isSelected ? 'var(--glass)' : 'transparent',
                      cursor: 'pointer',
                      transition: 'background-color 0.15s'
                    }}
                  >
                    <td style={{ padding: '0.75rem' }}>
                      <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
                        <span style={{ 
                          transform: isExpanded ? 'rotate(90deg)' : 'rotate(0deg)',
                          transition: 'transform 0.15s',
                          fontSize: '0.75rem'
                        }}>
                          ▶
                        </span>
                        <AssetTypeIcon type={diff.type} />
                        <span>{getAssetName(diff)}</span>
                      </div>
                    </td>
                    <td style={{ padding: '0.75rem', color: 'var(--text-dim)' }}>
                      {getOriginalValue(diff)}
                    </td>
                    <td style={{ padding: '0.75rem', color: 'var(--accent)' }}>
                      {getModifiedValue(diff)}
                    </td>
                    <td style={{ padding: '0.75rem' }}>
                      {getChangeSummary(diff)}
                    </td>
                  </tr>
                  {isExpanded && (
                    <tr>
                      <td colSpan={4} style={{ 
                        padding: '0 0 1rem 2rem',
                        backgroundColor: 'var(--glass)'
                      }}>
                        <ExpandedDiffContent diff={diff} />
                      </td>
                    </tr>
                  )}
                </React.Fragment>
              );
            })}
          </tbody>
        </table>
      )}
    </div>
  );
};

// Helper components
const AssetTypeIcon: React.FC<{ type: string }> = ({ type }) => {
  const icons: Record<string, string> = {
    'Palette': '🎨',
    'Sprite': '🖼️',
    'Header': '📋',
    'Animation': '🎬',
    'Binary': '💾'
  };
  return <span>{icons[type] || '❓'}</span>;
};

const getAssetName = (diff: Difference): string => {
  switch (diff.type) {
    case 'Palette':
      return `${(diff as any).boxer} - Palette`;
    case 'Sprite':
      return `${(diff as any).boxer} - ${(diff as any).bin_name}`;
    case 'Header':
      return `${(diff as any).boxer} - Header`;
    case 'Animation':
      return `${(diff as any).boxer} - ${(diff as any).anim_name}`;
    case 'Binary':
      return `Binary at 0x${(diff as any).offset?.toString(16)}`;
    default:
      return 'Unknown';
  }
};

const getOriginalValue = (diff: Difference): React.ReactNode => {
  switch (diff.type) {
    case 'Palette':
      const p = diff as any;
      return `${p.changed_indices?.length || 0} colors`;
    case 'Sprite':
      const s = diff as any;
      return `${s.total_tiles - (s.changed_tile_indices?.length || 0)} tiles unchanged`;
    case 'Header':
      const h = diff as any;
      return `${h.changed_fields?.length || 0} fields`;
    default:
      return '-';
  }
};

const getModifiedValue = (diff: Difference): React.ReactNode => {
  switch (diff.type) {
    case 'Palette':
      const p = diff as any;
      return `${p.changed_indices?.length || 0} colors changed`;
    case 'Sprite':
      const s = diff as any;
      return `${s.changed_tile_indices?.length || 0} tiles changed`;
    case 'Header':
      const h = diff as any;
      return `${h.changed_fields?.length || 0} fields modified`;
    default:
      return '-';
  }
};

const getChangeSummary = (diff: Difference): React.ReactNode => {
  switch (diff.type) {
    case 'Palette':
      const p = diff as any;
      const changedColors = p.changed_indices?.length || 0;
      if (changedColors === 0) return <span style={{ color: 'var(--text-dim)' }}>No change</span>;
      return <span style={{ color: '#f59e0b' }}>🔴 {changedColors} colors</span>;
    case 'Sprite':
      const s = diff as any;
      const changedTiles = s.changed_tile_indices?.length || 0;
      if (changedTiles === 0) return <span style={{ color: 'var(--text-dim)' }}>No change</span>;
      return <span style={{ color: '#3b82f6' }}>🔵 {changedTiles} tiles</span>;
    case 'Header':
      const h = diff as any;
      const changedFields = h.changed_fields?.length || 0;
      if (changedFields === 0) return <span style={{ color: 'var(--text-dim)' }}>No change</span>;
      return <span style={{ color: '#10b981' }}>🟢 {changedFields} fields</span>;
    default:
      return '-';
  }
};

const ExpandedDiffContent: React.FC<{ diff: Difference }> = ({ diff }) => {
  switch (diff.type) {
    case 'Palette':
      return <PaletteDiffDetails diff={diff as any} />;
    case 'Sprite':
      return <SpriteDiffDetails diff={diff as any} />;
    case 'Header':
      return <HeaderDiffDetails diff={diff as any} />;
    default:
      return <div style={{ color: 'var(--text-dim)' }}>No detailed view available</div>;
  }
};

const PaletteDiffDetails: React.FC<{ diff: any }> = ({ diff }) => {
  if (!diff.changed_indices || diff.changed_indices.length === 0) {
    return <div>No color changes</div>;
  }

  return (
    <div>
      <h4 style={{ margin: '0 0 0.5rem 0', fontSize: '0.875rem' }}>Changed Colors</h4>
      <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem' }}>
        {diff.changed_indices.slice(0, 10).map((idx: number) => {
          const orig = diff.original_colors?.[idx];
          const mod = diff.modified_colors?.[idx];
          return (
            <div key={idx} style={{ 
              display: 'flex', 
              alignItems: 'center', 
              gap: '1rem',
              fontSize: '0.75rem'
            }}>
              <span style={{ width: '60px' }}>Color {idx}</span>
              <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
                <ColorSwatch color={orig} />
                <span style={{ color: 'var(--text-dim)' }}>→</span>
                <ColorSwatch color={mod} />
              </div>
              {orig && mod && (
                <span style={{ color: 'var(--text-dim)' }}>
                  {orig.toHex?.() || `rgb(${orig.r},${orig.g},${orig.b})`} → {mod.toHex?.() || `rgb(${mod.r},${mod.g},${mod.b})`}
                </span>
              )}
            </div>
          );
        })}
        {diff.changed_indices.length > 10 && (
          <div style={{ color: 'var(--text-dim)', fontSize: '0.75rem' }}>
            ...and {diff.changed_indices.length - 10} more
          </div>
        )}
      </div>
    </div>
  );
};

const ColorSwatch: React.FC<{ color: any }> = ({ color }) => {
  if (!color) return <div style={{ width: '24px', height: '24px', backgroundColor: '#333' }} />;
  return (
    <div style={{
      width: '24px',
      height: '24px',
      backgroundColor: `rgb(${color.r}, ${color.g}, ${color.b})`,
      border: '1px solid var(--border)',
      borderRadius: '4px'
    }} />
  );
};

const SpriteDiffDetails: React.FC<{ diff: any }> = ({ diff }) => {
  if (!diff.changed_tile_indices || diff.changed_tile_indices.length === 0) {
    return <div>No tile changes</div>;
  }

  return (
    <div>
      <h4 style={{ margin: '0 0 0.5rem 0', fontSize: '0.875rem' }}>Changed Tiles</h4>
      <div style={{ 
        display: 'flex', 
        flexWrap: 'wrap', 
        gap: '0.5rem' 
      }}>
        {diff.changed_tile_indices.map((idx: number) => (
          <span key={idx} style={{
            padding: '0.25rem 0.5rem',
            backgroundColor: 'var(--accent)',
            borderRadius: '4px',
            fontSize: '0.75rem'
          }}>
            Tile {idx}
          </span>
        ))}
      </div>
    </div>
  );
};

const HeaderDiffDetails: React.FC<{ diff: any }> = ({ diff }) => {
  if (!diff.changed_fields || diff.changed_fields.length === 0) {
    return <div>No field changes</div>;
  }

  return (
    <div>
      <h4 style={{ margin: '0 0 0.5rem 0', fontSize: '0.875rem' }}>Changed Fields</h4>
      <table style={{ width: '100%', fontSize: '0.75rem' }}>
        <thead>
          <tr>
            <th style={{ textAlign: 'left', padding: '0.25rem' }}>Field</th>
            <th style={{ textAlign: 'left', padding: '0.25rem' }}>Original</th>
            <th style={{ textAlign: 'left', padding: '0.25rem' }}>Modified</th>
            <th style={{ textAlign: 'left', padding: '0.25rem' }}>Delta</th>
          </tr>
        </thead>
        <tbody>
          {diff.changed_fields.map((field: any, i: number) => (
            <tr key={i}>
              <td style={{ padding: '0.25rem' }}>{field.display_name || field.field_name}</td>
              <td style={{ padding: '0.25rem', color: 'var(--text-dim)' }}>{field.original_value}</td>
              <td style={{ padding: '0.25rem', color: 'var(--accent)' }}>{field.modified_value}</td>
              <td style={{ padding: '0.25rem' }}>
                {field.modified_value > field.original_value ? (
                  <span style={{ color: '#4ade80' }}>+{field.modified_value - field.original_value}</span>
                ) : field.modified_value < field.original_value ? (
                  <span style={{ color: '#f87171' }}>{field.modified_value - field.original_value}</span>
                ) : (
                  <span style={{ color: 'var(--text-dim)' }}>0</span>
                )}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
};

export default ComparisonTable;
