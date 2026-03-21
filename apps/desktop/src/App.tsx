import {
  Component,
  CSSProperties,
  ErrorInfo,
  ReactNode,
  useCallback,
  useEffect,
  useMemo,
  useState,
} from "react";
import { invoke, isTauri } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { useStore } from "./store/useStore";
import { ThemeProvider, useTheme } from "./context/ThemeProvider";
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
import { ComparisonView } from "./components/ComparisonView";
import { AIEditor } from "./components/AIEditor";
import { LayoutPackBrowser } from "./components/LayoutPackBrowser";
import { ExternalToolsManager } from "./components/ExternalToolsManager";
import { RosterEditor } from "./components/RosterEditor";
import { PluginManager } from "./components/PluginManager";
import { BankVisualization } from "./components/BankVisualization";
import { AnimationPlayer } from "./components/AnimationPlayer";
import { AudioEditor } from "./components/AudioEditor";

import { HelpButton, KeyboardShortcutsHelp, HelpSystem } from "./components/help";
import { UpdateSettings } from "./components/UpdateSettings";
import { UpdateChecker } from "./components/UpdateChecker";
import { EmbeddedEmulator } from "./components/EmbeddedEmulator";
import menuSheetUrl from "./assets/menu-fonts.png";
import "./styles/emulator.css";

type TabKey =
  | "editor"
  | "viewer"
  | "scripts"
  | "animations"
  | "frames"
  | "compare"
  | "project"
  | "packs"
  | "roster"
  | "ai"
  | "plugins"
  | "banks"
  | "animation-player"
  | "audio"
  | "test"
  | "settings";

const MODAL_STYLE_TABS = new Set<TabKey>(["plugins", "packs", "test", "settings"]);

const TAB_ITEMS: Array<{ key: TabKey; label: string }> = [
  { key: "editor", label: "Editor" },
  { key: "roster", label: "Character Create" },
  { key: "viewer", label: "Viewer" },
  { key: "scripts", label: "Scripts" },
  { key: "animations", label: "Animations" },
  { key: "frames", label: "Frames" },
  { key: "compare", label: "Compare" },
  { key: "project", label: "Project" },
  { key: "packs", label: "Packs" },
  { key: "ai", label: "AI" },
  { key: "plugins", label: "Plugins" },
  { key: "banks", label: "Banks" },
  { key: "animation-player", label: "Player" },
  { key: "audio", label: "Audio" },
  { key: "test", label: "Test" },
  { key: "settings", label: "Settings" },
];

const RUNTIME_ERROR =
  "Desktop runtime not detected. Start this app with `npm run tauri dev` from apps/desktop.";

const bytesToDataUrl = (bytes: number[] | null | undefined): string | null => {
  if (!bytes || bytes.length === 0) return null;
  let binary = "";
  for (const byte of bytes) {
    binary += String.fromCharCode(byte);
  }
  return `data:image/png;base64,${btoa(binary)}`;
};

interface AppRenderBoundaryProps {
  children: ReactNode;
}

interface AppRenderBoundaryState {
  hasError: boolean;
  message: string | null;
}

class AppRenderBoundary extends Component<AppRenderBoundaryProps, AppRenderBoundaryState> {
  state: AppRenderBoundaryState = {
    hasError: false,
    message: null,
  };

  static getDerivedStateFromError(error: Error): AppRenderBoundaryState {
    return {
      hasError: true,
      message: error.message,
    };
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    console.error("Main content render failed:", error, info);
  }

