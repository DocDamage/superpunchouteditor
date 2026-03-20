import React, { useEffect, useState, useRef, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { AnimationFrame } from './AnimationEditor';

interface AnimationPreviewProps {
  fighterId: number | null;
  currentFrame: AnimationFrame | null;
  isPlaying?: boolean;
}

export const AnimationPreview: React.FC<AnimationPreviewProps> = ({
  fighterId,
  currentFrame,
  isPlaying: externalIsPlaying = true,
}) => {
  const [imageSrc, setImageSrc] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [isPlaying, setIsPlaying] = useState(externalIsPlaying);
  const [playbackSpeed, setPlaybackSpeed] = useState(1.0);
  const [showGrid, setShowGrid] = useState(false);
  const [showHitbox, setShowHitbox] = useState(true);
  const [zoom, setZoom] = useState(2);
  
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);

  // Load pose image when frame changes
  useEffect(() => {
    if (fighterId !== null && currentFrame) {
      loadPosePreview(fighterId, currentFrame.pose_id);
    }
  }, [fighterId, currentFrame?.pose_id]);

  const loadPosePreview = async (fid: number, poseId: number) => {
    setLoading(true);
    try {
      const bytes = await invoke<number[]>('preview_animation_frame', {
        fighterId: fid,
        poseId,
      });
      const blob = new Blob([new Uint8Array(bytes)], { type: 'image/png' });
      const url = URL.createObjectURL(blob);
      setImageSrc(url);
    } catch (e) {
      console.error('Failed to load pose preview:', e);
      setImageSrc(null);
    } finally {
      setLoading(false);
    }
  };

  // Cleanup blob URLs
  useEffect(() => {
    return () => {
      if (imageSrc) {
        URL.revokeObjectURL(imageSrc);
      }
    };
  }, [imageSrc]);

  const handleZoomIn = () => setZoom(z => Math.min(4, z + 0.5));
  const handleZoomOut = () => setZoom(z => Math.max(1, z - 0.5));
  const handleResetZoom = () => setZoom(2);

  const speedOptions = [0.25, 0.5, 1.0, 2.0];

  return (
    <div className="h-full flex flex-col bg-slate-800 rounded-lg border border-slate-700 overflow-hidden">
      {/* Preview Header */}
      <div className="flex items-center justify-between p-3 border-b border-slate-700 bg-slate-750">
        <div className="flex items-center gap-3">
          <span className="text-sm font-semibold text-slate-400">Preview</span>
          {currentFrame && (
            <span className="text-xs text-slate-500">
              Pose {currentFrame.pose_id} • Tileset {currentFrame.tileset_id}
            </span>
          )}
        </div>
        
        <div className="flex items-center gap-2">
          {/* Playback Controls */}
          <button
            onClick={() => setIsPlaying(!isPlaying)}
            className="px-3 py-1 text-xs bg-slate-700 hover:bg-slate-600 rounded flex items-center gap-1"
          >
            {isPlaying ? '⏸️' : '▶️'} {isPlaying ? 'Pause' : 'Play'}
          </button>
          
          {/* Speed Control */}
          <select
            value={playbackSpeed}
            onChange={(e) => setPlaybackSpeed(parseFloat(e.target.value))}
            className="bg-slate-900 border border-slate-600 rounded px-2 py-1 text-xs"
          >
            {speedOptions.map(speed => (
              <option key={speed} value={speed}>
                {speed}x
              </option>
            ))}
          </select>

          <div className="w-px h-4 bg-slate-600 mx-1" />

          {/* Zoom Controls */}
          <button
            onClick={handleZoomOut}
            className="px-2 py-1 text-xs bg-slate-700 hover:bg-slate-600 rounded"
            title="Zoom Out"
          >
            −
          </button>
          <span className="text-xs text-slate-400 w-12 text-center">
            {Math.round(zoom * 100)}%
          </span>
          <button
            onClick={handleZoomIn}
            className="px-2 py-1 text-xs bg-slate-700 hover:bg-slate-600 rounded"
            title="Zoom In"
          >
            +
          </button>
          <button
            onClick={handleResetZoom}
            className="px-2 py-1 text-xs bg-slate-700 hover:bg-slate-600 rounded"
            title="Reset Zoom"
          >
            ⟲
          </button>

          <div className="w-px h-4 bg-slate-600 mx-1" />

          {/* View Options */}
          <label className="flex items-center gap-1 text-xs text-slate-400 cursor-pointer">
            <input
              type="checkbox"
              checked={showGrid}
              onChange={(e) => setShowGrid(e.target.checked)}
              className="rounded"
            />
            Grid
          </label>
          <label className="flex items-center gap-1 text-xs text-slate-400 cursor-pointer">
            <input
              type="checkbox"
              checked={showHitbox}
              onChange={(e) => setShowHitbox(e.target.checked)}
              className="rounded"
            />
            Hitbox
          </label>
        </div>
      </div>

      {/* Preview Canvas */}
      <div 
        ref={containerRef}
        className="flex-1 relative bg-slate-950 flex items-center justify-center overflow-hidden"
      >
        {/* Grid Background */}
        {showGrid && (
          <div 
            className="absolute inset-0 pointer-events-none opacity-20"
            style={{
              backgroundImage: `
                linear-gradient(to right, #334155 1px, transparent 1px),
                linear-gradient(to bottom, #334155 1px, transparent 1px)
              `,
              backgroundSize: `${8 * zoom}px ${8 * zoom}px`,
            }}
          />
        )}

        {/* Loading Spinner */}
        {loading && (
          <div className="absolute inset-0 z-10 bg-slate-950/50 flex items-center justify-center">
            <div className="animate-spin rounded-full h-12 w-12 border-4 border-blue-500 border-t-transparent"></div>
          </div>
        )}

        {/* Image Display */}
        {imageSrc ? (
          <div 
            className="relative transition-transform duration-200"
            style={{ 
              transform: `scale(${zoom})`,
              imageRendering: 'pixelated',
            }}
          >
            <img 
              src={imageSrc} 
              alt="Animation Frame"
              className="pixelated"
              style={{
                imageRendering: 'pixelated',
                maxWidth: '256px',
                maxHeight: '256px',
              }}
            />

            {/* Effect Overlays */}
            {currentFrame?.effects.map((effect, i) => {
              if (effect.type === 'Hitbox' && showHitbox) {
                const { x, y, w, h } = effect.data as { x: number; y: number; w: number; h: number };
                return (
                  <div
                    key={i}
                    className="absolute border-2 border-red-500 bg-red-500/20 pointer-events-none"
                    style={{
                      left: `${128 + x}px`,
                      top: `${128 + y}px`,
                      width: `${w}px`,
                      height: `${h}px`,
                    }}
                  >
                    <span className="absolute -top-4 left-0 text-[8px] text-red-400 whitespace-nowrap">
                      HITBOX
                    </span>
                  </div>
                );
              }
              if (effect.type === 'Shake') {
                return (
                  <div
                    key={i}
                    className="absolute inset-0 flex items-center justify-center pointer-events-none"
                  >
                    <span className="text-2xl animate-pulse">💥</span>
                  </div>
                );
              }
              if (effect.type === 'Flash') {
                return (
                  <div
                    key={i}
                    className="absolute inset-0 bg-white/30 pointer-events-none animate-pulse"
                  />
                );
              }
              return null;
            })}
          </div>
        ) : (
          <div className="text-slate-600 text-center">
            <div className="text-4xl mb-2">🥊</div>
            <p className="text-sm">No frame selected</p>
          </div>
        )}

        {/* Frame Info Overlay */}
        {currentFrame && (
          <div className="absolute bottom-2 left-2 bg-slate-900/80 backdrop-blur rounded px-3 py-2 text-xs">
            <div className="text-slate-400">
              Effects: {currentFrame.effects.length > 0 
                ? currentFrame.effects.map(e => e.type).join(', ')
                : 'None'}
            </div>
          </div>
        )}

        {/* Canvas for potential overlay drawing */}
        <canvas 
          ref={canvasRef}
          className="absolute inset-0 pointer-events-none"
          width={512}
          height={512}
        />
      </div>

      {/* Playback Status Bar */}
      <div className="px-3 py-2 border-t border-slate-700 bg-slate-750 flex items-center justify-between text-xs">
        <div className="flex items-center gap-4">
          <span className="text-slate-500">
            Canvas: 256×256
          </span>
          <span className="text-slate-500">
            Scale: {zoom.toFixed(1)}x
          </span>
        </div>
        <div className="flex items-center gap-2">
          {currentFrame?.effects.map((effect, i) => (
            <span 
              key={i}
              className="px-2 py-0.5 rounded bg-slate-700 text-slate-300"
            >
              {effect.type === 'Shake' && '💥 Shake'}
              {effect.type === 'Flash' && '⚡ Flash'}
              {effect.type === 'Sound' && `🔊 Sound #${effect.data}`}
              {effect.type === 'Hitbox' && '🎯 Hitbox'}
            </span>
          ))}
        </div>
      </div>

      <style>{`
        .pixelated {
          image-rendering: -moz-crisp-edges;
          image-rendering: -webkit-crisp-edges;
          image-rendering: pixelated;
          image-rendering: crisp-edges;
        }
      `}</style>
    </div>
  );
};
