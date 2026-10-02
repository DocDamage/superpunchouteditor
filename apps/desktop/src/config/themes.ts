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
  bgPrimary: '#14131c',
  bgSecondary: '#1b1a26',
  bgTertiary: '#2c2a3c',
  bgPanel: '#22202f',
  textPrimary: '#f7f5fc',
  textSecondary: '#cbc7da',
  textMuted: '#9a95ae',
  textInverse: '#14131c',
  accent: '#e5121d',
  accentHover: '#ff3340',
  accentMuted: '#45141b',
  success: '#3ddc84',
  warning: '#ffc53d',
  error: '#ff6b74',
  info: '#6ea8ff',
  border: '#37344b',
  borderHover: '#575270',
  grid: '#26243a',
  canvasBg: '#0d0c13',
};

/**
 * Light theme color palette
 */
export const lightTheme: ThemeColors = {
  bgPrimary: '#f2f0f6',
  bgSecondary: '#e8e5ef',
  bgTertiary: '#dcd8e6',
  bgPanel: '#ffffff',
  textPrimary: '#1c1a26',
  textSecondary: '#474357',
  textMuted: '#6a657c',
  textInverse: '#ffffff',
  accent: '#e5121d',
  accentHover: '#c90c16',
  accentMuted: '#fde0e2',
  success: '#0b8043',
  warning: '#a35f00',
  error: '#c8101b',
  info: '#2152c9',
  border: '#cfcadb',
  borderHover: '#a8a1bc',
  grid: '#dcd8e6',
  canvasBg: '#f8f7fb',
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
 * Select the active theme.
 *
 * Colour tokens are owned by `styles/tokens.css`; this only chooses which set
 * applies and clears inline overrides written by older builds.
 */
export function applyThemeToCSS(effectiveTheme: 'dark' | 'light'): void {
  const root = document.documentElement;
  root.dataset.theme = effectiveTheme;

  (Object.keys(cssVariableMap) as Array<keyof ThemeColors>).forEach((key) => {
    root.style.removeProperty(cssVariableMap[key]);
  });
  Object.keys(legacyVariableMap).forEach((name) => root.style.removeProperty(name));
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

/**
 * Pick one readable accent colour from the selected boxer's palette.
 *
 * The palette is read from the user's own ROM at runtime. Only this single
 * accent is taken from it; the rest of the interface keeps its own colours so
 * text stays readable whichever boxer is selected.
 */
export function deriveBoxerAccent(skin: RuntimeSkin): string {
  const { accent } = deriveSkinPalette(skin.palette);
  return rgbToHex(accent);
}

export function applyRuntimeSkinToCSS(skin: RuntimeSkin | null): void {
  const root = document.documentElement;

  if (!skin) {
    root.style.removeProperty('--boxer-accent');
    root.style.removeProperty('--auth-icon-image');
    root.style.removeProperty('--auth-portrait-image');
    return;
  }

  root.style.setProperty('--boxer-accent', deriveBoxerAccent(skin));
  root.style.setProperty('--auth-icon-image', skin.iconDataUrl ? `url("${skin.iconDataUrl}")` : 'none');
  root.style.setProperty(
    '--auth-portrait-image',
    skin.portraitDataUrl ? `url("${skin.portraitDataUrl}")` : 'none'
  );
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
