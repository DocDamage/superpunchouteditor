/**
 * useAnimation Hook
 * 
 * React hooks for animation system Tauri commands.
 * Provides functionality for managing boxer animations, playback control, and hitbox editing.
 */

import { useState, useEffect, useCallback, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type {
  AnimationData,
  AnimationFrameData,
  AnimationPlayerState,
  HitboxData,
  BoxerKey,
} from '../types/api';

// ============================================================================
// Animation Data Hooks
// ============================================================================

export interface UseBoxerAnimationState {
  /** Animation data */
  data: AnimationData | null;
  /** Whether data is loading */
  isLoading: boolean;
  /** Error message if loading failed */
  error: string | null;
}

export interface UseBoxerAnimationReturn extends UseBoxerAnimationState {
  /** Refresh the animation data */
  refresh: () => Promise<void>;
}

/**
 * Hook for fetching boxer animation data
 * @param boxerKey - The boxer identifier
 * @param animationName - The animation name
 * @returns Animation data and loading state
 */
export function useBoxerAnimation(
  boxerKey: BoxerKey,
  animationName: string
): UseBoxerAnimationReturn {
  const [data, setData] = useState<AnimationData | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async (): Promise<void> => {
    setIsLoading(true);
    setError(null);
    try {
      const result = await invoke<AnimationData>('get_boxer_animation', {
        boxer_key: boxerKey,
        animation_name: animationName,
      });
      setData(result);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      setError(`Failed to load animation: ${errorMessage}`);
      console.error('Failed to get boxer animation:', err);
    } finally {
      setIsLoading(false);
    }
  }, [boxerKey, animationName]);

  useEffect(() => {
    refresh();
  }, [refresh]);

  return { data, isLoading, error, refresh };
}

// ============================================================================
// Animation Player Hooks
// ============================================================================

export interface UsePlayAnimationReturn {
  /** Start playing an animation */
  play: (boxerKey: BoxerKey, animationName: string) => Promise<boolean>;
  /** Whether operation is in progress */
  isLoading: boolean;
  /** Error message if operation failed */
  error: string | null;
}

/**
 * Hook for playing an animation
 * @returns Mutation function and state for playing an animation
 */
export function usePlayAnimation(): UsePlayAnimationReturn {
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const play = useCallback(
    async (boxerKey: BoxerKey, animationName: string): Promise<boolean> => {
      setIsLoading(true);
      setError(null);
      try {
        await invoke<void>('play_animation', {
          boxer_key: boxerKey,
          animation_name: animationName,
        });
        return true;
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : String(err);
        setError(`Failed to play animation: ${errorMessage}`);
        console.error('Failed to play animation:', err);
        return false;
      } finally {
        setIsLoading(false);
      }
    },
    []
  );

  return { play, isLoading, error };
}

export interface UsePauseAnimationReturn {
  /** Pause the current animation */
  pause: () => Promise<boolean>;
  /** Whether operation is in progress */
  isLoading: boolean;
  /** Error message if operation failed */
  error: string | null;
}

/**
 * Hook for pausing animation playback
 * @returns Mutation function and state for pausing animation
 */
export function usePauseAnimation(): UsePauseAnimationReturn {
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const pause = useCallback(async (): Promise<boolean> => {
    setIsLoading(true);
    setError(null);
    try {
      await invoke<void>('pause_animation');
      return true;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      setError(`Failed to pause animation: ${errorMessage}`);
      console.error('Failed to pause animation:', err);
      return false;
    } finally {
      setIsLoading(false);
    }
  }, []);

  return { pause, isLoading, error };
}

export interface UseStopAnimationReturn {
  /** Stop the current animation */
  stop: () => Promise<boolean>;
  /** Whether operation is in progress */
  isLoading: boolean;
  /** Error message if operation failed */
  error: string | null;
}

/**
 * Hook for stopping animation playback
 * @returns Mutation function and state for stopping animation
 */
export function useStopAnimation(): UseStopAnimationReturn {
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const stop = useCallback(async (): Promise<boolean> => {
    setIsLoading(true);
    setError(null);
    try {
      await invoke<void>('stop_animation');
      return true;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      setError(`Failed to stop animation: ${errorMessage}`);
      console.error('Failed to stop animation:', err);
      return false;
    } finally {
      setIsLoading(false);
    }
  }, []);

  return { stop, isLoading, error };
}

export interface UseSeekAnimationReturn {
  /** Seek to a specific frame */
  seek: (frameIndex: number) => Promise<boolean>;
  /** Whether operation is in progress */
  isLoading: boolean;
  /** Error message if operation failed */
  error: string | null;
}

/**
 * Hook for seeking to a specific animation frame
 * @returns Mutation function and state for seeking animation
 */
