//! The document: rope buffer + selections + history, with multi-cursor
//! editing primitives.

use std::fmt;
use std::ops::Range;

use ropey::Rope;

use crate::history::{CommitKind, History};
use crate::movement::{next_grapheme_boundary, prev_grapheme_boundary};
use crate::selection::{Selection, SelectionSet};
use crate::transaction::{Bias, Transaction};

#[derive(Debug)]
pub struct Document {
    rope: Rope,
    selections: SelectionSet,
    history: History,
}

impl Default for Document {
    fn default() -> Self {
        Self::load("")
    }
}

impl From<&str> for Document {
    fn from(text: &str) -> Self {
        Self::load(text)
    }
}

impl fmt::Display for Document {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.rope)
    }
}

/// True when every char is something one keeps typing inside a word;
/// whitespace/punctuation break undo coalescing.
fn is_word_like(text: &str) -> bool {
    !text.is_empty() && text.chars().all(|c| c.is_alphanumeric() || c == '_')
}

impl Document {
    pub fn new() -> Self {
        Self::default()
    }

    /// Load text, normalizing CRLF line endings to LF.
    pub fn load(text: &str) -> Self {
        let text = if text.contains('\r') {
            text.replace("\r\n", "\n")
        } else {
            text.into()
        };
        Self {
            rope: Rope::from_str(&text),
            selections: SelectionSet::caret(0),
            history: History::new(),
        }
    }

    pub fn rope(&self) -> &Rope {
        &self.rope
    }

    pub fn len_bytes(&self) -> usize {
        self.rope.len_bytes()
    }

    pub fn len_chars(&self) -> usize {
        self.rope.len_chars()
    }

    pub fn len_lines(&self) -> usize {
        self.rope.len_lines()
    }

    pub fn selections(&self) -> &SelectionSet {
        &self.selections
    }

    /// Replace the selections. Closes the open undo group: moving the
    /// cursor breaks typing coalescence.
    pub fn set_selections(&mut self, selections: SelectionSet) {
        self.selections = selections;
        self.history.commit_group();
    }

    /// Apply a movement-style function to every selection.
    pub fn transform_selections(&mut self, mut f: impl FnMut(&Rope, Selection) -> Selection) {
        let moved: Vec<Selection> = self.selections.iter().map(|s| f(&self.rope, *s)).collect();
        self.set_selections(SelectionSet::new(moved, self.selections.primary_index()));
    }

    /// Explicitly close the open undo group.
    pub fn commit_group(&mut self) {
        self.history.commit_group();
    }

    pub fn undo_depth(&self) -> usize {
        self.history.undo_depth()
    }

    pub fn redo_depth(&self) -> usize {
        self.history.redo_depth()
    }

