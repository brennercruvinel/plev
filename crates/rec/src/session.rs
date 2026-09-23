//! `Session`, the read side: a recorded window as plain data, read by
//! time. it answers what happened and never makes anything happen again:
//! it holds no dispatcher and no app, it takes nothing it could run, and
//! every answer is a copy or a borrow of what was recorded. replay never
//! creates a request (docs/ADR/session-recording-replay-not-reexecution.md).

use std::collections::HashMap;

use engine::compositor::SceneNode;
use sem::{Millis, Receipt, RequestId, SemId, Stage};

use crate::error::ReadError;
use crate::event::{Event, Kind};
use crate::format::Build;
use crate::screen::Screen;
use crate::state::State;

/// a recorded window, read by time: the semantic track (events) and the
/// visual track (frames). the semantic track reads without decoding a
/// single frame.
#[derive(Clone, Debug, PartialEq)]
pub struct Session {
    events: Vec<Event>,
    /// indices of the full snapshots, the semantic keyframes, in order.
    keys: Vec<usize>,
    screen: Screen,
    build: Build,
}

impl Default for Session {
    fn default() -> Self {
        Session::from_events(Vec::new())
    }
}

/// one receipt as the window holds it at some time: the stages recorded
/// up to then. `whole` is false when the window starts after the receipt
/// did (its first stages were cut); the request is still whole, since
/// every stage carries it.
#[derive(Clone, Debug, PartialEq)]
pub struct ReceiptAt {
    /// the dispatcher it came from (see `Recorder::sink`).
    pub origin: u16,
    pub receipt: Receipt,
    pub whole: bool,
}

impl Session {
    /// a session over recorded events, in the order they were recorded,
    /// without frames.
    pub fn from_events(events: Vec<Event>) -> Session {
        Session::from_parts(events, Screen::default())
    }

    /// a window of this build of the recorder.
    pub(crate) fn from_parts(events: Vec<Event>, screen: Screen) -> Session {
        Session::from_file(events, screen, Build::current())
    }

    /// a window some build wrote.
    pub(crate) fn from_file(events: Vec<Event>, screen: Screen, build: Build) -> Session {
        let keys = events
            .iter()
            .enumerate()
            .filter(|(_, e)| matches!(e.kind, Kind::Snapshot { .. }))
            .map(|(at, _)| at)
            .collect();
        Session {
            events,
            keys,
            screen,
            build,
        }
    }

    pub(crate) fn screen(&self) -> &Screen {
        &self.screen
    }

    /// the build of the recorder that recorded the window: its crate
    /// version and the git commit it was built from (for a window read
    /// from a file, the one in its header).
    pub fn build(&self) -> &Build {
        &self.build
    }

    /// the first time the window holds, events and frames together, on
    /// the recorder's clock; 0 for an empty window. a session file carries
    /// it in its header.
    pub fn start(&self) -> Millis {
        [self.span().map(|s| s.0), self.frames_span().map(|s| s.0)]
            .into_iter()
            .flatten()
            .min()
            .unwrap_or(0)
    }

    /// what was on screen at `t`: the frame taken last at or before `t`,
    /// played back from the visual track and lowered to compositor nodes
    /// (the kinds the monster IR v0 carries; see `Stats::skipped`).
    /// `None` before the first frame the window holds.
    pub fn scene_at(&self, t: Millis) -> Result<Option<Vec<SceneNode>>, ReadError> {
        self.screen.scene_at(t)
    }

    /// the time of the frame on screen at `t`.
    pub fn frame_at(&self, t: Millis) -> Option<Millis> {
        self.screen.frame_at(t)
    }

    /// every frame time the window holds, in order.
    pub fn frame_times(&self) -> Vec<Millis> {
        self.screen.times()
    }

    /// the first and the last frame time the window holds.
    pub fn frames_span(&self) -> Option<(Millis, Millis)> {
        self.screen.span()
    }

    /// the visual keyframes: where each chunk of frames starts. a time
    /// reads from the chunk that starts last at or before it.
    pub fn frame_keys(&self) -> Vec<Millis> {
        self.screen.keys()
    }

