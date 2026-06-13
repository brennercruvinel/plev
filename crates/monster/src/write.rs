//! monster v0 encoder, mode A of kdb/adr/monster-format-v0.md: lowering from an
//! authored track model. tracks already exist; the encoder packs
//! segments into modify ops and discovers per-field presence, so an
//! unchanged field costs zero bytes. byte layout in `crate::container`.
//!
//! determinism contract: the same timeline always encodes to the same
//! bytes. every traversal is canonically ordered (modify ops by start
//! offset then node id, props by wire id, structural ops after them by
//! start offset then place < replace < remove then depth, custom curves
//! first-seen in that same order), so no hash-map iteration ever
//! reaches the file. round-trip equality therefore holds for timelines
//! whose op lists are already in canonical order, the decoder's output
//! order.

use crate::container::{self, Asset, DeltaOp, Desc, SEC_DELTA, SEC_DESC, SEC_KEYFRAME, Section};
use crate::easing::EasingTable;
use crate::ir::{Depth, IrError, Keyframe, NodeId, Prop, Segment, Timeline};

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum WriteError {
    #[error(transparent)]
    Ir(#[from] IrError),
    #[error(
        "node {node_id} has two tracks on {prop:?} starting at the same time; \
         presence flags admit one chain per field per op"
    )]
    DuplicateTrack { node_id: NodeId, prop: Prop },
    #[error("description references keyframe {keyframe}, timeline has {keyframes}")]
    DescOutOfRange { keyframe: u16, keyframes: usize },
    #[error("keyframe {keyframe} has two descriptions; the track is one text per keyframe")]
    DuplicateDesc { keyframe: u16 },
    #[error("{what} count or length exceeds the u16 wire limit")]
    TooMany { what: &'static str },
}

/// One modify op under construction: tracks of the same node starting
/// at the same offset share presence flags.
struct OpDraft {
    at_s: f32,
    node_id: NodeId,
    props: Vec<(Prop, Vec<Segment>)>,
}

/// Encode an authored timeline plus its asset and description tables
/// into a v0 monster file. Sections come out as K0 D0 K1 D1 ... T, with
/// empty delta blocks omitted.
pub fn encode(
    timeline: &Timeline,
    assets: &[Asset],
    descs: &[Desc],
) -> Result<Vec<u8>, WriteError> {
    timeline.validate()?;
    check_limits(timeline, assets, descs)?;
    let blocks = group_tracks(timeline)?;
    let structural = group_structural(timeline);
    let easings = intern_curves(&blocks);
    if easings.len() > u16::MAX as usize {
        return Err(WriteError::TooMany {
            what: "custom easing curve",
        });
    }

    let mut sections = Vec::with_capacity(timeline.keyframes.len() * 2 + 1);
    for ((kf, ops), extra) in timeline.keyframes.iter().zip(&blocks).zip(&structural) {
        sections.push(Section {
            tag: SEC_KEYFRAME,
            payload: keyframe_payload(kf),
        });
        if ops.len() + extra.len() > u16::MAX as usize {
            return Err(WriteError::TooMany { what: "delta op" });
        }
        if !ops.is_empty() || !extra.is_empty() {
            sections.push(Section {
                tag: SEC_DELTA,
                payload: delta_payload(ops, extra, &easings),
            });
        }
    }
    if !descs.is_empty() {
        sections.push(Section {
            tag: SEC_DESC,
            payload: desc_payload(descs)?,
        });
    }
    Ok(container::assemble(
        timeline.duration_s,
        timeline.fps_hint,
        assets,
        &easings,
        &sections,
    ))
}

fn check_limits(timeline: &Timeline, assets: &[Asset], descs: &[Desc]) -> Result<(), WriteError> {
    let cap = u16::MAX as usize;
    if assets.len() > cap {
        return Err(WriteError::TooMany { what: "asset" });
    }
    if assets.iter().any(|a| a.data.len() > cap) {
        return Err(WriteError::TooMany {
            what: "asset data byte",
        });
    }
    if timeline.keyframes.iter().any(|kf| kf.snapshot.len() > cap) {
        return Err(WriteError::TooMany {
            what: "keyframe node",
        });
    }
    if timeline.tracks.iter().any(|t| t.segments.len() > cap) {
        return Err(WriteError::TooMany {
            what: "track segment",
        });
    }
    if descs.len() > cap {
        return Err(WriteError::TooMany {
            what: "description",
        });
    }
    let keyframes = timeline.keyframes.len();
    if let Some(d) = descs.iter().find(|d| usize::from(d.keyframe) >= keyframes) {
        return Err(WriteError::DescOutOfRange {
            keyframe: d.keyframe,
            keyframes,
        });
    }
    Ok(())
}

