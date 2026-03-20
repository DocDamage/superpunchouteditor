import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';

export interface ExternalTool {
  id: string;
  name: string;
  executable_path: string;
  arguments_template: string;
  supported_file_types: string[];
  category: 'graphics_editor' | 'hex_editor' | 'tile_editor' | 'emulator' | 'other';
  enabled: boolean;
  working_directory?: string;
  env_vars: Record<string, string>;
}

export interface ToolContext {
  offset?: string;
  size?: number;
  snes_address?: string;
  category?: string;
  boxer?: string;
  metadata?: Record<string, string>;
}

export interface ToolCategory {
  value: ExternalTool['category'];
  label: string;
  icon: string;
  description: string;
  extensions: string[];
}

interface ExternalToolsManagerProps {
  isOpen: boolean;
  onClose: () => void;
}

const PRESET_TOOLS: ExternalTool[] = [
  {
    id: 'tile_layer_pro',
    name: 'Tile Layer Pro',
    executable_path: 'C:/Program Files (x86)/Tile Layer Pro/TLP.exe',
    arguments_template: '{file}',
    supported_file_types: ['bin', 'chr', 'gb', 'nes', 'sfc', 'smc', 'vra', 'bmp'],
    category: 'tile_editor',
    enabled: true,
    env_vars: {},
  },
  {
    id: 'yy_chr',
    name: 'YY-CHR',
    executable_path: 'C:/Program Files/YY-CHR/yy-chr.exe',
    arguments_template: '{file}',
    supported_file_types: ['bin', 'chr', 'nes', 'sfc', 'smc', 'gb', 'gbc'],
    category: 'tile_editor',
    enabled: true,
    env_vars: {},
  },
  {
    id: 'hxd',
    name: 'HxD',
    executable_path: 'C:/Program Files/HxD/HxD.exe',
    arguments_template: '{file}',
    supported_file_types: ['bin', 'sfc', 'smc', 'nes', 'dat', 'ips', 'bps'],
    category: 'hex_editor',
    enabled: true,
    env_vars: {},
  },
  {
    id: '010_editor',
    name: '010 Editor',
    executable_path: 'C:/Program Files/010 Editor/010Editor.exe',
    arguments_template: '{file}',
    supported_file_types: ['bin', 'sfc', 'smc', 'nes', 'dat', 'hex', '1sc', '1pk'],
    category: 'hex_editor',
    enabled: true,
    env_vars: {},
  },
  {
    id: 'aseprite',
    name: 'Aseprite',
    executable_path: 'C:/Program Files/Aseprite/aseprite.exe',
    arguments_template: '{file}',
    supported_file_types: ['png', 'ase', 'aseprite', 'gif', 'jpg', 'bmp', 'tga'],
    category: 'graphics_editor',
    enabled: true,
    env_vars: {},
  },
  {
    id: 'gimp',
    name: 'GIMP',
    executable_path: 'C:/Program Files/GIMP 2/bin/gimp-2.10.exe',
    arguments_template: '{file}',
    supported_file_types: ['png', 'jpg', 'jpeg', 'gif', 'bmp', 'tga', 'xcf'],
    category: 'graphics_editor',
    enabled: true,
    env_vars: {},
  },
  {
    id: 'photoshop',
    name: 'Adobe Photoshop',
    executable_path: 'C:/Program Files/Adobe/Adobe Photoshop/Photoshop.exe',
    arguments_template: '{file}',
    supported_file_types: ['psd', 'png', 'jpg', 'jpeg', 'gif', 'bmp', 'tga', 'tif', 'tiff'],
    category: 'graphics_editor',
    enabled: true,
    env_vars: {},
  },
];

const CATEGORY_OPTIONS: { value: ExternalTool['category']; label: string; icon: string }[] = [
  { value: 'graphics_editor', label: 'Graphics Editor', icon: '🎨' },
  { value: 'hex_editor', label: 'Hex Editor', icon: '🔢' },
  { value: 'tile_editor', label: 'Tile Editor', icon: '🧩' },
  { value: 'emulator', label: 'Emulator', icon: '🎮' },
  { value: 'other', label: 'Other', icon: '🔧' },
];

