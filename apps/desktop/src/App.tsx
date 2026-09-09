import { useCallback, useEffect, useMemo, useRef, useState, type CSSProperties } from "react";
import { isTauri } from "@tauri-apps/api/core";
import { useStore } from "./store/useStore";
import { featureLabel, isFeatureVisible } from "./featureMaturity";
import { ThemeProvider } from "./context/ThemeProvider";
import "./App.css";
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
import { GuidedSidebar, type GuidedTabKey } from "./components/GuidedSidebar";
import { WelcomeWorkspace } from "./components/WelcomeWorkspace";
import { EditorWorkspace } from "./components/EditorWorkspace";
import { BoxerPicker } from "./components/BoxerPicker";
import { PanelBoundary } from "./components/PanelBoundary";
import { RomOpenDialog } from "./components/RomOpenDialog";
import { KeyboardShortcutsHelp, HelpSystem } from "./components/help";
import { ToastContainer } from "./components/ToastContainer";
import { UpdateSettings } from "./components/UpdateSettings";
import { UpdateChecker } from "./components/UpdateChecker";
import { EmbeddedEmulator } from "./components/EmbeddedEmulator";
import { useAppShortcuts } from "./hooks/useAppShortcuts";
import { useRomOpening } from "./hooks/useRomOpening";
import { useTestRomImage } from "./hooks/useTestRomImage";
import { useRuntimeArtwork } from "./hooks/useRuntimeArtwork";
import menuSheetUrl from "./assets/menu-fonts.png";
import "./styles/emulator.css";
import "./styles/clubhouse.css";

type TabKey = GuidedTabKey;
const MODAL_STYLE_TABS = new Set<TabKey>(["plugins", "packs", "test", "settings"]);
const ROM_OPTIONAL_TABS = new Set<TabKey>(["editor", "project", "settings"]);
const ALL_TAB_ITEMS: Array<{ key: TabKey; label: string }> = [
  { key: "roster", label: "Characters" }, { key: "editor", label: "Edit" },
  { key: "viewer", label: "Inspect" }, { key: "compare", label: "Compare" },
  { key: "test", label: "Test Game" }, { key: "project", label: "Projects" },
  { key: "scripts", label: "Scripts" }, { key: "animations", label: "Animations" },
  { key: "frames", label: "Frames" }, { key: "packs", label: "Packs" },
  { key: "ai", label: "AI" }, { key: "plugins", label: "Plugins" },
  { key: "banks", label: "Banks" }, { key: "animation-player", label: "Animation Player" },
  { key: "audio", label: "Audio" }, { key: "text", label: "Text" }, { key: "settings", label: "Settings" },
];
const TAB_ITEMS = ALL_TAB_ITEMS.filter(({ key }) => isFeatureVisible(key))
  .map(({ key, label }) => ({ key, label: featureLabel(label, key) }));
const RUNTIME_ERROR = "Desktop runtime not detected. Open the installed app, or run `npm run tauri dev` from apps/desktop.";
interface CreatorContext {
  boxerId?: number; boxerName?: string; circuit?: "Minor" | "Major" | "World" | "Special";
  unlockOrder?: number; introTextId?: number; assetOwnerKey?: string;
}