    /// the encoded size of each chunk of frames and how many assets it
    /// references.
    pub fn chunk_sizes(&self) -> Vec<(usize, usize)> {
        self.screen.sizes()
    }

    /// every event, in order.
    pub fn events(&self) -> &[Event] {
        &self.events
    }

    /// the events recorded at or before `t`.
    pub fn until(&self, t: Millis) -> &[Event] {
        &self.events[..self.upto(t)]
    }

    /// the events recorded from `t0` to `t1`, both included.
    pub fn between(&self, t0: Millis, t1: Millis) -> &[Event] {
        let from = self.events.partition_point(|e| e.t < t0);
        let to = self.upto(t1).max(from);
        &self.events[from..to]
    }

    /// the first and the last time the window holds.
    pub fn span(&self) -> Option<(Millis, Millis)> {
        Some((self.events.first()?.t, self.events.last()?.t))
    }

    /// how many events come at or before `t`.
    fn upto(&self, t: Millis) -> usize {
        self.events.partition_point(|e| e.t <= t)
    }

    /// the index of the full snapshot that governs `t`: the last one at or
    /// before it.
    fn key_at(&self, t: Millis) -> Option<usize> {
        let n = self
            .keys
            .partition_point(|&k| self.events.get(k).is_some_and(|e| e.t <= t));
        n.checked_sub(1).and_then(|i| self.keys.get(i)).copied()
    }

    /// the full snapshot that governs `t`, found by a search over the
    /// keyframes only.
    pub fn keyframe_at(&self, t: Millis) -> Option<&Event> {
        self.events.get(self.key_at(t)?)
    }

    /// the semantic state at `t`: the governing full snapshot with every
    /// change recorded after it, up to and at `t`, applied. `None` before
    /// the first full snapshot of the window.
    pub fn state_at(&self, t: Millis) -> Result<Option<State>, ReadError> {
        let Some(key) = self.key_at(t) else {
            return Ok(None);
        };
        let Some(Kind::Snapshot { snapshot, .. }) = self.events.get(key).map(|e| &e.kind) else {
            return Ok(None);
        };
        let mut state = State::of(snapshot);
        for event in self.events.get(key + 1..self.upto(t)).into_iter().flatten() {
            if let Kind::Changes(changes) = &event.kind {
                state.apply(changes)?;
            }
        }
        Ok(Some(state))
    }

    /// every receipt with a stage at or before `t`, in the order of their
    /// first recorded stage.
    pub fn receipts(&self, t: Millis) -> Vec<ReceiptAt> {
        let mut out: Vec<ReceiptAt> = Vec::new();
        let mut index: HashMap<(u16, RequestId), usize> = HashMap::new();
        for event in self.until(t) {
            let Kind::Stage {
                origin,
                id,
                request,
                stage,
                validated_against,
            } = &event.kind
            else {
                continue;
            };
            match index.get(&(*origin, *id)).and_then(|&at| out.get_mut(at)) {
                Some(seen) => {
                    seen.receipt.stages.push((stage.clone(), event.t));
                    seen.receipt.validated_against = *validated_against;
                }
                None => {
                    index.insert((*origin, *id), out.len());
                    out.push(ReceiptAt {
                        origin: *origin,
                        receipt: Receipt {
                            id: *id,
                            request: (**request).clone(),
                            validated_against: *validated_against,
                            stages: vec![(stage.clone(), event.t)],
                        },
                        whole: *stage == Stage::Requested,
                    });
                }
            }
        }
        out
    }

    /// the receipts touching `id` up to `t`: those whose request targets
    /// it or observed a state of it.
    pub fn receipts_for(&self, id: &SemId, t: Millis) -> Vec<ReceiptAt> {
        let mut out = self.receipts(t);
        out.retain(|r| {
            let req = &r.receipt.request;
            req.target == *id || req.observed.values.iter().any(|(at, _)| at.node == *id)
        });
        out
    }
}
