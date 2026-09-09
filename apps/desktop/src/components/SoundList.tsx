/**
 * Sound Effects List Component
 * 
 * Displays a list of sound effects with preview, export, and edit options.
 * 
 * @module SoundList
 */

import { useState, useMemo } from 'react';
import type { SoundEntry } from './AudioEditor';
import './SoundList.css';

interface SoundListProps {
  sounds: SoundEntry[];
  loading: boolean;
  currentPlayingId: number | null;
  playbackState: 'stopped' | 'playing' | 'paused';
  onPreview: (soundId: number) => void;
  onStop: () => void;
  onExport: (soundId: number, format: 'wav' | 'brr') => void;
  onImport: () => void;
}

export const SoundList = ({
  sounds,
  loading,
  currentPlayingId,
  playbackState,
  onPreview,
  onStop,
  onExport,
  onImport,
}: SoundListProps) => {
  const [selectedSound, setSelectedSound] = useState<SoundEntry | null>(null);
  const [filterCategory, setFilterCategory] = useState<string>('all');
  const [searchTerm, setSearchTerm] = useState('');

  // Get unique categories
  const categories = useMemo(() => {
    const cats = new Set(sounds.map(s => s.category));
    return ['all', ...Array.from(cats).sort()];
  }, [sounds]);

  // Filter sounds
  const filteredSounds = useMemo(() => {
    return sounds.filter(sound => {
      const matchesCategory = filterCategory === 'all' || sound.category === filterCategory;
      const matchesSearch = sound.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
                           sound.id.toString().includes(searchTerm);
      return matchesCategory && matchesSearch;
    });
  }, [sounds, filterCategory, searchTerm]);

  // Group by category
  const groupedSounds = useMemo(() => {
    const groups: Record<string, SoundEntry[]> = {};
    filteredSounds.forEach(sound => {
      if (!groups[sound.category]) {
        groups[sound.category] = [];
      }
      groups[sound.category].push(sound);
    });
    return groups;
  }, [filteredSounds]);

  const formatDuration = (ms: number): string => {
    if (ms < 1000) return `${ms}ms`;
    return `${(ms / 1000).toFixed(1)}s`;
  };

  const formatSize = (bytes: number): string => {
    if (bytes < 1024) return `${bytes}B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)}KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)}MB`;
  };

  if (loading) {
    return (
      <div className="sound-list-loading">
        <div className="loading-spinner">🎵</div>
        <p>Loading sounds...</p>
      </div>
    );
  }

  return (
    <div className="sound-list">
      <header className="sound-list-header">
        <div className="sound-list-filters">
          <input
            type="text"
            placeholder="Search sounds..."
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
            className="sound-search-input"
          />
          <select
            value={filterCategory}
            onChange={(e) => setFilterCategory(e.target.value)}
            className="sound-category-select"
          >
            {categories.map(cat => (
              <option key={cat} value={cat}>
                {cat.charAt(0).toUpperCase() + cat.slice(1)}
              </option>
            ))}
          </select>
        </div>
        <button onClick={onImport} className="import-sound-btn">
          + Import WAV
        </button>
      </header>

      <div className="sound-list-content">
        <div className="sound-list-sidebar">
          {Object.entries(groupedSounds).map(([category, categorySounds]) => (
            <div key={category} className="sound-category">
              <h4 className="sound-category-title">
                {category.charAt(0).toUpperCase() + category.slice(1)}
                <span className="sound-count">({categorySounds.length})</span>
              </h4>
              <ul className="sound-items">
                {categorySounds.map(sound => (
                  <li
                    key={sound.id}
                    className={`sound-item ${selectedSound?.id === sound.id ? 'selected' : ''} ${
                      currentPlayingId === sound.id ? 'playing' : ''
                    }`}
                    onClick={() => setSelectedSound(sound)}
                  >
                    <span className="sound-icon">
                      {currentPlayingId === sound.id && playbackState === 'playing' ? '▶️' : '🔊'}
                    </span>
                    <span className="sound-name">{sound.name}</span>
                    <span className="sound-id">${sound.id.toString(16).padStart(2, '0')}</span>
                  </li>
                ))}
              </ul>
            </div>
          ))}
        </div>

        <div className="sound-list-detail">
          {selectedSound ? (
            <div className="sound-detail">
              <header className="sound-detail-header">
                <h3>{selectedSound.name}</h3>
                <span className="sound-detail-id">
                  ID: ${selectedSound.id.toString(16).padStart(2, '0').toUpperCase()}
                </span>
              </header>

              <div className="sound-detail-info">
                <div className="info-row">
                  <label>Category:</label>
                  <span>{selectedSound.category}</span>
                </div>
                <div className="info-row">
                  <label>Format:</label>
                  <span>BRR (SNES native)</span>
                </div>
                <div className="info-row">
                  <label>Sample Rate:</label>
                  <span>32 kHz (estimated)</span>
                </div>
                <div className="info-row">
                  <label>Duration:</label>
                  <span>{formatDuration(selectedSound.duration_ms)}</span>
                </div>
                <div className="info-row">
                  <label>Size:</label>
                  <span>{formatSize(selectedSound.size_bytes)}</span>
                </div>
                <div className="info-row">
                  <label>Loop:</label>
                  <span>No</span>
                </div>
                <div className="info-row">
                  <label>Sample ID:</label>
                  <span>${selectedSound.sample_id.toString(16).padStart(2, '0').toUpperCase()}</span>
                </div>
              </div>

              <div className="sound-detail-actions">
                <button
                  className={`preview-btn ${currentPlayingId === selectedSound.id && playbackState === 'playing' ? 'active' : ''}`}
                  onClick={() => onPreview(selectedSound.id)}
                >
                  {currentPlayingId === selectedSound.id && playbackState === 'playing' 
                    ? '⏸ Pause' 
                    : '▶ Preview'}
                </button>
                <button className="stop-btn" onClick={onStop}>⏹ Stop</button>
                <div className="export-dropdown">
                  <button className="export-btn">Export ▼</button>
                  <div className="export-menu">
                    <button onClick={() => onExport(selectedSound.id, 'wav')}>
                      Export as WAV
                    </button>
                    <button onClick={() => onExport(selectedSound.id, 'brr')}>
                      Export as BRR
                    </button>
                  </div>
                </div>
                <button className="edit-btn" disabled>
                  ✏️ Edit (Coming Soon)
                </button>
              </div>

              <div className="sound-detail-note">
                <h4>⚠️ Research Required</h4>
                <p>
                  To fully support sound editing, the following ROM addresses need to be identified:
                </p>
                <ul>
                  <li>Sample table location</li>
                  <li>Sound effect ID mapping</li>
                  <li>SPC700 engine initialization</li>
                </ul>
              </div>
            </div>
          ) : (
            <div className="sound-detail-empty">
              <p>Select a sound to view details</p>
            </div>
          )}
        </div>
      </div>

      <footer className="sound-list-footer">
        <span>Showing {filteredSounds.length} of {sounds.length} sounds</span>
      </footer>
    </div>
  );
};

export default SoundList;
