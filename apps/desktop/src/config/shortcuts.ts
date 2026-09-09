/**
 * Keyboard Shortcut Registry for Super Punch-Out!! Editor
 * 
 * Central registry for all keyboard shortcuts used in the application.
 * Supports categorization, context-aware shortcuts, and formatting utilities.
 */

export enum ShortcutCategory {
  General = 'General',
  File = 'File Operations',
  Edit = 'Editing',
  View = 'View & Navigation',
  Tools = 'Tools',
  Advanced = 'Advanced',
}

export interface Shortcut {
  id: string;
  keys: string[];           // ['Ctrl', 'S']
  description: string;
  category: ShortcutCategory;
  context?: string;         // When active (e.g., 'palette-editor')
}

/**
 * All keyboard shortcuts used in the application
 */
export const SHORTCUTS: Shortcut[] = [
  // General
  { id: 'open-help', keys: ['F1'], description: 'Open Help', category: ShortcutCategory.General },
  { id: 'show-cheat-sheet', keys: ['Ctrl', '?'], description: 'Show this cheat sheet', category: ShortcutCategory.General },
  { id: 'close-modal', keys: ['Escape'], description: 'Close modal / Cancel action', category: ShortcutCategory.General },
  
  // File Operations
  { id: 'open-rom', keys: ['Ctrl', 'O'], description: 'Open ROM file', category: ShortcutCategory.File },
  { id: 'save', keys: ['Ctrl', 'S'], description: 'Save project', category: ShortcutCategory.File },
  { id: 'save-as', keys: ['Ctrl', 'Shift', 'S'], description: 'Save project as...', category: ShortcutCategory.File },
  { id: 'export-patch', keys: ['Ctrl', 'E'], description: 'Export IPS patch', category: ShortcutCategory.File },
  
  // Editing
  { id: 'undo', keys: ['Ctrl', 'Z'], description: 'Undo last action', category: ShortcutCategory.Edit },
  { id: 'redo', keys: ['Ctrl', 'Shift', 'Z'], description: 'Redo last undone action', category: ShortcutCategory.Edit },
  { id: 'redo-alt', keys: ['Ctrl', 'Y'], description: 'Redo (alternative)', category: ShortcutCategory.Edit },
  { id: 'delete', keys: ['Delete'], description: 'Remove selected item', category: ShortcutCategory.Edit },
  { id: 'select-all', keys: ['Ctrl', 'A'], description: 'Select all items', category: ShortcutCategory.Edit },
  { id: 'copy', keys: ['Ctrl', 'C'], description: 'Copy selection', category: ShortcutCategory.Edit },
  { id: 'paste', keys: ['Ctrl', 'V'], description: 'Paste selection', category: ShortcutCategory.Edit },
  
  // View & Navigation
  { id: 'next-tab', keys: ['Ctrl', 'Tab'], description: 'Next tab', category: ShortcutCategory.View },
  { id: 'prev-tab', keys: ['Ctrl', 'Shift', 'Tab'], description: 'Previous tab', category: ShortcutCategory.View },
  { id: 'focus-search', keys: ['Ctrl', 'F'], description: 'Focus search', category: ShortcutCategory.View },
  { id: 'zoom-in', keys: ['Ctrl', '+'], description: 'Zoom in', category: ShortcutCategory.View },
  { id: 'zoom-out', keys: ['Ctrl', '-'], description: 'Zoom out', category: ShortcutCategory.View },
  { id: 'reset-zoom', keys: ['Ctrl', '0'], description: 'Reset zoom', category: ShortcutCategory.View },
  
  // Tools
  { id: 'test-emulator', keys: ['F5'], description: 'Test in emulator', category: ShortcutCategory.Tools },
  { id: 'test-emulator-save', keys: ['Ctrl', 'F5'], description: 'Save & test in emulator', category: ShortcutCategory.Tools },
  { id: 'open-palette-editor', keys: ['P'], description: 'Open Palette Editor', category: ShortcutCategory.Tools },
  { id: 'open-sprite-editor', keys: ['S'], description: 'Open Sprite Editor', category: ShortcutCategory.Tools },
  { id: 'open-animation-editor', keys: ['A'], description: 'Open Animation Editor', category: ShortcutCategory.Tools },
  
  // Advanced
  { id: 'toggle-dev-tools', keys: ['F12'], description: 'Toggle Developer Tools', category: ShortcutCategory.Advanced },
  { id: 'reload', keys: ['Ctrl', 'R'], description: 'Reload application', category: ShortcutCategory.Advanced },
  { id: 'force-reload', keys: ['Ctrl', 'Shift', 'R'], description: 'Force reload (clear cache)', category: ShortcutCategory.Advanced },
];

