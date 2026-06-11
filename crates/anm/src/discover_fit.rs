//! sample bookkeeping and segment fitting for `crate::discover` (mode
//! B): one [`Life`] per stay of a node in its depth slot, per-prop
//! sample series, greedy collinear merging into linear segments, and
//! the quantization-grid helpers diffing works on. split from
//! discover.rs to keep both files within the repository's size budget.

use crate::easing::Easing;
use crate::ir::{Node, NodeId, Prop, Segment, Track, Value};
use crate::play_eval::{default_value, lerp_value};
use crate::quant;

/// Quantize `value` to the wire grid of `prop` and back to f32: the
/// exact value an encode/decode round trip would carry (the container
/// `put_value` choices: angles u16 fixed, other scalars twips, colors
/// rgba8). Discovery normalizes every sample through this first, so
/// "changed" is exact grid inequality and the discovered timeline
/// round-trips the codec without drift.
pub fn quantize_value(prop: Prop, value: Value) -> Value {
    match (prop, value) {
        (Prop::AngleDeg, Value::Scalar(deg)) => {
            Value::Scalar(quant::u16_to_angle_deg(quant::angle_deg_to_u16(deg)))
        }
        (_, Value::Scalar(px)) => Value::Scalar(quant::twips_to_px(quant::px_to_twips(px))),
        (_, Value::Color(rgba)) => Value::Color(quant::bytes_to_rgba(quant::rgba_to_bytes(rgba))),
    }
}

/// Largest per-component distance between two values of one prop:
/// scalars differ by |a - b|, colors by the worst channel. Mixed kinds
/// never share a prop past validation; infinite distance keeps such a
/// pair a discontinuity, never a merge. Shared with `crate::optimize`,
/// whose tolerances measure the same playback error.
pub(crate) fn dist(a: Value, b: Value) -> f32 {
    match (a, b) {
        (Value::Scalar(a), Value::Scalar(b)) => (a - b).abs(),
        (Value::Color(a), Value::Color(b)) => a
            .iter()
            .zip(b.iter())
            .map(|(x, y)| (x - y).abs())
            .fold(0.0, f32::max),
        _ => f32::INFINITY,
    }
}

/// One quantization step of the prop's wire grid (the container
/// `put_value` choices): the natural unit for fitting and optimizer
/// tolerances, so one number means the same wire-visible error on
/// every prop.
pub(crate) fn grid_step(prop: Prop) -> f32 {
    if prop.is_color() {
        1.0 / 255.0
    } else if prop == Prop::AngleDeg {
        360.0 / 65536.0
    } else {
        1.0 / quant::TWIPS_PER_PX as f32
    }
}

/// Half a quantization step of the prop's wire grid: merging below it
/// (strictly) keeps every reproduced sample rounding back to its own
/// grid point, so grid-aligned input plays back bit-exactly.
fn fit_eps(prop: Prop) -> f32 {
    0.5 * grid_step(prop)
}

/// Did `prop` move further than the discontinuity tolerance in one
/// sample step?
pub(crate) fn jumped(prop: Prop, a: Value, b: Value, scalar_tol: f32, color_tol: f32) -> bool {
    let tol = if prop.is_color() { color_tol } else { scalar_tol };
    dist(a, b) > tol
}

/// `prop` of `node`, with the v0 default for an absent entry, so a
/// prop the author never set diffs as the constant it lowers to.
pub(crate) fn value_of(node: &Node, prop: Prop) -> Value {
    node.props.get(prop).unwrap_or_else(|| default_value(prop))
}

/// One stay of a node in its depth slot: from the window snapshot or a
/// place/replace op until the slot changes hands or the window closes.
/// Chains are anchored at the life start, so a later life of the same
/// id always overrides the held tail of an earlier life's chains (in
/// op segments the player applies tracks in timeline order).
pub(crate) struct Life {
    pub(crate) node: Node,
    /// Frame index where the life begins.
    pub(crate) start: usize,
    pub(crate) start_t: f32,
    /// Per animatable prop: (sample time, quantized value), the value
    /// at the life start first.
    series: Vec<(Prop, Vec<(f32, Value)>)>,
}

