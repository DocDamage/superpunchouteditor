//! Animation instruction boundaries from Yoshifanatic1/Super-Punch-Out-Disassembly
//! commit 3a1bd913e5ff6aefe7c7bcdb2c797919bcea7cba,
//! SPO/AsarScripts/DisassembleAnimationScript.asm. Unknown semantics are preserved.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct AnimationRegion {
    pub id: &'static str,
    pub label: &'static str,
    pub pc_offset: usize,
    pub length: usize,
}

const fn region(
    id: &'static str,
    label: &'static str,
    start: usize,
    end: usize,
) -> AnimationRegion {
    AnimationRegion {
        id,
        label,
        pc_offset: ((start >> 16) & 0x7f) * 0x8000 + (start & 0x7fff),
        length: end - start,
    }
}

/// Source-bounded instruction intervals, not inferred spans between arbitrary
/// table entries. Animation tables can themselves point to nested pointer tables.
pub const VERIFIED_USA_ANIMATION_REGIONS: &[AnimationRegion] = &[
    region(
        "bear_09c620",
        "Bear Hugger block $09:C620",
        0x09c620,
        0x09c63b,
    ),
    region(
        "bob_09d022",
        "Bob Charlie block $09:D022",
        0x09d022,
        0x09d03d,
    ),
    region("mad_09db97", "Mad Clown block $09:DB97", 0x09db97, 0x09dbb0),
    region(
        "piston_0a82a8",
        "Piston Hurricane block $0A:82A8",
        0x0a82a8,
        0x0a82b0,
    ),
    region(
        "bald_0acb4b",
        "Bald Bull block $0A:CB4B",
        0x0acb4b,
        0x0acb66,
    ),
    region(
        "sandman_0ad8bb",
        "Mr. Sandman block $0A:D8BB",
        0x0ad8bb,
        0x0ad8d5,
    ),
    region(
        "aran_0ae17b",
        "Aran Ryan block $0A:E17B",
        0x0ae17b,
        0x0ae195,
    ),
    region(
        "dragon_0b8308",
        "Dragon Chan block $0B:8308",
        0x0b8308,
        0x0b8328,
    ),
    region(
        "masked_0bd25f",
        "Masked Muscle block $0B:D25F",
        0x0bd25f,
        0x0bd27a,
    ),
    region(
        "heike_0bdea6",
        "Heike Kagero block $0B:DEA6",
        0x0bdea6,
        0x0bdf0b,
    ),
    region(
        "macho_0bea9e",
        "Super Macho Man block $0B:EA9E",
        0x0bea9e,
        0x0beab8,
    ),
    region(
        "narcis_0c832f",
        "Narcis Prince block $0C:832F",
        0x0c832f,
        0x0c8337,
    ),
    region(
        "nick_0cd320",
        "Nick Bruiser block $0C:D320",
        0x0cd320,
        0x0cd33a,
    ),
    region(
        "rick_0ce19f",
        "Rick Bruiser block $0C:E19F",
        0x0ce19f,
        0x0ce1b6,
    ),
    region(
        "hoy_0d8249",
        "Hoy Quarlow block $0D:8249",
        0x0d8249,
        0x0d8270,
    ),
    AnimationRegion {
        id: "gabby_jay_098267",
        label: "Gabby Jay sequence $09:8267",
        pc_offset: 0x48267,
        length: 0x98287 - 0x98267,
    },
    AnimationRegion {
        id: "reference_0d8ebe",
        label: "Reference sequence $0D:8EBE",
        pc_offset: 0x68ebe,
        length: 133,
    },
];

