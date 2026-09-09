//! Lossless decoding of the game's AI instruction stream.
//!
//! Instruction lengths verified against Yoshifanatic1/Super-Punch-Out-Disassembly,
//! commit 3a1bd913e5ff6aefe7c7bcdb2c797919bcea7cba,
//! SPO/AsarScripts/DisassembleAIScript.asm. Unknown semantics stay unknown.

use serde::{Deserialize, Serialize};

/// Source-verified bytecode interval, not an inferred free-space allocation.
#[derive(Debug, Clone, Serialize)]
pub struct AiScriptRegion {
    pub id: &'static str,
    pub fighter: &'static str,
    pub pc_offset: usize,
    pub length: usize,
}

const fn region(
    id: &'static str,
    fighter: &'static str,
    start: usize,
    end: usize,
) -> AiScriptRegion {
    AiScriptRegion {
        id,
        fighter,
        pc_offset: ((start >> 16) & 0x7f) * 0x8000 + (start & 0x7fff),
        length: end - start,
    }
}

/// Routine_Macros_SPO.asm: header pointer +8 is the primary AI stream;
/// pointer +10 marks the following animation table. Each listed interval was
/// checked to contain only AI macros and internal labels, not padding/data.
/// These are USA source extents, not an allocation or ownership proof for hacks.
pub const VERIFIED_USA_AI_REGIONS: &[AiScriptRegion] = &[
    region("gabby_jay_primary", "Gabby Jay", 0x0980ae, 0x098166),
    region("bear_hugger_primary", "Bear Hugger", 0x09c494, 0x09c540),
    region(
        "piston_hurricane_primary",
        "Piston Hurricane",
        0x0a80a9,
        0x0a81a7,
    ),
    region("bald_bull_primary", "Bald Bull", 0x0ac639, 0x0aca4e),
    region("bob_charlie_primary", "Bob Charlie", 0x09ce21, 0x09cf0c),
    region("dragon_chan_primary", "Dragon Chan", 0x0b80ea, 0x0b821d),
    region("masked_muscle_primary", "Masked Muscle", 0x0bd09b, 0x0bd188),
    region("mr_sandman_primary", "Mr. Sandman", 0x0ad6c5, 0x0ad7db),
    region("aran_ryan_primary", "Aran Ryan", 0x0adf9e, 0x0ae09f),
    region("heike_kagero_primary", "Heike Kagero", 0x0bdcc3, 0x0bdddd),
    region("mad_clown_primary", "Mad Clown", 0x09d962, 0x09da84),
    region(
        "super_macho_man_primary",
        "Super Macho Man",
        0x0be86f,
        0x0be984,
    ),
    region("narcis_prince_primary", "Narcis Prince", 0x0c807c, 0x0c81c2),
    region("hoy_quarlow_primary", "Hoy Quarlow", 0x0d8065, 0x0d8149),
    region("rick_bruiser_primary", "Rick Bruiser", 0x0cdf67, 0x0ce040),
    region("nick_bruiser_primary", "Nick Bruiser", 0x0ccf9c, 0x0cd0f9),
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AiInstruction {
    pub offset: usize,
    pub opcode: u8,
    pub operands: Vec<u8>,
}

impl AiInstruction {
    /// Address operands identified as script targets by the upstream
    /// disassembler. Opcode $66's word is a memory address, not a script target.
    pub fn script_target(&self) -> Option<u16> {
        let index = match self.opcode {
            0x10 => 3,
            0x2e | 0x40 | 0x42 | 0x44 | 0x48 | 0x4a => 0,
            0x60 | 0x62 => 1,
            _ => return None,
        };
        let bytes = self.operands.get(index..index + 2)?;
        Some(u16::from_le_bytes([bytes[0], bytes[1]]))
    }
}

/// Resolve a same-bank LoROM script destination against known instruction
/// boundaries, never by merely checking whether the target byte is an opcode.
pub fn validate_script_target(
    rom: &[u8],
    source_pc: usize,
    target: u16,
    regions: &[AiScriptRegion],
) -> Result<(), String> {
    if target < 0x8000 {
        return Err("AI target must be in the ROM half of its bank".into());
    }
    let target_pc = (source_pc / 0x8000) * 0x8000 + usize::from(target & 0x7fff);
    let region = regions
        .iter()
        .find(|region| {
            target_pc >= region.pc_offset && target_pc - region.pc_offset < region.length
        })
        .ok_or("AI target is outside verified script regions")?;
    let end = region
        .pc_offset
        .checked_add(region.length)
        .ok_or("AI region overflow")?;
    let bytes = rom
        .get(region.pc_offset..end)
        .ok_or("AI target region is outside ROM")?;
    let instructions = decode_ai_stream(bytes)?;
    if !instructions
        .iter()
        .any(|instruction| instruction.offset == target_pc - region.pc_offset)
    {
        return Err("AI target must land on an instruction boundary".into());
    }
    Ok(())
}

fn operand_count(opcode: u8) -> Option<usize> {
    match opcode {
        0x00 | 0x02 | 0x04 | 0x06 | 0x0a => Some(0),
        0x10 => Some(5),
        0x12 => Some(8),
        0x20 | 0x22 | 0x24 | 0x26 | 0x28 | 0x2a | 0x2c | 0x30 | 0x32 | 0x34 | 0x36 | 0x38
        | 0x3a | 0x3c => Some(1),
        0x2e | 0x40 | 0x42 | 0x44 | 0x48 | 0x4a => Some(2),
        0x60 | 0x62 | 0x66 => Some(3),
        _ => None,
    }
}

/// Decode an explicitly bounded stream; do not guess its extent from a return,
/// since subroutines or branch targets can follow it in the same allocation.
pub fn decode_ai_stream(bytes: &[u8]) -> Result<Vec<AiInstruction>, String> {
    let mut instructions = Vec::new();
    let mut offset = 0;
    while offset < bytes.len() {
        let opcode = bytes[offset];
        let count = operand_count(opcode)
            .ok_or_else(|| format!("Invalid AI opcode {opcode:#04x} at {offset:#x}"))?;
        let end = offset + 1 + count;
        let operands = bytes
            .get(offset + 1..end)
            .ok_or_else(|| format!("Truncated AI instruction at {offset:#x}"))?;
        instructions.push(AiInstruction {
            offset,
            opcode,
            operands: operands.to_vec(),
        });
        offset = end;
    }
    Ok(instructions)
}

/// Encode without normalizing or dropping any operands. Offsets must describe a
/// contiguous stream, preventing accidental reuse after an instruction resize.
pub fn encode_ai_stream(instructions: &[AiInstruction]) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    for instruction in instructions {
        if instruction.offset != bytes.len() {
            return Err("AI instruction offsets are not contiguous".into());
        }
        if operand_count(instruction.opcode) != Some(instruction.operands.len()) {
            return Err(format!(
                "Invalid AI instruction at {:#x}",
                instruction.offset
            ));
        }
        bytes.push(instruction.opcode);
        bytes.extend_from_slice(&instruction.operands);
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn script_target_fields_do_not_confuse_memory_addresses_with_branches() {
        for (opcode, operands, target) in [
            (0x10, vec![0x34, 0x12, 0x01, 0xcd, 0xab], Some(0xabcd)),
            (0x44, vec![0xcd, 0xab], Some(0xabcd)),
            (0x60, vec![0x01, 0xcd, 0xab], Some(0xabcd)),
            (0x66, vec![0x01, 0xcd, 0xab], None),
            (0x2a, vec![20], None),
        ] {
            assert_eq!(
                AiInstruction {
                    offset: 0,
                    opcode,
                    operands
                }
                .script_target(),
                target
            );
        }
    }

    #[test]
    fn target_validation_uses_boundaries_not_opcode_looking_operand_bytes() {
        let mut rom = vec![0; 32];
        rom[8..14].copy_from_slice(&[0x2a, 0x00, 0x44, 0x08, 0x80, 0x00]);
        let regions = [AiScriptRegion {
            id: "test",
            fighter: "test",
            pc_offset: 8,
            length: 6,
        }];
        for target in [0x8008, 0x800a, 0x800d] {
            validate_script_target(&rom, 8, target, &regions).unwrap();
        }
        for target in [0x0008, 0x8009, 0x800e, 0x9000] {
            assert!(validate_script_target(&rom, 8, target, &regions).is_err());
        }
        assert!(validate_script_target(&rom, 0x8000, 0x8008, &regions).is_err());
        assert!(validate_script_target(&rom[..10], 8, 0x8008, &regions).is_err());
    }

    #[test]
    fn catalog_covers_roster_without_overlapping_regions() {
        assert_eq!(
            VERIFIED_USA_AI_REGIONS.len(),
            super::super::constants::MAX_FIGHTERS
        );
        for (i, region) in VERIFIED_USA_AI_REGIONS.iter().enumerate() {
            assert_eq!(region.fighter, super::super::constants::FIGHTER_NAMES[i]);
            assert!(region.length > 0);
            assert_eq!(
                region.pc_offset / 0x8000,
                (region.pc_offset + region.length - 1) / 0x8000
            );
            for other in &VERIFIED_USA_AI_REGIONS[i + 1..] {
                assert_ne!(region.id, other.id);
                assert!(
                    region.pc_offset + region.length <= other.pc_offset
                        || other.pc_offset + other.length <= region.pc_offset
                );
            }
        }
    }

    #[test]
    #[ignore = "requires a user-supplied headerless USA ROM via SPO_USA_ROM"]
    fn usa_reference_stream_round_trip() {
        let path = std::env::var("SPO_USA_ROM").expect("SPO_USA_ROM is required");
        let rom = std::fs::read(path).unwrap();
        assert_eq!(rom.len(), 0x200000);
        // Upstream disassembler's example: SNES $0C:DF67, 217 bytes.
        let bytes = &rom[0x65f67..0x65f67 + 217];
        let instructions = decode_ai_stream(bytes).unwrap();
        assert!(!instructions.is_empty());
        assert_eq!(encode_ai_stream(&instructions).unwrap(), bytes);
        for region in VERIFIED_USA_AI_REGIONS {
            let bytes = &rom[region.pc_offset..region.pc_offset + region.length];
            let decoded =
                decode_ai_stream(bytes).unwrap_or_else(|error| panic!("{}: {error}", region.id));
            assert_eq!(encode_ai_stream(&decoded).unwrap(), bytes, "{}", region.id);
            assert!(!decoded.is_empty(), "{}", region.id);
        }
    }

    #[test]
    fn every_supported_opcode_round_trips_all_operand_values() {
        for opcode in 0..=u8::MAX {
            if let Some(count) = operand_count(opcode) {
                for value in 0..=u8::MAX {
                    let mut bytes = vec![opcode];
                    bytes.extend(vec![value; count]);
                    assert_eq!(
                        encode_ai_stream(&decode_ai_stream(&bytes).unwrap()).unwrap(),
                        bytes
                    );
                }
            } else {
                assert!(decode_ai_stream(&[opcode]).is_err());
            }
        }
    }

    #[test]
    fn retains_subroutine_after_return_and_rejects_truncation() {
        let bytes = [0x2a, 30, 0x44, 0x00, 0xe0, 0x00, 0x2a, 10, 0x00];
        let decoded = decode_ai_stream(&bytes).unwrap();
        assert_eq!(decoded.len(), 5);
        assert_eq!(encode_ai_stream(&decoded).unwrap(), bytes);
        assert!(decode_ai_stream(&[0x44, 0x00]).is_err());
        let mut invalid = decoded;
        invalid[1].offset += 1;
        assert!(encode_ai_stream(&invalid).is_err());
    }
}
