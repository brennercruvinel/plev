//! Showcase view: the app chrome (sidebar tree by breakpoint, header)
//! around one page at a time. This file holds the view's state and
//! layout; `input.rs` routes events, `render.rs` builds the frame,
//! `pages.rs` is the page table (the sidebar tree), `panes.rs` holds the
//! pages' state, `dispatch.rs` routes layout, events, ticks and drawing
//! to the active pane, `draw.rs` has the helpers the pages share. The
//! pages themselves live in `foundations/`, `components/` and `apps/`,
//! one module per pane.

mod apps;
mod components;
mod foundations;
mod pages;

#[cfg(test)]
mod tests;
pub mod compositor;
mod dispatcher;
pub mod text_input;

pub use apps::git::{GitWake, NavKey};
pub use text_input::ime::ImeEvent;
pub use text_input::input::EditKey;
pub use pages::{Body, Group, PAGES, Page, Pane};

pub(crate) use compositor::draw::{group_label, panel, text, with_alpha};

use dispatcher::keyboard::Walk;

use winit::keyboard::{Key, ModifiersState};

use engine::widgets::nav::{NavGroup, NavLink, PanelHeader, Sidebar};
use engine::widgets::overlay::{OverlayId, OverlayManager};
use engine::widgets::prelude::{AppShell, ContextMenu, Modal, Rect, ToastManager};
use engine::drawing::compositor::LayerId;
use engine::control::input::scroll::ScrollState;
use engine::styling::theme::Theme;

/// Digit hotkeys `1`..`9` and `0` (as ten) reach the first pages of the
/// table; the rest are reachable from the sidebar.
const HOTKEY_PAGES: usize = 10;

// ---------------------------------------------------------------------------
// Active overlay bookkeeping
// ---------------------------------------------------------------------------

enum ActiveOverlay {
    Modal {
        id: OverlayId,
        widget: Box<Modal>,
    },
    Menu {
        id: OverlayId,
        widget: ContextMenu,
        x: f32,
        y: f32,
    },
}

impl ActiveOverlay {
    fn id(&self) -> OverlayId {
        match self {
            ActiveOverlay::Modal { id, .. } | ActiveOverlay::Menu { id, .. } => *id,
        }
    }
}

// ---------------------------------------------------------------------------
// ShowcaseView
// ---------------------------------------------------------------------------

pub struct ShowcaseView {
    pub width: f32,
    pub height: f32,
    pub scale_factor: f32,
    pub theme: Theme,
    pub theme_name: String,
    /// Index into [`PAGES`].
    pub page: usize,
    /// The app chrome: sidebar by breakpoint (full, rail, drawer), the
    /// page header, and the content rect the page lays out in.
    pub shell: AppShell,
    /// Page-level vertical scroll, one per page: HOFF pages are taller
    /// than the window (Card, Theme, …); without this the wheel is dead
    /// everywhere outside the virtualized list.
    page_scroll: Vec<ScrollState>,
    /// A page offset requested before the first layout (launch argument
    /// for snapshots); applied once the content height is known.
    pending_scroll: Option<f32>,

    pub overlay_mgr: OverlayManager,
    layers: Option<Layers>,

    pub toasts: ToastManager,
    active_overlay: Option<ActiveOverlay>,
    /// The button of the modal or the item of the menu a screen reader
    /// moved the focus to (`dispatcher::overlay`); `None`, the overlay's
    /// own (the modal's cancel, the menu's first item).
    overlay_focus: Option<accord::SemId>,
    /// The shift key is held: Tab walks backward (`dispatcher::keyboard`).
    shift: bool,

    /// One struct per pane, the pages' state.
    panes: pages::panes::Panes,
    /// The clipboard of the text fields, made on first use
    /// (`clipboard.rs`).
    clipboard: Option<Box<dyn engine::runtime::platform::clipboard::ClipboardProvider>>,
}

#[derive(Clone, Copy)]
struct Layers {
    list: LayerId,
    overlay: LayerId,
    toast: LayerId,
}

