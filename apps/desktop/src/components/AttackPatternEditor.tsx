/**
 * Attack Pattern Editor
 * 
 * Component for editing individual attack patterns including move sequences,
 * conditions, and timing parameters.
 */

import { useState } from 'react';
import { AttackPattern, AttackMove, MoveTypeOption, ConditionTypeOption } from '../types/aiBehavior';
import './AttackPatternEditor.css';

interface AttackPatternEditorProps {
  pattern: AttackPattern;
  moveTypes: MoveTypeOption[];
  conditionTypes: ConditionTypeOption[];
  onSave: (pattern: AttackPattern) => void;
  onCancel: () => void;
}

export function AttackPatternEditor({ 
  pattern, 
  moveTypes, 
  conditionTypes,
  onSave, 
  onCancel 
}: AttackPatternEditorProps) {
  const [editedPattern, setEditedPattern] = useState<AttackPattern>({ ...pattern });
  const [selectedMoveIndex, setSelectedMoveIndex] = useState<number | null>(null);
  const [showAddMove, setShowAddMove] = useState(false);

  const handleAddMove = (moveType: string) => {
    const moveTypeInfo = moveTypes.find(m => m.id === moveType);
    if (!moveTypeInfo) return;

    const newMove: AttackMove = {
      move_type: moveType as AttackMove['move_type'],
      windup_frames: 12,
      active_frames: 8,
      recovery_frames: 20,
      damage: 10,
      stun: 5,
      hitbox: {
        x: -8,
        y: -20,
        width: 32,
        height: 48,
        height_zone: 'mid',
      },
      pose_id: 0,
      sound_effect: null,
    };

    setEditedPattern(prev => ({
      ...prev,
      sequence: [...prev.sequence, newMove],
    }));
    setShowAddMove(false);
    setSelectedMoveIndex(editedPattern.sequence.length);
  };

  const handleUpdateMove = (index: number, move: AttackMove) => {
    setEditedPattern(prev => ({
      ...prev,
      sequence: prev.sequence.map((m, i) => i === index ? move : m),
    }));
  };

  const handleRemoveMove = (index: number) => {
    setEditedPattern(prev => ({
      ...prev,
      sequence: prev.sequence.filter((_, i) => i !== index),
    }));
    if (selectedMoveIndex === index) {
      setSelectedMoveIndex(null);
    } else if (selectedMoveIndex !== null && selectedMoveIndex > index) {
      setSelectedMoveIndex(selectedMoveIndex - 1);
    }
  };

  const handleMoveUp = (index: number) => {
    if (index === 0) return;
    setEditedPattern(prev => {
      const sequence = [...prev.sequence];
      [sequence[index - 1], sequence[index]] = [sequence[index], sequence[index - 1]];
      return { ...prev, sequence };
    });
    setSelectedMoveIndex(index - 1);
  };

  const handleMoveDown = (index: number) => {
    if (index >= editedPattern.sequence.length - 1) return;
    setEditedPattern(prev => {
      const sequence = [...prev.sequence];
      [sequence[index], sequence[index + 1]] = [sequence[index + 1], sequence[index]];
      return { ...prev, sequence };
    });
    setSelectedMoveIndex(index + 1);
  };

  const totalFrames = editedPattern.sequence.reduce(
    (sum, move) => sum + move.windup_frames + move.active_frames + move.recovery_frames,
    0
  );

  const totalDamage = editedPattern.sequence.reduce(
    (sum, move) => sum + move.damage,
    0
  );

  return (
    <div className="attack-pattern-editor">
      <div className="pattern-editor-header">
        <div className="pattern-name-section">
          <label>Pattern Name</label>
          <input
            type="text"
            value={editedPattern.name}
            onChange={(e) => setEditedPattern(prev => ({ ...prev, name: e.target.value }))}
            className="pattern-name-input"
          />
        </div>
        <div className="pattern-stats-summary">
          <div className="stat-pill">
            <span className="stat-value">{editedPattern.sequence.length}</span>
            <span className="stat-label">moves</span>
          </div>
          <div className="stat-pill">
            <span className="stat-value">{totalFrames}</span>
            <span className="stat-label">frames</span>
          </div>
          <div className="stat-pill">
            <span className="stat-value">{totalDamage}</span>
            <span className="stat-label">dmg</span>
          </div>
        </div>
        <div className="editor-actions">
          <button className="cancel-btn" onClick={onCancel}>Cancel</button>
          <button className="save-btn" onClick={() => onSave(editedPattern)}>Save Pattern</button>
        </div>
      </div>

      <div className="pattern-editor-body">
        <div className="sequence-section">
          <div className="sequence-header">
            <h4>Move Sequence</h4>
            <button 
              className="add-move-btn"
              onClick={() => setShowAddMove(true)}
            >
              + Add Move
            </button>
          </div>

          <div className="sequence-timeline">
            {editedPattern.sequence.map((move, index) => {
              const moveType = moveTypes.find(m => m.id === move.move_type);
              const isSelected = selectedMoveIndex === index;
              
              return (
                <div 
                  key={index}
                  className={`move-node ${isSelected ? 'selected' : ''}`}
                  onClick={() => setSelectedMoveIndex(index)}
                >
                  <div className="move-node-header">
                    <span className="move-icon">{moveType?.icon || '🥊'}</span>
                    <span className="move-name">{moveType?.name || move.move_type}</span>
                    <span className="move-index">#{index + 1}</span>
                  </div>
                  <div className="move-timing">
                    <div className="timing-bar">
                      <span 
                        className="timing-segment windup"
                        style={{ flex: move.windup_frames }}
                        title={`Windup: ${move.windup_frames}f`}
                      />
                      <span 
                        className="timing-segment active"
                        style={{ flex: move.active_frames }}
                        title={`Active: ${move.active_frames}f`}
                      />
                      <span 
                        className="timing-segment recovery"
                        style={{ flex: move.recovery_frames }}
                        title={`Recovery: ${move.recovery_frames}f`}
                      />
                    </div>
                    <span className="move-damage">{move.damage} dmg</span>
                  </div>
                  {isSelected && (
                    <div className="move-actions">
                      <button 
                        onClick={(e) => { e.stopPropagation(); handleMoveUp(index); }}
                        disabled={index === 0}
                      >
                        ↑
                      </button>
                      <button 
                        onClick={(e) => { e.stopPropagation(); handleMoveDown(index); }}
                        disabled={index === editedPattern.sequence.length - 1}
                      >
                        ↓
                      </button>
                      <button 
                        className="remove-move-btn"
                        onClick={(e) => { e.stopPropagation(); handleRemoveMove(index); }}
                      >
                        ×
                      </button>
                    </div>
                  )}
                </div>
              );
            })}
            
            {editedPattern.sequence.length === 0 && (
              <div className="empty-sequence">
                <p>No moves in this pattern</p>
                <button onClick={() => setShowAddMove(true)}>Add your first move</button>
              </div>
            )}
          </div>

          {showAddMove && (
            <div className="add-move-panel">
              <h5>Select Move Type</h5>
              <div className="move-type-grid">
                {moveTypes.map(moveType => (
                  <button
                    key={moveType.id}
                    className="move-type-btn"
                    onClick={() => handleAddMove(moveType.id)}
                  >
                    <span className="move-type-icon">{moveType.icon}</span>
                    <span className="move-type-name">{moveType.name}</span>
                  </button>
                ))}
              </div>
              <button 
                className="cancel-add-btn"
                onClick={() => setShowAddMove(false)}
              >
                Cancel
              </button>
            </div>
          )}
        </div>

        {selectedMoveIndex !== null && editedPattern.sequence[selectedMoveIndex] && (
          <MoveEditor
            move={editedPattern.sequence[selectedMoveIndex]}
            moveTypes={moveTypes}
            onChange={(move) => handleUpdateMove(selectedMoveIndex, move)}
          />
        )}

        <div className="pattern-settings">
          <h4>Pattern Settings</h4>
          
          <div className="setting-group">
            <label>Frequency (0-255)</label>
            <div className="slider-with-value">
              <input
                type="range"
                min="0"
                max="255"
                value={editedPattern.frequency}
                onChange={(e) => setEditedPattern(prev => ({ 
                  ...prev, 
                  frequency: parseInt(e.target.value) 
                }))}
              />
              <span>{editedPattern.frequency}</span>
            </div>
          </div>

          <div className="setting-group">
            <label>Difficulty Range</label>
            <div className="difficulty-range">
              <div className="range-input">
                <label>Min</label>
                <input
                  type="number"
                  min="0"
                  max="255"
                  value={editedPattern.difficulty_min}
                  onChange={(e) => setEditedPattern(prev => ({ 
                    ...prev, 
                    difficulty_min: parseInt(e.target.value) 
                  }))}
                />
              </div>
              <div className="range-input">
                <label>Max</label>
                <input
                  type="number"
                  min="0"
                  max="255"
                  value={editedPattern.difficulty_max}
                  onChange={(e) => setEditedPattern(prev => ({ 
                    ...prev, 
                    difficulty_max: parseInt(e.target.value) 
                  }))}
                />
              </div>
            </div>
          </div>

          <div className="setting-group">
            <label>Available Rounds</label>
            <div className="round-checkboxes">
              <label className="checkbox">
                <input
                  type="checkbox"
                  checked={editedPattern.available_round_1}
                  onChange={(e) => setEditedPattern(prev => ({ 
                    ...prev, 
                    available_round_1: e.target.checked 
                  }))}
                />
                <span>Round 1</span>
              </label>
              <label className="checkbox">
                <input
                  type="checkbox"
                  checked={editedPattern.available_round_2}
                  onChange={(e) => setEditedPattern(prev => ({ 
                    ...prev, 
                    available_round_2: e.target.checked 
                  }))}
                />
                <span>Round 2</span>
              </label>
              <label className="checkbox">
                <input
                  type="checkbox"
                  checked={editedPattern.available_round_3}
                  onChange={(e) => setEditedPattern(prev => ({ 
                    ...prev, 
                    available_round_3: e.target.checked 
                  }))}
                />
                <span>Round 3</span>
              </label>
            </div>
          </div>

          <div className="setting-group">
            <label>Weight (Selection Priority)</label>
            <div className="slider-with-value">
              <input
                type="range"
                min="1"
                max="100"
                value={editedPattern.weight}
                onChange={(e) => setEditedPattern(prev => ({ 
                  ...prev, 
                  weight: parseInt(e.target.value) 
                }))}
              />
              <span>{editedPattern.weight}</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

// Move Editor Sub-component
interface MoveEditorProps {
  move: AttackMove;
  moveTypes: MoveTypeOption[];
  onChange: (move: AttackMove) => void;
}

function MoveEditor({ move, moveTypes, onChange }: MoveEditorProps) {
  const moveType = moveTypes.find(m => m.id === move.move_type);

  return (
    <div className="move-editor">
      <h4>Edit Move: {moveType?.name || move.move_type}</h4>
      
      <div className="move-editor-grid">
        <div className="form-group">
          <label>Move Type</label>
          <select
            value={move.move_type}
            onChange={(e) => onChange({ ...move, move_type: e.target.value as AttackMove['move_type'] })}
          >
            {moveTypes.map(mt => (
              <option key={mt.id} value={mt.id}>
                {mt.icon} {mt.name}
              </option>
            ))}
          </select>
        </div>

        <div className="form-group">
          <label>Damage</label>
          <input
            type="number"
            min="0"
            max="255"
            value={move.damage}
            onChange={(e) => onChange({ ...move, damage: parseInt(e.target.value) || 0 })}
          />
        </div>

        <div className="form-group">
          <label>Stun</label>
          <input
            type="number"
            min="0"
            max="255"
            value={move.stun}
            onChange={(e) => onChange({ ...move, stun: parseInt(e.target.value) || 0 })}
          />
        </div>

        <div className="form-group">
          <label>Windup Frames</label>
          <input
            type="number"
            min="1"
            max="255"
            value={move.windup_frames}
            onChange={(e) => onChange({ ...move, windup_frames: parseInt(e.target.value) || 1 })}
          />
        </div>

        <div className="form-group">
          <label>Active Frames</label>
          <input
            type="number"
            min="1"
            max="255"
            value={move.active_frames}
            onChange={(e) => onChange({ ...move, active_frames: parseInt(e.target.value) || 1 })}
          />
        </div>

        <div className="form-group">
          <label>Recovery Frames</label>
          <input
            type="number"
            min="1"
            max="255"
            value={move.recovery_frames}
            onChange={(e) => onChange({ ...move, recovery_frames: parseInt(e.target.value) || 1 })}
          />
        </div>

        <div className="form-group">
          <label>Hitbox X</label>
          <input
            type="number"
            min="-128"
            max="127"
            value={move.hitbox.x}
            onChange={(e) => onChange({ 
              ...move, 
              hitbox: { ...move.hitbox, x: parseInt(e.target.value) || 0 }
            })}
          />
        </div>

        <div className="form-group">
          <label>Hitbox Y</label>
          <input
            type="number"
            min="-128"
            max="127"
            value={move.hitbox.y}
            onChange={(e) => onChange({ 
              ...move, 
              hitbox: { ...move.hitbox, y: parseInt(e.target.value) || 0 }
            })}
          />
        </div>

        <div className="form-group">
          <label>Hitbox Width</label>
          <input
            type="number"
            min="1"
            max="255"
            value={move.hitbox.width}
            onChange={(e) => onChange({ 
              ...move, 
              hitbox: { ...move.hitbox, width: parseInt(e.target.value) || 1 }
            })}
          />
        </div>

        <div className="form-group">
          <label>Hitbox Height</label>
          <input
            type="number"
            min="1"
            max="255"
            value={move.hitbox.height}
            onChange={(e) => onChange({ 
              ...move, 
              hitbox: { ...move.hitbox, height: parseInt(e.target.value) || 1 }
            })}
          />
        </div>

        <div className="form-group">
          <label>Height Zone</label>
          <select
            value={move.hitbox.height_zone}
            onChange={(e) => onChange({ 
              ...move, 
              hitbox: { ...move.hitbox, height_zone: e.target.value as AttackMove['hitbox']['height_zone'] }
            })}
          >
            <option value="high">High</option>
            <option value="mid">Mid</option>
            <option value="low">Low</option>
          </select>
        </div>

        <div className="form-group">
          <label>Pose ID</label>
          <input
            type="number"
            min="0"
            max="255"
            value={move.pose_id}
            onChange={(e) => onChange({ ...move, pose_id: parseInt(e.target.value) || 0 })}
          />
        </div>
      </div>

      <div className="move-timing-summary">
        <div className="timing-bar-large">
          <div 
            className="timing-block windup" 
            style={{ width: `${(move.windup_frames / 60) * 100}%` }}
          >
            Windup<br/>{move.windup_frames}f
          </div>
          <div 
            className="timing-block active"
            style={{ width: `${(move.active_frames / 60) * 100}%` }}
          >
            Active<br/>{move.active_frames}f
          </div>
          <div 
            className="timing-block recovery"
            style={{ width: `${(move.recovery_frames / 60) * 100}%` }}
          >
            Recovery<br/>{move.recovery_frames}f
          </div>
        </div>
        <div className="total-frames">
          Total: {move.windup_frames + move.active_frames + move.recovery_frames} frames
          (~{((move.windup_frames + move.active_frames + move.recovery_frames) / 60).toFixed(1)}s)
        </div>
      </div>
    </div>
  );
}
