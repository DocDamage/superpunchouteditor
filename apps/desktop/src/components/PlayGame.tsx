import { useCallback, useEffect, useState } from "react";
import type { ReactNode } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { PlayIcon } from "./icons";
import "./PlayGame.css";

interface EmulatorSettingsRecord {
  emulator_path: string;
  emulator_type: string;
  auto_save_before_launch: boolean;
  command_line_args: string;
  jump_to_selected_boxer: boolean;
  default_round: number;
  save_state_dir: string | null;
}

interface FoundEmulator {
  path: string;
  emulator_type: string;
  name: string;
  folder: string;
}

interface PlayGameProps {
  /** How many changes are waiting to be tried. */
  changeCount: number;
  selectedBoxerKey: string | null;
  /** The in-editor emulator, shown as an advanced option. */
  embeddedEmulator: ReactNode;
  /** Open the in-editor emulator straight away (used by the boxer creator). */
  startEmbedded?: boolean;
}

type Phase = "checking" | "ready" | "found" | "missing";

const fileNameOf = (path: string): string => path.split(/[\\/]/).pop() ?? path;

const folderNameOf = (folder: string): string => {
  const parts = folder.split(/[\\/]/).filter(Boolean);
  return parts.slice(-2).join(" › ") || folder;
};

const guessType = (path: string): string => {
  const name = fileNameOf(path).toLowerCase();
  if (name.startsWith("snes9x")) return "snes9x";
  if (name.startsWith("bsnes") || name.startsWith("higan")) return "bsnes";
  if (name.startsWith("mesen")) return "mesen-s";
  return "other";
};

const TYPE_NAMES: Record<string, string> = {
  snes9x: "Snes9x",
  bsnes: "bsnes",
  "mesen-s": "Mesen",
  other: "your emulator",
};

/**
 * Play Game: run the current edited game in an emulator with one button.
 * The editor looks for an emulator already on the computer so the user does
 * not have to know where it is.
 */
