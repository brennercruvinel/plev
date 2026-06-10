//! Icons section: the full Lucide set in a hoverable grid.

use plev::compositor::{Compositor, SceneNode, TextNodeKey};
use plev::text::TextMeasurer;
use plev::theme::Theme;
use plev::ui::icons;
use plev::ui::widgets::{
    EventResult, Rect, WidgetEvent, path_rounded_rect, path_rounded_rect_stroke,
};

const CELL_W: f32 = 104.0;
const CELL_H: f32 = 72.0;
const GAP: f32 = 10.0;
const ICON: f32 = 22.0;

pub struct IconsSection {
    names: Vec<&'static str>,
    hovered: Option<usize>,
}

impl IconsSection {
    pub fn new() -> Self {
        Self {
            names: icons::icon_names(),
            hovered: None,
        }
    }

    fn cell_rects(&self, content: Rect) -> Vec<Rect> {
        let cols = ((content.w + GAP) / (CELL_W + GAP)).floor().max(1.0) as usize;
        self.names
            .iter()
            .enumerate()
            .map(|(i, _)| {
                let col = i % cols;
                let row = i / cols;
                Rect::new(
                    content.x + col as f32 * (CELL_W + GAP),
                    content.y + row as f32 * (CELL_H + GAP),
                    CELL_W,
                    CELL_H,
                )
            })
            .collect()
    }

    /// Natural height of the icon grid (page scrolling needs it).
    pub fn content_height(&self, content: Rect) -> f32 {
        self.cell_rects(content)
            .last()
            .map(|r| r.y + r.h - content.y + GAP)
            .unwrap_or(0.0)
    }

    pub fn handle_event(&mut self, event: &WidgetEvent, content: Rect) -> EventResult {
        if let WidgetEvent::MouseMove { x, y } = *event {
            let hit = self
                .cell_rects(content)
                .iter()
                .position(|r| r.contains(x, y));
            if hit != self.hovered {
                self.hovered = hit;
                return EventResult::changed();
            }
        }
        EventResult::IGNORED
    }

    pub fn render(&self, c: &mut Compositor, content: Rect, theme: &Theme) {
        for (i, (name, rect)) in self.names.iter().zip(self.cell_rects(content)).enumerate() {
            let hovered = self.hovered == Some(i);
            // Path-based cards: the icons on them are paths.
            c.push(path_rounded_rect(
                rect.x,
                rect.y,
                rect.w,
                rect.h,
                theme.radius.lg,
                theme.colors.surface.0,
            ));
            c.push(path_rounded_rect_stroke(
                rect.x,
                rect.y,
                rect.w,
                rect.h,
                theme.radius.lg,
                if hovered {
                    theme.colors.border_active.0
                } else {
                    theme.colors.divider.0
                },
                1.0,
            ));

            let fg = if hovered {
                theme.colors.text.0
            } else {
                theme.colors.text_mid.0
            };
            if let Some(node) = icons::icon_at(
                name,
                ICON,
                fg,
                rect.x + (rect.w - ICON) / 2.0,
                rect.y + 12.0,
            ) {
                c.push(node);
            }

            // Caption centered with real text measurement.
            let caption = 10.0;
            let (tw, _) = TextMeasurer::measure(name, caption, None);
            c.push(SceneNode::Text {
                key: TextNodeKey::new(name, caption, caption * 1.3, None),
                x: rect.x + ((rect.w - tw) / 2.0).max(4.0),
                y: rect.y + rect.h - 22.0,
                color: theme.colors.text_dim.0,
            });
        }
    }
}
