use debugger_core::cpu::{Disassembler, MemoryAccess};
use debugger_core::{DisassembledInstruction, SnesAddress};
use rom_core::Rom;
use std::collections::HashMap;

use crate::roster_expansion::expand_roster_tables;
use crate::types::{
    ExpansionError, ExpansionOptions, ExpansionReport, ExpansionResult, HookSiteCandidate, WriteRange,
    EDITOR_HEADER_MAGIC,
};

const EDITOR_HEADER_VERSION: u8 = 2;
const EDITOR_HEADER_SIZE: usize = 96;
const MIN_HOOK_OVERWRITE_LEN: usize = 4;
const MAX_HOOK_OVERWRITE_LEN: usize = 32;
const CREATOR_MODE_FLAG_WRAM: [u8; 4] = [0xFF, 0x1F, 0x7E, 0x00]; // $7E1FFF
const CREATOR_MODE_MAGIC_WRAM: [u8; 4] = [0xFE, 0x1F, 0x7E, 0x00]; // $7E1FFE
const CREATOR_MODE_HEARTBEAT_WRAM: [u8; 4] = [0xFD, 0x1F, 0x7E, 0x00]; // $7E1FFD
const CREATOR_MODE_INPUT_LOW_WRAM: [u8; 4] = [0xFC, 0x1F, 0x7E, 0x00]; // $7E1FFC
const CREATOR_MODE_INPUT_HIGH_WRAM: [u8; 4] = [0xFB, 0x1F, 0x7E, 0x00]; // $7E1FFB
const CREATOR_MODE_CURSOR_WRAM: [u8; 4] = [0xFA, 0x1F, 0x7E, 0x00]; // $7E1FFA
const CREATOR_MODE_ACTION_WRAM: [u8; 4] = [0xF9, 0x1F, 0x7E, 0x00]; // $7E1FF9
const CREATOR_MODE_PAGE_WRAM: [u8; 4] = [0xF8, 0x1F, 0x7E, 0x00]; // $7E1FF8
const CREATOR_MODE_DIRTY_WRAM: [u8; 4] = [0xF7, 0x1F, 0x7E, 0x00]; // $7E1FF7
const CREATOR_RENDER_VISIBLE_WRAM: [u8; 4] = [0xF6, 0x1F, 0x7E, 0x00]; // $7E1FF6
const CREATOR_RENDER_PAGE_WRAM: [u8; 4] = [0xF5, 0x1F, 0x7E, 0x00]; // $7E1FF5
const CREATOR_RENDER_CURSOR_WRAM: [u8; 4] = [0xF4, 0x1F, 0x7E, 0x00]; // $7E1FF4
const CREATOR_RENDER_ROW0_WRAM: [u8; 4] = [0xF3, 0x1F, 0x7E, 0x00]; // $7E1FF3
const CREATOR_RENDER_ROW1_WRAM: [u8; 4] = [0xF2, 0x1F, 0x7E, 0x00]; // $7E1FF2
const CREATOR_RENDER_ROW2_WRAM: [u8; 4] = [0xF1, 0x1F, 0x7E, 0x00]; // $7E1FF1
const CREATOR_RENDER_ROW3_WRAM: [u8; 4] = [0xF0, 0x1F, 0x7E, 0x00]; // $7E1FF0
const CREATOR_RENDER_REVISION_WRAM: [u8; 4] = [0xEF, 0x1F, 0x7E, 0x00]; // $7E1FEF
const CREATOR_SESSION_CIRCUIT_WRAM: [u8; 4] = [0xEC, 0x1F, 0x7E, 0x00]; // $7E1FEC
const CREATOR_SESSION_UNLOCK_ORDER_WRAM: [u8; 4] = [0xEB, 0x1F, 0x7E, 0x00]; // $7E1FEB
const CREATOR_SESSION_INTRO_TEXT_ID_WRAM: [u8; 4] = [0xEA, 0x1F, 0x7E, 0x00]; // $7E1FEA
const CREATOR_SESSION_STATUS_WRAM: [u8; 4] = [0xE9, 0x1F, 0x7E, 0x00]; // $7E1FE9
const CREATOR_SESSION_ERROR_CODE_WRAM: [u8; 4] = [0xE8, 0x1F, 0x7E, 0x00]; // $7E1FE8
const CREATOR_NAME_EDIT_ACTIVE_WRAM: [u8; 4] = [0xE7, 0x1F, 0x7E, 0x00]; // $7E1FE7
const CREATOR_NAME_CURSOR_WRAM: [u8; 4] = [0xE6, 0x1F, 0x7E, 0x00]; // $7E1FE6
const CREATOR_INTRO_EDIT_ACTIVE_WRAM: [u8; 4] = [0xE4, 0x1F, 0x7E, 0x00]; // $7E1FE4
const CREATOR_INTRO_CURSOR_WRAM: [u8; 4] = [0xE3, 0x1F, 0x7E, 0x00]; // $7E1FE3
const CREATOR_INTRO_BUFFER_BASE: [u8; 4] = [0xD0, 0x1F, 0x7E, 0x00]; // $7E1FD0
const CREATOR_NAME_BUFFER_BASE: [u8; 4] = [0xC0, 0x1F, 0x7E, 0x00]; // $7E1FC0
const JOY1_LOW_MMIO: [u8; 4] = [0x18, 0x42, 0x00, 0x00]; // $00:4218
const JOY1_HIGH_MMIO: [u8; 4] = [0x19, 0x42, 0x00, 0x00]; // $00:4219
const CREATOR_MODE_LOW_MASK: u8 = 0x0C; // Select + Start
const CREATOR_MODE_HIGH_MASK: u8 = 0x0C; // L + R
const CREATOR_MODE_MAGIC_VALUE: u8 = 0x43; // 'C'
const CREATOR_MODE_HEARTBEAT_VALUE: u8 = 0xA5;
const CREATOR_MODE_PAGE_MAX: u8 = 0x03;
const CREATOR_MODE_CURSOR_MAX: u8 = 0x03;
const CREATOR_ACTION_NAME_EDIT: u8 = 0x11;
const CREATOR_ACTION_CIRCUIT_EDIT: u8 = 0x12;
const CREATOR_ACTION_PORTRAIT_EDIT: u8 = 0x13;
const CREATOR_ACTION_COMMIT: u8 = 0x14;
const CREATOR_ACTION_INTRO_EDIT: u8 = 0x15;
const CREATOR_ACTION_CANCEL: u8 = 0x16;
const CREATOR_ACTION_EXIT: u8 = 0xFF;
const CREATOR_SESSION_STATUS_DRAFT_READY: u8 = 0x02;
const CREATOR_SESSION_STATUS_COMMIT_PENDING: u8 = 0x03;
const CREATOR_SESSION_STATUS_CANCELLED: u8 = 0x07;
const CREATOR_INTRO_MAX_LEN: u8 = 16;
const CREATOR_NAME_MAX_LEN: u8 = 16;
const CREATOR_PAGE0_ROW0: u8 = 0x21;
const CREATOR_PAGE0_ROW1: u8 = 0x22;
const CREATOR_PAGE0_ROW2: u8 = 0x23;
const CREATOR_PAGE0_ROW3: u8 = 0x24;
const CREATOR_PAGE1_ROW0: u8 = 0x31;
const CREATOR_PAGE1_ROW1: u8 = 0x32;
const CREATOR_PAGE1_ROW2: u8 = 0x33;
const CREATOR_PAGE1_ROW3: u8 = 0x34;
const CREATOR_PAGE2_ROW0: u8 = 0x41;
const CREATOR_PAGE2_ROW1: u8 = 0x42;
const CREATOR_PAGE2_ROW2: u8 = 0x43;
const CREATOR_PAGE2_ROW3: u8 = 0x44;
const CREATOR_PAGE3_ROW0: u8 = 0x51;
const CREATOR_PAGE3_ROW1: u8 = 0x52;
const CREATOR_PAGE3_ROW2: u8 = 0x53;
const CREATOR_PAGE3_ROW3: u8 = 0x54;

#[derive(Debug, Clone)]
struct HookPatchPlan {
    hook_pc: usize,
    overwrite_len: usize,
    overwritten_bytes: Vec<u8>,
    return_pc: usize,
}

struct RomMemoryView<'a> {
    rom: &'a Rom,
}

impl MemoryAccess for RomMemoryView<'_> {
    fn read_byte(&self, addr: u32) -> Option<u8> {
        self.rom.data.get(addr as usize).copied()
    }

    fn write_byte(&mut self, _addr: u32, _value: u8) {}

    fn write_bytes(&mut self, _addr: u32, _values: &[u8]) {}
}

