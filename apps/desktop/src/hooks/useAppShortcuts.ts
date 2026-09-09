import { useEffect } from "react";
import type { GuidedTabKey } from "../components/GuidedSidebar";

interface ShortcutOptions {
  enabled: boolean;
  canUndo: boolean;
  canRedo: boolean;
  onUndo: () => void;
  onRedo: () => void;
  onNavigate: (tab: GuidedTabKey) => void;
  onHelp: () => void;
}

/** Leave text editing, dialogs, IME composition, and handled game keys alone. */
export function ownsKeyboardInput(target: EventTarget | null): boolean {
  return target instanceof Element && Boolean(target.closest(
    'input, textarea, select, [contenteditable]:not([contenteditable="false"]), [role="textbox"], [role="dialog"], dialog',
  ));
}

export function useAppShortcuts(options: ShortcutOptions): void {
  const { enabled, canUndo, canRedo, onUndo, onRedo, onNavigate, onHelp } = options;
  useEffect(() => {
    if (!enabled) return;
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.defaultPrevented || event.isComposing || event.repeat || event.altKey) return;
      if (document.querySelector('dialog[open], [role="dialog"][aria-modal="true"]')) return;
      if (ownsKeyboardInput(event.target)) return;
      if (event.key === "F1") {
        event.preventDefault();
        onHelp();
        return;
      }
      if (!(event.ctrlKey || event.metaKey)) return;
      const key = event.key.toLowerCase();
      if (key === "z" && !event.shiftKey) {
        event.preventDefault();
        if (canUndo) onUndo();
      } else if ((key === "z" && event.shiftKey) || key === "y") {
        event.preventDefault();
        if (canRedo) onRedo();
      } else {
        const tabs: Record<string, GuidedTabKey> = {
          "1": "editor", "2": "viewer", "3": "project",
          "4": "test", "5": "compare", "0": "settings",
        };
        if (!event.shiftKey && tabs[key]) {
          event.preventDefault();
          onNavigate(tabs[key]);
        }
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [enabled, canUndo, canRedo, onUndo, onRedo, onNavigate, onHelp]);
}
