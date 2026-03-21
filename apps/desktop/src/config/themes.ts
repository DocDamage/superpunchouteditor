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

export interface RuntimeSkin {
  boxerKey: string;
  boxerName: string;
  palette: Array<{ r: number; g: number; b: number }>;
  iconDataUrl?: string | null;
  portraitDataUrl?: string | null;
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

function clamp(value: number, min: number, max: number): number {
  return Math.max(min, Math.min(max, value));
}

function toHex(value: number): string {
  return clamp(Math.round(value), 0, 255).toString(16).padStart(2, '0');
}

function rgbToHex(color: { r: number; g: number; b: number }): string {
  return `#${toHex(color.r)}${toHex(color.g)}${toHex(color.b)}`;
}

function mixColor(
  a: { r: number; g: number; b: number },
  b: { r: number; g: number; b: number },
  t: number
): { r: number; g: number; b: number } {
  return {
    r: a.r + (b.r - a.r) * t,
    g: a.g + (b.g - a.g) * t,
    b: a.b + (b.b - a.b) * t,
  };
}

function luminance(color: { r: number; g: number; b: number }): number {
  return color.r * 0.2126 + color.g * 0.7152 + color.b * 0.0722;
}

function saturation(color: { r: number; g: number; b: number }): number {
  const max = Math.max(color.r, color.g, color.b);
  const min = Math.min(color.r, color.g, color.b);
  return max - min;
}

function deriveSkinPalette(palette: RuntimeSkin['palette']) {
  const unique = palette.filter(
    (color, index, list) =>
      list.findIndex(
        (candidate) =>
          candidate.r === color.r && candidate.g === color.g && candidate.b === color.b
      ) === index
  );

  const colors = unique.length > 0 ? unique : [{ r: 231, g: 76, b: 60 }];
  const byLuma = [...colors].sort((a, b) => luminance(a) - luminance(b));
  const darkest = byLuma[0];
  const dark = byLuma[Math.min(2, byLuma.length - 1)];
  const brightest = byLuma[byLuma.length - 1];
  const saturated = [...colors].sort((a, b) => saturation(b) - saturation(a));
  const accent =
    saturated.find((color) => luminance(color) > 72) ??
    saturated[0] ??
    brightest;
  const cool =
    saturated.find((color) => color.b >= color.r && color.b >= color.g && luminance(color) > 48) ??
    saturated.find((color) => color.g >= color.r && luminance(color) > 48) ??
    accent;

  return { darkest, dark, brightest, accent, cool };
}

export function deriveRuntimeThemeColors(base: ThemeColors, skin: RuntimeSkin): ThemeColors {
  const { darkest, dark, brightest, accent, cool } = deriveSkinPalette(skin.palette);
  const bgPrimary = rgbToHex(mixColor(darkest, accent, 0.08));
  const bgSecondary = rgbToHex(mixColor(dark, accent, 0.14));
  const bgTertiary = rgbToHex(mixColor(dark, brightest, 0.16));
  const bgPanel = rgbToHex(mixColor(dark, accent, 0.2));
  const border = rgbToHex(mixColor(accent, dark, 0.52));
  const borderHover = rgbToHex(mixColor(accent, brightest, 0.28));
  const accentHex = rgbToHex(accent);
  const accentHover = rgbToHex(mixColor(accent, brightest, 0.18));
  const accentMuted = rgbToHex(mixColor(accent, dark, 0.72));
  const info = rgbToHex(cool);
  const textPrimary = rgbToHex(mixColor(brightest, { r: 255, g: 248, b: 220 }, 0.16));
  const textSecondary = rgbToHex(mixColor(brightest, dark, 0.34));
  const textMuted = rgbToHex(mixColor(brightest, dark, 0.58));

  return {
    ...base,
    bgPrimary,
    bgSecondary,
    bgTertiary,
    bgPanel,
    textPrimary,
    textSecondary,
    textMuted,
    accent: accentHex,
    accentHover,
    accentMuted,
    success: base.success,
    warning: rgbToHex(mixColor(accent, { r: 255, g: 212, b: 92 }, 0.45)),
    error: rgbToHex(mixColor(accent, { r: 255, g: 106, b: 106 }, 0.3)),
    info,
    border,
    borderHover,
    grid: rgbToHex(mixColor(dark, accent, 0.12)),
    canvasBg: bgPrimary,
  };
}

export function applyRuntimeSkinToCSS(skin: RuntimeSkin | null): void {
  const root = document.documentElement;

  if (!skin) {
    root.style.removeProperty('--auth-icon-image');
    root.style.removeProperty('--auth-portrait-image');
    root.style.removeProperty('--auth-boxer-name');
    document.body.classList.remove('theme-authentic');
    return;
  }

  root.style.setProperty('--auth-icon-image', skin.iconDataUrl ? `url("${skin.iconDataUrl}")` : 'none');
  root.style.setProperty(
    '--auth-portrait-image',
    skin.portraitDataUrl ? `url("${skin.portraitDataUrl}")` : 'none'
  );
  root.style.setProperty('--auth-boxer-name', `"${skin.boxerName}"`);
  document.body.classList.add('theme-authentic');
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
