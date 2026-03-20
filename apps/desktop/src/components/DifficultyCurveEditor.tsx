/**
 * Difficulty Curve Editor
 * 
 * Component for editing round-by-round difficulty settings including
 * aggression, defense, speed, and pattern complexity.
 */

import { DifficultyCurve, RoundDifficulty } from '../types/aiBehavior';
import './DifficultyCurveEditor.css';

interface DifficultyCurveEditorProps {
  curve: DifficultyCurve;
  onChange: (curve: DifficultyCurve) => void;
}

export function DifficultyCurveEditor({ curve, onChange }: DifficultyCurveEditorProps) {
  const updateRound = (roundIndex: number, updates: Partial<RoundDifficulty>) => {
    const newRounds = [...curve.rounds];
    newRounds[roundIndex] = { ...newRounds[roundIndex], ...updates };
    onChange({ ...curve, rounds: newRounds });
  };

  const roundNames = ['First Round', 'Second Round', 'Third Round'];
  const roundColors = ['#4ade80', '#fbbf24', '#f87171']; // Green, Yellow, Red

  return (
    <div className="difficulty-curve-editor">
      <div className="curve-header">
        <h3>Difficulty Curve</h3>
        <div className="base-stats">
          <div className="base-stat">
            <label>Base Aggression</label>
            <div className="slider-with-value">
              <input
                type="range"
                min="0"
                max="255"
                value={curve.base_aggression}
                onChange={(e) => onChange({ ...curve, base_aggression: parseInt(e.target.value) })}
              />
              <span>{curve.base_aggression}</span>
            </div>
          </div>
          <div className="base-stat">
            <label>Base Defense</label>
            <div className="slider-with-value">
              <input
                type="range"
                min="0"
                max="255"
                value={curve.base_defense}
                onChange={(e) => onChange({ ...curve, base_defense: parseInt(e.target.value) })}
              />
              <span>{curve.base_defense}</span>
            </div>
          </div>
          <div className="base-stat">
            <label>Base Speed</label>
            <div className="slider-with-value">
              <input
                type="range"
                min="0"
                max="255"
                value={curve.base_speed}
                onChange={(e) => onChange({ ...curve, base_speed: parseInt(e.target.value) })}
              />
              <span>{curve.base_speed}</span>
            </div>
          </div>
        </div>
      </div>

      <div className="rounds-container">
        {curve.rounds.map((round, index) => (
          <RoundEditor
            key={round.round}
            round={round}
            name={roundNames[index]}
            color={roundColors[index]}
            onChange={(updates) => updateRound(index, updates)}
          />
        ))}
      </div>

      <div className="difficulty-visualization">
        <h4>Difficulty Progression</h4>
        <div className="chart">
          <svg viewBox="0 0 300 150" className="difficulty-chart">
            {/* Grid lines */}
            {[0, 50, 100, 150, 200, 250].map(y => (
              <line
                key={y}
                x1="30"
                y1={140 - (y / 255) * 120}
                x2="280"
                y2={140 - (y / 255) * 120}
                stroke="var(--border)"
                strokeWidth="0.5"
                strokeDasharray="2,2"
              />
            ))}
            
            {/* Y-axis labels */}
            {[0, 128, 255].map(y => (
              <text
                key={y}
                x="25"
                y={145 - (y / 255) * 120}
                textAnchor="end"
                fontSize="8"
                fill="var(--text-muted)"
              >
                {y}
              </text>
            ))}

            {/* X-axis labels */}
            {['R1', 'R2', 'R3'].map((label, i) => (
              <text
                key={label}
                x={50 + i * 100}
                y="155"
                textAnchor="middle"
                fontSize="10"
                fill="var(--text)"
              >
                {label}
              </text>
            ))}

            {/* Aggression line */}
            <polyline
              fill="none"
              stroke="#ef4444"
              strokeWidth="2"
              points={curve.rounds.map((r, i) => 
                `${50 + i * 100},${140 - (r.aggression / 255) * 120}`
              ).join(' ')}
            />
            {curve.rounds.map((r, i) => (
              <circle
                key={`agg-${i}`}
                cx={50 + i * 100}
                cy={140 - (r.aggression / 255) * 120}
                r="4"
                fill="#ef4444"
              />
            ))}

            {/* Defense line */}
            <polyline
              fill="none"
              stroke="#3b82f6"
              strokeWidth="2"
              points={curve.rounds.map((r, i) => 
                `${50 + i * 100},${140 - (r.defense / 255) * 120}`
              ).join(' ')}
            />
            {curve.rounds.map((r, i) => (
              <circle
                key={`def-${i}`}
                cx={50 + i * 100}
                cy={140 - (r.defense / 255) * 120}
                r="4"
                fill="#3b82f6"
              />
            ))}

            {/* Speed line */}
            <polyline
              fill="none"
              stroke="#22c55e"
              strokeWidth="2"
              points={curve.rounds.map((r, i) => 
                `${50 + i * 100},${140 - (r.speed / 255) * 120}`
              ).join(' ')}
            />
            {curve.rounds.map((r, i) => (
              <circle
                key={`spd-${i}`}
                cx={50 + i * 100}
                cy={140 - (r.speed / 255) * 120}
                r="4"
                fill="#22c55e"
              />
            ))}

            {/* Complexity line */}
            <polyline
              fill="none"
              stroke="#a855f7"
              strokeWidth="2"
              strokeDasharray="4,2"
              points={curve.rounds.map((r, i) => 
                `${50 + i * 100},${140 - (r.pattern_complexity / 255) * 120}`
              ).join(' ')}
            />
            {curve.rounds.map((r, i) => (
              <circle
                key={`cmp-${i}`}
                cx={50 + i * 100}
                cy={140 - (r.pattern_complexity / 255) * 120}
                r="3"
                fill="#a855f7"
              />
            ))}
          </svg>
          
          <div className="chart-legend">
            <div className="legend-item">
              <span className="legend-color" style={{ background: '#ef4444' }} />
              <span>Aggression</span>
            </div>
            <div className="legend-item">
              <span className="legend-color" style={{ background: '#3b82f6' }} />
              <span>Defense</span>
            </div>
            <div className="legend-item">
              <span className="legend-color" style={{ background: '#22c55e' }} />
              <span>Speed</span>
            </div>
            <div className="legend-item">
              <span className="legend-color" style={{ background: '#a855f7' }} />
              <span>Complexity</span>
            </div>
          </div>
        </div>
      </div>

      <div className="difficulty-presets">
        <h4>Quick Presets</h4>
        <div className="preset-buttons">
          <button 
            onClick={() => onChange({
              ...curve,
              rounds: [
                { round: 1, aggression: 60, defense: 70, speed: 90, pattern_complexity: 40, damage_multiplier: 80, reaction_time: 10 },
                { round: 2, aggression: 80, defense: 85, speed: 100, pattern_complexity: 70, damage_multiplier: 100, reaction_time: 8 },
                { round: 3, aggression: 100, defense: 100, speed: 110, pattern_complexity: 100, damage_multiplier: 100, reaction_time: 6 },
              ]
            })}
          >
            Steady
          </button>
          <button 
            onClick={() => onChange({
              ...curve,
              rounds: [
                { round: 1, aggression: 80, defense: 80, speed: 100, pattern_complexity: 60, damage_multiplier: 100, reaction_time: 8 },
                { round: 2, aggression: 130, defense: 110, speed: 115, pattern_complexity: 120, damage_multiplier: 110, reaction_time: 5 },
                { round: 3, aggression: 180, defense: 140, speed: 130, pattern_complexity: 180, damage_multiplier: 125, reaction_time: 3 },
              ]
            })}
          >
            Difficulty Spike
          </button>
          <button 
            onClick={() => onChange({
              ...curve,
              rounds: [
                { round: 1, aggression: 150, defense: 130, speed: 120, pattern_complexity: 150, damage_multiplier: 120, reaction_time: 4 },
                { round: 2, aggression: 100, defense: 100, speed: 100, pattern_complexity: 100, damage_multiplier: 100, reaction_time: 6 },
                { round: 3, aggression: 80, defense: 90, speed: 90, pattern_complexity: 80, damage_multiplier: 90, reaction_time: 8 },
              ]
            })}
          >
            Reverse (Desperate Start)
          </button>
        </div>
      </div>
    </div>
  );
}

