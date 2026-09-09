import { useCallback, useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

interface ImageOptions {
  enabled: boolean;
  romSha1: string | null;
  pendingWrites: ReadonlySet<string>;
  undoStack: readonly unknown[];
}
interface Snapshot {
  token: object;
  data: Uint8Array | null;
  error: string | null;
}

/** Read only the canonical working image. Never fall back to a previous revision. */
export function useTestRomImage({ enabled, romSha1, pendingWrites, undoStack }: ImageOptions) {
  const [attempt, setAttempt] = useState(0);
  const [snapshot, setSnapshot] = useState<Snapshot | null>(null);
  // References, NOT counts: editing the same range again leaves Set.size unchanged.
  const token = useMemo(() => ({}), [enabled, romSha1, pendingWrites, undoStack, attempt]);
  const retry = useCallback(() => setAttempt((value) => value + 1), []);

  useEffect(() => {
    let disposed = false;
    if (!enabled || !romSha1) {
      setSnapshot(null);
      return;
    }
    void invoke<number[]>("get_loaded_rom_image").then((bytes) => {
      if (!Array.isArray(bytes) || bytes.length === 0) throw new Error("The current ROM image is empty.");
      if (!disposed) setSnapshot({ token, data: new Uint8Array(bytes), error: null });
    }).catch((error: unknown) => {
      if (!disposed) setSnapshot({ token, data: null, error: String(error) });
    });
    return () => { disposed = true; };
  }, [enabled, romSha1, token]);

  // Invalidate synchronously during render, before the next effect gets a turn.
  const current = snapshot?.token === token ? snapshot : null;
  const status = !enabled || !romSha1 ? "idle"
    : !current ? "loading" : current.error ? "error" : "ready";
  return { status, data: current?.data ?? null, error: current?.error ?? null, retry };
}
