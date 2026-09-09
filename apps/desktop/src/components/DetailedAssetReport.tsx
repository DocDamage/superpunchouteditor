import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { save } from '@tauri-apps/plugin-dialog';
import { useStore } from '../store/useStore';

// Type definitions matching Rust structures
interface ColorRgb {
  r: number;
  g: number;
  b: number;
  a?: number;
}

interface PaletteChangeDetail {
  name: string;
  pc_offset: string;
  snes_offset: string;
  colors_changed: number[];
  preview_before: ColorRgb[];
  preview_after: ColorRgb[];
  total_colors: number;
}

interface SpriteChangeDetail {
  bin_name: string;
  pc_offset: string;
  snes_offset: string;
  tiles_modified: number[];
  total_tiles: number;
  size_change: number;
  original_size: number;
  new_size: number;
  is_compressed: boolean;
  compression_ratio?: number;
}

interface StatChangeDetail {
  field: string;
  before: string;
  after: string;
  numeric_change?: number;
  percent_change?: number;
  significant: boolean;
}

interface AnimationChangeDetail {
  name: string;
  pc_offset: string;
  frames_changed: number[];
  total_frames: number;
  duration_change?: number;
}

interface SharedAssetReport {
  asset_name: string;
  pc_offset: string;
  shared_between: string[];
  change_type: string;
  warning_level: 'safe' | 'caution' | 'warning' | 'critical';
  description: string;
}

interface BoxerAssetReport {
  boxer_name: string;
  boxer_key: string;
  palettes: PaletteChangeDetail[];
  sprites: SpriteChangeDetail[];
  stats: StatChangeDetail[];
  animations: AnimationChangeDetail[];
  total_changes: number;
}

interface ChangeSummary {
  total_boxers_modified: number;
  total_palettes_changed: number;
  total_sprites_edited: number;
  total_animations_modified: number;
  total_headers_edited: number;
  total_changes: number;
}

interface BinaryChangeSummary {
  total_bytes_changed: number;
  total_regions_affected: number;
  largest_single_change: number;
  estimated_patch_size: number;
  original_sha1: string;
  modified_sha1: string;
}

interface DetailedAssetReportData {
  summary: ChangeSummary;
  boxer_reports: BoxerAssetReport[];
  shared_assets_touched: SharedAssetReport[];
  binary_changes: BinaryChangeSummary;
  generated_at: string;
  project_name: string;
  project_version: string;
}

type ReportTab = 'summary' | 'boxers' | 'shared' | 'binary';
type ExportFormat = 'html' | 'markdown' | 'json' | 'csv';

const WARNING_COLORS = {
  safe: '#4ade80',
  caution: '#fbbf24',
  warning: '#f87171',
  critical: '#dc2626',
};

const WARNING_LABELS = {
  safe: 'Safe - Unique Asset',
  caution: 'Caution - Shared Asset',
  warning: 'Warning - Multi-Boxer Impact',
  critical: 'Critical - Core Game Data',
};

