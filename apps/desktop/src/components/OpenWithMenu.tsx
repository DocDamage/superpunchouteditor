import { useState, useEffect, useRef, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';

export interface ExternalTool {
  id: string;
  name: string;
  executable_path: string;
  arguments_template: string;
  supported_file_types: string[];
  category: 'graphics_editor' | 'hex_editor' | 'tile_editor' | 'emulator' | 'other';
  enabled: boolean;
}

export interface ToolContext {
  offset?: string;
  size?: number;
  snes_address?: string;
  category?: string;
  boxer?: string;
  metadata?: Record<string, string>;
}

export interface OpenWithMenuProps {
  filePath: string;
  fileExtension?: string;
  context?: ToolContext;
  children: React.ReactNode;
  onLaunch?: (toolName: string) => void;
  onError?: (error: string) => void;
}

interface MenuPosition {
  x: number;
  y: number;
}

const CATEGORY_ICONS: Record<ExternalTool['category'], string> = {
  graphics_editor: '🎨',
  hex_editor: '🔢',
  tile_editor: '🧩',
  emulator: '🎮',
  other: '🔧',
};

export function OpenWithMenu({
  filePath,
  fileExtension,
  context,
  children,
  onLaunch,
  onError,
}: OpenWithMenuProps) {
  const [isOpen, setIsOpen] = useState(false);
  const [menuPosition, setMenuPosition] = useState<MenuPosition>({ x: 0, y: 0 });
  const [tools, setTools] = useState<ExternalTool[]>([]);
  const [defaultTool, setDefaultTool] = useState<ExternalTool | null>(null);
  const [loading, setLoading] = useState(false);
  const menuRef = useRef<HTMLDivElement>(null);
  const buttonRef = useRef<HTMLDivElement>(null);

  const ext = fileExtension || filePath.split('.').pop()?.toLowerCase() || '';

  // Load compatible tools when menu opens
  useEffect(() => {
    if (isOpen) {
      loadTools();
    }
  }, [isOpen, ext]);

  // Close menu when clicking outside
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (
        menuRef.current &&
        !menuRef.current.contains(event.target as Node) &&
        buttonRef.current &&
        !buttonRef.current.contains(event.target as Node)
      ) {
        setIsOpen(false);
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  const loadTools = async () => {
    try {
      setLoading(true);
      const compatible = await invoke<ExternalTool[]>('get_compatible_tools', {
        fileExtension: ext,
      });
      setTools(compatible);

      // Load default tool
      const default_ = await invoke<ExternalTool | null>('get_default_tool', {
        fileExtension: ext,
      });
      setDefaultTool(default_);
    } catch (e) {
      console.error('Failed to load tools:', e);
    } finally {
      setLoading(false);
    }
  };

  const handleContextMenu = (e: React.MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();

    // Position menu near click but ensure it stays on screen
    const x = Math.min(e.clientX, window.innerWidth - 250);
    const y = Math.min(e.clientY, window.innerHeight - 200);

    setMenuPosition({ x, y });
    setIsOpen(true);
  };

  const handleLaunchTool = async (toolId: string, toolName: string) => {
    try {
      setLoading(true);
      await invoke('launch_with_tool', {
        toolId,
        filePath,
        context,
      });
      setIsOpen(false);
      onLaunch?.(toolName);
    } catch (e) {
      const error = `Failed to launch ${toolName}: ${e}`;
      console.error(error);
      onError?.(error);
    } finally {
      setLoading(false);
    }
  };

  const handleLaunchDefault = async () => {
    if (!defaultTool) return;
    await handleLaunchTool(defaultTool.id, defaultTool.name);
  };

  // Group tools by category
  const toolsByCategory = tools.reduce((acc, tool) => {
    if (!acc[tool.category]) {
      acc[tool.category] = [];
    }
    acc[tool.category].push(tool);
    return acc;
  }, {} as Record<ExternalTool['category'], ExternalTool[]>);

  const categoryOrder: ExternalTool['category'][] = [
    'graphics_editor',
    'hex_editor',
    'tile_editor',
    'emulator',
    'other',
  ];

  return (
    <div style={{ display: 'inline-block', position: 'relative' }}>
      <div
        ref={buttonRef}
        onContextMenu={handleContextMenu}
        style={{ cursor: 'context-menu' }}
      >
        {children}
      </div>

      {isOpen && (
        <div
          ref={menuRef}
          style={{
            position: 'fixed',
            left: menuPosition.x,
            top: menuPosition.y,
            backgroundColor: 'var(--panel-bg)',
            border: '1px solid var(--border)',
            borderRadius: '8px',
            boxShadow: '0 4px 12px rgba(0, 0, 0, 0.3)',
            zIndex: 1001,
            minWidth: '200px',
            maxWidth: '300px',
            overflow: 'hidden',
          }}
        >
          {/* Header */}
          <div
            style={{
              padding: '0.75rem 1rem',
              borderBottom: '1px solid var(--border)',
              backgroundColor: 'var(--glass)',
              fontSize: '0.85rem',
              color: 'var(--text-dim)',
            }}
          >
            Open With
            {ext && (
              <span style={{ textTransform: 'uppercase', marginLeft: '0.5rem' }}>
                (.{ext})
              </span>
            )}
          </div>

          {loading ? (
            <div
              style={{
                padding: '1rem',
                textAlign: 'center',
                color: 'var(--text-dim)',
                fontSize: '0.9rem',
              }}
            >
              Loading tools...
            </div>
          ) : tools.length === 0 ? (
            <div
              style={{
                padding: '1rem',
                textAlign: 'center',
                color: 'var(--text-dim)',
                fontSize: '0.9rem',
              }}
            >
              No compatible tools configured
              <div style={{ marginTop: '0.5rem', fontSize: '0.8rem' }}>
                Configure tools in External Tools settings
              </div>
            </div>
          ) : (
            <div style={{ maxHeight: '400px', overflowY: 'auto' }}>
              {/* Default Tool */}
              {defaultTool && (
                <>
                  <button
                    onClick={handleLaunchDefault}
                    style={{
                      width: '100%',
                      padding: '0.75rem 1rem',
                      textAlign: 'left',
                      backgroundColor: 'transparent',
                      border: 'none',
                      color: 'var(--text)',
                      cursor: 'pointer',
                      display: 'flex',
                      alignItems: 'center',
                      gap: '0.75rem',
                      fontSize: '0.95rem',
                      fontWeight: 500,
                    }}
                    onMouseEnter={(e) => {
                      e.currentTarget.style.backgroundColor = 'var(--glass)';
                    }}
                    onMouseLeave={(e) => {
                      e.currentTarget.style.backgroundColor = 'transparent';
                    }}
                  >
                    <span>{CATEGORY_ICONS[defaultTool.category]}</span>
                    <span style={{ flex: 1 }}>{defaultTool.name}</span>
                    <span
                      style={{
                        fontSize: '0.75rem',
                        color: 'var(--blue)',
                        backgroundColor: 'rgba(59, 130, 246, 0.2)',
                        padding: '0.15rem 0.4rem',
                        borderRadius: '4px',
                      }}
                    >
                      Default
                    </span>
                  </button>
                  <div
                    style={{
                      height: '1px',
                      backgroundColor: 'var(--border)',
                      margin: '0.25rem 0',
                    }}
                  />
                </>
              )}

              {/* Tools by Category */}
              {categoryOrder.map((category) => {
                const categoryTools = toolsByCategory[category];
                if (!categoryTools || categoryTools.length === 0) return null;

                // Filter out default tool since we already showed it
                const nonDefaultTools = defaultTool
                  ? categoryTools.filter((t) => t.id !== defaultTool.id)
                  : categoryTools;

                if (nonDefaultTools.length === 0) return null;

                return (
                  <div key={category}>
                    <div
                      style={{
                        padding: '0.5rem 1rem',
                        fontSize: '0.75rem',
                        color: 'var(--text-dim)',
                        textTransform: 'uppercase',
                        letterSpacing: '0.05em',
                      }}
                    >
                      {CATEGORY_ICONS[category]} {category.replace('_', ' ')}
                    </div>
                    {nonDefaultTools.map((tool) => (
                      <button
                        key={tool.id}
                        onClick={() => handleLaunchTool(tool.id, tool.name)}
                        style={{
                          width: '100%',
                          padding: '0.6rem 1rem 0.6rem 2rem',
                          textAlign: 'left',
                          backgroundColor: 'transparent',
                          border: 'none',
                          color: 'var(--text)',
                          cursor: 'pointer',
                          fontSize: '0.9rem',
                        }}
                        onMouseEnter={(e) => {
                          e.currentTarget.style.backgroundColor = 'var(--glass)';
                        }}
                        onMouseLeave={(e) => {
                          e.currentTarget.style.backgroundColor = 'transparent';
                        }}
                      >
                        {tool.name}
                      </button>
                    ))}
                  </div>
                );
              })}
            </div>
          )}

          {/* Footer with Configure link */}
          <div
            style={{
              padding: '0.5rem',
              borderTop: '1px solid var(--border)',
              backgroundColor: 'var(--glass)',
              textAlign: 'center',
            }}
          >
            <button
              onClick={() => {
                setIsOpen(false);
                // Dispatch event to open tools manager
                window.dispatchEvent(new CustomEvent('openExternalToolsManager'));
              }}
              style={{
                padding: '0.4rem 0.8rem',
                fontSize: '0.8rem',
                backgroundColor: 'transparent',
                border: 'none',
                color: 'var(--blue)',
                cursor: 'pointer',
              }}
            >
              Configure External Tools...
            </button>
          </div>
        </div>
      )}
    </div>
  );
}

// Hook to launch a tool programmatically
export function useToolLauncher() {
  const launchTool = useCallback(
    async (toolId: string, filePath: string, context?: ToolContext) => {
      await invoke('launch_with_tool', {
        toolId,
        filePath,
        context,
      });
    },
    []
  );

  const launchWithDefault = useCallback(
    async (filePath: string, fileExtension: string, context?: ToolContext) => {
      const defaultTool = await invoke<ExternalTool | null>('get_default_tool', {
        fileExtension,
      });

      if (!defaultTool) {
        throw new Error(`No default tool configured for .${fileExtension} files`);
      }

      await launchTool(defaultTool.id, filePath, context);
      return defaultTool.name;
    },
    [launchTool]
  );

  const getCompatibleTools = useCallback(async (fileExtension: string) => {
    return await invoke<ExternalTool[]>('get_compatible_tools', { fileExtension });
  }, []);

  return {
    launchTool,
    launchWithDefault,
    getCompatibleTools,
  };
}

export default OpenWithMenu;
