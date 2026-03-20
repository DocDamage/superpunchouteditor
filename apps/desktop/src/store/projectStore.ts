/**
 * Project Store
 * 
 * Manages project save/load, metadata, and project-related operations.
 */

import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import { invoke } from '@tauri-apps/api/core';
import type { ProjectFile, ProjectMetadata, ProjectThumbnail } from './types';

// ============================================================================
// Types
// ============================================================================

export interface ProjectState {
  // Current project
  currentProject: ProjectFile | null;
  currentProjectPath: string | null;
  isModified: boolean;
  
  // Recent projects (persisted)
  recentProjects: string[];
  
  // Loading state
  isLoading: boolean;
  isSaving: boolean;
  error: string | null;
  
  // Computed
  readonly hasProject: boolean;
  readonly projectName: string | null;
}

export interface ProjectActions {
  // Project lifecycle
  createProject: (path: string, name: string, author?: string, description?: string) => Promise<void>;
  loadProject: (path: string) => Promise<void>;
  saveProject: (path?: string, metadata?: ProjectMetadata) => Promise<void>;
  closeProject: () => void;
  validateProject: (path: string) => Promise<boolean>;
  
  // Recent projects
  addRecentProject: (path: string) => void;
  removeRecentProject: (path: string) => void;
  clearRecentProjects: () => void;
  
  // Thumbnails
  captureThumbnail: (viewType: string) => Promise<ProjectThumbnail | null>;
  saveThumbnail: (thumbnail: ProjectThumbnail) => Promise<void>;
  loadThumbnailFromPath: (projectPath: string) => Promise<ProjectThumbnail | null>;
  
  // State management
  setModified: (modified: boolean) => void;
  setError: (error: string | null) => void;
  clearError: () => void;
  loadCurrentProject: () => Promise<void>;
}

export type ProjectStore = ProjectState & ProjectActions;

// ============================================================================
// Constants
// ============================================================================

const MAX_RECENT_PROJECTS = 10;

// ============================================================================
// Store Implementation
// ============================================================================

export const useProjectStore = create<ProjectStore>()(
  persist(
    (set, get) => ({
      // Initial state
      currentProject: null,
      currentProjectPath: null,
      isModified: false,
      recentProjects: [],
      isLoading: false,
      isSaving: false,
      error: null,
      
      // Computed getters
      get hasProject() {
        return get().currentProject !== null;
      },
      get projectName() {
        return get().currentProject?.metadata.name || null;
      },
      
      // Actions
      createProject: async (path, name, author, description) => {
        set({ isLoading: true, error: null });
        try {
          const project = await invoke<ProjectFile>('create_project', {
            projectPath: path,
            name,
            author: author || null,
            description: description || null,
          });
          
          set({
            currentProject: project,
            currentProjectPath: path,
            isModified: false,
            isLoading: false,
          });
          
          get().addRecentProject(path);
        } catch (e) {
          console.error('Failed to create project:', e);
          set({ error: (e as Error).message, isLoading: false });
          throw e;
        }
      },
      
      loadProject: async (path: string) => {
        set({ isLoading: true, error: null });
        try {
          const project = await invoke<ProjectFile>('load_project', { projectPath: path });
          
          set({
            currentProject: project,
            currentProjectPath: path,
            isModified: false,
            isLoading: false,
          });
          
          get().addRecentProject(path);
        } catch (e) {
          console.error('Failed to load project:', e);
          set({ error: (e as Error).message, isLoading: false });
          throw e;
        }
      },
      
      saveProject: async (path?, metadata?) => {
        set({ isSaving: true, error: null });
        try {
          const project = await invoke<ProjectFile>('save_project', {
            projectPath: path || null,
            metadata: metadata || null,
          });
          
          set({
            currentProject: project,
            isModified: false,
            isSaving: false,
          });
          
          if (path) {
            set({ currentProjectPath: path });
            get().addRecentProject(path);
          }
        } catch (e) {
          console.error('Failed to save project:', e);
          set({ error: (e as Error).message, isSaving: false });
          throw e;
        }
      },
      
      closeProject: () => {
        set({
          currentProject: null,
          currentProjectPath: null,
          isModified: false,
        });
      },
      
      validateProject: async (path: string) => {
        try {
          return await invoke<boolean>('validate_project', { projectPath: path });
        } catch (e) {
          console.error('Failed to validate project:', e);
          return false;
        }
      },
      
      addRecentProject: (path: string) => {
        set((state) => {
          const filtered = state.recentProjects.filter(p => p !== path);
          const updated = [path, ...filtered].slice(0, MAX_RECENT_PROJECTS);
          return { recentProjects: updated };
        });
      },
      
      removeRecentProject: (path: string) => {
        set((state) => ({
          recentProjects: state.recentProjects.filter(p => p !== path)
        }));
      },
      
      clearRecentProjects: () => {
        set({ recentProjects: [] });
      },
      
      captureThumbnail: async (viewType: string) => {
        try {
          // This will be implemented with the actual screenshot capture
          // For now, return null as placeholder
          return null;
        } catch (e) {
          console.error('Failed to capture thumbnail:', e);
          return null;
        }
      },
      
      saveThumbnail: async (thumbnail: ProjectThumbnail) => {
        try {
          await invoke('save_project_thumbnail', { thumbnailData: thumbnail });
          
          // Update current project with new thumbnail
          set((state) => {
            if (!state.currentProject) return state;
            return {
              currentProject: {
                ...state.currentProject,
                thumbnail,
              }
            };
          });
        } catch (e) {
          console.error('Failed to save thumbnail:', e);
          throw e;
        }
      },
      
      loadThumbnailFromPath: async (projectPath: string) => {
        try {
          return await invoke<ProjectThumbnail | null>('load_project_thumbnail_from_path', { projectPath });
        } catch (e) {
          console.error('Failed to load thumbnail:', e);
          return null;
        }
      },
      
      setModified: (modified: boolean) => set({ isModified: modified }),
      setError: (error: string | null) => set({ error }),
      clearError: () => set({ error: null }),
      
      loadCurrentProject: async () => {
        try {
          const [project, path] = await Promise.all([
            invoke<ProjectFile | null>('get_current_project'),
            invoke<string | null>('get_current_project_path'),
          ]);
          set({ currentProject: project, currentProjectPath: path });
        } catch (e) {
          console.error('Failed to load current project:', e);
        }
      },
    }),
    {
      name: 'spo-project-storage',
      partialize: (state) => ({
        recentProjects: state.recentProjects,
        // Don't persist current project - it should be loaded from backend
      }),
    }
  )
);

// ============================================================================
// Selectors
// ============================================================================

export const selectCurrentProject = (state: ProjectStore) => state.currentProject;
export const selectHasProject = (state: ProjectStore) => state.hasProject;
export const selectIsProjectModified = (state: ProjectStore) => state.isModified;
export const selectRecentProjects = (state: ProjectStore) => state.recentProjects;
export const selectProjectError = (state: ProjectStore) => state.error;
