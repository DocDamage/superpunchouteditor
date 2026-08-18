use std::fs::File;
use std::io::{self, ErrorKind, Write};

pub mod bps;
pub use bps::{apply_bps, generate_bps, generate_bps_to_file, BpsMetadata};

/// Build an IPS patch in memory. IPS cannot safely represent ROM expansion in this implementation,
/// so callers must use BPS when source and target lengths differ.
pub fn generate_ips_bytes(original: &[u8], edited: &[u8]) -> io::Result<Vec<u8>> {
    if original.len() != edited.len() {
        return Err(io::Error::new(
            ErrorKind::InvalidInput,
            "IPS export requires source and target ROMs to have equal length; use BPS for expansion",
        ));
    }

    let mut output = Vec::new();
    output.extend_from_slice(b"PATCH");

    let mut index = 0usize;
    while index < original.len() {
        if original[index] == edited[index] {
            index += 1;
            continue;
        }

        let start = index;
        while index < original.len()
            && original[index] != edited[index]
            && index - start < u16::MAX as usize
        {
            index += 1;
        }
        if start > 0x00ff_ffff {
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                "IPS record offset exceeds 24-bit format limit",
            ));
        }

        let length = index - start;
        output.extend_from_slice(&[
            ((start >> 16) & 0xff) as u8,
            ((start >> 8) & 0xff) as u8,
            (start & 0xff) as u8,
        ]);
        output.extend_from_slice(&(length as u16).to_be_bytes());
        output.extend_from_slice(&edited[start..index]);
    }

    output.extend_from_slice(b"EOF");
    Ok(output)
}

pub fn generate_ips(original: &[u8], edited: &[u8], output_path: &str) -> io::Result<()> {
    let patch = generate_ips_bytes(original, edited)?;
    let verified = apply_ips(original, &patch)?;
    if verified != edited {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "generated IPS did not reproduce the target image",
        ));
    }
    let mut file = File::create(output_path)?;
    file.write_all(&patch)
}

/// Apply an IPS patch in memory. Both ordinary and RLE records are supported.
pub fn apply_ips(original: &[u8], patch: &[u8]) -> io::Result<Vec<u8>> {
    if patch.len() < 8 || !patch.starts_with(b"PATCH") {
        return Err(io::Error::new(ErrorKind::InvalidData, "invalid IPS header"));
    }

    let mut output = original.to_vec();
    let mut cursor = 5usize;
    loop {
        if patch.get(cursor..cursor + 3) == Some(b"EOF") {
            cursor += 3;
            break;
        }
        let header = patch
            .get(cursor..cursor + 5)
            .ok_or_else(|| io::Error::new(ErrorKind::UnexpectedEof, "truncated IPS record"))?;
        let offset = ((header[0] as usize) << 16)
            | ((header[1] as usize) << 8)
            | header[2] as usize;
        let size = u16::from_be_bytes([header[3], header[4]]) as usize;
        cursor += 5;

        if size == 0 {
            let rle = patch.get(cursor..cursor + 3).ok_or_else(|| {
                io::Error::new(ErrorKind::UnexpectedEof, "truncated IPS RLE record")
            })?;
            let count = u16::from_be_bytes([rle[0], rle[1]]) as usize;
            let end = offset.checked_add(count).ok_or_else(|| {
                io::Error::new(ErrorKind::InvalidData, "IPS RLE range overflow")
            })?;
            if end > output.len() {
                return Err(io::Error::new(
                    ErrorKind::InvalidData,
                    "IPS RLE record exceeds target ROM length",
                ));
            }
            output[offset..end].fill(rle[2]);
            cursor += 3;
        } else {
            let end = offset.checked_add(size).ok_or_else(|| {
                io::Error::new(ErrorKind::InvalidData, "IPS record range overflow")
            })?;
            if end > output.len() {
                return Err(io::Error::new(
                    ErrorKind::InvalidData,
                    "IPS record exceeds target ROM length",
                ));
            }
            let bytes = patch.get(cursor..cursor + size).ok_or_else(|| {
                io::Error::new(ErrorKind::UnexpectedEof, "truncated IPS record data")
            })?;
            output[offset..end].copy_from_slice(bytes);
            cursor += size;
        }
    }

    // A three-byte truncation/expansion field may follow EOF in some IPS variants. Stable export
    // does not emit it; reject unexpected trailing bytes so verification cannot silently ignore data.
    if cursor != patch.len() {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            "unexpected trailing data after IPS EOF",
        ));
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ips_round_trip() {
        let source = vec![0, 1, 2, 3, 4, 5];
        let target = vec![0, 9, 8, 3, 7, 5];
        let patch = generate_ips_bytes(&source, &target).unwrap();
        assert_eq!(apply_ips(&source, &patch).unwrap(), target);
    }

    #[test]
    fn ips_rejects_expansion() {
        assert!(generate_ips_bytes(&[0, 1], &[0, 1, 2]).is_err());
    }

    #[test]
    fn ips_rejects_out_of_range_record() {
        let patch = b"PATCH\x00\x00\x03\x00\x01\xffEOF";
        assert!(apply_ips(&[0, 1, 2], patch).is_err());
    }
}
