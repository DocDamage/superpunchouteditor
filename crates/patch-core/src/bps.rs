//! BPS (Binary Patch System) patch generation
//!
//! BPS is a more efficient patch format than IPS, supporting:
//! - Files > 16MB (not needed for SNES but good practice)
//! - Delta encoding for better compression
//! - Metadata (author, description)
//! - CRC32 validation

use std::io::{self, Write};

/// BPS metadata for the patch header
#[derive(Debug, Clone, Default)]
pub struct BpsMetadata {
    /// Patch name/title
    pub patch_name: Option<String>,
    /// Patch author name
    pub author: Option<String>,
    /// Patch description
    pub description: Option<String>,
}

impl BpsMetadata {
    /// Create metadata with author and description
    pub fn new(author: Option<String>, description: Option<String>) -> Self {
        Self {
            patch_name: None,
            author,
            description,
        }
    }
    
    /// Create metadata with patch name, author and description
    pub fn with_name(patch_name: Option<String>, author: Option<String>, description: Option<String>) -> Self {
        Self {
            patch_name,
            author,
            description,
        }
    }
}

/// BPS action types
#[derive(Debug, Clone, Copy, PartialEq)]
enum BpsAction {
    /// Read from source file (identical bytes)
    SourceRead,
    /// Read literal data from patch
    TargetRead,
    /// Copy from source file at offset
    SourceCopy,
    /// Copy from already-written target data
    TargetCopy,
}

/// Encode a number using BPS variable-length encoding
/// Each byte has 7 data bits, high bit indicates continuation
fn encode_number(n: u64) -> Vec<u8> {
    let mut result = Vec::new();
    let mut n = n;

    loop {
        let mut byte = (n & 0x7F) as u8;
        n >>= 7;
        if n > 0 {
            byte |= 0x80; // Set continuation bit
        }
        result.push(byte);
        if n == 0 {
            break;
        }
    }

    result
}

/// Calculate CRC32 using the standard polynomial (IEEE 802.3)
/// Same algorithm as used in BPS specification
fn crc32(data: &[u8]) -> u32 {
    const CRC_TABLE: [u32; 256] = {
        let mut table = [0u32; 256];
        let mut i = 0;
        while i < 256 {
            let mut crc = i as u32;
            let mut j = 0;
            while j < 8 {
                crc = if crc & 1 != 0 {
                    0xEDB88320 ^ (crc >> 1)
                } else {
                    crc >> 1
                };
                j += 1;
            }
            table[i] = crc;
            i += 1;
        }
        table
    };

    let mut crc: u32 = 0xFFFFFFFF;
    for byte in data {
        crc = CRC_TABLE[((crc ^ (*byte as u32)) & 0xFF) as usize] ^ (crc >> 8);
    }
    !crc
}

/// Create BPS metadata string (null-separated key-value pairs, double-null terminated)
fn create_metadata_string(metadata: &BpsMetadata) -> Vec<u8> {
    let mut result = Vec::new();

    if let Some(author) = &metadata.author {
        result.extend_from_slice(b"author\0");
        result.extend_from_slice(author.as_bytes());
        result.push(0);
    }

    if let Some(description) = &metadata.description {
        result.extend_from_slice(b"description\0");
        result.extend_from_slice(description.as_bytes());
        result.push(0);
    }

    // Double-null terminator
    if !result.is_empty() {
        result.push(0);
    }

    result
}

/// Generate a BPS patch from original and modified data
///
/// # Arguments
/// * `original` - The original ROM data
/// * `modified` - The modified ROM data with changes applied
/// * `metadata` - Optional metadata (author, description)
///
/// # Returns
/// The BPS patch as a byte vector
pub fn generate_bps(
    original: &[u8],
    modified: &[u8],
    metadata: &BpsMetadata,
) -> io::Result<Vec<u8>> {
    let mut output = Vec::new();

    // Magic header
    output.write_all(b"BPS1")?;

    // Source and target sizes (variable length encoded)
    output.write_all(&encode_number(original.len() as u64))?;
    output.write_all(&encode_number(modified.len() as u64))?;

    // Metadata (variable length encoded size + content)
    let metadata_bytes = create_metadata_string(metadata);
    output.write_all(&encode_number(metadata_bytes.len() as u64))?;
    output.write_all(&metadata_bytes)?;

    // Generate delta actions
    let actions = generate_actions(original, modified);

    // Write actions
    write_actions(&mut output, &actions, original, modified)?;

    // Footer: source CRC, target CRC, patch CRC
    output.write_all(&crc32(original).to_le_bytes())?;
    output.write_all(&crc32(modified).to_le_bytes())?;

    // Patch CRC is calculated over everything except the last 4 bytes (the patch CRC itself)
    let patch_crc = crc32(&output);
    output.write_all(&patch_crc.to_le_bytes())?;

    Ok(output)
}

