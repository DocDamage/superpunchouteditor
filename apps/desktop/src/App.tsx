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
import { featureLabel, isFeatureVisible } from "./featureMaturity";
import { ThemeProvider, useTheme } from "./context/ThemeProvider";
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
import { TextEditor } from "./components/TextEditor";
import { GuidedSidebar, GuidedTabKey } from "./components/GuidedSidebar";
import { WelcomeWorkspace } from "./components/WelcomeWorkspace";

import { KeyboardShortcutsHelp, HelpSystem } from "./components/help";
import { ToastContainer } from "./components/ToastContainer";
import { UpdateSettings } from "./components/UpdateSettings";
import { UpdateChecker } from "./components/UpdateChecker";
import { EmbeddedEmulator } from "./components/EmbeddedEmulator";
import menuSheetUrl from "./assets/menu-fonts.png";
import "./styles/emulator.css";

type TabKey = GuidedTabKey;

const MODAL_STYLE_TABS = new Set<TabKey>(["plugins", "packs", "test", "settings"]);

const ALL_TAB_ITEMS: Array<{ key: TabKey; label: string }> = [
  { key: "roster", label: "Characters" },
  { key: "editor", label: "Edit" },
  { key: "viewer", label: "Inspect" },
  { key: "compare", label: "Compare" },
  { key: "test", label: "Test Game" },
  { key: "project", label: "Projects" },
  { key: "scripts", label: "Scripts" },
  { key: "animations", label: "Animations" },
  { key: "frames", label: "Frames" },
  { key: "packs", label: "Packs" },
  { key: "ai", label: "AI" },
  { key: "plugins", label: "Plugins" },
  { key: "banks", label: "Banks" },
  { key: "animation-player", label: "Animation Player" },
  { key: "audio", label: "Audio" },
  { key: "text", label: "Text" },
  { key: "settings", label: "Settings" },
];

