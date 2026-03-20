import React, { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useStore, EditableFighterParams, ParamValidationResult } from '../store/useStore';

export interface ScriptRecord {
  label: string;
  bank: number;
  snes_addr: number;
  pc_offset: number;
  preview_bytes: number[];
  category: 'AnimationScript' | 'AiScript' | 'SpriteScript' | 'CornerManScript' | 'PlayerScript' | 'Unknown';
  risk: 'Low' | 'Medium' | 'High';
  description: string;
  is_shared: boolean;
  owners: string[];
}

export interface FighterHeader {
  pc_offset: number;
  snes_bank: number;
  snes_addr: number;
  raw_bytes: number[];
  palette_id: number;
  attack_power: number;
  defense_rating: number;
  speed_rating: number;
  pose_table_ptr: number;
  ai_script_ptr: number;
  corner_man_ptr: number;
}

const categoryColors: Record<ScriptRecord['category'], string> = {
  AnimationScript: '#8b5cf6', // violet
  AiScript: '#ef4444', // red
  SpriteScript: '#22c55e', // green
  CornerManScript: '#f59e0b', // amber
  PlayerScript: '#3b82f6', // blue
  Unknown: '#6b7280', // gray
};

const categoryLabels: Record<ScriptRecord['category'], string> = {
  AnimationScript: 'Animation',
  AiScript: 'AI Behavior',
  SpriteScript: 'Sprite',
  CornerManScript: 'Corner Man',
  PlayerScript: 'Player',
  Unknown: 'Unknown',
};

const riskColors: Record<ScriptRecord['risk'], string> = {
  Low: '#22c55e',
  Medium: '#f59e0b',
  High: '#ef4444',
};

