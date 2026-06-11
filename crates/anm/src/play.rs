//! anm v0 player (doc/anm-format-v0.md "player contract"): owns a
//! deterministic f32 timeline, driven by [`AnimationTick`] handed in by
//! the runner; FrameClock is the clock, the player never reads a wall
//! clock, so the same tick sequence always produces the same scenes.
//! segments are sampled through plev's `ease()` (via
//! [`crate::easing::Easing::sample`]) and [`Interpolate`], the same
//! curve and lerp library ui animation uses.
//!
//! evaluation is windowed (theatre lesson): each track caches the
//! segment whose validity window `[t0, t1)` contains the playhead plus
//! the value the segment starts from. ticks inside the window never
//! re-search the chain; only crossing a boundary re-derives the cursor,
//! and [`AnmPlayer::segment_searches`] counts those re-derivations so
//! tests can prove it. seek is O(1) in frames (the swf lesson): the
//! governing keyframe snapshot plus direct evaluation of its delta
//! tracks at t, never a per-frame replay. sub-epsilon ticks are dropped
//! whole (thorvg lesson), so a paused-in-all-but-name frame costs
//! nothing and changes nothing.
//!
//! the reactive surface (playing, time) publishes through plev signals;
//! the showcase motion tab binds to those. the player produces scenes,
//! it does not push them: the embedding app pushes `scene()` per frame
//! and the compositor's dirty hash makes unchanged pushes free.

use crate::ir::{IrError, Node, Prop, Timeline, Track, Value};
use crate::lower::{LoweredAsset, lower_scene};
use plev::animation::{AnimationTick, Interpolate};
use plev::compositor::SceneNode;
use plev::signal::{ReadSignal, WriteSignal, create_signal};

/// Ticks advancing the playhead by less than this many seconds are
/// dropped whole: at the format's quantization grid nothing visible can
/// change, so skipping the update beats re-evaluating the scene.
pub const EPSILON_S: f32 = 1e-4;

/// v0 default for a prop never given a value: scalars read 0,
/// colors transparent black (mirrors `crate::lower`).
fn default_value(prop: Prop) -> Value {
    if prop.is_color() {
        Value::Color([0.0; 4])
    } else {
        Value::Scalar(0.0)
    }
}

/// Lerp through plev's [`Interpolate`]; `k` is already eased.
/// `Timeline::validate` guarantees matching kinds on any constructed
/// player, so the mixed arm is unreachable there.
fn lerp_value(from: Value, to: Value, k: f32) -> Value {
    match (from, to) {
        (Value::Scalar(a), Value::Scalar(b)) => Value::Scalar(a.lerp(&b, k)),
        (Value::Color(a), Value::Color(b)) => Value::Color(a.lerp(&b, k)),
        _ => to,
    }
}

/// Window-of-validity cache for one track: the segment under the
/// playhead, its absolute window `[t0, t1)` and the value it starts
/// from. `seg == segments.len()` means past the chain (value = `from`).
#[derive(Clone, Copy)]
struct Cursor {
    valid: bool,
    seg: usize,
    t0: f32,
    t1: f32,
    from: Value,
}

const DEAD: Cursor = Cursor {
    valid: false,
    seg: 0,
    t0: 0.0,
    t1: 0.0,
    from: Value::Scalar(0.0),
};

/// Walk the chain once and cache the segment containing `t`. The from
/// value accumulates exactly like the wire semantics: snapshot value,
/// then each segment's target. Only runs on a window miss.
fn derive_cursor(track: &Track, base: Value, t: f32) -> Cursor {
    let mut from = base;
    let mut start = track.start_t;
    for (i, seg) in track.segments.iter().enumerate() {
        let end = start + seg.dur_s;
        if t < end {
            return Cursor {
                valid: true,
                seg: i,
                t0: start,
                t1: end,
                from,
            };
        }
        from = seg.target;
        start = end;
    }
    Cursor {
        valid: true,
        seg: track.segments.len(),
        t0: start,
        t1: f32::INFINITY,
        from,
    }
}

