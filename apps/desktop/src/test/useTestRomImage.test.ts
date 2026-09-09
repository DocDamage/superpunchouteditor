import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { useTestRomImage } from "../hooks/useTestRomImage";
vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
const call = vi.mocked(invoke);
const initial = () => ({ enabled: true, romSha1: "rom-a", pendingWrites: new Set(["80"]), undoStack: [] as unknown[] });
function deferred() {
  let resolve!: (bytes: number[]) => void;
  const promise = new Promise<number[]>((done) => { resolve = done; });
  return { promise, resolve };
}
beforeEach(() => call.mockReset());

describe("canonical test ROM image", () => {
  it("does not read a ROM outside an active test session", () => {
    renderHook(() => useTestRomImage({ ...initial(), enabled: false }));
    expect(call).not.toHaveBeenCalled();
  });
  it("refreshes a same-size changed-range projection", async () => {
    call.mockResolvedValueOnce([1, 2]).mockResolvedValueOnce([3, 4]);
    const props = initial(); const { result, rerender } = renderHook(useTestRomImage, { initialProps: props });
    await waitFor(() => expect(result.current.status).toBe("ready"));
    rerender({ ...props, pendingWrites: new Set(["80"]) });
    await waitFor(() => expect(Array.from(result.current.data!)).toEqual([3, 4]));
    expect(call).toHaveBeenCalledTimes(2);
  });
  it("refreshes on history changes even when the changed-range reference is unchanged", async () => {
    call.mockResolvedValueOnce([1]).mockResolvedValueOnce([2]);
    const props = initial(); const { result, rerender } = renderHook(useTestRomImage, { initialProps: props });
    await waitFor(() => expect(result.current.status).toBe("ready"));
    rerender({ ...props, undoStack: [{ description: "Undo" }] });
    await waitFor(() => expect(result.current.data?.[0]).toBe(2));
  });
  it("invalidates immediately and ignores out-of-order responses", async () => {
    const first = deferred(); const second = deferred();
    call.mockReturnValueOnce(first.promise).mockReturnValueOnce(second.promise);
    const props = initial(); const { result, rerender } = renderHook(useTestRomImage, { initialProps: props });
    rerender({ ...props, pendingWrites: new Set(["80"]) });
    expect(result.current.data).toBeNull();
    await act(async () => second.resolve([9]));
    expect(result.current.data?.[0]).toBe(9);
    await act(async () => first.resolve([1]));
    expect(result.current.data?.[0]).toBe(9);
  });
  it("never exposes a previous ready image while the next read is pending", async () => {
    const next = deferred(); call.mockResolvedValueOnce([1]).mockReturnValueOnce(next.promise);
    const props = initial(); const { result, rerender } = renderHook(useTestRomImage, { initialProps: props });
    await waitFor(() => expect(result.current.status).toBe("ready"));
    rerender({ ...props, romSha1: "rom-b" });
    expect(result.current.status).toBe("loading"); expect(result.current.data).toBeNull();
    await act(async () => next.resolve([2])); expect(result.current.data?.[0]).toBe(2);
  });
  it("offers retry after failure and never substitutes stale bytes", async () => {
    call.mockRejectedValueOnce(new Error("Read failed")).mockResolvedValueOnce([7]);
    const props = initial(); const { result } = renderHook(() => useTestRomImage(props));
    await waitFor(() => expect(result.current.status).toBe("error"));
    expect(result.current.data).toBeNull(); expect(result.current.error).toContain("Read failed");
    act(() => result.current.retry());
    await waitFor(() => expect(result.current.data?.[0]).toBe(7));
  });
  it("rejects an empty image", async () => {
    call.mockResolvedValueOnce([]);
    const props = initial(); const { result } = renderHook(() => useTestRomImage(props));
    await waitFor(() => expect(result.current.status).toBe("error"));
    expect(result.current.error).toContain("empty");
  });
  it("drops data when the user leaves Test Game", async () => {
    call.mockResolvedValueOnce([1]); const props = initial();
    const { result, rerender } = renderHook(useTestRomImage, { initialProps: props });
    await waitFor(() => expect(result.current.status).toBe("ready"));
    rerender({ ...props, enabled: false });
    expect(result.current.status).toBe("idle"); expect(result.current.data).toBeNull();
  });
});
