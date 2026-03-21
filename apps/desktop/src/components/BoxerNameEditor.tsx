/**
 * Boxer Name Editor Component
 * 
 * Allows editing of boxer names with character validation,
 * encoding preview, and length limits.
 */

import { useState, useCallback, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { BoxerRosterEntry, NameValidationResult, TextEncodingInfo } from '../types/roster';

interface BoxerNameEditorProps {
  boxers: BoxerRosterEntry[];
  onUpdateName: (fighterId: number, newName: string) => void;
  disabled?: boolean;
}

const getBoxerId = (boxer: BoxerRosterEntry): number => boxer.boxer_id ?? boxer.fighter_id ?? -1;

interface BoxerNameState {
  name: string;
  validation: NameValidationResult | null;
  isEditing: boolean;
}

export function BoxerNameEditor({ boxers, onUpdateName, disabled }: BoxerNameEditorProps) {
  const [nameStates, setNameStates] = useState<Record<number, BoxerNameState>>({});
  const [encodingInfo, setEncodingInfo] = useState<TextEncodingInfo | null>(null);
  const [showEncodingInfo, setShowEncodingInfo] = useState(false);

  // Load text encoding info on mount
  useEffect(() => {
    const loadEncodingInfo = async () => {
      try {
        const info = await invoke<TextEncodingInfo>('get_text_encoding_info');
        setEncodingInfo(info);
      } catch (err) {
        console.error('Failed to load encoding info:', err);
      }
    };
    
    loadEncodingInfo();
  }, []);

  // Initialize name states from boxers
  useEffect(() => {
    setNameStates(prev => {
      const newStates: Record<number, BoxerNameState> = { ...prev };
      boxers.forEach(boxer => {
        const boxerId = getBoxerId(boxer);
        if (!newStates[boxerId]) {
          newStates[boxerId] = {
            name: boxer.name,
            validation: null,
            isEditing: false,
          };
        }
      });
      return newStates;
    });
  }, [boxers]);

  // Validate a name
  const validateName = useCallback(async (name: string) => {
    try {
      const result = await invoke<NameValidationResult>('validate_boxer_name', { name });
      return result;
    } catch (err) {
      console.error('Validation error:', err);
      return null;
    }
  }, []);

  // Handle name change
  const handleNameChange = useCallback(async (fighterId: number, newName: string) => {
    // Update the name state immediately for responsiveness
    setNameStates(prev => ({
      ...prev,
      [fighterId]: {
        ...prev[fighterId],
        name: newName,
        isEditing: true,
      },
    }));

    // Validate the new name
    const validation = await validateName(newName);
    
    setNameStates(prev => ({
      ...prev,
      [fighterId]: {
        ...prev[fighterId],
        validation,
      },
    }));
  }, [validateName]);

  // Handle save
  const handleSave = useCallback(async (fighterId: number) => {
    const state = nameStates[fighterId];
    if (!state || !state.validation?.valid) return;

    onUpdateName(fighterId, state.name);
    
    setNameStates(prev => ({
      ...prev,
      [fighterId]: {
        ...prev[fighterId],
        isEditing: false,
      },
    }));
  }, [nameStates, onUpdateName]);

  // Handle key press (Enter to save, Escape to cancel)
  const handleKeyDown = useCallback((
    fighterId: number, 
    originalName: string,
    e: React.KeyboardEvent
  ) => {
    if (e.key === 'Enter') {
      handleSave(fighterId);
    } else if (e.key === 'Escape') {
      // Reset to original name
      setNameStates(prev => ({
        ...prev,
        [fighterId]: {
          name: originalName,
          validation: null,
          isEditing: false,
        },
      }));
    }
  }, [handleSave]);

  // Get circuit class for styling
  const getCircuitClass = (circuit: string): string => {
    return `circuit-${circuit.toLowerCase()}`;
  };

  // Sort boxers by fighter ID for consistent display
  const sortedBoxers = [...boxers].sort((a, b) => getBoxerId(a) - getBoxerId(b));

  return (
    <div className="name-editor">
      <div className="editor-help">
        <div className="help-header">
          <p>
            Edit boxer names below. Names are limited to {encodingInfo?.max_name_length ?? 16} bytes
            in the ROM. Some characters may use multiple bytes.
          </p>
          <button 
            className="info-toggle"
            onClick={() => setShowEncodingInfo(!showEncodingInfo)}
          >
            {showEncodingInfo ? 'Hide' : 'Show'} encoding info
          </button>
        </div>
        
        {showEncodingInfo && encodingInfo && (
          <div className="encoding-details">
            <div className="encoding-section">
              <h5>Supported Characters</h5>
              <div className="char-grid">
                <span className="char-item">A-Z</span>
                <span className="char-item">a-z</span>
                <span className="char-item">Space</span>
                {encodingInfo.supported_chars.map((char, i) => (
                  <span key={i} className="char-item">{char}</span>
                ))}
              </div>
            </div>
            <div className="encoding-note">
              <strong>Note:</strong> Characters not in the supported list will be 
              replaced with '?' when encoded.
            </div>
          </div>
        )}
      </div>

      <div className="name-list">
        {sortedBoxers.map(boxer => {
          const boxerId = getBoxerId(boxer);
          const state = nameStates[boxerId] || {
            name: boxer.name,
            validation: null,
            isEditing: false,
          };
          
          const hasChanges = state.name !== boxer.name;
          const isValid = state.validation?.valid ?? true;
          
          return (
            <div key={boxerId} className="name-item">
              <div className="name-item-id">
                {boxerId}
              </div>
              
              <span className={`name-item-circuit ${getCircuitClass(boxer.circuit)}`}>
                {boxer.circuit.charAt(0)}
              </span>
              
              <div className="name-input-wrapper">
                <input
                  type="text"
                  value={state.name}
                  onChange={(e) => handleNameChange(boxerId, e.target.value)}
                  onBlur={() => hasChanges && isValid && handleSave(boxerId)}
                  onKeyDown={(e) => handleKeyDown(boxerId, boxer.name, e)}
                  disabled={disabled}
                  className={`name-input ${!isValid ? 'invalid' : ''}`}
                  placeholder="Boxer name"
                />
                
                {state.validation && (
                  <div className={`name-validation ${isValid ? 'valid' : 'invalid'}`}>
                    <span className="char-count">
                      {state.validation.encoded_length} / {state.validation.max_length} bytes
                    </span>
                    {!state.validation.can_encode && (
                      <span className="encoding-warning">
                        Contains unsupported characters
                      </span>
                    )}
                    {state.validation.error && (
                      <span className="error-text">{state.validation.error}</span>
                    )}
                  </div>
                )}
              </div>
              
              {hasChanges && (
                <button
                  onClick={() => handleSave(boxerId)}
                  disabled={disabled || !isValid}
                  className="save-btn"
                  title="Save changes"
                >
                  Save
                </button>
              )}
            </div>
          );
        })}
      </div>

      {/* Add extra styles */}
      <style>{`
        .help-header {
          display: flex;
          justify-content: space-between;
          align-items: flex-start;
          gap: 1rem;
        }
        
        .info-toggle {
          padding: 0.25rem 0.75rem;
          background: var(--blue);
          border: none;
          border-radius: 4px;
          color: white;
          font-size: 0.75rem;
          cursor: pointer;
          white-space: nowrap;
        }
        
        .encoding-details {
          margin-top: 0.75rem;
          padding-top: 0.75rem;
          border-top: 1px solid var(--border);
        }
        
        .encoding-section h5 {
          margin: 0 0 0.5rem 0;
          font-size: 0.75rem;
          color: var(--text-dim);
          text-transform: uppercase;
        }
        
        .char-grid {
          display: flex;
          flex-wrap: wrap;
          gap: 0.375rem;
        }
        
        .char-item {
          padding: 0.25rem 0.5rem;
          background: var(--panel-bg);
          border-radius: 4px;
          font-size: 0.875rem;
          font-family: monospace;
        }
        
        .encoding-note {
          margin-top: 0.75rem;
          padding: 0.5rem;
          background: rgba(251, 191, 36, 0.1);
          border-radius: 4px;
          font-size: 0.8125rem;
          color: var(--text-dim);
        }
        
        .encoding-warning {
          color: #fbbf24;
          font-size: 0.75rem;
        }
        
        .error-text {
          color: #f87171;
          font-size: 0.75rem;
        }
        
        .save-btn {
          padding: 0.375rem 0.75rem;
          background: var(--blue);
          border: none;
          border-radius: 4px;
          color: white;
          font-size: 0.75rem;
          font-weight: 500;
          cursor: pointer;
          transition: all 0.2s;
        }
        
        .save-btn:hover:not(:disabled) {
          background: var(--blue-light, #3b82f6);
        }
        
        .save-btn:disabled {
          opacity: 0.5;
          cursor: not-allowed;
        }
      `}</style>
    </div>
  );
}
