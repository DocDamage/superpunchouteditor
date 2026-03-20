import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

export interface SharedBankIndicatorProps {
  sharedWith: string[];
  currentBoxer?: string;
  showDetails?: boolean;
  size?: 'small' | 'medium' | 'large';
}

interface SharedBankPair {
  fighters: string[];
  note: string;
}

export interface SharedBankSummaryProps {
  uniqueCount: number;
  sharedCount: number;
  sharedBins: Array<{ shared_with: string[]; filename?: string }>;
  currentBoxer?: string;
}

export const SharedBankIndicator = ({
  sharedWith,
  currentBoxer,
  showDetails = true,
  size = 'medium',
}: SharedBankIndicatorProps) => {
  const [sharedPairs, setSharedPairs] = useState<SharedBankPair[]>([]);
  const [showTooltip, setShowTooltip] = useState(false);

  useEffect(() => {
    loadSharedPairs();
  }, []);

  const loadSharedPairs = async () => {
    try {
      const layouts = await invoke<{ shared_pairs?: SharedBankPair[] }>('get_all_layouts');
      setSharedPairs(layouts.shared_pairs || []);
    } catch (e) {
      console.error('Failed to load shared pairs:', e);
    }
  };

  if (!sharedWith || sharedWith.length === 0) return null;

  const otherFighters = currentBoxer
    ? sharedWith.filter(f => f.toLowerCase() !== currentBoxer.toLowerCase())
    : sharedWith;

  if (otherFighters.length === 0) return null;

  // Get pair info
  const pairInfo = sharedPairs.find(pair =>
    pair.fighters.some(f => otherFighters.some(of => of.toLowerCase() === f.toLowerCase()))
  );

  const sizeStyles = {
    small: { padding: '1px 4px', fontSize: '0.65rem', icon: '⚠' },
    medium: { padding: '2px 6px', fontSize: '0.72rem', icon: '⚠' },
    large: { padding: '4px 8px', fontSize: '0.8rem', icon: '⚠️' },
  };

  const style = sizeStyles[size];

  return (
    <span
      style={{
        display: 'inline-flex',
        alignItems: 'center',
        gap: '4px',
        padding: style.padding,
        borderRadius: '4px',
        backgroundColor: 'rgba(255, 80, 80, 0.15)',
        border: '1px solid rgba(255, 80, 80, 0.3)',
        color: '#ff8888',
        fontSize: style.fontSize,
        fontWeight: 600,
        whiteSpace: 'nowrap',
        cursor: showDetails ? 'help' : 'default',
        position: 'relative',
      }}
      onMouseEnter={() => showDetails && setShowTooltip(true)}
      onMouseLeave={() => setShowTooltip(false)}
    >
      <span>{style.icon}</span>
      <span>SHARED</span>

      {showTooltip && showDetails && (
        <div
          style={{
            position: 'absolute',
            bottom: '100%',
            left: '50%',
            transform: 'translateX(-50%)',
            marginBottom: '8px',
            backgroundColor: 'rgba(30, 32, 48, 0.98)',
            border: '1px solid rgba(255, 80, 80, 0.4)',
            borderRadius: '8px',
            padding: '12px',
            minWidth: '220px',
            maxWidth: '280px',
            zIndex: 100,
            boxShadow: '0 8px 32px rgba(0, 0, 0, 0.4)',
            fontSize: '0.8rem',
            fontWeight: 'normal',
          }}
        >
          <div style={{ fontWeight: 600, color: '#ffaaaa', marginBottom: '6px' }}>
            Shared with:
          </div>
          <div style={{ display: 'flex', flexWrap: 'wrap', gap: '4px', marginBottom: '8px' }}>
            {otherFighters.map((fighter, idx) => (
              <span
                key={idx}
                style={{
                  backgroundColor: 'rgba(255, 80, 80, 0.2)',
                  padding: '2px 6px',
                  borderRadius: '4px',
                  fontSize: '0.75rem',
                }}
              >
                {fighter}
              </span>
            ))}
          </div>
          {pairInfo?.note && (
            <div style={{ color: '#ffcc88', fontSize: '0.75rem', fontStyle: 'italic' }}>
              {pairInfo.note}
            </div>
          )}
          <div
            style={{
              color: 'var(--text-dim)',
              fontSize: '0.7rem',
              marginTop: '6px',
              borderTop: '1px solid rgba(255, 80, 80, 0.2)',
              paddingTop: '6px',
            }}
          >
            ⚠️ Edits will affect all fighters using this bank
          </div>

          {/* Tooltip arrow */}
          <div
            style={{
              position: 'absolute',
              top: '100%',
              left: '50%',
              transform: 'translateX(-50%)',
              width: 0,
              height: 0,
              borderLeft: '6px solid transparent',
              borderRight: '6px solid transparent',
              borderTop: '6px solid rgba(255, 80, 80, 0.4)',
            }}
          />
        </div>
      )}
    </span>
  );
};

