import { useState } from 'react';
import { open } from '@tauri-apps/plugin-opener';
import { UpdateInfo } from '../store/useStore';

interface UpdateAvailableModalProps {
  update: UpdateInfo;
  currentVersion: string;
  onClose: () => void;
  onDownload: () => void;
  onSkip: (version: string) => void;
  manualDownloadUrl: string;
}

export function UpdateAvailableModal({
  update,
  currentVersion,
  onClose,
  onDownload,
  onSkip,
  manualDownloadUrl,
}: UpdateAvailableModalProps) {
  const [showFullNotes, setShowFullNotes] = useState(false);

  const handleOpenChangelog = async () => {
    try {
      await open(manualDownloadUrl);
    } catch (e) {
      console.error('Failed to open changelog:', e);
    }
  };

  // Parse release notes into bullet points if possible
  const parseReleaseNotes = (notes: string): string[] => {
    if (!notes) return ['No release notes available.'];
    
    // Try to split by common bullet patterns
    const lines = notes.split(/\n|\r\n/);
    const bullets: string[] = [];
    
    for (const line of lines) {
      const trimmed = line.trim();
      if (trimmed.startsWith('- ') || trimmed.startsWith('• ') || trimmed.startsWith('* ')) {
        bullets.push(trimmed.substring(2));
      } else if (trimmed.match(/^\d+\.\s/)) {
        bullets.push(trimmed.replace(/^\d+\.\s/, ''));
      } else if (trimmed && !trimmed.startsWith('#') && !trimmed.startsWith('==')) {
        // Include non-empty lines that aren't headers
        bullets.push(trimmed);
      }
    }
    
    return bullets.length > 0 ? bullets.slice(0, 5) : ['See full release notes for details.'];
  };

  const bulletPoints = parseReleaseNotes(update.notes);
  const hasMoreNotes = update.notes && update.notes.length > 200;

  return (
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
      onClick={onClose}
    >
      <div
        style={{
          backgroundColor: 'var(--panel-bg)',
          borderRadius: '12px',
          border: '1px solid var(--border)',
          maxWidth: '480px',
          width: '90%',
          maxHeight: '80vh',
          overflow: 'auto',
          boxShadow: '0 20px 60px rgba(0, 0, 0, 0.5)',
        }}
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div
          style={{
            background: 'linear-gradient(135deg, var(--blue) 0%, #4ade80 100%)',
            padding: '1.5rem',
            borderRadius: '12px 12px 0 0',
            textAlign: 'center',
          }}
        >
          <div style={{ fontSize: '2.5rem', marginBottom: '0.5rem' }}>🎉</div>
          <h2
            style={{
              margin: 0,
              color: 'white',
              fontSize: '1.5rem',
              fontWeight: 600,
            }}
          >
            Update Available!
          </h2>
        </div>

        {/* Content */}
        <div style={{ padding: '1.5rem' }}>
          {/* Version Info */}
          <div
            style={{
              textAlign: 'center',
              marginBottom: '1.5rem',
              padding: '1rem',
              backgroundColor: 'var(--glass)',
              borderRadius: '8px',
            }}
          >
            <div style={{ color: 'var(--text-dim)', fontSize: '0.875rem', marginBottom: '0.25rem' }}>
              Super Punch-Out!! Editor
            </div>
            <div style={{ fontSize: '1.25rem', fontWeight: 600, color: 'var(--text)' }}>
              {update.version}
            </div>
            <div style={{ fontSize: '0.75rem', color: 'var(--text-dim)', marginTop: '0.25rem' }}>
              Current: v{currentVersion}
            </div>
          </div>

          {/* Release Notes */}
          <div style={{ marginBottom: '1.5rem' }}>
            <h3
              style={{
                margin: '0 0 0.75rem 0',
                fontSize: '0.875rem',
                color: 'var(--text-dim)',
                textTransform: 'uppercase',
                letterSpacing: '0.05em',
              }}
            >
              What's New
            </h3>
            <ul
              style={{
                margin: 0,
                paddingLeft: '1.25rem',
                color: 'var(--text)',
                lineHeight: 1.6,
              }}
            >
              {bulletPoints.map((point, index) => (
                <li key={index} style={{ marginBottom: '0.375rem' }}>
                  {point}
                </li>
              ))}
            </ul>
            
            {hasMoreNotes && !showFullNotes && (
              <button
                onClick={() => setShowFullNotes(true)}
                style={{
                  background: 'none',
                  border: 'none',
                  color: 'var(--blue)',
                  cursor: 'pointer',
                  fontSize: '0.875rem',
                  marginTop: '0.5rem',
                  padding: 0,
                }}
              >
                + Show more
              </button>
            )}
            
            {showFullNotes && (
              <div
                style={{
                  marginTop: '1rem',
                  padding: '1rem',
                  backgroundColor: 'var(--glass)',
                  borderRadius: '6px',
                  fontSize: '0.875rem',
                  whiteSpace: 'pre-wrap',
                  maxHeight: '200px',
                  overflow: 'auto',
                }}
              >
                {update.notes}
              </div>
            )}
          </div>

          {/* View Full Changelog Link */}
          <button
            onClick={handleOpenChangelog}
            style={{
              width: '100%',
              padding: '0.625rem',
              backgroundColor: 'transparent',
              border: '1px solid var(--border)',
              borderRadius: '6px',
              color: 'var(--text-dim)',
              cursor: 'pointer',
              fontSize: '0.875rem',
              marginBottom: '1rem',
              transition: 'all 0.2s',
            }}
            onMouseEnter={(e) => {
              e.currentTarget.style.borderColor = 'var(--blue)';
              e.currentTarget.style.color = 'var(--blue)';
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.borderColor = 'var(--border)';
              e.currentTarget.style.color = 'var(--text-dim)';
            }}
          >
            📋 View Full Changelog
          </button>

          {/* Action Buttons */}
          <div style={{ display: 'flex', flexDirection: 'column', gap: '0.75rem' }}>
            <button
              onClick={onDownload}
              style={{
                width: '100%',
                padding: '0.875rem',
                background: 'linear-gradient(135deg, var(--blue) 0%, #3b82f6 100%)',
                border: 'none',
                borderRadius: '8px',
                color: 'white',
                fontSize: '1rem',
                fontWeight: 600,
                cursor: 'pointer',
                transition: 'transform 0.2s, box-shadow 0.2s',
              }}
              onMouseEnter={(e) => {
                e.currentTarget.style.transform = 'translateY(-1px)';
                e.currentTarget.style.boxShadow = '0 4px 12px rgba(59, 130, 246, 0.4)';
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.transform = 'translateY(0)';
                e.currentTarget.style.boxShadow = 'none';
              }}
            >
              ⬇️ Download & Install
            </button>

            <div style={{ display: 'flex', gap: '0.5rem' }}>
              <button
                onClick={onClose}
                style={{
                  flex: 1,
                  padding: '0.625rem',
                  backgroundColor: 'var(--glass)',
                  border: '1px solid var(--border)',
                  borderRadius: '6px',
                  color: 'var(--text)',
                  cursor: 'pointer',
                  fontSize: '0.875rem',
                }}
              >
                ⏰ Remind Me Later
              </button>

              <button
                onClick={() => onSkip(update.version)}
                style={{
                  flex: 1,
                  padding: '0.625rem',
                  backgroundColor: 'var(--glass)',
                  border: '1px solid var(--border)',
                  borderRadius: '6px',
                  color: 'var(--text-dim)',
                  cursor: 'pointer',
                  fontSize: '0.875rem',
                }}
              >
                🚫 Skip This Version
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
