import { useEffect, useRef, useState } from "react";
import type { ReactElement } from "react";
import type { BoxerRecord } from "../../store/useStore";
import { AssetManager } from "../AssetManager";
import { BoxerPreviewSheet } from "../BoxerPreviewSheet";
import { ExportPanel } from "../ExportPanel";
import { PatchNotesGenerator } from "../PatchNotesGenerator";
import { SpriteBinEditor } from "../SpriteBinEditor";
import { BoxersIcon, PaintIcon, LookIcon, SaveIcon, ToolsIcon, PlayIcon } from "../icons";
import { ColorStudio } from "./ColorStudio";
import { FaceStamp } from "./FaceStamp";
import { PicturesPanel } from "./PicturesPanel";
import "./Workshop.css";

type WorkshopTab = "colors" | "photo" | "pictures" | "save" | "expert";

interface BoxerSummary {
  key: string;
  name: string;
}

interface BoxerWorkshopProps {
  boxer: BoxerRecord;
  boxers: BoxerSummary[];
  boxerPortraits: Record<string, string>;
  /** Changes whenever an edit, undo or redo happens. */
  revisionKey: string;
  changeCount: number;
  onSelectBoxer: (boxerKey: string) => void;
  onPlay: () => void;
}

const TABS: Array<{ key: WorkshopTab; label: string; hint: string; icon: ReactElement }> = [
  { key: "colors", label: "Colors", hint: "Repaint the boxer", icon: <PaintIcon size={20} /> },
  { key: "photo", label: "Photo Face", hint: "Put a photo on the boxer", icon: <BoxersIcon size={20} /> },
  { key: "pictures", label: "Pictures", hint: "Swap the menu face", icon: <LookIcon size={20} /> },
  { key: "save", label: "Save & Share", hint: "Keep your work", icon: <SaveIcon size={20} /> },
  { key: "expert", label: "Expert Tools", hint: "For advanced editing", icon: <ToolsIcon size={20} /> },
];

// The workshop is remounted when another boxer is picked; remember the open tab.
let rememberedTab: WorkshopTab = "colors";

export function BoxerWorkshop({
  boxer,
  boxers,
  boxerPortraits,
  revisionKey,
  changeCount,
  onSelectBoxer,
  onPlay,
}: BoxerWorkshopProps): ReactElement {
  const [tab, setTab] = useState<WorkshopTab>(rememberedTab);
  const activeChipRef = useRef<HTMLButtonElement>(null);
  const portrait = boxerPortraits[boxer.key];

  useEffect(() => {
    rememberedTab = tab;
  }, [tab]);

  useEffect(() => {
    activeChipRef.current?.scrollIntoView?.({ block: "nearest", inline: "center" });
  }, [boxer.key]);

  return (
    <div className="boxer-detail boxer-workshop">
      <section className="boxer-picker" aria-label="Choose boxer">
        <p className="eyebrow">Pick a boxer</p>
        <div className="boxer-picker-strip">
          {boxers.map((candidate) => {
            const isActive = candidate.key === boxer.key;
            return (
              <button
                key={candidate.key}
                ref={isActive ? activeChipRef : undefined}
                type="button"
                className={`boxer-chip ${isActive ? "active" : ""}`}
                onClick={() => onSelectBoxer(candidate.key)}
                aria-current={isActive ? "true" : undefined}
              >
                {boxerPortraits[candidate.key] ? (
                  <img src={boxerPortraits[candidate.key]} alt="" aria-hidden="true" />
                ) : (
                  <span className="boxer-chip-initial" aria-hidden="true">{candidate.name.charAt(0)}</span>
                )}
                <span>{candidate.name}</span>
              </button>
            );
          })}
        </div>
      </section>

      <header className="fight-card">
        {portrait && <img src={portrait} alt="" className="fight-card-portrait" />}
        <div className="fight-card-body">
          <p className="eyebrow">Now editing</p>
          <h2>{boxer.name}</h2>
          <p>Change one thing at a time. Undo in the sidebar takes back a change.</p>
        </div>
        <button type="button" className="fight-card-play" onClick={onPlay}>
          <PlayIcon size={22} />
          <span>
            Play Game
            <small>{changeCount > 0 ? "Try your changes" : "See the game"}</small>
          </span>
        </button>
      </header>

      <div className="workshop-tabs" role="tablist" aria-label={`Edit ${boxer.name}`}>
        {TABS.map((item) => (
          <button
            key={item.key}
            type="button"
            role="tab"
            id={`workshop-tab-${item.key}`}
            aria-selected={tab === item.key}
            aria-controls="workshop-panel"
            className={`workshop-tab ${tab === item.key ? "active" : ""}`}
            onClick={() => setTab(item.key)}
          >
            {item.icon}
            <span>
              {item.label}
              <small>{item.hint}</small>
            </span>
          </button>
        ))}
      </div>

      <div
        id="workshop-panel"
        role="tabpanel"
        aria-labelledby={`workshop-tab-${tab}`}
        className="workshop-panel"
      >
        {tab === "colors" && <ColorStudio boxer={boxer} revisionKey={revisionKey} />}

        {tab === "photo" && <FaceStamp boxer={boxer} revisionKey={revisionKey} />}

        {tab === "pictures" && <PicturesPanel boxer={boxer} revisionKey={revisionKey} />}

        {tab === "save" && <ExportPanel />}

        {tab === "expert" && (
          <div className="expert-stack">
            <div className="workshop-tip">
              <strong>These tools are for experienced editors.</strong> They show the game&apos;s raw picture data.
              You do not need them to recolor a boxer or swap a picture.
            </div>
            <section className="workspace-section framed">
              <BoxerPreviewSheet boxer={boxer} />
            </section>
            <section className="workspace-section framed">
              <AssetManager boxer={boxer} />
            </section>
            <section className="workspace-section framed">
              <SpriteBinEditor boxer={boxer} />
            </section>
            <section className="workspace-section framed">
              <PatchNotesGenerator />
            </section>
          </div>
        )}
      </div>
    </div>
  );
}
