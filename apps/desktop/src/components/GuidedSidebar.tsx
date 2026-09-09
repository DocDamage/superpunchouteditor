import { useEffect, useMemo, useState } from "react";
import { ThemeToggle } from "./ThemeToggle";
import { TesterPanel } from "./TesterPanel";
import { BoxerPicker } from "./BoxerPicker";
import "./Usability.css";

export type GuidedTabKey = "editor" | "viewer" | "scripts" | "animations" | "frames" | "compare"
  | "project" | "packs" | "roster" | "ai" | "plugins" | "banks" | "animation-player" | "audio" | "text" | "test" | "settings";
interface NavigationItem { key: GuidedTabKey; label: string }
interface BoxerSummary { key: string; name: string }
interface GuidedSidebarProps {
  tabItems: NavigationItem[];
  currentTab: GuidedTabKey;
  romSha1: string | null;
  detectedRegionLabel?: string | null;
  detectedRegionSupported?: boolean;
  currentProjectName?: string | null;
  runtimeIconUrl?: string | null;
  runtimeBoxerName?: string | null;
  boxers: BoxerSummary[];
  selectedBoxerKey?: string | null;
  boxerPortraits: Record<string, string>;
  canUndo: boolean;
  canRedo: boolean;
  editCount: number;
  pendingWritesCount: number;
  isDesktopRuntime: boolean;
  runtimeError: string;
  error: string | null;
  openingRom?: boolean;
  selectingBoxer?: boolean;
  onDismissError?: () => void;
  onOpenRom: () => void;
  onUndo: () => void;
  onRedo: () => void;
  onNavigate: (tab: GuidedTabKey) => void;
  onSelectBoxer: (boxerKey: string) => void;
  onOpenHelp: () => void;
  onOpenKeyboardShortcuts: () => void;
  onOpenEmulatorSettings: () => void;
  onOpenExternalTools: () => void;
}
const WORKFLOW_ORDER: GuidedTabKey[] = ["roster", "editor", "viewer", "compare", "test", "project"];
const ROM_OPTIONAL = new Set<GuidedTabKey>(["editor", "project", "settings"]);
const META: Partial<Record<GuidedTabKey, { label: string; description: string }>> = {
  roster: { label: "Characters", description: "Choose or create a boxer" },
  editor: { label: "Edit & Export", description: "Colors, sprites, artwork & exports" },
  viewer: { label: "Inspect", description: "Look through ROM assets safely" },
  compare: { label: "Compare", description: "Review what changed" },
  test: { label: "Test Game", description: "Play your current edited revision" },
  project: { label: "Projects", description: "Save or continue your work" },
};
const ADVANCED_KEY = "spo-editor-show-advanced-tools";
function loadAdvancedPreference() {
  try { return localStorage.getItem(ADVANCED_KEY) === "true"; } catch { return false; }
}

