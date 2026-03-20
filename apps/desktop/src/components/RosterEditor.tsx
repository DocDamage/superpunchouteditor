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
import { useStore } from '../store/useStore';
import { BoxerRosterEntry, Circuit, ValidationReport, CircuitTypeInfo, RosterData } from '../types/roster';
import { BoxerNameEditor } from './BoxerNameEditor';
import { CircuitEditor } from './CircuitEditor';
import './RosterEditor.css';

type TabType = 'names' | 'circuits' | 'unlock' | 'intro';

interface RosterEditorProps {
  initialTab?: TabType;
}

export function RosterEditor({ initialTab = 'names' }: RosterEditorProps) {
  const { romSha1 } = useStore();
  const [activeTab, setActiveTab] = useState<TabType>(initialTab);
  const [rosterData, setRosterData] = useState<RosterData | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [validationReport, setValidationReport] = useState<ValidationReport | null>(null);
  const [saving, setSaving] = useState(false);

  // Load roster data
  const loadRosterData = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      
      const data = await invoke<RosterData>('get_roster_data');
      setRosterData(data);
      
      // Also run validation
      const report = await invoke<ValidationReport>('validate_roster_changes');
      setValidationReport(report);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load roster data');
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadRosterData();
  }, [loadRosterData]);

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
        <h3>Roster Metadata Editor</h3>
        <p>Open a ROM to edit roster data.</p>
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
          <h3>Roster Metadata Editor</h3>
          {renderValidationBadge()}
        </div>
        
        <div className="roster-editor-actions">
          <button 
            onClick={handleReset}
            disabled={saving}
            className="btn-secondary"
          >
            Reset to Defaults
          </button>
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
          <span className="error-icon">⚠</span>
          {error}
        </div>
      )}
      
      <div className="roster-editor-tabs">
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
            key={boxer.fighter_id}
            className={`unlock-item ${boxer.is_champion ? 'champion' : ''}`}
          >
            <div className="unlock-rank">{index + 1}</div>
            
            <div className="unlock-info">
              <span className="boxer-name">{boxer.name}</span>
              <span className="circuit-badge circuit-{boxer.circuit.toLowerCase()}">
                {boxer.circuit}
              </span>
            </div>
            
            <div className="unlock-controls">
              <input
                type="number"
                min={0}
                max={15}
                value={boxer.unlock_order}
                onChange={(e) => onUpdateOrder(boxer.fighter_id, parseInt(e.target.value))}
                disabled={disabled}
                className="order-input"
              />
              
              <label className="champion-toggle">
                <input
                  type="checkbox"
                  checked={boxer.is_champion}
                  onChange={(e) => onUpdateChampion(boxer.fighter_id, e.target.checked)}
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
      alert('Intro text saved successfully!');
    } catch (err) {
      alert('Failed to save intro text: ' + (err instanceof Error ? err.message : String(err)));
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
                key={boxer.fighter_id}
                className={`boxer-btn ${selectedBoxer?.fighter_id === boxer.fighter_id ? 'active' : ''}`}
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

// Helper for invoke
const invoke = <T,>(cmd: string, args?: Record<string, unknown>): Promise<T> => {
  return window.__TAURI__.invoke(cmd, args);
};

// Add type declaration for window.__TAURI__
declare global {
  interface Window {
    __TAURI__: {
      invoke: <T>(cmd: string, args?: Record<string, unknown>) => Promise<T>;
    };
  }
}
