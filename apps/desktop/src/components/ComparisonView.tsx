import React, { useEffect, useState, useCallback } from 'react';
import { useStore } from '../store/useStore';
import type { Difference } from '../store/useStore';
import { ComparisonCanvas } from './ComparisonCanvas';
import { ComparisonTable } from './ComparisonTable';
import { DiffReport } from './DiffReport';

export const ComparisonView: React.FC = () => {
  const {
    boxers,
    comparison,
    comparisonLoading,
    selectedComparisonAsset,
    comparisonViewMode,
    generateComparison,
    selectComparisonAsset,
    setComparisonViewMode,
  } = useStore();

  const [activeTab, setActiveTab] = useState<'visual' | 'data' | 'binary'>('visual');
  const [selectedBoxer, setSelectedBoxer] = useState<string>('all');
  const [showDiffOnly, setShowDiffOnly] = useState(false);
  const [assetTypeFilter, setAssetTypeFilter] = useState<'all' | 'palette' | 'sprite' | 'header' | 'animation'>('all');

  useEffect(() => {
    if (!comparison && !comparisonLoading) {
      generateComparison();
    }
  }, []);

  const filteredDifferences = React.useMemo(() => {
    if (!comparison) return [];
    
    let diffs = comparison.differences;
    
    if (selectedBoxer !== 'all') {
      diffs = diffs.filter(d => 'boxer' in d && d.boxer === selectedBoxer);
    }

    if (assetTypeFilter !== 'all') {
      diffs = diffs.filter(d => d.type.toLowerCase() === assetTypeFilter);
    }

    if (showDiffOnly) {
      diffs = diffs.filter(d => {
        if (d.type === 'Palette') return d.changed_indices.length > 0;
        if (d.type === 'Sprite') return d.changed_tile_indices.length > 0;
        if (d.type === 'Header') return d.changed_fields.length > 0;
        return true;
      });
    }
    
    return diffs;
  }, [comparison, selectedBoxer, assetTypeFilter, showDiffOnly]);

  const handleRefresh = useCallback(() => {
    generateComparison();
  }, [generateComparison]);

  const viewModeButtons = [
    { mode: 'side-by-side' as const, label: 'Side-by-Side', icon: '◫' },
    { mode: 'overlay' as const, label: 'Overlay', icon: '◐' },
    { mode: 'difference' as const, label: 'Difference', icon: '⊘' },
    { mode: 'split' as const, label: 'Split', icon: '◨' },
  ];

  return (
    <div className="comparison-view" style={{ 
      height: '100%', 
      display: 'flex', 
      flexDirection: 'column',
      backgroundColor: 'var(--secondary-bg)',
      color: 'var(--text-main)'
    }}>
      {/* Header */}
      <div style={{ 
        padding: '1rem', 
        borderBottom: '1px solid var(--border)',
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        gap: '1rem',
        flexWrap: 'wrap'
      }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: '1rem' }}>
          <h2 style={{ margin: 0, fontSize: '1.25rem' }}>Compare Mode</h2>
          {comparison?.summary && (
            <span style={{ 
              fontSize: '0.875rem', 
              color: 'var(--text-dim)',
              backgroundColor: 'var(--glass)',
              padding: '0.25rem 0.75rem',
              borderRadius: '9999px'
            }}>
              {comparison.summary.total_changes} changes
            </span>
          )}
        </div>

        <div style={{ display: 'flex', gap: '0.5rem', alignItems: 'center' }}>
          <div style={{ 
            display: 'flex', 
            backgroundColor: 'var(--glass)', 
            borderRadius: '6px',
            padding: '2px'
          }}>
            {viewModeButtons.map(({ mode, label, icon }) => (
              <button
                key={mode}
                onClick={() => setComparisonViewMode(mode)}
                title={label}
                style={{
                  padding: '0.375rem 0.75rem',
                  borderRadius: '4px',
                  border: 'none',
                  backgroundColor: comparisonViewMode === mode ? 'var(--accent)' : 'transparent',
                  color: comparisonViewMode === mode ? 'white' : 'var(--text-dim)',
                  cursor: 'pointer',
                  fontSize: '0.875rem',
                  display: 'flex',
                  alignItems: 'center',
                  gap: '0.25rem'
                }}
              >
                <span>{icon}</span>
                <span className="hide-mobile">{label}</span>
              </button>
            ))}
          </div>

          <button
            onClick={handleRefresh}
            disabled={comparisonLoading}
            style={{
              padding: '0.5rem 1rem',
              backgroundColor: 'var(--blue)',
              border: 'none',
              borderRadius: '6px',
              color: 'white',
              cursor: comparisonLoading ? 'not-allowed' : 'pointer',
              opacity: comparisonLoading ? 0.6 : 1,
              display: 'flex',
              alignItems: 'center',
              gap: '0.5rem'
            }}
          >
            {comparisonLoading ? 'Loading...' : 'Refresh'}
          </button>
        </div>
      </div>

      {/* Filters Bar */}
      <div style={{ 
        padding: '0.75rem 1rem',
        borderBottom: '1px solid var(--border)',
        display: 'flex',
        gap: '1rem',
        alignItems: 'center',
        flexWrap: 'wrap',
        backgroundColor: 'var(--glass)'
      }}>
        <div style={{ display: 'flex', gap: '0.25rem' }}>
          {(['visual', 'data', 'binary'] as const).map(tab => (
            <button
              key={tab}
              onClick={() => setActiveTab(tab)}
              style={{
                padding: '0.375rem 0.875rem',
                borderRadius: '4px',
                border: 'none',
                backgroundColor: activeTab === tab ? 'var(--accent)' : 'transparent',
                color: activeTab === tab ? 'white' : 'var(--text-dim)',
                cursor: 'pointer',
                textTransform: 'capitalize',
                fontSize: '0.875rem'
              }}
            >
              {tab}
            </button>
          ))}
        </div>

        <div style={{ width: '1px', height: '1.5rem', backgroundColor: 'var(--border)' }} />

        <select
          value={selectedBoxer}
          onChange={(e) => setSelectedBoxer(e.target.value)}
          style={{
            padding: '0.375rem 0.75rem',
            borderRadius: '4px',
            border: '1px solid var(--border)',
            backgroundColor: 'var(--secondary-bg)',
            color: 'var(--text-main)',
            fontSize: '0.875rem'
          }}
        >
          <option value="all">All Boxers</option>
          {boxers.map(b => (
            <option key={b.key} value={b.name}>{b.name}</option>
          ))}
        </select>

        <select
          value={assetTypeFilter}
          onChange={(e) => setAssetTypeFilter(e.target.value as 'all' | 'palette' | 'sprite' | 'header' | 'animation')}
          style={{
            padding: '0.375rem 0.75rem',
            borderRadius: '4px',
            border: '1px solid var(--border)',
            backgroundColor: 'var(--secondary-bg)',
            color: 'var(--text-main)',
            fontSize: '0.875rem'
          }}
        >
          <option value="all">All Types</option>
          <option value="palette">Palettes</option>
          <option value="sprite">Sprites</option>
          <option value="header">Headers</option>
          <option value="animation">Animations</option>
        </select>

        <label style={{ 
          display: 'flex', 
          alignItems: 'center', 
          gap: '0.5rem',
          fontSize: '0.875rem',
          cursor: 'pointer'
        }}>
          <input
            type="checkbox"
            checked={showDiffOnly}
            onChange={(e) => setShowDiffOnly(e.target.checked)}
          />
          Changed only
        </label>
      </div>

      {/* Main Content */}
      <div style={{ flex: 1, display: 'flex', overflow: 'hidden' }}>
        <div style={{ 
          width: '280px', 
          borderRight: '1px solid var(--border)',
          overflowY: 'auto',
          padding: '0.5rem'
        }}>
          {filteredDifferences.length === 0 ? (
            <div style={{ 
              padding: '2rem 1rem', 
              textAlign: 'center', 
              color: 'var(--text-dim)',
              fontSize: '0.875rem'
            }}>
              {comparisonLoading ? 'Loading...' : 'No differences found'}
            </div>
          ) : (
            filteredDifferences.map((diff, index) => (
              <AssetListItem
                key={`${diff.type}-${index}`}
                diff={diff}
                isSelected={selectedComparisonAsset === `${diff.type}-${index}`}
                onClick={() => selectComparisonAsset(`${diff.type}-${index}`)}
              />
            ))
          )}
        </div>

        <div style={{ flex: 1, overflow: 'hidden', display: 'flex', flexDirection: 'column' }}>
          {activeTab === 'visual' && (
            <ComparisonCanvas 
              viewMode={comparisonViewMode}
              selectedAsset={selectedComparisonAsset}
            />
          )}
          {activeTab === 'data' && (
            <ComparisonTable 
              differences={filteredDifferences}
              selectedAsset={selectedComparisonAsset}
            />
          )}
          {activeTab === 'binary' && (
            <DiffReport 
              comparison={comparison}
            />
          )}
        </div>
      </div>

      {/* Summary Footer */}
      {comparison?.summary && (
        <div style={{ 
          padding: '0.75rem 1rem',
          borderTop: '1px solid var(--border)',
          backgroundColor: 'var(--glass)',
          fontSize: '0.875rem',
          display: 'flex',
          gap: '1.5rem',
          flexWrap: 'wrap'
        }}>
          <span><strong>{comparison.summary.palettes_modified}</strong> palettes</span>
          <span><strong>{comparison.summary.sprite_bins_changed}</strong> sprite bins</span>
          <span><strong>{comparison.summary.tiles_changed}</strong> tiles</span>
          <span><strong>{comparison.summary.total_bytes_changed}</strong> bytes</span>
        </div>
      )}
    </div>
  );
};

