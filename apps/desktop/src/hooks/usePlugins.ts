/**
 * usePlugins Hook
 * 
 * React hooks for the plugin system Tauri commands.
 * Manages plugin loading, unloading, enabling/disabling, and script execution.
 */

import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { PluginInfo, ScriptExecutionResult, BatchJobInfo } from '../types/api';

export interface UsePluginsState {
  /** List of all loaded plugins */
  plugins: PluginInfo[];
  /** Whether plugins are being loaded */
  isLoading: boolean;
  /** Error message if loading failed */
  error: string | null;
}

export interface UsePluginsReturn extends UsePluginsState {
  /** Refresh the plugin list */
  refreshPlugins: () => Promise<void>;
  /** Load a plugin from a file path */
  loadPlugin: (path: string) => Promise<PluginInfo | null>;
  /** Unload a plugin by ID */
  unloadPlugin: (pluginId: string) => Promise<boolean>;
  /** Enable a plugin */
  enablePlugin: (pluginId: string) => Promise<boolean>;
  /** Disable a plugin */
  disablePlugin: (pluginId: string) => Promise<boolean>;
  /** Execute a command on a plugin */
  executePluginCommand: (
    pluginId: string,
    command: string,
    args?: Record<string, unknown>
  ) => Promise<unknown>;
  /** Run a script string */
  runScript: (script: string) => Promise<ScriptExecutionResult>;
  /** Run a script from a file */
  runScriptFile: (path: string) => Promise<ScriptExecutionResult>;
  /** Get list of batch jobs */
  listBatchJobs: () => Promise<BatchJobInfo[]>;
}

/**
 * Hook for managing plugins
 */
