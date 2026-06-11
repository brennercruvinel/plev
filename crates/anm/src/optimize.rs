//! optimizer passes over the IR timeline: they run between authoring
//! or discovery and `crate::write::encode`, never on the wire, so the
//! frozen container layout and the golden fixtures are untouched.
//! three passes apply to any validated [`Timeline`]:
//!
//! 1. static collapse: a track whose chain never strays from its base
//!    value past the tolerance is redundant; the governing snapshot
//!    (or placing op) already carries the value, so the track goes
//! 2. keyframe reduction: Ramer-Douglas-Peucker over each track's
//!    value x time polyline drops interior targets a straight line
//!    reproduces within the tolerance; endpoints and extremes survive
//!    by construction
//! 3. collinear fusion: a consecutive segment pair whose shared
//!    landing point sits on the line between its outer endpoints
//!    merges into one segment
//!
//! tolerances are in quantization steps of each prop's wire grid
//! (spec decision 5: a twentieth of a px for coordinates, 1/255 per
//! color channel, 360/65536 of a degree for angles), so one number
//! means the same wire-visible error on every prop. the defaults are
//! half a step, the error quantization already commits, so default
//! optimization is lossless on the wire. only `Easing::Linear`
//! segments merge: an eased curve does not survive re-parameterization
//! over a longer duration, so non-linear segments are fixed run
//! boundaries for passes 2 and 3. the segment passes iterate to a
//! joint fixpoint and the collapse decisions are order-independent,
//! so optimizing twice equals optimizing once.

use crate::discover_fit::{dist, grid_step, value_of};
use crate::easing::Easing;
use crate::ir::{IrError, NodeId, Prop, Segment, Timeline, Track, Value};
use crate::play_eval::{default_value, lerp_value};

/// Tuning of the optimizer; tolerances are in quantization steps of
/// the prop's wire grid. `Default` is conservative: half a step, so
/// every change stays under what quantization itself rounds away.
/// A non-finite or negative tolerance disables its comparisons, which
/// degrades the pass to the identity instead of corrupting the chain.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OptimizeCfg {
    /// Pass 1 bound: largest deviation of a chain value from the
    /// track's base before the track stops being "static".
    pub static_tol: f32,
    /// Pass 2 and 3 bound: largest deviation between a dropped target
    /// and the straight line that replaces it.
    pub rdp_tol: f32,
    pub collapse_static: bool,
    pub reduce_rdp: bool,
    pub fuse_collinear: bool,
}

impl Default for OptimizeCfg {
    fn default() -> Self {
        Self {
            static_tol: 0.5,
            rdp_tol: 0.5,
            collapse_static: true,
            reduce_rdp: true,
            fuse_collinear: true,
        }
    }
}

/// Run the enabled passes over a validated timeline. The result plays
/// back within the configured tolerances of the input, encodes through
/// [`crate::write::encode`] unchanged in meaning, and is a fixpoint:
/// optimizing it again with the same config returns it unchanged.
pub fn optimize(timeline: &Timeline, cfg: &OptimizeCfg) -> Result<Timeline, IrError> {
    timeline.validate()?;
    let mut out = timeline.clone();
    // bases depend on snapshots and structural ops only, both
    // untouched by every pass, so one resolution serves all of them.
    let bases: Vec<Option<Value>> = out.tracks.iter().map(|t| stable_base(&out, t)).collect();
    if cfg.reduce_rdp || cfg.fuse_collinear {
        // joint fixpoint: a changing sweep removes at least one
        // segment, so this terminates; at the fixpoint a second
        // optimize() sweeps once, changes nothing, and idempotence of
        // the whole pipeline follows.
        loop {
            let mut changed = false;
            for (track, base) in out.tracks.iter_mut().zip(&bases) {
                if cfg.reduce_rdp {
                    changed |= rdp_track(track, *base, cfg.rdp_tol);
                }
                if cfg.fuse_collinear {
                    changed |= fuse_track(track, *base, cfg.rdp_tol);
                }
            }
            if !changed {
                break;
            }
        }
    }
    if cfg.collapse_static {
        collapse_static(&mut out, &bases, cfg.static_tol);
    }
    debug_assert_eq!(out.validate(), Ok(()));
    Ok(out)
}

