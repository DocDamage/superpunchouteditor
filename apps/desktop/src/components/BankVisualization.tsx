/**
 * Bank Visualization Component
 * 
 * Visualizes SNES ROM bank structure showing memory regions,
 * statistics, fragmentation analysis, and defragmentation controls.
 */

import { useState, useCallback } from 'react';
import {
  useBankMap,
  useBankStatistics,
  useFragmentationAnalysis,
  useDefragmentationPlan,
  useDefragmentation,
  useFreeRegions,
} from '../hooks/useBankManagement';
import type {
  BankMap,
  BankMapEntry,
  BankStatistics,
  BankStats,
  MemoryRegion,
  FragmentationAnalysis,
  DefragPlan,
  DefragStep,
} from '../types/api';

// Color coding for data types
const TYPE_COLORS: Record<string, string> = {
  free: '#2d3436',
  graphics_compressed: '#e17055',
  graphics_uncompressed: '#d63031',
  palette: '#fdcb6e',
  audio: '#6c5ce7',
  code: '#00cec9',
  text: '#dfe6e9',
  data: '#74b9ff',
  unknown: '#636e72',
};

const TYPE_LABELS: Record<string, string> = {
  free: 'Free Space',
  graphics_compressed: 'Graphics (Compressed)',
  graphics_uncompressed: 'Graphics (Uncompressed)',
  palette: 'Palette',
  audio: 'Audio',
  code: 'Code',
  text: 'Text',
  data: 'Data',
  unknown: 'Unknown',
};

// Safety rating colors (based on fragmentation score)
const getSafetyFromScore = (score: number): 'safe' | 'caution' | 'warning' => {
  if (score < 20) return 'safe';
  if (score < 50) return 'caution';
  return 'warning';
};

const SAFETY_COLORS = {
  safe: '#4ade80',
  caution: '#fbbf24',
  warning: '#f87171',
};

interface TooltipState {
  visible: boolean;
  x: number;
  y: number;
  entry: BankMapEntry | null;
}

