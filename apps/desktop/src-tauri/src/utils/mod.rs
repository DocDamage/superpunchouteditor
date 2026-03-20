//! Utility Functions Module
//!
//! Shared helper functions and utilities used across the application.

pub mod validation;

// Re-export commonly used utilities
pub use validation::*;

use std::path::Path;

/// Parse an offset string (hex or decimal) into a usize
///
/// Supports formats:
/// - Hexadecimal: "0x1234", "0XABC"
/// - Decimal: "4660"
///
/// # Examples
/// ```
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
/// ```
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
/// ```
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
}
