//! Breadcrumb page: a trail that collapses when narrow; the clicked
//! crumb is echoed beside it.
//!
//! Projected (task-75 phase 2b): the trail, `breadcrumb_page.trail`, named
//! "path", with its crumbs as the comps projects them (links, the last one
//! where the trail is); a crumb's `activate` is what its click does here,
//! the echo, which is a status, `breadcrumb_page.clicked`, a reader hears
//! as it changes.

use comps::nav::Breadcrumb;
use comps::prelude::{EventResult, Rect, WidgetEvent};
use engine::compositor::{Compositor, SceneNode, TextNodeKey};
use engine::theme::Theme;
use sem::{Request, Role, SemId, SnapshotBuilder};

use super::{After, Component, child_id, key_under, root_id, under};

pub struct BreadcrumbPage {
    root: Option<SemId>,
    crumbs: Breadcrumb,
    last_crumb: Option<usize>,
    /// The crumb a screen reader or the keyboard focused (its ring
    /// shows); Escape or a click lets it go.
    focus: Option<usize>,
}

impl BreadcrumbPage {
    pub fn new() -> Self {
        let root = root_id("breadcrumb_page");
        let crumbs = Breadcrumb::new(["crates", "comps", "src", "content", "table.rs"]);
        let crumbs = match child_id(root.as_ref(), "trail") {
            Some(id) => crumbs.sem(id),
            None => crumbs,
        };
        Self {
            root,
            crumbs,
            last_crumb: None,
            focus: None,
        }
    }

    /// The crumb behind `id`, `breadcrumb_page.trail[<i>]`.
    fn crumb_at(&self, id: &SemId) -> Option<usize> {
        let i = key_under(self.crumbs.sem_id()?, id)?.parse().ok()?;
        (i < self.crumbs.crumbs.len()).then_some(i)
    }

    /// What the page echoes for the crumb clicked last.
    fn echo(i: usize) -> String {
        format!("clicked crumb {i}")
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
        let blurred = matches!(event, WidgetEvent::MouseDown { .. }) && self.blur();
        let (r, crumb) = self
            .crumbs
            .handle_event(event, Self::rect(content, theme), theme);
        if let Some(i) = crumb {
            self.last_crumb = Some(i);
        }
        if blurred {
            return r.merge(EventResult::changed());
        }
        r
    }

    pub fn render(&self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let rect = Self::rect(content, theme);
        self.crumbs.render(c, rect, theme);
        if let Some(i) = self.last_crumb {
            let note = Self::echo(i);
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

impl Component for BreadcrumbPage {
    fn root(&self) -> Option<&SemId> {
        self.root.as_ref()
    }

    fn project(&self, b: &mut SnapshotBuilder, root: &SemId) {
        let Some(id) = self.crumbs.sem_id() else {
            return;
        };
        self.crumbs.project(b, id, "path", Some(root));
        if let (Some(i), Some(echo)) = (self.last_crumb, child_id(Some(root), "clicked")) {
            b.node(&echo, Role::Status, Self::echo(i)).parent(root);
        }
        if let Some(i) = self.focus {
            b.focus(id.key(i.to_string()));
        }
    }

    /// A crumb's `activate` is echoed, as its click is.
    fn run(&mut self, req: &Request) -> Result<After, String> {
        let last = self.crumbs.crumbs.len().saturating_sub(1);
        match (self.crumb_at(&req.target), req.action.as_str()) {
            (Some(i), "activate") if i != last => {
                self.last_crumb = Some(i);
                Ok(After::Nothing)
            }
            _ => Err(format!(
                "the breadcrumb page has no {} on {}",
                req.action, req.target
            )),
        }
    }

    fn focus(&mut self, id: &SemId) -> bool {
        if !self.root.as_ref().is_some_and(|root| under(root, id)) {
            return false;
        }
        let last = self.crumbs.crumbs.len().saturating_sub(1);
        self.focus = self.crumb_at(id).filter(|i| *i != last);
        self.crumbs.set_focused(self.focus);
        true
    }

    fn blur(&mut self) -> bool {
        self.crumbs.set_focused(None);
        self.focus.take().is_some()
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