impl ShowcaseView {
    pub fn new(width: f32, height: f32) -> Self {
        let theme = Theme::hoff_core();
        let links = PAGES
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let link = NavLink::new(p.title).icon(p.icon);
                if i < HOTKEY_PAGES {
                    link.hint(format!("{}", (i + 1) % 10))
                } else {
                    link
                }
            })
            .collect();
        let groups = Group::ALL
            .iter()
            .map(|g| NavGroup::new(g.label(), g.pages()))
            .collect();
        let sidebar = Sidebar::new(links)
            .groups(groups)
            .brand("plev", "HOFF DESIGN SYSTEM")
            .footer(vec![
                "T  cycle core / hoff / dark / light".into(),
                "Esc  close overlays".into(),
            ]);
        let first = &PAGES[0];
        let header = PanelHeader::new(first.title).blurb(first.blurb);
        Self {
            width,
            height,
            scale_factor: 1.0,
            shell: AppShell::new(sidebar, header, width, height),
            panes: pages::panes::Panes::new(&theme),
            theme,
            theme_name: "core".to_string(),
            page: 0,
            page_scroll: (0..PAGES.len()).map(|_| ScrollState::new()).collect(),
            pending_scroll: None,
            overlay_mgr: OverlayManager::new(),
            layers: None,
            toasts: match dispatcher::overlay::id(dispatcher::overlay::TOASTS) {
                Some(id) => ToastManager::new().sem(id),
                None => ToastManager::new(),
            },
            active_overlay: None,
            overlay_focus: None,
            shift: false,
            clipboard: None,
        }
    }

    /// Start the Git tab's worker against the cwd repository. `wake` is
    /// how the worker asks the host for a frame.
    pub fn connect_git(&mut self, wake: GitWake) {
        self.panes.git.connect(wake);
    }

    /// Modifier state from the platform shell (alt/shift clicks in the
    /// editor, Shift-Tab on a component page).
    pub fn set_modifiers(&mut self, modifiers: ModifiersState) {
        self.shift = modifiers.shift_key();
        self.panes.editor.set_modifiers(modifiers);
    }

    /// The Editor page takes the raw key first (it wants the winit key
    /// and modifiers). `false` when the editor is not the active page or
    /// did not consume the key.
    pub fn handle_editor_key(&mut self, key: &Key, modifiers: ModifiersState) -> bool {
        self.pane() == Some(Pane::Editor) && self.panes.editor.handle_key(key, modifiers)
    }

    /// Arrow up/down: the Git tab's file list; on a component page, or
    /// over the modal or the menu, the widget the focus is in
    /// (`dispatcher::keyboard`).
    pub fn handle_nav(&mut self, key: NavKey) -> bool {
        if self.walks() {
            let walk = match key {
                NavKey::Up => Walk::Up,
                NavKey::Down => Walk::Down,
            };
            return self.walk(walk);
        }
        self.pane() == Some(Pane::Git) && self.panes.git.handle_nav(key)
    }

    pub fn resize(&mut self, width: f32, height: f32, scale_factor: f32) {
        self.width = width;
        self.height = height;
        self.scale_factor = scale_factor;
        self.shell.resize(width, height);
    }

    /// The active page.
    pub fn current(&self) -> &'static Page {
        &PAGES[self.page]
    }

    /// The active page's title (what the launch argument names).
    pub fn page_title(&self) -> &'static str {
        self.current().title
    }

    /// The pane behind the active page (none on a placeholder).
    fn pane(&self) -> Option<Pane> {
        self.current().pane()
    }

    /// Switch the active page and keep the chrome in step (sidebar
    /// link, header title and blurb).
    fn set_page(&mut self, index: usize) {
        let index = index.min(PAGES.len() - 1);
        self.page = index;
        let page = &PAGES[index];
        self.shell.sidebar.set_active(index);
        self.shell.header = PanelHeader::new(page.title).blurb(page.blurb);
    }

    /// Content area the shell hands the page: past the sidebar, under
    /// the header, inside the gutter, never narrower than a card.
    fn content_rect(&self) -> Rect {
        let mut r = self.shell.layout(&self.theme).content;
        r.w = r.w.max(self.theme.size.card_min_w);
        r.h = r.h.max(self.theme.size.field_min_w);
        r
    }

    /// The page column's left edge: the sidebar's right edge (zero when
    /// the sidebar is a drawer).
    fn page_x(&self) -> f32 {
        self.shell
            .layout(&self.theme)
            .sidebar
            .map_or(0.0, |r| r.x + r.w)
    }

    /// The rect an embedded app (Editor, Git) fills: from the sidebar's
    /// edge to the window's, from under the header to the bottom. No
    /// gutter: these pages are apps, not pages.
    fn app_rect(&self) -> Rect {
        let viewport = self.content_rect();
        let x = self.page_x();
        Rect::new(x, viewport.y, self.width - x, self.height - viewport.y)
    }

    /// True for the pages that are apps filling `app_rect`.
    fn is_app(&self) -> bool {
        self.pane().is_some_and(Pane::is_app)
    }

    /// Height of the note an alias page shows above the shared example
    /// (zero for the other pages).
    fn note_h(&self) -> f32 {
        match self.current().body {
            Body::Alias { .. } => {
                self.theme.typography.caption_r().line_height + self.theme.spacing.xl
            }
            _ => 0.0,
        }
    }

    /// Current page scroll offset of the active page.
    pub fn page_offset(&self) -> f32 {
        self.page_scroll[self.page].offset()
    }

    /// Content rect shifted up by the page scroll offset and down by
    /// the alias note. Pages lay out, hit-test and render against this
    /// rect so events and pixels always agree; the render clips it back
    /// to the content viewport.
    fn page_rect(&self) -> Rect {
        if self.is_app() {
            return self.app_rect();
        }
        let mut rect = self.content_rect();
        rect.y -= self.page_offset();
        rect.y += self.note_h();
        rect.h -= self.note_h();
        rect
    }

    /// Sync the active page's scroll limits with the viewport.
    fn sync_page_scroll(&mut self) {
        let content = self.content_rect();
        let height = self.page_content_height(content);
        let scroll = &mut self.page_scroll[self.page];
        scroll.set_viewport(content.h);
        scroll.set_content(height);
    }

    fn set_theme(&mut self, name: &str) {
        if let Some(theme) = foundations::theme::resolve(name) {
            self.panes.editor.set_theme(&theme);
            self.panes.git.set_theme(name);
            self.theme = theme;
            self.theme_name = name.to_string();
        }
    }

    /// Apply a theme by name (launch argument / snapshot tooling).
    pub fn apply_theme(&mut self, name: &str) {
        self.set_theme(name);
    }

    /// Scroll the active page to `offset` px once it has laid out
    /// (launch argument / snapshot tooling).
    pub fn scroll_page_to(&mut self, offset: f32) {
        self.pending_scroll = Some(offset);
    }

    /// Jump to a page by title (launch argument / snapshot tooling).
    pub fn jump_to_page(&mut self, name: &str) {
        if let Some(index) = pages::find(name) {
            self.set_page(index);
        }
    }
}
