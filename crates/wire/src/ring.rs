//! `Ring`, the last revisions the app served, so a later message can rest
//! on what an agent saw: `since` rebuilds a delta from it, `from_revision`
//! takes the action's declaration from it, and the observed values of an
//! act are read in it, in the snapshots of the observations that showed
//! them to the agent (`Shown`).

use std::collections::VecDeque;

use sem::{Revision, Snapshot};

/// how many served revisions a port keeps unless told otherwise.
pub const DEFAULT_RING: usize = 16;

/// the whole snapshots last served, newest last, at most `cap` of them.
/// serving a revision the ring holds makes it the newest again; serving a
/// new one past the cap evicts the oldest. a revision out of the ring is
/// unknown, and the agent observes again.
#[derive(Clone, Debug)]
pub struct Ring {
    cap: usize,
    served: VecDeque<(Revision, Snapshot)>,
}

impl Default for Ring {
    fn default() -> Self {
        Ring::new(DEFAULT_RING)
    }
}

impl Ring {
    /// a ring of `cap` revisions; a cap of zero keeps one, since the
    /// revision just served must be usable by the next act.
    pub fn new(cap: usize) -> Self {
        let cap = cap.max(1);
        Ring {
            cap,
            served: VecDeque::with_capacity(cap),
        }
    }

    pub fn cap(&self) -> usize {
        self.cap
    }

    pub fn len(&self) -> usize {
        self.served.len()
    }

    pub fn is_empty(&self) -> bool {
        self.served.is_empty()
    }

    /// records `snapshot` as served now and returns its revision.
    pub fn serve(&mut self, snapshot: Snapshot) -> Revision {
        let revision = snapshot.revision();
        if let Some(at) = self.served.iter().position(|(r, _)| *r == revision) {
            self.served.remove(at);
        }
        self.served.push_back((revision, snapshot));
        while self.served.len() > self.cap {
            self.served.pop_front();
        }
        revision
    }

    /// the snapshot served under `revision`, if the ring still holds it.
    pub fn get(&self, revision: Revision) -> Option<&Snapshot> {
        self.served
            .iter()
            .rev()
            .find(|(r, _)| *r == revision)
            .map(|(_, s)| s)
    }

    /// the revisions held, oldest first.
    pub fn revisions(&self) -> impl Iterator<Item = Revision> + '_ {
        self.served.iter().map(|(r, _)| *r)
    }
}
