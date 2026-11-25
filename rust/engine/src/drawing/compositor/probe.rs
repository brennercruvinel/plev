//! What is drawn where: the scene node at a point of the frame.
//!
//! [`Compositor::hit`] answers with a bound id, which only a widget that
//! binds itself has. A tool that looks at a frame (the dev mode's picker)
//! also needs what no id names: an icon, a line of text, a card's ground.
//! [`Compositor::node_at`] answers with the topmost scene node whose
//! bounds hold the point, as the frame composites: the layer on top
//! first, in a layer the node drawn last, each inside the clip that was
//! open when it was pushed.
//!
//! A node's bounds are what it draws in ([`SceneNode::bounds`]): its rect,
//! the box of a path's vertices, a text's measure at its position. A
//! shadow takes no part: it is the dress of the rect that casts it, and a
//! point beside a card is not on the card.

use super::clip::{ClipRect, intersect_rects};
use super::{Compositor, LayerId, SceneNode};
use crate::drawing::text::{TextMeasurer, TextStyle};

/// A scene node found at a point.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NodeAt {
    pub layer: LayerId,
    /// The node's place in [`Layer::nodes`](super::Layer::nodes).
    pub index: usize,
    /// `(x, y, w, h)` in logical window pixels: its bounds inside the
    /// clip it was drawn under.
    pub rect: [f32; 4],
}

impl SceneNode {
    /// What the node is, in a word.
    pub fn kind(&self) -> &'static str {
        match self {
            SceneNode::Rect { .. } => "Rect",
            SceneNode::RoundedRect { .. } => "RoundedRect",
            SceneNode::GradientRect { .. } => "GradientRect",
            SceneNode::Text { .. } => "Text",
            SceneNode::Path { .. } => "Path",
            SceneNode::Image { .. } => "Image",
            SceneNode::PushClip { .. } => "PushClip",
            SceneNode::PopClip => "PopClip",
            SceneNode::BackdropBlur { .. } => "BackdropBlur",
            SceneNode::Shadow { .. } => "Shadow",
        }
    }

    /// `(x, y, w, h)` of what the node draws in, in logical window
    /// pixels: its rect, the box of a path's vertices, a text's measure
    /// at its position (a shadow: the rect that casts it). `None` for a
    /// clip, which draws nothing, and for a path without a vertex.
    pub fn bounds(&self) -> Option<[f32; 4]> {
        match self {
            SceneNode::Rect { x, y, w, h, .. }
            | SceneNode::RoundedRect { x, y, w, h, .. }
            | SceneNode::GradientRect { x, y, w, h, .. }
            | SceneNode::Image { x, y, w, h, .. }
            | SceneNode::BackdropBlur { x, y, w, h, .. }
            | SceneNode::Shadow { x, y, w, h, .. } => Some([*x, *y, *w, *h]),
            SceneNode::Text { key, x, y, .. } => {
                let style = TextStyle {
                    font_size: f32::from_bits(key.font_size_bits),
                    line_height: f32::from_bits(key.line_height_bits),
                    font_weight: key.font_weight,
                    letter_spacing: f32::from_bits(key.letter_spacing_bits),
                    font_family: key.font_family.clone(),
                    features: key.features,
                    overflow_wrap: key.overflow_wrap,
                };
                let max_width = key.max_width_bits.map(f32::from_bits);
                let (w, h) = TextMeasurer::measure_styled(&key.text, &style, max_width);
                Some([*x, *y, w, h])
            }
            SceneNode::Path { data } => {
                let mut points = data.vertices.iter().map(|v| v.position);
                let [x, y] = points.next()?;
                let (mut left, mut top, mut right, mut bottom) = (x, y, x, y);
                for [x, y] in points {
                    left = left.min(x);
                    top = top.min(y);
                    right = right.max(x);
                    bottom = bottom.max(y);
                }
                Some([left, top, right - left, bottom - top])
            }
            SceneNode::PushClip { .. } | SceneNode::PopClip => None,
        }
    }
}

fn holds(r: [f32; 4], x: f32, y: f32) -> bool {
    r[2] > 0.0 && r[3] > 0.0 && x >= r[0] && x <= r[0] + r[2] && y >= r[1] && y <= r[1] + r[3]
}

impl Compositor {
    /// The topmost scene node at `(x, y)` in logical window pixels: on a
    /// visible layer (`visible` and an opacity above zero), the layers by
    /// z-order then by creation as the frame composites them, in a layer
    /// the node pushed last, and only where the clip open at its push
    /// holds the point. `skip` leaves a layer out (a tool's own). A
    /// shadow and a clip are never the answer.
    pub fn node_at(&self, x: f32, y: f32, skip: Option<LayerId>) -> Option<NodeAt> {
        let mut layers: Vec<(usize, &super::Layer)> = self
            .layers
            .iter()
            .enumerate()
            .filter(|(_, l)| l.visible && l.opacity > 0.0 && Some(l.id) != skip)
            .collect();
        layers.sort_by_key(|(rank, l)| std::cmp::Reverse((l.z_order, *rank)));
        for (_, layer) in layers {
            let mut clips: Vec<ClipRect> = Vec::new();
            let mut found = None;
            for (index, node) in layer.nodes().iter().enumerate() {
                match node {
                    SceneNode::PushClip { x, y, w, h } => {
                        let clip = [*x, *y, *w, *h];
                        clips.push(
                            clips
                                .last()
                                .map_or(clip, |open| intersect_rects(*open, clip)),
                        );
                    }
                    SceneNode::PopClip => {
                        clips.pop();
                    }
                    SceneNode::Shadow { .. } => {}
                    node => {
                        let Some(bounds) = node.bounds() else {
                            continue;
                        };
                        let rect = clips
                            .last()
                            .map_or(bounds, |clip| intersect_rects(bounds, *clip));
                        if holds(rect, x, y) {
                            found = Some(NodeAt {
                                layer: layer.id,
                                index,
                                rect,
                            });
                        }
                    }
                }
            }
            if found.is_some() {
                return found;
            }
        }
        None
    }

