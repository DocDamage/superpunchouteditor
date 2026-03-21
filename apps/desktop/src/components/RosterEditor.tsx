/**
 * Roster Metadata Editor - Main Component
 * 
 * Provides editing capabilities for game-level roster data:
 * - Boxer names
 * - Circuit assignments
 * - Unlock order
 * - Introductory text
 */

import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useStore, type BoxerRecord as ManifestBoxerRecord } from '../store/useStore';
import type { InGameExpansionReport, InGameHookPreset, InGameHookSiteCandidate } from '../store/useStore';
import { BoxerRosterEntry, CircuitType, ValidationReport, RosterData } from '../types/roster';
import { BoxerNameEditor } from './BoxerNameEditor';
import { CircuitEditor } from './CircuitEditor';
import './RosterEditor.css';
import { showToast } from './ToastContainer';

type TabType = 'create' | 'names' | 'circuits' | 'unlock' | 'intro';
type RosterEditorMode = 'game' | 'dev';

interface RosterEditorProps {
  initialTab?: TabType;
  mode?: RosterEditorMode;
  onLaunchCreatorTest?: (context?: {
    boxerId?: number;
    boxerName?: string;
    circuit?: CircuitType;
    unlockOrder?: number;
    introTextId?: number;
    assetOwnerKey?: string;
  }) => void;
}

function parseOptionalOverwriteLen(input: string): number | null {
  const trimmed = input.trim();
  if (!trimmed) {
    return null;
  }

  const parsed = Number.parseInt(trimmed, 10);
  if (Number.isNaN(parsed) || parsed < 4 || parsed > 32) {
    throw new Error('Hook overwrite length must be a number between 4 and 32.');
  }

  return parsed;
}

const getBoxerId = (boxer: BoxerRosterEntry): number => boxer.boxer_id ?? boxer.fighter_id ?? -1;

