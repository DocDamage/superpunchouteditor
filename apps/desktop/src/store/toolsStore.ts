/**
 * Tools Store
 * 
 * Manages external tools configuration and integration.
 */

import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import { invoke } from '@tauri-apps/api/core';
import type { ExternalTool, ToolContext } from './types';

// ============================================================================
// Types
// ============================================================================

export interface ToolsState {
  // Tools list
  tools: ExternalTool[];
  defaultToolIds: Record<string, string>; // file extension -> tool id
  
  // Loading state
  isLoading: boolean;
  error: string | null;
  
  // Computed
  readonly enabledTools: ExternalTool[];
  readonly toolsByCategory: Record<string, ExternalTool[]>;
}

export interface ToolsActions {
  // Data loading
  loadTools: () => Promise<void>;
  
  // Tool CRUD
  addTool: (tool: ExternalTool) => Promise<void>;
  removeTool: (toolId: string) => Promise<void>;
  updateTool: (tool: ExternalTool) => Promise<void>;
  
  // Default tools
  setDefaultTool: (fileExtension: string, toolId: string) => Promise<void>;
  getDefaultTool: (fileExtension: string) => ExternalTool | null;
  
  // Tool operations
  launchWithTool: (toolId: string, filePath: string, context?: ToolContext) => Promise<void>;
  getCompatibleTools: (fileExtension: string) => ExternalTool[];
  verifyTool: (tool: ExternalTool) => Promise<{ valid: boolean; message: string }>;
  
  // Error handling
  setError: (error: string | null) => void;
  clearError: () => void;
}

export type ToolsStore = ToolsState & ToolsActions;

// ============================================================================
// Store Implementation
// ============================================================================

export const useToolsStore = create<ToolsStore>()(
  persist(
    (set, get) => ({
      // Initial state
      tools: [],
      defaultToolIds: {},
      isLoading: false,
      error: null,
      
      // Computed getters
      get enabledTools() {
        return get().tools.filter(t => t.enabled);
      },
      get toolsByCategory() {
        const grouped: Record<string, ExternalTool[]> = {};
        for (const tool of get().tools) {
          if (!grouped[tool.category]) {
            grouped[tool.category] = [];
          }
          grouped[tool.category].push(tool);
        }
        return grouped;
      },
      
      // Actions
      loadTools: async () => {
        set({ isLoading: true, error: null });
        try {
          const tools = await invoke<ExternalTool[]>('get_external_tools');
          set({ tools, isLoading: false });
        } catch (e) {
          console.error('Failed to load external tools:', e);
          set({ error: (e as Error).message, isLoading: false });
        }
      },
      
      addTool: async (tool: ExternalTool) => {
        try {
          await invoke('add_external_tool', { tool });
          await get().loadTools();
        } catch (e) {
          console.error('Failed to add tool:', e);
          throw e;
        }
      },
      
      removeTool: async (toolId: string) => {
        try {
          await invoke('remove_external_tool', { toolId });
          await get().loadTools();
        } catch (e) {
          console.error('Failed to remove tool:', e);
          throw e;
        }
      },
      
      updateTool: async (tool: ExternalTool) => {
        try {
          await invoke('update_external_tool', { tool });
          await get().loadTools();
        } catch (e) {
          console.error('Failed to update tool:', e);
          throw e;
        }
      },
      
      setDefaultTool: async (fileExtension: string, toolId: string) => {
        try {
          await invoke('set_default_tool', { fileExtension, toolId });
          set((state) => ({
            defaultToolIds: { ...state.defaultToolIds, [fileExtension]: toolId }
          }));
        } catch (e) {
          console.error('Failed to set default tool:', e);
          throw e;
        }
      },
      
      getDefaultTool: (fileExtension: string) => {
        const { tools, defaultToolIds } = get();
        const toolId = defaultToolIds[fileExtension];
        if (!toolId) return null;
        return tools.find(t => t.id === toolId) || null;
      },
      
      launchWithTool: async (toolId: string, filePath: string, context?: ToolContext) => {
        try {
          await invoke('launch_with_tool', { toolId, filePath, context });
        } catch (e) {
          console.error('Failed to launch tool:', e);
          throw e;
        }
      },
      
      getCompatibleTools: (fileExtension: string) => {
        return get().tools.filter(tool =>
          tool.enabled &&
          tool.supported_file_types.some(ext =>
            ext.toLowerCase() === fileExtension.toLowerCase()
          )
        );
      },
      
      verifyTool: async (tool: ExternalTool) => {
        try {
          return await invoke<{ valid: boolean; message: string }>('verify_tool', { tool });
        } catch (e) {
          return { valid: false, message: String(e) };
        }
      },
      
      setError: (error: string | null) => set({ error }),
      clearError: () => set({ error: null }),
    }),
    {
      name: 'spo-tools-storage',
      partialize: (state) => ({
        defaultToolIds: state.defaultToolIds,
        // Don't persist tools list - it comes from backend
      }),
    }
  )
);

// ============================================================================
// Selectors
// ============================================================================

export const selectTools = (state: ToolsStore) => state.tools;
export const selectEnabledTools = (state: ToolsStore) => state.enabledTools;
export const selectToolsByCategory = (state: ToolsStore) => state.toolsByCategory;