export function PlayGame({
  changeCount,
  selectedBoxerKey,
  embeddedEmulator,
  startEmbedded = false,
}: PlayGameProps): React.ReactElement {
  const [phase, setPhase] = useState<Phase>("checking");
  const [settings, setSettings] = useState<EmulatorSettingsRecord | null>(null);
  const [found, setFound] = useState<FoundEmulator[]>([]);
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState<{ kind: "ok" | "error"; text: string } | null>(null);
  const [showEmbedded, setShowEmbedded] = useState(startEmbedded);

  const lookForEmulators = useCallback(async () => {
    try {
      const candidates = await invoke<FoundEmulator[]>("find_installed_emulators");
      setFound(candidates);
      setPhase(candidates.length > 0 ? "found" : "missing");
    } catch (searchError) {
      console.error("Emulator search failed:", searchError);
      setFound([]);
      setPhase("missing");
    }
  }, []);

  useEffect(() => {
    let cancelled = false;
    void (async () => {
      try {
        const current = await invoke<EmulatorSettingsRecord>("get_emulator_settings");
        if (cancelled) return;
        setSettings(current);
        if (current.emulator_path) {
          const check = await invoke<{ valid: boolean }>("verify_emulator");
          if (cancelled) return;
          if (check.valid) {
            setPhase("ready");
            return;
          }
        }
      } catch (settingsError) {
        console.error("Could not read emulator settings:", settingsError);
      }
      if (!cancelled) await lookForEmulators();
    })();
    return () => {
      cancelled = true;
    };
  }, [lookForEmulators]);

  const chooseEmulator = async (path: string, emulatorType: string) => {
    setBusy(true);
    setMessage(null);
    try {
      const next: EmulatorSettingsRecord = {
        emulator_path: path,
        emulator_type: emulatorType,
        auto_save_before_launch: settings?.auto_save_before_launch ?? true,
        command_line_args: settings?.command_line_args ?? "",
        jump_to_selected_boxer: settings?.jump_to_selected_boxer ?? true,
        default_round: settings?.default_round ?? 1,
        save_state_dir: settings?.save_state_dir ?? null,
      };
      await invoke("set_emulator_settings", { settings: next });
      setSettings(next);
      setPhase("ready");
    } catch (saveError) {
      setMessage({ kind: "error", text: `That program could not be used. ${saveError}` });
    } finally {
      setBusy(false);
    }
  };

  const pickEmulator = async () => {
    const selected = await open({
      multiple: false,
      title: "Pick your SNES emulator program",
      filters: [{ name: "Programs", extensions: ["exe", "app", "AppImage", "*"] }],
    });
    if (typeof selected === "string") {
      await chooseEmulator(selected, guessType(selected));
    }
  };

  const play = async () => {
    setBusy(true);
    setMessage(null);
    try {
      await invoke("test_in_emulator", {
        autoSave: true,
        quickLoadSlot: null,
        boxerKey: selectedBoxerKey,
        round: 1,
      });
      setMessage({
        kind: "ok",
        text: "Your game is opening in a new window. Made more changes? Press Play again to see them.",
      });
    } catch (playError) {
      setMessage({ kind: "error", text: `The game could not start. ${playError}` });
    } finally {
      setBusy(false);
    }
  };

  const emulatorName = settings ? TYPE_NAMES[settings.emulator_type] ?? "your emulator" : "your emulator";

  return (
    <div className="workspace-page play-page">
      <header className="play-header">
        <p className="eyebrow">Play Game</p>
        <h2>Try your version</h2>
        <p>
          This plays the game with every change you have made so far, even the ones you have not saved yet.
        </p>
      </header>

      <section className="play-card" aria-live="polite">
        {phase === "checking" && <p className="play-card-note">Getting things ready…</p>}

        {phase === "ready" && settings && (
          <>
            <button type="button" className="play-big-button" onClick={() => void play()} disabled={busy}>
              <span className="play-big-icon" aria-hidden="true"><PlayIcon size={34} /></span>
              <span>
                {busy ? "Starting…" : "Play My Game"}
                <small>
                  {changeCount > 0
                    ? `With your ${changeCount} change${changeCount === 1 ? "" : "s"}`
                    : "You have not changed anything yet"}
                </small>
              </span>
            </button>
            <p className="play-card-note">
              Opens in {emulatorName} ({fileNameOf(settings.emulator_path)}).{" "}
              <button type="button" className="guided-link-button" onClick={() => void lookForEmulators()}>
                Use a different emulator
              </button>
            </p>
          </>
        )}

        {phase === "found" && (
          <>
            <h3>Good news! An emulator is already on this computer.</h3>
            <p className="play-card-note">
              An emulator is the app that runs the game. Pick the one to use. The editor remembers your choice.
            </p>
            <ul className="play-found-list">
              {found.map((candidate) => (
                <li key={candidate.path}>
                  <div>
                    <strong>{candidate.name}</strong>
                    <span title={candidate.path}>
                      {fileNameOf(candidate.path)} in {folderNameOf(candidate.folder)}
                    </span>
                  </div>
                  <button
                    type="button"
                    className="btn-primary"
                    disabled={busy}
                    onClick={() => void chooseEmulator(candidate.path, candidate.emulator_type)}
                  >
                    Use This One
                  </button>
                </li>
              ))}
            </ul>
            <button type="button" className="secondary" onClick={() => void pickEmulator()} disabled={busy}>
              Pick a Different Program…
            </button>
          </>
        )}

        {phase === "missing" && (
          <>
            <h3>One more thing is needed to play</h3>
            <p className="play-card-note">
              An emulator is the app that runs the game. The editor could not find one on this computer.
              Ask a grown-up to install a Super NES emulator such as Snes9x, then show the editor where it is.
            </p>
            <div className="play-card-actions">
              <button type="button" className="btn-primary" onClick={() => void pickEmulator()} disabled={busy}>
                Show Where My Emulator Is…
              </button>
              <button type="button" className="secondary" onClick={() => void lookForEmulators()} disabled={busy}>
                Look Again
              </button>
            </div>
          </>
        )}

        {message && (
          <div className={`save-status ${message.kind === "error" ? "error" : ""}`} role="status">
            {message.text}
          </div>
        )}
      </section>

      <details
        className="play-embedded"
        open={showEmbedded}
        onToggle={(event) => setShowEmbedded((event.target as HTMLDetailsElement).open)}
      >
        <summary>
          <span>Play inside the editor</span>
          <small>Advanced. Needs an extra emulator file called a Snes9x libretro core.</small>
        </summary>
        {showEmbedded && <div className="play-embedded-body">{embeddedEmulator}</div>}
      </details>
    </div>
  );
}
