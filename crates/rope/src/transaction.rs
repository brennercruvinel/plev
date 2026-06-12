//! Transactions: ordered, non-overlapping edits applied atomically.

use std::ops::Range;

use ropey::Rope;

/// Which side a position sticks to when it lies exactly on an edit boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bias {
    /// The position stays before text inserted at it.
    Before,
    /// The position moves past text inserted at it.
    After,
}

/// A single replacement: `range` (byte offsets into the document as it was
/// *before* the transaction) is removed and `replacement` takes its place.
/// A pure insertion has an empty range; a pure deletion an empty replacement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edit {
    pub range: Range<usize>,
    pub replacement: String,
}

/// An atomic document change: a list of edits sorted by start position,
/// pairwise non-overlapping, all addressed in pre-transaction coordinates.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Transaction {
    edits: Vec<Edit>,
}

/// Internal op-based representation (Helix-style change set), used to
/// compose two transactions without access to the document text.
#[derive(Debug)]
enum Op {
    Retain(usize),
    Delete(usize),
    Insert(String),
}

impl Transaction {
    pub fn new() -> Self {
        Self::default()
    }

    /// Build a transaction from `(range, replacement)` pairs. Edits are
    /// sorted by start; no-op edits are dropped. Overlapping edits, or two
    /// distinct edits starting at the same position, are a programming
    /// error and panic.
    pub fn change<S: Into<String>>(changes: impl IntoIterator<Item = (Range<usize>, S)>) -> Self {
        let mut edits: Vec<Edit> = changes
            .into_iter()
            .map(|(range, replacement)| Edit {
                range,
                replacement: replacement.into(),
            })
            .filter(|e| !e.range.is_empty() || !e.replacement.is_empty())
            .collect();
        edits.sort_by_key(|e| e.range.start);
        for pair in edits.windows(2) {
            assert!(
                pair[0].range.end <= pair[1].range.start
                    && pair[0].range.start != pair[1].range.start,
                "transaction edits must be ordered and non-overlapping: {:?} / {:?}",
                pair[0].range,
                pair[1].range,
            );
        }
        Self { edits }
    }

    /// Insert `text` at `pos`.
    pub fn insert(pos: usize, text: &str) -> Self {
        Self::change([(pos..pos, text)])
    }

    /// Delete the byte range.
    pub fn delete(range: Range<usize>) -> Self {
        Self::change([(range, "")])
    }

    pub fn edits(&self) -> &[Edit] {
        &self.edits
    }

    pub fn is_empty(&self) -> bool {
        self.edits.is_empty()
    }

    /// Apply all edits to the rope. Edit ranges must lie on char boundaries.
    pub fn apply(&self, rope: &mut Rope) {
        // Apply back to front so earlier ranges stay valid.
        for edit in self.edits.iter().rev() {
            let start = rope.byte_to_char(edit.range.start);
            let end = rope.byte_to_char(edit.range.end);
            debug_assert_eq!(
                rope.char_to_byte(start),
                edit.range.start,
                "start off boundary"
            );
            debug_assert_eq!(rope.char_to_byte(end), edit.range.end, "end off boundary");
            rope.remove(start..end);
            rope.insert(start, &edit.replacement);
        }
    }

    /// The transaction that undoes `self`. `original` must be the document
    /// as it was *before* `self` was applied. The returned transaction is
    /// addressed in post-`self` coordinates.
    pub fn invert(&self, original: &Rope) -> Transaction {
        let mut delta = 0isize;
        let mut edits = Vec::with_capacity(self.edits.len());
        for edit in &self.edits {
            let start = (edit.range.start as isize + delta) as usize;
            let removed = original.byte_slice(edit.range.clone()).to_string();
            edits.push(Edit {
                range: start..start + edit.replacement.len(),
                replacement: removed,
            });
            delta += edit.replacement.len() as isize - edit.range.len() as isize;
        }
        Transaction { edits }
    }