export function GuidedSidebar(props: GuidedSidebarProps) {
  const { tabItems, currentTab, romSha1, detectedRegionLabel, detectedRegionSupported, currentProjectName,
    runtimeIconUrl, runtimeBoxerName, boxers, selectedBoxerKey, boxerPortraits, canUndo, canRedo, editCount,
    pendingWritesCount, isDesktopRuntime, runtimeError, error, openingRom = false, selectingBoxer = false,
    onDismissError, onOpenRom, onUndo, onRedo, onNavigate, onSelectBoxer, onOpenHelp, onOpenKeyboardShortcuts,
    onOpenEmulatorSettings, onOpenExternalTools } = props;
  const [showAdvanced, setShowAdvanced] = useState(loadAdvancedPreference);
  const [showTesterPanel, setShowTesterPanel] = useState(false);
  const visibleKeys = useMemo(() => new Set(tabItems.map((item) => item.key)), [tabItems]);
  const workflowItems = useMemo(() => WORKFLOW_ORDER.filter((key) => visibleKeys.has(key)).map((key) => {
    const original = tabItems.find((item) => item.key === key)!;
    const meta = META[key];
    return { key, label: `${meta?.label ?? original.label}${meta && original.label.includes("(Experimental)") ? " (Experimental)" : ""}`,
      description: meta?.description ?? "" };
  }), [tabItems, visibleKeys]);
  const advancedItems = useMemo(() => tabItems.filter((item) => !WORKFLOW_ORDER.includes(item.key) && item.key !== "settings"), [tabItems]);
  const currentIsAdvanced = advancedItems.some((item) => item.key === currentTab);
  const disabled = (key: GuidedTabKey) => selectingBoxer || openingRom || (!romSha1 && !ROM_OPTIONAL.has(key)) || (!isDesktopRuntime && key !== "editor");

  useEffect(() => { if (currentIsAdvanced) setShowAdvanced(true); }, [currentIsAdvanced]);
  useEffect(() => {
    try { localStorage.setItem(ADVANCED_KEY, String(showAdvanced)); } catch { /* Optional preference only. */ }
  }, [showAdvanced]);

  return <aside className="sidebar guided-sidebar club-sidebar" aria-label="Editor navigation">
    <div className="guided-brand-row"><div className="guided-brand">
      {runtimeIconUrl ? <img src={runtimeIconUrl} alt="" className="sidebar-brand-icon" /> : <span className="club-brand-mark" aria-hidden="true">SP</span>}
      <div><div className="guided-app-name">Super Punch-Out!!</div><div className="guided-app-subtitle">EDITOR · YOUR CREATIVE CORNER</div>
        {runtimeBoxerName && <div className="auth-mode-label">Theme: {runtimeBoxerName}</div>}</div>
    </div><ThemeToggle variant="minimal" size="small" /></div>
    {!isDesktopRuntime && <div className="runtime-warning">{runtimeError}</div>}
    {error && <div className="error-banner" role="alert"><p>{error}</p>
      {onDismissError && <button type="button" className="secondary" onClick={onDismissError}>Dismiss message</button>}</div>}
    <button type="button" className="guided-open-rom" onClick={onOpenRom} disabled={!isDesktopRuntime || openingRom || selectingBoxer}>
      <span>{openingRom ? "Opening ROM…" : romSha1 ? "Switch ROM" : "Open ROM"}</span>
      <small>{romSha1 ? "Choose another local file" : "Your own .sfc or .smc file"}</small>
    </button>
    {romSha1 && <div className="guided-session-card">
      <div className="guided-session-status"><span className="guided-status-dot" aria-hidden="true" /><strong>ROM ready</strong>
        {detectedRegionLabel && <span className={detectedRegionSupported === false ? "guided-region warning" : "guided-region"}>{detectedRegionLabel}</span>}</div>
      {currentProjectName && <div className="guided-session-meta" title={currentProjectName}>Project: {currentProjectName}</div>}
      <div className="guided-undo-row"><button type="button" className="secondary" onClick={onUndo} disabled={!canUndo || selectingBoxer || openingRom} title="Undo (Ctrl+Z)">Undo</button>
        <button type="button" className="secondary" onClick={onRedo} disabled={!canRedo || selectingBoxer || openingRom} title="Redo (Ctrl+Y or Ctrl+Shift+Z)">Redo</button>
        <span>{editCount} edit{editCount === 1 ? "" : "s"}</span></div>
      <details className="club-session-details"><summary>Session details</summary><p>ROM SHA-1: {romSha1}</p><p>{pendingWritesCount} changed range{pendingWritesCount === 1 ? "" : "s"}. This is not a saved-status indicator.</p></details>
    </div>}
    <nav className="guided-nav" aria-label="Main workflow"><div className="guided-section-title">Your toolbox</div>
      {workflowItems.map((item, index) => <button key={item.key} type="button"
        className={`guided-nav-button ${currentTab === item.key ? "active" : ""}`} disabled={disabled(item.key)}
        onClick={() => onNavigate(item.key)} aria-current={currentTab === item.key ? "page" : undefined}>
        <span className="club-nav-marker" aria-hidden="true">{index + 1}</span>
        <span className="club-nav-copy"><strong>{item.label}</strong><small>{!romSha1 && !ROM_OPTIONAL.has(item.key) ? "Open a ROM first" : item.description}</small></span>
      </button>)}
    </nav>
    {currentTab === "editor" && romSha1 && selectedBoxerKey && boxers.length > 0 &&
      <BoxerPicker compact boxers={boxers} selectedKey={selectedBoxerKey} portraits={boxerPortraits} onSelect={onSelectBoxer} busy={selectingBoxer || openingRom} />}
    {romSha1 && !selectedBoxerKey && <section className="guided-next-card" aria-label="Suggested next step">
      <p className="eyebrow">Your next move</p><strong>Choose a boxer</strong><p>The character cards are in Edit & Export. Pick one to get started.</p>
      <button type="button" onClick={() => onNavigate("editor")} disabled={disabled("editor")}>Choose Boxer</button>
    </section>}
    {advancedItems.length > 0 && <section className="guided-advanced-section">
      <button type="button" className="guided-advanced-toggle" onClick={() => setShowAdvanced((value) => !value)} aria-expanded={showAdvanced} aria-controls="advanced-tool-list">
        <span>Advanced tools</span><small>{showAdvanced ? "Hide" : `${advancedItems.length} available`}</small></button>
      {showAdvanced && <div className="guided-advanced-grid" id="advanced-tool-list">{advancedItems.map((item) => <button type="button" key={item.key}
        className={currentTab === item.key ? "active" : ""} disabled={disabled(item.key)} aria-current={currentTab === item.key ? "page" : undefined}
        onClick={() => onNavigate(item.key)}>{item.label}</button>)}</div>}
    </section>}
    <div className="guided-sidebar-footer"><button type="button" className="guided-tester-button" onClick={() => setShowTesterPanel(true)}>
      Tester Checklist<small>Share feedback, not your ROM</small></button>
      <div className="guided-utility-grid">
        {visibleKeys.has("settings") && <button type="button" className={currentTab === "settings" ? "active" : ""} disabled={disabled("settings")} onClick={() => onNavigate("settings")}>Settings</button>}
        <button type="button" onClick={onOpenHelp}>Help</button><button type="button" onClick={onOpenKeyboardShortcuts}>Shortcuts</button>
        {romSha1 && <button type="button" onClick={onOpenEmulatorSettings}>Emulator</button>}
        {romSha1 && <button type="button" onClick={onOpenExternalTools}>External Tools</button>}
      </div>
    </div>
    <TesterPanel isOpen={showTesterPanel} onClose={() => setShowTesterPanel(false)} romLoaded={Boolean(romSha1)} pendingWritesCount={pendingWritesCount} />
  </aside>;
}