/// One track bound to its owner keyframe: `node_at` is the node's
/// position in the owner snapshot, resolved once at construction.
struct Binding {
    track: usize,
    node_at: usize,
}

/// D-block semantics (spec decision 2/3): a track acts only while its
/// owner keyframe (last one at or before `start_t`) governs the
/// playhead; the next keyframe's snapshot resets everything. Tracks
/// whose node is absent from the owner snapshot have nothing to modify
/// in that window and are dropped here, once.
fn bind(timeline: &Timeline) -> Vec<Vec<Binding>> {
    let mut by_kf: Vec<Vec<Binding>> = (0..timeline.keyframes.len()).map(|_| Vec::new()).collect();
    for (track, t) in timeline.tracks.iter().enumerate() {
        // validate() guarantees the opening keyframe at t=0.
        let owner = timeline
            .keyframes
            .iter()
            .rposition(|kf| kf.t <= t.start_t)
            .expect("validated timeline opens at t=0");
        let snapshot = &timeline.keyframes[owner].snapshot;
        if let Some(node_at) = snapshot.iter().position(|n| n.id == t.node_id) {
            by_kf[owner].push(Binding { track, node_at });
        }
    }
    by_kf
}

/// Deterministic player over a validated [`Timeline`].
pub struct AnmPlayer {
    timeline: Timeline,
    assets: Vec<LoweredAsset>,
    bindings: Vec<Vec<Binding>>,
    cursors: Vec<Cursor>,
    kf: usize,
    kf_window: (f32, f32),
    kf_valid: bool,
    searches: u64,
    time_v: f32,
    playing_v: bool,
    time_r: ReadSignal<f32>,
    time_w: WriteSignal<f32>,
    playing_r: ReadSignal<bool>,
    playing_w: WriteSignal<bool>,
}

impl AnmPlayer {
    /// Build a player at t=0, paused. The timeline is validated first;
    /// a malformed one is an error, never a panicking player.
    pub fn new(timeline: Timeline) -> Result<Self, IrError> {
        timeline.validate()?;
        let bindings = bind(&timeline);
        let cursors = vec![DEAD; timeline.tracks.len()];
        let (time_r, time_w) = create_signal(0.0f32);
        let (playing_r, playing_w) = create_signal(false);
        Ok(Self {
            timeline,
            assets: Vec::new(),
            bindings,
            cursors,
            kf: 0,
            kf_window: (0.0, 0.0),
            kf_valid: false,
            searches: 0,
            time_v: 0.0,
            playing_v: false,
            time_r,
            time_w,
            playing_r,
            playing_w,
        })
    }

    /// Install the runtime resources for asset-backed nodes, indexed by
    /// `AssetId` like the container's asset table. Nodes whose asset is
    /// missing or mistyped are skipped by lowering, deterministically.
    pub fn set_assets(&mut self, assets: Vec<LoweredAsset>) {
        self.assets = assets;
    }

    pub fn timeline(&self) -> &Timeline {
        &self.timeline
    }

    pub fn duration_s(&self) -> f32 {
        self.timeline.duration_s
    }

    /// Reactive playhead in seconds; the motion ui binds to this.
    pub fn time(&self) -> ReadSignal<f32> {
        self.time_r
    }

    /// Reactive play state; the motion ui binds to this.
    pub fn playing(&self) -> ReadSignal<bool> {
        self.playing_r
    }

    /// Playhead without touching the signal graph (no tracking).
    pub fn current_time(&self) -> f32 {
        self.time_v
    }

    pub fn is_playing(&self) -> bool {
        self.playing_v
    }

    /// Cursor re-derivations (keyframe and segment window misses) since
    /// construction. Diagnostics: ticks inside a validity window add 0.
    pub fn segment_searches(&self) -> u64 {
        self.searches
    }

