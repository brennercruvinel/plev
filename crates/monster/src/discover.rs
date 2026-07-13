//! encoder mode B (docs/adr/monster-format-v0.md "encoder modes"): delta
//! discovery from a sampled frame sequence. the author declares
//! nothing; the encoder diffs consecutive snapshots, the h264 essence.
//! slot transitions become place | replace | remove ops (the scene is
//! a flat map depth -> node, spec decision 1), per-prop sample runs of
//! a continuing node become linear segment chains (greedy collinear
//! merge: linear motion is one segment, not N), and snapshot keyframes
//! are inserted at discontinuities (a one-sample jump past the
//! configured tolerance) and on the random access cadence. curve
//! fitting (easing recovery) is v1; mode B emits linear segments only.
//!
//! every sample is quantized to the wire grid up front, so "changed"
//! is exact grid inequality, the output round-trips
//! `crate::write::encode` / `crate::read::decode` structurally, and
//! playback reproduces each input sample to within half a quantization
//! step (bit-exactly for grid-aligned input). sample bookkeeping and
//! segment fitting live in `crate::discover_fit`.

use crate::container::prop_wire_id;
use crate::discover_fit::{Life, emit_tracks, jumped, quantize_value, value_of};
use crate::ir::{
    Depth, IrError, Keyframe, Node, NodeId, PlaceNode, Prop, Props, RemoveNode, ReplaceNode,
    Timeline, Value,
};

/// Tuning of the discovery pass; `Default` suits ui-scale motion
/// sampled at common frame rates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DiscoverConfig {
    /// Random access cadence: a snapshot keyframe at the first sample
    /// at or past this many seconds since the last keyframe (seek is
    /// O(1) from the governing snapshot, spec decision 2).
    pub keyframe_every_s: f32,
    /// Scalar discontinuity tolerance, in the prop's own unit (px for
    /// coordinates and widths, degrees for angles): a larger one-sample
    /// step becomes a keyframe, never a segment.
    pub scalar_jump: f32,
    /// Color discontinuity tolerance per channel in [0, 1].
    pub color_jump: f32,
}