/// Analyze ROM code and return safe candidate hook sites for in-game editor patching.
///
/// This is heuristic: it scans instruction boundaries and accepts candidates that
/// can be safely overwritten with a JML trampoline according to current safety rules.
pub fn analyze_ingame_hook_sites(
    rom: &Rom,
    start_pc: usize,
    end_pc: usize,
    limit: usize,
) -> Vec<HookSiteCandidate> {
    if limit == 0 || start_pc >= end_pc {
        return Vec::new();
    }

    let max_end = end_pc.min(rom.size());
    let mut candidates = Vec::new();
    let mut cursor = start_pc.min(max_end);

    while cursor < max_end && candidates.len() < limit {
        let instruction = match decode_hook_instruction(rom, cursor) {
            Ok(instruction) => instruction,
            Err(_) => {
                cursor += 1;
                continue;
            }
        };

        if let Ok(overwrite_len) = determine_hook_overwrite_len(rom, cursor, None) {
            if let Ok(bytes) = rom.read_bytes(cursor, overwrite_len) {
                candidates.push(HookSiteCandidate {
                    hook_pc: cursor,
                    overwrite_len,
                    return_pc: cursor + overwrite_len,
                    first_instruction: format!("{} {}", instruction.mnemonic, instruction.operands).trim().to_string(),
                    preview_bytes: bytes.to_vec(),
                });
            }
        }

        let step = (instruction.size as usize).max(1);
        cursor = cursor.saturating_add(step);
    }

    candidates
}

/// Verify a specific hook site and return the resolved trampoline plan details.
pub fn verify_ingame_hook_site(
    rom: &Rom,
    hook_pc: usize,
    overwrite_len: Option<usize>,
) -> ExpansionResult<HookSiteCandidate> {
    let resolved_len = determine_hook_overwrite_len(rom, hook_pc, overwrite_len)?;
    let first_instruction = decode_hook_instruction(rom, hook_pc)?;
    let preview_bytes = rom
        .read_bytes(hook_pc, resolved_len)
        .map_err(|err| ExpansionError::Rom(err.to_string()))?
        .to_vec();

    Ok(HookSiteCandidate {
        hook_pc,
        overwrite_len: resolved_len,
        return_pc: hook_pc + resolved_len,
        first_instruction: format!(
            "{} {}",
            first_instruction.mnemonic, first_instruction.operands
        )
        .trim()
        .to_string(),
        preview_bytes,
    })
}

/// Expand roster tables and install in-ROM editor bootstrap metadata/stub.
pub fn apply_ingame_editor_expansion(
    rom: &mut Rom,
    options: &ExpansionOptions,
) -> ExpansionResult<ExpansionReport> {
    let (layout, mut write_ranges, mut notes) =
        expand_roster_tables(rom, options.target_boxer_count)?;

    let hook_plan = build_hook_patch_plan(rom, options)?;
    let bootstrap_size = estimate_bootstrap_size(hook_plan.as_ref());
    let bootstrap_alloc = rom
        .find_or_expand_free_space(bootstrap_size, 16)
        .ok_or(ExpansionError::FreeSpaceNotFound(
            "in-game editor bootstrap block",
        ))?;

    let header_pc = align_up(bootstrap_alloc.offset, 16);
    let header_bytes = build_editor_header(&layout, header_pc, hook_plan.as_ref());
    rom.write_bytes(header_pc, &header_bytes)
        .map_err(|err| ExpansionError::Rom(err.to_string()))?;
    write_ranges.push(WriteRange {
        start_pc: header_pc,
        size: header_bytes.len(),
        description: "In-game editor bootstrap header".to_string(),
    });

    let editor_stub_pc = align_up(header_pc + header_bytes.len(), 8);
    let editor_stub = build_editor_stub(hook_plan.as_ref())?;
    rom.write_bytes(editor_stub_pc, &editor_stub)
        .map_err(|err| ExpansionError::Rom(err.to_string()))?;
    write_ranges.push(WriteRange {
        start_pc: editor_stub_pc,
        size: editor_stub.len(),
        description: "In-game editor bootstrap 65816 stub".to_string(),
    });

    let editor_hook_patched = if let Some(hook_plan) = hook_plan.as_ref() {
        let jml = build_jml_instruction(editor_stub_pc, rom);
        rom.write_bytes(hook_plan.hook_pc, &jml)
            .map_err(|err| ExpansionError::Rom(err.to_string()))?;
        write_ranges.push(WriteRange {
            start_pc: hook_plan.hook_pc,
            size: jml.len(),
            description: "Editor mode hook (JML)".to_string(),
        });

        if hook_plan.overwrite_len > jml.len() {
            let nop_count = hook_plan.overwrite_len - jml.len();
            let nops = vec![0xEA; nop_count];
            rom.write_bytes(hook_plan.hook_pc + jml.len(), &nops)
                .map_err(|err| ExpansionError::Rom(err.to_string()))?;
            write_ranges.push(WriteRange {
                start_pc: hook_plan.hook_pc + jml.len(),
                size: nop_count,
                description: "Editor hook trailing bytes replaced with NOP".to_string(),
            });
        }

        notes.push(format!(
            "Trampoline-safe hook patch applied at PC 0x{:06X}; preserved {} byte(s) and resumes at PC 0x{:06X}.",
            hook_plan.hook_pc, hook_plan.overwrite_len, hook_plan.return_pc
        ));
        true
    } else {
        notes.push(
            "No code hook patched. Bootstrap is installed, but game execution is not redirected yet."
                .to_string(),
        );
        false
    };

    notes.push(
        "Creator mode trigger installed: while running the ROM, hold Select+Start+L+R to set WRAM flags at $7E:1FFE/$7E:1FFF."
            .to_string(),
    );
    notes.push(
        "Creator mode dispatcher active: when $7E:1FFF is set, runtime heartbeat marker is written to $7E:1FFD."
            .to_string(),
    );
    notes.push(
        "Creator runtime input state mirrors to $7E:1FFC/$7E:1FFB, cursor index at $7E:1FFA, action latch at $7E:1FF9."
            .to_string(),
    );
    notes.push(
        "Renderer contract publishes $7E:1FF6..$7E:1FEF (visible/page/cursor/row0..row3/revision) and clears dirty flag $7E:1FF7 after each publish."
            .to_string(),
    );
    notes.push(
        "Entering creator mode initializes cursor/action latches to zero for deterministic first-frame behavior."
            .to_string(),
    );
    notes.push(
        "UI/menu/font rendering still needs full SNES-side implementation; this pass creates installable ROM-side scaffolding."
            .to_string(),
    );

    Ok(ExpansionReport {
        layout,
        header_pc,
        editor_stub_pc,
        editor_hook_patched,
        editor_hook_overwrite_len: hook_plan.as_ref().map_or(0, |plan| plan.overwrite_len),
        write_ranges,
        notes,
    })
}

fn estimate_bootstrap_size(hook_plan: Option<&HookPatchPlan>) -> usize {
    // Header (~96 bytes) + code stub + alignment slack + preserved bytes for trampoline flow.
    128 + hook_plan.map_or(0, |plan| plan.overwrite_len + 8)
}

fn build_editor_header(
    layout: &crate::types::ExpandedRosterLayout,
    header_pc: usize,
    hook_plan: Option<&HookPatchPlan>,
) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(96);
    bytes.extend_from_slice(&EDITOR_HEADER_MAGIC);
    bytes.push(EDITOR_HEADER_VERSION);
    bytes.push(0); // flags reserved
    bytes.extend_from_slice(&(layout.boxer_count as u16).to_le_bytes());

    bytes.extend_from_slice(&(layout.name_pointer_table_pc as u32).to_le_bytes());
    bytes.extend_from_slice(&(layout.name_long_pointer_table_pc as u32).to_le_bytes());
    bytes.extend_from_slice(&(layout.name_blob_pc as u32).to_le_bytes());
    bytes.extend_from_slice(&(layout.circuit_table_pc as u32).to_le_bytes());
    bytes.extend_from_slice(&(layout.unlock_table_pc as u32).to_le_bytes());
    bytes.extend_from_slice(&(layout.intro_table_pc as u32).to_le_bytes());

    bytes.extend_from_slice(&(header_pc as u32).to_le_bytes());
    bytes.extend_from_slice(&(hook_plan.map_or(0, |plan| plan.hook_pc) as u32).to_le_bytes());
    bytes.extend_from_slice(&(hook_plan.map_or(0, |plan| plan.overwrite_len) as u16).to_le_bytes());
    bytes.extend_from_slice(&0u16.to_le_bytes()); // reserved
    // Version 2 contract pointers consumed by ROM-side menu/font renderer.
    bytes.extend_from_slice(&CREATOR_RENDER_VISIBLE_WRAM);
    bytes.extend_from_slice(&CREATOR_RENDER_PAGE_WRAM);
    bytes.extend_from_slice(&CREATOR_RENDER_CURSOR_WRAM);
    bytes.extend_from_slice(&CREATOR_RENDER_ROW0_WRAM);
    bytes.extend_from_slice(&CREATOR_RENDER_ROW1_WRAM);
    bytes.extend_from_slice(&CREATOR_RENDER_ROW2_WRAM);
    bytes.extend_from_slice(&CREATOR_RENDER_ROW3_WRAM);
    bytes.extend_from_slice(&CREATOR_RENDER_REVISION_WRAM);
    bytes.resize(EDITOR_HEADER_SIZE, 0);
    bytes
}

