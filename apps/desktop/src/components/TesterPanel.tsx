import { useEffect, useMemo, useState } from "react";
import { getVersion } from "@tauri-apps/api/app";

const STORAGE_KEY = "spo-editor-community-test-report-v1";

export const TEST_CHECKS = [
  { id: "installLaunch", label: "Installer completed and the app launched normally" },
  { id: "romLoad", label: "My own Super Punch-Out!! ROM loaded and was recognized" },
  { id: "edit", label: "I made one obvious edit and could see the changed value" },
  { id: "undoRedo", label: "Undo restored the original value and Redo restored my edit" },
  { id: "saveRom", label: "I saved an edited ROM to a NEW file (not over the original)" },
  { id: "project", label: "I saved/reopened a project and my edit returned correctly" },
  { id: "testGame", label: "Test Game used my current edited revision" },
  { id: "safeSave", label: "Cancel/overwrite protection behaved safely and predictably" },
  { id: "uninstallData", label: "If tested: uninstall kept my project/settings data" },
] as const;

export type TesterCheckId = (typeof TEST_CHECKS)[number]["id"];

export interface TesterReportState {
  testerName: string;
  appVersion: string;
  windowsVersion: string;
  installSource: string;
  checks: Record<TesterCheckId, boolean>;
  externalEmulator: "not-tested" | "pass" | "fail";
  easeOfUse: "" | "1" | "2" | "3" | "4" | "5";
  notes: string;
}

function emptyChecks(): Record<TesterCheckId, boolean> {
  return TEST_CHECKS.reduce(
    (result, check) => ({ ...result, [check.id]: false }),
    {} as Record<TesterCheckId, boolean>
  );
}

export function createEmptyTesterReport(): TesterReportState {
  return {
    testerName: "",
    appVersion: "",
    windowsVersion: "",
    installSource: "",
    checks: emptyChecks(),
    externalEmulator: "not-tested",
    easeOfUse: "",
    notes: "",
  };
}

export function buildTesterReport(state: TesterReportState): string {
  const completed = TEST_CHECKS.filter((check) => state.checks[check.id]).length;
  const lines = [
    "# Super Punch-Out!! Editor Community Test Report",
    "",
    `Tester: ${state.testerName || "Not provided"}`,
    `App version: ${state.appVersion || "Not provided"}`,
    `Windows version: ${state.windowsVersion || "Not provided"}`,
    `Installer/source: ${state.installSource || "Not provided"}`,
    `Ease of use: ${state.easeOfUse ? `${state.easeOfUse}/5` : "Not rated"}`,
    `Checklist: ${completed}/${TEST_CHECKS.length} completed`,
    "",
    "## Test checklist",
    ...TEST_CHECKS.map((check) => `- [${state.checks[check.id] ? "x" : " "}] ${check.label}`),
    `- External emulator: ${state.externalEmulator}`,
    "",
    "## Notes / bugs / confusing moments",
    state.notes.trim() || "None provided.",
    "",
    "## Privacy confirmation",
    "No ROM, SRAM/save-state, ROM path, or copyrighted game bytes are included in this report.",
  ];

  return lines.join("\n");
}

function loadSavedState(): TesterReportState {
  const empty = createEmptyTesterReport();

  try {
    const saved = localStorage.getItem(STORAGE_KEY);
    if (!saved) return empty;

    const parsed = JSON.parse(saved) as Partial<TesterReportState>;
    return {
      ...empty,
      ...parsed,
      checks: {
        ...empty.checks,
        ...(parsed.checks ?? {}),
      },
    };
  } catch {
    return empty;
  }
}

interface TesterPanelProps {
  isOpen: boolean;
  onClose: () => void;
  romLoaded: boolean;
  pendingWritesCount: number;
}

