import { useState, useCallback, useEffect } from 'react';
import { useStore, ProjectThumbnail } from '../store/useStore';

export interface ProjectThumbnailDisplayProps {
  thumbnail: ProjectThumbnail | null;
  size?: 'small' | 'medium' | 'large';
  showMeta?: boolean;
}

export function ProjectThumbnailDisplay({ 
  thumbnail, 
  size = 'medium',
  showMeta = true 
}: ProjectThumbnailDisplayProps) {
  const [error, setError] = useState(false);

  const sizeStyles = {
    small: { width: 80, height: 60 },
    medium: { width: 160, height: 120 },
    large: { width: 320, height: 240 },
  };

  const dimensions = sizeStyles[size];

  if (!thumbnail || error) {
    return (
      <div
        style={{
          ...dimensions,
          backgroundColor: 'var(--glass)',
          borderRadius: '8px',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          border: '2px dashed var(--border)',
          color: 'var(--text-dim)',
          fontSize: size === 'small' ? '0.7rem' : '0.85rem',
        }}
      >
        {size !== 'small' && 'No Thumbnail'}
      </div>
    );
  }

  const dataUrl = `data:image/png;base64,${thumbnail.data_base64}`;
  const captureDate = thumbnail.captured_at 
    ? new Date(thumbnail.captured_at).toLocaleDateString()
    : null;

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem' }}>
      <div
        style={{
          ...dimensions,
          borderRadius: '8px',
          overflow: 'hidden',
          border: '2px solid var(--border)',
          backgroundColor: 'var(--panel-bg)',
        }}
      >
        <img
          src={dataUrl}
          alt="Project thumbnail"
          style={{
            width: '100%',
            height: '100%',
            objectFit: 'cover',
          }}
          onError={() => setError(true)}
        />
      </div>
      {showMeta && captureDate && (
        <div
          style={{
            fontSize: '0.75rem',
            color: 'var(--text-dim)',
            textAlign: 'center',
          }}
        >
          {captureDate}
          {thumbnail.captured_view && (
            <span style={{ textTransform: 'capitalize', marginLeft: '0.25rem' }}>
              • {thumbnail.captured_view}
            </span>
          )}
        </div>
      )}
    </div>
  );
}

export interface ThumbnailCaptureButtonProps {
  viewType?: string;
  onCapture?: (thumbnail: ProjectThumbnail) => void;
}

export function ThumbnailCaptureButton({ 
  viewType = 'editor',
  onCapture 
}: ThumbnailCaptureButtonProps) {
  const { captureThumbnail, saveThumbnail, currentProject } = useStore();
  const [isCapturing, setIsCapturing] = useState(false);
  const [showPreview, setShowPreview] = useState(false);
  const [capturedThumbnail, setCapturedThumbnail] = useState<ProjectThumbnail | null>(null);

  const handleCapture = useCallback(async () => {
    setIsCapturing(true);
    try {
      const thumbnail = await captureThumbnail(viewType);
      if (thumbnail) {
        setCapturedThumbnail(thumbnail);
        setShowPreview(true);
        onCapture?.(thumbnail);
      }
    } catch (e) {
      console.error('Failed to capture thumbnail:', e);
    } finally {
      setIsCapturing(false);
    }
  }, [captureThumbnail, viewType, onCapture]);

  const handleSave = useCallback(async () => {
    if (!capturedThumbnail) return;
    try {
      await saveThumbnail(capturedThumbnail);
      setShowPreview(false);
      setCapturedThumbnail(null);
    } catch (e) {
      console.error('Failed to save thumbnail:', e);
    }
  }, [capturedThumbnail, saveThumbnail]);

  const handleCancel = useCallback(() => {
    setShowPreview(false);
    setCapturedThumbnail(null);
  }, []);

  if (!currentProject) {
    return null;
  }

  return (
    <>
      <button
        onClick={handleCapture}
        disabled={isCapturing}
        style={{
          padding: '0.5rem 1rem',
          borderRadius: '6px',
          border: '1px solid var(--border)',
          backgroundColor: 'var(--glass)',
          color: 'var(--text)',
          cursor: isCapturing ? 'wait' : 'pointer',
          fontSize: '0.85rem',
          display: 'flex',
          alignItems: 'center',
          gap: '0.5rem',
        }}
      >
        {isCapturing ? (
          <>
            <span style={{ 
              display: 'inline-block',
              width: '14px',
              height: '14px',
              border: '2px solid var(--text-dim)',
              borderTopColor: 'transparent',
              borderRadius: '50%',
              animation: 'spin 1s linear infinite',
            }} />
            Capturing...
          </>
        ) : (
          <>
            📷 Capture Thumbnail
          </>
        )}
      </button>

      {showPreview && capturedThumbnail && (
        <ThumbnailPreviewDialog
          thumbnail={capturedThumbnail}
          onSave={handleSave}
          onCancel={handleCancel}
        />
      )}
    </>
  );
}

interface ThumbnailPreviewDialogProps {
  thumbnail: ProjectThumbnail;
  onSave: () => void;
  onCancel: () => void;
}

