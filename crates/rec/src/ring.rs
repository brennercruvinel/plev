//! the recorder's state behind its lock: the semantic track, the last
//! state the app gave (for the next diff and to know what is secret), the
//! requests still open, and the counters. every entry point takes the
//! caller's time; the recorder never reads a clock.
//!
//! the ring: after every event and frame the oldest segments and chunks
//! go while the limits ask for it, whole, so what stays opens at a full
//! snapshot and at a visual keyframe. by time, a segment or a chunk goes
//! once the one after it opened at or before `now - max_age`, which keeps
//! at least the last `max_age`. by bytes, the older of the two oldest
//! goes while the ring is over `max_bytes`; when only the newest of each
//! is left and still over, the next event re-keys and the next frame
//! opens a chunk, so the ring can cut there.

use std::collections::HashMap;
use std::sync::Arc;

use sem::redact::Secrets;
use sem::{Actor, Change, Millis, Receipt, Request, RequestId, Revision, SemId, Snapshot, Stage};

use engine::compositor::SceneNode;

use crate::error::FrameError;
use crate::event::{Event, InputKind, Kind};
use crate::limits::{Limits, Stats};
use crate::screen::Screen;
use crate::session::Session;
use crate::track::Track;
use crate::visual::{Cadence, Visual};

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
    /// a limit could not cut: the next event re-keys the track.
    rekey: bool,
    visual: Visual,
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

    /// true when the last segment opened `full_every` ago or more, or a
    /// limit waits for a keyframe to cut at.
    fn due(&self, t: Millis) -> bool {
        self.rekey
            || self
                .track
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
        self.rekey = false;
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
        self.evict();
    }

    /// the estimated bytes the ring holds.
    fn bytes(&self) -> usize {
        self.track.bytes() + self.visual.bytes()
    }

    /// drops what the limits no longer keep: whole segments and whole
    /// chunks, oldest first.
    fn evict(&mut self) {
        let Some(now) = self.last_t else {
            return;
        };
        let cut = now.saturating_sub(self.limits.max_age);
        while self.track.second_start().is_some_and(|start| start <= cut) {
            self.track.pop_front();
            self.stats.evicted_segments += 1;
        }
        while self.track.first_t().is_some_and(|t| t < cut) && self.track.trim_unkeyed() {
            self.stats.evicted_unkeyed += 1;
        }
        while self.visual.second_start().is_some_and(|start| start <= cut) {
            self.visual.pop_front();
            self.stats.evicted_chunks += 1;
        }
        while self.bytes() > self.limits.max_bytes {
            // the older of the two fronts that can go.
            let segment = (self.track.segments().len() > 1)
                .then(|| self.track.first_t())
                .flatten();
            let chunk = self
                .visual
                .second_start()
                .and_then(|_| self.visual.first_start());
            match (segment, chunk) {
                (Some(s), Some(c)) if c < s => self.drop_chunk(),
                (Some(_), _) => {
                    self.track.pop_front();
                    self.stats.evicted_segments += 1;
                }
                (None, Some(_)) => self.drop_chunk(),
                (None, None) if self.track.trim_unkeyed() => self.stats.evicted_unkeyed += 1,
                (None, None) => {
                    self.rekey = true;
                    self.visual.close_soon();
                    self.stats.over_budget += 1;
                    break;
                }
            }
        }
    }

    fn drop_chunk(&mut self) {
        self.visual.pop_front();
        self.stats.evicted_chunks += 1;
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
        self.evict();
    }

    /// records one stage a dispatcher reports. the request is redacted
    /// against the state now (`sem::redact`, the rule the agent port
    /// applies too; no state, or a node the state does not hold, counts as
    /// secret) and shared by every stage of its receipt.
    pub(crate) fn stage(&mut self, origin: u16, receipt: &Receipt, stage: &Stage) {
        let at = receipt.stages.last().map_or(0, |(_, t)| *t);
        let t = self.clock(at);
        let key = (origin, receipt.id);
        let secrets = Secrets::of(&receipt.request, self.state.as_deref());
        let request = match self.open.get(&key) {
            Some(request) => request.clone(),
            None => {
                let request = Arc::new(secrets.request(&receipt.request));
                self.open.insert(key, request.clone());
                request
            }
        };
        let stage = secrets.stage(&receipt.request.target, stage);
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

    /// takes the frame on screen at `t`, flattened in composite order;
    /// `approximated` counts the layers whose look the IR does not carry.
    pub(crate) fn frame<'a>(
        &mut self,
        t: Millis,
        scene: impl Iterator<Item = &'a SceneNode> + Clone,
        approximated: u64,
    ) -> Result<(), FrameError> {
        let t = self.clock(t);
        let cadence = Cadence {
            every: self.limits.chunk_every,
            frames: self.limits.chunk_frames,
        };
        let taken = self.visual.frame(t, scene, cadence);
        self.stats.layers_approximated += approximated;
        self.evict();
        taken
    }

    pub(crate) fn stats(&self) -> Stats {
        let counts = self.visual.counts;
        Stats {
            frames: counts.frames,
            frames_unchanged: counts.unchanged,
            frame_errors: counts.errors,
            skipped: counts.skipped,
            chunks_closed: counts.closed,
            segments: self.track.segments().len(),
            chunks: self.visual.chunk_count(),
            semantic_bytes: self.track.bytes(),
            visual_bytes: self.visual.bytes(),
            asset_bytes: self.visual.asset_bytes(),
            bytes: self.bytes(),
            ..self.stats
        }
    }

    /// a copy of what the ring holds now, to read.
    pub(crate) fn window(&self) -> Session {
        Session::from_parts(
            self.track.events().cloned().collect(),
            Screen::of(&self.visual),
        )
    }
}
