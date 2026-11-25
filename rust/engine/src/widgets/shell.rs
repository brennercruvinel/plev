//! App shell: the chrome every app rebuilds. A [`Sidebar`] that follows
//! the breakpoint (full, rail, or a drawer behind a menu button), or,
//! docked ([`AppShell::docked`]), the app's own panel where a
//! [`SidebarDock`] puts it (peeking, pinned or a drawer), a
//! [`PanelHeader`] for the current screen, as tall as its blurb wraps in
//! the width it gets, and the content rect the
//! screen lays out in, with the page gutter and the platform safe area
//! applied. The shell owns nothing about what a screen shows: it hands
//! the screen its rect and routes the events the chrome did not take.
//! Docked, it owns nothing of the panel either: it hands the app the
//! panel's rect and tells it which events are the panel's
//! ([`AppShell::route`]).

use crate::drawing::compositor::{Compositor, LayerId};
use crate::styling::theme::{Breakpoint, SidebarMode, Theme};

use crate::widgets::buttons::{IconButton, IconButtonVariant};
use crate::widgets::nav::{DockLayout, DockMode, PanelHeader, Sidebar, SidebarDock};
use crate::widgets::wake::Wake;
use crate::widgets::widget::{EventResult, Rect, WidgetEvent};

/// Insets the platform reserves (notch, home indicator, status bar), in
/// logical px. Zero on a desktop window.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SafeArea {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

/// One frame of shell geometry.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShellLayout {
    pub breakpoint: Breakpoint,
    pub sidebar_mode: SidebarMode,
    /// The column the sidebar takes from the page (none for a drawer).
    pub sidebar: Option<Rect>,
    /// The menu button that opens the drawer (compact only).
    pub menu_button: Option<Rect>,
    pub header: Rect,
    /// The header's padding left and right.
    pub header_pads: [f32; 2],
    /// Where the screen lays out: under the header, past the sidebar,
    /// inside the gutter and the safe area.
    pub content: Rect,
    /// A docked shell's panel: its mode, where it is now, the zone that
    /// reveals it. `None` for the breakpoint's sidebar.
    pub dock: Option<DockLayout>,
}

/// Where a docked shell sends an event ([`AppShell::route`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Route {
    /// The chrome took it: the drawer's menu button, a press off the open
    /// drawer, any event while the drawer holds the window.
    Shell(EventResult),
    /// It is on the panel: the app's sidebar handles it.
    Panel,
    /// The page's.
    Page,
}

#[derive(Debug)]
pub struct AppShell {
    pub sidebar: Sidebar,
    /// Docked: the panel's place and its modes, in place of `sidebar`'s
    /// breakpoint (which then draws nothing).
    pub dock: Option<SidebarDock>,
    pub header: PanelHeader,
    pub safe_area: SafeArea,
    menu_button: IconButton,
    width: f32,
    height: f32,
}

impl AppShell {
    pub fn new(sidebar: Sidebar, header: PanelHeader, width: f32, height: f32) -> Self {
        Self {
            sidebar,
            dock: None,
            header,
            safe_area: SafeArea::default(),
            menu_button: IconButton::new("layout-grid").variant(IconButtonVariant::Ghost),
            width,
            height,
        }
    }

    /// A shell whose sidebar is the app's own panel, placed by `dock`:
    /// peeking by default, pinned beside the page, or a drawer behind a
    /// menu button where the window cannot hold both. The app draws its
    /// panel in [`ShellLayout::dock`]'s `panel` and gives it the events
    /// [`AppShell::route`] says are the panel's.
    pub fn docked(dock: SidebarDock, header: PanelHeader, width: f32, height: f32) -> Self {
        // the menu button is the dock's (`SidebarDock::render_menu`)
        let mut shell = Self::new(Sidebar::new(Vec::new()), header, width, height);
        shell.dock = Some(dock);
        shell
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
    }

