//! The gallery's own sidebar: the kit's HomeSidebar (`nav::sidebars`,
//! front G) with the navigation alone. A dropdown per group of the page
//! table (Foundations, Components, Apps Demo, front D's collapsible) with
//! its icon and its count of tabs, its tabs in the rail list, the current
//! tab inked; the logo is front F's. The group of the current tab is
//! marked and opens, at rest, when navigation reaches it, as the kit's
//! `active` dropdown starts open on a page that loads; the others keep
//! how the person left them.
//!
//! Where the panel stands (peeking by default, pinned beside the page by
//! the pin at the right of its head, a drawer closed by the X in the same
//! place) is the shell's dock; this is what it holds. A tab picked here comes back
//! as its page index, and the view asks `shell.page` for it, as a key
//! and an agent do.

use accord::SemId;
use engine::drawing::compositor::{Compositor, LayerId};
use engine::styling::theme::Theme;
use engine::widgets::content::{Collapsible, DisclosureList};
use engine::widgets::header::Logo;
use engine::widgets::nav::{DockMode, HomeRow, HomeSidebar, NavLink};
use engine::widgets::overlay::sheet::paint_scrim;
use engine::widgets::prelude::{EventResult, Rect, WidgetEvent};
use engine::widgets::shell::Route;
use engine::widgets::wake::Wake;

use super::ShowcaseView;
use super::pages::{Group, PAGES};

/// The product the logo names.
pub const PRODUCT: &str = "plev";

/// A group's icon, the kit's own paths.
fn icon(group: Group) -> &'static str {
    match group {
        Group::Foundations => "kit-colors",
        Group::Components => "kit-buttons",
        Group::AppsDemo => "kit-explore",
    }
}

/// Where page `i` is: its group's row and its place in the group.
pub fn place(i: usize) -> (usize, usize) {
    let group = PAGES[i].group;
    let row = Group::ALL
        .iter()
        .position(|g| *g == group)
        .expect("every page is in a group");
    let at = PAGES[..i].iter().filter(|p| p.group == group).count();
    (row, at)
}

/// The page at place `at` of row `row`.
pub fn page_at(row: usize, at: usize) -> Option<usize> {
    let group = *Group::ALL.get(row)?;
    PAGES
        .iter()
        .enumerate()
        .filter(|(_, p)| p.group == group)
        .nth(at)
        .map(|(i, _)| i)
}

pub struct Navigation {
    pub sidebar: HomeSidebar,
    /// The page the sidebar marks.
    current: usize,
}

impl Navigation {
    /// The groups, the group of page `current` open on it.
    pub fn new(current: usize) -> Self {
        let current = current.min(PAGES.len() - 1);
        let (open, _) = place(current);
        let rows = Group::ALL
            .iter()
            .enumerate()
            .map(|(r, g)| {
                let titles = PAGES.iter().filter(|p| p.group == *g).map(|p| p.title);
                // at rest from the start
                let trigger = Collapsible::new(g.label())
                    .icon(icon(*g))
                    .counter(g.pages().to_string())
                    .open(r == open);
                HomeRow::Menu(trigger, DisclosureList::new(titles))
            })
            .collect();
        let mut sidebar = HomeSidebar::new(rows, "", NavLink::new("")).library(false);
        sidebar.logo = Logo::new(PRODUCT);
        let mut nav = Self {
            sidebar,
            current: 0,
        };
        nav.set_current(current);
        nav
    }

    pub fn current(&self) -> usize {
        self.current
    }

    /// The groups' dropdowns and their lists, in `Group::ALL` order.
    pub fn menus(&self) -> impl Iterator<Item = (&Collapsible, &DisclosureList)> {
        self.sidebar.rows.iter().filter_map(|r| match r {
            HomeRow::Menu(m, l) => Some((m, l)),
            HomeRow::Link(_) => None,
        })
    }

    /// Marks page `i`: its tab inked, its group marked and open; the
    /// other groups' tabs none.
    pub fn set_current(&mut self, i: usize) {
        let i = i.min(PAGES.len() - 1);
        self.current = i;
        let (row, at) = place(i);
        for (r, item) in self.sidebar.rows.iter_mut().enumerate() {
            if let HomeRow::Menu(m, links) = item {
                let on = r == row;
                // the page loads with its group open, as the kit's
                // `active` dropdown: at rest, nothing slides
                let mut next = m.clone().current(on);
                if on && !m.is_open() {
                    next = next.open(true);
                }
                *m = next;
                links.set_current(on.then_some(at));
            }
        }
    }