export const DetailedAssetReport = () => {
  const { romSha1, pendingWrites, currentProject } = useStore();
  
  const [report, setReport] = useState<DetailedAssetReportData | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<ReportTab>('summary');
  const [expandedBoxers, setExpandedBoxers] = useState<Set<string>>(new Set());
  const [selectedExportFormat, setSelectedExportFormat] = useState<ExportFormat>('html');

  const hasChanges = pendingWrites.size > 0;

  const generateReport = useCallback(async () => {
    if (!romSha1) {
      setError('No ROM loaded');
      return;
    }

    setIsLoading(true);
    setError(null);

    try {
      const data = await invoke<DetailedAssetReportData>('generate_detailed_asset_report', {
        projectName: currentProject?.metadata?.name || 'Untitled Project',
        projectVersion: currentProject?.metadata?.version || '1.0.0',
      });
      setReport(data);
    } catch (e) {
      console.error('Failed to generate detailed report:', e);
      setError(`Failed to generate report: ${e}`);
    } finally {
      setIsLoading(false);
    }
  }, [romSha1, currentProject]);

  useEffect(() => {
    if (hasChanges) {
      generateReport();
    }
  }, [hasChanges, pendingWrites.size, generateReport]);

  const toggleBoxerExpansion = (boxerKey: string) => {
    setExpandedBoxers(prev => {
      const next = new Set(prev);
      if (next.has(boxerKey)) {
        next.delete(boxerKey);
      } else {
        next.add(boxerKey);
      }
      return next;
    });
  };

  const handleExport = async () => {
    if (!report) return;

    const extensions: Record<ExportFormat, string> = {
      html: 'html',
      markdown: 'md',
      json: 'json',
      csv: 'csv',
    };

    const path = await save({
      filters: [{
        name: 'Detailed Asset Report',
        extensions: [extensions[selectedExportFormat]],
      }],
      defaultPath: `asset_report_${report.project_version}.${extensions[selectedExportFormat]}`,
    });

    if (!path) return;

    try {
      await invoke('export_detailed_asset_report', {
        format: selectedExportFormat,
        outputPath: path,
        projectName: report.project_name,
        projectVersion: report.project_version,
      });
    } catch (e) {
      setError(`Failed to export report: ${e}`);
    }
  };

  const formatBytes = (bytes: number): string => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  };

  if (!romSha1) {
    return (
      <div style={{
        backgroundColor: 'var(--panel-bg)',
        border: '1px solid var(--border)',
        borderRadius: '12px',
        padding: '2rem',
        textAlign: 'center',
        color: 'var(--text-dim)',
      }}>
        Load a ROM to generate detailed asset reports
      </div>
    );
  }

  return (
    <div style={{
      backgroundColor: 'var(--panel-bg)',
      border: '1px solid var(--border)',
      borderRadius: '12px',
      padding: '1.5rem',
    }}>
      {/* Header */}
      <div style={{
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        marginBottom: '1.5rem',
        flexWrap: 'wrap',
        gap: '1rem',
      }}>
        <div>
          <h3 style={{ margin: 0 }}>Detailed Asset Report</h3>
          {report && (
            <div style={{ fontSize: '0.85rem', color: 'var(--text-dim)', marginTop: '0.25rem' }}>
              Generated {new Date(report.generated_at).toLocaleString()}
            </div>
          )}
        </div>
        <div style={{ display: 'flex', gap: '0.5rem' }}>
          <button
            onClick={generateReport}
            disabled={isLoading || !hasChanges}
            style={{
              padding: '8px 16px',
              backgroundColor: hasChanges ? 'var(--blue)' : 'var(--border)',
              border: 'none',
              borderRadius: '6px',
              color: 'white',
              fontWeight: 600,
              cursor: hasChanges ? 'pointer' : 'not-allowed',
              opacity: isLoading ? 0.6 : 1,
            }}
          >
            {isLoading ? 'Generating...' : 'Refresh'}
          </button>
        </div>
      </div>

      {error && (
        <div style={{
          marginBottom: '1rem',
          padding: '10px 14px',
          borderRadius: '8px',
          background: 'rgba(255, 80, 80, 0.1)',
          border: '1px solid rgba(255, 80, 80, 0.3)',
          color: '#ff6666',
          fontSize: '0.85rem',
        }}>
          {error}
        </div>
      )}

      {!hasChanges && !isLoading && (
        <div style={{
          textAlign: 'center',
          padding: '3rem 2rem',
          color: 'var(--text-dim)',
        }}>
          <div style={{ fontSize: '1.1rem', marginBottom: '0.5rem' }}>No Pending Changes</div>
          <div style={{ fontSize: '0.9rem' }}>Make edits to generate a detailed asset report</div>
        </div>
      )}

      {report && hasChanges && (
        <>
          {/* Tabs */}
          <div style={{
            display: 'flex',
            gap: '0.5rem',
            marginBottom: '1.5rem',
            borderBottom: '1px solid var(--border)',
            paddingBottom: '0.5rem',
          }}>
            {(['summary', 'boxers', 'shared', 'binary'] as ReportTab[]).map(tab => (
              <button
                key={tab}
                onClick={() => setActiveTab(tab)}
                style={{
                  padding: '8px 16px',
                  backgroundColor: activeTab === tab ? 'var(--blue)' : 'transparent',
                  border: 'none',
                  borderRadius: '6px',
                  color: activeTab === tab ? 'white' : 'var(--text)',
                  fontWeight: 500,
                  cursor: 'pointer',
                  textTransform: 'capitalize',
                }}
              >
                {tab}
                {tab === 'shared' && report.shared_assets_touched.length > 0 && (
                  <span style={{
                    marginLeft: '6px',
                    backgroundColor: 'rgba(255,255,255,0.2)',
                    padding: '1px 6px',
                    borderRadius: '10px',
                    fontSize: '0.75rem',
                  }}>
                    {report.shared_assets_touched.length}
                  </span>
                )}
              </button>
            ))}
          </div>

          {/* Tab Content */}
          <div style={{ marginBottom: '1.5rem' }}>
            {activeTab === 'summary' && <SummaryTab report={report} formatBytes={formatBytes} />}
            {activeTab === 'boxers' && (
              <BoxersTab 
                report={report} 
                expandedBoxers={expandedBoxers}
                onToggleBoxer={toggleBoxerExpansion}
              />
            )}
            {activeTab === 'shared' && <SharedTab report={report} />}
            {activeTab === 'binary' && <BinaryTab report={report} formatBytes={formatBytes} />}
          </div>

          {/* Export */}
          <div style={{ borderTop: '1px solid var(--border)', paddingTop: '1rem' }}>
            <div style={{ display: 'flex', gap: '0.75rem', alignItems: 'center', flexWrap: 'wrap' }}>
              <span style={{ fontSize: '0.9rem', color: 'var(--text-dim)' }}>Export as:</span>
              <select
                value={selectedExportFormat}
                onChange={(e) => setSelectedExportFormat(e.target.value as ExportFormat)}
                style={{
                  padding: '6px 12px',
                  backgroundColor: 'var(--panel-bg)',
                  border: '1px solid var(--border)',
                  borderRadius: '4px',
                  color: 'var(--text)',
                }}
              >
                <option value="html">HTML (with visual previews)</option>
                <option value="markdown">Markdown</option>
                <option value="json">JSON (machine-readable)</option>
                <option value="csv">CSV (spreadsheet)</option>
              </select>
              <button
                onClick={handleExport}
                style={{
                  padding: '6px 16px',
                  backgroundColor: '#10b981',
                  border: 'none',
                  borderRadius: '6px',
                  color: 'white',
                  fontWeight: 600,
                  cursor: 'pointer',
                }}
              >
                Export Report
              </button>
            </div>
          </div>
        </>
      )}
    </div>
  );
};