export function usePlugins(): UsePluginsReturn {
  const [plugins, setPlugins] = useState<PluginInfo[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  /**
   * Fetch the list of loaded plugins
   */
  const refreshPlugins = useCallback(async (): Promise<void> => {
    setIsLoading(true);
    setError(null);
    try {
      const result = await invoke<PluginInfo[]>('list_plugins');
      setPlugins(result);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      setError(`Failed to load plugins: ${errorMessage}`);
      console.error('Failed to list plugins:', err);
    } finally {
      setIsLoading(false);
    }
  }, []);

  /**
   * Load a plugin from a file path
   */
  const loadPlugin = useCallback(async (path: string): Promise<PluginInfo | null> => {
    try {
      const result = await invoke<PluginInfo>('load_plugin', { path });
      // Refresh the plugin list after loading
      await refreshPlugins();
      return result;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      setError(`Failed to load plugin: ${errorMessage}`);
      console.error('Failed to load plugin:', err);
      return null;
    }
  }, [refreshPlugins]);

  /**
   * Unload a plugin by ID
   */
  const unloadPlugin = useCallback(async (pluginId: string): Promise<boolean> => {
    try {
      await invoke<void>('unload_plugin', { plugin_id: pluginId });
      // Refresh the plugin list after unloading
      await refreshPlugins();
      return true;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      setError(`Failed to unload plugin: ${errorMessage}`);
      console.error('Failed to unload plugin:', err);
      return false;
    }
  }, [refreshPlugins]);

  /**
   * Enable a plugin
   */
  const enablePlugin = useCallback(async (pluginId: string): Promise<boolean> => {
    try {
      await invoke<void>('enable_plugin', { plugin_id: pluginId });
      // Update local state to reflect the change
      setPlugins(prev =>
        prev.map(p => (p.id === pluginId ? { ...p, enabled: true } : p))
      );
      return true;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      setError(`Failed to enable plugin: ${errorMessage}`);
      console.error('Failed to enable plugin:', err);
      return false;
    }
  }, []);

  /**
   * Disable a plugin
   */
  const disablePlugin = useCallback(async (pluginId: string): Promise<boolean> => {
    try {
      await invoke<void>('disable_plugin', { plugin_id: pluginId });
      // Update local state to reflect the change
      setPlugins(prev =>
        prev.map(p => (p.id === pluginId ? { ...p, enabled: false } : p))
      );
      return true;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      setError(`Failed to disable plugin: ${errorMessage}`);
      console.error('Failed to disable plugin:', err);
      return false;
    }
  }, []);

  /**
   * Execute a command on a plugin
   */
  const executePluginCommand = useCallback(
    async (
      pluginId: string,
      command: string,
      args?: Record<string, unknown>
    ): Promise<unknown> => {
      try {
        const result = await invoke<unknown>('execute_plugin_command', {
          plugin_id: pluginId,
          command,
          args: args ?? {},
        });
        return result;
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : String(err);
        setError(`Failed to execute plugin command: ${errorMessage}`);
        console.error('Failed to execute plugin command:', err);
        throw err;
      }
    },
    []
  );

  /**
   * Run a script string
   */
  const runScript = useCallback(async (script: string): Promise<ScriptExecutionResult> => {
    try {
      const result = await invoke<ScriptExecutionResult>('run_script', { script });
      return result;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      setError(`Failed to run script: ${errorMessage}`);
      console.error('Failed to run script:', err);
      return {
        success: false,
        output: '',
        error: errorMessage,
        execution_time_ms: 0,
      };
    }
  }, []);

  /**
   * Run a script from a file
   */
  const runScriptFile = useCallback(async (path: string): Promise<ScriptExecutionResult> => {
    try {
      const result = await invoke<ScriptExecutionResult>('run_script_file', { path });
      return result;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      setError(`Failed to run script file: ${errorMessage}`);
      console.error('Failed to run script file:', err);
      return {
        success: false,
        output: '',
        error: errorMessage,
        execution_time_ms: 0,
      };
    }
  }, []);

  /**
   * Get list of batch jobs
   */
  const listBatchJobs = useCallback(async (): Promise<BatchJobInfo[]> => {
    try {
      const result = await invoke<BatchJobInfo[]>('list_batch_jobs');
      return result;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      setError(`Failed to list batch jobs: ${errorMessage}`);
      console.error('Failed to list batch jobs:', err);
      return [];
    }
  }, []);

  // Load plugins on mount
  useEffect(() => {
    refreshPlugins();
  }, [refreshPlugins]);

  return {
    plugins,
    isLoading,
    error,
    refreshPlugins,
    loadPlugin,
    unloadPlugin,
    enablePlugin,
    disablePlugin,
    executePluginCommand,
    runScript,
    runScriptFile,
    listBatchJobs,
  };
}

/**
 * Hook for loading a plugin
 * @returns Mutation function and state for loading a plugin
 */
export function useLoadPlugin() {
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [data, setData] = useState<PluginInfo | null>(null);

  const mutate = useCallback(async (path: string): Promise<PluginInfo | null> => {
    setIsLoading(true);
    setError(null);
    try {
      const result = await invoke<PluginInfo>('load_plugin', { path });
      setData(result);
      return result;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      setError(errorMessage);
      console.error('Failed to load plugin:', err);
      return null;
    } finally {
      setIsLoading(false);
    }
  }, []);

  return { mutate, isLoading, error, data };
}

/**
 * Hook for unloading a plugin
 * @returns Mutation function and state for unloading a plugin
 */
export function useUnloadPlugin() {
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const mutate = useCallback(async (pluginId: string): Promise<boolean> => {
    setIsLoading(true);
    setError(null);
    try {
      await invoke<void>('unload_plugin', { plugin_id: pluginId });
      return true;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      setError(errorMessage);
      console.error('Failed to unload plugin:', err);
      return false;
    } finally {
      setIsLoading(false);
    }
  }, []);

  return { mutate, isLoading, error };
}

/**
 * Hook for enabling a plugin
 * @returns Mutation function and state for enabling a plugin
 */
export function useEnablePlugin() {
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const mutate = useCallback(async (pluginId: string): Promise<boolean> => {
    setIsLoading(true);
    setError(null);
    try {
      await invoke<void>('enable_plugin', { plugin_id: pluginId });
      return true;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      setError(errorMessage);
      console.error('Failed to enable plugin:', err);
      return false;
    } finally {
      setIsLoading(false);
    }
  }, []);

  return { mutate, isLoading, error };
}

/**
 * Hook for disabling a plugin
 * @returns Mutation function and state for disabling a plugin
 */
export function useDisablePlugin() {
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const mutate = useCallback(async (pluginId: string): Promise<boolean> => {
    setIsLoading(true);
    setError(null);
    try {
      await invoke<void>('disable_plugin', { plugin_id: pluginId });
      return true;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      setError(errorMessage);
      console.error('Failed to disable plugin:', err);
      return false;
    } finally {
      setIsLoading(false);
    }
  }, []);

  return { mutate, isLoading, error };
}

/**
 * Hook for executing a plugin command
 * @returns Mutation function and state for executing plugin commands
 */
export function useExecutePluginCommand() {
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [data, setData] = useState<unknown>(null);

  const mutate = useCallback(
    async (
      pluginId: string,
      command: string,
      args?: Record<string, unknown>
    ): Promise<unknown> => {
      setIsLoading(true);
      setError(null);
      try {
        const result = await invoke<unknown>('execute_plugin_command', {
          plugin_id: pluginId,
          command,
          args: args ?? {},
        });
        setData(result);
        return result;
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : String(err);
        setError(errorMessage);
        console.error('Failed to execute plugin command:', err);
        throw err;
      } finally {
        setIsLoading(false);
      }
    },
    []
  );

  return { mutate, isLoading, error, data };
}

/**
 * Hook for running a script
 * @returns Mutation function and state for running a script
 */
export function useRunScript() {
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [data, setData] = useState<ScriptExecutionResult | null>(null);

  const mutate = useCallback(async (script: string): Promise<ScriptExecutionResult> => {
    setIsLoading(true);
    setError(null);
    try {
      const result = await invoke<ScriptExecutionResult>('run_script', { script });
      setData(result);
      return result;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      setError(errorMessage);
      console.error('Failed to run script:', err);
      const failureResult: ScriptExecutionResult = {
        success: false,
        output: '',
        error: errorMessage,
        execution_time_ms: 0,
      };
      setData(failureResult);
      return failureResult;
    } finally {
      setIsLoading(false);
    }
  }, []);

  return { mutate, isLoading, error, data };
}

/**
 * Hook for running a script file
 * @returns Mutation function and state for running a script file
 */
export function useRunScriptFile() {
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [data, setData] = useState<ScriptExecutionResult | null>(null);

  const mutate = useCallback(async (path: string): Promise<ScriptExecutionResult> => {
    setIsLoading(true);
    setError(null);
    try {
      const result = await invoke<ScriptExecutionResult>('run_script_file', { path });
      setData(result);
      return result;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      setError(errorMessage);
      console.error('Failed to run script file:', err);
      const failureResult: ScriptExecutionResult = {
        success: false,
        output: '',
        error: errorMessage,
        execution_time_ms: 0,
      };
      setData(failureResult);
      return failureResult;
    } finally {
      setIsLoading(false);
    }
  }, []);

  return { mutate, isLoading, error, data };
}

/**
 * Hook for listing batch jobs
 * @returns Query result for batch jobs
 */
export function useBatchJobs() {
  const [jobs, setJobs] = useState<BatchJobInfo[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async (): Promise<void> => {
    setIsLoading(true);
    setError(null);
    try {
      const result = await invoke<BatchJobInfo[]>('list_batch_jobs');
      setJobs(result);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      setError(`Failed to load batch jobs: ${errorMessage}`);
      console.error('Failed to list batch jobs:', err);
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    refresh();
  }, [refresh]);

  return { jobs, isLoading, error, refresh };
}

export default usePlugins;
