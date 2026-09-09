/**
 * Music/Sequence Editor Component
 * 
 * Displays music tracks and provides sequence editing capabilities.
 * 
 * @module MusicEditor
 */

import { useState, useMemo } from 'react';
import type { MusicEntry } from './AudioEditor';
import './MusicEditor.css';

interface MusicEditorProps {
  tracks: MusicEntry[];
  loading: boolean;
  currentPlayingId: number | null;
  playbackState: 'stopped' | 'playing' | 'paused';
  onPreview: (musicId: number) => void;
  onStop: () => void;
  onExport: (musicId: number, format: 'wav' | 'spc') => void;
}

export const MusicEditor = ({
  tracks,
  loading,
  currentPlayingId,
  playbackState,
  onPreview,
  onStop,
  onExport,
}: MusicEditorProps) => {
  const [selectedTrack, setSelectedTrack] = useState<MusicEntry | null>(null);
  const [filterContext, setFilterContext] = useState<string>('all');
  const [searchTerm, setSearchTerm] = useState('');

  // Get unique contexts
  const contexts = useMemo(() => {
    const ctxs = new Set(tracks.map(t => t.play_context));
    return ['all', ...Array.from(ctxs).sort()];
  }, [tracks]);

  // Filter tracks
  const filteredTracks = useMemo(() => {
    return tracks.filter(track => {
      const matchesContext = filterContext === 'all' || track.play_context === filterContext;
      const matchesSearch = track.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
                           track.id.toString().includes(searchTerm);
      return matchesContext && matchesSearch;
    });
  }, [tracks, filterContext, searchTerm]);

  // Group by context
  const groupedTracks = useMemo(() => {
    const groups: Record<string, MusicEntry[]> = {};
    filteredTracks.forEach(track => {
      if (!groups[track.play_context]) {
        groups[track.play_context] = [];
      }
      groups[track.play_context].push(track);
    });
    return groups;
  }, [filteredTracks]);

  // Get track type icon
  const getTrackIcon = (context: string): string => {
    switch (context) {
      case 'title': return '🎬';
      case 'menu': return '📋';
      case 'match': return '🥊';
      case 'boxer': return '👤';
      case 'training': return '🏋️';
      case 'ending': return '🎬';
      default: return '🎵';
    }
  };

  if (loading) {
    return (
      <div className="music-editor-loading">
        <div className="loading-spinner">🎼</div>
        <p>Loading music tracks...</p>
      </div>
    );
  }

  return (
    <div className="music-editor">
      <header className="music-editor-header">
        <div className="music-editor-filters">
          <input
            type="text"
            placeholder="Search tracks..."
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
            className="music-search-input"
          />
          <select
            value={filterContext}
            onChange={(e) => setFilterContext(e.target.value)}
            className="music-context-select"
          >
            {contexts.map(ctx => (
              <option key={ctx} value={ctx}>
                {ctx.charAt(0).toUpperCase() + ctx.slice(1)}
              </option>
            ))}
          </select>
        </div>
      </header>

      <div className="music-editor-content">
        <div className="music-list-sidebar">
          {Object.entries(groupedTracks).map(([context, contextTracks]) => (
            <div key={context} className="music-context-group">
              <h4 className="music-context-title">
                <span className="context-icon">{getTrackIcon(context)}</span>
                {context.charAt(0).toUpperCase() + context.slice(1)}
                <span className="track-count">({contextTracks.length})</span>
              </h4>
              <ul className="music-items">
                {contextTracks.map(track => (
                  <li
                    key={track.id}
                    className={`music-item ${selectedTrack?.id === track.id ? 'selected' : ''} ${
                      currentPlayingId === track.id ? 'playing' : ''
                    }`}
                    onClick={() => setSelectedTrack(track)}
                  >
                    <span className="music-icon">
                      {currentPlayingId === track.id && playbackState === 'playing' ? '▶️' : '🎵'}
                    </span>
                    <div className="music-info">
                      <span className="music-name">{track.name}</span>
                      <span className="music-meta">
                        {track.tempo} BPM • {track.channel_count} ch
                      </span>
                    </div>
                    <span className="music-id">${track.id.toString(16).padStart(2, '0')}</span>
                  </li>
                ))}
              </ul>
            </div>
          ))}
        </div>

        <div className="music-detail-panel">
          {selectedTrack ? (
            <div className="music-detail">
              <header className="music-detail-header">
                <h3>{selectedTrack.name}</h3>
                <span className="music-detail-id">
                  ID: ${selectedTrack.id.toString(16).padStart(2, '0').toUpperCase()}
                </span>
              </header>

              <div className="music-detail-info">
                <div className="info-grid">
                  <div className="info-item">
                    <label>Tempo</label>
                    <span>{selectedTrack.tempo} BPM</span>
                  </div>
                  <div className="info-item">
                    <label>Channels</label>
                    <span>{selectedTrack.channel_count}</span>
                  </div>
                  <div className="info-item">
                    <label>Type</label>
                    <span>{selectedTrack.track_type}</span>
                  </div>
                  <div className="info-item">
                    <label>Context</label>
                    <span>{selectedTrack.play_context}</span>
                  </div>
                </div>

                {selectedTrack.associated_boxer && (
                  <div className="info-row boxer-info">
                    <label>Boxer Theme:</label>
                    <span>{selectedTrack.associated_boxer}</span>
                  </div>
                )}
              </div>

              <div className="music-detail-actions">
                <button
                  className={`preview-btn ${currentPlayingId === selectedTrack.id && playbackState === 'playing' ? 'active' : ''}`}
                  onClick={() => onPreview(selectedTrack.id)}
                >
                  {currentPlayingId === selectedTrack.id && playbackState === 'playing' 
                    ? '⏸ Pause' 
                    : '▶ Preview'}
                </button>
                <button className="stop-btn" onClick={onStop}>⏹ Stop</button>
                <div className="export-dropdown">
                  <button className="export-btn">Export ▼</button>
                  <div className="export-menu">
                    <button onClick={() => onExport(selectedTrack.id, 'spc')}>
                      Export as SPC
                    </button>
                    <button onClick={() => onExport(selectedTrack.id, 'wav')}>
                      Export as WAV (60s)
                    </button>
                  </div>
                </div>
              </div>

              <div className="sequence-editor-placeholder">
                <h4>🎹 Sequence Editor</h4>
                <p>
                  Music sequence editing requires reverse engineering of the SPO music format.
                </p>
                
                <div className="sequence-preview">
                  <div className="channel-strip">
                    <label>CH1</label>
                    <div className="channel-visualization">
                      <div className="note-bars" style={{ 
                        background: 'linear-gradient(90deg, #4ade80 20%, #22c55e 40%, #16a34a 60%, #4ade80 80%)' 
                      }} />
                    </div>
                  </div>
                  <div className="channel-strip">
                    <label>CH2</label>
                    <div className="channel-visualization">
                      <div className="note-bars" style={{ 
                        background: 'linear-gradient(90deg, #60a5fa 10%, #3b82f6 30%, #2563eb 50%, #60a5fa 70%)' 
                      }} />
                    </div>
                  </div>
                  <div className="channel-strip">
                    <label>CH3</label>
                    <div className="channel-visualization">
                      <div className="note-bars" style={{ 
                        background: 'linear-gradient(90deg, #f472b6 15%, #ec4899 35%, #db2777 55%, #f472b6 75%)' 
                      }} />
                    </div>
                  </div>
                  <div className="channel-strip muted">
                    <label>CH4</label>
                    <div className="channel-visualization">
                      <div className="note-bars" style={{ 
                        background: 'linear-gradient(90deg, #9ca3af 0%, #6b7280 100%)' 
                      }} />
                    </div>
                  </div>
                </div>

                <div className="research-todo">
                  <h5>Research TODOs:</h5>
                  <ul>
                    <li>Identify sequence data format</li>
                    <li>Map music IDs to sequence addresses</li>
                    <li>Document tempo/timing format</li>
                    <li>Implement pattern/loop detection</li>
                    <li>Create piano roll editor UI</li>
                  </ul>
                </div>
              </div>
            </div>
          ) : (
            <div className="music-detail-empty">
              <p>Select a music track to view details</p>
            </div>
          )}
        </div>
      </div>

      <footer className="music-editor-footer">
        <span>Showing {filteredTracks.length} of {tracks.length} tracks</span>
        <span className="format-info">
          Format: SPC700 / BRR
        </span>
      </footer>
    </div>
  );
};

export default MusicEditor;
