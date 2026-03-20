//! ROM Region Detection and Configuration
//!
//! This module provides support for multiple regional versions of Super Punch-Out!!
//! including USA, Japanese (JPN), and PAL (European) versions.
//!
//! ## Supported Regions
//!
//! - **USA**: Fully supported, native development target
//! - **JPN**: Planned support (addresses TBD)
//! - **PAL**: Planned support (addresses TBD)
//!
//! ## Research Notes
//!
//! JPN and PAL region support is disabled until ROM addresses are researched.
//!
//! ### JPN Version Research Needed:
//! - ROM SHA1 hash
//! - Internal header title (usually "SUPER PUNCH-OUT!!" in Japanese)
//! - Fighter header table address
//! - Palette table address
//! - Sprite table address
//! - Text/translation table address
//! - Music table address
//! - Any JPN-specific text encoding differences
//!
//! ### PAL Version Research Needed:
//! - ROM SHA1 hash
//! - Internal header title
//! - Fighter header table address
//! - Palette table address
//! - Sprite table address
//! - Text table address (may have multiple languages)
//! - Music table address
//! - Timing adjustments for 50Hz
//! - PAL-specific boxer name translations

use crate::Rom;
use serde::{Deserialize, Serialize};

/// ROM region variants for Super Punch-Out!!
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RomRegion {
    /// USA version - fully supported
    Usa,
    /// Japanese version - planned support
    Jpn,
    /// European/PAL version - planned support
    Pal,
}

impl RomRegion {
    /// Known SHA1 hashes for verified ROM dumps
    pub const KNOWN_SHA1_USA: &'static str = "3604c855790f37db567e9b425252625045f86697";
    // NOTE: JPN and PAL SHA1 hashes need to be added when those regions are supported

    /// Detect the ROM region from ROM data
    ///
    /// Uses a combination of SHA1 hash verification and internal header checking
    /// to determine which regional version a ROM represents.
    ///
    /// # Arguments
    /// * `rom` - The ROM to analyze
    ///
    /// # Returns
    /// * `Some(RomRegion)` if the ROM is recognized
    /// * `None` if the ROM doesn't match any known region
    pub fn detect(rom: &Rom) -> Option<Self> {
        let sha1 = rom.calculate_sha1();

        // Check SHA1 against known hashes first
        match sha1.as_str() {
            Self::KNOWN_SHA1_USA => return Some(RomRegion::Usa),
            // NOTE: JPN and PAL SHA1 checks to be added when those regions are supported
            _ => {}
        }

        // Fall back to header detection if SHA1 doesn't match
        Self::detect_from_header(rom)
    }

    /// Attempt to detect region from internal SNES header
    ///
    /// The SNES header is located at various offsets depending on the mapping mode.
    /// For LoROM, it's typically at $7FC0-$7FFF (PC: 0x007FC0)
    ///
    /// # Header Offsets (PC):
    /// - Title: 0x007FC0 - 0x007FD4 (21 bytes)
    /// - Map/Type: 0x007FD5
    /// - ROM Size: 0x007FD7
    /// - Version: 0x007FDB
    fn detect_from_header(rom: &Rom) -> Option<Self> {
        // SNES LoROM header location (with and without SMC header)
        let header_offsets = [0x007FC0, 0x0081C0]; // without, with SMC header

        for &base in &header_offsets {
            if base + 21 > rom.data.len() {
                continue;
            }

            let title = &rom.data[base..base + 21];
            let title_str = String::from_utf8_lossy(title);

            // USA title check
            if title_str.contains("SUPER PUNCH-OUT!!") {
                return Some(RomRegion::Usa);
            }

            // NOTE: JPN and PAL title checks to be added when those regions are supported
        }

        None
    }

    /// Display name for UI presentation
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Usa => "Super Punch-Out!! (USA)",
            Self::Jpn => "Super Punch-Out!! (Japan)",
            Self::Pal => "Super Punch-Out!! (Europe)",
        }
    }

    /// Short code for the region (for filenames, etc.)
    pub fn code(&self) -> &'static str {
        match self {
            Self::Usa => "USA",
            Self::Jpn => "JPN",
            Self::Pal => "PAL",
        }
    }

    /// Check if this region is fully supported
    ///
    /// # Security Note
    /// Only USA region is supported to prevent data corruption from untested
    /// regional versions. JPN and PAL regions require additional research
    /// into memory addresses before they can be safely used.
    pub fn is_supported(&self) -> bool {
        matches!(self, RomRegion::Usa) // Only USA for now - JPN/PAL need address research
    }

    /// Get support status description for UI
    pub fn support_status(&self) -> &'static str {
        match self {
            Self::Usa => "Fully Supported",
            Self::Jpn => "Planned - Research Needed",
            Self::Pal => "Planned - Research Needed",
        }
    }

    /// Get list of known SHA1 hashes for this region
    pub fn known_hashes(&self) -> &[&'static str] {
        match self {
            Self::Usa => &[Self::KNOWN_SHA1_USA],
            Self::Jpn => &[], // NOTE: JPN hashes to be added when region is supported
            Self::Pal => &[], // NOTE: PAL hashes to be added when region is supported
        }
    }

    /// Check if a SHA1 hash matches this region
    pub fn matches_hash(&self, sha1: &str) -> bool {
        self.known_hashes().contains(&sha1)
    }
}