/**
 * Format shortcut keys into a readable string
 * @example ['Ctrl', 'S'] → 'Ctrl+S'
 */
export function formatShortcut(keys: string[]): string {
  return keys.join('+');
}

/**
 * Format shortcut keys with display-friendly symbols
 * @example ['Ctrl', 'S'] → '⌘S' (on Mac) or 'Ctrl+S'
 */
export function formatShortcutDisplay(keys: string[]): string {
  const isMac = navigator.platform.toLowerCase().includes('mac');
  
  return keys.map(key => {
    switch (key.toLowerCase()) {
      case 'ctrl':
        return isMac ? '⌘' : 'Ctrl';
      case 'alt':
        return isMac ? '⌥' : 'Alt';
      case 'shift':
        return isMac ? '⇧' : 'Shift';
      case 'meta':
        return '⌘';
      case 'escape':
        return 'Esc';
      default:
        return key;
    }
  }).join(isMac ? '' : '+');
}

/**
 * Get individual key badges for display
 * @example ['Ctrl', 'S'] → ['Ctrl', 'S']
 */
export function getShortcutBadges(keys: string[]): string[] {
  return keys.map(key => {
    switch (key.toLowerCase()) {
      case 'ctrl':
        return 'Ctrl';
      case 'alt':
        return 'Alt';
      case 'shift':
        return 'Shift';
      case 'meta':
        return '⌘';
      case 'escape':
        return 'Esc';
      default:
        return key;
    }
  });
}

/**
 * Check if a keyboard event matches a shortcut
 */
export function matchesShortcut(event: KeyboardEvent, shortcut: Shortcut): boolean {
  const keys = shortcut.keys.map(k => k.toLowerCase());
  
  // Check modifier keys
  if (keys.includes('ctrl') !== event.ctrlKey) return false;
  if (keys.includes('alt') !== event.altKey) return false;
  if (keys.includes('shift') !== event.shiftKey) return false;
  if (keys.includes('meta') !== event.metaKey) return false;
  
  // Check main key
  const mainKey = keys.find(k => !['ctrl', 'alt', 'shift', 'meta'].includes(k));
  if (mainKey && event.key.toLowerCase() !== mainKey) return false;
  
  return true;
}

/**
 * Find a shortcut by its ID
 */
export function getShortcutById(id: string): Shortcut | undefined {
  return SHORTCUTS.find(s => s.id === id);
}

/**
 * Get all shortcuts for a specific category
 */
export function getShortcutsByCategory(category: ShortcutCategory): Shortcut[] {
  return SHORTCUTS.filter(s => s.category === category);
}

/**
 * Get all unique categories
 */
export function getCategories(): ShortcutCategory[] {
  return Object.values(ShortcutCategory);
}

/**
 * Search shortcuts by query string
 */
export function searchShortcuts(query: string): Shortcut[] {
  const lowerQuery = query.toLowerCase();
  return SHORTCUTS.filter(s => 
    s.description.toLowerCase().includes(lowerQuery) ||
    s.keys.some(k => k.toLowerCase().includes(lowerQuery)) ||
    s.category.toLowerCase().includes(lowerQuery)
  );
}

/**
 * Export shortcuts as JSON
 */
export function exportShortcutsAsJson(): string {
  return JSON.stringify({
    version: '1.0',
    generatedAt: new Date().toISOString(),
    shortcuts: SHORTCUTS,
  }, null, 2);
}

/**
 * Group shortcuts by category
 */
export function groupShortcutsByCategory(shortcuts: Shortcut[] = SHORTCUTS): Record<ShortcutCategory, Shortcut[]> {
  return shortcuts.reduce((acc, shortcut) => {
    if (!acc[shortcut.category]) {
      acc[shortcut.category] = [];
    }
    acc[shortcut.category].push(shortcut);
    return acc;
  }, {} as Record<ShortcutCategory, Shortcut[]>);
}
