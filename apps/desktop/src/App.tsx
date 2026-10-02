import {
  Component,
  ErrorInfo,
  ReactNode,
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import { invoke, isTauri } from "@tauri-apps/api/core";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { open } from "@tauri-apps/plugin-dialog";
import { useStore } from "./store/useStore";
import { featureLabel, isFeatureVisible } from "./featureMaturity";
import { ThemeProvider, useTheme } from "./context/ThemeProvider";
import "./App.css";

import { RegionSelector, RegionDetectionResult } from "./components/RegionSelector";
import { FighterViewer } from "./components/FighterViewer";
import { ScriptViewer } from "./components/ScriptViewer";
import { ProjectManager } from "./components/ProjectManager";
import { FrameReconstructor } from "./components/FrameReconstructor";
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
import { BoxerWorkshop } from "./components/workshop/BoxerWorkshop";
import { PlayGame } from "./components/PlayGame";
import { WelcomeWorkspace } from "./components/WelcomeWorkspace";

import { KeyboardShortcutsHelp, HelpSystem } from "./components/help";
import { ToastContainer, showToast } from "./components/ToastContainer";
import { UpdateSettings } from "./components/UpdateSettings";
import { UpdateChecker } from "./components/UpdateChecker";
import { EmbeddedEmulator } from "./components/EmbeddedEmulator";
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

const RUNTIME_ERROR = import.meta.env.DEV
  ? "This page needs the desktop app. Run `npm run tauri dev` from apps/desktop, or add ?preview to the address to try the interface with sample data."
  : "This page needs the desktop app. Please start Super Punch-Out!! Editor from your computer.";

/** Remembers which ROM was open so the editor can reopen it next time. */
const LAST_ROM_STORAGE_KEY = "spo-editor-last-rom-path";

const readLastRomPath = (): string | null => {
  try {
    return localStorage.getItem(LAST_ROM_STORAGE_KEY);
  } catch {
    return null;
  }
};

const writeLastRomPath = (path: string | null): void => {
  try {
    if (path) localStorage.setItem(LAST_ROM_STORAGE_KEY, path);
    else localStorage.removeItem(LAST_ROM_STORAGE_KEY);
  } catch {
    // Remembering the last ROM is a convenience; the editor works without it.
  }
};

const fileNameOf = (path: string): string => path.split(/[\\/]/).pop() ?? path;

/** "Super Punch-Out!! (USA)" -> "USA". Falls back to the full name. */
const shortRegionLabel = (displayName: string | null | undefined): string | null => {
  if (!displayName) return null;
  return /\(([^)]+)\)\s*$/.exec(displayName)?.[1] ?? displayName;
};

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
    redoStack,
    pendingWrites,
    resumeSession,
    openRom,
    selectBoxer,
    getCurrentProject,
    undo,
    redo,
    setError,
    error,
  } = useStore();
  const { runtimeSkin, setRuntimeSkin, theme, setTheme } = useTheme();

  // Changes after every edit, undo and redo so previews know to redraw.
  const revisionKey = `${undoStack.length}:${redoStack.length}:${pendingWrites.size}`;

  const isDesktopRuntime = useMemo(() => isTauri(), []);

  const [showEmulatorSettings, setShowEmulatorSettings] = useState(false);
  const [showExternalTools, setShowExternalTools] = useState(false);
  const [showKeyboardShortcuts, setShowKeyboardShortcuts] = useState(false);
  const [showHelp, setShowHelp] = useState(false);
  const [helpContext, setHelpContext] = useState<string | undefined>(undefined);
  const [showRegionSelector, setShowRegionSelector] = useState(false);
  const [detectedRegion, setDetectedRegion] = useState<RegionDetectionResult | null>(null);
  const [romPath, setRomPath] = useState("");
  // False while a game is being opened; see the automatic safekeeping effect.
  const sessionReadyRef = useRef(false);
  // True once startup has confirmed that no game is open yet.
  const [needsStartupRom, setNeedsStartupRom] = useState(false);
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

  useEffect(() => {
    if (!isDesktopRuntime) {
      // The sidebar already explains how to start the desktop app.
      return;
    }

    void getCurrentProject();

    // If the backend still has a game open (the window was reloaded), pick up
    // where the user left off instead of showing the welcome screen.
    void (async () => {
      if (!(await resumeSession())) {
        setNeedsStartupRom(true);
        return;
      }
      sessionReadyRef.current = true;
      const resumed = useStore.getState();
      if (!resumed.selectedBoxer && resumed.boxers.length > 0) {
        void selectBoxer(resumed.boxers[0].key);
      }
      try {
        const path = await invoke<string | null>("get_rom_path");
        if (!path) return;
        setRomPath(path);
        setDetectedRegion(await invoke<RegionDetectionResult>("detect_rom_region", { romPath: path }));
      } catch (resumeError) {
        console.error("Could not restore ROM details:", resumeError);
      }
    })();
  }, [isDesktopRuntime, getCurrentProject, resumeSession, selectBoxer, setError]);

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

  // Automatic safekeeping: shortly after every edit, undo or redo, quietly keep
  // the changes in the app's own data folder so closing the editor never loses
  // work. They come back the next time the same game is opened.
  useEffect(() => {
    if (!isDesktopRuntime || !romSha1 || !sessionReadyRef.current) return;
    const timer = window.setTimeout(() => {
      if (!sessionReadyRef.current) return;
      void invoke("autosave_session").catch((keepError) =>
        console.error("Changes could not be kept automatically:", keepError)
      );
    }, 600);
    return () => window.clearTimeout(timer);
  }, [isDesktopRuntime, romSha1, revisionKey]);

  // After an edit, undo or redo, redraw the selected boxer's small picture so
  // the picker and header always match the current colors.
  useEffect(() => {
    const boxerKey = selectedBoxer?.key;
    if (!isDesktopRuntime || !romSha1 || !boxerKey) return;
    let isCancelled = false;

    void invoke<{ portrait_png: number[] | null; icon_png: number[] | null }>("get_runtime_theme_assets", {
      boxerKey,
    })
      .then((assets) => {
        if (isCancelled) return;
        const imageUrl = bytesToDataUrl(assets.portrait_png) ?? bytesToDataUrl(assets.icon_png);
        if (imageUrl) {
          setBoxerPortraits((current) =>
            current[boxerKey] === imageUrl ? current : { ...current, [boxerKey]: imageUrl }
          );
        }
      })
      .catch((portraitError) => console.error("Failed to refresh portrait:", portraitError));

    return () => {
      isCancelled = true;
    };
    // The portrait only needs redrawing when the edit revision changes.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [revisionKey]);

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

  /** Load the ROM, land on the first useful screen and pick a boxer so nothing is empty. */
  const finishOpenRom = useCallback(
    async (path: string): Promise<{ opened: boolean; restoredChanges: number }> => {
      // Automatic safekeeping pauses while a game is being opened so an empty,
      // half-loaded session can never overwrite the work kept from last time.
      sessionReadyRef.current = false;
      await openRom(path);
      setShowRegionSelector(false);

      const loaded = useStore.getState();
      if (!loaded.romSha1) return { opened: false, restoredChanges: 0 };
      writeLastRomPath(path);

      // Bring back any changes that were kept automatically for this exact game.
      let restoredChanges = 0;
      try {
        restoredChanges = await invoke<number>("restore_autosave");
        if (restoredChanges > 0) {
          await Promise.all([loaded.refreshUndoState(), loaded.refreshPendingWrites()]);
        }
      } catch (restoreError) {
        console.error("Kept work could not be restored:", restoreError);
      }
      sessionReadyRef.current = true;

      // Stable builds must never land inside a hidden experimental surface.
      const landingTab: TabKey = isFeatureVisible("roster") ? "roster" : "editor";
      setCurrentTab(landingTab);
      setLastNonModalTab(landingTab);

      if (!loaded.selectedBoxer && loaded.boxers.length > 0) {
        void selectBoxer(loaded.boxers[0].key);
      }
      return { opened: true, restoredChanges };
    },
    [openRom, selectBoxer]
  );

  /**
   * Check a chosen ROM. A game the editor fully recognizes opens straight
   * away; anything unusual falls back to the confirmation dialog.
   */
  const beginOpenRom = useCallback(
    async (path: string, options: { fromStartup?: boolean } = {}) => {
      setRomPath(path);
      try {
        const result = await invoke<RegionDetectionResult>("detect_rom_region", { romPath: path });
        setDetectedRegion(result);
        if (result.success && result.is_supported) {
          const { opened, restoredChanges } = await finishOpenRom(path);
          if (opened) {
            const greeting = options.fromStartup
              ? `Welcome back! ${fileNameOf(path)} is open again.`
              : `${result.display_name ?? "Your game"} is ready to edit!`;
            const kept =
              restoredChanges > 0
                ? ` Your ${restoredChanges} change${restoredChanges === 1 ? "" : "s"} from last time ${
                    restoredChanges === 1 ? "is" : "are"
                  } back.`
                : "";
            showToast(greeting + kept, "success", restoredChanges > 0 ? 7000 : 5000);
          }
          return;
        }
      } catch (detectError) {
        console.error("Automatic ROM check failed:", detectError);
      }

      if (options.fromStartup) {
        // The remembered file moved or is no longer a game the editor knows.
        // Forget it quietly and show the normal welcome screen.
        writeLastRomPath(null);
        setRomPath("");
        setError(null);
        return;
      }
      setShowRegionSelector(true);
    },
    [finishOpenRom, setError]
  );

  // Reopen the game that was open last time, so the editor starts where the
  // user left it.
  useEffect(() => {
    if (!needsStartupRom) return;
    setNeedsStartupRom(false);
    const lastPath = readLastRomPath();
    if (lastPath) void beginOpenRom(lastPath, { fromStartup: true });
  }, [needsStartupRom, beginOpenRom]);

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
        await beginOpenRom(selected);
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
    await finishOpenRom(romPath);
  }, [finishOpenRom, romPath]);

  // Dropping a ROM file anywhere on the window opens it.
  useEffect(() => {
    if (!isDesktopRuntime) return;
    let unlisten: (() => void) | undefined;
    let cancelled = false;

    void getCurrentWebview()
      .onDragDropEvent((event) => {
        if (event.payload.type !== "drop") return;
        const romFile = event.payload.paths.find((path) => /\.(sfc|smc)$/i.test(path));
        if (romFile) {
          void beginOpenRom(romFile);
        } else if (event.payload.paths.length > 0) {
          showToast("That is not a game file. Drop a .sfc or .smc file here.", "warning");
        }
      })
      .then((stop) => {
        if (cancelled) stop();
        else unlisten = stop;
      })
      .catch((dropError) => console.error("Drag and drop is unavailable:", dropError));

    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, [isDesktopRuntime, beginOpenRom]);

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
          <h2>Getting the boxers ready…</h2>
          <p>If nothing shows up, open your ROM again from the sidebar.</p>
        </div>
      );
    }

    return (
      <BoxerWorkshop
        boxer={selectedBoxer}
        boxers={boxers}
        boxerPortraits={boxerPortraits}
        revisionKey={revisionKey}
        changeCount={pendingWrites.size}
        onSelectBoxer={(boxerKey) => void selectBoxer(boxerKey)}
        onPlay={() => setCurrentTab("test")}
      />
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
          <div className="workspace-page">
            <LayoutPackBrowser onClose={handleCloseModalStyleTab} />
          </div>
        );
      case "roster":
        return (
          <div className="workspace-page">
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
          <div className="workspace-page">
            <div className="tab-close-header">
              <div>
                <p className="eyebrow">Settings</p>
                <h2 style={{ marginBottom: 0 }}>Settings</h2>
              </div>
              <button type="button" className="tab-close-button" onClick={handleCloseModalStyleTab}>
                Close
              </button>
            </div>
            <section className="card settings-look">
              <h3>How the editor looks</h3>
              <p>Pick the one that is easiest on your eyes.</p>
              <div className="settings-look-row" role="radiogroup" aria-label="Color theme">
                {(
                  [
                    ["light", "Light"],
                    ["dark", "Dark"],
                    ["system", "Match My Computer"],
                  ] as const
                ).map(([value, label]) => (
                  <button
                    key={value}
                    type="button"
                    role="radio"
                    aria-checked={theme === value}
                    className={theme === value ? "btn-primary" : "secondary"}
                    onClick={() => setTheme(value)}
                  >
                    {label}
                  </button>
                ))}
              </div>
            </section>
            <UpdateSettings />
          </div>
        );
      case "test":
        return (
          <PlayGame
            changeCount={undoStack.length}
            selectedBoxerKey={selectedBoxer?.key ?? null}
            startEmbedded={creatorAutoEnterToken > 0}
            embeddedEmulator={
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
            }
          />
        );
      case "plugins":
        return (
          <div className="workspace-page">
            <PluginManager isOpen={true} onClose={handleCloseModalStyleTab} />
          </div>
        );
      case "banks":
        return (
          <div className="workspace-page">
            <BankVisualization />
          </div>
        );
      case "animation-player":
        return (
          <div className="workspace-page">
            <AnimationPlayer />
          </div>
        );
      case "project":
        return (
          <div className="workspace-page">
            <ProjectManager />
          </div>
        );
      case "audio":
        return (
          <div className="workspace-page">
            <AudioEditor />
          </div>
        );
      case "text":
        return (
          <div className="workspace-page">
            <TextEditor />
          </div>
        );
      case "editor":
      default:
        return renderEditorContent();
    }
  };

  return (
    <div className="app-container">
      <GuidedSidebar
        tabItems={TAB_ITEMS}
        currentTab={currentTab}
        romSha1={romSha1}
        detectedRegionLabel={shortRegionLabel(detectedRegion?.display_name)}
        detectedRegionSupported={detectedRegion?.is_supported}
        currentProjectName={currentProject?.metadata?.name ?? null}
        runtimeIconUrl={runtimeSkin?.iconDataUrl ?? null}
        runtimeBoxerName={runtimeSkin?.boxerName ?? null}
        selectedBoxerKey={selectedBoxer?.key ?? null}
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
        onOpenHelp={() => {
          setHelpContext(currentTab === "editor" ? "palette-editor" : currentTab);
          setShowHelp(true);
        }}
        onOpenKeyboardShortcuts={() => setShowKeyboardShortcuts(true)}
        onOpenEmulatorSettings={() => setShowEmulatorSettings(true)}
        onOpenExternalTools={() => setShowExternalTools(true)}
      />

      <div className="main-column">
        <div className="ring-ropes" aria-hidden="true" />
        <main className="main-content">
          <AppRenderBoundary key={`${currentTab}:${selectedBoxer?.key ?? "none"}`}>
            {renderMainContent()}
          </AppRenderBoundary>
        </main>
      </div>

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
        <div className="dialog-backdrop" role="presentation">
          <div className="dialog-card" role="dialog" aria-modal="true" aria-labelledby="confirm-rom-title">
            <div className="dialog-card-header">
              <div>
                <p className="eyebrow">One quick check</p>
                <h2 id="confirm-rom-title">Checking your ROM</h2>
              </div>
              <button
                type="button"
                className="quiet-button dialog-close"
                onClick={() => setShowRegionSelector(false)}
                aria-label="Cancel ROM selection"
              >
                ×
              </button>
            </div>
            <div className="dialog-card-body">
              <p style={{ color: "var(--text-muted)", fontSize: "0.88rem" }}>
                The editor makes sure this is a game it knows before opening it. Your original file is never changed.
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