export const SharedBankSummary = ({
  uniqueCount,
  sharedCount,
  sharedBins,
  currentBoxer,
}: SharedBankSummaryProps) => {
  const [expanded, setExpanded] = useState(false);

  // Get all unique fighters that share banks with this boxer
  const allSharedFighters = new Set<string>();
  sharedBins.forEach(bin => {
    bin.shared_with.forEach(f => {
      if (!currentBoxer || f.toLowerCase() !== currentBoxer.toLowerCase()) {
        allSharedFighters.add(f);
      }
    });
  });

  const sharedFighterList = Array.from(allSharedFighters);

  if (sharedCount === 0) {
    return (
      <div
        style={{
          padding: '10px 14px',
          borderRadius: '8px',
          backgroundColor: 'rgba(107, 219, 125, 0.08)',
          border: '1px solid rgba(107, 219, 125, 0.25)',
          fontSize: '0.82rem',
          color: '#6bdb7d',
          display: 'flex',
          alignItems: 'center',
          gap: '8px',
        }}
      >
        <span>✓</span>
        <span>All {uniqueCount} bins are unique — safe to edit</span>
      </div>
    );
  }

  return (
    <div
      style={{
        padding: '12px 14px',
        borderRadius: '8px',
        backgroundColor: 'rgba(255, 80, 80, 0.08)',
        border: '1px solid rgba(255, 80, 80, 0.25)',
        fontSize: '0.82rem',
      }}
    >
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          cursor: 'pointer',
        }}
        onClick={() => setExpanded(!expanded)}
      >
        <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
          <span style={{ color: '#ff6666' }}>⚠</span>
          <span style={{ color: '#ff8888' }}>
            {sharedCount} shared bin{sharedCount !== 1 ? 's' : ''} affect{sharedCount === 1 ? 's' : ''}{' '}
            {sharedFighterList.length} other fighter{sharedFighterList.length !== 1 ? 's' : ''}
          </span>
        </div>
        <span style={{ color: 'var(--text-dim)', fontSize: '0.75rem' }}>
          {expanded ? '▼' : '▶'}
        </span>
      </div>

      {expanded && (
        <div style={{ marginTop: '10px', paddingTop: '10px', borderTop: '1px solid rgba(255, 80, 80, 0.15)' }}>
          <div style={{ marginBottom: '8px', color: 'var(--text-dim)', fontSize: '0.75rem' }}>
            Shared with:
          </div>
          <div style={{ display: 'flex', flexWrap: 'wrap', gap: '6px', marginBottom: '12px' }}>
            {sharedFighterList.map((fighter, idx) => (
              <span
                key={idx}
                style={{
                  backgroundColor: 'rgba(255, 80, 80, 0.15)',
                  border: '1px solid rgba(255, 80, 80, 0.25)',
                  padding: '4px 10px',
                  borderRadius: '6px',
                  fontSize: '0.8rem',
                  color: '#ff9999',
                }}
              >
                {fighter}
              </span>
            ))}
          </div>

          <div style={{ color: 'var(--text-dim)', fontSize: '0.75rem', lineHeight: 1.5 }}>
            <strong>⚠️ Warning:</strong> Editing any shared bin will change the appearance
            of all fighters listed above. Consider creating unique bins if you want
            to customize only {currentBoxer || 'this fighter'}.
          </div>
        </div>
      )}
    </div>
  );
};

export default SharedBankIndicator;
