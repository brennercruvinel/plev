//! What the Inspect view's pointer is on, or what is chosen: a control
//! the page bound to an id, or a scene node the frame drew.
//!
//! Under a point there can be both. The control is the answer when the
//! node on top is its whole ground (a button's fill); the node when it is
//! smaller than the control it sits in (a button's label), when nothing
//! is bound there (an icon), and when it is drawn over a control it is no
//! part of (a scrim): a node sits in a control only when what shows of
//! the node is inside the control's rect. A larger node drawn before the
//! control was bound is under it (the card a row lies on), and the
//! control is the answer.
//!
//! The frame is built again every time, so a node chosen is kept by what
//! it is (`SceneNode::mark`) with the control it sat in, and found again
//! in each frame: the ring follows it where it moves, and a frame that no
//! longer draws it says so, never another node in its place.

use accord::{SemId, Snapshot};

use crate::drawing::compositor::{Compositor, LayerId};
use crate::widgets::prelude::Rect;

/// A node at least this much of the control it sits in is that control's
/// own ground: the control is what the pointer is on.
const GROUND_SHARE: f32 = 0.9;
/// A node sits in a control when at least this much of what shows of it
/// is inside the control's rect.
const INSIDE_SHARE: f32 = 0.5;

/// What the pointer is on, or what is chosen.
#[derive(Clone, Debug, PartialEq)]
pub(super) enum Target {
    /// A control the page bound to an id.
    Control(SemId),
    /// A scene node, by what it was when chosen: its kind, its mark
    /// (`SceneNode::mark`), its place in its layer then, and the control
    /// it sat in. The frame is built again every time: the node is found
    /// again by its mark, nearest to that place.
    Node {
        layer: LayerId,
        index: usize,
        kind: &'static str,
        mark: u64,
        owner: Option<SemId>,
    },
}

fn rect([x, y, w, h]: [f32; 4]) -> Rect {
    Rect::new(x, y, w, h)
}

/// What two rects share.
fn overlap(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    let (x0, y0) = (a[0].max(b[0]), a[1].max(b[1]));
    let x1 = (a[0] + a[2]).min(b[0] + b[2]);
    let y1 = (a[1] + a[3]).min(b[1] + b[3]);
    [x0, y0, x1 - x0, y1 - y0]
}

impl Target {
    /// What is at `(x, y)` of the frame `c`, the layer `skip` left out.
    pub fn at(c: &Compositor, x: f32, y: f32, skip: Option<LayerId>) -> Option<Target> {
        let control = c.hit(x, y).and_then(|id| {
            let binding = c.binding(id)?;
            Some((id, binding.visible_rect()?, binding))
        });
        let node = c
            .node_at(x, y, skip)
            .and_then(|at| Some((at, c.node(at.layer, at.index)?)));
        let area = |r: [f32; 4]| r[2].max(0.0) * r[3].max(0.0);
        let Some((at, node)) = node else {
            return control.map(|(id, ..)| Target::Control(id.clone()));
        };
        let mut owner = None;
        if let Some((id, bound, binding)) = control {
            // the node sits in the control: what shows of it is inside
            let inside = area(overlap(at.rect, bound)) >= area(at.rect) * INSIDE_SHARE;
            // or it was drawn before the control, and is under it (the
            // card a row lies on): the control is what the pointer is on
            let under = !inside && at.layer == binding.layer && at.index < binding.nodes;
            if under || (inside && area(at.rect) >= area(bound) * GROUND_SHARE) {
                return Some(Target::Control(id.clone()));
            }
            owner = inside.then(|| id.clone());
        }
        Some(Target::Node {
            layer: at.layer,
            index: at.index,
            kind: node.kind(),
            mark: node.mark(),
            owner,
        })
    }

    /// Where a node chosen is in the frame `c`: its place in its layer
    /// now, `None` once the frame no longer draws it (or for a control).
    pub fn place(&self, c: &Compositor) -> Option<(LayerId, usize)> {
        match self {
            Target::Control(_) => None,
            Target::Node {
                layer, index, mark, ..
            } => Some((*layer, c.node_marked(*layer, *index, *mark)?)),
        }
    }

    /// What shows of it in the frame `c`: a control's rect, a node's
    /// bounds inside its clip. `None` once the frame no longer has it,
    /// or shows none of it.
    pub fn rect(&self, c: &Compositor) -> Option<Rect> {
        match self {
            Target::Control(id) => c.binding(id)?.visible_rect().map(rect),
            Target::Node { .. } => {
                let (layer, at) = self.place(c)?;
                c.node_rect(layer, at).map(rect)
            }
        }
    }

    /// The control it is, or the one it sat in when chosen.
    pub fn owner(&self) -> Option<&SemId> {
        match self {
            Target::Control(id) => Some(id),
            Target::Node { owner, .. } => owner.as_ref(),
        }
    }

    /// What it is, in a word: a control's role, a node's kind.
    pub fn word(&self, snapshot: Option<&Snapshot>) -> String {
        match self {
            Target::Control(id) => snapshot
                .and_then(|s| s.get(id))
                .map_or_else(|| "Control".to_owned(), |n| format!("{:?}", n.role)),
            Target::Node { kind, .. } => (*kind).to_owned(),
        }
    }
}
