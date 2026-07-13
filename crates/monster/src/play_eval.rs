//! evaluation internals of [`crate::play::MonsterPlayer`]: per-track
//! validity-window cursors, per-keyframe plans binding tracks and
//! structural ops to their owner segment, and the op replay that turns
//! a snapshot into the scene at t. split from `play.rs` to keep both
//! files within the repository's size budget.
//!
//! structural semantics (spec decision 1: the scene is a flat map
//! depth -> node): place and replace put a node into its depth slot,
//! overwriting any occupant; remove empties a slot. ops act only inside
//! the keyframe segment owning their t, so seek stays O(1) in frames:
//! evaluation replays the current segment's ops only, never the file.

use crate::ir::{Node, Prop, Timeline, Track, Value};
use engine::animation::Interpolate;

/// v0 default for a prop never given a value: scalars read 0,
/// colors transparent black (mirrors `crate::lower`).
pub(crate) fn default_value(prop: Prop) -> Value {
    if prop.is_color() {
        Value::Color([0.0; 4])
    } else {
        Value::Scalar(0.0)
    }
}

/// Lerp through plev's [`Interpolate`]; `k` is already eased.
/// `Timeline::validate` guarantees matching kinds on any constructed
/// player, so the mixed arm is unreachable there. Shared with
/// `crate::discover_fit`, whose segment fitting must predict exactly
/// what playback will interpolate.
pub(crate) fn lerp_value(from: Value, to: Value, k: f32) -> Value {
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
pub(crate) struct Cursor {
    pub(crate) valid: bool,
    pub(crate) seg: usize,
    pub(crate) t0: f32,
    pub(crate) t1: f32,
    pub(crate) from: Value,
}

pub(crate) const DEAD: Cursor = Cursor {
    valid: false,
    seg: 0,
    t0: 0.0,
    t1: 0.0,
    from: Value::Scalar(0.0),
};

/// Walk the chain once and cache the segment containing `t`. The from
/// value accumulates exactly like the wire semantics: snapshot value,
/// then each segment's target. Only runs on a window miss.
pub(crate) fn derive_cursor(track: &Track, base: Value, t: f32) -> Cursor {
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

/// Eased value of `track` at `t` under cursor `cur`.
pub(crate) fn sample(track: &Track, cur: &Cursor, t: f32) -> Value {
    if cur.seg == track.segments.len() {
        cur.from
    } else {
        let seg = &track.segments[cur.seg];
        let k = seg.easing.sample((t - cur.t0) / seg.dur_s);
        lerp_value(cur.from, seg.target, k)
    }
}

/// One track bound to its owner keyframe. `node_at` is the node's
/// position in the owner snapshot; in segments with structural ops the
/// scene drifts from the snapshot, nodes are found by id instead, and
/// `node_at` is `usize::MAX` for tracks on placed or replaced nodes.
pub(crate) struct Binding {
    pub(crate) track: usize,
    pub(crate) node_at: usize,
}

/// Index into one of the timeline's structural op lists.
pub(crate) enum OpRef {
    Place(usize),
    Replace(usize),
    Remove(usize),
}

/// One structural op scheduled inside a keyframe segment, `t` absolute.
pub(crate) struct PlannedOp {
    pub(crate) t: f32,
    pub(crate) op: OpRef,
}

/// Everything one keyframe segment owns: its tracks and its structural
/// ops, the latter sorted by (t, place < replace < remove, depth), the
/// canonical application order.
#[derive(Default)]
pub(crate) struct KfPlan {
    pub(crate) bindings: Vec<Binding>,
    pub(crate) ops: Vec<PlannedOp>,
}

fn owner_keyframe(timeline: &Timeline, t: f32) -> usize {
    // validate() guarantees the opening keyframe at t=0.
    timeline
        .keyframes
        .iter()
        .rposition(|kf| kf.t <= t)
        .expect("validated timeline opens at t=0")
}

/// D-block semantics (spec decision 2/3): tracks and ops act only while
/// their owner keyframe governs the playhead; the next snapshot resets
/// everything. Tracks whose node never exists in their window (absent
/// from the owner snapshot and never placed or replaced within it) have
/// nothing to modify and are dropped here, once.
pub(crate) fn plan(timeline: &Timeline) -> Vec<KfPlan> {
    let mut plans: Vec<KfPlan> = (0..timeline.keyframes.len())
        .map(|_| KfPlan::default())
        .collect();
    for (i, p) in timeline.places.iter().enumerate() {
        plans[owner_keyframe(timeline, p.t)].ops.push(PlannedOp {
            t: p.t,
            op: OpRef::Place(i),
        });
    }
    for (i, r) in timeline.replaces.iter().enumerate() {
        plans[owner_keyframe(timeline, r.t)].ops.push(PlannedOp {
            t: r.t,
            op: OpRef::Replace(i),
        });
    }
    for (i, r) in timeline.removes.iter().enumerate() {
        plans[owner_keyframe(timeline, r.t)].ops.push(PlannedOp {
            t: r.t,
            op: OpRef::Remove(i),
        });
    }
    for plan in &mut plans {
        // t >= 0 always, so the bit pattern orders like the float.
        plan.ops.sort_by_key(|po| {
            let (rank, depth) = match po.op {
                OpRef::Place(i) => (0u8, timeline.places[i].node.depth),
                OpRef::Replace(i) => (1, timeline.replaces[i].depth),
                OpRef::Remove(i) => (2, timeline.removes[i].depth),
            };
            (po.t.to_bits(), rank, depth)
        });
    }
    for (track, t) in timeline.tracks.iter().enumerate() {
        let owner = owner_keyframe(timeline, t.start_t);
        let snapshot = &timeline.keyframes[owner].snapshot;
        match snapshot.iter().position(|n| n.id == t.node_id) {
            Some(node_at) => plans[owner].bindings.push(Binding { track, node_at }),
            None => {
                let introduced = plans[owner].ops.iter().any(|po| match po.op {
                    OpRef::Place(i) => timeline.places[i].node.id == t.node_id,
                    OpRef::Replace(i) => timeline.replaces[i].node.id == t.node_id,
                    OpRef::Remove(_) => false,
                });
                if introduced {
                    plans[owner].bindings.push(Binding {
                        track,
                        node_at: usize::MAX,
                    });
                }
            }
        }
    }
    plans
}

/// Replay the segment's structural ops up to and including `t` onto the
/// snapshot clone. `ops` is sorted by t, so the scan stops at the first
/// op still in the future.
pub(crate) fn apply_ops(scene: &mut Vec<Node>, timeline: &Timeline, ops: &[PlannedOp], t: f32) {
    for po in ops {
        if po.t > t {
            break;
        }
        match po.op {
            OpRef::Place(i) => set_slot(scene, &timeline.places[i].node),
            OpRef::Replace(i) => set_slot(scene, &timeline.replaces[i].node),
            OpRef::Remove(i) => {
                let depth = timeline.removes[i].depth;
                scene.retain(|n| n.depth != depth);
            }
        }
    }
}

/// Put `node` into its depth slot, overwriting any occupant.
fn set_slot(scene: &mut Vec<Node>, node: &Node) {
    match scene.iter().position(|n| n.depth == node.depth) {
        Some(at) => scene[at] = node.clone(),
        None => scene.push(node.clone()),
    }
}
