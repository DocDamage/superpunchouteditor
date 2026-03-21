use rom_core::roster::{
    detect_roster_layout, RosterLayout, BOXER_NAME_POINTERS, INTRO_DATA_SIZE, INTRO_FIELD_SIZE,
};
use rom_core::{Rom, TextEncoder};

use crate::types::{
    ExpandedRosterLayout, ExpansionError, ExpansionResult, VANILLA_BOXER_COUNT, WriteRange,
};

const MAX_BOXER_COUNT: usize = 64;

pub fn expand_roster_tables(
    rom: &mut Rom,
    target_boxer_count: usize,
) -> ExpansionResult<(ExpandedRosterLayout, Vec<WriteRange>, Vec<String>)> {
    if !(VANILLA_BOXER_COUNT..=MAX_BOXER_COUNT).contains(&target_boxer_count) {
        return Err(ExpansionError::InvalidTargetCount {
            min: VANILLA_BOXER_COUNT,
            max: MAX_BOXER_COUNT,
            actual: target_boxer_count,
        });
    }

    let encoder = TextEncoder::new();
    let source_layout = detect_roster_layout(rom);
    let names = (0..target_boxer_count)
        .map(|idx| {
            let text = if idx < source_layout.boxer_count {
                read_boxer_name_from_layout(rom, &source_layout, idx).unwrap_or_else(|| default_boxer_name(idx))
            } else {
                format!("NEW BOXER {}", idx + 1)
            };
            encoder.encode_with_terminator(&text)
        })
        .collect::<Vec<Vec<u8>>>();

    let names_blob_size = names.iter().map(|entry| entry.len()).sum::<usize>();
    let name_pointer_table_size = target_boxer_count * 2;
    let name_long_pointer_table_size = target_boxer_count * 3;
    let circuit_table_size = target_boxer_count;
    let unlock_table_size = target_boxer_count;
    let intro_table_size = target_boxer_count * INTRO_DATA_SIZE;

    let total_size = align_up(name_pointer_table_size, 2)
        + align_up(name_long_pointer_table_size, 2)
        + align_up(names_blob_size, 2)
        + align_up(circuit_table_size, 2)
        + align_up(unlock_table_size, 2)
        + align_up(intro_table_size, 32);

    let allocation = rom
        .find_or_expand_free_space(total_size, 32)
        .ok_or(ExpansionError::FreeSpaceNotFound("expanded roster tables"))?;

    let mut cursor = align_up(allocation.offset, 32);
    let name_pointer_table_pc = cursor;
    cursor += align_up(name_pointer_table_size, 2);

    let name_long_pointer_table_pc = cursor;
    cursor += align_up(name_long_pointer_table_size, 2);

    let name_blob_pc = cursor;
    cursor += align_up(names_blob_size, 2);

    let circuit_table_pc = cursor;
    cursor += align_up(circuit_table_size, 2);

    let unlock_table_pc = cursor;
    cursor += align_up(unlock_table_size, 2);

    let intro_table_pc = cursor;

    let mut write_ranges = Vec::new();
    let mut notes = Vec::new();

    // Write name blob first and track pointer targets.
    let mut pointer_targets = Vec::with_capacity(target_boxer_count);
    let mut name_cursor = name_blob_pc;
    for encoded_name in &names {
        rom.write_bytes(name_cursor, encoded_name)
            .map_err(|err| ExpansionError::Rom(err.to_string()))?;
        pointer_targets.push(name_cursor);
        name_cursor += encoded_name.len();
    }
    write_ranges.push(WriteRange {
        start_pc: name_blob_pc,
        size: names_blob_size,
        description: "Expanded boxer name blob".to_string(),
    });

    // 2-byte legacy pointers: low/high address only.
    let mut legacy_ptrs = Vec::with_capacity(name_pointer_table_size);
    let mut long_ptrs = Vec::with_capacity(name_long_pointer_table_size);
    for pointer_pc in &pointer_targets {
        let (bank, addr) = rom.pc_to_snes(*pointer_pc);
        legacy_ptrs.push((addr & 0xFF) as u8);
        legacy_ptrs.push(((addr >> 8) & 0xFF) as u8);

        // Long pointer format used by expansion bootstrap: [bank, lo, hi].
        long_ptrs.push(bank);
        long_ptrs.push((addr & 0xFF) as u8);
        long_ptrs.push(((addr >> 8) & 0xFF) as u8);
    }
    rom.write_bytes(name_pointer_table_pc, &legacy_ptrs)
        .map_err(|err| ExpansionError::Rom(err.to_string()))?;
    rom.write_bytes(name_long_pointer_table_pc, &long_ptrs)
        .map_err(|err| ExpansionError::Rom(err.to_string()))?;
    write_ranges.push(WriteRange {
        start_pc: name_pointer_table_pc,
        size: legacy_ptrs.len(),
        description: "Expanded boxer name pointer table (legacy 16-bit)".to_string(),
    });
    write_ranges.push(WriteRange {
        start_pc: name_long_pointer_table_pc,
        size: long_ptrs.len(),
        description: "Expanded boxer name pointer table (long 24-bit)".to_string(),
    });

    // Circuit and unlock tables.
    let mut circuit_bytes = Vec::with_capacity(circuit_table_size);
    let mut unlock_bytes = Vec::with_capacity(unlock_table_size);
    for idx in 0..target_boxer_count {
        let circuit = if idx < source_layout.boxer_count {
            rom.read_bytes(source_layout.circuit_table_pc + idx, 1)
                .ok()
                .map(|bytes| bytes[0])
                .unwrap_or((idx / 4 % 4) as u8)
        } else {
            ((idx / 4) % 4) as u8
        };
        circuit_bytes.push(circuit);

        let unlock = if idx < source_layout.boxer_count {
            rom.read_bytes(source_layout.unlock_table_pc + idx, 1)
                .ok()
                .map(|bytes| bytes[0])
                .unwrap_or(idx as u8)
        } else {
            idx.min(255) as u8
        };
        unlock_bytes.push(unlock);
    }
    rom.write_bytes(circuit_table_pc, &circuit_bytes)
        .map_err(|err| ExpansionError::Rom(err.to_string()))?;
    rom.write_bytes(unlock_table_pc, &unlock_bytes)
        .map_err(|err| ExpansionError::Rom(err.to_string()))?;
    write_ranges.push(WriteRange {
        start_pc: circuit_table_pc,
        size: circuit_bytes.len(),
        description: "Expanded circuit table".to_string(),
    });
    write_ranges.push(WriteRange {
        start_pc: unlock_table_pc,
        size: unlock_bytes.len(),
        description: "Expanded unlock order table".to_string(),
    });

    // Intro table.
    let mut intro_bytes = Vec::with_capacity(intro_table_size);
    for idx in 0..target_boxer_count {
        if idx < source_layout.boxer_count {
            let original_start = source_layout.intro_table_pc + idx * INTRO_DATA_SIZE;
            let original = rom
                .read_bytes(original_start, INTRO_DATA_SIZE)
                .ok()
                .map(|slice| slice.to_vec())
                .unwrap_or_else(|| blank_intro_bytes(&encoder, idx).to_vec());
            intro_bytes.extend_from_slice(&original);
        } else {
            intro_bytes.extend_from_slice(&blank_intro_bytes(&encoder, idx));
        }
    }
    rom.write_bytes(intro_table_pc, &intro_bytes)
        .map_err(|err| ExpansionError::Rom(err.to_string()))?;
    write_ranges.push(WriteRange {
        start_pc: intro_table_pc,
        size: intro_bytes.len(),
        description: "Expanded boxer intro table".to_string(),
    });

    notes.push(
        "Expanded roster tables were written to free space. Gameplay code still needs hook-based redirection to consume these tables."
            .to_string(),
    );
    if source_layout.expanded {
        notes.push(format!(
            "Expansion source preserved {} existing expanded boxer entries from prior layout before growing to {}.",
            source_layout.boxer_count, target_boxer_count
        ));
    } else {
        notes.push("Expansion source used vanilla roster tables as baseline.".to_string());
    }
    notes.push(
        "Long pointer entries are serialized as [bank, lo, hi] and intended for the in-ROM editor runtime."
            .to_string(),
    );
    if allocation.offset < rom_core::EXPECTED_SIZE {
        notes.push(
            "Some expansion data landed inside original ROM space; verify collisions before shipping a production patch."
                .to_string(),
        );
    }

    Ok((
        ExpandedRosterLayout {
            boxer_count: target_boxer_count,
            name_pointer_table_pc,
            name_long_pointer_table_pc,
            name_blob_pc,
            circuit_table_pc,
            unlock_table_pc,
            intro_table_pc,
        },
        write_ranges,
        notes,
    ))
}