/// BPS action with length and optional offset/data
#[derive(Debug, Clone)]
struct Action {
    action_type: BpsAction,
    length: u64,
    offset: Option<u64>,   // For SourceCopy and TargetCopy
    data: Option<Vec<u8>>, // For TargetRead
}

/// Generate the list of BPS actions to transform original into modified
fn generate_actions(original: &[u8], modified: &[u8]) -> Vec<Action> {
    let mut actions = Vec::new();
    let mut output_pos: usize = 0;
    let min_len = original.len().min(modified.len());

    while output_pos < modified.len() {
        // Try different action types and pick the best one
        let source_read_len = calculate_source_read_len(original, modified, output_pos, min_len);
        let (target_read_len, target_read_data) =
            calculate_target_read(original, modified, output_pos, min_len);
        let (source_copy_len, source_copy_offset) =
            calculate_source_copy(original, modified, output_pos, min_len);
        let (target_copy_len, target_copy_offset) = calculate_target_copy(modified, output_pos);

        // Pick the longest action
        let mut best_action = Action {
            action_type: BpsAction::TargetRead,
            length: target_read_len as u64,
            offset: None,
            data: Some(target_read_data),
        };
        let mut best_len = target_read_len;

        if source_read_len > best_len {
            best_action = Action {
                action_type: BpsAction::SourceRead,
                length: source_read_len as u64,
                offset: None,
                data: None,
            };
            best_len = source_read_len;
        }

        if source_copy_len > best_len {
            best_action = Action {
                action_type: BpsAction::SourceCopy,
                length: source_copy_len as u64,
                offset: Some(source_copy_offset as u64),
                data: None,
            };
            best_len = source_copy_len;
        }

        if target_copy_len > best_len {
            best_action = Action {
                action_type: BpsAction::TargetCopy,
                length: target_copy_len as u64,
                offset: Some(target_copy_offset as u64),
                data: None,
            };
            best_len = target_copy_len;
        }

        output_pos += best_len;
        actions.push(best_action);
    }

    // Optimize: merge consecutive SourceRead actions
    optimize_actions(&mut actions);

    actions
}

/// Calculate how many bytes can be SourceRead (identical in both files)
fn calculate_source_read_len(
    original: &[u8],
    modified: &[u8],
    pos: usize,
    min_len: usize,
) -> usize {
    if pos >= min_len {
        return 0;
    }

    let mut len = 0;
    while pos + len < min_len && original[pos + len] == modified[pos + len] {
        len += 1;
        // Prevent excessively long single actions - BPS can encode very large lengths
        // but we chunk at a reasonable size for memory efficiency
        if len >= 65536 {
            break;
        }
    }
    len
}

/// Calculate TargetRead (literal data from modified file that's different from original)
fn calculate_target_read(
    original: &[u8],
    modified: &[u8],
    pos: usize,
    min_len: usize,
) -> (usize, Vec<u8>) {
    let mut len = 0;
    let mut data = Vec::new();

    // Read until we find bytes that match original again
    while pos + len < modified.len() {
        if pos + len < min_len && original[pos + len] == modified[pos + len] {
            // This byte matches, stop here
            break;
        }
        data.push(modified[pos + len]);
        len += 1;
        // Chunk large TargetRead actions
        if len >= 65536 {
            break;
        }
    }

    (len, data)
}

