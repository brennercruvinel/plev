//! App shell: the chrome every app rebuilds. A [`Sidebar`] that follows
//! the breakpoint (full, rail, or a drawer behind a menu button), a
//! [`PanelHeader`] for the current screen, and the content rect the
//! screen lays out in, with the page gutter and the platform safe area
//! applied. The shell owns nothing about what a screen shows: it hands
//! the screen its rect and routes the events the chrome did not take.

use engine::compositor::{Compositor, LayerId};
use engine::theme::{Breakpoint, SidebarMode, Theme};

use crate::action::{ButtonVariant, IconButton};
use crate::core::{EventResult, Rect, WidgetEvent};
use crate::nav::{PanelHeader, Sidebar};

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
    /// Where the screen lays out: under the header, past the sidebar,
    /// inside the gutter and the safe area.
    pub content: Rect,
}

#[derive(Debug)]
pub struct AppShell {
    pub sidebar: Sidebar,
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
            header,
            safe_area: SafeArea::default(),
            menu_button: IconButton::new("layout-grid").variant(ButtonVariant::Ghost),
            width,
            height,
        }
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
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
        let mode = self.sidebar.mode(theme, self.width);
        let gutter = theme.layout.gutter(bp);
        let sidebar_w = self.sidebar.page_width(theme, self.width);
        let sidebar = (sidebar_w > 0.0).then(|| Rect::new(sa.left, 0.0, sidebar_w, self.height));
        let page_x = sa.left + sidebar_w;
        let page_w = (self.width - page_x - sa.right).max(0.0);

        let header_h = self.header.height(theme);
        let (menu_button, header_x) = if mode == SidebarMode::Drawer {
            let (bw, bh) = self.menu_button.preferred_size(theme);
            let r = Rect::new(
                page_x + gutter,
                sa.top + gutter + (header_h - bh) / 2.0,
                bw,
                bh,
            );
            (Some(r), r.x + r.w)
        } else {
            (None, page_x + gutter - PanelHeader::pad(theme))
        };
        let header = Rect::new(
            header_x,
            sa.top + gutter,
            (page_x + page_w - gutter - header_x).max(0.0),
            header_h,
        );
        let content_y = header.y + header.h + theme.spacing.lg;
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
            content,
        }
    }

    /// Route an event through the chrome. Returns the result and the
    /// sidebar link that was activated, if any; when the result is not
    /// handled the app passes the event on to its screen.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        theme: &Theme,
    ) -> (EventResult, Option<usize>) {
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
        self.header.render_to_layer(c, layer, layout.header, theme);
        if let Some(mb) = layout.menu_button {
            self.menu_button.render(c, mb, theme);
        }
        self.sidebar
            .render(c, sidebar_layer, theme, self.width, self.height);
    }
}
