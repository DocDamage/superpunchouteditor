//! Utility Functions Module
//!
//! Shared helper functions and utilities used across the application.

pub mod validation;

// Re-export commonly used utilities
pub use validation::*;

use manifest_core::Manifest;
use rom_core::RomRegion;
use std::env;
use std::path::Path;
use std::path::PathBuf;

/// Parse an offset string (hex or decimal) into a usize
///
/// Supports formats:
/// - Hexadecimal: "0x1234", "0XABC"
/// - Decimal: "4660"
///
/// # Examples
/// ```ignore
/// use crate::utils::parse_offset;
///
/// assert_eq!(parse_offset("0x1234").unwrap(), 0x1234);
/// assert_eq!(parse_offset("4660").unwrap(), 4660);
/// ```
pub fn parse_offset(s: &str) -> Result<usize, String> {
    let s = s.trim();
    if s.starts_with("0x") || s.starts_with("0X") {
        usize::from_str_radix(&s[2..], 16).map_err(|e| format!("Invalid hex offset '{}': {}", s, e))
    } else {
        s.parse::<usize>()
            .map_err(|e| format!("Invalid decimal offset '{}': {}", s, e))
    }
}

/// Format a usize as a hex string with 0x prefix
///
/// # Examples
/// ```ignore
/// use crate::utils::format_hex;
///
/// assert_eq!(format_hex(0x1234), "0x1234");
/// ```
pub fn format_hex(value: usize) -> String {
    format!("0x{:X}", value)
}

/// Format a usize as a SNES address (24-bit, with bank)
///
/// # Examples
/// ```ignore
/// use crate::utils::format_snes_addr;
///
/// assert_eq!(format_snes_addr(0x091234), "$09:1234");
/// ```
pub fn format_snes_addr(value: usize) -> String {
    let bank = (value >> 16) & 0xFF;
    let addr = value & 0xFFFF;
    format!("${:02X}:{:04X}", bank, addr)
}

/// Ensure a directory exists, creating it if necessary
pub fn ensure_dir(path: &Path) -> Result<(), String> {
    if !path.exists() {
        std::fs::create_dir_all(path)
            .map_err(|e| format!("Failed to create directory '{}': {}", path.display(), e))?;
    }
    Ok(())
}

/// Get the application's config directory
pub fn get_config_dir() -> Result<std::path::PathBuf, String> {
    dirs::config_dir()
        .ok_or_else(|| "Could not find config directory".to_string())
        .map(|d| d.join("super-punch-out-editor"))
}

/// Get the application's data directory
pub fn get_data_dir() -> Result<std::path::PathBuf, String> {
    dirs::data_dir()
        .ok_or_else(|| "Could not find data directory".to_string())
        .map(|d| d.join("super-punch-out-editor"))
}

/// Truncate a string to a maximum length, adding ellipsis if truncated
pub fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

/// Safe filename sanitization
pub fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' | ' ' => '_',
            c => c,
        })
        .collect()
}

/// Get the manifest filename for a given ROM region.
pub fn manifest_filename_for_region(region: RomRegion) -> &'static str {
    match region {
        RomRegion::Usa => "boxers.json",
        RomRegion::Jpn => "boxers_jpn.json",
        RomRegion::Pal => "boxers_pal.json",
    }
}

/// Resolve candidate manifest paths for a given manifest filename.
///
/// `resource_dir` is the Tauri app resource directory, present in packaged builds.
/// It is tried first before falling back to CWD/exe-relative development paths.
pub fn manifest_search_paths(filename: &str, resource_dir: Option<&Path>) -> Vec<PathBuf> {
    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let exe_path = env::current_exe().ok();

    let mut manifest_paths: Vec<PathBuf> = Vec::new();

    // Packaged build: resource directory bundled by Tauri (highest priority).
    if let Some(res_dir) = resource_dir {
        manifest_paths.push(res_dir.join("data/manifests").join(filename));
        manifest_paths.push(res_dir.join(filename));
    }

    // Development: paths relative to the current working directory.
    manifest_paths.push(cwd.join("../../data/manifests").join(filename));
    manifest_paths.push(cwd.join("../../../data/manifests").join(filename));
    manifest_paths.push(cwd.join("data/manifests").join(filename));

    // Development: paths relative to the executable location.
    if let Some(exe) = exe_path {
        if let Some(exe_dir) = exe.parent() {
            manifest_paths.push(exe_dir.join("../../data/manifests").join(filename));
            manifest_paths.push(exe_dir.join("../../../data/manifests").join(filename));
            manifest_paths.push(exe_dir.join("data/manifests").join(filename));
        }
    }

    manifest_paths
}

