//! Undo/redo history with deterministic coalescing of typed text.
//!
//! Coalescing is purely structural — no wall clock involved: consecutive
//! word-like insertions that continue exactly where the previous commit
//! left the selections are merged into one undo group. Whitespace or
//! punctuation still joins the open group but closes it afterwards, so
//! typing "hello world" yields two groups ("hello " and "world"). A group
//! also breaks on selection changes, non-typing edits, undo/redo, or an
//! explicit [`History::commit_group`].

use crate::selection::SelectionSet;
use crate::transaction::Transaction;

/// How a commit was produced; drives undo-group coalescing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommitKind {
    /// Text typed at the selections. `word_like` keeps the group open so
    /// the next typed characters coalesce into the same group.
    Typing { word_like: bool },
    /// Any other edit; never coalesces and closes the open group.
    Other,
}

/// One undo group: the transaction that redoes it, the inverse that undoes
/// it, and the selections on both sides.
#[derive(Debug, Clone)]
pub struct UndoStep {
    pub(crate) redo: Transaction,
    pub(crate) inverse: Transaction,
    pub(crate) selections_before: SelectionSet,
    pub(crate) selections_after: SelectionSet,
}

#[derive(Debug, Default)]
pub struct History {
    undo: Vec<UndoStep>,
    redo: Vec<UndoStep>,
    group_open: bool,
}

impl History {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a committed transaction. `redo` is the applied transaction,
    /// `inverse` its inversion (computed against the pre-apply document),
    /// `before`/`after` the selections around it.
    pub fn commit(
        &mut self,
        redo: Transaction,
        inverse: Transaction,
        before: SelectionSet,
        after: SelectionSet,
        kind: CommitKind,
    ) {
        self.redo.clear();
        let coalesce = self.group_open
            && matches!(kind, CommitKind::Typing { .. })
            && self
                .undo
                .last()
                .is_some_and(|step| step.selections_after == before);
        if coalesce {
            let top = self.undo.last_mut().expect("group_open implies a step");
            top.redo = std::mem::take(&mut top.redo).compose(redo);
            top.inverse = inverse.compose(std::mem::take(&mut top.inverse));
            top.selections_after = after;
        } else {
            self.undo.push(UndoStep {
                redo,
                inverse,
                selections_before: before,
                selections_after: after,
            });
        }
        self.group_open = matches!(kind, CommitKind::Typing { word_like: true });
    }

    /// Explicitly close the open coalescing group.
    pub fn commit_group(&mut self) {
        self.group_open = false;
    }

    /// Pop the most recent undo group. The caller applies `inverse` and
    /// restores `selections_before`; the step is kept for redo.
    pub fn undo(&mut self) -> Option<&UndoStep> {
        self.group_open = false;
        let step = self.undo.pop()?;
        self.redo.push(step);
        self.redo.last()
    }

    /// Pop the most recent redo group. The caller applies `redo` and
    /// restores `selections_after`; the step goes back onto the undo stack.
    pub fn redo(&mut self) -> Option<&UndoStep> {
        self.group_open = false;
        let step = self.redo.pop()?;
        self.undo.push(step);
        self.undo.last()
    }

    pub fn undo_depth(&self) -> usize {
        self.undo.len()
    }

    pub fn redo_depth(&self) -> usize {
        self.redo.len()
    }
}

#[cfg(test)]
mod tests {
    use ropey::Rope;

    use super::*;
    use crate::selection::SelectionSet;

    /// Type one character at `pos`, committing it to history.
    fn type_char(history: &mut History, rope: &mut Rope, pos: usize, ch: char) {
        let text = ch.to_string();
        let tx = Transaction::insert(pos, &text);
        let inverse = tx.invert(rope);
        tx.apply(rope);
        history.commit(
            tx,
            inverse,
            SelectionSet::caret(pos),
            SelectionSet::caret(pos + ch.len_utf8()),
            CommitKind::Typing {
                word_like: ch.is_alphanumeric() || ch == '_',
            },
        );
    }

    #[test]
    fn typing_word_coalesces_into_one_group() {
        let mut history = History::new();
        let mut rope = Rope::from_str("");
        for (i, ch) in "hello".chars().enumerate() {
            type_char(&mut history, &mut rope, i, ch);
        }
        assert_eq!(rope.to_string(), "hello");
        assert_eq!(history.undo_depth(), 1);
        let step = history.undo().unwrap();
        step.inverse.apply(&mut rope);
        assert_eq!(rope.to_string(), "");
        assert_eq!(step.selections_before, SelectionSet::caret(0));
    }

    #[test]
    fn whitespace_joins_then_closes_group() {
        let mut history = History::new();
        let mut rope = Rope::from_str("");
        for (i, ch) in "hello world".chars().enumerate() {
            type_char(&mut history, &mut rope, i, ch);
        }
        assert_eq!(history.undo_depth(), 2);
        let step = history.undo().unwrap();
        step.inverse.apply(&mut rope);
        assert_eq!(rope.to_string(), "hello ");
        let step = history.undo().unwrap();
        step.inverse.apply(&mut rope);
        assert_eq!(rope.to_string(), "");
    }

    #[test]
    fn commit_group_breaks_coalescing() {
        let mut history = History::new();
        let mut rope = Rope::from_str("");
        type_char(&mut history, &mut rope, 0, 'a');
        history.commit_group();
        type_char(&mut history, &mut rope, 1, 'b');
        assert_eq!(history.undo_depth(), 2);
    }

    #[test]
    fn discontinuous_typing_breaks_coalescing() {
        let mut history = History::new();
        let mut rope = Rope::from_str("xx");
        type_char(&mut history, &mut rope, 0, 'a');
        // Next char typed somewhere else: selections do not continue.
        type_char(&mut history, &mut rope, 3, 'b');
        assert_eq!(history.undo_depth(), 2);
    }

    #[test]
    fn undo_redo_move_steps_between_stacks() {
        let mut history = History::new();
        let mut rope = Rope::from_str("");
        type_char(&mut history, &mut rope, 0, 'a');
        let step = history.undo().unwrap();
        step.inverse.apply(&mut rope);
        assert_eq!(rope.to_string(), "");
        assert_eq!(history.undo_depth(), 0);
        assert_eq!(history.redo_depth(), 1);
        let step = history.redo().unwrap();
        step.redo.apply(&mut rope);
        assert_eq!(rope.to_string(), "a");
        assert_eq!(history.redo_depth(), 0);
    }

    #[test]
    fn new_commit_clears_redo_stack() {
        let mut history = History::new();
        let mut rope = Rope::from_str("");
        type_char(&mut history, &mut rope, 0, 'a');
        let step = history.undo().unwrap();
        step.inverse.apply(&mut rope);
        assert_eq!(history.redo_depth(), 1);
        type_char(&mut history, &mut rope, 0, 'b');
        assert_eq!(history.redo_depth(), 0);
    }

    #[test]
    fn undo_breaks_open_group() {
        let mut history = History::new();
        let mut rope = Rope::from_str("");
        type_char(&mut history, &mut rope, 0, 'a');
        let step = history.undo().unwrap();
        step.inverse.apply(&mut rope);
        // Type again after undo: must be a fresh group, not a merge.
        type_char(&mut history, &mut rope, 0, 'b');
        assert_eq!(history.undo_depth(), 1);
        let step = history.undo().unwrap();
        step.inverse.apply(&mut rope);
        assert_eq!(rope.to_string(), "");
    }
}