    /// Map a byte position from pre- to post-transaction coordinates.
    /// `bias` decides which side the position sticks to when it falls
    /// exactly on an edit boundary (or inside a replaced range).
    pub fn map_pos(&self, pos: usize, bias: Bias) -> usize {
        let mut delta = 0isize;
        for edit in &self.edits {
            let (start, end) = (edit.range.start, edit.range.end);
            let new_len = edit.replacement.len() as isize;
            if pos < start || (pos == start && bias == Bias::Before) {
                break;
            }
            if pos == start {
                // Bias::After at the left edge: land after the replacement.
                return (start as isize + delta + new_len) as usize;
            }
            if pos < end {
                // Strictly inside the replaced range: clamp to an edge.
                let base = start as isize + delta;
                return match bias {
                    Bias::Before => base as usize,
                    Bias::After => (base + new_len) as usize,
                };
            }
            delta += new_len - edit.range.len() as isize;
        }
        (pos as isize + delta) as usize
    }

    /// Combine `self` followed by `other` (whose ranges address the document
    /// produced by `self`) into one equivalent transaction addressed in
    /// pre-`self` coordinates.
    pub fn compose(self, other: Transaction) -> Transaction {
        let mut out: Vec<Op> = Vec::new();
        let mut a = self.into_ops().into_iter();
        let mut b = other.into_ops().into_iter();
        let mut head_a = a.next();
        let mut head_b = b.next();
        loop {
            match (head_a.take(), head_b.take()) {
                (None, None) => break,
                // Deletions in the first transaction are invisible to the second.
                (Some(Op::Delete(n)), b_op) => {
                    out.push(Op::Delete(n));
                    head_a = a.next();
                    head_b = b_op;
                }
                // Insertions in the second transaction are taken verbatim.
                (a_op, Some(Op::Insert(s))) => {
                    out.push(Op::Insert(s));
                    head_a = a_op;
                    head_b = b.next();
                }
                // Past the explicit ops of one side everything is retained.
                (None, Some(op)) => {
                    out.push(op);
                    head_b = b.next();
                }
                (Some(op), None) => {
                    out.push(op);
                    head_a = a.next();
                }
                (Some(Op::Retain(i)), Some(Op::Retain(j))) => {
                    let n = i.min(j);
                    out.push(Op::Retain(n));
                    head_a = if i > n {
                        Some(Op::Retain(i - n))
                    } else {
                        a.next()
                    };
                    head_b = if j > n {
                        Some(Op::Retain(j - n))
                    } else {
                        b.next()
                    };
                }
                (Some(Op::Retain(i)), Some(Op::Delete(j))) => {
                    let n = i.min(j);
                    out.push(Op::Delete(n));
                    head_a = if i > n {
                        Some(Op::Retain(i - n))
                    } else {
                        a.next()
                    };
                    head_b = if j > n {
                        Some(Op::Delete(j - n))
                    } else {
                        b.next()
                    };
                }
                (Some(Op::Insert(mut s)), Some(Op::Retain(j))) => {
                    if s.len() <= j {
                        let n = s.len();
                        out.push(Op::Insert(s));
                        head_a = a.next();
                        head_b = if j > n {
                            Some(Op::Retain(j - n))
                        } else {
                            b.next()
                        };
                    } else {
                        let rest = s.split_off(j);
                        out.push(Op::Insert(s));
                        head_a = Some(Op::Insert(rest));
                        head_b = b.next();
                    }
                }
                (Some(Op::Insert(mut s)), Some(Op::Delete(j))) => {
                    if s.len() <= j {
                        // The second transaction deleted this insertion.
                        let n = s.len();
                        head_a = a.next();
                        head_b = if j > n {
                            Some(Op::Delete(j - n))
                        } else {
                            b.next()
                        };
                    } else {
                        head_a = Some(Op::Insert(s.split_off(j)));
                        head_b = b.next();
                    }
                }
            }
        }
        Transaction::from_ops(out)
    }

