/**
 * Theme System Configuration
 * 
 * Defines theme types, color palettes, and CSS variable mappings
 * for the Super Punch-Out!! Editor's dark/light mode support.
 */

export type Theme = 'dark' | 'light' | 'system';

export interface ThemeColors {
  // Backgrounds
  bgPrimary: string;
  bgSecondary: string;
  bgTertiary: string;
  bgPanel: string;
  
  // Text
  textPrimary: string;
  textSecondary: string;
  textMuted: string;
  textInverse: string;
  
  // Accents
  accent: string;
  accentHover: string;
  accentMuted: string;
  
  // Semantic
  success: string;
  warning: string;
  error: string;
  info: string;
  
  // Borders
  border: string;
  borderHover: string;
  
  // Special
  grid: string;
  canvasBg: string;
}

/**
 * Dark theme color palette (default)
 */
export const darkTheme: ThemeColors = {
  bgPrimary: '#0f172a',
  bgSecondary: '#1e293b',
  bgTertiary: '#334155',
  bgPanel: '#1e293b',
  textPrimary: '#f8fafc',
  textSecondary: '#cbd5e1',
  textMuted: '#64748b',
  textInverse: '#0f172a',
  accent: '#e74c3c',
  accentHover: '#c0392b',
  accentMuted: '#1e3a5f',
  success: '#22c55e',
  warning: '#f59e0b',
  error: '#ef4444',
  info: '#3b82f6',
  border: '#334155',
  borderHover: '#475569',
  grid: '#1e293b',
  canvasBg: '#0f172a',
};

/**
 * Light theme color palette
 */
export const lightTheme: ThemeColors = {
  bgPrimary: '#ffffff',
  bgSecondary: '#f1f5f9',
  bgTertiary: '#e2e8f0',
  bgPanel: '#f8fafc',
  textPrimary: '#0f172a',
  textSecondary: '#475569',
  textMuted: '#94a3b8',
  textInverse: '#ffffff',
  accent: '#dc2626',
  accentHover: '#b91c1c',
  accentMuted: '#fee2e2',
  success: '#16a34a',
  warning: '#d97706',
  error: '#dc2626',
  info: '#2563eb',
  border: '#e2e8f0',
  borderHover: '#cbd5e1',
  grid: '#f1f5f9',
  canvasBg: '#ffffff',
};

/**
 * CSS custom property mapping
 * Maps ThemeColors keys to CSS variable names
 */
export const cssVariableMap: Record<keyof ThemeColors, string> = {
  bgPrimary: '--bg-primary',
  bgSecondary: '--bg-secondary',
  bgTertiary: '--bg-tertiary',
  bgPanel: '--bg-panel',
  textPrimary: '--text-primary',
  textSecondary: '--text-secondary',
  textMuted: '--text-muted',
  textInverse: '--text-inverse',
  accent: '--accent',
  accentHover: '--accent-hover',
  accentMuted: '--accent-muted',
  success: '--success',
  warning: '--warning',
  error: '--error',
  info: '--info',
  border: '--border',
  borderHover: '--border-hover',
  grid: '--grid',
  canvasBg: '--canvas-bg',
};

/**
 * Legacy variable mappings for backward compatibility
 * Maps old variable names to new theme system
 */
export const legacyVariableMap: Record<string, keyof ThemeColors> = {
  '--primary-bg': 'bgPrimary',
  '--secondary-bg': 'bgSecondary',
  '--panel-bg': 'bgPanel',
  '--text-main': 'textPrimary',
  '--text-dim': 'textMuted',
  '--text': 'textPrimary',
  '--glass': 'bgTertiary',
  '--blue': 'info',
  '--blue-hover': 'info',
};

/**
 * Apply theme colors to CSS custom properties
 */
export function applyThemeToCSS(colors: ThemeColors): void {
  const root = document.documentElement;
  
  (Object.keys(cssVariableMap) as Array<keyof ThemeColors>).forEach((key) => {
    root.style.setProperty(cssVariableMap[key], colors[key]);
  });
  
  // Apply legacy mappings for backward compatibility
  root.style.setProperty('--primary-bg', colors.bgPrimary);
  root.style.setProperty('--secondary-bg', colors.bgSecondary);
  root.style.setProperty('--panel-bg', colors.bgPanel);
  root.style.setProperty('--text-main', colors.textPrimary);
  root.style.setProperty('--text-dim', colors.textMuted);
  root.style.setProperty('--text', colors.textPrimary);
  root.style.setProperty('--glass', `${colors.bgTertiary}80`); // 50% opacity
  root.style.setProperty('--blue', colors.info);
  root.style.setProperty('--blue-hover', colors.info);
}

/**
 * Get effective theme based on system preference
 */
export function getEffectiveTheme(theme: Theme): 'dark' | 'light' {
  if (theme === 'system') {
    return window.matchMedia('(prefers-color-scheme: dark)').matches 
      ? 'dark' 
      : 'light';
  }
  return theme;
}

/**
 * Get theme colors for a specific theme
 */
export function getThemeColors(theme: Theme): ThemeColors {
  const effectiveTheme = getEffectiveTheme(theme);
  return effectiveTheme === 'dark' ? darkTheme : lightTheme;
}

/**
 * Storage key for persisting theme preference
 */
export const THEME_STORAGE_KEY = 'spo-editor-theme';

/**
 * Default theme
 */
export const DEFAULT_THEME: Theme = 'dark';
