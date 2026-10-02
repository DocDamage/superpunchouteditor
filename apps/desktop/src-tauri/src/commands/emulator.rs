//! External Emulator Integration Commands
//!
//! Commands for launching the exact current editor revision in external emulators.

use std::path::{Path, PathBuf};

use serde::Serialize;
use tauri::State;

use crate::app_state::AppState;
use crate::emulator::{EmulatorLauncher, EmulatorType};

/// Launch the current materialized ROM in the configured external emulator.
///
/// `auto_save` is retained for IPC compatibility with existing frontend callers. External testing
/// always uses the canonical base-plus-journal materialization so it cannot silently launch stale
/// source bytes when the editor has unsaved changes.
#[tauri::command]
pub async fn test_in_emulator(
    state: State<'_, AppState>,
    auto_save: bool,
    quick_load_slot: Option<u8>,
    boxer_key: Option<String>,
    round: u8,
) -> Result<(), String> {
    let _ = auto_save;
    // boxer_key and round are reserved for future save-state/test-preset integration.
    let _ = boxer_key;
    let _ = round;

    // Keep the original source path only for save-state naming/location compatibility.
    let source_rom_path = state.rom_path.lock().clone().ok_or("No ROM loaded")?;
    let source_rom = PathBuf::from(source_rom_path);

    let settings = state.emulator_settings.lock().clone();
    if settings.emulator_path.is_empty() {
        return Err(
            "No emulator configured. Please configure an emulator in settings.".to_string(),
        );
    }

    let emulator_path = PathBuf::from(&settings.emulator_path);
    if !emulator_path.exists() {
        return Err(format!("Emulator not found at: {}", settings.emulator_path));
    }

    // The canonical editor session is the sole source of truth for external testing.
    let materialized = state.materialize_current_rom()?;
    let temp_dir = std::env::temp_dir().join("super-punch-out-editor");
    std::fs::create_dir_all(&temp_dir)
        .map_err(|e| format!("Failed to create temp directory: {e}"))?;

    let extension = source_rom
        .extension()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .unwrap_or("sfc");
    let hash_prefix = materialized
        .current_sha1
        .get(..8)
        .unwrap_or(materialized.current_sha1.as_str());
    let temp_rom_path = temp_dir.join(format!(
        "testing_rom_r{}_{}.{}",
        materialized.revision, hash_prefix, extension
    ));

    std::fs::write(&temp_rom_path, &materialized.bytes)
        .map_err(|e| format!("Failed to write materialized test ROM: {e}"))?;

    let extra_args: Vec<String> = if settings.command_line_args.is_empty() {
        vec![]
    } else {
        settings
            .command_line_args
            .split_whitespace()
            .map(String::from)
            .collect()
    };

    let _child = if let Some(slot) = quick_load_slot {
        // Locate save states using the original ROM identity while launching the materialized image.
        let state_path =
            EmulatorLauncher::get_save_state_path(settings.emulator_type, &source_rom, Some(slot));
        EmulatorLauncher::launch_with_state(
            &temp_rom_path,
            &emulator_path,
            settings.emulator_type,
            &state_path,
            &extra_args,
        )
    } else {
        EmulatorLauncher::launch(
            &temp_rom_path,
            &emulator_path,
            settings.emulator_type,
            &extra_args,
        )
    }
    .map_err(|e| format!("Failed to launch emulator: {e}"))?;

    Ok(())
}

/// Get emulator presets for quick testing.
#[tauri::command]
pub fn get_emulator_presets() -> Vec<serde_json::Value> {
    vec![
        serde_json::json!({
            "id": "round_1",
            "name": "Round 1",
            "description": "Start at Round 1, full health",
            "boxer_index": null,
            "round": 1,
            "player_health": 128,
            "opponent_health": 128,
            "time_seconds": 180,
        }),
        serde_json::json!({
            "id": "knockdown",
            "name": "Knockdown Test",
            "description": "Low opponent health for easy KO",
            "boxer_index": null,
            "round": 1,
            "player_health": 128,
            "opponent_health": 10,
            "time_seconds": 180,
        }),
        serde_json::json!({
            "id": "low_health",
            "name": "Low Health",
            "description": "Test low health scenarios",
            "boxer_index": null,
            "round": 2,
            "player_health": 20,
            "opponent_health": 128,
            "time_seconds": 60,
        }),
    ]
}

