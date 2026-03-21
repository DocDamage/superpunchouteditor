import React, { useState, useCallback } from 'react';
import { useStore, type RomComparison } from '../store/useStore';
import { save } from '@tauri-apps/plugin-dialog';
import { showToast } from './ToastContainer';

interface DiffReportProps {
  comparison: RomComparison | null;
}

export const DiffReport: React.FC<DiffReportProps> = ({ comparison }) => {
  const { exportComparisonReport } = useStore();
  const [selectedHexOffset, setSelectedHexOffset] = useState<string | null>(null);
  const [exporting, setExporting] = useState(false);

  const handleExport = useCallback(async (format: 'json' | 'html' | 'text') => {
    if (!comparison) return;

    const extensions = format === 'json' ? ['json'] : format === 'html' ? ['html'] : ['txt'];
    const selected = await save({
      filters: [{
        name: format.toUpperCase(),
        extensions
      }]
    });

    if (selected) {
      setExporting(true);
      try {
        await exportComparisonReport(selected, format);
        showToast(`Report exported to ${selected}`, 'success');
      } catch (e) {
        console.error('Export failed:', e);
        showToast('Export failed: ' + (e as Error).message, 'error');
      } finally {
        setExporting(false);
      }
    }
  }, [comparison, exportComparisonReport]);

  if (!comparison) {
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
        <div style={{ fontSize: '3rem', opacity: 0.5 }}>📊</div>
        <p>Generate a comparison to view the report</p>
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
      {/* Export Toolbar */}
      <div style={{
        padding: '0.75rem 1rem',
        borderBottom: '1px solid var(--border)',
        display: 'flex',
        gap: '0.5rem',
        alignItems: 'center',
        backgroundColor: 'var(--glass)'
      }}>
        <span style={{ fontSize: '0.875rem', marginRight: '0.5rem' }}>Export:</span>
        <ExportButton format="json" label="JSON" onClick={() => handleExport('json')} disabled={exporting} />
        <ExportButton format="html" label="HTML" onClick={() => handleExport('html')} disabled={exporting} />
        <ExportButton format="text" label="Text" onClick={() => handleExport('text')} disabled={exporting} />
      </div>

      {/* Summary Cards */}
      <div style={{
        padding: '1rem',
        display: 'grid',
        gridTemplateColumns: 'repeat(auto-fit, minmax(150px, 1fr))',
        gap: '1rem',
        borderBottom: '1px solid var(--border)'
      }}>
        <SummaryCard
          label="Total Changes"
          value={comparison.summary.total_changes}
          color="var(--accent)"
        />
        <SummaryCard
          label="Palettes"
          value={comparison.summary.palettes_modified}
          color="#f59e0b"
        />
        <SummaryCard
          label="Sprite Bins"
          value={comparison.summary.sprite_bins_changed}
          color="#3b82f6"
        />
        <SummaryCard
          label="Tiles"
          value={comparison.summary.tiles_changed}
          color="#8b5cf6"
        />
        <SummaryCard
          label="Headers"
          value={comparison.summary.fighter_headers_edited}
          color="#10b981"
        />
        <SummaryCard
          label="Bytes Changed"
          value={comparison.summary.total_bytes_changed}
          color="#6b7280"
        />
      </div>

      {/* Differences List */}
      <div style={{
        flex: 1,
        overflow: 'auto',
        padding: '1rem'
      }}>
        <h3 style={{ margin: '0 0 1rem 0', fontSize: '1rem' }}>Detailed Changes</h3>
        
        {comparison.differences.length === 0 ? (
          <div style={{ textAlign: 'center', padding: '3rem', color: 'var(--text-dim)' }}>
            No differences found between original and modified ROM
          </div>
        ) : (
          <div style={{ display: 'flex', flexDirection: 'column', gap: '0.75rem' }}>
            {comparison.differences.map((diff, index) => (
              <DifferenceCard
                key={`${diff.type}-${index}`}
                diff={diff}
                index={index}
                isExpanded={selectedHexOffset === `${diff.type}-${index}`}
                onToggle={() => setSelectedHexOffset(
                  selectedHexOffset === `${diff.type}-${index}` ? null : `${diff.type}-${index}`
                )}
              />
            ))}
          </div>
        )}
      </div>

      {/* SHA1 Footer */}
      <div style={{
        padding: '0.75rem 1rem',
        borderTop: '1px solid var(--border)',
        backgroundColor: 'var(--glass)',
        fontSize: '0.75rem',
        fontFamily: 'monospace',
        display: 'flex',
        gap: '2rem',
        flexWrap: 'wrap'
      }}>
        <span>
          <span style={{ color: 'var(--text-dim)' }}>Original: </span>
          {comparison.original_sha1}
        </span>
        <span>
          <span style={{ color: 'var(--text-dim)' }}>Modified: </span>
          {comparison.modified_sha1}
        </span>
      </div>
    </div>
  );
};