struct StubAssembler {
    bytes: Vec<u8>,
    labels: HashMap<&'static str, usize>,
    branches: Vec<(usize, &'static str)>,
    long_branches: Vec<(usize, &'static str)>,
}

impl StubAssembler {
    fn new(capacity: usize) -> Self {
        Self {
            bytes: Vec::with_capacity(capacity),
            labels: HashMap::new(),
            branches: Vec::new(),
            long_branches: Vec::new(),
        }
    }

    fn pos(&self) -> usize {
        self.bytes.len()
    }

    fn byte(&mut self, value: u8) {
        self.bytes.push(value);
    }

    fn bytes(&mut self, values: &[u8]) {
        self.bytes.extend_from_slice(values);
    }

    fn label(&mut self, name: &'static str) {
        self.labels.insert(name, self.pos());
    }

    fn branch(&mut self, opcode: u8, target: &'static str) {
        let pos = self.pos();
        self.bytes(&[opcode, 0x00]);
        self.branches.push((pos, target));
    }

    fn branch_long(&mut self, target: &'static str) {
        let pos = self.pos();
        self.bytes(&[0x82, 0x00, 0x00]); // BRL
        self.long_branches.push((pos, target));
    }

    fn finalize(mut self) -> ExpansionResult<Vec<u8>> {
        for (branch_pos, target_label) in &self.branches {
            let target_pos = *self
                .labels
                .get(target_label)
                .ok_or_else(|| ExpansionError::Rom(format!("unresolved stub label: {target_label}")))?;
            let rel = target_pos as isize - (*branch_pos as isize + 2);
            if !(-128..=127).contains(&rel) {
                return Err(ExpansionError::Rom(format!(
                    "stub branch out of range: label {target_label} from {branch_pos:#X} to {target_pos:#X}"
                )));
            }
            self.bytes[*branch_pos + 1] = (rel as i8) as u8;
        }
        for (branch_pos, target_label) in &self.long_branches {
            let target_pos = *self
                .labels
                .get(target_label)
                .ok_or_else(|| ExpansionError::Rom(format!("unresolved stub label: {target_label}")))?;
            let rel = target_pos as isize - (*branch_pos as isize + 3);
            if !(-32768..=32767).contains(&rel) {
                return Err(ExpansionError::Rom(format!(
                    "stub long branch out of range: label {target_label} from {branch_pos:#X} to {target_pos:#X}"
                )));
            }
            let rel = rel as i16 as u16;
            self.bytes[*branch_pos + 1] = (rel & 0xFF) as u8;
            self.bytes[*branch_pos + 2] = (rel >> 8) as u8;
        }
        Ok(self.bytes)
    }
}

fn lda_long(asm: &mut StubAssembler, addr: [u8; 4]) {
    asm.bytes(&[0xAF, addr[0], addr[1], addr[2]]);
}

fn sta_long(asm: &mut StubAssembler, addr: [u8; 4]) {
    asm.bytes(&[0x8F, addr[0], addr[1], addr[2]]);
}

fn emit_stub_tail(asm: &mut StubAssembler, hook_plan: Option<&HookPatchPlan>) {
    if let Some(hook_plan) = hook_plan {
        asm.bytes(&hook_plan.overwritten_bytes);
        let (bank, addr) = pc_to_lorom(hook_plan.return_pc);
        asm.bytes(&[0x5C, (addr & 0xFF) as u8, (addr >> 8) as u8, bank]);
    } else {
        asm.byte(0x6B); // RTL
    }
}

fn build_editor_stub(hook_plan: Option<&HookPatchPlan>) -> ExpansionResult<Vec<u8>> {
    let mut asm = StubAssembler::new(320 + hook_plan.map_or(0, |plan| plan.overwrite_len));
    // Preserve state and force 8-bit accumulator.
    asm.byte(0x08); // PHP
    asm.bytes(&[0xE2, 0x20]); // SEP #$20

    // Mirror current controller state every hook pass.
    lda_long(&mut asm, JOY1_LOW_MMIO);
    sta_long(&mut asm, CREATOR_MODE_INPUT_LOW_WRAM);
    lda_long(&mut asm, JOY1_HIGH_MMIO);
    sta_long(&mut asm, CREATOR_MODE_INPUT_HIGH_WRAM);

    // Combo trigger: Select+Start+L+R enters creator mode.
    lda_long(&mut asm, CREATOR_MODE_INPUT_LOW_WRAM);
    asm.bytes(&[0x29, CREATOR_MODE_LOW_MASK]); // AND #$0C
    asm.bytes(&[0xC9, CREATOR_MODE_LOW_MASK]); // CMP #$0C
    asm.branch(0xD0, "after_combo_enter"); // BNE
    lda_long(&mut asm, CREATOR_MODE_INPUT_HIGH_WRAM);
    asm.bytes(&[0x29, CREATOR_MODE_HIGH_MASK]); // AND #$0C
    asm.bytes(&[0xC9, CREATOR_MODE_HIGH_MASK]); // CMP #$0C
    asm.branch(0xD0, "after_combo_enter"); // BNE
    asm.bytes(&[0xA9, 0x01]); // LDA #$01
    sta_long(&mut asm, CREATOR_MODE_FLAG_WRAM);
    asm.bytes(&[0xA9, CREATOR_MODE_MAGIC_VALUE]); // LDA #'C'
    sta_long(&mut asm, CREATOR_MODE_MAGIC_WRAM);
    asm.bytes(&[0xA9, 0x00]); // LDA #$00
    sta_long(&mut asm, CREATOR_MODE_CURSOR_WRAM);
    sta_long(&mut asm, CREATOR_MODE_ACTION_WRAM);
    sta_long(&mut asm, CREATOR_MODE_PAGE_WRAM);
    sta_long(&mut asm, CREATOR_NAME_EDIT_ACTIVE_WRAM);
    sta_long(&mut asm, CREATOR_NAME_CURSOR_WRAM);
    sta_long(&mut asm, CREATOR_INTRO_EDIT_ACTIVE_WRAM);
    sta_long(&mut asm, CREATOR_INTRO_CURSOR_WRAM);
    asm.bytes(&[0xA9, 0x01]); // LDA #$01
    sta_long(&mut asm, CREATOR_MODE_DIRTY_WRAM);
    asm.label("after_combo_enter");

    // Dispatch only while creator mode is active.
    lda_long(&mut asm, CREATOR_MODE_FLAG_WRAM);
    asm.branch(0xF0, "finish_quick"); // BEQ
    asm.branch(0x80, "active_start"); // BRA

    // Fast exit path when creator mode is not active.
    asm.label("finish_quick");
    asm.bytes(&[0xA9, 0x00]); // LDA #$00
    sta_long(&mut asm, CREATOR_RENDER_VISIBLE_WRAM);
    asm.byte(0x28); // PLP
    emit_stub_tail(&mut asm, hook_plan);

    asm.label("active_start");
    asm.bytes(&[0xA9, CREATOR_MODE_HEARTBEAT_VALUE]); // LDA #$A5
    sta_long(&mut asm, CREATOR_MODE_HEARTBEAT_WRAM);

    // Reset action latch each frame before evaluating input.
    asm.bytes(&[0xA9, 0x00]); // LDA #$00
    sta_long(&mut asm, CREATOR_MODE_ACTION_WRAM);

    // Intro edit mode owns the D-pad until cancelled/accepted.
    lda_long(&mut asm, CREATOR_INTRO_EDIT_ACTIVE_WRAM);
    asm.branch(0xD0, "intro_edit_active"); // BNE
    asm.branch_long("check_name_edit_mode");

    asm.label("intro_edit_active");
    // B exits intro edit mode without leaving creator mode.
    lda_long(&mut asm, CREATOR_MODE_INPUT_LOW_WRAM);
    asm.bytes(&[0x29, 0x01]); // AND #B
    asm.branch(0xF0, "intro_edit_check_left"); // BEQ
    asm.bytes(&[0xA9, 0x00]);
    sta_long(&mut asm, CREATOR_INTRO_EDIT_ACTIVE_WRAM);
    asm.bytes(&[0xA9, 0x01]);
    sta_long(&mut asm, CREATOR_MODE_DIRTY_WRAM);
    asm.branch_long("update_render_contract");

    asm.label("intro_edit_check_left");
    lda_long(&mut asm, CREATOR_MODE_INPUT_LOW_WRAM);
    asm.bytes(&[0x29, 0x40]); // AND #Left
    asm.branch(0xF0, "intro_edit_check_right"); // BEQ
    lda_long(&mut asm, CREATOR_INTRO_CURSOR_WRAM);
    asm.branch(0xF0, "intro_edit_check_right"); // BEQ
    asm.byte(0x3A); // DEC A
    sta_long(&mut asm, CREATOR_INTRO_CURSOR_WRAM);
    asm.bytes(&[0xA9, 0x01]);
    sta_long(&mut asm, CREATOR_MODE_DIRTY_WRAM);

    asm.label("intro_edit_check_right");
    lda_long(&mut asm, CREATOR_MODE_INPUT_LOW_WRAM);
    asm.bytes(&[0x29, 0x80]); // AND #Right
    asm.branch(0xF0, "intro_edit_check_up"); // BEQ
    lda_long(&mut asm, CREATOR_INTRO_CURSOR_WRAM);
    asm.bytes(&[0xC9, CREATOR_INTRO_MAX_LEN - 1]);
    asm.branch(0xB0, "intro_edit_check_up"); // BCS
    asm.byte(0x1A); // INC A
    sta_long(&mut asm, CREATOR_INTRO_CURSOR_WRAM);
    asm.bytes(&[0xA9, 0x01]);
    sta_long(&mut asm, CREATOR_MODE_DIRTY_WRAM);

    asm.label("intro_edit_check_up");
    lda_long(&mut asm, CREATOR_MODE_INPUT_LOW_WRAM);
    asm.bytes(&[0x29, 0x10]); // AND #Up
    asm.branch(0xF0, "intro_edit_check_down"); // BEQ
    asm.bytes(&[0xDA, 0xE2, 0x10]); // PHX / SEP #$10
    lda_long(&mut asm, CREATOR_INTRO_CURSOR_WRAM);
    asm.byte(0xAA); // TAX
    asm.bytes(&[0xBF, CREATOR_INTRO_BUFFER_BASE[0], CREATOR_INTRO_BUFFER_BASE[1], CREATOR_INTRO_BUFFER_BASE[2]]); // LDA $7E1FD0,X
    asm.bytes(&[0xC9, 0x20]); // CMP #' '
    asm.branch(0x90, "intro_edit_up_wrap"); // BCC
    asm.bytes(&[0xC9, 0x5A]); // CMP #'Z'
    asm.branch(0xB0, "intro_edit_up_wrap"); // BCS
    asm.byte(0x1A); // INC A
    asm.branch(0x80, "intro_edit_up_store"); // BRA
    asm.label("intro_edit_up_wrap");
    asm.bytes(&[0xA9, 0x20]); // LDA #' '
    asm.label("intro_edit_up_store");
    asm.bytes(&[0x9F, CREATOR_INTRO_BUFFER_BASE[0], CREATOR_INTRO_BUFFER_BASE[1], CREATOR_INTRO_BUFFER_BASE[2]]); // STA $7E1FD0,X
    asm.byte(0xFA); // PLX
    asm.bytes(&[0xA9, CREATOR_SESSION_STATUS_DRAFT_READY]);
    sta_long(&mut asm, CREATOR_SESSION_STATUS_WRAM);
    asm.bytes(&[0xA9, 0x00]);
    sta_long(&mut asm, CREATOR_SESSION_ERROR_CODE_WRAM);
    asm.bytes(&[0xA9, CREATOR_ACTION_INTRO_EDIT]);
    sta_long(&mut asm, CREATOR_MODE_ACTION_WRAM);
    asm.bytes(&[0xA9, 0x01]);
    sta_long(&mut asm, CREATOR_MODE_DIRTY_WRAM);

    asm.label("intro_edit_check_down");
    lda_long(&mut asm, CREATOR_MODE_INPUT_LOW_WRAM);
    asm.bytes(&[0x29, 0x20]); // AND #Down
    asm.branch(0xF0, "intro_edit_check_a"); // BEQ
    asm.bytes(&[0xDA, 0xE2, 0x10]); // PHX / SEP #$10
    lda_long(&mut asm, CREATOR_INTRO_CURSOR_WRAM);
    asm.byte(0xAA); // TAX
    asm.bytes(&[0xBF, CREATOR_INTRO_BUFFER_BASE[0], CREATOR_INTRO_BUFFER_BASE[1], CREATOR_INTRO_BUFFER_BASE[2]]); // LDA $7E1FD0,X
    asm.bytes(&[0xC9, 0x20]); // CMP #' '
    asm.branch(0xB0, "intro_edit_down_decrement"); // BCS
    asm.bytes(&[0xA9, 0x5A]); // LDA #'Z'
    asm.branch(0x80, "intro_edit_down_store"); // BRA
    asm.label("intro_edit_down_decrement");
    asm.bytes(&[0xC9, 0x21]); // CMP #'!'+1
    asm.branch(0xB0, "intro_edit_down_real_decrement"); // BCS
    asm.bytes(&[0xA9, 0x5A]); // LDA #'Z'
    asm.branch(0x80, "intro_edit_down_store"); // BRA
    asm.label("intro_edit_down_real_decrement");
    asm.byte(0x3A); // DEC A
    asm.label("intro_edit_down_store");
    asm.bytes(&[0x9F, CREATOR_INTRO_BUFFER_BASE[0], CREATOR_INTRO_BUFFER_BASE[1], CREATOR_INTRO_BUFFER_BASE[2]]); // STA $7E1FD0,X
    asm.byte(0xFA); // PLX
    asm.bytes(&[0xA9, CREATOR_SESSION_STATUS_DRAFT_READY]);
    sta_long(&mut asm, CREATOR_SESSION_STATUS_WRAM);
    asm.bytes(&[0xA9, 0x00]);
    sta_long(&mut asm, CREATOR_SESSION_ERROR_CODE_WRAM);
    asm.bytes(&[0xA9, CREATOR_ACTION_INTRO_EDIT]);
    sta_long(&mut asm, CREATOR_MODE_ACTION_WRAM);
    asm.bytes(&[0xA9, 0x01]);
    sta_long(&mut asm, CREATOR_MODE_DIRTY_WRAM);

    asm.label("intro_edit_check_a");
    lda_long(&mut asm, CREATOR_MODE_INPUT_HIGH_WRAM);
    asm.bytes(&[0x29, 0x01]); // AND #A
    asm.branch(0xD0, "intro_edit_accept_input"); // BNE
    asm.branch_long("update_render_contract");
    asm.label("intro_edit_accept_input");
    lda_long(&mut asm, CREATOR_INTRO_CURSOR_WRAM);
    asm.bytes(&[0xC9, CREATOR_INTRO_MAX_LEN - 1]);
    asm.branch(0xB0, "intro_edit_finish"); // BCS
    asm.byte(0x1A); // INC A
    sta_long(&mut asm, CREATOR_INTRO_CURSOR_WRAM);
    asm.branch(0x80, "intro_edit_accept_done"); // BRA
    asm.label("intro_edit_finish");
    asm.bytes(&[0xA9, 0x00]);
    sta_long(&mut asm, CREATOR_INTRO_EDIT_ACTIVE_WRAM);
    asm.label("intro_edit_accept_done");
    asm.bytes(&[0xA9, CREATOR_ACTION_INTRO_EDIT]);
    sta_long(&mut asm, CREATOR_MODE_ACTION_WRAM);
    asm.bytes(&[0xA9, 0x01]);
    sta_long(&mut asm, CREATOR_MODE_DIRTY_WRAM);
    asm.branch_long("update_render_contract");

    asm.label("check_name_edit_mode");
    // Name edit mode owns the D-pad until cancelled/accepted.
    lda_long(&mut asm, CREATOR_NAME_EDIT_ACTIVE_WRAM);
    asm.branch(0xD0, "name_edit_active"); // BNE
    asm.branch_long("check_left");

    asm.label("name_edit_active");
    // B exits name edit mode without leaving creator mode.
    lda_long(&mut asm, CREATOR_MODE_INPUT_LOW_WRAM);
    asm.bytes(&[0x29, 0x01]); // AND #B
    asm.branch(0xF0, "name_edit_check_left"); // BEQ
    asm.bytes(&[0xA9, 0x00]);
    sta_long(&mut asm, CREATOR_NAME_EDIT_ACTIVE_WRAM);
    asm.bytes(&[0xA9, 0x01]);
    sta_long(&mut asm, CREATOR_MODE_DIRTY_WRAM);
    asm.branch_long("update_render_contract");

    asm.label("name_edit_check_left");
    lda_long(&mut asm, CREATOR_MODE_INPUT_LOW_WRAM);
    asm.bytes(&[0x29, 0x40]); // AND #Left
    asm.branch(0xF0, "name_edit_check_right"); // BEQ
    lda_long(&mut asm, CREATOR_NAME_CURSOR_WRAM);
    asm.branch(0xF0, "name_edit_check_right"); // BEQ
    asm.byte(0x3A); // DEC A
    sta_long(&mut asm, CREATOR_NAME_CURSOR_WRAM);
    asm.bytes(&[0xA9, 0x01]);
    sta_long(&mut asm, CREATOR_MODE_DIRTY_WRAM);

    asm.label("name_edit_check_right");
    lda_long(&mut asm, CREATOR_MODE_INPUT_LOW_WRAM);
    asm.bytes(&[0x29, 0x80]); // AND #Right
    asm.branch(0xF0, "name_edit_check_up"); // BEQ
    lda_long(&mut asm, CREATOR_NAME_CURSOR_WRAM);
    asm.bytes(&[0xC9, CREATOR_NAME_MAX_LEN - 1]);
    asm.branch(0xB0, "name_edit_check_up"); // BCS
    asm.byte(0x1A); // INC A
    sta_long(&mut asm, CREATOR_NAME_CURSOR_WRAM);
    asm.bytes(&[0xA9, 0x01]);
    sta_long(&mut asm, CREATOR_MODE_DIRTY_WRAM);

    asm.label("name_edit_check_up");
    lda_long(&mut asm, CREATOR_MODE_INPUT_LOW_WRAM);
    asm.bytes(&[0x29, 0x10]); // AND #Up
    asm.branch(0xF0, "name_edit_check_down"); // BEQ
    asm.bytes(&[0xDA, 0xE2, 0x10]); // PHX / SEP #$10
    lda_long(&mut asm, CREATOR_NAME_CURSOR_WRAM);
    asm.byte(0xAA); // TAX
    asm.bytes(&[0xBF, CREATOR_NAME_BUFFER_BASE[0], CREATOR_NAME_BUFFER_BASE[1], CREATOR_NAME_BUFFER_BASE[2]]); // LDA $7E1FC0,X
    asm.bytes(&[0xC9, 0x20]); // CMP #' '
    asm.branch(0x90, "name_edit_up_wrap"); // BCC
    asm.bytes(&[0xC9, 0x5A]); // CMP #'Z'
    asm.branch(0xB0, "name_edit_up_wrap"); // BCS
    asm.byte(0x1A); // INC A
    asm.branch(0x80, "name_edit_up_store"); // BRA
    asm.label("name_edit_up_wrap");
    asm.bytes(&[0xA9, 0x20]); // LDA #' '
    asm.label("name_edit_up_store");
    asm.bytes(&[0x9F, CREATOR_NAME_BUFFER_BASE[0], CREATOR_NAME_BUFFER_BASE[1], CREATOR_NAME_BUFFER_BASE[2]]); // STA $7E1FC0,X
    asm.byte(0xFA); // PLX
    asm.bytes(&[0xA9, CREATOR_SESSION_STATUS_DRAFT_READY]);
    sta_long(&mut asm, CREATOR_SESSION_STATUS_WRAM);
    asm.bytes(&[0xA9, 0x00]);
    sta_long(&mut asm, CREATOR_SESSION_ERROR_CODE_WRAM);
    asm.bytes(&[0xA9, CREATOR_ACTION_NAME_EDIT]);
    sta_long(&mut asm, CREATOR_MODE_ACTION_WRAM);
    asm.bytes(&[0xA9, 0x01]);
    sta_long(&mut asm, CREATOR_MODE_DIRTY_WRAM);

    asm.label("name_edit_check_down");
    lda_long(&mut asm, CREATOR_MODE_INPUT_LOW_WRAM);
    asm.bytes(&[0x29, 0x20]); // AND #Down
    asm.branch(0xF0, "name_edit_check_a"); // BEQ
    asm.bytes(&[0xDA, 0xE2, 0x10]); // PHX / SEP #$10
    lda_long(&mut asm, CREATOR_NAME_CURSOR_WRAM);
    asm.byte(0xAA); // TAX
    asm.bytes(&[0xBF, CREATOR_NAME_BUFFER_BASE[0], CREATOR_NAME_BUFFER_BASE[1], CREATOR_NAME_BUFFER_BASE[2]]); // LDA $7E1FC0,X
    asm.bytes(&[0xC9, 0x20]); // CMP #' '
    asm.branch(0xB0, "name_edit_down_decrement"); // BCS
    asm.bytes(&[0xA9, 0x5A]); // LDA #'Z'
    asm.branch(0x80, "name_edit_down_store"); // BRA
    asm.label("name_edit_down_decrement");
    asm.bytes(&[0xC9, 0x21]); // CMP #'!'+1
    asm.branch(0xB0, "name_edit_down_real_decrement"); // BCS
    asm.bytes(&[0xA9, 0x5A]); // LDA #'Z'
    asm.branch(0x80, "name_edit_down_store"); // BRA
    asm.label("name_edit_down_real_decrement");
    asm.byte(0x3A); // DEC A
    asm.label("name_edit_down_store");
    asm.bytes(&[0x9F, CREATOR_NAME_BUFFER_BASE[0], CREATOR_NAME_BUFFER_BASE[1], CREATOR_NAME_BUFFER_BASE[2]]); // STA $7E1FC0,X
    asm.byte(0xFA); // PLX
    asm.bytes(&[0xA9, CREATOR_SESSION_STATUS_DRAFT_READY]);
    sta_long(&mut asm, CREATOR_SESSION_STATUS_WRAM);
    asm.bytes(&[0xA9, 0x00]);
    sta_long(&mut asm, CREATOR_SESSION_ERROR_CODE_WRAM);
    asm.bytes(&[0xA9, CREATOR_ACTION_NAME_EDIT]);
    sta_long(&mut asm, CREATOR_MODE_ACTION_WRAM);
    asm.bytes(&[0xA9, 0x01]);
    sta_long(&mut asm, CREATOR_MODE_DIRTY_WRAM);

    asm.label("name_edit_check_a");
    lda_long(&mut asm, CREATOR_MODE_INPUT_HIGH_WRAM);
    asm.bytes(&[0x29, 0x01]); // AND #A
    asm.branch(0xD0, "name_edit_accept_input"); // BNE
    asm.branch_long("update_render_contract");
    asm.label("name_edit_accept_input");
    lda_long(&mut asm, CREATOR_NAME_CURSOR_WRAM);
    asm.bytes(&[0xC9, CREATOR_NAME_MAX_LEN - 1]);
    asm.branch(0xB0, "name_edit_finish"); // BCS
    asm.byte(0x1A); // INC A
    sta_long(&mut asm, CREATOR_NAME_CURSOR_WRAM);
    asm.branch(0x80, "name_edit_accept_done"); // BRA
    asm.label("name_edit_finish");
    asm.bytes(&[0xA9, 0x00]);
    sta_long(&mut asm, CREATOR_NAME_EDIT_ACTIVE_WRAM);
    asm.label("name_edit_accept_done");
    asm.bytes(&[0xA9, CREATOR_ACTION_NAME_EDIT]);
    sta_long(&mut asm, CREATOR_MODE_ACTION_WRAM);
    asm.bytes(&[0xA9, 0x01]);
    sta_long(&mut asm, CREATOR_MODE_DIRTY_WRAM);
    asm.branch_long("update_render_contract");

    // Left: previous page (wrap), reset cursor, mark dirty.
    asm.label("check_left");
    lda_long(&mut asm, CREATOR_MODE_INPUT_LOW_WRAM);
    asm.bytes(&[0x29, 0x40]); // AND #Left
    asm.branch(0xF0, "check_right"); // BEQ
    lda_long(&mut asm, CREATOR_MODE_PAGE_WRAM);
    asm.branch(0xF0, "wrap_page_left"); // BEQ
    asm.byte(0x3A); // DEC A
    asm.branch(0x80, "store_page_left"); // BRA
    asm.label("wrap_page_left");
    asm.bytes(&[0xA9, CREATOR_MODE_PAGE_MAX]); // LDA #max
    asm.label("store_page_left");
    sta_long(&mut asm, CREATOR_MODE_PAGE_WRAM);
    asm.bytes(&[0xA9, 0x00]); // LDA #$00
    sta_long(&mut asm, CREATOR_MODE_CURSOR_WRAM);
    asm.bytes(&[0xA9, 0x01]); // LDA #$01
    sta_long(&mut asm, CREATOR_MODE_DIRTY_WRAM);

    // Right: next page (wrap), reset cursor, mark dirty.
    asm.label("check_right");
    lda_long(&mut asm, CREATOR_MODE_INPUT_LOW_WRAM);
    asm.bytes(&[0x29, 0x80]); // AND #Right
    asm.branch(0xF0, "check_up"); // BEQ
    lda_long(&mut asm, CREATOR_MODE_PAGE_WRAM);
    asm.bytes(&[0xC9, CREATOR_MODE_PAGE_MAX]); // CMP #max
    asm.branch(0x90, "inc_page_right"); // BCC
    asm.bytes(&[0xA9, 0x00]); // LDA #$00
    asm.branch(0x80, "store_page_right"); // BRA
    asm.label("inc_page_right");
    asm.byte(0x1A); // INC A
    asm.label("store_page_right");
    sta_long(&mut asm, CREATOR_MODE_PAGE_WRAM);
    asm.bytes(&[0xA9, 0x00]); // LDA #$00
    sta_long(&mut asm, CREATOR_MODE_CURSOR_WRAM);
    asm.bytes(&[0xA9, 0x01]); // LDA #$01
    sta_long(&mut asm, CREATOR_MODE_DIRTY_WRAM);

    // Up: move cursor up.
    asm.label("check_up");
    lda_long(&mut asm, CREATOR_MODE_INPUT_LOW_WRAM);
    asm.bytes(&[0x29, 0x10]); // AND #Up
    asm.branch(0xF0, "check_down"); // BEQ
    lda_long(&mut asm, CREATOR_MODE_CURSOR_WRAM);
    asm.branch(0xF0, "check_down"); // BEQ
    asm.byte(0x3A); // DEC A
    sta_long(&mut asm, CREATOR_MODE_CURSOR_WRAM);
    asm.bytes(&[0xA9, 0x01]); // LDA #$01
    sta_long(&mut asm, CREATOR_MODE_DIRTY_WRAM);

    // Down: move cursor down.
    asm.label("check_down");
    lda_long(&mut asm, CREATOR_MODE_INPUT_LOW_WRAM);
    asm.bytes(&[0x29, 0x20]); // AND #Down
    asm.branch(0xF0, "check_a"); // BEQ
    lda_long(&mut asm, CREATOR_MODE_CURSOR_WRAM);
    asm.bytes(&[0xC9, CREATOR_MODE_CURSOR_MAX]); // CMP #max
    asm.branch(0xB0, "check_a"); // BCS
    asm.byte(0x1A); // INC A
    sta_long(&mut asm, CREATOR_MODE_CURSOR_WRAM);
    asm.bytes(&[0xA9, 0x01]); // LDA #$01
    sta_long(&mut asm, CREATOR_MODE_DIRTY_WRAM);

    // A button latches page-specific action.
    asm.label("check_a");
    lda_long(&mut asm, CREATOR_MODE_INPUT_HIGH_WRAM);
    asm.bytes(&[0x29, 0x01]); // AND #A
    asm.branch(0xD0, "dispatch_action"); // BNE
    asm.branch_long("check_b");
    asm.label("dispatch_action");
    lda_long(&mut asm, CREATOR_MODE_PAGE_WRAM);
    asm.bytes(&[0xC9, 0x00]); // CMP #0
    asm.branch(0xD0, "check_page_one"); // BNE
    asm.branch_long("action_identity");
    asm.label("check_page_one");
    asm.bytes(&[0xC9, 0x01]); // CMP #1
    asm.branch(0xD0, "check_page_two"); // BNE
    asm.branch_long("action_circuit");
    asm.label("check_page_two");
    asm.bytes(&[0xC9, 0x02]); // CMP #2
    asm.branch(0xD0, "check_page_three"); // BNE
    asm.branch_long("action_portrait");
    asm.label("check_page_three");
    asm.bytes(&[0xC9, 0x03]); // CMP #3
    asm.branch(0xD0, "dispatch_action_done"); // BNE
    asm.branch_long("action_commit");
    asm.label("dispatch_action_done");
    asm.branch_long("check_b");

    asm.label("action_identity");
    lda_long(&mut asm, CREATOR_MODE_CURSOR_WRAM);
    asm.bytes(&[0xC9, 0x00]); // CMP #name row
    asm.branch(0xF0, "action_name_enter");
    asm.bytes(&[0xC9, 0x01]); // CMP #intro row
    asm.branch(0xF0, "action_intro_enter");
    asm.bytes(&[0xC9, 0x02]); // CMP #unlock row
    asm.branch(0xF0, "action_unlock_order");
    asm.bytes(&[0xC9, 0x03]); // CMP #intro slot row
    asm.branch(0xF0, "action_intro_text");
    asm.bytes(&[0xA9, CREATOR_ACTION_NAME_EDIT]);
    asm.branch_long("store_action");
    asm.label("action_name_enter");
    asm.bytes(&[0xA9, 0x00]);
    sta_long(&mut asm, CREATOR_INTRO_EDIT_ACTIVE_WRAM);
    asm.bytes(&[0xA9, 0x01]);
    sta_long(&mut asm, CREATOR_NAME_EDIT_ACTIVE_WRAM);
    asm.bytes(&[0xA9, 0x00]);
    sta_long(&mut asm, CREATOR_NAME_CURSOR_WRAM);
    asm.bytes(&[0xA9, CREATOR_ACTION_NAME_EDIT]);
    asm.branch_long("store_action");
    asm.label("action_intro_enter");
    asm.bytes(&[0xA9, 0x00]);
    sta_long(&mut asm, CREATOR_NAME_EDIT_ACTIVE_WRAM);
    asm.bytes(&[0xA9, 0x01]);
    sta_long(&mut asm, CREATOR_INTRO_EDIT_ACTIVE_WRAM);
    asm.bytes(&[0xA9, 0x00]);
    sta_long(&mut asm, CREATOR_INTRO_CURSOR_WRAM);
    asm.bytes(&[0xA9, CREATOR_ACTION_INTRO_EDIT]);
    asm.branch_long("store_action");
    asm.label("action_intro_text");
    lda_long(&mut asm, CREATOR_SESSION_INTRO_TEXT_ID_WRAM);
    asm.byte(0x1A); // INC A
    sta_long(&mut asm, CREATOR_SESSION_INTRO_TEXT_ID_WRAM);
    asm.bytes(&[0xA9, CREATOR_SESSION_STATUS_DRAFT_READY]);
    sta_long(&mut asm, CREATOR_SESSION_STATUS_WRAM);
    asm.bytes(&[0xA9, 0x00]);
    sta_long(&mut asm, CREATOR_SESSION_ERROR_CODE_WRAM);
    asm.bytes(&[0xA9, CREATOR_ACTION_INTRO_EDIT]);
    asm.branch_long("store_action");
    asm.label("action_unlock_order");
    lda_long(&mut asm, CREATOR_SESSION_UNLOCK_ORDER_WRAM);
    asm.byte(0x1A); // INC A
    sta_long(&mut asm, CREATOR_SESSION_UNLOCK_ORDER_WRAM);
    asm.bytes(&[0xA9, CREATOR_SESSION_STATUS_DRAFT_READY]);
    sta_long(&mut asm, CREATOR_SESSION_STATUS_WRAM);
    asm.bytes(&[0xA9, 0x00]);
    sta_long(&mut asm, CREATOR_SESSION_ERROR_CODE_WRAM);
    asm.bytes(&[0xA9, CREATOR_ACTION_NAME_EDIT]);
    asm.branch_long("store_action");
    asm.label("action_circuit");
    lda_long(&mut asm, CREATOR_MODE_CURSOR_WRAM);
    sta_long(&mut asm, CREATOR_SESSION_CIRCUIT_WRAM);
    asm.bytes(&[0xA9, CREATOR_SESSION_STATUS_DRAFT_READY]);
    sta_long(&mut asm, CREATOR_SESSION_STATUS_WRAM);
    asm.bytes(&[0xA9, 0x00]);
    sta_long(&mut asm, CREATOR_SESSION_ERROR_CODE_WRAM);
    asm.bytes(&[0xA9, CREATOR_ACTION_CIRCUIT_EDIT]);
    asm.branch_long("store_action");
    asm.label("action_portrait");
    asm.bytes(&[0xA9, CREATOR_ACTION_PORTRAIT_EDIT]);
    asm.branch_long("store_action");
    asm.label("action_commit");
    lda_long(&mut asm, CREATOR_MODE_CURSOR_WRAM);
    asm.bytes(&[0xC9, 0x01]); // CMP #commit row
    asm.branch(0xD0, "action_cancel_check"); // BNE
    asm.bytes(&[0xA9, CREATOR_SESSION_STATUS_COMMIT_PENDING]);
    sta_long(&mut asm, CREATOR_SESSION_STATUS_WRAM);
    asm.bytes(&[0xA9, 0x00]);
    sta_long(&mut asm, CREATOR_SESSION_ERROR_CODE_WRAM);
    asm.bytes(&[0xA9, CREATOR_ACTION_COMMIT]);
    asm.branch_long("store_action");
    asm.label("action_cancel_check");
    lda_long(&mut asm, CREATOR_MODE_CURSOR_WRAM);
    asm.bytes(&[0xC9, 0x02]); // CMP #cancel row
    asm.branch(0xD0, "action_commit_noop"); // BNE
    asm.bytes(&[0xA9, CREATOR_SESSION_STATUS_CANCELLED]);
    sta_long(&mut asm, CREATOR_SESSION_STATUS_WRAM);
    asm.bytes(&[0xA9, 0x00]);
    sta_long(&mut asm, CREATOR_SESSION_ERROR_CODE_WRAM);
    asm.bytes(&[0xA9, CREATOR_ACTION_CANCEL]);
    asm.branch_long("store_action");
    asm.label("action_commit_noop");
    asm.bytes(&[0xA9, 0x00]);
    asm.label("store_action");
    sta_long(&mut asm, CREATOR_MODE_ACTION_WRAM);
    asm.bytes(&[0xA9, 0x01]); // LDA #$01
    sta_long(&mut asm, CREATOR_MODE_DIRTY_WRAM);
    asm.branch_long("check_b");

    // B button exits creator mode.
    asm.label("check_b");
    lda_long(&mut asm, CREATOR_MODE_INPUT_LOW_WRAM);
    asm.bytes(&[0x29, 0x01]); // AND #B
    asm.branch(0xF0, "update_render_contract"); // BEQ
    asm.bytes(&[0xA9, CREATOR_ACTION_EXIT]); // LDA #$FF
    sta_long(&mut asm, CREATOR_MODE_ACTION_WRAM);
    asm.bytes(&[0xA9, 0x00]); // LDA #$00
    sta_long(&mut asm, CREATOR_MODE_FLAG_WRAM);
    sta_long(&mut asm, CREATOR_RENDER_VISIBLE_WRAM);
    sta_long(&mut asm, CREATOR_RENDER_PAGE_WRAM);
    sta_long(&mut asm, CREATOR_RENDER_CURSOR_WRAM);
    sta_long(&mut asm, CREATOR_RENDER_ROW0_WRAM);
    sta_long(&mut asm, CREATOR_RENDER_ROW1_WRAM);
    sta_long(&mut asm, CREATOR_RENDER_ROW2_WRAM);
    sta_long(&mut asm, CREATOR_RENDER_ROW3_WRAM);
    sta_long(&mut asm, CREATOR_MODE_DIRTY_WRAM);
    lda_long(&mut asm, CREATOR_RENDER_REVISION_WRAM);
    asm.byte(0x1A); // INC A
    sta_long(&mut asm, CREATOR_RENDER_REVISION_WRAM);
    asm.byte(0x28); // PLP
    emit_stub_tail(&mut asm, hook_plan);

    asm.label("update_render_contract");
    lda_long(&mut asm, CREATOR_MODE_DIRTY_WRAM);
    asm.branch(0xD0, "render_rows_begin"); // BNE
    asm.byte(0x28); // PLP
    emit_stub_tail(&mut asm, hook_plan);

    asm.label("render_rows_begin");

    asm.bytes(&[0xA9, 0x01]); // LDA #$01
    sta_long(&mut asm, CREATOR_RENDER_VISIBLE_WRAM);
    lda_long(&mut asm, CREATOR_MODE_PAGE_WRAM);
    sta_long(&mut asm, CREATOR_RENDER_PAGE_WRAM);
    lda_long(&mut asm, CREATOR_MODE_CURSOR_WRAM);
    sta_long(&mut asm, CREATOR_RENDER_CURSOR_WRAM);

    lda_long(&mut asm, CREATOR_MODE_PAGE_WRAM);
    asm.bytes(&[0xC9, 0x00]); // CMP #0
    asm.branch(0xF0, "render_rows_page0"); // BEQ
    asm.bytes(&[0xC9, 0x01]); // CMP #1
    asm.branch(0xF0, "render_rows_page1"); // BEQ
    asm.bytes(&[0xC9, 0x02]); // CMP #2
    asm.branch(0xF0, "render_rows_page2"); // BEQ
    asm.branch(0x80, "render_rows_page3"); // BRA

    asm.label("render_rows_page0");
    asm.bytes(&[0xA9, CREATOR_PAGE0_ROW0]);
    sta_long(&mut asm, CREATOR_RENDER_ROW0_WRAM);
    asm.bytes(&[0xA9, CREATOR_PAGE0_ROW1]);
    sta_long(&mut asm, CREATOR_RENDER_ROW1_WRAM);
    asm.bytes(&[0xA9, CREATOR_PAGE0_ROW2]);
    sta_long(&mut asm, CREATOR_RENDER_ROW2_WRAM);
    asm.bytes(&[0xA9, CREATOR_PAGE0_ROW3]);
    sta_long(&mut asm, CREATOR_RENDER_ROW3_WRAM);
    asm.branch(0x80, "render_rows_done"); // BRA

    asm.label("render_rows_page1");
    asm.bytes(&[0xA9, CREATOR_PAGE1_ROW0]);
    sta_long(&mut asm, CREATOR_RENDER_ROW0_WRAM);
    asm.bytes(&[0xA9, CREATOR_PAGE1_ROW1]);
    sta_long(&mut asm, CREATOR_RENDER_ROW1_WRAM);
    asm.bytes(&[0xA9, CREATOR_PAGE1_ROW2]);
    sta_long(&mut asm, CREATOR_RENDER_ROW2_WRAM);
    asm.bytes(&[0xA9, CREATOR_PAGE1_ROW3]);
    sta_long(&mut asm, CREATOR_RENDER_ROW3_WRAM);
    asm.branch(0x80, "render_rows_done"); // BRA

    asm.label("render_rows_page2");
    asm.bytes(&[0xA9, CREATOR_PAGE2_ROW0]);
    sta_long(&mut asm, CREATOR_RENDER_ROW0_WRAM);
    asm.bytes(&[0xA9, CREATOR_PAGE2_ROW1]);
    sta_long(&mut asm, CREATOR_RENDER_ROW1_WRAM);
    asm.bytes(&[0xA9, CREATOR_PAGE2_ROW2]);
    sta_long(&mut asm, CREATOR_RENDER_ROW2_WRAM);
    asm.bytes(&[0xA9, CREATOR_PAGE2_ROW3]);
    sta_long(&mut asm, CREATOR_RENDER_ROW3_WRAM);
    asm.branch(0x80, "render_rows_done"); // BRA

    asm.label("render_rows_page3");
    asm.bytes(&[0xA9, CREATOR_PAGE3_ROW0]);
    sta_long(&mut asm, CREATOR_RENDER_ROW0_WRAM);
    asm.bytes(&[0xA9, CREATOR_PAGE3_ROW1]);
    sta_long(&mut asm, CREATOR_RENDER_ROW1_WRAM);
    asm.bytes(&[0xA9, CREATOR_PAGE3_ROW2]);
    sta_long(&mut asm, CREATOR_RENDER_ROW2_WRAM);
    asm.bytes(&[0xA9, CREATOR_PAGE3_ROW3]);
    sta_long(&mut asm, CREATOR_RENDER_ROW3_WRAM);

    asm.label("render_rows_done");
    lda_long(&mut asm, CREATOR_RENDER_REVISION_WRAM);
    asm.byte(0x1A); // INC A
    sta_long(&mut asm, CREATOR_RENDER_REVISION_WRAM);
    asm.bytes(&[0xA9, 0x00]); // LDA #$00
    sta_long(&mut asm, CREATOR_MODE_DIRTY_WRAM);
    asm.byte(0x28); // PLP
    emit_stub_tail(&mut asm, hook_plan);

    // Signature bytes for quick scanner/debugger checks.
    asm.bytes(b"INGAME");
    asm.finalize()
}

fn build_hook_patch_plan(rom: &Rom, options: &ExpansionOptions) -> ExpansionResult<Option<HookPatchPlan>> {
    if !options.patch_editor_hook {
        return Ok(None);
    }

    let hook_pc = options
        .editor_hook_pc_offset
        .ok_or(ExpansionError::MissingHookOffset)?;

    let overwrite_len = determine_hook_overwrite_len(rom, hook_pc, options.editor_hook_overwrite_len)?;
    let overwritten_bytes = rom
        .read_bytes(hook_pc, overwrite_len)
        .map_err(|err| ExpansionError::Rom(err.to_string()))?
        .to_vec();

    Ok(Some(HookPatchPlan {
        hook_pc,
        overwrite_len,
        return_pc: hook_pc + overwrite_len,
        overwritten_bytes,
    }))
}

fn determine_hook_overwrite_len(
    rom: &Rom,
    hook_pc: usize,
    manual_len: Option<usize>,
) -> ExpansionResult<usize> {
    if let Some(manual_len) = manual_len {
        if !(MIN_HOOK_OVERWRITE_LEN..=MAX_HOOK_OVERWRITE_LEN).contains(&manual_len) {
            return Err(ExpansionError::InvalidHookOverwriteLen {
                min: MIN_HOOK_OVERWRITE_LEN,
                max: MAX_HOOK_OVERWRITE_LEN,
                actual: manual_len,
            });
        }
        validate_hook_instruction_span(rom, hook_pc, manual_len)?;
        return Ok(manual_len);
    }

    let mut cursor = hook_pc;
    let mut total_len = 0usize;

    while total_len < MIN_HOOK_OVERWRITE_LEN {
        let instruction = decode_hook_instruction(rom, cursor)?;
        validate_hook_instruction(cursor, &instruction)?;

        let size = instruction.size as usize;
        total_len += size;
        cursor += size;

        if total_len > MAX_HOOK_OVERWRITE_LEN {
            return Err(ExpansionError::InvalidHookOverwriteLen {
                min: MIN_HOOK_OVERWRITE_LEN,
                max: MAX_HOOK_OVERWRITE_LEN,
                actual: total_len,
            });
        }
    }

    Ok(total_len)
}

fn validate_hook_instruction_span(rom: &Rom, start_pc: usize, expected_len: usize) -> ExpansionResult<()> {
    let mut cursor = start_pc;
    let end_pc = start_pc + expected_len;

    while cursor < end_pc {
        let instruction = decode_hook_instruction(rom, cursor)?;
        validate_hook_instruction(cursor, &instruction)?;

        let size = instruction.size as usize;
        let next = cursor + size;
        if next > end_pc {
            return Err(ExpansionError::HookSplitInstruction { pc: cursor });
        }
        cursor = next;
    }

    if cursor != end_pc {
        return Err(ExpansionError::HookSplitInstruction { pc: cursor });
    }

    Ok(())
}

fn decode_hook_instruction(rom: &Rom, pc: usize) -> ExpansionResult<DisassembledInstruction> {
    let memory = RomMemoryView { rom };
    let disassembler = Disassembler::new();
    let address = SnesAddress::from_pc(pc as u32);

    disassembler
        .disassemble(address, &memory, true, true)
        .ok_or(ExpansionError::HookDecodeFailed { pc })
}

fn validate_hook_instruction(pc: usize, instruction: &DisassembledInstruction) -> ExpansionResult<()> {
    if instruction.size == 0 {
        return Err(ExpansionError::HookDecodeFailed { pc });
    }

    if instruction.is_branch || instruction.is_call || instruction.is_return {
        return Err(ExpansionError::UnsafeHookInstruction {
            pc,
            mnemonic: instruction.mnemonic.clone(),
        });
    }

    if is_ambiguous_immediate_mnemonic(&instruction.mnemonic)
        || matches!(instruction.mnemonic.as_str(), "REP" | "SEP" | "PER")
    {
        return Err(ExpansionError::UnsafeHookInstruction {
            pc,
            mnemonic: instruction.mnemonic.clone(),
        });
    }

    Ok(())
}

fn is_ambiguous_immediate_mnemonic(mnemonic: &str) -> bool {
    matches!(
        mnemonic,
        "ADC" | "AND" | "BIT" | "CMP" | "CPX" | "CPY" | "EOR" | "LDA" | "LDX" | "LDY" | "ORA" | "SBC"
    )
}

fn build_jml_instruction(target_pc: usize, rom: &Rom) -> [u8; 4] {
    let (bank, addr) = rom.pc_to_snes(target_pc);
    [0x5C, (addr & 0xFF) as u8, ((addr >> 8) as u8), bank]
}

fn pc_to_lorom(pc: usize) -> (u8, u16) {
    let bank = ((pc / 0x8000) | 0x80) as u8;
    let addr = ((pc % 0x8000) | 0x8000) as u16;
    (bank, addr)
}

fn align_up(value: usize, alignment: usize) -> usize {
    if alignment <= 1 {
        value
    } else {
        value.div_ceil(alignment) * alignment
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ExpansionError;

    fn build_rom(size: usize, fill: u8) -> Rom {
        Rom::new(vec![fill; size])
    }

    #[test]
    fn verify_hook_auto_len_with_nops() {
        let rom = build_rom(0x40_000, 0xEA); // NOP
        let candidate = verify_ingame_hook_site(&rom, 0x12000, None).expect("hook should verify");

        assert_eq!(candidate.hook_pc, 0x12000);
        assert_eq!(candidate.overwrite_len, 4);
        assert_eq!(candidate.return_pc, 0x12004);
        assert_eq!(candidate.preview_bytes.len(), 4);
    }

    #[test]
    fn verify_hook_rejects_branch_instruction() {
        let mut rom = build_rom(0x40_000, 0xEA);
        // BRA +1 (branch instruction is unsafe for trampoline overwrite)
        rom.write_bytes(0x13000, &[0x80, 0x01, 0xEA, 0xEA, 0xEA])
            .expect("write branch test bytes");

        let result = verify_ingame_hook_site(&rom, 0x13000, None);
        assert!(matches!(
            result,
            Err(ExpansionError::UnsafeHookInstruction { .. })
        ));
    }

    #[test]
    fn verify_hook_detects_manual_split_instruction_span() {
        let mut rom = build_rom(0x40_000, 0xEA);
        // Two 3-byte STA absolute instructions
        rom.write_bytes(0x14000, &[0x8D, 0x34, 0x12, 0x8D, 0x78, 0x56])
            .expect("write split-span test bytes");

        let result = verify_ingame_hook_site(&rom, 0x14000, Some(4));
        assert!(matches!(result, Err(ExpansionError::HookSplitInstruction { .. })));
    }

    #[test]
    fn verify_hook_rejects_invalid_manual_len() {
        let rom = build_rom(0x40_000, 0xEA);
        let result = verify_ingame_hook_site(&rom, 0x15000, Some(33));

        assert!(matches!(
            result,
            Err(ExpansionError::InvalidHookOverwriteLen { .. })
        ));
    }

    #[test]
    fn analyze_hook_sites_respects_limit_and_uniqueness() {
        let rom = build_rom(0x40_000, 0xEA);
        let candidates = analyze_ingame_hook_sites(&rom, 0x10000, 0x10100, 3);

        assert_eq!(candidates.len(), 3);
        assert_eq!(candidates[0].hook_pc, 0x10000);
        assert_eq!(candidates[1].hook_pc, 0x10001);
        assert_eq!(candidates[2].hook_pc, 0x10002);
    }

    #[test]
    fn editor_header_v2_contains_render_contract_pointers() {
        let layout = crate::types::ExpandedRosterLayout {
            boxer_count: 32,
            name_pointer_table_pc: 0x20000,
            name_long_pointer_table_pc: 0x20100,
            name_blob_pc: 0x20200,
            circuit_table_pc: 0x20300,
            unlock_table_pc: 0x20400,
            intro_table_pc: 0x20500,
        };
        let header = build_editor_header(&layout, 0x30000, None);

        assert_eq!(&header[..8], &EDITOR_HEADER_MAGIC);
        assert_eq!(header[8], EDITOR_HEADER_VERSION);
        assert_eq!(header.len(), EDITOR_HEADER_SIZE);
        assert!(header
            .windows(4)
            .any(|window| window == CREATOR_RENDER_VISIBLE_WRAM));
        assert!(header
            .windows(4)
            .any(|window| window == CREATOR_RENDER_PAGE_WRAM));
        assert!(header
            .windows(4)
            .any(|window| window == CREATOR_RENDER_CURSOR_WRAM));
        assert!(header
            .windows(4)
            .any(|window| window == CREATOR_RENDER_ROW0_WRAM));
        assert!(header
            .windows(4)
            .any(|window| window == CREATOR_RENDER_ROW1_WRAM));
        assert!(header
            .windows(4)
            .any(|window| window == CREATOR_RENDER_ROW2_WRAM));
        assert!(header
            .windows(4)
            .any(|window| window == CREATOR_RENDER_ROW3_WRAM));
        assert!(header
            .windows(4)
            .any(|window| window == CREATOR_RENDER_REVISION_WRAM));
    }

    #[test]
    fn editor_stub_contains_creator_combo_trigger_sequence() {
        let stub = build_editor_stub(None).expect("stub should build");

        // PHP, SEP #$20, LDA $00:4218
        assert!(stub
            .windows(7)
            .any(|window| window == [0x08, 0xE2, 0x20, 0xAF, 0x18, 0x42, 0x00]));
        // STA $7E1FFF marker write exists.
        assert!(stub
            .windows(4)
            .any(|window| window == [0x8F, 0xFF, 0x1F, 0x7E]));
        // Creator-mode dispatch heartbeat write exists.
        assert!(stub
            .windows(4)
            .any(|window| window == [0x8F, 0xFD, 0x1F, 0x7E]));
        // Creator input mirrors (low/high) exist.
        assert!(stub
            .windows(4)
            .any(|window| window == [0x8F, 0xFC, 0x1F, 0x7E]));
        assert!(stub
            .windows(4)
            .any(|window| window == [0x8F, 0xFB, 0x1F, 0x7E]));
        // Cursor/action state writes exist.
        assert!(stub
            .windows(4)
            .any(|window| window == [0x8F, 0xFA, 0x1F, 0x7E]));
        assert!(stub
            .windows(4)
            .any(|window| window == [0x8F, 0xF9, 0x1F, 0x7E]));
        // Render contract writes exist.
        assert!(stub
            .windows(4)
            .any(|window| window == [0x8F, 0xF6, 0x1F, 0x7E]));
        assert!(stub
            .windows(4)
            .any(|window| window == [0x8F, 0xF5, 0x1F, 0x7E]));
        assert!(stub
            .windows(4)
            .any(|window| window == [0x8F, 0xF4, 0x1F, 0x7E]));
        assert!(stub
            .windows(4)
            .any(|window| window == [0x8F, 0xF3, 0x1F, 0x7E]));
        assert!(stub
            .windows(4)
            .any(|window| window == [0x8F, 0xF2, 0x1F, 0x7E]));
        assert!(stub
            .windows(4)
            .any(|window| window == [0x8F, 0xF1, 0x1F, 0x7E]));
        assert!(stub
            .windows(4)
            .any(|window| window == [0x8F, 0xF0, 0x1F, 0x7E]));
        assert!(stub
            .windows(4)
            .any(|window| window == [0x8F, 0xEF, 0x1F, 0x7E]));
        assert!(stub
            .windows(4)
            .any(|window| window == [0x8F, 0xEC, 0x1F, 0x7E]));
        assert!(stub
            .windows(4)
            .any(|window| window == [0x8F, 0xEB, 0x1F, 0x7E]));
        assert!(stub
            .windows(4)
            .any(|window| window == [0x8F, 0xE9, 0x1F, 0x7E]));
        assert!(stub
            .windows(4)
            .any(|window| window == [0x8F, 0xE8, 0x1F, 0x7E]));
        assert!(stub
            .windows(4)
            .any(|window| window == [0x8F, 0xE7, 0x1F, 0x7E]));
        assert!(stub
            .windows(4)
            .any(|window| window == [0x8F, 0xE6, 0x1F, 0x7E]));
        assert!(stub
            .windows(4)
            .any(|window| window == [0x8F, 0xE4, 0x1F, 0x7E]));
        assert!(stub
            .windows(4)
            .any(|window| window == [0x8F, 0xE3, 0x1F, 0x7E]));
        assert!(stub
            .windows(4)
            .any(|window| window == [0x9F, 0xC0, 0x1F, 0x7E]));
        assert!(stub
            .windows(4)
            .any(|window| window == [0x9F, 0xD0, 0x1F, 0x7E]));
        assert!(stub
            .windows(2)
            .any(|window| window == [0xA9, CREATOR_ACTION_CANCEL]));
        // Signature remains present.
        assert!(stub.ends_with(b"INGAME"));
    }
}
