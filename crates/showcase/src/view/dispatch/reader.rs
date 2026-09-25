//! What a screen reader is shown (task-75 phase 2a): the view's
//! projection (`ShowcaseView::semantics`: the page, and the toasts and
//! the overlay over it, what an agent reads too), and what only a reader
//! hears. The shell's bridge builds the tree from this snapshot
//! (`app/pace.rs`), and reads a reader's requests back against it.
//!
//! What only a reader hears:
//!
//! - the shell's notice (`shell:notice`, the last thing the shell told
//!   the person on its own, a job that ended while no window had the
//!   focus), as a status, `shell.notice`: a polite live region, heard
//!   when it changes. The port reads it as the shell's state, as before.
//!
//! With nothing only a reader hears, the snapshot is the view's, byte for
//! byte.

use sem::{Role, SemId, Snapshot, SnapshotBuilder};

use super::super::ShowcaseView;
use super::detached;

/// The id of the notice's status in the reader's snapshot.
pub(in crate::view) const NOTICE: &str = "shell.notice";

impl ShowcaseView {
    /// What a screen reader is shown this frame: [`ShowcaseView::semantics`]
    /// and what only a reader hears (see the module docs). `None` when
    /// there is nothing to show but the window.
    pub fn reader_semantics(&self) -> Option<Snapshot> {
        let seen = self.semantics();
        let notice = self.panes.shell.notice();
        if notice.is_empty() {
            return seen;
        }
        let mut b = SnapshotBuilder::new();
        if let Some(seen) = &seen {
            detached::extend(&mut b, seen);
        }
        if !notice.is_empty()
            && let Ok(id) = SemId::parse(NOTICE)
        {
            b.node(&id, Role::Status, notice);
        }
        match b.finish() {
            Ok(snapshot) => Some(snapshot),
            Err(e) => {
                log::warn!("accessibility: the reader's snapshot does not validate: {e}");
                seen
            }
        }
    }
}
