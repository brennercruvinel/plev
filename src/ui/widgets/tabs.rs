use crate::compositor::{Compositor, SceneNode, TextNodeKey};
use crate::text::TextMeasurer;
use crate::theme::{Theme, TypographyScale};

use super::{EventResult, Rect, WidgetEvent, glass_pill};

/// HOFF tabs: container radius 22 / pad 4 / rgba(40,40,40,.6); the active
/// segment is an 18-radius glass block with its own edge-light + shadow.
const PAD: f32 = 4.0;

/// Segmented tab strip. Tabs share the width equally (CSS `flex: 1`);
/// give the widget 44px of height for the canonical 36px segments.
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

    /// Hit rects for each tab within `bounds`: equal-width segments
    /// inside the 4px container padding.
    pub fn item_rects(&self, bounds: Rect) -> Vec<Rect> {
        let n = self.labels.len().max(1) as f32;
        let w = (bounds.w - PAD * 2.0) / n;
        let h = bounds.h - PAD * 2.0;
        (0..self.labels.len())
            .map(|i| Rect::new(bounds.x + PAD + i as f32 * w, bounds.y + PAD, w, h))
            .collect()
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
        let glass = &theme.glass;

        // Container: rgba(40,40,40,.6), pill radius (22 at 44px height).
        let container = {
            let b = glass.button.0;
            [b[0], b[1], b[2], 0.6]
        };
        compositor.push(SceneNode::RoundedRect {
            x: bounds.x,
            y: bounds.y,
            w: bounds.w,
            h: bounds.h,
            color: container,
            corner_radius: bounds.h / 2.0,
            border_width: 0.0,
            border_color: [0.0; 4],
        });

        // Labels: base-2sm (Tabs.module.sass), one style for measure+render.
        let style = TypographyScale::hoff().base_2sm();
        let rects = self.item_rects(bounds);

        // Active block: shadow + edge-light + rgba($n2,.05) fill.
        if let Some(rect) = rects.get(self.active) {
            let radius = rect.h / 2.0;
            compositor.push(SceneNode::Shadow {
                x: rect.x,
                y: rect.y,
                w: rect.w,
                h: rect.h,
                corner_radius: radius,
                // 0 8px 16px -4px rgba(18,18,18,.20).
                blur_radius: 16.0,
                offset: [0.0, 8.0],
                color: [18.0 / 255.0, 18.0 / 255.0, 18.0 / 255.0, 0.20],
                inset: false,
            });
            for node in glass_pill(*rect, radius, glass.edge.0, 1.5, glass.surface_hover.0) {
                compositor.push(node);
            }
        }
        for (i, (label, rect)) in self.labels.iter().zip(&rects).enumerate() {
            let is_active = i == self.active;
            let is_hovered = self.hovered == Some(i);

            // base-2sm $text-secondary -> $text-primary on hover/active.
            let color = if is_active || is_hovered {
                theme.colors.text
            } else {
                theme.colors.text_mid
            };
            let (text_w, _) = TextMeasurer::measure_styled(label, &style, None);
            compositor.push(SceneNode::Text {
                key: TextNodeKey::from_style(label, &style, None),
                x: rect.x + (rect.w - text_w) / 2.0,
                y: rect.y + TextMeasurer::vertical_center(&style, rect.h),
                color: color.0,
            });
        }
    }
}
