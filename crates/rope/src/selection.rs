//! Selections and multi-cursor selection sets.

use ropey::Rope;

use crate::movement::line_content_end;
use crate::transaction::{Bias, Transaction};

/// A single selection as a pair of byte offsets. `anchor` is the fixed end,
/// `head` the moving end (where the cursor blinks). A caret is a selection
/// with `anchor == head`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Selection {
    pub anchor: usize,
    pub head: usize,
}

impl Selection {
    pub fn new(anchor: usize, head: usize) -> Self {
        Self { anchor, head }
    }

    pub fn caret(pos: usize) -> Self {
        Self {
            anchor: pos,
            head: pos,
        }
    }

    pub fn min(&self) -> usize {
        self.anchor.min(self.head)
    }

    pub fn max(&self) -> usize {
        self.anchor.max(self.head)
    }

    pub fn is_caret(&self) -> bool {
        self.anchor == self.head
    }

    /// Map both endpoints through a transaction.
    pub fn map(&self, tx: &Transaction, bias: Bias) -> Selection {
        Selection {
            anchor: tx.map_pos(self.anchor, bias),
            head: tx.map_pos(self.head, bias),
        }
    }
}

/// A non-empty set of selections, kept sorted by start with overlaps merged,
/// plus the index of the primary selection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectionSet {
    selections: Vec<Selection>,
    primary: usize,
}

impl SelectionSet {
    /// A single caret at `pos`.
    pub fn caret(pos: usize) -> Self {
        Self::single(Selection::caret(pos))
    }

    pub fn single(selection: Selection) -> Self {
        Self {
            selections: vec![selection],
            primary: 0,
        }
    }

    /// Build from a list of selections; `primary` indexes into the input
    /// list and is tracked through normalization.
    pub fn new(selections: Vec<Selection>, primary: usize) -> Self {
        assert!(!selections.is_empty(), "a SelectionSet cannot be empty");
        let primary = primary.min(selections.len() - 1);
        let mut set = Self {
            selections,
            primary,
        };
        set.normalize();
        set
    }