    /// The tab a list marked that is not the current page: what a
    /// press picked.
    fn picked(&self) -> Option<usize> {
        let here = place(self.current);
        self.menus()
            .enumerate()
            .find_map(|(r, (_, l))| {
                l.current_link()
                    .filter(|at| (r, *at) != here)
                    .map(|at| (r, at))
            })
            .and_then(|(r, at)| page_at(r, at))
    }

    /// An event on the panel at `bounds`, and the page a tab picked, if
    /// one was. The marks stay on the current page until the view moves
    /// them ([`Navigation::set_current`]).
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        bounds: Rect,
        theme: &Theme,
    ) -> (EventResult, Option<usize>) {
        let r = self.sidebar.handle_event(event, bounds, theme);
        if !r.clicked {
            return (r, None);
        }
        let picked = self.picked();
        if picked.is_some() {
            self.set_current(self.current);
        }
        (r, picked)
    }

    /// Where page `i`'s tab is in the panel at `bounds`, while its group
    /// is open (it may be scrolled out of the body).
    pub fn tab_rect(&self, i: usize, bounds: Rect, theme: &Theme) -> Option<Rect> {
        let (row, at) = place(i);
        let r = *self.sidebar.layout(bounds, theme).rows.get(row)?;
        let (m, links) = self.menus().nth(row)?;
        if !m.is_open() {
            return None;
        }
        let content = m.content_rect(r, links.height(r.w, theme), theme);
        links.link_rects(content, theme).get(at).copied()
    }

    /// Where group `row`'s dropdown trigger is in the panel at `bounds`.
    pub fn trigger_rect(&self, row: usize, bounds: Rect, theme: &Theme) -> Option<Rect> {
        let r = *self.sidebar.layout(bounds, theme).rows.get(row)?;
        Some(self.menus().nth(row)?.0.trigger_rect(r, theme))
    }

    pub fn wake(&mut self, dt: f32) -> Wake {
        self.sidebar.wake(dt)
    }

    pub fn render(&self, c: &mut Compositor, bounds: Rect, theme: &Theme) {
        self.sidebar.render(c, bounds, theme);
    }
}

// The sidebar in the view: where the dock puts it, what reaches it.

impl ShowcaseView {
    /// The panel's rect now, while any of it shows.
    pub(super) fn panel_rect(&self) -> Option<Rect> {
        self.shell.layout(&self.theme).dock.and_then(|d| d.panel)
    }

    /// Pins the sidebar beside the page (`true`) or lets it peek, as the
    /// pin does: tests and tools that read the page at a known width.
    pub fn pin_sidebar(&mut self, on: bool) {
        let w = self.shell.dock_window().w;
        let theme = &self.theme;
        if let Some(dock) = self.shell.dock.as_mut()
            && dock.is_pinned() != on
        {
            dock.toggle_pin(theme, w);
        }
    }

    /// The strip of the panel at `panel` the logo and the pin sit in.
    pub(super) fn panel_head(&self, panel: Rect) -> Rect {
        let body = HomeSidebar::body_rect(panel, &self.theme);
        Rect::new(panel.x, panel.y, panel.w, body.y - panel.y)
    }