    fn into_ops(self) -> Vec<Op> {
        let mut ops = Vec::new();
        let mut last = 0;
        for edit in self.edits {
            if edit.range.start > last {
                ops.push(Op::Retain(edit.range.start - last));
            }
            last = edit.range.end;
            if !edit.range.is_empty() {
                ops.push(Op::Delete(edit.range.len()));
            }
            if !edit.replacement.is_empty() {
                ops.push(Op::Insert(edit.replacement));
            }
        }
        ops
    }

    fn from_ops(ops: Vec<Op>) -> Transaction {
        let mut edits: Vec<Edit> = Vec::new();
        let mut pos = 0usize;
        let mut pending: Option<Edit> = None;
        let flush = |pending: &mut Option<Edit>, edits: &mut Vec<Edit>| {
            if let Some(e) = pending.take()
                && (!e.range.is_empty() || !e.replacement.is_empty())
            {
                edits.push(e);
            }
        };
        for op in ops {
            match op {
                Op::Retain(0) => {}
                Op::Retain(n) => {
                    flush(&mut pending, &mut edits);
                    pos += n;
                }
                Op::Delete(n) => {
                    let e = pending.get_or_insert_with(|| Edit {
                        range: pos..pos,
                        replacement: String::new(),
                    });
                    e.range.end += n;
                    pos += n;
                }
                Op::Insert(s) => {
                    let e = pending.get_or_insert_with(|| Edit {
                        range: pos..pos,
                        replacement: String::new(),
                    });
                    e.replacement.push_str(&s);
                }
            }
        }
        flush(&mut pending, &mut edits);
        Transaction { edits }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rope(s: &str) -> Rope {
        Rope::from_str(s)
    }

    #[test]
    fn apply_single_insert() {
        let mut r = rope("hello");
        Transaction::insert(5, " world").apply(&mut r);
        assert_eq!(r.to_string(), "hello world");
    }

    #[test]
    fn apply_multiple_edits_use_original_coordinates() {
        let mut r = rope("ab\ncd\nef");
        // Insert at the start of each line; offsets are all pre-transaction.
        Transaction::change([(0..0, "x"), (3..3, "x"), (6..6, "x")]).apply(&mut r);
        assert_eq!(r.to_string(), "xab\nxcd\nxef");
    }

    #[test]
    fn apply_replace_multibyte() {
        let mut r = rope("ação");
        // "ç" is 2 bytes at offset 1..3.
        Transaction::change([(1..3, "c")]).apply(&mut r);
        assert_eq!(r.to_string(), "acão");
    }

    #[test]
    fn change_sorts_edits_and_drops_noops() {
        let tx = Transaction::change([(5..5, "b"), (0..0, "a"), (3..3, "")]);
        assert_eq!(tx.edits().len(), 2);
        assert_eq!(tx.edits()[0].range, 0..0);
        assert_eq!(tx.edits()[1].range, 5..5);
    }

    #[test]
    #[should_panic(expected = "non-overlapping")]
    fn overlapping_edits_panic() {
        let _ = Transaction::change([(0..4, "a"), (2..6, "b")]);
    }

    #[test]
    fn invert_roundtrip() {
        let original = rope("hello world");
        let tx = Transaction::change([(0..5, "bye"), (6..11, "")]);
        let inverse = tx.invert(&original);
        let mut r = original.clone();
        tx.apply(&mut r);
        assert_eq!(r.to_string(), "bye ");
        inverse.apply(&mut r);
        assert_eq!(r.to_string(), "hello world");
    }

    #[test]
    fn invert_roundtrip_multibyte() {
        let original = rope("ção 日本語 👨‍👩‍👧‍👦");
        let tx = Transaction::change([(0..5, "x"), (6..15, "y")]);
        let inverse = tx.invert(&original);
        let mut r = original.clone();
        tx.apply(&mut r);
        assert_eq!(r.to_string(), "x y 👨‍👩‍👧‍👦");
        inverse.apply(&mut r);
        assert_eq!(r.to_string(), "ção 日本語 👨‍👩‍👧‍👦");
    }

    #[test]
    fn map_pos_through_insertion() {
        let tx = Transaction::insert(3, "ab");
        assert_eq!(tx.map_pos(0, Bias::Before), 0);
        assert_eq!(tx.map_pos(3, Bias::Before), 3);
        assert_eq!(tx.map_pos(3, Bias::After), 5);
        assert_eq!(tx.map_pos(4, Bias::Before), 6);
        assert_eq!(tx.map_pos(4, Bias::After), 6);
    }

    #[test]
    fn map_pos_through_replacement() {
        // Replace bytes 2..5 with "xy" (delta -1).
        let tx = Transaction::change([(2..5, "xy")]);
        assert_eq!(tx.map_pos(0, Bias::After), 0);
        assert_eq!(tx.map_pos(2, Bias::Before), 2);
        assert_eq!(tx.map_pos(2, Bias::After), 4);
        // Inside the deleted range: clamp to an edge of the replacement.
        assert_eq!(tx.map_pos(3, Bias::Before), 2);
        assert_eq!(tx.map_pos(3, Bias::After), 4);
        // At the right boundary both biases agree.
        assert_eq!(tx.map_pos(5, Bias::Before), 4);
        assert_eq!(tx.map_pos(5, Bias::After), 4);
        assert_eq!(tx.map_pos(7, Bias::Before), 6);
    }

    #[test]
    fn map_pos_accumulates_deltas() {
        let tx = Transaction::change([(0..0, "x"), (3..3, "x"), (6..6, "x")]);
        assert_eq!(tx.map_pos(0, Bias::After), 1);
        assert_eq!(tx.map_pos(3, Bias::After), 5);
        assert_eq!(tx.map_pos(6, Bias::After), 9);
        assert_eq!(tx.map_pos(6, Bias::Before), 8);
    }

    #[test]
    fn compose_insert_then_delete_inside() {
        let t1 = Transaction::insert(0, "abc");
        let t2 = Transaction::delete(1..2);
        let composed = t1.clone().compose(t2.clone());
        let mut sequential = rope("");
        t1.apply(&mut sequential);
        t2.apply(&mut sequential);
        let mut composed_rope = rope("");
        composed.apply(&mut composed_rope);
        assert_eq!(composed_rope.to_string(), sequential.to_string());
        assert_eq!(composed_rope.to_string(), "ac");
    }

    #[test]
    fn compose_delete_then_insert() {
        let t1 = Transaction::delete(0..2);
        let t2 = Transaction::insert(1, "X");
        let composed = t1.clone().compose(t2.clone());
        let mut sequential = rope("abcd");
        t1.apply(&mut sequential);
        t2.apply(&mut sequential);
        assert_eq!(sequential.to_string(), "cXd");
        let mut composed_rope = rope("abcd");
        composed.apply(&mut composed_rope);
        assert_eq!(composed_rope.to_string(), "cXd");
    }

    #[test]
    fn compose_typing_coalesces_to_single_insert() {
        let t1 = Transaction::insert(0, "h");
        let t2 = Transaction::insert(1, "e");
        let composed = t1.compose(t2);
        assert_eq!(composed.edits().len(), 1);
        assert_eq!(composed.edits()[0].replacement, "he");
        let mut r = rope("");
        composed.apply(&mut r);
        assert_eq!(r.to_string(), "he");
    }

    #[test]
    fn compose_with_empty_is_identity() {
        let t = Transaction::change([(2..4, "zz")]);
        assert_eq!(t.clone().compose(Transaction::new()), t);
        assert_eq!(Transaction::new().compose(t.clone()), t);
    }
}
