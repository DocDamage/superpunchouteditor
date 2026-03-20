/**
 * Sound & Music Editor for Super Punch-Out!!
 * 
 * Provides a UI for browsing, previewing, and editing SPC700 audio data.
 * 
 * @module AudioEditor
 */

import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { open, save } from '@tauri-apps/plugin-dialog';
import { SoundList } from './SoundList';
import { MusicEditor } from './MusicEditor';
import './AudioEditor.css';

export interface SoundEntry {
  id: number;
  name: string;
  category: string;
  sample_id: number;
  size_bytes: number;
  duration_ms: number;
  associated_music?: number;
}

export interface MusicEntry {
  id: number;
  name: string;
  track_type: string;
  tempo: number;
  channel_count: number;
  associated_boxer?: string;
  play_context: string;
}

export interface SampleDetail {
  id: number;
  name: string;
  format: string;
  sample_rate: number;
  loop_enabled: boolean;
  duration_ms: number;
  size_bytes: number;
  has_loop: boolean;
  loop_start: number;
  loop_end: number;
}

export type AudioTab = 'sounds' | 'music' | 'samples' | 'spc';

export const AudioEditor = () => {
  const [activeTab, setActiveTab] = useState<AudioTab>('sounds');
  const [sounds, setSounds] = useState<SoundEntry[]>([]);
  const [music, setMusic] = useState<MusicEntry[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [playbackState, setPlaybackState] = useState<'stopped' | 'playing' | 'paused'>('stopped');
  const [currentPlayingId, setCurrentPlayingId] = useState<number | null>(null);

  // Load initial data
  useEffect(() => {
    loadSounds();
    loadMusic();
  }, []);

  // Poll playback state
  useEffect(() => {
    const interval = setInterval(async () => {
      try {
        const state = await invoke<string>('get_playback_state');
        setPlaybackState(state as 'stopped' | 'playing' | 'paused');
      } catch (e) {
        // Ignore errors
      }
    }, 500);
    return () => clearInterval(interval);
  }, []);

  const loadSounds = async () => {
    try {
      setLoading(true);
      const response = await invoke<{ sounds: SoundEntry[] }>('get_sound_list');
      setSounds(response.sounds);
    } catch (e) {
      setError(`Failed to load sounds: ${e}`);
    } finally {
      setLoading(false);
    }
  };

  const loadMusic = async () => {
    try {
      setLoading(true);
      const response = await invoke<{ tracks: MusicEntry[] }>('get_music_list');
      setMusic(response.tracks);
    } catch (e) {
      setError(`Failed to load music: ${e}`);
    } finally {
      setLoading(false);
    }
  };

  const handlePreviewSound = useCallback(async (soundId: number) => {
    try {
      if (playbackState === 'playing' && currentPlayingId === soundId) {
        // Stop if already playing this sound
        await invoke('stop_preview');
        setPlaybackState('stopped');
        setCurrentPlayingId(null);
      } else {
        // Play new sound
        await invoke('preview_sound', { soundId });
        setPlaybackState('playing');
        setCurrentPlayingId(soundId);
      }
    } catch (e) {
      setError(`Failed to preview sound: ${e}`);
    }
  }, [playbackState, currentPlayingId]);

  const handleStopPreview = useCallback(async () => {
    try {
      await invoke('stop_preview');
      setPlaybackState('stopped');
      setCurrentPlayingId(null);
    } catch (e) {
      setError(`Failed to stop preview: ${e}`);
    }
  }, []);

  const handleExportSound = useCallback(async (soundId: number, format: 'wav' | 'brr') => {
    try {
      const extension = format === 'wav' ? 'wav' : 'brr';
      const selected = await save({
        filters: [{
          name: format.toUpperCase(),
          extensions: [extension]
        }]
      });
      
      if (selected) {
        if (format === 'wav') {
          await invoke('export_sound_as_wav', { 
            soundId, 
            outputPath: selected,
            options: { format: 'wav', sample_rate: 32000 }
          });
        } else {
          await invoke('export_sound_as_brr', { soundId, outputPath: selected });
        }
      }
    } catch (e) {
      setError(`Failed to export sound: ${e}`);
    }
  }, []);

  const handlePreviewMusic = useCallback(async (musicId: number) => {
    try {
      if (playbackState === 'playing' && currentPlayingId === musicId) {
        await invoke('stop_preview');
        setPlaybackState('stopped');
        setCurrentPlayingId(null);
      } else {
        await invoke('preview_music', { musicId });
        setPlaybackState('playing');
        setCurrentPlayingId(musicId);
      }
    } catch (e) {
      setError(`Failed to preview music: ${e}`);
    }
  }, [playbackState, currentPlayingId]);

  const handleExportMusic = useCallback(async (musicId: number, format: 'wav' | 'spc') => {
    try {
      const extension = format === 'wav' ? 'wav' : 'spc';
      const selected = await save({
        filters: [{
          name: format.toUpperCase(),
          extensions: [extension]
        }]
      });
      
      if (selected) {
        if (format === 'wav') {
          await invoke('export_music_as_wav', { 
            musicId, 
            outputPath: selected,
            durationSeconds: 60
          });
        } else {
          await invoke('export_music_as_spc', { musicId, outputPath: selected });
        }
      }
    } catch (e) {
      setError(`Failed to export music: ${e}`);
    }
  }, []);

  const handleImportWav = useCallback(async () => {
    try {
      const selected = await open({
        multiple: false,
        filters: [{
          name: 'WAV Audio',
          extensions: ['wav', 'wave']
        }]
      });
      
      if (typeof selected === 'string') {
        // Show import dialog with options
        const confirmed = window.confirm(
          'Import this WAV file? It will be converted to BRR format.'
        );
        
        if (confirmed) {
          const result = await invoke('import_sound_from_wav', {
            wavPath: selected,
            options: {
              sampleId: 0xFF, // Auto-assign
              enableLoop: false,
              targetSampleRate: 32000
            }
          });
          console.log('Import result:', result);
          // Reload sounds list
          await loadSounds();
        }
      }
    } catch (e) {
      setError(`Failed to import WAV: ${e}`);
    }
  }, []);

  const renderTabContent = () => {
    switch (activeTab) {
      case 'sounds':
        return (
          <SoundList
            sounds={sounds}
            loading={loading}
            currentPlayingId={currentPlayingId}
            playbackState={playbackState}
            onPreview={handlePreviewSound}
            onStop={handleStopPreview}
            onExport={handleExportSound}
            onImport={handleImportWav}
          />
        );
      
      case 'music':
        return (
          <MusicEditor
            tracks={music}
            loading={loading}
            currentPlayingId={currentPlayingId}
            playbackState={playbackState}
            onPreview={handlePreviewMusic}
            onStop={handleStopPreview}
            onExport={handleExportMusic}
          />
        );
      
      case 'samples':
        return (
          <div className="audio-tab-content">
            <div className="audio-placeholder">
              <h3>Sample Editor</h3>
              <p>Advanced BRR sample editing coming soon.</p>
              <div className="research-todo">
                <h4>Research TODOs:</h4>
                <ul>
                  <li>Locate sample table in ROM</li>
                  <li>Implement BRR encoding with quality options</li>
                  <li>Add loop point editor</li>
                  <li>Implement ADSR envelope editor</li>
                </ul>
              </div>
            </div>
          </div>
        );
      
      case 'spc':
        return (
          <div className="audio-tab-content">
            <div className="audio-placeholder">
              <h3>SPC File Manager</h3>
              <p>Import and edit SPC700 save states.</p>
              <div className="spc-actions">
                <button onClick={async () => {
                  const selected = await open({
                    multiple: false,
                    filters: [{ name: 'SPC File', extensions: ['spc'] }]
                  });
                  if (typeof selected === 'string') {
                    await invoke('load_spc', { path: selected });
                  }
                }}>
                  Load SPC File
                </button>
                <button onClick={async () => {
                  await invoke('create_new_spc');
                }}>
                  Create New SPC
                </button>
              </div>
            </div>
          </div>
        );
      
      default:
        return null;
    }
  };

  return (
    <div className="audio-editor">
      <header className="audio-editor-header">
        <h2>🎵 Sound & Music Editor</h2>
        <div className="audio-playback-controls">
          <button 
            onClick={handleStopPreview}
            disabled={playbackState === 'stopped'}
            className="stop-btn"
            title="Stop Preview"
          >
            ⏹ Stop
          </button>
          <span className={`playback-status ${playbackState}`}>
            {playbackState === 'playing' ? '▶ Playing' : 
             playbackState === 'paused' ? '⏸ Paused' : '⏹ Stopped'}
          </span>
        </div>
      </header>

      {error && (
        <div className="audio-error" onClick={() => setError(null)}>
          ⚠️ {error}
        </div>
      )}

      <nav className="audio-tabs">
        <button
          className={`audio-tab ${activeTab === 'sounds' ? 'active' : ''}`}
          onClick={() => setActiveTab('sounds')}
        >
          🔊 Sounds
        </button>
        <button
          className={`audio-tab ${activeTab === 'music' ? 'active' : ''}`}
          onClick={() => setActiveTab('music')}
        >
          🎵 Music
        </button>
        <button
          className={`audio-tab ${activeTab === 'samples' ? 'active' : ''}`}
          onClick={() => setActiveTab('samples')}
        >
          🎚️ Samples
        </button>
        <button
          className={`audio-tab ${activeTab === 'spc' ? 'active' : ''}`}
          onClick={() => setActiveTab('spc')}
        >
          💾 SPC Files
        </button>
      </nav>

      <main className="audio-editor-content">
        {renderTabContent()}
      </main>

      <footer className="audio-editor-footer">
        <div className="audio-info">
          <span>🎧 {sounds.length} Sounds</span>
          <span>🎼 {music.length} Music Tracks</span>
        </div>
        <div className="audio-research-note">
          ⚠️ Audio editing requires ROM research. Many features are placeholders.
        </div>
      </footer>
    </div>
  );
};

export default AudioEditor;
