/**
 * useBankManagement Hook
 * 
 * React hooks for bank management Tauri commands.
 * Provides functionality for analyzing ROM bank usage, fragmentation, and defragmentation.
 */

import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type {
  BankMap,
  BankStatistics,
  MemoryRegion,
  FragmentationAnalysis,
  DefragPlan,
} from '../types/api';

export interface UseBankMapState {
  /** Bank map data */
  data: BankMap | null;
  /** Whether data is loading */
  isLoading: boolean;
  /** Error message if loading failed */
  error: string | null;
}

export interface UseBankMapReturn extends UseBankMapState {
  /** Refresh the bank map */
  refresh: () => Promise<void>;
}

/**
 * Hook for fetching the ROM bank map
 * @returns Bank map data and loading state
 */
export function useBankMap(): UseBankMapReturn {
  const [data, setData] = useState<BankMap | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async (): Promise<void> => {
    setIsLoading(true);
    setError(null);
    try {
      const result = await invoke<BankMap>('get_bank_visualization');
      setData(result);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      setError(`Failed to load bank map: ${errorMessage}`);
      console.error('Failed to get bank map:', err);
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    refresh();
  }, [refresh]);

  return { data, isLoading, error, refresh };
}

export interface UseBankStatisticsState {
  /** Bank statistics data */
  data: BankStatistics | null;
  /** Whether data is loading */
  isLoading: boolean;
  /** Error message if loading failed */
  error: string | null;
}

export interface UseBankStatisticsReturn extends UseBankStatisticsState {
  /** Refresh the bank statistics */
  refresh: () => Promise<void>;
}

/**
 * Hook for fetching ROM bank statistics
 * @returns Bank statistics data and loading state
 */
export function useBankStatistics(): UseBankStatisticsReturn {
  const [data, setData] = useState<BankStatistics | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async (): Promise<void> => {
    setIsLoading(true);
    setError(null);
    try {
      const result = await invoke<BankStatistics>('get_rom_statistics');
      setData(result);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      setError(`Failed to load bank statistics: ${errorMessage}`);
      console.error('Failed to get bank statistics:', err);
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    refresh();
  }, [refresh]);

  return { data, isLoading, error, refresh };
}

export interface UseFragmentationAnalysisState {
  /** Fragmentation analysis data */
  data: FragmentationAnalysis | null;
  /** Whether data is loading */
  isLoading: boolean;
  /** Error message if loading failed */
  error: string | null;
}

export interface UseFragmentationAnalysisReturn extends UseFragmentationAnalysisState {
  /** Refresh the fragmentation analysis */
  refresh: () => Promise<void>;
}

/**
 * Hook for analyzing ROM bank fragmentation
 * @returns Fragmentation analysis data and loading state
 */
export function useFragmentationAnalysis(): UseFragmentationAnalysisReturn {
  const [data, setData] = useState<FragmentationAnalysis | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async (): Promise<void> => {
    setIsLoading(true);
    setError(null);
    try {
      const result = await invoke<FragmentationAnalysis>('analyze_fragmentation');
      setData(result);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      setError(`Failed to analyze fragmentation: ${errorMessage}`);
      console.error('Failed to analyze fragmentation:', err);
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    refresh();
  }, [refresh]);

  return { data, isLoading, error, refresh };
}

export interface UseDefragmentationPlanState {
  /** Defragmentation plan data */
  data: DefragPlan | null;
  /** Whether data is loading */
  isLoading: boolean;
  /** Error message if loading failed */
  error: string | null;
}

export interface UseDefragmentationPlanReturn extends UseDefragmentationPlanState {
  /** Refresh the defragmentation plan */
  refresh: () => Promise<void>;
}

/**
 * Hook for getting a defragmentation plan
 * @returns Defragmentation plan data and loading state
 */
export function useDefragmentationPlan(): UseDefragmentationPlanReturn {
  const [data, setData] = useState<DefragPlan | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async (): Promise<void> => {
    setIsLoading(true);
    setError(null);
    try {
      const result = await invoke<DefragPlan>('generate_defrag_plan');
      setData(result);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      setError(`Failed to get defragmentation plan: ${errorMessage}`);
      console.error('Failed to get defragmentation plan:', err);
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    refresh();
  }, [refresh]);

  return { data, isLoading, error, refresh };
}

export interface UseDefragmentationReturn {
  /** Execute defragmentation */
  execute: () => Promise<boolean>;
  /** Whether operation is in progress */
  isLoading: boolean;
  /** Error message if operation failed */
  error: string | null;
  /** Success message if operation succeeded */
  success: string | null;
}

/**
 * Hook for executing ROM bank defragmentation
 * @returns Mutation function and state for defragmentation
 */
export function useDefragmentation(): UseDefragmentationReturn {
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  const execute = useCallback(async (): Promise<boolean> => {
    setIsLoading(true);
    setError(null);
    setSuccess(null);
    try {
      await invoke<void>('execute_defrag_plan');
      setSuccess('Defragmentation completed successfully');
      return true;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      setError(`Defragmentation failed: ${errorMessage}`);
      console.error('Failed to execute defragmentation:', err);
      return false;
    } finally {
      setIsLoading(false);
    }
  }, []);

  return { execute, isLoading, error, success };
}

export interface UseFreeRegionsState {
  /** List of free memory regions */
  regions: MemoryRegion[];
  /** Whether data is loading */
  isLoading: boolean;
  /** Error message if loading failed */
  error: string | null;
}

export interface UseFreeRegionsReturn extends UseFreeRegionsState {
  /** Refresh free regions with optional minimum size filter */
  refresh: (minSize?: number) => Promise<void>;
}

/**
 * Hook for finding free memory regions
 * @returns Free regions data and loading state
 */
export function useFreeRegions(): UseFreeRegionsReturn {
  const [regions, setRegions] = useState<MemoryRegion[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async (minSize?: number): Promise<void> => {
    setIsLoading(true);
    setError(null);
    try {
      const result = await invoke<MemoryRegion[]>('find_free_regions', {
        min_size: minSize ?? 0,
      });
      setRegions(result);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      setError(`Failed to find free regions: ${errorMessage}`);
      console.error('Failed to find free regions:', err);
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    refresh();
  }, [refresh]);

  return { regions, isLoading, error, refresh };
}

/**
 * Combined hook for all bank management operations
 * @returns All bank management hooks and data
 */
export function useBankManagement() {
  const bankMap = useBankMap();
  const bankStatistics = useBankStatistics();
  const fragmentationAnalysis = useFragmentationAnalysis();
  const defragmentationPlan = useDefragmentationPlan();
  const defragmentation = useDefragmentation();
  const freeRegions = useFreeRegions();

  /**
   * Refresh all bank management data
   */
  const refreshAll = useCallback(async (): Promise<void> => {
    await Promise.all([
      bankMap.refresh(),
      bankStatistics.refresh(),
      fragmentationAnalysis.refresh(),
      defragmentationPlan.refresh(),
      freeRegions.refresh(),
    ]);
  }, [
    bankMap,
    bankStatistics,
    fragmentationAnalysis,
    defragmentationPlan,
    freeRegions,
  ]);

  return {
    // Individual hooks
    bankMap,
    bankStatistics,
    fragmentationAnalysis,
    defragmentationPlan,
    defragmentation,
    freeRegions,
    // Combined actions
    refreshAll,
  };
}

export default useBankManagement;
