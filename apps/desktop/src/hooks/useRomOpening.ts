import { useCallback, useEffect, useRef, useState } from "react";
import { confirm, open } from "@tauri-apps/plugin-dialog";
import { useStore } from "../store/useStore";
import type { RegionDetectionResult } from "../components/RegionSelector";

interface RomSelection { path: string; region: RegionDetectionResult | null }

/** File selection is provisional until openRom actually establishes a session. */
export function useRomOpening(isDesktopRuntime: boolean, onOpened: () => void) {
  const [candidate, setCandidate] = useState<RomSelection | null>(null);
  const [active, setActive] = useState<RomSelection | null>(null);
  const [busy, setBusy] = useState(false);
  const [openingError, setOpeningError] = useState<string | null>(null);
  const locked = useRef(false);
  const candidateRef = useRef<RomSelection | null>(null);
  const opener = useRef<HTMLElement | null>(null);

  // Capture focus before disabling the opener for the asynchronous native picker.
  // Restore only after React has re-enabled controls and removed any modal.
  useEffect(() => {
    if (busy || candidate || !opener.current) return;
    const element = opener.current;
    opener.current = null;
    if (element.isConnected) element.focus();
  }, [busy, candidate]);

  const changeCandidate = useCallback((value: RomSelection | null) => {
    candidateRef.current = value;
    setCandidate(value);
  }, []);

  const chooseRom = useCallback(async () => {
    if (locked.current || candidateRef.current) return;
    if (!isDesktopRuntime) {
      useStore.getState().setError("Open the desktop app to choose a local ROM.");
      return;
    }
    opener.current = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    locked.current = true;
    setBusy(true);
    try {
      const state = useStore.getState();
      if (state.romSha1 && (state.canUndo || state.isProjectModified || state.pendingWrites.size > 0)) {
        const proceed = await confirm(
          "Opening another ROM ends this editing session. Save your project or export your edited ROM first if you need to keep this work. Continue?",
          { title: "Switch ROM?", kind: "warning", okLabel: "Switch ROM", cancelLabel: "Keep editing" },
        );
        if (!proceed) return;
      }
      const path = await open({ multiple: false, filters: [{ name: "SNES ROM", extensions: ["sfc", "smc"] }] });
      if (typeof path === "string") {
        setOpeningError(null);
        changeCandidate({ path, region: null });
      }
    } catch (error) {
      useStore.getState().setError(String(error));
    } finally {
      locked.current = false;
      setBusy(false);
    }
  }, [isDesktopRuntime, changeCandidate]);

  const cancel = useCallback(() => {
    if (locked.current) return;
    changeCandidate(null);
    setOpeningError(null);
  }, [changeCandidate]);

  const regionDetected = useCallback((region: RegionDetectionResult) => {
    const selection = candidateRef.current;
    if (selection) changeCandidate({ ...selection, region });
  }, [changeCandidate]);

  const confirmSelection = useCallback(async () => {
    const selection = candidateRef.current;
    if (!selection || locked.current) return;
    locked.current = true;
    setBusy(true);
    setOpeningError(null);
    try {
      await useStore.getState().openRom(selection.path);
      // openRom reports failures through the store rather than rejecting.
      const state = useStore.getState();
      if (!state.romSha1) throw new Error(state.error ?? "The ROM could not be opened.");
      setActive(selection);
      await state.getCurrentProject();
      changeCandidate(null);
      onOpened();
    } catch (error) {
      setOpeningError(String(error));
    } finally {
      locked.current = false;
      setBusy(false);
    }
  }, [changeCandidate, onOpened]);

  return { candidate, active, busy, openingError, chooseRom, cancel, regionDetected, confirmSelection };
}