// Sub-components

const SummaryTab = ({ report, formatBytes }: { report: DetailedAssetReportData; formatBytes: (n: number) => string }) => {
  const { summary, binary_changes } = report;

  return (
    <div style={{ display: 'grid', gap: '1.5rem' }}>
      <div style={{
        display: 'grid',
        gridTemplateColumns: 'repeat(auto-fit, minmax(150px, 1fr))',
        gap: '1rem',
      }}>
        <StatCard value={summary.total_boxers_modified} label="Boxers Modified" color="#3498db" />
        <StatCard value={summary.total_palettes_changed} label="Palettes Changed" color="#9b59b6" />
        <StatCard value={summary.total_sprites_edited} label="Sprites Edited" color="#e67e22" />
        <StatCard value={summary.total_animations_modified} label="Animations" color="#1abc9c" />
        <StatCard value={summary.total_changes} label="Total Changes" color="#2ecc71" />
      </div>

      <div style={{ backgroundColor: 'var(--glass)', borderRadius: '8px', padding: '1rem' }}>
        <h4 style={{ margin: '0 0 1rem 0' }}>Binary Impact</h4>
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))', gap: '1rem' }}>
          <div><div style={{ fontSize: '0.85rem', color: 'var(--text-dim)' }}>Total Bytes Changed</div>
            <div style={{ fontSize: '1.25rem', fontWeight: 600 }}>{formatBytes(binary_changes.total_bytes_changed)}</div></div>
          <div><div style={{ fontSize: '0.85rem', color: 'var(--text-dim)' }}>Regions Affected</div>
            <div style={{ fontSize: '1.25rem', fontWeight: 600 }}>{binary_changes.total_regions_affected}</div></div>
          <div><div style={{ fontSize: '0.85rem', color: 'var(--text-dim)' }}>Est. Patch Size</div>
            <div style={{ fontSize: '1.25rem', fontWeight: 600 }}>{formatBytes(binary_changes.estimated_patch_size)}</div></div>
        </div>
      </div>

      {report.shared_assets_touched.length > 0 && (
        <div style={{
          backgroundColor: 'rgba(248, 113, 113, 0.1)',
          border: '1px solid rgba(248, 113, 113, 0.3)',
          borderRadius: '8px',
          padding: '1rem',
        }}>
          <strong>Shared Assets Detected</strong>
          <p style={{ margin: '0.5rem 0 0 0', fontSize: '0.9rem' }}>
            {report.shared_assets_touched.length} shared asset(s) will be modified.
          </p>
        </div>
      )}
    </div>
  );
};

