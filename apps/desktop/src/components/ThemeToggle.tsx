/**
 * Theme Toggle Component
 * 
 * Provides UI for switching between dark, light, and system themes.
 * Includes a dropdown menu and an optional quick toggle button.
 */

import React, { useState, useRef, useEffect } from 'react';
import { useTheme } from '../context/ThemeProvider';
import type { Theme } from '../config/themes';

export interface ThemeToggleProps {
  /** Display style variant */
  variant?: 'button' | 'dropdown' | 'minimal';
  /** Custom class name */
  className?: string;
  /** Whether to show text labels */
  showLabels?: boolean;
  /** Size of the toggle */
  size?: 'small' | 'medium' | 'large';
}

interface ThemeOption {
  value: Theme;
  label: string;
  icon: string;
  description: string;
}

const themeOptions: ThemeOption[] = [
  {
    value: 'dark',
    label: 'Dark',
    icon: '🌙',
    description: 'Dark color scheme',
  },
  {
    value: 'light',
    label: 'Light',
    icon: '☀️',
    description: 'Light color scheme',
  },
  {
    value: 'system',
    label: 'System',
    icon: '💻',
    description: 'Follow system preference',
  },
];

/**
 * Get size-based styles
 */
function getSizeStyles(size: 'small' | 'medium' | 'large') {
  const sizes = {
    small: {
      padding: '0.375rem',
      fontSize: '0.875rem',
      iconSize: '1rem',
      gap: '0.25rem',
    },
    medium: {
      padding: '0.5rem',
      fontSize: '1rem',
      iconSize: '1.25rem',
      gap: '0.5rem',
    },
    large: {
      padding: '0.75rem',
      fontSize: '1.125rem',
      iconSize: '1.5rem',
      gap: '0.75rem',
    },
  };
  return sizes[size];
}

/**
 * Theme Toggle Component
 */