impl std::fmt::Display for RomRegion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

impl Default for RomRegion {
    fn default() -> Self {
        Self::Usa
    }
}

/// Region-specific memory addresses and configuration
///
/// This struct contains all the memory addresses and offsets that vary
/// between different regional versions of the game.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionConfig {
    pub region: RomRegion,
    /// Fighter header table base address (PC offset)
    pub fighter_header_table: usize,
    /// Palette table base address (PC offset)
    pub palette_table: usize,
    /// Sprite table base address (PC offset)
    pub sprite_table: usize,
    /// Text/translation table base address (PC offset)
    pub text_table: usize,
    /// Music table base address (PC offset)
    pub music_table: usize,
    /// Script table base address (PC offset)
    pub script_table: usize,
    /// Animation data table base address (PC offset)
    pub animation_table: usize,
    /// Boxer names table address (PC offset)
    pub boxer_names_table: usize,
    /// Circuit data table address (PC offset)
    pub circuit_table: usize,
}

impl RegionConfig {
    /// Get the configuration for a specific region
    ///
    /// # Panics
    /// Panics if called for an unsupported region (JPN/PAL)
    /// until their addresses are properly researched.
    pub fn for_region(region: RomRegion) -> Self {
        match region {
            RomRegion::Usa => Self::usa(),
            RomRegion::Jpn => Self::jpn(),
            RomRegion::Pal => Self::pal(),
        }
    }

    /// USA version configuration (verified addresses)
    fn usa() -> Self {
        Self {
            region: RomRegion::Usa,
            fighter_header_table: 0x098000,
            palette_table: 0x108000,
            sprite_table: 0x118000,
            text_table: 0x128000,
            music_table: 0x138000,
            script_table: 0x148000,
            animation_table: 0x158000,
            boxer_names_table: 0x168000,
            circuit_table: 0x178000,
        }
    }

    /// JPN version configuration (RESEARCH NEEDED)
    fn jpn() -> Self {
        // NOTE: JPN addresses disabled - need research before enabling
        // These are placeholders - DO NOT USE
        Self {
            region: RomRegion::Jpn,
            fighter_header_table: 0x000000, // NOTE: Needs research
            palette_table: 0x000000,        // NOTE: Needs research
            sprite_table: 0x000000,         // NOTE: Needs research
            text_table: 0x000000,           // NOTE: Needs research
            music_table: 0x000000,          // NOTE: Needs research
            script_table: 0x000000,         // NOTE: Needs research
            animation_table: 0x000000,      // NOTE: Needs research
            boxer_names_table: 0x000000,    // NOTE: Needs research
            circuit_table: 0x000000,        // NOTE: Needs research
        }
    }

    /// PAL version configuration (RESEARCH NEEDED)
    fn pal() -> Self {
        // NOTE: PAL addresses disabled - need research before enabling
        // These are placeholders - DO NOT USE
        Self {
            region: RomRegion::Pal,
            fighter_header_table: 0x000000, // NOTE: Needs research
            palette_table: 0x000000,        // NOTE: Needs research
            sprite_table: 0x000000,         // NOTE: Needs research
            text_table: 0x000000,           // NOTE: Needs research
            music_table: 0x000000,          // NOTE: Needs research
            script_table: 0x000000,         // NOTE: Needs research
            animation_table: 0x000000,      // NOTE: Needs research
            boxer_names_table: 0x000000,    // NOTE: Needs research
            circuit_table: 0x000000,        // NOTE: Needs research
        }
    }

    /// Check if all addresses have been properly configured
    pub fn is_configured(&self) -> bool {
        // All addresses should be non-zero for a properly configured region
        self.fighter_header_table != 0
            && self.palette_table != 0
            && self.sprite_table != 0
            && self.text_table != 0
            && self.music_table != 0
    }