export const BankVisualization = () => {
  // Hooks for data
  const bankMap = useBankMap();
  const bankStats = useBankStatistics();
  const fragAnalysis = useFragmentationAnalysis();
  const defragPlan = useDefragmentationPlan();
  const defrag = useDefragmentation();
  const freeRegions = useFreeRegions();

  // Local state
  const [tooltip, setTooltip] = useState<TooltipState>({
    visible: false,
    x: 0,
    y: 0,
    entry: null,
  });
  const [minSizeInput, setMinSizeInput] = useState<string>('');
  const [showConfirmDialog, setShowConfirmDialog] = useState(false);
  const [activeTab, setActiveTab] = useState<'overview' | 'fragmentation' | 'search'>('overview');
  const [selectedBank, setSelectedBank] = useState<number | null>(null);

  // Handle bank block hover
  const handleMouseEnter = useCallback((entry: BankMapEntry, e: React.MouseEvent) => {
    setTooltip({
      visible: true,
      x: e.clientX + 10,
      y: e.clientY + 10,
      entry,
    });
  }, []);

  const handleMouseMove = useCallback((e: React.MouseEvent) => {
    setTooltip(prev => ({
      ...prev,
      x: e.clientX + 10,
      y: e.clientY + 10,
    }));
  }, []);

  const handleMouseLeave = useCallback(() => {
    setTooltip(prev => ({ ...prev, visible: false }));
  }, []);

  // Handle free region search
  const handleSearch = useCallback(() => {
    const minSize = parseInt(minSizeInput, 10);
    freeRegions.refresh(minSize > 0 ? minSize : undefined);
  }, [minSizeInput, freeRegions]);

  // Handle defragmentation execution
  const handleExecute = useCallback(async () => {
    setShowConfirmDialog(false);
    await defrag.execute();
    // Refresh all data after defragmentation
    bankMap.refresh();
    bankStats.refresh();
    fragAnalysis.refresh();
    defragPlan.refresh();
    freeRegions.refresh();
  }, [defrag, bankMap, bankStats, fragAnalysis, defragPlan, freeRegions]);

  // Format bytes to human readable
  const formatBytes = (bytes: number): string => {
    if (bytes >= 1024 * 1024) {
      return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
    }
    if (bytes >= 1024) {
      return `${(bytes / 1024).toFixed(2)} KB`;
    }
    return `${bytes} bytes`;
  };

  // Format address
  const formatAddress = (addr: number): string => {
    return `0x${addr.toString(16).toUpperCase().padStart(6, '0')}`;
  };

  // Get safety rating display
  const getSafetyDisplay = (score: number) => {
    const rating = getSafetyFromScore(score);
    const colors = SAFETY_COLORS;
    const labels = { safe: 'Safe', caution: 'Caution', warning: 'Warning' };
    return (
      <span
        style={{
          display: 'inline-flex',
          alignItems: 'center',
          gap: '0.5rem',
          padding: '0.25rem 0.75rem',
          backgroundColor: `${colors[rating]}20`,
          border: `1px solid ${colors[rating]}`,
          borderRadius: '4px',
          color: colors[rating],
          fontWeight: 600,
          fontSize: '0.85rem',
        }}
      >
        <span
          style={{
            width: '8px',
            height: '8px',
            borderRadius: '50%',
            backgroundColor: colors[rating],
          }}
        />
        {labels[rating]}
      </span>
    );
  };

  // Get unique banks from entries
  const getUniqueBanks = (bankMapData: BankMap | null): number[] => {
    if (!bankMapData) return [];
    const banks = new Set(bankMapData.entries.map(e => e.bank));
    return Array.from(banks).sort((a, b) => a - b);
  };

  // Get entries for a specific bank
  const getEntriesForBank = (bankMapData: BankMap | null, bankNum: number): BankMapEntry[] => {
    if (!bankMapData) return [];
    return bankMapData.entries.filter(e => e.bank === bankNum);
  };

  // Calculate bank usage stats
  const getBankStats = (bankMapData: BankMap | null, bankNum: number) => {
    const entries = getEntriesForBank(bankMapData, bankNum);
    const used = entries
      .filter(e => e.data_type !== 'free')
      .reduce((sum, e) => sum + e.size, 0);
    const free = entries
      .filter(e => e.data_type === 'free')
      .reduce((sum, e) => sum + e.size, 0);
    const total = used + free || bankMapData?.bank_size || 0x8000;
    return { used, free, total, percent: (used / total) * 100 };
  };

  // Get dominant type for bank coloring
  const getDominantType = (entries: BankMapEntry[]): string => {
    if (entries.length === 0) return 'unknown';
    const typeSizes: Record<string, number> = {};
    entries.forEach(e => {
      typeSizes[e.data_type] = (typeSizes[e.data_type] || 0) + e.size;
    });
    return Object.entries(typeSizes).sort((a, b) => b[1] - a[1])[0][0];
  };

  // Combined error display
  const error = bankMap.error || bankStats.error || fragAnalysis.error || defragPlan.error || defrag.error || freeRegions.error;

  const uniqueBanks = getUniqueBanks(bankMap.data);

  return (
    <div
      style={{
        padding: '1.5rem',
        backgroundColor: 'var(--bg-secondary)',
        borderRadius: '12px',
        border: '1px solid var(--border)',
        maxWidth: '1200px',
        margin: '0 auto',
      }}
    >
      {/* Header */}
      <div
        style={{
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
          marginBottom: '1.5rem',
          flexWrap: 'wrap',
          gap: '1rem',
        }}
      >
        <h2 style={{ margin: 0, display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
          <span>🏦</span>
          Bank Visualization
        </h2>
        
        {/* Tab Navigation */}
        <div
          style={{
            display: 'flex',
            gap: '0.5rem',
            backgroundColor: 'var(--bg-tertiary)',
            padding: '0.25rem',
            borderRadius: '8px',
          }}
        >
          {(['overview', 'fragmentation', 'search'] as const).map((tab) => (
            <button
              key={tab}
              onClick={() => setActiveTab(tab)}
              style={{
                padding: '0.5rem 1rem',
                borderRadius: '6px',
                border: 'none',
                backgroundColor: activeTab === tab ? 'var(--primary)' : 'transparent',
                color: activeTab === tab ? 'white' : 'var(--text-secondary)',
                cursor: 'pointer',
                fontSize: '0.9rem',
                fontWeight: 500,
                transition: 'all 0.2s ease',
              }}
            >
              {tab.charAt(0).toUpperCase() + tab.slice(1)}
            </button>
          ))}
        </div>
      </div>

      {/* Error Display */}
      {error && (
        <div
          style={{
            padding: '1rem',
            backgroundColor: 'rgba(248, 113, 113, 0.1)',
            border: '1px solid #f87171',
            borderRadius: '8px',
            color: '#f87171',
            marginBottom: '1rem',
          }}
        >
          ⚠️ {error}
        </div>
      )}

      {/* Success Display */}
      {defrag.success && (
        <div
          style={{
            padding: '1rem',
            backgroundColor: 'rgba(74, 222, 128, 0.1)',
            border: '1px solid #4ade80',
            borderRadius: '8px',
            color: '#4ade80',
            marginBottom: '1rem',
          }}
        >
          ✅ {defrag.success}
        </div>
      )}

      {/* Tab Content */}
      {activeTab === 'overview' && (
        <>
          {/* Statistics Panel */}
          {bankStats.data && (
            <StatisticsPanel 
              stats={bankStats.data} 
              formatBytes={formatBytes}
            />
          )}

          {/* Bank Map */}
          <div>
            <h3
              style={{
                fontSize: '0.9rem',
                marginBottom: '0.75rem',
                color: 'var(--text-secondary)',
              }}
            >
              Bank Map {bankMap.data && `(${uniqueBanks.length} banks, ${formatBytes(bankMap.data.total_size)})`}
            </h3>

            {bankMap.isLoading ? (
              <LoadingState message="Loading bank map..." />
            ) : (
              <div
                style={{
                  display: 'grid',
                  gridTemplateColumns: 'repeat(auto-fill, minmax(60px, 1fr))',
                  gap: '4px',
                  maxHeight: '400px',
                  overflowY: 'auto',
                  padding: '0.5rem',
                  backgroundColor: 'var(--bg-tertiary)',
                  borderRadius: '8px',
                }}
              >
                {uniqueBanks.map((bankNum) => {
                  const entries = getEntriesForBank(bankMap.data, bankNum);
                  const dominantType = getDominantType(entries);
                  const stats = getBankStats(bankMap.data, bankNum);
                  
                  return (
                    <BankBlock
                      key={bankNum}
                      bankNum={bankNum}
                      dominantType={dominantType}
                      stats={stats}
                      isSelected={selectedBank === bankNum}
                      onSelect={() => setSelectedBank(bankNum)}
                      onMouseEnter={(e) => entries[0] && handleMouseEnter(entries[0], e)}
                      onMouseMove={handleMouseMove}
                      onMouseLeave={handleMouseLeave}
                    />
                  );
                })}
              </div>
            )}
          </div>

          {/* Selected Bank Details */}
          {selectedBank !== null && bankMap.data && (
            <BankDetails
              bankNum={selectedBank}
              entries={getEntriesForBank(bankMap.data, selectedBank)}
              bankSize={bankMap.data.bank_size}
              onClose={() => setSelectedBank(null)}
              formatBytes={formatBytes}
              formatAddress={formatAddress}
            />
          )}
        </>
      )}

      {activeTab === 'fragmentation' && (
        <FragmentationPanel
          analysis={fragAnalysis.data}
          plan={defragPlan.data}
          isAnalyzing={fragAnalysis.isLoading}
          isGeneratingPlan={defragPlan.isLoading}
          isExecuting={defrag.isLoading}
          onAnalyze={fragAnalysis.refresh}
          onGeneratePlan={defragPlan.refresh}
          onExecute={() => setShowConfirmDialog(true)}
          getSafetyDisplay={getSafetyDisplay}
          formatBytes={formatBytes}
          formatAddress={formatAddress}
        />
      )}

      {activeTab === 'search' && (
        <FreeRegionSearch
          regions={freeRegions.regions}
          minSizeInput={minSizeInput}
          setMinSizeInput={setMinSizeInput}
          onSearch={handleSearch}
          isLoading={freeRegions.isLoading}
          formatBytes={formatBytes}
          formatAddress={formatAddress}
        />
      )}

      {/* Tooltip */}
      {tooltip.visible && tooltip.entry && (
        <div
          style={{
            position: 'fixed',
            left: tooltip.x,
            top: tooltip.y,
            backgroundColor: 'var(--bg-secondary)',
            border: '1px solid var(--border)',
            borderRadius: '8px',
            padding: '0.75rem',
            boxShadow: '0 4px 12px rgba(0,0,0,0.3)',
            zIndex: 1000,
            pointerEvents: 'none',
            minWidth: '200px',
          }}
        >
          <div style={{ fontWeight: 600, marginBottom: '0.5rem' }}>
            Bank {tooltip.entry.bank}
          </div>
          <div style={{ fontSize: '0.85rem', color: 'var(--text-dim)' }}>
            <div>Type: {TYPE_LABELS[tooltip.entry.data_type] || tooltip.entry.data_type}</div>
            <div>Size: {formatBytes(tooltip.entry.size)}</div>
            <div>Start: {formatAddress(tooltip.entry.start_addr)}</div>
            <div>End: {formatAddress(tooltip.entry.end_addr)}</div>
            {tooltip.entry.description && (
              <div style={{ marginTop: '0.5rem', fontStyle: 'italic' }}>
                {tooltip.entry.description}
              </div>
            )}
          </div>
        </div>
      )}

      {/* Confirmation Dialog */}
      {showConfirmDialog && defragPlan.data && (
        <ConfirmDialog
          plan={defragPlan.data}
          getSafetyDisplay={getSafetyDisplay}
          formatBytes={formatBytes}
          onCancel={() => setShowConfirmDialog(false)}
          onConfirm={handleExecute}
          isExecuting={defrag.isLoading}
        />
      )}
    </div>
  );
};

// ============================================================================
// Sub-components
// ============================================================================

// Statistics Panel Component
interface StatisticsPanelProps {
  stats: BankStatistics;
  formatBytes: (bytes: number) => string;
}

const StatisticsPanel = ({ stats, formatBytes }: StatisticsPanelProps) => {
  const usedPercent = (stats.total_used / stats.total_capacity) * 100;
  const freePercent = (stats.total_free / stats.total_capacity) * 100;
  
  // Build breakdown from bank stats
  const typeStats: Record<string, number> = {};
  stats.banks.forEach(bank => {
    const type = bank.usage_percent > 80 ? 'code' : bank.usage_percent > 50 ? 'data' : 'free';
    typeStats[type] = (typeStats[type] || 0) + bank.used;
  });
  
  const breakdown = Object.entries(typeStats).map(([type, bytes]) => ({
    type,
    bytes,
    percentage: (bytes / stats.total_capacity) * 100,
  }));

  return (
    <div
      style={{
        marginBottom: '1.5rem',
        padding: '1rem',
        backgroundColor: 'var(--bg-tertiary)',
        borderRadius: '8px',
      }}
    >
      <h3
        style={{
          fontSize: '0.9rem',
          marginBottom: '0.75rem',
          color: 'var(--text-secondary)',
        }}
      >
        ROM Statistics
      </h3>
      <div
        style={{
          display: 'grid',
          gridTemplateColumns: 'repeat(auto-fit, minmax(140px, 1fr))',
          gap: '1rem',
        }}
      >
        <StatCard label="Total Size" value={formatBytes(stats.total_capacity)} />
        <StatCard label="Used Space" value={`${usedPercent.toFixed(1)}%`} color="#fbbf24" />
        <StatCard label="Free Space" value={`${freePercent.toFixed(1)}%`} color="#4ade80" />
        <StatCard label="Banks" value={stats.banks.length.toString()} color="#60a5fa" />
      </div>

      {/* Usage Bar */}
      <div
        style={{
          marginTop: '1rem',
          height: '12px',
          backgroundColor: 'var(--bg-secondary)',
          borderRadius: '6px',
          overflow: 'hidden',
          display: 'flex',
        }}
      >
        {breakdown.map((item, idx) => (
          <div
            key={idx}
            style={{
              width: `${item.percentage}%`,
              height: '100%',
              backgroundColor: TYPE_COLORS[item.type] || TYPE_COLORS.unknown,
              transition: 'width 0.3s ease',
            }}
            title={`${TYPE_LABELS[item.type] || item.type}: ${formatBytes(item.bytes)} (${item.percentage.toFixed(1)}%)`}
          />
        ))}
      </div>

      {/* Legend */}
      <div
        style={{
          display: 'flex',
          flexWrap: 'wrap',
          gap: '0.75rem',
          marginTop: '0.75rem',
        }}
      >
        {breakdown.map((item, idx) => (
          <div
            key={idx}
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: '0.4rem',
              fontSize: '0.8rem',
              color: 'var(--text-dim)',
            }}
          >
            <span
              style={{
                width: '12px',
                height: '12px',
                borderRadius: '3px',
                backgroundColor: TYPE_COLORS[item.type] || TYPE_COLORS.unknown,
              }}
            />
            {TYPE_LABELS[item.type] || item.type}
          </div>
        ))}
      </div>
    </div>
  );
};