    /// Start advancing on ticks. Playing a finished timeline replays
    /// from the start.
    pub fn play(&mut self) {
        if self.time_v >= self.timeline.duration_s {
            self.set_time(0.0);
        }
        self.set_playing(true);
    }

    pub fn pause(&mut self) {
        self.set_playing(false);
    }

    /// Jump the playhead to `t` (clamped to `[0, duration]`); play
    /// state is untouched. Non-finite input is ignored. O(1): the next
    /// evaluation snapshots the governing keyframe and evaluates its
    /// tracks at `t` directly.
    pub fn scrub(&mut self, t: f32) {
        if t.is_finite() {
            self.set_time(t.clamp(0.0, self.timeline.duration_s));
        }
    }

    /// Advance by the runner's tick. The player never owns a clock: dt
    /// accumulation is the only time source, which is what makes the
    /// timeline deterministic and scrub-consistent. Reaching the end
    /// clamps to the duration and pauses.
    pub fn tick(&mut self, tick: &AnimationTick) {
        if !self.playing_v || tick.dt < EPSILON_S {
            return;
        }
        let t = (self.time_v + tick.dt).min(self.timeline.duration_s);
        self.set_time(t);
        if t >= self.timeline.duration_s {
            self.set_playing(false);
        }
    }

    /// Lowered scene at the current playhead, ready to push.
    pub fn scene(&mut self) -> Vec<SceneNode> {
        self.scene_at(self.time_v)
    }

    /// Lowered scene at an arbitrary `t` (clamped); the playhead does
    /// not move. Shares the validity-window cache with playback.
    pub fn scene_at(&mut self, t: f32) -> Vec<SceneNode> {
        let t = if t.is_finite() {
            t.clamp(0.0, self.timeline.duration_s)
        } else {
            0.0
        };
        let ir = self.eval_ir(t);
        lower_scene(&ir, &self.assets)
    }

    fn set_time(&mut self, t: f32) {
        if self.time_v.to_bits() != t.to_bits() {
            self.time_v = t;
            self.time_w.set(t);
        }
    }

    fn set_playing(&mut self, on: bool) {
        if self.playing_v != on {
            self.playing_v = on;
            self.playing_w.set(on);
        }
    }

    /// Keyframe under `t`, cached as a window `[kf.t, next.t)`; binary
    /// search only on a miss (counted), O(1) while inside.
    fn seek_keyframe(&mut self, t: f32) {
        if self.kf_valid && t >= self.kf_window.0 && t < self.kf_window.1 {
            return;
        }
        let kfs = &self.timeline.keyframes;
        let i = kfs.partition_point(|kf| kf.t <= t).saturating_sub(1);
        self.kf = i;
        self.kf_window = (kfs[i].t, kfs.get(i + 1).map_or(f32::INFINITY, |k| k.t));
        self.kf_valid = true;
        self.searches += 1;
    }

    /// IR scene at `t`: governing snapshot cloned, owner tracks applied
    /// at their eased values. Replay-free by construction.
    fn eval_ir(&mut self, t: f32) -> Vec<Node> {
        self.seek_keyframe(t);
        let kf = &self.timeline.keyframes[self.kf];
        let mut scene = kf.snapshot.clone();
        for b in &self.bindings[self.kf] {
            let track = &self.timeline.tracks[b.track];
            if t < track.start_t {
                continue; // chain not started: snapshot value governs
            }
            let cur = &mut self.cursors[b.track];
            if !cur.valid || t < cur.t0 || t >= cur.t1 {
                let base = kf.snapshot[b.node_at]
                    .props
                    .get(track.prop)
                    .unwrap_or_else(|| default_value(track.prop));
                *cur = derive_cursor(track, base, t);
                self.searches += 1;
            }
            let value = if cur.seg == track.segments.len() {
                cur.from
            } else {
                let seg = &track.segments[cur.seg];
                let k = seg.easing.sample((t - cur.t0) / seg.dur_s);
                lerp_value(cur.from, seg.target, k)
            };
            scene[b.node_at].props.set(track.prop, value);
        }
        scene
    }
}
