import React, { useState, useMemo } from 'react';
import './KeyboardShortcutsHelp.css';

interface Shortcut {
  keys: string[];
  description: string;
  category: string;
}

const SHORTCUTS: Shortcut[] = [
  // Global
  { keys: ['F1'], description: 'Open Help System', category: 'Global' },
  { keys: ['F5'], description: 'Test in Emulator', category: 'Global' },
  { keys: ['Ctrl', 'O'], description: 'Open ROM', category: 'Global' },
  { keys: ['Ctrl', 'S'], description: 'Save Project', category: 'Global' },
  { keys: ['Ctrl', 'Shift', 'S'], description: 'Save Project As', category: 'Global' },
  { keys: ['Ctrl', 'P'], description: 'Export Patch', category: 'Global' },
  { keys: ['Ctrl', ','], description: 'Open Settings', category: 'Global' },

  // Undo/Redo
  { keys: ['Ctrl', 'Z'], description: 'Undo', category: 'Editing' },
  { keys: ['Ctrl', 'Shift', 'Z'], description: 'Redo', category: 'Editing' },
  { keys: ['Ctrl', 'Y'], description: 'Redo (alternative)', category: 'Editing' },

  // Palette Editor
  { keys: ['1-9', '0'], description: 'Quick-select color index', category: 'Palette Editor' },
  { keys: ['Ctrl', 'C'], description: 'Copy selected color(s)', category: 'Palette Editor' },
  { keys: ['Ctrl', 'V'], description: 'Paste color(s)', category: 'Palette Editor' },
  { keys: ['Ctrl', 'D'], description: 'Duplicate palette', category: 'Palette Editor' },
  { keys: ['Delete'], description: 'Reset color to black', category: 'Palette Editor' },

  // Sprite Editor
  { keys: ['P'], description: 'Pencil tool', category: 'Sprite Editor' },
  { keys: ['L'], description: 'Line tool', category: 'Sprite Editor' },
  { keys: ['R'], description: 'Rectangle tool', category: 'Sprite Editor' },
  { keys: ['C'], description: 'Circle tool', category: 'Sprite Editor' },
  { keys: ['F'], description: 'Fill tool', category: 'Sprite Editor' },
  { keys: ['I'], description: 'Eyedropper tool', category: 'Sprite Editor' },
  { keys: ['S'], description: 'Select tool', category: 'Sprite Editor' },
  { keys: ['Ctrl', '+'], description: 'Zoom in', category: 'Sprite Editor' },
  { keys: ['Ctrl', '-'], description: 'Zoom out', category: 'Sprite Editor' },
  { keys: ['Ctrl', '0'], description: 'Reset zoom', category: 'Sprite Editor' },
  { keys: ['X'], description: 'Swap primary/secondary color', category: 'Sprite Editor' },
  { keys: ['['], description: 'Decrease brush size', category: 'Sprite Editor' },
  { keys: [']'], description: 'Increase brush size', category: 'Sprite Editor' },

  // Frame Reconstructor
  { keys: ['A'], description: 'Add sprite mode', category: 'Frame Editor' },
  { keys: ['S'], description: 'Select mode', category: 'Frame Editor' },
  { keys: ['M'], description: 'Move mode', category: 'Frame Editor' },
  { keys: ['Delete'], description: 'Delete selected sprite(s)', category: 'Frame Editor' },
  { keys: ['Ctrl', 'A'], description: 'Select all sprites', category: 'Frame Editor' },
  { keys: ['Arrow Keys'], description: 'Nudge selected sprite 1px', category: 'Frame Editor' },
  { keys: ['Shift', 'Arrow Keys'], description: 'Nudge selected sprite 8px', category: 'Frame Editor' },
  { keys: ['G'], description: 'Toggle grid', category: 'Frame Editor' },
  { keys: ['H'], description: 'Toggle hitboxes', category: 'Frame Editor' },

  // Animation Editor
  { keys: ['Space'], description: 'Play/Pause animation', category: 'Animation Editor' },
  { keys: ['←'], description: 'Previous frame', category: 'Animation Editor' },
  { keys: ['→'], description: 'Next frame', category: 'Animation Editor' },
  { keys: ['Home'], description: 'Go to first frame', category: 'Animation Editor' },
  { keys: ['End'], description: 'Go to last frame', category: 'Animation Editor' },
  { keys: ['+'], description: 'Increase frame duration', category: 'Animation Editor' },
  { keys: ['-'], description: 'Decrease frame duration', category: 'Animation Editor' },

  // Navigation
  { keys: ['Tab'], description: 'Next field', category: 'Navigation' },
  { keys: ['Shift', 'Tab'], description: 'Previous field', category: 'Navigation' },
  { keys: ['Esc'], description: 'Close modal/cancel action', category: 'Navigation' },

  // V4 Features - Tab Navigation
  { keys: ['Ctrl', '7'], description: 'Open Plugin Manager tab', category: 'V4 Tools' },
  { keys: ['Ctrl', '8'], description: 'Open Bank Visualization tab', category: 'V4 Tools' },
  { keys: ['Ctrl', '9'], description: 'Open Animation Player tab', category: 'V4 Tools' },
];