    pub fn selections(&self) -> &[Selection] {
        &self.selections
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Selection> {
        self.selections.iter()
    }

    pub fn len(&self) -> usize {
        self.selections.len()
    }

    pub fn is_empty(&self) -> bool {
        false // invariant: never empty
    }

    pub fn primary(&self) -> Selection {
        self.selections[self.primary]
    }

    pub fn primary_index(&self) -> usize {
        self.primary
    }

    /// Add a selection; it becomes the primary one.
    pub fn add_selection(&mut self, selection: Selection) {
        self.selections.push(selection);
        self.primary = self.selections.len() - 1;
        self.normalize();
    }

    /// Split every multi-line selection into one caret per line, placed at
    /// the end of the selected portion of each line.
    pub fn split_lines(&mut self, rope: &Rope) {
        let mut out: Vec<Selection> = Vec::new();
        let mut primary = 0;
        for (idx, sel) in self.selections.iter().enumerate() {
            if sel.is_caret() {
                out.push(*sel);
            } else {
                let (min, max) = (sel.min(), sel.max());
                let first = rope.byte_to_line(min);
                let mut last = rope.byte_to_line(max);
                // A selection ending exactly at a line start does not
                // include that line.
                if last > first && rope.line_to_byte(last) == max {
                    last -= 1;
                }
                for line in first..=last {
                    let end = line_content_end(rope, line);
                    out.push(Selection::caret(end.min(max).max(min)));
                }
            }
            if idx == self.primary {
                primary = out.len() - 1;
            }
        }
        *self = Self::new(out, primary);
    }

    /// Replace the set with one selection per literal occurrence of
    /// `pattern`. Returns false (leaving the set untouched) if the pattern
    /// is empty or absent.
    pub fn select_all_matches(&mut self, rope: &Rope, pattern: &str) -> bool {
        if pattern.is_empty() {
            return false;
        }
        let text = rope.to_string();
        let matches: Vec<Selection> = text
            .match_indices(pattern)
            .map(|(start, m)| Selection::new(start, start + m.len()))
            .collect();
        if matches.is_empty() {
            return false;
        }
        *self = Self::new(matches, 0);
        true
    }

    /// Collapse every selection to a caret at its head.
    pub fn collapse_to_carets(&mut self) {
        for sel in &mut self.selections {
            *sel = Selection::caret(sel.head);
        }
        self.normalize();
    }

    /// Keep only the primary selection.
    pub fn remove_secondary(&mut self) {
        let primary = self.selections[self.primary];
        self.selections = vec![primary];
        self.primary = 0;
    }

    /// Map every selection through a transaction.
    pub fn map(&self, tx: &Transaction, bias: Bias) -> SelectionSet {
        Self::new(self.iter().map(|s| s.map(tx, bias)).collect(), self.primary)
    }

    /// Restore the invariants: selections sorted by start, overlapping
    /// ranges merged (carets touching a range edge are absorbed by it),
    /// primary index tracked through the reordering.
    fn normalize(&mut self) {
        let mut tagged: Vec<(Selection, bool)> = self
            .selections
            .iter()
            .enumerate()
            .map(|(i, s)| (*s, i == self.primary))
            .collect();
        tagged.sort_by_key(|(s, _)| (s.min(), s.max()));
        let mut merged: Vec<(Selection, bool)> = Vec::new();
        for (sel, is_primary) in tagged {
            if let Some((last, last_primary)) = merged.last_mut() {
                let overlaps = sel.min() < last.max()
                    || (sel.min() == last.max() && (sel.is_caret() || last.is_caret()));
                if overlaps {
                    let min = last.min();
                    let max = last.max().max(sel.max());
                    // Keep the direction of the existing entry.
                    *last = if last.head >= last.anchor {
                        Selection::new(min, max)
                    } else {
                        Selection::new(max, min)
                    };
                    *last_primary |= is_primary;
                    continue;
                }
            }
            merged.push((sel, is_primary));
        }
        self.primary = merged.iter().position(|(_, p)| *p).unwrap_or(0);
        self.selections = merged.into_iter().map(|(s, _)| s).collect();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn min_max_caret() {
        let sel = Selection::new(7, 3);
        assert_eq!(sel.min(), 3);
        assert_eq!(sel.max(), 7);
        assert!(!sel.is_caret());
        assert!(Selection::caret(5).is_caret());
    }

    #[test]
    fn normalize_sorts_and_merges_overlapping() {
        let set = SelectionSet::new(
            vec![
                Selection::new(8, 12),
                Selection::new(0, 5),
                Selection::new(3, 7),
            ],
            0,
        );
        assert_eq!(
            set.selections(),
            &[Selection::new(0, 7), Selection::new(8, 12)]
        );
        assert_eq!(set.primary(), Selection::new(8, 12));
    }

    #[test]
    fn normalize_dedupes_carets() {
        let set = SelectionSet::new(vec![Selection::caret(4), Selection::caret(4)], 1);
        assert_eq!(set.len(), 1);
        assert_eq!(set.primary(), Selection::caret(4));
    }

    #[test]
    fn normalize_absorbs_caret_touching_range() {
        let set = SelectionSet::new(vec![Selection::caret(5), Selection::new(5, 8)], 0);
        assert_eq!(set.selections(), &[Selection::new(5, 8)]);
        let set = SelectionSet::new(vec![Selection::new(2, 5), Selection::caret(5)], 1);
        assert_eq!(set.selections(), &[Selection::new(2, 5)]);
    }

    #[test]
    fn touching_ranges_stay_separate() {
        let set = SelectionSet::new(vec![Selection::new(2, 5), Selection::new(5, 8)], 0);
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn add_selection_becomes_primary() {
        let mut set = SelectionSet::caret(10);
        set.add_selection(Selection::caret(2));
        assert_eq!(set.len(), 2);
        assert_eq!(set.primary(), Selection::caret(2));
        assert_eq!(set.selections()[0], Selection::caret(2));
    }

    #[test]
    fn split_lines_one_caret_per_line() {
        let rope = Rope::from_str("hello\nworld\nfoo");
        let mut set = SelectionSet::single(Selection::new(2, 14));
        set.split_lines(&rope);
        assert_eq!(
            set.selections(),
            &[
                Selection::caret(5),
                Selection::caret(11),
                Selection::caret(14)
            ]
        );
        assert_eq!(set.primary(), Selection::caret(14));
    }

    #[test]
    fn split_lines_excludes_line_start_boundary() {
        let rope = Rope::from_str("ab\ncd");
        // Selection ends exactly at the start of line 1.
        let mut set = SelectionSet::single(Selection::new(0, 3));
        set.split_lines(&rope);
        assert_eq!(set.selections(), &[Selection::caret(2)]);
    }

    #[test]
    fn split_lines_keeps_carets() {
        let rope = Rope::from_str("ab\ncd");
        let mut set = SelectionSet::caret(1);
        set.split_lines(&rope);
        assert_eq!(set.selections(), &[Selection::caret(1)]);
    }

    #[test]
    fn select_all_matches_literal() {
        let rope = Rope::from_str("foo bar foo baz foo");
        let mut set = SelectionSet::caret(0);
        assert!(set.select_all_matches(&rope, "foo"));
        assert_eq!(
            set.selections(),
            &[
                Selection::new(0, 3),
                Selection::new(8, 11),
                Selection::new(16, 19)
            ]
        );
    }

    #[test]
    fn select_all_matches_multibyte() {
        let rope = Rope::from_str("ção e ção");
        let mut set = SelectionSet::caret(0);
        assert!(set.select_all_matches(&rope, "ção"));
        assert_eq!(
            set.selections(),
            &[Selection::new(0, 5), Selection::new(8, 13)]
        );
    }

    #[test]
    fn select_all_matches_absent_leaves_set_untouched() {
        let rope = Rope::from_str("abc");
        let mut set = SelectionSet::caret(1);
        assert!(!set.select_all_matches(&rope, "zzz"));
        assert!(!set.select_all_matches(&rope, ""));
        assert_eq!(set.selections(), &[Selection::caret(1)]);
    }

    #[test]
    fn collapse_to_carets_and_remove_secondary() {
        let mut set = SelectionSet::new(vec![Selection::new(0, 3), Selection::new(10, 6)], 1);
        set.collapse_to_carets();
        assert_eq!(
            set.selections(),
            &[Selection::caret(3), Selection::caret(6)]
        );
        assert_eq!(set.primary(), Selection::caret(6));
        set.remove_secondary();
        assert_eq!(set.selections(), &[Selection::caret(6)]);
        assert_eq!(set.primary_index(), 0);
    }
}