    /// The head's buttons' part of a pointer event on the panel at
    /// `panel`: the pin's and the open drawer's close's, merged, and the
    /// ask a click on one makes (`toggle` on the pin, `activate` on the
    /// close).
    fn head_event(
        &mut self,
        event: &WidgetEvent,
        panel: Rect,
    ) -> (EventResult, Option<(SemId, &'static str)>) {
        let head = self.panel_head(panel);
        let w = self.shell.dock_window().w;
        let theme = &self.theme;
        let Some(dock) = self.shell.dock.as_mut() else {
            return (EventResult::IGNORED, None);
        };
        let pin = dock.pin_event(event, head, theme, w);
        let close = dock.close_event(event, head, theme, w);
        let ask = if pin.clicked {
            dock.pin_id().map(|id| (id, "toggle"))
        } else if close.clicked {
            dock.close_id().map(|id| (id, "activate"))
        } else {
            None
        };
        (pin.merge(close), ask)
    }

    /// The chrome's part of a pointer event: `(taken, changed)`. The dock
    /// takes the drawer's menu button and the open drawer; the panel
    /// takes what is on it, and a tab picked there asks `shell.page`.
    /// What the chrome leaves is the page's, though the panel still hears
    /// a pointer that left it (its hover goes) and a release (a press
    /// ends).
    pub(super) fn chrome_event(&mut self, event: &WidgetEvent) -> (bool, bool) {
        let panel = self.panel_rect();
        match self.shell.route(event, &self.theme) {
            Route::Shell(r) => (true, r.changed || r.clicked),
            Route::Panel => {
                let Some(bounds) = panel.or_else(|| self.panel_rect()) else {
                    return (true, false);
                };
                let (pin, ask) = self.head_event(event, bounds);
                if let Some((id, action)) = ask {
                    // the person's ask, through the dispatcher
                    self.ask_sidebar(&id, action);
                    return (true, true);
                }
                let (r, picked) = self.nav.handle_event(event, bounds, &self.theme);
                let r = r.merge(pin);
                if let Some(i) = picked {
                    self.ask_page(i);
                    let w = self.shell.dock_window().w;
                    if let Some(dock) = self.shell.dock.as_mut() {
                        dock.picked(&self.theme, w);
                    }
                    return (true, true);
                }
                (true, r.changed || r.clicked)
            }
            Route::Page => {
                let heard = matches!(
                    event,
                    WidgetEvent::MouseMove { .. } | WidgetEvent::MouseUp { .. }
                );
                let changed = match panel {
                    Some(bounds) if heard => {
                        let pin = self.head_event(event, bounds).0.changed;
                        self.nav.handle_event(event, bounds, &self.theme).0.changed || pin
                    }
                    _ => false,
                };
                (false, changed)
            }
        }
    }

    /// Escape: the open drawer closes, a panel shown by the focus hides
    /// and the focus leaves it.
    pub(super) fn escape_sidebar(&mut self) -> bool {
        let w = self.shell.dock_window().w;
        let theme = &self.theme;
        let escaped = self.shell.dock.as_mut().is_some_and(|d| d.escape(theme, w));
        self.blur_sidebar() || escaped
    }

    /// The sidebar's clock and the dock's (its pauses and its slide).
    pub(super) fn sidebar_wake(&mut self, dt: f32) -> Wake {
        let nav = self.nav.wake(dt);
        nav.merge(self.shell.wake(dt, &self.theme))
    }

    /// The panel where the dock has it: beside the page while pinned, on
    /// the overlay layer while it floats over the page, over the drawer's
    /// scrim.
    pub(super) fn render_sidebar(&self, c: &mut Compositor, overlay: LayerId) {
        let theme = &self.theme;
        let Some(d) = self.shell.layout(theme).dock else {
            return;
        };
        if d.mode == DockMode::Drawer && d.shown > 0.0 {
            let window = Rect::new(0.0, 0.0, self.width, self.height);
            paint_scrim(c, overlay, window, &theme.overlay.drawer.panel, d.shown);
        }
        let Some(panel) = d.panel else {
            return;
        };
        let layer = if d.floats { overlay } else { LayerId::DEFAULT };
        let head = self.panel_head(panel);
        let w = self.shell.dock_window().w;
        c.on_layer(layer, |c| {
            self.nav.render(c, panel, theme);
            if let Some(dock) = &self.shell.dock {
                dock.render_pin(c, head, theme, w);
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_page_has_one_place_and_back() {
        for (i, page) in PAGES.iter().enumerate() {
            let (row, at) = place(i);
            assert_eq!(page_at(row, at), Some(i), "{}", page.title);
        }
        assert_eq!(page_at(Group::ALL.len(), 0), None);
    }

    #[test]
    fn the_current_page_marks_its_tab_and_its_group_alone() {
        let last = PAGES.len() - 1;
        let mut nav = Navigation::new(0);
        nav.set_current(last);
        let (row, at) = place(last);
        for (r, (m, l)) in nav.menus().enumerate() {
            assert_eq!(l.current_link(), (r == row).then_some(at));
            if r == row {
                assert!(m.is_open(), "navigation opens the group");
            }
        }
        assert_eq!(nav.menus().count(), Group::ALL.len());
    }
}