// Bank Block Component
interface BankBlockProps {
  bankNum: number;
  dominantType: string;
  stats: { used: number; free: number; total: number; percent: number };
  isSelected: boolean;
  onSelect: () => void;
  onMouseEnter: (e: React.MouseEvent) => void;
  onMouseMove: (e: React.MouseEvent) => void;
  onMouseLeave: () => void;
}

const BankBlock = ({
  bankNum,
  dominantType,
  isSelected,
  onSelect,
  onMouseEnter,
  onMouseMove,
  onMouseLeave,
}: BankBlockProps) => {
  const color = TYPE_COLORS[dominantType] || TYPE_COLORS.unknown;
  const isDark = dominantType !== 'free' && dominantType !== 'text';

  return (
    <div
      onClick={onSelect}
      onMouseEnter={onMouseEnter}
      onMouseMove={onMouseMove}
      onMouseLeave={onMouseLeave}
      style={{
        aspectRatio: '1',
        backgroundColor: color,
        borderRadius: '4px',
        cursor: 'pointer',
        border: isSelected ? '2px solid white' : '2px solid transparent',
        boxShadow: isSelected ? '0 0 8px rgba(255,255,255,0.5)' : 'none',
        transition: 'all 0.15s ease',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        fontSize: '0.65rem',
        color: isDark ? '#fff' : '#000',
        fontWeight: 600,
      }}
      title={`Bank ${bankNum}: ${TYPE_LABELS[dominantType] || dominantType}`}
    >
      {bankNum}
    </div>
  );
};

