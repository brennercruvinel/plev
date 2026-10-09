//! The keyboard focus of the proposing row (task-75 phase 2a): the field
//! keeps its own; the proposer and the two buttons keep theirs in the
//! widget (the ring it draws), and the row projects whichever has it, so
//! a screen reader that moved the focus there hears it there.

use accord::SemId;

use super::{Ids, Proposing};

impl Proposing {
    /// Keyboard focus from outside (a screen reader): the field, the
    /// proposer or a button of the row takes it, and the others let it
    /// go; any other node of the row takes it from all of them.
    pub fn focus(&mut self, id: &SemId) {
        let Some(ids) = &self.ids else {
            return;
        };
        if *id == ids.intent {
            self.intent.focus();
        } else {
            self.intent.unfocus();
        }
        self.proposer.set_focused(*id == ids.proposer);
        self.button.set_focused(*id == ids.propose);
        self.cancel.set_focused(*id == ids.cancel);
    }

    /// The row lets the focus go: the field, the proposer and the
    /// buttons. `true` when one of them had it.
    pub fn blur(&mut self) -> bool {
        let had = self.intent.is_focused() || self.let_go();
        self.intent.unfocus();
        had
    }

    /// The proposer and the buttons let the focus go (a click of the
    /// person: the focus a reader left there goes). `true` when one had
    /// it.
    pub(super) fn let_go(&mut self) -> bool {
        let had =
            self.proposer.is_focused() || self.button.is_focused() || self.cancel.is_focused();
        self.proposer.set_focused(false);
        self.button.set_focused(false);
        self.cancel.set_focused(false);
        had
    }

    /// The node of the row that has the focus: the field, the proposer,
    /// or a button while it is on (`ready`, `waiting`, as projected).
    pub(super) fn focused<'a>(
        &self,
        ids: &'a Ids,
        ready: bool,
        waiting: bool,
    ) -> Option<&'a SemId> {
        if self.intent.is_focused() {
            Some(&ids.intent)
        } else if self.proposer.is_focused() {
            Some(&ids.proposer)
        } else if self.button.is_focused() && ready {
            Some(&ids.propose)
        } else if self.cancel.is_focused() && waiting {
            Some(&ids.cancel)
        } else {
            None
        }
    }
}
