//! Property tests: random valid transactions and document operations must
//! satisfy the core invariant `undo(apply(t, doc)) == doc`.

use std::ops::Range;

use editor_core::{Bias, Document, Rope, Selection, SelectionSet, Transaction};
use proptest::prelude::*;

/// Text mixing ASCII, Latin diacritics, CJK, ZWJ emoji and newlines.
fn arb_text() -> impl Strategy<Value = String> {
    prop::collection::vec(
        prop_oneof![
            Just("a"),
            Just("b"),
            Just("ç"),
            Just("ã"),
            Just("é"),
            Just("日"),
            Just("語"),
            Just("👨‍👩‍👧‍👦"),
            Just("🦀"),
            Just(" "),
            Just("\n"),
        ],
        0..32,
    )
    .prop_map(|parts| parts.concat())
}

fn arb_small_text() -> impl Strategy<Value = String> {
    prop::collection::vec(
        prop_oneof![
            Just("x"),
            Just("ç"),
            Just("日"),
            Just("👨‍👩‍👧‍👦"),
            Just(" "),
            Just("\n")
        ],
        0..4,
    )
    .prop_map(|parts| parts.concat())
}

/// Like [`arb_small_text`] but never empty, so every insert commits a step.
fn arb_nonempty_text() -> impl Strategy<Value = String> {
    prop::collection::vec(
        prop_oneof![
            Just("x"),
            Just("ç"),
            Just("日"),
            Just("👨‍👩‍👧‍👦"),
            Just(" "),
            Just("\n")
        ],
        1..4,
    )
    .prop_map(|parts| parts.concat())
}

type EditSeed = (usize, usize, String);

fn arb_seeds(max_edits: usize) -> impl Strategy<Value = Vec<EditSeed>> {
    prop::collection::vec(
        (any::<usize>(), any::<usize>(), arb_small_text()),
        0..max_edits,
    )
}

/// Turn seeds into a valid transaction for the current rope: ranges snapped
/// to char boundaries, sorted, overlaps dropped.
fn materialize(rope: &Rope, seeds: &[EditSeed]) -> Transaction {
    let n_chars = rope.len_chars();
    let mut changes: Vec<(Range<usize>, String)> = seeds
        .iter()
        .map(|(a, b, replacement)| {
            let c1 = a % (n_chars + 1);
            let c2 = b % (n_chars + 1);
            let (lo, hi) = if c1 <= c2 { (c1, c2) } else { (c2, c1) };
            (
                rope.char_to_byte(lo)..rope.char_to_byte(hi),
                replacement.clone(),
            )
        })
        .collect();
    changes.sort_by_key(|(range, _)| range.start);
    let mut filtered: Vec<(Range<usize>, String)> = Vec::new();
    for (range, replacement) in changes {
        let valid = filtered
            .last()
            .is_none_or(|(prev, _)| range.start >= prev.end && range.start > prev.start);
        if valid {
            filtered.push((range, replacement));
        }
    }
    Transaction::change(filtered)
}

#[derive(Debug, Clone)]
enum DocOp {
    Insert(String),
    DeleteBackward,
    DeleteForward,
    MoveTo(usize),
    AddCursor(usize),
    Undo,
}

fn arb_doc_op() -> impl Strategy<Value = DocOp> {
    prop_oneof![
        arb_small_text().prop_map(DocOp::Insert),
        Just(DocOp::DeleteBackward),
        Just(DocOp::DeleteForward),
        any::<usize>().prop_map(DocOp::MoveTo),
        any::<usize>().prop_map(DocOp::AddCursor),
        Just(DocOp::Undo),
    ]
}

fn boundary(rope: &Rope, seed: usize) -> usize {
    rope.char_to_byte(seed % (rope.len_chars() + 1))
}

