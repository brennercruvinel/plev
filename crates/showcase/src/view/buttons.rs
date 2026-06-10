//! Buttons section: every variant x size x intent x state.

use plev::compositor::Compositor;
use plev::theme::{Intent, Theme};
use plev::ui::widgets::{Button, ButtonSize, ButtonVariant, EventResult, Rect, WidgetEvent};

use super::group_label;

const GAP: f32 = 12.0;
const ROW_GAP: f32 = 30.0;
const LABEL_H: f32 = 24.0;

pub struct ButtonsSection {
    groups: Vec<(&'static str, Vec<Button>)>,
}

impl ButtonsSection {
    pub fn new() -> Self {
        let variants = vec![
            Button::new("Solid"),
            Button::new("Outline").variant(ButtonVariant::Outline),
            Button::new("Ghost").variant(ButtonVariant::Ghost),
            Button::new("Danger").variant(ButtonVariant::Danger),
        ];
        let sizes = vec![
            Button::new("Small").size(ButtonSize::Sm),
            Button::new("Medium").size(ButtonSize::Md),
            Button::new("Large").size(ButtonSize::Lg),
        ];
        let intents = vec![
            Button::new("Neutral"),
            Button::new("Constructive").intent(Intent::Constructive),
            Button::new("Destructive").intent(Intent::Destructive),
            Button::new("Informational").intent(Intent::Informational),
        ];
        let disabled = vec![
            Button::new("Solid").disabled(true),
            Button::new("Outline")
                .variant(ButtonVariant::Outline)
                .disabled(true),
            Button::new("Ghost")
                .variant(ButtonVariant::Ghost)
                .disabled(true),
            Button::new("Danger")
                .variant(ButtonVariant::Danger)
                .disabled(true),
        ];
        let iconic = vec![
            Button::new("Save").icon("save"),
            Button::new("Run").icon("play").intent(Intent::Constructive),
            Button::new("Search")
                .variant(ButtonVariant::Outline)
                .icon("search"),
            Button::new("Delete")
                .variant(ButtonVariant::Danger)
                .icon("trash"),
        ];
        Self {
            groups: vec![
                ("VARIANTS", variants),
                ("SIZES", sizes),
                ("INTENTS (SOLID)", intents),
                ("DISABLED", disabled),
                ("WITH ICONS", iconic),
            ],
        }
    }

    /// Per-button rects, parallel to `groups`. Rows wrap against
    /// `content.w`: a button that would overflow the content rect starts
    /// a new line instead of getting cropped by the window edge.
    fn layout(&self, content: Rect) -> Vec<Vec<Rect>> {
        let right = content.x + content.w;
        let mut all = Vec::with_capacity(self.groups.len());
        let mut y = content.y;
        for (_, group) in &self.groups {
            y += LABEL_H;
            let mut rects: Vec<Rect> = Vec::with_capacity(group.len());
            let mut line_start = 0;
            let mut x = content.x;
            let mut line_h: f32 = 0.0;
            for button in group {
                let (w, h) = button.preferred_size();
                if x > content.x && x + w > right {
                    // Wrap: close the current line (centering buttons of
                    // differing heights on its baseline) and start fresh.
                    for r in &mut rects[line_start..] {
                        r.y += (line_h - r.h) / 2.0;
                    }
                    line_start = rects.len();
                    x = content.x;
                    y += line_h + GAP;
                    line_h = 0.0;
                }
                rects.push(Rect::new(x, y, w, h));
                x += w + GAP;
                line_h = line_h.max(h);
            }
            for r in &mut rects[line_start..] {
                r.y += (line_h - r.h) / 2.0;
            }
            all.push(rects);
            y += line_h + ROW_GAP;
        }
        all
    }

    /// Natural height of all button rows (page scrolling needs it).
    pub fn content_height(&self, content: Rect) -> f32 {
        self.layout(content)
            .iter()
            .flatten()
            .map(|r| r.y + r.h)
            .fold(content.y, f32::max)
            - content.y
            + GAP
    }

    pub fn handle_event(&mut self, event: &WidgetEvent, content: Rect) -> EventResult {
        let rects = self.layout(content);
        let mut result = EventResult::IGNORED;
        for (group, row) in self.groups.iter_mut().zip(&rects) {
            for (button, rect) in group.1.iter_mut().zip(row) {
                result = result.merge(button.handle_event(event, *rect));
            }
        }
        result
    }

    pub fn render(&self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let rects = self.layout(content);
        for ((label, group), row) in self.groups.iter().zip(&rects) {
            // The group label sits one LABEL_H above the first line (the
            // tallest button of a line carries no centering offset, so the
            // minimum y is the line top).
            let top = row.iter().map(|r| r.y).fold(f32::INFINITY, f32::min);
            group_label(c, label, content.x, top - LABEL_H, theme);
            for (button, rect) in group.iter().zip(row) {
                button.render(c, *rect, theme);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Narrow viewport (~500px of content): rows must wrap instead of
    /// running past `content.w` and getting cropped by the window edge.
    #[test]
    fn buttons_wrap_rows_in_narrow_content() {
        let section = ButtonsSection::new();
        let content = Rect::new(288.0, 80.0, 500.0, 900.0);
        let rects = section.layout(content);

        let right = content.x + content.w;
        for (row, (label, _)) in rects.iter().zip(&section.groups) {
            for r in row {
                assert!(
                    r.x + r.w <= right + 0.5,
                    "{label}: button at x={} w={} overflows content right edge {right}",
                    r.x,
                    r.w
                );
            }
        }

        // At 500px at least one group actually wrapped: a wrapped line
        // restarts at content.x, so some group hosts it more than once.
        let wrapped = rects
            .iter()
            .any(|row| row.iter().filter(|r| r.x == content.x).count() > 1);
        assert!(wrapped, "narrow content must force at least one wrap");
    }

    /// Wide content: everything fits on one line per group, exactly as
    /// before — wrapping must not change the roomy layout.
    #[test]
    fn buttons_keep_single_lines_in_wide_content() {
        let section = ButtonsSection::new();
        let content = Rect::new(288.0, 80.0, 1272.0, 900.0);
        let rects = section.layout(content);
        for row in &rects {
            assert_eq!(
                row.iter().filter(|r| r.x == content.x).count(),
                1,
                "wide content must keep each group on a single line"
            );
        }
    }
}