export function useSeekAnimation(): UseSeekAnimationReturn {
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const seek = useCallback(async (frameIndex: number): Promise<boolean> => {
    setIsLoading(true);
    setError(null);
    try {
      await invoke<void>('seek_animation_frame', { frame: frameIndex });
      return true;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      setError(`Failed to seek animation: ${errorMessage}`);
      console.error('Failed to seek animation frame:', err);
      return false;
    } finally {
      setIsLoading(false);
    }
  }, []);

  return { seek, isLoading, error };
}

export interface UseUpdateAnimationReturn {
  /** Update animation state with delta time */
  update: (deltaTimeMs: number) => Promise<AnimationPlayerState | null>;
  /** Whether operation is in progress */
  isLoading: boolean;
  /** Error message if operation failed */
  error: string | null;
}

/**
 * Hook for updating animation state
 * @returns Mutation function and state for updating animation
 */
export function useUpdateAnimation(): UseUpdateAnimationReturn {
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const update = useCallback(
    async (deltaTimeMs: number): Promise<AnimationPlayerState | null> => {
      setIsLoading(true);
      setError(null);
      try {
        const result = await invoke<AnimationPlayerState>('update_animation', {
          delta_time_ms: deltaTimeMs,
        });
        return result;
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : String(err);
        setError(`Failed to update animation: ${errorMessage}`);
        console.error('Failed to update animation:', err);
        return null;
      } finally {
        setIsLoading(false);
      }
    },
    []
  );

  return { update, isLoading, error };
}

// ============================================================================
// Animation Player State Hook
// ============================================================================

export interface UseAnimationPlayerState {
  /** Current player state */
  state: AnimationPlayerState | null;
  /** Whether state is loading */
  isLoading: boolean;
  /** Error message if loading failed */
  error: string | null;
}

export interface UseAnimationPlayerReturn extends UseAnimationPlayerState {
  /** Play an animation */
  play: (boxerKey: BoxerKey, animationName: string) => Promise<boolean>;
  /** Pause playback */
  pause: () => Promise<boolean>;
  /** Stop playback */
  stop: () => Promise<boolean>;
  /** Seek to a frame */
  seek: (frameIndex: number) => Promise<boolean>;
  /** Update animation with delta time (for manual frame stepping) */
  update: (deltaTimeMs: number) => Promise<void>;
  /** Start automatic playback loop */
  startPlayback: () => void;
  /** Stop automatic playback loop */
  stopPlayback: () => void;
  /** Whether automatic playback is active */
  isPlaying: boolean;
}

/**
 * Comprehensive hook for animation player control with optional auto-playback
 * @returns Full animation player control
 */
export function useAnimationPlayer(): UseAnimationPlayerReturn {
  const [state, setState] = useState<AnimationPlayerState | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [isPlaying, setIsPlaying] = useState(false);
  const animationFrameRef = useRef<number | null>(null);
  const lastTimeRef = useRef<number>(0);

  const play = useCallback(async (boxerKey: BoxerKey, animationName: string): Promise<boolean> => {
    setIsLoading(true);
    setError(null);
    try {
      await invoke<void>('play_animation', {
        boxer_key: boxerKey,
        animation_name: animationName,
      });
      // Fetch initial state
      const initialState = await invoke<AnimationPlayerState>('update_animation', {
        delta_time_ms: 0,
      });
      setState(initialState);
      return true;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      setError(`Failed to play animation: ${errorMessage}`);
      console.error('Failed to play animation:', err);
      return false;
    } finally {
      setIsLoading(false);
    }
  }, []);

  const pause = useCallback(async (): Promise<boolean> => {
    try {
      await invoke<void>('pause_animation');
      if (state) {
        setState({ ...state, is_paused: true });
      }
      return true;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      setError(`Failed to pause animation: ${errorMessage}`);
      console.error('Failed to pause animation:', err);
      return false;
    }
  }, [state]);

  const stop = useCallback(async (): Promise<boolean> => {
    try {
      await invoke<void>('stop_animation');
      setIsPlaying(false);
      if (animationFrameRef.current) {
        cancelAnimationFrame(animationFrameRef.current);
        animationFrameRef.current = null;
      }
      if (state) {
        setState({ ...state, is_playing: false, current_frame: 0, current_time_ms: 0 });
      }
      return true;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      setError(`Failed to stop animation: ${errorMessage}`);
      console.error('Failed to stop animation:', err);
      return false;
    }
  }, [state]);

  const seek = useCallback(async (frameIndex: number): Promise<boolean> => {
    try {
      await invoke<void>('seek_animation_frame', { frame: frameIndex });
      if (state) {
        setState({ ...state, current_frame: frameIndex });
      }
      return true;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      setError(`Failed to seek animation: ${errorMessage}`);
      console.error('Failed to seek animation frame:', err);
      return false;
    }
  }, [state]);

  const update = useCallback(async (deltaTimeMs: number): Promise<void> => {
    try {
      const result = await invoke<AnimationPlayerState>('update_animation', {
        delta_time_ms: deltaTimeMs,
      });
      setState(result);
    } catch (err) {
      console.error('Failed to update animation:', err);
    }
  }, []);

  const startPlayback = useCallback((): void => {
    setIsPlaying(true);
    lastTimeRef.current = performance.now();

    const loop = async (currentTime: number) => {
      if (!isPlaying) return;

      const deltaTime = currentTime - lastTimeRef.current;
      lastTimeRef.current = currentTime;

      try {
        const result = await invoke<AnimationPlayerState>('update_animation', {
          delta_time_ms: deltaTime,
        });
        setState(result);
      } catch (err) {
        console.error('Animation update error:', err);
      }

      animationFrameRef.current = requestAnimationFrame(loop);
    };

    animationFrameRef.current = requestAnimationFrame(loop);
  }, [isPlaying]);

  const stopPlayback = useCallback((): void => {
    setIsPlaying(false);
    if (animationFrameRef.current) {
      cancelAnimationFrame(animationFrameRef.current);
      animationFrameRef.current = null;
    }
  }, []);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (animationFrameRef.current) {
        cancelAnimationFrame(animationFrameRef.current);
      }
    };
  }, []);

  return {
    state,
    isLoading,
    error,
    play,
    pause,
    stop,
    seek,
    update,
    startPlayback,
    stopPlayback,
    isPlaying,
  };
}

