import { useState } from 'react';
import { save } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';
import { useStore } from '../store/useStore';
import { SaveIcon, StarIcon } from './icons';

type PatchFormat = 'ips' | 'bps';

type Status = { kind: 'ok' | 'error'; text: string } | null;

const fileNameOf = (path: string): string => path.split(/[\\/]/).pop() ?? path;

/**
 * Save & Share: write the edited game to a new file, or make a small patch
 * file that holds only the changes.
 */
export const ExportPanel = () => {
  const { romSha1, pendingWrites, undoStack, undo } = useStore();
  const [status, setStatus] = useState<Status>(null);
  const [isExporting, setIsExporting] = useState(false);
  const [selectedFormat, setSelectedFormat] = useState<PatchFormat>('bps');
  const [author, setAuthor] = useState('');
  const [description, setDescription] = useState('');

  // Count the user's changes (Undo steps), not the byte ranges they touch.
  const hasChanges = pendingWrites.size > 0;
  const changeCount = hasChanges ? Math.max(undoStack.length, 1) : 0;
  const canExport = !!romSha1;
  const canExportPatch = canExport && hasChanges;

  const handleSaveRomAs = async () => {
    if (!romSha1) return;
    const path = await save({
      filters: [{ name: 'SNES ROM', extensions: ['sfc', 'smc'] }],
      defaultPath: 'Super Punch-Out!! (My Version).sfc',
    });
    if (!path) return;
    setIsExporting(true);
    setStatus(null);
    try {
      await invoke('save_rom_as', { outputPath: path });
      setStatus({ kind: 'ok', text: `Saved! Your new game file is ${fileNameOf(path)}.` });
    } catch (e) {
      setStatus({ kind: 'error', text: `The game could not be saved. ${e}` });
    } finally {
      setIsExporting(false);
    }
  };

  /** Undo every change. Redo can still bring them back until a new change is made. */
  const handleStartOver = async () => {
    const count = undoStack.length;
    if (count === 0) return;
    const confirmed = window.confirm(
      `Take back all ${count} change${count === 1 ? '' : 's'} and go back to the original game?

` +
        'You can still press Redo to bring them back.',
    );
    if (!confirmed) return;
    setIsExporting(true);
    try {
      // Bounded by the number of changes so a failed Undo cannot loop forever.
      for (let step = 0; step < count && useStore.getState().canUndo; step += 1) {
        await undo();
      }
      setStatus({ kind: 'ok', text: 'All changes were taken back. The game is like the original again.' });
    } finally {
      setIsExporting(false);
    }
  };

  const handleExportPatch = async () => {
    if (!romSha1) return;
    const isBps = selectedFormat === 'bps';
    const path = await save({
      filters: [{ name: isBps ? 'BPS Patch' : 'IPS Patch', extensions: [selectedFormat] }],
      defaultPath: `My Super Punch-Out!! Changes.${selectedFormat}`,
    });
    if (!path) return;
    setIsExporting(true);
    setStatus(null);
    try {
      if (isBps) {
        await invoke<number>('export_bps_patch', {
          outputPath: path,
          author: author.trim() || null,
          description: description.trim() || null,
        });
      } else {
        await invoke<number>('export_ips_patch', { outputPath: path });
      }
      setStatus({ kind: 'ok', text: `Patch made! Share ${fileNameOf(path)} with your friends.` });
    } catch (e) {
      setStatus({ kind: 'error', text: `The patch could not be made. ${e}` });
    } finally {
      setIsExporting(false);
    }
  };

  return (
    <div className="export-panel">
      <div className="save-summary" data-has-changes={changeCount > 0}>
        {!romSha1
          ? 'Open your ROM first.'
          : changeCount === 0
            ? 'You have not changed anything yet. Try a new color on the Colors tab.'
            : `You have made ${changeCount} change${changeCount === 1 ? '' : 's'}. Nice work!`}
      </div>

      <div className="save-grid">
        <section className="save-card">
          <span className="save-card-icon pad-green" aria-hidden="true"><SaveIcon /></span>
          <h3>Save My Game</h3>
          <p>
            Makes a brand-new game file with all your changes. Your original game file is never touched.
          </p>
          <button
            id="btn-save-rom-as"
            type="button"
            className="btn-primary save-card-action"
            onClick={() => void handleSaveRomAs()}
            disabled={!canExport || isExporting}
          >
            {isExporting ? 'Saving…' : 'Save My Game'}
          </button>
        </section>

        <section className="save-card">
          <span className="save-card-icon pad-blue" aria-hidden="true"><StarIcon size={22} /></span>
          <h3>Share My Changes</h3>
          <p>
            Makes a tiny patch file with only your changes. Friends who own the game can add it to their copy.
            Never share the game file itself.
          </p>
          <button
            id={selectedFormat === 'ips' ? 'btn-export-ips' : 'btn-export-bps'}
            type="button"
            className="secondary save-card-action"
            onClick={() => void handleExportPatch()}
            disabled={!canExportPatch || isExporting}
            title={changeCount === 0 ? 'Change something first, then you can share it' : undefined}
          >
            {isExporting ? 'Working…' : 'Make a Patch File'}
          </button>

          <details className="save-options">
            <summary>Patch options</summary>
            <div className="save-options-body">
              <div className="save-format-row" role="radiogroup" aria-label="Patch type">
                <label>
                  <input
                    type="radio"
                    name="patch-format"
                    checked={selectedFormat === 'bps'}
                    onChange={() => setSelectedFormat('bps')}
                  />
                  <span>BPS <small>Best choice. Checks it is used on the right game.</small></span>
                </label>
                <label>
                  <input
                    type="radio"
                    name="patch-format"
                    checked={selectedFormat === 'ips'}
                    onChange={() => setSelectedFormat('ips')}
                  />
                  <span>IPS <small>Older type that more tools can open.</small></span>
                </label>
              </div>

              {selectedFormat === 'bps' && (
                <>
                  <label className="save-field">
                    Made by
                    <input
                      type="text"
                      value={author}
                      onChange={(e) => setAuthor(e.target.value)}
                      placeholder="Your name or nickname"
                      maxLength={64}
                    />
                  </label>
                  <label className="save-field">
                    What did you change?
                    <textarea
                      value={description}
                      onChange={(e) => setDescription(e.target.value)}
                      placeholder="For example: gold boxers!"
                      maxLength={256}
                      rows={2}
                    />
                  </label>
                </>
              )}
            </div>
          </details>
        </section>
      </div>

      {status && (
        <div className={`save-status ${status.kind}`} role="status">
          {status.text}
        </div>
      )}

      <div className="save-footnote">
        <p>
          Your changes are also kept automatically. Close the editor any time and they will be here when you
          open the same game again.
        </p>
        <button
          type="button"
          className="quiet-button"
          disabled={undoStack.length === 0 || isExporting}
          onClick={() => void handleStartOver()}
        >
          Start Over
        </button>
      </div>
    </div>
  );
};