function App() {
  const { romSha1, boxers, selectedBoxer, currentProject, isProjectModified, canUndo, canRedo,
    undoStack, pendingWrites, loadBoxers, selectBoxer, getCurrentProject, undo, redo, setError, error } = useStore();
  const desktop = useMemo(() => isTauri(), []);
  const [dialog, setDialog] = useState<"emulator" | "external" | "shortcuts" | "help" | null>(null);
  const [helpContext, setHelpContext] = useState<string | undefined>();
  const [currentTab, setCurrentTab] = useState<TabKey>("editor");
  const [lastNonModalTab, setLastNonModalTab] = useState<TabKey>("editor");
  const [creatorToken, setCreatorToken] = useState(0);
  const [creatorContext, setCreatorContext] = useState<CreatorContext | null>(null);
  const [selectingBoxer, setSelectingBoxer] = useState(false);
  const selectionLock = useRef(false);
  const menuStyle = useMemo(() => ({ "--menu-sheet-image": `url("${menuSheetUrl}")` }) as CSSProperties, []);
  const { runtimeSkin, portraits } = useRuntimeArtwork(desktop, romSha1, selectedBoxer?.key ?? null, boxers);
  const onRomOpened = useCallback(() => { setCurrentTab("editor"); setLastNonModalTab("editor"); }, []);
  const romOpening = useRomOpening(desktop, onRomOpened);
  const testImage = useTestRomImage({ enabled: desktop && currentTab === "test", romSha1, pendingWrites, undoStack });
  const navigate = useCallback((tab: TabKey) => {
    if (!selectionLock.current && isFeatureVisible(tab) && (romSha1 || ROM_OPTIONAL_TABS.has(tab))) setCurrentTab(tab);
  }, [romSha1]);
  const openHelp = useCallback(() => {
    setHelpContext(currentTab === "editor" ? "palette-editor" : currentTab);
    setDialog("help");
  }, [currentTab]);
  const undoEdit = useCallback(() => { void undo(); }, [undo]);
  const redoEdit = useCallback(() => { void redo(); }, [redo]);
  useAppShortcuts({ enabled: !dialog && !romOpening.candidate && !romOpening.busy && currentTab !== "test",
    canUndo, canRedo, onUndo: undoEdit, onRedo: redoEdit, onNavigate: navigate, onHelp: openHelp });

  useEffect(() => {
    if (!desktop) return;
    void loadBoxers();
    void getCurrentProject();
  }, [desktop, loadBoxers, getCurrentProject]);
  useEffect(() => {
    if (!MODAL_STYLE_TABS.has(currentTab)) setLastNonModalTab(currentTab);
  }, [currentTab]);
  useEffect(() => { setCreatorContext(null); }, [romSha1]);

  const chooseBoxer = useCallback(async (key: string) => {
    if (selectionLock.current) return;
    selectionLock.current = true;
    setSelectingBoxer(true);
    setCurrentTab("editor");
    setError(null);
    try {
      await selectBoxer(key);
      if (useStore.getState().selectedBoxer?.key !== key) setError("That boxer could not be loaded. Choose another boxer or reopen your ROM.");
    } catch (selectError) { setError(String(selectError)); }
    finally { selectionLock.current = false; setSelectingBoxer(false); }
  }, [selectBoxer, setError]);
  const closePanel = useCallback(() => navigate(lastNonModalTab), [lastNonModalTab, navigate]);
  const launchCreator = useCallback((context?: CreatorContext) => {
    setCreatorContext(context ?? null);
    setCreatorToken((value) => value + 1);
    setCurrentTab("test");
  }, []);

  const welcome = () => <WelcomeWorkspace isDesktopRuntime={desktop} busy={romOpening.busy}
    onOpenRom={() => void romOpening.chooseRom()} onOpenProject={() => navigate("project")} onHelp={openHelp} />;
  const renderEditor = () => {
    if (!romSha1) return welcome();
    if (selectingBoxer) return <div className="club-empty-panel" role="status"><h2>Getting your corner ready…</h2><p>Loading the boxer and palette.</p></div>;
    if (!selectedBoxer) return <section className="club-choose-workspace">
      <p className="eyebrow">Round one · Choose your boxer</p><h1>Who is getting a new look?</h1>
      <p>Choose a character below. Colors are a great place to start.</p>
      <BoxerPicker boxers={boxers} portraits={portraits} onSelect={(key) => void chooseBoxer(key)} />
    </section>;
    return <EditorWorkspace key={selectedBoxer.key} boxer={selectedBoxer} portrait={portraits[selectedBoxer.key]}
      editCount={undoStack.length} projectName={currentProject?.metadata?.name ?? null} projectModified={isProjectModified}
      canUndo={canUndo} canRedo={canRedo} onUndo={undoEdit} onRedo={redoEdit}
      onTest={() => navigate("test")} onProject={() => navigate("project")} />;
  };

  const renderMainContent = () => {
    if (!romSha1 && !ROM_OPTIONAL_TABS.has(currentTab)) return welcome();
    switch (currentTab) {
      case "viewer": return <FighterViewer />;
      case "scripts": return <ScriptViewer />;
      case "animations": return <AnimationEditor />;
      case "compare": return <ComparisonView />;
      case "frames": return <FrameReconstructor />;
      case "packs": return <div className="club-legacy-panel"><LayoutPackBrowser onClose={closePanel} /></div>;
      case "roster": return <div className="club-legacy-panel"><RosterEditor mode="game" onLaunchCreatorTest={launchCreator} /></div>;
      case "ai": return <div className="club-legacy-panel club-tall-panel"><AIEditor /></div>;
      case "settings": return <div className="club-legacy-panel"><div className="tab-close-header">
        <h2>Settings</h2><button className="tab-close-button" onClick={closePanel}>Back to editing</button></div><UpdateSettings /></div>;
      case "test": return <div className="club-test-workspace">
        <div className="tab-close-header"><div><p className="eyebrow">Take it to the ring</p><h2>Test Current Revision</h2>
          <p>Your current working ROM, including journal edits not yet exported.</p></div>
          <button className="tab-close-button" onClick={closePanel}>Back to editing</button></div>
        {testImage.status === "loading" && <div className="club-empty-panel" role="status"><h3>Preparing your latest changes…</h3><p>The game starts only after the current ROM is ready.</p></div>}
        {testImage.status === "error" && <div className="club-empty-panel" role="alert"><h3>The test ROM could not be prepared</h3><p>{testImage.error}</p>
          <p>No older revision has been substituted.</p><button type="button" onClick={testImage.retry}>Try again</button></div>}
        {testImage.status === "ready" && testImage.data && <div className="club-test-content"><EmbeddedEmulator layout="tab"
          editedRomData={testImage.data} originalRomData={undefined} romPath={romOpening.active?.path ?? null}
          romName={currentProject?.metadata?.name || "Super Punch-Out!!"} autoEnterCreatorToken={creatorToken}
          creatorSessionContext={creatorContext} onOpenAssetOwner={(key) => void chooseBoxer(key)} /></div>}
      </div>;
      case "plugins": return <div className="club-legacy-panel"><PluginManager isOpen onClose={closePanel} /></div>;
      case "banks": return <div className="club-legacy-panel"><BankVisualization /></div>;
      case "animation-player": return <div className="club-legacy-panel"><AnimationPlayer /></div>;
      case "project": return <div className="club-legacy-panel"><ProjectManager /></div>;
      case "audio": return <div className="club-legacy-panel"><AudioEditor /></div>;
      case "text": return <div className="club-legacy-panel"><TextEditor /></div>;
      default: return renderEditor();
    }
  };

  return <div className={`app-container clubhouse-shell ${romSha1 ? "menu-sheet-enabled" : ""}`} style={menuStyle}>
    <a className="club-skip-link" href="#editor-workspace">Skip to workspace</a>
    <GuidedSidebar tabItems={TAB_ITEMS} currentTab={currentTab} romSha1={romSha1}
      detectedRegionLabel={romSha1 ? romOpening.active?.region?.display_name ?? null : null}
      detectedRegionSupported={romOpening.active?.region?.is_supported}
      currentProjectName={currentProject?.metadata?.name ?? null} runtimeIconUrl={runtimeSkin?.iconDataUrl ?? null}
      runtimeBoxerName={runtimeSkin?.boxerName ?? null} boxers={boxers} selectedBoxerKey={selectedBoxer?.key ?? null}
      boxerPortraits={portraits} canUndo={canUndo} canRedo={canRedo} editCount={undoStack.length}
      pendingWritesCount={pendingWrites.size} isDesktopRuntime={desktop} runtimeError={RUNTIME_ERROR} error={error}
      openingRom={romOpening.busy || Boolean(romOpening.candidate)} selectingBoxer={selectingBoxer}
      onDismissError={() => setError(null)} onOpenRom={() => void romOpening.chooseRom()} onUndo={undoEdit} onRedo={redoEdit}
      onNavigate={navigate} onSelectBoxer={(key) => void chooseBoxer(key)} onOpenHelp={openHelp}
      onOpenKeyboardShortcuts={() => setDialog("shortcuts")} onOpenEmulatorSettings={() => setDialog("emulator")}
      onOpenExternalTools={() => setDialog("external")} />
    <main className="main-content" id="editor-workspace" tabIndex={-1}>
      <PanelBoundary key={`${currentTab}:${selectedBoxer?.key ?? "none"}`}>{renderMainContent()}</PanelBoundary>
    </main>
    <EmulatorSettings isOpen={dialog === "emulator"} onClose={() => setDialog(null)} onSave={() => {}} />
    <ExternalToolsManager isOpen={dialog === "external"} onClose={() => setDialog(null)} />
    <KeyboardShortcutsHelp isOpen={dialog === "shortcuts"} onClose={() => setDialog(null)} />
    <HelpSystem isOpen={dialog === "help"} onClose={() => { setDialog(null); setHelpContext(undefined); }} initialContext={helpContext} />
    {romOpening.candidate && <RomOpenDialog path={romOpening.candidate.path} busy={romOpening.busy}
      error={romOpening.openingError} onCancel={romOpening.cancel} onDetected={romOpening.regionDetected} onConfirm={romOpening.confirmSelection} />}
  </div>;
}

export default function AppWithTheme() {
  return <ThemeProvider><UpdateChecker><App /><ToastContainer /></UpdateChecker></ThemeProvider>;
}
