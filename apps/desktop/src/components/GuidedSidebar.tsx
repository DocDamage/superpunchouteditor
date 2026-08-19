import { useEffect, useMemo, useState } from "react";
import { ThemeToggle } from "./ThemeToggle";
import { TesterPanel } from "./TesterPanel";
import "./Usability.css";

export type GuidedTabKey =
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
  | "text"
  | "test"
  | "settings";

interface NavigationItem {
  key: GuidedTabKey;
  label: string;
}

interface BoxerSummary {
  key: string;
  name: string;
}

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

const WORKFLOW_ORDER: GuidedTabKey[] = [
  "roster",
  "editor",
  "viewer",
  "compare",
  "test",
  "project",
];

const FRIENDLY_META: Partial<Record<GuidedTabKey, { label: string; description: string }>> = {
  roster: { label: "Characters", description: "Choose or create a boxer" },
  editor: { label: "Edit & Export", description: "Change palettes, sprites, and assets" },
  viewer: { label: "Inspect", description: "Look through ROM assets safely" },
  compare: { label: "Compare", description: "Review exactly what changed" },
  test: { label: "Test Game", description: "Run the current edited revision" },
  project: { label: "Projects", description: "Save and reopen your work" },
  settings: { label: "Settings", description: "Updates and app preferences" },
};

const ADVANCED_STORAGE_KEY = "spo-editor-show-advanced-tools";

function loadAdvancedPreference(): boolean {
  try {
    return localStorage.getItem(ADVANCED_STORAGE_KEY) === "true";
  } catch {
    return false;
  }
}

