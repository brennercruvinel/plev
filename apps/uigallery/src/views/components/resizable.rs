//! Resizable page: a split pane with a draggable divider.
//!
//! Projected (task-75 phase 2b): the split, `resizable_page.split`, named
//! "panes", as the comps projects it, described by the first pane's share;
//! `set_ratio` moves the divider where a drag would. A reader has no way
//! to ask it yet: the tree sets a value only through `set_value` (and a
//! select's `select`), and the contract has no splitter role.

use accord::{Request, SemId, SnapshotBuilder, Value};
use engine::drawing::compositor::Compositor;
use engine::styling::theme::Theme;
use engine::widgets::prelude::{EventResult, Rect, SplitDirection, SplitPane, WidgetEvent};

use super::{After, Component, child_id, panel, root_id, text, under};

pub struct ResizablePage {
    root: Option<SemId>,
    split: SplitPane,
}

impl ResizablePage {
    pub fn new(theme: &Theme) -> Self {
        let root = root_id("resizable_page");
        let split = SplitPane::new(SplitDirection::Horizontal, 0.35, theme);
        let split = match child_id(root.as_ref(), "split") {
            Some(id) => split.sem(id),
            None => split,
        };
        Self { root, split }
    }

    fn rect(content: Rect, theme: &Theme) -> Rect {
        Rect::new(content.x, content.y, content.w, theme.size.card_min_w)
    }

    pub fn content_height(&self, _content: Rect, theme: &Theme) -> f32 {
        theme.size.card_min_w
    }

    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        content: Rect,
        theme: &Theme,
    ) -> EventResult {
        self.split.handle_event(event, Self::rect(content, theme))
    }

    pub fn render(&self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let rect = Self::rect(content, theme);
        panel(c, rect, theme);
        let first = self.split.first_rect(rect);
        let second = self.split.second_rect(rect);
        let style = theme.typography.base_2r();
        let pad = theme.spacing.lg;
        text(
            c,
            &format!("first pane: {:.0}px (35% desired)", first.w),
            style.font_size,
            style.font_weight,
            first.x + pad,
            first.y + pad,
            theme.colors.text_mid.0,
        );
        text(
            c,
            "second pane",
            style.font_size,
            style.font_weight,
            second.x + pad,
            second.y + pad,
            theme.colors.text_mid.0,
        );
        self.split.render(c, rect, theme);
    }
}

impl Component for ResizablePage {
    fn root(&self) -> Option<&SemId> {
        self.root.as_ref()
    }

    fn project(&self, b: &mut SnapshotBuilder, root: &SemId) {
        if let Some(id) = self.split.sem_id() {
            let share = (self.split.ratio() * 100.0).round();
            self.split
                .project(b, id, "panes", Some(root))
                .description(format!("the first pane takes {share}%"));
        }
    }

    /// `set_ratio` moves the divider, as a drag does.
    fn run(&mut self, req: &Request) -> Result<After, String> {
        let on_split = self.split.sem_id() == Some(&req.target);
        match (req.action.as_str(), &req.param) {
            ("set_ratio", Some(Value::Num(r))) if on_split => {
                self.split.set_ratio(*r as f32);
                Ok(After::Nothing)
            }
            _ => Err(format!(
                "the resizable page has no {} on {}",
                req.action, req.target
            )),
        }
    }

    /// Nothing of the split takes focus; the page's nodes are its own all
    /// the same.
    fn focus(&mut self, id: &SemId) -> bool {
        self.root.as_ref().is_some_and(|root| under(root, id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn divider_drags() {
        let theme = Theme::hoff();
        let mut page = ResizablePage::new(&theme);
        let content = Rect::new(288.0, 80.0, 1272.0, 900.0);
        let d = page
            .split
            .divider_rect(ResizablePage::rect(content, &theme));
        let (x, y) = (d.x + 1.0, d.y + 20.0);
        page.handle_event(&WidgetEvent::MouseDown { x, y }, content, &theme);
        assert!(page.split.is_dragging());
        page.handle_event(&WidgetEvent::MouseMove { x: x + 100.0, y }, content, &theme);
        assert!(page.split.ratio() > 0.35);
    }
}
