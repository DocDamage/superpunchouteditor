import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface ChangeSummary {
  total_boxers_modified: number;
  total_palettes_changed: number;
  total_sprites_edited: number;
  total_animations_modified: number;
  total_headers_edited: number;
  total_changes: number;
}

interface ChangeStatsProps {
  refreshTrigger?: number;
}

export const ChangeStats = ({ refreshTrigger = 0 }: ChangeStatsProps) => {
  const [summary, setSummary] = useState<ChangeSummary | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadSummary();
  }, [refreshTrigger]);

  const loadSummary = async () => {
    try {
      setLoading(true);
      const data = await invoke<ChangeSummary>('get_change_summary');
      setSummary(data);
    } catch (e) {
      console.error('Failed to load change summary:', e);
    } finally {
      setLoading(false);
    }
  };

  if (loading) {
    return (
      <div style={{
        backgroundColor: 'var(--panel-bg)',
        border: '1px solid var(--border)',
        borderRadius: '12px',
        padding: '1.5rem',
        textAlign: 'center',
        color: 'var(--text-dim)',
      }}>
        <div style={{ fontSize: '1.5rem', marginBottom: '0.5rem' }}>📊</div>
        <div>Loading statistics...</div>
      </div>
    );
  }

  if (!summary || summary.total_changes === 0) {
    return (
      <div style={{
        backgroundColor: 'var(--panel-bg)',
        border: '1px solid var(--border)',
        borderRadius: '12px',
        padding: '1.5rem',
        textAlign: 'center',
        color: 'var(--text-dim)',
      }}>
        <div style={{ fontSize: '2rem', marginBottom: '0.5rem' }}>📊</div>
        <div style={{ fontWeight: 600, marginBottom: '0.5rem' }}>No Changes Yet</div>
        <div style={{ fontSize: '0.85rem' }}>Make some edits to see statistics</div>
      </div>
    );
  }

  const maxValue = Math.max(
    summary.total_palettes_changed,
    summary.total_sprites_edited,
    summary.total_animations_modified,
    summary.total_headers_edited,
    1
  );

  const getBarWidth = (value: number) => {
    return `${(value / maxValue) * 100}%`;
  };

  const getBarColor = (type: string) => {
    switch (type) {
      case 'palettes': return '#60a5fa'; // blue
      case 'sprites': return '#34d399'; // green
      case 'animations': return '#fbbf24'; // yellow
      case 'headers': return '#f87171'; // red
      default: return '#9ca3af';
    }
  };

  return (
    <div style={{
      backgroundColor: 'var(--panel-bg)',
      border: '1px solid var(--border)',
      borderRadius: '12px',
      padding: '1.5rem',
    }}>
      <div style={{
        fontWeight: 600,
        fontSize: '1rem',
        marginBottom: '1rem',
        display: 'flex',
        alignItems: 'center',
        gap: '0.5rem',
      }}>
        📊 Change Statistics
      </div>

      {/* Summary stats */}
      <div style={{
        display: 'grid',
        gridTemplateColumns: 'repeat(2, 1fr)',
        gap: '0.75rem',
        marginBottom: '1.5rem',
      }}>
        <div style={{
          backgroundColor: 'var(--glass)',
          padding: '1rem',
          borderRadius: '8px',
          textAlign: 'center',
        }}>
          <div style={{ fontSize: '1.5rem', fontWeight: 700, color: 'var(--blue)' }}>
            {summary.total_boxers_modified}
          </div>
          <div style={{ fontSize: '0.75rem', color: 'var(--text-dim)' }}>Boxers Modified</div>
        </div>
        <div style={{
          backgroundColor: 'var(--glass)',
          padding: '1rem',
          borderRadius: '8px',
          textAlign: 'center',
        }}>
          <div style={{ fontSize: '1.5rem', fontWeight: 700, color: '#fbbf24' }}>
            {summary.total_changes}
          </div>
          <div style={{ fontSize: '0.75rem', color: 'var(--text-dim)' }}>Total Changes</div>
        </div>
      </div>

      {/* Visual bars */}
      <div style={{ display: 'flex', flexDirection: 'column', gap: '0.75rem' }}>
        {/* Palettes */}
        {summary.total_palettes_changed > 0 && (
          <div>
            <div style={{
              display: 'flex',
              justifyContent: 'space-between',
              fontSize: '0.8rem',
              marginBottom: '0.25rem',
            }}>
              <span>Palettes</span>
              <span style={{ color: 'var(--text-dim)' }}>{summary.total_palettes_changed}</span>
            </div>
            <div style={{
              height: '8px',
              backgroundColor: 'var(--glass)',
              borderRadius: '4px',
              overflow: 'hidden',
            }}>
              <div style={{
                width: getBarWidth(summary.total_palettes_changed),
                height: '100%',
                backgroundColor: getBarColor('palettes'),
                borderRadius: '4px',
                transition: 'width 0.3s ease',
              }} />
            </div>
          </div>
        )}

        {/* Sprites */}
        {summary.total_sprites_edited > 0 && (
          <div>
            <div style={{
              display: 'flex',
              justifyContent: 'space-between',
              fontSize: '0.8rem',
              marginBottom: '0.25rem',
            }}>
              <span>Sprites</span>
              <span style={{ color: 'var(--text-dim)' }}>{summary.total_sprites_edited}</span>
            </div>
            <div style={{
              height: '8px',
              backgroundColor: 'var(--glass)',
              borderRadius: '4px',
              overflow: 'hidden',
            }}>
              <div style={{
                width: getBarWidth(summary.total_sprites_edited),
                height: '100%',
                backgroundColor: getBarColor('sprites'),
                borderRadius: '4px',
                transition: 'width 0.3s ease',
              }} />
            </div>
          </div>
        )}

        {/* Animations */}
        {summary.total_animations_modified > 0 && (
          <div>
            <div style={{
              display: 'flex',
              justifyContent: 'space-between',
              fontSize: '0.8rem',
              marginBottom: '0.25rem',
            }}>
              <span>Animations</span>
              <span style={{ color: 'var(--text-dim)' }}>{summary.total_animations_modified}</span>
            </div>
            <div style={{
              height: '8px',
              backgroundColor: 'var(--glass)',
              borderRadius: '4px',
              overflow: 'hidden',
            }}>
              <div style={{
                width: getBarWidth(summary.total_animations_modified),
                height: '100%',
                backgroundColor: getBarColor('animations'),
                borderRadius: '4px',
                transition: 'width 0.3s ease',
              }} />
            </div>
          </div>
        )}

        {/* Stats/Headers */}
        {summary.total_headers_edited > 0 && (
          <div>
            <div style={{
              display: 'flex',
              justifyContent: 'space-between',
              fontSize: '0.8rem',
              marginBottom: '0.25rem',
            }}>
              <span>Stats</span>
              <span style={{ color: 'var(--text-dim)' }}>{summary.total_headers_edited}</span>
            </div>
            <div style={{
              height: '8px',
              backgroundColor: 'var(--glass)',
              borderRadius: '4px',
              overflow: 'hidden',
            }}>
              <div style={{
                width: getBarWidth(summary.total_headers_edited),
                height: '100%',
                backgroundColor: getBarColor('headers'),
                borderRadius: '4px',
                transition: 'width 0.3s ease',
              }} />
            </div>
          </div>
        )}
      </div>
    </div>
  );
};

export default ChangeStats;