// Round Editor Sub-component
interface RoundEditorProps {
  round: RoundDifficulty;
  name: string;
  color: string;
  onChange: (updates: Partial<RoundDifficulty>) => void;
}

function RoundEditor({ round, name, color, onChange }: RoundEditorProps) {
  return (
    <div className="round-editor" style={{ borderLeftColor: color }}>
      <div className="round-header" style={{ backgroundColor: `${color}20` }}>
        <h4 style={{ color }}>Round {round.round}</h4>
        <span className="round-name">{name}</span>
      </div>

      <div className="round-stats">
        <div className="stat-row">
          <label>Aggression</label>
          <div className="slider-with-value">
            <input
              type="range"
              min="0"
              max="255"
              value={round.aggression}
              onChange={(e) => onChange({ aggression: parseInt(e.target.value) })}
            />
            <span>{round.aggression}</span>
          </div>
        </div>

        <div className="stat-row">
          <label>Defense</label>
          <div className="slider-with-value">
            <input
              type="range"
              min="0"
              max="255"
              value={round.defense}
              onChange={(e) => onChange({ defense: parseInt(e.target.value) })}
            />
            <span>{round.defense}</span>
          </div>
        </div>

        <div className="stat-row">
          <label>Speed</label>
          <div className="slider-with-value">
            <input
              type="range"
              min="0"
              max="255"
              value={round.speed}
              onChange={(e) => onChange({ speed: parseInt(e.target.value) })}
            />
            <span>{round.speed}</span>
          </div>
        </div>

        <div className="stat-row">
          <label>Pattern Complexity</label>
          <div className="slider-with-value">
            <input
              type="range"
              min="0"
              max="255"
              value={round.pattern_complexity}
              onChange={(e) => onChange({ pattern_complexity: parseInt(e.target.value) })}
            />
            <span>{round.pattern_complexity}</span>
          </div>
        </div>

        <div className="stat-row compact">
          <div className="compact-stat">
            <label>Damage %</label>
            <input
              type="number"
              min="50"
              max="200"
              value={round.damage_multiplier}
              onChange={(e) => onChange({ damage_multiplier: parseInt(e.target.value) || 100 })}
            />
          </div>
          <div className="compact-stat">
            <label>Reaction (frames)</label>
            <input
              type="number"
              min="1"
              max="30"
              value={round.reaction_time}
              onChange={(e) => onChange({ reaction_time: parseInt(e.target.value) || 6 })}
            />
          </div>
        </div>
      </div>

      <div className="round-summary">
        <div className="summary-stat">
          <span className="summary-label">Est. Difficulty</span>
          <DifficultyBadge 
            aggression={round.aggression} 
            defense={round.defense} 
            speed={round.speed}
          />
        </div>
      </div>
    </div>
  );
}

// Difficulty Badge Component
interface DifficultyBadgeProps {
  aggression: number;
  defense: number;
  speed: number;
}

function DifficultyBadge({ aggression, defense, speed }: DifficultyBadgeProps) {
  const avg = (aggression + defense + speed) / 3;
  
  let label: string;
  let color: string;
  
  if (avg < 60) {
    label = 'Very Easy';
    color = '#4ade80';
  } else if (avg < 90) {
    label = 'Easy';
    color = '#60a5fa';
  } else if (avg < 120) {
    label = 'Medium';
    color = '#fbbf24';
  } else if (avg < 150) {
    label = 'Hard';
    color = '#fb923c';
  } else if (avg < 180) {
    label = 'Very Hard';
    color = '#f87171';
  } else {
    label = 'Extreme';
    color = '#a855f7';
  }
  
  return (
    <span 
      className="difficulty-badge"
      style={{ backgroundColor: color, color: '#000' }}
    >
      {label}
    </span>
  );
}
