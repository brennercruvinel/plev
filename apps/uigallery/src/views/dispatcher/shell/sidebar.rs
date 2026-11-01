//! The sidebar as the shell's (task-104 front H, #163): `shell.sidebar`,
//! the navigation landmark the dock declares (`expanded`, `mode`,
//! `pinned`, a `toggle_expanded`), its pin `shell.sidebar.pin` (a toggle,
//! `on`), in the drawer its menu button `shell.sidebar.menu` and, open,
//! its close `shell.sidebar.close` (where the pin was), the Search under
//! the logo `shell.sidebar.search` (a button whose `activate` opens the
//! command palette, as ⌘K does), and while
//! it shows, the HomeSidebar's parts under it: the logo (a link to the
//! first page), each group's dropdown `shell.sidebar.row[<r>]` and, while
//! open, its tabs `shell.sidebar.row[<r>].links[<i>]`. A tab is the
//! `shell.page` it names: its `activate` selects that page, as
//! `shell.page select` does. Every actor asks the same way (the pin
//! clicked, a key on the focused tab, a reader's press, an agent's act)
//! and gets a receipt; while a modal holds the pointer the sidebar is
//! inert, as the page is.
//!
//! The keyboard (`dispatch::keyboard`) walks the sidebar after the page:
//! Tab past the page's last stop lands on the pin, which shows a peeking
//! panel; the focus inside keeps it; Tab out of it, or Escape, hides it
//! again unless it is pinned. A reader meets the same order
//! (`reader_semantics` builds on [`ShowcaseView::keyboard_semantics`]).

use accord::{Execution, Outcome, Role, SemId, Snapshot, SnapshotBuilder, Value};
use engine::styling::theme::Theme;
use engine::widgets::nav::{DockMode, SidebarDock};

use super::super::super::navigation::{self, Navigation};
use super::super::super::palette;
use super::super::super::{PAGES, ShowcaseView};
use super::super::detached;
use super::Asks;

/// The landmark's id and name.
pub(crate) const SIDEBAR: &str = "shell.sidebar";
const LABEL: &str = "Pages";

/// The page the logo leads to: where the gallery starts.
const HOME: usize = 0;

/// What of the view the sidebar projects from: where the dock has the
/// panel, what the panel holds, the theme, the dock's window width, and
/// whether a modal leaves it inert.
#[derive(Clone, Copy)]
pub(super) struct Bar<'a> {
    pub dock: &'a SidebarDock,
    pub nav: &'a Navigation,
    pub theme: &'a Theme,
    pub w: f32,
    pub inert: bool,
}

/// What a request on the sidebar asked of the view, applied right after
/// the dispatch (a page goes as `Asks::page`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SidebarAsk {
    Pin,
    Expand,
    Group(usize),
    /// A tab or the logo picked: the drawer closes.
    Picked,
    /// The Search pressed: the command palette opens (`palette.rs`).
    Search,
}

/// A part of the sidebar a request names.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Part {
    Root,
    Pin,
    Menu,
    Close,
    Search,
    Logo,
    Group(usize),
    /// A tab, by its page.
    Tab(usize),
}

/// Whether `id` is `root` or under it, by its path.
pub(super) fn under(id: &SemId, root: &SemId) -> bool {
    let mut at = Some(id.clone());
    while let Some(now) = at {
        if &now == root {
            return true;
        }
        at = now.parent_path();
    }
    false
}

fn part(bar: Bar<'_>, id: &SemId) -> Option<Part> {
    let dock = bar.dock;
    if dock.sem_id() == Some(id) {
        return Some(Part::Root);
    }
    if dock.pin_id().as_ref() == Some(id) {
        return Some(Part::Pin);
    }
    if dock.menu_id().as_ref() == Some(id) {
        return Some(Part::Menu);
    }
    if dock.close_id().as_ref() == Some(id) {
        return Some(Part::Close);
    }
    if bar.nav.search.sem_id() == Some(id) {
        return Some(Part::Search);
    }
    if bar.nav.sidebar.logo_id() == Some(id) {
        return Some(Part::Logo);
    }
    bar.nav.menus().enumerate().find_map(|(r, (m, links))| {
        if m.sem_id() == Some(id) {
            return Some(Part::Group(r));
        }
        let at = links.position(id)?;
        navigation::page_at(r, at).map(Part::Tab)
    })
}

