import { useEffect, useRef } from "react";
import { RegionSelector, type RegionDetectionResult } from "./RegionSelector";

interface Props {
  path: string;
  busy: boolean;
  error: string | null;
  onCancel: () => void;
  onDetected: (region: RegionDetectionResult) => void;
  onConfirm: () => Promise<void>;
}

export function RomOpenDialog({ path, busy, error, onCancel, onDetected, onConfirm }: Props) {
  const dialog = useRef<HTMLDialogElement>(null);
  const returnFocus = useRef(document.activeElement);
  useEffect(() => {
    const element = dialog.current;
    const previousFocus = returnFocus.current;
    element?.showModal();
    return () => {
      element?.close();
      if (previousFocus instanceof HTMLElement && previousFocus.isConnected) previousFocus.focus();
    };
  }, []);

  return (
    <dialog ref={dialog} className="club-rom-dialog" aria-labelledby="confirm-rom-title"
      aria-describedby="confirm-rom-description" aria-modal="true"
      onCancel={(event) => { event.preventDefault(); if (!busy) onCancel(); }}>
      <header className="club-dialog-header">
        <div><p className="eyebrow">One quick check</p><h2 id="confirm-rom-title">Confirm ROM Region</h2></div>
        <button type="button" className="secondary" onClick={onCancel} disabled={busy}
          aria-label="Cancel ROM selection" autoFocus>Cancel</button>
      </header>
      <p id="confirm-rom-description">Check this file before starting. Selecting a file does not replace your current session until you confirm.</p>
      {error && <p role="alert" className="error-banner">{error} Cancel to choose another file, or try again.</p>}
      <fieldset disabled={busy} aria-busy={busy} className="club-region-fields">
        <RegionSelector romPath={path} onRegionDetected={onDetected} onRegionSelected={onConfirm} />
      </fieldset>
      {busy && <p role="status">Opening your ROM…</p>}
    </dialog>
  );
}
