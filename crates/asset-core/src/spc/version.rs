/// SPC file version.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpcVersion {
    /// Version 0.30 (most common)
    V030,
    /// Version 0.31 (extended)
    V031,
    /// Unknown version
    Unknown,
}

impl SpcVersion {
    /// Detects version from header bytes.
    pub fn from_bytes(minor: u8, major: u8) -> Self {
        match (major, minor) {
            (0x30, 0x30) => Self::V030,
            (0x31, 0x30) => Self::V031,
            _ => Self::Unknown,
        }
    }
}
