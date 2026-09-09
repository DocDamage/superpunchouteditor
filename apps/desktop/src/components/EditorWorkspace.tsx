import type { BoxerRecord } from "../store/useStore";
import { PaletteEditor } from "./PaletteEditor";
import { BoxerPreviewSheet } from "./BoxerPreviewSheet";
import { SpriteBinEditor } from "./SpriteBinEditor";
import { AssetManager } from "./AssetManager";
import { ExportPanel } from "./ExportPanel";
import { PatchNotesGenerator } from "./PatchNotesGenerator";
import { EditorSections } from "./EditorSections";

interface Props {
  boxer: BoxerRecord;
  portrait?: string;
  editCount: number;
  projectName: string | null;
  projectModified: boolean;
  canUndo: boolean;
  canRedo: boolean;
  onUndo: () => void;
  onRedo: () => void;
  onTest: () => void;
  onProject: () => void;
}

export function EditorWorkspace(props: Props) {
  const { boxer, portrait, editCount, projectName, projectModified, canUndo, canRedo, onUndo, onRedo, onTest, onProject } = props;
  return (
    <div className="club-editor-workspace">
      <header className="club-editor-header">
        <div className="club-boxer-heading">
          {portrait && <img src={portrait} alt="" className="club-hero-portrait" />}
          <div><p className="eyebrow">In your corner</p><h1>{boxer.name}</h1><p>Pick a tool. Make it yours. Take it into the ring.</p></div>
        </div>
        <div className="club-session-badges">
          <span className="club-badge">{editCount} journal edit{editCount === 1 ? "" : "s"}</span>
          <span className="club-badge">{projectName ? (projectModified ? "Project has unsaved changes" : "Project open") : "No project file yet"}</span>
        </div>
      </header>
      <div className="club-action-bar" aria-label="Editing actions">
        <div className="club-history-actions">
          <button type="button" className="secondary" disabled={!canUndo} onClick={onUndo} title="Undo (Ctrl+Z)">↶ Undo</button>
          <button type="button" className="secondary" disabled={!canRedo} onClick={onRedo} title="Redo (Ctrl+Shift+Z or Ctrl+Y)">↷ Redo</button>
        </div>
        <div className="club-primary-actions">
          <button type="button" className="secondary" onClick={onProject}>Save / Open Project</button>
          <button type="button" className="club-play-button" onClick={onTest}><span aria-hidden="true">▶</span> Test Game</button>
        </div>
      </div>
      <EditorSections panels={{
        colors: <><div className="club-tool-intro"><h2>Give your boxer a new look</h2><p>Choose a palette, then a color. Try a small change first; Undo brings you back.</p></div><PaletteEditor /></>,
        sprites: <><div className="club-tool-intro"><h2>See the fighter, then edit the pixels</h2><p>The preview helps you find your artwork. Shared sprite data can affect more than one boxer.</p></div><div className="club-content-card"><BoxerPreviewSheet boxer={boxer} /></div><div className="club-content-card"><SpriteBinEditor boxer={boxer} /></div></>,
        assets: <><div className="club-tool-intro"><h2>Your artwork toolbox</h2><p>Move artwork between the ROM and your image editor. Check shared-asset warnings before importing.</p></div><AssetManager boxer={boxer} /></>,
        export: <><div className="club-tool-intro"><h2>Take your changes into the ring</h2><p>Test your revision, then export a new ROM file. A project file is for continuing your work; an edited ROM is for playing it.</p></div><ExportPanel /><details className="club-details"><summary>Write patch notes</summary><PatchNotesGenerator /></details></>,
      }} />
      <details className="club-details">
        <summary>Technical asset details</summary>
        <dl className="club-asset-summary">
          <div><dt>Boxer ID</dt><dd>{boxer.key}</dd></div>
          <div><dt>Palettes</dt><dd>{boxer.palette_files.length}</dd></div>
          <div><dt>Icons</dt><dd>{boxer.icon_files.length}</dd></div>
          <div><dt>Unique sprite bins</dt><dd>{boxer.unique_sprite_bins.length}</dd></div>
          <div><dt>Shared sprite bins</dt><dd>{boxer.shared_sprite_bins.length}</dd></div>
        </dl>
      </details>
    </div>
  );
}
