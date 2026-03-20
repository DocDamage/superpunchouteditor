import type { ComparisonData } from './BoxerCompare';
import './StatComparisonTable.css';

interface StatComparisonTableProps {
  comparison: ComparisonData;
  boxerAName: string;
  boxerBName: string;
  onCopyStat: (field: string, source: 'a' | 'b') => void;
  onCopyAll: (source: 'a' | 'b') => void;
}

interface StatRow {
  field: string;
  label: string;
  valueA: number;
  valueB: number;
  unit?: string;
}

export function StatComparisonTable({
  comparison,
  boxerAName,
  boxerBName,
  onCopyStat,
  onCopyAll,
}: StatComparisonTableProps) {
  const { stat_comparison, asset_comparison } = comparison;

  const statRows: StatRow[] = [
    {
      field: 'attack',
      label: 'Attack Power',
      valueA: stat_comparison.attack[0],
      valueB: stat_comparison.attack[1],
    },
    {
      field: 'defense',
      label: 'Defense Rating',
      valueA: stat_comparison.defense[0],
      valueB: stat_comparison.defense[1],
    },
    {
      field: 'speed',
      label: 'Speed Rating',
      valueA: stat_comparison.speed[0],
      valueB: stat_comparison.speed[1],
    },
    {
      field: 'palette_id',
      label: 'Palette ID',
      valueA: stat_comparison.palette_id[0],
      valueB: stat_comparison.palette_id[1],
    },
    {
      field: 'unique_bins',
      label: 'Unique Bins',
      valueA: asset_comparison.unique_count_a,
      valueB: asset_comparison.unique_count_b,
    },
    {
      field: 'shared_bins',
      label: 'Shared Bins',
      valueA: asset_comparison.shared_count,
      valueB: asset_comparison.shared_count,
    },
  ];

  const getDiff = (a: number, b: number): number => b - a;

  const getDiffClass = (diff: number): string => {
    if (diff === 0) return 'diff-zero';
    if (Math.abs(diff) <= 5) return 'diff-small';
    if (Math.abs(diff) <= 15) return 'diff-medium';
    return 'diff-large';
  };

  const getDiffIndicator = (diff: number): string => {
    if (diff > 0) return `+${diff} 🔴`;
    if (diff < 0) return `${diff} 🟢`;
    return '= ⚪';
  };

  return (
    <div className="stat-comparison">
      <div className="stat-header">
        <div className="boxer-header">
          <h3>{boxerAName}</h3>
          <button
            className="copy-all-btn"
            onClick={() => onCopyAll('a')}
            title={`Copy all stats from ${boxerAName} to ${boxerBName}`}
          >
            Copy all →
          </button>
        </div>
        <div className="vs-divider">VS</div>
        <div className="boxer-header">
          <h3>{boxerBName}</h3>
          <button
            className="copy-all-btn"
            onClick={() => onCopyAll('b')}
            title={`Copy all stats from ${boxerBName} to ${boxerAName}`}
          >
            ← Copy all
          </button>
        </div>
      </div>

      <div className="stat-table-container">
        <table className="stat-table">
          <thead>
            <tr>
              <th className="col-stat">Stat</th>
              <th className="col-value">{boxerAName}</th>
              <th className="col-diff">Diff</th>
              <th className="col-value">{boxerBName}</th>
              <th className="col-action">Copy</th>
            </tr>
          </thead>
          <tbody>
            {statRows.map((row) => {
              const diff = getDiff(row.valueA, row.valueB);
              const isDifferent = diff !== 0;
              const canCopy = row.field !== 'unique_bins' && row.field !== 'shared_bins';

              return (
                <tr
                  key={row.field}
                  className={isDifferent ? 'row-different' : 'row-same'}
                >
                  <td className="col-stat">
                    <span className="stat-name">{row.label}</span>
                  </td>
                  <td className="col-value">
                    <span className={`stat-value ${!isDifferent ? 'same' : ''}`}>
                      {row.valueA}
                    </span>
                  </td>
                  <td className="col-diff">
                    <span className={`diff-badge ${getDiffClass(diff)}`}>
                      {getDiffIndicator(diff)}
                    </span>
                  </td>
                  <td className="col-value">
                    <span className={`stat-value ${!isDifferent ? 'same' : ''}`}>
                      {row.valueB}
                    </span>
                  </td>
                  <td className="col-action">
                    {canCopy ? (
                      <div className="copy-actions">
                        <button
                          className="copy-btn"
                          onClick={() => onCopyStat(row.field, 'a')}
                          title={`Copy ${row.label} from ${boxerAName}`}
                          disabled={!isDifferent}
                        >
                          →
                        </button>
                        <button
                          className="copy-btn"
                          onClick={() => onCopyStat(row.field, 'b')}
                          title={`Copy ${row.label} from ${boxerBName}`}
                          disabled={!isDifferent}
                        >
                          ←
                        </button>
                      </div>
                    ) : (
                      <span className="no-copy">—</span>
                    )}
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>

      <div className="stat-legend">
        <div className="legend-item">
          <span className="legend-badge diff-zero">= ⚪</span>
          <span>Identical</span>
        </div>
        <div className="legend-item">
          <span className="legend-badge diff-small">±1-5 🟡</span>
          <span>Small difference</span>
        </div>
        <div className="legend-item">
          <span className="legend-badge diff-medium">±6-15 🟠</span>
          <span>Medium difference</span>
        </div>
        <div className="legend-item">
          <span className="legend-badge diff-large">±16+ 🔴</span>
          <span>Large difference</span>
        </div>
      </div>

      {stat_comparison.differences.length > 0 && (
        <div className="differences-summary">
          <h4>Differences Found:</h4>
          <ul>
            {stat_comparison.differences.map((diff) => (
              <li key={diff}>
                {diff.charAt(0).toUpperCase() + diff.slice(1).replace('_', ' ')}
              </li>
            ))}
          </ul>
        </div>
      )}
    </div>
  );
}
