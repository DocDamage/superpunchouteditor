import { useState, useCallback } from 'react';
import { open } from '@tauri-apps/plugin-dialog';

// ============================================================================
// Type Definitions
// ============================================================================

export interface PluginCommand {
  name: string;
  description: string;
  args: PluginCommandArg[];
}

export interface PluginCommandArg {
  name: string;
  type: 'string' | 'number' | 'boolean' | 'hex';
  required: boolean;
  default?: string | number | boolean;
  description?: string;
}

export interface Plugin {
  id: string;
  name: string;
  version: string;
  author: string;
  description: string;
  enabled: boolean;
  path: string;
  commands: PluginCommand[];
  loaded_at: string;
}

export interface BatchJob {
  id: string;
  name: string;
  plugin_id: string;
  status: 'pending' | 'running' | 'completed' | 'failed';
  progress: number;
  total: number;
  current_item?: string;
  error?: string;
  started_at?: string;
  completed_at?: string;
}

export interface ScriptOutput {
  success: boolean;
  output: string;
  error?: string;
  execution_time_ms: number;
}

// ============================================================================
// Hook Type Definitions (to be implemented by another agent)
// ============================================================================

interface UsePluginsResult {
  plugins: Plugin[];
  isLoading: boolean;
  error: string | null;
  refetch: () => void;
}

interface UseLoadPluginResult {
  loadPlugin: (path: string) => Promise<void>;
  isLoading: boolean;
  error: string | null;
}

interface UseUnloadPluginResult {
  unloadPlugin: (pluginId: string) => Promise<void>;
  isLoading: boolean;
  error: string | null;
}

interface UseEnablePluginResult {
  enablePlugin: (pluginId: string) => Promise<void>;
  isLoading: boolean;
  error: string | null;
}

interface UseDisablePluginResult {
  disablePlugin: (pluginId: string) => Promise<void>;
  isLoading: boolean;
  error: string | null;
}

interface UseExecutePluginCommandResult {
  executeCommand: (pluginId: string, command: string, args: Record<string, unknown>) => Promise<unknown>;
  isLoading: boolean;
  error: string | null;
  result: unknown | null;
}

interface UseRunScriptResult {
  runScript: (script: string) => Promise<ScriptOutput>;
  isLoading: boolean;
  error: string | null;
  output: ScriptOutput | null;
}

interface UseBatchJobsResult {
  jobs: BatchJob[];
  isLoading: boolean;
  error: string | null;
  refetch: () => void;
  cancelJob: (jobId: string) => Promise<void>;
}

// ============================================================================
// Stub Hooks (to be replaced with actual implementations)
// ============================================================================

// These are placeholder implementations that will be replaced by another agent
const usePlugins = (): UsePluginsResult => ({
  plugins: [],
  isLoading: false,
  error: null,
  refetch: () => {},
});

const useLoadPlugin = (): UseLoadPluginResult => ({
  loadPlugin: async () => {},
  isLoading: false,
  error: null,
});

const useUnloadPlugin = (): UseUnloadPluginResult => ({
  unloadPlugin: async () => {},
  isLoading: false,
  error: null,
});

const useEnablePlugin = (): UseEnablePluginResult => ({
  enablePlugin: async () => {},
  isLoading: false,
  error: null,
});

const useDisablePlugin = (): UseDisablePluginResult => ({
  disablePlugin: async () => {},
  isLoading: false,
  error: null,
});

const useExecutePluginCommand = (): UseExecutePluginCommandResult => ({
  executeCommand: async () => ({}),
  isLoading: false,
  error: null,
  result: null,
});

const useRunScript = (): UseRunScriptResult => ({
  runScript: async () => ({ success: true, output: '', execution_time_ms: 0 }),
  isLoading: false,
  error: null,
  output: null,
});

const useBatchJobs = (): UseBatchJobsResult => ({
  jobs: [],
  isLoading: false,
  error: null,
  refetch: () => {},
  cancelJob: async () => {},
});

// ============================================================================
// Plugin List Item Component
// ============================================================================

interface PluginListItemProps {
  plugin: Plugin;
  isSelected: boolean;
  onSelect: () => void;
  onToggleEnabled: () => void;
  onUnload: () => void;
  isToggling: boolean;
  isUnloading: boolean;
}