/// Declares the sidebar under `parent`: the landmark, its pin (its menu
/// in the drawer), and while it shows, the HomeSidebar's parts.
pub(super) fn project(bar: Bar<'_>, b: &mut SnapshotBuilder, parent: Option<&SemId>) {
    let Some(id) = bar.dock.sem_id() else {
        return;
    };
    bar.dock.project(b, LABEL, parent, bar.theme, bar.w);
    if bar.inert
        && let Some(mut node) = b.reopen(id)
    {
        node.inert(true);
    }
    // the Search heads the panel with the pin: ⌘K opens the palette
    // whether the panel shows or not
    if let Some(search) = bar.nav.search.sem_id() {
        let mut node = b.node(search, Role::Button, palette::NAME);
        node.parent(id).description("⌘ K");
        node.action("activate");
    }
    if bar.dock.expanded(bar.theme, bar.w) {
        bar.nav.sidebar.project_parts(b, id);
    }
}

/// A request on the sidebar, recorded for the view: the pin flips, the
/// panel shows or hides (the drawer opens or closes), a dropdown opens or
/// closes, a tab selects its page.
pub(super) fn execute(bar: Bar<'_>, asks: &mut Asks, action: &str, target: &SemId) -> Execution {
    let (theme, w) = (bar.theme, bar.w);
    let mut outcome = Outcome::default();
    let mut put = |key: &str, value: Value| {
        outcome.result.insert(key.to_owned(), value);
    };
    match (action, part(bar, target)) {
        ("toggle", Some(Part::Pin)) => {
            asks.sidebar = Some(SidebarAsk::Pin);
            put("on", Value::Bool(!bar.dock.is_pinned()));
        }
        ("toggle_expanded", Some(Part::Root))
        | ("activate", Some(Part::Menu))
        | ("activate", Some(Part::Close)) => {
            asks.sidebar = Some(SidebarAsk::Expand);
            put("expanded", Value::Bool(!bar.dock.expanded(theme, w)));
        }
        ("toggle_expanded", Some(Part::Group(r))) => {
            asks.sidebar = Some(SidebarAsk::Group(r));
            let open = bar.nav.menus().nth(r).is_some_and(|(m, _)| m.is_open());
            put("expanded", Value::Bool(!open));
        }
        ("activate", Some(Part::Search)) => {
            asks.sidebar = Some(SidebarAsk::Search);
            put("open", Value::Bool(true));
        }
        ("activate", Some(at @ (Part::Tab(_) | Part::Logo))) => {
            let page = match at {
                Part::Tab(page) => page,
                _ => HOME,
            };
            asks.page = Some(page);
            asks.sidebar = Some(SidebarAsk::Picked);
            put("page", Value::Text(PAGES[page].title.to_owned()));
        }
        _ => {
            return Execution::Failed(format!("the sidebar has no {action} on {target}"));
        }
    }
    Execution::Done(outcome)
}