const BoxersTab = ({ 
  report, 
  expandedBoxers, 
  onToggleBoxer 
}: { 
  report: DetailedAssetReportData; 
  expandedBoxers: Set<string>;
  onToggleBoxer: (key: string) => void;
}) => {
  if (!report.boxer_reports.length) {
    return <div style={{ textAlign: 'center', padding: '2rem', color: 'var(--text-dim)' }}>No boxer changes to display</div>;
  }

  return (
    <div style={{ display: 'grid', gap: '1rem' }}>
      {report.boxer_reports.map(boxer => (
        <div key={boxer.boxer_key} style={{ backgroundColor: 'var(--glass)', borderRadius: '8px', overflow: 'hidden' }}>
          <button
            onClick={() => onToggleBoxer(boxer.boxer_key)}
            style={{
              width: '100%',
              padding: '1rem',
              backgroundColor: 'transparent',
              border: 'none',
              cursor: 'pointer',
              display: 'flex',
              justifyContent: 'space-between',
              alignItems: 'center',
              color: 'var(--text)',
              fontSize: '1rem',
              fontWeight: 600,
            }}
          >
            <span>{boxer.boxer_name}</span>
            <div style={{ display: 'flex', alignItems: 'center', gap: '1rem' }}>
              <span style={{ fontSize: '0.85rem', color: 'var(--text-dim)' }}>
                {boxer.total_changes} change{boxer.total_changes !== 1 ? 's' : ''}
              </span>
              <span style={{ transform: expandedBoxers.has(boxer.boxer_key) ? 'rotate(180deg)' : 'none' }}>▼</span>
            </div>
          </button>

          {expandedBoxers.has(boxer.boxer_key) && (
            <div style={{ padding: '0 1rem 1rem 1rem' }}>
              {boxer.palettes.length > 0 && (
                <div style={{ marginBottom: '1rem' }}>
                  <h5 style={{ margin: '0 0 0.5rem 0' }}>Palettes</h5>
                  {boxer.palettes.map((p, i) => <PaletteDetail key={i} palette={p} />)}
                </div>
              )}
              {boxer.sprites.length > 0 && (
                <div>
                  <h5 style={{ margin: '0 0 0.5rem 0' }}>Sprites</h5>
                  {boxer.sprites.map((s, i) => <SpriteDetail key={i} sprite={s} />)}
                </div>
              )}
            </div>
          )}
        </div>
      ))}
    </div>
  );
};

const SharedTab = ({ report }: { report: DetailedAssetReportData }) => {
  if (!report.shared_assets_touched.length) {
    return <div style={{ textAlign: 'center', padding: '2rem', color: 'var(--text-dim)' }}>No shared assets affected</div>;
  }

  return (
    <div style={{ display: 'grid', gap: '1rem' }}>
      {report.shared_assets_touched.map((asset, idx) => (
        <div
          key={idx}
          style={{
            backgroundColor: 'var(--glass)',
            borderRadius: '8px',
            padding: '1rem',
            borderLeft: `4px solid ${WARNING_COLORS[asset.warning_level]}`,
          }}
        >
          <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '0.5rem' }}>
            <strong>{asset.asset_name}</strong>
            <span style={{
              fontSize: '0.75rem',
              padding: '2px 8px',
              borderRadius: '4px',
              backgroundColor: WARNING_COLORS[asset.warning_level] + '33',
              color: WARNING_COLORS[asset.warning_level],
            }}>
              {WARNING_LABELS[asset.warning_level]}
            </span>
          </div>
          <div style={{ fontSize: '0.85rem', color: 'var(--text-dim)' }}>
            <div>Location: {asset.pc_offset}</div>
            <div>Shared with: {asset.shared_between.join(', ')}</div>
          </div>
        </div>
      ))}
    </div>
  );
};

