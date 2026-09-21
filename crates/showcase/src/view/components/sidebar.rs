//! Sidebar page: the sidebar widget embedded in the page, pinned to its
//! full mode, with a brand, grouped links (fold a group by its caption)
//! and footer hints.

use comps::nav::{NavGroup, NavLink, Sidebar};
use comps::prelude::{EventResult, Rect, WidgetEvent};
use engine::compositor::{Compositor, LayerId};
use engine::theme::{SidebarMode, Theme};

use super::panel;

pub struct SidebarPage {
    sidebar: Sidebar,
}

impl SidebarPage {
    pub fn new() -> Self {
        let sidebar = Sidebar::new(vec![
            NavLink::new("Workspace").icon("layout-grid"),
            NavLink::new("Branches").icon("git-branch"),
            NavLink::new("History").icon("history"),
            NavLink::new("Search").icon("search"),
            NavLink::new("Settings").icon("settings"),
        ])
        .groups(vec![
            NavGroup::new("Repository", 3),
            NavGroup::new("Tools", 2),
        ])
        .brand("plev", "EMBEDDED SIDEBAR")
        .footer(vec!["click a caption to fold".into()])
        .fixed_mode(SidebarMode::Full);
        Self { sidebar }
    }

    fn rect(content: Rect, theme: &Theme) -> Rect {
        Rect::new(
            content.x,
            content.y,
            theme.size.sidebar_w,
            theme.size.card_min_w * 1.5,
        )
    }

    pub fn content_height(&self, _content: Rect, theme: &Theme) -> f32 {
        theme.size.card_min_w * 1.5
    }

    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        content: Rect,
        theme: &Theme,
    ) -> EventResult {
        let rect = Self::rect(content, theme);
        self.sidebar.origin = (rect.x, rect.y);
        self.sidebar.handle_event(event, theme, rect.w, rect.h).0
    }

    pub fn render(&mut self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let rect = Self::rect(content, theme);
        self.sidebar.origin = (rect.x, rect.y);
        panel(c, rect.outset(1.0), theme);
        self.sidebar
            .render(c, LayerId::DEFAULT, theme, rect.w, rect.h);
    }
}
