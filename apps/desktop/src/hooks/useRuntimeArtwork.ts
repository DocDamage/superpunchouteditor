import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useTheme } from "../context/ThemeProvider";
import type { BoxerRecord } from "../store/useStore";

function imageUrl(bytes: number[] | null | undefined): string | null {
  if (!bytes?.length) return null;
  let binary = "";
  for (const byte of bytes) binary += String.fromCharCode(byte);
  return `data:image/png;base64,${btoa(binary)}`;
}
interface ThemeAssets {
  boxer_key: string;
  boxer_name: string;
  palette: Array<{ r: number; g: number; b: number }>;
  icon_png: number[] | null;
  portrait_png: number[] | null;
}

export function useRuntimeArtwork(desktop: boolean, romSha1: string | null,
  selectedKey: string | null, boxers: BoxerRecord[]) {
  const { runtimeSkin, setRuntimeSkin } = useTheme();
  const [portraits, setPortraits] = useState<Record<string, string>>({});
  useEffect(() => {
    if (!desktop || !romSha1) { setRuntimeSkin(null); return; }
    let cancelled = false;
    void invoke<ThemeAssets>("get_runtime_theme_assets", { boxerKey: selectedKey }).then((assets) => {
      if (!cancelled) setRuntimeSkin({
        boxerKey: assets.boxer_key, boxerName: assets.boxer_name, palette: assets.palette,
        iconDataUrl: imageUrl(assets.icon_png), portraitDataUrl: imageUrl(assets.portrait_png),
      });
    }).catch((error) => {
      console.error("Failed to load runtime theme assets:", error);
      if (!cancelled) setRuntimeSkin(null);
    });
    return () => { cancelled = true; };
  }, [desktop, romSha1, selectedKey, setRuntimeSkin]);

  useEffect(() => {
    if (!desktop || !romSha1 || boxers.length === 0) { setPortraits({}); return; }
    let cancelled = false;
    void Promise.all(boxers.map(async (boxer) => {
      try {
        const assets = await invoke<ThemeAssets>("get_runtime_theme_assets", { boxerKey: boxer.key });
        return [boxer.key, imageUrl(assets.portrait_png) ?? imageUrl(assets.icon_png)] as const;
      } catch (error) {
        console.error(`Failed to load portrait for ${boxer.key}:`, error);
        return [boxer.key, null] as const;
      }
    })).then((entries) => {
      if (cancelled) return;
      const next: Record<string, string> = {};
      for (const [key, url] of entries) if (url) next[key] = url;
      setPortraits(next);
    });
    return () => { cancelled = true; };
  }, [desktop, romSha1, boxers]);
  return { runtimeSkin, portraits };
}
