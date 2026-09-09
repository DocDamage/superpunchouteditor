/**
 * AI Behavior Editor for Super Punch-Out!!
 * 
 * Visual editor for boxer AI attack patterns, defense behaviors, and difficulty scaling.
 */

import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useStore } from '../store/useStore';
import { 
  AiBehavior, 
  AttackPattern, 
  SimulationResult, 
  MoveTypeOption, 
  DefenseTypeOption,
  ConditionTypeOption 
} from '../types/aiBehavior';
import { AttackPatternEditor } from './AttackPatternEditor';
import { DifficultyCurveEditor } from './DifficultyCurveEditor';
import { SimulationPreview } from './SimulationPreview';
import './AIEditor.css';

type TabType = 'attack' | 'defense' | 'difficulty' | 'triggers' | 'simulation';

export function AIEditor() {
  const { selectedBoxer, fighters } = useStore();
  const [activeTab, setActiveTab] = useState<TabType>('attack');
  const [aiBehavior, setAiBehavior] = useState<AiBehavior | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [selectedPattern, setSelectedPattern] = useState<AttackPattern | null>(null);
  const [simulationResult, setSimulationResult] = useState<SimulationResult | null>(null);
  const [isSimulating, setIsSimulating] = useState(false);
  const [moveTypes, setMoveTypes] = useState<MoveTypeOption[]>([]);
  const [defenseTypes, setDefenseTypes] = useState<DefenseTypeOption[]>([]);
  const [conditionTypes, setConditionTypes] = useState<ConditionTypeOption[]>([]);
  const [showNewPatternDialog, setShowNewPatternDialog] = useState(false);
  const [newPatternName, setNewPatternName] = useState('');

  // Get fighter index from selected boxer
  const fighterIndex = fighters.findIndex(f => f.name === selectedBoxer?.name);

  // Load AI behavior
  const loadAiBehavior = useCallback(async () => {
    if (fighterIndex === -1) return;
    
    setLoading(true);
    setError(null);
    
    try {
      const behavior = await invoke<AiBehavior>('get_ai_behavior', { fighterId: fighterIndex });
      setAiBehavior(behavior);
    } catch (err) {
      setError(`Failed to load AI behavior: ${err}`);
    } finally {
      setLoading(false);
    }
  }, [fighterIndex]);

  // Load reference data
  useEffect(() => {
    const loadReferenceData = async () => {
      try {
        const [moves, defenses, conditions] = await Promise.all([
          invoke<MoveTypeOption[]>('get_move_types'),
          invoke<DefenseTypeOption[]>('get_defense_types'),
          invoke<ConditionTypeOption[]>('get_condition_types'),
        ]);
        setMoveTypes(moves);
        setDefenseTypes(defenses);
        setConditionTypes(conditions);
      } catch (err) {
        console.error('Failed to load reference data:', err);
      }
    };
    
    loadReferenceData();
  }, []);

  // Load AI behavior when fighter changes
  useEffect(() => {
    loadAiBehavior();
  }, [loadAiBehavior]);

  // Run simulation
  const runSimulation = async () => {
    if (fighterIndex === -1) return;
    
    setIsSimulating(true);
    setError(null);
    
    try {
      const result = await invoke<SimulationResult>('test_ai_behavior', { fighterId: fighterIndex });
      setSimulationResult(result);
      setActiveTab('simulation');
    } catch (err) {
      setError(`Simulation failed: ${err}`);
    } finally {
      setIsSimulating(false);
    }
  };

  // Update attack pattern
  const handleUpdatePattern = async (pattern: AttackPattern) => {
    if (fighterIndex === -1) return;
    
    try {
      await invoke('update_attack_pattern', {
        fighterId: fighterIndex,
        patternId: pattern.id,
        pattern,
      });
      
      // Update local state
      setAiBehavior(prev => {
        if (!prev) return null;
        return {
          ...prev,
          attack_patterns: prev.attack_patterns.map(p => 
            p.id === pattern.id ? pattern : p
          ),
        };
      });
      
      setSelectedPattern(null);
    } catch (err) {
      setError(`Failed to update pattern: ${err}`);
    }
  };

  // Add new attack pattern
  const handleAddPattern = async () => {
    if (fighterIndex === -1 || !newPatternName.trim()) return;
    
    const newPattern: AttackPattern = {
      id: `pattern_${Date.now()}`,
      name: newPatternName,
      sequence: [],
      frequency: 50,
      conditions: [],
      difficulty_min: 0,
      difficulty_max: 255,
      available_round_1: true,
      available_round_2: true,
      available_round_3: true,
      weight: 10,
    };
    
    try {
      await invoke('add_attack_pattern', {
        fighterId: fighterIndex,
        pattern: newPattern,
      });
      
      setAiBehavior(prev => {
        if (!prev) return null;
        return {
          ...prev,
          attack_patterns: [...prev.attack_patterns, newPattern],
        };
      });
      
      setNewPatternName('');
      setShowNewPatternDialog(false);
      setSelectedPattern(newPattern);
    } catch (err) {
      setError(`Failed to add pattern: ${err}`);
    }
  };

  // Remove attack pattern
  const handleRemovePattern = async (patternId: string) => {
    if (fighterIndex === -1) return;
    
    try {
      await invoke('remove_attack_pattern', {
        fighterId: fighterIndex,
        patternId,
      });
      
      setAiBehavior(prev => {
        if (!prev) return null;
        return {
          ...prev,
          attack_patterns: prev.attack_patterns.filter(p => p.id !== patternId),
        };
      });
      
      if (selectedPattern?.id === patternId) {
        setSelectedPattern(null);
      }
    } catch (err) {
      setError(`Failed to remove pattern: ${err}`);
    }
  };

  // Update difficulty curve
  const handleUpdateDifficulty = async (curve: AiBehavior['difficulty_curve']) => {
    if (fighterIndex === -1) return;
    
    try {
      await invoke('update_difficulty_curve', {
        fighterId: fighterIndex,
        curve,
      });
      
      setAiBehavior(prev => {
        if (!prev) return null;
        return { ...prev, difficulty_curve: curve };
      });
    } catch (err) {
      setError(`Failed to update difficulty: ${err}`);
    }
  };

  if (!selectedBoxer) {
    return (
      <div className="ai-editor-empty">
        <p>Select a boxer to edit their AI behavior.</p>
      </div>
    );
  }

  if (loading) {
    return (
      <div className="ai-editor-loading">
        <div className="spinner" />
        <p>Loading AI behavior data...</p>
      </div>
    );
  }

  return (
    <div className="ai-editor">
      <header className="ai-editor-header">
        <div className="ai-editor-title">
          <h2>AI Behavior Editor - {selectedBoxer.name}</h2>
          <span className="research-warning">
            ⚠️ ROM Research Required - Placeholder Data
          </span>
        </div>
        <button 
          className="test-ai-btn"
          onClick={runSimulation}
          disabled={isSimulating}
        >
          {isSimulating ? 'Testing...' : '🎮 Test AI'}
        </button>
      </header>

      {error && (
        <div className="ai-editor-error">
          {error}
          <button onClick={() => setError(null)}>×</button>
        </div>
      )}

      <nav className="ai-editor-tabs">
        <button 
          className={activeTab === 'attack' ? 'active' : ''}
          onClick={() => setActiveTab('attack')}
        >
          🥊 Attack Patterns
        </button>
        <button 
          className={activeTab === 'defense' ? 'active' : ''}
          onClick={() => setActiveTab('defense')}
        >
          🛡️ Defense
        </button>
        <button 
          className={activeTab === 'difficulty' ? 'active' : ''}
          onClick={() => setActiveTab('difficulty')}
        >
          📈 Difficulty
        </button>
        <button 
          className={activeTab === 'triggers' ? 'active' : ''}
          onClick={() => setActiveTab('triggers')}
        >
          ⚡ Triggers
        </button>
        <button 
          className={activeTab === 'simulation' ? 'active' : ''}
          onClick={() => setActiveTab('simulation')}
        >
          🧪 Simulation
        </button>
      </nav>

      <div className="ai-editor-content">
        {activeTab === 'attack' && (
          <div className="attack-patterns-panel">
            <div className="patterns-list">
              <div className="patterns-header">
                <h3>Attack Patterns</h3>
                <button 
                  className="add-btn"
                  onClick={() => setShowNewPatternDialog(true)}
                >
                  + Add Pattern
                </button>
              </div>
              
              <div className="patterns-grid">
                {aiBehavior?.attack_patterns.map(pattern => (
                  <div 
                    key={pattern.id}
                    className={`pattern-card ${selectedPattern?.id === pattern.id ? 'selected' : ''}`}
                    onClick={() => setSelectedPattern(pattern)}
                  >
                    <div className="pattern-header">
                      <span className="pattern-name">{pattern.name}</span>
                      <button 
                        className="remove-btn"
                        onClick={(e) => {
                          e.stopPropagation();
                          handleRemovePattern(pattern.id);
                        }}
                      >
                        ×
                      </button>
                    </div>
                    <div className="pattern-stats">
                      <span className="stat">
                        {pattern.sequence.length} move{pattern.sequence.length !== 1 ? 's' : ''}
                      </span>
                      <span className="stat">
                        {pattern.frequency}/255 freq
                      </span>
                    </div>
                    <div className="pattern-rounds">
                      {pattern.available_round_1 && <span className="round">R1</span>}
                      {pattern.available_round_2 && <span className="round">R2</span>}
                      {pattern.available_round_3 && <span className="round">R3</span>}
                    </div>
                  </div>
                ))}
              </div>
            </div>

            {selectedPattern && (
              <AttackPatternEditor
                pattern={selectedPattern}
                moveTypes={moveTypes}
                conditionTypes={conditionTypes}
                onSave={handleUpdatePattern}
                onCancel={() => setSelectedPattern(null)}
              />
            )}
          </div>
        )}

        {activeTab === 'defense' && (
          <div className="defense-panel">
            <h3>Defense Behaviors</h3>
            <div className="defense-list">
              {aiBehavior?.defense_behaviors.map((behavior, index) => (
                <div key={index} className="defense-card">
                  <span className="defense-icon">
                    {defenseTypes.find(d => d.id === behavior.behavior_type)?.icon || '🛡️'}
                  </span>
                  <span className="defense-name">
                    {defenseTypes.find(d => d.id === behavior.behavior_type)?.name || behavior.behavior_type}
                  </span>
                  <div className="defense-stats">
                    <div className="stat-bar">
                      <label>Frequency</label>
                      <input 
                        type="range" 
                        min="0" 
                        max="255" 
                        value={behavior.frequency}
                        readOnly
                      />
                      <span>{behavior.frequency}</span>
                    </div>
                    <div className="stat-bar">
                      <label>Success Rate</label>
                      <input 
                        type="range" 
                        min="0" 
                        max="255" 
                        value={behavior.success_rate}
                        readOnly
                      />
                      <span>{behavior.success_rate}</span>
                    </div>
                  </div>
                  {behavior.leads_to_counter && (
                    <span className="counter-badge">⚔️ Counter</span>
                  )}
                </div>
              ))}
            </div>
          </div>
        )}

        {activeTab === 'difficulty' && aiBehavior && (
          <DifficultyCurveEditor
            curve={aiBehavior.difficulty_curve}
            onChange={handleUpdateDifficulty}
          />
        )}

        {activeTab === 'triggers' && (
          <div className="triggers-panel">
            <h3>AI Triggers</h3>
            <div className="triggers-list">
              {aiBehavior?.triggers.map((trigger, index) => (
                <div key={index} className="trigger-card">
                  <div className="trigger-condition">
                    <span className="label">When:</span>
                    <span className="condition">{formatCondition(trigger.condition)}</span>
                  </div>
                  <div className="trigger-action">
                    <span className="label">Do:</span>
                    <span className="action">{formatAction(trigger.action)}</span>
                  </div>
                  <div className="trigger-meta">
                    <span>Priority: {trigger.priority}</span>
                    <span>Cooldown: {trigger.cooldown}f</span>
                    {trigger.once_per_round && <span className="badge">Once per round</span>}
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}

        {activeTab === 'simulation' && (
          <SimulationPreview 
            result={simulationResult}
            isRunning={isSimulating}
            onRun={runSimulation}
          />
        )}
      </div>

      {showNewPatternDialog && (
        <div className="modal-overlay" onClick={() => setShowNewPatternDialog(false)}>
          <div className="modal" onClick={e => e.stopPropagation()}>
            <h3>New Attack Pattern</h3>
            <input
              type="text"
              placeholder="Pattern name (e.g., 'Left Hook Combo')"
              value={newPatternName}
              onChange={(e) => setNewPatternName(e.target.value)}
              autoFocus
            />
            <div className="modal-actions">
              <button onClick={() => setShowNewPatternDialog(false)}>Cancel</button>
              <button 
                onClick={handleAddPattern}
                disabled={!newPatternName.trim()}
                className="primary"
              >
                Create Pattern
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

// Helper functions for formatting
function formatCondition(condition: { type: string; value?: number }): string {
  switch (condition.type) {
    case 'health_below':
      return `Health < ${condition.value}%`;
    case 'health_above':
      return `Health > ${condition.value}%`;
    case 'round':
      return `Round ${condition.value}`;
    case 'time_below':
      return `Time < ${condition.value}s`;
    case 'player_stunned':
      return 'Player Stunned';
    case 'player_blocking':
      return 'Player Blocking';
    case 'random_chance':
      return `${Math.round((condition.value || 0) * 100 / 255)}% Chance`;
    case 'combo_count':
      return `${condition.value} Hit Combo`;
    default:
      return condition.type;
  }
}

function formatAction(action: { type: string; pattern_id?: string; behavior_id?: string }): string {
  switch (action.type) {
    case 'use_pattern':
      return `Use Pattern "${action.pattern_id}"`;
    case 'change_behavior':
      return `Change to "${action.behavior_id}"`;
    case 'taunt':
      return 'Taunt';
    case 'special_move':
      return 'Special Move';
    case 'defend':
      return 'Defend';
    case 'reset_behavior':
      return 'Reset Behavior';
    default:
      return action.type;
  }
}
