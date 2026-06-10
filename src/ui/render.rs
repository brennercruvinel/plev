use crate::compositor::{Compositor, SceneNode, TextNodeKey};
use crate::layout::LayoutItem;

use super::builder::Ui;
use super::node::{UiHitRect, Visual};

impl Ui {
    pub fn render(&mut self, compositor: &mut Compositor, vw: f32, vh: f32) {
        self.hit_rects.clear();

        // Build LayoutItems from nodes
        let items: Vec<LayoutItem> = self
            .nodes
            .iter()
            .map(|n| LayoutItem {
                style: n.layout.clone(),
                children: n.children.clone(),
                text: None,
            })
            .collect();

        let bounds = self.engine.compute(&items, vw, vh);

        // Emit SceneNodes
        for (i, node) in self.nodes.iter().enumerate() {
            let b = &bounds[i];
            if b.width < 0.5 || b.height < 0.5 {
                continue;
            }

            match &node.visual {
                Visual::None => {}
                Visual::Box {
                    bg,
                    border_color,
                    border_width,
                    corner_radius,
                } => {
                    if *corner_radius > 0.5 || *border_width > 0.0 {
                        compositor.push(SceneNode::RoundedRect {
                            x: b.x,
                            y: b.y,
                            w: b.width,
                            h: b.height,
                            color: *bg,
                            corner_radius: *corner_radius,
                            border_width: *border_width,
                            border_color: *border_color,
                        });
                    } else if bg[3] > 0.001 {
                        compositor.push(SceneNode::Rect {
                            x: b.x,
                            y: b.y,
                            w: b.width,
                            h: b.height,
                            color: *bg,
                        });
                    }
                }
                Visual::Text {
                    content,
                    size,
                    line_height,
                    weight,
                    color,
                    family,
                } => {
                    let mut key = TextNodeKey::new(content, *size, *line_height, Some(b.width))
                        .with_weight(*weight);
                    if let Some(fam) = family {
                        key = key.with_family(fam);
                    }
                    compositor.push(SceneNode::Text {
                        key,
                        x: b.x,
                        y: b.y,
                        color: *color,
                    });
                }
            }

            // Collect hit rects
            if let Some(id) = node.click_id {
                self.hit_rects.push(UiHitRect {
                    id,
                    x: b.x,
                    y: b.y,
                    w: b.width,
                    h: b.height,
                });
            }
        }
    }

    /// Query: was this click_id hit at (cx, cy)?
    pub fn hit_test(&self, cx: f32, cy: f32) -> Option<u64> {
        self.hit_rects.iter().rev().find_map(|hr| {
            if cx >= hr.x && cx <= hr.x + hr.w && cy >= hr.y && cy <= hr.y + hr.h {
                Some(hr.id)
            } else {
                None
            }
        })
    }

    /// Reset for next frame.
    pub fn reset(&mut self) {
        self.nodes.truncate(1);
        self.nodes[0].children.clear();
        self.stack.clear();
        self.stack.push(0);
        self.hit_rects.clear();
    }
}