// Export button component
const ExportButton: React.FC<{
  format: string;
  label: string;
  onClick: () => void;
  disabled: boolean;
}> = ({ label, onClick, disabled }) => (
  <button
    onClick={onClick}
    disabled={disabled}
    style={{
      padding: '0.375rem 0.75rem',
      borderRadius: '4px',
      border: '1px solid var(--border)',
      backgroundColor: 'var(--secondary-bg)',
      color: 'var(--text-main)',
      cursor: disabled ? 'not-allowed' : 'pointer',
      opacity: disabled ? 0.6 : 1,
      fontSize: '0.75rem'
    }}
  >
    {label}
  </button>
);

// Summary card component
const SummaryCard: React.FC<{
  label: string;
  value: number;
  color: string;
}> = ({ label, value, color }) => (
  <div style={{
    padding: '1rem',
    backgroundColor: 'var(--glass)',
    borderRadius: '8px',
    borderLeft: `3px solid ${color}`
  }}>
    <div style={{
      fontSize: '1.5rem',
      fontWeight: 'bold',
      color: value > 0 ? color : 'var(--text-dim)'
    }}>
      {value}
    </div>
    <div style={{
      fontSize: '0.75rem',
      color: 'var(--text-dim)',
      marginTop: '0.25rem'
    }}>
      {label}
    </div>
  </div>
);

// Difference card component
const DifferenceCard: React.FC<{
  diff: any;
  index: number;
  isExpanded: boolean;
  onToggle: () => void;
}> = ({ diff, isExpanded, onToggle }) => {
  const typeColors: Record<string, string> = {
    'Palette': '#f59e0b',
    'Sprite': '#3b82f6',
    'Header': '#10b981',
    'Animation': '#8b5cf6',
    'Binary': '#6b7280'
  };

  const typeIcons: Record<string, string> = {
    'Palette': '🎨',
    'Sprite': '🖼️',
    'Header': '📋',
    'Animation': '🎬',
    'Binary': '💾'
  };

  const color = typeColors[diff.type] || '#6b7280';
  const icon = typeIcons[diff.type] || '❓';

  const getChangeCount = () => {
    switch (diff.type) {
      case 'Palette':
        return `${diff.changed_indices?.length || 0} colors`;
      case 'Sprite':
        return `${diff.changed_tile_indices?.length || 0} tiles`;
      case 'Header':
        return `${diff.changed_fields?.length || 0} fields`;
      case 'Binary':
        return `${diff.bytes_changed} bytes`;
      default:
        return 'modified';
    }
  };

  return (
    <div style={{
      border: '1px solid var(--border)',
      borderRadius: '8px',
      overflow: 'hidden'
    }}>
      <div
        onClick={onToggle}
        style={{
          padding: '0.75rem 1rem',
          display: 'flex',
          alignItems: 'center',
          gap: '0.75rem',
          cursor: 'pointer',
          backgroundColor: isExpanded ? 'var(--glass)' : 'transparent',
          transition: 'background-color 0.15s'
        }}
      >
        <span style={{ fontSize: '1.25rem' }}>{icon}</span>
        <div style={{ flex: 1 }}>
          <div style={{ fontWeight: 500, fontSize: '0.875rem' }}>
            {diff.boxer || diff.type}
          </div>
          <div style={{ fontSize: '0.75rem', color: 'var(--text-dim)' }}>
            {diff.bin_name || diff.asset_id || diff.anim_name || diff.description || ''}
          </div>
        </div>
        <span style={{
          padding: '0.25rem 0.5rem',
          backgroundColor: color,
          color: 'white',
          borderRadius: '4px',
          fontSize: '0.75rem'
        }}>
          {getChangeCount()}
        </span>
        <span style={{
          transform: isExpanded ? 'rotate(180deg)' : 'rotate(0deg)',
          transition: 'transform 0.15s'
        }}>
          ▼
        </span>
      </div>

      {isExpanded && (
        <div style={{
          padding: '1rem',
          borderTop: '1px solid var(--border)',
          backgroundColor: 'var(--glass)',
          fontSize: '0.75rem'
        }}>
          <DifferenceDetails diff={diff} />
        </div>
      )}
    </div>
  );
};

