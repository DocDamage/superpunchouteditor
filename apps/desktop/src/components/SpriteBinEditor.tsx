import { useState, useRef, useEffect } from 'react';
import { save, open } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';
import { useStore, AssetFile, BoxerRecord, BankDuplication } from '../store/useStore';
import { SharedBankWarning, SharedBankInfo } from './SharedBankWarning';
import { SharedBankIndicator, SharedBankSummary } from './SharedBankIndicator';
import { showToast } from './ToastContainer';

interface SpriteBinEditorProps {
  boxer: BoxerRecord;
}

interface BinState {
  tileCount: number | null;
  diff: boolean[];
  status: string | null;
  isEdited: boolean;
  isDuplicated: boolean;
  duplicationInfo?: BankDuplication;
}

export const SpriteBinEditor = ({ boxer }: SpriteBinEditorProps) => {
  const { 
    currentPalette, 
    pendingWrites, 
    setPendingWrite, 
    removePendingWrite,
    bankDuplications,
    loadBankDuplications,
    duplicateSharedBank,
    recordSpriteBinEdit,
    recordAssetImport,
  } = useStore();
  const [binStates, setBinStates] = useState<Record<string, BinState>>({});
  const [loadingKey, setLoadingKey] = useState<string | null>(null);
  const [duplicatingKey, setDuplicatingKey] = useState<string | null>(null);
  const previewRef = useRef<HTMLImageElement>(null);
  const uniqueSpriteBins = boxer.unique_sprite_bins ?? [];
  const sharedSpriteBins = boxer.shared_sprite_bins ?? [];
  const paletteFiles = boxer.palette_files ?? [];
  const getSharedWith = (sharedWith: string[] | undefined): string[] =>
    Array.isArray(sharedWith) ? sharedWith : [];
  const getOrCreateState = (key: string): BinState =>
    binStates[key] ?? { tileCount: null, diff: [], status: null, isEdited: false, isDuplicated: false };

  // Load bank duplications when boxer changes
  useEffect(() => {
    loadBankDuplications(boxer.key);
  }, [boxer.key, loadBankDuplications]);

  // Update bin states when duplications change
  useEffect(() => {
    const newStates = { ...binStates };
    let hasChanges = false;
    
    for (const bin of [...uniqueSpriteBins, ...sharedSpriteBins]) {
      const existingState = getOrCreateState(bin.start_pc);
      const duplication = bankDuplications.find(
        d => d.original_pc_offset === parseInt(bin.start_pc)
      );
      
      const isDuplicated = !!duplication;
      if (existingState?.isDuplicated !== isDuplicated) {
        newStates[bin.start_pc] = {
          ...existingState,
          isDuplicated,
          duplicationInfo: duplication,
        };
        hasChanges = true;
      }
    }
    
    if (hasChanges) {
      setBinStates(newStates);
    }
  }, [bankDuplications, boxer.key, uniqueSpriteBins, sharedSpriteBins]);

  // Shared bank warning modal state
  const [warningOpen, setWarningOpen] = useState(false);
  const [selectedBin, setSelectedBin] = useState<(AssetFile & { isShared: boolean }) | null>(null);
  const pendingImportRef = useRef<{
    bin: AssetFile & { isShared: boolean };
    path: string;
  } | null>(null);

  const allBins = [
    ...uniqueSpriteBins.map(b => ({ ...b, isShared: false })),
    ...sharedSpriteBins.map(b => ({ ...b, isShared: true })),
  ];

  if (allBins.length === 0) {
    return (
      <div style={{ color: 'var(--text-dim)', padding: '1rem', textAlign: 'center' }}>
        No sprite bins available for this fighter.
      </div>
    );
  }

  const pal = paletteFiles[0];
  const widthTiles = 16; // Standard 16-tile wide sheet

  const updateState = (key: string, patch: Partial<BinState>) => {
    setBinStates(prev => ({
      ...prev,
      [key]: { ...getOrCreateState(key), ...patch }
    }));
  };

  const handleExport = async (bin: AssetFile & { isShared: boolean }) => {
    if (!pal) return;
    const defaultName = `${boxer.name}_${bin.filename.replace(/\.[^.]+$/, '')}.png`;
    const path = await save({ filters: [{ name: 'PNG Image', extensions: ['png'] }], defaultPath: defaultName });
    if (!path) return;

    setLoadingKey(bin.start_pc);
    try {
      const count = await invoke<number>('export_sprite_bin_to_png', {
        pcOffset: bin.start_pc,
        size: bin.size,
        widthTiles,
        palettePcOffset: pal.start_pc,
        paletteSize: pal.size,
        outputPath: path,
      });
      updateState(bin.start_pc, { tileCount: count, status: `✓ Exported ${count} tiles to ${path.split(/[\\/]/).pop()}` });
    } catch (e) {
      updateState(bin.start_pc, { status: `✗ Export failed: ${e}` });
    } finally {
      setLoadingKey(null);
    }
  };

  const handleImportClick = async (bin: AssetFile & { isShared: boolean }) => {
    if (!pal) return;

    // If shared, show warning modal first
    if (bin.isShared) {
      setSelectedBin(bin);
      setWarningOpen(true);
      return;
    }

    // If not shared, proceed directly
    await proceedWithImport(bin);
  };

  const proceedWithImport = async (bin: AssetFile & { isShared: boolean }) => {
    const path = await open({ filters: [{ name: 'PNG Image', extensions: ['png'] }], multiple: false });
    if (typeof path !== 'string') return;

    setLoadingKey(bin.start_pc);
    try {
      // Get original bytes before import for history
      let oldBytes: number[] = [];
      try {
        oldBytes = await invoke<number[]>('get_bin_original_bytes', { pcOffset: bin.start_pc, size: bin.size });
      } catch {
        // Command may not exist, skip history recording
        oldBytes = [];
      }

      const [newLen, origLen, fits] = await invoke<[number, number, boolean]>('import_sprite_bin_from_png', {
        pcOffset: bin.start_pc,
        originalSize: bin.size,
        palettePcOffset: pal.start_pc,
        paletteSize: pal.size,
        pngPath: path,
      });

      // Fetch diff immediately
      const diff = await invoke<boolean[]>('get_sprite_bin_diff', { pcOffset: bin.start_pc, size: bin.size });
      const changedCount = diff.filter(Boolean).length;
      setPendingWrite(bin.start_pc);

      // Record the edit in history
      if (changedCount > 0) {
        try {
          // Get the new bytes from pending_writes
          const newBytes: number[] = await invoke('get_pending_bytes', { pcOffset: bin.start_pc }) || [];
          if (oldBytes.length > 0 && newBytes.length > 0) {
            await recordAssetImport(bin.start_pc, oldBytes, newBytes, path);
          }
        } catch (e) {
          // History recording failed but import succeeded, continue
          console.warn('Failed to record import in history:', e);
        }
      }

      if (!fits) {
        updateState(bin.start_pc, {
          isEdited: true,
          diff,
          status: `⚠️ OVERFLOW: ${newLen} bytes > ${origLen} bytes capacity. Import staged but CANNOT be written safely.`,
        });
      } else {
        updateState(bin.start_pc, {
          isEdited: true,
          diff,
          status: `✓ Staged: ${changedCount} / ${diff.length} tiles changed (${newLen}/${origLen} bytes used)`,
        });
      }
    } catch (e) {
      updateState(bin.start_pc, { status: `✗ Import failed: ${e}` });
    } finally {
      setLoadingKey(null);
    }
  };

  const handleWarningConfirm = async (shouldDuplicate: boolean) => {
    setWarningOpen(false);

    if (!selectedBin) return;

    if (shouldDuplicate) {
      // Perform the duplication
      setDuplicatingKey(selectedBin.start_pc);
      try {
        const result = await duplicateSharedBank(
          selectedBin.start_pc,
          selectedBin.size,
          boxer.key,
          selectedBin.filename,
          selectedBin.category.includes('Compressed'),
          true // allow expansion
        );
        
        if (result.success && result.duplication) {
          updateState(selectedBin.start_pc, {
            isDuplicated: true,
            duplicationInfo: result.duplication,
            status: `✓ Bank duplicated! Now unique for ${boxer.name}. You can safely edit.`,
          });
          // Now proceed with import
          await proceedWithImport(selectedBin);
        } else {
          updateState(selectedBin.start_pc, {
            status: `✗ Duplication failed: ${result.error || 'Unknown error'}`,
          });
          showToast(`Bank duplication failed: ${result.error || 'Unknown error'}. Proceeding with shared bank — changes will affect all fighters using it.`, 'warning', 8000);
          await proceedWithImport(selectedBin);
        }
      } catch (e) {
        updateState(selectedBin.start_pc, {
          status: `✗ Duplication failed: ${e}`,
        });
        showToast(`Bank duplication failed: ${e}. Proceeding with shared bank — changes will affect all fighters using it.`, 'warning', 8000);
        await proceedWithImport(selectedBin);
      } finally {
        setDuplicatingKey(null);
      }
    } else {
      // Proceed with import without duplicating
      await proceedWithImport(selectedBin);
    }
    
    setSelectedBin(null);
  };

  const handleWarningClose = () => {
    setWarningOpen(false);
    setSelectedBin(null);
  };

  const handleRevert = async (bin: AssetFile) => {
    await invoke('discard_bin_edit', { pcOffset: bin.start_pc });
    removePendingWrite(bin.start_pc);
    updateState(bin.start_pc, { isEdited: false, diff: [], status: '↩ Reverted to original ROM data.' });
  };

  // Prepare shared bank info for warning modal
  const selectedBankInfo: SharedBankInfo | null = selectedBin
    ? {
        filename: selectedBin.filename,
        start_pc: selectedBin.start_pc,
        shared_with: selectedBin.shared_with,
        category: selectedBin.category,
        size: selectedBin.size,
      }
    : null;

  return (
    <div className="sprite-bin-editor">
      {/* Shared Bank Warning Modal */}
      <SharedBankWarning
        isOpen={warningOpen}
        onClose={handleWarningClose}
        onConfirm={handleWarningConfirm}
        bankInfo={selectedBankInfo}
        currentBoxer={boxer.name}
      />

      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', marginBottom: '1rem' }}>
        <div>
          <h3 style={{ margin: 0 }}>Sprite Bins</h3>
          <p style={{ margin: '4px 0 0', color: 'var(--text-dim)', fontSize: '0.85rem' }}>
            {boxer.unique_sprite_bins.length} unique · {boxer.shared_sprite_bins.length} shared
          </p>
        </div>
        {pendingWrites.size > 0 && (
          <div style={{ 
            padding: '6px 12px', 
            borderRadius: '6px', 
            background: 'rgba(255,200,0,0.15)', 
            border: '1px solid rgba(255,200,0,0.4)',
            color: '#ffd700',
            fontSize: '0.85rem',
            fontWeight: 600
          }}>
            ✏️ {pendingWrites.size} unsaved change{pendingWrites.size > 1 ? 's' : ''}
          </div>
        )}
      </div>

      {/* Shared Bank Summary */}
      <div style={{ marginBottom: '1rem' }}>
        <SharedBankSummary
          uniqueCount={boxer.unique_sprite_bins.length}
          sharedCount={boxer.shared_sprite_bins.length}
          sharedBins={boxer.shared_sprite_bins}
          currentBoxer={boxer.name}
        />
      </div>

      <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem' }}>
        {allBins.map((bin) => {
          const state = getOrCreateState(bin.start_pc);
          const isLoading = loadingKey === bin.start_pc;
          const changedTiles = state.diff.filter(Boolean).length;

          return (
            <div
              key={bin.start_pc}
              style={{
                padding: '12px 16px',
                borderRadius: '8px',
                border: `1px solid ${state.isEdited ? 'rgba(255,200,0,0.5)' : bin.isShared ? 'rgba(255,80,80,0.3)' : 'var(--border)'}`,
                background: state.isEdited ? 'rgba(255,200,0,0.05)' : bin.isShared ? 'rgba(255,80,80,0.05)' : 'var(--glass)',
                transition: 'all 0.2s',
              }}
            >
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', gap: '1rem' }}>
                <div style={{ flex: 1, minWidth: 0 }}>
                  <div style={{ display: 'flex', alignItems: 'center', gap: '8px', flexWrap: 'wrap' }}>
                    <span style={{ fontWeight: 600, fontSize: '0.9rem' }}>{bin.filename}</span>
                    {bin.isShared && (
                      <SharedBankIndicator
                        sharedWith={bin.shared_with}
                        currentBoxer={boxer.name}
                        size="small"
                      />
                    )}
                    {state.isEdited && (
                      <span style={{ 
                        fontSize: '0.72rem', 
                        padding: '2px 6px', 
                        borderRadius: '4px', 
                        background: 'rgba(255,200,0,0.2)', 
                        color: '#ffd700',
                        fontWeight: 600 
                      }}>
                        ✏ EDITED
                      </span>
                    )}
                  </div>
                  <div style={{ fontSize: '0.78rem', color: 'var(--text-dim)', marginTop: '3px' }}>
                    {bin.size} bytes @ {bin.start_pc}
                    {state.isDuplicated && state.duplicationInfo ? (
                      <span style={{ marginLeft: '8px', color: '#6bdb7d' }}>
                        (duplicated to 0x{state.duplicationInfo.new_pc_offset.toString(16).toUpperCase().padStart(6, '0')})
                      </span>
                    ) : bin.isShared && getSharedWith(bin.shared_with).length > 0 ? (
                      <span style={{ marginLeft: '8px', color: '#ff8888' }}>
                        (shared with {getSharedWith(bin.shared_with).filter(f => f.toLowerCase() !== boxer.name.toLowerCase()).join(', ')})
                      </span>
                    ) : null}
                  </div>

                  {/* Diff strip visualization */}
                  {state.diff.length > 0 && (
                    <div style={{ marginTop: '6px', display: 'flex', flexWrap: 'wrap', gap: '2px' }}>
                      {state.diff.slice(0, 64).map((changed, i) => (
                        <div
                          key={i}
                          title={`Tile ${i}: ${changed ? 'changed' : 'unchanged'}`}
                          style={{
                            width: '8px',
                            height: '8px',
                            borderRadius: '1px',
                            background: changed ? '#ffd700' : 'var(--border)',
                          }}
                        />
                      ))}
                      {state.diff.length > 64 && (
                        <span style={{ fontSize: '0.75rem', color: 'var(--text-dim)', alignSelf: 'center' }}>
                          +{state.diff.length - 64} more
                        </span>
                      )}
                    </div>
                  )}

                  {state.status && (
                    <div style={{ 
                      marginTop: '6px', 
                      fontSize: '0.78rem', 
                      color: state.status.startsWith('✗') ? '#ff6666' : state.status.startsWith('⚠') ? '#ffd700' : '#6bdb7d',
                      fontFamily: 'monospace'
                    }}>
                      {state.status}
                    </div>
                  )}
                </div>

                <div style={{ display: 'flex', gap: '6px', flexShrink: 0 }}>
                  <button
                    id={`export-bin-${bin.start_pc.replace(/0x/, '')}`}
                    onClick={() => handleExport(bin)}
                    disabled={isLoading || !pal}
                    style={{ fontSize: '0.8rem', padding: '6px 10px', opacity: isLoading ? 0.6 : 1 }}
                    title="Export this bin as a PNG tile sheet"
                  >
                    {isLoading ? '⏳' : '↓ PNG'}
                  </button>
                  <button
                    id={`import-bin-${bin.start_pc.replace(/0x/, '')}`}
                    onClick={() => handleImportClick(bin)}
                    disabled={isLoading || duplicatingKey === bin.start_pc || !pal}
                    style={{ 
                      fontSize: '0.8rem', 
                      padding: '6px 10px', 
                      background: (bin.isShared && !state.isDuplicated) ? 'rgba(255,80,80,0.2)' : 'var(--border)',
                      opacity: (isLoading || duplicatingKey === bin.start_pc) ? 0.6 : 1,
                      borderColor: (bin.isShared && !state.isDuplicated) ? 'rgba(255,80,80,0.5)' : undefined,
                      color: (bin.isShared && !state.isDuplicated) ? '#ff8888' : undefined,
                    }}
                    title={(bin.isShared && !state.isDuplicated)
                      ? `⚠️ SHARED: Editing affects ${getSharedWith(bin.shared_with).filter(f => f.toLowerCase() !== boxer.name.toLowerCase()).join(' & ')}!` 
                      : state.isDuplicated
                        ? '✓ This bank is now unique - safe to edit'
                        : 'Import a PNG tile sheet into this bin'
                    }
                  >
                    {duplicatingKey === bin.start_pc ? '⏳ DUP' : isLoading ? '⏳' : (bin.isShared && !state.isDuplicated) ? '⚠ Import' : '↑ PNG'}
                  </button>
                  {state.isEdited && (
                    <button
                      id={`revert-bin-${bin.start_pc.replace(/0x/, '')}`}
                      onClick={() => handleRevert(bin)}
                      style={{ fontSize: '0.8rem', padding: '6px 10px', background: 'rgba(255,80,80,0.15)', color: '#ff8888' }}
                      title="Discard import and revert this bin"
                    >
                      ↩
                    </button>
                  )}
                </div>
              </div>
            </div>
          );
        })}
      </div>

      <div style={{ marginTop: '1rem', fontSize: '0.8rem', color: 'var(--text-dim)' }}>
        * Yellow squares show changed tiles after import. Use "Save ROM As" or "Export IPS" in the Export panel to commit changes.
        {boxer.shared_sprite_bins.length > 0 && (
          <>
            <br />
            <span style={{ color: '#ff8888' }}>
              ⚠️ Red-bordered bins are shared with other fighters. Changes will affect all fighters using that bank.
            </span>
          </>
        )}
      </div>
    </div>
  );
};

export default SpriteBinEditor;
