/**
 * Settings Configuration for Super Punch-Out!! Editor
 * 
 * Defines the structure for application settings and provides
 * import/export functionality for backing up and sharing configurations.
 */

import { ExternalTool } from '../store/useStore';

/**
 * Panel layout type for the editor interface
 */
export type PanelLayout = 'default' | 'compact' | 'wide' | 'custom';

/**
 * Main application settings structure
 */
export interface AppSettings {
  /** Settings file version */
  version: string;
  
  // Appearance
  /** UI theme preference */
  theme: 'dark' | 'light' | 'system';
  /** UI scale factor */
  uiScale: number;
  /** Whether sidebar is collapsed */
  sidebarCollapsed: boolean;
  
  // Editor
  /** Default export format for assets */
  defaultExportFormat: 'png' | 'bmp';
  /** Auto-save interval in minutes (0 = disabled) */
  autoSaveInterval: number;
  /** Show confirmation dialog on close with unsaved changes */
  confirmOnClose: boolean;
  /** Show tooltips throughout the UI */
  showTooltips: boolean;
  
  // Emulator
  /** Path to emulator executable */
  emulatorPath?: string;
  /** Type of emulator */
  emulatorType?: 'snes9x' | 'bsnes' | 'mesen-s' | 'other';
  /** Auto-save ROM before launching emulator */
  autoSaveBeforeLaunch: boolean;
  
  // Paths
  /** Default directory for exports */
  defaultExportDirectory?: string;
  /** Recently opened projects */
  recentProjects: string[];
  /** Recently opened ROM files */
  recentRoms: string[];
  
  // External Tools
  /** Configured external tools */
  externalTools: ExternalTool[];
  /** Default tool IDs by file extension */
  defaultToolIds: Record<string, string>;
  
  // Shortcuts
  /** Custom keyboard shortcuts */
  customShortcuts?: Record<string, string[]>;
  
  // Layout
  /** Panel layout preference */
  panelLayout: PanelLayout;
}

/**
 * External tool definition (re-exported from store)
 */
export type { ExternalTool };

/**
 * Default settings values
 */
export const DEFAULT_SETTINGS: AppSettings = {
  version: '2.0',
  theme: 'system',
  uiScale: 1.0,
  sidebarCollapsed: false,
  defaultExportFormat: 'png',
  autoSaveInterval: 5,
  confirmOnClose: true,
  showTooltips: true,
  autoSaveBeforeLaunch: true,
  recentProjects: [],
  recentRoms: [],
  externalTools: [],
  defaultToolIds: {},
  panelLayout: 'default',
};

/**
 * Settings export file structure
 */
export interface SettingsExport {
  /** Export format version */
  version: string;
  /** Export timestamp (ISO 8601) */
  exported_at: string;
  /** Application name */
  app: string;
  /** Settings data */
  settings: Partial<AppSettings>;
}

/**
 * Import report showing what settings were changed
 */
export interface ImportReport {
  /** Whether the import was successful */
  success: boolean;
  /** List of settings that were imported */
  imported: string[];
  /** List of settings that were merged */
  merged: string[];
  /** List of settings that were skipped */
  skipped: string[];
  /** Any errors encountered */
  errors: string[];
  /** Warnings for the user */
  warnings: string[];
}

/**
 * Validation result for imported settings
 */
export interface SettingsValidation {
  /** Whether the settings file is valid */
  valid: boolean;
  /** Version compatibility status */
  versionCompatible: boolean;
  /** List of validation errors */
  errors: string[];
  /** List of validation warnings */
  warnings: string[];
  /** Parsed settings (if valid) */
  settings?: Partial<AppSettings>;
}

/**
 * Settings change preview for import dialog
 */
export interface SettingsChangePreview {
  /** Category of the setting */
  category: string;
  /** Setting key */
  key: string;
  /** Human-readable name */
  displayName: string;
  /** Current value */
  currentValue: unknown;
  /** New value from import */
  newValue: unknown;
  /** Whether this will change the current value */
  willChange: boolean;
  /** Whether there's a conflict (both values exist and differ) */
  hasConflict: boolean;
}

/**
 * Category labels for settings display
 */
export const SETTINGS_CATEGORIES: Record<string, string> = {
  appearance: 'Appearance',
  editor: 'Editor',
  emulator: 'Emulator',
  paths: 'Paths',
  externalTools: 'External Tools',
  shortcuts: 'Keyboard Shortcuts',
  layout: 'Layout',
};

/**
 * Human-readable names for settings keys
 */
export const SETTINGS_DISPLAY_NAMES: Record<string, string> = {
  theme: 'UI Theme',
  uiScale: 'UI Scale',
  sidebarCollapsed: 'Sidebar Collapsed',
  defaultExportFormat: 'Default Export Format',
  autoSaveInterval: 'Auto-Save Interval',
  confirmOnClose: 'Confirm on Close',
  showTooltips: 'Show Tooltips',
  emulatorPath: 'Emulator Path',
  emulatorType: 'Emulator Type',
  autoSaveBeforeLaunch: 'Auto-Save Before Launch',
  defaultExportDirectory: 'Default Export Directory',
  recentProjects: 'Recent Projects',
  recentRoms: 'Recent ROMs',
  externalTools: 'External Tools',
  defaultToolIds: 'Default Tool Assignments',
  customShortcuts: 'Custom Shortcuts',
  panelLayout: 'Panel Layout',
};

/**
 * Validate imported settings file
 */
