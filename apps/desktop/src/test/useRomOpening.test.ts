import { act, renderHook } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { useRomOpening } from "../hooks/useRomOpening";
const mocks = vi.hoisted(() => ({ open: vi.fn(), confirm: vi.fn(), openRom: vi.fn(), getCurrentProject: vi.fn(), setError: vi.fn() }));
vi.mock("@tauri-apps/plugin-dialog", () => ({ open: mocks.open, confirm: mocks.confirm }));
vi.mock("../store/useStore", () => ({ useStore: { getState: () => state } }));
let state: {
  romSha1: string | null; canUndo: boolean; isProjectModified: boolean; pendingWrites: Set<string>; error: string | null;
  openRom: typeof mocks.openRom; getCurrentProject: typeof mocks.getCurrentProject; setError: typeof mocks.setError;
};
beforeEach(() => {
  Object.values(mocks).forEach((mock) => mock.mockReset());
  state = { romSha1: null, canUndo: false, isProjectModified: false, pendingWrites: new Set(), error: null,
    openRom: mocks.openRom, getCurrentProject: mocks.getCurrentProject, setError: mocks.setError };
  mocks.openRom.mockImplementation(async () => { state.romSha1 = "loaded-sha"; });
  mocks.getCurrentProject.mockResolvedValue(undefined);
  mocks.confirm.mockResolvedValue(true);
});

describe("provisional ROM selection", () => {
  it("commits the active path only after an explicitly confirmed successful open", async () => {
    const opened = vi.fn(); mocks.open.mockResolvedValue("C:/first.sfc");
    const { result } = renderHook(() => useRomOpening(true, opened));
    await act(async () => result.current.chooseRom());
    expect(result.current.active).toBeNull(); expect(mocks.openRom).not.toHaveBeenCalled();
    await act(async () => result.current.confirmSelection());
    expect(result.current.active?.path).toBe("C:/first.sfc"); expect(result.current.candidate).toBeNull();
    expect(opened).toHaveBeenCalledOnce(); expect(mocks.getCurrentProject).toHaveBeenCalledOnce();
  });
  it("canceling a second selection keeps the first ROM path and region", async () => {
    mocks.open.mockResolvedValueOnce("C:/first.sfc").mockResolvedValueOnce("C:/cancelled.sfc");
    const { result } = renderHook(() => useRomOpening(true, vi.fn()));
    await act(async () => result.current.chooseRom());
    act(() => result.current.regionDetected({ success: true, region: "Usa", display_name: "USA", is_supported: true, sha1: "first", error_message: null }));
    await act(async () => result.current.confirmSelection());
    await act(async () => result.current.chooseRom());
    act(() => result.current.regionDetected({ success: true, region: "Jpn", display_name: "Japan", is_supported: false, sha1: "second", error_message: null }));
    act(() => result.current.cancel());
    expect(result.current.active?.path).toBe("C:/first.sfc"); expect(result.current.active?.region?.region).toBe("Usa");
    expect(mocks.openRom).toHaveBeenCalledTimes(1);
  });
  it("keeps the confirmation open when openRom reports failure without rejecting", async () => {
    mocks.open.mockResolvedValue("C:/bad.sfc");
    mocks.openRom.mockImplementation(async () => { state.romSha1 = null; state.error = "Invalid ROM"; });
    const opened = vi.fn(); const { result } = renderHook(() => useRomOpening(true, opened));
    await act(async () => result.current.chooseRom()); await act(async () => result.current.confirmSelection());
    expect(result.current.candidate?.path).toBe("C:/bad.sfc"); expect(result.current.active).toBeNull();
    expect(result.current.openingError).toContain("Invalid ROM"); expect(opened).not.toHaveBeenCalled();
    expect(result.current.busy).toBe(false);
  });
  it("warns before discarding a session with edits and honors Keep editing", async () => {
    state.romSha1 = "existing"; state.pendingWrites.add("80"); mocks.confirm.mockResolvedValue(false);
    const { result } = renderHook(() => useRomOpening(true, vi.fn()));
    await act(async () => result.current.chooseRom());
    expect(mocks.confirm).toHaveBeenCalledOnce(); expect(mocks.open).not.toHaveBeenCalled();
    expect(state.romSha1).toBe("existing"); expect(result.current.busy).toBe(false);
  });
  it("does nothing when the file picker is canceled", async () => {
    mocks.open.mockResolvedValue(null); const { result } = renderHook(() => useRomOpening(true, vi.fn()));
    await act(async () => result.current.chooseRom()); expect(result.current.candidate).toBeNull(); expect(mocks.openRom).not.toHaveBeenCalled();
  });
  it("prevents duplicate dialogs before React has rerendered", async () => {
    mocks.open.mockResolvedValue("C:/first.sfc"); const { result } = renderHook(() => useRomOpening(true, vi.fn()));
    await act(async () => { await Promise.all([result.current.chooseRom(), result.current.chooseRom()]); });
    expect(mocks.open).toHaveBeenCalledOnce();
  });
  it("does not call native dialogs in browser preview", async () => {
    const { result } = renderHook(() => useRomOpening(false, vi.fn()));
    await act(async () => result.current.chooseRom()); expect(mocks.open).not.toHaveBeenCalled(); expect(mocks.setError).toHaveBeenCalledOnce();
  });
  it("restores the original opener after the asynchronous picker and region cancellation", async () => {
    const button = document.createElement("button"); document.body.appendChild(button); button.focus();
    mocks.open.mockResolvedValue("C:/first.sfc");
    const { result } = renderHook(() => useRomOpening(true, vi.fn()));
    await act(async () => result.current.chooseRom());
    button.blur();
    expect(document.activeElement).not.toBe(button);
    act(() => result.current.cancel());
    expect(button).toHaveFocus(); button.remove();
  });

});
