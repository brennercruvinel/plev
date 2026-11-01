//! Separator page: a horizontal rule between two captions and a
//! vertical one between two columns.

use engine::drawing::compositor::Compositor;
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

    pub fn content_height(&self, _content: Rect, theme: &Theme) -> f32 {
        let line = theme.typography.role(TypeRole::Body).line_height;
        line * 3.0 + theme.spacing.lg * 2.0 + theme.spacing.xs
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
        let w = content.w.min(theme.size.readable_max_w / 2.0);
        let color = theme.colors.text_mid.0;
        let mut y = content.y;
        text(c, "above the rule", run, (content.x, y), color, theme);
        y += line + theme.spacing.lg;
        Separator::Horizontal.render(c, Rect::new(content.x, y, w, theme.spacing.xs), theme);
        y += theme.spacing.xs + theme.spacing.lg;
        text(c, "left column", run, (content.x, y), color, theme);
        let mid = content.x + w / 2.0;
        Separator::Vertical.render(c, Rect::new(mid, y, theme.spacing.xs, line * 2.0), theme);
        text(
            c,
            "right column",
            run,
            (mid + theme.spacing.lg, y),
            color,
            theme,
        );
    }
}