// Indexed by even opcode / 2. Opcode zero dispatches to a second opcode table.
const LENGTHS: [usize; 70] = [
    1, 1, 2, 1, 2, 1, 0, 1, 2, 2, 3, 3, 0, 2, 1, 1, 2, 0, 0, 2, 4, 1, 0, 2, 0, 1, 0, 0, 0, 0, 2, 1,
    0, 1, 1, 1, 2, 2, 2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 3, 5, 5, 1, 0,
    1, 0, 1, 0, 4, 1,
];
const SUB_LENGTHS: [usize; 45] = [
    0, 0, 0, 0, 0, 0, 3, 4, 2, 0, 2, 1, 12, 0, 0, 2, 13, 5, 0, 2, 0, 0, 0, 0, 2, 5, 8, 0, 0, 16, 1,
    5, 0, 1, 2, 1, 0, 2, 1, 0, 2, 2, 3, 3, 10,
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnimationInstruction {
    pub offset: usize,
    pub opcode: u8,
    /// Includes the secondary opcode for opcode zero.
    pub operands: Vec<u8>,
}

impl AnimationInstruction {
    /// Semantics verified in runtime handlers CODE_018B04..CODE_018B31.
    /// Delay remains a raw counter byte; see uninterrupted_delay_updates.
    pub fn timing_operation(&self) -> Option<TimingOperation> {
        match (self.opcode, self.operands.as_slice()) {
            (0x10, [x, y]) => Some(TimingOperation::MoveBoth {
                x: *x as i8,
                y: *y as i8,
            }),
            (0x44, [packed]) => {
                let step = |n: u8| if n < 8 { n as i8 + 1 } else { (n | 0xf0) as i8 };
                Some(TimingOperation::MoveBoth {
                    x: step(packed >> 4),
                    y: step(packed & 0x0f),
                })
            }
            (0x1e, [delta]) => Some(TimingOperation::MoveHorizontal {
                delta: *delta as i8,
            }),
            (0x1c, [delta]) => Some(TimingOperation::MoveVertical {
                delta: *delta as i8,
            }),
            (0x28, [a, b, c, d]) => Some(TimingOperation::SetMirroredState {
                values: [*a, *b, *c, *d],
            }),
            (0x36, []) => Some(TimingOperation::SetState76),
            (0x80, [value]) => Some(TimingOperation::Request0324 { value: *value }),
            (0x18, []) => Some(TimingOperation::ResetPosition),
            (0x1a, [x, y]) => Some(TimingOperation::Reposition {
                x: *x as i8,
                y: *y as i8,
            }),
            (0x22, []) => Some(TimingOperation::Face {
                request: FacingRequest::Left,
            }),
            (0x24, []) => Some(TimingOperation::Face {
                request: FacingRequest::Right,
            }),
            (0x02, [counter]) => Some(TimingOperation::Delay { counter: *counter }),
            (0x04, [low, high]) => Some(TimingOperation::Jump {
                target: u16::from_le_bytes([*low, *high]),
            }),
            (0x06, [counter]) => Some(TimingOperation::SetLoopCounter { counter: *counter }),
            (0x08, [low, high]) => Some(TimingOperation::DecrementLoopAndBranch {
                target: u16::from_le_bytes([*low, *high]),
            }),
            (0x20, [pose, counter]) => Some(TimingOperation::DisplayPose {
                pose: *pose,
                counter: *counter,
            }),
            _ => None,
        }
    }

    /// CODE_018991 decrements the byte before testing zero. This counts delay
    /// updates, not wall-clock frames: sequence replacement can interrupt it.
    pub fn uninterrupted_delay_updates(&self) -> Option<u16> {
        let counter = match self.timing_operation()? {
            TimingOperation::Delay { counter } | TimingOperation::DisplayPose { counter, .. } => {
                counter
            }
            _ => return None,
        };
        Some(if counter == 0 {
            256
        } else {
            u16::from(counter)
        })
    }

    /// Same-bank address operands marked as script references by the pinned
    /// disassembler. This identifies possible destinations, not branch conditions
    /// or proof that every destination is executed.
    pub fn script_targets(&self) -> Result<Vec<u16>, String> {
        let offsets: &[usize] = match self.opcode {
            0x04 | 0x08 => &[0],
            0x7a => &[3],
            0x00 => match self.operands.first().copied() {
                Some(0x0e) => &[1, 3],
                Some(0x14 | 0x1e | 0x26 | 0x30 | 0x44) => &[1],
                Some(0x18) => &[1, 3, 5, 7, 9, 11],
                Some(0x20) => &[2, 6, 10, 12],
                Some(0x22 | 0x32) => &[4],
                Some(0x3a) => &[1, 3, 5, 7, 9, 11, 13, 15],
                None => return Err("Missing animation subopcode".into()),
                _ => &[],
            },
            _ => &[],
        };
        offsets
            .iter()
            .map(|&offset| {
                let pair = self
                    .operands
                    .get(offset..offset + 2)
                    .ok_or("Truncated animation script target")?;
                Ok(u16::from_le_bytes([pair[0], pair[1]]))
            })
            .collect()
    }

    pub fn displayed_pose(&self) -> Option<(u8, u8)> {
        if self.opcode == 0x20 && self.operands.len() == 2 {
            Some((self.operands[0], self.operands[1]))
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "operation", rename_all = "snake_case")]
pub enum TimingOperation {
    MoveBoth { x: i8, y: i8 },
    MoveHorizontal { delta: i8 },
    MoveVertical { delta: i8 },
    SetMirroredState { values: [u8; 4] },
    SetState76,
    Request0324 { value: u8 },
    ResetPosition,
    Reposition { x: i8, y: i8 },
    Face { request: FacingRequest },
    Delay { counter: u8 },
    Jump { target: u16 },
    SetLoopCounter { counter: u8 },
    DecrementLoopAndBranch { target: u16 },
    DisplayPose { pose: u8, counter: u8 },
}

/// Opcode intent only. CODE_018D6D/CODE_018D7B invert the effective direction
/// when runtime byte $0E is nonzero; a static trace does not know that byte.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FacingRequest {
    Left,
    Right,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TargetResolution {
    Instruction,
    InsideOperand,
    OutsideInterval,
    InvalidAddress,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TimedPose {
    pub known_state: Vec<KnownStateByte>,
    pub position_request: Option<PositionRequest>,
    pub facing_request: Option<FacingRequest>,
    pub instruction_offset: usize,
    pub pose: Option<u8>,
    pub updates: u16,
}

/// Only bits established by traced instructions are known. Direct-page offsets
/// are not absolute WRAM addresses; the CPU's D register supplies their base.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct KnownStateByte {
    pub direct_page: bool,
    pub address: u16,
    pub value: u8,
    pub known_mask: u8,
}

/// CODE_018D1C sets an origin-relative position, not an accumulated delta.
/// Horizontal choice depends on runtime $0E XOR $8C, unavailable to static traces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct PositionRequest {
    pub x_unmirrored: i16,
    pub x_mirrored: i16,
    pub y: i16,
}

/// CODE_018C09/CODE_018C52 and CODE_018C2C: sign-extend the movement,
/// optionally negate it, add with 16-bit wrapping and clamp to the ring bounds.
/// Zero movement bypasses clamping. The caller must supply the runtime direction.
pub fn move_horizontal_position(position: i16, delta: i8, mirrored: bool) -> i16 {
    let delta = if mirrored {
        -i16::from(delta)
    } else {
        i16::from(delta)
    };
    if delta == 0 {
        return position;
    }
    position.wrapping_add(delta).clamp(72, 184)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "reason", rename_all = "snake_case")]
pub enum TraceStop {
    UnknownMirroring { offset: usize },
    UninitializedPosition { offset: usize },
    IntervalEnd,
    UnknownInstruction { offset: usize },
    UninitializedLoop { offset: usize },
    UnverifiedTarget { address: u16 },
    StepLimit,
}

#[derive(Debug, Clone, Serialize)]
pub struct TimingTrace {
    pub events: Vec<TimedPose>,
    pub stop: TraceStop,
}

