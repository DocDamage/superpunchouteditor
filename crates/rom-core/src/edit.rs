//! Canonical ROM edit journal and deterministic materialization.
//!
//! This module deliberately has no Tauri/UI dependencies. A loaded ROM is represented by an
//! immutable [`BaseRom`]; every supported mutation is appended to an [`EditJournal`]. The current
//! working image is always reconstructed from the base plus the active journal prefix.

use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use std::ops::Range;
use thiserror::Error;

use crate::{Rom, RomRegion};

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum EditError {
    #[error("range arithmetic overflow at offset {offset} with length {length}")]
    RangeOverflow { offset: usize, length: usize },
    #[error("range {offset}..{end} is outside ROM length {rom_len}")]
    RangeOutOfBounds {
        offset: usize,
        end: usize,
        rom_len: usize,
    },
    #[error("transaction contains overlapping writes: {first_start}..{first_end} and {second_start}..{second_end}")]
    OverlappingWrites {
        first_start: usize,
        first_end: usize,
        second_start: usize,
        second_end: usize,
    },
    #[error("journal before-bytes do not match the materialized ROM at offset {offset}")]
    BeforeBytesMismatch { offset: usize },
    #[error("resize expected ROM length {expected}, found {actual}")]
    ResizeBaseMismatch { expected: usize, actual: usize },
    #[error("shrinking a ROM through the edit journal is not supported ({from} -> {to})")]
    ShrinkUnsupported { from: usize, to: usize },
    #[error("transaction contains no changes")]
    EmptyTransaction,
    #[error("edit does not change the working ROM")]
    NoChange,
}

/// Validate a byte range without unchecked `offset + length` or `len - offset` arithmetic.
pub fn validate_range(
    offset: usize,
    length: usize,
    rom_len: usize,
) -> Result<Range<usize>, EditError> {
    let end = offset
        .checked_add(length)
        .ok_or(EditError::RangeOverflow { offset, length })?;
    if end > rom_len {
        return Err(EditError::RangeOutOfBounds {
            offset,
            end,
            rom_len,
        });
    }
    Ok(offset..end)
}

fn sha1_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha1::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

/// Immutable source identity for one editing session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BaseRom {
    bytes: Vec<u8>,
    sha1: String,
    region: Option<RomRegion>,
}

impl BaseRom {
    pub fn from_rom(rom: &Rom) -> Self {
        Self {
            bytes: rom.data.clone(),
            sha1: rom.calculate_sha1(),
            region: rom.detect_region(),
        }
    }

