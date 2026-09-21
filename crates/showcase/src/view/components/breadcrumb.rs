//! Breadcrumb page: a trail that collapses when narrow; the clicked
//! crumb is echoed beside it.

use comps::nav::Breadcrumb;
use comps::prelude::{EventResult, Rect, WidgetEvent};
use engine::compositor::{Compositor, SceneNode, TextNodeKey};
use engine::theme::Theme;

pub struct BreadcrumbPage {
    crumbs: Breadcrumb,
    last_crumb: Option<usize>,
}

impl BreadcrumbPage {
    pub fn new() -> Self {
        Self {
            crumbs: Breadcrumb::new(["crates", "comps", "src", "content", "table.rs"]),
            last_crumb: None,
        }
    }

    fn rect(content: Rect, theme: &Theme) -> Rect {
        Rect::new(
            content.x,
            content.y,
            content.w.min(theme.size.readable_max_w / 2.0),
            theme.typography.base_2r().line_height,
        )
    }

    pub fn content_height(&self, _content: Rect, theme: &Theme) -> f32 {
        theme.typography.base_2r().line_height
    }

    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        content: Rect,
        theme: &Theme,
    ) -> EventResult {
        let (r, crumb) = self
            .crumbs
            .handle_event(event, Self::rect(content, theme), theme);
        if let Some(i) = crumb {
            self.last_crumb = Some(i);
        }
        r
    }

    pub fn render(&self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let rect = Self::rect(content, theme);
        self.crumbs.render(c, rect, theme);
        if let Some(i) = self.last_crumb {
            let note = format!("clicked crumb {i}");
            let style = theme.typography.caption_r();
            c.push(SceneNode::Text {
                key: TextNodeKey::from_style(&note, &style, None),
                x: rect.right() + theme.spacing.lg,
                y: rect.y,
                color: theme.glass.text_faint.0,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crumb_click_is_echoed() {
        let theme = Theme::hoff();
        let mut page = BreadcrumbPage::new();
        let content = Rect::new(40.0, 80.0, 1272.0, 900.0);
        let rects = page
            .crumbs
            .crumb_rects(BreadcrumbPage::rect(content, &theme), &theme);
        let (x, y) = rects[1].1.center();
        page.handle_event(&WidgetEvent::MouseDown { x, y }, content, &theme);
        let r = page.handle_event(&WidgetEvent::MouseUp { x, y }, content, &theme);
        assert!(r.clicked);
        assert_eq!(page.last_crumb, Some(1));
    }
}
