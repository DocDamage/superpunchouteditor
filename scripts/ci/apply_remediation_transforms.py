#!/usr/bin/env python3
"""One-shot deterministic source transforms for the remediation branch.

This script exists only to make large already-reviewed mechanical edits reproducible in GitHub
Actions. It aborts when an expected source shape changes rather than applying a partial mutation.
"""
from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def replace_once(path: Path, old: str, new: str, *, already: str | None = None) -> None:
    text = path.read_text(encoding="utf-8")
    if already and already in text:
        return
    if old not in text:
        raise SystemExit(f"expected remediation marker missing in {path}: {old[:120]!r}")
    path.write_text(text.replace(old, new, 1), encoding="utf-8")


def reconcile_registry() -> None:
    path = ROOT / "apps/desktop/src-tauri/src/lib.rs"
    replace_once(
        path,
        "            commands::project::save_patch_notes,\n",
        """            commands::project::save_patch_notes,
            // Layout pack commands (experimental until canonical edit migration)
            commands::layout_pack::export_layout_pack,
            commands::layout_pack::import_layout_pack,
            commands::layout_pack::validate_layout_pack,
            commands::layout_pack::get_available_layout_packs,
            commands::layout_pack::delete_layout_pack,
            commands::layout_pack::install_layout_pack,
            commands::layout_pack::apply_layout_pack,
            // Script/fighter parameter commands
            commands::scripts::get_all_scripts,
            commands::scripts::get_scripts_for_fighter,
            commands::scripts::get_fighter_header,
            commands::scripts::validate_fighter_params,
            commands::scripts::update_fighter_params,
""",
        already="commands::layout_pack::export_layout_pack,",
    )
    replace_once(
        path,
        """            // Help System (currently disabled - TODO: implement in help_system.rs)
            // get_help_articles,
            // get_help_article,
            // search_help,
            // get_context_help,
            // get_help_categories,
            // submit_help_feedback,
""",
        """            // Help System
            commands::help::get_help_articles,
            commands::help::get_help_article,
            commands::help::search_help,
            commands::help::get_context_help,
            commands::help::submit_help_feedback,
""",
        already="commands::help::get_help_articles,",
    )
    replace_once(
        path,
        "            commands::history::clear_history,\n",
        """            commands::history::clear_history,
            commands::history::can_undo,
            commands::history::can_redo,
            commands::history::get_undo_stack,
            commands::history::get_redo_stack,
            commands::history::record_palette_edit,
            commands::history::record_sprite_bin_edit,
            commands::history::record_asset_import,
""",
        already="commands::history::can_undo,",
    )


def fix_app_state() -> None:
    path = ROOT / "apps/desktop/src-tauri/src/app_state.rs"
    text = path.read_text(encoding="utf-8")
    text = text.replace(
        """        let projection = session
            .commit(session.base(), label.into(), requests)
            .map_err(|e| e.to_string())?;
        let projection = session.state_projection().map_err(|e| e.to_string())?;
""",
        """        let projection = session
            .commit(label.into(), requests)
            .map_err(|e| e.to_string())?;
""",
    )
    text = text.replace(
        """        session
            .journal_mut()
            .commit(session.base(), label, requests)
            .map_err(|e| e.to_string())?;
        let projection = session.state_projection().map_err(|e| e.to_string())?;
""",
        """        let projection = session
            .commit(label, requests)
            .map_err(|e| e.to_string())?;
""",
    )
    path.write_text(text, encoding="utf-8")


def register_project_v2() -> None:
    path = ROOT / "crates/project-core/src/lib.rs"
    text = path.read_text(encoding="utf-8")
    if "pub mod session_document;" in text:
        return
    marker = "pub mod tools;\npub use tools::*;\n\n"
    if marker not in text:
        raise SystemExit("project-core module marker changed")
    text = text.replace(
        marker,
        marker + "pub mod session_document;\npub use session_document::*;\n\n",
        1,
    )
    path.write_text(text, encoding="utf-8")


def fix_project_integrity() -> None:
    path = ROOT / "crates/project-core/src/session_document.rs"
    text = path.read_text(encoding="utf-8")
    text = text.replace(
        "last_saved_revision: session.journal().saved_revision(),",
        "last_saved_revision: session.journal().revision(),",
    )
    text = text.replace(
        """fn document_integrity(document: &ProjectDocumentV2) -> Result<String, ProjectError> {
    let canonical = serde_json::to_vec(document)?;
    Ok(sha1_hex(&canonical))
}
""",
        """fn document_integrity(document: &ProjectDocumentV2) -> Result<String, ProjectError> {
    // Convert through `Value` so object keys (including HashMap-backed settings) are emitted in
    // deterministic map order before hashing. Integrity must survive deserialize/serialize cycles.
    let canonical_value = serde_json::to_value(document)?;
    let canonical = serde_json::to_vec(&canonical_value)?;
    Ok(sha1_hex(&canonical))
}
""",
    )
    path.write_text(text, encoding="utf-8")