function PluginListItem({
  plugin,
  isSelected,
  onSelect,
  onToggleEnabled,
  onUnload,
  isToggling,
  isUnloading,
}: PluginListItemProps) {
  return (
    <div
      onClick={onSelect}
      style={{
        padding: '1rem',
        backgroundColor: isSelected ? 'var(--blue-muted, rgba(59, 130, 246, 0.2))' : 'var(--glass)',
        borderRadius: '8px',
        border: `1px solid ${isSelected ? 'var(--blue)' : 'var(--border)'}`,
        cursor: 'pointer',
        display: 'flex',
        alignItems: 'center',
        gap: '1rem',
        opacity: plugin.enabled ? 1 : 0.6,
        transition: 'all 0.2s ease',
      }}
    >
      <div
        style={{
          width: '40px',
          height: '40px',
          borderRadius: '8px',
          backgroundColor: plugin.enabled ? 'var(--blue)' : 'var(--text-dim)',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          fontSize: '1.25rem',
          flexShrink: 0,
        }}
      >
        🔌
      </div>

      <div style={{ flex: 1, minWidth: 0 }}>
        <div
          style={{
            fontWeight: 500,
            marginBottom: '0.25rem',
            display: 'flex',
            alignItems: 'center',
            gap: '0.5rem',
          }}
        >
          <span style={{ overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
            {plugin.name}
          </span>
          <span
            style={{
              fontSize: '0.75rem',
              color: 'var(--text-dim)',
              backgroundColor: 'var(--panel-bg)',
              padding: '0.125rem 0.375rem',
              borderRadius: '4px',
              flexShrink: 0,
            }}
          >
            v{plugin.version}
          </span>
        </div>
        <div
          style={{
            fontSize: '0.8rem',
            color: 'var(--text-dim)',
            overflow: 'hidden',
            textOverflow: 'ellipsis',
            whiteSpace: 'nowrap',
          }}
        >
          by {plugin.author}
        </div>
      </div>

      <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
        <label
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: '0.375rem',
            cursor: 'pointer',
            fontSize: '0.8rem',
            color: 'var(--text-dim)',
            padding: '0.25rem 0.5rem',
            borderRadius: '4px',
            backgroundColor: 'var(--panel-bg)',
          }}
          onClick={(e) => e.stopPropagation()}
        >
          <input
            type="checkbox"
            checked={plugin.enabled}
            onChange={onToggleEnabled}
            disabled={isToggling}
          />
          {isToggling ? '...' : 'On'}
        </label>

        <button
          onClick={(e) => {
            e.stopPropagation();
            onUnload();
          }}
          disabled={isUnloading}
          style={{
            padding: '0.375rem 0.75rem',
            fontSize: '0.8rem',
            backgroundColor: 'transparent',
            border: '1px solid var(--border)',
            color: 'var(--text-dim)',
            cursor: 'pointer',
            borderRadius: '4px',
          }}
          title="Unload plugin"
        >
          {isUnloading ? '...' : '×'}
        </button>
      </div>
    </div>
  );
}

// ============================================================================
// Plugin Details Component
// ============================================================================

interface PluginDetailsProps {
  plugin: Plugin;
  onExecuteCommand: (command: string, args: Record<string, unknown>) => void;
  isExecuting: boolean;
}