    /// The window the dock places its panel in: the shell's, inside the
    /// safe area's sides. Its width is what the dock's asks take.
    pub fn dock_window(&self) -> Rect {
        let sa = self.safe_area;
        let w = (self.width - sa.left - sa.right).max(0.0);
        Rect::new(sa.left, 0.0, w, self.height)
    }

    /// A docked shell's clock: the dock's pauses and slide (idle when not
    /// docked).
    pub fn wake(&mut self, dt: f32, theme: &Theme) -> Wake {
        let w = self.dock_window().w;
        match self.dock.as_mut() {
            Some(dock) => dock.wake(dt, theme, w),
            None => Wake::idle(),
        }
    }

    pub fn size(&self) -> (f32, f32) {
        (self.width, self.height)
    }

    pub fn set_safe_area(&mut self, safe_area: SafeArea) {
        self.safe_area = safe_area;
    }

    pub fn layout(&self, theme: &Theme) -> ShellLayout {
        let sa = self.safe_area;
        let bp = theme.layout.breakpoint(self.width);
        // `drawer`: the menu button is wanted
        let (mode, sidebar, page_x, drawer, dock) = match &self.dock {
            Some(d) => {
                let l = d.layout(theme, self.dock_window());
                let pinned = l.mode == DockMode::Pinned;
                let mode = if pinned {
                    SidebarMode::Full
                } else {
                    SidebarMode::Drawer
                };
                let sidebar = pinned.then_some(l.rest);
                (mode, sidebar, l.page_x, l.menu, Some(l))
            }
            None => {
                let mode = self.sidebar.mode(theme, self.width);
                let sidebar_w = self.sidebar.page_width(theme, self.width);
                let sidebar =
                    (sidebar_w > 0.0).then(|| Rect::new(sa.left, 0.0, sidebar_w, self.height));
                let drawer = mode == SidebarMode::Drawer;
                (mode, sidebar, sa.left + sidebar_w, drawer, None)
            }
        };
        let page_w = (self.width - page_x - sa.right).max(0.0);
        // The gutter is the page's own: it grows with the width the page
        // gets, not with the window's class.
        let gutter = theme.layout.gutter_at(page_w);

        let (bw, bh) = match &self.dock {
            Some(_) => {
                let b = SidebarDock::menu_size(theme);
                (b, b)
            }
            None => self.menu_button.preferred_size(theme),
        };
        let top = sa.top + gutter;
        // A head that bleeds (the kit's Layout title) spans the page column
        // from the top, its title on the gutter or past the menu button;
        // any other sits inside the gutter, its padding outside the page's
        // edge. Either way its blurb wraps in the width it gets, and the
        // page starts under its last line.
        let (header, pads) = if self.header.bleeds(theme) {
            let left = if drawer {
                gutter + bw + theme.spacing.md
            } else {
                gutter
            };
            let pads = [left, gutter];
            let h = self.header.height_in(page_w, pads, theme);
            let header = Rect::new(page_x, sa.top, page_w, h);
            (header, pads)
        } else {
            let header_x = if drawer {
                page_x + gutter + bw
            } else {
                page_x + gutter - PanelHeader::pad(theme)
            };
            let w = (page_x + page_w - gutter - header_x).max(0.0);
            let pads = self.header.pads(w, theme);
            let h = self.header.height_in(w, pads, theme);
            let header = Rect::new(header_x, top, w, h);
            (header, pads)
        };
        // A head that bleeds ends on its rule and the page's first group
        // pads itself under it (the kit's Layout); a head inside the
        // gutter keeps `lg` under it.
        let under = if self.header.bleeds(theme) {
            0.0
        } else {
            theme.spacing.lg
        };
        let content_y = header.y + header.h + under;
        // The menu button centers on the title's band in a head that
        // bleeds; elsewhere on the one-line head, as it always did.
        let menu_button = drawer.then(|| {
            let (y, h) = if self.header.bleeds(theme) {
                (header.y, self.header.band(header.w, theme))
            } else {
                (top, self.header.height(theme))
            };
            Rect::new(page_x + gutter, y + (h - bh) / 2.0, bw, bh)
        });
        let content = Rect::new(
            page_x + gutter,
            content_y,
            (page_w - gutter * 2.0).max(0.0),
            (self.height - content_y - gutter - sa.bottom).max(0.0),
        );
        ShellLayout {
            breakpoint: bp,
            sidebar_mode: mode,
            sidebar,
            menu_button,
            header,
            header_pads: pads,
            content,
            dock,
        }
    }

