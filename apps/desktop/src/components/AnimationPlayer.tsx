/**
 * Animation Player and Hitbox Editor Component
 * 
 * Provides animation playback controls, frame management, and hitbox/hurtbox editing
 * for Super Punch-Out!! boxer animations.
 */

import React, { useState, useEffect, useRef, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useStore } from '../store/useStore';

// ============================================================================
// Types
// ============================================================================

export type HitboxType = 'jab' | 'hook' | 'uppercut' | 'special';
export type InterpolationType = 'Linear' | 'EaseIn' | 'EaseOut' | 'EaseInOut';
export type AnimationType = 'idle' | 'punch_left' | 'punch_right' | 'hit' | 'ko' | 'victory' | 'defeat' | 'dodge_left' | 'dodge_right' | 'block' | 'taunt' | 'intro' | 'special' | 'custom';

export interface Hitbox {
  id: string;
  type: HitboxType;
  x: number;
  y: number;
  width: number;
  height: number;
  damage: number;
  hitstun: number;
  knockback: number;
}

export interface Hurtbox {
  id: string;
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface AnimationFrame {
  id: string;
  poseId: number;
  duration: number;
  hitboxes: Hitbox[];
  hurtboxes: Hurtbox[];
  effects: FrameEffect[];
}

export interface FrameEffect {
  type: 'Shake' | 'Flash' | 'Sound' | 'Particle';
  data?: number | { x: number; y: number; count: number };
}

export interface BoxerAnimation {
  id: string;
  name: string;
  type: AnimationType;
  frames: AnimationFrame[];
  looping: boolean;
  frameRate: number;
  totalFrames: number;
}

export interface Boxer {
  id: number;
  name: string;
  animations: BoxerAnimation[];
}

export interface PlaybackState {
  isPlaying: boolean;
  currentFrame: number;
  totalFrames: number;
  speed: number;
  isLooping: boolean;
}

// ============================================================================
// Constants
// ============================================================================

const BOXERS_LIST: { id: number; name: string }[] = [
  { id: 0, name: 'Little Mac' },
  { id: 1, name: 'Gabby Jay' },
  { id: 2, name: 'Bear Hugger' },
  { id: 3, name: 'Piston Hurricane' },
  { id: 4, name: 'Bald Bull' },
  { id: 5, name: 'Bob Charlie' },
  { id: 6, name: 'Dragon Chan' },
  { id: 7, name: 'Masked Muscle' },
  { id: 8, name: 'Mr. Sandman' },
  { id: 9, name: 'Aran Ryan' },
  { id: 10, name: 'Heike Kagero' },
  { id: 11, name: 'Mad Clown' },
  { id: 12, name: 'Super Macho Man' },
  { id: 13, name: 'Narcis Prince' },
  { id: 14, name: 'Hoy Quarlow' },
  { id: 15, name: 'Rick Bruiser' },
  { id: 16, name: 'Nick Bruiser' },
];

const ANIMATION_TYPES: { value: AnimationType; label: string; icon: string }[] = [
  { value: 'idle', label: 'Idle', icon: '⏸️' },
  { value: 'punch_left', label: 'Punch Left', icon: '👊' },
  { value: 'punch_right', label: 'Punch Right', icon: '🥊' },
  { value: 'hit', label: 'Hit', icon: '💥' },
  { value: 'ko', label: 'KO', icon: '😵' },
  { value: 'victory', label: 'Victory', icon: '🏆' },
  { value: 'defeat', label: 'Defeat', icon: '😔' },
  { value: 'dodge_left', label: 'Dodge Left', icon: '↩️' },
  { value: 'dodge_right', label: 'Dodge Right', icon: '↪️' },
  { value: 'block', label: 'Block', icon: '🛡️' },
  { value: 'taunt', label: 'Taunt', icon: '😏' },
  { value: 'intro', label: 'Intro', icon: '🎬' },
  { value: 'special', label: 'Special', icon: '✨' },
  { value: 'custom', label: 'Custom', icon: '⚙️' },
];

const HITBOX_TYPES: { value: HitboxType; label: string; color: string }[] = [
  { value: 'jab', label: 'Jab', color: '#f59e0b' },
  { value: 'hook', label: 'Hook', color: '#ef4444' },
  { value: 'uppercut', label: 'Uppercut', color: '#8b5cf6' },
  { value: 'special', label: 'Special', color: '#ec4899' },
];

const INTERPOLATION_TYPES: { value: InterpolationType; label: string }[] = [
  { value: 'Linear', label: 'Linear' },
  { value: 'EaseIn', label: 'Ease In' },
  { value: 'EaseOut', label: 'Ease Out' },
  { value: 'EaseInOut', label: 'Ease In/Out' },
];

const SPEED_OPTIONS = [0.25, 0.5, 1, 2];

// ============================================================================
// Props Interface
// ============================================================================

export interface AnimationPlayerProps {
  /** Initial boxer ID to select */
  initialBoxerId?: number;
  /** Initial animation ID to select */
  initialAnimationId?: string;
  /** Callback when animation changes */
  onAnimationChange?: (animation: BoxerAnimation | null) => void;
  /** Callback when frame changes */
  onFrameChange?: (frame: AnimationFrame | null, index: number) => void;
  /** Callback when hitbox is edited */
  onHitboxEdit?: (hitbox: Hitbox) => void;
  /** Callback when export is requested */
  onExport?: (format: 'gif' | 'png', animation: BoxerAnimation) => void;
  /** Whether to show hitbox editor */
  showHitboxEditor?: boolean;
  /** Canvas dimensions */
  canvasWidth?: number;
  canvasHeight?: number;
}

// ============================================================================
// Main Component
// ============================================================================

export const AnimationPlayer: React.FC<AnimationPlayerProps> = ({
  initialBoxerId,
  initialAnimationId,
  onAnimationChange,
  onFrameChange,
  onHitboxEdit,
  onExport,
  showHitboxEditor = true,
  canvasWidth = 512,
  canvasHeight = 512,
}) => {
  // Store hooks
  const { fighters, loadFighterList } = useStore();

  // Selection state
  const [selectedBoxerId, setSelectedBoxerId] = useState<number | null>(initialBoxerId ?? null);
  const [selectedAnimation, setSelectedAnimation] = useState<BoxerAnimation | null>(null);
  const [selectedAnimationType, setSelectedAnimationType] = useState<AnimationType>('idle');
  const [selectedHitboxId, setSelectedHitboxId] = useState<string | null>(null);

  // Playback state
  const [playbackState, setPlaybackState] = useState<PlaybackState>({
    isPlaying: false,
    currentFrame: 0,
    totalFrames: 0,
    speed: 1,
    isLooping: true,
  });

  // UI state
  const [showHitboxes, setShowHitboxes] = useState(true);
  const [showHurtboxes, setShowHurtboxes] = useState(true);
  const [interpolation, setInterpolation] = useState<InterpolationType>('Linear');
  const [zoom, setZoom] = useState(2);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Canvas and drag state
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const [isDraggingHitbox, setIsDraggingHitbox] = useState(false);
  const [dragStart, setDragStart] = useState<{ x: number; y: number } | null>(null);
  const [dragHitbox, setDragHitbox] = useState<Hitbox | null>(null);

  // Animation loop ref
  const animationRef = useRef<number | null>(null);
  const lastFrameTimeRef = useRef<number>(0);

  // ============================================================================
  // Effects
  // ============================================================================

  // Load fighters on mount
  useEffect(() => {
    loadFighterList();
  }, [loadFighterList]);

  // Set initial boxer
  useEffect(() => {
    if (initialBoxerId !== undefined && fighters.length > 0) {
      setSelectedBoxerId(initialBoxerId);
    }
  }, [initialBoxerId, fighters]);

  // Notify parent of animation changes
  useEffect(() => {
    onAnimationChange?.(selectedAnimation);
  }, [selectedAnimation, onAnimationChange]);

  // Notify parent of frame changes
  useEffect(() => {
    const frame = selectedAnimation?.frames[playbackState.currentFrame] || null;
    onFrameChange?.(frame, playbackState.currentFrame);
  }, [playbackState.currentFrame, selectedAnimation, onFrameChange]);

  // Animation playback loop
  useEffect(() => {
    if (!playbackState.isPlaying || !selectedAnimation) return;

    const animate = (timestamp: number) => {
      if (!lastFrameTimeRef.current) {
        lastFrameTimeRef.current = timestamp;
      }

      const elapsed = timestamp - lastFrameTimeRef.current;
      const frameDuration = (1000 / selectedAnimation.frameRate) / playbackState.speed;

      if (elapsed >= frameDuration) {
        lastFrameTimeRef.current = timestamp;
        
        setPlaybackState(prev => {
          const nextFrame = prev.currentFrame + 1;
          
          if (nextFrame >= prev.totalFrames) {
            if (prev.isLooping) {
              return { ...prev, currentFrame: 0 };
            } else {
              return { ...prev, isPlaying: false, currentFrame: prev.totalFrames - 1 };
            }
          }
          
          return { ...prev, currentFrame: nextFrame };
        });
      }

      animationRef.current = requestAnimationFrame(animate);
    };

    animationRef.current = requestAnimationFrame(animate);

    return () => {
      if (animationRef.current) {
        cancelAnimationFrame(animationRef.current);
      }
    };
  }, [playbackState.isPlaying, playbackState.speed, selectedAnimation]);

  // Keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Space: Play/Pause
      if (e.code === 'Space' && !e.repeat) {
        e.preventDefault();
        togglePlayback();
      }
      
      // Arrow Left: Previous frame
      if (e.code === 'ArrowLeft') {
        e.preventDefault();
        goToPreviousFrame();
      }
      
      // Arrow Right: Next frame
      if (e.code === 'ArrowRight') {
        e.preventDefault();
        goToNextFrame();
      }
      
      // Home: First frame
      if (e.code === 'Home') {
        e.preventDefault();
        seekToFrame(0);
      }
      
      // End: Last frame
      if (e.code === 'End') {
        e.preventDefault();
        if (selectedAnimation) {
          seekToFrame(selectedAnimation.frames.length - 1);
        }
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [selectedAnimation, playbackState.isPlaying]);

  // ============================================================================
  // Handlers
  // ============================================================================

  const loadAnimation = async () => {
    if (selectedBoxerId === null) return;

    setIsLoading(true);
    setError(null);

    try {
      // Call the Tauri command to load animation
      const animation = await invoke<BoxerAnimation>('load_boxer_animation', {
        boxerId: selectedBoxerId,
        animationType: selectedAnimationType,
      });

      setSelectedAnimation(animation);
      setPlaybackState(prev => ({
        ...prev,
        currentFrame: 0,
        totalFrames: animation.frames.length,
        isPlaying: false,
      }));
    } catch (err) {
      console.error('Failed to load animation:', err);
      setError(err instanceof Error ? err.message : 'Failed to load animation');
    } finally {
      setIsLoading(false);
    }
  };

  const togglePlayback = () => {
    setPlaybackState(prev => ({ ...prev, isPlaying: !prev.isPlaying }));
  };

  const stopPlayback = () => {
    setPlaybackState(prev => ({
      ...prev,
      isPlaying: false,
      currentFrame: 0,
    }));
    lastFrameTimeRef.current = 0;
  };

  const seekToFrame = (frameIndex: number) => {
    if (!selectedAnimation) return;
    
    const clampedIndex = Math.max(0, Math.min(frameIndex, selectedAnimation.frames.length - 1));
    setPlaybackState(prev => ({ ...prev, currentFrame: clampedIndex }));
  };

  const goToPreviousFrame = () => {
    seekToFrame(playbackState.currentFrame - 1);
  };

  const goToNextFrame = () => {
    seekToFrame(playbackState.currentFrame + 1);
  };

  const handleSpeedChange = (speed: number) => {
    setPlaybackState(prev => ({ ...prev, speed }));
  };

  const handleLoopToggle = () => {
    setPlaybackState(prev => ({ ...prev, isLooping: !prev.isLooping }));
  };

  const handleAddFrame = () => {
    if (!selectedAnimation) return;

    const newFrame: AnimationFrame = {
      id: `frame_${Date.now()}`,
      poseId: 0,
      duration: 4,
      hitboxes: [],
      hurtboxes: [],
      effects: [],
    };

    const updatedAnimation: BoxerAnimation = {
      ...selectedAnimation,
      frames: [...selectedAnimation.frames, newFrame],
      totalFrames: selectedAnimation.frames.length + 1,
    };

    setSelectedAnimation(updatedAnimation);
    setPlaybackState(prev => ({ ...prev, totalFrames: updatedAnimation.frames.length }));
  };

  const handleRemoveFrame = () => {
    if (!selectedAnimation || selectedAnimation.frames.length <= 1) return;

    const newFrames = [...selectedAnimation.frames];
    newFrames.splice(playbackState.currentFrame, 1);

    const updatedAnimation: BoxerAnimation = {
      ...selectedAnimation,
      frames: newFrames,
      totalFrames: newFrames.length,
    };

    setSelectedAnimation(updatedAnimation);
    
    // Adjust current frame if necessary
    if (playbackState.currentFrame >= newFrames.length) {
      setPlaybackState(prev => ({ 
        ...prev, 
        currentFrame: newFrames.length - 1,
        totalFrames: newFrames.length,
      }));
    } else {
      setPlaybackState(prev => ({ ...prev, totalFrames: newFrames.length }));
    }
  };

  const handleDuplicateFrame = () => {
    if (!selectedAnimation) return;

    const currentFrame = selectedAnimation.frames[playbackState.currentFrame];
    if (!currentFrame) return;

    const duplicatedFrame: AnimationFrame = {
      ...currentFrame,
      id: `frame_${Date.now()}`,
      hitboxes: currentFrame.hitboxes.map(h => ({ ...h, id: `hb_${Date.now()}_${Math.random()}` })),
      hurtboxes: currentFrame.hurtboxes.map(h => ({ ...h, id: `hu_${Date.now()}_${Math.random()}` })),
    };

    const newFrames = [...selectedAnimation.frames];
    newFrames.splice(playbackState.currentFrame + 1, 0, duplicatedFrame);

    const updatedAnimation: BoxerAnimation = {
      ...selectedAnimation,
      frames: newFrames,
      totalFrames: newFrames.length,
    };

    setSelectedAnimation(updatedAnimation);
    setPlaybackState(prev => ({ ...prev, totalFrames: newFrames.length }));
  };

  const handleAddHitbox = () => {
    if (!selectedAnimation) return;

    const newHitbox: Hitbox = {
      id: `hb_${Date.now()}`,
      type: 'jab',
      x: 100,
      y: 100,
      width: 32,
      height: 32,
      damage: 10,
      hitstun: 10,
      knockback: 5,
    };

    const updatedFrames = [...selectedAnimation.frames];
    const currentFrame = updatedFrames[playbackState.currentFrame];
    
    if (currentFrame) {
      currentFrame.hitboxes = [...currentFrame.hitboxes, newHitbox];
      
      setSelectedAnimation({
        ...selectedAnimation,
        frames: updatedFrames,
      });
      
      setSelectedHitboxId(newHitbox.id);
    }
  };

  const handleRemoveHitbox = (hitboxId: string) => {
    if (!selectedAnimation) return;

    const updatedFrames = [...selectedAnimation.frames];
    const currentFrame = updatedFrames[playbackState.currentFrame];
    
    if (currentFrame) {
      currentFrame.hitboxes = currentFrame.hitboxes.filter(h => h.id !== hitboxId);
      
      setSelectedAnimation({
        ...selectedAnimation,
        frames: updatedFrames,
      });
      
      if (selectedHitboxId === hitboxId) {
        setSelectedHitboxId(null);
      }
    }
  };

  const handleUpdateHitbox = (hitboxId: string, updates: Partial<Hitbox>) => {
    if (!selectedAnimation) return;

    const updatedFrames = [...selectedAnimation.frames];
    const currentFrame = updatedFrames[playbackState.currentFrame];
    
    if (currentFrame) {
      const hitboxIndex = currentFrame.hitboxes.findIndex(h => h.id === hitboxId);
      
      if (hitboxIndex !== -1) {
        currentFrame.hitboxes[hitboxIndex] = {
          ...currentFrame.hitboxes[hitboxIndex],
          ...updates,
        };
        
        setSelectedAnimation({
          ...selectedAnimation,
          frames: updatedFrames,
        });
        
        onHitboxEdit?.(currentFrame.hitboxes[hitboxIndex]);
      }
    }
  };

  const handleCanvasMouseDown = (e: React.MouseEvent) => {
    if (!selectedAnimation || !showHitboxEditor) return;

    const rect = canvasRef.current?.getBoundingClientRect();
    if (!rect) return;

    const x = (e.clientX - rect.left) / zoom;
    const y = (e.clientY - rect.top) / zoom;

    const currentFrame = selectedAnimation.frames[playbackState.currentFrame];
    if (!currentFrame) return;

    // Check if clicking on a hitbox
    const clickedHitbox = currentFrame.hitboxes.find(h => 
      x >= h.x && x <= h.x + h.width &&
      y >= h.y && y <= h.y + h.height
    );

    if (clickedHitbox) {
      setSelectedHitboxId(clickedHitbox.id);
      setIsDraggingHitbox(true);
      setDragStart({ x, y });
      setDragHitbox({ ...clickedHitbox });
    } else {
      setSelectedHitboxId(null);
    }
  };

  const handleCanvasMouseMove = (e: React.MouseEvent) => {
    if (!isDraggingHitbox || !dragStart || !dragHitbox || !selectedAnimation) return;

    const rect = canvasRef.current?.getBoundingClientRect();
    if (!rect) return;

    const x = (e.clientX - rect.left) / zoom;
    const y = (e.clientY - rect.top) / zoom;

    const deltaX = x - dragStart.x;
    const deltaY = y - dragStart.y;

    handleUpdateHitbox(dragHitbox.id, {
      x: dragHitbox.x + deltaX,
      y: dragHitbox.y + deltaY,
    });
  };

  const handleCanvasMouseUp = () => {
    setIsDraggingHitbox(false);
    setDragStart(null);
    setDragHitbox(null);
  };

  const handleExport = (format: 'gif' | 'png') => {
    if (selectedAnimation) {
      onExport?.(format, selectedAnimation);
    }
  };

  const handleZoomIn = () => setZoom(z => Math.min(4, z + 0.5));
  const handleZoomOut = () => setZoom(z => Math.max(1, z - 0.5));
  const handleZoomReset = () => setZoom(2);

  // ============================================================================
  // Render Helpers
  // ============================================================================

  const currentFrame = selectedAnimation?.frames[playbackState.currentFrame];

  const renderHitboxOverlay = () => {
    if (!currentFrame) return null;

    return (
      <>
        {/* Hitboxes */}
        {showHitboxes && currentFrame.hitboxes.map(hitbox => {
          const typeInfo = HITBOX_TYPES.find(t => t.value === hitbox.type);
          const isSelected = selectedHitboxId === hitbox.id;

          return (
            <div
              key={hitbox.id}
              style={{
                position: 'absolute',
                left: hitbox.x * zoom,
                top: hitbox.y * zoom,
                width: hitbox.width * zoom,
                height: hitbox.height * zoom,
                border: `2px solid ${typeInfo?.color || '#ef4444'}`,
                backgroundColor: isSelected ? `${typeInfo?.color}40` : `${typeInfo?.color}20`,
                cursor: 'move',
                zIndex: isSelected ? 10 : 1,
              }}
              onMouseDown={(e) => {
                e.stopPropagation();
                setSelectedHitboxId(hitbox.id);
                setIsDraggingHitbox(true);
                setDragStart({ x: hitbox.x, y: hitbox.y });
                setDragHitbox({ ...hitbox });
              }}
            >
              <span
                style={{
                  position: 'absolute',
                  top: -20,
                  left: 0,
                  fontSize: 10,
                  color: typeInfo?.color,
                  backgroundColor: 'rgba(0,0,0,0.7)',
                  padding: '2px 4px',
                  borderRadius: 2,
                  whiteSpace: 'nowrap',
                }}
              >
                {typeInfo?.label} ({hitbox.damage} dmg)
              </span>
              
              {isSelected && (
                <>
                  {/* Resize handles */}
                  <div style={{ position: 'absolute', top: -4, left: -4, width: 8, height: 8, backgroundColor: typeInfo?.color, borderRadius: '50%' }} />
                  <div style={{ position: 'absolute', top: -4, right: -4, width: 8, height: 8, backgroundColor: typeInfo?.color, borderRadius: '50%' }} />
                  <div style={{ position: 'absolute', bottom: -4, left: -4, width: 8, height: 8, backgroundColor: typeInfo?.color, borderRadius: '50%' }} />
                  <div style={{ position: 'absolute', bottom: -4, right: -4, width: 8, height: 8, backgroundColor: typeInfo?.color, borderRadius: '50%' }} />
                </>
              )}
            </div>
          );
        })}

        {/* Hurtboxes */}
        {showHurtboxes && currentFrame.hurtboxes.map(hurtbox => (
          <div
            key={hurtbox.id}
            style={{
              position: 'absolute',
              left: hurtbox.x * zoom,
              top: hurtbox.y * zoom,
              width: hurtbox.width * zoom,
              height: hurtbox.height * zoom,
              border: '2px solid #22c55e',
              backgroundColor: 'rgba(34, 197, 94, 0.1)',
            }}
          >
            <span
              style={{
                position: 'absolute',
                bottom: -18,
                left: 0,
                fontSize: 10,
                color: '#22c55e',
                backgroundColor: 'rgba(0,0,0,0.7)',
                padding: '2px 4px',
                borderRadius: 2,
              }}
            >
              Hurtbox
            </span>
          </div>
        ))}
      </>
    );
  };

  // ============================================================================
  // Render
  // ============================================================================

  return (
    <div className="flex flex-col h-full bg-slate-900 text-white p-4 gap-4">
      {/* Header */}
      <div className="flex items-center justify-between">
        <h1 className="text-xl font-bold text-blue-400">Animation Player & Hitbox Editor</h1>
        <div className="flex items-center gap-2">
          <button
            onClick={() => handleExport('gif')}
            disabled={!selectedAnimation}
            className="px-3 py-1.5 text-sm bg-purple-600 hover:bg-purple-500 disabled:opacity-50 rounded font-medium"
            title="Export as GIF"
          >
            📤 GIF
          </button>
          <button
            onClick={() => handleExport('png')}
            disabled={!selectedAnimation}
            className="px-3 py-1.5 text-sm bg-purple-600 hover:bg-purple-500 disabled:opacity-50 rounded font-medium"
            title="Export as PNG Sequence"
          >
            📤 PNG
          </button>
        </div>
      </div>

      {/* Animation Selector */}
      <div className="flex items-center gap-4 bg-slate-800 p-3 rounded-lg border border-slate-700">
        <div className="flex items-center gap-2">
          <label className="text-sm text-slate-400">Boxer:</label>
          <select
            value={selectedBoxerId ?? ''}
            onChange={(e) => setSelectedBoxerId(parseInt(e.target.value) || null)}
            className="bg-slate-900 border border-slate-600 rounded px-3 py-1.5 text-sm min-w-[150px]"
          >
            <option value="">Select Boxer...</option>
            {BOXERS_LIST.map(boxer => (
              <option key={boxer.id} value={boxer.id}>{boxer.name}</option>
            ))}
          </select>
        </div>

        <div className="flex items-center gap-2">
          <label className="text-sm text-slate-400">Animation:</label>
          <select
            value={selectedAnimationType}
            onChange={(e) => setSelectedAnimationType(e.target.value as AnimationType)}
            className="bg-slate-900 border border-slate-600 rounded px-3 py-1.5 text-sm min-w-[150px]"
          >
            {ANIMATION_TYPES.map(type => (
              <option key={type.value} value={type.value}>
                {type.icon} {type.label}
              </option>
            ))}
          </select>
        </div>

        <button
          onClick={loadAnimation}
          disabled={selectedBoxerId === null || isLoading}
          className="px-4 py-1.5 text-sm bg-blue-600 hover:bg-blue-500 disabled:opacity-50 rounded font-medium"
        >
          {isLoading ? '⏳ Loading...' : '📂 Load'}
        </button>

        {error && (
          <span className="text-red-400 text-sm">❌ {error}</span>
        )}
      </div>

      {/* Main Content */}
      <div className="flex flex-1 gap-4 overflow-hidden">
        {/* Left: Canvas Display */}
        <div className="flex-1 flex flex-col gap-2 min-w-0">
          {/* Canvas Toolbar */}
          <div className="flex items-center justify-between bg-slate-800 p-2 rounded-lg border border-slate-700">
            <div className="flex items-center gap-4">
              {/* View Toggles */}
              <label className="flex items-center gap-1 text-sm text-slate-400 cursor-pointer">
                <input
                  type="checkbox"
                  checked={showHitboxes}
                  onChange={(e) => setShowHitboxes(e.target.checked)}
                  className="rounded"
                />
                <span>Hitboxes</span>
              </label>
              <label className="flex items-center gap-1 text-sm text-slate-400 cursor-pointer">
                <input
                  type="checkbox"
                  checked={showHurtboxes}
                  onChange={(e) => setShowHurtboxes(e.target.checked)}
                  className="rounded"
                />
                <span>Hurtboxes</span>
              </label>
            </div>

            {/* Zoom Controls */}
            <div className="flex items-center gap-1">
              <button
                onClick={handleZoomOut}
                className="px-2 py-1 text-xs bg-slate-700 hover:bg-slate-600 rounded"
              >
                −
              </button>
              <span className="text-xs text-slate-400 w-14 text-center">
                {Math.round(zoom * 100)}%
              </span>
              <button
                onClick={handleZoomIn}
                className="px-2 py-1 text-xs bg-slate-700 hover:bg-slate-600 rounded"
              >
                +
              </button>
              <button
                onClick={handleZoomReset}
                className="px-2 py-1 text-xs bg-slate-700 hover:bg-slate-600 rounded ml-1"
              >
                ⟲
              </button>
            </div>
          </div>

          {/* Canvas Container */}
          <div 
            ref={containerRef}
            className="flex-1 relative bg-slate-950 rounded-lg border border-slate-700 overflow-hidden flex items-center justify-center"
            style={{ minHeight: 300 }}
          >
            {selectedAnimation ? (
              <div className="relative">
                {/* Main Canvas */}
                <canvas
                  ref={canvasRef}
                  width={canvasWidth}
                  height={canvasHeight}
                  style={{
                    width: canvasWidth * zoom,
                    height: canvasHeight * zoom,
                    imageRendering: 'pixelated',
                    backgroundColor: '#0f172a',
                    border: '1px solid #334155',
                  }}
                  onMouseDown={handleCanvasMouseDown}
                  onMouseMove={handleCanvasMouseMove}
                  onMouseUp={handleCanvasMouseUp}
                  onMouseLeave={handleCanvasMouseUp}
                />

                {/* Hitbox/Hurtbox Overlays */}
                <div
                  style={{
                    position: 'absolute',
                    top: 0,
                    left: 0,
                    width: canvasWidth * zoom,
                    height: canvasHeight * zoom,
                    pointerEvents: 'none',
                  }}
                >
                  {renderHitboxOverlay()}
                </div>

                {/* Frame Info Overlay */}
                <div className="absolute top-2 left-2 bg-slate-900/80 backdrop-blur px-3 py-2 rounded text-xs">
                  <div className="text-slate-300">
                    Frame {playbackState.currentFrame + 1} / {playbackState.totalFrames}
                  </div>
                  <div className="text-slate-400">
                    Duration: {currentFrame?.duration ?? '-'} frames
                  </div>
                  {selectedAnimation.looping && (
                    <div className="text-green-400 mt-1">🔄 Looping</div>
                  )}
                </div>
              </div>
            ) : (
              <div className="text-slate-600 text-center">
                <div className="text-4xl mb-2">🎬</div>
                <p className="text-sm">Load an animation to start editing</p>
              </div>
            )}
          </div>

          {/* Playback Controls */}
          <div className="bg-slate-800 p-3 rounded-lg border border-slate-700">
            {/* Progress Bar */}
            <div className="mb-3">
              <div className="flex items-center gap-2 mb-1">
                <span className="text-xs text-slate-400 w-16">
                  {playbackState.currentFrame + 1} / {playbackState.totalFrames}
                </span>
                <input
                  type="range"
                  min={0}
                  max={Math.max(0, playbackState.totalFrames - 1)}
                  value={playbackState.currentFrame}
                  onChange={(e) => seekToFrame(parseInt(e.target.value))}
                  disabled={!selectedAnimation}
                  className="flex-1 h-2 bg-slate-700 rounded-lg appearance-none cursor-pointer"
                />
              </div>
            </div>

            {/* Control Buttons */}
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <button
                  onClick={togglePlayback}
                  disabled={!selectedAnimation}
                  className="px-4 py-2 bg-blue-600 hover:bg-blue-500 disabled:opacity-50 rounded font-medium"
                >
                  {playbackState.isPlaying ? '⏸️ Pause' : '▶️ Play'}
                </button>
                <button
                  onClick={stopPlayback}
                  disabled={!selectedAnimation}
                  className="px-3 py-2 bg-slate-700 hover:bg-slate-600 disabled:opacity-50 rounded"
                >
                  ⏹️ Stop
                </button>
              </div>

              <div className="flex items-center gap-4">
                {/* Speed Control */}
                <div className="flex items-center gap-2">
                  <span className="text-xs text-slate-400">Speed:</span>
                  <select
                    value={playbackState.speed}
                    onChange={(e) => handleSpeedChange(parseFloat(e.target.value))}
                    disabled={!selectedAnimation}
                    className="bg-slate-900 border border-slate-600 rounded px-2 py-1 text-xs"
                  >
                    {SPEED_OPTIONS.map(speed => (
                      <option key={speed} value={speed}>{speed}x</option>
                    ))}
                  </select>
                </div>

                {/* Loop Toggle */}
                <label className="flex items-center gap-1 text-xs text-slate-400 cursor-pointer">
                  <input
                    type="checkbox"
                    checked={playbackState.isLooping}
                    onChange={handleLoopToggle}
                    disabled={!selectedAnimation}
                    className="rounded"
                  />
                  <span>Loop</span>
                </label>

                {/* Frame Rate Display */}
                <div className="text-xs text-slate-500">
                  {selectedAnimation?.frameRate ?? 60} FPS
                </div>
              </div>

              {/* Frame Navigation */}
              <div className="flex items-center gap-1">
                <button
                  onClick={goToPreviousFrame}
                  disabled={!selectedAnimation || playbackState.currentFrame === 0}
                  className="px-3 py-2 bg-slate-700 hover:bg-slate-600 disabled:opacity-50 rounded"
                >
                  ← Prev
                </button>
                <button
                  onClick={goToNextFrame}
                  disabled={!selectedAnimation || playbackState.currentFrame >= playbackState.totalFrames - 1}
                  className="px-3 py-2 bg-slate-700 hover:bg-slate-600 disabled:opacity-50 rounded"
                >
                  Next →
                </button>
              </div>
            </div>
          </div>

          {/* Frame Management */}
          <div className="flex items-center gap-2 bg-slate-800 p-2 rounded-lg border border-slate-700">
            <span className="text-sm text-slate-400 mr-2">Frame:</span>
            <button
              onClick={handleAddFrame}
              disabled={!selectedAnimation}
              className="px-3 py-1.5 text-sm bg-green-600 hover:bg-green-500 disabled:opacity-50 rounded"
            >
              ➕ Add
            </button>
            <button
              onClick={handleDuplicateFrame}
              disabled={!selectedAnimation}
              className="px-3 py-1.5 text-sm bg-blue-600 hover:bg-blue-500 disabled:opacity-50 rounded"
            >
              📋 Duplicate
            </button>
            <button
              onClick={handleRemoveFrame}
              disabled={!selectedAnimation || (selectedAnimation?.frames.length ?? 0) <= 1}
              className="px-3 py-1.5 text-sm bg-red-600 hover:bg-red-500 disabled:opacity-50 rounded"
            >
              🗑️ Remove
            </button>

            <div className="flex-1" />

            {/* Interpolation */}
            <div className="flex items-center gap-2">
              <span className="text-sm text-slate-400">Interpolation:</span>
              <select
                value={interpolation}
                onChange={(e) => setInterpolation(e.target.value as InterpolationType)}
                disabled={!selectedAnimation}
                className="bg-slate-900 border border-slate-600 rounded px-2 py-1 text-sm"
              >
                {INTERPOLATION_TYPES.map(type => (
                  <option key={type.value} value={type.value}>{type.label}</option>
                ))}
              </select>
            </div>
          </div>
        </div>

        {/* Right: Hitbox Editor Panel */}
        {showHitboxEditor && (
          <div className="w-80 flex flex-col gap-2 bg-slate-800 rounded-lg border border-slate-700 overflow-hidden">
            {/* Hitbox List Header */}
            <div className="p-3 border-b border-slate-700 bg-slate-750">
              <div className="flex items-center justify-between">
                <span className="text-sm font-semibold text-slate-400">
                  Hitboxes ({currentFrame?.hitboxes.length ?? 0})
                </span>
                <button
                  onClick={handleAddHitbox}
                  disabled={!selectedAnimation}
                  className="px-2 py-1 text-xs bg-blue-600 hover:bg-blue-500 disabled:opacity-50 rounded"
                >
                  + Add
                </button>
              </div>
            </div>

            {/* Hitbox List */}
            <div className="flex-1 overflow-y-auto p-2 space-y-2">
              {!selectedAnimation ? (
                <div className="text-center text-slate-500 py-8">
                  <div className="text-2xl mb-2">🎯</div>
                  <p className="text-xs">Load an animation to edit hitboxes</p>
                </div>
              ) : !currentFrame || currentFrame.hitboxes.length === 0 ? (
                <div className="text-center text-slate-500 py-8">
                  <div className="text-2xl mb-2">🎯</div>
                  <p className="text-xs">No hitboxes on this frame</p>
                  <p className="text-xs text-slate-600 mt-1">Click "+ Add" to create one</p>
                </div>
              ) : (
                currentFrame.hitboxes.map(hitbox => {
                  const typeInfo = HITBOX_TYPES.find(t => t.value === hitbox.type);
                  const isSelected = selectedHitboxId === hitbox.id;

                  return (
                    <div
                      key={hitbox.id}
                      onClick={() => setSelectedHitboxId(hitbox.id)}
                      className={`p-3 rounded border cursor-pointer transition ${
                        isSelected
                          ? 'bg-slate-700 border-blue-500'
                          : 'bg-slate-800 border-slate-600 hover:border-slate-500'
                      }`}
                    >
                      <div className="flex items-center justify-between mb-2">
                        <span
                          className="text-xs px-2 py-0.5 rounded"
                          style={{
                            backgroundColor: `${typeInfo?.color}20`,
                            color: typeInfo?.color,
                          }}
                        >
                          {typeInfo?.label}
                        </span>
                        <button
                          onClick={(e) => {
                            e.stopPropagation();
                            handleRemoveHitbox(hitbox.id);
                          }}
                          className="text-xs text-red-400 hover:text-red-300 px-1"
                        >
                          🗑️
                        </button>
                      </div>

                      {isSelected && (
                        <div className="space-y-2 mt-2">
                          {/* Type */}
                          <div>
                            <label className="text-xs text-slate-500">Type</label>
                            <select
                              value={hitbox.type}
                              onChange={(e) => handleUpdateHitbox(hitbox.id, { type: e.target.value as HitboxType })}
                              className="w-full bg-slate-900 border border-slate-600 rounded px-2 py-1 text-xs"
                            >
                              {HITBOX_TYPES.map(type => (
                                <option key={type.value} value={type.value}>{type.label}</option>
                              ))}
                            </select>
                          </div>

                          {/* Position */}
                          <div className="grid grid-cols-2 gap-2">
                            <div>
                              <label className="text-xs text-slate-500">X</label>
                              <input
                                type="number"
                                value={hitbox.x}
                                onChange={(e) => handleUpdateHitbox(hitbox.id, { x: parseInt(e.target.value) || 0 })}
                                className="w-full bg-slate-900 border border-slate-600 rounded px-2 py-1 text-xs"
                              />
                            </div>
                            <div>
                              <label className="text-xs text-slate-500">Y</label>
                              <input
                                type="number"
                                value={hitbox.y}
                                onChange={(e) => handleUpdateHitbox(hitbox.id, { y: parseInt(e.target.value) || 0 })}
                                className="w-full bg-slate-900 border border-slate-600 rounded px-2 py-1 text-xs"
                              />
                            </div>
                          </div>

                          {/* Size */}
                          <div className="grid grid-cols-2 gap-2">
                            <div>
                              <label className="text-xs text-slate-500">Width</label>
                              <input
                                type="number"
                                value={hitbox.width}
                                onChange={(e) => handleUpdateHitbox(hitbox.id, { width: parseInt(e.target.value) || 1 })}
                                className="w-full bg-slate-900 border border-slate-600 rounded px-2 py-1 text-xs"
                              />
                            </div>
                            <div>
                              <label className="text-xs text-slate-500">Height</label>
                              <input
                                type="number"
                                value={hitbox.height}
                                onChange={(e) => handleUpdateHitbox(hitbox.id, { height: parseInt(e.target.value) || 1 })}
                                className="w-full bg-slate-900 border border-slate-600 rounded px-2 py-1 text-xs"
                              />
                            </div>
                          </div>

                          {/* Damage Properties */}
                          <div>
                            <label className="text-xs text-slate-500">Damage</label>
                            <input
                              type="number"
                              min={0}
                              max={255}
                              value={hitbox.damage}
                              onChange={(e) => handleUpdateHitbox(hitbox.id, { damage: parseInt(e.target.value) || 0 })}
                              className="w-full bg-slate-900 border border-slate-600 rounded px-2 py-1 text-xs"
                            />
                          </div>

                          <div className="grid grid-cols-2 gap-2">
                            <div>
                              <label className="text-xs text-slate-500">Hitstun</label>
                              <input
                                type="number"
                                min={0}
                                max={255}
                                value={hitbox.hitstun}
                                onChange={(e) => handleUpdateHitbox(hitbox.id, { hitstun: parseInt(e.target.value) || 0 })}
                                className="w-full bg-slate-900 border border-slate-600 rounded px-2 py-1 text-xs"
                              />
                            </div>
                            <div>
                              <label className="text-xs text-slate-500">Knockback</label>
                              <input
                                type="number"
                                min={0}
                                max={255}
                                value={hitbox.knockback}
                                onChange={(e) => handleUpdateHitbox(hitbox.id, { knockback: parseInt(e.target.value) || 0 })}
                                className="w-full bg-slate-900 border border-slate-600 rounded px-2 py-1 text-xs"
                              />
                            </div>
                          </div>
                        </div>
                      )}

                      {!isSelected && (
                        <div className="text-xs text-slate-500 mt-1">
                          Pos: ({hitbox.x}, {hitbox.y}) • Size: {hitbox.width}×{hitbox.height}
                        </div>
                      )}
                    </div>
                  );
                })
              )}
            </div>

            {/* Hurtbox Section */}
            <div className="border-t border-slate-700 p-3">
              <div className="flex items-center justify-between mb-2">
                <span className="text-sm font-semibold text-slate-400">
                  Hurtboxes ({currentFrame?.hurtboxes.length ?? 0})
                </span>
                <button
                  disabled={!selectedAnimation}
                  className="px-2 py-1 text-xs bg-green-600 hover:bg-green-500 disabled:opacity-50 rounded"
                >
                  + Add
                </button>
              </div>
              <p className="text-xs text-slate-600">
                Define vulnerable areas on the boxer
              </p>
            </div>

            {/* Keyboard Shortcuts */}
            <div className="border-t border-slate-700 p-3 bg-slate-750">
              <div className="text-xs text-slate-500 font-semibold mb-2">Shortcuts</div>
              <div className="text-xs text-slate-600 space-y-1">
                <div><kbd className="bg-slate-800 px-1 rounded">Space</kbd> Play/Pause</div>
                <div><kbd className="bg-slate-800 px-1 rounded">←</kbd> <kbd className="bg-slate-800 px-1 rounded">→</kbd> Navigate frames</div>
                <div><kbd className="bg-slate-800 px-1 rounded">Home</kbd> First frame</div>
                <div><kbd className="bg-slate-800 px-1 rounded">End</kbd> Last frame</div>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};

export default AnimationPlayer;