impl Life {
    pub(crate) fn open(node: &Node, start: usize, t: f32) -> Self {
        let series = node
            .kind
            .animatable_props()
            .iter()
            .map(|&p| (p, vec![(t, value_of(node, p))]))
            .collect();
        Self {
            node: node.clone(),
            start,
            start_t: t,
            series,
        }
    }

    /// Append the sample of every prop at `t`; the slot still holds
    /// this node.
    pub(crate) fn push(&mut self, node: &Node, t: f32) {
        for (prop, samples) in &mut self.series {
            samples.push((t, value_of(node, *prop)));
        }
    }

    /// Land continuous props on the closing keyframe at `t`, keeping
    /// motion smooth across a cadence snapshot; a prop past the jump
    /// tolerance is left to the snapshot reset (the jump is the point).
    pub(crate) fn land(&mut self, node: &Node, t: f32, scalar_tol: f32, color_tol: f32) {
        for (prop, samples) in &mut self.series {
            let v = value_of(node, *prop);
            let last = samples.last().expect("a series opens with one sample").1;
            if !jumped(*prop, last, v, scalar_tol, color_tol) {
                samples.push((t, v));
            }
        }
    }
}

/// Emit the tracks of one life: a chain per prop that changed, plus a
/// pin per prop a dead earlier life of the same id would otherwise
/// bleed onto (a past-end chain holds its final target forever, and in
/// op segments it re-applies to whichever node carries the id).
/// `prior` maps (id, prop) to that held value across the window;
/// `pin_dur` is any positive length, the chain holds after it.
pub(crate) fn emit_tracks(
    life: &Life,
    prior: &mut Vec<((NodeId, Prop), Value)>,
    pin_dur: f32,
    out: &mut Vec<Track>,
) {
    for (prop, samples) in &life.series {
        let first = samples[0].1;
        let segments = if samples.iter().any(|(_, v)| *v != first) {
            fit_segments(samples, fit_eps(*prop))
        } else if held(prior, life.node.id, *prop).is_some_and(|h| h != first) {
            vec![Segment {
                target: first,
                easing: Easing::Linear,
                dur_s: pin_dur,
            }]
        } else {
            continue;
        };
        let target = segments.last().expect("a chain has a segment").target;
        set_held(prior, life.node.id, *prop, target);
        out.push(Track {
            node_id: life.node.id,
            prop: *prop,
            start_t: life.start_t,
            segments,
        });
    }
}

fn held(prior: &[((NodeId, Prop), Value)], id: NodeId, prop: Prop) -> Option<Value> {
    prior
        .iter()
        .find(|((i, p), _)| *i == id && *p == prop)
        .map(|(_, v)| *v)
}

fn set_held(prior: &mut Vec<((NodeId, Prop), Value)>, id: NodeId, prop: Prop, v: Value) {
    match prior.iter_mut().find(|((i, p), _)| *i == id && *p == prop) {
        Some(entry) => entry.1 = v,
        None => prior.push(((id, prop), v)),
    }
}

/// Greedy merge of a sample run into the fewest linear segments that
/// keep every interior sample strictly within half a quantization step
/// of the line: linear motion becomes one segment regardless of the
/// sample rate (the h264 essence), anything else splits at the first
/// sample the line cannot reproduce. The constant tail is dropped: a
/// chain holds its last target. The caller guarantees at least one
/// sample differs from the first.
fn fit_segments(samples: &[(f32, Value)], eps: f32) -> Vec<Segment> {
    let mut last = samples.len() - 1;
    while samples[last].1 == samples[last - 1].1 {
        last -= 1;
    }
    let mut segments = Vec::new();
    let mut i0 = 0;
    while i0 < last {
        let mut j = i0 + 1;
        while j < last && fits(samples, i0, j + 1, eps) {
            j += 1;
        }
        segments.push(Segment {
            target: samples[j].1,
            easing: Easing::Linear,
            dur_s: samples[j].0 - samples[i0].0,
        });
        i0 = j;
    }
    segments
}

/// Is every interior sample strictly within `eps` of the line from
/// sample `i0` to sample `j`?
fn fits(samples: &[(f32, Value)], i0: usize, j: usize, eps: f32) -> bool {
    let (t0, v0) = samples[i0];
    let (t1, v1) = samples[j];
    samples[i0 + 1..j]
        .iter()
        .all(|(t, v)| dist(*v, lerp_value(v0, v1, (t - t0) / (t1 - t0))) < eps)
}