/// An emulator program found on this computer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FoundEmulator {
    pub path: String,
    /// Kebab-case emulator type, matching `EmulatorType`'s serialized form.
    pub emulator_type: String,
    /// Human-readable name, for example "Snes9x".
    pub name: String,
    /// Folder the program is in, shown so the user can recognise it.
    pub folder: String,
}

/// How deep below each search folder to look.
const EMULATOR_SEARCH_DEPTH: usize = 3;
/// Upper bound on directory entries examined, so a huge folder cannot stall the app.
const EMULATOR_SEARCH_BUDGET: usize = 20_000;

/// Recognise a well-known SNES emulator by its program file name.
pub fn classify_emulator_file(file_name: &str) -> Option<EmulatorType> {
    let lower = file_name.to_ascii_lowercase();
    let stem = if cfg!(target_os = "windows") {
        lower.strip_suffix(".exe")?
    } else {
        lower.strip_suffix(".app").unwrap_or(&lower)
    };

    // Skip installers, updaters and uninstallers that share the product name.
    if ["setup", "install", "unins", "update"]
        .iter()
        .any(|word| stem.contains(word))
    {
        return None;
    }

    if stem.starts_with("snes9x") {
        Some(EmulatorType::Snes9x)
    } else if stem.starts_with("bsnes") || stem.starts_with("higan") {
        Some(EmulatorType::Bsnes)
    } else if stem.starts_with("mesen") {
        Some(EmulatorType::MesenS)
    } else {
        None
    }
}

fn emulator_type_slug(emulator_type: EmulatorType) -> &'static str {
    match emulator_type {
        EmulatorType::Snes9x => "snes9x",
        EmulatorType::Bsnes => "bsnes",
        EmulatorType::MesenS => "mesen-s",
        EmulatorType::Other => "other",
    }
}

/// Look for emulator programs under `roots`, by file name only. Nothing is
/// opened, run or changed. The search is bounded in depth and in the number of
/// entries examined.
pub fn scan_for_emulators(
    roots: &[PathBuf],
    max_depth: usize,
    budget: usize,
) -> Vec<FoundEmulator> {
    let mut found: Vec<FoundEmulator> = Vec::new();
    let mut remaining = budget;
    let mut queue: std::collections::VecDeque<(PathBuf, usize)> =
        roots.iter().cloned().map(|root| (root, 0)).collect();

    while let Some((dir, depth)) = queue.pop_front() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            if remaining == 0 {
                return found;
            }
            remaining -= 1;

            let path = entry.path();
            let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            let Ok(file_type) = entry.file_type() else {
                continue;
            };

            let is_app_bundle = cfg!(target_os = "macos") && file_name.ends_with(".app");
            if file_type.is_file() || is_app_bundle {
                if let Some(emulator_type) = classify_emulator_file(file_name) {
                    let path_text = path.to_string_lossy().to_string();
                    if !found.iter().any(|existing| existing.path == path_text) {
                        found.push(FoundEmulator {
                            path: path_text,
                            emulator_type: emulator_type_slug(emulator_type).to_string(),
                            name: emulator_type.display_name().to_string(),
                            folder: dir.to_string_lossy().to_string(),
                        });
                    }
                }
            } else if file_type.is_dir() && depth < max_depth && !file_name.starts_with('.') {
                queue.push_back((path, depth + 1));
            }
        }
    }

    found
}

