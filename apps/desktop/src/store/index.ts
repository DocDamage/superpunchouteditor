/**
 * State Management Index
 *
 * The authoritative frontend store is useStore (monolithic Zustand store).
 * Modular stores (romStore, boxerStore, etc.) were never fully adopted and
 * are not used by any component. They remain as files but are not exported
 * from this index.
 *
 * uiStore is a separate store for toast notifications and is used by ToastContainer.
 */

// Export shared types
export * from './types';

// Authoritative monolithic store
export { useStore } from './useStore';

// Toast / UI store (used by ToastContainer)
export { useUiStore } from './uiStore';
