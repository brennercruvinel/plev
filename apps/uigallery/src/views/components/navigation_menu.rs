//! Navigation Menu page: nav links in every state (active, hover, with
//! a badge, truncated) in a column, and the rail form beside them. A
//! click moves the active mark, like a real sidebar.
//!
//! Projected (task-75 phase 2b): each link of the column,
//! `navigation_menu_page.link[<i>]`, as the comps projects it, described by
//! its shortcut hint or its badge; `activate` moves the active mark as a
//! click does. The rail form is a picture of the first link: its own
//! `NavLink`, with no id, that mirrors it.

use engine::widgets::nav::NavLink;
use engine::widgets::prelude::{Badge, EventResult, Rect, WidgetEvent};
use engine::drawing::compositor::Compositor;
use engine::styling::theme::Theme;
use accord::{Request, SemId, SnapshotBuilder};

use super::{After, Component, child_id, root_id, under};

pub struct NavigationMenuPage {
    root: Option<SemId>,
    links: Vec<NavLink>,
    /// The first link in the rail form: bound to nothing (the link is the
    /// column's), fed the first link's events so it hovers and presses
    /// with it, and active when it is.
    rail: NavLink,
    /// The link a screen reader or the keyboard focused (its ring shows);
    /// Escape or a click lets it go.
    focus: Option<usize>,
}

struct Layout {
    links: Vec<Rect>,
    rail_link: Rect,
    total_h: f32,
}

impl NavigationMenuPage {
    pub fn new() -> Self {
        let links = vec![
            NavLink::new("Workspace")
                .icon("layout-grid")
                .hint("1")
                .active(true),
            NavLink::new("Branches").icon("git-branch").hint("2"),
            NavLink::new("History")
                .icon("history")
                .badge(Badge::count("4")),
            NavLink::new("A label long enough to truncate in a narrow rail")
                .icon("file")
                .hint("9"),
        ];
        let rail = links[0].clone();
        let root = root_id("navigation_menu_page");
        let family = child_id(root.as_ref(), "link");
        let links = links
            .into_iter()
            .enumerate()
            .map(|(i, link)| match &family {
                Some(f) => link.sem(f.key(i.to_string())),
                None => link,
            })
            .collect();
        Self {
            root,
            links,
            rail,
            focus: None,
        }
    }

    /// The link behind `id`.
    fn at(&self, id: &SemId) -> Option<usize> {
        self.links.iter().position(|l| l.sem_id() == Some(id))
    }

    /// The focused link rings, the others do not.
    fn ring(&mut self) {
        for (i, link) in self.links.iter_mut().enumerate() {
            link.set_focused(self.focus == Some(i));
        }
    }

    /// The active mark on link `i`: the click's path and a request's.
    fn activate(&mut self, i: usize) {
        for (k, link) in self.links.iter_mut().enumerate() {
            link.active = k == i;
        }
        self.rail.active = i == 0;
    }

    fn layout(&self, content: Rect, theme: &Theme) -> Layout {
        let link_w = theme.size.sidebar_w - theme.spacing.md * 2.0;
        let link_h = NavLink::height(theme);
        let step = link_h + theme.spacing.xs;
        let links = (0..self.links.len())
            .map(|i| Rect::new(content.x, content.y + i as f32 * step, link_w, link_h))
            .collect::<Vec<_>>();
        let rail_w = theme.size.sidebar_rail_w - theme.spacing.md * 2.0;
        let rail_link = Rect::new(
            content.x + link_w + theme.spacing.md,
            content.y,
            rail_w,
            link_h,
        );
        Layout {
            links,
            rail_link,
            total_h: self.links.len() as f32 * step,
        }
    }

    pub fn content_height(&self, content: Rect, theme: &Theme) -> f32 {
        self.layout(content, theme).total_h
    }

    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        content: Rect,
        theme: &Theme,
    ) -> EventResult {
        let l = self.layout(content, theme);
        let mut result = EventResult::IGNORED;
        if matches!(event, WidgetEvent::MouseDown { .. }) && self.blur() {
            result = EventResult::changed();
        }
        let mut clicked = None;
        for (i, (link, rect)) in self.links.iter_mut().zip(&l.links).enumerate() {
            let r = link.handle_event(event, *rect);
            if r.clicked {
                clicked = Some(i);
            }
            result = result.merge(r);
        }
        if let Some(first) = l.links.first() {
            self.rail.handle_event(event, *first);
        }
        if let Some(i) = clicked {
            self.activate(i);
        }
        result
    }

    pub fn render(&self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let l = self.layout(content, theme);
        for (link, rect) in self.links.iter().zip(&l.links) {
            link.render(c, *rect, theme);
        }
        self.rail.render(c, l.rail_link, theme);
    }
}

impl Component for NavigationMenuPage {
    fn root(&self) -> Option<&SemId> {
        self.root.as_ref()
    }

    fn project(&self, b: &mut SnapshotBuilder, root: &SemId) {
        for link in &self.links {
            let Some(id) = link.sem_id() else {
                continue;
            };
            let mut node = link.project(b, id, Some(root));
            let aside = link
                .hint
                .as_deref()
                .or(link.badge.as_ref().map(|b| b.label.as_str()));
            if let Some(aside) = aside {
                node.description(aside);
            }
        }
        let focused = self.focus.and_then(|i| self.links.get(i));
        if let Some(id) = focused.and_then(NavLink::sem_id) {
            b.focus(id);
        }
    }

    /// A link's `activate` moves the active mark to it, as its click does.
    fn run(&mut self, req: &Request) -> Result<After, String> {
        match (self.at(&req.target), req.action.as_str()) {
            (Some(i), "activate") => {
                self.activate(i);
                Ok(After::Nothing)
            }
            _ => Err(format!(
                "the navigation menu page has no {} on {}",
                req.action, req.target
            )),
        }
    }

    fn focus(&mut self, id: &SemId) -> bool {
        if !self.root.as_ref().is_some_and(|root| under(root, id)) {
            return false;
        }
        self.focus = self.at(id);
        self.ring();
        true
    }

    fn blur(&mut self) -> bool {
        let had = self.focus.take().is_some();
        self.ring();
        had
    }
}
