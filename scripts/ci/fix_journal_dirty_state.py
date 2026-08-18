#!/usr/bin/env python3
"""Make journal dirty state compare the exact saved transaction prefix, not a monotonic counter."""
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
PATH = ROOT / "crates/rom-core/src/edit.rs"
text = PATH.read_text(encoding="utf-8")

old = '''    state_revision: u64,
    saved_revision: u64,
}'''
new = '''    state_revision: u64,
    saved_revision: u64,
    #[serde(default)]
    saved_transaction_ids: Vec<u64>,
}'''
if old in text:
    text = text.replace(old, new, 1)
elif "saved_transaction_ids" not in text:
    raise SystemExit("EditJournal field shape changed")

old = '''            state_revision: 0,
            saved_revision: 0,
        }'''
new = '''            state_revision: 0,
            saved_revision: 0,
            saved_transaction_ids: Vec::new(),
        }'''
if old in text:
    text = text.replace(old, new, 1)

old = '''    pub fn is_dirty(&self) -> bool {
        self.state_revision != self.saved_revision
    }'''
new = '''    pub fn is_dirty(&self) -> bool {
        let current_ids: Vec<u64> = self
            .active_transactions()
            .iter()
            .map(|transaction| transaction.id)
            .collect();
        current_ids != self.saved_transaction_ids
    }'''
if old in text:
    text = text.replace(old, new, 1)
elif "current_ids != self.saved_transaction_ids" not in text:
    raise SystemExit("EditJournal dirty-state method changed")

old = '''    pub fn mark_saved(&mut self) {
        self.saved_revision = self.state_revision;
    }'''
new = '''    pub fn mark_saved(&mut self) {
        self.saved_revision = self.state_revision;
        self.saved_transaction_ids = self
            .active_transactions()
            .iter()
            .map(|transaction| transaction.id)
            .collect();
    }'''
if old in text:
    text = text.replace(old, new, 1)
elif "self.saved_transaction_ids = self" not in text:
    raise SystemExit("EditJournal mark_saved method changed")

marker = '''    #[test]
    fn serialized_journal_round_trips_and_validates() {'''
test = '''    #[test]
    fn redo_back_to_exact_saved_prefix_is_clean() {
        let base = base();
        let mut journal = EditJournal::new();
        journal.commit(&base, "one", vec![write(1, &[9])]).unwrap();
        journal.mark_saved();
        assert!(!journal.is_dirty());
        journal.undo();
        assert!(journal.is_dirty());
        journal.redo();
        assert!(!journal.is_dirty());
        journal.undo();
        journal.commit(&base, "replacement", vec![write(2, &[8])]).unwrap();
        assert!(journal.is_dirty());
    }

'''
if "redo_back_to_exact_saved_prefix_is_clean" not in text:
    if marker not in text:
        raise SystemExit("EditJournal test insertion marker changed")
    text = text.replace(marker, test + marker, 1)

PATH.write_text(text, encoding="utf-8")
print("Journal dirty-state semantics fixed.")
