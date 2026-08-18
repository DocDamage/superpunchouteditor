//! BPS (Beat Patch System) generation and verification.
//!
//! The writer intentionally emits only SourceRead and TargetRead actions. That is less compact than
//! a copy-searching encoder, but it is deterministic, simple to audit, and fully compatible with the
//! BPS format. Export code verifies every generated patch by applying it in memory.

use std::io::{self, ErrorKind, Write};

#[derive(Debug, Clone, Default)]
pub struct BpsMetadata {
    pub patch_name: Option<String>,
    pub author: Option<String>,
    pub description: Option<String>,
}

impl BpsMetadata {
    pub fn new(author: Option<String>, description: Option<String>) -> Self {
        Self {
            patch_name: None,
            author,
            description,
        }
    }

    pub fn with_name(
        patch_name: Option<String>,
        author: Option<String>,
        description: Option<String>,
    ) -> Self {
        Self {
            patch_name,
            author,
            description,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u64)]
enum BpsAction {
    SourceRead = 0,
    TargetRead = 1,
    SourceCopy = 2,
    TargetCopy = 3,
}

/// BPS uses a biased variable-length integer, not ordinary LEB128.
fn encode_number(mut value: u64) -> Vec<u8> {
    let mut output = Vec::new();
    loop {
        let byte = (value & 0x7f) as u8;
        value >>= 7;
        if value == 0 {
            output.push(byte | 0x80);
            break;
        }
        output.push(byte);
        value -= 1;
    }
    output
}

fn decode_number(data: &[u8], cursor: &mut usize) -> io::Result<u64> {
    let mut value = 0u64;
    let mut shift = 1u64;
    loop {
        let byte = *data
            .get(*cursor)
            .ok_or_else(|| io::Error::new(ErrorKind::UnexpectedEof, "truncated BPS integer"))?;
        *cursor += 1;
        value = value
            .checked_add(((byte & 0x7f) as u64).saturating_mul(shift))
            .ok_or_else(|| io::Error::new(ErrorKind::InvalidData, "BPS integer overflow"))?;
        if byte & 0x80 != 0 {
            return Ok(value);
        }
        shift = shift
            .checked_shl(7)
            .ok_or_else(|| io::Error::new(ErrorKind::InvalidData, "BPS integer overflow"))?;
        value = value
            .checked_add(shift)
            .ok_or_else(|| io::Error::new(ErrorKind::InvalidData, "BPS integer overflow"))?;
    }
}

fn encode_signed(value: i64) -> Vec<u8> {
    let magnitude = value.unsigned_abs();
    let encoded = (magnitude << 1) | u64::from(value < 0);
    encode_number(encoded)
}

fn decode_signed(data: &[u8], cursor: &mut usize) -> io::Result<i64> {
    let encoded = decode_number(data, cursor)?;
    let magnitude = (encoded >> 1) as i64;
    Ok(if encoded & 1 != 0 {
        -magnitude
    } else {
        magnitude
    })
}

fn crc32(data: &[u8]) -> u32 {
    const CRC_TABLE: [u32; 256] = {
        let mut table = [0u32; 256];
        let mut i = 0usize;
        while i < 256 {
            let mut crc = i as u32;
            let mut j = 0;
            while j < 8 {
                crc = if crc & 1 != 0 {
                    0xedb8_8320 ^ (crc >> 1)
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

    let mut crc = 0xffff_ffffu32;
    for byte in data {
        crc = CRC_TABLE[((crc ^ u32::from(*byte)) & 0xff) as usize] ^ (crc >> 8);
    }
    !crc
}

fn metadata_bytes(metadata: &BpsMetadata) -> Vec<u8> {
    let mut fields = Vec::new();
    if let Some(value) = &metadata.patch_name {
        fields.push(format!("name={value}"));
    }
    if let Some(value) = &metadata.author {
        fields.push(format!("author={value}"));
    }
    if let Some(value) = &metadata.description {
        fields.push(format!("description={value}"));
    }
    fields.join("\n").into_bytes()
}

fn write_action(output: &mut Vec<u8>, action: BpsAction, length: usize) -> io::Result<()> {
    if length == 0 {
        return Err(io::Error::new(
            ErrorKind::InvalidInput,
            "BPS action length must be non-zero",
        ));
    }
    let encoded = ((length as u64 - 1) << 2) | action as u64;
    output.write_all(&encode_number(encoded))
}

/// Generate a standards-compatible BPS patch.
pub fn generate_bps(
    original: &[u8],
    modified: &[u8],
    metadata: &BpsMetadata,
) -> io::Result<Vec<u8>> {
    let mut output = Vec::new();
    output.write_all(b"BPS1")?;
    output.write_all(&encode_number(original.len() as u64))?;
    output.write_all(&encode_number(modified.len() as u64))?;

    let metadata = metadata_bytes(metadata);
    output.write_all(&encode_number(metadata.len() as u64))?;
    output.write_all(&metadata)?;

    let mut position = 0usize;
    while position < modified.len() {
        if position < original.len() && original[position] == modified[position] {
            let start = position;
            while position < modified.len()
                && position < original.len()
                && original[position] == modified[position]
            {
                position += 1;
            }
            write_action(&mut output, BpsAction::SourceRead, position - start)?;
        } else {
            let start = position;
            while position < modified.len()
                && (position >= original.len() || original[position] != modified[position])
            {
                position += 1;
            }
            write_action(&mut output, BpsAction::TargetRead, position - start)?;
            output.write_all(&modified[start..position])?;
        }
    }

    output.write_all(&crc32(original).to_le_bytes())?;
    output.write_all(&crc32(modified).to_le_bytes())?;
    let patch_crc = crc32(&output);
    output.write_all(&patch_crc.to_le_bytes())?;
    Ok(output)
}

/// Apply a BPS patch in memory and verify source, target, and patch CRCs.
pub fn apply_bps(original: &[u8], patch: &[u8]) -> io::Result<Vec<u8>> {
    if patch.len() < 4 + 12 || &patch[..4] != b"BPS1" {
        return Err(io::Error::new(ErrorKind::InvalidData, "invalid BPS header"));
    }

    let footer_start = patch
        .len()
        .checked_sub(12)
        .ok_or_else(|| io::Error::new(ErrorKind::InvalidData, "truncated BPS footer"))?;
    let source_crc = u32::from_le_bytes(
        patch[footer_start..footer_start + 4]
            .try_into()
            .map_err(|_| io::Error::new(ErrorKind::InvalidData, "invalid source CRC"))?,
    );
    let target_crc = u32::from_le_bytes(
        patch[footer_start + 4..footer_start + 8]
            .try_into()
            .map_err(|_| io::Error::new(ErrorKind::InvalidData, "invalid target CRC"))?,
    );
    let patch_crc = u32::from_le_bytes(
        patch[footer_start + 8..footer_start + 12]
            .try_into()
            .map_err(|_| io::Error::new(ErrorKind::InvalidData, "invalid patch CRC"))?,
    );
    if source_crc != crc32(original) {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "BPS source CRC does not match the supplied base image",
        ));
    }
    if patch_crc != crc32(&patch[..patch.len() - 4]) {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "BPS patch CRC mismatch",
        ));
    }

    let mut cursor = 4usize;
    let source_size = usize::try_from(decode_number(patch, &mut cursor)?)
        .map_err(|_| io::Error::new(ErrorKind::InvalidData, "source size overflow"))?;
    let target_size = usize::try_from(decode_number(patch, &mut cursor)?)
        .map_err(|_| io::Error::new(ErrorKind::InvalidData, "target size overflow"))?;
    let metadata_size = usize::try_from(decode_number(patch, &mut cursor)?)
        .map_err(|_| io::Error::new(ErrorKind::InvalidData, "metadata size overflow"))?;
    if source_size != original.len() {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "BPS source size mismatch",
        ));
    }
    cursor = cursor
        .checked_add(metadata_size)
        .filter(|value| *value <= footer_start)
        .ok_or_else(|| io::Error::new(ErrorKind::InvalidData, "invalid BPS metadata size"))?;

    let mut target = Vec::with_capacity(target_size);
    let mut source_relative = 0i64;
    let mut target_relative = 0i64;

    while target.len() < target_size {
        if cursor >= footer_start {
            return Err(io::Error::new(
                ErrorKind::UnexpectedEof,
                "truncated BPS actions",
            ));
        }
        let command = decode_number(patch, &mut cursor)?;
        let action = command & 3;
        let length = usize::try_from((command >> 2) + 1)
            .map_err(|_| io::Error::new(ErrorKind::InvalidData, "BPS action length overflow"))?;
        if target.len().saturating_add(length) > target_size {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                "BPS action exceeds target size",
            ));
        }

        match action {
            0 => {
                let start = target.len();
                let end = start.checked_add(length).ok_or_else(|| {
                    io::Error::new(ErrorKind::InvalidData, "BPS source read overflow")
                })?;
                let bytes = original.get(start..end).ok_or_else(|| {
                    io::Error::new(ErrorKind::InvalidData, "BPS source read out of range")
                })?;
                target.extend_from_slice(bytes);
            }
            1 => {
                let end = cursor.checked_add(length).ok_or_else(|| {
                    io::Error::new(ErrorKind::InvalidData, "BPS target read overflow")
                })?;
                if end > footer_start {
                    return Err(io::Error::new(
                        ErrorKind::UnexpectedEof,
                        "truncated target data",
                    ));
                }
                target.extend_from_slice(&patch[cursor..end]);
                cursor = end;
            }
            2 => {
                source_relative = source_relative
                    .checked_add(decode_signed(patch, &mut cursor)?)
                    .ok_or_else(|| {
                        io::Error::new(ErrorKind::InvalidData, "source copy overflow")
                    })?;
                if source_relative < 0 {
                    return Err(io::Error::new(
                        ErrorKind::InvalidData,
                        "negative source copy",
                    ));
                }
                let start = source_relative as usize;
                let end = start.checked_add(length).ok_or_else(|| {
                    io::Error::new(ErrorKind::InvalidData, "source copy range overflow")
                })?;
                let bytes = original.get(start..end).ok_or_else(|| {
                    io::Error::new(ErrorKind::InvalidData, "source copy out of range")
                })?;
                target.extend_from_slice(bytes);
                source_relative = end as i64;
            }
            3 => {
                target_relative = target_relative
                    .checked_add(decode_signed(patch, &mut cursor)?)
                    .ok_or_else(|| {
                        io::Error::new(ErrorKind::InvalidData, "target copy overflow")
                    })?;
                if target_relative < 0 {
                    return Err(io::Error::new(
                        ErrorKind::InvalidData,
                        "negative target copy",
                    ));
                }
                for _ in 0..length {
                    let source_index = target_relative as usize;
                    let byte = *target.get(source_index).ok_or_else(|| {
                        io::Error::new(ErrorKind::InvalidData, "target copy out of range")
                    })?;
                    target.push(byte);
                    target_relative += 1;
                }
            }
            _ => unreachable!(),
        }
    }

    if target_crc != crc32(&target) {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "BPS target CRC mismatch",
        ));
    }
    Ok(target)
}

