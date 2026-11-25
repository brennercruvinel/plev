//! Separator page: a horizontal rule between two captions and a
//! vertical one between two columns.

use engine::drawing::compositor::{Compositor, SceneNode, TextNodeKey};
use engine::drawing::text::TextMeasurer;
use engine::styling::theme::Theme;
use engine::styling::theme::TypeRole;
use engine::widgets::content::Separator;
use engine::widgets::prelude::{EventResult, Rect, WidgetEvent};

use super::text;

pub struct SeparatorPage;

impl SeparatorPage {
    pub fn new() -> Self {
        Self
    }

    /// The rule's width: the content's, up to half the readable measure.
    fn width(content: Rect, theme: &Theme) -> f32 {
        content.w.min(theme.size.readable_max_w / 2.0)
    }

    /// Each column's text and where it wraps: either side of the
    /// vertical rule, inside the gap the rule keeps from them.
    fn columns(content: Rect, theme: &Theme) -> [(&'static str, f32, f32); 2] {
        let w = Self::width(content, theme);
        let mid = content.x + w / 2.0;
        let gap = theme.spacing.lg;
        let left = (mid - gap - content.x).max(1.0);
        let right = (content.x + w - mid - gap).max(1.0);
        [
            ("left column", content.x, left),
            ("right column", mid + gap, right),
        ]
    }

    /// The columns' height: the taller of the two texts as they wrap (a
    /// phone, a larger text, a longer language), never under two lines.
    fn columns_h(content: Rect, theme: &Theme) -> f32 {
        let style = theme.typography.run(TypeRole::Body.run());
        Self::columns(content, theme)
            .iter()
            .map(|(t, _, w)| TextMeasurer::measure_styled(t, &style, Some(*w)).1)
            .fold(style.line_height * 2.0, f32::max)
    }

    pub fn content_height(&self, content: Rect, theme: &Theme) -> f32 {
        let line = theme.typography.role(TypeRole::Body).line_height;
        line + theme.spacing.lg * 2.0 + theme.spacing.xs + Self::columns_h(content, theme)
    }

    pub fn handle_event(
        &mut self,
        _event: &WidgetEvent,
        _content: Rect,
        _theme: &Theme,
    ) -> EventResult {
        EventResult::IGNORED
    }

    pub fn render(&self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let run = TypeRole::Body.run();
        let style = theme.typography.run(run);
        let line = style.line_height;
        let w = Self::width(content, theme);
        let color = theme.colors.text_mid.0;
        let mut y = content.y;
        text(c, "above the rule", run, (content.x, y), color, theme);
        y += line + theme.spacing.lg;
        Separator::Horizontal.render(c, Rect::new(content.x, y, w, theme.spacing.xs), theme);
        y += theme.spacing.xs + theme.spacing.lg;
        let h = Self::columns_h(content, theme);
        let mid = content.x + w / 2.0;
        Separator::Vertical.render(c, Rect::new(mid, y, theme.spacing.xs, h), theme);
        for (t, x, tw) in Self::columns(content, theme) {
            c.push(SceneNode::Text {
                key: TextNodeKey::from_style(t, &style, Some(tw)),
                x,
                y,
                color,
            });
        }
    }
}