impl Default for DiscoverConfig {
    fn default() -> Self {
        Self {
            keyframe_every_s: 2.0,
            scalar_jump: 8.0,
            color_jump: 0.5,
        }
    }
}

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum DiscoverError {
    #[error("a frame sequence needs at least one frame")]
    Empty,
    #[error("frame {at} has a non-finite time")]
    NonFiniteTime { at: usize },
    #[error("the sequence must open at t=0 for O(1) random access, got t={t}")]
    FirstFrameNotZero { t: f32 },
    #[error("frame times must be strictly increasing; offender at t={t}")]
    FrameOutOfOrder { t: f32 },
    #[error("duplicate depth {depth} in the frame at t={t}; the scene is a flat map depth -> node")]
    DuplicateDepth { t: f32, depth: Depth },
    #[error("duplicate node id {id} in the frame at t={t}; discovery matches nodes by id")]
    DuplicateId { t: f32, id: NodeId },
    #[error("discovered timeline failed validation: {0}")]
    Ir(#[from] IrError),
}

/// Discover a delta timeline from full-scene snapshots sampled at
/// strictly increasing times starting at 0. The result passes
/// [`Timeline::validate`] and feeds [`crate::write::encode`] directly.
pub fn discover(
    frames: &[(f32, Vec<Node>)],
    cfg: &DiscoverConfig,
) -> Result<Timeline, DiscoverError> {
    let frames = normalize(frames)?;
    let marks = keyframe_indices(&frames, cfg);
    let last_t = frames.last().expect("normalize rejects empty input").0;
    let mut timeline = Timeline {
        duration_s: last_t,
        fps_hint: fps_hint(frames.len(), last_t),
        ..Timeline::default()
    };
    for (w, &k) in marks.iter().enumerate() {
        timeline.keyframes.push(Keyframe {
            t: frames[k].0,
            snapshot: frames[k].1.clone(),
        });
        walk_window(&frames, k, marks.get(w + 1).copied(), cfg, &mut timeline);
    }
    timeline.validate()?;
    Ok(timeline)
}

/// Quantize every prop to its wire grid, sort props by wire id and
/// nodes by depth (the codec's canonical orders, so the discovered
/// timeline round-trips structurally), and check the sequence
/// contract: finite strictly increasing times from 0, unique depths
/// and ids per frame.
fn normalize(frames: &[(f32, Vec<Node>)]) -> Result<Vec<(f32, Vec<Node>)>, DiscoverError> {
    if frames.is_empty() {
        return Err(DiscoverError::Empty);
    }
    let mut out = Vec::with_capacity(frames.len());
    let mut prev_t = f32::NEG_INFINITY;
    for (i, (t, nodes)) in frames.iter().enumerate() {
        if !t.is_finite() {
            return Err(DiscoverError::NonFiniteTime { at: i });
        }
        if i == 0 && *t != 0.0 {
            return Err(DiscoverError::FirstFrameNotZero { t: *t });
        }
        if *t <= prev_t {
            return Err(DiscoverError::FrameOutOfOrder { t: *t });
        }
        prev_t = *t;
        let mut scene: Vec<Node> = nodes.iter().map(quantize_node).collect();
        scene.sort_by_key(|n| n.depth);
        if let Some(pair) = scene.windows(2).find(|p| p[0].depth == p[1].depth) {
            return Err(DiscoverError::DuplicateDepth {
                t: *t,
                depth: pair[0].depth,
            });
        }
        for (j, node) in scene.iter().enumerate() {
            if scene[..j].iter().any(|m| m.id == node.id) {
                return Err(DiscoverError::DuplicateId { t: *t, id: node.id });
            }
        }
        out.push((*t, scene));
    }
    Ok(out)
}

fn quantize_node(node: &Node) -> Node {
    let mut entries: Vec<(Prop, Value)> = node
        .props
        .iter()
        .map(|(p, v)| (*p, quantize_value(*p, *v)))
        .collect();
    entries.sort_by_key(|(p, _)| prop_wire_id(*p));
    let props = entries
        .into_iter()
        .fold(Props::new(), |ps, (p, v)| ps.with(p, v));
    Node {
        id: node.id,
        depth: node.depth,
        kind: node.kind,
        props,
    }
}

/// Frame indices that become snapshot keyframes: the opening frame,
/// every frame a matched node reaches by jumping past the tolerance
/// (the snapshot carries the jump, segments never do), and the random
/// access cadence.
fn keyframe_indices(frames: &[(f32, Vec<Node>)], cfg: &DiscoverConfig) -> Vec<usize> {
    let mut marks = vec![0];
    let mut last_t = frames[0].0;
    for i in 1..frames.len() {
        if frames[i].0 - last_t >= cfg.keyframe_every_s
            || frame_jumped(&frames[i - 1].1, &frames[i].1, cfg)
        {
            marks.push(i);
            last_t = frames[i].0;
        }
    }
    marks
}

/// Did any prop of a node continuing in its slot move past the
/// discontinuity tolerance in this one step?
fn frame_jumped(prev: &[Node], cur: &[Node], cfg: &DiscoverConfig) -> bool {
    cur.iter().any(|n| {
        prev.iter()
            .find(|p| p.depth == n.depth && p.id == n.id && p.kind == n.kind)
            .is_some_and(|p| {
                n.kind.animatable_props().iter().any(|&prop| {
                    jumped(
                        prop,
                        value_of(p, prop),
                        value_of(n, prop),
                        cfg.scalar_jump,
                        cfg.color_jump,
                    )
                })
            })
    })
}

/// Mean sample rate of the input, rounded; the format carries it as a
/// hint only.
fn fps_hint(samples: usize, duration_s: f32) -> u16 {
    if samples < 2 || duration_s <= 0.0 {
        return 60;
    }
    ((samples - 1) as f32 / duration_s)
        .round()
        .clamp(1.0, 65535.0) as u16
}

/// Diff consecutive frames inside one keyframe window. Slot
/// transitions become structural ops: an occupied slot changing hands
/// is a replace, an emptied slot a remove, a filled one a place (an id
/// moving depth is a remove plus a place). Continuing nodes accumulate
/// per-prop samples as lives. At a closing keyframe no ops are emitted
/// (the snapshot resets the scene) and continuous props land on it, so
/// motion stays smooth across cadence snapshots.
fn walk_window(
    frames: &[(f32, Vec<Node>)],
    k: usize,
    next_kf: Option<usize>,
    cfg: &DiscoverConfig,
    timeline: &mut Timeline,
) {
    let end = next_kf.unwrap_or(frames.len() - 1);
    let kf_t = frames[k].0;
    let mut open: Vec<Life> = frames[k].1.iter().map(|n| Life::open(n, k, kf_t)).collect();
    let mut done: Vec<Life> = Vec::new();
    for (i, (t, scene)) in frames.iter().enumerate().take(end + 1).skip(k + 1) {
        let closing = next_kf == Some(i);
        let mut next_open = Vec::with_capacity(scene.len());
        for node in scene {
            match open.iter().position(|l| l.node.depth == node.depth) {
                Some(at) if open[at].node.id == node.id && open[at].node.kind == node.kind => {
                    let mut life = open.swap_remove(at);
                    if closing {
                        life.land(node, *t, cfg.scalar_jump, cfg.color_jump);
                    } else {
                        life.push(node, *t);
                    }
                    next_open.push(life);
                }
                Some(at) => {
                    done.push(open.swap_remove(at));
                    if !closing {
                        timeline.replaces.push(ReplaceNode {
                            t: *t,
                            depth: node.depth,
                            node: node.clone(),
                        });
                        next_open.push(Life::open(node, i, *t));
                    }
                }
                None if !closing => {
                    timeline.places.push(PlaceNode {
                        t: *t,
                        node: node.clone(),
                    });
                    next_open.push(Life::open(node, i, *t));
                }
                None => {}
            }
        }
        // whatever stayed behind lost its slot: ops in depth order, the
        // canonical order the decoder hands back.
        open.sort_by_key(|l| l.node.depth);
        for life in open.drain(..) {
            if !closing {
                timeline.removes.push(RemoveNode {
                    t: *t,
                    depth: life.node.depth,
                });
            }
            done.push(life);
        }
        open = next_open;
    }
    done.extend(open);
    // chronological per id, so the pin bookkeeping sees lives in order.
    done.sort_by_key(|l| (l.start, l.node.id));
    let mut tracks = Vec::new();
    let mut prior = Vec::new();
    for life in &done {
        let pin_dur = frames
            .get(life.start + 1)
            .map_or(1e-4, |(t, _)| t - life.start_t);
        emit_tracks(life, &mut prior, pin_dur, &mut tracks);
    }
    // canonical track order: the decoder returns (start offset, node
    // id, prop wire id); matching it keeps round-trip equality. t >= 0
    // always, so the bit pattern orders like the float.
    tracks.sort_by_key(|t| (t.start_t.to_bits(), t.node_id, prop_wire_id(t.prop)));
    timeline.tracks.extend(tracks);
}