const CATEGORIES = Array.from(new Set(SHORTCUTS.map((s) => s.category)));

interface KeyboardShortcutsHelpProps {
  isOpen: boolean;
  onClose: () => void;
}

export const KeyboardShortcutsHelp: React.FC<KeyboardShortcutsHelpProps> = ({
  isOpen,
  onClose,
}) => {
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedCategory, setSelectedCategory] = useState<string | null>(null);

  const filteredShortcuts = useMemo(() => {
    let shortcuts = SHORTCUTS;

    if (selectedCategory) {
      shortcuts = shortcuts.filter((s) => s.category === selectedCategory);
    }

    if (searchQuery) {
      const query = searchQuery.toLowerCase();
      shortcuts = shortcuts.filter(
        (s) =>
          s.description.toLowerCase().includes(query) ||
          s.keys.some((k) => k.toLowerCase().includes(query))
      );
    }

    return shortcuts;
  }, [searchQuery, selectedCategory]);

  const shortcutsByCategory = useMemo(() => {
    const grouped: Record<string, Shortcut[]> = {};
    filteredShortcuts.forEach((shortcut) => {
      if (!grouped[shortcut.category]) {
        grouped[shortcut.category] = [];
      }
      grouped[shortcut.category].push(shortcut);
    });
    return grouped;
  }, [filteredShortcuts]);

  const formatKey = (key: string) => {
    // Map special keys to display names
    const keyMap: Record<string, string> = {
      'Ctrl': '⌘/Ctrl',
      'Shift': '⇧',
      'Alt': '⌥/Alt',
      'Arrow Keys': '↑↓←→',
      'ArrowLeft': '←',
      'ArrowRight': '→',
      'ArrowUp': '↑',
      'ArrowDown': '↓',
      'Delete': 'Del',
      'Escape': 'Esc',
      'Home': '↖',
      'End': '↘',
      'PageUp': 'PgUp',
      'PageDown': 'PgDn',
      'Space': '␣',
    };
    return keyMap[key] || key;
  };

  const handlePrint = () => {
    window.print();
  };

  if (!isOpen) return null;

  return (
    <div className="shortcuts-overlay" onClick={onClose}>
      <div className="shortcuts-modal" onClick={(e) => e.stopPropagation()}>
        <div className="shortcuts-header">
          <h2>⌨️ Keyboard Shortcuts</h2>
          <div className="shortcuts-actions">
            <button onClick={handlePrint} className="print-btn">
              🖨️ Print
            </button>
            <button onClick={onClose} className="close-btn">
              ×
            </button>
          </div>
        </div>

        <div className="shortcuts-toolbar">
          <div className="shortcuts-search">
            <span className="search-icon">🔍</span>
            <input
              type="text"
              placeholder="Search shortcuts..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
            />
            {searchQuery && (
              <button className="clear-btn" onClick={() => setSearchQuery('')}>
                ×
              </button>
            )}
          </div>

          <div className="category-filter">
            <button
              className={!selectedCategory ? 'active' : ''}
              onClick={() => setSelectedCategory(null)}
            >
              All
            </button>
            {CATEGORIES.map((cat) => (
              <button
                key={cat}
                className={selectedCategory === cat ? 'active' : ''}
                onClick={() => setSelectedCategory(cat)}
              >
                {cat}
              </button>
            ))}
          </div>
        </div>

        <div className="shortcuts-content">
          {Object.entries(shortcutsByCategory).map(([category, shortcuts]) => (
            <div key={category} className="shortcut-category">
              <h3>{category}</h3>
              <div className="shortcut-list">
                {shortcuts.map((shortcut, index) => (
                  <div key={index} className="shortcut-item">
                    <div className="shortcut-keys">
                      {shortcut.keys.map((key, i) => (
                        <React.Fragment key={i}>
                          <kbd>{formatKey(key)}</kbd>
                          {i < shortcut.keys.length - 1 && (
                            <span className="key-separator">+</span>
                          )}
                        </React.Fragment>
                      ))}
                    </div>
                    <span className="shortcut-description">
                      {shortcut.description}
                    </span>
                  </div>
                ))}
              </div>
            </div>
          ))}

          {filteredShortcuts.length === 0 && (
            <div className="no-results">
              <p>No shortcuts found matching "{searchQuery}"</p>
              <button onClick={() => setSearchQuery('')}>Clear Search</button>
            </div>
          )}
        </div>

        <div className="shortcuts-footer">
          <div className="legend">
            <span>
              <kbd>⌘/Ctrl</kbd> = Command (Mac) / Ctrl (Windows)
            </span>
            <span>
              <kbd>⌥/Alt</kbd> = Option (Mac) / Alt (Windows)
            </span>
          </div>
          <div className="total-count">
            {filteredShortcuts.length} shortcut
            {filteredShortcuts.length !== 1 ? 's' : ''}
          </div>
        </div>
      </div>
    </div>
  );
};

export default KeyboardShortcutsHelp;
