import { useState, useEffect, useCallback } from 'react';
import { useStore, FreeSpaceRegion, RomSpaceInfo, RelocationValidationResult, RelocationPreview } from '../store/useStore';

interface RelocationHelperProps {
  boxerKey?: string;
  assetFile?: string;
  initialSourceAddress?: string;
  initialSize?: number;
  onRelocationComplete?: () => void;
}

export const RelocationHelper = ({
  boxerKey,
  assetFile,
  initialSourceAddress,
  initialSize,
  onRelocationComplete,
}: RelocationHelperProps) => {
  const {
    getFreeSpaceRegions,
    getRomSpaceInfo,
    validateRelocation,
    previewRelocation,
    relocateAsset,
    getAssetsAtAddress,
    selectedBoxer,
  } = useStore();

  const [spaceInfo, setSpaceInfo] = useState<RomSpaceInfo | null>(null);
  const [freeRegions, setFreeRegions] = useState<FreeSpaceRegion[]>([]);
  const [sourceAddress, setSourceAddress] = useState(initialSourceAddress || '');
  const [destAddress, setDestAddress] = useState('');
  const [size, setSize] = useState(initialSize?.toString() || '');
  const [selectedRegion, setSelectedRegion] = useState<FreeSpaceRegion | null>(null);
  const [validation, setValidation] = useState<RelocationValidationResult | null>(null);
  const [preview, setPreview] = useState<RelocationPreview | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [showConfirm, setShowConfirm] = useState(false);
  const [destAssets, setDestAssets] = useState<Array<{ file: string; start_pc: string; end_pc: string }>>([]);
  const [filterMinSize, setFilterMinSize] = useState<string>('');

  const currentBoxerKey = boxerKey || selectedBoxer?.key || '';

  // Load ROM space info on mount
  useEffect(() => {
    loadSpaceInfo();
  }, []);

  const loadSpaceInfo = async () => {
    setIsLoading(true);
    try {
      const info = await getRomSpaceInfo();
      setSpaceInfo(info);
      setFreeRegions(info.free_regions);
    } catch (e) {
      console.error('Failed to load space info:', e);
    } finally {
      setIsLoading(false);
    }
  };

  const handleFilterChange = async (minSizeStr: string) => {
    setFilterMinSize(minSizeStr);
    const minSize = parseInt(minSizeStr, 10);
    if (!isNaN(minSize) && minSize > 0) {
      const regions = await getFreeSpaceRegions(minSize);
      setFreeRegions(regions);
    } else {
      setFreeRegions(spaceInfo?.free_regions || []);
    }
  };

  const handleRegionSelect = (region: FreeSpaceRegion) => {
    setSelectedRegion(region);
    setDestAddress(`0x${region.start_pc.toString(16).toUpperCase()}`);
  };

  const handleValidate = async () => {
    if (!sourceAddress || !destAddress || !size) return;

    setIsLoading(true);
    try {
      const result = await validateRelocation(sourceAddress, destAddress, parseInt(size, 10));
      setValidation(result);
      
      // Also get preview
      const previewResult = await previewRelocation(sourceAddress, destAddress, parseInt(size, 10));
      setPreview(previewResult);

      // Get assets at destination
      if (previewResult.dest_occupied_by.length > 0) {
        const assets = await getAssetsAtAddress(destAddress);
        setDestAssets(assets.map(a => ({ file: a.file, start_pc: a.start_pc, end_pc: a.end_pc })));
      } else {
        setDestAssets([]);
      }
    } catch (e) {
      console.error('Validation failed:', e);
    } finally {
      setIsLoading(false);
    }
  };

  const handleRelocate = async () => {
    if (!currentBoxerKey || !assetFile || !destAddress) return;

    setIsLoading(true);
    try {
      const result = await relocateAsset(currentBoxerKey, assetFile, destAddress);
      if (result.success) {
        setShowConfirm(false);
        onRelocationComplete?.();
        // Reload space info to reflect changes
        await loadSpaceInfo();
      }
    } catch (e) {
      console.error('Relocation failed:', e);
    } finally {
      setIsLoading(false);
    }
  };

  const formatSize = (bytes: number): string => {
    if (bytes >= 1024) {
      return `${(bytes / 1024).toFixed(2)} KB`;
    }
    return `${bytes} bytes`;
  };

  const formatAddress = (addr: number): string => {
    return `0x${addr.toString(16).toUpperCase().padStart(4, '0')}`;
  };

  return (
    <div className="relocation-helper" style={{
      padding: '1.5rem',
      backgroundColor: 'var(--bg-secondary)',
      borderRadius: '12px',
      border: '1px solid var(--border)',
    }}>
      <h2 style={{ marginBottom: '1rem', display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
        <span>🚚</span>
        Relocation Helper
      </h2>

      {/* ROM Space Summary */}
      {spaceInfo && (
        <div style={{
          marginBottom: '1.5rem',
          padding: '1rem',
          backgroundColor: 'var(--bg-tertiary)',
          borderRadius: '8px',
        }}>
          <h3 style={{ fontSize: '0.9rem', marginBottom: '0.75rem', color: 'var(--text-secondary)' }}>
            ROM Space Usage
          </h3>
          <div style={{
            display: 'grid',
            gridTemplateColumns: 'repeat(auto-fit, minmax(120px, 1fr))',
            gap: '1rem',
          }}>
            <SpaceStat label="Total" value={formatSize(spaceInfo.total_size)} />
            <SpaceStat label="Used" value={formatSize(spaceInfo.allocated_bytes)} color="#fbbf24" />
            <SpaceStat label="Free" value={formatSize(spaceInfo.free_bytes)} color="#4ade80" />
            <SpaceStat 
              label="Utilization" 
              value={`${spaceInfo.utilization_percent.toFixed(1)}%`} 
              color={spaceInfo.utilization_percent > 90 ? '#f87171' : '#60a5fa'}
            />
          </div>
          
          {/* Visual bar */}
          <div style={{
            marginTop: '0.75rem',
            height: '8px',
            backgroundColor: 'var(--bg-secondary)',
            borderRadius: '4px',
            overflow: 'hidden',
          }}>
            <div style={{
              width: `${spaceInfo.utilization_percent}%`,
              height: '100%',
              backgroundColor: spaceInfo.utilization_percent > 90 ? '#f87171' : '#60a5fa',
              transition: 'width 0.3s ease',
            }} />
          </div>

          {spaceInfo.fragmentation_score > 0.3 && (
            <div style={{
              marginTop: '0.5rem',
              fontSize: '0.8rem',
              color: '#fbbf24',
            }}>
              ⚠️ High fragmentation detected ({(spaceInfo.fragmentation_score * 100).toFixed(0)}%)
            </div>
          )}
        </div>
      )}

      {/* Relocation Form */}
      <div style={{ marginBottom: '1.5rem' }}>
        <h3 style={{ fontSize: '0.9rem', marginBottom: '0.75rem', color: 'var(--text-secondary)' }}>
          Relocation Parameters
        </h3>
        
        <div style={{ display: 'grid', gap: '0.75rem' }}>
          <InputRow>
            <label>Source Address (PC):</label>
            <input
              type="text"
              value={sourceAddress}
              onChange={(e) => setSourceAddress(e.target.value)}
              placeholder="0x8000"
              style={inputStyle}
            />
          </InputRow>

          <InputRow>
            <label>Destination Address (PC):</label>
            <input
              type="text"
              value={destAddress}
              onChange={(e) => setDestAddress(e.target.value)}
              placeholder="0x10000"
              style={inputStyle}
            />
          </InputRow>

          <InputRow>
            <label>Size (bytes):</label>
            <input
              type="text"
              value={size}
              onChange={(e) => setSize(e.target.value)}
              placeholder="1024"
              style={inputStyle}
            />
          </InputRow>
        </div>

        <button
          onClick={handleValidate}
          disabled={!sourceAddress || !destAddress || !size || isLoading}
          style={{
            marginTop: '1rem',
            padding: '0.5rem 1rem',
            backgroundColor: 'var(--primary)',
            color: 'white',
            border: 'none',
            borderRadius: '6px',
            cursor: isLoading ? 'not-allowed' : 'pointer',
            opacity: isLoading ? 0.6 : 1,
          }}
        >
          {isLoading ? 'Validating...' : 'Validate Relocation'}
        </button>
      </div>

      {/* Validation Results */}
      {validation && (
        <div style={{
          marginBottom: '1.5rem',
          padding: '1rem',
          backgroundColor: validation.valid ? 'rgba(74, 222, 128, 0.1)' : 'rgba(248, 113, 113, 0.1)',
          border: `1px solid ${validation.valid ? '#4ade80' : '#f87171'}`,
          borderRadius: '8px',
        }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', marginBottom: '0.5rem' }}>
            <span style={{ fontSize: '1.2rem' }}>
              {validation.valid ? '✅' : '❌'}
            </span>
            <strong style={{ color: validation.valid ? '#4ade80' : '#f87171' }}>
              {validation.valid ? 'Relocation is Valid' : 'Relocation Invalid'}
            </strong>
            <span style={{
              marginLeft: 'auto',
              padding: '0.25rem 0.5rem',
              backgroundColor: validation.risk_color,
              color: 'white',
              borderRadius: '4px',
              fontSize: '0.75rem',
              fontWeight: 'bold',
            }}>
              {validation.risk_level} Risk
            </span>
          </div>

          {validation.warnings.length > 0 && (
            <div style={{ marginTop: '0.75rem' }}>
              <strong style={{ fontSize: '0.8rem', color: '#fbbf24' }}>Warnings:</strong>
              <ul style={{ margin: '0.25rem 0', paddingLeft: '1.25rem', fontSize: '0.85rem' }}>
                {validation.warnings.map((w, i) => (
                  <li key={i} style={{ color: '#fbbf24' }}>{w}</li>
                ))}
              </ul>
            </div>
          )}

          {validation.errors.length > 0 && (
            <div style={{ marginTop: '0.75rem' }}>
              <strong style={{ fontSize: '0.8rem', color: '#f87171' }}>Errors:</strong>
              <ul style={{ margin: '0.25rem 0', paddingLeft: '1.25rem', fontSize: '0.85rem' }}>
                {validation.errors.map((e, i) => (
                  <li key={i} style={{ color: '#f87171' }}>{e}</li>
                ))}
              </ul>
            </div>
          )}

          {validation.valid && (
            <div style={{ marginTop: '0.75rem', fontSize: '0.85rem' }}>
              <div>📊 Estimated pointer updates needed: {validation.estimated_pointer_updates}</div>
            </div>
          )}

          {validation.valid && currentBoxerKey && assetFile && (
            <button
              onClick={() => setShowConfirm(true)}
              style={{
                marginTop: '1rem',
                padding: '0.5rem 1rem',
                backgroundColor: '#f97316',
                color: 'white',
                border: 'none',
                borderRadius: '6px',
                cursor: 'pointer',
              }}
            >
              Proceed with Relocation
            </button>
          )}
        </div>
      )}

      {/* Destination Preview */}
      {preview && preview.dest_occupied_by.length > 0 && (
        <div style={{
          marginBottom: '1.5rem',
          padding: '1rem',
          backgroundColor: 'rgba(251, 191, 36, 0.1)',
          border: '1px solid #fbbf24',
          borderRadius: '8px',
        }}>
          <h4 style={{ marginBottom: '0.5rem', color: '#fbbf24' }}>
            ⚠️ Destination Currently Occupied
          </h4>
          <p style={{ fontSize: '0.85rem', marginBottom: '0.5rem' }}>
            The following assets are currently at the destination address:
          </p>
          <ul style={{ margin: 0, paddingLeft: '1.25rem', fontSize: '0.85rem' }}>
            {destAssets.map((asset, i) => (
              <li key={i}>
                {asset.file} ({asset.start_pc} - {asset.end_pc})
              </li>
            ))}
          </ul>
        </div>
      )}

      {/* Free Space Regions */}
      <div>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '0.75rem' }}>
          <h3 style={{ fontSize: '0.9rem', color: 'var(--text-secondary)' }}>
            Free Space Regions
          </h3>
          <InputRow style={{ margin: 0 }}>
            <label style={{ fontSize: '0.8rem' }}>Min Size:</label>
            <input
              type="text"
              value={filterMinSize}
              onChange={(e) => handleFilterChange(e.target.value)}
              placeholder="bytes"
              style={{ ...inputStyle, width: '80px' }}
            />
          </InputRow>
        </div>

        <div style={{
          maxHeight: '300px',
          overflowY: 'auto',
          border: '1px solid var(--border)',
          borderRadius: '8px',
        }}>
          {freeRegions.length === 0 ? (
            <div style={{ padding: '1rem', textAlign: 'center', color: 'var(--text-dim)' }}>
              No free regions found
            </div>
          ) : (
            freeRegions.map((region, i) => (
              <div
                key={i}
                onClick={() => handleRegionSelect(region)}
                style={{
                  padding: '0.75rem 1rem',
                  borderBottom: i < freeRegions.length - 1 ? '1px solid var(--border)' : 'none',
                  cursor: 'pointer',
                  backgroundColor: selectedRegion?.start_pc === region.start_pc 
                    ? 'var(--primary-hover)' 
                    : 'transparent',
                  transition: 'background-color 0.15s ease',
                }}
              >
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                  <div>
                    <div style={{ fontFamily: 'monospace', fontSize: '0.9rem' }}>
                      {formatAddress(region.start_pc)} - {formatAddress(region.end_pc)}
                    </div>
                    <div style={{ fontSize: '0.8rem', color: 'var(--text-dim)' }}>
                      SNES: {region.start_snes} - {region.end_snes}
                    </div>
                  </div>
                  <div style={{
                    padding: '0.25rem 0.5rem',
                    backgroundColor: 'var(--bg-tertiary)',
                    borderRadius: '4px',
                    fontSize: '0.8rem',
                    fontWeight: 'bold',
                  }}>
                    {formatSize(region.size)}
                  </div>
                </div>
              </div>
            ))
          )}
        </div>
      </div>

      {/* Confirmation Dialog */}
      {showConfirm && (
        <div style={{
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
        }}>
          <div style={{
            backgroundColor: 'var(--bg-secondary)',
            padding: '1.5rem',
            borderRadius: '12px',
            maxWidth: '400px',
            width: '90%',
          }}>
            <h3 style={{ marginBottom: '1rem' }}>Confirm Relocation</h3>
            <p style={{ marginBottom: '1rem', fontSize: '0.9rem' }}>
              Are you sure you want to relocate <strong>{assetFile}</strong> from{' '}
              <code>{sourceAddress}</code> to <code>{destAddress}</code>?
            </p>
            <div style={{
              padding: '0.75rem',
              backgroundColor: 'rgba(251, 191, 36, 0.1)',
              borderRadius: '6px',
              marginBottom: '1rem',
              fontSize: '0.85rem',
            }}>
              <strong>⚠️ Warning:</strong> This operation will modify the ROM. Make sure you have a backup.
            </div>
            <div style={{ display: 'flex', gap: '0.5rem', justifyContent: 'flex-end' }}>
              <button
                onClick={() => setShowConfirm(false)}
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
                onClick={handleRelocate}
                disabled={isLoading}
                style={{
                  padding: '0.5rem 1rem',
                  backgroundColor: '#f97316',
                  color: 'white',
                  border: 'none',
                  borderRadius: '6px',
                  cursor: isLoading ? 'not-allowed' : 'pointer',
                  opacity: isLoading ? 0.6 : 1,
                }}
              >
                {isLoading ? 'Relocating...' : 'Confirm Relocation'}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

// Helper components

const SpaceStat = ({ label, value, color }: { label: string; value: string; color?: string }) => (
  <div>
    <div style={{ fontSize: '0.75rem', color: 'var(--text-dim)', marginBottom: '0.25rem' }}>
      {label}
    </div>
    <div style={{ 
      fontSize: '1rem', 
      fontWeight: 'bold',
      color: color || 'var(--text-primary)',
    }}>
      {value}
    </div>
  </div>
);

const InputRow = ({ children, style }: { children: React.ReactNode; style?: React.CSSProperties }) => (
  <div style={{ 
    display: 'flex', 
    alignItems: 'center', 
    gap: '0.75rem',
    ...style,
  }}>
    {children}
  </div>
);

const inputStyle: React.CSSProperties = {
  flex: 1,
  padding: '0.5rem 0.75rem',
  backgroundColor: 'var(--bg-tertiary)',
  border: '1px solid var(--border)',
  borderRadius: '6px',
  color: 'var(--text-primary)',
  fontFamily: 'monospace',
  fontSize: '0.9rem',
};

export default RelocationHelper;