    /// The node at `index` of `layer`, as [`Compositor::node_at`] names
    /// one; `None` once the frame no longer has it.
    pub fn node(&self, layer: LayerId, index: usize) -> Option<&SceneNode> {
        self.layer(layer)?.nodes().get(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::drawing::compositor::QuadVertex;
    use crate::drawing::compositor::TextNodeKey;
    use crate::drawing::path::TessellatedPath;

    fn rect(x: f32, y: f32, w: f32, h: f32) -> SceneNode {
        SceneNode::Rect {
            x,
            y,
            w,
            h,
            color: [1.0; 4],
        }
    }

    fn kind_at(c: &Compositor, x: f32, y: f32) -> Option<(&'static str, usize)> {
        let at = c.node_at(x, y, None)?;
        Some((c.node(at.layer, at.index)?.kind(), at.index))
    }

    #[test]
    fn the_node_drawn_last_is_the_one_on_top() {
        let mut c = Compositor::new();
        c.push(rect(0.0, 0.0, 200.0, 200.0));
        c.push(rect(50.0, 50.0, 20.0, 20.0));
        assert_eq!(kind_at(&c, 60.0, 60.0), Some(("Rect", 1)), "the small one");
        assert_eq!(kind_at(&c, 10.0, 10.0), Some(("Rect", 0)), "the ground");
        assert_eq!(kind_at(&c, 300.0, 300.0), None);
    }

    #[test]
    fn a_path_is_found_by_the_box_of_its_vertices_and_a_text_by_its_measure() {
        let mut c = Compositor::new();
        let vertex = |x, y| QuadVertex {
            position: [x, y],
            color: [1.0; 4],
        };
        c.push(SceneNode::Path {
            data: TessellatedPath {
                vertices: vec![vertex(10.0, 10.0), vertex(34.0, 12.0), vertex(20.0, 34.0)],
                indices: vec![0, 1, 2],
                hash: 0,
            },
        });
        let at = c.node_at(20.0, 20.0, None).expect("inside the icon's box");
        assert_eq!(at.rect, [10.0, 10.0, 24.0, 24.0]);
        assert_eq!(c.node_at(40.0, 40.0, None), None);

        c.push(SceneNode::Text {
            key: TextNodeKey::new("a line of text", 14.0, 20.0, None),
            x: 100.0,
            y: 100.0,
            color: [1.0; 4],
        });
        let text = c.node_at(104.0, 110.0, None).expect("on the line");
        assert_eq!((text.rect[0], text.rect[1]), (100.0, 100.0));
        assert!(text.rect[2] > 20.0 && text.rect[3] >= 14.0, "{text:?}");
    }

    #[test]
    fn a_node_is_found_only_inside_the_clip_it_was_drawn_under() {
        let mut c = Compositor::new();
        c.push(SceneNode::PushClip {
            x: 0.0,
            y: 0.0,
            w: 50.0,
            h: 50.0,
        });
        c.push(rect(0.0, 0.0, 200.0, 200.0));
        c.push(SceneNode::PopClip);
        let inside = c.node_at(10.0, 10.0, None).expect("inside the clip");
        assert_eq!(inside.rect, [0.0, 0.0, 50.0, 50.0], "what shows of it");
        assert_eq!(c.node_at(100.0, 100.0, None), None, "clipped away there");
    }

    #[test]
    fn the_layer_on_top_answers_first_and_a_shadow_never_does() {
        let mut c = Compositor::new();
        c.push(rect(0.0, 0.0, 200.0, 200.0));
        let over = c.create_layer(10);
        c.push_to_layer(over, rect(0.0, 0.0, 100.0, 100.0));
        c.push_to_layer(
            over,
            SceneNode::Shadow {
                x: 0.0,
                y: 0.0,
                w: 100.0,
                h: 100.0,
                corner_radius: 0.0,
                blur_radius: 8.0,
                offset: [0.0, 0.0],
                color: [0.0, 0.0, 0.0, 0.5],
                inset: true,
            },
        );
        let top = c.node_at(20.0, 20.0, None).expect("a node there");
        assert_eq!(
            (top.layer, top.index),
            (over, 0),
            "the rect, not its shadow"
        );
        // the tool's own layer left out: what is under it answers
        let under = c.node_at(20.0, 20.0, Some(over)).expect("the ground");
        assert_eq!(under.layer, LayerId::DEFAULT);
        c.set_layer_visible(over, false);
        assert_eq!(
            c.node_at(20.0, 20.0, None).map(|n| n.layer),
            Some(LayerId::DEFAULT)
        );
    }
}
