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
use engine::widgets::form::input_group::{Addon, InputGroup};
use engine::widgets::form::tokens::Look;
use engine::widgets::header::Logo;
use engine::widgets::hit::hit_rect;
use engine::widgets::nav::{DockMode, HomeRow, HomeSidebar, NavLink, SidebarDock};
use engine::widgets::overlay::sheet::{Edge, paint_panel, paint_scrim};
use engine::widgets::prelude::{EventResult, Rect, WidgetEvent};
use engine::widgets::shell::{Route, SafeArea};
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
    /// The Search under the logo, a top bar's (`Look::Header`, front F)
    /// with its `⌘ K`: a press opens the command palette (`palette.rs`).
    pub search: InputGroup,
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
                    .grow_hit(true)
                    .open(r == open);
                HomeRow::Menu(trigger, DisclosureList::new(titles).grow_hit(true))
            })
            .collect();
        let mut sidebar = HomeSidebar::new(rows, "", NavLink::new("")).library(false);
        sidebar.logo = Logo::new(PRODUCT);
        let search = InputGroup::new("Search...")
            .look(Look::Header)
            .start(Addon::Icon("search"))
            .end(Addon::Hint("⌘ K".into()));
        let mut nav = Self {
            sidebar,
            search,
            current: 0,
        };
        nav.set_current(current);
        nav
    }

    pub fn current(&self) -> usize {
        self.current
    }

    /// The theme's measures: the strip under the logo is the Search's
    /// height.
    pub fn set_theme(&mut self, theme: &Theme) {
        self.sidebar.set_slot(self.search.height(theme));
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

    /// The panel at `bounds` on its own surface, or (`surface` off) on
    /// one the caller painted: the drawer's edge panel.
    pub fn render(&self, c: &mut Compositor, bounds: Rect, theme: &Theme, surface: bool) {
        if surface {
            self.sidebar.render(c, bounds, theme);
        } else {
            self.sidebar.render_parts(c, bounds, theme);
        }
        if let Some(at) = self.sidebar.slot_rect(bounds, theme) {
            if let Some(id) = self.search.sem_id() {
                let hit = hit_rect(at, theme);
                c.bind(id, [hit.x, hit.y, hit.w, hit.h]);
            }
            self.search.render(c, at, theme);
        }
    }

    /// A pointer event on the Search in the panel at `bounds`: its hover;
    /// `clicked` on a press, which asks the palette open.
    fn search_event(&mut self, event: &WidgetEvent, bounds: Rect, theme: &Theme) -> EventResult {
        let Some(at) = self.sidebar.slot_rect(bounds, theme) else {
            return EventResult::IGNORED;
        };
        let r = self.search.handle_event(event, at, theme);
        match *event {
            WidgetEvent::MouseDown { x, y } if hit_rect(at, theme).contains(x, y) => {
                r.merge(EventResult::clicked())
            }
            _ => r,
        }
    }

    /// What a press in the panel at `bounds` can land on, as drawn: the
    /// Search, and in the body each group's trigger and its open tabs.
    fn targets(&self, bounds: Rect, theme: &Theme) -> Vec<Rect> {
        let inside = |r: &Rect, o: Rect| {
            let (x, y) = r.center();
            o.contains(x, y)
        };
        let body = self.sidebar.body_rect(bounds, theme);
        let l = self.sidebar.layout(bounds, theme);
        let mut out: Vec<Rect> = self.sidebar.slot_rect(bounds, theme).into_iter().collect();
        for (row, r) in self.sidebar.rows.iter().zip(&l.rows) {
            let HomeRow::Menu(m, links) = row else {
                continue;
            };
            let trigger = m.trigger_rect(*r, theme);
            if inside(&trigger, body) {
                out.push(trigger);
            }
            if m.is_open() {
                let shown = m.shown_rect(*r, theme);
                let content = m.content_rect(*r, links.height(r.w, theme), theme);
                let open = links.link_rects(content, theme).into_iter();
                out.extend(open.filter(|t| inside(t, body) && inside(t, shown)));
            }
        }
        out
    }
}

