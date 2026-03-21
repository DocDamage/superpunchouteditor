use crate::Rom;

use super::constants::{
    BOXER_COUNT, BOXER_INTRO_TABLE, BOXER_NAME_POINTERS, BOXER_NAME_TABLE, CIRCUIT_TABLE, INTRO_DATA_SIZE,
    UNLOCK_ORDER_TABLE,
};

const EDITOR_HEADER_MAGIC: [u8; 8] = *b"SPOEDITR";
const MIN_HEADER_SIZE: usize = 46;
const MAX_BOXER_COUNT: usize = 64;

#[derive(Debug, Clone, Copy)]
pub struct RosterLayout {
    pub boxer_count: usize,
    pub name_pointer_table_pc: usize,
    pub name_long_pointer_table_pc: usize,
    pub name_blob_pc: usize,
    pub circuit_table_pc: usize,
    pub unlock_table_pc: usize,
    pub intro_table_pc: usize,
    pub expanded: bool,
}

impl RosterLayout {
    pub fn vanilla() -> Self {
        Self {
            boxer_count: BOXER_COUNT,
            name_pointer_table_pc: BOXER_NAME_POINTERS,
            name_long_pointer_table_pc: 0,
            name_blob_pc: BOXER_NAME_TABLE,
            circuit_table_pc: CIRCUIT_TABLE,
            unlock_table_pc: UNLOCK_ORDER_TABLE,
            intro_table_pc: BOXER_INTRO_TABLE,
            expanded: false,
        }
    }
}

pub fn detect_roster_layout(rom: &Rom) -> RosterLayout {
    find_latest_expanded_layout(rom).unwrap_or_else(RosterLayout::vanilla)
}

fn find_latest_expanded_layout(rom: &Rom) -> Option<RosterLayout> {
    if rom.size() < MIN_HEADER_SIZE {
        return None;
    }

    let data = &rom.data;
    for start in (0..=data.len().saturating_sub(EDITOR_HEADER_MAGIC.len())).rev() {
        if data[start..start + EDITOR_HEADER_MAGIC.len()] != EDITOR_HEADER_MAGIC {
            continue;
        }

        let candidate = parse_header_at(rom, start)?;
        return Some(candidate);
    }

    None
}

fn parse_header_at(rom: &Rom, start: usize) -> Option<RosterLayout> {
    let bytes = rom.read_bytes(start, MIN_HEADER_SIZE).ok()?;
    let version = bytes[8];
    if version == 0 {
        return None;
    }

    let boxer_count = u16::from_le_bytes([bytes[10], bytes[11]]) as usize;
    if !(BOXER_COUNT..=MAX_BOXER_COUNT).contains(&boxer_count) {
        return None;
    }

    let name_pointer_table_pc = read_u32(bytes, 12)?;
    let name_long_pointer_table_pc = read_u32(bytes, 16)?;
    let name_blob_pc = read_u32(bytes, 20)?;
    let circuit_table_pc = read_u32(bytes, 24)?;
    let unlock_table_pc = read_u32(bytes, 28)?;
    let intro_table_pc = read_u32(bytes, 32)?;

    let rom_size = rom.size();
    if !span_in_bounds(name_pointer_table_pc, boxer_count * 2, rom_size)
        || !span_in_bounds(name_long_pointer_table_pc, boxer_count * 3, rom_size)
        || !span_in_bounds(name_blob_pc, 1, rom_size)
        || !span_in_bounds(circuit_table_pc, boxer_count, rom_size)
        || !span_in_bounds(unlock_table_pc, boxer_count, rom_size)
        || !span_in_bounds(intro_table_pc, boxer_count * INTRO_DATA_SIZE, rom_size)
    {
        return None;
    }

    Some(RosterLayout {
        boxer_count,
        name_pointer_table_pc,
        name_long_pointer_table_pc,
        name_blob_pc,
        circuit_table_pc,
        unlock_table_pc,
        intro_table_pc,
        expanded: true,
    })
}

fn read_u32(bytes: &[u8], start: usize) -> Option<usize> {
    let end = start.checked_add(4)?;
    let slice = bytes.get(start..end)?;
    Some(u32::from_le_bytes([slice[0], slice[1], slice[2], slice[3]]) as usize)
}

fn span_in_bounds(start: usize, size: usize, rom_size: usize) -> bool {
    start < rom_size && start.checked_add(size).is_some_and(|end| end <= rom_size)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_roster_layout_defaults_to_vanilla_when_no_header() {
        let rom = Rom::new(vec![0; 0x20_0000]);
        let layout = detect_roster_layout(&rom);
        assert!(!layout.expanded);
        assert_eq!(layout.boxer_count, BOXER_COUNT);
        assert_eq!(layout.name_pointer_table_pc, BOXER_NAME_POINTERS);
    }

    #[test]
    fn detect_roster_layout_reads_expanded_header() {
        let mut rom = Rom::new(vec![0; 0x20_0000]);
        let header_pc = 0x1F0000usize;
        let boxer_count = 24usize;
        let name_ptr = 0x1F0100usize;
        let long_ptr = 0x1F0140usize;
        let name_blob = 0x1F0200usize;
        let circuit = 0x1F0600usize;
        let unlock = 0x1F0620usize;
        let intro = 0x1F0640usize;

        let mut header = vec![0u8; MIN_HEADER_SIZE];
        header[..8].copy_from_slice(&EDITOR_HEADER_MAGIC);
        header[8] = 2;
        header[10..12].copy_from_slice(&(boxer_count as u16).to_le_bytes());
        header[12..16].copy_from_slice(&(name_ptr as u32).to_le_bytes());
        header[16..20].copy_from_slice(&(long_ptr as u32).to_le_bytes());
        header[20..24].copy_from_slice(&(name_blob as u32).to_le_bytes());
        header[24..28].copy_from_slice(&(circuit as u32).to_le_bytes());
        header[28..32].copy_from_slice(&(unlock as u32).to_le_bytes());
        header[32..36].copy_from_slice(&(intro as u32).to_le_bytes());
        rom.write_bytes(header_pc, &header).expect("write expansion header");

        let layout = detect_roster_layout(&rom);
        assert!(layout.expanded);
        assert_eq!(layout.boxer_count, boxer_count);
        assert_eq!(layout.name_pointer_table_pc, name_ptr);
        assert_eq!(layout.name_long_pointer_table_pc, long_ptr);
        assert_eq!(layout.name_blob_pc, name_blob);
        assert_eq!(layout.circuit_table_pc, circuit);
        assert_eq!(layout.unlock_table_pc, unlock);
        assert_eq!(layout.intro_table_pc, intro);
    }
}