// Difference details component
const DifferenceDetails: React.FC<{ diff: any }> = ({ diff }) => {
  switch (diff.type) {
    case 'Palette':
      return (
        <div>
          <DetailRow label="Boxer" value={diff.boxer} />
          <DetailRow label="Asset ID" value={diff.asset_id} />
          <DetailRow label="Offset" value={`0x${diff.offset?.toString(16).toUpperCase()}`} />
          <DetailRow label="Colors Changed" value={diff.changed_indices?.length || 0} />
        </div>
      );
    case 'Sprite':
      return (
        <div>
          <DetailRow label="Boxer" value={diff.boxer} />
          <DetailRow label="Bin Name" value={diff.bin_name} />
          <DetailRow label="PC Offset" value={`0x${diff.pc_offset?.toString(16).toUpperCase()}`} />
          <DetailRow label="Total Tiles" value={diff.total_tiles} />
          <DetailRow label="Changed Tiles" value={diff.changed_tile_indices?.length || 0} />
        </div>
      );
    case 'Header':
      return (
        <div>
          <DetailRow label="Boxer" value={diff.boxer} />
          <DetailRow label="Fighter Index" value={diff.fighter_index} />
          <DetailRow label="Fields Changed" value={diff.changed_fields?.length || 0} />
          {diff.changed_fields && (
            <div style={{ marginTop: '0.5rem' }}>
              {diff.changed_fields.map((field: any, i: number) => (
                <div key={i} style={{ 
                  padding: '0.25rem 0',
                  borderBottom: '1px solid var(--border)'
                }}>
                  {field.display_name || field.field_name}: {' '}
                  <span style={{ color: 'var(--text-dim)' }}>{field.original_value}</span>
                  {' → '}
                  <span style={{ color: 'var(--accent)' }}>{field.modified_value}</span>
                </div>
              ))}
            </div>
          )}
        </div>
      );
    case 'Binary':
      return (
        <div>
          <DetailRow label="Offset" value={`0x${diff.offset?.toString(16).toUpperCase()}`} />
          <DetailRow label="Size" value={diff.size} />
          <DetailRow label="Bytes Changed" value={diff.bytes_changed} />
          <DetailRow label="Description" value={diff.description} />
        </div>
      );
    default:
      return <pre style={{ margin: 0, overflow: 'auto' }}>{JSON.stringify(diff, null, 2)}</pre>;
  }
};

// Detail row component
const DetailRow: React.FC<{ label: string; value: any }> = ({ label, value }) => (
  <div style={{ display: 'flex', padding: '0.25rem 0' }}>
    <span style={{ 
      width: '120px', 
      color: 'var(--text-dim)',
      flexShrink: 0
    }}>
      {label}:
    </span>
    <span>{value}</span>
  </div>
);

export default DiffReport;
