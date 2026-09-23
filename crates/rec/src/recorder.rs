//! `Recorder`, the handle an app records through, and `Sink`, the
//! `sem::ReceiptSink` a dispatcher reports its stages to. both share one
//! ring behind a lock that is held only inside a recorder call, never
//! across the app's code, so no call can find it taken by the same thread.

use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use engine::compositor::{Compositor, Layer, SceneNode};
use sem::{Actor, Millis, Receipt, ReceiptSink, Revision, SemId, Snapshot, Stage};

use crate::error::{FileError, FrameError};
use crate::event::InputKind;
use crate::limits::{Limits, Stats};
use crate::persist::Persisted;
use crate::ring::Inner;
use crate::session::Session;

/// records one app session as data: what each actor observed, the state
/// the app projected, every stage of every receipt, a person's input, and
/// the frames on screen.
///
/// it only reads: nothing it is given is changed, no request is created,
/// nothing is executed. times are the caller's (`sem` never reads a clock
/// and neither does the recorder), so a test drives it with its own.
/// cloning the handle shares the recording.
#[derive(Clone, Debug)]
pub struct Recorder {
    inner: Arc<Mutex<Inner>>,
}

/// the lock, even after a panic elsewhere left it poisoned: the ring is
/// plain data and every call leaves it consistent.
fn lock(inner: &Mutex<Inner>) -> MutexGuard<'_, Inner> {
    inner.lock().unwrap_or_else(PoisonError::into_inner)
}

impl Default for Recorder {
    fn default() -> Self {
        Recorder::new(Limits::default())
    }
}

impl Recorder {
    pub fn new(limits: Limits) -> Self {
        Recorder {
            inner: Arc::new(Mutex::new(Inner::new(limits))),
        }
    }

    pub fn limits(&self) -> Limits {
        lock(&self.inner).limits()
    }

    /// a sink for one dispatcher (`Dispatcher::add_sink(Box::new(..))`).
    /// each sink gets its own origin number, so the request ids of two
    /// dispatchers never mix in the recording.
    pub fn sink(&self) -> Sink {
        let origin = lock(&self.inner).origin();
        Sink {
            inner: self.inner.clone(),
            origin,
        }
    }

    /// `actor` observed the app at `t`: the whole snapshot, or the subtree
    /// under `scope`, at `revision`.
    pub fn observed(&self, t: Millis, actor: Actor, scope: Option<SemId>, revision: Revision) {
        lock(&self.inner).observed(t, actor, scope, revision);
    }

    /// the state the app projected at `t`. call it whenever the app
    /// projects (each frame, after an action): an unchanged state costs a
    /// diff and nothing is stored; a changed one is stored as its changes,
    /// or whole on the cadence and when nodes are added, moved or
    /// reordered.
    pub fn snapshot(&self, t: Millis, snapshot: &Snapshot) {
        lock(&self.inner).snapshot(t, snapshot, false);
    }

    /// the same, always stored whole: a keyframe on demand.
    pub fn snapshot_full(&self, t: Millis, snapshot: &Snapshot) {
        lock(&self.inner).snapshot(t, snapshot, true);
    }

    /// a person's input at `t`: its kind (never its content), where it
    /// landed in logical window pixels, and the control bound under it
    /// (`Compositor::hit`).
    pub fn input(&self, t: Millis, kind: InputKind, at: Option<[f32; 2]>, target: Option<SemId>) {
        lock(&self.inner).input(t, kind, at, target);
    }

    /// the frame on screen at `t`, as the compositor holds it after the
    /// app built it: the visible layers in composite order (z order, then
    /// creation), their nodes flattened in push order. call it once per
    /// rendered frame. a layer's opacity, effects and clip rect are not in
    /// the monster IR v0: its nodes are recorded as drawn inside it and
    /// `Stats::layers_approximated` counts the layer.
    pub fn frame(&self, t: Millis, compositor: &Compositor) -> Result<(), FrameError> {
        let mut layers: Vec<&Layer> = compositor
            .layers()
            .iter()
            .filter(|l| l.visible && l.opacity > 0.0)
            .collect();
        layers.sort_by_key(|l| l.z_order);
        let approximated = layers
            .iter()
            .filter(|l| l.opacity < 1.0 || l.has_effects() || l.clip_rect.is_some())
            .count();
        let scene = layers.iter().flat_map(|l| l.nodes().iter());
        let approximated = u64::try_from(approximated).unwrap_or(u64::MAX);
        lock(&self.inner).frame(t, scene, approximated)
    }

    /// a frame given as its nodes in draw order, for a scene that does
    /// not live in a compositor.
    pub fn frame_scene(&self, t: Millis, scene: &[SceneNode]) -> Result<(), FrameError> {
        lock(&self.inner).frame(t, scene.iter(), 0)
    }

    /// a copy of what the ring holds now, to read back.
    pub fn snapshot_window(&self) -> Session {
        lock(&self.inner).window()
    }

    /// writes the retained window to `path` as a session `.monster` file
    /// (magic `MONS`). the window is copied under the lock and written
    /// after it, so recording goes on while the file is written.
    pub fn persist(&self, path: impl AsRef<Path>) -> Result<Persisted, FileError> {
        self.snapshot_window().write(path)
    }

    pub fn stats(&self) -> Stats {
        lock(&self.inner).stats()
    }
}

/// the receipt sink of one dispatcher. it records each stage as it
/// happens, at the time the dispatcher's caller gave for it.
#[derive(Debug)]
pub struct Sink {
    inner: Arc<Mutex<Inner>>,
    origin: u16,
}

impl Sink {
    /// the origin number the stages of this sink carry.
    pub fn origin(&self) -> u16 {
        self.origin
    }
}

impl ReceiptSink for Sink {
    fn on_stage(&mut self, receipt: &Receipt, stage: &Stage) {
        lock(&self.inner).stage(self.origin, receipt, stage);
    }
}
