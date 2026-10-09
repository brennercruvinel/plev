//! Code Block page: a mono block with its caption.

use comps::content::CodeBlock;
use comps::prelude::{EventResult, Rect, WidgetEvent};
use engine::compositor::Compositor;
use engine::theme::Theme;

pub struct CodeBlockPage {
    code: CodeBlock,
}

impl CodeBlockPage {
    pub fn new() -> Self {
        Self {
            code: CodeBlock::new(
                "cargo run -p showcase -- \"Code Block\" hoff\ncargo test -p comps matrix",
            )
            .caption("shell"),
        }
    }

    fn rect(&self, content: Rect, theme: &Theme) -> Rect {
        let w = content.w.min(theme.size.readable_max_w);
        Rect::new(content.x, content.y, w, self.code.height_for(w, theme))
    }

    pub fn content_height(&self, content: Rect, theme: &Theme) -> f32 {
        self.rect(content, theme).h
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
        self.code.render(c, self.rect(content, theme), theme);
    }
}
