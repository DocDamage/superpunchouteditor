import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// Vitest types augment the vite config when the triple-slash ref is present.
/// <reference types="vitest" />

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
export default defineConfig(async () => ({
  plugins: [react()],

  test: {
    environment: "jsdom",
    globals: true,
    setupFiles: ["./src/test/setup.ts"],
  },

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent Vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // 3. tell Vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },
  build: {
    rollupOptions: {
      output: {
        manualChunks(id) {
          const normalizedId = id.replace(/\\/g, "/");

          if (normalizedId.includes("/node_modules/")) {
            if (normalizedId.includes("/react")) return "vendor-react";
            if (normalizedId.includes("/@tauri-apps/")) return "vendor-tauri";
            return "vendor";
          }

          if (
            normalizedId.includes("/src/components/Animation")
            || normalizedId.includes("/src/components/Frame")
            || normalizedId.includes("/src/components/Tile")
          ) {
            return "feature-animation";
          }

          if (
            normalizedId.includes("/src/components/EmbeddedEmulator")
            || normalizedId.includes("/src/components/Emulator")
            || normalizedId.includes("/src/hooks/useEmulator")
            || normalizedId.includes("/src/components/InputMapper")
          ) {
            return "feature-emulator";
          }

          if (
            normalizedId.includes("/src/components/Plugin")
            || normalizedId.includes("/src/components/Script")
            || normalizedId.includes("/src/components/LayoutPack")
          ) {
            return "feature-tools";
          }

          return undefined;
        },
      },
    },
  },
}));
