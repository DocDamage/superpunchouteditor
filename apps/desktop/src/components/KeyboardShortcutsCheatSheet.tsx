/**
 * Keyboard Shortcuts Cheat Sheet Component
 * 
 * A searchable, printable modal showing all keyboard shortcuts organized by category.
 * Features:
 * - Search filters shortcuts
 * - Category tabs
 * - Click to copy shortcut
 * - Printable version (CSS media query)
 * - Export as JSON
 */

import { useState, useMemo, useCallback, useEffect } from 'react';
import {
  Shortcut,
  ShortcutCategory,
  SHORTCUTS,
  formatShortcut,
  formatShortcutDisplay,
  getShortcutBadges,
  searchShortcuts,
  groupShortcutsByCategory,
  getCategories,
  exportShortcutsAsJson,
} from '../config/shortcuts';

interface KeyboardShortcutsCheatSheetProps {
  isOpen: boolean;
  onClose: () => void;
}

export function KeyboardShortcutsCheatSheet({ isOpen, onClose }: KeyboardShortcutsCheatSheetProps) {
  const [searchQuery, setSearchQuery] = useState('');
  const [activeCategory, setActiveCategory] = useState<ShortcutCategory | 'All'>('All');
  const [copiedId, setCopiedId] = useState<string | null>(null);

  // Reset state when modal opens
  useEffect(() => {
    if (isOpen) {
      setSearchQuery('');
      setActiveCategory('All');
      setCopiedId(null);
    }
  }, [isOpen]);

  // Close on Escape key
  useEffect(() => {
    if (!isOpen) return;
    
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        onClose();
      }
    };
    
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [isOpen, onClose]);

  // Filter shortcuts based on search and category
  const filteredShortcuts = useMemo(() => {
    let shortcuts = SHORTCUTS;
    
    if (searchQuery.trim()) {
      shortcuts = searchShortcuts(searchQuery);
    }
    
    if (activeCategory !== 'All') {
      shortcuts = shortcuts.filter(s => s.category === activeCategory);
    }
    
    return shortcuts;
  }, [searchQuery, activeCategory]);

  // Group shortcuts by category for display
  const groupedShortcuts = useMemo(() => {
    return groupShortcutsByCategory(filteredShortcuts);
  }, [filteredShortcuts]);

  // Get categories that have shortcuts after filtering
  const availableCategories = useMemo(() => {
    return getCategories().filter(cat => groupedShortcuts[cat]?.length > 0);
  }, [groupedShortcuts]);

  // Handle copy to clipboard
  const handleCopyShortcut = useCallback((shortcut: Shortcut) => {
    const shortcutText = formatShortcut(shortcut.keys);
    navigator.clipboard.writeText(shortcutText).then(() => {
      setCopiedId(shortcut.id);
      setTimeout(() => setCopiedId(null), 1500);
    });
  }, []);

  // Handle export to JSON
  const handleExportJson = useCallback(() => {
    const json = exportShortcutsAsJson();
    const blob = new Blob([json], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `spo-shortcuts-${new Date().toISOString().split('T')[0]}.json`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  }, []);

  // Handle print
  const handlePrint = useCallback(() => {
    window.print();
  }, []);

  if (!isOpen) return null;

  const categories = getCategories();

  return (
    <div
      className="cheat-sheet-overlay"
      style={{
        position: 'fixed',
        top: 0,
        left: 0,
        right: 0,
        bottom: 0,
        backgroundColor: 'rgba(0, 0, 0, 0.8)',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        zIndex: 10000,
      }}
      onClick={onClose}
    >
      <div
        className="cheat-sheet-modal"
        style={{
          backgroundColor: 'var(--panel-bg)',
          borderRadius: '12px',
          border: '1px solid var(--border)',
          width: '100%',
          maxWidth: '800px',
          maxHeight: '90vh',
          display: 'flex',
          flexDirection: 'column',
          overflow: 'hidden',
          boxShadow: '0 25px 50px rgba(0, 0, 0, 0.5)',
        }}
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div
          className="cheat-sheet-header"
          style={{
            padding: '1.25rem 1.5rem',
            borderBottom: '1px solid var(--border)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            gap: '1rem',
          }}
        >
          <div style={{ display: 'flex', alignItems: 'center', gap: '0.75rem' }}>
            <span style={{ fontSize: '1.5rem' }}>⌨️</span>
            <h2 style={{ margin: 0, fontSize: '1.25rem' }}>Keyboard Shortcuts</h2>
          </div>
          <input
            type="text"
            placeholder="Search shortcuts..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="cheat-sheet-search"
            style={{
              padding: '0.5rem 1rem',
              borderRadius: '6px',
              border: '1px solid var(--border)',
              backgroundColor: 'var(--glass)',
              color: 'var(--text)',
              width: '200px',
              fontSize: '0.9rem',
            }}
          />
        </div>

        {/* Category Tabs */}
        <div
          className="cheat-sheet-tabs"
          style={{
            padding: '0.75rem 1.5rem',
            borderBottom: '1px solid var(--border)',
            display: 'flex',
            gap: '0.5rem',
            overflowX: 'auto',
            flexWrap: 'wrap',
          }}
        >
          <TabButton
            active={activeCategory === 'All'}
            onClick={() => setActiveCategory('All')}
            label="All"
            count={SHORTCUTS.length}
          />
          {categories.map((cat) => {
            const count = searchQuery 
              ? filteredShortcuts.filter(s => s.category === cat).length
              : SHORTCUTS.filter(s => s.category === cat).length;
            return (
              <TabButton
                key={cat}
                active={activeCategory === cat}
                onClick={() => setActiveCategory(cat)}
                label={cat}
                count={count}
              />
            );
          })}
        </div>

        {/* Content */}
        <div
          className="cheat-sheet-content"
          style={{
            flex: 1,
            overflow: 'auto',
            padding: '1.5rem',
          }}
        >
          {filteredShortcuts.length === 0 ? (
            <div
              style={{
                textAlign: 'center',
                padding: '3rem',
                color: 'var(--text-dim)',
              }}
            >
              <p>No shortcuts found matching &quot;{searchQuery}&quot;</p>
            </div>
          ) : (
            <div style={{ display: 'flex', flexDirection: 'column', gap: '1.5rem' }}>
              {(activeCategory === 'All' ? availableCategories : [activeCategory]).map(
                (category) =>
                  groupedShortcuts[category]?.length > 0 && (
                    <CategorySection
                      key={category}
                      category={category}
                      shortcuts={groupedShortcuts[category]}
                      copiedId={copiedId}
                      onCopy={handleCopyShortcut}
                    />
                  )
              )}
            </div>
          )}
        </div>

        {/* Footer */}
        <div
          className="cheat-sheet-footer"
          style={{
            padding: '1rem 1.5rem',
            borderTop: '1px solid var(--border)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
          }}
        >
          <div style={{ display: 'flex', gap: '0.5rem' }}>
            <button
              onClick={handlePrint}
              style={{
                padding: '0.5rem 1rem',
                fontSize: '0.85rem',
                backgroundColor: 'var(--glass)',
              }}
            >
              🖨️ Print
            </button>
            <button
              onClick={handleExportJson}
              style={{
                padding: '0.5rem 1rem',
                fontSize: '0.85rem',
                backgroundColor: 'var(--glass)',
              }}
            >
              📥 Export JSON
            </button>
          </div>
          <span
            style={{
              fontSize: '0.8rem',
              color: 'var(--text-dim)',
            }}
          >
            Press <kbd
              style={{
                backgroundColor: 'var(--glass)',
                padding: '0.125rem 0.375rem',
                borderRadius: '4px',
                fontFamily: 'monospace',
              }}
            >?</kbd> to open
          </span>
        </div>
      </div>

      {/* Print Styles */}
      <style>{`
        @media print {
          .cheat-sheet-overlay {
            position: static !important;
            background: white !important;
            display: block !important;
            padding: 1rem;
          }
          
          .cheat-sheet-modal {
            position: static !important;
            width: 100% !important;
            max-width: none !important;
            max-height: none !important;
            box-shadow: none !important;
            border: 1px solid #ccc !important;
            background: white !important;
            color: black !important;
          }
          
          .cheat-sheet-search,
          .cheat-sheet-tabs {
            display: none !important;
          }
          
          .cheat-sheet-header {
            border-bottom-color: #ccc !important;
          }
          
          .cheat-sheet-content {
            max-height: none !important;
            overflow: visible !important;
          }
          
          .cheat-sheet-footer {
            display: none !important;
          }
          
          .shortcut-key {
            border: 1px solid #999 !important;
            background: #f0f0f0 !important;
            color: black !important;
            box-shadow: none !important;
          }
          
          .category-title {
            color: black !important;
            border-bottom-color: #ccc !important;
          }
          
          .shortcut-row:hover {
            background: transparent !important;
          }
        }
      `}</style>
    </div>
  );
}