fn read_boxer_name_from_layout(rom: &Rom, layout: &RosterLayout, boxer_id: usize) -> Option<String> {
    let name_pc = if layout.expanded {
        let long_ptr_pc = layout.name_long_pointer_table_pc + (boxer_id * 3);
        let ptr = rom.read_bytes(long_ptr_pc, 3).ok()?;
        let bank = ptr[0];
        let addr = u16::from_le_bytes([ptr[1], ptr[2]]);
        if bank == 0 || addr < 0x8000 {
            let fallback_ptr_pc = layout.name_pointer_table_pc + (boxer_id * 2);
            let fallback_ptr = rom.read_bytes(fallback_ptr_pc, 2).ok()?;
            let fallback_addr = u16::from_le_bytes([fallback_ptr[0], fallback_ptr[1]]);
            let fallback_bank = rom.pc_to_snes(layout.name_blob_pc).0;
            rom.snes_to_pc(fallback_bank, fallback_addr)
        } else {
            rom.snes_to_pc(bank, addr)
        }
    } else {
        let pointer_pc = BOXER_NAME_POINTERS + (boxer_id * 2);
        let ptr = rom.read_bytes(pointer_pc, 2).ok()?;
        let snes_addr = u16::from_le_bytes([ptr[0], ptr[1]]);
        ((0x0Cusize & 0x7F) * 0x8000) | ((snes_addr as usize) & 0x7FFF)
    };

    let mut encoded = Vec::new();
    for idx in 0..64usize {
        let byte = rom.read_bytes(name_pc + idx, 1).ok()?.first().copied()?;
        if byte == 0xFF {
            break;
        }
        encoded.push(byte);
    }

    let decoded = TextEncoder::new().decode(&encoded);
    if decoded.is_empty() {
        None
    } else {
        Some(decoded)
    }
}