impl ShowcaseView {
    /// The sidebar as it projects now; `None` without a dock.
    pub(super) fn bar(&self) -> Option<Bar<'_>> {
        let dock = self.shell.dock.as_ref()?;
        Some(Bar {
            dock,
            nav: &self.nav,
            theme: &self.theme,
            w: self.shell.dock_window().w,
            inert: self.overlay_holds_pointer(),
        })
    }

    /// Whether `id` is the sidebar's (`shell.sidebar` or under it).
    pub fn is_sidebar(&self, id: &SemId) -> bool {
        SemId::parse(SIDEBAR).is_ok_and(|root| under(id, &root))
    }

    /// What the keyboard walks and a reader is shown (before what only a
    /// reader hears): the view's projection ([`ShowcaseView::semantics`])
    /// and the sidebar after it, with the focus where it is, the page's or
    /// the sidebar's.
    pub fn keyboard_semantics(&self) -> Option<Snapshot> {
        let seen = self.semantics();
        let Some(bar) = self.bar() else {
            return seen;
        };
        let mut b = SnapshotBuilder::new();
        // the open drawer is modal: the walk stays in its panel (the
        // trap), the page and the menu button under its scrim out of it
        let trapped =
            bar.dock.is_drawer_open() && bar.dock.mode(bar.theme, bar.w) == DockMode::Drawer;
        if let Some(seen) = seen.as_ref().filter(|_| !trapped) {
            detached::extend(&mut b, seen);
        }
        project(bar, &mut b, None);
        if let Some(menu) = bar.dock.menu_id().filter(|_| trapped)
            && let Some(mut node) = b.reopen(&menu)
        {
            node.inert(true);
        }
        if let Some(at) = &self.sidebar_focus {
            b.focus(at.clone());
        }
        b.finish()
            .inspect_err(|e| log::warn!("the sidebar's projection does not validate: {e}"))
            .ok()
            .or(seen)
    }

    /// The node of the sidebar the keyboard focus is on.
    pub fn sidebar_focus(&self) -> Option<&SemId> {
        self.sidebar_focus.as_ref()
    }

    /// The keyboard focus to `id` of the sidebar (Tab, a reader): the page
    /// lets its own go, the part shows its ring, and a peeking panel shows
    /// while the focus is in it. The landmark itself takes none; the menu
    /// button is outside the panel.
    pub(in crate::views) fn focus_sidebar(&mut self, id: &SemId) -> bool {
        let Some(bar) = self.bar() else {
            return false;
        };
        let at = part(bar, id);
        if matches!(at, None | Some(Part::Root)) || bar.inert {
            return false;
        }
        self.blur_page();
        self.sidebar_focus = Some(id.clone());
        self.nav.sidebar.focus_id(Some(id));
        self.nav.search.set_action_focus(at == Some(Part::Search));
        let w = self.shell.dock_window().w;
        let theme = &self.theme;
        if let Some(dock) = self.shell.dock.as_mut() {
            dock.set_pin_focused(at == Some(Part::Pin));
            dock.set_close_focused(at == Some(Part::Close));
            dock.set_menu_focused(at == Some(Part::Menu));
            dock.set_focus_inside(at != Some(Part::Menu), theme, w);
        }
        true
    }

    /// The keyboard focus leaves the sidebar (to the page, Escape): a
    /// panel the focus showed hides. `true` when it had it.
    pub(in crate::views) fn blur_sidebar(&mut self) -> bool {
        if self.sidebar_focus.take().is_none() {
            return false;
        }
        self.nav.sidebar.focus_id(None);
        self.nav.search.set_action_focus(false);
        let w = self.shell.dock_window().w;
        let theme = &self.theme;
        if let Some(dock) = self.shell.dock.as_mut() {
            dock.set_pin_focused(false);
            dock.set_close_focused(false);
            dock.set_menu_focused(false);
            dock.set_focus_inside(false, theme, w);
        }
        true
    }

    /// What a request on the sidebar asked, now that it ran.
    pub(super) fn apply_sidebar(&mut self, ask: SidebarAsk) {
        if ask == SidebarAsk::Search {
            // the palette takes the keys: the sidebar lets its focus go,
            // and has it back when the palette closes
            let opener = self.sidebar_focus.clone();
            self.blur_sidebar();
            self.open_palette(opener);
            return;
        }
        let w = self.shell.dock_window().w;
        let theme = &self.theme;
        let Some(dock) = self.shell.dock.as_mut() else {
            return;
        };
        match ask {
            SidebarAsk::Pin => {
                dock.toggle_pin(theme, w);
            }
            SidebarAsk::Expand => {
                let drawer = dock.mode(theme, w) == DockMode::Drawer;
                dock.toggle_expanded(theme, w);
                let open = dock.expanded(theme, w);
                let (menu, close) = (dock.menu_id(), dock.close_id());
                if drawer && self.sidebar_focus.is_some() {
                    // the keyboard follows the drawer: into it on its
                    // close, back to the menu button when it shuts
                    let to = if open { close } else { menu };
                    if let Some(to) = to {
                        self.focus_sidebar(&to);
                    }
                } else if !open {
                    self.blur_sidebar();
                }
            }
            SidebarAsk::Group(r) => {
                let id = self
                    .nav
                    .menus()
                    .nth(r)
                    .and_then(|(m, _)| m.sem_id().cloned());
                if let Some(id) = id {
                    self.nav.sidebar.activate(&id);
                }
            }
            SidebarAsk::Picked => dock.picked(theme, w),
            // before the dock (`open_palette`)
            SidebarAsk::Search => {}
        }
        // a focused part that is gone (its dropdown closed) lets it go
        if let Some(at) = self.sidebar_focus.clone()
            && self.bar().is_none_or(|bar| {
                // the menu button stands outside the panel
                let p = part(bar, &at);
                p.is_none() || (p != Some(Part::Menu) && !bar.dock.expanded(bar.theme, bar.w))
            })
        {
            self.blur_sidebar();
        }
    }

    /// The person's ask of a part of the sidebar (the pin clicked):
    /// observed on the shell as it is now, through the dispatcher.
    pub(in crate::views) fn ask_sidebar(&mut self, id: &SemId, action: &str) -> bool {
        let Ok(seen) = self.shell_semantics() else {
            return false;
        };
        match accord::Request::observe(&seen, accord::Actor::Human, id, action, None) {
            Ok(req) => self.act_shell(req).is_some(),
            Err(e) => {
                log::debug!("sidebar: {id} {action}: {e}");
                false
            }
        }
    }
}

#[cfg(test)]
mod tests;
