use crate::ir::{IrError, Node, Timeline};
use crate::lower::{LoweredAsset, lower_scene};
use crate::play_eval::{
    Cursor, DEAD, KfPlan, apply_ops, default_value, derive_cursor, plan, sample,
};
use engine::animation::AnimationTick;
use engine::compositor::SceneNode;
use engine::signal::{ReadSignal, WriteSignal, create_signal};

/// Ticks advancing the playhead by less than this many seconds are
/// dropped whole: at the format's quantization grid nothing visible can
/// change, so skipping the update beats re-evaluating the scene.
pub const EPSILON_S: f32 = 1e-4;

/// Deterministic player over a validated [`Timeline`].
pub struct MonsterPlayer {
    timeline: Timeline,
    assets: Vec<LoweredAsset>,
    plans: Vec<KfPlan>,
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

impl MonsterPlayer {
    /// Build a player at t=0, paused. The timeline is validated first;
    /// a malformed one is an error, never a panicking player.
    pub fn new(timeline: Timeline) -> Result<Self, IrError> {
        timeline.validate()?;
        let plans = plan(&timeline);
        let cursors = vec![DEAD; timeline.tracks.len()];
        let (time_r, time_w) = create_signal(0.0f32);
        let (playing_r, playing_w) = create_signal(false);
        Ok(Self {
            timeline,
            assets: Vec::new(),
            plans,
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
    /// Segments owning structural ops re-derive their tracks on every
    /// evaluation (the scene there is replayed, not cached) and count
    /// each derivation.
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

    /// IR scene at `t`: governing snapshot cloned, the segment's
    /// structural ops up to `t` replayed (current segment only, so seek
    /// stays O(1) in frames), owner tracks applied at their eased
    /// values. A segment without ops keeps the pure windowed path.
    fn eval_ir(&mut self, t: f32) -> Vec<Node> {
        self.seek_keyframe(t);
        let kf = &self.timeline.keyframes[self.kf];
        let plan = &self.plans[self.kf];
        let mut scene = kf.snapshot.clone();
        if plan.ops.is_empty() {
            for b in &plan.bindings {
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
                let value = sample(track, &self.cursors[b.track], t);
                scene[b.node_at].props.set(track.prop, value);
            }
            return scene;
        }
        apply_ops(&mut scene, &self.timeline, &plan.ops, t);
        // ops may have shifted, swapped or dropped nodes: resolve by id
        // against the pre-track scene, so staggered chains on one prop
        // all derive from the node's structural value, as in the fast
        // path they derive from the snapshot.
        let base_scene = scene.clone();
        for b in &plan.bindings {
            let track = &self.timeline.tracks[b.track];
            if t < track.start_t {
                continue;
            }
            let Some(at) = base_scene.iter().position(|n| n.id == track.node_id) else {
                continue; // node removed (or never placed): nothing to modify
            };
            let base = base_scene[at]
                .props
                .get(track.prop)
                .unwrap_or_else(|| default_value(track.prop));
            let cur = derive_cursor(track, base, t);
            self.searches += 1;
            scene[at].props.set(track.prop, sample(track, &cur, t));
        }
        scene
    }
}