function PluginDetails({ plugin, onExecuteCommand, isExecuting }: PluginDetailsProps) {
  const [selectedCommand, setSelectedCommand] = useState<string>('');
  const [commandArgs, setCommandArgs] = useState<Record<string, string>>({});

  const currentCommand = plugin.commands.find((cmd) => cmd.name === selectedCommand);

  const handleExecute = () => {
    if (!selectedCommand) return;

    // Parse args based on their types
    const parsedArgs: Record<string, unknown> = {};
    if (currentCommand) {
      currentCommand.args.forEach((arg) => {
        const value = commandArgs[arg.name];
        if (value !== undefined && value !== '') {
          switch (arg.type) {
            case 'number':
              parsedArgs[arg.name] = parseFloat(value);
              break;
            case 'boolean':
              parsedArgs[arg.name] = value === 'true';
              break;
            case 'hex':
              parsedArgs[arg.name] = parseInt(value.replace(/^0x/, ''), 16);
              break;
            default:
              parsedArgs[arg.name] = value;
          }
        } else if (arg.default !== undefined) {
          parsedArgs[arg.name] = arg.default;
        }
      });
    }

    onExecuteCommand(selectedCommand, parsedArgs);
  };

  return (
    <div
      style={{
        backgroundColor: 'var(--glass)',
        borderRadius: '12px',
        border: '1px solid var(--border)',
        padding: '1.5rem',
        height: '100%',
        overflow: 'auto',
      }}
    >
      <div style={{ marginBottom: '1.5rem' }}>
        <div
          style={{
            display: 'flex',
            alignItems: 'flex-start',
            justifyContent: 'space-between',
            marginBottom: '0.5rem',
          }}
        >
          <h3 style={{ margin: 0, fontSize: '1.25rem' }}>{plugin.name}</h3>
          <span
            style={{
              fontSize: '0.8rem',
              color: 'var(--text-dim)',
              backgroundColor: 'var(--panel-bg)',
              padding: '0.25rem 0.5rem',
              borderRadius: '4px',
            }}
          >
            v{plugin.version}
          </span>
        </div>
        <div style={{ fontSize: '0.9rem', color: 'var(--text-dim)', marginBottom: '0.5rem' }}>
          by {plugin.author}
        </div>
        <div
          style={{
            fontSize: '0.85rem',
            color: 'var(--text)',
            lineHeight: 1.6,
          }}
        >
          {plugin.description || 'No description available.'}
        </div>
      </div>

      {plugin.commands.length > 0 && (
        <div style={{ marginTop: '1.5rem' }}>
          <h4
            style={{
              margin: '0 0 1rem 0',
              fontSize: '1rem',
              color: 'var(--text-dim)',
              textTransform: 'uppercase',
              letterSpacing: '0.05em',
            }}
          >
            Available Commands
          </h4>

          <div style={{ marginBottom: '1rem' }}>
            <label
              style={{
                display: 'block',
                marginBottom: '0.5rem',
                fontSize: '0.85rem',
                color: 'var(--text-dim)',
              }}
            >
              Select Command
            </label>
            <select
              value={selectedCommand}
              onChange={(e) => {
                setSelectedCommand(e.target.value);
                setCommandArgs({});
              }}
              style={{
                width: '100%',
                padding: '0.5rem',
                borderRadius: '6px',
                border: '1px solid var(--border)',
                backgroundColor: 'var(--panel-bg)',
                color: 'var(--text)',
                fontSize: '0.9rem',
              }}
            >
              <option value="">-- Select a command --</option>
              {plugin.commands.map((cmd) => (
                <option key={cmd.name} value={cmd.name}>
                  {cmd.name}
                </option>
              ))}
            </select>
          </div>

          {currentCommand && (
            <div
              style={{
                backgroundColor: 'var(--panel-bg)',
                borderRadius: '8px',
                padding: '1rem',
                marginBottom: '1rem',
              }}
            >
              <div
                style={{
                  fontSize: '0.85rem',
                  color: 'var(--text-dim)',
                  marginBottom: '1rem',
                }}
              >
                {currentCommand.description}
              </div>

              {currentCommand.args.length > 0 && (
                <div style={{ display: 'flex', flexDirection: 'column', gap: '0.75rem' }}>
                  {currentCommand.args.map((arg) => (
                    <div key={arg.name}>
                      <label
                        style={{
                          display: 'flex',
                          alignItems: 'center',
                          gap: '0.25rem',
                          marginBottom: '0.25rem',
                          fontSize: '0.8rem',
                          color: 'var(--text-dim)',
                        }}
                      >
                        {arg.name}
                        {arg.required && <span style={{ color: 'var(--accent)' }}>*</span>}
                        <span
                          style={{
                            fontSize: '0.7rem',
                            color: 'var(--text-dim)',
                            marginLeft: 'auto',
                            textTransform: 'uppercase',
                          }}
                        >
                          {arg.type}
                        </span>
                      </label>
                      <input
                        type={arg.type === 'boolean' ? 'checkbox' : 'text'}
                        checked={arg.type === 'boolean' ? commandArgs[arg.name] === 'true' : undefined}
                        value={arg.type !== 'boolean' ? commandArgs[arg.name] || '' : undefined}
                        placeholder={arg.default !== undefined ? String(arg.default) : undefined}
                        onChange={(e) =>
                          setCommandArgs({
                            ...commandArgs,
                            [arg.name]:
                              arg.type === 'boolean' ? String(e.target.checked) : e.target.value,
                          })
                        }
                        style={{
                          width: '100%',
                          padding: '0.375rem 0.5rem',
                          borderRadius: '4px',
                          border: '1px solid var(--border)',
                          backgroundColor: 'var(--glass)',
                          color: 'var(--text)',
                          fontSize: '0.85rem',
                          boxSizing: 'border-box',
                        }}
                      />
                      {arg.description && (
                        <div
                          style={{
                            fontSize: '0.75rem',
                            color: 'var(--text-dim)',
                            marginTop: '0.25rem',
                          }}
                        >
                          {arg.description}
                        </div>
                      )}
                    </div>
                  ))}
                </div>
              )}

              <button
                onClick={handleExecute}
                disabled={isExecuting || !selectedCommand}
                style={{
                  marginTop: '1rem',
                  width: '100%',
                  padding: '0.5rem 1rem',
                  backgroundColor: 'var(--blue)',
                  color: 'white',
                  border: 'none',
                  borderRadius: '6px',
                  cursor: isExecuting ? 'not-allowed' : 'pointer',
                  opacity: isExecuting ? 0.6 : 1,
                }}
              >
                {isExecuting ? '⏳ Executing...' : '▶ Execute Command'}
              </button>
            </div>
          )}
        </div>
      )}

      <div
        style={{
          marginTop: '1.5rem',
          paddingTop: '1rem',
          borderTop: '1px solid var(--border)',
          fontSize: '0.75rem',
          color: 'var(--text-dim)',
        }}
      >
        <div>ID: {plugin.id}</div>
        <div>Path: {plugin.path}</div>
        <div>Loaded: {new Date(plugin.loaded_at).toLocaleString()}</div>
      </div>
    </div>
  );
}

