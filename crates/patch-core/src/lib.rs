use std::fs::File;
use std::io::{Result, Write};

pub mod bps;
pub use bps::{generate_bps, generate_bps_to_file, BpsMetadata};

pub fn generate_ips(original: &[u8], edited: &[u8], output_path: &str) -> Result<()> {
    let mut file = File::create(output_path)?;
    file.write_all(b"PATCH")?;

    let mut i = 0;
    while i < original.len() && i < edited.len() {
        if original[i] != edited[i] {
            let start = i;
            // Find end of contiguous change (max 65535 bytes for IPS record)
            while i < original.len()
                && i < edited.len()
                && original[i] != edited[i]
                && (i - start) < 65535
            {
                i += 1;
            }
            let len = i - start;

            // Offset (3 bytes)
            file.write_all(&[(start >> 16) as u8, (start >> 8) as u8, start as u8])?;
            // Size (2 bytes)
            file.write_all(&[(len >> 8) as u8, len as u8])?;
            // Data
            file.write_all(&edited[start..i])?;
        } else {
            i += 1;
        }
    }

    file.write_all(b"EOF")?;
    Ok(())
}
