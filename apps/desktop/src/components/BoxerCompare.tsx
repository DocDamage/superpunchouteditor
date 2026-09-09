import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useStore, type BoxerRecord } from '../store/useStore';
import { StatComparisonTable } from './StatComparisonTable';
import { SimilarBoxers } from './SimilarBoxers';
import './BoxerCompare.css';

export interface ComparisonData {
  boxer_a: string;
  boxer_b: string;
  boxer_a_key: string;
  boxer_b_key: string;
  stat_comparison: {
    attack: [number, number];
    defense: [number, number];
    speed: [number, number];
    palette_id: [number, number];
    differences: string[];
  };
  asset_comparison: {
    unique_a: string[];
    unique_b: string[];
    shared: string[];
    total_size_a: number;
    total_size_b: number;
    unique_count_a: number;
    unique_count_b: number;
    shared_count: number;
  };
  palette_comparison: Array<{
    name: string;
    file_a: string;
    file_b: string;
    size_a: number;
    size_b: number;
    color_count_a: number;
    color_count_b: number;
    differences: number[];
  }>;
  similarity_score: number;
}

export interface SimilarBoxerData {
  boxer_name: string;
  boxer_key: string;
  similarity_score: number;
  similarity_percentage: number;
  reason: string;
}

export function BoxerCompare() {
  const { boxers, selectedBoxer } = useStore();
  const [boxerA, setBoxerA] = useState<BoxerRecord | null>(null);
  const [boxerB, setBoxerB] = useState<BoxerRecord | null>(null);
  const [comparison, setComparison] = useState<ComparisonData | null>(null);
  const [similarBoxers, setSimilarBoxers] = useState<SimilarBoxerData[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<'stats' | 'assets' | 'similar'>('stats');

  // Initialize boxer A to selected boxer
  useEffect(() => {
    if (selectedBoxer && !boxerA) {
      setBoxerA(selectedBoxer);
    }
  }, [selectedBoxer, boxerA]);

  // Load comparison when both boxers are selected
  const loadComparison = useCallback(async () => {
    if (!boxerA || !boxerB) return;
    
    setIsLoading(true);
    setError(null);
    
    try {
      const result = await invoke<ComparisonData>('compare_boxers', {
        boxerAKey: boxerA.key,
        boxerBKey: boxerB.key,
      });
      setComparison(result);
    } catch (e) {
      setError(`Failed to compare boxers: ${e}`);
    } finally {
      setIsLoading(false);
    }
  }, [boxerA, boxerB]);

  useEffect(() => {
    loadComparison();
  }, [loadComparison]);

  // Load similar boxers when boxer A changes
  const loadSimilarBoxers = useCallback(async () => {
    if (!boxerA) return;
    
    try {
      const result = await invoke<SimilarBoxerData[]>('get_similar_boxers', {
        referenceKey: boxerA.key,
        limit: 5,
      });
      setSimilarBoxers(result);
    } catch (e) {
      console.error('Failed to load similar boxers:', e);
    }
  }, [boxerA]);

  useEffect(() => {
    loadSimilarBoxers();
  }, [loadSimilarBoxers]);

  const handleSwap = () => {
    const temp = boxerA;
    setBoxerA(boxerB);
    setBoxerB(temp);
  };

  const handleCopyStat = async (field: string, source: 'a' | 'b') => {
    if (!boxerA || !boxerB) return;
    
    const sourceKey = source === 'a' ? boxerA.key : boxerB.key;
    const targetKey = source === 'a' ? boxerB.key : boxerA.key;
    
    try {
      await invoke('copy_boxer_stat', {
        sourceKey,
        targetKey,
        statField: field,
      });
      // Refresh comparison
      await loadComparison();
    } catch (e) {
      setError(`Failed to copy stat: ${e}`);
    }
  };

  const handleCopyAllStats = async (source: 'a' | 'b') => {
    if (!boxerA || !boxerB) return;
    
    const sourceKey = source === 'a' ? boxerA.key : boxerB.key;
    const targetKey = source === 'a' ? boxerB.key : boxerA.key;
    
    try {
      await invoke('copy_all_boxer_stats', {
        sourceKey,
        targetKey,
      });
      // Refresh comparison
      await loadComparison();
    } catch (e) {
      setError(`Failed to copy stats: ${e}`);
    }
  };

  const handleUseAsTemplate = (boxerKey: string) => {
    const selected = boxers.find(b => b.key === boxerKey);
    if (selected) {
      setBoxerB(selected);
      setActiveTab('stats');
    }
  };

  const formatSize = (bytes: number) => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  };

  return (
    <div className="boxer-compare">
      <header className="compare-header">
        <h2>Boxer Compare Mode</h2>
        <p className="compare-description">
          Compare stats, graphics, and assets between different boxers. 
          Useful for balancing and creating new boxers based on existing templates.
        </p>
      </header>

      <section className="compare-selector">
        <div className="selector-row">
          <div className="selector-group">
            <label>Compare:</label>
            <select
              value={boxerA?.key || ''}
              onChange={(e) => {
                const selected = boxers.find(b => b.key === e.target.value);
                setBoxerA(selected || null);
              }}
            >
              <option value="">Select boxer...</option>
              {boxers.map((boxer) => (
                <option key={boxer.key} value={boxer.key}>
                  {boxer.name}
                </option>
              ))}
            </select>
          </div>

          <span className="vs-label">with</span>

          <div className="selector-group">
            <select
              value={boxerB?.key || ''}
              onChange={(e) => {
                const selected = boxers.find(b => b.key === e.target.value);
                setBoxerB(selected || null);
              }}
            >
              <option value="">Select boxer...</option>
              {boxers.map((boxer) => (
                <option key={boxer.key} value={boxer.key}>
                  {boxer.name}
                </option>
              ))}
            </select>
          </div>

          <button
            className="swap-btn"
            onClick={handleSwap}
            disabled={!boxerA || !boxerB}
            title="Swap boxers"
          >
            ↔ Swap
          </button>
        </div>
      </section>

      {error && (
        <div className="compare-error">
          {error}
        </div>
      )}

      {isLoading && (
        <div className="compare-loading">
          <div className="spinner"></div>
          <span>Comparing boxers...</span>
        </div>
      )}

      {comparison && boxerA && boxerB && !isLoading && (
        <>
          <nav className="compare-tabs">
            <button
              className={activeTab === 'stats' ? 'active' : ''}
              onClick={() => setActiveTab('stats')}
            >
              📊 Stats
            </button>
            <button
              className={activeTab === 'assets' ? 'active' : ''}
              onClick={() => setActiveTab('assets')}
            >
              📁 Assets
            </button>
            <button
              className={activeTab === 'similar' ? 'active' : ''}
              onClick={() => setActiveTab('similar')}
            >
              🔍 Similar Boxers
            </button>
          </nav>

          <div className="compare-content">
            {activeTab === 'stats' && (
              <StatComparisonTable
                comparison={comparison}
                boxerAName={boxerA.name}
                boxerBName={boxerB.name}
                onCopyStat={handleCopyStat}
                onCopyAll={handleCopyAllStats}
              />
            )}

            {activeTab === 'assets' && (
              <section className="asset-comparison">
                <div className="asset-summary">
                  <div className="asset-card">
                    <h4>{boxerA.name}</h4>
                    <div className="asset-stats">
                      <div className="stat-item">
                        <span className="stat-value">{comparison.asset_comparison.unique_count_a}</span>
                        <span className="stat-label">Unique Bins</span>
                      </div>
                      <div className="stat-item">
                        <span className="stat-value">{comparison.asset_comparison.shared_count}</span>
                        <span className="stat-label">Shared Bins</span>
                      </div>
                      <div className="stat-item">
                        <span className="stat-value">{formatSize(comparison.asset_comparison.total_size_a)}</span>
                        <span className="stat-label">Total Size</span>
                      </div>
                    </div>
                  </div>

                  <div className="similarity-badge">
                    <div className="similarity-score">
                      {Math.round(comparison.similarity_score * 100)}%
                    </div>
                    <span>Similar</span>
                  </div>

                  <div className="asset-card">
                    <h4>{boxerB.name}</h4>
                    <div className="asset-stats">
                      <div className="stat-item">
                        <span className="stat-value">{comparison.asset_comparison.unique_count_b}</span>
                        <span className="stat-label">Unique Bins</span>
                      </div>
                      <div className="stat-item">
                        <span className="stat-value">{comparison.asset_comparison.shared_count}</span>
                        <span className="stat-label">Shared Bins</span>
                      </div>
                      <div className="stat-item">
                        <span className="stat-value">{formatSize(comparison.asset_comparison.total_size_b)}</span>
                        <span className="stat-label">Total Size</span>
                      </div>
                    </div>
                  </div>
                </div>

                <div className="asset-details">
                  {comparison.asset_comparison.unique_a.length > 0 && (
                    <div className="asset-list unique-a">
                      <h5>Only in {boxerA.name}</h5>
                      <ul>
                        {comparison.asset_comparison.unique_a.slice(0, 10).map((file, i) => (
                          <li key={i}>{file}</li>
                        ))}
                        {comparison.asset_comparison.unique_a.length > 10 && (
                          <li className="more-items">
                            ...and {comparison.asset_comparison.unique_a.length - 10} more
                          </li>
                        )}
                      </ul>
                    </div>
                  )}

                  {comparison.asset_comparison.unique_b.length > 0 && (
                    <div className="asset-list unique-b">
                      <h5>Only in {boxerB.name}</h5>
                      <ul>
                        {comparison.asset_comparison.unique_b.slice(0, 10).map((file, i) => (
                          <li key={i}>{file}</li>
                        ))}
                        {comparison.asset_comparison.unique_b.length > 10 && (
                          <li className="more-items">
                            ...and {comparison.asset_comparison.unique_b.length - 10} more
                          </li>
                        )}
                      </ul>
                    </div>
                  )}

                  {comparison.asset_comparison.shared.length > 0 && (
                    <div className="asset-list shared">
                      <h5>Shared Assets</h5>
                      <ul>
                        {comparison.asset_comparison.shared.slice(0, 10).map((file, i) => (
                          <li key={i}>{file}</li>
                        ))}
                        {comparison.asset_comparison.shared.length > 10 && (
                          <li className="more-items">
                            ...and {comparison.asset_comparison.shared.length - 10} more
                          </li>
                        )}
                      </ul>
                    </div>
                  )}

                  {comparison.asset_comparison.shared.length === 0 && 
                   comparison.asset_comparison.unique_a.length === 0 && 
                   comparison.asset_comparison.unique_b.length === 0 && (
                    <div className="asset-list empty">
                      <p>No asset differences found</p>
                    </div>
                  )}
                </div>

                <div className="safety-notice">
                  {comparison.asset_comparison.shared_count === 0 ? (
                    <div className="safe-badge">
                      ✓ Both boxers have no shared bins - safe editing targets
                    </div>
                  ) : (
                    <div className="warning-badge">
                      ⚠ These boxers share {comparison.asset_comparison.shared_count} asset(s) - editing affects both
                    </div>
                  )}
                </div>
              </section>
            )}

            {activeTab === 'similar' && (
              <SimilarBoxers
                boxers={similarBoxers}
                referenceName={boxerA.name}
                onUseAsTemplate={handleUseAsTemplate}
              />
            )}
          </div>
        </>
      )}

      {!comparison && !isLoading && boxerA && boxerB && (
        <div className="compare-empty">
          <p>Select two boxers to compare them</p>
        </div>
      )}

      {!boxerA && (
        <div className="compare-empty">
          <p>Select a boxer to start comparison</p>
        </div>
      )}
    </div>
  );
}
