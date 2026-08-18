export type FeatureMaturity =
  | "stable"
  | "experimental"
  | "research-blocked"
  | "deprecated"
  | "removed";

export type ProductFeatureKey =
  | "editor"
  | "roster"
  | "viewer"
  | "scripts"
  | "animations"
  | "frames"
  | "compare"
  | "project"
  | "packs"
  | "ai"
  | "plugins"
  | "banks"
  | "animation-player"
  | "audio"
  | "text"
  | "test"
  | "settings";

export interface FeatureMaturityRecord {
  status: FeatureMaturity;
  releaseDecision: string;
}

/**
 * Single product-surface maturity registry.
 *
 * Stable builds expose only `stable` features. Experimental UI is opt-in and only available in a
 * development build when VITE_ENABLE_EXPERIMENTAL=true. Research-blocked/deprecated/removed
 * features never become visible through that flag.
 */
export const FEATURE_MATURITY: Record<ProductFeatureKey, FeatureMaturityRecord> = {
  editor: {
    status: "stable",
    releaseDecision: "Core asset/palette editing surface; mutations must use the canonical journal.",
  },
  roster: {
    status: "stable",
    releaseDecision: "Stable after roster writers are journal-backed.",
  },
  viewer: {
    status: "stable",
    releaseDecision: "Read-only ROM/asset inspection.",
  },
  scripts: {
    status: "experimental",
    releaseDecision: "Developer/research tooling; hidden from stable navigation.",
  },
  animations: {
    status: "research-blocked",
    releaseDecision: "Read/write format is not sufficiently proven for stable mutation.",
  },
  frames: {
    status: "research-blocked",
    releaseDecision: "Frame reconstruction/mutation remains research dependent.",
  },
  compare: {
    status: "experimental",
    releaseDecision: "Binary/report comparison uses the canonical revision; visual renderer remains experimental.",
  },
  project: {
    status: "stable",
    releaseDecision: "Project format v2 persists the complete edit journal.",
  },
  packs: {
    status: "experimental",
    releaseDecision: "Import/validation is available; apply remains disabled until packs contain payloads.",
  },
  ai: {
    status: "experimental",
    releaseDecision: "Hidden until every AI mutation commits through the canonical journal.",
  },
  plugins: {
    status: "research-blocked",
    releaseDecision: "Stable IPC execution is disabled pending a constrained trust/capability sandbox.",
  },
  banks: {
    status: "experimental",
    releaseDecision: "Relocation/bank developer tooling is hidden until journal migration is complete.",
  },
  "animation-player": {
    status: "experimental",
    releaseDecision: "Inspection/playback only; hidden from stable navigation.",
  },
  audio: {
    status: "experimental",
    releaseDecision: "Browse/import/export is being separated from research-blocked ROM sequence editing.",
  },
  text: {
    status: "experimental",
    releaseDecision: "Hidden until text mutation commands are journal-backed end-to-end.",
  },
  test: {
    status: "stable",
    releaseDecision: "Embedded emulator consumes the exact materialized current image.",
  },
  settings: {
    status: "stable",
    releaseDecision: "Application preferences/update settings only.",
  },
};

export const experimentalUiEnabled =
  import.meta.env.DEV && import.meta.env.VITE_ENABLE_EXPERIMENTAL === "true";

export function isFeatureVisible(key: ProductFeatureKey): boolean {
  const status = FEATURE_MATURITY[key].status;
  if (status === "stable") return true;
  return status === "experimental" && experimentalUiEnabled;
}

export function featureLabel(label: string, key: ProductFeatureKey): string {
  return FEATURE_MATURITY[key].status === "experimental" ? `${label} (Experimental)` : label;
}