proptest! {
    /// Applying a sequence of transactions and then their inverses in
    /// reverse order restores the original document.
    #[test]
    fn apply_then_undo_restores_original(
        text in arb_text(),
        steps in prop::collection::vec(arb_seeds(4), 0..8),
    ) {
        let mut rope = Rope::from_str(&text);
        let original = rope.to_string();
        let mut inverses = Vec::new();
        for seeds in &steps {
            let tx = materialize(&rope, seeds);
            inverses.push(tx.invert(&rope));
            tx.apply(&mut rope);
        }
        for inverse in inverses.iter().rev() {
            inverse.apply(&mut rope);
        }
        prop_assert_eq!(rope.to_string(), original);
    }

    /// `compose(t1, t2)` applied to the original equals applying t1 then t2.
    #[test]
    fn compose_matches_sequential_application(
        text in arb_text(),
        s1 in arb_seeds(4),
        s2 in arb_seeds(4),
    ) {
        let rope0 = Rope::from_str(&text);
        let t1 = materialize(&rope0, &s1);
        let mut rope1 = rope0.clone();
        t1.apply(&mut rope1);
        let t2 = materialize(&rope1, &s2);
        let mut rope2 = rope1.clone();
        t2.apply(&mut rope2);

        let mut composed_rope = rope0.clone();
        t1.compose(t2).apply(&mut composed_rope);
        prop_assert_eq!(composed_rope.to_string(), rope2.to_string());
    }

    /// Mapped positions stay inside the new document, on char boundaries,
    /// and Bias::Before never lands after Bias::After.
    #[test]
    fn map_pos_lands_on_char_boundaries(
        text in arb_text(),
        seeds in arb_seeds(4),
        pos_seed in any::<usize>(),
    ) {
        let rope = Rope::from_str(&text);
        let tx = materialize(&rope, &seeds);
        let pos = boundary(&rope, pos_seed);
        let mut new_rope = rope.clone();
        tx.apply(&mut new_rope);
        let new_text = new_rope.to_string();
        let before = tx.map_pos(pos, Bias::Before);
        let after = tx.map_pos(pos, Bias::After);
        prop_assert!(before <= after);
        for mapped in [before, after] {
            prop_assert!(mapped <= new_text.len());
            prop_assert!(new_text.is_char_boundary(mapped));
        }
    }

    /// Random multi-cursor document operations, then undoing everything,
    /// restores the original text.
    #[test]
    fn document_ops_then_full_undo_restores_original(
        text in arb_text(),
        ops in prop::collection::vec(arb_doc_op(), 0..12),
    ) {
        let mut doc = Document::load(&text);
        let original = doc.to_string();
        for op in ops {
            match op {
                DocOp::Insert(s) => doc.insert(&s),
                DocOp::DeleteBackward => doc.delete_backward(),
                DocOp::DeleteForward => doc.delete_forward(),
                DocOp::MoveTo(seed) => {
                    let pos = boundary(doc.rope(), seed);
                    doc.set_selections(SelectionSet::caret(pos));
                }
                DocOp::AddCursor(seed) => {
                    let pos = boundary(doc.rope(), seed);
                    let mut sels = doc.selections().clone();
                    sels.add_selection(Selection::caret(pos));
                    doc.set_selections(sels);
                }
                DocOp::Undo => {
                    doc.undo();
                }
            }
        }
        while doc.undo() {}
        prop_assert_eq!(doc.to_string(), original);
    }

    /// Redoing everything after a full undo restores the final state.
    #[test]
    fn full_undo_then_full_redo_restores_final_state(
        text in arb_text(),
        inserts in prop::collection::vec(arb_nonempty_text(), 0..6),
    ) {
        let mut doc = Document::load(&text);
        for (i, s) in inserts.iter().enumerate() {
            let pos = boundary(doc.rope(), i * 7919);
            doc.set_selections(SelectionSet::caret(pos));
            doc.insert(s);
        }
        let final_text = doc.to_string();
        let final_selections = doc.selections().clone();
        while doc.undo() {}
        prop_assert_eq!(doc.to_string(), text.replace("\r\n", "\n"));
        while doc.redo() {}
        prop_assert_eq!(doc.to_string(), final_text);
        prop_assert_eq!(doc.selections(), &final_selections);
    }
}
