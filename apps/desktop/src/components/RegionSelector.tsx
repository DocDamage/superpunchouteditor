/**
 * ROM Region Selector Component
 * 
 * Provides UI for detecting and selecting ROM regions (USA, JPN, PAL).
 * Shows region-specific information and support status.
 */

import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';

export interface RomRegionInfo {
  region: 'Usa' | 'Jpn' | 'Pal';
  display_name: string;
  code: string;
  is_supported: boolean;
  support_status: string;
  detected: boolean;
}

export interface RegionDetectionResult {
  success: boolean;
  region: 'Usa' | 'Jpn' | 'Pal' | null;
  display_name: string | null;
  is_supported: boolean;
  sha1: string;
  error_message: string | null;
}

export interface RegionSelectorProps {
  romPath?: string;
  onRegionDetected?: (result: RegionDetectionResult) => void;
  /** Called exactly once when the user confirms loading the ROM. */
  onRegionSelected?: () => void;
  showForceLoad?: boolean;
  disabled?: boolean;
}

export function RegionSelector({ 
  romPath, 
  onRegionDetected, 
  onRegionSelected,
  showForceLoad = true,
  disabled = false 
}: RegionSelectorProps) {
  const [detectedRegion, setDetectedRegion] = useState<RegionDetectionResult | null>(null);
  const [allRegions, setAllRegions] = useState<RomRegionInfo[]>([]);
  const [selectedRegion, setSelectedRegion] = useState<string>('');
  const [isDetecting, setIsDetecting] = useState(false);
  const [showForceConfirm, setShowForceConfirm] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Load all supported regions on mount
  useEffect(() => {
    const loadRegions = async () => {
      try {
        const regions = await invoke<RomRegionInfo[]>('get_supported_regions');
        setAllRegions(regions);
      } catch (err) {
        console.error('Failed to load regions:', err);
        setError('Failed to load region information');
      }
    };
    
    loadRegions();
  }, []);

  // Detect region when romPath changes
  useEffect(() => {
    if (!romPath) {
      setDetectedRegion(null);
      return;
    }

    const detectRegion = async () => {
      setIsDetecting(true);
      setError(null);
      
      try {
        const result = await invoke<RegionDetectionResult>('detect_rom_region', {
          romPath
        });
        
        setDetectedRegion(result);
        
        if (result.success && result.region) {
          setSelectedRegion(result.region.toLowerCase());
        }
        
        onRegionDetected?.(result);
      } catch (err) {
        console.error('Region detection failed:', err);
        setError(err instanceof Error ? err.message : 'Region detection failed');
      } finally {
        setIsDetecting(false);
      }
    };

    detectRegion();
  }, [romPath, onRegionDetected]);

  // Handle radio change: update local selection only.
  // The actual load is triggered by the explicit confirm button below.
  const handleRegionSelect = useCallback((regionCode: string) => {
    setSelectedRegion(regionCode);
  }, []);

  // Handle force load confirmation
  const handleForceLoad = useCallback(() => {
    setShowForceConfirm(true);
  }, []);

  const confirmForceLoad = useCallback(() => {
    setShowForceConfirm(false);
    onRegionSelected?.();
  }, [onRegionSelected]);

  // Get status icon for a region
  const getStatusIcon = (region: RomRegionInfo) => {
    if (region.detected) {
      return region.is_supported ? '✅' : '⚠️';
    }
    return region.is_supported ? '○' : '◌';
  };

  // Get status color for a region
  const getStatusColor = (region: RomRegionInfo) => {
    if (region.detected) {
      return region.is_supported ? 'var(--success)' : 'var(--warning)';
    }
    return 'var(--text-muted)';
  };

  if (isDetecting) {
    return (
      <div className="region-selector loading" style={containerStyle}>
        <div style={{ textAlign: 'center', padding: '2rem' }}>
          <div className="spinner" style={spinnerStyle} />
          <p style={{ marginTop: '1rem', color: 'var(--text-muted)' }}>
            Detecting ROM region...
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="region-selector" style={containerStyle}>
      <h3 style={headerStyle}>ROM Region</h3>
      
      {/* Detection Result */}
      {detectedRegion && (
        <div style={detectionResultStyle}>
          {detectedRegion.success ? (
            <>
              <div style={detectedHeaderStyle}>
                <span style={{ fontSize: '1.5rem' }}>
                  {detectedRegion.is_supported ? '✅' : '⚠️'}
                </span>
                <div>
                  <div style={{ fontWeight: 'bold' }}>
                    Detected: {detectedRegion.display_name}
                  </div>
                  <div style={{ fontSize: '0.875rem', color: 'var(--text-muted)' }}>
                    SHA1: {detectedRegion.sha1.substring(0, 16)}...
                  </div>
                </div>
              </div>
              <div style={{
                ...statusBadgeStyle,
                backgroundColor: detectedRegion.is_supported 
                  ? 'var(--success-bg, rgba(34, 197, 94, 0.2))' 
                  : 'var(--warning-bg, rgba(234, 179, 8, 0.2))',
                color: detectedRegion.is_supported 
                  ? 'var(--success)' 
                  : 'var(--warning)',
              }}>
                {detectedRegion.is_supported ? 'Fully Supported' : 'Limited Support'}
              </div>
            </>
          ) : (
            <>
              <div style={detectedHeaderStyle}>
                <span style={{ fontSize: '1.5rem' }}>❌</span>
                <div>
                  <div style={{ fontWeight: 'bold', color: 'var(--error)' }}>
                    Unknown ROM
                  </div>
                  <div style={{ fontSize: '0.875rem', color: 'var(--text-muted)' }}>
                    SHA1: {detectedRegion.sha1.substring(0, 16)}...
                  </div>
                </div>
              </div>
              {detectedRegion.error_message && (
                <div style={{ fontSize: '0.875rem', color: 'var(--error)', marginTop: '0.5rem' }}>
                  {detectedRegion.error_message}
                </div>
              )}
            </>
          )}
        </div>
      )}

      {/* Error Display */}
      {error && (
        <div style={errorStyle}>
          ⚠️ {error}
        </div>
      )}

      {/* Region Selection List */}
      <div style={regionListStyle}>
        <div style={sectionLabelStyle}>Available Regions:</div>
        {allRegions.map((region) => (
          <label
            key={region.code}
            style={{
              ...regionOptionStyle,
              color: getStatusColor(region),
              opacity: disabled ? 0.5 : 1,
              cursor: disabled ? 'not-allowed' : 'pointer',
            }}
          >
            <input
              type="radio"
              name="rom-region"
              value={region.code.toLowerCase()}
              checked={selectedRegion === region.code.toLowerCase()}
              onChange={() => handleRegionSelect(region.code.toLowerCase())}
              disabled={disabled || !region.is_supported}
              style={{ marginRight: '0.5rem' }}
            />
            <span style={{ marginRight: '0.5rem' }}>
              {getStatusIcon(region)}
            </span>
            <span style={{ flex: 1 }}>{region.display_name}</span>
            <span style={{ fontSize: '0.75rem', opacity: 0.7 }}>
              {region.support_status}
            </span>
          </label>
        ))}
      </div>

      {/* Region Notes */}
      <div style={notesStyle}>
        <div style={sectionLabelStyle}>Region-Specific Notes:</div>
        <ul style={notesListStyle}>
          <li>Text encoding may differ between regions</li>
          <li>Some boxers may have different names in JPN/PAL versions</li>
          <li>Assets may be at different offsets between regions</li>
          <li>The app now loads region-specific manifests for USA, JPN, and PAL ROMs</li>
        </ul>
      </div>

      {/* Force Load Option */}
      {showForceLoad && detectedRegion && !detectedRegion.success && (
        <div style={forceLoadSectionStyle}>
          {!showForceConfirm ? (
            <button
              onClick={handleForceLoad}
              style={forceLoadButtonStyle}
              disabled={disabled}
            >
              ⚠️ Force Load (for research)
            </button>
          ) : (
            <div style={confirmBoxStyle}>
              <p style={{ margin: '0 0 0.5rem 0', fontSize: '0.875rem' }}>
                ⚠️ <strong>Warning:</strong> Force loading an unsupported ROM may cause 
                crashes or data corruption. Only use this for research purposes.
              </p>
              <div style={{ display: 'flex', gap: '0.5rem' }}>
                <button
                  onClick={confirmForceLoad}
                  style={{ ...forceLoadButtonStyle, flex: 1 }}
                >
                  Confirm Force Load
                </button>
                <button
                  onClick={() => setShowForceConfirm(false)}
                  style={cancelButtonStyle}
                >
                  Cancel
                </button>
              </div>
            </div>
          )}
        </div>
      )}

      {/* Load ROM button — the single confirmation action that triggers loading */}
      {selectedRegion && detectedRegion?.is_supported && (
        <button
          type="button"
          onClick={() => onRegionSelected?.()}
          disabled={disabled}
          style={{
            ...loadButtonStyle,
            opacity: disabled ? 0.5 : 1,
            cursor: disabled ? 'not-allowed' : 'pointer',
          }}
        >
          Open ROM
        </button>
      )}
    </div>
  );
}

// Styles
const containerStyle: React.CSSProperties = {
  padding: '1.5rem',
  backgroundColor: 'var(--bg-panel)',
  borderRadius: '8px',
  border: '1px solid var(--border)',
};

const headerStyle: React.CSSProperties = {
  margin: '0 0 1rem 0',
  fontSize: '1.125rem',
  fontWeight: 600,
};

const detectionResultStyle: React.CSSProperties = {
  padding: '1rem',
  backgroundColor: 'var(--glass)',
  borderRadius: '6px',
  marginBottom: '1rem',
};

const detectedHeaderStyle: React.CSSProperties = {
  display: 'flex',
  alignItems: 'center',
  gap: '0.75rem',
};

const statusBadgeStyle: React.CSSProperties = {
  display: 'inline-block',
  padding: '0.25rem 0.5rem',
  borderRadius: '4px',
  fontSize: '0.75rem',
  fontWeight: 600,
  marginTop: '0.5rem',
};

const errorStyle: React.CSSProperties = {
  padding: '0.75rem',
  backgroundColor: 'var(--error-bg, rgba(239, 68, 68, 0.1))',
  color: 'var(--error)',
  borderRadius: '6px',
  marginBottom: '1rem',
  fontSize: '0.875rem',
};

const regionListStyle: React.CSSProperties = {
  marginBottom: '1rem',
};

const sectionLabelStyle: React.CSSProperties = {
  fontSize: '0.75rem',
  fontWeight: 600,
  color: 'var(--text-muted)',
  textTransform: 'uppercase',
  letterSpacing: '0.05em',
  marginBottom: '0.5rem',
};

const regionOptionStyle: React.CSSProperties = {
  display: 'flex',
  alignItems: 'center',
  padding: '0.5rem',
  borderRadius: '4px',
  transition: 'background-color 0.2s',
};

const notesStyle: React.CSSProperties = {
  padding: '0.75rem',
  backgroundColor: 'var(--info-bg, rgba(59, 130, 246, 0.1))',
  borderRadius: '6px',
  marginBottom: '1rem',
};

const notesListStyle: React.CSSProperties = {
  margin: '0',
  paddingLeft: '1.25rem',
  fontSize: '0.875rem',
  color: 'var(--text-muted)',
  lineHeight: 1.6,
};

const forceLoadSectionStyle: React.CSSProperties = {
  marginBottom: '1rem',
};

const forceLoadButtonStyle: React.CSSProperties = {
  padding: '0.5rem 1rem',
  backgroundColor: 'var(--warning-bg, rgba(234, 179, 8, 0.2))',
  color: 'var(--warning)',
  border: '1px solid var(--warning)',
  borderRadius: '4px',
  cursor: 'pointer',
  fontSize: '0.875rem',
};

const confirmBoxStyle: React.CSSProperties = {
  padding: '0.75rem',
  backgroundColor: 'var(--warning-bg, rgba(234, 179, 8, 0.1))',
  border: '1px solid var(--warning)',
  borderRadius: '6px',
};

const cancelButtonStyle: React.CSSProperties = {
  padding: '0.5rem 1rem',
  backgroundColor: 'var(--glass)',
  border: '1px solid var(--border)',
  borderRadius: '4px',
  cursor: 'pointer',
  fontSize: '0.875rem',
};

const loadButtonStyle: React.CSSProperties = {
  width: '100%',
  padding: '0.75rem',
  backgroundColor: 'var(--accent)',
  color: 'white',
  border: 'none',
  borderRadius: '6px',
  cursor: 'pointer',
  fontSize: '0.875rem',
  fontWeight: 600,
};

const spinnerStyle: React.CSSProperties = {
  width: '32px',
  height: '32px',
  border: '3px solid var(--border)',
  borderTopColor: 'var(--accent)',
  borderRadius: '50%',
  animation: 'spin 1s linear infinite',
  margin: '0 auto',
};

// Spinner keyframes are defined in App.css (@keyframes spin).

export default RegionSelector;