pub fn generate_bps_to_file(
    original: &[u8],
    modified: &[u8],
    output_path: &str,
    metadata: &BpsMetadata,
) -> io::Result<()> {
    let patch = generate_bps(original, modified, metadata)?;
    let verified = apply_bps(original, &patch)?;
    if verified != modified {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "generated BPS did not reproduce the target image",
        ));
    }
    std::fs::write(output_path, patch)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bps_number_round_trip() {
        for value in [0, 1, 127, 128, 255, 256, 16_384, 1_000_000] {
            let encoded = encode_number(value);
            let mut cursor = 0;
            assert_eq!(decode_number(&encoded, &mut cursor).unwrap(), value);
            assert_eq!(cursor, encoded.len());
        }
    }

    #[test]
    fn signed_number_round_trip() {
        for value in [-10_000, -1, 0, 1, 10_000] {
            let encoded = encode_signed(value);
            let mut cursor = 0;
            assert_eq!(decode_signed(&encoded, &mut cursor).unwrap(), value);
        }
    }

    #[test]
    fn crc32_known_vector() {
        assert_eq!(crc32(b"123456789"), 0xcbf4_3926);
    }

    #[test]
    fn generated_patch_reproduces_same_size_target() {
        let source = vec![0, 1, 2, 3, 4, 5];
        let target = vec![0, 1, 9, 8, 4, 5];
        let patch = generate_bps(&source, &target, &BpsMetadata::default()).unwrap();
        assert_eq!(apply_bps(&source, &patch).unwrap(), target);
    }

    #[test]
    fn generated_patch_reproduces_expanded_target() {
        let source = vec![0, 1, 2];
        let target = vec![0, 1, 2, 3, 4, 5];
        let patch = generate_bps(&source, &target, &BpsMetadata::default()).unwrap();
        assert_eq!(apply_bps(&source, &patch).unwrap(), target);
    }

    #[test]
    fn wrong_source_is_rejected() {
        let source = vec![0, 1, 2];
        let target = vec![0, 9, 2];
        let patch = generate_bps(&source, &target, &BpsMetadata::default()).unwrap();
        assert!(apply_bps(&[9, 9, 9], &patch).is_err());
    }
}
