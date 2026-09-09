/**
 * Shortcut Display Component
 * 
 * Shows keyboard shortcuts inline with UI elements.
 * Usage: <ShortcutDisplay shortcutId="save" />
 * Renders: [Ctrl] + [S]
 */

import { useMemo } from 'react';
import {
  getShortcutById,
  getShortcutBadges,
  formatShortcutDisplay,
} from '../config/shortcuts';

interface ShortcutDisplayProps {
  /** The ID of the shortcut to display */
  shortcutId: string;
  /** Optional additional className */
  className?: string;
  /** Optional inline styles */
  style?: React.CSSProperties;
  /** Show as compact badges (default: false) */
  compact?: boolean;
  /** Show only the display text without individual key badges */
  textOnly?: boolean;
}

/**
 * Display a keyboard shortcut inline
 * 
 * @example
 * // Display with individual key badges
 * <ShortcutDisplay shortcutId="save" />
 * // Renders: [Ctrl] [S]
 * 
 * @example
 * // Display as compact text
 * <ShortcutDisplay shortcutId="save" textOnly />
 * // Renders: Ctrl+S
 * 
 * @example
 * // Display as compact badges
 * <ShortcutDisplay shortcutId="save" compact />
 * // Renders: smaller [Ctrl] [S]
 */
export function ShortcutDisplay({
  shortcutId,
  className = '',
  style = {},
  compact = false,
  textOnly = false,
}: ShortcutDisplayProps) {
  const shortcut = useMemo(() => getShortcutById(shortcutId), [shortcutId]);

  if (!shortcut) {
    console.warn(`Shortcut with id "${shortcutId}" not found`);
    return null;
  }

  const badges = getShortcutBadges(shortcut.keys);
  const displayText = formatShortcutDisplay(shortcut.keys);

  if (textOnly) {
    return (
      <kbd
        className={`shortcut-display-text ${className}`}
        style={{
          fontFamily: 'monospace',
          fontSize: compact ? '0.75rem' : '0.85rem',
          color: 'var(--text-dim)',
          ...style,
        }}
        title={shortcut.description}
      >
        {displayText}
      </kbd>
    );
  }

  return (
    <span
      className={`shortcut-display ${className}`}
      style={{
        display: 'inline-flex',
        alignItems: 'center',
        gap: compact ? '0.125rem' : '0.25rem',
        ...style,
      }}
      title={shortcut.description}
    >
      {badges.map((badge, idx) => (
        <kbd
          key={idx}
          style={{
            display: 'inline-flex',
            alignItems: 'center',
            justifyContent: 'center',
            padding: compact ? '0.125rem 0.375rem' : '0.25rem 0.5rem',
            backgroundColor: 'var(--glass)',
            border: '1px solid var(--border)',
            borderRadius: '4px',
            fontSize: compact ? '0.7rem' : '0.8rem',
            fontFamily: 'monospace',
            minWidth: compact ? '1.25rem' : '1.5rem',
            boxShadow: '0 2px 0 var(--border)',
            color: 'var(--text-dim)',
          }}
        >
          {badge}
        </kbd>
      ))}
    </span>
  );
}

interface ShortcutGroupProps {
  /** Array of shortcut IDs to display */
  shortcutIds: string[];
  /** Separator between shortcuts (default: ", ") */
  separator?: string;
  /** Additional className */
  className?: string;
  /** Inline styles */
  style?: React.CSSProperties;
  compact?: boolean;
  textOnly?: boolean;
}

/**
 * Display multiple shortcuts separated by a delimiter
 * 
 * @example
 * <ShortcutGroup shortcutIds={['undo', 'redo']} separator=" / " />
 * // Renders: [Ctrl] [Z] / [Ctrl] [Shift] [Z]
 */
export function ShortcutGroup({
  shortcutIds,
  separator = ', ',
  className = '',
  style = {},
  compact = false,
  textOnly = false,
}: ShortcutGroupProps) {
  const validShortcuts = useMemo(() => {
    return shortcutIds
      .map(id => getShortcutById(id))
      .filter((s): s is NonNullable<typeof s> => s !== undefined);
  }, [shortcutIds]);

  if (validShortcuts.length === 0) {
    return null;
  }

  return (
    <span className={`shortcut-group ${className}`} style={style}>
      {validShortcuts.map((shortcut, idx) => (
        <span key={shortcut.id}>
          <ShortcutDisplay
            shortcutId={shortcut.id}
            compact={compact}
            textOnly={textOnly}
          />
          {idx < validShortcuts.length - 1 && (
            <span style={{ color: 'var(--text-dim)', margin: '0 0.25rem' }}>
              {separator}
            </span>
          )}
        </span>
      ))}
    </span>
  );
}

interface ShortcutHintProps {
  /** The shortcut ID */
  shortcutId: string;
  /** Position of the hint relative to children */
  position?: 'left' | 'right';
  /** Children to wrap */
  children: React.ReactNode;
  /** Additional className */
  className?: string;
  /** Inline styles for the wrapper */
  style?: React.CSSProperties;
}

/**
 * Wraps content and shows a shortcut hint next to it
 * 
 * @example
 * <ShortcutHint shortcutId="save" position="right">
 *   <button>Save</button>
 * </ShortcutHint>
 * // Renders: [Save] [Ctrl] [S]
 */
export function ShortcutHint({
  shortcutId,
  position = 'right',
  children,
  className = '',
  style = {},
}: ShortcutHintProps) {
  const shortcut = useMemo(() => getShortcutById(shortcutId), [shortcutId]);

  if (!shortcut) {
    return <>{children}</>;
  }

  const hint = (
    <ShortcutDisplay
      shortcutId={shortcutId}
      compact
      style={{ marginLeft: position === 'right' ? '0.5rem' : 0, marginRight: position === 'left' ? '0.5rem' : 0 }}
    />
  );

  return (
    <span
      className={`shortcut-hint ${className}`}
      style={{
        display: 'inline-flex',
        alignItems: 'center',
        ...style,
      }}
    >
      {position === 'left' && hint}
      {children}
      {position === 'right' && hint}
    </span>
  );
}

export default ShortcutDisplay;
