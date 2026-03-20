import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { save } from '@tauri-apps/plugin-dialog';
import { useStore } from '../store/useStore';
import { ChangeStats } from './ChangeStats';
import { DetailedAssetReport } from './DetailedAssetReport';

export type OutputFormat = 'markdown' | 'html' | 'text' | 'json' | 'bbcode';
type ViewTab = 'summary' | 'detailed';

interface PatchNotesData {
  title: string;
  author: string;
  version: string;
  date: string;
  base_rom_sha1: string;
  summary: {
    total_boxers_modified: number;
    total_palettes_changed: number;
    total_sprites_edited: number;
    total_animations_modified: number;
    total_headers_edited: number;
    total_changes: number;
  };
  boxer_changes: Array<{
    boxer_name: string;
    boxer_key: string;
    changes: Array<{
      type: string;
      name?: string;
      description?: string;
      colors_changed?: number;
      tiles_modified?: number;
      field?: string;
      before?: string;
      after?: string;
    }>;
  }>;
  system_changes: Array<{
    category: string;
    description: string;
  }>;
}

const formatOptions: { value: OutputFormat; label: string; extension: string; description: string }[] = [
  { value: 'markdown', label: 'Markdown', extension: 'md', description: 'GitHub-friendly format' },
  { value: 'html', label: 'HTML', extension: 'html', description: 'Web-ready format' },
  { value: 'text', label: 'Plain Text', extension: 'txt', description: 'Simple text format' },
  { value: 'json', label: 'JSON', extension: 'json', description: 'Machine-readable data' },
  { value: 'bbcode', label: 'BBCode', extension: 'bb', description: 'Forum post format' },
];