const BinaryTab = ({ report, formatBytes }: { report: DetailedAssetReportData; formatBytes: (n: number) => string }) => {
  const { binary_changes } = report;

  return (
    <div style={{ display: 'grid', gap: '1.5rem' }}>
      <div style={{ backgroundColor: 'var(--glass)', borderRadius: '8px', padding: '1rem' }}>
        <h4 style={{ margin: '0 0 1rem 0' }}>Binary Change Analysis</h4>
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(250px, 1fr))', gap: '1.5rem' }}>
          <div>
            <h5 style={{ margin: '0 0 0.5rem 0', fontSize: '0.9rem' }}>Size Metrics</h5>
            <table style={{ width: '100%', fontSize: '0.9rem' }}>
              <tbody>
                <tr><td style={{ color: 'var(--text-dim)' }}>Total Bytes:</td><td style={{ textAlign: 'right' }}>{formatBytes(binary_changes.total_bytes_changed)}</td></tr>
                <tr><td style={{ color: 'var(--text-dim)' }}>Regions:</td><td style={{ textAlign: 'right' }}>{binary_changes.total_regions_affected}</td></tr>
                <tr><td style={{ color: 'var(--text-dim)' }}>Est. Patch:</td><td style={{ textAlign: 'right' }}>{formatBytes(binary_changes.estimated_patch_size)}</td></tr>
              </tbody>
            </table>
          </div>
          <div>
            <h5 style={{ margin: '0 0 0.5rem 0', fontSize: '0.9rem' }}>SHA1 Checksums</h5>
            <div style={{ fontSize: '0.8rem', wordBreak: 'break-all' }}>
              <div style={{ color: 'var(--text-dim)' }}>Original: <code>{binary_changes.original_sha1 || 'N/A'}</code></div>
              <div style={{ color: 'var(--text-dim)', marginTop: '0.5rem' }}>Modified: <code>{binary_changes.modified_sha1 || 'N/A'}</code></div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

const StatCard = ({ value, label, color }: { value: number; label: string; color: string }) => (
  <div style={{ backgroundColor: 'var(--glass)', borderRadius: '8px', padding: '1rem', textAlign: 'center' }}>
    <div style={{ fontSize: '2rem', fontWeight: 'bold', color }}>{value}</div>
    <div style={{ fontSize: '0.85rem', color: 'var(--text-dim)' }}>{label}</div>
  </div>
);

const PaletteDetail = ({ palette }: { palette: PaletteChangeDetail }) => (
  <div style={{ backgroundColor: 'rgba(0,0,0,0.2)', borderRadius: '6px', padding: '0.75rem', marginBottom: '0.5rem' }}>
    <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '0.5rem' }}>
      <strong style={{ fontSize: '0.9rem' }}>{palette.name}</strong>
      <span style={{ fontSize: '0.8rem', color: 'var(--text-dim)' }}>
        {palette.colors_changed.length} / {palette.total_colors} colors
      </span>
    </div>
    <div style={{ display: 'flex', flexWrap: 'wrap', gap: '2px' }}>
      {palette.colors_changed.slice(0, 16).map((idx, i) => {
        const color = palette.preview_after[idx];
        return (
          <div
            key={i}
            style={{
              width: '24px',
              height: '24px',
              borderRadius: '3px',
              backgroundColor: color ? `rgb(${color.r}, ${color.g}, ${color.b})` : '#000',
              border: '1px solid rgba(255,255,255,0.2)',
            }}
            title={`Color ${idx}`}
          />
        );
      })}
      {palette.colors_changed.length > 16 && (
        <div style={{ width: '24px', height: '24px', display: 'flex', alignItems: 'center', justifyContent: 'center', fontSize: '0.7rem', color: 'var(--text-dim)' }}>
          +{palette.colors_changed.length - 16}
        </div>
      )}
    </div>
    <div style={{ fontSize: '0.75rem', color: 'var(--text-dim)', marginTop: '0.5rem' }}>
      {palette.pc_offset}
    </div>
  </div>
);

const SpriteDetail = ({ sprite }: { sprite: SpriteChangeDetail }) => (
  <div style={{ backgroundColor: 'rgba(0,0,0,0.2)', borderRadius: '6px', padding: '0.75rem', marginBottom: '0.5rem' }}>
    <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '0.25rem' }}>
      <strong style={{ fontSize: '0.9rem' }}>{sprite.bin_name}</strong>
      {sprite.is_compressed && (
        <span style={{ fontSize: '0.7rem', padding: '1px 6px', backgroundColor: 'var(--blue)', borderRadius: '4px' }}>
          Compressed
        </span>
      )}
    </div>
    <div style={{ fontSize: '0.85rem', color: 'var(--text-dim)' }}>
      <div>{sprite.tiles_modified.length} tiles modified</div>
      <div>Size: {sprite.original_size}B → {sprite.new_size}B ({sprite.size_change > 0 ? '+' : ''}{sprite.size_change})</div>
    </div>
  </div>
);

export default DetailedAssetReport;