/// Partial interpreter for proven timing opcodes, never a guessed full VM.
/// A block may depend on state established before entry; unknown state stops it.
pub fn trace_animation_timing(
    bytes: &[u8],
    pc_offset: usize,
    step_limit: usize,
) -> Result<TimingTrace, String> {
    trace_animation_with_mirroring(bytes, pc_offset, step_limit, None)
}

/// Runtime direct-page bytes $0E and $8C. Values are not assumed to be boolean:
/// origin positioning uses XOR, while relative horizontal motion tests $0E alone.
#[derive(Debug, Clone, Copy)]
pub struct RuntimeMirroring {
    pub dp_0e: u8,
    pub dp_8c: u8,
}

pub fn trace_animation_with_mirroring(
    bytes: &[u8],
    pc_offset: usize,
    step_limit: usize,
    mirroring: Option<RuntimeMirroring>,
) -> Result<TimingTrace, String> {
    if !(1..=100_000).contains(&step_limit) {
        return Err("Timing step limit must be 1..100000".into());
    }
    analyze_animation_references(bytes, pc_offset)?;
    let instructions = decode_animation_stream(bytes)?;
    let indices: std::collections::BTreeMap<_, _> = instructions
        .iter()
        .enumerate()
        .map(|(i, instruction)| (instruction.offset, i))
        .collect();
    let mut cursor = 0;
    let mut loop_counter: Option<u8> = None;
    let mut pose = None;
    let mut facing_request = None;
    let mut position_request: Option<PositionRequest> = None;
    let mut known_state = std::collections::BTreeMap::new();
    let mut events = Vec::new();
    for _ in 0..step_limit {
        let Some(instruction) = instructions.get(cursor) else {
            return Ok(TimingTrace {
                events,
                stop: TraceStop::IntervalEnd,
            });
        };
        let mut target = None;
        match instruction.timing_operation() {
            Some(
                operation @ (TimingOperation::MoveHorizontal { .. }
                | TimingOperation::MoveBoth { .. }),
            ) => {
                let (delta, vertical) = match operation {
                    TimingOperation::MoveHorizontal { delta } => (delta, None),
                    TimingOperation::MoveBoth { x, y } => (x, Some(y)),
                    _ => unreachable!(),
                };
                let Some(flags) = mirroring else {
                    return Ok(TimingTrace {
                        events,
                        stop: TraceStop::UnknownMirroring {
                            offset: instruction.offset,
                        },
                    });
                };
                let Some(position) = position_request.as_mut() else {
                    return Ok(TimingTrace {
                        events,
                        stop: TraceStop::UninitializedPosition {
                            offset: instruction.offset,
                        },
                    });
                };
                let x = move_horizontal_position(position.x_unmirrored, delta, flags.dp_0e != 0);
                position.x_unmirrored = x;
                position.x_mirrored = x;
                if let Some(delta_y) = vertical {
                    position.y = position.y.wrapping_add(i16::from(delta_y));
                    known_state.insert(
                        (true, 0x0d),
                        KnownStateByte {
                            direct_page: true,
                            address: 0x0d,
                            value: delta_y as u8,
                            known_mask: 0xff,
                        },
                    );
                }
                known_state.insert(
                    (true, 0x0c),
                    KnownStateByte {
                        direct_page: true,
                        address: 0x0c,
                        value: delta as u8,
                        known_mask: 0xff,
                    },
                );
            }
            Some(TimingOperation::MoveVertical { delta }) => {
                // CODE_018B93/CODE_018C71 sign-extend DP $0D and add to $8A.
                let Some(position) = position_request.as_mut() else {
                    return Ok(TimingTrace {
                        events,
                        stop: TraceStop::UninitializedPosition {
                            offset: instruction.offset,
                        },
                    });
                };
                position.y = position.y.wrapping_add(i16::from(delta));
                known_state.insert(
                    (true, 0x0d),
                    KnownStateByte {
                        direct_page: true,
                        address: 0x0d,
                        value: delta as u8,
                        known_mask: 0xff,
                    },
                );
            }
            Some(TimingOperation::SetMirroredState { values }) => {
                // CODE_018DFF swaps each pair when DP $0E is zero. Without that
                // flag, retain only bits equal in both alternatives, not a guess.
                for (index, value) in values.iter().enumerate() {
                    let other = values[index ^ 1];
                    let (value, known_mask) = match mirroring {
                        Some(flags) => (if flags.dp_0e == 0 { other } else { *value }, 0xff),
                        None => (*value & !(value ^ other), !(value ^ other)),
                    };
                    let address = 0x68 + index as u16;
                    known_state.insert(
                        (true, address),
                        KnownStateByte {
                            direct_page: true,
                            address,
                            value,
                            known_mask,
                        },
                    );
                }
            }
            Some(TimingOperation::SetState76) => {
                known_state.insert(
                    (true, 0x76),
                    KnownStateByte {
                        direct_page: true,
                        address: 0x76,
                        value: 0x81,
                        known_mask: 0xff,
                    },
                );
            }
            Some(TimingOperation::Request0324 { value }) => {
                known_state.insert(
                    (false, 0x324),
                    KnownStateByte {
                        direct_page: false,
                        address: 0x324,
                        value,
                        known_mask: 0xff,
                    },
                );
                let state = known_state.entry((false, 0x326)).or_insert(KnownStateByte {
                    direct_page: false,
                    address: 0x326,
                    value: 0,
                    known_mask: 0,
                });
                state.value |= 0x20;
                state.known_mask |= 0x20;
            }
            Some(TimingOperation::ResetPosition) => {
                position_request = Some(PositionRequest {
                    x_unmirrored: 128,
                    x_mirrored: 128,
                    y: 136,
                })
            }
            Some(TimingOperation::Reposition { x, y }) => {
                let unmirrored = 128 + i16::from(x);
                let mirrored = 128 - i16::from(x);
                let resolved = mirroring.map(|flags| {
                    if flags.dp_0e ^ flags.dp_8c != 0 {
                        mirrored
                    } else {
                        unmirrored
                    }
                });
                position_request = Some(PositionRequest {
                    x_unmirrored: resolved.unwrap_or(unmirrored),
                    x_mirrored: resolved.unwrap_or(mirrored),
                    y: 136 + i16::from(y),
                })
            }
            Some(TimingOperation::Face { request }) => facing_request = Some(request),
            Some(TimingOperation::DisplayPose { pose: next, .. }) => {
                pose = Some(next);
                events.push(TimedPose {
                    known_state: known_state.values().cloned().collect(),
                    position_request,
                    facing_request,
                    instruction_offset: instruction.offset,
                    pose,
                    updates: instruction.uninterrupted_delay_updates().unwrap(),
                });
            }
            Some(TimingOperation::Delay { .. }) => events.push(TimedPose {
                known_state: known_state.values().cloned().collect(),
                position_request,
                facing_request,
                instruction_offset: instruction.offset,
                pose,
                updates: instruction.uninterrupted_delay_updates().unwrap(),
            }),
            Some(TimingOperation::SetLoopCounter { counter }) => loop_counter = Some(counter),
            Some(TimingOperation::Jump { target: address }) => target = Some(address),
            Some(TimingOperation::DecrementLoopAndBranch { target: address }) => {
                let Some(counter) = loop_counter else {
                    return Ok(TimingTrace {
                        events,
                        stop: TraceStop::UninitializedLoop {
                            offset: instruction.offset,
                        },
                    });
                };
                let next = counter.wrapping_sub(1);
                loop_counter = Some(next);
                if next != 0 {
                    target = Some(address);
                }
            }
            None => {
                return Ok(TimingTrace {
                    events,
                    stop: TraceStop::UnknownInstruction {
                        offset: instruction.offset,
                    },
                })
            }
        }
        if let Some(address) = target {
            let target_pc = (pc_offset & !0x7fff) + (address as usize & 0x7fff);
            let index = if address >= 0x8000 {
                target_pc
                    .checked_sub(pc_offset)
                    .and_then(|offset| indices.get(&offset).copied())
            } else {
                None
            };
            let Some(index) = index else {
                return Ok(TimingTrace {
                    events,
                    stop: TraceStop::UnverifiedTarget { address },
                });
            };
            cursor = index;
        } else {
            cursor += 1;
        }
    }
    Ok(TimingTrace {
        events,
        stop: if cursor == instructions.len() {
            TraceStop::IntervalEnd
        } else {
            TraceStop::StepLimit
        },
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AnimationReference {
    pub instruction_offset: usize,
    pub target_address: u16,
    pub target_pc: Option<usize>,
    pub resolution: TargetResolution,
}

#[derive(Debug, Clone, Serialize)]
pub struct CatalogReference {
    pub source_region: String,
    pub reference: AnimationReference,
    pub target_region: Option<String>,
}

/// Resolve references across supplied proven intervals. This is a reference graph,
/// not a playback trace: conditional branches and fallthrough are not inferred.
pub fn analyze_animation_catalog(
    rom: &[u8],
    regions: &[AnimationRegion],
) -> Result<Vec<CatalogReference>, String> {
    let mut decoded = Vec::new();
    let mut ids = std::collections::BTreeSet::new();
    let mut ranges = Vec::new();
    for region in regions {
        if !ids.insert(region.id) {
            return Err("Duplicate animation region ID".into());
        }
        let end = region
            .pc_offset
            .checked_add(region.length)
            .ok_or("Animation interval overflow")?;
        if ranges
            .iter()
            .any(|&(start, previous_end)| region.pc_offset < previous_end && start < end)
        {
            return Err("Overlapping animation catalog intervals".into());
        }
        let bytes = rom
            .get(region.pc_offset..end)
            .ok_or("Animation catalog interval outside ROM")?;
        let references = analyze_animation_references(bytes, region.pc_offset)?;
        let boundaries: std::collections::BTreeSet<_> = decode_animation_stream(bytes)?
            .iter()
            .map(|instruction| region.pc_offset + instruction.offset)
            .collect();
        ranges.push((region.pc_offset, end));
        decoded.push((region, boundaries, references));
    }
    let mut result = Vec::new();
    for (source, _, references) in &decoded {
        for original in references {
            let mut reference = original.clone();
            let target = reference.target_pc.and_then(|pc| {
                decoded.iter().find(|(region, _, _)| {
                    pc >= region.pc_offset && pc < region.pc_offset + region.length
                })
            });
            if let Some((_, boundaries, _)) = target {
                reference.resolution = if boundaries.contains(&reference.target_pc.unwrap()) {
                    TargetResolution::Instruction
                } else {
                    TargetResolution::InsideOperand
                };
            }
            result.push(CatalogReference {
                source_region: source.id.into(),
                reference,
                target_region: target.map(|(region, _, _)| region.id.into()),
            });
        }
    }
    Ok(result)
}

/// Inspect references within a proven bytecode interval. External targets remain
/// unresolved: arbitrary data must never be promoted to executable instructions.
pub fn analyze_animation_references(
    bytes: &[u8],
    pc_offset: usize,
) -> Result<Vec<AnimationReference>, String> {
    let end = pc_offset
        .checked_add(bytes.len())
        .ok_or("Animation interval overflow")?;
    if bytes.is_empty() || end > 0x400000 || pc_offset / 0x8000 != (end - 1) / 0x8000 {
        return Err("Expected a nonempty ordinary LoROM interval within one bank".into());
    }
    let instructions = decode_animation_stream(bytes)?;
    let boundaries: std::collections::BTreeSet<_> = instructions.iter().map(|i| i.offset).collect();
    let bank_pc = pc_offset & !0x7fff;
    let mut references = Vec::new();
    for instruction in instructions {
        for address in instruction.script_targets()? {
            let target_pc = (address >= 0x8000).then_some(bank_pc + (address as usize & 0x7fff));
            let resolution = match target_pc {
                None => TargetResolution::InvalidAddress,
                Some(pc) if pc < pc_offset || pc >= end => TargetResolution::OutsideInterval,
                Some(pc) if boundaries.contains(&(pc - pc_offset)) => TargetResolution::Instruction,
                Some(_) => TargetResolution::InsideOperand,
            };
            references.push(AnimationReference {
                instruction_offset: instruction.offset,
                target_address: address,
                target_pc,
                resolution,
            });
        }
    }
    Ok(references)
}

pub fn decode_animation_stream(bytes: &[u8]) -> Result<Vec<AnimationInstruction>, String> {
    let mut result = Vec::new();
    let mut offset = 0;
    while offset < bytes.len() {
        let opcode = bytes[offset];
        if opcode & 1 != 0 || opcode > 0x8a {
            return Err(format!("Invalid animation opcode at {offset:#x}"));
        }
        let length = if opcode == 0 {
            let sub = *bytes.get(offset + 1).ok_or("Missing animation subopcode")?;
            if sub & 1 != 0 || sub > 0x58 {
                return Err(format!("Invalid animation subopcode at {offset:#x}"));
            }
            1 + SUB_LENGTHS[sub as usize / 2]
        } else {
            LENGTHS[opcode as usize / 2]
        };
        let end = offset + 1 + length;
        let operands = bytes
            .get(offset + 1..end)
            .ok_or("Truncated animation instruction")?;
        result.push(AnimationInstruction {
            offset,
            opcode,
            operands: operands.to_vec(),
        });
        offset = end;
    }
    Ok(result)
}

pub fn encode_animation_stream(instructions: &[AnimationInstruction]) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    for instruction in instructions {
        if instruction.offset != bytes.len() {
            return Err("Noncontiguous animation instruction offsets".into());
        }
        bytes.push(instruction.opcode);
        bytes.extend_from_slice(&instruction.operands);
    }
    if decode_animation_stream(&bytes)? != instructions {
        return Err("Animation operand lengths do not match opcodes".into());
    }
    Ok(bytes)
}

/// Produce a same-size stream with one display-pose duration changed. The caller
/// must establish ROM identity, stream provenance and commit through its journal.
pub fn edit_pose_duration(
    bytes: &[u8],
    instruction_offset: usize,
    duration: u8,
) -> Result<Vec<u8>, String> {
    edit_pose_durations(
        bytes,
        &[DurationEdit {
            instruction_offset,
            duration,
        }],
    )
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DurationEdit {
    pub instruction_offset: usize,
    pub duration: u8,
}

/// Validate all selected frames before returning a complete replacement stream.
pub fn edit_pose_durations(bytes: &[u8], edits: &[DurationEdit]) -> Result<Vec<u8>, String> {
    if edits.is_empty() {
        return Err("Select at least one frame duration".into());
    }
    let mut instructions = decode_animation_stream(bytes)?;
    let mut seen = std::collections::BTreeSet::new();
    for edit in edits {
        if !seen.insert(edit.instruction_offset) {
            return Err("Duplicate frame duration edit".into());
        }
        let instruction = instructions
            .iter_mut()
            .find(|instruction| instruction.offset == edit.instruction_offset)
            .ok_or("Selected offset is not an animation instruction boundary")?;
        if instruction.displayed_pose().is_none() {
            return Err("Selected animation instruction does not display a pose".into());
        }
        instruction.operands[1] = edit.duration;
    }
    encode_animation_stream(&instructions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packed_motion_uses_nonzero_signed_nibbles_and_matches_two_byte_motion() {
        for (packed, x, y) in [
            (0x00, 1, 1),
            (0x77, 8, 8),
            (0x88, -8, -8),
            (0xff, -1, -1),
            (0x7f, 8, -1),
        ] {
            let instruction = decode_animation_stream(&[0x44, packed]).unwrap().remove(0);
            assert_eq!(
                instruction.timing_operation(),
                Some(TimingOperation::MoveBoth { x, y })
            );
            let flags = Some(RuntimeMirroring { dp_0e: 0, dp_8c: 0 });
            let packed_trace = trace_animation_with_mirroring(
                &[0x18, 0x44, packed, 0x20, 7, 1],
                0x48100,
                10,
                flags,
            )
            .unwrap();
            let plain_trace = trace_animation_with_mirroring(
                &[0x18, 0x10, x as u8, y as u8, 0x20, 7, 1],
                0x48100,
                10,
                flags,
            )
            .unwrap();
            assert_eq!(packed_trace.stop, TraceStop::IntervalEnd);
            assert_eq!(
                packed_trace.events[0].position_request,
                plain_trace.events[0].position_request
            );
            assert_eq!(
                packed_trace.events[0].known_state,
                plain_trace.events[0].known_state
            );
        }
    }

    #[test]
    fn horizontal_trace_uses_explicit_runtime_flags_without_booleanizing_xor() {
        let bytes = [0x1a, 10, 0, 0x1e, 10, 0x20, 7, 1];
        assert_eq!(
            trace_animation_timing(&bytes, 0x48100, 10).unwrap().stop,
            TraceStop::UnknownMirroring { offset: 3 }
        );
        for (dp_0e, dp_8c, x) in [(0, 0, 148), (1, 0, 108), (1, 1, 128), (2, 1, 108)] {
            let trace = trace_animation_with_mirroring(
                &bytes,
                0x48100,
                10,
                Some(RuntimeMirroring { dp_0e, dp_8c }),
            )
            .unwrap();
            assert_eq!(trace.stop, TraceStop::IntervalEnd);
            let position = trace.events[0].position_request.unwrap();
            assert_eq!(position.x_unmirrored, x);
            assert_eq!(position.x_mirrored, x);
        }
    }

    #[test]
    fn horizontal_movement_matches_runtime_clamping_and_zero_bypass() {
        assert_eq!(move_horizontal_position(128, 10, false), 138);
        assert_eq!(move_horizontal_position(128, 10, true), 118);
        assert_eq!(move_horizontal_position(128, -128, false), 72);
        assert_eq!(move_horizontal_position(128, -128, true), 184);
        assert_eq!(move_horizontal_position(0, 0, false), 0);
        assert_eq!(move_horizontal_position(256, 0, true), 256);
        assert_eq!(move_horizontal_position(256, 1, false), 184);
        assert_eq!(move_horizontal_position(i16::MAX, 1, false), 72);
        for position in 72..=184 {
            for delta in i8::MIN..=i8::MAX {
                assert!((72..=184).contains(&move_horizontal_position(position, delta, false)));
                assert!((72..=184).contains(&move_horizontal_position(position, delta, true)));
            }
        }
    }

    #[test]
    fn vertical_motion_accumulates_signed_offsets_and_requires_position_state() {
        let trace =
            trace_animation_timing(&[0x18, 0x1c, 0xfe, 0x1c, 3, 0x20, 7, 1], 0x48100, 10).unwrap();
        assert_eq!(trace.stop, TraceStop::IntervalEnd);
        assert_eq!(trace.events[0].position_request.unwrap().y, 137);
        assert_eq!(
            trace_animation_timing(&[0x1c, 1], 0x48100, 10)
                .unwrap()
                .stop,
            TraceStop::UninitializedPosition { offset: 0 }
        );
    }

    #[test]
    fn explicit_mirroring_resolves_state_byte_ordering() {
        let bytes = [0x28, 1, 2, 3, 4, 0x20, 7, 1];
        for (dp_0e, expected) in [(0, [2, 1, 4, 3]), (1, [1, 2, 3, 4]), (128, [1, 2, 3, 4])] {
            let trace = trace_animation_with_mirroring(
                &bytes,
                0x48100,
                10,
                Some(RuntimeMirroring { dp_0e, dp_8c: 0 }),
            )
            .unwrap();
            assert_eq!(trace.stop, TraceStop::IntervalEnd);
            let state = &trace.events[0].known_state;
            assert_eq!(
                state.iter().map(|byte| byte.value).collect::<Vec<_>>(),
                expected
            );
            assert!(state.iter().all(|byte| byte.known_mask == 0xff));
        }
    }

    #[test]
    fn mirrored_state_writes_keep_only_bits_shared_by_both_orderings() {
        let trace =
            trace_animation_timing(&[0x28, 0xa2, 0xa3, 0x55, 0x55, 0x20, 7, 1], 0x48100, 10)
                .unwrap();
        assert_eq!(trace.stop, TraceStop::IntervalEnd);
        let state = &trace.events[0].known_state;
        assert_eq!(state.len(), 4);
        for entry in &state[..2] {
            assert_eq!(entry.value, 0xa2);
            assert_eq!(entry.known_mask, 0xfe);
        }
        for entry in &state[2..] {
            assert_eq!(entry.value, 0x55);
            assert_eq!(entry.known_mask, 0xff);
        }
    }

    #[test]
    fn state_writes_preserve_known_bits_and_do_not_accumulate_duplicate_requests() {
        let trace =
            trace_animation_timing(&[0x36, 0x80, 7, 0x80, 8, 0x20, 1, 2], 0x48100, 10).unwrap();
        assert_eq!(trace.stop, TraceStop::IntervalEnd);
        assert_eq!(
            trace.events[0].known_state,
            vec![
                KnownStateByte {
                    direct_page: false,
                    address: 0x324,
                    value: 8,
                    known_mask: 0xff
                },
                KnownStateByte {
                    direct_page: false,
                    address: 0x326,
                    value: 0x20,
                    known_mask: 0x20
                },
                KnownStateByte {
                    direct_page: true,
                    address: 0x76,
                    value: 0x81,
                    known_mask: 0xff
                },
            ]
        );
    }

    #[test]
    fn reset_position_restores_origin_after_repositioning() {
        let trace =
            trace_animation_timing(&[0x1a, 10, 20, 0x20, 7, 1, 0x18, 2, 1], 0x48100, 10).unwrap();
        assert_eq!(trace.stop, TraceStop::IntervalEnd);
        assert_eq!(trace.events[0].position_request.unwrap().y, 156);
        assert_eq!(
            trace.events[1].position_request,
            Some(PositionRequest {
                x_unmirrored: 128,
                x_mirrored: 128,
                y: 136,
            })
        );
    }

    #[test]
    fn position_requests_are_signed_origin_relative_not_accumulated() {
        let trace = trace_animation_timing(
            &[0x1a, 0x80, 0xff, 0x20, 7, 1, 0x1a, 2, 3, 2, 1],
            0x48100,
            10,
        )
        .unwrap();
        assert_eq!(trace.stop, TraceStop::IntervalEnd);
        assert_eq!(
            trace.events[0].position_request,
            Some(PositionRequest {
                x_unmirrored: 0,
                x_mirrored: 256,
                y: 135
            })
        );
        assert_eq!(
            trace.events[1].position_request,
            Some(PositionRequest {
                x_unmirrored: 130,
                x_mirrored: 126,
                y: 139
            })
        );
    }

    #[test]
    fn timing_trace_preserves_facing_intent_without_assuming_runtime_mirroring() {
        let trace = trace_animation_timing(&[0x22, 0x20, 7, 3, 0x24, 2, 4], 0x48100, 10).unwrap();
        assert_eq!(trace.stop, TraceStop::IntervalEnd);
        assert_eq!(trace.events[0].facing_request, Some(FacingRequest::Left));
        assert_eq!(trace.events[1].facing_request, Some(FacingRequest::Right));
        assert_eq!(trace.events[1].pose, Some(7));
    }

    #[test]
    fn timing_trace_executes_loops_and_stops_without_guessing() {
        let trace = trace_animation_timing(&[6, 2, 0x20, 7, 3, 8, 2, 0x81], 0x48100, 20).unwrap();
        assert_eq!(trace.stop, TraceStop::IntervalEnd);
        assert_eq!(trace.events.len(), 2);
        assert!(trace
            .events
            .iter()
            .all(|event| event.pose == Some(7) && event.updates == 3));
        assert_eq!(
            trace_animation_timing(&[4, 0, 0x81], 0x48100, 5)
                .unwrap()
                .stop,
            TraceStop::StepLimit
        );
        assert_eq!(
            trace_animation_timing(&[8, 0, 0x81], 0x48100, 5)
                .unwrap()
                .stop,
            TraceStop::UninitializedLoop { offset: 0 }
        );
        assert_eq!(
            trace_animation_timing(&[0x76, 0, 0, 0], 0x48100, 5)
                .unwrap()
                .stop,
            TraceStop::UnknownInstruction { offset: 0 }
        );
        assert_eq!(
            trace_animation_timing(&[4, 1, 0x81], 0x48100, 5)
                .unwrap()
                .stop,
            TraceStop::UnverifiedTarget { address: 0x8101 }
        );
        let delay = trace_animation_timing(&[2, 0], 0x48100, 1).unwrap();
        assert_eq!(
            delay.events[0],
            TimedPose {
                known_state: vec![],
                position_request: None,
                facing_request: None,
                instruction_offset: 0,
                pose: None,
                updates: 256
            }
        );
        assert_eq!(delay.stop, TraceStop::IntervalEnd);
    }

    #[test]
    fn zero_delay_wraps_for_256_updates() {
        for counter in 0..=255u8 {
            let instruction = AnimationInstruction {
                offset: 0,
                opcode: 0x20,
                operands: vec![7, counter],
            };
            let mut remaining = counter;
            let mut updates = 0u16;
            loop {
                remaining = remaining.wrapping_sub(1);
                updates += 1;
                if remaining == 0 {
                    break;
                }
            }
            assert_eq!(instruction.uninterrupted_delay_updates(), Some(updates));
        }
        assert_eq!(
            decode_animation_stream(&[4, 0, 0x81]).unwrap()[0].uninterrupted_delay_updates(),
            None
        );
    }

    #[test]
    fn timing_operations_distinguish_jumps_loops_and_raw_delays() {
        let instructions =
            decode_animation_stream(&[6, 3, 0x20, 7, 0, 8, 0, 0x81, 4, 0, 0x82, 2, 5]).unwrap();
        assert_eq!(
            instructions[0].timing_operation(),
            Some(TimingOperation::SetLoopCounter { counter: 3 })
        );
        assert_eq!(
            instructions[1].timing_operation(),
            Some(TimingOperation::DisplayPose {
                pose: 7,
                counter: 0
            })
        );
        assert_eq!(
            instructions[2].timing_operation(),
            Some(TimingOperation::DecrementLoopAndBranch { target: 0x8100 })
        );
        assert_eq!(
            instructions[3].timing_operation(),
            Some(TimingOperation::Jump { target: 0x8200 })
        );
        assert_eq!(
            instructions[4].timing_operation(),
            Some(TimingOperation::Delay { counter: 5 })
        );
    }

    #[test]
    fn multiple_duration_edits_validate_as_one_replacement() {
        let bytes = [0x20, 7, 12, 0x22, 0x20, 8, 16];
        let edits = [
            DurationEdit {
                instruction_offset: 0,
                duration: 24,
            },
            DurationEdit {
                instruction_offset: 4,
                duration: 32,
            },
        ];
        assert_eq!(
            edit_pose_durations(&bytes, &edits).unwrap(),
            [0x20, 7, 24, 0x22, 0x20, 8, 32]
        );
        assert!(edit_pose_durations(&bytes, &[]).is_err());
        assert!(edit_pose_durations(&bytes, &[edits[0].clone(), edits[0].clone()]).is_err());
        assert!(edit_pose_durations(
            &bytes,
            &[
                edits[0].clone(),
                DurationEdit {
                    instruction_offset: 5,
                    duration: 1
                }
            ]
        )
        .is_err());
        assert_eq!(bytes, [0x20, 7, 12, 0x22, 0x20, 8, 16]);
    }

    #[test]
    fn catalog_resolves_cross_interval_references_but_not_unknown_destinations() {
        let mut rom = vec![0; 0x120];
        rom[0x100..0x109].copy_from_slice(&[4, 0x10, 0x81, 4, 0x11, 0x81, 4, 0x40, 0x81]);
        rom[0x110..0x113].copy_from_slice(&[0x20, 7, 12]);
        let regions = [
            region("a", "A", 0x008100, 0x008109),
            region("b", "B", 0x008110, 0x008113),
        ];
        let graph = analyze_animation_catalog(&rom, &regions).unwrap();
        assert_eq!(graph[0].target_region.as_deref(), Some("b"));
        assert_eq!(graph[0].reference.resolution, TargetResolution::Instruction);
        assert_eq!(
            graph[1].reference.resolution,
            TargetResolution::InsideOperand
        );
        assert_eq!(graph[2].target_region, None);
        assert_eq!(
            graph[2].reference.resolution,
            TargetResolution::OutsideInterval
        );
        assert!(
            analyze_animation_catalog(&rom, &[regions[0].clone(), regions[0].clone()]).is_err()
        );
        assert!(analyze_animation_catalog(
            &rom,
            &[
                regions[0].clone(),
                region("overlap", "", 0x008101, 0x008109)
            ]
        )
        .is_err());
    }

    #[test]
    fn reference_analysis_resolves_boundaries_without_following_unknown_data() {
        let bytes = [4, 0, 0x81, 4, 1, 0x81, 4, 0, 0x82, 4, 0, 0x10];
        let references = analyze_animation_references(&bytes, 0x48100).unwrap();
        assert_eq!(
            references
                .iter()
                .map(|r| r.resolution.clone())
                .collect::<Vec<_>>(),
            vec![
                TargetResolution::Instruction,
                TargetResolution::InsideOperand,
                TargetResolution::OutsideInterval,
                TargetResolution::InvalidAddress,
            ]
        );
        assert_eq!(references[0].target_pc, Some(0x48100));
        assert_eq!(references[3].target_pc, None);
        assert!(analyze_animation_references(&bytes, 0x47fff).is_err());
        assert!(analyze_animation_references(&bytes, usize::MAX).is_err());
        assert!(analyze_animation_references(&[], 0).is_err());
    }

    #[test]
    fn extracts_script_targets_without_treating_memory_addresses_as_branches() {
        let bytes = [
            0x00, 0x22, 0x50, 0x09, 2, 0x2e, 0x82, 0x7a, 1, 0x50, 0x09, 0x30, 0x82,
        ];
        let instructions = decode_animation_stream(&bytes).unwrap();
        assert_eq!(instructions[0].script_targets().unwrap(), vec![0x822e]);
        assert_eq!(instructions[1].script_targets().unwrap(), vec![0x8230]);
        let mixed =
            decode_animation_stream(&[0, 0x20, 1, 0, 0x81, 2, 3, 0, 0x82, 4, 5, 0, 0x83, 0, 0x84])
                .unwrap();
        assert_eq!(
            mixed[0].script_targets().unwrap(),
            vec![0x8100, 0x8200, 0x8300, 0x8400]
        );
        let malformed = AnimationInstruction {
            offset: 0,
            opcode: 4,
            operands: vec![0],
        };
        assert!(malformed.script_targets().is_err());
    }

    #[test]
    fn duration_edit_preserves_pose_and_all_other_instructions() {
        let bytes = [0x22, 0x20, 7, 12, 0x24];
        assert_eq!(
            edit_pose_duration(&bytes, 1, 24).unwrap(),
            [0x22, 0x20, 7, 24, 0x24]
        );
        assert!(edit_pose_duration(&bytes, 2, 24).is_err());
        assert!(edit_pose_duration(&bytes, 0, 24).is_err());
        assert!(edit_pose_duration(&[0x20, 7], 0, 24).is_err());
    }

    #[test]
    fn preserves_extended_instructions_and_pose_duration() {
        let bytes = [0x20, 7, 12, 0, 0x0c, 1, 2, 3, 0x22];
        let instructions = decode_animation_stream(&bytes).unwrap();
        assert_eq!(instructions.len(), 3);
        assert_eq!(instructions[0].displayed_pose(), Some((7, 12)));
        assert_eq!(instructions[2].offset, 8);
        assert_eq!(encode_animation_stream(&instructions).unwrap(), bytes);
    }

    #[test]
    fn rejects_malformed_and_truncated_instructions() {
        for bytes in [
            vec![1],
            vec![0x8c],
            vec![0],
            vec![0, 1],
            vec![0, 0x5a],
            vec![0x20, 1],
            vec![0, 0x18, 0],
        ] {
            assert!(decode_animation_stream(&bytes).is_err(), "{bytes:?}");
        }
        let malformed = [AnimationInstruction {
            offset: 0,
            opcode: 0x20,
            operands: vec![1],
        }];
        assert!(encode_animation_stream(&malformed).is_err());
    }

    #[test]
    #[ignore = "requires user-supplied USA ROM via SPO_USA_ROM"]
    fn usa_reference_animation_round_trip() {
        let bytes = std::fs::read(std::env::var("SPO_USA_ROM").unwrap()).unwrap();
        for region in VERIFIED_USA_ANIMATION_REGIONS {
            let stream = &bytes[region.pc_offset..region.pc_offset + region.length];
            let decoded = decode_animation_stream(stream).unwrap();
            let trace = trace_animation_timing(stream, region.pc_offset, 4096).unwrap();
            let opcode = match trace.stop {
                TraceStop::UnknownInstruction { offset } => Some(stream[offset]),
                _ => None,
            };
            eprintln!(
                "{}: {} timing events, stop {:?}, opcode {:?}",
                region.id,
                trace.events.len(),
                trace.stop,
                opcode
            );
            let references = analyze_animation_references(stream, region.pc_offset).unwrap();
            for reference in references {
                assert!(
                    matches!(
                        reference.resolution,
                        TargetResolution::Instruction | TargetResolution::OutsideInterval
                    ),
                    "{}: {reference:?}",
                    region.id
                );
            }
            assert!(
                decoded
                    .iter()
                    .any(|instruction| instruction.displayed_pose().is_some()),
                "{}",
                region.id
            );
            assert_eq!(
                encode_animation_stream(&decoded).unwrap(),
                stream,
                "{}",
                region.id
            );
        }
        // Exact example interval supplied by the pinned upstream disassembler.
        let stream = &bytes[0x68ebe..0x68ebe + 133];
        let decoded = decode_animation_stream(stream).unwrap();
        assert_eq!(encode_animation_stream(&decoded).unwrap(), stream);
        let frame = decoded
            .iter()
            .find(|instruction| instruction.displayed_pose().is_some())
            .unwrap();
        let edited =
            edit_pose_duration(stream, frame.offset, frame.operands[1].wrapping_add(1)).unwrap();
        let changed: Vec<_> = stream
            .iter()
            .zip(&edited)
            .enumerate()
            .filter_map(|(index, (before, after))| (before != after).then_some(index))
            .collect();
        assert_eq!(changed, vec![frame.offset + 2]);
    }
}
