//! A live graph's frame. The base layer holds what the layout placed: group
//! boxes (outer first) with their labels above, edges (width and alpha each,
//! a head where asked), node circles and labels. The overlay layer holds
//! only the hover and the selection, so a still graph under a moving pointer
//! changes one layer's hash and the compositor redraws that layer alone.
//! The hit grid is rebuilt from the same primitives, in the same order.

use web_time::Instant;

use super::LiveGraph;
use super::hit::Hit;
use crate::drawing::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use crate::drawing::path::PathBuilder;
use crate::drawing::text::{TextMeasurer, TextStyle};
use crate::styling::theme::Theme;
use crate::widgets::graph::layout::compound::LABEL;
use crate::widgets::graph::look::{LabelAt, Tone};
use crate::widgets::widget::Rect;

/// The overlay's place above the base layer.
const OVERLAY_Z: i32 = 1;

fn with_alpha(c: [f32; 4], a: f32) -> [f32; 4] {
    [c[0], c[1], c[2], c[3] * a]
}

/// A label centered on `at` (screen px), measured with the style it is drawn with.
fn centered_text(
    c: &mut Compositor,
    layer: LayerId,
    text: &str,
    size: f32,
    at: (f32, f32),
    color: [f32; 4],
) {
    if text.is_empty() || size < 1.0 {
        return;
    }
    let style = TextStyle::new(size);
    let (w, h) = TextMeasurer::measure_styled(text, &style, None);
    let key = TextNodeKey::from_style(text, &style, None);
    c.push_to_layer(
        layer,
        SceneNode::Text {
            key,
            x: at.0 - w / 2.0,
            y: at.1 - h / 2.0,
            color,
        },
    );
}

/// An arrow head with its tip at `tip`, pointing away from `from` (screen px).
fn head(from: (f32, f32), tip: (f32, f32), size: f32) -> PathBuilder {
    let (dx, dy) = (tip.0 - from.0, tip.1 - from.1);
    let len = dx.hypot(dy).max(1e-6);
    let (ux, uy) = (dx / len, dy / len);
    let base = (tip.0 - ux * size, tip.1 - uy * size);
    let (nx, ny) = (-uy * size * 0.45, ux * size * 0.45);
    PathBuilder::new()
        .move_to(tip.0, tip.1)
        .line_to(base.0 + nx, base.1 + ny)
        .line_to(base.0 - nx, base.1 - ny)
        .close()
}

impl LiveGraph {
    /// An edge's ends in world px: center to center, or border to border under a head.
    fn edge_ends(&self, k: usize) -> ((f32, f32), (f32, f32)) {
        let e = &self.graph.edges[k];
        let (a, b) = (self.pos[e.from], self.pos[e.to]);
        if !self.look.edges[k].arrow {
            return (a, b);
        }
        let (dx, dy) = (b.0 - a.0, b.1 - a.1);
        let len = dx.hypot(dy).max(1e-6);
        let (ra, rb) = (self.look.nodes[e.from].radius, self.look.nodes[e.to].radius);
        if len <= ra + rb {
            return (a, b);
        }
        (
            (a.0 + dx / len * ra, a.1 + dy / len * ra),
            (b.0 - dx / len * rb, b.1 - dy / len * rb),
        )
    }

    fn overlay_layer(&mut self, c: &mut Compositor) -> LayerId {
        match self.overlay {
            Some(id) if c.layer(id).is_some() => id,
            _ => {
                let id = c.create_layer(OVERLAY_Z);
                self.overlay = Some(id);
                id
            }
        }
    }