// ============================================================================
// Script Runner Component
// ============================================================================

interface ScriptRunnerProps {
  onRunScript: (script: string) => void;
  output: ScriptOutput | null;
  isRunning: boolean;
  error: string | null;
}

function ScriptRunner({ onRunScript, output, isRunning, error }: ScriptRunnerProps) {
  const [script, setScript] = useState<string>('-- Enter Lua script here\n-- Example: print("Hello, SPO!")');

  const handleRun = () => {
    onRunScript(script);
  };

  return (
    <div
      style={{
        backgroundColor: 'var(--glass)',
        borderRadius: '12px',
        border: '1px solid var(--border)',
        padding: '1.5rem',
      }}
    >
      <h4
        style={{
          margin: '0 0 1rem 0',
          fontSize: '1rem',
          color: 'var(--text-dim)',
          textTransform: 'uppercase',
          letterSpacing: '0.05em',
        }}
      >
        📝 Script Runner
      </h4>

      <textarea
        value={script}
        onChange={(e) => setScript(e.target.value)}
        rows={8}
        spellCheck={false}
        style={{
          width: '100%',
          padding: '0.75rem',
          borderRadius: '6px',
          border: '1px solid var(--border)',
          backgroundColor: 'var(--panel-bg)',
          color: 'var(--text)',
          fontFamily: 'Consolas, Monaco, "Courier New", monospace',
          fontSize: '0.85rem',
          resize: 'vertical',
          boxSizing: 'border-box',
          lineHeight: 1.5,
        }}
      />

      <div style={{ display: 'flex', justifyContent: 'flex-end', marginTop: '0.75rem' }}>
        <button
          onClick={handleRun}
          disabled={isRunning || !script.trim()}
          style={{
            padding: '0.5rem 1.25rem',
            backgroundColor: 'var(--blue)',
            color: 'white',
            border: 'none',
            borderRadius: '6px',
            cursor: isRunning ? 'not-allowed' : 'pointer',
            opacity: isRunning || !script.trim() ? 0.6 : 1,
            fontSize: '0.9rem',
          }}
        >
          {isRunning ? '⏳ Running...' : '▶ Run Script'}
        </button>
      </div>

      {(error || output) && (
        <div
          style={{
            marginTop: '1rem',
            padding: '1rem',
            borderRadius: '8px',
            backgroundColor: error
              ? 'rgba(220, 38, 38, 0.1)'
              : output?.success
                ? 'rgba(34, 197, 94, 0.1)'
                : 'rgba(251, 191, 36, 0.1)',
            border: `1px solid ${error ? 'var(--accent)' : output?.success ? '#22c55e' : '#fbbf24'}`,
          }}
        >
          {error && (
            <div style={{ color: 'var(--accent)', marginBottom: '0.5rem' }}>
              <strong>Error:</strong> {error}
            </div>
          )}

          {output && (
            <>
              <div
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: '0.5rem',
                  marginBottom: '0.5rem',
                  fontSize: '0.85rem',
                }}
              >
                <span style={{ color: output.success ? '#22c55e' : '#fbbf24' }}>
                  {output.success ? '✓ Success' : '⚠ Completed with errors'}
                </span>
                <span style={{ color: 'var(--text-dim)' }}>•</span>
                <span style={{ color: 'var(--text-dim)' }}>
                  {output.execution_time_ms.toFixed(1)}ms
                </span>
              </div>

              {output.output && (
                <pre
                  style={{
                    margin: 0,
                    padding: '0.75rem',
                    backgroundColor: 'var(--panel-bg)',
                    borderRadius: '4px',
                    fontFamily: 'Consolas, Monaco, "Courier New", monospace',
                    fontSize: '0.8rem',
                    overflow: 'auto',
                    maxHeight: '200px',
                    color: 'var(--text)',
                    whiteSpace: 'pre-wrap',
                    wordBreak: 'break-word',
                  }}
                >
                  {output.output}
                </pre>
              )}

              {output.error && (
                <pre
                  style={{
                    margin: '0.5rem 0 0 0',
                    padding: '0.75rem',
                    backgroundColor: 'rgba(220, 38, 38, 0.1)',
                    borderRadius: '4px',
                    fontFamily: 'Consolas, Monaco, "Courier New", monospace',
                    fontSize: '0.8rem',
                    color: 'var(--accent)',
                    overflow: 'auto',
                    maxHeight: '150px',
                    whiteSpace: 'pre-wrap',
                    wordBreak: 'break-word',
                  }}
                >
                  {output.error}
                </pre>
              )}
            </>
          )}
        </div>
      )}
    </div>
  );
}

