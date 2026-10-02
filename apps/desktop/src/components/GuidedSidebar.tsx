import { useEffect, useMemo, useState } from "react";
import type { ReactElement } from "react";
import { ThemeToggle } from "./ThemeToggle";
import { TesterPanel } from "./TesterPanel";
import {
  BoxersIcon,
  ChecklistIcon,
  CompareIcon,
  GearIcon,
  GloveIcon,
  HelpIcon,
  LookIcon,
  PaintIcon,
  PlayIcon,
  RedoIcon,
  SaveIcon,
  ToolsIcon,
  UndoIcon,
} from "./icons";
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

interface GuidedSidebarProps {
  tabItems: NavigationItem[];
  currentTab: GuidedTabKey;
  romSha1: string | null;
  detectedRegionLabel?: string | null;
  detectedRegionSupported?: boolean;
  currentProjectName?: string | null;
  runtimeIconUrl?: string | null;
  runtimeBoxerName?: string | null;
  selectedBoxerKey?: string | null;
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

type PadColor = "red" | "yellow" | "green" | "blue";

interface FriendlyMeta {
  label: string;
  description: string;
  icon: ReactElement;
  color: PadColor;
}

/** Plain-language names for the main menu. A child should understand each one. */
const FRIENDLY_META: Partial<Record<GuidedTabKey, FriendlyMeta>> = {
  roster: { label: "Boxers", description: "Pick or make a boxer", icon: <BoxersIcon />, color: "blue" },
  editor: { label: "Edit Boxers", description: "Change colors and pictures", icon: <PaintIcon />, color: "red" },
  viewer: { label: "Look Around", description: "See every boxer and pose", icon: <LookIcon />, color: "blue" },
  compare: { label: "What Changed", description: "See before and after", icon: <CompareIcon />, color: "yellow" },
  test: { label: "Play Game", description: "Try your changes right now", icon: <PlayIcon />, color: "green" },
  project: { label: "My Projects", description: "Save your work for later", icon: <SaveIcon />, color: "yellow" },
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
  selectedBoxerKey,
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
        return { ...original, ...(FRIENDLY_META[key] ?? {}) } as NavigationItem & Partial<FriendlyMeta>;
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
        title: "Pick a boxer",
        detail: "Choose who you want to change.",
        label: "Pick a Boxer",
        tab: "editor" as GuidedTabKey,
      }
    : pendingWritesCount === 0
      ? {
          title: "Change something",
          detail: "Tap a Magic Paint color. You can always undo it.",
          label: "Edit Boxer",
          tab: "editor" as GuidedTabKey,
        }
      : {
          title: "Try it out!",
          detail: "Play the game with your changes, then save your work.",
          label: "Play Now",
          tab: "test" as GuidedTabKey,
        };

  return (
    <aside className="sidebar guided-sidebar" aria-label="Editor navigation">
      <div className="guided-brand-row">
        <div className="guided-brand" role="img" aria-label="Super Punch-Out!! Editor">
          {runtimeIconUrl ? (
            <img src={runtimeIconUrl} alt="" className="sidebar-brand-icon" aria-hidden="true" />
          ) : (
            <span className="guided-brand-mark" aria-hidden="true">
              <GloveIcon size={26} />
            </span>
          )}
          <div className="guided-brand-text" aria-hidden="true">
            <div className="guided-app-kicker">Super Punch-Out!!</div>
            <div className="guided-app-name">Editor</div>
          </div>
        </div>
        <ThemeToggle variant="minimal" size="small" />
      </div>

      {!isDesktopRuntime && <div className="runtime-warning">{runtimeError}</div>}
      {error && <div className="error-banner" role="alert">{error}</div>}

      {!romSha1 && (
        <button
          type="button"
          className="guided-open-rom"
          onClick={onOpenRom}
          disabled={!isDesktopRuntime}
        >
          <span>Open ROM</span>
          <small>Pick your own game file to start</small>
        </button>
      )}

      {romSha1 && (
        <div className="guided-session-card">
          <div className="guided-session-status">
            <span className="guided-status-dot" aria-hidden="true" />
            <strong>ROM loaded</strong>
            {detectedRegionLabel && (
              <span className={detectedRegionSupported === false ? "guided-region warning" : "guided-region"}>
                {detectedRegionLabel}
              </span>
            )}
          </div>
          {runtimeBoxerName && <div className="guided-session-meta">Editing {runtimeBoxerName}</div>}
          {currentProjectName && <div className="guided-session-meta">Project: {currentProjectName}</div>}
          <div className="guided-undo-row">
            <button type="button" className="secondary" onClick={onUndo} disabled={!canUndo} title="Undo (Ctrl+Z)">
              <UndoIcon size={16} /> Undo
            </button>
            <button type="button" className="secondary" onClick={onRedo} disabled={!canRedo} title="Redo (Ctrl+Y)">
              <RedoIcon size={16} /> Redo
            </button>
          </div>
          <div className="guided-session-foot">
            <span>{editCount} change{editCount === 1 ? "" : "s"}</span>
            <span className="guided-fingerprint" title={`ROM fingerprint (SHA-1): ${romSha1}`}>
              ID {romSha1.slice(0, 8)}
            </span>
            <button
              type="button"
              className="guided-link-button"
              onClick={onOpenRom}
              disabled={!isDesktopRuntime}
            >
              Switch ROM
            </button>
          </div>
        </div>
      )}

      {romSha1 && (
        <section className="guided-next-card" aria-label="Suggested next step">
          <p className="eyebrow">Next step</p>
          <strong>{nextAction.title}</strong>
          <p>{nextAction.detail}</p>
          {nextAction.tab !== currentTab && (
            <button type="button" className="btn-primary" onClick={() => onNavigate(nextAction.tab)}>
              {nextAction.label}
            </button>
          )}
        </section>
      )}

      <nav className="guided-nav" aria-label="Main workflow">
        <div className="guided-section-title">Menu</div>
        {workflowItems.map((item) => (
          <button
            key={item.key}
            type="button"
            className={`guided-nav-button ${currentTab === item.key ? "active" : ""}`}
            onClick={() => onNavigate(item.key)}
            aria-current={currentTab === item.key ? "page" : undefined}
          >
            <span className={`guided-nav-icon pad-${item.color ?? "blue"}`} aria-hidden="true">
              {item.icon ?? <ToolsIcon />}
            </span>
            <span className="guided-nav-text">
              <span>{item.label}</span>
              {item.description && <small>{item.description}</small>}
            </span>
          </button>
        ))}
      </nav>

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
        <div className="guided-utility-grid">
          {settingsVisible && (
            <button type="button" className={currentTab === "settings" ? "active" : ""} onClick={() => onNavigate("settings")}>
              <GearIcon size={16} /> Settings
            </button>
          )}
          <button type="button" onClick={onOpenHelp}><HelpIcon size={16} /> Help</button>
          <button type="button" onClick={onOpenKeyboardShortcuts}>Shortcuts</button>
          {romSha1 && <button type="button" onClick={onOpenEmulatorSettings}><PlayIcon size={16} /> Emulator</button>}
          {romSha1 && <button type="button" onClick={onOpenExternalTools}><ToolsIcon size={16} /> Other Apps</button>}
          <button type="button" onClick={() => setShowTesterPanel(true)}>
            <ChecklistIcon size={16} /> Tester Checklist
          </button>
        </div>
        <p className="guided-legal">
          Unofficial fan-made tool. Not affiliated with or endorsed by Nintendo.
        </p>
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