/// Calculate SourceCopy (copy a chunk from source file at a different offset)
fn calculate_source_copy(
    original: &[u8],
    modified: &[u8],
    pos: usize,
    min_len: usize,
) -> (usize, usize) {
    if pos >= min_len {
        return (0, 0);
    }

    // Look for a matching chunk in the original file
    // Simple approach: just check if there's a match at a different position
    // For efficiency, we use a hash-based approach for larger files

    let target_chunk = &modified[pos..modified.len().min(pos + 16)];
    if target_chunk.is_empty() {
        return (0, 0);
    }

    // Simple search for small files (SNES ROMs are ~2-4MB)
    // For larger files, a rolling hash would be more efficient
    let search_start = if pos > 32768 { pos - 32768 } else { 0 };
    let search_end = original.len().min(pos + 32768);

    let mut best_len = 0;
    let mut best_offset = 0;

    for offset in search_start..search_end {
        if offset == pos || offset + target_chunk.len() > original.len() {
            continue;
        }

        // Quick check first byte
        if original[offset] != target_chunk[0] {
            continue;
        }

        // Check full match
        let mut match_len = 0;
        while offset + match_len < original.len()
            && pos + match_len < modified.len()
            && original[offset + match_len] == modified[pos + match_len]
            && match_len < 65536
        {
            match_len += 1;
        }

        if match_len > best_len && match_len >= 4 {
            best_len = match_len;
            best_offset = offset;
        }
    }

    (best_len, best_offset)
}

/// Calculate TargetCopy (copy from already-written target data)
fn calculate_target_copy(modified: &[u8], pos: usize) -> (usize, usize) {
    if pos == 0 {
        return (0, 0);
    }

    // Look for a matching chunk in the already-written target data
    let target_chunk = &modified[pos..modified.len().min(pos + 16)];
    if target_chunk.is_empty() {
        return (0, 0);
    }

    // Search backwards in the already-written portion
    let search_end = pos;
    let search_start = pos.saturating_sub(65536);

    let mut best_len = 0;
    let mut best_offset = 0;

    for offset in search_start..search_end {
        if offset + target_chunk.len() > modified.len() {
            continue;
        }

        if modified[offset] != target_chunk[0] {
            continue;
        }

        let mut match_len = 0;
        while offset + match_len < pos
            && pos + match_len < modified.len()
            && modified[offset + match_len] == modified[pos + match_len]
            && match_len < 65536
        {
            match_len += 1;
        }

        if match_len > best_len && match_len >= 4 {
            best_len = match_len;
            best_offset = offset;
        }
    }

    (best_len, best_offset)
}

/// Optimize actions by merging consecutive SourceRead actions
fn optimize_actions(actions: &mut Vec<Action>) {
    if actions.len() < 2 {
        return;
    }

    let mut i = 0;
    while i + 1 < actions.len() {
        if actions[i].action_type == BpsAction::SourceRead
            && actions[i + 1].action_type == BpsAction::SourceRead
        {
            // Merge them
            actions[i].length += actions[i + 1].length;
            actions.remove(i + 1);
        } else {
            i += 1;
        }
    }
}

/// Write actions to the output buffer in BPS format
fn write_actions(
    output: &mut Vec<u8>,
    actions: &[Action],
    _original: &[u8],
    _modified: &[u8],
) -> io::Result<()> {
    for action in actions {
        match action.action_type {
            BpsAction::SourceRead => {
                // SourceRead: length encoded as (length - 1) * 4 + 0
                let encoded = (action.length - 1) * 4 + 0;
                output.write_all(&encode_number(encoded))?;
            }
            BpsAction::TargetRead => {
                // TargetRead: length encoded as (length - 1) * 4 + 1, followed by data
                let encoded = (action.length - 1) * 4 + 1;
                output.write_all(&encode_number(encoded))?;
                if let Some(data) = &action.data {
                    output.write_all(data)?;
                }
            }
            BpsAction::SourceCopy => {
                // SourceCopy: length encoded as (length - 1) * 4 + 2, followed by relative offset
                let encoded = (action.length - 1) * 4 + 2;
                output.write_all(&encode_number(encoded))?;
                // Offset is encoded as a signed relative offset
                let offset = action.offset.unwrap_or(0) as i64;
                output.write_all(&encode_signed_number(offset))?;
            }
            BpsAction::TargetCopy => {
                // TargetCopy: length encoded as (length - 1) * 4 + 3, followed by relative offset
                let encoded = (action.length - 1) * 4 + 3;
                output.write_all(&encode_number(encoded))?;
                let offset = action.offset.unwrap_or(0) as i64;
                output.write_all(&encode_signed_number(offset))?;
            }
        }
    }

    Ok(())
}