    pub fn from_bytes(bytes: Vec<u8>, region: Option<RomRegion>) -> Self {
        let sha1 = sha1_hex(&bytes);
        Self {
            bytes,
            sha1,
            region,
        }
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn sha1(&self) -> &str {
        &self.sha1
    }

    pub fn region(&self) -> Option<RomRegion> {
        self.region
    }

    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EditOperation {
    WriteBytes {
        offset: usize,
        before: Vec<u8>,
        after: Vec<u8>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        asset_id: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        description: Option<String>,
    },
    ResizeRom {
        before_len: usize,
        after_len: usize,
        fill_byte: u8,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        description: Option<String>,
    },
}

impl EditOperation {
    pub fn write_range(&self) -> Option<Range<usize>> {
        match self {
            Self::WriteBytes { offset, after, .. } => {
                offset.checked_add(after.len()).map(|end| *offset..end)
            }
            Self::ResizeRom { .. } => None,
        }
    }
}

/// Request used while building a transaction. `before` bytes are captured by the backend from the
/// current materialized revision and are never trusted from the frontend.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditRequest {
    WriteBytes {
        offset: usize,
        after: Vec<u8>,
        asset_id: Option<String>,
        description: Option<String>,
    },
    ResizeRom {
        after_len: usize,
        fill_byte: u8,
        description: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EditTransaction {
    pub id: u64,
    pub label: String,
    pub operations: Vec<EditOperation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EditJournal {
    transactions: Vec<EditTransaction>,
    cursor: usize,
    next_id: u64,
    state_revision: u64,
    saved_revision: u64,
}

impl Default for EditJournal {
    fn default() -> Self {
        Self::new()
    }
}

impl EditJournal {
    pub fn new() -> Self {
        Self {
            transactions: Vec::new(),
            cursor: 0,
            next_id: 1,
            state_revision: 0,
            saved_revision: 0,
        }
    }

    pub fn transactions(&self) -> &[EditTransaction] {
        &self.transactions
    }

    pub fn active_transactions(&self) -> &[EditTransaction] {
        &self.transactions[..self.cursor]
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn revision(&self) -> u64 {
        self.state_revision
    }

    pub fn saved_revision(&self) -> u64 {
        self.saved_revision
    }

    pub fn is_dirty(&self) -> bool {
        self.state_revision != self.saved_revision
    }

    pub fn can_undo(&self) -> bool {
        self.cursor > 0
    }

    pub fn can_redo(&self) -> bool {
        self.cursor < self.transactions.len()
    }

    pub fn mark_saved(&mut self) {
        self.saved_revision = self.state_revision;
    }

    pub fn clear(&mut self) {
        *self = Self::new();
    }

    pub fn undo(&mut self) -> Option<&EditTransaction> {
        if self.cursor == 0 {
            return None;
        }
        self.cursor -= 1;
        self.state_revision = self.state_revision.saturating_add(1);
        self.transactions.get(self.cursor)
    }

    pub fn redo(&mut self) -> Option<&EditTransaction> {
        if self.cursor >= self.transactions.len() {
            return None;
        }
        let index = self.cursor;
        self.cursor += 1;
        self.state_revision = self.state_revision.saturating_add(1);
        self.transactions.get(index)
    }

    pub fn materialize(&self, base: &BaseRom) -> Result<Vec<u8>, EditError> {
        let mut bytes = base.bytes().to_vec();
        for transaction in self.active_transactions() {
            apply_transaction(&mut bytes, transaction)?;
        }
        Ok(bytes)
    }

    pub fn commit(
        &mut self,
        base: &BaseRom,
        label: impl Into<String>,
        requests: Vec<EditRequest>,
    ) -> Result<&EditTransaction, EditError> {
        if requests.is_empty() {
            return Err(EditError::EmptyTransaction);
        }

        let mut scratch = self.materialize(base)?;
        let mut operations = Vec::with_capacity(requests.len());
        let mut write_ranges: Vec<Range<usize>> = Vec::new();

        for request in requests {
            match request {
                EditRequest::WriteBytes {
                    offset,
                    after,
                    asset_id,
                    description,
                } => {
                    let range = validate_range(offset, after.len(), scratch.len())?;
                    for previous in &write_ranges {
                        if ranges_overlap(previous, &range) {
                            return Err(EditError::OverlappingWrites {
                                first_start: previous.start,
                                first_end: previous.end,
                                second_start: range.start,
                                second_end: range.end,
                            });
                        }
                    }
                    let before = scratch[range.clone()].to_vec();
                    if before == after {
                        return Err(EditError::NoChange);
                    }
                    scratch[range.clone()].copy_from_slice(&after);
                    write_ranges.push(range);
                    operations.push(EditOperation::WriteBytes {
                        offset,
                        before,
                        after,
                        asset_id,
                        description,
                    });
                }
                EditRequest::ResizeRom {
                    after_len,
                    fill_byte,
                    description,
                } => {
                    let before_len = scratch.len();
                    if after_len < before_len {
                        return Err(EditError::ShrinkUnsupported {
                            from: before_len,
                            to: after_len,
                        });
                    }
                    if after_len == before_len {
                        return Err(EditError::NoChange);
                    }
                    scratch.resize(after_len, fill_byte);
                    operations.push(EditOperation::ResizeRom {
                        before_len,
                        after_len,
                        fill_byte,
                        description,
                    });
                }
            }
        }

        if self.cursor < self.transactions.len() {
            self.transactions.truncate(self.cursor);
        }

        let transaction = EditTransaction {
            id: self.next_id,
            label: label.into(),
            operations,
        };
        self.next_id = self.next_id.saturating_add(1);
        self.transactions.push(transaction);
        self.cursor = self.transactions.len();
        self.state_revision = self.state_revision.saturating_add(1);

        Ok(self
            .transactions
            .last()
            .expect("a committed transaction was just pushed"))
    }

    /// Load a serialized journal and validate its complete active history before it is accepted.
    pub fn validate_against(&self, base: &BaseRom) -> Result<(), EditError> {
        if self.cursor > self.transactions.len() {
            return Err(EditError::EmptyTransaction);
        }
        self.materialize(base).map(|_| ())
    }
}

fn ranges_overlap(a: &Range<usize>, b: &Range<usize>) -> bool {
    a.start < b.end && b.start < a.end
}

fn apply_transaction(bytes: &mut Vec<u8>, transaction: &EditTransaction) -> Result<(), EditError> {
    for operation in &transaction.operations {
        match operation {
            EditOperation::WriteBytes {
                offset,
                before,
                after,
                ..
            } => {
                if before.len() != after.len() {
                    return Err(EditError::RangeOutOfBounds {
                        offset: *offset,
                        end: offset.saturating_add(after.len()),
                        rom_len: bytes.len(),
                    });
                }
                let range = validate_range(*offset, after.len(), bytes.len())?;
                if bytes[range.clone()] != *before {
                    return Err(EditError::BeforeBytesMismatch { offset: *offset });
                }
                bytes[range].copy_from_slice(after);
            }
            EditOperation::ResizeRom {
                before_len,
                after_len,
                fill_byte,
                ..
            } => {
                if bytes.len() != *before_len {
                    return Err(EditError::ResizeBaseMismatch {
                        expected: *before_len,
                        actual: bytes.len(),
                    });
                }
                if after_len < before_len {
                    return Err(EditError::ShrinkUnsupported {
                        from: *before_len,
                        to: *after_len,
                    });
                }
                bytes.resize(*after_len, *fill_byte);
            }
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChangeRange {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MaterializedRom {
    pub bytes: Vec<u8>,
    pub base_sha1: String,
    pub current_sha1: String,
    pub revision: u64,
    pub region: Option<RomRegion>,
    pub change_ranges: Vec<ChangeRange>,
    pub changed_byte_count: usize,
    pub transaction_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EditStateProjection {
    pub revision: u64,
    pub saved_revision: u64,
    pub dirty: bool,
    pub can_undo: bool,
    pub can_redo: bool,
    pub transaction_count: usize,
    pub active_transaction_count: usize,
    pub current_sha1: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RomSession {
    base: BaseRom,
    journal: EditJournal,
    source_path: Option<String>,
}

impl RomSession {
    pub fn new(base: BaseRom, source_path: Option<String>) -> Self {
        Self {
            base,
            journal: EditJournal::new(),
            source_path,
        }
    }

    pub fn from_rom(rom: &Rom, source_path: Option<String>) -> Self {
        Self::new(BaseRom::from_rom(rom), source_path)
    }

    pub fn base(&self) -> &BaseRom {
        &self.base
    }

    pub fn journal(&self) -> &EditJournal {
        &self.journal
    }

    pub fn journal_mut(&mut self) -> &mut EditJournal {
        &mut self.journal
    }

    pub fn source_path(&self) -> Option<&str> {
        self.source_path.as_deref()
    }

    pub fn replace_journal(&mut self, journal: EditJournal) -> Result<(), EditError> {
        journal.validate_against(&self.base)?;
        self.journal = journal;
        Ok(())
    }

    pub fn commit(
        &mut self,
        label: impl Into<String>,
        requests: Vec<EditRequest>,
    ) -> Result<EditStateProjection, EditError> {
        self.journal.commit(&self.base, label, requests)?;
        self.state_projection()
    }

    pub fn materialize(&self) -> Result<MaterializedRom, EditError> {
        let bytes = self.journal.materialize(&self.base)?;
        let current_sha1 = sha1_hex(&bytes);
        let (change_ranges, changed_byte_count) = diff_summary(self.base.bytes(), &bytes);
        Ok(MaterializedRom {
            bytes,
            base_sha1: self.base.sha1().to_string(),
            current_sha1,
            revision: self.journal.revision(),
            region: self.base.region(),
            change_ranges,
            changed_byte_count,
            transaction_count: self.journal.active_transactions().len(),
        })
    }

    pub fn state_projection(&self) -> Result<EditStateProjection, EditError> {
        let materialized = self.materialize()?;
        Ok(EditStateProjection {
            revision: self.journal.revision(),
            saved_revision: self.journal.saved_revision(),
            dirty: self.journal.is_dirty(),
            can_undo: self.journal.can_undo(),
            can_redo: self.journal.can_redo(),
            transaction_count: self.journal.transactions().len(),
            active_transaction_count: self.journal.active_transactions().len(),
            current_sha1: materialized.current_sha1,
        })
    }

    pub fn undo(&mut self) -> Result<Option<EditStateProjection>, EditError> {
        if self.journal.undo().is_none() {
            return Ok(None);
        }
        self.state_projection().map(Some)
    }

    pub fn redo(&mut self) -> Result<Option<EditStateProjection>, EditError> {
        if self.journal.redo().is_none() {
            return Ok(None);
        }
        self.state_projection().map(Some)
    }

    pub fn mark_saved(&mut self) {
        self.journal.mark_saved();
    }
}

fn diff_summary(base: &[u8], current: &[u8]) -> (Vec<ChangeRange>, usize) {
    let max_len = base.len().max(current.len());
    let mut ranges = Vec::new();
    let mut changed = 0usize;
    let mut range_start: Option<usize> = None;

    for index in 0..max_len {
        let differs = base.get(index) != current.get(index);
        if differs {
            changed = changed.saturating_add(1);
            if range_start.is_none() {
                range_start = Some(index);
            }
        } else if let Some(start) = range_start.take() {
            ranges.push(ChangeRange { start, end: index });
        }
    }
    if let Some(start) = range_start {
        ranges.push(ChangeRange {
            start,
            end: max_len,
        });
    }

    (ranges, changed)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> BaseRom {
        BaseRom::from_bytes(vec![0, 1, 2, 3, 4, 5, 6, 7], None)
    }

    fn write(offset: usize, after: &[u8]) -> EditRequest {
        EditRequest::WriteBytes {
            offset,
            after: after.to_vec(),
            asset_id: None,
            description: None,
        }
    }

    #[test]
    fn checked_range_rejects_overflow_and_out_of_bounds() {
        assert!(matches!(
            validate_range(usize::MAX, 2, 8),
            Err(EditError::RangeOverflow { .. })
        ));
        assert!(matches!(
            validate_range(7, 2, 8),
            Err(EditError::RangeOutOfBounds { .. })
        ));
        assert_eq!(validate_range(8, 0, 8).unwrap(), 8..8);
    }

    #[test]
    fn journal_materialization_is_deterministic() {
        let base = base();
        let mut journal = EditJournal::new();
        journal.commit(&base, "edit", vec![write(2, &[9, 9])]).unwrap();
        let first = journal.materialize(&base).unwrap();
        let second = journal.materialize(&base).unwrap();
        assert_eq!(first, second);
        assert_eq!(first, vec![0, 1, 9, 9, 4, 5, 6, 7]);
    }

    #[test]
    fn undo_redo_and_new_commit_manage_cursor_correctly() {
        let base = base();
        let mut journal = EditJournal::new();
        journal.commit(&base, "one", vec![write(1, &[9])]).unwrap();
        journal.commit(&base, "two", vec![write(2, &[8])]).unwrap();
        assert_eq!(journal.materialize(&base).unwrap()[1..3], [9, 8]);

        journal.undo();
        assert_eq!(journal.materialize(&base).unwrap()[1..3], [9, 2]);
        assert!(journal.can_redo());

        journal.commit(&base, "replacement", vec![write(3, &[7])]).unwrap();
        assert!(!journal.can_redo());
        assert_eq!(journal.transactions().len(), 2);
    }

    #[test]
    fn overlapping_writes_in_one_transaction_are_rejected_atomically() {
        let base = base();
        let mut journal = EditJournal::new();
        let result = journal.commit(
            &base,
            "overlap",
            vec![write(1, &[8, 8]), write(2, &[9, 9])],
        );
        assert!(matches!(result, Err(EditError::OverlappingWrites { .. })));
        assert!(journal.transactions().is_empty());
        assert_eq!(journal.materialize(&base).unwrap(), base.bytes());
    }

    #[test]
    fn resize_then_write_materializes_expanded_rom() {
        let base = base();
        let mut journal = EditJournal::new();
        journal
            .commit(
                &base,
                "expand",
                vec![
                    EditRequest::ResizeRom {
                        after_len: 12,
                        fill_byte: 0xff,
                        description: None,
                    },
                    write(10, &[0xaa, 0xbb]),
                ],
            )
            .unwrap();
        let bytes = journal.materialize(&base).unwrap();
        assert_eq!(bytes.len(), 12);
        assert_eq!(&bytes[8..], &[0xff, 0xff, 0xaa, 0xbb]);
    }

    #[test]
    fn serialized_journal_round_trips_and_validates() {
        let base = base();
        let mut journal = EditJournal::new();
        journal.commit(&base, "edit", vec![write(4, &[1, 1])]).unwrap();
        let json = serde_json::to_string(&journal).unwrap();
        let restored: EditJournal = serde_json::from_str(&json).unwrap();
        restored.validate_against(&base).unwrap();
        assert_eq!(restored.materialize(&base).unwrap(), journal.materialize(&base).unwrap());
    }
}
