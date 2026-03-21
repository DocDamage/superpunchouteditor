/**
 * Theme Provider
 * 
 * Manages application-wide theme state, including:
 * - Theme selection (dark/light/system)
 * - CSS variable application
 * - System preference detection
 * - Theme persistence via Tauri settings
 */

import React, { 
  createContext, 
  useContext, 
  useState, 
  useEffect, 
  useCallback,
  useMemo,
  type ReactNode 
} from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  type Theme,
  type ThemeColors,
  type RuntimeSkin,
  darkTheme,
  lightTheme,
  applyThemeToCSS,
  applyRuntimeSkinToCSS,
  deriveRuntimeThemeColors,
  getEffectiveTheme,
  getThemeColors,
  THEME_STORAGE_KEY,
  DEFAULT_THEME,
} from '../config/themes';

interface ThemeContextType {
  /** Current theme setting ('dark', 'light', or 'system') */
  theme: Theme;
  /** Currently active theme colors */
  colors: ThemeColors;
  /** Set theme to a specific value */
  setTheme: (theme: Theme) => void;
  /** Toggle between dark and light (skips system) */
  toggleTheme: () => void;
  setRuntimeSkin: (skin: RuntimeSkin | null) => void;
  runtimeSkin: RuntimeSkin | null;
  /** Whether the effective theme is dark */
  isDark: boolean;
  /** Whether the theme has been loaded from storage */
  isLoaded: boolean;
}

const ThemeContext = createContext<ThemeContextType | undefined>(undefined);

interface ThemeProviderProps {
  children: ReactNode;
}

/**
 * Settings data structure for theme persistence
 */
interface ThemeSettings {
  theme: Theme;
}

/**
 * Theme Provider Component
 * 
 * Wraps the application and provides theme state management.
 * Handles system preference detection and theme persistence.
 */
export function ThemeProvider({ children }: ThemeProviderProps): React.ReactElement {
  const [theme, setThemeState] = useState<Theme>(DEFAULT_THEME);
  const [runtimeSkin, setRuntimeSkin] = useState<RuntimeSkin | null>(null);
  const [isLoaded, setIsLoaded] = useState(false);

  // Get effective theme (resolve 'system' to actual theme)
  const effectiveTheme = useMemo(() => getEffectiveTheme(theme), [theme]);
  
  // Get current theme colors
  const colors = useMemo(() => {
    const baseColors = getThemeColors(theme);
    return runtimeSkin ? deriveRuntimeThemeColors(baseColors, runtimeSkin) : baseColors;
  }, [theme, runtimeSkin]);
  
  // Check if currently in dark mode
  const isDark = effectiveTheme === 'dark';

  /**
   * Apply theme to document
   */
  const applyTheme = useCallback((newTheme: Theme, skin: RuntimeSkin | null) => {
    const effective = getEffectiveTheme(newTheme);
    const baseThemeColors = effective === 'dark' ? darkTheme : lightTheme;
    const themeColors = skin
      ? deriveRuntimeThemeColors(baseThemeColors, skin)
      : baseThemeColors;
    
    // Apply CSS variables
    applyThemeToCSS(themeColors);
    applyRuntimeSkinToCSS(skin);
    
    // Update body class for global styling
    document.body.classList.remove('theme-dark', 'theme-light');
    document.body.classList.add(`theme-${effective}`);
    
    // Update meta theme-color for mobile browsers
    const metaThemeColor = document.querySelector('meta[name="theme-color"]');
    if (metaThemeColor) {
      metaThemeColor.setAttribute('content', themeColors.bgPrimary);
    }
  }, []);

  /**
   * Save theme to settings via Tauri
   */
  const saveTheme = useCallback(async (newTheme: Theme) => {
    try {
      await invoke('save_theme_settings', { theme: newTheme });
    } catch (error) {
      // Fallback to localStorage if Tauri is not available
      try {
        localStorage.setItem(THEME_STORAGE_KEY, newTheme);
      } catch {
        // Ignore localStorage errors
      }
    }
  }, []);

  /**
   * Load theme from settings via Tauri
   */
  const loadTheme = useCallback(async (): Promise<Theme> => {
    try {
      const settings = await invoke<ThemeSettings | null>('load_theme_settings');
      if (settings && settings.theme) {
        return settings.theme;
      }
    } catch (error) {
      // Fallback to localStorage
      try {
        const stored = localStorage.getItem(THEME_STORAGE_KEY);
        if (stored && ['dark', 'light', 'system'].includes(stored)) {
          return stored as Theme;
        }
      } catch {
        // Ignore localStorage errors
      }
    }
    return DEFAULT_THEME;
  }, []);

  /**
   * Set theme with persistence
   */
  const setTheme = useCallback((newTheme: Theme) => {
    setThemeState(newTheme);
    saveTheme(newTheme);
  }, [saveTheme]);

  /**
   * Toggle between dark and light themes
   * Cycles: dark -> light -> dark
   */
  const toggleTheme = useCallback(() => {
    const newTheme = effectiveTheme === 'dark' ? 'light' : 'dark';
    setTheme(newTheme);
  }, [effectiveTheme, setTheme]);

  // Load saved theme on mount
  useEffect(() => {
    let isMounted = true;
    
    loadTheme().then((savedTheme) => {
      if (isMounted) {
        setThemeState(savedTheme);
        setIsLoaded(true);
      }
    });

    return () => {
      isMounted = false;
    };
  }, [loadTheme]);

  useEffect(() => {
    applyTheme(theme, runtimeSkin);
  }, [theme, runtimeSkin, applyTheme]);

  // Listen for system theme changes
  useEffect(() => {
    if (theme !== 'system') return;

    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    
    const handleChange = (e: MediaQueryListEvent) => {
      const newEffectiveTheme = e.matches ? 'dark' : 'light';
      const baseThemeColors = newEffectiveTheme === 'dark' ? darkTheme : lightTheme;
      const themeColors = runtimeSkin
        ? deriveRuntimeThemeColors(baseThemeColors, runtimeSkin)
        : baseThemeColors;
      applyThemeToCSS(themeColors);
      applyRuntimeSkinToCSS(runtimeSkin);
      document.body.classList.remove('theme-dark', 'theme-light');
      document.body.classList.add(`theme-${newEffectiveTheme}`);
    };

    // Modern browsers
    mediaQuery.addEventListener('change', handleChange);
    
    return () => {
      mediaQuery.removeEventListener('change', handleChange);
    };
  }, [theme, runtimeSkin]);

  const contextValue: ThemeContextType = {
    theme,
    colors,
    setTheme,
    toggleTheme,
    setRuntimeSkin,
    runtimeSkin,
    isDark,
    isLoaded,
  };

  return (
    <ThemeContext.Provider value={contextValue}>
      {children}
    </ThemeContext.Provider>
  );
}

/**
 * Hook to access theme context
 * 
 * @throws Error if used outside of ThemeProvider
 */
export function useTheme(): ThemeContextType {
  const context = useContext(ThemeContext);
  if (context === undefined) {
    throw new Error('useTheme must be used within a ThemeProvider');
  }
  return context;
}

/**
 * Higher-order component for class components that need theme access
 */
export function withTheme<T extends object>(
  Component: React.ComponentType<T & { theme: ThemeContextType }>
): React.FC<T> {
  return function WithThemeComponent(props: T) {
    const theme = useTheme();
    return <Component {...props} theme={theme} />;
  };
}
