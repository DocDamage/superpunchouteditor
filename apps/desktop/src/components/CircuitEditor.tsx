/**
 * Circuit Editor Component
 * 
 * Visual editor for circuit assignments. Allows moving boxers
 * between circuits and setting champion flags.
 */

import { useState, useCallback } from 'react';
import { BoxerRosterEntry, Circuit, CircuitType } from '../types/roster';

interface CircuitEditorProps {
  boxers: BoxerRosterEntry[];
  circuits: Circuit[];
  onUpdateCircuit: (fighterId: number, circuit: string) => void;
  onUpdateChampion: (fighterId: number, isChampion: boolean) => void;
  disabled?: boolean;
}

export function CircuitEditor({
  boxers,
  circuits,
  onUpdateCircuit,
  onUpdateChampion,
  disabled,
}: CircuitEditorProps) {
  const [draggedBoxer, setDraggedBoxer] = useState<BoxerRosterEntry | null>(null);
  const [dragOverCircuit, setDragOverCircuit] = useState<string | null>(null);

  // Get boxers in a circuit
  const getBoxersInCircuit = useCallback((circuitType: CircuitType) => {
    return boxers
      .filter(b => b.circuit === circuitType)
      .sort((a, b) => a.fighter_id - b.fighter_id);
  }, [boxers]);

  // Handle drag start
  const handleDragStart = (boxer: BoxerRosterEntry) => {
    setDraggedBoxer(boxer);
  };

  // Handle drag end
  const handleDragEnd = () => {
    setDraggedBoxer(null);
    setDragOverCircuit(null);
  };

  // Handle drag over
  const handleDragOver = (e: React.DragEvent, circuitType: CircuitType) => {
    e.preventDefault();
    setDragOverCircuit(circuitType);
  };

  // Handle drag leave
  const handleDragLeave = () => {
    setDragOverCircuit(null);
  };

  // Handle drop
  const handleDrop = (e: React.DragEvent, targetCircuit: CircuitType) => {
    e.preventDefault();
    
    if (draggedBoxer && draggedBoxer.circuit !== targetCircuit) {
      onUpdateCircuit(draggedBoxer.fighter_id, targetCircuit);
    }
    
    setDraggedBoxer(null);
    setDragOverCircuit(null);
  };

  // Get circuit icon
  const getCircuitIcon = (circuitType: CircuitType): string => {
    switch (circuitType) {
      case 'Minor': return '🥉';
      case 'Major': return '🥈';
      case 'World': return '🥇';
      case 'Special': return '⭐';
      default: return '🏆';
    }
  };

  // Get circuit color
  const getCircuitColor = (circuitType: CircuitType): string => {
    switch (circuitType) {
      case 'Minor': return '#4ade80';
      case 'Major': return '#60a5fa';
      case 'World': return '#fbbf24';
      case 'Special': return '#f87171';
      default: return '#9ca3af';
    }
  };

  // Handle circuit change from dropdown
  const handleCircuitChange = (fighterId: number, newCircuit: string) => {
    onUpdateCircuit(fighterId, newCircuit);
  };

  return (
    <div className="circuit-editor">
      <div className="editor-help">
        <p>
          Drag and drop boxers between circuits to change their assignments,
          or use the dropdown menu. Toggle the Champion flag for the final
          boxer in each circuit.
        </p>
      </div>

      <div className="circuits-grid">
        {circuits.map(circuit => {
          const circuitBoxers = getBoxersInCircuit(circuit.circuit_type);
          const isDragOver = dragOverCircuit === circuit.circuit_type;
          
          return (
            <div
              key={circuit.circuit_type}
              className={`circuit-card ${isDragOver ? 'drag-over' : ''}`}
              style={{
                borderColor: isDragOver ? getCircuitColor(circuit.circuit_type) : undefined,
                boxShadow: isDragOver ? `0 0 0 2px ${getCircuitColor(circuit.circuit_type)}40` : undefined,
              }}
              onDragOver={(e) => handleDragOver(e, circuit.circuit_type)}
              onDragLeave={handleDragLeave}
              onDrop={(e) => handleDrop(e, circuit.circuit_type)}
            >
              <div 
                className="circuit-header"
                style={{ background: `${getCircuitColor(circuit.circuit_type)}20` }}
              >
                <span className="circuit-icon">{getCircuitIcon(circuit.circuit_type)}</span>
                <h4>{circuit.name}</h4>
                <span className="boxer-count">{circuitBoxers.length}</span>
              </div>

              <div className="circuit-boxers">
                {circuitBoxers.length === 0 ? (
                  <div className="empty-circuit">
                    Drop boxers here
                  </div>
                ) : (
                  circuitBoxers.map(boxer => (
                    <div
                      key={boxer.fighter_id}
                      className={`circuit-boxer-item ${boxer.is_champion ? 'champion' : ''}`}
                      draggable={!disabled}
                      onDragStart={() => handleDragStart(boxer)}
                      onDragEnd={handleDragEnd}
                      style={{
                        opacity: draggedBoxer?.fighter_id === boxer.fighter_id ? 0.5 : 1,
                        borderLeft: boxer.is_champion 
                          ? `3px solid ${getCircuitColor(circuit.circuit_type)}` 
                          : undefined,
                      }}
                    >
                      <span className="drag-handle">⋮⋮</span>
                      <span className="boxer-name">{boxer.name}</span>
                      
                      <select
                        value={boxer.circuit}
                        onChange={(e) => handleCircuitChange(boxer.fighter_id, e.target.value)}
                        disabled={disabled}
                        className="circuit-selector"
                        onClick={(e) => e.stopPropagation()}
                      >
                        <option value="Minor">Minor</option>
                        <option value="Major">Major</option>
                        <option value="World">World</option>
                        <option value="Special">Special</option>
                      </select>

                      <label className="champion-checkbox" title="Circuit Champion">
                        <input
                          type="checkbox"
                          checked={boxer.is_champion}
                          onChange={(e) => onUpdateChampion(boxer.fighter_id, e.target.checked)}
                          disabled={disabled}
                        />
                        <span className="champion-star">★</span>
                      </label>
                    </div>
                  ))
                )}
              </div>
            </div>
          );
        })}
      </div>

      <div className="circuit-legend">
        <div className="circuit-legend-item">
          <span className="legend-icon">⋮⋮</span>
          <span>Drag to move</span>
        </div>
        <div className="circuit-legend-item">
          <span className="legend-star">★</span>
          <span>Champion</span>
        </div>
        <div className="circuit-legend-item">
          <span className="legend-box minor" />
          <span>Minor</span>
        </div>
        <div className="circuit-legend-item">
          <span className="legend-box major" />
          <span>Major</span>
        </div>
        <div className="circuit-legend-item">
          <span className="legend-box world" />
          <span>World</span>
        </div>
        <div className="circuit-legend-item">
          <span className="legend-box special" />
          <span>Special</span>
        </div>
      </div>

      {/* Additional styles */}
      <style>{`
        .circuit-card.drag-over {
          background: var(--blue-10, rgba(59, 130, 246, 0.1));
        }
        
        .empty-circuit {
          padding: 2rem;
          text-align: center;
          color: var(--text-dim);
          font-size: 0.875rem;
          border: 2px dashed var(--border);
          border-radius: 6px;
          margin: 0.5rem;
        }
        
        .drag-handle {
          color: var(--text-dim);
          font-size: 0.75rem;
          cursor: grab;
          user-select: none;
          padding: 0 0.25rem;
        }
        
        .circuit-boxer-item.champion {
          background: linear-gradient(90deg, var(--panel-bg) 0%, rgba(251, 191, 36, 0.1) 100%);
        }
        
        .champion-checkbox {
          display: flex;
          align-items: center;
          justify-content: center;
          width: 24px;
          height: 24px;
          cursor: pointer;
        }
        
        .champion-checkbox input {
          display: none;
        }
        
        .champion-star {
          font-size: 1rem;
          color: var(--text-dim);
          opacity: 0.3;
          transition: all 0.2s;
        }
        
        .champion-checkbox input:checked + .champion-star {
          color: #fbbf24;
          opacity: 1;
        }
        
        .champion-checkbox:hover .champion-star {
          opacity: 0.7;
        }
        
        .boxer-count {
          padding: 0.125rem 0.5rem;
          background: var(--panel-bg);
          border-radius: 12px;
          font-size: 0.75rem;
          font-weight: 600;
          color: var(--text-dim);
        }
        
        .legend-icon {
          color: var(--text-dim);
          font-size: 0.875rem;
        }
        
        .legend-star {
          color: #fbbf24;
          font-size: 1rem;
        }
        
        .legend-box {
          width: 12px;
          height: 12px;
          border-radius: 2px;
        }
        
        .legend-box.minor { background: #4ade80; }
        .legend-box.major { background: #60a5fa; }
        .legend-box.world { background: #fbbf24; }
        .legend-box.special { background: #f87171; }
      `}</style>
    </div>
  );
}