/// Encode a signed number using BPS variable-length encoding
fn encode_signed_number(n: i64) -> Vec<u8> {
    // BPS uses a specific encoding for signed numbers:
    // The sign is stored in bit 0, and the absolute value is shifted right by 1
    let (sign_bit, abs_value) = if n < 0 {
        (1, (-n) as u64)
    } else {
        (0, n as u64)
    };

    let encoded = (abs_value << 1) | sign_bit as u64;
    encode_number(encoded)
}

/// Generate a BPS patch and write it directly to a file
///
/// # Arguments
/// * `original` - The original ROM data
/// * `modified` - The modified ROM data with changes applied
/// * `output_path` - Path to write the BPS patch file
/// * `metadata` - Optional metadata (author, description)
pub fn generate_bps_to_file(
    original: &[u8],
    modified: &[u8],
    output_path: &str,
    metadata: &BpsMetadata,
) -> io::Result<()> {
    let patch_data = generate_bps(original, modified, metadata)?;
    let mut file = std::fs::File::create(output_path)?;
    file.write_all(&patch_data)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_number() {
        // Small numbers fit in one byte
        assert_eq!(encode_number(0), vec![0x00]);
        assert_eq!(encode_number(127), vec![0x7F]);

        // Larger numbers need continuation
        assert_eq!(encode_number(128), vec![0x80, 0x01]);
        assert_eq!(encode_number(255), vec![0xFF, 0x01]);
        assert_eq!(encode_number(256), vec![0x80, 0x02]);
    }

    #[test]
    fn test_crc32() {
        // Test CRC32 calculation
        let data = b"123456789";
        let crc = crc32(data);
        // Known CRC32 value for "123456789"
        assert_eq!(crc, 0xCBF43926);
    }

    #[test]
    fn test_encode_signed_number() {
        // Test signed number encoding
        assert_eq!(encode_signed_number(0), vec![0x00]);
        assert_eq!(encode_signed_number(1), vec![0x02]); // 1 << 1 = 2
        assert_eq!(encode_signed_number(-1), vec![0x03]); // (1 << 1) | 1 = 3
        assert_eq!(encode_signed_number(2), vec![0x04]); // 2 << 1 = 4
        assert_eq!(encode_signed_number(-2), vec![0x05]); // (2 << 1) | 1 = 5
    }

    #[test]
    fn test_create_metadata() {
        let metadata = BpsMetadata::new(
            Some("Test Author".to_string()),
            Some("Test Description".to_string()),
        );
        let bytes = create_metadata_string(&metadata);
        let expected = b"author\0Test Author\0description\0Test Description\0\0";
        assert_eq!(bytes, expected.to_vec());
    }

    #[test]
    fn test_generate_bps_simple() {
        let original = vec![0x00, 0x01, 0x02, 0x03, 0x04];
        let modified = vec![0x00, 0x01, 0xFF, 0x03, 0x04];
        let metadata = BpsMetadata::default();

        let patch = generate_bps(&original, &modified, &metadata).unwrap();

        // Check magic
        assert_eq!(&patch[0..4], b"BPS1");

        // Should have valid structure (header + actions + footer)
        // Minimum size: 4 (magic) + 1 (source size) + 1 (target size) + 1 (metadata size) +
        //               12 (footer: 3 * 4 byte CRCs)
        assert!(patch.len() >= 20);
    }

    #[test]
    fn test_generate_bps_with_metadata() {
        let original = vec![0x00; 100];
        let modified = vec![0xFF; 100];
        let metadata = BpsMetadata::new(Some("Author".to_string()), None);

        let patch = generate_bps(&original, &modified, &metadata).unwrap();
        assert!(!patch.is_empty());
        assert_eq!(&patch[0..4], b"BPS1");
    }
}