fn default_boxer_name(boxer_id: usize) -> String {
    const DEFAULTS: [&str; VANILLA_BOXER_COUNT] = [
        "GABBY JAY",
        "BEAR HUGGER",
        "PISTON HURRICANE",
        "BALD BULL",
        "BOB CHARLIE",
        "DRAGON CHAN",
        "MASKED MUSCLE",
        "MR. SANDMAN",
        "ARAN RYAN",
        "HEIKE KAGERO",
        "MAD CLOWN",
        "SUPER MACHO MAN",
        "NARCIS PRINCE",
        "HOY QUARLOW",
        "RICK BRUISER",
        "NICK BRUISER",
    ];
    DEFAULTS
        .get(boxer_id)
        .map(ToString::to_string)
        .unwrap_or_else(|| format!("BOXER {}", boxer_id + 1))
}

fn blank_intro_bytes(encoder: &TextEncoder, boxer_id: usize) -> [u8; INTRO_DATA_SIZE] {
    let mut bytes = [0x24u8; INTRO_DATA_SIZE];
    let fields = [
        default_boxer_name(boxer_id),
        "FROM: UNKNOWN".to_string(),
        "RECORD: 0-0".to_string(),
        format!("RANK: #{}", boxer_id + 1),
        "READY TO FIGHT!".to_string(),
    ];

    for (field_idx, value) in fields.iter().enumerate() {
        let encoded = encoder.encode_fixed(value, INTRO_FIELD_SIZE);
        let start = field_idx * INTRO_FIELD_SIZE;
        let end = start + INTRO_FIELD_SIZE;
        bytes[start..end].copy_from_slice(&encoded[..INTRO_FIELD_SIZE]);
    }

    bytes
}

fn align_up(value: usize, alignment: usize) -> usize {
    if alignment <= 1 {
        value
    } else {
        value.div_ceil(alignment) * alignment
    }
}
