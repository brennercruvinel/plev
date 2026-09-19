//! Left rail: the design system [`Sidebar`] pinned to its rail mode
//! (icons only, `size.sidebar_rail_w` wide), three tabs in the band and
//! the settings link pinned in the foot.

use comps::nav::{NavLink, Sidebar as Rail};
use comps::prelude::{EventResult, WidgetEvent};
use engine::compositor::{Compositor, LayerId};
use engine::theme::{SidebarMode, Theme};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SidebarTab {
    Workspace,
    Branches,
    History,
    Settings,
}

impl SidebarTab {
    const ALL: [SidebarTab; 4] = [
        SidebarTab::Workspace,
        SidebarTab::Branches,
        SidebarTab::History,
        SidebarTab::Settings,
    ];

    fn index(self) -> usize {
        Self::ALL.iter().position(|t| *t == self).unwrap_or(0)
    }
}

pub struct Sidebar {
    pub active: SidebarTab,
    rail: Rail,
}

impl Sidebar {
    pub fn new() -> Self {
        let rail = Rail::new(vec![
            NavLink::new("Workspace").icon("layout-grid"),
            NavLink::new("Branches").icon("git-branch"),
            NavLink::new("History").icon("history"),
        ])
        .footer_links(vec![NavLink::new("Settings").icon("settings")])
        .fixed_mode(SidebarMode::Rail);
        Self {
            active: SidebarTab::Workspace,
            rail,
        }
    }

    /// Rail width for a theme.
    pub fn width(&self, theme: &Theme) -> f32 {
        self.rail.page_width(theme, 0.0)
    }

    pub fn set_active(&mut self, tab: SidebarTab) {
        self.active = tab;
        self.rail.set_active(tab.index());
    }

    /// Route a pointer event; returns the tab that was activated.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        theme: &Theme,
        vw: f32,
        vh: f32,
    ) -> (EventResult, Option<SidebarTab>) {
        let (r, nav) = self.rail.handle_event(event, theme, vw, vh);
        let tab = nav.and_then(|i| SidebarTab::ALL.get(i).copied());
        if let Some(tab) = tab {
            self.set_active(tab);
        }
        (r, tab)
    }

    pub fn render(&mut self, compositor: &mut Compositor, theme: &Theme, vw: f32, vh: f32) {
        self.rail
            .render(compositor, LayerId::DEFAULT, theme, vw, vh);
    }
}