export const PatchNotesGenerator = () => {
  const { currentProject, pendingWrites, romSha1 } = useStore();
  
  const [title, setTitle] = useState('');
  const [author, setAuthor] = useState('');
  const [version, setVersion] = useState('1.0.0');
  const [format, setFormat] = useState<OutputFormat>('markdown');
  const [includePreviews, setIncludePreviews] = useState(true);
  const [generatedContent, setGeneratedContent] = useState('');
  const [isGenerating, setIsGenerating] = useState(false);
  const [status, setStatus] = useState<string | null>(null);
  const [refreshTrigger, setRefreshTrigger] = useState(0);
  const [patchNotesData, setPatchNotesData] = useState<PatchNotesData | null>(null);
  const [activeView, setActiveView] = useState<ViewTab>('summary');

  // Initialize form with project metadata
  useEffect(() => {
    if (currentProject) {
      setTitle(currentProject.metadata.name || 'Untitled Mod');
      setAuthor(currentProject.metadata.author || '');
      setVersion(currentProject.metadata.version || '1.0.0');
    }
  }, [currentProject]);

  // Trigger refresh when pending writes change
  useEffect(() => {
    setRefreshTrigger(prev => prev + 1);
    loadPatchNotesData();
  }, [pendingWrites.size]);

  const loadPatchNotesData = useCallback(async () => {
    if (!romSha1) return;
    
    try {
      const data = await invoke<PatchNotesData>('get_patch_notes_data', {
        title: title || undefined,
        author: author || undefined,
        version: version || undefined,
      });
      setPatchNotesData(data);
    } catch (e) {
      console.error('Failed to load patch notes data:', e);
    }
  }, [romSha1, title, author, version]);

  useEffect(() => {
    loadPatchNotesData();
  }, [loadPatchNotesData]);

  const handleGenerate = async () => {
    if (!romSha1) {
      setStatus('Error: No ROM loaded');
      return;
    }

    setIsGenerating(true);
    setStatus(null);

    try {
      const content = await invoke<string>('generate_patch_notes', {
        format,
        title: title || undefined,
        author: author || undefined,
        version: version || undefined,
      });
      setGeneratedContent(content);
      setStatus('✓ Patch notes generated');
    } catch (e) {
      console.error('Failed to generate patch notes:', e);
      setStatus(`✗ Error: ${e}`);
    } finally {
      setIsGenerating(false);
    }
  };

  const handleCopyToClipboard = async () => {
    if (!generatedContent) return;

    try {
      await navigator.clipboard.writeText(generatedContent);
      setStatus('✓ Copied to clipboard');
      setTimeout(() => setStatus(null), 2000);
    } catch (e) {
      setStatus('✗ Failed to copy');
    }
  };

  const handleSaveToFile = async () => {
    if (!generatedContent) return;

    const selectedFormat = formatOptions.find(f => f.value === format);
    const extension = selectedFormat?.extension || 'md';

    const path = await save({
      filters: [{
        name: selectedFormat?.label || 'Patch Notes',
        extensions: [extension],
      }],
      defaultPath: `patch_notes_${version}.${extension}`,
    });

    if (!path) return;

    try {
      await invoke('save_patch_notes', {
        content: generatedContent,
        outputPath: path,
      });
      setStatus(`✓ Saved to ${path.split(/[\\/]/).pop()}`);
    } catch (e) {
      setStatus(`✗ Error: ${e}`);
    }
  };

  const handleExportWithPatch = async () => {
    if (!romSha1 || pendingWrites.size === 0) {
      setStatus('Error: No changes to export');
      return;
    }

    const patchPath = await save({
      filters: [{ name: 'IPS Patch', extensions: ['ips'] }],
      defaultPath: `mod_${version}.ips`,
    });

    if (!patchPath) return;

    const notesPath = patchPath.replace('.ips', '_notes.md');

    try {
      await invoke('export_patch_notes_with_patch', {
        patchPath,
        notesPath,
        format,
        title: title || undefined,
        author: author || undefined,
        version: version || undefined,
      });
      setStatus(`✓ Exported patch and notes`);
    } catch (e) {
      setStatus(`✗ Error: ${e}`);
    }
  };

  const hasChanges = pendingWrites.size > 0 || (patchNotesData?.summary.total_changes ?? 0) > 0;

  return (
    <div style={{
      backgroundColor: 'var(--panel-bg)',
      border: '1px solid var(--border)',
      borderRadius: '12px',
      padding: '1.5rem',
    }}>
      {/* Header */}
      <div style={{
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
        marginBottom: '1.5rem',
      }}>
        <h3 style={{ margin: 0 }}>📝 Patch Notes & Asset Report</h3>
        <div style={{ display: 'flex', gap: '0.5rem' }}>
          <button
            onClick={handleGenerate}
            disabled={!romSha1 || isGenerating}
            style={{
              padding: '8px 16px',
              backgroundColor: romSha1 ? 'var(--blue)' : 'var(--border)',
              opacity: isGenerating ? 0.6 : 1,
              cursor: romSha1 ? 'pointer' : 'not-allowed',
              border: 'none',
              borderRadius: '6px',
              fontWeight: 600,
              color: 'white',
            }}
          >
            {isGenerating ? '⏳ Generating...' : 'Generate'}
          </button>
        </div>
      </div>

      {/* View Tabs */}
      <div style={{
        display: 'flex',
        gap: '0.5rem',
        marginBottom: '1.5rem',
        borderBottom: '1px solid var(--border)',
        paddingBottom: '0.5rem',
      }}>
        <button
          onClick={() => setActiveView('summary')}
          style={{
            padding: '8px 16px',
            backgroundColor: activeView === 'summary' ? 'var(--blue)' : 'transparent',
            border: 'none',
            borderRadius: '6px',
            color: activeView === 'summary' ? 'white' : 'var(--text)',
            fontWeight: 500,
            cursor: 'pointer',
          }}
        >
          Summary Report
        </button>
        <button
          onClick={() => setActiveView('detailed')}
          style={{
            padding: '8px 16px',
            backgroundColor: activeView === 'detailed' ? 'var(--blue)' : 'transparent',
            border: 'none',
            borderRadius: '6px',
            color: activeView === 'detailed' ? 'white' : 'var(--text)',
            fontWeight: 500,
            cursor: 'pointer',
          }}
        >
          Detailed Asset Report
        </button>
      </div>

      {/* Stats Dashboard */}
      <div style={{ marginBottom: '1.5rem' }}>
        <ChangeStats refreshTrigger={refreshTrigger} />
      </div>

      {/* Conditional Content Based on View */}
      {activeView === 'detailed' ? (
        <DetailedAssetReport />
      ) : (
        <>
          {/* Metadata Form */}
      <div style={{
        display: 'grid',
        gap: '1rem',
        marginBottom: '1.5rem',
        padding: '1rem',
        backgroundColor: 'var(--glass)',
        borderRadius: '8px',
      }}>
        <div>
          <label style={{
            display: 'block',
            fontSize: '0.75rem',
            color: 'var(--text-dim)',
            marginBottom: '0.25rem',
          }}>
            Mod Title
          </label>
          <input
            type="text"
            value={title}
            onChange={(e) => setTitle(e.target.value)}
            placeholder="My SPO Mod"
            style={{
              width: '100%',
              padding: '8px 12px',
              backgroundColor: 'var(--panel-bg)',
              border: '1px solid var(--border)',
              borderRadius: '6px',
              color: 'var(--text)',
              fontSize: '0.9rem',
              boxSizing: 'border-box',
            }}
          />
        </div>

        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '1rem' }}>
          <div>
            <label style={{
              display: 'block',
              fontSize: '0.75rem',
              color: 'var(--text-dim)',
              marginBottom: '0.25rem',
            }}>
              Author
            </label>
            <input
              type="text"
              value={author}
              onChange={(e) => setAuthor(e.target.value)}
              placeholder="Your Name"
              style={{
                width: '100%',
                padding: '8px 12px',
                backgroundColor: 'var(--panel-bg)',
                border: '1px solid var(--border)',
                borderRadius: '6px',
                color: 'var(--text)',
                fontSize: '0.9rem',
                boxSizing: 'border-box',
              }}
            />
          </div>
          <div>
            <label style={{
              display: 'block',
              fontSize: '0.75rem',
              color: 'var(--text-dim)',
              marginBottom: '0.25rem',
            }}>
              Version
            </label>
            <input
              type="text"
              value={version}
              onChange={(e) => setVersion(e.target.value)}
              placeholder="1.0.0"
              style={{
                width: '100%',
                padding: '8px 12px',
                backgroundColor: 'var(--panel-bg)',
                border: '1px solid var(--border)',
                borderRadius: '6px',
                color: 'var(--text)',
                fontSize: '0.9rem',
                boxSizing: 'border-box',
              }}
            />
          </div>
        </div>

        {/* Format Selection */}
        <div>
          <label style={{
            display: 'block',
            fontSize: '0.75rem',
            color: 'var(--text-dim)',
            marginBottom: '0.5rem',
          }}>
            Output Format
          </label>
          <div style={{ display: 'flex', flexWrap: 'wrap', gap: '0.5rem' }}>
            {formatOptions.map((opt) => (
              <button
                key={opt.value}
                onClick={() => setFormat(opt.value)}
                style={{
                  padding: '6px 12px',
                  fontSize: '0.8rem',
                  backgroundColor: format === opt.value ? 'var(--blue)' : 'var(--panel-bg)',
                  border: `1px solid ${format === opt.value ? 'var(--blue)' : 'var(--border)'}`,
                  borderRadius: '4px',
                  cursor: 'pointer',
                  color: format === opt.value ? 'white' : 'var(--text)',
                }}
                title={opt.description}
              >
                {opt.label}
              </button>
            ))}
          </div>
        </div>

        {/* Options */}
        <label style={{
          display: 'flex',
          alignItems: 'center',
          gap: '0.5rem',
          fontSize: '0.85rem',
          cursor: 'pointer',
        }}>
          <input
            type="checkbox"
            checked={includePreviews}
            onChange={(e) => setIncludePreviews(e.target.checked)}
          />
          Include visual previews (when available)
        </label>
      </div>

      {/* Preview */}
      {generatedContent && (
        <div style={{ marginBottom: '1.5rem' }}>
          <label style={{
            display: 'block',
            fontSize: '0.75rem',
            color: 'var(--text-dim)',
            marginBottom: '0.5rem',
          }}>
            Preview
          </label>
          <pre style={{
            backgroundColor: 'var(--glass)',
            border: '1px solid var(--border)',
            borderRadius: '8px',
            padding: '1rem',
            maxHeight: '300px',
            overflow: 'auto',
            fontSize: '0.8rem',
            lineHeight: 1.5,
            whiteSpace: 'pre-wrap',
            wordBreak: 'break-word',
            margin: 0,
          }}>
            {generatedContent}
          </pre>
        </div>
      )}

      {/* Action Buttons */}
      <div style={{
        display: 'flex',
        gap: '0.75rem',
        flexWrap: 'wrap',
      }}>
        <button
          onClick={handleCopyToClipboard}
          disabled={!generatedContent}
          style={{
            flex: 1,
            minWidth: '140px',
            padding: '10px 16px',
            backgroundColor: generatedContent ? 'var(--glass)' : 'var(--border)',
            border: '1px solid var(--border)',
            borderRadius: '6px',
            cursor: generatedContent ? 'pointer' : 'not-allowed',
            color: 'var(--text)',
            fontSize: '0.85rem',
            opacity: generatedContent ? 1 : 0.5,
          }}
        >
          📋 Copy to Clipboard
        </button>
        <button
          onClick={handleSaveToFile}
          disabled={!generatedContent}
          style={{
            flex: 1,
            minWidth: '140px',
            padding: '10px 16px',
            backgroundColor: generatedContent ? 'var(--blue)' : 'var(--border)',
            border: 'none',
            borderRadius: '6px',
            cursor: generatedContent ? 'pointer' : 'not-allowed',
            color: 'white',
            fontSize: '0.85rem',
            opacity: generatedContent ? 1 : 0.5,
          }}
        >
          💾 Save to File
        </button>
        <button
          onClick={handleExportWithPatch}
          disabled={!hasChanges}
          style={{
            flex: 1,
            minWidth: '140px',
            padding: '10px 16px',
            backgroundColor: hasChanges ? '#10b981' : 'var(--border)',
            border: 'none',
            borderRadius: '6px',
            cursor: hasChanges ? 'pointer' : 'not-allowed',
            color: 'white',
            fontSize: '0.85rem',
            opacity: hasChanges ? 1 : 0.5,
          }}
        >
          📦 Export with Patch
        </button>
      </div>

      {/* Status */}
      {status && (
        <div style={{
          marginTop: '1rem',
          padding: '10px 14px',
          borderRadius: '8px',
          background: status.startsWith('✓') ? 'rgba(107,219,125,0.1)' : 'rgba(255,80,80,0.1)',
          border: `1px solid ${status.startsWith('✓') ? 'rgba(107,219,125,0.3)' : 'rgba(255,80,80,0.3)'}`,
          color: status.startsWith('✓') ? '#6bdb7d' : '#ff6666',
          fontSize: '0.85rem',
          fontFamily: 'monospace',
        }}>
          {status}
        </div>
      )}

      {/* Help Text */}
      <div style={{
        marginTop: '1rem',
        padding: '1rem',
        backgroundColor: 'rgba(59,130,246,0.1)',
        borderRadius: '8px',
        fontSize: '0.8rem',
        color: 'var(--text-dim)',
        lineHeight: 1.5,
      }}>
        <strong style={{ color: 'var(--blue)' }}>Tip:</strong> Patch notes are automatically generated from your edit history and pending changes. The generator tracks palette changes, sprite edits, animation adjustments, and stat modifications across all boxers.
      </div>
        </>
      )}
    </div>
  );
};

export default PatchNotesGenerator;