    /// Route an event through the chrome. Returns the result and the
    /// sidebar link that was activated, if any; when the result is not
    /// handled the app passes the event on to its screen.
    /// A docked shell routes with [`AppShell::route`]; here the panel's
    /// events come back handled (the page must not see them) and the
    /// link is the app's to read.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        theme: &Theme,
    ) -> (EventResult, Option<usize>) {
        if self.dock.is_some() {
            let r = match self.route(event, theme) {
                Route::Shell(r) => r,
                Route::Panel => EventResult {
                    handled: true,
                    ..EventResult::IGNORED
                },
                Route::Page => EventResult::IGNORED,
            };
            return (r, None);
        }
        let layout = self.layout(theme);
        let (vw, vh) = (self.width, self.height);

        // An open drawer is exclusive.
        if layout.sidebar_mode == SidebarMode::Drawer && self.sidebar.drawer_open {
            let (r, nav) = self.sidebar.handle_event(event, theme, vw, vh);
            return (EventResult { handled: true, ..r }, nav);
        }
        if let Some(mb) = layout.menu_button {
            let r = self.menu_button.handle_event(event, mb);
            if r.clicked {
                self.sidebar.drawer_open = true;
                return (EventResult::clicked(), None);
            }
            if r.handled {
                return (r, None);
            }
        }
        if layout.sidebar.is_some() {
            let (r, nav) = self.sidebar.handle_event(event, theme, vw, vh);
            if nav.is_some() || r.handled {
                return (r, nav);
            }
            return (r, None);
        }
        (EventResult::IGNORED, None)
    }

    /// Where a docked shell sends a pointer event: the dock sees it first
    /// (the edge, the panel, the open drawer), then the drawer's menu
    /// button; what is left is the panel's or the page's. `Page` for
    /// every event when not docked.
    pub fn route(&mut self, event: &WidgetEvent, theme: &Theme) -> Route {
        let layout = self.layout(theme);
        let window = self.dock_window();
        let Some(dock) = self.dock.as_mut() else {
            return Route::Page;
        };
        let modal = layout.dock.is_some_and(|l| l.modal);
        let r = dock.pointer(event, theme, window);
        let (x, y) = event.pos();
        let on_panel = dock.covers(x, y, theme, window);
        if modal {
            return if on_panel && !r.clicked {
                Route::Panel
            } else {
                Route::Shell(r)
            };
        }
        if let Some(mb) = layout.menu_button {
            let m = dock.menu_event(event, mb, theme, window.w);
            if m.clicked {
                dock.open_drawer(theme, window.w);
                return Route::Shell(EventResult::clicked());
            }
            if m.handled {
                return Route::Shell(m);
            }
        }
        if on_panel { Route::Panel } else { Route::Page }
    }

    /// Draw the chrome. The sidebar goes on `sidebar_layer` (a drawer
    /// must sit above the content, so apps pass their overlay layer
    /// there); the header and menu button on `layer`.
    pub fn render(
        &mut self,
        c: &mut Compositor,
        layer: LayerId,
        sidebar_layer: LayerId,
        theme: &Theme,
    ) {
        let layout = self.layout(theme);
        self.header
            .render_in(c, layer, layout.header, layout.header_pads, theme);
        if let Some(mb) = layout.menu_button {
            match &self.dock {
                Some(dock) => dock.render_menu(c, layer, mb, theme, self.dock_window().w),
                None => self.menu_button.render(c, mb, theme),
            }
        }
        // docked, the panel is the app's to draw
        if self.dock.is_none() {
            self.sidebar
                .render(c, sidebar_layer, theme, self.width, self.height);
        }
    }
}