// Bank Details Panel
interface BankDetailsProps {
  bankNum: number;
  entries: BankMapEntry[];
  bankSize: number;
  onClose: () => void;
  formatBytes: (bytes: number) => string;
  formatAddress: (addr: number) => string;
}

const BankDetails = ({ bankNum, entries, bankSize, onClose, formatBytes, formatAddress }: BankDetailsProps) => {
  const used = entries
    .filter(e => e.data_type !== 'free')
    .reduce((sum, e) => sum + e.size, 0);
  const percent = (used / bankSize) * 100;

  return (
    <div
      style={{
        marginTop: '1.5rem',
        padding: '1rem',
        backgroundColor: 'var(--bg-tertiary)',
        borderRadius: '8px',
        border: '1px solid var(--border)',
      }}
    >
      <div
        style={{
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
          marginBottom: '1rem',
        }}
      >
        <div>
          <h3 style={{ margin: 0 }}>Bank {bankNum} Details</h3>
          <div style={{ fontSize: '0.8rem', color: 'var(--text-dim)', marginTop: '0.25rem' }}>
            {formatBytes(used)} used ({percent.toFixed(1)}%) · {formatBytes(bankSize - used)} free
          </div>
        </div>
        <button
          onClick={onClose}
          style={{
            padding: '0.25rem 0.5rem',
            fontSize: '0.8rem',
            backgroundColor: 'transparent',
            border: '1px solid var(--border)',
            borderRadius: '4px',
            cursor: 'pointer',
          }}
        >
          Close
        </button>
      </div>

      <div style={{ display: 'grid', gap: '0.5rem' }}>
        {entries.map((entry, idx) => (
          <div
            key={idx}
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: '0.75rem',
              padding: '0.5rem',
              backgroundColor: 'var(--bg-secondary)',
              borderRadius: '6px',
            }}
          >
            <span
              style={{
                width: '16px',
                height: '16px',
                borderRadius: '4px',
                backgroundColor: TYPE_COLORS[entry.data_type] || TYPE_COLORS.unknown,
                flexShrink: 0,
              }}
            />
            <div style={{ flex: 1 }}>
              <div style={{ fontWeight: 500, fontSize: '0.9rem' }}>
                {TYPE_LABELS[entry.data_type] || entry.data_type}
              </div>
              <div style={{ fontSize: '0.75rem', color: 'var(--text-dim)' }}>
                {formatAddress(entry.start_addr)} - {formatAddress(entry.end_addr)} · {formatBytes(entry.size)}
              </div>
            </div>
            {entry.description && (
              <div style={{ fontSize: '0.8rem', color: 'var(--text-dim)', fontStyle: 'italic' }}>
                {entry.description}
              </div>
            )}
          </div>
        ))}
      </div>
    </div>
  );
};

