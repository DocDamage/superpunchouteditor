/**
 * UI Store
 * 
 * Manages UI state like active tabs, modals, toasts, and theme preferences.
 */

import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import type { Toast, ModalState } from './types';

// ============================================================================
// Types
// ============================================================================

export type TabId = 
  | 'boxer-select'
  | 'palette-editor'
  | 'sprite-editor'
  | 'animation-editor'
  | 'frame-reconstructor'
  | 'script-viewer'
  | 'audio-editor'
  | 'project-manager'
  | 'comparison'
  | 'settings';

export type ThemeMode = 'light' | 'dark' | 'system';

export interface UiState {
  // Navigation
  activeTab: TabId;
  sidebarCollapsed: boolean;
  
  // Modals
  activeModal: ModalState;
  modalStack: ModalState[];
  
  // Toasts
  toasts: Toast[];
  
  // Theme
  theme: ThemeMode;
  
  // Layout preferences (persisted)
  layout: {
    panelSizes: Record<string, number>;
    visiblePanels: string[];
    sidebarWidth: number;
  };
  
  // Computed
  readonly isModalOpen: boolean;
  readonly activeModalType: string | null;
}

export interface UiActions {
  // Tab navigation
  setActiveTab: (tab: TabId) => void;
  
  // Sidebar
  toggleSidebar: () => void;
  setSidebarCollapsed: (collapsed: boolean) => void;
  
  // Modals
  openModal: (type: string, data?: unknown) => void;
  closeModal: () => void;
  closeAllModals: () => void;
  pushModal: (type: string, data?: unknown) => void;
  popModal: () => void;
  
  // Toasts
  addToast: (message: string, type?: Toast['type'], duration?: number) => string;
  removeToast: (id: string) => void;
  clearToasts: () => void;
  
  // Theme
  setTheme: (theme: ThemeMode) => void;
  toggleTheme: () => void;
  
  // Layout
  setPanelSize: (panelId: string, size: number) => void;
  togglePanel: (panelId: string) => void;
  setSidebarWidth: (width: number) => void;
  
  // Reset
  resetLayout: () => void;
}

export type UiStore = UiState & UiActions;

// ============================================================================
// Constants
// ============================================================================

const DEFAULT_LAYOUT = {
  panelSizes: {
    'preview': 300,
    'properties': 250,
  },
  visiblePanels: ['preview', 'properties'],
  sidebarWidth: 240,
};

// ============================================================================
// Store Implementation
// ============================================================================

export const useUiStore = create<UiStore>()(
  persist(
    (set, get) => ({
      // Initial state
      activeTab: 'boxer-select',
      sidebarCollapsed: false,
      activeModal: { isOpen: false, type: null },
      modalStack: [],
      toasts: [],
      theme: 'system',
      layout: { ...DEFAULT_LAYOUT },
      
      // Computed getters
      get isModalOpen() {
        return get().activeModal.isOpen;
      },
      get activeModalType() {
        return get().activeModal.type;
      },
      
      // Actions
      setActiveTab: (tab: TabId) => set({ activeTab: tab }),
      
      toggleSidebar: () => set((state) => ({ sidebarCollapsed: !state.sidebarCollapsed })),
      setSidebarCollapsed: (collapsed: boolean) => set({ sidebarCollapsed: collapsed }),
      
      openModal: (type: string, data?: unknown) => {
        set({
          activeModal: { isOpen: true, type, data }
        });
      },
      
      closeModal: () => {
        set({ activeModal: { isOpen: false, type: null } });
      },
      
      closeAllModals: () => {
        set({
          activeModal: { isOpen: false, type: null },
          modalStack: []
        });
      },
      
      pushModal: (type: string, data?: unknown) => {
        set((state) => ({
          modalStack: [...state.modalStack, state.activeModal],
          activeModal: { isOpen: true, type, data }
        }));
      },
      
      popModal: () => {
        set((state) => {
          const previous = state.modalStack[state.modalStack.length - 1];
          if (!previous) {
            return { activeModal: { isOpen: false, type: null }, modalStack: [] };
          }
          return {
            activeModal: previous,
            modalStack: state.modalStack.slice(0, -1)
          };
        });
      },
      
      addToast: (message: string, type: Toast['type'] = 'info', duration = 5000) => {
        const id = Math.random().toString(36).substring(2, 9);
        const toast: Toast = { id, message, type, duration };
        
        set((state) => ({ toasts: [...state.toasts, toast] }));
        
        // Auto-remove toast after duration
        if (duration > 0) {
          setTimeout(() => {
            get().removeToast(id);
          }, duration);
        }
        
        return id;
      },
      
      removeToast: (id: string) => {
        set((state) => ({
          toasts: state.toasts.filter(t => t.id !== id)
        }));
      },
      
      clearToasts: () => set({ toasts: [] }),
      
      setTheme: (theme: ThemeMode) => set({ theme }),
      
      toggleTheme: () => {
        set((state) => {
          const themes: ThemeMode[] = ['light', 'dark', 'system'];
          const currentIndex = themes.indexOf(state.theme);
          const nextTheme = themes[(currentIndex + 1) % themes.length];
          return { theme: nextTheme };
        });
      },
      
      setPanelSize: (panelId: string, size: number) => {
        set((state) => ({
          layout: {
            ...state.layout,
            panelSizes: { ...state.layout.panelSizes, [panelId]: size }
          }
        }));
      },
      
      togglePanel: (panelId: string) => {
        set((state) => {
          const visible = state.layout.visiblePanels;
          const isVisible = visible.includes(panelId);
          return {
            layout: {
              ...state.layout,
              visiblePanels: isVisible
                ? visible.filter(id => id !== panelId)
                : [...visible, panelId]
            }
          };
        });
      },
      
      setSidebarWidth: (width: number) => {
        set((state) => ({
          layout: { ...state.layout, sidebarWidth: width }
        }));
      },
      
      resetLayout: () => {
        set({ layout: { ...DEFAULT_LAYOUT } });
      },
    }),
    {
      name: 'spo-ui-storage',
      partialize: (state) => ({
        theme: state.theme,
        layout: state.layout,
        sidebarCollapsed: state.sidebarCollapsed,
      }),
    }
  )
);

// ============================================================================
// Selectors
// ============================================================================

export const selectActiveTab = (state: UiStore) => state.activeTab;
export const selectIsModalOpen = (state: UiStore) => state.isModalOpen;
export const selectActiveModalType = (state: UiStore) => state.activeModalType;
export const selectToasts = (state: UiStore) => state.toasts;
export const selectTheme = (state: UiStore) => state.theme;