const TAB_ITEMS: Array<{ key: TabKey; label: string }> = ALL_TAB_ITEMS
  .filter(({ key }) => isFeatureVisible(key))
  .map(({ key, label }) => ({ key, label: featureLabel(label, key) }));

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
        <div className="empty-state" style={{ padding: "2rem", flexDirection: "column" }}>
          <h2 style={{ marginTop: 0 }}>This panel could not be displayed</h2>
          <p style={{ color: "var(--text-muted)" }}>
            Your ROM and project data were not changed by this display error. Switch panels or restart the editor, then include this message in a tester report if it repeats.
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
                maxWidth: "100%",
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
  const [creatorAutoEnterToken, setCreatorAutoEnterToken] = useState(0);
  const [testRomData, setTestRomData] = useState<Uint8Array | null>(null);
  const [creatorSessionContext, setCreatorSessionContext] = useState<{
    boxerId?: number;
    boxerName?: string;
    circuit?: "Minor" | "Major" | "World" | "Special";
    unlockOrder?: number;
    introTextId?: number;
    assetOwnerKey?: string;
  } | null>(null);
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

  useEffect(() => {
    setCreatorSessionContext(null);
    setTestRomData(null);
  }, [romSha1]);

  const refreshTestRomData = useCallback(async () => {
    if (!isDesktopRuntime || !romSha1) {
      setTestRomData(null);
      return;
    }

    try {
      const romImage = await invoke<number[]>("get_loaded_rom_image");
      setTestRomData(new Uint8Array(romImage));
    } catch (refreshError) {
      console.error("Failed to load current ROM image for embedded emulator:", refreshError);
    }
  }, [isDesktopRuntime, romSha1]);

  useEffect(() => {
    if (currentTab !== "test") return;
    void refreshTestRomData();
  }, [currentTab, refreshTestRomData, pendingWrites.size]);

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
    const handleKeyDown = (event: KeyboardEvent) => {
      if ((event.ctrlKey || event.metaKey) && event.key === "z" && !event.shiftKey) {
        event.preventDefault();
        if (canUndo) void undo();
        return;
      }

      if (
        (event.ctrlKey || event.metaKey) &&
        ((event.shiftKey && event.key === "z") || event.key === "y")
      ) {
        event.preventDefault();
        if (canRedo) void redo();
        return;
      }

      if (event.key === "F1") {
        event.preventDefault();
        setShowHelp(true);
        setHelpContext(currentTab === "editor" ? "palette-editor" : currentTab);
        return;
      }

      if (!(event.ctrlKey || event.metaKey)) return;

      const quickTabs: Record<string, TabKey> = {
        "1": "editor",
        "2": "viewer",
        "3": "project",
        "4": "test",
        "5": "compare",
        "0": "settings",
      };

      const targetTab = quickTabs[event.key];
      if (targetTab && isFeatureVisible(targetTab)) {
        event.preventDefault();
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
    } catch (openError) {
      console.error(openError);
      setError(String(openError));
    }
  };

  const handleRegionDetected = useCallback((result: RegionDetectionResult) => {
    setDetectedRegion(result);
  }, []);

  const handleRegionSelected = useCallback(async () => {
    if (!romPath) return;
    await openRom(romPath);

    // Stable builds must never land inside a hidden experimental surface.
    const landingTab: TabKey = isFeatureVisible("roster") ? "roster" : "editor";
    setCurrentTab(landingTab);
    setLastNonModalTab(landingTab);
    setShowRegionSelector(false);
  }, [openRom, romPath]);

  const handleCloseModalStyleTab = useCallback(() => {
    setCurrentTab(lastNonModalTab);
  }, [lastNonModalTab]);

  const handleLaunchCreatorTest = useCallback((context?: {
    boxerId?: number;
    boxerName?: string;
    circuit?: "Minor" | "Major" | "World" | "Special";
    unlockOrder?: number;
    introTextId?: number;
    assetOwnerKey?: string;
  }) => {
    setCreatorSessionContext(context ?? null);
    setCreatorAutoEnterToken((current) => current + 1);
    setCurrentTab("test");
  }, []);

  const handleOpenCreatorAssetOwner = useCallback(
    (boxerKey: string) => {
      void selectBoxer(boxerKey);
      setCurrentTab("editor");
      setLastNonModalTab("editor");
    },
    [selectBoxer]
  );

  const renderEditorContent = () => {
    if (!romSha1) {
      return <WelcomeWorkspace isDesktopRuntime={isDesktopRuntime} onOpenRom={() => void handleOpenRom()} />;
    }

    if (!selectedBoxer) {
      return (
        <div className="empty-state" style={{ flexDirection: "column", textAlign: "center", padding: "2rem" }}>
          <h2>Choose a boxer to edit</h2>
          <p>Pick a boxer from the left sidebar. A palette change is a good first test because it is obvious and reversible.</p>
        </div>
      );
    }

    return (
      <div className="boxer-detail">
        <h2 style={{ fontSize: "2rem", marginBottom: "0.35rem" }}>{selectedBoxer.name}</h2>
        <p style={{ marginBottom: "1.5rem", color: "var(--text-muted)" }}>
          Make one change at a time. Use Undo/Redo in the sidebar, then Test Game before saving or exporting.
        </p>

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
            <RosterEditor mode="game" onLaunchCreatorTest={handleLaunchCreatorTest} />
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
              <div>
                <h2 style={{ marginBottom: "0.2rem" }}>Test Current Revision</h2>
                <p style={{ margin: 0, color: "var(--text-muted)", fontSize: "0.85rem" }}>
                  This uses the editor's current materialized ROM, including unsaved journal edits.
                </p>
              </div>
              <button className="tab-close-button" onClick={handleCloseModalStyleTab}>
                Close
              </button>
            </div>
            <div style={{ flex: 1, minHeight: 0 }}>
              <EmbeddedEmulator
                layout="tab"
                editedRomData={testRomData}
                originalRomData={undefined}
                romPath={romPath || null}
                romName={currentProject?.metadata?.name || "Super Punch-Out!!"}
                autoEnterCreatorToken={creatorAutoEnterToken}
                creatorSessionContext={creatorSessionContext}
                onOpenAssetOwner={handleOpenCreatorAssetOwner}
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
      case "text":
        return (
          <div style={{ padding: "1.5rem", maxWidth: "1200px", margin: "0 auto" }}>
            <TextEditor />
          </div>
        );
      case "editor":
      default:
        return renderEditorContent();
    }
  };

  return (
    <div className={`app-container ${romSha1 ? "menu-sheet-enabled" : ""}`} style={menuSheetStyle}>
      <GuidedSidebar
        tabItems={TAB_ITEMS}
        currentTab={currentTab}
        romSha1={romSha1}
        detectedRegionLabel={detectedRegion?.display_name ?? null}
        detectedRegionSupported={detectedRegion?.is_supported}
        currentProjectName={currentProject?.metadata?.name ?? null}
        runtimeIconUrl={runtimeSkin?.iconDataUrl ?? null}
        runtimeBoxerName={runtimeSkin?.boxerName ?? null}
        boxers={boxers}
        selectedBoxerKey={selectedBoxer?.key ?? null}
        boxerPortraits={boxerPortraits}
        canUndo={canUndo}
        canRedo={canRedo}
        editCount={undoStack.length}
        pendingWritesCount={pendingWrites.size}
        isDesktopRuntime={isDesktopRuntime}
        runtimeError={RUNTIME_ERROR}
        error={error}
        onOpenRom={() => void handleOpenRom()}
        onUndo={() => void undo()}
        onRedo={() => void redo()}
        onNavigate={setCurrentTab}
        onSelectBoxer={(boxerKey) => void selectBoxer(boxerKey)}
        onOpenHelp={() => {
          setHelpContext(currentTab === "editor" ? "palette-editor" : currentTab);
          setShowHelp(true);
        }}
        onOpenKeyboardShortcuts={() => setShowKeyboardShortcuts(true)}
        onOpenEmulatorSettings={() => setShowEmulatorSettings(true)}
        onOpenExternalTools={() => setShowExternalTools(true)}
      />

      <main className="main-content">
        <AppRenderBoundary key={`${currentTab}:${selectedBoxer?.key ?? "none"}`}>
          {renderMainContent()}
        </AppRenderBoundary>
      </main>

      <EmulatorSettings
        isOpen={showEmulatorSettings}
        onClose={() => setShowEmulatorSettings(false)}
        onSave={() => {}}
      />

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
            backgroundColor: "rgba(0, 0, 0, 0.72)",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            zIndex: 1000,
            padding: "2rem",
          }}
          role="presentation"
        >
          <div
            style={{
              backgroundColor: "var(--bg-panel)",
              borderRadius: "12px",
              maxWidth: "540px",
              width: "100%",
              maxHeight: "90vh",
              overflow: "auto",
              boxShadow: "0 25px 50px -12px rgba(0, 0, 0, 0.5)",
              border: "1px solid var(--border)",
            }}
            role="dialog"
            aria-modal="true"
            aria-labelledby="confirm-rom-title"
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
              <div>
                <p className="eyebrow">One quick check</p>
                <h2 id="confirm-rom-title" style={{ margin: 0, fontSize: "1.25rem" }}>Confirm ROM Region</h2>
              </div>
              <button
                onClick={() => setShowRegionSelector(false)}
                aria-label="Cancel ROM selection"
                style={{
                  background: "none",
                  border: "1px solid var(--border)",
                  minWidth: "42px",
                  minHeight: "42px",
                  padding: "0.25rem",
                  fontSize: "1.25rem",
                  cursor: "pointer",
                  color: "var(--text-muted)",
                }}
              >
                ×
              </button>
            </div>
            <div style={{ padding: "1rem" }}>
              <p style={{ color: "var(--text-muted)", fontSize: "0.85rem" }}>
                The editor checks the selected file before opening it. Your original ROM is treated as the immutable base for this editing session.
              </p>
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

function AppWithTheme(): React.ReactElement {
  return (
    <ThemeProvider>
      <UpdateChecker>
        <App />
        <ToastContainer />
      </UpdateChecker>
    </ThemeProvider>
  );
}

export default AppWithTheme;