// ============================================================================
// Batch Jobs Component
// ============================================================================

interface BatchJobsProps {
  jobs: BatchJob[];
  onCancelJob: (jobId: string) => void;
  isLoading: boolean;
}

function BatchJobs({ jobs, onCancelJob, isLoading }: BatchJobsProps) {
  const getStatusColor = (status: BatchJob['status']) => {
    switch (status) {
      case 'completed':
        return '#22c55e';
      case 'failed':
        return 'var(--accent)';
      case 'running':
        return 'var(--blue)';
      default:
        return 'var(--text-dim)';
    }
  };

  const getStatusIcon = (status: BatchJob['status']) => {
    switch (status) {
      case 'completed':
        return '✓';
      case 'failed':
        return '✗';
      case 'running':
        return '⏳';
      case 'pending':
        return '⏸';
      default:
        return '?';
    }
  };

  if (jobs.length === 0 && !isLoading) {
    return (
      <div
        style={{
          backgroundColor: 'var(--glass)',
          borderRadius: '12px',
          border: '1px solid var(--border)',
          padding: '1.5rem',
          textAlign: 'center',
          color: 'var(--text-dim)',
        }}
      >
        <div style={{ fontSize: '2rem', marginBottom: '0.5rem' }}>📋</div>
        <div>No batch jobs</div>
        <div style={{ fontSize: '0.85rem', marginTop: '0.25rem' }}>
          Jobs will appear here when plugins create them
        </div>
      </div>
    );
  }

  return (
    <div
      style={{
        backgroundColor: 'var(--glass)',
        borderRadius: '12px',
        border: '1px solid var(--border)',
        padding: '1.5rem',
      }}
    >
      <div
        style={{
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
          marginBottom: '1rem',
        }}
      >
        <h4
          style={{
            margin: 0,
            fontSize: '1rem',
            color: 'var(--text-dim)',
            textTransform: 'uppercase',
            letterSpacing: '0.05em',
          }}
        >
          📋 Batch Jobs
        </h4>
        {isLoading && <span style={{ fontSize: '0.85rem', color: 'var(--text-dim)' }}>Loading...</span>}
      </div>

      <div style={{ display: 'flex', flexDirection: 'column', gap: '0.75rem' }}>
        {jobs.map((job) => (
          <div
            key={job.id}
            style={{
              backgroundColor: 'var(--panel-bg)',
              borderRadius: '8px',
              padding: '1rem',
              border: '1px solid var(--border)',
            }}
          >
            <div
              style={{
                display: 'flex',
                justifyContent: 'space-between',
                alignItems: 'center',
                marginBottom: '0.5rem',
              }}
            >
              <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
                <span style={{ color: getStatusColor(job.status) }}>{getStatusIcon(job.status)}</span>
                <span style={{ fontWeight: 500 }}>{job.name}</span>
              </div>
              <div style={{ display: 'flex', alignItems: 'center', gap: '0.75rem' }}>
                <span
                  style={{
                    fontSize: '0.8rem',
                    color: getStatusColor(job.status),
                    textTransform: 'capitalize',
                  }}
                >
                  {job.status}
                </span>
                {(job.status === 'pending' || job.status === 'running') && (
                  <button
                    onClick={() => onCancelJob(job.id)}
                    style={{
                      padding: '0.25rem 0.5rem',
                      fontSize: '0.75rem',
                      backgroundColor: 'transparent',
                      border: '1px solid var(--border)',
                      color: 'var(--text-dim)',
                      cursor: 'pointer',
                      borderRadius: '4px',
                    }}
                  >
                    Cancel
                  </button>
                )}
              </div>
            </div>

            {(job.status === 'running' || job.status === 'pending') && job.total > 0 && (
              <div style={{ marginBottom: '0.5rem' }}>
                <div
                  style={{
                    height: '4px',
                    backgroundColor: 'var(--border)',
                    borderRadius: '2px',
                    overflow: 'hidden',
                  }}
                >
                  <div
                    style={{
                      height: '100%',
                      width: `${(job.progress / job.total) * 100}%`,
                      backgroundColor: 'var(--blue)',
                      borderRadius: '2px',
                      transition: 'width 0.3s ease',
                    }}
                  />
                </div>
                <div
                  style={{
                    fontSize: '0.75rem',
                    color: 'var(--text-dim)',
                    marginTop: '0.25rem',
                  }}
                >
                  {job.progress} / {job.total}
                  {job.current_item && ` • ${job.current_item}`}
                </div>
              </div>
            )}

            {job.error && (
              <div
                style={{
                  fontSize: '0.8rem',
                  color: 'var(--accent)',
                  marginTop: '0.5rem',
                  padding: '0.5rem',
                  backgroundColor: 'rgba(220, 38, 38, 0.1)',
                  borderRadius: '4px',
                }}
              >
                {job.error}
              </div>
            )}

            {(job.started_at || job.completed_at) && (
              <div
                style={{
                  fontSize: '0.75rem',
                  color: 'var(--text-dim)',
                  marginTop: '0.5rem',
                  display: 'flex',
                  gap: '1rem',
                }}
              >
                {job.started_at && <span>Started: {new Date(job.started_at).toLocaleTimeString()}</span>}
                {job.completed_at && (
                  <span>Completed: {new Date(job.completed_at).toLocaleTimeString()}</span>
                )}
              </div>
            )}
          </div>
        ))}
      </div>
    </div>
  );
}