/// Last keyframe at or before `t`; validate() guarantees one at 0.
fn owner(tl: &Timeline, t: f32) -> usize {
    tl.keyframes
        .iter()
        .rposition(|kf| kf.t <= t)
        .expect("validated timeline opens at t=0")
}

/// The value a track's first segment interpolates from, when it is
/// stable for the whole keyframe window: the node's prop in the owner
/// snapshot, overridden by the latest place or replace of the same id
/// at or before `start_t` (at one instant a replace lands after a
/// place, mirroring the player's application order). `None` when a
/// later op in the window re-introduces the id: the player would
/// re-derive the base mid-chain, so passes that lean on the base must
/// leave the first segment alone and skip the collapse.
fn stable_base(tl: &Timeline, track: &Track) -> Option<Value> {
    let own = owner(tl, track.start_t);
    let kf = &tl.keyframes[own];
    let end = tl.keyframes.get(own + 1).map_or(f32::INFINITY, |k| k.t);
    let mut base = kf
        .snapshot
        .iter()
        .find(|n| n.id == track.node_id)
        .map(|n| (kf.t, n));
    let intros = tl
        .places
        .iter()
        .map(|p| (p.t, &p.node))
        .chain(tl.replaces.iter().map(|r| (r.t, &r.node)));
    for (t, node) in intros {
        if node.id != track.node_id || t < kf.t || t >= end {
            continue;
        }
        if t > track.start_t {
            return None;
        }
        if base.is_none_or(|(held, _)| t >= held) {
            base = Some((t, node));
        }
    }
    Some(base.map_or_else(
        || default_value(track.prop),
        |(_, n)| value_of(n, track.prop),
    ))
}

/// Chain polyline of a track: the base at `start_t` when stable, then
/// every segment's landing (time, target).
fn points(track: &Track, base: Option<Value>) -> Vec<(f32, Value)> {
    let mut pts = Vec::with_capacity(track.segments.len() + 1);
    if let Some(b) = base {
        pts.push((track.start_t, b));
    }
    let mut t = track.start_t;
    for seg in &track.segments {
        t += seg.dur_s;
        pts.push((t, seg.target));
    }
    pts
}

/// Pass 2 on one track: RDP over every maximal run of consecutive
/// linear segments (without a stable base, segment 0 has no known
/// start point and never joins a run). Returns whether segments were
/// removed. Kept targets and durations are carried over verbatim, so
/// reduction never drifts the landing times or the final value.
fn rdp_track(track: &mut Track, base: Option<Value>, tol: f32) -> bool {
    let n = track.segments.len();
    if n < 2 {
        return false;
    }
    let off = usize::from(base.is_some());
    let pts = points(track, base);
    let mut keep = vec![true; pts.len()];
    let tol_abs = tol * grid_step(track.prop);
    let mut s = 1 - off;
    while s < n {
        if track.segments[s].easing != Easing::Linear {
            s += 1;
            continue;
        }
        let mut e = s;
        while e + 1 < n && track.segments[e + 1].easing == Easing::Linear {
            e += 1;
        }
        if e > s {
            // segment i runs from point i+off-1 to point i+off.
            rdp(&pts, &mut keep, s + off - 1, e + off, tol_abs);
        }
        s = e + 1;
    }
    if keep.iter().all(|k| *k) {
        return false;
    }
    track.segments = rebuild(&track.segments, &keep, off);
    true
}

/// Ramer-Douglas-Peucker over `pts[lo..=hi]`: clear the keep flag of
/// every interior point strictly within `tol_abs` of the chord, split
/// at the farthest otherwise. Deviation is vertical, at the point's
/// own time: exactly the error playback would show there. First-max
/// splitting keeps the pass idempotent: re-run on the kept points, the
/// same splits are picked again.
fn rdp(pts: &[(f32, Value)], keep: &mut [bool], lo: usize, hi: usize, tol_abs: f32) {
    let mut spans = vec![(lo, hi)];
    while let Some((lo, hi)) = spans.pop() {
        if hi - lo < 2 {
            continue;
        }
        let (t0, v0) = pts[lo];
        let (t1, v1) = pts[hi];
        let (mut far, mut far_d) = (lo, f32::NEG_INFINITY);
        for (i, (t, v)) in pts.iter().enumerate().take(hi).skip(lo + 1) {
            let d = dist(*v, lerp_value(v0, v1, (t - t0) / (t1 - t0)));
            if d > far_d {
                (far, far_d) = (i, d);
            }
        }
        if far_d < tol_abs {
            for k in &mut keep[lo + 1..hi] {
                *k = false;
            }
        } else {
            spans.push((lo, far));
            spans.push((far, hi));
        }
    }
}

