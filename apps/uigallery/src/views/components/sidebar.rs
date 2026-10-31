//! Sidebar page: the sidebar widget embedded in the page, pinned to its
//! full mode, with a brand, grouped links (fold a group by its caption)
//! and footer hints.

use engine::drawing::compositor::{Compositor, LayerId};
use engine::styling::theme::{SidebarMode, Theme};
use engine::widgets::nav::{NavGroup, NavLink, Sidebar};
use engine::widgets::prelude::{EventResult, Rect, WidgetEvent, rounded_rect_stroke};

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

    /// The sidebar's rect: its own width, as tall as its rows (folding a
    /// group shortens it), so nothing scrolls under a fade.
    fn rect(&self, content: Rect, theme: &Theme) -> Rect {
        let h = self.sidebar.natural_h(theme, content.w);
        Rect::new(content.x, content.y, theme.size.sidebar_w, h)
    }

    pub fn content_height(&self, content: Rect, theme: &Theme) -> f32 {
        self.rect(content, theme).h
    }

    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        content: Rect,
        theme: &Theme,
    ) -> EventResult {
        let rect = self.rect(content, theme);
        self.sidebar.origin = (rect.x, rect.y);
        self.sidebar.handle_event(event, theme, rect.w, rect.h).0
    }

    pub fn render(&mut self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let rect = self.rect(content, theme);
        self.sidebar.origin = (rect.x, rect.y);
        let frame = rect.outset(theme.control.edge_width);
        panel(c, frame, theme);
        self.sidebar
            .render(c, LayerId::DEFAULT, theme, rect.w, rect.h);
        // The sidebar's surface is a square: the frame's rim goes over it,
        // or its corners cover the rounded rim's arcs.
        c.push(rounded_rect_stroke(
            frame.x,
            frame.y,
            frame.w,
            frame.h,
            theme.radius.lg,
            theme.glass.edge_soft.0,
            theme.control.edge_width,
        ));
    }
}