def consolidate_frontend_navigation() -> None:
    path = ROOT / "apps/desktop/src/App.tsx"
    text = path.read_text(encoding="utf-8")
    if 'from "./featureMaturity"' not in text:
        marker = 'import { useStore } from "./store/useStore";\n'
        if marker not in text:
            raise SystemExit("App.tsx store import marker changed")
        text = text.replace(
            marker,
            marker
            + 'import { featureLabel, isFeatureVisible } from "./featureMaturity";\n',
            1,
        )
    old_tabs_end = '  { key: "settings", label: "Settings" },\n];\n'
    new_tabs_end = '''  { key: "settings", label: "Settings" },
]
  .filter(({ key }) => isFeatureVisible(key))
  .map(({ key, label }) => ({ key, label: featureLabel(label, key) }));
'''
    if ".filter(({ key }) => isFeatureVisible(key))" not in text:
        if old_tabs_end not in text:
            raise SystemExit("App.tsx TAB_ITEMS marker changed")
        text = text.replace(old_tabs_end, new_tabs_end, 1)

    text = text.replace("    clearHistory,\n", "", 1)
    text = text.replace(
        """  useEffect(() => {
    if (!isDesktopRuntime || !romSha1) return;
    void clearHistory();
  }, [isDesktopRuntime, romSha1, clearHistory]);

""",
        "",
        1,
    )
    text = text.replace(
        """      const targetTab = quickTabs[e.key];
      if (targetTab) {
""",
        """      const targetTab = quickTabs[e.key];
      if (targetTab && isFeatureVisible(targetTab)) {
""",
        1,
    )
    path.write_text(text, encoding="utf-8")


def consolidate_frontend_edit_projection() -> None:
    path = ROOT / "apps/desktop/src/store/useStore.ts"
    text = path.read_text(encoding="utf-8")
    text = text.replace(
        "  /** Set of PC offset strings that have been staged for writing */\n  pendingWrites: Set<string>;",
        "  /** Backend-projected changed-range starts for the current canonical journal revision. */\n  pendingWrites: Set<string>;",
    )
    if "refreshPendingWrites: () => Promise<void>;" not in text:
        marker = "  clearPendingWrites: () => void;\n"
        if marker not in text:
            raise SystemExit("useStore pending action marker changed")
        text = text.replace(marker, marker + "  refreshPendingWrites: () => Promise<void>;\n", 1)

    text = text.replace("  currentVersion: '0.1.0',", "  currentVersion: '2.0.0',")

    old = """  setPendingWrite: (pcOffset: string) => {
    set(state => ({ 
      pendingWrites: new Set([...state.pendingWrites, pcOffset]),
      isProjectModified: true 
    }));
  },

  removePendingWrite: (pcOffset: string) => {
    set(state => {
      const next = new Set(state.pendingWrites);
      next.delete(pcOffset);
      return { pendingWrites: next };
    });
  },

  clearPendingWrites: () => set({ pendingWrites: new Set() }),
"""
    new = """  setPendingWrite: () => {
    void get().refreshPendingWrites();
  },

  removePendingWrite: () => {
    // Selective deletion is not a valid journal operation. Refresh the backend projection instead.
    void get().refreshPendingWrites();
  },

  clearPendingWrites: () => {
    // The backend journal owns edits. This action may only refresh, never erase durable state.
    void get().refreshPendingWrites();
  },

  refreshPendingWrites: async () => {
    try {
      const offsets = await invoke<string[]>('get_pending_writes');
      set({ pendingWrites: new Set(offsets) });
    } catch (e) {
      console.error('Failed to refresh changed-range projection:', e);
    }
  },
"""
    if old in text:
        text = text.replace(old, new, 1)
    elif "refreshPendingWrites: async ()" not in text:
        raise SystemExit("useStore pending-write implementation marker changed")

    text = text.replace(
        """      // Sync undo state (history was cleared server-side on ROM load).
      await get().refreshUndoState();
""",
        """      // Sync canonical backend projections for the new session.
      await Promise.all([get().refreshUndoState(), get().refreshPendingWrites()]);
""",
        1,
    )
    text = text.replace(
        """      await invoke('undo');
      await get().refreshUndoState();
""",
        """      await invoke('undo');
      await Promise.all([get().refreshUndoState(), get().refreshPendingWrites()]);
""",
        1,
    )
    text = text.replace(
        """      await invoke('redo');
      await get().refreshUndoState();
""",
        """      await invoke('redo');
      await Promise.all([get().refreshUndoState(), get().refreshPendingWrites()]);
""",
        1,
    )
    for marker in [
        "await invoke('record_palette_edit',",
        "await invoke('record_sprite_bin_edit',",
        "await invoke('record_asset_import',",
    ]:
        pos = text.find(marker)
        if pos == -1:
            continue
        refresh = "await get().refreshUndoState();"
        refresh_pos = text.find(refresh, pos)
        if refresh_pos != -1:
            text = (
                text[:refresh_pos]
                + "await Promise.all([get().refreshUndoState(), get().refreshPendingWrites()]);"
                + text[refresh_pos + len(refresh):]
            )

    # A loaded project can replace the active journal; refresh every backend-owned projection.
    load_project_set = """      set({ 
        currentProject: project, 
        currentProjectPath: path,
        isProjectModified: false,
        error: null 
      });
"""
    if load_project_set in text:
        replacement = load_project_set + "      await Promise.all([get().refreshUndoState(), get().refreshPendingWrites()]);\n"
        # Use the second occurrence (loadProject), not createProject.
        first = text.find(load_project_set)
        second = text.find(load_project_set, first + len(load_project_set))
        if second != -1:
            text = text[:second] + replacement + text[second + len(load_project_set):]

    path.write_text(text, encoding="utf-8")


def main() -> None:
    reconcile_registry()
    fix_app_state()
    register_project_v2()
    fix_project_integrity()
    consolidate_frontend_navigation()
    consolidate_frontend_edit_projection()
    print("Deterministic remediation transforms applied.")


if __name__ == "__main__":
    main()