export function ThemeToggle({
  variant = 'dropdown',
  className = '',
  showLabels = true,
  size = 'medium',
}: ThemeToggleProps): React.ReactElement {
  const { theme, setTheme, toggleTheme, isDark, isLoaded } = useTheme();
  const [isOpen, setIsOpen] = useState(false);
  const dropdownRef = useRef<HTMLDivElement>(null);
  const sizeStyles = getSizeStyles(size);

  // Close dropdown when clicking outside
  useEffect(() => {
    function handleClickOutside(event: MouseEvent) {
      if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
        setIsOpen(false);
      }
    }

    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  // Handle keyboard shortcuts
  useEffect(() => {
    function handleKeyDown(event: KeyboardEvent) {
      // Ctrl/Cmd + Shift + L to toggle theme
      if ((event.ctrlKey || event.metaKey) && event.shiftKey && event.key === 'L') {
        event.preventDefault();
        toggleTheme();
      }
    }

    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, [toggleTheme]);

  const currentOption = themeOptions.find((opt) => opt.value === theme) || themeOptions[0];

  // Minimal variant - just an icon button
  if (variant === 'minimal') {
    return (
      <button
        onClick={toggleTheme}
        className={`theme-toggle-minimal ${className}`}
        title={`Toggle theme (currently: ${currentOption.label})`}
        aria-label={`Toggle theme, currently ${currentOption.label}`}
        disabled={!isLoaded}
        style={{
          padding: sizeStyles.padding,
          fontSize: sizeStyles.iconSize,
          background: 'transparent',
          border: 'none',
          borderRadius: '0.375rem',
          cursor: 'pointer',
          opacity: isLoaded ? 1 : 0.5,
          transition: 'all 0.2s ease',
        }}
      >
        {isDark ? '🌙' : '☀️'}
      </button>
    );
  }

  // Button variant - quick toggle with current state display
  if (variant === 'button') {
    return (
      <button
        onClick={toggleTheme}
        className={`theme-toggle-button ${className}`}
        title={`Toggle theme (Ctrl+Shift+L)`}
        aria-label={`Toggle theme, currently ${currentOption.label}`}
        disabled={!isLoaded}
        style={{
          display: 'flex',
          alignItems: 'center',
          gap: sizeStyles.gap,
          padding: sizeStyles.padding,
          fontSize: sizeStyles.fontSize,
          backgroundColor: 'var(--bg-tertiary)',
          border: '1px solid var(--border)',
          borderRadius: '0.5rem',
          color: 'var(--text-primary)',
          cursor: 'pointer',
          opacity: isLoaded ? 1 : 0.5,
          transition: 'all 0.2s ease',
        }}
      >
        <span role="img" aria-hidden="true">{currentOption.icon}</span>
        {showLabels && <span>{currentOption.label}</span>}
      </button>
    );
  }

  // Dropdown variant - full theme selection
  return (
    <div
      ref={dropdownRef}
      className={`theme-toggle-dropdown ${className}`}
      style={{ position: 'relative', display: 'inline-block' }}
    >
      <button
        onClick={() => setIsOpen(!isOpen)}
        aria-haspopup="listbox"
        aria-expanded={isOpen}
        aria-label="Select theme"
        disabled={!isLoaded}
        style={{
          display: 'flex',
          alignItems: 'center',
          gap: sizeStyles.gap,
          padding: sizeStyles.padding,
          fontSize: sizeStyles.fontSize,
          backgroundColor: 'var(--bg-tertiary)',
          border: '1px solid var(--border)',
          borderRadius: '0.5rem',
          color: 'var(--text-primary)',
          cursor: 'pointer',
          opacity: isLoaded ? 1 : 0.5,
          transition: 'all 0.2s ease',
        }}
      >
        <span role="img" aria-hidden="true">{currentOption.icon}</span>
        {showLabels && <span>{currentOption.label}</span>}
        <span
          style={{
            marginLeft: '0.25rem',
            transform: isOpen ? 'rotate(180deg)' : 'rotate(0)',
            transition: 'transform 0.2s ease',
          }}
        >
          ▼
        </span>
      </button>

      {isOpen && (
        <div
          role="listbox"
          aria-label="Theme options"
          style={{
            position: 'absolute',
            top: '100%',
            right: 0,
            marginTop: '0.25rem',
            minWidth: '12rem',
            backgroundColor: 'var(--bg-panel)',
            border: '1px solid var(--border)',
            borderRadius: '0.5rem',
            boxShadow: '0 10px 15px -3px rgba(0, 0, 0, 0.3)',
            zIndex: 1000,
            overflow: 'hidden',
          }}
        >
          {themeOptions.map((option) => (
            <button
              key={option.value}
              role="option"
              aria-selected={theme === option.value}
              onClick={() => {
                setTheme(option.value);
                setIsOpen(false);
              }}
              style={{
                display: 'flex',
                alignItems: 'center',
                gap: '0.75rem',
                width: '100%',
                padding: '0.75rem 1rem',
                fontSize: sizeStyles.fontSize,
                textAlign: 'left',
                backgroundColor: theme === option.value ? 'var(--accent-muted)' : 'transparent',
                color: 'var(--text-primary)',
                border: 'none',
                cursor: 'pointer',
                transition: 'background-color 0.15s ease',
              }}
              onMouseEnter={(e) => {
                e.currentTarget.style.backgroundColor = 'var(--bg-tertiary)';
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.backgroundColor = 
                  theme === option.value ? 'var(--accent-muted)' : 'transparent';
              }}
            >
              <span role="img" aria-hidden="true" style={{ fontSize: '1.25rem' }}>
                {option.icon}
              </span>
              <div style={{ flex: 1 }}>
                <div style={{ fontWeight: 500 }}>{option.label}</div>
                <div style={{ fontSize: '0.75rem', color: 'var(--text-muted)' }}>
                  {option.description}
                </div>
              </div>
              {theme === option.value && (
                <span style={{ color: 'var(--accent)' }}>✓</span>
              )}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}

/**
 * Theme Toggle with Label
 * Includes a descriptive label for settings panels
 */
export function ThemeToggleWithLabel(props: Omit<ThemeToggleProps, 'showLabels'>): React.ReactElement {
  const { theme } = useTheme();
  const currentOption = themeOptions.find((opt) => opt.value === theme);

  return (
    <div style={{ display: 'flex', alignItems: 'center', gap: '1rem' }}>
      <div>
        <div style={{ fontWeight: 500, color: 'var(--text-primary)' }}>Theme</div>
        <div style={{ fontSize: '0.875rem', color: 'var(--text-muted)' }}>
          {currentOption?.description}
        </div>
      </div>
      <ThemeToggle {...props} showLabels={false} />
    </div>
  );
}

export default ThemeToggle;
