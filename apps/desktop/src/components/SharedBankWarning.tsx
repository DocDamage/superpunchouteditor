import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

export interface SharedBankInfo {
  filename: string;
  start_pc: string;
  shared_with: string[];
  category: string;
  size: number;
}

export interface SharedBankPair {
  fighters: string[];
  note: string;
}

interface SharedBankWarningProps {
  isOpen: boolean;
  onClose: () => void;
  onConfirm: (duplicate: boolean) => void;
  bankInfo: SharedBankInfo | null;
  currentBoxer: string;
}

export const SharedBankWarning = ({
  isOpen,
  onClose,
  onConfirm,
  bankInfo,
  currentBoxer,
}: SharedBankWarningProps) => {
  const [sharedPairs, setSharedPairs] = useState<SharedBankPair[]>([]);
  const [loading, setLoading] = useState(false);
  const [showDuplicateOption, setShowDuplicateOption] = useState(false);

  useEffect(() => {
    if (isOpen) {
      loadSharedPairs();
    }
  }, [isOpen]);

  const loadSharedPairs = async () => {
    try {
      const layouts = await invoke<{ shared_pairs?: SharedBankPair[] }>('get_all_layouts');
      setSharedPairs(layouts.shared_pairs || []);
    } catch (e) {
      console.error('Failed to load shared pairs:', e);
    }
  };

  if (!isOpen || !bankInfo) return null;
  const sharedWith = Array.isArray(bankInfo.shared_with) ? bankInfo.shared_with : [];

  // Get the pair info for this bank
  const pairInfo = sharedPairs.find(pair =>
    pair.fighters.some(f =>
      sharedWith.some(sf => sf.toLowerCase() === f.toLowerCase())
    )
  );

  const otherFighters = sharedWith.filter(
    f => f.toLowerCase() !== currentBoxer.toLowerCase()
  );

  return (
    <div
      style={{
        position: 'fixed',
        inset: 0,
        backgroundColor: 'rgba(0, 0, 0, 0.7)',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        zIndex: 1000,
        backdropFilter: 'blur(4px)',
      }}
      onClick={onClose}
    >
      <div
        style={{
          backgroundColor: 'var(--panel-bg, #1a1b26)',
          border: '1px solid rgba(255, 80, 80, 0.4)',
          borderRadius: '12px',
          padding: '1.5rem',
          maxWidth: '500px',
          width: '90%',
          boxShadow: '0 20px 60px rgba(0, 0, 0, 0.5), 0 0 0 1px rgba(255, 80, 80, 0.1)',
          animation: 'modalSlideIn 0.2s ease-out',
        }}
        onClick={e => e.stopPropagation()}
      >
        {/* Header */}
        <div style={{ display: 'flex', alignItems: 'center', gap: '12px', marginBottom: '1rem' }}>
          <div
            style={{
              width: '40px',
              height: '40px',
              borderRadius: '50%',
              backgroundColor: 'rgba(255, 80, 80, 0.15)',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              fontSize: '1.5rem',
            }}
          >
            ⚠️
          </div>
          <div>
            <h3 style={{ margin: 0, fontSize: '1.1rem', color: '#ff8888' }}>
              Shared Bank Warning
            </h3>
            <p style={{ margin: '4px 0 0', fontSize: '0.8rem', color: 'var(--text-dim)' }}>
              This action affects multiple fighters
            </p>
          </div>
        </div>

        {/* Content */}
        <div style={{ marginBottom: '1.5rem' }}>
          <div
            style={{
              backgroundColor: 'rgba(255, 80, 80, 0.08)',
              border: '1px solid rgba(255, 80, 80, 0.2)',
              borderRadius: '8px',
              padding: '12px',
              marginBottom: '1rem',
            }}
          >
            <div style={{ fontWeight: 600, marginBottom: '4px', color: '#ffaaaa' }}>
              {bankInfo.filename}
            </div>
            <div style={{ fontSize: '0.8rem', color: 'var(--text-dim)' }}>
              {bankInfo.size} bytes @ {bankInfo.start_pc}
            </div>
          </div>

          <p style={{ margin: '0 0 12px', lineHeight: 1.5, fontSize: '0.9rem' }}>
            This sprite bank is <strong>shared</strong> with the following fighter{otherFighters.length > 1 ? 's' : ''}:
          </p>

          <div
            style={{
              display: 'flex',
              flexWrap: 'wrap',
              gap: '8px',
              marginBottom: '1rem',
            }}
          >
            {otherFighters.map((fighter, idx) => (
              <span
                key={idx}
                style={{
                  backgroundColor: 'rgba(255, 100, 100, 0.15)',
                  border: '1px solid rgba(255, 100, 100, 0.3)',
                  borderRadius: '6px',
                  padding: '6px 12px',
                  fontSize: '0.85rem',
                  fontWeight: 600,
                  color: '#ff9999',
                }}
              >
                {fighter}
              </span>
            ))}
          </div>

          {pairInfo?.note && (
            <div
              style={{
                backgroundColor: 'rgba(255, 200, 100, 0.08)',
                border: '1px solid rgba(255, 200, 100, 0.2)',
                borderRadius: '6px',
                padding: '8px 12px',
                fontSize: '0.8rem',
                color: '#ffcc88',
                marginBottom: '1rem',
              }}
            >
              <strong>Note:</strong> {pairInfo.note}
            </div>
          )}

          <div
            style={{
              backgroundColor: 'var(--glass)',
              borderRadius: '6px',
              padding: '12px',
              fontSize: '0.85rem',
            }}
          >
            <p style={{ margin: '0 0 8px', color: 'var(--text-dim)' }}>
              <strong>Implications:</strong>
            </p>
            <ul
              style={{
                margin: 0,
                paddingLeft: '1.2rem',
                color: 'var(--text-dim)',
                lineHeight: 1.6,
              }}
            >
              <li>Any changes you make will affect <strong>all</strong> fighters using this bank</li>
              <li>The original shared bank data will be overwritten</li>
              <li>This cannot be undone without reloading the ROM</li>
            </ul>
          </div>
        </div>

        {/* Duplicate option toggle */}
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: '8px',
            marginBottom: '1rem',
            padding: '8px',
            borderRadius: '6px',
            backgroundColor: 'rgba(100, 150, 255, 0.08)',
            cursor: 'pointer',
          }}
          onClick={() => setShowDuplicateOption(!showDuplicateOption)}
        >
          <input
            type="checkbox"
            checked={showDuplicateOption}
            onChange={e => setShowDuplicateOption(e.target.checked)}
            style={{ cursor: 'pointer' }}
          />
          <span style={{ fontSize: '0.85rem', color: '#88aaff' }}>
            I want to duplicate this bank first (create a unique copy)
          </span>
        </div>

        {showDuplicateOption && (
          <div
            style={{
              backgroundColor: 'rgba(100, 150, 255, 0.1)',
              border: '1px solid rgba(100, 150, 255, 0.3)',
              borderRadius: '6px',
              padding: '12px',
              marginBottom: '1rem',
              fontSize: '0.8rem',
              color: '#88aaff',
            }}
          >
            <strong>💡 Duplicate feature:</strong> This will:
            <ul style={{ margin: '8px 0', paddingLeft: '1.2rem' }}>
              <li>Create a copy of the graphics data in free ROM space</li>
              <li>Make {currentBoxer} use the new unique bank</li>
              <li>Preserve the original shared bank for {otherFighters.join(' and ')}</li>
            </ul>
            <em>⚠️ Advanced: The ROM may be expanded to 2.5MB or 4MB if needed.</em>
          </div>
        )}

        {/* Actions */}
        <div
          style={{
            display: 'flex',
            gap: '10px',
            justifyContent: 'flex-end',
          }}
        >
          <button
            onClick={onClose}
            style={{
              padding: '8px 16px',
              fontSize: '0.9rem',
              backgroundColor: 'transparent',
              border: '1px solid var(--border)',
              borderRadius: '6px',
              cursor: 'pointer',
            }}
          >
            Cancel
          </button>
          <button
            onClick={() => onConfirm(showDuplicateOption)}
            style={{
              padding: '8px 16px',
              fontSize: '0.9rem',
              backgroundColor: showDuplicateOption
                ? 'rgba(100, 150, 255, 0.3)'
                : 'rgba(255, 80, 80, 0.3)',
              border: `1px solid ${showDuplicateOption ? 'rgba(100, 150, 255, 0.5)' : 'rgba(255, 80, 80, 0.5)'}`,
              borderRadius: '6px',
              cursor: 'pointer',
              color: showDuplicateOption ? '#88aaff' : '#ff8888',
            }}
          >
            {showDuplicateOption ? 'Duplicate & Edit' : 'Edit Shared Bank'}
          </button>
        </div>
      </div>

      <style>{`
        @keyframes modalSlideIn {
          from {
            opacity: 0;
            transform: translateY(-20px) scale(0.95);
          }
          to {
            opacity: 1;
            transform: translateY(0) scale(1);
          }
        }
      `}</style>
    </div>
  );
};

export default SharedBankWarning;