// ============================================================================
// Hitbox Management Hooks
// ============================================================================

export interface UseHitboxesState {
  /** Hitboxes for the current frame */
  hitboxes: HitboxData[];
  /** Whether data is loading */
  isLoading: boolean;
  /** Error message if loading failed */
  error: string | null;
}

export interface UseHitboxesReturn extends UseHitboxesState {
  /** Refresh hitboxes */
  refresh: () => Promise<void>;
}

/**
 * Hook for fetching hitboxes for a specific frame
 * @param boxerKey - The boxer identifier
 * @param animationName - The animation name
 * @param frameIndex - The frame index
 * @returns Hitboxes data and loading state
 */
export function useHitboxes(
  boxerKey: BoxerKey,
  animationName: string,
  frameIndex: number
): UseHitboxesReturn {
  const [hitboxes, setHitboxes] = useState<HitboxData[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async (): Promise<void> => {
    setIsLoading(true);
    setError(null);
    try {
      const result = await invoke<HitboxData[]>('get_hitbox_editor_state', {
        boxer_key: boxerKey,
        animation_name: animationName,
        frame_index: frameIndex,
      });
      setHitboxes(result);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      setError(`Failed to load hitboxes: ${errorMessage}`);
      console.error('Failed to get hitboxes:', err);
    } finally {
      setIsLoading(false);
    }
  }, [boxerKey, animationName, frameIndex]);

  useEffect(() => {
    refresh();
  }, [refresh]);

  return { hitboxes, isLoading, error, refresh };
}

export interface UseUpdateHitboxReturn {
  /** Update a hitbox */
  update: (hitbox: HitboxData) => Promise<boolean>;
  /** Whether operation is in progress */
  isLoading: boolean;
  /** Error message if operation failed */
  error: string | null;
}

/**
 * Hook for updating a hitbox
 * @param boxerKey - The boxer identifier
 * @param animationName - The animation name
 * @param frameIndex - The frame index
 * @returns Mutation function and state for updating a hitbox
 */
export function useUpdateHitbox(
  boxerKey: BoxerKey,
  animationName: string,
  frameIndex: number
): UseUpdateHitboxReturn {
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const update = useCallback(
    async (hitbox: HitboxData): Promise<boolean> => {
      setIsLoading(true);
      setError(null);
      try {
        await invoke<void>('update_hitbox', {
          boxer_key: boxerKey,
          animation_name: animationName,
          frame_index: frameIndex,
          hitbox,
        });
        return true;
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : String(err);
        setError(`Failed to update hitbox: ${errorMessage}`);
        console.error('Failed to update hitbox:', err);
        return false;
      } finally {
        setIsLoading(false);
      }
    },
    [boxerKey, animationName, frameIndex]
  );

  return { update, isLoading, error };
}

export interface UseAddHitboxReturn {
  /** Add a new hitbox */
  add: (hitbox: Omit<HitboxData, 'id'>) => Promise<HitboxData | null>;
  /** Whether operation is in progress */
  isLoading: boolean;
  /** Error message if operation failed */
  error: string | null;
}

