//! structural validation of [`Timeline`]: the IR contract every codec
//! entry point (encoder, decoder, player) enforces before trusting a
//! timeline. split from `crate::ir` to keep the model file within the
//! repository's size budget; the impl stays on [`Timeline`].

use crate::ir::{Depth, IrError, Node, NodeId, Prop, Timeline, Value};

fn check_value(prop: Prop, value: Value) -> Result<(), IrError> {
    let ok = match value {
        Value::Scalar(_) => !prop.is_color(),
        Value::Color(_) => prop.is_color(),
    };
    if ok {
        Ok(())
    } else {
        let expected = if prop.is_color() { "color" } else { "scalar" };
        Err(IrError::ValueKindMismatch { prop, expected })
    }
}

/// Every prop a node carries must be in its kind's animatable surface
/// and of the right value kind; shared by snapshots and placed nodes.
fn check_node(node: &Node) -> Result<(), IrError> {
    for (prop, value) in node.props.iter() {
        if !node.kind.allows(*prop) {
            return Err(IrError::PropNotAnimatable {
                kind: node.kind.name(),
                prop: *prop,
            });
        }
        check_value(*prop, *value)?;
    }
    Ok(())
}

impl Timeline {
    fn check_keyframes(&self) -> Result<(), IrError> {
        match self.keyframes.first() {
            Some(first) if first.t == 0.0 => {}
            _ => return Err(IrError::MissingOpeningKeyframe),
        }
        let mut prev_t = f32::NEG_INFINITY;
        for kf in &self.keyframes {
            if kf.t <= prev_t || kf.t > self.duration_s {
                return Err(IrError::KeyframeOutOfOrder { t: kf.t });
            }
            prev_t = kf.t;
            let mut seen: Vec<(Depth, NodeId)> = Vec::with_capacity(kf.snapshot.len());
            for node in &kf.snapshot {
                if seen.iter().any(|(d, _)| *d == node.depth) {
                    return Err(IrError::DuplicateDepth {
                        t: kf.t,
                        depth: node.depth,
                    });
                }
                if seen.iter().any(|(_, i)| *i == node.id) {
                    return Err(IrError::DuplicateNodeId {
                        t: kf.t,
                        id: node.id,
                    });
                }
                seen.push((node.depth, node.id));
                check_node(node)?;
            }
        }
        Ok(())
    }

    fn check_tracks(&self) -> Result<(), IrError> {
        for track in &self.tracks {
            // a track may animate a snapshot node or one introduced by
            // a place/replace op within some segment.
            let kind = self
                .keyframes
                .iter()
                .flat_map(|kf| kf.snapshot.iter())
                .chain(self.places.iter().map(|p| &p.node))
                .chain(self.replaces.iter().map(|r| &r.node))
                .find(|n| n.id == track.node_id)
                .map(|n| n.kind)
                .ok_or(IrError::UnknownNode {
                    node_id: track.node_id,
                })?;
            if !kind.allows(track.prop) {
                return Err(IrError::PropNotAnimatable {
                    kind: kind.name(),
                    prop: track.prop,
                });
            }
            for seg in &track.segments {
                if seg.dur_s <= 0.0 {
                    return Err(IrError::NonPositiveDuration {
                        node_id: track.node_id,
                    });
                }
                check_value(track.prop, seg.target)?;
            }
            // half a frame of f32 slack at the default 60fps hint
            let end_t = track.end_t();
            if track.start_t < 0.0 || end_t > self.duration_s + 1e-4 {
                return Err(IrError::TrackPastEnd {
                    node_id: track.node_id,
                    end_t,
                    duration_s: self.duration_s,
                });
            }
        }
        Ok(())
    }

    fn check_op_t(&self, t: f32) -> Result<(), IrError> {
        if (0.0..=self.duration_s).contains(&t) {
            Ok(())
        } else {
            Err(IrError::OpOutOfRange {
                t,
                duration_s: self.duration_s,
            })
        }
    }

    fn check_ops(&self) -> Result<(), IrError> {
        for place in &self.places {
            self.check_op_t(place.t)?;
            check_node(&place.node)?;
        }
        for replace in &self.replaces {
            self.check_op_t(replace.t)?;
            if replace.node.depth != replace.depth {
                return Err(IrError::ReplaceDepthMismatch {
                    t: replace.t,
                    depth: replace.depth,
                    node_depth: replace.node.depth,
                });
            }
            check_node(&replace.node)?;
        }
        for remove in &self.removes {
            self.check_op_t(remove.t)?;
        }
        Ok(())
    }

    /// Structural validation: ordered keyframes, flat-map snapshots,
    /// props within each kind's animatable surface, tracks anchored to
    /// known nodes and inside the duration, structural ops inside the
    /// duration with well-formed nodes.
    pub fn validate(&self) -> Result<(), IrError> {
        self.check_keyframes()?;
        self.check_tracks()?;
        self.check_ops()
    }
}