// Tab Button Component
interface TabButtonProps {
  active: boolean;
  onClick: () => void;
  label: string;
  count: number;
}

function TabButton({ active, onClick, label, count }: TabButtonProps) {
  return (
    <button
      onClick={onClick}
      style={{
        padding: '0.375rem 0.75rem',
        borderRadius: '6px',
        border: '1px solid',
        borderColor: active ? 'var(--blue)' : 'var(--border)',
        backgroundColor: active ? 'var(--blue)' : 'transparent',
        color: active ? 'white' : 'var(--text)',
        fontSize: '0.85rem',
        cursor: 'pointer',
        whiteSpace: 'nowrap',
        display: 'flex',
        alignItems: 'center',
        gap: '0.375rem',
      }}
    >
      {label}
      <span
        style={{
          fontSize: '0.75rem',
          opacity: 0.7,
          backgroundColor: active ? 'rgba(255,255,255,0.2)' : 'var(--glass)',
          padding: '0.125rem 0.375rem',
          borderRadius: '4px',
        }}
      >
        {count}
      </span>
    </button>
  );
}

// Category Section Component
interface CategorySectionProps {
  category: ShortcutCategory | string;
  shortcuts: Shortcut[];
  copiedId: string | null;
  onCopy: (shortcut: Shortcut) => void;
}