export function validateSettingsImport(data: unknown): SettingsValidation {
  const errors: string[] = [];
  const warnings: string[] = [];
  
  // Check if data is an object
  if (!data || typeof data !== 'object') {
    return {
      valid: false,
      versionCompatible: false,
      errors: ['Invalid settings file: must be a JSON object'],
      warnings: [],
    };
  }
  
  const exportData = data as SettingsExport;
  
  // Check version
  if (!exportData.version) {
    errors.push('Missing version field');
  } else {
    const majorVersion = exportData.version.split('.')[0];
    const currentMajor = DEFAULT_SETTINGS.version.split('.')[0];
    if (majorVersion !== currentMajor) {
      warnings.push(`Version mismatch: file is v${exportData.version}, expected v${DEFAULT_SETTINGS.version}`);
    }
  }
  
  // Check settings object
  if (!exportData.settings || typeof exportData.settings !== 'object') {
    errors.push('Missing or invalid settings object');
  }
  
  // Check exported_at
  if (!exportData.exported_at) {
    warnings.push('Missing export timestamp');
  }
  
  const valid = errors.length === 0;
  const versionCompatible = !warnings.some(w => w.includes('Version mismatch'));
  
  return {
    valid,
    versionCompatible,
    errors,
    warnings,
    settings: valid ? exportData.settings : undefined,
  };
}

/**
 * Create a settings export object
 */
export function createSettingsExport(settings: Partial<AppSettings>): SettingsExport {
  return {
    version: DEFAULT_SETTINGS.version,
    exported_at: new Date().toISOString(),
    app: 'Super Punch-Out!! Editor',
    settings,
  };
}

/**
 * Generate a preview of settings changes for import
 */
export function generateImportPreview(
  currentSettings: Partial<AppSettings>,
  importedSettings: Partial<AppSettings>
): SettingsChangePreview[] {
  const preview: SettingsChangePreview[] = [];
  
  // Helper to categorize settings
  const getCategory = (key: string): string => {
    if (['theme', 'uiScale', 'sidebarCollapsed'].includes(key)) return 'appearance';
    if (['defaultExportFormat', 'autoSaveInterval', 'confirmOnClose', 'showTooltips'].includes(key)) return 'editor';
    if (['emulatorPath', 'emulatorType', 'autoSaveBeforeLaunch'].includes(key)) return 'emulator';
    if (['defaultExportDirectory', 'recentProjects', 'recentRoms'].includes(key)) return 'paths';
    if (['externalTools', 'defaultToolIds'].includes(key)) return 'externalTools';
    if (['customShortcuts'].includes(key)) return 'shortcuts';
    if (['panelLayout'].includes(key)) return 'layout';
    return 'other';
  };
  
  // Process all imported settings
  for (const [key, value] of Object.entries(importedSettings)) {
    if (value === undefined) continue;
    
    const currentValue = (currentSettings as Record<string, unknown>)[key];
    const hasConflict = currentValue !== undefined && 
                        JSON.stringify(currentValue) !== JSON.stringify(value);
    
    preview.push({
      category: getCategory(key),
      key,
      displayName: SETTINGS_DISPLAY_NAMES[key] || key,
      currentValue,
      newValue: value,
      willChange: hasConflict || currentValue === undefined,
      hasConflict,
    });
  }
  
  // Sort by category then by key
  return preview.sort((a, b) => {
    if (a.category !== b.category) {
      return a.category.localeCompare(b.category);
    }
    return a.key.localeCompare(b.key);
  });
}

/**
 * Merge imported settings with current settings
 */
export function mergeSettings(
  current: Partial<AppSettings>,
  imported: Partial<AppSettings>,
  mergeArrays = true
): AppSettings {
  const merged = { ...DEFAULT_SETTINGS, ...current };
  
  for (const [key, value] of Object.entries(imported)) {
    if (value === undefined) continue;
    
    // Handle array merging
    if (Array.isArray(value) && mergeArrays) {
      const currentArray = ((merged as Record<string, unknown>)[key] as unknown[]) || [];
      // For recent projects/ROMs, combine and deduplicate
      if (key === 'recentProjects' || key === 'recentRoms') {
        const combined = [...currentArray, ...value];
        const seen = new Set();
        (merged as Record<string, unknown>)[key] = combined.filter(item => {
          const str = JSON.stringify(item);
          if (seen.has(str)) return false;
          seen.add(str);
          return true;
        }).slice(0, 10); // Keep only last 10
      } else {
        // For other arrays, replace unless merge is requested
        (merged as Record<string, unknown>)[key] = value;
      }
    } else if (typeof value === 'object' && value !== null && !Array.isArray(value)) {
      // Merge nested objects (like defaultToolIds)
      const currentObj = ((merged as Record<string, unknown>)[key] as Record<string, unknown>) || {};
      (merged as Record<string, unknown>)[key] = { ...currentObj, ...value };
    } else {
      // Simple value replacement
      (merged as Record<string, unknown>)[key] = value;
    }
  }
  
  return merged as AppSettings;
}

/**
 * Filter sensitive or machine-specific paths from settings
 */
export function sanitizeSettingsForExport(settings: Partial<AppSettings>): Partial<AppSettings> {
  const sanitized = { ...settings };
  
  // Remove recent paths that might be machine-specific
  // Keep them in for now, user can choose to exclude them
  
  return sanitized;
}

/**
 * Export settings to JSON string
 */
export function exportSettingsToJson(settings: Partial<AppSettings>, pretty = true): string {
  const exportData = createSettingsExport(settings);
  return JSON.stringify(exportData, null, pretty ? 2 : 0);
}

/**
 * Parse settings from JSON string
 */
export function parseSettingsFromJson(json: string): SettingsValidation {
  try {
    const data = JSON.parse(json);
    return validateSettingsImport(data);
  } catch (e) {
    return {
      valid: false,
      versionCompatible: false,
      errors: [`Failed to parse JSON: ${e}`],
      warnings: [],
    };
  }
}