/// Rebuild a chain from its keep flags: a segment whose start point
/// was dropped extends the previous one (only interiors of linear runs
/// drop, so the merged segment stays linear). With no base point
/// (`off == 0`) segment 0 has no flag and is always kept.
fn rebuild(segments: &[Segment], keep: &[bool], off: usize) -> Vec<Segment> {
    let mut out: Vec<Segment> = Vec::with_capacity(segments.len());
    for (i, seg) in segments.iter().enumerate() {
        if i + off == 0 || keep[i + off - 1] {
            out.push(seg.clone());
        } else {
            let last = out.last_mut().expect("rdp keeps run endpoints");
            last.dur_s += seg.dur_s;
            last.target = seg.target;
        }
    }
    out
}

/// Pass 3 on one track: greedy pairwise fusion of consecutive linear
/// segments whose shared landing point sits strictly within the
/// tolerance of the pair's chord. Only linear easing admits exact
/// fusion (an eased curve re-parameterized over the summed duration is
/// a different curve), so a same-easing pair of anything else is left
/// alone. Returns whether segments were removed.
fn fuse_track(track: &mut Track, base: Option<Value>, tol: f32) -> bool {
    let tol_abs = tol * grid_step(track.prop);
    let mut changed = false;
    let mut i = usize::from(base.is_none());
    while i + 1 < track.segments.len() {
        let (a, b) = (&track.segments[i], &track.segments[i + 1]);
        let from = if i == 0 {
            base.expect("the loop starts past segment 0 without a base")
        } else {
            track.segments[i - 1].target
        };
        let collinear = a.easing == Easing::Linear && b.easing == Easing::Linear && {
            let mid = lerp_value(from, b.target, a.dur_s / (a.dur_s + b.dur_s));
            dist(a.target, mid) < tol_abs
        };
        if collinear {
            let b = track.segments.remove(i + 1);
            let a = &mut track.segments[i];
            a.dur_s += b.dur_s;
            a.target = b.target;
            changed = true;
        } else {
            i += 1;
        }
    }
    changed
}

/// Pass 1: drop every track whose chain values all stay strictly
/// within the tolerance of its base; the player then holds the base,
/// within tolerance of everything the chain produced. The bound is on
/// the chain's anchor values: an overshooting easing (back, elastic,
/// custom bezier) can exceed it mid-flight by its overshoot ratio over
/// a sub-tolerance span, which the conservative default keeps under a
/// grid step. Skipped: tracks without a stable base, and (node, prop)
/// twins sharing one keyframe window; discover pins a re-placed id
/// against the held tail of a dead chain with exactly such a twin, and
/// dropping one of the pair changes which chain wins.
fn collapse_static(tl: &mut Timeline, bases: &[Option<Value>], tol: f32) {
    let keys: Vec<(NodeId, Prop, usize)> = tl
        .tracks
        .iter()
        .map(|t| (t.node_id, t.prop, owner(tl, t.start_t)))
        .collect();
    let redundant: Vec<bool> = tl
        .tracks
        .iter()
        .enumerate()
        .map(|(i, track)| {
            let Some(base) = bases[i] else { return false };
            if keys
                .iter()
                .enumerate()
                .any(|(j, k)| j != i && *k == keys[i])
            {
                return false;
            }
            let tol_abs = tol * grid_step(track.prop);
            track
                .segments
                .iter()
                .all(|s| dist(s.target, base) < tol_abs)
        })
        .collect();
    let mut flags = redundant.iter();
    tl.tracks
        .retain(|_| !flags.next().expect("one flag per track"));
}