export function GuidedSidebar({
  tabItems,
  currentTab,
  romSha1,
  detectedRegionLabel,
  detectedRegionSupported,
  currentProjectName,
  runtimeIconUrl,
  runtimeBoxerName,
  boxers,
  selectedBoxerKey,
  boxerPortraits,
  canUndo,
  canRedo,
  editCount,
  pendingWritesCount,
  isDesktopRuntime,
  runtimeError,
  error,
  onOpenRom,
  onUndo,
  onRedo,
  onNavigate,
  onSelectBoxer,
  onOpenHelp,
  onOpenKeyboardShortcuts,
  onOpenEmulatorSettings,
  onOpenExternalTools,
}: GuidedSidebarProps): React.ReactElement {
  const [showAdvanced, setShowAdvanced] = useState(loadAdvancedPreference);
  const [showTesterPanel, setShowTesterPanel] = useState(false);

  const visibleKeys = useMemo(() => new Set(tabItems.map((item) => item.key)), [tabItems]);
  const workflowItems = useMemo(
    () =>
      WORKFLOW_ORDER.filter((key) => visibleKeys.has(key)).map((key) => {
        const original = tabItems.find((item) => item.key === key)!;
        return { ...original, ...(FRIENDLY_META[key] ?? {}) };
      }),
    [tabItems, visibleKeys]
  );
  const advancedItems = useMemo(
    () =>
      tabItems.filter(
        (item) => !WORKFLOW_ORDER.includes(item.key) && item.key !== "settings"
      ),
    [tabItems]
  );
  const settingsVisible = visibleKeys.has("settings");
  const currentIsAdvanced = advancedItems.some((item) => item.key === currentTab);

  useEffect(() => {
    if (currentIsAdvanced) setShowAdvanced(true);
  }, [currentIsAdvanced]);

  useEffect(() => {
    try {
      localStorage.setItem(ADVANCED_STORAGE_KEY, String(showAdvanced));
    } catch {
      // Navigation still works when localStorage is unavailable.
    }
  }, [showAdvanced]);

  const nextAction = !selectedBoxerKey
    ? {
        title: "Choose what to edit",
        detail: "Pick a boxer, then make one small change.",
        label: "Go to Edit",
        action: () => onNavigate("editor"),
      }
    : pendingWritesCount === 0
      ? {
          title: "Make one change",
          detail: "Try a palette edit first; Undo is always available.",
          label: "Edit Boxer",
          action: () => onNavigate("editor"),
        }
      : {
          title: "Review your changes",
          detail: "Your current revision is ready to test.",
          label: "Test Game",
          action: () => onNavigate("test"),
        };

  return (
    <aside className="sidebar guided-sidebar" aria-label="Editor navigation">
      <div className="guided-brand-row">
        <div className="guided-brand">
          {runtimeIconUrl && (
            <img src={runtimeIconUrl} alt="" className="sidebar-brand-icon" aria-hidden="true" />
          )}
          <div>
            <div className="guided-app-name">Super Punch-Out!! Editor</div>
            <div className="guided-app-subtitle">Windows editor</div>
            {runtimeBoxerName && <div className="auth-mode-label">Theme: {runtimeBoxerName}</div>}
          </div>
        </div>
        <ThemeToggle variant="minimal" size="small" />
      </div>

      {!isDesktopRuntime && <div className="runtime-warning">{runtimeError}</div>}
      {error && <div className="error-banner" role="alert">{error}</div>}

      <button
        type="button"
        className="guided-open-rom"
        onClick={onOpenRom}
        disabled={!isDesktopRuntime}
      >
        <span>{romSha1 ? "Switch ROM" : "Open ROM"}</span>
        <small>{romSha1 ? "Choose a different local file" : "Use your own .sfc or .smc file"}</small>
      </button>

      {romSha1 && (
        <div className="guided-session-card">
          <div className="guided-session-status">
            <span className="guided-status-dot" aria-hidden="true" />
            <strong>ROM loaded</strong>
          </div>
          <div className="guided-session-meta" title={`ROM SHA-1: ${romSha1}`}>
            SHA-1 {romSha1.slice(0, 8)}…
            {detectedRegionLabel && (
              <span className={detectedRegionSupported === false ? "guided-region warning" : "guided-region"}>
                {detectedRegionLabel}
              </span>
            )}
          </div>
          {currentProjectName && <div className="guided-session-meta">Project: {currentProjectName}</div>}
          <div className="guided-undo-row">
            <button type="button" className="secondary" onClick={onUndo} disabled={!canUndo} title="Undo (Ctrl+Z)">
              Undo
            </button>
            <button type="button" className="secondary" onClick={onRedo} disabled={!canRedo} title="Redo (Ctrl+Y)">
              Redo
            </button>
            <span>{editCount} edit{editCount === 1 ? "" : "s"}</span>
          </div>
        </div>
      )}

      {romSha1 && (
        <section className="guided-next-card" aria-label="Suggested next step">
          <p className="eyebrow">Next step</p>
          <strong>{nextAction.title}</strong>
          <p>{nextAction.detail}</p>
          <button type="button" onClick={nextAction.action}>{nextAction.label}</button>
        </section>
      )}

      <nav className="guided-nav" aria-label="Main workflow">
        <div className="guided-section-title">Main workflow</div>
        {workflowItems.map((item) => (
          <button
            key={item.key}
            type="button"
            className={`guided-nav-button ${currentTab === item.key ? "active" : ""}`}
            onClick={() => onNavigate(item.key)}
            aria-current={currentTab === item.key ? "page" : undefined}
          >
            <span>{item.label}</span>
            <small>{item.description}</small>
          </button>
        ))}
      </nav>

      {currentTab === "editor" && romSha1 && boxers.length > 0 && (
        <section className="guided-boxer-section" aria-label="Choose boxer">
          <div className="guided-section-title">Choose boxer</div>
          <ul className="boxer-list guided-boxer-list">
            {boxers.map((boxer) => (
              <li key={boxer.key}>
                <button
                  type="button"
                  className={`boxer-item guided-boxer-button ${selectedBoxerKey === boxer.key ? "active" : ""}`}
                  onClick={() => onSelectBoxer(boxer.key)}
                  aria-current={selectedBoxerKey === boxer.key ? "true" : undefined}
                >
                  {boxerPortraits[boxer.key] && (
                    <img src={boxerPortraits[boxer.key]} alt="" className="boxer-item-portrait" aria-hidden="true" />
                  )}
                  <span>{boxer.name}</span>
                </button>
              </li>
            ))}
          </ul>
        </section>
      )}

      {advancedItems.length > 0 && (
        <section className="guided-advanced-section">
          <button
            type="button"
            className="guided-advanced-toggle"
            onClick={() => setShowAdvanced((value) => !value)}
            aria-expanded={showAdvanced}
          >
            <span>Advanced tools</span>
            <small>{showAdvanced ? "Hide" : `${advancedItems.length} available`}</small>
          </button>
          {showAdvanced && (
            <div className="guided-advanced-grid">
              {advancedItems.map((item) => (
                <button
                  type="button"
                  key={item.key}
                  className={currentTab === item.key ? "active" : ""}
                  onClick={() => onNavigate(item.key)}
                >
                  {item.label}
                </button>
              ))}
            </div>
          )}
        </section>
      )}

      <div className="guided-sidebar-footer">
        <button type="button" className="guided-tester-button" onClick={() => setShowTesterPanel(true)}>
          Tester Checklist
          <small>Record bugs and usability feedback</small>
        </button>

        <div className="guided-utility-grid">
          {settingsVisible && (
            <button type="button" className={currentTab === "settings" ? "active" : ""} onClick={() => onNavigate("settings")}>
              Settings
            </button>
          )}
          <button type="button" onClick={onOpenHelp}>Help</button>
          <button type="button" onClick={onOpenKeyboardShortcuts}>Shortcuts</button>
          {romSha1 && <button type="button" onClick={onOpenEmulatorSettings}>Emulator</button>}
          {romSha1 && <button type="button" onClick={onOpenExternalTools}>External Tools</button>}
        </div>
      </div>

      <TesterPanel
        isOpen={showTesterPanel}
        onClose={() => setShowTesterPanel(false)}
        romLoaded={Boolean(romSha1)}
        pendingWritesCount={pendingWritesCount}
      />
    </aside>
  );
}
