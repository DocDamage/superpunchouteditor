import type { SimilarBoxerData } from './BoxerCompare';
import './SimilarBoxers.css';

interface SimilarBoxersProps {
  boxers: SimilarBoxerData[];
  referenceName: string;
  onUseAsTemplate: (boxerKey: string) => void;
}

export function SimilarBoxers({
  boxers,
  referenceName,
  onUseAsTemplate,
}: SimilarBoxersProps) {
  if (boxers.length === 0) {
    return (
      <div className="similar-boxers empty">
        <p>No similar boxers found.</p>
        <p className="hint">Try selecting a different reference boxer.</p>
      </div>
    );
  }

  const getScoreClass = (score: number): string => {
    if (score >= 0.8) return 'score-high';
    if (score >= 0.5) return 'score-medium';
    return 'score-low';
  };

  const getScoreLabel = (score: number): string => {
    if (score >= 0.8) return 'Very Similar';
    if (score >= 0.5) return 'Similar';
    if (score >= 0.3) return 'Somewhat Similar';
    return 'Different';
  };

  return (
    <div className="similar-boxers">
      <header className="similar-header">
        <h3>Boxers Similar to {referenceName}</h3>
        <p className="similar-description">
          These boxers share characteristics with {referenceName} and could be used as templates.
        </p>
      </header>

      <div className="similar-list">
        {boxers.map((boxer, index) => (
          <div
            key={boxer.boxer_key}
            className={`similar-card ${getScoreClass(boxer.similarity_score)}`}
            style={{ animationDelay: `${index * 0.1}s` }}
          >
            <div className="similar-rank">
              #{index + 1}
            </div>

            <div className="similar-info">
              <h4 className="similar-name">{boxer.boxer_name}</h4>
              <p className="similar-reason">{boxer.reason}</p>
            </div>

            <div className="similar-score-section">
              <div className={`similar-score ${getScoreClass(boxer.similarity_score)}`}>
                <div className="score-ring">
                  <svg viewBox="0 0 36 36">
                    <path
                      className="score-ring-bg"
                      d="M18 2.0845
                        a 15.9155 15.9155 0 0 1 0 31.831
                        a 15.9155 15.9155 0 0 1 0 -31.831"
                    />
                    <path
                      className="score-ring-fill"
                      strokeDasharray={`${boxer.similarity_percentage}, 100`}
                      d="M18 2.0845
                        a 15.9155 15.9155 0 0 1 0 31.831
                        a 15.9155 15.9155 0 0 1 0 -31.831"
                    />
                  </svg>
                  <span className="score-value">{boxer.similarity_percentage}%</span>
                </div>
                <span className="score-label">{getScoreLabel(boxer.similarity_score)}</span>
              </div>
            </div>

            <div className="similar-actions">
              <button
                className="use-template-btn"
                onClick={() => onUseAsTemplate(boxer.boxer_key)}
                title={`Compare ${referenceName} with ${boxer.boxer_name}`}
              >
                Compare
              </button>
            </div>
          </div>
        ))}
      </div>

      <div className="similar-legend">
        <h4>Similarity Guide</h4>
        <div className="legend-items">
          <div className="legend-item">
            <span className="legend-dot score-high"></span>
            <span>80-100%: Very Similar - Great template candidate</span>
          </div>
          <div className="legend-item">
            <span className="legend-dot score-medium"></span>
            <span>50-79%: Similar - Good starting point</span>
          </div>
          <div className="legend-item">
            <span className="legend-dot score-low"></span>
            <span>0-49%: Different - Unique characteristics</span>
          </div>
        </div>
      </div>
    </div>
  );
}