    /// Get a description of which addresses need research
    pub fn research_status(&self) -> Vec<(String, bool)> {
        vec![
            (
                "Fighter Headers".to_string(),
                self.fighter_header_table != 0,
            ),
            ("Palettes".to_string(), self.palette_table != 0),
            ("Sprites".to_string(), self.sprite_table != 0),
            ("Text".to_string(), self.text_table != 0),
            ("Music".to_string(), self.music_table != 0),
            ("Scripts".to_string(), self.script_table != 0),
            ("Animations".to_string(), self.animation_table != 0),
            ("Boxer Names".to_string(), self.boxer_names_table != 0),
            ("Circuit Data".to_string(), self.circuit_table != 0),
        ]
    }
}

impl Default for RegionConfig {
    fn default() -> Self {
        Self::for_region(RomRegion::Usa)
    }
}

/// Information about a detected ROM for UI display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RomRegionInfo {
    pub region: RomRegion,
    pub display_name: String,
    pub code: String,
    pub is_supported: bool,
    pub support_status: String,
    pub detected: bool,
}

impl RomRegionInfo {
    /// Create info for all available regions (for UI selector)
    pub fn all_regions() -> Vec<Self> {
        vec![
            Self::from_region(RomRegion::Usa, false),
            Self::from_region(RomRegion::Jpn, false),
            Self::from_region(RomRegion::Pal, false),
        ]
    }

    /// Create info for a specific region
    pub fn from_region(region: RomRegion, detected: bool) -> Self {
        Self {
            region,
            display_name: region.display_name().to_string(),
            code: region.code().to_string(),
            is_supported: region.is_supported(),
            support_status: region.support_status().to_string(),
            detected,
        }
    }
}

/// Result of attempting to detect a ROM's region
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionDetectionResult {
    pub success: bool,
    pub region: Option<RomRegion>,
    pub display_name: Option<String>,
    pub is_supported: bool,
    pub sha1: String,
    pub error_message: Option<String>,
}

impl RegionDetectionResult {
    /// Create a successful detection result
    pub fn success(region: RomRegion, sha1: String) -> Self {
        Self {
            success: true,
            region: Some(region),
            display_name: Some(region.display_name().to_string()),
            is_supported: region.is_supported(),
            sha1,
            error_message: None,
        }
    }

    /// Create a failure result
    pub fn failure(sha1: String, message: String) -> Self {
        Self {
            success: false,
            region: None,
            display_name: None,
            is_supported: false,
            sha1,
            error_message: Some(message),
        }
    }

    /// Detect from a ROM
    pub fn from_rom(rom: &Rom) -> Self {
        let sha1 = rom.calculate_sha1();

        match RomRegion::detect(rom) {
            Some(region) => Self::success(region, sha1),
            None => Self::failure(
                sha1,
                "Unknown ROM region - not a recognized Super Punch-Out!! version".to_string(),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_region_display_names() {
        assert_eq!(RomRegion::Usa.display_name(), "Super Punch-Out!! (USA)");
        assert_eq!(RomRegion::Jpn.display_name(), "Super Punch-Out!! (Japan)");
        assert_eq!(RomRegion::Pal.display_name(), "Super Punch-Out!! (Europe)");
    }

    #[test]
    fn test_region_codes() {
        assert_eq!(RomRegion::Usa.code(), "USA");
        assert_eq!(RomRegion::Jpn.code(), "JPN");
        assert_eq!(RomRegion::Pal.code(), "PAL");
    }

    #[test]
    fn test_region_support_status() {
        assert!(RomRegion::Usa.is_supported());
        assert!(!RomRegion::Jpn.is_supported());
        assert!(!RomRegion::Pal.is_supported());
    }

    #[test]
    fn test_region_config_usa_is_configured() {
        let config = RegionConfig::for_region(RomRegion::Usa);
        assert!(config.is_configured());
        assert_eq!(config.region, RomRegion::Usa);
    }

    #[test]
    fn test_region_config_jpn_not_configured() {
        let config = RegionConfig::for_region(RomRegion::Jpn);
        assert!(!config.is_configured());
    }

    #[test]
    fn test_region_config_pal_not_configured() {
        let config = RegionConfig::for_region(RomRegion::Pal);
        assert!(!config.is_configured());
    }

    #[test]
    fn test_detection_result_success() {
        let result = RegionDetectionResult::success(RomRegion::Usa, "test_sha1".to_string());
        assert!(result.success);
        assert!(result.is_supported);
        assert_eq!(
            result.display_name,
            Some("Super Punch-Out!! (USA)".to_string())
        );
    }

    #[test]
    fn test_detection_result_failure() {
        let result =
            RegionDetectionResult::failure("unknown_sha1".to_string(), "Test error".to_string());
        assert!(!result.success);
        assert!(!result.is_supported);
        assert_eq!(result.error_message, Some("Test error".to_string()));
    }
}