    /// Insert `text` at every selection (replacing non-caret selections),
    /// as a single transaction. Each cursor ends up after its insertion.
    pub fn insert(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }
        let edits: Vec<(Range<usize>, &str)> = self
            .selections
            .iter()
            .map(|s| (s.min()..s.max(), text))
            .collect();
        let tx = Transaction::change(edits);
        let carets: Vec<Selection> = self
            .selections
            .iter()
            .map(|s| Selection::caret(tx.map_pos(s.min(), Bias::After)))
            .collect();
        let selections = SelectionSet::new(carets, self.selections.primary_index());
        self.commit_edit(
            tx,
            selections,
            CommitKind::Typing {
                word_like: is_word_like(text),
            },
        );
    }

    /// Delete one grapheme before each caret, or the selected text of
    /// non-caret selections.
    pub fn delete_backward(&mut self) {
        let ranges: Vec<Range<usize>> = self
            .selections
            .iter()
            .map(|s| {
                if s.is_caret() {
                    prev_grapheme_boundary(&self.rope, s.head)..s.head
                } else {
                    s.min()..s.max()
                }
            })
            .collect();
        self.delete_ranges(ranges);
    }

    /// Delete one grapheme after each caret, or the selected text of
    /// non-caret selections.
    pub fn delete_forward(&mut self) {
        let ranges: Vec<Range<usize>> = self
            .selections
            .iter()
            .map(|s| {
                if s.is_caret() {
                    s.head..next_grapheme_boundary(&self.rope, s.head)
                } else {
                    s.min()..s.max()
                }
            })
            .collect();
        self.delete_ranges(ranges);
    }

    fn delete_ranges(&mut self, mut ranges: Vec<Range<usize>>) {
        // Selections are disjoint, but grapheme extension can make ranges
        // overlap (e.g. two carets inside one ZWJ cluster): clamp starts.
        for i in 1..ranges.len() {
            let prev_end = ranges[i - 1].end;
            if ranges[i].start < prev_end {
                ranges[i].start = prev_end.min(ranges[i].end);
            }
        }
        let tx = Transaction::change(ranges.iter().map(|r| (r.clone(), "")));
        if tx.is_empty() {
            return;
        }
        let carets: Vec<Selection> = ranges
            .iter()
            .map(|r| Selection::caret(tx.map_pos(r.start, Bias::Before)))
            .collect();
        let selections = SelectionSet::new(carets, self.selections.primary_index());
        self.commit_edit(tx, selections, CommitKind::Other);
    }

    /// Apply an arbitrary transaction, mapping the selections through it.
    pub fn apply(&mut self, tx: Transaction) {
        let selections = self.selections.map(&tx, Bias::After);
        self.commit_edit(tx, selections, CommitKind::Other);
    }

    fn commit_edit(&mut self, tx: Transaction, selections: SelectionSet, kind: CommitKind) {
        if tx.is_empty() {
            return;
        }
        let before = self.selections.clone();
        let inverse = tx.invert(&self.rope);
        tx.apply(&mut self.rope);
        self.selections = selections;
        self.history
            .commit(tx, inverse, before, self.selections.clone(), kind);
    }

    /// Undo the last group, restoring text and selections. Returns false
    /// when there is nothing to undo.
    pub fn undo(&mut self) -> bool {
        match self.history.undo() {
            Some(step) => {
                step.inverse.apply(&mut self.rope);
                self.selections = step.selections_before.clone();
                true
            }
            None => false,
        }
    }

    /// Redo the last undone group, restoring text and selections. Returns
    /// false when there is nothing to redo.
    pub fn redo(&mut self) -> bool {
        match self.history.redo() {
            Some(step) => {
                step.redo.apply(&mut self.rope);
                self.selections = step.selections_after.clone();
                true
            }
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FAMILY: &str = "👨‍👩‍👧‍👦"; // one grapheme, 25 bytes

    fn carets(doc: &Document) -> Vec<usize> {
        doc.selections()
            .iter()
            .map(|s| {
                assert!(s.is_caret());
                s.head
            })
            .collect()
    }

    #[test]
    fn load_normalizes_crlf() {
        let doc = Document::load("a\r\nb\r\nc");
        assert_eq!(doc.to_string(), "a\nb\nc");
        assert_eq!(doc.len_lines(), 3);
    }

    #[test]
    fn len_and_line_count() {
        let doc = Document::load("ção\n日本語");
        assert_eq!(doc.len_bytes(), 15);
        assert_eq!(doc.len_chars(), 7);
        assert_eq!(doc.len_lines(), 2);
        assert_eq!(doc.to_string(), "ção\n日本語");
    }

    #[test]
    fn insert_at_single_caret() {
        let mut doc = Document::load("hello");
        doc.set_selections(SelectionSet::caret(5));
        doc.insert(" world");
        assert_eq!(doc.to_string(), "hello world");
        assert_eq!(carets(&doc), vec![11]);
    }

    #[test]
    fn insert_with_three_cursors_offsets_are_mapped() {
        let mut doc = Document::load("ab\ncd\nef");
        doc.set_selections(SelectionSet::new(
            vec![
                Selection::caret(0),
                Selection::caret(3),
                Selection::caret(6),
            ],
            0,
        ));
        doc.insert("x");
        assert_eq!(doc.to_string(), "xab\nxcd\nxef");
        assert_eq!(carets(&doc), vec![1, 5, 9]);
    }

    #[test]
    fn insert_replaces_selected_text() {
        let mut doc = Document::load("foo bar foo");
        let mut sels = SelectionSet::caret(0);
        assert!(sels.select_all_matches(doc.rope(), "foo"));
        doc.set_selections(sels);
        doc.insert("qux");
        assert_eq!(doc.to_string(), "qux bar qux");
        assert_eq!(carets(&doc), vec![3, 11]);
    }

    #[test]
    fn delete_backward_simple() {
        let mut doc = Document::load("abc");
        doc.set_selections(SelectionSet::caret(3));
        doc.delete_backward();
        assert_eq!(doc.to_string(), "ab");
        assert_eq!(carets(&doc), vec![2]);
    }

    #[test]
    fn delete_backward_removes_whole_zwj_emoji() {
        let text = format!("a{FAMILY}");
        let mut doc = Document::load(&text);
        doc.set_selections(SelectionSet::caret(doc.len_bytes()));
        doc.delete_backward();
        assert_eq!(doc.to_string(), "a");
        assert_eq!(carets(&doc), vec![1]);
    }

    #[test]
    fn delete_backward_multibyte() {
        let mut doc = Document::load("ção");
        doc.set_selections(SelectionSet::caret(2));
        doc.delete_backward();
        assert_eq!(doc.to_string(), "ão");
    }

    #[test]
    fn delete_backward_multi_cursor() {
        let mut doc = Document::load("xab\nxcd");
        doc.set_selections(SelectionSet::new(
            vec![Selection::caret(1), Selection::caret(5)],
            0,
        ));
        doc.delete_backward();
        assert_eq!(doc.to_string(), "ab\ncd");
        assert_eq!(carets(&doc), vec![0, 3]);
    }

    #[test]
    fn delete_backward_at_document_start_is_noop() {
        let mut doc = Document::load("ab");
        doc.set_selections(SelectionSet::caret(0));
        doc.delete_backward();
        assert_eq!(doc.to_string(), "ab");
        assert_eq!(doc.undo_depth(), 0);
    }

    #[test]
    fn delete_forward_grapheme_and_end_noop() {
        let text = format!("{FAMILY}a");
        let mut doc = Document::load(&text);
        doc.set_selections(SelectionSet::caret(0));
        doc.delete_forward();
        assert_eq!(doc.to_string(), "a");
        assert_eq!(carets(&doc), vec![0]);
        doc.set_selections(SelectionSet::caret(1));
        doc.delete_forward();
        assert_eq!(doc.to_string(), "a");
    }

    #[test]
    fn delete_forward_selection() {
        let mut doc = Document::load("hello world");
        doc.set_selections(SelectionSet::single(Selection::new(5, 11)));
        doc.delete_forward();
        assert_eq!(doc.to_string(), "hello");
        assert_eq!(carets(&doc), vec![5]);
    }

    #[test]
    fn undo_redo_full_sequence_restores_text_and_selections() {
        let mut doc = Document::load("base");
        doc.set_selections(SelectionSet::caret(4));
        doc.insert("!");
        doc.commit_group();
        doc.set_selections(SelectionSet::caret(0));
        doc.insert("# ");
        doc.commit_group();
        doc.set_selections(SelectionSet::caret(3));
        doc.delete_backward();
        assert_eq!(doc.to_string(), "# ase!");

        assert!(doc.undo());
        assert_eq!(doc.to_string(), "# base!");
        assert_eq!(doc.selections(), &SelectionSet::caret(3));
        assert!(doc.undo());
        assert_eq!(doc.to_string(), "base!");
        assert_eq!(doc.selections(), &SelectionSet::caret(0));
        assert!(doc.undo());
        assert_eq!(doc.to_string(), "base");
        assert_eq!(doc.selections(), &SelectionSet::caret(4));
        assert!(!doc.undo());

        assert!(doc.redo());
        assert!(doc.redo());
        assert!(doc.redo());
        assert_eq!(doc.to_string(), "# ase!");
        assert_eq!(doc.selections(), &SelectionSet::caret(2));
        assert!(!doc.redo());
    }

    #[test]
    fn undo_restores_multi_cursor_selections() {
        let mut doc = Document::load("ab\ncd\nef");
        let before = SelectionSet::new(
            vec![
                Selection::caret(0),
                Selection::caret(3),
                Selection::caret(6),
            ],
            1,
        );
        doc.set_selections(before.clone());
        doc.insert("x");
        assert!(doc.undo());
        assert_eq!(doc.to_string(), "ab\ncd\nef");
        assert_eq!(doc.selections(), &before);
        assert!(doc.redo());
        assert_eq!(doc.to_string(), "xab\nxcd\nxef");
        assert_eq!(carets(&doc), vec![1, 5, 9]);
    }

    #[test]
    fn typing_hello_is_one_undo() {
        let mut doc = Document::new();
        for ch in "hello".chars() {
            doc.insert(&ch.to_string());
        }
        assert_eq!(doc.to_string(), "hello");
        assert_eq!(doc.undo_depth(), 1);
        assert!(doc.undo());
        assert_eq!(doc.to_string(), "");
        assert_eq!(doc.selections(), &SelectionSet::caret(0));
    }

    #[test]
    fn typing_hello_world_is_two_undos() {
        let mut doc = Document::new();
        for ch in "hello world".chars() {
            doc.insert(&ch.to_string());
        }
        assert_eq!(doc.undo_depth(), 2);
        assert!(doc.undo());
        assert_eq!(doc.to_string(), "hello ");
        assert!(doc.undo());
        assert_eq!(doc.to_string(), "");
        assert!(doc.redo());
        assert!(doc.redo());
        assert_eq!(doc.to_string(), "hello world");
    }

    #[test]
    fn selection_change_breaks_coalescing() {
        let mut doc = Document::new();
        doc.insert("he");
        doc.set_selections(doc.selections().clone());
        doc.insert("llo");
        assert_eq!(doc.to_string(), "hello");
        assert_eq!(doc.undo_depth(), 2);
    }

    #[test]
    fn deletes_do_not_coalesce() {
        let mut doc = Document::new();
        doc.insert("hi");
        doc.delete_backward();
        doc.delete_backward();
        assert_eq!(doc.undo_depth(), 3);
    }

    #[test]
    fn multi_cursor_typing_coalesces() {
        let mut doc = Document::load("a\nb");
        doc.set_selections(SelectionSet::new(
            vec![Selection::caret(1), Selection::caret(3)],
            0,
        ));
        doc.insert("x");
        doc.insert("y");
        assert_eq!(doc.to_string(), "axy\nbxy");
        assert_eq!(doc.undo_depth(), 1);
        assert!(doc.undo());
        assert_eq!(doc.to_string(), "a\nb");
    }

    #[test]
    fn redo_cleared_by_new_edit() {
        let mut doc = Document::new();
        doc.insert("a");
        assert!(doc.undo());
        assert_eq!(doc.redo_depth(), 1);
        doc.insert("b");
        assert_eq!(doc.redo_depth(), 0);
        assert!(!doc.redo());
        assert_eq!(doc.to_string(), "b");
    }

    #[test]
    fn apply_generic_transaction_maps_selections() {
        let mut doc = Document::load("hello world");
        doc.set_selections(SelectionSet::caret(11));
        doc.apply(Transaction::change([(0..5, "bye")]));
        assert_eq!(doc.to_string(), "bye world");
        assert_eq!(carets(&doc), vec![9]);
        assert!(doc.undo());
        assert_eq!(doc.to_string(), "hello world");
        assert_eq!(carets(&doc), vec![11]);
    }
}
