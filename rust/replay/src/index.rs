//! `Inspector`: a borrowed session with the receipts it holds indexed by
//! the events their stages came in, and the state after any number of
//! events. everything else reads from here.

use std::collections::HashMap;

use accord::{Millis, RequestId, Stage};
use record::{Kind, ReadError, ReceiptAt, Session, State};

/// an instant of the session: `t` on the recorder's clock, `at` in
/// milliseconds from the start of the session (the time every view
/// prints).
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Stamp {
    pub t: Millis,
    pub at: Millis,
}

/// one receipt of the window, with the index of the event each of its
/// stages came in, in the same order as `at.receipt.stages`.
#[derive(Clone, Debug)]
pub(crate) struct Tracked {
    pub(crate) at: ReceiptAt,
    pub(crate) index: Vec<usize>,
}

impl Tracked {
    pub(crate) fn key(&self) -> (u16, RequestId) {
        (self.at.origin, self.at.receipt.id)
    }

    /// the index of the first stage the window holds.
    pub(crate) fn first(&self) -> Option<usize> {
        self.index.first().copied()
    }

    /// true when its last recorded stage is a refusal: at the check, at a
    /// confirmation, or at the point of truth, nothing it asked for ran.
    pub(crate) fn refused(&self) -> bool {
        matches!(self.at.receipt.stage(), Some(Stage::Rejected(_)))
    }

    /// the stages that say it ran, with the index of their event and
    /// their time: executing, completed, failed.
    pub(crate) fn ran(&self) -> impl Iterator<Item = (usize, &Stage, Millis)> {
        self.at
            .receipt
            .stages
            .iter()
            .zip(&self.index)
            .filter(|((stage, _), _)| {
                matches!(
                    stage,
                    Stage::Executing | Stage::Completed(_) | Stage::Failed { .. }
                )
            })
            .map(|((stage, t), i)| (*i, stage, *t))
    }
}

/// reads a session and answers the thesis questions about it. it borrows
/// the session and holds nothing that could run an action.
#[derive(Clone, Debug)]
pub struct Inspector<'s> {
    session: &'s Session,
    start: Millis,
    end: Millis,
    tracked: Vec<Tracked>,
}

impl<'s> Inspector<'s> {
    pub fn new(session: &'s Session) -> Inspector<'s> {
        let mut index: HashMap<(u16, RequestId), Vec<usize>> = HashMap::new();
        for (i, event) in session.events().iter().enumerate() {
            if let Kind::Stage { origin, id, .. } = &event.kind {
                index.entry((*origin, *id)).or_default().push(i);
            }
        }
        let tracked = session
            .receipts(Millis::MAX)
            .into_iter()
            .map(|at| {
                let index = index
                    .remove(&(at.origin, at.receipt.id))
                    .unwrap_or_default();
                Tracked { at, index }
            })
            .collect();
        let start = session.start();
        let end = [
            session.span().map(|s| s.1),
            session.frames_span().map(|s| s.1),
        ]
        .into_iter()
        .flatten()
        .max()
        .unwrap_or(start);
        Inspector {
            session,
            start,
            end,
            tracked,
        }
    }

    pub fn session(&self) -> &'s Session {
        self.session
    }

    /// the first instant the session holds, on the recorder's clock.
    pub fn start(&self) -> Millis {
        self.start
    }

    /// the last instant the session holds, events and frames together.
    pub fn end(&self) -> Millis {
        self.end
    }

    /// `t` as an instant of this session.
    pub fn stamp(&self, t: Millis) -> Stamp {
        Stamp {
            t,
            at: t.saturating_sub(self.start),
        }
    }

    /// every request the window holds, as (origin, id), in the order of
    /// their first recorded stage.
    pub fn requests(&self) -> Vec<(u16, RequestId)> {
        self.tracked.iter().map(Tracked::key).collect()
    }

    /// the origins (one per dispatcher the recorder had a sink on) that
    /// hold a request with this id.
    pub fn origins_of(&self, id: RequestId) -> Vec<u16> {
        let mut out: Vec<u16> = self
            .tracked
            .iter()
            .filter(|t| t.at.receipt.id == id)
            .map(|t| t.at.origin)
            .collect();
        out.sort_unstable();
        out.dedup();
        out
    }

    pub(crate) fn tracked(&self) -> &[Tracked] {
        &self.tracked
    }

    pub(crate) fn find(&self, origin: u16, id: RequestId) -> Option<&Tracked> {
        self.tracked.iter().find(|t| t.key() == (origin, id))
    }

    /// the semantic state the first `n` events leave: the full snapshot
    /// that governs them with the changes after it applied, by the
    /// session's own reader over that stretch of events. `None` before the
    /// first full snapshot. unlike `Session::state_at`, which reads by
    /// time, this splits events recorded in the same millisecond, so the
    /// state right before a request never holds that request's own effect.
    pub(crate) fn state_upto(&self, n: usize) -> Result<Option<State>, ReadError> {
        let events = self.session.events();
        let n = n.min(events.len());
        let Some(key) = events
            .get(..n)
            .unwrap_or_default()
            .iter()
            .rposition(|e| matches!(e.kind, Kind::Snapshot { .. }))
        else {
            return Ok(None);
        };
        let stretch = events.get(key..n).unwrap_or_default().to_vec();
        Session::from_events(stretch).state_at(Millis::MAX)
    }

    /// true for an event that sets the state: a full snapshot or changes.
    pub(crate) fn sets_state(&self, i: usize) -> bool {
        self.session
            .events()
            .get(i)
            .is_some_and(|e| matches!(e.kind, Kind::Snapshot { .. } | Kind::Changes(_)))
    }

    /// the index of the last event before `i` that set the state.
    pub(crate) fn state_before(&self, i: usize) -> Option<usize> {
        (0..i).rev().find(|&j| self.sets_state(j))
    }
}