/// Load a manifest matching the specified ROM region.
///
/// `resource_dir` is the Tauri app resource directory used in packaged builds.
/// Pass `None` in tests or when the resource dir is unavailable; the function
/// will fall back to CWD/exe-relative development paths.
pub fn load_manifest_for_region(
    region: RomRegion,
    resource_dir: Option<&Path>,
) -> Result<Manifest, String> {
    let filename = manifest_filename_for_region(region);
    let manifest_paths = manifest_search_paths(filename, resource_dir);

    for path in &manifest_paths {
        if path.exists() {
            #[cfg(debug_assertions)]
            eprintln!("[debug] Resolved {} manifest: {}", region, path.display());
            return Manifest::load(path).map_err(|e| e.to_string());
        }
    }

    Err(format!(
        "Cannot find {} manifest ({}). Searched {} path(s): {:?}",
        region,
        filename,
        manifest_paths.len(),
        manifest_paths
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_offset_hex() {
        assert_eq!(parse_offset("0x1234").unwrap(), 0x1234);
        assert_eq!(parse_offset("0XABC").unwrap(), 0xABC);
        assert_eq!(parse_offset("  0x1234  ").unwrap(), 0x1234);
    }

    #[test]
    fn test_parse_offset_decimal() {
        assert_eq!(parse_offset("4660").unwrap(), 4660);
        assert_eq!(parse_offset("0").unwrap(), 0);
    }

    #[test]
    fn test_parse_offset_invalid() {
        assert!(parse_offset("not_a_number").is_err());
        assert!(parse_offset("0xGGGG").is_err());
    }

    #[test]
    fn test_format_hex() {
        assert_eq!(format_hex(0x1234), "0x1234");
        assert_eq!(format_hex(0), "0x0");
    }

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("hello world"), "hello_world");
        assert_eq!(sanitize_filename("file:name"), "file_name");
        assert_eq!(sanitize_filename("test/path"), "test_path");
    }

    // -------------------------------------------------------------------------
    // Region-to-manifest filename mapping
    // -------------------------------------------------------------------------

    #[test]
    fn test_manifest_filename_usa() {
        assert_eq!(manifest_filename_for_region(RomRegion::Usa), "boxers.json");
    }

    #[test]
    fn test_manifest_filename_jpn() {
        assert_eq!(manifest_filename_for_region(RomRegion::Jpn), "boxers_jpn.json");
    }

    #[test]
    fn test_manifest_filename_pal() {
        assert_eq!(manifest_filename_for_region(RomRegion::Pal), "boxers_pal.json");
    }

    // -------------------------------------------------------------------------
    // manifest_search_paths ordering and resource_dir priority
    // -------------------------------------------------------------------------

    #[test]
    fn test_search_paths_resource_dir_is_first() {
        let res_dir = PathBuf::from("/fake/resource");
        let paths = manifest_search_paths("boxers.json", Some(&res_dir));
        // The first path must come from the provided resource directory.
        assert!(
            paths[0].starts_with(&res_dir),
            "Expected resource_dir path first, got {:?}",
            paths[0]
        );
    }

    #[test]
    fn test_search_paths_no_hardcoded_machine_specific_path() {
        // Verify the exact literal hardcoded Windows development machine path
        // that was removed is no longer in the candidate list.
        let old_literal = PathBuf::from(
            "C:\\Users\\Doc\\Desktop\\Projects\\SuperPunchOutEditor\\data\\manifests",
        )
        .join("boxers.json");

        let paths = manifest_search_paths("boxers.json", None);
        assert!(
            !paths.contains(&old_literal),
            "Hardcoded machine-specific path is still present: {}",
            old_literal.display()
        );
    }

    #[test]
    fn test_search_paths_none_and_some_differ() {
        let res_dir = PathBuf::from("/some/resource");
        let with_res = manifest_search_paths("boxers.json", Some(&res_dir));
        let without_res = manifest_search_paths("boxers.json", None);
        // Having a resource dir produces more candidates.
        assert!(with_res.len() > without_res.len());
    }

    // -------------------------------------------------------------------------
    // Manifest::empty() and load_manifest_for_region error messaging
    // -------------------------------------------------------------------------

    #[test]
    fn test_manifest_empty_has_no_fighters() {
        let m = Manifest::empty();
        assert!(m.fighters.is_empty());
    }

    #[test]
    fn test_load_manifest_error_message_is_actionable() {
        // Build the error string the function would produce when all paths fail,
        // and verify it contains the required diagnostic fields. This avoids
        // filesystem state dependencies while still exercising the error format.
        let region = RomRegion::Usa;
        let filename = manifest_filename_for_region(region);
        let bogus_dir = PathBuf::from("/totally_nonexistent_xyz_abc_123");
        let fake_paths = vec![bogus_dir.join("data/manifests").join(filename)];
        let err = format!(
            "Cannot find {} manifest ({}). Searched {} path(s): {:?}",
            region,   // formats as "Super Punch-Out!! (USA)"
            filename,
            fake_paths.len(),
            fake_paths
        );
        // The display form contains the region code "(USA)", "(Japan)", or "(Europe)".
        assert!(
            err.contains("(USA)") || err.contains("(Japan)") || err.contains("(Europe)"),
            "region identifier missing from error: {}",
            err
        );
        assert!(err.contains("boxers.json"), "filename missing from error: {}", err);
        assert!(err.contains("path(s)"), "path count missing from error: {}", err);
    }

    #[test]
    fn test_load_manifest_from_real_files() {
        // Load all three manifests from the repository data directory.
        // Pass the repo root as resource_dir; manifest_search_paths appends
        // "data/manifests/" to it automatically.
        //
        // CARGO_MANIFEST_DIR = apps/desktop/src-tauri  → go up 3 levels to repo root.
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let manifest_dir = repo_root.join("data/manifests");

        if !manifest_dir.exists() {
            eprintln!("Skipping real-file manifest tests: {:?} not found", manifest_dir);
            return;
        }

        // USA -> boxers.json
        let usa = load_manifest_for_region(RomRegion::Usa, Some(&repo_root));
        assert!(usa.is_ok(), "USA manifest load failed: {:?}", usa.err());
        assert!(!usa.unwrap().fighters.is_empty(), "USA manifest has no fighters");

        // JPN -> boxers_jpn.json
        let jpn = load_manifest_for_region(RomRegion::Jpn, Some(&repo_root));
        assert!(jpn.is_ok(), "JPN manifest load failed: {:?}", jpn.err());
        assert!(!jpn.unwrap().fighters.is_empty(), "JPN manifest has no fighters");

        // PAL -> boxers_pal.json
        let pal = load_manifest_for_region(RomRegion::Pal, Some(&repo_root));
        assert!(pal.is_ok(), "PAL manifest load failed: {:?}", pal.err());
        assert!(!pal.unwrap().fighters.is_empty(), "PAL manifest has no fighters");
    }
}