// ============================================================================
// Main Plugin Manager Component
// ============================================================================

interface PluginManagerProps {
  isOpen?: boolean;
  onClose?: () => void;
}

export function PluginManager({ isOpen = true, onClose }: PluginManagerProps) {
  // Hooks (stub implementations to be replaced)
  const { plugins, isLoading: isLoadingPlugins, error: pluginsError, refetch } = usePlugins();
  const { loadPlugin, isLoading: isLoadingPlugin, error: loadError } = useLoadPlugin();
  const { unloadPlugin, isLoading: isUnloading } = useUnloadPlugin();
  const { enablePlugin, isLoading: isEnabling } = useEnablePlugin();
  const { disablePlugin, isLoading: isDisabling } = useDisablePlugin();
  const { executeCommand, isLoading: isExecuting, error: executeError, result } =
    useExecutePluginCommand();
  const { runScript, isLoading: isRunningScript, error: scriptError, output } = useRunScript();
  const { jobs, isLoading: isLoadingJobs, error: jobsError, cancelJob } = useBatchJobs();

  // Local state
  const [selectedPluginId, setSelectedPluginId] = useState<string | null>(null);
  const [togglingPluginId, setTogglingPluginId] = useState<string | null>(null);
  const [unloadingPluginId, setUnloadingPluginId] = useState<string | null>(null);

  const selectedPlugin = plugins.find((p) => p.id === selectedPluginId) || null;

  // Combined error state
  const error = pluginsError || loadError || executeError || scriptError || jobsError;

  // Handlers
  const handleLoadPlugin = useCallback(async () => {
    const selected = await open({
      multiple: false,
      filters: [
        { name: 'Lua Plugin', extensions: ['lua'] },
        { name: 'All Files', extensions: ['*'] },
      ],
    });

    if (typeof selected === 'string') {
      try {
        await loadPlugin(selected);
        refetch();
      } catch (e) {
        console.error('Failed to load plugin:', e);
      }
    }
  }, [loadPlugin, refetch]);

  const handleToggleEnabled = useCallback(
    async (plugin: Plugin) => {
      setTogglingPluginId(plugin.id);
      try {
        if (plugin.enabled) {
          await disablePlugin(plugin.id);
        } else {
          await enablePlugin(plugin.id);
        }
        refetch();
      } catch (e) {
        console.error('Failed to toggle plugin:', e);
      } finally {
        setTogglingPluginId(null);
      }
    },
    [disablePlugin, enablePlugin, refetch]
  );

  const handleUnload = useCallback(
    async (pluginId: string) => {
      setUnloadingPluginId(pluginId);
      try {
        await unloadPlugin(pluginId);
        if (selectedPluginId === pluginId) {
          setSelectedPluginId(null);
        }
        refetch();
      } catch (e) {
        console.error('Failed to unload plugin:', e);
      } finally {
        setUnloadingPluginId(null);
      }
    },
    [unloadPlugin, selectedPluginId, refetch]
  );

  const handleExecuteCommand = useCallback(
    async (command: string, args: Record<string, unknown>) => {
      if (!selectedPlugin) return;
      try {
        await executeCommand(selectedPlugin.id, command, args);
      } catch (e) {
        console.error('Failed to execute command:', e);
      }
    },
    [executeCommand, selectedPlugin]
  );

  const handleRunScript = useCallback(
    async (script: string) => {
      try {
        await runScript(script);
      } catch (e) {
        console.error('Failed to run script:', e);
      }
    },
    [runScript]
  );

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
          width: '100%',
          maxWidth: '1100px',
          maxHeight: '90vh',
          height: '80vh',
          overflow: 'hidden',
          display: 'flex',
          flexDirection: 'column',
        }}
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div
          style={{
            padding: '1.5rem',
            borderBottom: '1px solid var(--border)',
            display: 'flex',
            justifyContent: 'space-between',
            alignItems: 'center',
          }}
        >
          <h2 style={{ margin: 0, display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
            🔌 Plugin Manager
          </h2>
          <button
            onClick={onClose}
            style={{
              padding: '0.5rem 1rem',
              backgroundColor: 'transparent',
              border: '1px solid var(--border)',
              borderRadius: '6px',
              cursor: 'pointer',
              color: 'var(--text)',
            }}
          >
            Close
          </button>
        </div>

        {/* Error Display */}
        {error && (
          <div
            style={{
              padding: '1rem',
              backgroundColor: 'rgba(220, 38, 38, 0.2)',
              borderBottom: '1px solid var(--border)',
              color: 'var(--accent)',
            }}
          >
            ⚠️ {error}
          </div>
        )}

        {/* Main Content */}
        <div
          style={{
            flex: 1,
            overflow: 'hidden',
            display: 'grid',
            gridTemplateColumns: '320px 1fr',
            gap: '1.5rem',
            padding: '1.5rem',
          }}
        >
          {/* Left Panel - Plugin List */}
          <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem', overflow: 'hidden' }}>
            {/* Load Plugin Button */}
            <button
              onClick={handleLoadPlugin}
              disabled={isLoadingPlugin}
              style={{
                padding: '0.75rem 1rem',
                backgroundColor: 'var(--blue)',
                color: 'white',
                border: 'none',
                borderRadius: '8px',
                cursor: isLoadingPlugin ? 'not-allowed' : 'pointer',
                opacity: isLoadingPlugin ? 0.6 : 1,
                fontSize: '0.9rem',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                gap: '0.5rem',
              }}
            >
              {isLoadingPlugin ? '⏳ Loading...' : '📂 Load Plugin...'}
            </button>

            {/* Plugin List */}
            <div
              style={{
                flex: 1,
                overflow: 'auto',
                display: 'flex',
                flexDirection: 'column',
                gap: '0.75rem',
              }}
            >
              {isLoadingPlugins ? (
                <div
                  style={{
                    textAlign: 'center',
                    padding: '2rem',
                    color: 'var(--text-dim)',
                  }}
                >
                  Loading plugins...
                </div>
              ) : plugins.length === 0 ? (
                <div
                  style={{
                    textAlign: 'center',
                    padding: '2rem',
                    color: 'var(--text-dim)',
                    backgroundColor: 'var(--glass)',
                    borderRadius: '8px',
                  }}
                >
                  <div style={{ fontSize: '2rem', marginBottom: '0.5rem' }}>🔌</div>
                  <div>No plugins loaded</div>
                  <div style={{ fontSize: '0.85rem', marginTop: '0.25rem' }}>
                    Click "Load Plugin" to add .lua files
                  </div>
                </div>
              ) : (
                plugins.map((plugin) => (
                  <PluginListItem
                    key={plugin.id}
                    plugin={plugin}
                    isSelected={selectedPluginId === plugin.id}
                    onSelect={() => setSelectedPluginId(plugin.id)}
                    onToggleEnabled={() => handleToggleEnabled(plugin)}
                    onUnload={() => handleUnload(plugin.id)}
                    isToggling={togglingPluginId === plugin.id}
                    isUnloading={unloadingPluginId === plugin.id}
                  />
                ))
              )}
            </div>
          </div>

          {/* Right Panel - Details / Script Runner / Batch Jobs */}
          <div
            style={{
              overflow: 'auto',
              display: 'flex',
              flexDirection: 'column',
              gap: '1rem',
            }}
          >
            {selectedPlugin ? (
              <>
                <PluginDetails
                  plugin={selectedPlugin}
                  onExecuteCommand={handleExecuteCommand}
                  isExecuting={isExecuting}
                />

                {result && (
                  <div
                    style={{
                      backgroundColor: 'var(--glass)',
                      borderRadius: '12px',
                      border: '1px solid var(--border)',
                      padding: '1rem',
                    }}
                  >
                    <h5
                      style={{
                        margin: '0 0 0.5rem 0',
                        fontSize: '0.85rem',
                        color: 'var(--text-dim)',
                      }}
                    >
                      Last Result
                    </h5>
                    <pre
                      style={{
                        margin: 0,
                        padding: '0.75rem',
                        backgroundColor: 'var(--panel-bg)',
                        borderRadius: '6px',
                        fontFamily: 'Consolas, Monaco, "Courier New", monospace',
                        fontSize: '0.8rem',
                        overflow: 'auto',
                        maxHeight: '150px',
                        color: 'var(--text)',
                      }}
                    >
                      {JSON.stringify(result, null, 2)}
                    </pre>
                  </div>
                )}
              </>
            ) : (
              <div
                style={{
                  backgroundColor: 'var(--glass)',
                  borderRadius: '12px',
                  border: '1px solid var(--border)',
                  padding: '2rem',
                  textAlign: 'center',
                  color: 'var(--text-dim)',
                  flex: 1,
                  display: 'flex',
                  flexDirection: 'column',
                  justifyContent: 'center',
                }}
              >
                <div style={{ fontSize: '3rem', marginBottom: '1rem' }}>🔌</div>
                <div style={{ fontSize: '1.1rem', marginBottom: '0.5rem' }}>No Plugin Selected</div>
                <div>Select a plugin from the list to view details and execute commands</div>
              </div>
            )}

            <ScriptRunner
              onRunScript={handleRunScript}
              output={output}
              isRunning={isRunningScript}
              error={scriptError}
            />

            <BatchJobs jobs={jobs} onCancelJob={cancelJob} isLoading={isLoadingJobs} />
          </div>
        </div>
      </div>
    </div>
  );
}

export default PluginManager;
