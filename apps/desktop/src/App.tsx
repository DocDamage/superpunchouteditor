import { useEffect, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { useStore } from "./store/useStore";
import { ThemeProvider } from "./context/ThemeProvider";
import { ThemeToggle } from "./components/ThemeToggle";
import "./App.css";

import { RegionSelector, RegionDetectionResult } from "./components/RegionSelector";
import { PaletteEditor } from "./components/PaletteEditor";
import { AssetManager } from "./components/AssetManager";
import { FighterViewer } from "./components/FighterViewer";
import { SpriteBinEditor } from "./components/SpriteBinEditor";
import { ExportPanel } from "./components/ExportPanel";
import { BoxerPreviewSheet } from "./components/BoxerPreviewSheet";
import { ScriptViewer } from "./components/ScriptViewer";
import { ProjectManager } from "./components/ProjectManager";
import { FrameReconstructor } from "./components/FrameReconstructor";
import { PatchNotesGenerator } from "./components/PatchNotesGenerator";
import { EmulatorSettings } from "./components/EmulatorSettings";
import { TestInEmulatorButton } from "./components/TestInEmulatorButton";
import { AnimationEditor } from "./components/AnimationEditor";
import { BoxerCompare } from "./components/BoxerCompare";
import { ComparisonView } from "./components/ComparisonView";
import { AIEditor } from "./components/AIEditor";
import { LayoutPackManager } from "./components/LayoutPackManager";
import { LayoutPackBrowser } from "./components/LayoutPackBrowser";
import { ExternalToolsManager } from "./components/ExternalToolsManager";
import { RosterEditor } from "./components/RosterEditor";
import { PluginManager } from "./components/PluginManager";
import { BankVisualization } from "./components/BankVisualization";
import { AnimationPlayer } from "./components/AnimationPlayer";

import { HelpButton, KeyboardShortcutsHelp, HelpSystem } from "./components/help";
import { UpdateSettings } from "./components/UpdateSettings";
import { UpdateChecker } from "./components/UpdateChecker";
import { EmbeddedEmulator } from "./components/EmbeddedEmulator";
import "./styles/emulator.css";


function App() {
  const { 
    romSha1, 
    boxers, 
    selectedBoxer, 
    currentProject,
    canUndo,
    canRedo,
    undoStack,
    pendingWrites,
    loadBoxers, 
    openRom, 
    selectBoxer,
    getCurrentProject,
    undo,
    redo,
    refreshUndoState,
    clearHistory,
    error,
  } = useStore();

  const [showEmulatorSettings, setShowEmulatorSettings] = useState(false);
  const [showExternalTools, setShowExternalTools] = useState(false);
  const [showKeyboardShortcuts, setShowKeyboardShortcuts] = useState(false);
  const [showHelp, setShowHelp] = useState(false);
  const [helpContext, setHelpContext] = useState<string | undefined>(undefined);
  const [showRegionSelector, setShowRegionSelector] = useState(false);
  const [detectedRegion, setDetectedRegion] = useState<RegionDetectionResult | null>(null);
  const [romPath, setRomPath] = useState<string>('');

  const [currentTab, setCurrentTab] = useState<'editor' | 'viewer' | 'scripts' | 'project' | 'animations' | 'compare' | 'frames' | 'packs' | 'roster' | 'ai' | 'test' | 'settings' | 'plugins' | 'banks' | 'animation-player'>('editor');

  useEffect(() => {
    loadBoxers();
    getCurrentProject();
  }, [loadBoxers, getCurrentProject]);

  // Refresh undo state periodically and on mount
  useEffect(() => {
    refreshUndoState();
    const interval = setInterval(refreshUndoState, 1000);
    return () => clearInterval(interval);
  }, [refreshUndoState]);

  // Clear history when ROM changes
  useEffect(() => {
    if (romSha1) {
      clearHistory();
    }
  }, [romSha1]);

  // Keyboard shortcuts for undo/redo
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Ctrl+Z = Undo
      if ((e.ctrlKey || e.metaKey) && e.key === 'z' && !e.shiftKey) {
        e.preventDefault();
        if (canUndo) {
          undo();
        }
      }
      // Ctrl+Shift+Z = Redo
      else if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key === 'z') {
        e.preventDefault();
        if (canRedo) {
          redo();
        }
      }
      // Ctrl+Y = Redo (alternative)
      else if ((e.ctrlKey || e.metaKey) && e.key === 'y') {
        e.preventDefault();
        if (canRedo) {
          redo();
        }
      }
      // F1 = Open Help
      else if (e.key === 'F1') {
        e.preventDefault();
        setShowHelp(true);
        setHelpContext(currentTab === 'editor' ? 'palette-editor' : currentTab);
      }
      // Number keys 1-0 for tab switching
      else if ((e.ctrlKey || e.metaKey) && e.key === '7') {
        e.preventDefault();
        setCurrentTab('plugins');
      }
      else if ((e.ctrlKey || e.metaKey) && e.key === '8') {
        e.preventDefault();
        setCurrentTab('banks');
      }
      else if ((e.ctrlKey || e.metaKey) && e.key === '9') {
        e.preventDefault();
        setCurrentTab('animation-player');
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [canUndo, canRedo, undo, redo, setCurrentTab, currentTab]);

  const handleOpenRom = async () => {
    try {
      const selected = await open({
        multiple: false,
        filters: [{
          name: 'SFC ROM',
          extensions: ['sfc', 'smc']
        }]
      });
      if (typeof selected === 'string') {
        setRomPath(selected);
        setShowRegionSelector(true);
      }
    } catch (e) {
      console.error(e);
    }
  };

  const handleRegionDetected = (result: RegionDetectionResult) => {
    setDetectedRegion(result);
  };

  const handleRegionSelected = async (region: string) => {
    if (romPath) {
      try {
        await openRom(romPath);
        setShowRegionSelector(false);
      } catch (e) {
        console.error(e);
      }
    }
  };

  return (
    <div className="app-container">
      <aside className="sidebar">
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '1rem' }}>
          <h1 style={{ margin: 0 }}>SPO!! Editor</h1>
          <ThemeToggle variant="minimal" size="small" />
        </div>
        
        <div className="header-bar" style={{ display: 'flex', gap: '0.5rem', marginBottom: '1rem', flexWrap: 'wrap' }}>
          <button onClick={handleOpenRom} style={{ flex: 1, minWidth: '100px' }}>
            {romSha1 ? "Switch ROM" : "Open ROM"}
          </button>
          {romSha1 && (
            <>
              <HelpButton 
                context="rom-loading" 
                size="small"
                style={{ padding: '0.5rem' }}
              />
              <button 
                onClick={undo} 
                disabled={!canUndo}
                title="Undo (Ctrl+Z)"
                style={{ 
                  padding: '0.5rem 0.75rem',
                  opacity: canUndo ? 1 : 0.5,
                  cursor: canUndo ? 'pointer' : 'not-allowed',
                }}
              >
                ↶ Undo
              </button>
              <button 
                onClick={redo} 
                disabled={!canRedo}
                title="Redo (Ctrl+Shift+Z or Ctrl+Y)"
                style={{ 
                  padding: '0.5rem 0.75rem',
                  opacity: canRedo ? 1 : 0.5,
                  cursor: canRedo ? 'pointer' : 'not-allowed',
                }}
              >
                ↷ Redo
              </button>
              <TestInEmulatorButton
                romSha1={romSha1}
                selectedBoxerKey={selectedBoxer?.key || null}
                selectedBoxerName={selectedBoxer?.name || null}
                pendingWritesCount={pendingWrites.size}
                disabled={false}
              />
            </>
          )}
        </div>
        
        {/* Edit History Indicator */}
        {romSha1 && undoStack.length > 0 && (
          <div 
            className="status-badge" 
            style={{ 
              marginBottom: '0.5rem',
              backgroundColor: 'var(--accent)',
              color: 'white',
              fontSize: '0.75rem',
              padding: '0.25rem 0.5rem',
            }}
            title={`${undoStack.length} edit${undoStack.length === 1 ? '' : 's'} in history`}
          >
            {undoStack.length} edit{undoStack.length === 1 ? '' : 's'} • Ctrl+Z to undo
          </div>
        )}
        
        {romSha1 && (
          <div className="status-badge active" style={{ marginBottom: "0.5rem" }}>
            ROM OK: {romSha1.substring(0, 8)}...
            {detectedRegion?.display_name && (
              <span style={{ 
                marginLeft: '0.5rem', 
                padding: '0.125rem 0.375rem',
                backgroundColor: detectedRegion.is_supported ? 'var(--success)' : 'var(--warning)',
                borderRadius: '3px',
                fontSize: '0.7rem',
                fontWeight: 'bold',
                color: 'white',
              }}>
                {detectedRegion.display_name.split('(')[1]?.replace(')', '') || 'USA'}
              </span>
            )}
          </div>
        )}

        {romSha1 && detectedRegion && !detectedRegion.is_supported && (
          <div style={{ 
            marginBottom: '0.5rem',
            padding: '0.5rem',
            backgroundColor: 'var(--warning-bg, rgba(234, 179, 8, 0.1))',
            border: '1px solid var(--warning)',
            borderRadius: '4px',
            fontSize: '0.75rem',
            color: 'var(--warning)',
          }}>
            ⚠️ {detectedRegion.display_name || 'Unknown ROM'} - Limited Support
          </div>
        )}

        {currentProject && (
          <div 
            className="status-badge" 
            style={{ 
              marginBottom: "1rem",
              backgroundColor: 'var(--info)',
              color: 'white',
            }}
          >
            Project: {currentProject.metadata.name}
          </div>
        )}

        {error && <div style={{ color: "var(--error)", fontSize: "0.8rem", marginBottom: "1rem" }}>{error}</div>}
        
        <div className="nav-tabs" style={{ display: 'flex', gap: '0.25rem', marginBottom: '1rem', flexWrap: 'wrap' }}>
          <button 
            className={`tab-btn ${currentTab === 'editor' ? 'active' : ''}`}
            onClick={() => setCurrentTab('editor')}
            style={{ 
              flex: 1, 
              padding: '0.5rem', 
              borderRadius: '4px', 
              border: '1px solid var(--border)', 
              background: currentTab === 'editor' ? 'var(--info)' : 'transparent',
              minWidth: '60px'
            }}
          >
            Editor
          </button>
          <button 
            className={`tab-btn ${currentTab === 'viewer' ? 'active' : ''}`}
            onClick={() => setCurrentTab('viewer')}
            style={{ 
              flex: 1, 
              padding: '0.5rem', 
              borderRadius: '4px', 
              border: '1px solid var(--border)', 
              background: currentTab === 'viewer' ? 'var(--info)' : 'transparent',
              minWidth: '60px'
            }}
          >
            Viewer
          </button>
          <button 
            className={`tab-btn ${currentTab === 'scripts' ? 'active' : ''}`}
            onClick={() => setCurrentTab('scripts')}
            style={{ 
              flex: 1, 
              padding: '0.5rem', 
              borderRadius: '4px', 
              border: '1px solid var(--border)', 
              background: currentTab === 'scripts' ? 'var(--info)' : 'transparent',
              minWidth: '60px'
            }}
          >
            Scripts
          </button>
          <button 
            className={`tab-btn ${currentTab === 'project' ? 'active' : ''}`}
            onClick={() => setCurrentTab('project')}
            style={{ 
              flex: 1, 
              padding: '0.5rem', 
              borderRadius: '4px', 
              border: '1px solid var(--border)', 
              background: currentTab === 'project' ? 'var(--info)' : 'transparent',
              minWidth: '60px'
            }}
          >
            Project
          </button>
          <button 
            className={`tab-btn ${currentTab === 'animations' ? 'active' : ''}`}
            onClick={() => setCurrentTab('animations')}
            style={{ 
              flex: 1, 
              padding: '0.5rem', 
              borderRadius: '4px', 
              border: '1px solid var(--border)', 
              background: currentTab === 'animations' ? 'var(--info)' : 'transparent',
              minWidth: '60px'
            }}
          >
            Anims
          </button>
          <button 
            className={`tab-btn ${currentTab === 'compare' ? 'active' : ''}`}
            onClick={() => setCurrentTab('compare')}
            style={{ 
              flex: 1, 
              padding: '0.5rem', 
              borderRadius: '4px', 
              border: '1px solid var(--border)', 
              background: currentTab === 'compare' ? 'var(--info)' : 'transparent',
              minWidth: '60px'
            }}
          >
            Compare
          </button>
          <button 
            className={`tab-btn ${currentTab === 'frames' ? 'active' : ''}`}
            onClick={() => setCurrentTab('frames')}
            style={{ 
              flex: 1, 
              padding: '0.5rem', 
              borderRadius: '4px', 
              border: '1px solid var(--border)', 
              background: currentTab === 'frames' ? 'var(--info)' : 'transparent',
              minWidth: '60px'
            }}
          >
            Frames
          </button>
          <button 
            className={`tab-btn ${currentTab === 'packs' ? 'active' : ''}`}
            onClick={() => setCurrentTab('packs')}
            style={{ 
              flex: 1, 
              padding: '0.5rem', 
              borderRadius: '4px', 
              border: '1px solid var(--border)', 
              background: currentTab === 'packs' ? 'var(--info)' : 'transparent',
              minWidth: '60px'
            }}
          >
            Packs
          </button>
          <button 
            className={`tab-btn ${currentTab === 'roster' ? 'active' : ''}`}
            onClick={() => setCurrentTab('roster')}
            style={{ 
              flex: 1, 
              padding: '0.5rem', 
              borderRadius: '4px', 
              border: '1px solid var(--border)', 
              background: currentTab === 'roster' ? 'var(--info)' : 'transparent',
              minWidth: '60px'
            }}
          >
            Roster
          </button>
          <button 
            className={`tab-btn ${currentTab === 'ai' ? 'active' : ''}`}
            onClick={() => setCurrentTab('ai')}
            style={{ 
              flex: 1, 
              padding: '0.5rem', 
              borderRadius: '4px', 
              border: '1px solid var(--border)', 
              background: currentTab === 'ai' ? 'var(--info)' : 'transparent',
              minWidth: '60px'
            }}
          >
            AI
          </button>
          <button 
            className={`tab-btn ${currentTab === 'plugins' ? 'active' : ''}`}
            onClick={() => setCurrentTab('plugins')}
            style={{ 
              flex: 1, 
              padding: '0.5rem', 
              borderRadius: '4px', 
              border: '1px solid var(--border)', 
              background: currentTab === 'plugins' ? 'var(--accent)' : 'transparent',
              minWidth: '60px'
            }}
          >
            🔌 Plugins
          </button>
          <button 
            className={`tab-btn ${currentTab === 'banks' ? 'active' : ''}`}
            onClick={() => setCurrentTab('banks')}
            style={{ 
              flex: 1, 
              padding: '0.5rem', 
              borderRadius: '4px', 
              border: '1px solid var(--border)', 
              background: currentTab === 'banks' ? 'var(--accent)' : 'transparent',
              minWidth: '60px'
            }}
          >
            🏦 Banks
          </button>
          <button 
            className={`tab-btn ${currentTab === 'animation-player' ? 'active' : ''}`}
            onClick={() => setCurrentTab('animation-player')}
            style={{ 
              flex: 1, 
              padding: '0.5rem', 
              borderRadius: '4px', 
              border: '1px solid var(--border)', 
              background: currentTab === 'animation-player' ? 'var(--accent)' : 'transparent',
              minWidth: '60px'
            }}
          >
            🎬 Player
          </button>
          <button 
            className={`tab-btn ${currentTab === 'audio' ? 'active' : ''}`}
            onClick={() => setCurrentTab('audio')}
            style={{ 
              flex: 1, 
              padding: '0.5rem', 
              borderRadius: '4px', 
              border: '1px solid var(--border)', 
              background: currentTab === 'audio' ? 'var(--accent)' : 'transparent',
              minWidth: '60px'
            }}
          >
            Audio
          </button>
          <button 
            className={`tab-btn ${currentTab === 'settings' ? 'active' : ''}`}
            onClick={() => setCurrentTab('settings')}
            style={{ 
              flex: 1, 
              padding: '0.5rem', 
              borderRadius: '4px', 
              border: '1px solid var(--border)', 
              background: currentTab === 'settings' ? 'var(--blue)' : 'transparent',
              minWidth: '60px'
            }}
          >
            Settings
          </button>
          <button 
            className={`tab-btn ${currentTab === 'test' ? 'active' : ''}`}
            onClick={() => setCurrentTab('test')}
            style={{ 
              flex: 1, 
              padding: '0.5rem', 
              borderRadius: '4px', 
              border: '1px solid var(--border)', 
              background: currentTab === 'test' ? 'var(--accent)' : 'transparent',
              minWidth: '60px',
              color: currentTab === 'test' ? 'white' : 'inherit',
            }}
          >
            🎮 Test
          </button>
          <button 
            className={`tab-btn ${currentTab === 'plugins' ? 'active' : ''}`}
            onClick={() => setCurrentTab('plugins')}
            style={{ 
              flex: 1, 
              padding: '0.5rem', 
              borderRadius: '4px', 
              border: '1px solid var(--border)', 
              background: currentTab === 'plugins' ? 'var(--info)' : 'transparent',
              minWidth: '60px'
            }}
          >
            Plugins
          </button>
          <button 
            className={`tab-btn ${currentTab === 'banks' ? 'active' : ''}`}
            onClick={() => setCurrentTab('banks')}
            style={{ 
              flex: 1, 
              padding: '0.5rem', 
              borderRadius: '4px', 
              border: '1px solid var(--border)', 
              background: currentTab === 'banks' ? 'var(--info)' : 'transparent',
              minWidth: '60px'
            }}
          >
            Bank Map
          </button>
          <button 
            className={`tab-btn ${currentTab === 'animation-player' ? 'active' : ''}`}
            onClick={() => setCurrentTab('animation-player')}
            style={{ 
              flex: 1, 
              padding: '0.5rem', 
              borderRadius: '4px', 
              border: '1px solid var(--border)', 
              background: currentTab === 'animation-player' ? 'var(--info)' : 'transparent',
              minWidth: '60px'
            }}
          >
            Animation Player
          </button>
        </div>

        {currentTab === 'editor' && (
          <nav>
            <ul className="boxer-list">
              {boxers.map((boxer) => (
                <li 
                  key={boxer.key} 
                  className={`boxer-item ${selectedBoxer?.key === boxer.key ? 'active' : ''}`}
                  onClick={() => selectBoxer(boxer.key)}
                >
                  <span>{boxer.name}</span>
                </li>
              ))}
            </ul>
          </nav>
        )}

        {currentTab === 'project' && (
          <div style={{ padding: '0.5rem 0' }}>
            <ProjectManager />
          </div>
        )}

        {currentTab === 'packs' && (
          <div style={{ padding: '0.5rem 0', overflow: 'auto' }}>
            <LayoutPackManager onBrowsePack={() => {}} />
          </div>
        )}

        {currentTab === 'roster' && (
          <div style={{ padding: '0.5rem 0' }}>
            <RosterEditor />
          </div>
        )}

        {currentTab === 'audio' && (
          <div style={{ padding: '0.5rem 0', overflow: 'auto' }}>
            <AudioEditor />
          </div>
        )}

        {currentTab === 'settings' && (
          <div style={{ padding: '0.5rem 0' }}>
            <div style={{ padding: '0.5rem', color: 'var(--text-dim)', fontSize: '0.875rem' }}>
              <UpdateSettings />
            </div>
          </div>
        )}

        {currentTab === 'plugins' && (
          <div style={{ padding: '0.5rem 0' }}>
            <PluginManager />
          </div>
        )}

        {currentTab === 'banks' && (
          <div style={{ padding: '0.5rem 0' }}>
            <BankVisualization />
          </div>
        )}

        {currentTab === 'animation-player' && (
          <div style={{ padding: '0.5rem 0' }}>
            <AnimationPlayer />
          </div>
        )}

        {/* Emulator Settings & External Tools Buttons in Sidebar */}
        {romSha1 && (
          <div style={{ marginTop: 'auto', padding: '1rem 0', borderTop: '1px solid var(--border)' }}>
            <button
              onClick={() => setShowEmulatorSettings(true)}
              style={{
                width: '100%',
                padding: '0.5rem',
                backgroundColor: 'var(--glass)',
                fontSize: '0.9rem',
                marginBottom: '0.5rem',
              }}
            >
              ⚙️ Emulator Settings
            </button>
            <button
              onClick={() => setShowExternalTools(true)}
              style={{
                width: '100%',
                padding: '0.5rem',
                backgroundColor: 'var(--glass)',
                fontSize: '0.9rem',
              }}
            >
              🛠️ External Tools
            </button>
            <div style={{ 
              marginTop: '0.5rem', 
              fontSize: '0.75rem', 
              color: 'var(--text-muted)',
              textAlign: 'center' 
            }}>
              Press F5 to test
            </div>
          </div>
        )}
      </aside>

      <main className="main-content">
        {currentTab === 'viewer' ? (
          <FighterViewer />
        ) : currentTab === 'scripts' ? (
          <ScriptViewer />
        ) : currentTab === 'animations' ? (
          <AnimationEditor />
        ) : currentTab === 'compare' ? (
          <ComparisonView />
        ) : currentTab === 'frames' ? (
          <FrameReconstructor />
        ) : currentTab === 'packs' ? (
          <div style={{ padding: '1.5rem', maxWidth: '1200px', margin: '0 auto' }}>
            <LayoutPackBrowser />
          </div>
        ) : currentTab === 'roster' ? (
          <div style={{ padding: '1.5rem', maxWidth: '1200px', margin: '0 auto' }}>
            <RosterEditor />
          </div>
        ) : currentTab === 'ai' ? (
          <div style={{ padding: '1.5rem', maxWidth: '1400px', margin: '0 auto', height: 'calc(100vh - 200px)' }}>
            <AIEditor />
          </div>
        ) : currentTab === 'settings' ? (
          <div style={{ padding: '1.5rem', maxWidth: '1200px', margin: '0 auto' }}>
            <h2 style={{ marginBottom: '1.5rem' }}>Settings</h2>
            <UpdateSettings />
          </div>
        ) : currentTab === 'test' ? (
          <div style={{ height: 'calc(100vh - 100px)', padding: '1rem' }}>
            <EmbeddedEmulator
              layout="tab"
              editedRomData={undefined}
              originalRomData={undefined}
              romName={currentProject?.metadata?.name || 'Super Punch-Out!!'}
            />
          </div>
        ) : currentTab === 'plugins' ? (
          <div style={{ padding: '1.5rem', maxWidth: '1200px', margin: '0 auto' }}>
            <PluginManager />
          </div>
        ) : currentTab === 'banks' ? (
          <div style={{ padding: '1.5rem', maxWidth: '1200px', margin: '0 auto' }}>
            <BankVisualization />
          </div>
        ) : currentTab === 'animation-player' ? (
          <div style={{ padding: '1.5rem', maxWidth: '1200px', margin: '0 auto' }}>
            <AnimationPlayer />
          </div>
        ) : currentTab === 'project' ? (
          <div className="empty-state" style={{ maxWidth: '600px', margin: '0 auto' }}>
            <h2>Project Management</h2>
            <p>Use the sidebar to create, open, or manage your projects.</p>
            <div style={{ 
              marginTop: '2rem', 
              padding: '1.5rem', 
              backgroundColor: 'var(--bg-panel)', 
              borderRadius: '12px',
              textAlign: 'left'
            }}>
              <h3 style={{ marginTop: 0 }}>About Projects</h3>
              <ul style={{ lineHeight: '1.8', color: 'var(--text-muted)' }}>
                <li>Save your entire editing session as a project</li>
                <li>Projects track which ROM they are based on (SHA1 hash)</li>
                <li>Export modified assets for backup or sharing</li>
                <li>Package edits as IPS patches</li>
                <li>Organize multiple hacks in separate project files</li>
              </ul>
              
              <h4 style={{ marginTop: '1.5rem' }}>Project File Structure</h4>
              <pre style={{ 
                backgroundColor: 'var(--glass)', 
                padding: '1rem', 
                borderRadius: '6px',
                fontSize: '0.85rem',
                overflow: 'auto'
              }}>
{`my_project.spo/
  project.json       # metadata, SHA1, list of edits
  assets/            # exported PNGs of modified assets  
  patches/           # IPS/BPS patches`}
              </pre>
            </div>
          </div>
        ) : selectedBoxer ? (
          <div className="boxer-detail">
            <h2 style={{ fontSize: "2rem", marginBottom: "1.5rem" }}>{selectedBoxer.name}</h2>
            
            <section style={{ backgroundColor: "var(--bg-panel)", padding: "2rem", borderRadius: "12px", border: "1px solid var(--border)" }}>
              <h3>Asset Summary</h3>
              <p style={{ color: "var(--text-muted)" }}>ID: {selectedBoxer.key}</p>
              
              <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(200px, 1fr))", gap: "1rem" }}>
                <div style={{ padding: "1rem", backgroundColor: "var(--glass)", borderRadius: "8px" }}>
                  <strong>Palettes:</strong> {selectedBoxer.palette_files.length}
                </div>
                <div style={{ padding: "1rem", backgroundColor: "var(--glass)", borderRadius: "8px" }}>
                  <strong>Icons:</strong> {selectedBoxer.icon_files.length}
                </div>
                <div style={{ padding: "1rem", backgroundColor: "var(--glass)", borderRadius: "8px" }}>
                  <strong>Unique Sprite Bins:</strong> {selectedBoxer.unique_sprite_bins.length}
                </div>
                <div style={{ padding: "1rem", backgroundColor: "var(--glass)", borderRadius: "8px" }}>
                  <strong>Shared Sprite Bins:</strong> {selectedBoxer.shared_sprite_bins.length}
                </div>
              </div>
            </section>

            <section style={{ marginTop: "2rem" }}>
              <PaletteEditor />
            </section>

            <section style={{ marginTop: "2rem", backgroundColor: "var(--bg-panel)", padding: "2rem", borderRadius: "12px", border: "1px solid var(--border)" }}>
              <BoxerPreviewSheet boxer={selectedBoxer} />
            </section>

            <section style={{ marginTop: "2rem" }}>
              <AssetManager boxer={selectedBoxer} />
            </section>

            <section style={{ marginTop: "2rem", backgroundColor: "var(--bg-panel)", padding: "2rem", borderRadius: "12px", border: "1px solid var(--border)" }}>
              <SpriteBinEditor boxer={selectedBoxer} />
            </section>

            <section style={{ marginTop: "2rem" }}>
              <ExportPanel />
            </section>

            <section style={{ marginTop: "2rem" }}>
              <PatchNotesGenerator />
            </section>
          </div>
        ) : (
          <div className="empty-state">
            <p>Select a boxer from the sidebar to begin editing.</p>
          </div>
        )}
      </main>

      {/* Modals */}
      <EmulatorSettings
        isOpen={showEmulatorSettings}
        onClose={() => setShowEmulatorSettings(false)}
        onSave={() => {}}
      />
      
      <ExternalToolsManager
        isOpen={showExternalTools}
        onClose={() => setShowExternalTools(false)}
      />
      
      <KeyboardShortcutsHelp
        isOpen={showKeyboardShortcuts}
        onClose={() => setShowKeyboardShortcuts(false)}
      />
      
      <HelpSystem
        isOpen={showHelp}
        onClose={() => {
          setShowHelp(false);
          setHelpContext(undefined);
        }}
        initialContext={helpContext}
      />

      {/* Region Selector Modal */}
      {showRegionSelector && (
        <div style={{
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
          padding: '2rem',
        }}>
          <div style={{
            backgroundColor: 'var(--bg-panel)',
            borderRadius: '12px',
            maxWidth: '500px',
            width: '100%',
            maxHeight: '90vh',
            overflow: 'auto',
            boxShadow: '0 25px 50px -12px rgba(0, 0, 0, 0.5)',
          }}>
            <div style={{
              display: 'flex',
              justifyContent: 'space-between',
              alignItems: 'center',
              padding: '1rem 1.5rem',
              borderBottom: '1px solid var(--border)',
            }}>
              <h2 style={{ margin: 0, fontSize: '1.25rem' }}>Select ROM Region</h2>
              <button
                onClick={() => setShowRegionSelector(false)}
                style={{
                  background: 'none',
                  border: 'none',
                  fontSize: '1.5rem',
                  cursor: 'pointer',
                  color: 'var(--text-muted)',
                }}
              >
                ×
              </button>
            </div>
            <div style={{ padding: '1rem' }}>
              <RegionSelector
                romPath={romPath}
                onRegionDetected={handleRegionDetected}
                onRegionSelected={handleRegionSelected}
              />
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

/**
 * Root App component wrapped with ThemeProvider
 */
function AppWithTheme(): React.ReactElement {
  return (
    <ThemeProvider>
      <UpdateChecker>
        <App />
      </UpdateChecker>
    </ThemeProvider>
  );
}

export default AppWithTheme;
