/**
 * Simulation Preview
 * 
 * Component for displaying AI simulation results including difficulty rating,
 * pattern usage statistics, and fight predictions.
 */

import { SimulationResult, DifficultyRating } from '../types/aiBehavior';
import './SimulationPreview.css';

interface SimulationPreviewProps {
  result: SimulationResult | null;
  isRunning: boolean;
  onRun: () => void;
}

export function SimulationPreview({ result, isRunning, onRun }: SimulationPreviewProps) {
  if (isRunning) {
    return (
      <div className="simulation-preview loading">
        <div className="simulation-spinner">
          <div className="spinner-ring" />
          <div className="spinner-ring" />
          <div className="spinner-ring" />
        </div>
        <h3>Running AI Simulation...</h3>
        <p>Simulating 100 fights to analyze behavior patterns</p>
      </div>
    );
  }

  if (!result) {
    return (
      <div className="simulation-preview empty">
        <div className="simulation-icon">🎮</div>
        <h3>Test AI Behavior</h3>
        <p>
          Run a simulation to analyze the AI's difficulty, pattern usage, and 
          estimated fight statistics.
        </p>
        <button className="run-simulation-btn" onClick={onRun}>
          Start Simulation
        </button>
        <div className="simulation-info">
          <h4>What gets tested:</h4>
          <ul>
            <li>Average fight duration</li>
            <li>Damage dealt/received</li>
            <li>Pattern selection frequency</li>
            <li>Overall difficulty rating</li>
            <li>Estimated player win rate</li>
          </ul>
        </div>
      </div>
    );
  }

  const difficultyInfo = getDifficultyInfo(result.difficulty_rating);

  return (
    <div className="simulation-preview results">
      <header className="results-header">
        <h3>Simulation Results</h3>
        <button className="rerun-btn" onClick={onRun}>
          🔄 Re-run
        </button>
      </header>

      <div className="results-grid">
        {/* Difficulty Rating Card */}
        <div className="result-card difficulty-card">
          <h4>Difficulty Rating</h4>
          <div 
            className="difficulty-display"
            style={{ 
              background: `linear-gradient(135deg, ${difficultyInfo.color}40, ${difficultyInfo.color}20)`,
              borderColor: difficultyInfo.color 
            }}
          >
            <span 
              className="difficulty-rating"
              style={{ color: difficultyInfo.color }}
            >
              {difficultyInfo.label}
            </span>
            <div className="win-rate">
              <span className="win-rate-label">AI Win Rate</span>
              <span className="win-rate-value">
                {result.estimated_win_rate.toFixed(1)}%
              </span>
            </div>
          </div>
        </div>

        {/* Fight Time Card */}
        <div className="result-card">
          <h4>Avg. Fight Time</h4>
          <div className="stat-display">
            <span className="stat-value">
              {formatTime(result.average_fight_time)}
            </span>
            <TimeBar seconds={result.average_fight_time} />
          </div>
        </div>

        {/* Damage Stats Card */}
        <div className="result-card">
          <h4>Damage Statistics</h4>
          <div className="damage-stats">
            <div className="damage-row">
              <span className="damage-label">Player Takes</span>
              <span className="damage-value player-damage">
                {result.player_damage_taken.toFixed(1)}
              </span>
            </div>
            <div className="damage-row">
              <span className="damage-label">AI Deals</span>
              <span className="damage-value ai-damage">
                {result.ai_damage_dealt.toFixed(1)}
              </span>
            </div>
            <div className="damage-ratio">
              Ratio: {(result.ai_damage_dealt / Math.max(result.player_damage_taken, 1)).toFixed(2)}:1
            </div>
          </div>
        </div>

        {/* Pattern Usage Card */}
        <div className="result-card patterns-card">
          <h4>Pattern Usage</h4>
          <div className="pattern-usage-list">
            {Object.entries(result.pattern_usage)
              .sort(([,a], [,b]) => b - a)
              .map(([patternId, usage]) => (
                <div key={patternId} className="usage-row">
                  <span className="pattern-id">{patternId}</span>
                  <div className="usage-bar">
                    <div 
                      className="usage-fill"
                      style={{ width: `${usage}%` }}
                    />
                  </div>
                  <span className="usage-percent">{usage.toFixed(1)}%</span>
                </div>
              ))}
            {Object.keys(result.pattern_usage).length === 0 && (
              <p className="no-patterns">No patterns recorded</p>
            )}
          </div>
        </div>

        {/* Insights Card */}
        {result.insights.length > 0 && (
          <div className="result-card insights-card">
            <h4>💡 Insights</h4>
            <ul className="insights-list">
              {result.insights.map((insight, index) => (
                <li key={index} className="insight-item">
                  {insight}
                </li>
              ))}
            </ul>
          </div>
        )}

        {/* Warnings Card */}
        {result.warnings.length > 0 && (
          <div className="result-card warnings-card">
            <h4>⚠️ Warnings</h4>
            <ul className="warnings-list">
              {result.warnings.map((warning, index) => (
                <li key={index} className="warning-item">
                  {warning}
                </li>
              ))}
            </ul>
          </div>
        )}
      </div>

      <div className="simulation-footer">
        <p>
          Simulation ran 100 test fights with varying player skill levels.
          Results are estimates based on current AI configuration.
        </p>
      </div>
    </div>
  );
}

// Helper functions
function getDifficultyInfo(rating: DifficultyRating): { label: string; color: string } {
  const difficultyMap: Record<DifficultyRating, { label: string; color: string }> = {
    'VeryEasy': { label: 'Very Easy', color: '#4ade80' },
    'Easy': { label: 'Easy', color: '#60a5fa' },
    'Medium': { label: 'Medium', color: '#fbbf24' },
    'Hard': { label: 'Hard', color: '#fb923c' },
    'VeryHard': { label: 'Very Hard', color: '#f87171' },
    'Extreme': { label: 'Extreme', color: '#a855f7' },
  };
  
  return difficultyMap[rating] || { label: 'Unknown', color: '#9ca3af' };
}

function formatTime(seconds: number): string {
  const mins = Math.floor(seconds / 60);
  const secs = Math.floor(seconds % 60);
  return `${mins}:${secs.toString().padStart(2, '0')}`;
}

// Time Bar Component
interface TimeBarProps {
  seconds: number;
}

function TimeBar({ seconds }: TimeBarProps) {
  // Normalize to 3 minutes (180 seconds) max
  const percentage = Math.min((seconds / 180) * 100, 100);
  
  let color = '#4ade80'; // Green for short fights
  if (seconds > 60) color = '#fbbf24'; // Yellow
  if (seconds > 120) color = '#f87171'; // Red for long fights
  
  return (
    <div className="time-bar">
      <div 
        className="time-fill"
        style={{ 
          width: `${percentage}%`,
          backgroundColor: color
        }}
      />
      <div className="time-markers">
        <span>0s</span>
        <span>90s</span>
        <span>180s</span>
      </div>
    </div>
  );
}