// Fragmentation Panel Component
interface FragmentationPanelProps {
  analysis: FragmentationAnalysis | null;
  plan: DefragPlan | null;
  isAnalyzing: boolean;
  isGeneratingPlan: boolean;
  isExecuting: boolean;
  onAnalyze: () => void;
  onGeneratePlan: () => void;
  onExecute: () => void;
  getSafetyDisplay: (score: number) => React.ReactNode;
  formatBytes: (bytes: number) => string;
  formatAddress: (addr: number) => string;
}

const FragmentationPanel = ({
  analysis,
  plan,
  isAnalyzing,
  isGeneratingPlan,
  isExecuting,
  onAnalyze,
  onGeneratePlan,
  onExecute,
  getSafetyDisplay,
  formatBytes,
  formatAddress,
}: FragmentationPanelProps) => {
  return (
    <div>
      {/* Control Buttons */}
      <div
        style={{
          display: 'flex',
          gap: '0.75rem',
          marginBottom: '1.5rem',
          flexWrap: 'wrap',
        }}
      >
        <button
          onClick={onAnalyze}
          disabled={isAnalyzing}
          style={{
            padding: '0.75rem 1.5rem',
            display: 'flex',
            alignItems: 'center',
            gap: '0.5rem',
          }}
        >
          {isAnalyzing ? '⏳ Analyzing...' : '🔍 Analyze Fragmentation'}
        </button>

        <button
          onClick={onGeneratePlan}
          disabled={isGeneratingPlan || !analysis}
          style={{
            padding: '0.75rem 1.5rem',
            display: 'flex',
            alignItems: 'center',
            gap: '0.5rem',
          }}
        >
          {isGeneratingPlan ? '⏳ Generating...' : '📋 Generate Plan'}
        </button>

        <button
          onClick={onExecute}
          disabled={isExecuting || !plan}
          style={{
            padding: '0.75rem 1.5rem',
            display: 'flex',
            alignItems: 'center',
            gap: '0.5rem',
            backgroundColor: '#f97316',
          }}
        >
          {isExecuting ? '⏳ Executing...' : '⚡ Execute Defragmentation'}
        </button>
      </div>

      {/* Fragmentation Analysis */}
      {analysis && (
        <div
          style={{
            padding: '1rem',
            backgroundColor: 'var(--bg-tertiary)',
            borderRadius: '8px',
            marginBottom: '1rem',
          }}
        >
          <h3
            style={{
              fontSize: '1rem',
              marginBottom: '0.75rem',
              display: 'flex',
              alignItems: 'center',
              gap: '0.5rem',
            }}
          >
            📊 Fragmentation Analysis
            {getSafetyDisplay(analysis.fragmentation_score)}
          </h3>

          <div
            style={{
              display: 'grid',
              gridTemplateColumns: 'repeat(auto-fit, minmax(150px, 1fr))',
              gap: '1rem',
              marginBottom: '1rem',
            }}
          >
            <StatCard
              label="Fragmentation Score"
              value={`${analysis.fragmentation_score.toFixed(1)}%`}
              color={analysis.fragmentation_score > 50 ? '#f87171' : '#4ade80'}
            />
            <StatCard label="Free Regions" value={analysis.free_region_count.toString()} />
            <StatCard label="Total Free" value={formatBytes(analysis.total_free_bytes)} color="#4ade80" />
            <StatCard
              label="Avg Region Size"
              value={formatBytes(analysis.avg_free_region_size)}
              color="#60a5fa"
            />
          </div>

          {analysis.largest_free_region && (
            <div style={{ marginBottom: '1rem' }}>
              <h4 style={{ fontSize: '0.9rem', marginBottom: '0.5rem', color: 'var(--text-secondary)' }}>
                Largest Free Region
              </h4>
              <div
                style={{
                  padding: '0.75rem',
                  backgroundColor: 'var(--bg-secondary)',
                  borderRadius: '6px',
                  display: 'flex',
                  justifyContent: 'space-between',
                  alignItems: 'center',
                }}
              >
                <span style={{ fontFamily: 'monospace' }}>
                  Bank {analysis.largest_free_region.bank}: {formatAddress(analysis.largest_free_region.start_addr)} - {formatAddress(analysis.largest_free_region.end_addr)}
                </span>
                <span style={{ color: '#4ade80', fontWeight: 600 }}>
                  {formatBytes(analysis.largest_free_region.size)}
                </span>
              </div>
            </div>
          )}

          {/* Consolidation Opportunities */}
          {analysis.consolidation_opportunities.length > 0 && (
            <div>
              <h4 style={{ fontSize: '0.9rem', marginBottom: '0.5rem', color: 'var(--text-secondary)' }}>
                Consolidation Opportunities
              </h4>
              <div
                style={{
                  maxHeight: '150px',
                  overflowY: 'auto',
                  border: '1px solid var(--border)',
                  borderRadius: '6px',
                }}
              >
                {analysis.consolidation_opportunities.map((opp, idx) => (
                  <div
                    key={idx}
                    style={{
                      padding: '0.5rem 0.75rem',
                      borderBottom: idx < analysis.consolidation_opportunities.length - 1 ? '1px solid var(--border)' : 'none',
                      display: 'flex',
                      justifyContent: 'space-between',
                      alignItems: 'center',
                      fontSize: '0.85rem',
                    }}
                  >
                    <span>{opp.regions.length} regions can be consolidated</span>
                    <span style={{ color: '#4ade80' }}>Save {formatBytes(opp.potential_savings)}</span>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      )}

      {/* Defragmentation Plan */}
      {plan && (
        <div
          style={{
            padding: '1rem',
            backgroundColor: 'var(--bg-tertiary)',
            borderRadius: '8px',
          }}
        >
          <h3
            style={{
              fontSize: '1rem',
              marginBottom: '0.75rem',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'space-between',
            }}
          >
            <span>📋 Defragmentation Plan</span>
            {plan.recommended ? (
              <span style={{ color: '#4ade80', fontSize: '0.85rem' }}>✓ Recommended</span>
            ) : (
              <span style={{ color: '#fbbf24', fontSize: '0.85rem' }}>⚠ Not Recommended</span>
            )}
          </h3>

          <div
            style={{
              display: 'grid',
              gridTemplateColumns: 'repeat(auto-fit, minmax(150px, 1fr))',
              gap: '1rem',
              marginBottom: '1rem',
            }}
          >
            <StatCard label="Operations" value={plan.operation_count.toString()} />
            <StatCard
              label="Data to Move"
              value={formatBytes(plan.total_bytes_to_move)}
              color="#60a5fa"
            />
            <StatCard
              label="Est. Time"
              value={`${plan.estimated_time_ms}ms`}
              color="#a78bfa"
            />
            <StatCard
              label="Score Improvement"
              value={`${plan.current_score.toFixed(0)}% → ${plan.projected_score.toFixed(0)}%`}
              color="#4ade80"
            />
          </div>

          {plan.warnings.length > 0 && (
            <div
              style={{
                padding: '0.75rem',
                backgroundColor: 'rgba(251, 191, 36, 0.1)',
                borderRadius: '6px',
                marginBottom: '1rem',
              }}
            >
              <h4 style={{ fontSize: '0.85rem', marginBottom: '0.5rem', color: '#fbbf24' }}>
                ⚠ Warnings
              </h4>
              <ul
                style={{
                  margin: 0,
                  paddingLeft: '1.25rem',
                  fontSize: '0.85rem',
                  color: 'var(--text-dim)',
                }}
              >
                {plan.warnings.map((warning, idx) => (
                  <li key={idx} style={{ marginBottom: '0.25rem' }}>
                    {warning}
                  </li>
                ))}
              </ul>
            </div>
          )}

          {/* Steps Preview */}
          {plan.steps.length > 0 && (
            <div>
              <h4 style={{ fontSize: '0.9rem', marginBottom: '0.5rem', color: 'var(--text-secondary)' }}>
                Steps Preview ({Math.min(plan.steps.length, 10)} of {plan.steps.length})
              </h4>
              <div
                style={{
                  maxHeight: '150px',
                  overflowY: 'auto',
                  border: '1px solid var(--border)',
                  borderRadius: '6px',
                }}
              >
                {plan.steps.slice(0, 10).map((step, idx) => (
                  <StepRow key={idx} step={step} formatAddress={formatAddress} formatBytes={formatBytes} />
                ))}
                {plan.steps.length > 10 && (
                  <div
                    style={{
                      padding: '0.5rem 0.75rem',
                      fontSize: '0.8rem',
                      color: 'var(--text-dim)',
                      textAlign: 'center',
                    }}
                  >
                    ...and {plan.steps.length - 10} more steps
                  </div>
                )}
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  );
};

// Step Row Component
interface StepRowProps {
  step: DefragStep;
  formatAddress: (addr: number) => string;
  formatBytes: (bytes: number) => string;
}

const StepRow = ({ step, formatAddress, formatBytes }: StepRowProps) => (
  <div
    style={{
      padding: '0.5rem 0.75rem',
      borderBottom: '1px solid var(--border)',
      fontSize: '0.8rem',
    }}
  >
    <div style={{ fontWeight: 500, marginBottom: '0.25rem' }}>
      Step {step.step}: {step.description}
    </div>
    <div style={{ fontFamily: 'monospace', color: 'var(--text-dim)' }}>
      Bank {step.source.bank} {formatAddress(step.source.start_addr)} → Bank {step.destination.bank} {formatAddress(step.destination.start_addr)} ({formatBytes(step.size)})
    </div>
  </div>
);

// Free Region Search Component
interface FreeRegionSearchProps {
  regions: MemoryRegion[];
  minSizeInput: string;
  setMinSizeInput: (value: string) => void;
  onSearch: () => void;
  isLoading: boolean;
  formatBytes: (bytes: number) => string;
  formatAddress: (addr: number) => string;
}

const FreeRegionSearch = ({
  regions,
  minSizeInput,
  setMinSizeInput,
  onSearch,
  isLoading,
  formatBytes,
  formatAddress,
}: FreeRegionSearchProps) => {
  return (
    <div>
      {/* Search Input */}
      <div
        style={{
          display: 'flex',
          gap: '0.75rem',
          marginBottom: '1.5rem',
          alignItems: 'center',
        }}
      >
        <label style={{ fontWeight: 500 }}>Minimum Size:</label>
        <input
          type="number"
          value={minSizeInput}
          onChange={(e) => setMinSizeInput(e.target.value)}
          placeholder="e.g., 1024"
          min="0"
          style={{
            flex: 1,
            maxWidth: '200px',
            padding: '0.5rem 0.75rem',
            backgroundColor: 'var(--bg-tertiary)',
            border: '1px solid var(--border)',
            borderRadius: '6px',
            color: 'var(--text-primary)',
            fontFamily: 'monospace',
          }}
        />
        <span style={{ color: 'var(--text-dim)' }}>bytes</span>
        <button onClick={onSearch} disabled={isLoading} style={{ padding: '0.5rem 1.5rem' }}>
          {isLoading ? '⏳ Searching...' : '🔍 Search'}
        </button>
      </div>

      {/* Results */}
      {regions.length > 0 ? (
        <div>
          <h3
            style={{
              fontSize: '0.9rem',
              marginBottom: '0.75rem',
              color: 'var(--text-secondary)',
            }}
          >
            Found {regions.length} free region{regions.length !== 1 ? 's' : ''}
          </h3>
          <div
            style={{
              maxHeight: '400px',
              overflowY: 'auto',
              border: '1px solid var(--border)',
              borderRadius: '8px',
            }}
          >
            {regions.map((region, idx) => (
              <div
                key={idx}
                style={{
                  padding: '0.75rem 1rem',
                  borderBottom: idx < regions.length - 1 ? '1px solid var(--border)' : 'none',
                  backgroundColor: idx % 2 === 0 ? 'var(--bg-tertiary)' : 'transparent',
                  display: 'flex',
                  justifyContent: 'space-between',
                  alignItems: 'center',
                }}
              >
                <div>
                  <div style={{ fontFamily: 'monospace', fontSize: '0.9rem', fontWeight: 500 }}>
                    Bank {region.bank}: {formatAddress(region.start_addr)} - {formatAddress(region.end_addr)}
                  </div>
                  {region.description && (
                    <div style={{ fontSize: '0.75rem', color: 'var(--text-dim)', marginTop: '0.25rem' }}>
                      {region.description}
                    </div>
                  )}
                </div>
                <div
                  style={{
                    padding: '0.4rem 0.8rem',
                    backgroundColor: '#4ade80',
                    color: '#000',
                    borderRadius: '6px',
                    fontSize: '0.85rem',
                    fontWeight: 600,
                  }}
                >
                  {formatBytes(region.size)}
                </div>
              </div>
            ))}
          </div>
        </div>
      ) : (
        <EmptyState
          icon="🔍"
          message={minSizeInput ? 'No free regions found matching your criteria.' : 'Enter a minimum size and click Search to find free regions.'}
        />
      )}
    </div>
  );
};

// Confirmation Dialog Component
interface ConfirmDialogProps {
  plan: DefragPlan;
  getSafetyDisplay: (score: number) => React.ReactNode;
  formatBytes: (bytes: number) => string;
  onCancel: () => void;
  onConfirm: () => void;
  isExecuting: boolean;
}

const ConfirmDialog = ({ plan, getSafetyDisplay, formatBytes, onCancel, onConfirm, isExecuting }: ConfirmDialogProps) => (
  <div
    style={{
      position: 'fixed',
      top: 0,
      left: 0,
      right: 0,
      bottom: 0,
      backgroundColor: 'rgba(0, 0, 0, 0.7)',
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'center',
      zIndex: 1000,
    }}
    onClick={onCancel}
  >
    <div
      style={{
        backgroundColor: 'var(--bg-secondary)',
        padding: '1.5rem',
        borderRadius: '12px',
        maxWidth: '450px',
        width: '90%',
        border: '1px solid var(--border)',
      }}
      onClick={(e) => e.stopPropagation()}
    >
      <h3 style={{ marginBottom: '1rem', color: '#f87171' }}>
        ⚠️ Confirm Defragmentation
      </h3>
      <p style={{ marginBottom: '1rem', fontSize: '0.9rem' }}>
        This operation will rearrange data in the ROM to optimize space usage.
        Make sure you have a backup before proceeding.
      </p>
      <div
        style={{
          padding: '0.75rem',
          backgroundColor: 'var(--bg-tertiary)',
          borderRadius: '6px',
          marginBottom: '1rem',
          fontSize: '0.85rem',
        }}
      >
        <div>Operations: {plan.operation_count}</div>
        <div>Data to move: {formatBytes(plan.total_bytes_to_move)}</div>
        <div style={{ marginTop: '0.5rem' }}>
          Status: {getSafetyDisplay(plan.current_score)}
        </div>
      </div>
      <div
        style={{
          padding: '0.75rem',
          backgroundColor: 'rgba(248, 113, 113, 0.1)',
          borderRadius: '6px',
          marginBottom: '1rem',
          fontSize: '0.85rem',
          color: '#f87171',
        }}
      >
        <strong>Warning:</strong> This will modify the ROM file. This action cannot be undone.
      </div>
      <div style={{ display: 'flex', gap: '0.5rem', justifyContent: 'flex-end' }}>
        <button
          onClick={onCancel}
          style={{
            padding: '0.5rem 1rem',
            backgroundColor: 'var(--bg-tertiary)',
            border: '1px solid var(--border)',
            borderRadius: '6px',
            cursor: 'pointer',
          }}
        >
          Cancel
        </button>
        <button
          onClick={onConfirm}
          disabled={isExecuting || !plan.recommended}
          style={{
            padding: '0.5rem 1rem',
            backgroundColor: !plan.recommended ? '#636e72' : '#f97316',
            color: 'white',
            border: 'none',
            borderRadius: '6px',
            cursor: isExecuting || !plan.recommended ? 'not-allowed' : 'pointer',
            opacity: isExecuting || !plan.recommended ? 0.6 : 1,
          }}
        >
          {isExecuting ? '⏳ Executing...' : 'Execute Defragmentation'}
        </button>
      </div>
    </div>
  </div>
);

// Stat Card Component
interface StatCardProps {
  label: string;
  value: string;
  color?: string;
}

const StatCard = ({ label, value, color }: StatCardProps) => (
  <div
    style={{
      padding: '0.75rem',
      backgroundColor: 'var(--bg-secondary)',
      borderRadius: '6px',
      textAlign: 'center',
    }}
  >
    <div style={{ fontSize: '0.75rem', color: 'var(--text-dim)', marginBottom: '0.25rem' }}>
      {label}
    </div>
    <div
      style={{
        fontSize: '1.1rem',
        fontWeight: 700,
        color: color || 'var(--text-primary)',
      }}
    >
      {value}
    </div>
  </div>
);

// Loading State Component
interface LoadingStateProps {
  message: string;
}

const LoadingState = ({ message }: LoadingStateProps) => (
  <div
    style={{
      padding: '3rem',
      textAlign: 'center',
      color: 'var(--text-dim)',
    }}
  >
    <div style={{ fontSize: '1.5rem', marginBottom: '0.5rem' }}>⏳</div>
    {message}
  </div>
);

// Empty State Component
interface EmptyStateProps {
  icon: string;
  message: string;
}

const EmptyState = ({ icon, message }: EmptyStateProps) => (
  <div
    style={{
      padding: '3rem',
      textAlign: 'center',
      color: 'var(--text-dim)',
      backgroundColor: 'var(--bg-tertiary)',
      borderRadius: '8px',
    }}
  >
    <div style={{ fontSize: '2rem', marginBottom: '0.5rem' }}>{icon}</div>
    {message}
  </div>
);

export default BankVisualization;
