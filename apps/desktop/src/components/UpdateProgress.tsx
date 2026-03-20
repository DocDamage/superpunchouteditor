import { DownloadProgress } from '../store/useStore';

interface UpdateProgressProps {
  progress: DownloadProgress;
  onCancel?: () => void;
}

export function UpdateProgress({ progress, onCancel }: UpdateProgressProps) {
  const getStatusMessage = () => {
    switch (progress.state) {
      case 'checking':
        return 'Checking for updates...';
      case 'downloading':
        return `Downloading update... ${progress.percent}%`;
      case 'verifying':
        return 'Verifying update...';
      case 'installing':
        return 'Installing update...';
      case 'ready':
        return 'Update ready! Restarting...';
      case 'error':
        return 'Update failed. Please try again.';
      default:
        return 'Preparing...';
    }
  };

  const getStatusIcon = () => {
    switch (progress.state) {
      case 'checking':
        return '🔍';
      case 'downloading':
        return '⬇️';
      case 'verifying':
        return '🔐';
      case 'installing':
        return '⚙️';
      case 'ready':
        return '✅';
      case 'error':
        return '❌';
      default:
        return '⏳';
    }
  };

  const formatBytes = (bytes: number): string => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  return (
    <div
      style={{
        position: 'fixed',
        top: 0,
        left: 0,
        right: 0,
        bottom: 0,
        backgroundColor: 'rgba(0, 0, 0, 0.8)',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        zIndex: 1001,
      }}
    >
      <div
        style={{
          backgroundColor: 'var(--panel-bg)',
          borderRadius: '12px',
          border: '1px solid var(--border)',
          maxWidth: '400px',
          width: '90%',
          padding: '2rem',
          textAlign: 'center',
          boxShadow: '0 20px 60px rgba(0, 0, 0, 0.5)',
        }}
      >
        {/* Status Icon */}
        <div
          style={{
            fontSize: '3rem',
            marginBottom: '1rem',
            animation: progress.state === 'downloading' || progress.state === 'installing' 
              ? 'pulse 1.5s ease-in-out infinite' 
              : undefined,
          }}
        >
          {getStatusIcon()}
        </div>

        {/* Status Message */}
        <h3
          style={{
            margin: '0 0 1.5rem 0',
            color: 'var(--text)',
            fontSize: '1.125rem',
            fontWeight: 600,
          }}
        >
          {getStatusMessage()}
        </h3>

        {/* Progress Bar */}
        <div
          style={{
            width: '100%',
            height: '8px',
            backgroundColor: 'var(--glass)',
            borderRadius: '4px',
            overflow: 'hidden',
            marginBottom: '0.75rem',
          }}
        >
          <div
            style={{
              width: `${progress.percent}%`,
              height: '100%',
              background: 'linear-gradient(90deg, var(--blue), #4ade80)',
              borderRadius: '4px',
              transition: 'width 0.3s ease',
              boxShadow: progress.state === 'downloading' 
                ? '0 0 10px rgba(59, 130, 246, 0.5)' 
                : undefined,
            }}
          />
        </div>

        {/* Progress Details */}
        <div
          style={{
            display: 'flex',
            justifyContent: 'space-between',
            fontSize: '0.75rem',
            color: 'var(--text-dim)',
            marginBottom: '1.5rem',
          }}
        >
          <span>{progress.percent}%</span>
          {progress.total > 0 && (
            <span>
              {formatBytes(progress.downloaded)} / {formatBytes(progress.total)}
            </span>
          )}
        </div>

        {/* Cancel Button (only show during download) */}
        {progress.state === 'downloading' && onCancel && (
          <button
            onClick={onCancel}
            style={{
              padding: '0.5rem 1rem',
              backgroundColor: 'var(--glass)',
              border: '1px solid var(--border)',
              borderRadius: '6px',
              color: 'var(--text)',
              cursor: 'pointer',
              fontSize: '0.875rem',
            }}
          >
            Cancel
          </button>
        )}

        {/* Error State */}
        {progress.state === 'error' && (
          <div
            style={{
              padding: '0.75rem',
              backgroundColor: 'rgba(239, 68, 68, 0.1)',
              border: '1px solid rgba(239, 68, 68, 0.3)',
              borderRadius: '6px',
              color: '#ef4444',
              fontSize: '0.875rem',
            }}
          >
            Update failed. Please check your connection and try again.
          </div>
        )}

        {/* Ready State */}
        {progress.state === 'ready' && (
          <div
            style={{
              padding: '0.75rem',
              backgroundColor: 'rgba(74, 222, 128, 0.1)',
              border: '1px solid rgba(74, 222, 128, 0.3)',
              borderRadius: '6px',
              color: '#4ade80',
              fontSize: '0.875rem',
            }}
          >
            Update downloaded! The app will restart automatically.
          </div>
        )}
      </div>

      {/* Keyframe Animation */}
      <style>{`
        @keyframes pulse {
          0%, 100% { opacity: 1; transform: scale(1); }
          50% { opacity: 0.7; transform: scale(0.95); }
        }
      `}</style>
    </div>
  );
}
