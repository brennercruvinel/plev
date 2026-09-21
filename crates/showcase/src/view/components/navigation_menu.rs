//! Navigation Menu page: nav links in every state (active, hover, with
//! a badge, truncated) in a column, and the rail form beside them. A
//! click moves the active mark, like a real sidebar.

use comps::nav::NavLink;
use comps::prelude::{Badge, EventResult, Rect, WidgetEvent};
use engine::compositor::Compositor;
use engine::theme::Theme;

pub struct NavigationMenuPage {
    links: Vec<NavLink>,
}

struct Layout {
    links: Vec<Rect>,
    rail_link: Rect,
    total_h: f32,
}

impl NavigationMenuPage {
    pub fn new() -> Self {
        Self {
            links: vec![
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
            ],
        }
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
        let mut clicked = None;
        for (i, (link, rect)) in self.links.iter_mut().zip(&l.links).enumerate() {
            let r = link.handle_event(event, *rect);
            if r.clicked {
                clicked = Some(i);
            }
            result = result.merge(r);
        }
        if let Some(i) = clicked {
            for (k, link) in self.links.iter_mut().enumerate() {
                link.active = k == i;
            }
        }
        result
    }

    pub fn render(&self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let l = self.layout(content, theme);
        for (link, rect) in self.links.iter().zip(&l.links) {
            link.render(c, *rect, theme);
        }
        self.links[0].render(c, l.rail_link, theme);
    }
}
