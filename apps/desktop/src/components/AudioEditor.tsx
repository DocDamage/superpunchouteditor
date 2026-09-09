/**
 * Sound & Music Editor for Super Punch-Out!!
 * 
 * Provides a UI for browsing, previewing, and editing SPC700 audio data.
 * 
 * @module AudioEditor
 */

import { useState, useEffect, useCallback, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { convertFileSrc } from '@tauri-apps/api/core';
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

export type AudioTab = 'sounds' | 'music';

export const AudioEditor = () => {
  const [activeTab, setActiveTab] = useState<AudioTab>('sounds');
  const [sounds, setSounds] = useState<SoundEntry[]>([]);
  const [music, setMusic] = useState<MusicEntry[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [info, setInfo] = useState<string | null>(null);
  const [playbackState, setPlaybackState] = useState<'stopped' | 'playing' | 'paused'>('stopped');
  const [currentPlayingId, setCurrentPlayingId] = useState<number | null>(null);
  // Hidden audio element used for preview playback
  const audioRef = useRef<HTMLAudioElement | null>(null);

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
    setError(null);
    setInfo(null);
    if (playbackState === 'playing' && currentPlayingId === soundId) {
      // Stop if already playing this sound
      audioRef.current?.pause();
      await invoke('stop_preview');
      setPlaybackState('stopped');
      setCurrentPlayingId(null);
      return;
    }

    try {
      // Backend returns the path to a decoded temp WAV file
      const wavPath = await invoke<string>('preview_sound', { soundId });
      const url = convertFileSrc(wavPath);

      if (!audioRef.current) {
        audioRef.current = new Audio();
      }
      audioRef.current.pause();
      audioRef.current.src = url;
      audioRef.current.onended = () => {
        setPlaybackState('stopped');
        setCurrentPlayingId(null);
      };
      await audioRef.current.play();
      setPlaybackState('playing');
      setCurrentPlayingId(soundId);
    } catch (e) {
      // Show import-needed info rather than a hard error
      setInfo(String(e));
    }
  }, [playbackState, currentPlayingId]);

  const handleStopPreview = useCallback(async () => {
    audioRef.current?.pause();
    await invoke('stop_preview').catch(() => {});
    setPlaybackState('stopped');
    setCurrentPlayingId(null);
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
    setError(null);
    setInfo(null);
    try {
      await invoke('preview_music', { musicId });
    } catch (e) {
      // Backend intentionally returns an informational error for music preview
      setInfo(String(e));
    }
  }, []);

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
    setError(null);
    setInfo(null);
    try {
      const selected = await open({
        multiple: false,
        filters: [{ name: 'WAV Audio', extensions: ['wav', 'wave'] }]
      });

      if (typeof selected === 'string') {
        const confirmed = window.confirm(
          'Import this WAV file? It will be encoded to BRR format and stored for preview/export.'
        );
        if (!confirmed) return;

        const result = await invoke<{
          sample_id: number;
          brr_size: number;
          duration_ms: number;
          ready_for_preview: boolean;
        }>('import_sound_from_wav', {
          wavPath: selected,
          options: { sampleId: 0xFF, enableLoop: false, targetSampleRate: 32000 }
        });

        setInfo(
          `Imported: ${result.brr_size} bytes BRR (sample ID ${result.sample_id}). ` +
          `Click the preview button to listen.`
        );
        await loadSounds();
      }
    } catch (e) {
      setError(`Failed to import WAV: ${e}`);
    }
  }, [loadSounds]);

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

      {info && !error && (
        <div className="audio-info-banner" onClick={() => setInfo(null)}>
          ℹ️ {info}
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
      </nav>

      <main className="audio-editor-content">
        {renderTabContent()}
      </main>

      <footer className="audio-editor-footer">
        <div className="audio-info">
          <span>🎧 {sounds.length} Sounds</span>
          <span>🎼 {music.length} Music Tracks</span>
        </div>
      </footer>
    </div>
  );
};

export default AudioEditor;