function ThumbnailPreviewDialog({ thumbnail, onSave, onCancel }: ThumbnailPreviewDialogProps) {
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
    >
      <div
        style={{
          backgroundColor: 'var(--panel-bg)',
          borderRadius: '12px',
          border: '1px solid var(--border)',
          padding: '1.5rem',
          maxWidth: '480px',
          width: '90%',
        }}
      >
        <h3 style={{ marginTop: 0, marginBottom: '1rem' }}>
          Save Thumbnail Preview
        </h3>
        
        <div style={{ 
          display: 'flex', 
          justifyContent: 'center',
          marginBottom: '1.5rem' 
        }}>
          <ProjectThumbnailDisplay 
            thumbnail={thumbnail} 
            size="large" 
            showMeta={false}
          />
        </div>

        <div
          style={{
            backgroundColor: 'var(--glass)',
            borderRadius: '6px',
            padding: '0.75rem',
            marginBottom: '1.5rem',
            fontSize: '0.85rem',
            color: 'var(--text-dim)',
          }}
        >
          <div>Size: {thumbnail.width} × {thumbnail.height}</div>
          <div>View: <span style={{ textTransform: 'capitalize' }}>{thumbnail.captured_view}</span></div>
          <div>Captured: {new Date(thumbnail.captured_at).toLocaleString()}</div>
        </div>

        <div style={{ display: 'flex', gap: '0.75rem', justifyContent: 'flex-end' }}>
          <button
            onClick={onCancel}
            style={{
              padding: '0.75rem 1.5rem',
              borderRadius: '6px',
              border: '1px solid var(--border)',
              backgroundColor: 'transparent',
              color: 'var(--text)',
              cursor: 'pointer',
              fontSize: '0.95rem',
            }}
          >
            Cancel
          </button>
          <button
            onClick={onSave}
            style={{
              padding: '0.75rem 1.5rem',
              borderRadius: '6px',
              border: 'none',
              backgroundColor: 'var(--blue)',
              color: 'white',
              cursor: 'pointer',
              fontSize: '0.95rem',
              fontWeight: 500,
            }}
          >
            Save Thumbnail
          </button>
        </div>
      </div>
    </div>
  );
}

export interface ThumbnailManagerProps {
  className?: string;
  style?: React.CSSProperties;
}

export function ThumbnailManager({ className, style }: ThumbnailManagerProps) {
  const { currentProject, getThumbnail, clearThumbnail } = useStore();
  const [thumbnail, setThumbnail] = useState<ProjectThumbnail | null>(
    currentProject?.thumbnail || null
  );
  const [isLoading, setIsLoading] = useState(false);

  // Refresh thumbnail when project changes
  useEffect(() => {
    setThumbnail(currentProject?.thumbnail || null);
  }, [currentProject]);

  const handleRefresh = useCallback(async () => {
    setIsLoading(true);
    try {
      const result = await getThumbnail();
      setThumbnail(result);
    } finally {
      setIsLoading(false);
    }
  }, [getThumbnail]);

  const handleClear = useCallback(async () => {
    if (confirm('Are you sure you want to remove the project thumbnail?')) {
      try {
        await clearThumbnail();
        setThumbnail(null);
      } catch (e) {
        console.error('Failed to clear thumbnail:', e);
      }
    }
  }, [clearThumbnail]);

  if (!currentProject) {
    return null;
  }

  return (
    <div
      className={className}
      style={{
        backgroundColor: 'var(--glass)',
        borderRadius: '8px',
        padding: '1rem',
        border: '1px solid var(--border)',
        ...style,
      }}
    >
      <div
        style={{
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
          marginBottom: '0.75rem',
        }}
      >
        <h4 style={{ margin: 0, fontSize: '0.9rem' }}>Project Thumbnail</h4>
        {thumbnail && (
          <button
            onClick={handleClear}
            style={{
              padding: '0.25rem 0.5rem',
              borderRadius: '4px',
              border: 'none',
              backgroundColor: 'transparent',
              color: 'var(--accent)',
              cursor: 'pointer',
              fontSize: '0.75rem',
            }}
          >
            Remove
          </button>
        )}
      </div>

      <div style={{ display: 'flex', gap: '1rem', alignItems: 'flex-start' }}>
        <ProjectThumbnailDisplay 
          thumbnail={thumbnail} 
          size="medium" 
        />
        <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem' }}>
          <ThumbnailCaptureButton />
          {thumbnail && (
            <button
              onClick={handleRefresh}
              disabled={isLoading}
              style={{
                padding: '0.5rem 1rem',
                borderRadius: '6px',
                border: '1px solid var(--border)',
                backgroundColor: 'transparent',
                color: 'var(--text)',
                cursor: isLoading ? 'wait' : 'pointer',
                fontSize: '0.8rem',
              }}
            >
              {isLoading ? 'Refreshing...' : 'Refresh'}
            </button>
          )}
        </div>
      </div>
    </div>
  );
}

export default ProjectThumbnailDisplay;
