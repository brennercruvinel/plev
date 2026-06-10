use crate::compositor::{Compositor, SceneNode, TextNodeKey};
use crate::text::{TextMeasurer, TextStyle};
use crate::theme::Theme;

use super::{EventResult, Rect, WidgetEvent, with_alpha};

const FONT: f32 = 13.0;
const PAD_X: f32 = 14.0;
const UNDERLINE: f32 = 2.0;

/// Horizontal tab strip. Tab widths come from real text measurement.
#[derive(Clone, Debug)]
pub struct Tabs {
    pub labels: Vec<String>,
    pub active: usize,
    hovered: Option<usize>,
}

impl Tabs {
    pub fn new(labels: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            labels: labels.into_iter().map(Into::into).collect(),
            active: 0,
            hovered: None,
        }
    }

    pub fn hovered(&self) -> Option<usize> {
        self.hovered
    }

    /// Hit rects for each tab within `bounds`.
    pub fn item_rects(&self, bounds: Rect) -> Vec<Rect> {
        let mut rects = Vec::with_capacity(self.labels.len());
        let mut x = bounds.x;
        for label in &self.labels {
            let style = TextStyle::new(FONT).with_weight(500);
            let (text_w, _) = TextMeasurer::measure_styled(label, &style, None);
            let w = (text_w + PAD_X * 2.0).ceil();
            rects.push(Rect::new(x, bounds.y, w, bounds.h));
            x += w;
        }
        rects
    }

    fn tab_at(&self, x: f32, y: f32, bounds: Rect) -> Option<usize> {
        self.item_rects(bounds)
            .iter()
            .position(|r| r.contains(x, y))
    }

    pub fn handle_event(&mut self, event: &WidgetEvent, bounds: Rect) -> EventResult {
        match *event {
            WidgetEvent::MouseMove { x, y } => {
                let hit = self.tab_at(x, y, bounds);
                if hit != self.hovered {
                    self.hovered = hit;
                    EventResult::changed()
                } else {
                    EventResult::IGNORED
                }
            }
            WidgetEvent::MouseDown { x, y } => {
                if let Some(i) = self.tab_at(x, y, bounds) {
                    if i != self.active {
                        self.active = i;
                        EventResult::clicked()
                    } else {
                        EventResult {
                            handled: true,
                            ..EventResult::IGNORED
                        }
                    }
                } else {
                    EventResult::IGNORED
                }
            }
            _ => EventResult::IGNORED,
        }
    }

    pub fn render(&self, compositor: &mut Compositor, bounds: Rect, theme: &Theme) {
        // Baseline under the whole strip.
        compositor.push(SceneNode::Rect {
            x: bounds.x,
            y: bounds.y + bounds.h - 1.0,
            w: bounds.w,
            h: 1.0,
            color: with_alpha(theme.colors.divider, 1.0),
        });

        let line_height = FONT * 1.3;
        for (i, (label, rect)) in self.labels.iter().zip(self.item_rects(bounds)).enumerate() {
            let is_active = i == self.active;
            let is_hovered = self.hovered == Some(i);

            if is_hovered && !is_active {
                compositor.push(SceneNode::RoundedRect {
                    x: rect.x + 2.0,
                    y: rect.y + 4.0,
                    w: rect.w - 4.0,
                    h: rect.h - 8.0 - UNDERLINE,
                    color: with_alpha(theme.colors.bg_hover, 1.0),
                    corner_radius: theme.radius.md,
                    border_width: 0.0,
                    border_color: [0.0; 4],
                });
            }

            if is_active {
                compositor.push(SceneNode::Rect {
                    x: rect.x,
                    y: rect.y + rect.h - UNDERLINE,
                    w: rect.w,
                    h: UNDERLINE,
                    color: with_alpha(theme.colors.accent, 1.0),
                });
            }

            let color = if is_active {
                theme.colors.text
            } else if is_hovered {
                theme.colors.text_mid
            } else {
                theme.colors.text_dim
            };
            compositor.push(SceneNode::Text {
                key: TextNodeKey::new(label, FONT, line_height, None).with_weight(if is_active {
                    600
                } else {
                    500
                }),
                x: rect.x + PAD_X,
                y: rect.y + (rect.h - UNDERLINE - line_height) / 2.0,
                color: with_alpha(color, 1.0),
            });
        }
    }
}