export function RosterEditor({ initialTab, mode = 'game', onLaunchCreatorTest }: RosterEditorProps) {
  const isGameMode = mode === 'game';
  const defaultTab = initialTab ?? (isGameMode ? 'create' : 'names');
  const {
    romSha1,
    boxers: manifestBoxers,
    selectedBoxer: selectedManifestBoxer,
    loadBoxers,
    applyInGameExpansion,
    analyzeInGameHookSites,
    verifyInGameHookSite,
    getInGameHookPresets,
  } = useStore();
  const [activeTab, setActiveTab] = useState<TabType>(defaultTab);
  const [rosterData, setRosterData] = useState<RosterData | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [validationReport, setValidationReport] = useState<ValidationReport | null>(null);
  const [saving, setSaving] = useState(false);
  const [targetBoxerCount, setTargetBoxerCount] = useState(24);
  const [patchEditorHook, setPatchEditorHook] = useState(true);
  const [editorHookPcOffset, setEditorHookPcOffset] = useState('');
  const [editorHookOverwriteLen, setEditorHookOverwriteLen] = useState('');
  const [showAdvancedHookControls, setShowAdvancedHookControls] = useState(false);
  const [expansionRunning, setExpansionRunning] = useState(false);
  const [expansionError, setExpansionError] = useState<string | null>(null);
  const [expansionReport, setExpansionReport] = useState<InGameExpansionReport | null>(null);
  const [scanRunning, setScanRunning] = useState(false);
  const [scanError, setScanError] = useState<string | null>(null);
  const [hookCandidates, setHookCandidates] = useState<InGameHookSiteCandidate[]>([]);
  const [verifyRunning, setVerifyRunning] = useState(false);
  const [verifyError, setVerifyError] = useState<string | null>(null);
  const [verifiedHook, setVerifiedHook] = useState<InGameHookSiteCandidate | null>(null);
  const [presetLoading, setPresetLoading] = useState(false);
  const [presetError, setPresetError] = useState<string | null>(null);
  const [hookPresets, setHookPresets] = useState<InGameHookPreset[]>([]);
  const [createName, setCreateName] = useState('');
  const [createCircuit, setCreateCircuit] = useState<CircuitType>('Minor');
  const [createIntro, setCreateIntro] = useState('');
  const [createAssetOwnerKey, setCreateAssetOwnerKey] = useState('');
  const [creatingCharacter, setCreatingCharacter] = useState(false);
  const [createError, setCreateError] = useState<string | null>(null);
  const [createSuccess, setCreateSuccess] = useState<string | null>(null);
  const [lastCreatedBoxer, setLastCreatedBoxer] = useState<{
    boxerId: number;
    boxerName: string;
    circuit: CircuitType;
    unlockOrder: number;
    introTextId: number;
    assetOwnerKey?: string;
  } | null>(null);

  // Load roster data
  const loadRosterData = useCallback(async (): Promise<RosterData> => {
    try {
      setLoading(true);
      setError(null);
      
      const data = await invoke<RosterData>('get_roster_data');
      setRosterData(data);
      
      // Also run validation
      const report = await invoke<ValidationReport>('validate_roster_changes');
      setValidationReport(report);
      return data;
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Failed to load roster data';
      setError(message);
      throw new Error(message);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadRosterData();
  }, [loadRosterData]);

  useEffect(() => {
    if (!rosterData) return;
    setTargetBoxerCount((current) => Math.max(current, rosterData.boxers.length, 16));
  }, [rosterData]);

  useEffect(() => {
    setActiveTab(defaultTab);
  }, [defaultTab]);

  useEffect(() => {
    if (createAssetOwnerKey) {
      return;
    }

    if (selectedManifestBoxer?.key) {
      setCreateAssetOwnerKey(selectedManifestBoxer.key);
      return;
    }

    if (manifestBoxers.length > 0) {
      setCreateAssetOwnerKey(manifestBoxers[0].key);
    }
  }, [createAssetOwnerKey, manifestBoxers, selectedManifestBoxer?.key]);

  // Handle boxer name update
  const handleNameUpdate = async (fighterId: number, newName: string) => {
    try {
      setSaving(true);
      await invoke<BoxerRosterEntry>('update_boxer_name', {
        fighterId,
        newName,
      });
      
      // Reload roster data
      await loadRosterData();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to update name');
    } finally {
      setSaving(false);
    }
  };

  // Handle circuit update
  const handleCircuitUpdate = async (fighterId: number, circuit: string) => {
    try {
      setSaving(true);
      await invoke<RosterData>('update_boxer_circuit', {
        fighterId,
        circuit,
      });
      
      await loadRosterData();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to update circuit');
    } finally {
      setSaving(false);
    }
  };

  // Handle unlock order update
  const handleUnlockOrderUpdate = async (fighterId: number, order: number) => {
    try {
      setSaving(true);
      await invoke<BoxerRosterEntry>('update_unlock_order', {
        fighterId,
        order,
      });
      
      await loadRosterData();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to update unlock order');
    } finally {
      setSaving(false);
    }
  };

  // Handle champion status update
  const handleChampionUpdate = async (fighterId: number, isChampion: boolean) => {
    try {
      setSaving(true);
      await invoke<BoxerRosterEntry>('set_champion_status', {
        fighterId,
        isChampion,
      });
      
      await loadRosterData();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to update champion status');
    } finally {
      setSaving(false);
    }
  };

  // Handle reset to defaults
  const handleReset = async () => {
    if (!confirm('Reset all roster data to defaults? This will discard any changes.')) {
      return;
    }
    
    try {
      setSaving(true);
      await invoke<RosterData>('reset_roster_to_defaults');
      await loadRosterData();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to reset roster');
    } finally {
      setSaving(false);
    }
  };

  const handleCreateCharacter = async () => {
    if (!rosterData) {
      setCreateError('Roster data is not loaded yet.');
      return;
    }

    const trimmedName = createName.trim();
    if (!trimmedName) {
      setCreateError('Character name is required.');
      return;
    }

    const currentCount = rosterData.boxers.length;
    const targetCount = Math.min(64, currentCount + 1);
    if (targetCount <= currentCount) {
      setCreateError('Roster is already at maximum size (64).');
      return;
    }

    const existingIds = new Set(rosterData.boxers.map(getBoxerId));

    try {
      setCreatingCharacter(true);
      setCreateError(null);
      setCreateSuccess(null);

      const expansion = await applyInGameExpansion({
        targetBoxerCount: targetCount,
        patchEditorHook: true,
        editorHookPcOffset: null,
        editorHookOverwriteLen: null,
      });
      setExpansionReport(expansion);
      setTargetBoxerCount(expansion.boxer_count);

      const expanded = await loadRosterData();
      const created =
        expanded.boxers.find((boxer) => !existingIds.has(getBoxerId(boxer))) ??
        [...expanded.boxers].sort((a, b) => getBoxerId(b) - getBoxerId(a))[0];

      if (!created) {
        throw new Error('Failed to locate created boxer slot after expansion.');
      }

      const createdId = getBoxerId(created);
      const nextUnlockOrder = expanded.boxers.reduce(
        (maxOrder, boxer) => Math.max(maxOrder, boxer.unlock_order),
        0
      ) + 1;

      await invoke<BoxerRosterEntry>('update_boxer_name', {
        fighterId: createdId,
        newName: trimmedName,
      });

      await invoke<RosterData>('update_boxer_circuit', {
        fighterId: createdId,
        circuit: createCircuit,
      });

      await invoke<BoxerRosterEntry>('update_unlock_order', {
        fighterId: createdId,
        order: Math.min(nextUnlockOrder, 255),
      });

      if (createIntro.trim()) {
        await invoke('update_intro_text', {
          textId: created.intro_text_id,
          text: createIntro.trim(),
        });
      }

      let resolvedAssetOwnerKey = createAssetOwnerKey || undefined;
      if (createAssetOwnerKey) {
        const createdAssetOwner = await invoke<ManifestBoxerRecord>('create_boxer_asset_owner', {
          templateBoxerKey: createAssetOwnerKey,
          ownerDisplayName: trimmedName,
          preferredKey: `creator_asset_slot_${createdId}`,
        });
        resolvedAssetOwnerKey = createdAssetOwner.key;
        await loadBoxers();
      }

      await loadRosterData();
      setLastCreatedBoxer({
        boxerId: createdId,
        boxerName: trimmedName,
        circuit: createCircuit,
        unlockOrder: Math.min(nextUnlockOrder, 255),
        introTextId: created.intro_text_id,
        assetOwnerKey: resolvedAssetOwnerKey,
      });
      setCreateSuccess(
        `Character '${trimmedName}' created in slot #${createdId}. Dedicated portrait asset owner is ${resolvedAssetOwnerKey || 'not set'}. In-ROM creator mode trigger is available via Select+Start+L+R.`
      );
      setCreateName('');
      setCreateIntro('');
      setActiveTab(isGameMode ? 'create' : 'names');
    } catch (err) {
      setCreateError(err instanceof Error ? err.message : String(err));
    } finally {
      setCreatingCharacter(false);
    }
  };

  const handleOneClickIntegration = async () => {
    const normalizedTarget = Math.min(64, Math.max(16, Math.trunc(targetBoxerCount || 16)));

    try {
      setExpansionRunning(true);
      setExpansionError(null);
      setVerifyError(null);
      setCreateError(null);
      setCreateSuccess(null);

      const report = await applyInGameExpansion({
        targetBoxerCount: normalizedTarget,
        patchEditorHook: true,
        editorHookPcOffset: null,
        editorHookOverwriteLen: null,
      });

      setPatchEditorHook(true);
      setEditorHookPcOffset('');
      setEditorHookOverwriteLen('');
      setVerifiedHook(null);
      setExpansionReport(report);
      setTargetBoxerCount(report.boxer_count);
      setLastCreatedBoxer(null);
      await loadRosterData();
    } catch (err) {
      setExpansionError(err instanceof Error ? err.message : String(err));
    } finally {
      setExpansionRunning(false);
    }
  };

  const handleApplyExpansion = async () => {
    const normalizedTarget = Math.min(64, Math.max(16, Math.trunc(targetBoxerCount || 16)));
    let parsedOverwriteLen: number | null = null;
    try {
      parsedOverwriteLen = parseOptionalOverwriteLen(editorHookOverwriteLen);
    } catch (err) {
      setExpansionError(err instanceof Error ? err.message : String(err));
      return;
    }
    const hasManualHook = patchEditorHook && editorHookPcOffset.trim().length > 0;

    try {
      setExpansionRunning(true);
      setExpansionError(null);
      setVerifyError(null);

      let verifiedOverwriteLen = parsedOverwriteLen;
      if (hasManualHook) {
        setVerifyRunning(true);
        const verified = await verifyInGameHookSite({
          hookPcOffset: editorHookPcOffset.trim(),
          overwriteLen: parsedOverwriteLen,
        });
        setVerifiedHook(verified);
        verifiedOverwriteLen = verified.overwrite_len;
        setEditorHookOverwriteLen(String(verified.overwrite_len));
      }

      const report = await applyInGameExpansion({
        targetBoxerCount: normalizedTarget,
        patchEditorHook,
        editorHookPcOffset: hasManualHook ? editorHookPcOffset.trim() : null,
        editorHookOverwriteLen: patchEditorHook ? verifiedOverwriteLen : null,
      });

      setExpansionReport(report);
      setTargetBoxerCount(report.boxer_count);
      await loadRosterData();
    } catch (err) {
      setExpansionError(err instanceof Error ? err.message : String(err));
    } finally {
      setVerifyRunning(false);
      setExpansionRunning(false);
    }
  };

  const handleScanHookCandidates = async () => {
    try {
      setScanRunning(true);
      setScanError(null);
      const candidates = await analyzeInGameHookSites({ limit: 25 });
      setHookCandidates(candidates);
      if (candidates.length === 0) {
        setScanError('No safe hook candidates were found in the scanned range.');
      }
    } catch (err) {
      setScanError(err instanceof Error ? err.message : String(err));
    } finally {
      setScanRunning(false);
    }
  };

  const handleLoadHookPresets = async () => {
    try {
      setPresetLoading(true);
      setPresetError(null);
      const presets = await getInGameHookPresets(8);
      setHookPresets(presets);
      if (presets.length === 0) {
        setPresetError('No verified presets were found for this ROM.');
      }
    } catch (err) {
      setPresetError(err instanceof Error ? err.message : String(err));
    } finally {
      setPresetLoading(false);
    }
  };

  const handleVerifyHook = async () => {
    let parsedOverwriteLen: number | null = null;
    try {
      parsedOverwriteLen = parseOptionalOverwriteLen(editorHookOverwriteLen);
    } catch (err) {
      setVerifyError(err instanceof Error ? err.message : String(err));
      return;
    }

    if (!editorHookPcOffset.trim()) {
      setVerifyError('Provide a hook PC offset before verification.');
      return;
    }

    try {
      setVerifyRunning(true);
      setVerifyError(null);
      const verified = await verifyInGameHookSite({
        hookPcOffset: editorHookPcOffset.trim(),
        overwriteLen: parsedOverwriteLen,
      });
      setVerifiedHook(verified);
      setEditorHookOverwriteLen(String(verified.overwrite_len));
      setPatchEditorHook(true);
    } catch (err) {
      setVerifyError(err instanceof Error ? err.message : String(err));
    } finally {
      setVerifyRunning(false);
    }
  };

  const useHookCandidate = (candidate: InGameHookSiteCandidate) => {
    setPatchEditorHook(true);
    setEditorHookPcOffset(candidate.hook_pc);
    setEditorHookOverwriteLen(String(candidate.overwrite_len));
    setVerifiedHook(candidate);
    setVerifyError(null);
    setScanError(null);
  };

  const useHookPreset = (preset: InGameHookPreset) => {
    useHookCandidate(preset);
    setPresetError(null);
  };

  // Render validation badge
  const renderValidationBadge = () => {
    if (!validationReport) return null;
    
    const hasErrors = validationReport.errors.length > 0;
    const hasWarnings = validationReport.warnings.length > 0;
    
    if (!hasErrors && !hasWarnings) {
      return (
        <span className="validation-badge valid">
          Valid
        </span>
      );
    }
    
    return (
      <span className={`validation-badge ${hasErrors ? 'error' : 'warning'}`}>
        {hasErrors 
          ? `${validationReport.errors.length} error(s)` 
          : `${validationReport.warnings.length} warning(s)`}
      </span>
    );
  };

  // Render tab content
  const renderTabContent = () => {
    if (!rosterData) return null;
    
    switch (activeTab) {
      case 'create':
        return (
          <div className="character-create-panel">
            <div className="editor-help">
              <p>
                Create a new boxer directly in the ROM. The game expansion and hook integration run automatically in the background.
              </p>
            </div>

            <div className="character-create-grid">
              <label className="expansion-field">
                <span>Character Name</span>
                <input
                  type="text"
                  value={createName}
                  onChange={(e) => {
                    setCreateName(e.target.value);
                    setCreateError(null);
                    setCreateSuccess(null);
                  }}
                  maxLength={24}
                  placeholder="Enter boxer name"
                  disabled={saving || creatingCharacter}
                />
              </label>

              <label className="expansion-field">
                <span>Circuit</span>
                <select
                  value={createCircuit}
                  onChange={(e) => {
                    setCreateCircuit(e.target.value as CircuitType);
                    setCreateError(null);
                    setCreateSuccess(null);
                  }}
                  disabled={saving || creatingCharacter}
                >
                  <option value="Minor">Minor</option>
                  <option value="Major">Major</option>
                  <option value="World">World</option>
                  <option value="Special">Special</option>
                </select>
              </label>
            </div>

            <label className="expansion-field">
              <span>Intro Quote (optional)</span>
              <textarea
                value={createIntro}
                onChange={(e) => {
                  setCreateIntro(e.target.value);
                  setCreateError(null);
                  setCreateSuccess(null);
                }}
                placeholder="Enter pre-match quote"
                rows={4}
                disabled={saving || creatingCharacter}
              />
            </label>

            <label className="expansion-field">
              <span>Portrait Asset Template</span>
              <select
                value={createAssetOwnerKey}
                onChange={(e) => {
                  setCreateAssetOwnerKey(e.target.value);
                  setCreateError(null);
                  setCreateSuccess(null);
                }}
                disabled={saving || creatingCharacter}
              >
                <option value="">Select asset template</option>
                {manifestBoxers.map((boxer) => (
                  <option key={boxer.key} value={boxer.key}>
                    {boxer.name}
                  </option>
                ))}
              </select>
              <small style={{ color: 'var(--text-muted, #94a3b8)' }}>
                New roster slots now clone their own palette and portrait/icon ownership. This picks the manifest boxer used as the template source for that dedicated asset owner.
              </small>
            </label>

            <div className="roster-expansion-actions">
              <button
                className="btn-secondary"
                onClick={handleOneClickIntegration}
                disabled={saving || creatingCharacter || expansionRunning || verifyRunning}
              >
                {expansionRunning ? 'Installing Creator Integration...' : 'Install Creator Integration'}
              </button>
              <button
                className="btn-primary"
                onClick={handleCreateCharacter}
                disabled={saving || creatingCharacter || expansionRunning || verifyRunning}
              >
                {creatingCharacter ? 'Creating Character...' : 'Create Character'}
              </button>
              <button
                className="btn-secondary"
                onClick={() => onLaunchCreatorTest?.(lastCreatedBoxer ?? undefined)}
                disabled={saving || creatingCharacter || expansionRunning || verifyRunning || !onLaunchCreatorTest}
                title={lastCreatedBoxer ? `Open creator test for ${lastCreatedBoxer.boxerName}` : 'Open embedded creator test session'}
              >
                {lastCreatedBoxer ? `Test ${lastCreatedBoxer.boxerName} In Creator` : 'Open Creator Test'}
              </button>
            </div>

            {createError && <div className="roster-expansion-error">{createError}</div>}
            {createSuccess && <div className="roster-create-success">{createSuccess}</div>}
          </div>
        );

      case 'names':
        return (
          <BoxerNameEditor
            boxers={rosterData.boxers}
            onUpdateName={handleNameUpdate}
            disabled={saving}
          />
        );
        
      case 'circuits':
        return (
          <CircuitEditor
            boxers={rosterData.boxers}
            circuits={rosterData.circuits}
            onUpdateCircuit={handleCircuitUpdate}
            onUpdateChampion={handleChampionUpdate}
            disabled={saving}
          />
        );
        
      case 'unlock':
        return (
          <UnlockOrderEditor
            boxers={rosterData.boxers}
            onUpdateOrder={handleUnlockOrderUpdate}
            onUpdateChampion={handleChampionUpdate}
            disabled={saving}
          />
        );
        
      case 'intro':
        return (
          <IntroTextEditor
            boxers={rosterData.boxers}
            disabled={saving}
          />
        );
        
      default:
        return null;
    }
  };

  if (!romSha1) {
    return (
      <div className="roster-editor-empty">
        <h3>{isGameMode ? 'Character Creator' : 'Roster Metadata Editor'}</h3>
        <p>{isGameMode ? 'Open a ROM to start creating a character.' : 'Open a ROM to edit roster data.'}</p>
      </div>
    );
  }

  if (loading) {
    return (
      <div className="roster-editor-loading">
        <div className="spinner" />
        <p>Loading roster data...</p>
      </div>
    );
  }

  return (
    <div className="roster-editor">
      <div className="roster-editor-header">
        <div className="roster-editor-title">
          <h3>{isGameMode ? 'Character Creator' : 'Roster Metadata Editor'}</h3>
          {!isGameMode && renderValidationBadge()}
        </div>
        
        <div className="roster-editor-actions">
          {!isGameMode && (
            <button 
              onClick={handleReset}
              disabled={saving}
              className="btn-secondary"
            >
              Reset to Defaults
            </button>
          )}
          <button 
            onClick={loadRosterData}
            disabled={saving}
            className="btn-secondary"
          >
            Refresh
          </button>
        </div>
      </div>
      
      {error && (
        <div className="roster-editor-error">
          <span className="error-icon">!</span>
          {error}
        </div>
      )}

      {!isGameMode && (
      <div className="roster-expansion-panel">
        <div className="roster-expansion-header">
          <h4>In-Game Integration</h4>
          <span className="roster-expansion-badge">Auto</span>
        </div>

        <p className="roster-expansion-help">
          Set your target roster size and apply. Hook placement and safety checks are resolved behind the scenes.
        </p>

        <div className="roster-expansion-controls">
          <label className="expansion-field">
            <span>Target Boxer Count</span>
            <input
              type="number"
              min={16}
              max={64}
              step={1}
              value={targetBoxerCount}
              onChange={(e) => setTargetBoxerCount(parseInt(e.target.value || '16', 10))}
              disabled={saving || expansionRunning}
            />
          </label>
        </div>

        <p className="roster-expansion-help">
          Hook integration is automatically resolved by region presets and safety-checked scans.
        </p>

        <div className="roster-expansion-actions">
          <button
            className="btn-primary"
            onClick={handleOneClickIntegration}
            disabled={saving || expansionRunning || verifyRunning}
          >
            {expansionRunning ? 'Installing Integration...' : 'One-Click Install In-Game Integration'}
          </button>
          {showAdvancedHookControls && (
            <button
              className="btn-secondary"
              onClick={handleApplyExpansion}
              disabled={saving || expansionRunning || verifyRunning}
            >
              {expansionRunning ? 'Applying Technical Settings...' : 'Apply Technical Settings'}
            </button>
          )}
          <button
            className="btn-secondary"
            onClick={() => setShowAdvancedHookControls((value) => !value)}
            disabled={saving || expansionRunning || verifyRunning}
          >
            {showAdvancedHookControls ? 'Hide Technical Controls' : 'Show Technical Controls'}
          </button>
        </div>

        {expansionError && (
          <div className="roster-expansion-error">{expansionError}</div>
        )}

        {showAdvancedHookControls && (
          <>
            <div className="roster-expansion-controls">
              <label className="expansion-checkbox">
                <input
                  type="checkbox"
                  checked={patchEditorHook}
                  onChange={(e) => {
                    const enabled = e.target.checked;
                    setPatchEditorHook(enabled);
                    if (!enabled) {
                      setVerifiedHook(null);
                      setVerifyError(null);
                    }
                  }}
                  disabled={saving || expansionRunning}
                />
                <span>Patch editor hook (JML)</span>
              </label>

              <label className="expansion-field">
                <span>Hook PC Offset (blank = automatic)</span>
                <input
                  type="text"
                  value={editorHookPcOffset}
                  onChange={(e) => {
                    setEditorHookPcOffset(e.target.value);
                    setVerifiedHook(null);
                    setVerifyError(null);
                  }}
                  placeholder="auto (or 0x123456)"
                  disabled={saving || expansionRunning || !patchEditorHook}
                />
              </label>

              <label className="expansion-field">
                <span>Hook Overwrite Length (optional)</span>
                <input
                  type="number"
                  min={4}
                  max={32}
                  step={1}
                  value={editorHookOverwriteLen}
                  onChange={(e) => {
                    setEditorHookOverwriteLen(e.target.value);
                    setVerifiedHook(null);
                    setVerifyError(null);
                  }}
                  placeholder="auto"
                  disabled={saving || expansionRunning || !patchEditorHook}
                />
              </label>
            </div>

            <div className="roster-expansion-actions">
              <button
                className="btn-secondary"
                onClick={handleLoadHookPresets}
                disabled={saving || expansionRunning || verifyRunning || presetLoading}
              >
                {presetLoading ? 'Loading Presets...' : 'Load Region Presets'}
              </button>
              <button
                className="btn-secondary"
                onClick={handleVerifyHook}
                disabled={saving || expansionRunning || verifyRunning || !patchEditorHook}
              >
                {verifyRunning ? 'Verifying Hook...' : 'Verify Hook'}
              </button>
              <button
                className="btn-secondary"
                onClick={handleScanHookCandidates}
                disabled={saving || expansionRunning || verifyRunning || scanRunning}
              >
                {scanRunning ? 'Scanning Hook Sites...' : 'Find Safe Hook Sites'}
              </button>
            </div>

            {presetError && (
              <div className="roster-expansion-error">{presetError}</div>
            )}

            {hookPresets.length > 0 && (
              <div className="roster-hook-candidates">
                <div className="roster-hook-candidates-header">Region Presets</div>
                <div className="roster-hook-candidates-list">
                  {hookPresets.map((preset) => (
                    <div className="hook-candidate-row" key={preset.id}>
                      <div className="hook-candidate-cell">
                        <strong>{preset.label}</strong>
                      </div>
                      <div className="hook-candidate-cell">
                        <strong>PC:</strong> <code>{preset.hook_pc}</code>
                      </div>
                      <div className="hook-candidate-cell">
                        <strong>Len:</strong> {preset.overwrite_len}
                      </div>
                      <div className="hook-candidate-cell hook-candidate-instr">
                        <strong>{preset.region}</strong> {preset.source}
                      </div>
                      <button
                        className="btn-secondary"
                        onClick={() => useHookPreset(preset)}
                        disabled={saving || expansionRunning}
                      >
                        Use
                      </button>
                    </div>
                  ))}
                </div>
              </div>
            )}

            {verifyError && (
              <div className="roster-expansion-error">{verifyError}</div>
            )}

            {verifiedHook && (
              <div className="roster-expansion-report">
                <div className="expansion-report-row">
                  <strong>Verified Hook:</strong> <code>{verifiedHook.hook_pc}</code>
                </div>
                <div className="expansion-report-row">
                  <strong>Overwrite:</strong> {verifiedHook.overwrite_len} byte(s)
                </div>
                <div className="expansion-report-row">
                  <strong>Return:</strong> <code>{verifiedHook.return_pc}</code>
                </div>
                <div className="expansion-report-row">
                  <strong>Instruction:</strong> {verifiedHook.first_instruction || '(unknown)'}
                </div>
              </div>
            )}

            {scanError && (
              <div className="roster-expansion-error">{scanError}</div>
            )}

            {hookCandidates.length > 0 && (
              <div className="roster-hook-candidates">
                <div className="roster-hook-candidates-header">Hook Candidates</div>
                <div className="roster-hook-candidates-list">
                  {hookCandidates.map((candidate) => (
                    <div className="hook-candidate-row" key={`${candidate.hook_pc}-${candidate.overwrite_len}`}>
                      <div className="hook-candidate-cell">
                        <strong>PC:</strong> <code>{candidate.hook_pc}</code>
                      </div>
                      <div className="hook-candidate-cell">
                        <strong>Len:</strong> {candidate.overwrite_len}
                      </div>
                      <div className="hook-candidate-cell hook-candidate-instr">
                        <strong>Instr:</strong> {candidate.first_instruction || '(unknown)'}
                      </div>
                      <div className="hook-candidate-cell hook-candidate-bytes">
                        <code>{candidate.preview_bytes_hex}</code>
                      </div>
                      <button
                        className="btn-secondary"
                        onClick={() => useHookCandidate(candidate)}
                        disabled={saving || expansionRunning}
                      >
                        Use
                      </button>
                    </div>
                  ))}
                </div>
              </div>
            )}
          </>
        )}

        {expansionReport && (
          <div className="roster-expansion-report">
            <div className="expansion-report-row">
              <strong>Integration Applied:</strong> roster capacity set to {expansionReport.boxer_count} boxers.
            </div>
            <div className="expansion-report-row">
              <strong>Hook status:</strong> {expansionReport.editor_hook_patched ? 'Installed' : 'Not installed'}
            </div>

            {showAdvancedHookControls && (
              <>
                <div className="expansion-report-row">
                  <strong>Header:</strong> <code>{expansionReport.header_pc}</code>
                </div>
                <div className="expansion-report-row">
                  <strong>Stub:</strong> <code>{expansionReport.editor_stub_pc}</code>
                </div>
                <div className="expansion-report-row">
                  <strong>Hook overwrite:</strong> {expansionReport.editor_hook_overwrite_len} byte(s)
                </div>
                <div className="expansion-report-row">
                  <strong>Tables:</strong>{' '}
                  <code>{expansionReport.name_pointer_table_pc}</code>{' '}
                  <code>{expansionReport.name_long_pointer_table_pc}</code>{' '}
                  <code>{expansionReport.name_blob_pc}</code>{' '}
                  <code>{expansionReport.circuit_table_pc}</code>{' '}
                  <code>{expansionReport.unlock_table_pc}</code>{' '}
                  <code>{expansionReport.intro_table_pc}</code>
                </div>
              </>
            )}

            {expansionReport.notes.length > 0 && (
              <ul className="expansion-notes">
                {expansionReport.notes.map((note, index) => (
                  <li key={`${note}-${index}`}>{note}</li>
                ))}
              </ul>
            )}
          </div>
        )}
      </div>
      )}
      
      {!isGameMode && (
        <div className="roster-editor-tabs">
          <button
            className={`tab-btn ${activeTab === 'create' ? 'active' : ''}`}
            onClick={() => setActiveTab('create')}
          >
            Character Create
          </button>
          <button
            className={`tab-btn ${activeTab === 'names' ? 'active' : ''}`}
            onClick={() => setActiveTab('names')}
          >
            Boxer Names
          </button>
          <button
            className={`tab-btn ${activeTab === 'circuits' ? 'active' : ''}`}
            onClick={() => setActiveTab('circuits')}
          >
            Circuits
          </button>
          <button
            className={`tab-btn ${activeTab === 'unlock' ? 'active' : ''}`}
            onClick={() => setActiveTab('unlock')}
          >
            Unlock Order
          </button>
          <button
            className={`tab-btn ${activeTab === 'intro' ? 'active' : ''}`}
            onClick={() => setActiveTab('intro')}
          >
            Intro Text
          </button>
        </div>
      )}
      
      <div className="roster-editor-content">
        {renderTabContent()}
      </div>
      
      {saving && (
        <div className="roster-editor-saving">
          <div className="spinner-small" />
          <span>Saving...</span>
        </div>
      )}
    </div>
  );
}

// ============================================================================
// Unlock Order Editor Sub-component
// ============================================================================

interface UnlockOrderEditorProps {
  boxers: BoxerRosterEntry[];
  onUpdateOrder: (fighterId: number, order: number) => void;
  onUpdateChampion: (fighterId: number, isChampion: boolean) => void;
  disabled?: boolean;
}

function UnlockOrderEditor({ 
  boxers, 
  onUpdateOrder, 
  onUpdateChampion,
  disabled 
}: UnlockOrderEditorProps) {
  const sortedBoxers = [...boxers].sort((a, b) => a.unlock_order - b.unlock_order);
  const maxUnlockOrder = Math.max(0, boxers.length - 1);
  
  return (
    <div className="unlock-order-editor">
      <div className="editor-help">
        <p>
          Drag boxers to change unlock order, or edit the order numbers directly.
          The starting boxer should have order 0.
        </p>
      </div>
      
      <div className="unlock-list">
        {sortedBoxers.map((boxer, index) => (
          <div 
            key={getBoxerId(boxer)}
            className={`unlock-item ${boxer.is_champion ? 'champion' : ''}`}
          >
            <div className="unlock-rank">{index + 1}</div>
            
            <div className="unlock-info">
              <span className="boxer-name">{boxer.name}</span>
              <span className={`circuit-badge circuit-${boxer.circuit.toLowerCase()}`}>
                {boxer.circuit}
              </span>
            </div>
            
            <div className="unlock-controls">
              <input
                type="number"
                min={0}
                max={maxUnlockOrder}
                value={boxer.unlock_order}
                onChange={(e) => onUpdateOrder(getBoxerId(boxer), parseInt(e.target.value))}
                disabled={disabled}
                className="order-input"
              />
              
              <label className="champion-toggle">
                <input
                  type="checkbox"
                  checked={boxer.is_champion}
                  onChange={(e) => onUpdateChampion(getBoxerId(boxer), e.target.checked)}
                  disabled={disabled}
                />
                <span>Champion</span>
              </label>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

// ============================================================================
// Intro Text Editor Sub-component
// ============================================================================

interface IntroTextEditorProps {
  boxers: BoxerRosterEntry[];
  disabled?: boolean;
}

function IntroTextEditor({ boxers, disabled }: IntroTextEditorProps) {
  const [selectedBoxer, setSelectedBoxer] = useState<BoxerRosterEntry | null>(null);
  const [introText, setIntroText] = useState('');
  const [loading, setLoading] = useState(false);
  const [saving, setSaving] = useState(false);
  const [validation, setValidation] = useState<{
    valid: boolean;
    encoded_length: number;
    max_length: number;
    error?: string;
  } | null>(null);

  // Load intro text when boxer is selected
  useEffect(() => {
    if (!selectedBoxer) {
      setIntroText('');
      return;
    }

    const loadIntroText = async () => {
      try {
        setLoading(true);
        const text = await invoke<{
          text_id: number;
          text: string;
          fighter_id: number;
        }>('get_intro_text', { textId: selectedBoxer.intro_text_id });
        setIntroText(text.text);
      } catch (err) {
        console.error('Failed to load intro text:', err);
        setIntroText('');
      } finally {
        setLoading(false);
      }
    };

    loadIntroText();
  }, [selectedBoxer]);

  // Validate text on change
  useEffect(() => {
    const validate = async () => {
      if (!introText) {
        setValidation(null);
        return;
      }

      try {
        const result = await invoke<{
          valid: boolean;
          encoded_length: number;
          max_length: number;
          error?: string;
        }>('validate_intro_text', { text: introText });
        setValidation(result);
      } catch (err) {
        console.error('Validation error:', err);
      }
    };

    validate();
  }, [introText]);

  const handleSave = async () => {
    if (!selectedBoxer || !validation?.valid) return;

    try {
      setSaving(true);
      await invoke('update_intro_text', {
        textId: selectedBoxer.intro_text_id,
        text: introText,
      });
      showToast('Intro text saved.', 'success');
    } catch (err) {
      showToast('Failed to save intro text: ' + (err instanceof Error ? err.message : String(err)), 'error');
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="intro-text-editor">
      <div className="editor-help">
        <p>
          Select a boxer to edit their pre-match introduction text.
          Keep text concise to fit within ROM space.
        </p>
      </div>
      
      <div className="intro-editor-layout">
        <div className="boxer-selector">
          <h4>Select Boxer</h4>
          <div className="boxer-list">
            {boxers.map(boxer => (
              <button
                key={getBoxerId(boxer)}
                className={`boxer-btn ${selectedBoxer && getBoxerId(selectedBoxer) === getBoxerId(boxer) ? 'active' : ''}`}
                onClick={() => setSelectedBoxer(boxer)}
                disabled={disabled}
              >
                <span className="boxer-name">{boxer.name}</span>
                <span className="circuit-badge">{boxer.circuit}</span>
              </button>
            ))}
          </div>
        </div>
        
        <div className="text-editor">
          {selectedBoxer ? (
            <>
              <div className="editor-header">
                <h4>Intro Text: {selectedBoxer.name}</h4>
                {loading && <span className="loading-indicator">Loading...</span>}
              </div>
              
              <textarea
                value={introText}
                onChange={(e) => setIntroText(e.target.value)}
                disabled={disabled || loading}
                rows={8}
                placeholder="Enter intro text here..."
                className={`intro-textarea ${validation && !validation.valid ? 'invalid' : ''}`}
              />
              
              {validation && (
                <div className={`validation-status ${validation.valid ? 'valid' : 'invalid'}`}>
                  <span className="char-count">
                    {validation.encoded_length} / {validation.max_length} bytes
                  </span>
                  {validation.error && (
                    <span className="error-message">{validation.error}</span>
                  )}
                </div>
              )}
              
              <div className="editor-actions">
                <button
                  onClick={handleSave}
                  disabled={disabled || saving || !validation?.valid}
                  className="btn-primary"
                >
                  {saving ? 'Saving...' : 'Save Intro Text'}
                </button>
              </div>
            </>
          ) : (
            <div className="no-selection">
              <p>Select a boxer from the list to edit their intro text.</p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

// Use invoke from @tauri-apps/api/core instead of custom implementation

// Add type declaration for window.__TAURI__
declare global {
  interface Window {
    __TAURI__: {
      invoke: <T>(cmd: string, args?: Record<string, unknown>) => Promise<T>;
    };
  }
}