  render() {
    if (this.state.hasError) {
      return (
        <div className="empty-state" style={{ padding: "2rem" }}>
          <h2 style={{ marginTop: 0 }}>Panel Render Failed</h2>
          <p style={{ color: "var(--text-muted)" }}>
            The selected boxer triggered a UI render error.
          </p>
          {this.state.message && (
            <pre
              style={{
                whiteSpace: "pre-wrap",
                wordBreak: "break-word",
                backgroundColor: "var(--bg-panel)",
                border: "1px solid var(--border)",
                borderRadius: "8px",
                padding: "1rem",
              }}
            >
              {this.state.message}
            </pre>
          )}
        </div>
      );
    }

    return this.props.children;
  }
}

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
    clearHistory,
    setError,
    error,
  } = useStore();
  const { runtimeSkin, setRuntimeSkin } = useTheme();

  const isDesktopRuntime = useMemo(() => isTauri(), []);

  const [showEmulatorSettings, setShowEmulatorSettings] = useState(false);
  const [showExternalTools, setShowExternalTools] = useState(false);
  const [showKeyboardShortcuts, setShowKeyboardShortcuts] = useState(false);
  const [showHelp, setShowHelp] = useState(false);
  const [helpContext, setHelpContext] = useState<string | undefined>(undefined);
  const [showRegionSelector, setShowRegionSelector] = useState(false);
  const [detectedRegion, setDetectedRegion] = useState<RegionDetectionResult | null>(null);
  const [romPath, setRomPath] = useState("");
  const [currentTab, setCurrentTab] = useState<TabKey>("editor");
  const [lastNonModalTab, setLastNonModalTab] = useState<TabKey>("editor");
  const [boxerPortraits, setBoxerPortraits] = useState<Record<string, string>>({});
  const menuSheetStyle = useMemo(
    () =>
      ({
        "--menu-sheet-image": `url("${menuSheetUrl}")`,
      }) as CSSProperties,
    []
  );

  useEffect(() => {
    if (!isDesktopRuntime) {
      setError(RUNTIME_ERROR);
      return;
    }

    void loadBoxers();
    void getCurrentProject();
  }, [isDesktopRuntime, loadBoxers, getCurrentProject, setError]);

  useEffect(() => {
    if (!MODAL_STYLE_TABS.has(currentTab)) {
      setLastNonModalTab(currentTab);
    }
  }, [currentTab]);

  // Undo state is refreshed explicitly after each mutating action (openRom, undo,
  // redo, recordPaletteEdit, etc.) — no polling needed.

  useEffect(() => {
    if (!isDesktopRuntime || !romSha1) return;
    void clearHistory();
  }, [isDesktopRuntime, romSha1, clearHistory]);

  useEffect(() => {
    if (!isDesktopRuntime || !romSha1) {
      setRuntimeSkin(null);
      return;
    }

    let isCancelled = false;

    void (async () => {
      try {
        const themeAssets = await invoke<{
          boxer_key: string;
          boxer_name: string;
          palette: Array<{ r: number; g: number; b: number }>;
          icon_png: number[] | null;
          portrait_png: number[] | null;
        }>("get_runtime_theme_assets", {
          boxerKey: selectedBoxer?.key ?? null,
        });

        if (isCancelled) return;

        setRuntimeSkin({
          boxerKey: themeAssets.boxer_key,
          boxerName: themeAssets.boxer_name,
          palette: themeAssets.palette,
          iconDataUrl: bytesToDataUrl(themeAssets.icon_png),
          portraitDataUrl: bytesToDataUrl(themeAssets.portrait_png),
        });
      } catch (themeError) {
        console.error("Failed to load runtime theme assets:", themeError);
        if (!isCancelled) {
          setRuntimeSkin(null);
        }
      }
    })();

    return () => {
      isCancelled = true;
    };
  }, [isDesktopRuntime, romSha1, selectedBoxer?.key, setRuntimeSkin]);

  useEffect(() => {
    if (!isDesktopRuntime || !romSha1 || boxers.length === 0) {
      setBoxerPortraits({});
      return;
    }

    let isCancelled = false;

    void (async () => {
      const entries = await Promise.all(
        boxers.map(async (boxer) => {
          try {
            const assets = await invoke<{
              portrait_png: number[] | null;
              icon_png: number[] | null;
            }>("get_runtime_theme_assets", {
              boxerKey: boxer.key,
            });
            const imageUrl = bytesToDataUrl(assets.portrait_png) ?? bytesToDataUrl(assets.icon_png);
            return [boxer.key, imageUrl] as const;
          } catch (thumbnailError) {
            console.error(`Failed to load portrait for ${boxer.key}:`, thumbnailError);
            return [boxer.key, null] as const;
          }
        })
      );

      if (isCancelled) return;

      const portraitMap = entries.reduce<Record<string, string>>((acc, [key, url]) => {
        if (url) {
          acc[key] = url;
        }
        return acc;
      }, {});

      setBoxerPortraits(portraitMap);
    })();

    return () => {
      isCancelled = true;
    };
  }, [isDesktopRuntime, romSha1, boxers]);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === "z" && !e.shiftKey) {
        e.preventDefault();
        if (canUndo) void undo();
        return;
      }

      if ((e.ctrlKey || e.metaKey) && ((e.shiftKey && e.key === "z") || e.key === "y")) {
        e.preventDefault();
        if (canRedo) void redo();
        return;
      }

      if (e.key === "F1") {
        e.preventDefault();
        setShowHelp(true);
        setHelpContext(currentTab === "editor" ? "palette-editor" : currentTab);
        return;
      }

      if (!(e.ctrlKey || e.metaKey)) return;

      const quickTabs: Record<string, TabKey> = {
        "1": "editor",
        "2": "viewer",
        "3": "scripts",
        "4": "animations",
        "5": "frames",
        "6": "compare",
        "7": "plugins",
        "8": "banks",
        "9": "animation-player",
        "0": "settings",
      };

      const targetTab = quickTabs[e.key];
      if (targetTab) {
        e.preventDefault();
        setCurrentTab(targetTab);
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [canUndo, canRedo, undo, redo, currentTab]);

  const handleOpenRom = async () => {
    if (!isDesktopRuntime) {
      setError(RUNTIME_ERROR);
      return;
    }

    try {
      const selected = await open({
        multiple: false,
        filters: [
          {
            name: "SNES ROM",
            extensions: ["sfc", "smc"],
          },
        ],
      });

      if (typeof selected === "string") {
        setRomPath(selected);
        setShowRegionSelector(true);
      }
    } catch (e) {
      console.error(e);
      setError(String(e));
    }
  };

  const handleRegionDetected = useCallback((result: RegionDetectionResult) => {
    setDetectedRegion(result);
  }, []);

  const handleRegionSelected = useCallback(async () => {
    if (!romPath) return;
    await openRom(romPath);
    setCurrentTab("roster");
    setLastNonModalTab("roster");
    setShowRegionSelector(false);
  }, [openRom, romPath]);

  const handleCloseModalStyleTab = useCallback(() => {
    setCurrentTab(lastNonModalTab);
  }, [lastNonModalTab]);

  const renderEditorContent = () => {
    if (!selectedBoxer) {
      return (
        <div className="empty-state">
          <p>Select a boxer from the sidebar to begin editing.</p>
        </div>
      );
    }

    return (
      <div className="boxer-detail">
        <h2 style={{ fontSize: "2rem", marginBottom: "1.5rem" }}>{selectedBoxer.name}</h2>

        <section
          style={{
            backgroundColor: "var(--bg-panel)",
            padding: "2rem",
            borderRadius: "12px",
            border: "1px solid var(--border)",
          }}
        >
          <h3>Asset Summary</h3>
          <p style={{ color: "var(--text-muted)" }}>ID: {selectedBoxer.key}</p>

          <div
            style={{
              display: "grid",
              gridTemplateColumns: "repeat(auto-fill, minmax(200px, 1fr))",
              gap: "1rem",
            }}
          >
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

        <section
          style={{
            marginTop: "2rem",
            backgroundColor: "var(--bg-panel)",
            padding: "2rem",
            borderRadius: "12px",
            border: "1px solid var(--border)",
          }}
        >
          <BoxerPreviewSheet boxer={selectedBoxer} />
        </section>

        <section style={{ marginTop: "2rem" }}>
          <AssetManager boxer={selectedBoxer} />
        </section>

        <section
          style={{
            marginTop: "2rem",
            backgroundColor: "var(--bg-panel)",
            padding: "2rem",
            borderRadius: "12px",
            border: "1px solid var(--border)",
          }}
        >
          <SpriteBinEditor boxer={selectedBoxer} />
        </section>

        <section style={{ marginTop: "2rem" }}>
          <ExportPanel />
        </section>

        <section style={{ marginTop: "2rem" }}>
          <PatchNotesGenerator />
        </section>
      </div>
    );
  };

  const renderMainContent = () => {
    switch (currentTab) {
      case "viewer":
        return <FighterViewer />;
      case "scripts":
        return <ScriptViewer />;
      case "animations":
        return <AnimationEditor />;
      case "compare":
        return <ComparisonView />;
      case "frames":
        return <FrameReconstructor />;
      case "packs":
        return (
          <div style={{ padding: "1.5rem", maxWidth: "1200px", margin: "0 auto" }}>
            <LayoutPackBrowser onClose={handleCloseModalStyleTab} />
          </div>
        );
      case "roster":
        return (
          <div style={{ padding: "1.5rem", maxWidth: "1200px", margin: "0 auto" }}>
            <RosterEditor mode="game" />
          </div>
        );
      case "ai":
        return (
          <div
            style={{
              padding: "1.5rem",
              maxWidth: "1400px",
              margin: "0 auto",
              height: "calc(100vh - 200px)",
            }}
          >
            <AIEditor />
          </div>
        );
      case "settings":
        return (
          <div style={{ padding: "1.5rem", maxWidth: "1200px", margin: "0 auto" }}>
            <div className="tab-close-header">
              <h2 style={{ marginBottom: 0 }}>Settings</h2>
              <button className="tab-close-button" onClick={handleCloseModalStyleTab}>
                Close
              </button>
            </div>
            <UpdateSettings />
          </div>
        );
      case "test":
        return (
          <div
            style={{
              height: "calc(100vh - 100px)",
              padding: "1rem",
              display: "flex",
              flexDirection: "column",
              minHeight: 0,
            }}
          >
            <div className="tab-close-header">
              <h2 style={{ marginBottom: 0 }}>Test Emulator</h2>
              <button className="tab-close-button" onClick={handleCloseModalStyleTab}>
                Close
              </button>
            </div>
            <div style={{ flex: 1, minHeight: 0 }}>
              <EmbeddedEmulator
                layout="tab"
                editedRomData={undefined}
                originalRomData={undefined}
                romName={currentProject?.metadata?.name || "Super Punch-Out!!"}
              />
            </div>
          </div>
        );
      case "plugins":
        return (
          <div style={{ padding: "1.5rem", maxWidth: "1200px", margin: "0 auto" }}>
            <PluginManager isOpen={true} onClose={handleCloseModalStyleTab} />
          </div>
        );
      case "banks":
        return (
          <div style={{ padding: "1.5rem", maxWidth: "1200px", margin: "0 auto" }}>
            <BankVisualization />
          </div>
        );
      case "animation-player":
        return (
          <div style={{ padding: "1.5rem", maxWidth: "1200px", margin: "0 auto" }}>
            <AnimationPlayer />
          </div>
        );
      case "project":
        return (
          <div style={{ padding: "1.5rem", maxWidth: "1200px", margin: "0 auto" }}>
            <ProjectManager />
          </div>
        );
      case "audio":
        return (
          <div style={{ padding: "1.5rem", maxWidth: "1200px", margin: "0 auto" }}>
            <AudioEditor />
          </div>
        );
      case "editor":
      default:
        return renderEditorContent();
    }
  };

  return (
    <div className={`app-container ${romSha1 ? "menu-sheet-enabled" : ""}`} style={menuSheetStyle}>
      <aside className="sidebar">
        <div className="sidebar-title-row">
          <div className="sidebar-brand">
            {runtimeSkin?.iconDataUrl && (
              <img
                src={runtimeSkin.iconDataUrl}
                alt={`${runtimeSkin.boxerName} icon`}
                className="sidebar-brand-icon"
              />
            )}
            <div>
              {romSha1 && <div className="menu-sheet-logo" aria-hidden={true} />}
              <h1 style={{ margin: 0, textShadow: "none" }} className={romSha1 ? "menu-sheet-title-hidden" : ""}>
                SPO!! Editor
              </h1>
              {runtimeSkin?.boxerName && (
                <div className="auth-mode-label">Authenticity Skin: {runtimeSkin.boxerName}</div>
              )}
            </div>
          </div>
          <ThemeToggle variant="minimal" size="small" />
        </div>

        {!isDesktopRuntime && <div className="runtime-warning">{RUNTIME_ERROR}</div>}

        <div className="header-bar" style={{ display: "flex", gap: "0.5rem", marginBottom: "1rem", flexWrap: "wrap" }}>
          <button onClick={handleOpenRom} style={{ flex: 1, minWidth: "110px" }} disabled={!isDesktopRuntime}>
            {romSha1 ? "Switch ROM" : "Open ROM"}
          </button>

          {romSha1 && (
            <>
              <HelpButton context="rom-loading" size="small" style={{ padding: "0.5rem" }} />
              <button
                onClick={() => void undo()}
                disabled={!canUndo}
                title="Undo (Ctrl+Z)"
                style={{
                  padding: "0.5rem 0.75rem",
                  opacity: canUndo ? 1 : 0.5,
                  cursor: canUndo ? "pointer" : "not-allowed",
                }}
              >
                Undo
              </button>
              <button
                onClick={() => void redo()}
                disabled={!canRedo}
                title="Redo (Ctrl+Shift+Z or Ctrl+Y)"
                style={{
                  padding: "0.5rem 0.75rem",
                  opacity: canRedo ? 1 : 0.5,
                  cursor: canRedo ? "pointer" : "not-allowed",
                }}
              >
                Redo
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

        {romSha1 && undoStack.length > 0 && (
          <div
            className="status-badge"
            style={{
              marginBottom: "0.5rem",
              backgroundColor: "var(--accent)",
              color: "white",
              fontSize: "0.75rem",
              padding: "0.25rem 0.5rem",
            }}
            title={`${undoStack.length} edit${undoStack.length === 1 ? "" : "s"} in history`}
          >
            {undoStack.length} edit{undoStack.length === 1 ? "" : "s"} in history
          </div>
        )}

        {romSha1 && (
          <div className="status-badge active" style={{ marginBottom: "0.5rem" }}>
            ROM OK: {romSha1.substring(0, 8)}...
            {detectedRegion?.display_name && (
              <span
                style={{
                  marginLeft: "0.5rem",
                  padding: "0.125rem 0.375rem",
                  backgroundColor: detectedRegion.is_supported ? "var(--success)" : "var(--warning)",
                  borderRadius: "3px",
                  fontSize: "0.7rem",
                  fontWeight: "bold",
                  color: "white",
                }}
              >
                {detectedRegion.display_name.split("(")[1]?.replace(")", "") || "USA"}
              </span>
            )}
          </div>
        )}

        {currentProject && (
          <div
            className="status-badge"
            style={{
              marginBottom: "1rem",
              backgroundColor: "var(--info)",
              color: "white",
            }}
          >
            Project: {currentProject.metadata.name}
          </div>
        )}

        {error && <div className="error-banner">{error}</div>}

        <div className="tab-grid">
          {TAB_ITEMS.map((tab) => (
            <button
              key={tab.key}
              className={`tab-button ${currentTab === tab.key ? "active" : ""}`}
              onClick={() => setCurrentTab(tab.key)}
            >
              {tab.label}
            </button>
          ))}
        </div>

        {currentTab === "editor" ? (
          <nav>
            <ul className="boxer-list">
              {boxers.map((boxer) => (
                <li
                  key={boxer.key}
                  className={`boxer-item ${selectedBoxer?.key === boxer.key ? "active" : ""}`}
                  onClick={() => void selectBoxer(boxer.key)}
                >
                  {boxerPortraits[boxer.key] && (
                    <img
                      src={boxerPortraits[boxer.key]}
                      alt={boxer.name}
                      className="boxer-item-portrait"
                    />
                  )}
                  <span>{boxer.name}</span>
                </li>
              ))}
            </ul>
          </nav>
        ) : (
          <p className="sidebar-hint">Use Ctrl+1..0 shortcuts to jump between tabs quickly.</p>
        )}

        {romSha1 && (
          <div style={{ marginTop: "auto", padding: "1rem 0", borderTop: "1px solid var(--border)" }}>
            <button
              onClick={() => setShowEmulatorSettings(true)}
              style={{ width: "100%", padding: "0.5rem", backgroundColor: "var(--glass)", fontSize: "0.9rem", marginBottom: "0.5rem" }}
            >
              Emulator Settings
            </button>
            <button
              onClick={() => setShowExternalTools(true)}
              style={{ width: "100%", padding: "0.5rem", backgroundColor: "var(--glass)", fontSize: "0.9rem" }}
            >
              External Tools
            </button>
          </div>
        )}
      </aside>

      <main className="main-content">
        <AppRenderBoundary key={`${currentTab}:${selectedBoxer?.key ?? "none"}`}>
          {renderMainContent()}
        </AppRenderBoundary>
      </main>

      <EmulatorSettings isOpen={showEmulatorSettings} onClose={() => setShowEmulatorSettings(false)} onSave={() => {}} />

      <ExternalToolsManager isOpen={showExternalTools} onClose={() => setShowExternalTools(false)} />

      <KeyboardShortcutsHelp isOpen={showKeyboardShortcuts} onClose={() => setShowKeyboardShortcuts(false)} />

      <HelpSystem
        isOpen={showHelp}
        onClose={() => {
          setShowHelp(false);
          setHelpContext(undefined);
        }}
        initialContext={helpContext}
      />

      {showRegionSelector && (
        <div
          style={{
            position: "fixed",
            top: 0,
            left: 0,
            right: 0,
            bottom: 0,
            backgroundColor: "rgba(0, 0, 0, 0.7)",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            zIndex: 1000,
            padding: "2rem",
          }}
        >
          <div
            style={{
              backgroundColor: "var(--bg-panel)",
              borderRadius: "12px",
              maxWidth: "500px",
              width: "100%",
              maxHeight: "90vh",
              overflow: "auto",
              boxShadow: "0 25px 50px -12px rgba(0, 0, 0, 0.5)",
            }}
          >
            <div
              style={{
                display: "flex",
                justifyContent: "space-between",
                alignItems: "center",
                padding: "1rem 1.5rem",
                borderBottom: "1px solid var(--border)",
              }}
            >
              <h2 style={{ margin: 0, fontSize: "1.25rem" }}>Select ROM Region</h2>
              <button
                onClick={() => setShowRegionSelector(false)}
                style={{ background: "none", border: "none", fontSize: "1.5rem", cursor: "pointer", color: "var(--text-muted)" }}
              >
                x
              </button>
            </div>
            <div style={{ padding: "1rem" }}>
              <RegionSelector romPath={romPath} onRegionDetected={handleRegionDetected} onRegionSelected={handleRegionSelected} />
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

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
