//! `Inspector`: a borrowed session with the receipts it holds. everything
//! else reads from here.

use rec::{ReceiptAt, Session};
use sem::{Millis, RequestId};

/// an instant of the session: `t` on the recorder's clock, `at` in
/// milliseconds from the start of the session (the time every view
/// prints).
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Stamp {
    pub t: Millis,
    pub at: Millis,
}

/// one receipt of the window.
#[derive(Clone, Debug)]
pub(crate) struct Tracked {
    pub(crate) at: ReceiptAt,
}

impl Tracked {
    pub(crate) fn key(&self) -> (u16, RequestId) {
        (self.at.origin, self.at.receipt.id)
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
        let tracked = session
            .receipts(Millis::MAX)
            .into_iter()
            .map(|at| Tracked { at })
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
}