export function TesterPanel({
  isOpen,
  onClose,
  romLoaded,
  pendingWritesCount,
}: TesterPanelProps): React.ReactElement | null {
  const [state, setState] = useState<TesterReportState>(() => loadSavedState());
  const [copyStatus, setCopyStatus] = useState<"idle" | "copied" | "failed">("idle");

  useEffect(() => {
    if (state.appVersion) return;

    void getVersion()
      .then((version) => {
        setState((current) => (current.appVersion ? current : { ...current, appVersion: version }));
      })
      .catch(() => {
        // Version is optional in a community report; leave it blank if unavailable.
      });
  }, [state.appVersion]);

  useEffect(() => {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(state));
    } catch {
      // Local persistence is a convenience only; testing still works without it.
    }
  }, [state]);

  const report = useMemo(() => buildTesterReport(state), [state]);
  const completed = TEST_CHECKS.filter((check) => state.checks[check.id]).length;

  if (!isOpen) return null;

  const setField = <K extends keyof TesterReportState,>(key: K, value: TesterReportState[K]) => {
    setState((current) => ({ ...current, [key]: value }));
  };

  const setCheck = (id: TesterCheckId, checked: boolean) => {
    setState((current) => ({
      ...current,
      checks: { ...current.checks, [id]: checked },
    }));
  };

  const copyReport = async () => {
    try {
      await navigator.clipboard.writeText(report);
      setCopyStatus("copied");
    } catch {
      setCopyStatus("failed");
    }
  };

  const downloadReport = () => {
    const blob = new Blob([report], { type: "text/markdown;charset=utf-8" });
    const url = URL.createObjectURL(blob);
    const anchor = document.createElement("a");
    anchor.href = url;
    anchor.download = "super-punch-out-editor-test-report.md";
    anchor.click();
    URL.revokeObjectURL(url);
  };

  const resetReport = () => {
    setState(createEmptyTesterReport());
    setCopyStatus("idle");
  };

  return (
    <div className="tester-modal-backdrop" role="presentation" onMouseDown={onClose}>
      <section
        className="tester-modal"
        role="dialog"
        aria-modal="true"
        aria-labelledby="tester-panel-title"
        onMouseDown={(event) => event.stopPropagation()}
      >
        <header className="tester-modal-header">
          <div>
            <p className="eyebrow">Community testing</p>
            <h2 id="tester-panel-title">Tester Checklist</h2>
            <p>
              Work down this list, then copy or download the report. Your progress is saved locally on this PC.
            </p>
          </div>
          <button className="quiet-button" type="button" onClick={onClose} aria-label="Close tester checklist">
            Close
          </button>
        </header>

        <div className="tester-live-status" aria-live="polite">
          <strong>{completed}/{TEST_CHECKS.length} checks complete</strong>
          <span>{romLoaded ? "ROM loaded" : "ROM not loaded yet"}</span>
          <span>{pendingWritesCount > 0 ? `${pendingWritesCount} changed range(s)` : "No current edits"}</span>
        </div>

        <div className="tester-warning">
          <strong>Do not send ROMs or save states.</strong> Reports should contain observations, screenshots, hashes, and non-copyrighted logs only.
        </div>

        <div className="tester-fields-grid">
          <label>
            Tester name (optional)
            <input
              type="text"
              value={state.testerName}
              onChange={(event) => setField("testerName", event.target.value)}
              placeholder="Name or handle"
            />
          </label>
          <label>
            App version
            <input
              type="text"
              value={state.appVersion}
              onChange={(event) => setField("appVersion", event.target.value)}
              placeholder="e.g. 2.0.0"
            />
          </label>
          <label>
            Windows version
            <input
              type="text"
              value={state.windowsVersion}
              onChange={(event) => setField("windowsVersion", event.target.value)}
              placeholder="e.g. Windows 11 24H2"
            />
          </label>
          <label>
            Installer/source
            <input
              type="text"
              value={state.installSource}
              onChange={(event) => setField("installSource", event.target.value)}
              placeholder="Tester kit / commit / filename"
            />
          </label>
        </div>

        <fieldset className="tester-checklist">
          <legend>Core smoke test</legend>
          {TEST_CHECKS.map((check) => (
            <label key={check.id} className="tester-check-row">
              <input
                type="checkbox"
                checked={state.checks[check.id]}
                onChange={(event) => setCheck(check.id, event.target.checked)}
              />
              <span>{check.label}</span>
            </label>
          ))}
        </fieldset>

        <div className="tester-fields-grid">
          <label>
            External emulator (optional)
            <select
              value={state.externalEmulator}
              onChange={(event) => setField("externalEmulator", event.target.value as TesterReportState["externalEmulator"])}
            >
              <option value="not-tested">Not tested</option>
              <option value="pass">Pass</option>
              <option value="fail">Fail</option>
            </select>
          </label>
          <label>
            How easy was the editor to understand?
            <select
              value={state.easeOfUse}
              onChange={(event) => setField("easeOfUse", event.target.value as TesterReportState["easeOfUse"])}
            >
              <option value="">Not rated</option>
              <option value="1">1 - Very confusing</option>
              <option value="2">2 - Difficult</option>
              <option value="3">3 - Usable</option>
              <option value="4">4 - Easy</option>
              <option value="5">5 - Extremely intuitive</option>
            </select>
          </label>
        </div>

        <label className="tester-notes-field">
          Bugs, confusion, or suggestions
          <textarea
            value={state.notes}
            onChange={(event) => setField("notes", event.target.value)}
            rows={6}
            placeholder="What happened? What did you expect? What felt confusing? Include steps to reproduce when possible."
          />
        </label>

        <details className="tester-report-preview">
          <summary>Preview report</summary>
          <textarea readOnly value={report} rows={14} aria-label="Generated tester report" />
        </details>

        <footer className="tester-actions">
          <button type="button" onClick={() => void copyReport()}>
            Copy Report
          </button>
          <button type="button" className="secondary" onClick={downloadReport}>
            Download .md
          </button>
          <button type="button" className="quiet-button" onClick={resetReport}>
            Reset
          </button>
          <span className="tester-copy-status" aria-live="polite">
            {copyStatus === "copied" && "Copied to clipboard."}
            {copyStatus === "failed" && "Clipboard access failed; use Download .md or copy the preview manually."}
          </span>
        </footer>
      </section>
    </div>
  );
}
