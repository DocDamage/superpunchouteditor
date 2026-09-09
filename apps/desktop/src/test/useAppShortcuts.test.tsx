import { fireEvent, render, renderHook } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { useAppShortcuts } from "../hooks/useAppShortcuts";

function options() {
  return { enabled: true, canUndo: true, canRedo: true, onUndo: vi.fn(), onRedo: vi.fn(), onNavigate: vi.fn(), onHelp: vi.fn() };
}
function press(key: string, extra: KeyboardEventInit = {}, target: EventTarget = window) {
  const event = new KeyboardEvent("keydown", { key, ctrlKey: true, bubbles: true, cancelable: true, ...extra });
  target.dispatchEvent(event);
  return event;
}

describe("app keyboard ownership", () => {
  it("handles Undo and uppercase Ctrl+Shift+Z, including Command on macOS", () => {
    const actions = options(); renderHook(() => useAppShortcuts(actions));
    expect(press("z").defaultPrevented).toBe(true);
    press("Z", { shiftKey: true });
    press("z", { ctrlKey: false, metaKey: true });
    press("y");
    expect(actions.onUndo).toHaveBeenCalledTimes(2);
    expect(actions.onRedo).toHaveBeenCalledTimes(2);
  });
  it.each(["input", "textarea", "select", "contenteditable", "textbox"])("leaves native %s editing alone", (kind) => {
    const actions = options(); renderHook(() => useAppShortcuts(actions));
    const element = document.createElement(["input", "textarea", "select"].includes(kind) ? kind : "div");
    if (kind === "contenteditable") element.setAttribute("contenteditable", "true");
    if (kind === "textbox") element.setAttribute("role", "textbox");
    document.body.appendChild(element);
    expect(press("z", {}, element).defaultPrevented).toBe(false);
    expect(actions.onUndo).not.toHaveBeenCalled();
    element.remove();
  });
  it("does not steal keys when a modal owns focus", () => {
    const actions = options(); renderHook(() => useAppShortcuts(actions));
    render(<div role="dialog" aria-modal="true"><button>Cancel</button></div>);
    press("z");
    expect(actions.onUndo).not.toHaveBeenCalled();
  });
  it("ignores repeats, composing input, Alt shortcuts and previously handled keys", () => {
    const actions = options(); renderHook(() => useAppShortcuts(actions));
    press("z", { repeat: true }); press("z", { isComposing: true }); press("z", { altKey: true });
    const handled = new KeyboardEvent("keydown", { key: "z", ctrlKey: true, cancelable: true });
    handled.preventDefault(); window.dispatchEvent(handled);
    expect(actions.onUndo).not.toHaveBeenCalled();
  });
  it("respects disabled history and an inactive shortcut scope", () => {
    const actions = { ...options(), canUndo: false, canRedo: false };
    const { rerender } = renderHook((props) => useAppShortcuts(props), { initialProps: actions });
    press("z"); press("y");
    expect(actions.onUndo).not.toHaveBeenCalled(); expect(actions.onRedo).not.toHaveBeenCalled();
    rerender({ ...actions, enabled: false }); press("1");
    expect(actions.onNavigate).not.toHaveBeenCalled();
  });
  it("keeps navigation and help available outside inputs", () => {
    const actions = options(); renderHook(() => useAppShortcuts(actions));
    press("4"); fireEvent.keyDown(window, { key: "F1" });
    expect(actions.onNavigate).toHaveBeenCalledWith("test"); expect(actions.onHelp).toHaveBeenCalledOnce();
  });
});