function CategorySection({ category, shortcuts, copiedId, onCopy }: CategorySectionProps) {
  return (
    <div>
      <h3
        className="category-title"
        style={{
          margin: '0 0 0.75rem 0',
          fontSize: '0.9rem',
          textTransform: 'uppercase',
          letterSpacing: '1px',
          color: 'var(--text-dim)',
          borderBottom: '1px solid var(--border)',
          paddingBottom: '0.5rem',
        }}
      >
        {category}
      </h3>
      <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem' }}>
        {shortcuts.map((shortcut) => (
          <ShortcutRow
            key={shortcut.id}
            shortcut={shortcut}
            isCopied={copiedId === shortcut.id}
            onCopy={() => onCopy(shortcut)}
          />
        ))}
      </div>
    </div>
  );
}

// Shortcut Row Component
interface ShortcutRowProps {
  shortcut: Shortcut;
  isCopied: boolean;
  onCopy: () => void;
}

function ShortcutRow({ shortcut, isCopied, onCopy }: ShortcutRowProps) {
  const badges = getShortcutBadges(shortcut.keys);
  const displayText = formatShortcutDisplay(shortcut.keys);

  return (
    <div
      className="shortcut-row"
      onClick={onCopy}
      style={{
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
        padding: '0.625rem 0.875rem',
        borderRadius: '6px',
        cursor: 'pointer',
        transition: 'background-color 0.15s',
        backgroundColor: isCopied ? 'rgba(74, 222, 128, 0.1)' : 'transparent',
      }}
      onMouseEnter={(e) => {
        if (!isCopied) {
          e.currentTarget.style.backgroundColor = 'var(--glass)';
        }
      }}
      onMouseLeave={(e) => {
        if (!isCopied) {
          e.currentTarget.style.backgroundColor = 'transparent';
        }
      }}
      title="Click to copy"
    >
      <span style={{ color: 'var(--text)', fontSize: '0.95rem' }}>
        {shortcut.description}
      </span>
      <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
        {isCopied && (
          <span
            style={{
              fontSize: '0.75rem',
              color: '#4ade80',
              marginRight: '0.5rem',
            }}
          >
            Copied!
          </span>
        )}
        <div style={{ display: 'flex', gap: '0.25rem' }}>
          {badges.map((badge, idx) => (
            <kbd
              key={idx}
              className="shortcut-key"
              style={{
                display: 'inline-flex',
                alignItems: 'center',
                justifyContent: 'center',
                padding: '0.25rem 0.5rem',
                backgroundColor: 'var(--glass)',
                border: '1px solid var(--border)',
                borderRadius: '4px',
                fontSize: '0.8rem',
                fontFamily: 'monospace',
                minWidth: '1.5rem',
                boxShadow: '0 2px 0 var(--border)',
              }}
            >
              {badge}
            </kbd>
          ))}
        </div>
      </div>
    </div>
  );
}

export default KeyboardShortcutsCheatSheet;
