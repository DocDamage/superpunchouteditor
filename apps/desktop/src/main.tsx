import React from "react";
import ReactDOM from "react-dom/client";
import { isTauri } from "@tauri-apps/api/core";
import App from "./App";

async function start(): Promise<void> {
  // Development only: `http://localhost:1420/?preview` in a normal browser
  // shows the interface with made-up sample data. This branch and the module
  // it loads are removed from production builds.
  if (import.meta.env.DEV && !isTauri() && new URLSearchParams(window.location.search).has("preview")) {
    const { installPreviewBackend } = await import("./dev/previewBackend");
    installPreviewBackend();
  }

  ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
    <React.StrictMode>
      <App />
    </React.StrictMode>,
  );
}

void start();