    pub fn render(&mut self, c: &mut Compositor, bounds: Rect, theme: &Theme) {
        let t0 = Instant::now();
        self.fit(bounds, theme.spacing.xl);
        let rebuild = self.grid_dirty;
        if rebuild {
            self.grid.clear();
        }
        let s = self.transform.scale;
        let base = LayerId::DEFAULT;
        let clip = SceneNode::PushClip {
            x: bounds.x,
            y: bounds.y,
            w: bounds.w,
            h: bounds.h,
        };
        c.push_to_layer(base, clip.clone());
        for k in 0..self.group_order.len() {
            let g = self.group_order[k];
            let Some([x, y, w, h]) = self.boxes[g] else {
                continue;
            };
            let (sx, sy) = self.to_screen((x, y));
            c.push_to_layer(
                base,
                SceneNode::RoundedRect {
                    x: sx,
                    y: sy,
                    w: w * s,
                    h: h * s,
                    color: with_alpha(Tone::Panel.color(theme), 0.5),
                    corner_radius: 0.0,
                    border_width: 1.0,
                    border_color: Tone::BorderActive.color(theme),
                },
            );
            let at = self.to_screen((x + w / 2.0, y - LABEL / 2.0));
            let label = self.graph.groups[g].label.clone();
            centered_text(
                c,
                base,
                &label,
                self.look.group_font * s,
                at,
                Tone::TextMid.color(theme),
            );
            if rebuild {
                self.grid.push_box([x, y, w, h], Hit::Group(g));
            }
        }
        for k in 0..self.graph.edges.len() {
            let (a, b) = self.edge_ends(k);
            let e = self.look.edges[k];
            let color = with_alpha(e.tone.color(theme), e.alpha);
            let (sa, sb) = (self.to_screen(a), self.to_screen(b));
            let line = PathBuilder::new()
                .move_to(sa.0, sa.1)
                .line_to(sb.0, sb.1)
                .end_open();
            c.push_to_layer(
                base,
                SceneNode::Path {
                    data: line.stroke(color, (e.width * s).max(0.5)),
                },
            );
            if e.arrow {
                c.push_to_layer(
                    base,
                    SceneNode::Path {
                        data: head(sa, sb, 8.0 * s.sqrt()).fill(color),
                    },
                );
            }
            if rebuild {
                self.grid.push_segment(a, b, e.width.max(1.0), Hit::Edge(k));
            }
        }
        for (i, n) in self.look.nodes.iter().enumerate() {
            let (sx, sy) = self.to_screen(self.pos[i]);
            let r = n.radius * s;
            c.push_to_layer(
                base,
                SceneNode::Path {
                    data: PathBuilder::circle(sx, sy, r).fill(n.fill.color(theme)),
                },
            );
            let ring = PathBuilder::circle(sx, sy, r).stroke(n.stroke.color(theme), 1.0);
            c.push_to_layer(base, SceneNode::Path { data: ring });
            if rebuild {
                self.grid.push_circle(self.pos[i], n.radius, Hit::Node(i));
            }
        }
        for i in 0..self.graph.nodes.len() {
            let n = &self.look.nodes[i];
            let (x, y) = self.pos[i];
            let at = match n.label_at {
                LabelAt::Center => (x, y),
                LabelAt::Above(gap) => (x, y - gap),
            };
            let (label, at) = (self.graph.nodes[i].label.clone(), self.to_screen(at));
            centered_text(
                c,
                base,
                &label,
                self.look.node_font * s,
                at,
                Tone::Text.color(theme),
            );
        }
        c.push_to_layer(base, SceneNode::PopClip);
        let overlay = self.overlay_layer(c);
        c.push_to_layer(overlay, clip);
        let marks = [(self.hovered, 1.5_f32), (self.selected, 2.5_f32)];
        for (mark, width) in marks.into_iter().filter_map(|(m, w)| m.map(|m| (m, w))) {
            self.draw_mark(c, overlay, mark, width, theme);
        }
        c.push_to_layer(overlay, SceneNode::PopClip);
        if rebuild {
            self.grid.build();
            self.grid_dirty = false;
        }
        self.record_build(t0.elapsed().as_secs_f32() * 1000.0);
    }

    /// The accent over one owner: a ring around a node (and its edges when
    /// selected), an edge drawn again in the accent, a group's border.
    fn draw_mark(&self, c: &mut Compositor, layer: LayerId, mark: Hit, width: f32, theme: &Theme) {
        let accent = Tone::Accent.color(theme);
        let s = self.transform.scale;
        let edge = |c: &mut Compositor, k: usize| {
            let (a, b) = self.edge_ends(k);
            let (sa, sb) = (self.to_screen(a), self.to_screen(b));
            let w = (self.look.edges[k].width * s).max(0.5) + width;
            let line = PathBuilder::new()
                .move_to(sa.0, sa.1)
                .line_to(sb.0, sb.1)
                .end_open();
            c.push_to_layer(
                layer,
                SceneNode::Path {
                    data: line.stroke(accent, w),
                },
            );
        };
        match mark {
            Hit::Node(i) => {
                if Some(mark) == self.selected {
                    let incident = self
                        .graph
                        .edges
                        .iter()
                        .enumerate()
                        .filter(|(_, e)| e.from == i || e.to == i);
                    for (k, _) in incident {
                        edge(c, k);
                    }
                }
                let (sx, sy) = self.to_screen(self.pos[i]);
                let r = self.look.nodes[i].radius * s + width;
                c.push_to_layer(
                    layer,
                    SceneNode::Path {
                        data: PathBuilder::circle(sx, sy, r).stroke(accent, width),
                    },
                );
            }
            Hit::Edge(k) => edge(c, k),
            Hit::Group(g) => {
                let Some([x, y, w, h]) = self.boxes[g] else {
                    return;
                };
                let (sx, sy) = self.to_screen((x, y));
                let frame = PathBuilder::new()
                    .move_to(sx, sy)
                    .line_to(sx + w * s, sy)
                    .line_to(sx + w * s, sy + h * s)
                    .line_to(sx, sy + h * s)
                    .close();
                c.push_to_layer(
                    layer,
                    SceneNode::Path {
                        data: frame.stroke(accent, width),
                    },
                );
            }
        }
    }
}
