//! the recorder's state behind its lock: the semantic track, the last
//! state the app gave (for the next diff and to know what is secret), the
//! requests still open, and the counters. every entry point takes the
//! caller's time; the recorder never reads a clock.

use std::collections::HashMap;
use std::sync::Arc;

use sem::{Actor, Change, Millis, Receipt, Request, RequestId, Revision, SemId, Snapshot, Stage};

use crate::event::{Event, InputKind, Kind};
use crate::limits::{Limits, Stats};
use crate::redact;
use crate::session::Session;
use crate::track::Track;

#[derive(Debug, Default)]
pub(crate) struct Inner {
    limits: Limits,
    track: Track,
    last_t: Option<Millis>,
    /// the last snapshot the app gave: the state now.
    state: Option<Arc<Snapshot>>,
    /// the request of every receipt still open, shared by its stages.
    open: HashMap<(u16, RequestId), Arc<Request>>,
    origins: u16,
    stats: Stats,
}

/// true for a change a `Kind::Changes` cannot carry: it needs the new
/// node or the new order, which a `Change` does not hold.
fn structural(c: &Change) -> bool {
    matches!(
        c,
        Change::NodeAdded(_) | Change::ParentChanged { .. } | Change::Reordered { .. }
    )
}

impl Inner {
    pub(crate) fn new(limits: Limits) -> Self {
        Inner {
            limits,
            ..Inner::default()
        }
    }

    pub(crate) fn limits(&self) -> Limits {
        self.limits
    }

    /// the next dispatcher's origin number.
    pub(crate) fn origin(&mut self) -> u16 {
        let origin = self.origins;
        self.origins = self.origins.wrapping_add(1);
        origin
    }

    /// the time to record `t` at: never before the last one.
    fn clock(&mut self, t: Millis) -> Millis {
        match self.last_t {
            Some(last) if t < last => {
                self.stats.clamped += 1;
                last
            }
            _ => {
                self.last_t = Some(t);
                t
            }
        }
    }

    /// true when the last segment opened `full_every` ago or more.
    fn due(&self, t: Millis) -> bool {
        self.track
            .last_start()
            .is_none_or(|start| t.saturating_sub(start) >= self.limits.full_every)
    }

    /// opens a segment at a full snapshot; `snapshot` is the state now.
    fn key(&mut self, t: Millis, snapshot: Arc<Snapshot>, changes: Vec<Change>) {
        self.state = Some(snapshot.clone());
        self.track.key(Event {
            t,
            kind: Kind::Snapshot { snapshot, changes },
        });
        self.stats.events += 1;
    }

    /// an event that is not a snapshot: the track is re-keyed first when
    /// the cadence is due, with the state as it stands.
    fn push(&mut self, t: Millis, kind: Kind) {
        if self.due(t) {
            if let Some(state) = self.state.clone() {
                self.key(t, state, Vec::new());
                self.stats.rekeys += 1;
            }
        }
        self.track.push(Event { t, kind });
        self.stats.events += 1;
    }

    pub(crate) fn observed(
        &mut self,
        t: Millis,
        actor: Actor,
        scope: Option<SemId>,
        revision: Revision,
    ) {
        let t = self.clock(t);
        self.push(
            t,
            Kind::Observed {
                actor,
                scope,
                revision,
            },
        );
    }

    /// records the state the app projected at `t`: whole when it is the
    /// first, when it adds, moves or reorders nodes, when the cadence is
    /// due or when `whole` asks for it; otherwise as its changes. a
    /// snapshot equal to the state before is not stored, unless asked.
    pub(crate) fn snapshot(&mut self, t: Millis, snapshot: &Snapshot, whole: bool) {
        let t = self.clock(t);
        let changes = match &self.state {
            Some(before) => Snapshot::diff(before, snapshot),
            None => Vec::new(),
        };
        let first = self.state.is_none();
        if !first && !whole && changes.is_empty() {
            self.stats.snapshots_unchanged += 1;
            return;
        }
        let now = Arc::new(snapshot.clone());
        if first || whole || self.due(t) || changes.iter().any(structural) {
            self.key(t, now, changes);
            self.stats.snapshots_full += 1;
        } else {
            self.state = Some(now);
            self.track.push(Event {
                t,
                kind: Kind::Changes(changes),
            });
            self.stats.events += 1;
            self.stats.snapshots_changed += 1;
        }
    }

    /// records one stage a dispatcher reports. the request is redacted
    /// against the state now and shared by every stage of its receipt.
    pub(crate) fn stage(&mut self, origin: u16, receipt: &Receipt, stage: &Stage) {
        let at = receipt.stages.last().map_or(0, |(_, t)| *t);
        let t = self.clock(at);
        let key = (origin, receipt.id);
        let state = self.state.as_deref();
        let request = match self.open.get(&key) {
            Some(request) => request.clone(),
            None => {
                let request = Arc::new(redact::request(&receipt.request, state));
                self.open.insert(key, request.clone());
                request
            }
        };
        let stage = redact::stage(stage, &receipt.request.target, state);
        if stage.is_terminal() {
            self.open.remove(&key);
        }
        self.push(
            t,
            Kind::Stage {
                origin,
                id: receipt.id,
                request,
                stage,
                validated_against: receipt.validated_against,
            },
        );
    }

    pub(crate) fn input(
        &mut self,
        t: Millis,
        kind: InputKind,
        at: Option<[f32; 2]>,
        target: Option<SemId>,
    ) {
        let t = self.clock(t);
        self.push(t, Kind::Input { kind, at, target });
    }

    pub(crate) fn stats(&self) -> Stats {
        Stats {
            segments: self.track.segments().len(),
            semantic_bytes: self.track.bytes(),
            ..self.stats
        }
    }

    /// a copy of what the ring holds now, to read.
    pub(crate) fn window(&self) -> Session {
        Session::from_events(self.track.events().cloned().collect())
    }
}