export function ExternalToolsManager({ isOpen, onClose }: ExternalToolsManagerProps) {
  const [tools, setTools] = useState<ExternalTool[]>([]);
  const [categories, setCategories] = useState<ToolCategory[]>([]);
  const [editingTool, setEditingTool] = useState<ExternalTool | null>(null);
  const [isAddingNew, setIsAddingNew] = useState(false);
  const [defaultTools, setDefaultTools] = useState<Record<string, string>>({});
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Load tools on mount
  useEffect(() => {
    if (isOpen) {
      loadTools();
      loadCategories();
    }
  }, [isOpen]);

  const loadTools = async () => {
    try {
      setLoading(true);
      const tools = await invoke<ExternalTool[]>('get_external_tools');
      setTools(tools);
      setError(null);
    } catch (e) {
      console.error('Failed to load tools:', e);
      setError('Failed to load external tools');
    } finally {
      setLoading(false);
    }
  };

  const loadCategories = async () => {
    try {
      const cats = await invoke<ToolCategory[]>('get_tool_categories');
      setCategories(cats);
    } catch (e) {
      console.error('Failed to load categories:', e);
    }
  };

  const handleBrowseExecutable = async () => {
    if (!editingTool) return;

    const selected = await open({
      multiple: false,
      filters: [
        {
          name: 'Executable',
          extensions: ['exe', 'bat', 'cmd'],
        },
      ],
    });

    if (typeof selected === 'string') {
      setEditingTool({ ...editingTool, executable_path: selected });
    }
  };

  const handleSaveTool = async () => {
    if (!editingTool) return;

    // Validate
    if (!editingTool.name.trim()) {
      setError('Tool name is required');
      return;
    }
    if (!editingTool.executable_path.trim()) {
      setError('Executable path is required');
      return;
    }

    try {
      setLoading(true);
      if (isAddingNew) {
        // Generate ID from name
        const id = editingTool.name
          .toLowerCase()
          .replace(/[^a-z0-9]+/g, '_')
          .replace(/^_+|_+$/g, '');
        const tool = { ...editingTool, id };
        await invoke('add_external_tool', { tool });
      } else {
        await invoke('update_external_tool', { tool: editingTool });
      }
      await loadTools();
      setEditingTool(null);
      setIsAddingNew(false);
      setError(null);
    } catch (e) {
      console.error('Failed to save tool:', e);
      setError(`Failed to save tool: ${e}`);
    } finally {
      setLoading(false);
    }
  };

  const handleDeleteTool = async (toolId: string) => {
    if (!confirm('Are you sure you want to delete this tool?')) return;

    try {
      setLoading(true);
      await invoke('remove_external_tool', { toolId });
      await loadTools();
    } catch (e) {
      console.error('Failed to delete tool:', e);
      setError(`Failed to delete tool: ${e}`);
    } finally {
      setLoading(false);
    }
  };

  const handleAddPreset = (preset: ExternalTool) => {
    setEditingTool({ ...preset });
    setIsAddingNew(true);
    setError(null);
  };

  const handleAddNew = () => {
    setEditingTool({
      id: '',
      name: '',
      executable_path: '',
      arguments_template: '{file}',
      supported_file_types: [],
      category: 'other',
      enabled: true,
      env_vars: {},
    });
    setIsAddingNew(true);
    setError(null);
  };

  const handleTestTool = async (tool: ExternalTool) => {
    try {
      const result = await invoke<{ valid: boolean; message: string }>('verify_tool', { tool });
      if (result.valid) {
        alert(`✓ ${tool.name} is valid and accessible`);
      } else {
        alert(`✗ ${result.message}`);
      }
    } catch (e) {
      alert(`✗ Failed to verify tool: ${e}`);
    }
  };

  const handleSetDefault = async (extension: string, toolId: string) => {
    try {
      await invoke('set_default_tool', { fileExtension: extension, toolId });
      setDefaultTools({ ...defaultTools, [extension]: toolId });
    } catch (e) {
      console.error('Failed to set default tool:', e);
    }
  };

  const handleToggleEnabled = async (tool: ExternalTool) => {
    try {
      const updated = { ...tool, enabled: !tool.enabled };
      await invoke('update_external_tool', { tool: updated });
      await loadTools();
    } catch (e) {
      console.error('Failed to toggle tool:', e);
    }
  };

  const getCategoryIcon = (category: ExternalTool['category']) => {
    return CATEGORY_OPTIONS.find(c => c.value === category)?.icon || '🔧';
  };

  const getCategoryLabel = (category: ExternalTool['category']) => {
    return CATEGORY_OPTIONS.find(c => c.value === category)?.label || 'Other';
  };

  if (!isOpen) return null;

  return (
    <div
      style={{
        position: 'fixed',
        top: 0,
        left: 0,
        right: 0,
        bottom: 0,
        backgroundColor: 'rgba(0, 0, 0, 0.7)',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        zIndex: 1000,
      }}
      onClick={onClose}
    >
      <div
        style={{
          backgroundColor: 'var(--panel-bg)',
          borderRadius: '12px',
          border: '1px solid var(--border)',
          padding: '2rem',
          width: '100%',
          maxWidth: '800px',
          maxHeight: '90vh',
          overflow: 'auto',
        }}
        onClick={(e) => e.stopPropagation()}
      >
        <h2 style={{ marginTop: 0, marginBottom: '1.5rem' }}>🛠️ External Tools</h2>

        {error && (
          <div
            style={{
              padding: '1rem',
              backgroundColor: 'rgba(220, 38, 38, 0.2)',
              borderRadius: '8px',
              color: 'var(--accent)',
              marginBottom: '1rem',
            }}
          >
            {error}
          </div>
        )}

        {editingTool ? (
          // Edit/Add Form
          <div>
            <h3 style={{ marginTop: 0 }}>{isAddingNew ? 'Add New Tool' : 'Edit Tool'}</h3>

            <div style={{ marginBottom: '1rem' }}>
              <label style={{ display: 'block', marginBottom: '0.5rem', color: 'var(--text-dim)' }}>
                Tool Name
              </label>
              <input
                type="text"
                value={editingTool.name}
                onChange={(e) => setEditingTool({ ...editingTool, name: e.target.value })}
                placeholder="e.g., HxD Hex Editor"
                style={{
                  width: '100%',
                  padding: '0.5rem',
                  borderRadius: '4px',
                  border: '1px solid var(--border)',
                  backgroundColor: 'var(--glass)',
                  color: 'var(--text)',
                }}
              />
            </div>

            <div style={{ marginBottom: '1rem' }}>
              <label style={{ display: 'block', marginBottom: '0.5rem', color: 'var(--text-dim)' }}>
                Category
              </label>
              <select
                value={editingTool.category}
                onChange={(e) => setEditingTool({ ...editingTool, category: e.target.value as ExternalTool['category'] })}
                style={{
                  width: '100%',
                  padding: '0.5rem',
                  borderRadius: '4px',
                  border: '1px solid var(--border)',
                  backgroundColor: 'var(--glass)',
                  color: 'var(--text)',
                }}
              >
                {CATEGORY_OPTIONS.map((cat) => (
                  <option key={cat.value} value={cat.value}>
                    {cat.icon} {cat.label}
                  </option>
                ))}
              </select>
            </div>

            <div style={{ marginBottom: '1rem' }}>
              <label style={{ display: 'block', marginBottom: '0.5rem', color: 'var(--text-dim)' }}>
                Executable Path
              </label>
              <div style={{ display: 'flex', gap: '0.5rem' }}>
                <input
                  type="text"
                  value={editingTool.executable_path}
                  onChange={(e) => setEditingTool({ ...editingTool, executable_path: e.target.value })}
                  placeholder="C:/Program Files/..."
                  style={{
                    flex: 1,
                    padding: '0.5rem',
                    borderRadius: '4px',
                    border: '1px solid var(--border)',
                    backgroundColor: 'var(--glass)',
                    color: 'var(--text)',
                  }}
                />
                <button onClick={handleBrowseExecutable}>Browse...</button>
              </div>
            </div>

            <div style={{ marginBottom: '1rem' }}>
              <label style={{ display: 'block', marginBottom: '0.5rem', color: 'var(--text-dim)' }}>
                Arguments Template
              </label>
              <input
                type="text"
                value={editingTool.arguments_template}
                onChange={(e) => setEditingTool({ ...editingTool, arguments_template: e.target.value })}
                placeholder="{file} --offset {offset}"
                style={{
                  width: '100%',
                  padding: '0.5rem',
                  borderRadius: '4px',
                  border: '1px solid var(--border)',
                  backgroundColor: 'var(--glass)',
                  color: 'var(--text)',
                }}
              />
              <p style={{ margin: '0.5rem 0 0 0', fontSize: '0.8rem', color: 'var(--text-dim)' }}>
                Available placeholders: {'{file}'}, {'{offset}'}, {'{size}'}, {'{snes_address}'}, {'{category}'}, {'{boxer}'}
              </p>
            </div>

            <div style={{ marginBottom: '1.5rem' }}>
              <label style={{ display: 'block', marginBottom: '0.5rem', color: 'var(--text-dim)' }}>
                Supported File Types (comma-separated)
              </label>
              <input
                type="text"
                value={editingTool.supported_file_types.join(', ')}
                onChange={(e) =>
                  setEditingTool({
                    ...editingTool,
                    supported_file_types: e.target.value.split(',').map((s) => s.trim().toLowerCase()),
                  })
                }
                placeholder="bin, png, sfc"
                style={{
                  width: '100%',
                  padding: '0.5rem',
                  borderRadius: '4px',
                  border: '1px solid var(--border)',
                  backgroundColor: 'var(--glass)',
                  color: 'var(--text)',
                }}
              />
            </div>

            <div style={{ display: 'flex', gap: '1rem', justifyContent: 'flex-end' }}>
              <button
                onClick={() => {
                  setEditingTool(null);
                  setIsAddingNew(false);
                  setError(null);
                }}
                style={{
                  padding: '0.75rem 1.5rem',
                  backgroundColor: 'transparent',
                  border: '1px solid var(--border)',
                }}
              >
                Cancel
              </button>
              <button onClick={handleSaveTool} disabled={loading}>
                {loading ? 'Saving...' : 'Save Tool'}
              </button>
            </div>
          </div>
        ) : (
          // Tools List
          <div>
            {/* Presets Section */}
            <div style={{ marginBottom: '2rem' }}>
              <h4 style={{ marginBottom: '1rem', color: 'var(--text-dim)' }}>Quick Add Presets</h4>
              <div style={{ display: 'flex', flexWrap: 'wrap', gap: '0.5rem' }}>
                {PRESET_TOOLS.map((preset) => (
                  <button
                    key={preset.id}
                    onClick={() => handleAddPreset(preset)}
                    style={{
                      padding: '0.5rem 1rem',
                      fontSize: '0.85rem',
                      backgroundColor: 'var(--glass)',
                      border: '1px solid var(--border)',
                      display: 'flex',
                      alignItems: 'center',
                      gap: '0.5rem',
                    }}
                  >
                    {getCategoryIcon(preset.category)} {preset.name}
                  </button>
                ))}
              </div>
            </div>

            {/* Configured Tools */}
            <div style={{ marginBottom: '2rem' }}>
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '1rem' }}>
                <h4 style={{ margin: 0, color: 'var(--text-dim)' }}>Configured Tools</h4>
                <button onClick={handleAddNew} style={{ padding: '0.5rem 1rem' }}>
                  + Add Custom Tool
                </button>
              </div>

              {tools.length === 0 ? (
                <div
                  style={{
                    padding: '2rem',
                    textAlign: 'center',
                    color: 'var(--text-dim)',
                    backgroundColor: 'var(--glass)',
                    borderRadius: '8px',
                  }}
                >
                  No tools configured yet. Use the presets above or add a custom tool.
                </div>
              ) : (
                <div style={{ display: 'flex', flexDirection: 'column', gap: '0.75rem' }}>
                  {tools.map((tool) => (
                    <div
                      key={tool.id}
                      style={{
                        padding: '1rem',
                        backgroundColor: 'var(--glass)',
                        borderRadius: '8px',
                        border: '1px solid var(--border)',
                        display: 'flex',
                        alignItems: 'center',
                        gap: '1rem',
                        opacity: tool.enabled ? 1 : 0.5,
                      }}
                    >
                      <span style={{ fontSize: '1.5rem' }}>{getCategoryIcon(tool.category)}</span>
                      
                      <div style={{ flex: 1 }}>
                        <div style={{ fontWeight: 500 }}>{tool.name}</div>
                        <div style={{ fontSize: '0.8rem', color: 'var(--text-dim)' }}>
                          {getCategoryLabel(tool.category)} • {tool.supported_file_types.join(', ')}
                        </div>
                      </div>

                      <label
                        style={{
                          display: 'flex',
                          alignItems: 'center',
                          gap: '0.5rem',
                          cursor: 'pointer',
                          fontSize: '0.85rem',
                        }}
                      >
                        <input
                          type="checkbox"
                          checked={tool.enabled}
                          onChange={() => handleToggleEnabled(tool)}
                        />
                        Enabled
                      </label>

                      <button
                        onClick={() => handleTestTool(tool)}
                        style={{ padding: '0.4rem 0.8rem', fontSize: '0.85rem' }}
                        title="Test if the tool is accessible"
                      >
                        Test
                      </button>

                      <button
                        onClick={() => {
                          setEditingTool(tool);
                          setIsAddingNew(false);
                        }}
                        style={{ padding: '0.4rem 0.8rem', fontSize: '0.85rem' }}
                      >
                        Edit
                      </button>

                      <button
                        onClick={() => handleDeleteTool(tool.id)}
                        style={{
                          padding: '0.4rem 0.8rem',
                          fontSize: '0.85rem',
                          backgroundColor: 'var(--accent)',
                        }}
                      >
                        Delete
                      </button>
                    </div>
                  ))}
                </div>
              )}
            </div>

            {/* Default Tools Section */}
            {tools.length > 0 && (
              <div style={{ marginBottom: '1.5rem' }}>
                <h4 style={{ marginBottom: '1rem', color: 'var(--text-dim)' }}>Default Tools by File Type</h4>
                <div style={{ display: 'flex', flexWrap: 'wrap', gap: '1rem' }}>
                  {['bin', 'png', 'sfc', 'smc'].map((ext) => (
                    <div key={ext} style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
                      <span style={{ fontSize: '0.85rem', textTransform: 'uppercase' }}>.{ext}:</span>
                      <select
                        value={defaultTools[ext] || ''}
                        onChange={(e) => e.target.value && handleSetDefault(ext, e.target.value)}
                        style={{
                          padding: '0.25rem 0.5rem',
                          borderRadius: '4px',
                          border: '1px solid var(--border)',
                          backgroundColor: 'var(--glass)',
                          color: 'var(--text)',
                          fontSize: '0.85rem',
                        }}
                      >
                        <option value="">None</option>
                        {tools
                          .filter((t) => t.enabled && t.supported_file_types.includes(ext))
                          .map((t) => (
                            <option key={t.id} value={t.id}>
                              {t.name}
                            </option>
                          ))}
                      </select>
                    </div>
                  ))}
                </div>
              </div>
            )}

            <div style={{ display: 'flex', gap: '1rem', justifyContent: 'flex-end' }}>
              <button
                onClick={onClose}
                style={{
                  padding: '0.75rem 1.5rem',
                  backgroundColor: 'transparent',
                  border: '1px solid var(--border)',
                }}
              >
                Close
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

export default ExternalToolsManager;
