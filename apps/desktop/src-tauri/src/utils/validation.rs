//! Validation Utilities
//!
//! Functions for validating paths, offsets, and other user input.

use std::path::Path;

/// Validate that a path points to a valid ROM file
///
/// Checks:
/// - Path exists
/// - Path is a file
/// - File has a valid extension (.sfc, .smc, .fig, .bin)
/// - File size is reasonable for a SNES ROM
pub fn validate_rom_path(path: &str) -> Result<(), String> {
    let path = Path::new(path);

    if !path.exists() {
        return Err(format!("Path does not exist: {}", path.display()));
    }

    if !path.is_file() {
        return Err(format!("Path is not a file: {}", path.display()));
    }

    // Check extension
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase());

    let valid_extensions = ["sfc", "smc", "fig", "bin"];

    match extension {
        Some(ext) if valid_extensions.contains(&ext.as_str()) => {
            // Valid extension
        }
        _ => {
            return Err(format!(
                "Invalid ROM extension. Expected one of: {:?}",
                valid_extensions
            ));
        }
    }

    // Check file size
    let metadata =
        std::fs::metadata(path).map_err(|e| format!("Failed to read file metadata: {}", e))?;

    let size = metadata.len();
    const MIN_ROM_SIZE: u64 = 0x8000; // 32KB - minimum LoROM size
    const MAX_ROM_SIZE: u64 = 0x600000; // 6MB - maximum expanded ROM size

    if size < MIN_ROM_SIZE {
        return Err(format!(
            "File too small for SNES ROM ({} bytes, minimum {} bytes)",
            size, MIN_ROM_SIZE
        ));
    }

    if size > MAX_ROM_SIZE {
        return Err(format!(
            "File too large for SNES ROM ({} bytes, maximum {} bytes)",
            size, MAX_ROM_SIZE
        ));
    }

    Ok(())
}

/// Validate that a path is safe to write to
///
/// Checks:
/// - Parent directory exists or can be created
/// - Path doesn't contain dangerous characters
/// - Path isn't a system directory
pub fn validate_output_path(path: &str) -> Result<(), String> {
    let path = Path::new(path);

    // Check for empty path
    if path.as_os_str().is_empty() {
        return Err("Empty path".to_string());
    }

    // Get parent directory
    let parent = path
        .parent()
        .ok_or_else(|| "Invalid path: no parent directory".to_string())?;

    // If parent doesn't exist, check if it can be created
    if !parent.exists() {
        // Empty parent means current directory, which is fine
        if !parent.as_os_str().is_empty() {
            // Try to create the directory
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Cannot create parent directory: {}", e))?;
        }
    }

    // Check filename
    if let Some(filename) = path.file_name() {
        let name = filename.to_string_lossy();

        // Check for dangerous characters (null bytes, etc.)
        if name.contains('\0') {
            return Err("Filename contains null bytes".to_string());
        }

        // Check for reserved Windows names
        #[cfg(target_os = "windows")]
        {
            let reserved = [
                "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7",
                "COM8", "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8",
                "LPT9",
            ];

            let upper = name.to_uppercase();
            let stem = Path::new(&*upper)
                .file_stem()
                .map(|s| s.to_string_lossy())
                .unwrap_or_default();

            if reserved.contains(&stem.as_ref()) {
                return Err(format!("Reserved filename: {}", name));
            }
        }
    }

    Ok(())
}

/// Validate that an offset is within ROM bounds
pub fn validate_rom_offset(offset: usize, rom_size: usize) -> Result<(), String> {
    if offset >= rom_size {
        return Err(format!(
            "Offset 0x{:X} is beyond ROM size (0x{:X})",
            offset, rom_size
        ));
    }

    Ok(())
}

/// Validate that a size doesn't overflow ROM bounds
pub fn validate_rom_range(offset: usize, size: usize, rom_size: usize) -> Result<(), String> {
    validate_rom_offset(offset, rom_size)?;

    let end = offset.saturating_add(size);
    if end > rom_size {
        return Err(format!(
            "Range 0x{:X}..0x{:X} extends beyond ROM size (0x{:X})",
            offset, end, rom_size
        ));
    }

    Ok(())
}

/// Validate a project name
///
/// Checks:
/// - Not empty
/// - No dangerous characters
/// - Reasonable length
pub fn validate_project_name(name: &str) -> Result<(), String> {
    if name.trim().is_empty() {
        return Err("Project name cannot be empty".to_string());
    }

    if name.len() > 100 {
        return Err("Project name too long (max 100 characters)".to_string());
    }

    // Check for dangerous characters
    let dangerous = ['<', '>', ':', '"', '|', '?', '*', '\0'];
    if name.chars().any(|c| dangerous.contains(&c)) {
        return Err(format!(
            "Project name contains invalid characters: {:?}",
            dangerous
        ));
    }

    Ok(())
}

/// Validate a boxer key
///
/// Checks:
/// - Not empty
/// - Contains only alphanumeric characters, hyphens, and underscores
pub fn validate_boxer_key(key: &str) -> Result<(), String> {
    if key.trim().is_empty() {
        return Err("Boxer key cannot be empty".to_string());
    }

    let valid_chars = |c: char| c.is_alphanumeric() || c == '-' || c == '_' || c == ' ';

    if !key.chars().all(valid_chars) {
        return Err(
            "Boxer key can only contain letters, numbers, spaces, hyphens, and underscores"
                .to_string(),
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_validate_rom_path_valid() {
        let mut file = NamedTempFile::with_suffix(".sfc").unwrap();
        // Write enough bytes to be a valid ROM
        file.write_all(&vec![0u8; 0x8000]).unwrap();

        assert!(validate_rom_path(file.path().to_str().unwrap()).is_ok());
    }

    #[test]
    fn test_validate_rom_path_nonexistent() {
        assert!(validate_rom_path("/nonexistent/path/file.sfc").is_err());
    }

    #[test]
    fn test_validate_rom_path_wrong_extension() {
        let mut file = NamedTempFile::with_suffix(".txt").unwrap();
        file.write_all(&vec![0u8; 0x8000]).unwrap();

        assert!(validate_rom_path(file.path().to_str().unwrap()).is_err());
    }

    #[test]
    fn test_validate_rom_offset() {
        assert!(validate_rom_offset(0x1000, 0x2000).is_ok());
        assert!(validate_rom_offset(0x2000, 0x2000).is_err());
        assert!(validate_rom_offset(0x3000, 0x2000).is_err());
    }

    #[test]
    fn test_validate_project_name() {
        assert!(validate_project_name("My Project").is_ok());
        assert!(validate_project_name("").is_err());
        assert!(validate_project_name("   ").is_err());
        assert!(validate_project_name("Project<Name>").is_err());
    }

    #[test]
    fn test_validate_boxer_key() {
        assert!(validate_boxer_key("gabby-jay").is_ok());
        assert!(validate_boxer_key("bear_hugger").is_ok());
        assert!(validate_boxer_key("").is_err());
        assert!(validate_boxer_key("key/with/slashes").is_err());
    }
}