/// Assign every track to the last keyframe at or before its start and
/// merge tracks of one node sharing a start offset into one modify op.
/// Output per keyframe, ops sorted by (start offset, node id), props
/// sorted by wire id: the canonical serialization order.
fn group_tracks(timeline: &Timeline) -> Result<Vec<Vec<OpDraft>>, WriteError> {
    let mut blocks: Vec<Vec<OpDraft>> = (0..timeline.keyframes.len()).map(|_| Vec::new()).collect();
    for track in &timeline.tracks {
        let owner = owner_keyframe(timeline, track.start_t);
        let at_s = track.start_t - timeline.keyframes[owner].t;
        let ops = &mut blocks[owner];
        let op = match ops
            .iter_mut()
            .find(|op| op.node_id == track.node_id && op.at_s.to_bits() == at_s.to_bits())
        {
            Some(op) => op,
            None => {
                ops.push(OpDraft {
                    at_s,
                    node_id: track.node_id,
                    props: Vec::new(),
                });
                ops.last_mut().expect("just pushed")
            }
        };
        if op.props.iter().any(|(p, _)| *p == track.prop) {
            return Err(WriteError::DuplicateTrack {
                node_id: track.node_id,
                prop: track.prop,
            });
        }
        op.props.push((track.prop, track.segments.clone()));
    }
    for ops in &mut blocks {
        if ops.len() > u16::MAX as usize {
            return Err(WriteError::TooMany { what: "delta op" });
        }
        // at_s >= 0 always, so the bit pattern orders like the float.
        ops.sort_by_key(|op| (op.at_s.to_bits(), op.node_id));
        for op in ops {
            op.props.sort_by_key(|(p, _)| container::prop_wire_id(*p));
        }
    }
    Ok(blocks)
}

/// Last keyframe at or before `t`. validate() guarantees an opening
/// keyframe at t=0 and t >= 0, so the owner always exists.
fn owner_keyframe(timeline: &Timeline, t: f32) -> usize {
    timeline
        .keyframes
        .iter()
        .rposition(|kf| kf.t <= t)
        .expect("validated timeline opens at t=0")
}

/// Assign every structural op to its owner keyframe and sort each block
/// canonically: start offset, then place < replace < remove, then
/// depth. This is also the player's application order at one instant.
fn group_structural(timeline: &Timeline) -> Vec<Vec<DeltaOp>> {
    let mut blocks: Vec<Vec<(u8, Depth, DeltaOp)>> =
        (0..timeline.keyframes.len()).map(|_| Vec::new()).collect();
    let mut push = |t: f32, rank: u8, depth: Depth, build: &dyn Fn(f32) -> DeltaOp| {
        let owner = owner_keyframe(timeline, t);
        let at_s = t - timeline.keyframes[owner].t;
        blocks[owner].push((rank, depth, build(at_s)));
    };
    for p in &timeline.places {
        push(p.t, 0, p.node.depth, &|at_s| DeltaOp::Place {
            at_s,
            node: p.node.clone(),
        });
    }
    for r in &timeline.replaces {
        push(r.t, 1, r.depth, &|at_s| DeltaOp::Replace {
            at_s,
            node: r.node.clone(),
        });
    }
    for r in &timeline.removes {
        push(r.t, 2, r.depth, &|at_s| DeltaOp::Remove {
            at_s,
            depth: r.depth,
        });
    }
    blocks
        .into_iter()
        .map(|mut ops| {
            // at_s >= 0 always, so the bit pattern orders like the float.
            ops.sort_by_key(|(rank, depth, op)| (op_at_s(op).to_bits(), *rank, *depth));
            ops.into_iter().map(|(.., op)| op).collect()
        })
        .collect()
}

fn op_at_s(op: &DeltaOp) -> f32 {
    match op {
        DeltaOp::Place { at_s, .. }
        | DeltaOp::Modify { at_s, .. }
        | DeltaOp::Replace { at_s, .. }
        | DeltaOp::Remove { at_s, .. } => *at_s,
    }
}

/// Walk the blocks in serialization order and intern every custom
/// curve, so table indices are stable and deduped (spec decision 4).
fn intern_curves(blocks: &[Vec<OpDraft>]) -> EasingTable {
    let mut table = EasingTable::default();
    for ops in blocks {
        for op in ops {
            for (_, segments) in &op.props {
                for seg in segments {
                    table.intern(seg.easing);
                }
            }
        }
    }
    table
}

fn keyframe_payload(kf: &Keyframe) -> Vec<u8> {
    let mut buf = Vec::new();
    container::put_f32(&mut buf, kf.t);
    container::put_u16(&mut buf, kf.snapshot.len() as u16);
    for node in &kf.snapshot {
        container::put_node(&mut buf, node);
    }
    buf
}

/// Modify ops first (their canonical order is the pre-structural wire,
/// keeping older files byte-identical), then the structural ops.
fn delta_payload(ops: &[OpDraft], structural: &[DeltaOp], easings: &EasingTable) -> Vec<u8> {
    let mut buf = Vec::new();
    container::put_u16(&mut buf, (ops.len() + structural.len()) as u16);
    for op in ops {
        let wire = DeltaOp::Modify {
            at_s: op.at_s,
            node_id: op.node_id,
            props: op.props.clone(),
        };
        container::put_op(&mut buf, &wire, easings);
    }
    for op in structural {
        container::put_op(&mut buf, op, easings);
    }
    buf
}

fn desc_payload(descs: &[Desc]) -> Result<Vec<u8>, WriteError> {
    let mut sorted: Vec<&Desc> = descs.iter().collect();
    sorted.sort_by_key(|d| d.keyframe);
    if let Some(w) = sorted.windows(2).find(|w| w[0].keyframe == w[1].keyframe) {
        return Err(WriteError::DuplicateDesc {
            keyframe: w[0].keyframe,
        });
    }
    let mut buf = Vec::new();
    container::put_u16(&mut buf, sorted.len() as u16);
    for desc in sorted {
        if desc.text.len() > u16::MAX as usize {
            return Err(WriteError::TooMany {
                what: "description byte",
            });
        }
        container::put_u16(&mut buf, desc.keyframe);
        container::put_u16(&mut buf, desc.text.len() as u16);
        buf.extend_from_slice(desc.text.as_bytes());
    }
    Ok(buf)
}