/// `event` where the press it makes lands among `targets` (each drawn
/// at its rect, pressed in [`hit_rect`] around it, 44 px with a
/// finger): a point inside one drawing and no other press area stays;
/// one two press areas share, or in a grown margin, moves to the centre
/// of the nearest drawing, so that part alone hears it.
fn settle(event: &WidgetEvent, targets: &[Rect], theme: &Theme) -> WidgetEvent {
    let (x, y) = event.pos();
    let off = |r: &Rect| {
        let dx = (r.x - x).max(x - r.right()).max(0.0);
        let dy = (r.y - y).max(y - r.bottom()).max(0.0);
        dx.hypot(dy)
    };
    let hits: Vec<&Rect> = targets
        .iter()
        .filter(|r| hit_rect(**r, theme).contains(x, y))
        .collect();
    let Some(owner) = hits.iter().min_by(|a, b| off(a).total_cmp(&off(b))) else {
        return *event;
    };
    if hits.len() == 1 && owner.contains(x, y) {
        return *event;
    }
    let (x, y) = owner.center();
    match *event {
        WidgetEvent::MouseMove { .. } => WidgetEvent::MouseMove { x, y },
        WidgetEvent::MouseDown { .. } => WidgetEvent::MouseDown { x, y },
        WidgetEvent::MouseUp { .. } => WidgetEvent::MouseUp { x, y },
        other => other,
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

    /// The last input was a finger (`true`) or a pointer, as the platform
    /// tells it: a finger has no hover to peek with, so the sidebar is a
    /// drawer behind the menu button. `true` when that changed.
    pub fn set_touch(&mut self, on: bool) -> bool {
        let w = self.shell.dock_window().w;
        let theme = &self.theme;
        let Some(dock) = self.shell.dock.as_mut() else {
            return false;
        };
        let before = dock.mode(theme, w);
        dock.set_touch(on, theme, w);
        dock.mode(theme, w) != before
    }

    /// The window's safe area (a notch, a home bar, rounded corners), in
    /// logical pixels: the chrome, the dock and the page stand inside it.
    pub fn set_safe_area(&mut self, safe_area: SafeArea) {
        self.shell.set_safe_area(safe_area);
    }

    /// The strip of the panel at `panel` the logo and the pin sit in.
    pub(super) fn panel_head(&self, panel: Rect) -> Rect {
        HomeSidebar::head_rect(panel, &self.theme)
    }

    /// `event` on the panel at `panel`, settled among its press targets
    /// (the pin or the close, the Search, the triggers, the open tabs).
    fn settle_panel(&self, event: &WidgetEvent, panel: Rect) -> WidgetEvent {
        let theme = &self.theme;
        let mut targets = self.nav.targets(panel, theme);
        let w = self.shell.dock_window().w;
        if let Some(dock) = &self.shell.dock
            && (dock.has_pin(theme, w) || dock.has_close(theme, w))
        {
            targets.push(SidebarDock::pin_rect(self.panel_head(panel), theme));
        }
        settle(event, &targets, theme)
    }

    /// The head's part of a pointer event on the panel at `panel`: the
    /// pin's, the open drawer's close's and the Search's, merged, and the
    /// ask a click on one makes (`toggle` on the pin, `activate` on the
    /// close and on the Search).
    fn head_event(
        &mut self,
        event: &WidgetEvent,
        panel: Rect,
    ) -> (EventResult, Option<(SemId, &'static str)>) {
        let head = self.panel_head(panel);
        let w = self.shell.dock_window().w;
        let theme = &self.theme;
        let search = self.nav.search_event(event, panel, theme);
        let Some(dock) = self.shell.dock.as_mut() else {
            return (search, None);
        };
        let pin = dock.pin_event(event, head, theme, w);
        let close = dock.close_event(event, head, theme, w);
        let ask = if pin.clicked {
            dock.pin_id().map(|id| (id, "toggle"))
        } else if close.clicked {
            dock.close_id().map(|id| (id, "activate"))
        } else if search.clicked {
            self.nav.search.sem_id().cloned().map(|id| (id, "activate"))
        } else {
            None
        };
        (pin.merge(close).merge(search), ask)
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
                let event = &self.settle_panel(event, bounds);
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
        let dock = self.shell.dock.as_mut();
        let drawer = dock.as_ref().is_some_and(|d| d.is_drawer_open());
        let menu = dock.as_ref().and_then(|d| d.menu_id());
        let escaped = dock.is_some_and(|d| d.escape(theme, w));
        // the drawer shut on the keyboard: its focus back to the menu
        // button that opened it
        if drawer
            && escaped
            && self.sidebar_focus.is_some()
            && let Some(menu) = menu
            && self.focus_sidebar(&menu)
        {
            return true;
        }
        self.blur_sidebar() || escaped
    }

    /// The sidebar's clock and the dock's (its pauses and its slide).
    pub(super) fn sidebar_wake(&mut self, dt: f32) -> Wake {
        let nav = self.nav.wake(dt);
        nav.merge(self.shell.wake(dt, &self.theme))
    }

    /// The panel where the dock has it: beside the page while pinned, on
    /// the overlay layer while it floats over the page; as a drawer, over
    /// its scrim on front B's edge panel (`sheet::paint_panel`, the
    /// theme's drawer surface: its fill, its free edge's rule, corners and
    /// shadow).
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
        let drawer = d.mode == DockMode::Drawer;
        if drawer {
            let window = Rect::new(0.0, 0.0, self.width, self.height);
            let t = &theme.overlay.drawer.panel;
            paint_panel(c, overlay, panel, window, Edge::Left, t);
        }
        c.on_layer(layer, |c| {
            self.nav.render(c, panel, theme, !drawer);
            if let Some(dock) = &self.shell.dock {
                dock.render_pin(c, head, theme, w);
            }
        });
    }
}

#[cfg(test)]
mod tests;