export const ScriptViewer: React.FC = () => {
  const { boxers, selectedBoxer, setPendingWrite } = useStore();
  const [scripts, setScripts] = useState<ScriptRecord[]>([]);
  const [header, setHeader] = useState<FighterHeader | null>(null);
  const [loading, setLoading] = useState(false);
  const [selectedScript, setSelectedScript] = useState<ScriptRecord | null>(null);
  const [filter, setFilter] = useState<ScriptRecord['category'] | 'All'>('All');

  // Edit mode state
  const [isEditMode, setIsEditMode] = useState(false);
  const [editParams, setEditParams] = useState<EditableFighterParams | null>(null);
  const [validationResult, setValidationResult] = useState<ParamValidationResult | null>(null);
  const [isSaving, setIsSaving] = useState(false);
  const [saveError, setSaveError] = useState<string | null>(null);
  const [showConfirmDialog, setShowConfirmDialog] = useState(false);
  const [saveSuccess, setSaveSuccess] = useState(false);

  // Fighter index mapping based on ROM structure
  const getFighterIndex = (boxerName: string): number => {
    const mapping: Record<string, number> = {
      'Gabby Jay': 0,
      'Bear Hugger': 1,
      'Piston Hurricane': 2,
      'Bald Bull': 3,
      'Bob Charlie': 4,
      'Dragon Chan': 5,
      'Masked Muscle': 6,
      'Mr. Sandman': 7,
      'Aran Ryan': 8,
      'Heike Kagero': 9,
      'Mad Clown': 10,
      'Super Macho Man': 11,
      'Narcis Prince': 12,
      'Hoy Quarlow': 13,
      'Rick Bruiser': 14,
      'Nick Bruiser': 15,
    };
    return mapping[boxerName] ?? -1;
  };

  useEffect(() => {
    if (selectedBoxer) {
      loadScriptsForBoxer(selectedBoxer.name);
    } else {
      loadAllScripts();
    }
    // Reset edit mode when boxer changes
    setIsEditMode(false);
    setEditParams(null);
    setValidationResult(null);
    setSaveError(null);
  }, [selectedBoxer]);

  // Validate params whenever they change
  useEffect(() => {
    if (editParams && isEditMode) {
      validateParams(editParams);
    }
  }, [editParams]);

  const loadAllScripts = async () => {
    setLoading(true);
    try {
      const allScripts = await invoke<ScriptRecord[]>('get_all_scripts');
      setScripts(allScripts);
      setHeader(null);
    } catch (e) {
      console.error('Failed to load scripts:', e);
    } finally {
      setLoading(false);
    }
  };

  const loadScriptsForBoxer = async (fighterName: string) => {
    setLoading(true);
    try {
      const [fighterScripts, fighterHeader] = await Promise.all([
        invoke<ScriptRecord[]>('get_scripts_for_fighter', { fighterName }),
        invoke<FighterHeader>('get_fighter_header', { 
          fighterIndex: getFighterIndex(fighterName) 
        }).catch(() => null),
      ]);
      setScripts(fighterScripts);
      setHeader(fighterHeader);
    } catch (e) {
      console.error('Failed to load fighter scripts:', e);
    } finally {
      setLoading(false);
    }
  };

  const validateParams = async (params: EditableFighterParams) => {
    try {
      const result = await invoke<ParamValidationResult>('validate_fighter_params', { params });
      setValidationResult(result);
    } catch (e) {
      console.error('Validation failed:', e);
      setValidationResult({
        valid: false,
        warnings: [(e as Error).message],
        is_extreme: false,
      });
    }
  };

  const handleEditToggle = () => {
    if (!isEditMode && header) {
      // Enter edit mode - initialize with current values
      setEditParams({
        palette_id: header.palette_id,
        attack_power: header.attack_power,
        defense_rating: header.defense_rating,
        speed_rating: header.speed_rating,
      });
      setSaveError(null);
      setSaveSuccess(false);
    } else {
      // Exit edit mode
      setEditParams(null);
      setValidationResult(null);
      setSaveError(null);
    }
    setIsEditMode(!isEditMode);
  };

  const handleCancel = () => {
    setIsEditMode(false);
    setEditParams(null);
    setValidationResult(null);
    setSaveError(null);
    setShowConfirmDialog(false);
  };

  const handleParamChange = (field: keyof EditableFighterParams, value: string) => {
    const numValue = parseInt(value, 10);
    if (isNaN(numValue)) return;
    
    // Clamp to 0-255 for u8 range
    const clampedValue = Math.max(0, Math.min(255, numValue));
    
    setEditParams(prev => prev ? { ...prev, [field]: clampedValue } : null);
    setSaveSuccess(false);
  };

  const handleSave = async () => {
    if (!editParams || !selectedBoxer || !validationResult) return;

    // If extreme values, show confirmation dialog first
    if (validationResult.is_extreme && !showConfirmDialog) {
      setShowConfirmDialog(true);
      return;
    }

    setIsSaving(true);
    setSaveError(null);

    try {
      const fighterIndex = getFighterIndex(selectedBoxer.name);
      if (fighterIndex < 0) {
        throw new Error('Invalid fighter index');
      }

      const result = await invoke<EditableFighterParams>('update_fighter_params', {
        fighterIndex,
        params: editParams,
      });

      // Mark as pending write
      if (header) {
        const pcOffset = `0x${header.pc_offset.toString(16).toUpperCase()}`;
        setPendingWrite(pcOffset);
      }

      // Update header with new values
      setHeader(prev => prev ? {
        ...prev,
        palette_id: result.palette_id,
        attack_power: result.attack_power,
        defense_rating: result.defense_rating,
        speed_rating: result.speed_rating,
      } : null);

      setSaveSuccess(true);
      setIsEditMode(false);
      setEditParams(null);
      setValidationResult(null);
      setShowConfirmDialog(false);
    } catch (e) {
      setSaveError((e as Error).message);
    } finally {
      setIsSaving(false);
    }
  };

  const filteredScripts = filter === 'All' 
    ? scripts 
    : scripts.filter(s => s.category === filter);

  const formatBytes = (bytes: number[]): string => {
    return bytes.slice(0, 16).map(b => b.toString(16).padStart(2, '0')).join(' ');
  };

  const formatAddr = (bank: number, addr: number): string => {
    return `$${bank.toString(16).toUpperCase()}/${addr.toString(16).toUpperCase().padStart(4, '0')}`;
  };

  // Check if save should be disabled
  const isSaveDisabled = !validationResult?.valid || isSaving || !editParams;

  return (
    <div className="flex flex-col h-full bg-slate-900 text-white p-6">
      <div className="flex items-center justify-between mb-6">
        <h1 className="text-2xl font-bold text-blue-400">Script Viewer</h1>
        <div className="flex items-center gap-4">
          <span className="text-sm text-slate-400">
            {selectedBoxer ? `Viewing: ${selectedBoxer.name}` : 'Viewing: All Scripts'}
          </span>
          <select
            value={filter}
            onChange={(e) => setFilter(e.target.value as ScriptRecord['category'] | 'All')}
            className="bg-slate-800 border border-slate-600 rounded px-3 py-1 text-sm"
          >
            <option value="All">All Categories</option>
            <option value="AnimationScript">Animation</option>
            <option value="AiScript">AI Behavior</option>
            <option value="SpriteScript">Sprite</option>
            <option value="CornerManScript">Corner Man</option>
            <option value="PlayerScript">Player</option>
          </select>
        </div>
      </div>

      <div className="flex gap-6 h-full overflow-hidden">
        {/* Left Panel: Script List */}
        <div className="w-80 flex flex-col gap-4 overflow-hidden">
          {/* Fighter Header (if applicable) */}
          {header && (
            <div className="bg-slate-800 rounded-lg p-4 border border-slate-700">
              <div className="flex items-center justify-between mb-3">
                <h3 className="text-sm font-semibold text-slate-400 uppercase">Fighter Header</h3>
                {selectedBoxer && (
                  <button
                    onClick={handleEditToggle}
                    disabled={isSaving}
                    className={`px-3 py-1 rounded text-xs font-medium transition ${
                      isEditMode
                        ? 'bg-slate-600 text-slate-300 hover:bg-slate-500'
                        : 'bg-blue-600 text-white hover:bg-blue-500'
                    } disabled:opacity-50 disabled:cursor-not-allowed`}
                  >
                    {isEditMode ? 'Cancel' : 'Edit'}
                  </button>
                )}
              </div>

              {isEditMode && editParams ? (
                // Edit Mode UI
                <div className="space-y-3">
                  {/* Palette ID */}
                  <div>
                    <label className="block text-xs text-slate-500 mb-1">Palette ID:</label>
                    <div className="flex items-center gap-2">
                      <input
                        type="number"
                        min={0}
                        max={255}
                        value={editParams.palette_id}
                        onChange={(e) => handleParamChange('palette_id', e.target.value)}
                        className="w-20 bg-slate-900 border border-slate-600 rounded px-2 py-1 text-sm text-slate-300 focus:border-blue-500 focus:outline-none"
                      />
                      <span className="text-xs text-slate-500">(0-255)</span>
                    </div>
                  </div>

                  {/* Attack Power */}
                  <div>
                    <label className="block text-xs text-slate-500 mb-1">Attack:</label>
                    <div className="flex items-center gap-2">
                      <input
                        type="number"
                        min={0}
                        max={255}
                        value={editParams.attack_power}
                        onChange={(e) => handleParamChange('attack_power', e.target.value)}
                        className={`w-20 bg-slate-900 border rounded px-2 py-1 text-sm text-slate-300 focus:outline-none ${
                          editParams.attack_power > 200
                            ? 'border-red-500 focus:border-red-400'
                            : editParams.attack_power > 150
                            ? 'border-amber-500 focus:border-amber-400'
                            : 'border-slate-600 focus:border-blue-500'
                        }`}
                      />
                      <span className="text-xs text-slate-500">(0-255)</span>
                      {editParams.attack_power > 200 && (
                        <span className="text-xs text-red-400">⚠️ Extreme</span>
                      )}
                    </div>
                  </div>

                  {/* Defense Rating */}
                  <div>
                    <label className="block text-xs text-slate-500 mb-1">Defense:</label>
                    <div className="flex items-center gap-2">
                      <input
                        type="number"
                        min={0}
                        max={255}
                        value={editParams.defense_rating}
                        onChange={(e) => handleParamChange('defense_rating', e.target.value)}
                        className={`w-20 bg-slate-900 border rounded px-2 py-1 text-sm text-slate-300 focus:outline-none ${
                          editParams.defense_rating > 200
                            ? 'border-red-500 focus:border-red-400'
                            : 'border-slate-600 focus:border-blue-500'
                        }`}
                      />
                      <span className="text-xs text-slate-500">(0-255)</span>
                      {editParams.defense_rating > 200 && (
                        <span className="text-xs text-red-400">⚠️ Extreme</span>
                      )}
                    </div>
                  </div>

                  {/* Speed Rating */}
                  <div>
                    <label className="block text-xs text-slate-500 mb-1">Speed:</label>
                    <div className="flex items-center gap-2">
                      <input
                        type="number"
                        min={0}
                        max={255}
                        value={editParams.speed_rating}
                        onChange={(e) => handleParamChange('speed_rating', e.target.value)}
                        className={`w-20 bg-slate-900 border rounded px-2 py-1 text-sm text-slate-300 focus:outline-none ${
                          editParams.speed_rating > 200
                            ? 'border-red-500 focus:border-red-400'
                            : editParams.speed_rating > 150
                            ? 'border-amber-500 focus:border-amber-400'
                            : 'border-slate-600 focus:border-blue-500'
                        }`}
                      />
                      <span className="text-xs text-slate-500">(0-255)</span>
                      {editParams.speed_rating > 200 && (
                        <span className="text-xs text-red-400">⚠️ Extreme</span>
                      )}
                    </div>
                  </div>

                  {/* Validation Warnings */}
                  {validationResult && validationResult.warnings.length > 0 && (
                    <div className="p-2 rounded bg-amber-500/10 border border-amber-500/30">
                      <div className="flex items-center gap-1 text-amber-400 text-xs font-medium mb-1">
                        <svg className="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
                          <path fillRule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clipRule="evenodd" />
                        </svg>
                        Warnings
                      </div>
                      <ul className="text-xs text-amber-300/80 space-y-0.5">
                        {validationResult.warnings.map((warning, i) => (
                          <li key={i}>• {warning}</li>
                        ))}
                      </ul>
                    </div>
                  )}

                  {/* Error Message */}
                  {saveError && (
                    <div className="p-2 rounded bg-red-500/10 border border-red-500/30">
                      <div className="flex items-center gap-1 text-red-400 text-xs">
                        <svg className="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
                          <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clipRule="evenodd" />
                        </svg>
                        {saveError}
                      </div>
                    </div>
                  )}

                  {/* Action Buttons */}
                  <div className="flex gap-2 pt-2">
                    <button
                      onClick={handleCancel}
                      disabled={isSaving}
                      className="flex-1 px-3 py-1.5 rounded bg-slate-700 text-slate-300 text-xs font-medium hover:bg-slate-600 transition disabled:opacity-50"
                    >
                      Cancel
                    </button>
                    <button
                      onClick={handleSave}
                      disabled={isSaveDisabled}
                      className="flex-1 px-3 py-1.5 rounded bg-green-600 text-white text-xs font-medium hover:bg-green-500 transition disabled:opacity-50 disabled:cursor-not-allowed"
                    >
                      {isSaving ? (
                        <span className="flex items-center justify-center gap-1">
                          <svg className="animate-spin h-3 w-3" fill="none" viewBox="0 0 24 24">
                            <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
                            <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
                          </svg>
                          Saving...
                        </span>
                      ) : (
                        'Save Changes'
                      )}
                    </button>
                  </div>

                  {/* Extreme Value Confirmation Dialog */}
                  {showConfirmDialog && (
                    <div className="p-3 rounded bg-red-500/10 border border-red-500/30">
                      <div className="flex items-center gap-1 text-red-400 text-xs font-medium mb-2">
                        <svg className="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
                          <path fillRule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7 4a1 1 0 11-2 0 1 1 0 012 0zm-1-9a1 1 0 00-1 1v4a1 1 0 102 0V6a1 1 0 00-1-1z" clipRule="evenodd" />
                        </svg>
                        Confirm Extreme Values
                      </div>
                      <p className="text-xs text-red-300/80 mb-2">
                        These values may make the boxer extremely difficult or unbalanced. Are you sure?
                      </p>
                      <div className="flex gap-2">
                        <button
                          onClick={() => setShowConfirmDialog(false)}
                          className="px-2 py-1 rounded bg-slate-700 text-slate-300 text-xs hover:bg-slate-600"
                        >
                          No, Adjust
                        </button>
                        <button
                          onClick={handleSave}
                          disabled={isSaving}
                          className="px-2 py-1 rounded bg-red-600 text-white text-xs hover:bg-red-500 disabled:opacity-50"
                        >
                          Yes, Save
                        </button>
                      </div>
                    </div>
                  )}
                </div>
              ) : (
                // Read-Only View
                <>
                  <div className="grid grid-cols-2 gap-2 text-xs">
                    <div className="text-slate-500">Palette ID:</div>
                    <div className="text-slate-300">{header.palette_id}</div>
                    <div className="text-slate-500">Attack:</div>
                    <div className={`${header.attack_power > 200 ? 'text-red-400 font-medium' : 'text-slate-300'}`}>
                      {header.attack_power}
                      {header.attack_power > 200 && ' ⚠️'}
                    </div>
                    <div className="text-slate-500">Defense:</div>
                    <div className={`${header.defense_rating > 200 ? 'text-red-400 font-medium' : 'text-slate-300'}`}>
                      {header.defense_rating}
                      {header.defense_rating > 200 && ' ⚠️'}
                    </div>
                    <div className="text-slate-500">Speed:</div>
                    <div className={`${header.speed_rating > 200 ? 'text-red-400 font-medium' : 'text-slate-300'}`}>
                      {header.speed_rating}
                      {header.speed_rating > 200 && ' ⚠️'}
                    </div>
                    <div className="text-slate-500">Pose Table:</div>
                    <div className="text-blue-400">${header.pose_table_ptr.toString(16).toUpperCase()}</div>
                    <div className="text-slate-500">AI Script:</div>
                    <div className="text-blue-400">${header.ai_script_ptr.toString(16).toUpperCase()}</div>
                  </div>

                  {/* Success Toast */}
                  {saveSuccess && (
                    <div className="mt-3 p-2 rounded bg-green-500/10 border border-green-500/30">
                      <div className="flex items-center gap-1 text-green-400 text-xs">
                        <svg className="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
                          <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clipRule="evenodd" />
                        </svg>
                        Changes saved! Remember to save the ROM.
                      </div>
                    </div>
                  )}
                </>
              )}
            </div>
          )}

          {/* Script List */}
          <div className="flex-1 bg-slate-800 rounded-lg border border-slate-700 overflow-hidden flex flex-col">
            <div className="p-3 border-b border-slate-700 bg-slate-750">
              <span className="text-sm font-semibold text-slate-400">
                Scripts ({filteredScripts.length})
              </span>
            </div>
            <div className="flex-1 overflow-y-auto">
              {loading ? (
                <div className="p-4 text-center text-slate-500">
                  <div className="animate-spin inline-block w-6 h-6 border-2 border-blue-500 border-t-transparent rounded-full"></div>
                </div>
              ) : filteredScripts.length === 0 ? (
                <div className="p-4 text-center text-slate-500 italic">
                  No scripts found
                </div>
              ) : (
                filteredScripts.map((script, idx) => (
                  <button
                    key={idx}
                    onClick={() => setSelectedScript(script)}
                    className={`w-full p-3 text-left border-b border-slate-700/50 hover:bg-slate-700/50 transition ${
                      selectedScript?.label === script.label ? 'bg-slate-700' : ''
                    }`}
                  >
                    <div className="flex items-center gap-2 mb-1">
                      <span 
                        className="px-2 py-0.5 rounded text-xs font-medium"
                        style={{ 
                          backgroundColor: `${categoryColors[script.category]}20`,
                          color: categoryColors[script.category]
                        }}
                      >
                        {categoryLabels[script.category]}
                      </span>
                      {script.is_shared && (
                        <span className="px-1.5 py-0.5 rounded text-xs bg-amber-500/20 text-amber-400">
                          Shared
                        </span>
                      )}
                      <span 
                        className="ml-auto px-1.5 py-0.5 rounded text-xs"
                        style={{ 
                          backgroundColor: `${riskColors[script.risk]}20`,
                          color: riskColors[script.risk]
                        }}
                      >
                        {script.risk}
                      </span>
                    </div>
                    <div className="text-sm text-slate-300 truncate">{script.label}</div>
                    <div className="text-xs text-slate-500 mt-0.5">
                      {formatAddr(script.bank, script.snes_addr)}
                    </div>
                  </button>
                ))
              )}
            </div>
          </div>
        </div>

        {/* Right Panel: Script Details */}
        <div className="flex-1 bg-slate-800 rounded-lg border border-slate-700 overflow-hidden flex flex-col">
          {selectedScript ? (
            <>
              <div className="p-4 border-b border-slate-700 bg-slate-750">
                <div className="flex items-center gap-3 mb-2">
                  <h2 className="text-lg font-semibold">{selectedScript.label}</h2>
                  <span 
                    className="px-2 py-1 rounded text-xs font-medium"
                    style={{ 
                      backgroundColor: `${categoryColors[selectedScript.category]}20`,
                      color: categoryColors[selectedScript.category]
                    }}
                  >
                    {categoryLabels[selectedScript.category]}
                  </span>
                </div>
                <div className="flex items-center gap-4 text-sm text-slate-400">
                  <span>Bank: <code className="text-slate-300">${selectedScript.bank.toString(16).toUpperCase()}</code></span>
                  <span>SNES Addr: <code className="text-slate-300">${selectedScript.snes_addr.toString(16).toUpperCase().padStart(4, '0')}</code></span>
                  <span>PC Offset: <code className="text-slate-300">0x{selectedScript.pc_offset.toString(16).toUpperCase()}</code></span>
                </div>
              </div>

              <div className="flex-1 overflow-y-auto p-4 space-y-6">
                {/* Description */}
                <div>
                  <h3 className="text-sm font-semibold text-slate-400 uppercase mb-2">Description</h3>
                  <p className="text-slate-300 text-sm leading-relaxed">{selectedScript.description}</p>
                </div>

                {/* Owners */}
                <div>
                  <h3 className="text-sm font-semibold text-slate-400 uppercase mb-2">
                    Used By {selectedScript.is_shared ? '(Shared)' : ''}
                  </h3>
                  <div className="flex flex-wrap gap-2">
                    {selectedScript.owners.map((owner, i) => (
                      <span key={i} className="px-2 py-1 rounded bg-slate-700 text-slate-300 text-sm">
                        {owner}
                      </span>
                    ))}
                  </div>
                </div>

                {/* Risk Warning */}
                {selectedScript.risk === 'High' && (
                  <div className="p-3 rounded-lg bg-red-500/10 border border-red-500/30">
                    <div className="flex items-center gap-2 text-red-400">
                      <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
                        <path fillRule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clipRule="evenodd" />
                      </svg>
                      <span className="font-medium">High Risk Script</span>
                    </div>
                    <p className="text-red-300/70 text-sm mt-1">
                      Editing this script may break game behavior. Make backups before modifying.
                    </p>
                  </div>
                )}

                {/* Raw Bytes Preview */}
                <div>
                  <h3 className="text-sm font-semibold text-slate-400 uppercase mb-2">Raw Bytes (Preview)</h3>
                  <div className="font-mono text-xs bg-slate-950 p-3 rounded border border-slate-700 overflow-x-auto">
                    <div className="text-slate-500 mb-2">First 64 bytes at PC offset 0x{selectedScript.pc_offset.toString(16).toUpperCase()}:</div>
                    <div className="grid grid-cols-16 gap-1 text-slate-400">
                      {selectedScript.preview_bytes.map((byte, i) => (
                        <span key={i} className="text-center">
                          {byte.toString(16).padStart(2, '0')}
                        </span>
                      ))}
                    </div>
                  </div>
                </div>

                {/* Parameter Editing Notice */}
                <div className="p-3 rounded-lg bg-green-500/10 border border-green-500/30">
                  <div className="flex items-center gap-2 text-green-400">
                    <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
                      <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clipRule="evenodd" />
                    </svg>
                    <span className="font-medium">Known-Safe Editing Available</span>
                  </div>
                  <p className="text-green-300/70 text-sm mt-1">
                    Fighter header parameters (Attack, Defense, Speed, Palette) can now be edited safely from the left panel.
                  </p>
                </div>
              </div>
            </>
          ) : (
            <div className="flex-1 flex items-center justify-center text-slate-500">
              <div className="text-center">
                <svg className="w-16 h-16 mx-auto mb-4 text-slate-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4" />
                </svg>
                <p className="text-lg">Select a script to view details</p>
                <p className="text-sm text-slate-600 mt-2">
                  {scripts.length} scripts available
                </p>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
