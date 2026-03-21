import { save, open } from '@tauri-apps/plugin-dialog';
import { useStore, BoxerRecord, AssetFile } from '../store/useStore';
import { SharedBankIndicator } from './SharedBankIndicator';

interface AssetManagerProps {
  boxer: BoxerRecord;
}

interface AssetWithShared extends AssetFile {
  assetType: 'icon' | 'portrait' | 'large_portrait' | 'other';
  index: number;
}

export const AssetManager = ({ boxer }: AssetManagerProps) => {
  const { exportAsset, importAsset } = useStore();
  const iconFiles = boxer.icon_files ?? [];
  const portraitFiles = boxer.portrait_files ?? [];
  const largePortraitFiles = boxer.large_portrait_files ?? [];
  const otherFiles = boxer.other_files ?? [];
  const paletteFiles = boxer.palette_files ?? [];

  // Combine all assets with their type info
  const allAssets: AssetWithShared[] = [
    ...iconFiles.map((a, i) => ({ ...a, assetType: 'icon' as const, index: i })),
    ...portraitFiles.map((a, i) => ({ ...a, assetType: 'portrait' as const, index: i })),
    ...largePortraitFiles.map((a, i) => ({ ...a, assetType: 'large_portrait' as const, index: i })),
    ...otherFiles.map((a, i) => ({ ...a, assetType: 'other' as const, index: i })),
  ];

  const getSharedWith = (sharedWith: string[] | undefined): string[] =>
    Array.isArray(sharedWith) ? sharedWith : [];

  const handleExport = async (asset: AssetWithShared) => {
    try {
      const path = await save({
        filters: [{ name: 'PNG Image', extensions: ['png'] }],
        defaultPath: `${boxer.name}_${asset.subtype}${asset.index > 0 ? '_' + (asset.index + 1) : ''}.png`
      });
      if (path && paletteFiles.length > 0) {
        await exportAsset(asset, paletteFiles[0], path);
        alert(`Exported to ${path}`);
      }
    } catch (e) {
      console.error(e);
    }
  };

  const handleImport = async (asset: AssetWithShared) => {
    // Check if this is a shared asset
    const sharedWith = getSharedWith(asset.shared_with);
    if (sharedWith.length > 0) {
      const otherFighters = sharedWith.filter(
        f => f.toLowerCase() !== boxer.name.toLowerCase()
      );
      
      if (otherFighters.length > 0) {
        const confirmed = window.confirm(
          `⚠️ SHARED ASSET WARNING\n\n` +
          `"${asset.filename}" is shared with: ${otherFighters.join(', ')}\n\n` +
          `Importing will affect ALL fighters that use this asset.\n\n` +
          `Do you want to continue?`
        );
        if (!confirmed) return;
      }
    }

    try {
      const path = await open({
        filters: [{ name: 'PNG Image', extensions: ['png'] }],
        multiple: false
      });
      if (typeof path === 'string' && paletteFiles.length > 0) {
        const bytes = await importAsset(paletteFiles[0], path);
        if (bytes) {
          alert(`Successfully imported ${bytes.length} bytes for ${asset.subtype}. (Saving to ROM pending Project System)`);
        }
      }
    } catch (e) {
      console.error(e);
    }
  };

  const getAssetTypeLabel = (asset: AssetWithShared) => {
    switch (asset.assetType) {
      case 'icon':
        return `Icon ${asset.index + 1}`;
      case 'portrait':
        return `Portrait ${asset.index + 1}`;
      case 'large_portrait':
        return `Large Portrait ${asset.index + 1}`;
      default:
        return `${asset.subtype} ${asset.index + 1}`;
    }
  };

  const getAssetIcon = (assetType: string) => {
    switch (assetType) {
      case 'icon':
        return '🎭';
      case 'portrait':
        return '🖼️';
      case 'large_portrait':
        return '📸';
      default:
        return '📄';
    }
  };

  // Count shared assets
  const sharedAssetCount = allAssets.filter(
    a => getSharedWith(a.shared_with).length > 0
  ).length;

  return (
    <div className="asset-manager" style={{ marginTop: '2rem' }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', marginBottom: '1rem' }}>
        <div>
          <h3 style={{ margin: 0 }}>Graphic Assets</h3>
          <p style={{ margin: '4px 0 0', color: 'var(--text-dim)', fontSize: '0.85rem' }}>
            {allAssets.length} assets · {sharedAssetCount} shared
          </p>
        </div>
      </div>

      {/* Shared asset warning summary */}
      {sharedAssetCount > 0 && (
        <div
          style={{
            marginBottom: '1rem',
            padding: '10px 14px',
            borderRadius: '8px',
            backgroundColor: 'rgba(255, 80, 80, 0.08)',
            border: '1px solid rgba(255, 80, 80, 0.25)',
            fontSize: '0.82rem',
            color: '#ff8888',
          }}
        >
          ⚠️ {sharedAssetCount} asset{sharedAssetCount !== 1 ? 's' : ''} in this section are shared with other fighters.
          Editing them will affect all paired fighters.
        </div>
      )}
      
      <div style={{ display: 'grid', gap: '1rem', marginTop: '1rem' }}>
        {allAssets.map((asset, idx) => {
          const sharedWith = getSharedWith(asset.shared_with);
          const isShared = sharedWith.length > 0;
          const otherFighters = isShared
            ? sharedWith.filter(f => f.toLowerCase() !== boxer.name.toLowerCase())
            : [];

          return (
            <div 
              key={`${asset.assetType}-${idx}`} 
              className="asset-card" 
              style={{ 
                display: 'flex', 
                justifyContent: 'space-between', 
                alignItems: 'center',
                backgroundColor: isShared ? 'rgba(255, 80, 80, 0.05)' : 'var(--glass)',
                border: `1px solid ${isShared ? 'rgba(255, 80, 80, 0.3)' : 'var(--border)'}`,
                padding: '1rem',
                borderRadius: '8px',
                transition: 'all 0.2s',
              }}
            >
              <div style={{ flex: 1, minWidth: 0 }}>
                <div style={{ display: 'flex', alignItems: 'center', gap: '8px', flexWrap: 'wrap' }}>
                  <span>{getAssetIcon(asset.assetType)}</span>
                  <strong>{getAssetTypeLabel(asset)}</strong>
                  {isShared && (
                    <SharedBankIndicator
                      sharedWith={sharedWith}
                      currentBoxer={boxer.name}
                      size="small"
                    />
                  )}
                </div>
                <div style={{ 
                  fontSize: '0.8rem', 
                  color: isShared ? '#ff8888' : 'var(--text-dim)',
                  marginTop: '4px' 
                }}>
                  {asset.size} bytes 
                  {asset.category.includes('Compressed') ? '(Compressed)' : ''} 
                  {' '}@ {asset.start_pc}
                  {isShared && otherFighters.length > 0 && (
                    <span style={{ marginLeft: '8px' }}>
                      → also used by: {otherFighters.join(', ')}
                    </span>
                  )}
                </div>
                <div style={{ fontSize: '0.75rem', color: 'var(--text-dim)', marginTop: '2px', fontFamily: 'monospace' }}>
                  {asset.filename}
                </div>
              </div>
              <div style={{ display: 'flex', gap: '8px', flexShrink: 0 }}>
                <button 
                  onClick={() => handleExport(asset)} 
                  style={{ 
                    fontSize: '0.8rem', 
                    padding: '6px 12px',
                  }}
                >
                  Export
                </button>
                {asset.assetType === 'icon' && (
                  <button 
                    onClick={() => handleImport(asset)} 
                    style={{ 
                      fontSize: '0.8rem', 
                      padding: '6px 12px', 
                      backgroundColor: isShared ? 'rgba(255, 80, 80, 0.15)' : 'var(--border)',
                      borderColor: isShared ? 'rgba(255, 80, 80, 0.4)' : undefined,
                      color: isShared ? '#ff8888' : undefined,
                    }}
                    title={isShared 
                      ? `⚠️ SHARED: Editing affects ${otherFighters.join(' & ')}!` 
                      : 'Import PNG'
                    }
                  >
                    {isShared ? '⚠ Import' : 'Import'}
                  </button>
                )}
              </div>
            </div>
          );
        })}
      </div>

      {allAssets.length === 0 && (
        <div style={{ 
          padding: '2rem', 
          textAlign: 'center', 
          color: 'var(--text-dim)',
          backgroundColor: 'var(--glass)',
          borderRadius: '8px',
          border: '1px dashed var(--border)'
        }}>
          No graphic assets available for this fighter.
        </div>
      )}

      <div style={{ marginTop: '1rem', color: 'var(--text-dim)', fontSize: '0.9rem' }}>
          * Portions of this data may be compressed. Exporting works for all assets, but re-importing compressed assets requires special handling.
      </div>
    </div>
  );
};

export default AssetManager;