interface AssetListItemProps {
  diff: Difference;
  isSelected: boolean;
  onClick: () => void;
}

const AssetListItem: React.FC<AssetListItemProps> = ({ diff, isSelected, onClick }) => {
  const getDiffInfo = () => {
    switch (diff.type) {
      case 'Palette':
        return {
          icon: '🎨',
          title: diff.boxer,
          subtitle: `Palette - ${diff.changed_indices.length} colors changed`,
          color: '#f59e0b'
        };
      case 'Sprite':
        return {
          icon: '🖼️',
          title: diff.boxer,
          subtitle: `${diff.bin_name} - ${diff.changed_tile_indices.length} tiles`,
          color: '#3b82f6'
        };
      case 'Header':
        return {
          icon: '📋',
          title: diff.boxer,
          subtitle: `${diff.changed_fields.length} fields changed`,
          color: '#10b981'
        };
      case 'Animation':
        return {
          icon: '🎬',
          title: diff.boxer,
          subtitle: diff.anim_name,
          color: '#8b5cf6'
        };
      case 'Binary':
        return {
          icon: '💾',
          title: 'Binary',
          subtitle: `${diff.bytes_changed} bytes at 0x${diff.offset.toString(16)}`,
          color: '#6b7280'
        };
      default:
        return { icon: '❓', title: 'Unknown', subtitle: '', color: '#6b7280' };
    }
  };

  const info = getDiffInfo();

  return (
    <div
      onClick={onClick}
      style={{
        padding: '0.75rem',
        marginBottom: '0.5rem',
        borderRadius: '6px',
        cursor: 'pointer',
        backgroundColor: isSelected ? 'var(--accent)' : 'var(--glass)',
        borderLeft: `3px solid ${isSelected ? 'white' : info.color}`,
        transition: 'all 0.15s ease'
      }}
    >
      <div style={{ 
        display: 'flex', 
        alignItems: 'center', 
        gap: '0.5rem',
        marginBottom: '0.25rem'
      }}>
        <span>{info.icon}</span>
        <span style={{ 
          fontWeight: 500, 
          fontSize: '0.875rem',
          color: isSelected ? 'white' : 'var(--text-main)',
          overflow: 'hidden',
          textOverflow: 'ellipsis',
          whiteSpace: 'nowrap'
        }}>
          {info.title}
        </span>
      </div>
      <div style={{ 
        fontSize: '0.75rem', 
        color: isSelected ? 'rgba(255,255,255,0.8)' : 'var(--text-dim)',
        marginLeft: '1.5rem'
      }}>
        {info.subtitle}
      </div>
    </div>
  );
};

export default ComparisonView;
