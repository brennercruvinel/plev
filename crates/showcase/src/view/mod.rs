//! Showcase view: the app chrome (sidebar by breakpoint, header) around
//! one module per gallery section. This file holds the view's state and
//! layout; `input.rs` routes events, `render.rs` builds the frame,
//! `section.rs` names the sections, `draw.rs` has the helpers the
//! sections share.

mod app;
mod builder_tour;
mod buttons;
mod cards;
mod charts;
mod chrome;
mod dock;
mod draw;
mod editor;
mod effects;
mod extras;
mod forms;
mod git;
mod icons_gallery;
mod input;
mod lists;
mod overlays;
mod render;
mod section;
mod theme_gallery;
mod typography;

#[cfg(test)]
mod tests;

pub use forms::EditKey;
pub use git::{NavKey, Wake};
pub use section::Section;

pub(crate) use draw::{group_label, panel, text, with_alpha};

use winit::keyboard::{Key, ModifiersState};

use comps::nav::{NavLink, PanelHeader, Sidebar};
use comps::overlay::{OverlayId, OverlayManager};
use comps::prelude::{AppShell, ContextMenu, Modal, Rect, ToastManager};
use engine::compositor::LayerId;
use engine::input::scroll::ScrollState;
use engine::theme::Theme;

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
    pub section: Section,
    /// The app chrome: sidebar by breakpoint (full, rail, drawer), the
    /// section header, and the content rect the section lays out in.
    pub shell: AppShell,
    /// Page-level vertical scroll, one per section: HOFF pages are taller
    /// than the window (Cards, Theme, …); without this the wheel is dead
    /// everywhere outside the virtualized list.
    page_scroll: [ScrollState; Section::ALL.len()],
    /// A page offset requested before the first layout (launch argument
    /// for snapshots); applied once the content height is known.
    pending_scroll: Option<f32>,

    pub overlay_mgr: OverlayManager,
    layers: Option<Layers>,

    pub toasts: ToastManager,
    active_overlay: Option<ActiveOverlay>,

    cards: cards::CardsSection,
    buttons: buttons::ButtonsSection,
    forms: forms::FormsSection,
    overlays: overlays::OverlaysSection,
    lists: lists::ListsSection,
    icons_gallery: icons_gallery::IconsSection,
    themes: theme_gallery::ThemeSection,
    charts: charts::ChartsSection,
    dock: dock::DockSection,
    app: app::AppSection,
    builder_tour: builder_tour::BuilderSection,
    extras: extras::ExtrasSection,
    typography: typography::TypographySection,
    effects: effects::EffectsSection,
    chrome: chrome::ChromeSection,
    editor: editor::EditorSection,
    git: git::GitSection,
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
        let links = Section::ALL
            .iter()
            .enumerate()
            .map(|(i, s)| {
                NavLink::new(s.title())
                    .icon(s.icon())
                    .hint(format!("{}", i + 1))
            })
            .collect();
        let sidebar = Sidebar::new(links)
            .brand("plev", "HOFF DESIGN SYSTEM")
            .footer(vec![
                "T  cycle core / hoff / dark / light".into(),
                "Esc  close overlays".into(),
            ]);
        let header = PanelHeader::new(Section::Cards.title()).blurb(Section::Cards.blurb());
        Self {
            width,
            height,
            scale_factor: 1.0,
            shell: AppShell::new(sidebar, header, width, height),
            cards: cards::CardsSection::new(),
            forms: forms::FormsSection::new(&theme),
            buttons: buttons::ButtonsSection::new(),
            overlays: overlays::OverlaysSection::new(),
            lists: lists::ListsSection::new(),
            icons_gallery: icons_gallery::IconsSection::new(),
            themes: theme_gallery::ThemeSection::new(),
            charts: charts::ChartsSection::new(),
            dock: dock::DockSection::new(),
            app: app::AppSection::new(),
            builder_tour: builder_tour::BuilderSection::new(),
            extras: extras::ExtrasSection::new(&theme),
            typography: typography::TypographySection::new(),
            effects: effects::EffectsSection::new(),
            chrome: chrome::ChromeSection::new(&theme),
            editor: editor::EditorSection::new(&theme),
            git: git::GitSection::new(),
            theme,
            theme_name: "core".to_string(),
            section: Section::Cards,
            page_scroll: std::array::from_fn(|_| ScrollState::new()),
            pending_scroll: None,
            overlay_mgr: OverlayManager::new(),
            layers: None,
            toasts: ToastManager::new(),
            active_overlay: None,
        }
    }

    /// Start the Git tab's worker against the cwd repository. `wake` is
    /// how the worker asks the host for a frame.
    pub fn connect_git(&mut self, wake: Wake) {
        self.git.connect(wake);
    }

    /// Modifier state from the platform shell (alt/shift clicks in the
    /// editor).
    pub fn set_modifiers(&mut self, modifiers: ModifiersState) {
        self.editor.set_modifiers(modifiers);
    }

    /// The Editor section takes the raw key first (it wants the winit key
    /// and modifiers). `false` when the editor is not the active section
    /// or did not consume the key.
    pub fn handle_editor_key(&mut self, key: &Key, modifiers: ModifiersState) -> bool {
        self.section == Section::Editor && self.editor.handle_key(key, modifiers)
    }

    /// Arrow up/down: the Git tab's file list.
    pub fn handle_nav(&mut self, key: NavKey) -> bool {
        self.section == Section::Git && self.git.handle_nav(key)
    }

    pub fn resize(&mut self, width: f32, height: f32, scale_factor: f32) {
        self.width = width;
        self.height = height;
        self.scale_factor = scale_factor;
        self.shell.resize(width, height);
    }

    /// Switch the active section and keep the chrome in step (sidebar
    /// link, header title and blurb).
    fn set_section(&mut self, section: Section) {
        self.section = section;
        let idx = self.section_idx();
        self.shell.sidebar.set_active(idx);
        self.shell.header = PanelHeader::new(section.title()).blurb(section.blurb());
    }

    /// Content area the shell hands the section: past the sidebar, under
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
    /// gutter: these sections are apps, not pages.
    fn app_rect(&self) -> Rect {
        let viewport = self.content_rect();
        let x = self.page_x();
        Rect::new(x, viewport.y, self.width - x, self.height - viewport.y)
    }

    /// True for the sections that are apps filling `app_rect`.
    fn is_app(&self) -> bool {
        matches!(self.section, Section::Editor | Section::Git)
    }

    fn section_idx(&self) -> usize {
        Section::ALL
            .iter()
            .position(|s| *s == self.section)
            .unwrap_or(0)
    }

    /// Current page scroll offset of the active section.
    pub fn page_offset(&self) -> f32 {
        self.page_scroll[self.section_idx()].offset()
    }

    /// Content rect shifted up by the page scroll offset. Sections lay out,
    /// hit-test and render against this rect so events and pixels always
    /// agree; the render clips it back to the content viewport.
    fn page_rect(&self) -> Rect {
        if self.is_app() {
            return self.app_rect();
        }
        let mut rect = self.content_rect();
        rect.y -= self.page_offset();
        rect
    }

    /// Natural (unclipped) height of the active section's content.
    fn section_content_height(&self, content: Rect) -> f32 {
        match self.section {
            Section::Cards => self.cards.content_height(content, &self.theme),
            Section::Buttons => self.buttons.content_height(content, &self.theme),
            Section::Forms => self.forms.content_height(content, &self.theme),
            Section::Overlays => self.overlays.content_height(content, &self.theme),
            // The lists page sizes itself to the viewport; the virtual list
            // and tree scroll internally.
            Section::Lists => content.h,
            Section::Icons => self.icons_gallery.content_height(content),
            Section::Theme => self.themes.content_height(content, &self.theme),
            Section::Charts => self.charts.content_height(content),
            Section::Dock => self.dock.content_height(content),
            Section::App => self.app.content_height(content),
            Section::Builder => self.builder_tour.content_height(content),
            Section::Extras => self.extras.content_height(content, &self.theme),
            Section::Typography => self.typography.content_height(content),
            Section::Effects => self.effects.content_height(content, &self.theme),
            Section::Chrome => self.chrome.content_height(content, &self.theme),
            Section::Editor => self.editor.content_height(content),
            Section::Git => self.git.content_height(content),
        }
    }

    /// Sync the active section's page scroll limits with the viewport.
    fn sync_page_scroll(&mut self) {
        let content = self.content_rect();
        let height = self.section_content_height(content);
        let scroll = &mut self.page_scroll[self.section_idx()];
        scroll.set_viewport(content.h);
        scroll.set_content(height);
    }

    fn set_theme(&mut self, name: &str) {
        if let Some(theme) = theme_gallery::resolve(name) {
            self.editor.set_theme(&theme);
            self.git.set_theme(name);
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

    /// Jump to a section by name (launch argument / snapshot tooling).
    pub fn jump_to_section(&mut self, name: &str) {
        if let Some(section) = Section::ALL
            .iter()
            .find(|s| s.title().eq_ignore_ascii_case(name))
        {
            self.set_section(*section);
        }
    }
}