/**
 * Hook for adding a hitbox
 * @param boxerKey - The boxer identifier
 * @param animationName - The animation name
 * @param frameIndex - The frame index
 * @returns Mutation function and state for adding a hitbox
 */
export function useAddHitbox(
  boxerKey: BoxerKey,
  animationName: string,
  frameIndex: number
): UseAddHitboxReturn {
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const add = useCallback(
    async (hitbox: Omit<HitboxData, 'id'>): Promise<HitboxData | null> => {
      setIsLoading(true);
      setError(null);
      try {
        const result = await invoke<HitboxData>('create_hitbox', {
          boxer_key: boxerKey,
          animation_name: animationName,
          frame_index: frameIndex,
          hitbox,
        });
        return result;
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : String(err);
        setError(`Failed to add hitbox: ${errorMessage}`);
        console.error('Failed to add hitbox:', err);
        return null;
      } finally {
        setIsLoading(false);
      }
    },
    [boxerKey, animationName, frameIndex]
  );

  return { add, isLoading, error };
}

export interface UseRemoveHitboxReturn {
  /** Remove a hitbox by index */
  remove: (hitboxIndex: number) => Promise<boolean>;
  /** Whether operation is in progress */
  isLoading: boolean;
  /** Error message if operation failed */
  error: string | null;
}

/**
 * Hook for removing a hitbox
 * @param boxerKey - The boxer identifier
 * @param animationName - The animation name
 * @param frameIndex - The frame index
 * @returns Mutation function and state for removing a hitbox
 */
export function useRemoveHitbox(
  boxerKey: BoxerKey,
  animationName: string,
  frameIndex: number
): UseRemoveHitboxReturn {
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const remove = useCallback(
    async (hitboxIndex: number): Promise<boolean> => {
      setIsLoading(true);
      setError(null);
      try {
        await invoke<void>('delete_hitbox', {
          boxer_key: boxerKey,
          animation_name: animationName,
          frame_index: frameIndex,
          hitbox_index: hitboxIndex,
        });
        return true;
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : String(err);
        setError(`Failed to remove hitbox: ${errorMessage}`);
        console.error('Failed to remove hitbox:', err);
        return false;
      } finally {
        setIsLoading(false);
      }
    },
    [boxerKey, animationName, frameIndex]
  );

  return { remove, isLoading, error };
}

// ============================================================================
// Combined Animation Hook
// ============================================================================

export interface UseAnimationReturn {
  // Animation data
  animation: UseBoxerAnimationReturn;
  // Player controls
  player: UseAnimationPlayerReturn;
  // Hitbox management
  hitboxes: {
    data: HitboxData[];
    isLoading: boolean;
    error: string | null;
    refresh: () => Promise<void>;
    update: (hitbox: HitboxData) => Promise<boolean>;
    add: (hitbox: Omit<HitboxData, 'id'>) => Promise<HitboxData | null>;
    remove: (hitboxIndex: number) => Promise<boolean>;
    isMutating: boolean;
  };
}

/**
 * Comprehensive hook for managing boxer animations
 * @param boxerKey - The boxer identifier
 * @param animationName - The animation name
 * @returns Combined animation data and controls
 */
export function useAnimation(
  boxerKey: BoxerKey,
  animationName: string
): UseAnimationReturn {
  const animation = useBoxerAnimation(boxerKey, animationName);
  const player = useAnimationPlayer();

  // Use the first frame for hitbox management (can be changed based on player state)
  const currentFrame = player.state?.current_frame ?? 0;
  const hitboxesQuery = useHitboxes(boxerKey, animationName, currentFrame);
  const updateHitboxMutation = useUpdateHitbox(boxerKey, animationName, currentFrame);
  const addHitboxMutation = useAddHitbox(boxerKey, animationName, currentFrame);
  const removeHitboxMutation = useRemoveHitbox(boxerKey, animationName, currentFrame);

  const isMutating =
    updateHitboxMutation.isLoading ||
    addHitboxMutation.isLoading ||
    removeHitboxMutation.isLoading;

  // Refresh hitboxes when frame changes
  useEffect(() => {
    hitboxesQuery.refresh();
  }, [currentFrame, boxerKey, animationName]);

  return {
    animation,
    player,
    hitboxes: {
      data: hitboxesQuery.hitboxes,
      isLoading: hitboxesQuery.isLoading,
      error: hitboxesQuery.error,
      refresh: hitboxesQuery.refresh,
      update: updateHitboxMutation.update,
      add: addHitboxMutation.add,
      remove: removeHitboxMutation.remove,
      isMutating,
    },
  };
}

export default useAnimation;