/// Folders where people usually keep an emulator.
fn default_emulator_search_roots() -> Vec<PathBuf> {
    let mut roots: Vec<PathBuf> = [
        dirs::download_dir(),
        dirs::desktop_dir(),
        dirs::document_dir(),
        dirs::data_local_dir().map(|dir| dir.join("Programs")),
    ]
    .into_iter()
    .flatten()
    .collect();

    if cfg!(target_os = "windows") {
        for variable in ["ProgramFiles", "ProgramFiles(x86)"] {
            if let Ok(value) = std::env::var(variable) {
                roots.push(PathBuf::from(value));
            }
        }
    } else if cfg!(target_os = "macos") {
        roots.push(PathBuf::from("/Applications"));
        if let Some(home) = dirs::home_dir() {
            roots.push(home.join("Applications"));
        }
    } else {
        roots.extend(["/usr/bin", "/usr/local/bin", "/usr/games"].map(PathBuf::from));
        if let Some(home) = dirs::home_dir() {
            roots.push(home.join(".local").join("bin"));
        }
    }

    roots.retain(|root| Path::new(root).is_dir());
    roots.dedup();
    roots
}

/// Find SNES emulators already on this computer so Play Game can be set up
/// without the user hunting for a program file. The search only reads file
/// names in the usual download, desktop, documents and program folders; the
/// frontend asks the user before using anything it finds.
#[tauri::command]
pub async fn find_installed_emulators() -> Result<Vec<FoundEmulator>, String> {
    tauri::async_runtime::spawn_blocking(|| {
        scan_for_emulators(
            &default_emulator_search_roots(),
            EMULATOR_SEARCH_DEPTH,
            EMULATOR_SEARCH_BUDGET,
        )
    })
    .await
    .map_err(|error| format!("Emulator search failed: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn program(name: &str) -> String {
        if cfg!(target_os = "windows") {
            format!("{name}.exe")
        } else {
            name.to_string()
        }
    }

    #[test]
    fn recognises_known_emulators_and_ignores_installers() {
        assert_eq!(
            classify_emulator_file(&program("snes9x-x64")),
            Some(EmulatorType::Snes9x)
        );
        assert_eq!(
            classify_emulator_file(&program("bsnes")),
            Some(EmulatorType::Bsnes)
        );
        assert_eq!(
            classify_emulator_file(&program("Mesen")),
            Some(EmulatorType::MesenS)
        );
        assert_eq!(classify_emulator_file(&program("snes9x-setup")), None);
        assert_eq!(classify_emulator_file(&program("notepad")), None);
        assert_eq!(classify_emulator_file("snes9x-readme.txt"), None);
    }

    #[test]
    fn finds_an_emulator_inside_a_nested_folder() {
        let dir = tempfile::tempdir().unwrap();
        let nested = dir.path().join("games").join("snes9x-1.63");
        std::fs::create_dir_all(&nested).unwrap();
        std::fs::write(nested.join(program("snes9x-x64")), b"").unwrap();
        std::fs::write(nested.join("readme.txt"), b"").unwrap();

        let found = scan_for_emulators(&[dir.path().to_path_buf()], 3, 1000);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].emulator_type, "snes9x");
        assert_eq!(found[0].name, "Snes9x");
        assert!(found[0].path.ends_with(&program("snes9x-x64")));
    }

    #[test]
    fn search_respects_depth_and_budget_limits() {
        let dir = tempfile::tempdir().unwrap();
        let deep = dir.path().join("a").join("b").join("c");
        std::fs::create_dir_all(&deep).unwrap();
        std::fs::write(deep.join(program("snes9x")), b"").unwrap();

        assert!(scan_for_emulators(&[dir.path().to_path_buf()], 1, 1000).is_empty());
        assert!(scan_for_emulators(&[dir.path().to_path_buf()], 3, 1).is_empty());
        assert_eq!(
            scan_for_emulators(&[dir.path().to_path_buf()], 3, 1000).len(),
            1
        );
        assert!(scan_for_emulators(&[dir.path().join("missing")], 3, 1000).is_empty());
    }
}
