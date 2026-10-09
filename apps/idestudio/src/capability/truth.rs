//! Where an execution that left for the git worker ends. The worker is the
//! point of truth: a commit is checked again there against the real index
//! (`GitCommand::CommitIf`), and its answer closes the receipt the request
//! opened: completed with the sha and the files, failed, or rejected with
//! the delta the index showed. A commit is shown in the view before its
//! receipt completes, so no observation after the receipt contradicts it.

use std::collections::BTreeSet;

use gitdrv::{GitEvent, NewCommit};
use accord::{Change, Outcome, Rejection, RequestId, StateRef, Value};

use super::Workbench;

/// What a request whose execution left for the worker waits for. The
/// worker runs commands in the order they were sent and answers each in
/// that order, so an answer belongs to the oldest request waiting for one
/// of its kind (and its path).
pub(super) enum Inflight {
    Stage(String),
    Unstage(String),
    Discard(String),
    Ignore(String),
    /// A `CommitIf`: the message it sent and the staged set it expected.
    Commit {
        message: String,
        expect: BTreeSet<String>,
    },
}

/// How a waiting request ends.
enum End {
    Done(Outcome),
    Failed(String),
    Refused(Rejection),
}

impl Workbench {
    /// Ends the receipt `event` answers, when a request waits for it.
    pub(super) fn settle(&mut self, event: &GitEvent) {
        let answer = match event {
            GitEvent::Staged { path, result } => {
                self.answer(result, |w| matches!(w, Inflight::Stage(p) if p == path))
            }
            GitEvent::Unstaged { path, result } => {
                self.answer(result, |w| matches!(w, Inflight::Unstage(p) if p == path))
            }
            GitEvent::Discarded { path, result } => {
                self.answer(result, |w| matches!(w, Inflight::Discard(p) if p == path))
            }
            GitEvent::Ignored { path, result } => {
                self.answer(result, |w| matches!(w, Inflight::Ignore(p) if p == path))
            }
            GitEvent::Committed(result) => self.committed(result),
            GitEvent::CommitRejected { added, removed } => self.refused(added, removed),
            _ => return,
        };
        let Some((id, end)) = answer else {
            log::warn!("ide: no request waits for {event:?}");
            return;
        };
        let now = self.now();
        let ended = match end {
            End::Done(outcome) => self.dispatcher.complete(id, outcome, now),
            End::Failed(error) => self.dispatcher.fail(id, error, now),
            End::Refused(why) => self.dispatcher.reject_at_truth(id, why, now),
        };
        match ended {
            Ok(()) => self.report(id),
            Err(e) => log::error!("ide: request {id} cannot end: {e}"),
        }
    }

    /// The oldest waiting request `waits` picks, out of the queue.
    fn take(&mut self, waits: impl Fn(&Inflight) -> bool) -> Option<(RequestId, Inflight)> {
        let at = self.inflight.iter().position(|(_, w)| waits(w))?;
        self.inflight.remove(at)
    }

    /// The oldest commit waiting: its request, message and expected set.
    fn take_commit(&mut self) -> Option<(RequestId, String, BTreeSet<String>)> {
        match self.take(|w| matches!(w, Inflight::Commit { .. }))? {
            (id, Inflight::Commit { message, expect }) => Some((id, message, expect)),
            _ => None,
        }
    }

    fn answer(
        &mut self,
        result: &gitdrv::Result<()>,
        waits: impl Fn(&Inflight) -> bool,
    ) -> Option<(RequestId, End)> {
        let (id, _) = self.take(waits)?;
        let end = match result {
            Ok(()) => End::Done(Outcome::default()),
            Err(e) => End::Failed(e.to_string()),
        };
        Some((id, end))
    }

    /// The index matched and git committed (or failed to): the view shows
    /// the commit (its files out of the staged rows, the commit on top of
    /// the stack), the form clears unless its message moved since the
    /// commit left, and then the receipt ends with the commit's sha and
    /// files.
    fn committed(&mut self, result: &gitdrv::Result<NewCommit>) -> Option<(RequestId, End)> {
        let (id, message, _) = self.take_commit()?;
        let commit = match result {
            Ok(commit) => commit,
            Err(e) => return Some((id, End::Failed(e.to_string()))),
        };
        if let Some(git) = &mut self.git {
            git.show_commit(&mut self.workspace, commit, &message);
        }
        let form = &mut self.workspace.commit_form;
        if form.message() == message {
            form.take_message();
            form.hide();
        }
        let mut outcome = Outcome::default();
        outcome
            .result
            .insert("commit".to_owned(), Value::Text(commit.sha.clone()));
        outcome
            .result
            .insert("files".to_owned(), Value::List(commit.files.clone()));
        Some((id, End::Done(outcome)))
    }

    /// The index staged something else than the request observed: nothing
    /// was committed, and the receipt ends rejected with the delta seen at
    /// the index, as a change of `changes:staged`.
    fn refused(&mut self, added: &[String], removed: &[String]) -> Option<(RequestId, End)> {
        let (id, _, expect) = self.take_commit()?;
        let mut found = expect.clone();
        for path in removed {
            found.remove(path);
        }
        found.extend(added.iter().cloned());
        let changes = self
            .staged_ref()
            .map(|at| {
                vec![Change::StateChanged {
                    at,
                    before: Some(Value::Set(expect)),
                    after: Some(Value::Set(found)),
                    added: added.to_vec(),
                    removed: removed.to_vec(),
                }]
            })
            .unwrap_or_default();
        Some((id, End::Refused(Rejection::stale(changes))))
    }

    /// `changes:staged`, the read a commit is checked on.
    fn staged_ref(&self) -> Option<StateRef> {
        let ids = self.workspace.ids.as_ref()?;
        StateRef::parse(&format!("{}:staged", ids.changes)).ok()
    }
}
