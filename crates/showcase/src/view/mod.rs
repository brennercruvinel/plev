//! Showcase view: HOFF sidebar navigation + one module per gallery
//! section, framed like the social app (graphite glass over #444444).

mod app;
mod builder_tour;
mod buttons;
mod cards;
mod charts;
mod chrome;
mod dock;
mod effects;
mod extras;
mod forms;
mod icons_gallery;
mod lists;
mod overlays;
mod theme_gallery;
mod typography;

pub use forms::EditKey;

use comps::nav::{NavLink, PanelHeader, Sidebar};
use comps::overlay::{OverlayId, OverlayKind, OverlayManager};
use comps::prelude::{
    AppShell, ContextMenu, EventResult, Modal, ModalAction, Rect, ToastManager, WidgetEvent,
    rounded_rect, rounded_rect_stroke,
};
use engine::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use engine::input::scroll::ScrollState;
use engine::theme::{Intent, Theme};

// ---------------------------------------------------------------------------
// Sections
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Section {
    Cards,
    Buttons,
    Forms,
    Overlays,
    Lists,
    Icons,
    Theme,
    Charts,
    Dock,
    App,
    Builder,
    Extras,
    Typography,
    Effects,
    Chrome,
}

impl Section {
    pub const ALL: [Section; 15] = [
        Section::Cards,
        Section::Buttons,
        Section::Forms,
        Section::Overlays,
        Section::Lists,
        Section::Icons,
        Section::Theme,
        Section::Charts,
        Section::Dock,
        Section::App,
        Section::Builder,
        Section::Extras,
        Section::Typography,
        Section::Effects,
        Section::Chrome,
    ];

    fn title(self) -> &'static str {
        match self {
            Section::Cards => "Cards",
            Section::Buttons => "Buttons",
            Section::Forms => "Forms",
            Section::Overlays => "Overlays",
            Section::Lists => "Lists",
            Section::Icons => "Icons",
            Section::Theme => "Theme",
            Section::Charts => "Charts",
            Section::Dock => "Dock",
            Section::App => "App",
            Section::Builder => "Builder",
            Section::Extras => "Extras",
            Section::Typography => "Typography",
            Section::Effects => "Effects",
            Section::Chrome => "Chrome",
        }
    }

    fn icon(self) -> &'static str {
        match self {
            Section::Cards => "clipboard",
            Section::Buttons => "square",
            Section::Forms => "settings",
            Section::Overlays => "copy",
            Section::Lists => "file",
            Section::Icons => "eye",
            Section::Theme => "sun",
            Section::Charts => "git-branch",
            Section::Dock => "terminal",
            Section::App => "check",
            Section::Builder => "code",
            Section::Extras => "plus",
            Section::Typography => "info",
            Section::Effects => "moon",
            Section::Chrome => "layout-grid",
        }
    }

    fn blurb(self) -> &'static str {
        match self {
            Section::Cards => {
                "The HOFF card deck: one glass shell, six preview families with live data."
            }
            Section::Buttons => "Variants, sizes, intents and states of comps::prelude::Button.",
            Section::Forms => "Checkbox, switch, slider, progress, select and tabs.",
            Section::Overlays => {
                "Modal, context menu, tooltip and toasts — spring physics per intent."
            }
            Section::Lists => "Virtualized list with 10,000 rows, and a tree view.",
            Section::Icons => "Lucide icon set, tessellated to GPU paths with per-size caching.",
            Section::Theme => "HOFF tokens and every built-in palette. Click a card to apply it.",
            Section::Charts => {
                "Line, bars, stacked area and donut, drawn from a pure tested geometry core."
            }
            Section::Dock => "Floating message dock: glass blur, hover lift, width morph on click.",
            Section::App => "A complete small todo app: domain state, per-item animation, input.",
            Section::Builder => {
                "Builder tour with real hit regions and measured, content-driven layout."
            }
            Section::Extras => {
                "App-level widgets: chips, icon buttons, spinners, empty state, split pane."
            }
            Section::Typography => {
                "The HOFF scale set in Inclusive Sans — every weight embedded, every readout measured live."
            }
            Section::Effects => {
                "Analytic shadows, gradients, backdrop blur, clip stack and tessellated vector shapes."
            }
            Section::Chrome => {
                "The app frame: avatars, badges, panel header, nav links, breadcrumb, stats, code, skeleton, table."
            }
        }
    }
}

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
}

#[derive(Clone, Copy)]
struct Layers {
    list: LayerId,
    overlay: LayerId,
    toast: LayerId,
}

impl ShowcaseView {
    pub fn new(width: f32, height: f32) -> Self {
        let theme = Theme::hoff();
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
                "T  cycle hoff / dark / light".into(),
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
            theme,
            theme_name: "hoff".to_string(),
            section: Section::Cards,
            page_scroll: std::array::from_fn(|_| ScrollState::new()),
            overlay_mgr: OverlayManager::new(),
            layers: None,
            toasts: ToastManager::new(),
            active_overlay: None,
        }
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
            self.theme = theme;
            self.theme_name = name.to_string();
        }
    }

    /// Apply a theme by name (launch argument / snapshot tooling).
    pub fn apply_theme(&mut self, name: &str) {
        self.set_theme(name);
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

    // -- Input ---------------------------------------------------------------

    /// Returns `true` when the key was consumed (and a redraw is needed).
    pub fn handle_key(&mut self, key: &str) -> bool {
        // A focused text field owns the characters: "t" types, it does
        // not switch the theme.
        if self.section == Section::Forms && self.forms.handle_text(key) {
            return true;
        }
        if self.section == Section::App && self.app.handle_text(key) {
            return true;
        }
        match key {
            "t" | "T" => {
                let next = match self.theme_name.as_str() {
                    "hoff" => "dark",
                    "dark" => "light",
                    _ => "hoff",
                };
                self.set_theme(next);
                true
            }
            d @ ("1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9") => {
                let idx = d.as_bytes()[0] - b'1';
                self.set_section(Section::ALL[idx as usize]);
                true
            }
            // "0" reads as 10 (matching the sidebar numbering); sections
            // past the tenth are reachable via the sidebar.
            "0" if Section::ALL.len() >= 10 => {
                self.set_section(Section::ALL[9]);
                true
            }
            _ => false,
        }
    }

    /// Close the topmost overlay (Escape): forms focus blurs first, then
    /// the open select, then the overlay stack. Returns `false` when
    /// there was nothing to close (caller may quit).
    pub fn close_top_overlay(&mut self) -> bool {
        if self.section == Section::Forms && self.forms.handle_escape() {
            return true;
        }
        if self.section == Section::App && self.app.handle_escape() {
            return true;
        }
        if self.forms.select_is_open() {
            self.forms.close_select();
            return true;
        }
        self.overlay_mgr.pop_animated().is_some()
    }

    /// Non-character editing keys (Tab, Backspace, arrows) forwarded by
    /// the platform shell. Forms: Tab cycles its widgets, the rest edit
    /// the focused text field. App: the add field.
    pub fn handle_edit_key(&mut self, key: EditKey) -> bool {
        match self.section {
            Section::Forms => self.forms.handle_edit_key(key),
            Section::App => self.app.handle_edit_key(key),
            _ => false,
        }
    }

    /// Enter: the App section adds the field text as a todo.
    pub fn handle_enter(&mut self) -> bool {
        self.section == Section::App && self.app.handle_enter()
    }

    pub fn handle_right_click(&mut self, x: f32, y: f32) -> bool {
        if self.section != Section::Overlays || self.active_overlay.is_some() {
            return false;
        }
        let content = self.page_rect();
        if !self.overlays.menu_area(content, &self.theme).contains(x, y) {
            return false;
        }
        self.open_menu(x, y);
        true
    }

    fn open_menu(&mut self, x: f32, y: f32) {
        let widget = overlays::demo_menu();
        let (w, h) = widget.size(&self.theme);
        // Keep the menu inside the viewport.
        let mx = x.min(self.width - w - 8.0);
        let my = y.min(self.height - h - 8.0);
        let id = self.overlay_mgr.push_animated(
            OverlayKind::ContextMenu { items: vec![] },
            mx,
            my,
            w,
            h,
            &self.theme.intent_motion(Intent::Neutral),
        );
        self.active_overlay = Some(ActiveOverlay::Menu {
            id,
            widget,
            x: mx,
            y: my,
        });
    }

    fn open_modal(&mut self, destructive: bool) {
        let (widget, intent) = if destructive {
            (
                Modal::new(
                    "Delete repository?",
                    "This permanently removes the repository and all of its \
                     history. This action cannot be undone.",
                    "Delete",
                    "Cancel",
                )
                .intent(Intent::Destructive),
                Intent::Destructive,
            )
        } else {
            (
                Modal::new(
                    "Apply settings?",
                    "Your workspace will reload to apply the new configuration.",
                    "Apply",
                    "Cancel",
                )
                .intent(Intent::Neutral),
                Intent::Neutral,
            )
        };
        let dialog = widget.dialog_rect(&self.theme, self.width, self.height);
        let id = self.overlay_mgr.push_animated(
            OverlayKind::Modal {
                title: widget.title.clone(),
                body: widget.body.clone(),
                confirm: String::new(),
                cancel: String::new(),
            },
            dialog.x,
            dialog.y,
            dialog.w,
            dialog.h,
            &self.theme.intent_motion(intent),
        );
        self.active_overlay = Some(ActiveOverlay::Modal {
            id,
            widget: Box::new(widget),
        });
    }

    /// Route a pointer event. Returns `true` if a redraw is needed.
    pub fn handle_event(&mut self, event: &WidgetEvent) -> bool {
        let (vw, vh) = (self.width, self.height);

        // Toasts float above everything.
        let toast_result = self.toasts.handle_event(event, &self.theme, vw, vh);
        if toast_result.clicked {
            return true;
        }
        let mut result = toast_result;

        // Active overlay is exclusive while it is not fading out.
        if let Some(active) = &mut self.active_overlay {
            let id = active.id();
            let closing = !self
                .overlay_mgr
                .stack
                .iter()
                .any(|o| o.id == id && !o.is_closing());
            if !closing {
                match active {
                    ActiveOverlay::Modal { widget, .. } => {
                        let (action, r) = widget.handle_event(event, &self.theme, vw, vh);
                        match action {
                            ModalAction::Confirm => {
                                let destructive = widget.intent == Intent::Destructive;
                                self.overlay_mgr.pop_id_animated(id);
                                if destructive {
                                    self.toasts.push(
                                        "Repository deleted (not really).",
                                        Intent::Destructive,
                                        &self.theme,
                                    );
                                } else {
                                    self.toasts.push(
                                        "Settings applied.",
                                        Intent::Constructive,
                                        &self.theme,
                                    );
                                }
                            }
                            ModalAction::Cancel => self.overlay_mgr.pop_id_animated(id),
                            ModalAction::None => {}
                        }
                        return r.changed || action != ModalAction::None;
                    }
                    ActiveOverlay::Menu { widget, x, y, .. } => {
                        let (r, clicked) = widget.handle_event(event, *x, *y, &self.theme);
                        if let Some(item) = clicked {
                            let label = overlays::menu_label(item);
                            self.overlay_mgr.pop_id_animated(id);
                            self.toasts.push(
                                format!("Context menu: {label}"),
                                Intent::Informational,
                                &self.theme,
                            );
                            return true;
                        }
                        if !r.handled
                            && let WidgetEvent::MouseDown { x: px, y: py } = *event
                        {
                            let (w, h) = widget.size(&self.theme);
                            if !Rect::new(*x, *y, w, h).contains(px, py) {
                                self.overlay_mgr.pop_id_animated(id);
                                return true;
                            }
                        }
                        return r.changed;
                    }
                }
            }
        }

        // Open select dropdown gets priority over everything beneath it.
        if self.section == Section::Forms && self.forms.select_is_open() {
            let r = self
                .forms
                .route_select(event, self.page_rect(), &self.theme);
            if r.handled || r.changed {
                return r.changed;
            }
        }

        // The chrome first: sidebar navigation (a drawer is exclusive),
        // the menu button on a phone.
        let (shell_result, nav) = self.shell.handle_event(event, &self.theme);
        if let Some(i) = nav {
            self.set_section(Section::ALL[i]);
            return true;
        }
        result = result.merge(shell_result);
        if shell_result.handled {
            return result.changed;
        }

        // Clicks on the header band belong to the chrome: widgets scrolled
        // underneath it must not receive them.
        let viewport = self.content_rect();
        let page_x = self.page_x();
        if let WidgetEvent::MouseDown { x, y } = *event
            && x >= page_x
            && y < viewport.y
        {
            return result.changed;
        }

        let content = self.page_rect();
        let section_result = match self.section {
            Section::Cards => self.cards.handle_event(event, content, &self.theme),
            Section::Buttons => self.buttons.handle_event(event, content, &self.theme),
            Section::Forms => self.forms.handle_event(event, content, &self.theme),
            Section::Overlays => {
                let (r, action) = self.overlays.handle_event(event, content, &self.theme);
                match action {
                    overlays::OverlayAction::OpenModal { destructive } => {
                        self.open_modal(destructive)
                    }
                    overlays::OverlayAction::PushToast(intent) => {
                        let msg = match intent {
                            Intent::Neutral => "Neutral toast — plain information.",
                            Intent::Constructive => "Saved! Everything went fine.",
                            Intent::Destructive => "Failed to push: remote rejected.",
                            Intent::Informational => "3 new commits fetched from origin.",
                        };
                        self.toasts.push(msg, intent, &self.theme);
                    }
                    overlays::OverlayAction::None => {}
                }
                r
            }
            Section::Lists => self.lists.handle_event(event, content, &self.theme),
            Section::Icons => self.icons_gallery.handle_event(event, content),
            Section::Charts => self.charts.handle_event(event, content),
            Section::Dock => self.dock.handle_event(event, content),
            Section::App => self.app.handle_event(event, content, &self.theme),
            Section::Builder => self.builder_tour.handle_event(event, content),
            Section::Extras => self.extras.handle_event(event, content, &self.theme),
            Section::Typography => self.typography.handle_event(event, content),
            Section::Effects => self.effects.handle_event(event, content),
            Section::Chrome => self.chrome.handle_event(event, content, &self.theme),
            Section::Theme => {
                let (r, picked) = self.themes.handle_event(event, content);
                if let Some(name) = picked {
                    self.set_theme(name);
                }
                r
            }
        };
        result = result.merge(section_result);

        // Page scroll: when no widget consumed the wheel, scroll the
        // section itself (HOFF pages overflow the window). Clamped by
        // ScrollState; only an actual offset change requests a frame.
        if let WidgetEvent::Scroll { x, delta, .. } = *event
            && !result.handled
            && x >= page_x
        {
            self.sync_page_scroll();
            let idx = self.section_idx();
            let scroll = &mut self.page_scroll[idx];
            let old = scroll.offset();
            scroll.scroll_by(delta);
            if scroll.offset() != old {
                result = result.merge(EventResult::changed());
            }
        }

        result.changed
    }

    // -- Animation -------------------------------------------------------------

    /// Advance all animations. Returns `true` while anything is moving.
    pub fn tick(&mut self, dt: f32) -> bool {
        let mut animating = false;
        animating |= self.overlay_mgr.tick(dt);
        animating |= self.toasts.tick(dt);
        animating |= self.forms.tick(dt);
        animating |= self.lists.tick(dt);
        animating |= self.overlays.tick(dt);
        // Dock motion only while its section is visible: an open panel
        // blinks its caret forever, which would busy-loop other sections.
        if self.section == Section::Dock {
            animating |= self.dock.tick(dt);
        }
        // Charts reveal only while visible: replay is click-triggered, so
        // off-screen frames would be wasted work.
        if self.section == Section::Charts {
            animating |= self.charts.tick(dt);
        }
        // Spinners rotate only while the Extras section is visible.
        if self.section == Section::Extras {
            animating |= self.extras.tick(dt);
        }
        if self.section == Section::Chrome {
            animating |= self.chrome.tick(dt);
        }
        // The todo app blinks its caret only while visible.
        if self.section == Section::App {
            animating |= self.app.tick(dt);
        }

        // Drop overlay widgets whose exit animation finished.
        if let Some(active) = &self.active_overlay {
            let id = active.id();
            if !self.overlay_mgr.stack.iter().any(|o| o.id == id) {
                self.active_overlay = None;
            }
        }
        animating
    }

    // -- Rendering -------------------------------------------------------------

    fn ensure_layers(&mut self, c: &mut Compositor) -> Layers {
        *self.layers.get_or_insert_with(|| Layers {
            list: c.create_layer(10),
            overlay: c.create_layer(OverlayManager::BASE_Z),
            toast: c.create_layer(OverlayManager::BASE_Z + 200),
        })
    }

    pub fn render(&mut self, c: &mut Compositor) {
        c.begin_frame();
        let layers = self.ensure_layers(c);
        let theme = self.theme.clone();

        // The chrome: header on the default layer, the sidebar on the
        // overlay layer so a drawer sits above the content.
        self.shell
            .render(c, LayerId::DEFAULT, layers.overlay, &theme);

        // Keep the page scroll clamped to the current viewport/content
        // (resize can shrink content; the offset must follow).
        self.sync_page_scroll();
        let content = self.page_rect();
        let viewport = self.content_rect();

        // Clip the virtualized list (overscan rows) to its panel.
        let clip = if self.section == Section::Lists {
            let b = self.lists.list_bounds(content);
            let sf = self.scale_factor;
            Some((
                (b.x * sf) as u32,
                (b.y * sf) as u32,
                (b.w * sf).ceil() as u32,
                (b.h * sf).ceil() as u32,
            ))
        } else {
            None
        };
        c.set_layer_clip_rect(layers.list, clip);

        // Scrolled section content clips to the viewport below the header
        // (PushClip rects are logical; the encoder scales them to physical).
        let page_x = self.page_x();
        c.push(SceneNode::PushClip {
            x: page_x,
            y: viewport.y,
            w: self.width - page_x,
            h: viewport.h,
        });
        match self.section {
            Section::Cards => self.cards.render(c, content, &theme),
            Section::Buttons => self.buttons.render(c, content, &theme),
            Section::Forms => self.forms.render(c, layers.overlay, content, &theme),
            Section::Overlays => {
                self.overlays.render(c, content, &theme);
                self.overlays
                    .render_tooltip(c, layers.toast, &theme, self.width, self.height);
            }
            Section::Lists => self.lists.render(c, layers.list, content, &theme),
            Section::Icons => self.icons_gallery.render(c, content, &theme),
            Section::Theme => self.themes.render(c, content, &theme, &self.theme_name),
            Section::Charts => self.charts.render(c, content, &theme),
            Section::Dock => self.dock.render(c, content, &theme),
            Section::App => self.app.render(c, content, &theme),
            Section::Builder => self.builder_tour.render(c, content, &theme),
            Section::Extras => self.extras.render(c, content, &theme),
            Section::Typography => self.typography.render(c, content, &theme),
            Section::Effects => self.effects.render(c, content, &theme),
            Section::Chrome => self.chrome.render(c, content, &theme),
        }
        c.push(SceneNode::PopClip);

        // Active overlay: fade via layer opacity driven by the manager.
        let mut overlay_opacity = 1.0;
        if let Some(active) = &self.active_overlay {
            let id = active.id();
            if let Some(overlay) = self.overlay_mgr.stack.iter().find(|o| o.id == id) {
                overlay_opacity = overlay.opacity();
            }
            match active {
                ActiveOverlay::Modal { widget, .. } => {
                    widget.render(c, layers.overlay, &theme, self.width, self.height);
                }
                ActiveOverlay::Menu { widget, x, y, .. } => {
                    widget.render(c, layers.overlay, &theme, *x, *y);
                }
            }
        }
        c.set_layer_opacity(layers.overlay, overlay_opacity);

        self.toasts
            .render(c, layers.toast, &theme, self.width, self.height);
    }
}

// ---------------------------------------------------------------------------
// Shared drawing helpers for the section modules
// ---------------------------------------------------------------------------

/// RGBA with overridden alpha.
pub(crate) fn with_alpha(c: [f32; 4], a: f32) -> [f32; 4] {
    [c[0], c[1], c[2], a]
}

/// Push a single-line text node to the default layer.
pub(crate) fn text(
    c: &mut Compositor,
    s: &str,
    size: f32,
    weight: u16,
    x: f32,
    y: f32,
    color: [f32; 4],
) {
    c.push(SceneNode::Text {
        key: TextNodeKey::new(s, size, size * 1.4, None).with_weight(weight),
        x,
        y,
        color,
    });
}

/// Uppercase group label — the HOFF accordion head (12/600, ls .05em,
/// rgba($n2,.25)).
pub(crate) fn group_label(c: &mut Compositor, s: &str, x: f32, y: f32, theme: &Theme) {
    text(c, s, 12.0, 600, x, y, theme.glass.text_placeholder.0);
}

/// Soft panel container — HOFF list card: radius 20, .02 white glass,
/// soft edge. Icon paths pushed after it stack on top (push order is
/// preserved across primitive types).
pub(crate) fn panel(c: &mut Compositor, rect: Rect, theme: &Theme) {
    c.push(rounded_rect(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        theme.radius.lg,
        theme.glass.surface.0,
    ));
    c.push(rounded_rect_stroke(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        theme.radius.lg,
        theme.glass.edge_soft.0,
        1.0,
    ));
}

// ---------------------------------------------------------------------------
// Headless view tests (no GPU: scenes build into a plain compositor)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_on_cards_with_the_hoff_theme() {
        let view = ShowcaseView::new(1200.0, 800.0);
        assert_eq!(view.section, Section::Cards);
        assert_eq!(view.theme_name, "hoff");
        // Page frame is the HOFF #444444.
        assert_eq!(view.theme.colors.bg.0, engine::theme::hoff::PAGE_BG.0);
    }

    // -- Sidebar responsiveness (14 links overflow short windows) ----------

    /// The sidebar's link rects at the current viewport.
    fn link_rects(view: &ShowcaseView) -> Vec<Rect> {
        let sb = &view.shell.sidebar;
        let rect = sb.rect(&view.theme, view.width, view.height).unwrap();
        sb.link_rects(rect, &view.theme, sb.mode(&view.theme, view.width))
    }

    fn band(view: &ShowcaseView) -> Rect {
        let sb = &view.shell.sidebar;
        let rect = sb.rect(&view.theme, view.width, view.height).unwrap();
        sb.band(rect, &view.theme, sb.mode(&view.theme, view.width))
    }

    #[test]
    fn sidebar_scrolls_in_short_windows_and_stays_put_in_tall_ones() {
        // Short: the links overflow the band, the rail becomes scrollable.
        let mut view = ShowcaseView::new(1280.0, 500.0);
        let mut c = Compositor::new();
        view.render(&mut c);
        let last_before = link_rects(&view).last().unwrap().y;
        let band_bottom = band(&view).bottom();
        assert!(
            last_before + NavLink::height(&view.theme) > band_bottom,
            "500px window: the last link must start below the visible band"
        );

        // Wheel over the rail scrolls the rail, not the page.
        let (bx, by) = band(&view).center();
        let changed = view.handle_event(&WidgetEvent::Scroll {
            x: bx,
            y: by,
            delta: 200.0,
        });
        assert!(changed, "sidebar scroll must request a redraw");
        assert_eq!(view.page_offset(), 0.0, "page must not scroll");
        let last_after = link_rects(&view).last().unwrap().y;
        assert!(last_after < last_before, "links must move up");

        // Tall: everything fits, nothing moves.
        let mut tall = ShowcaseView::new(1280.0, 1100.0);
        tall.render(&mut c);
        let (bx, by) = band(&tall).center();
        let before = link_rects(&tall)[0].y;
        let changed = tall.handle_event(&WidgetEvent::Scroll {
            x: bx,
            y: by,
            delta: 200.0,
        });
        assert!(!changed, "nothing to scroll: no redraw");
        assert_eq!(link_rects(&tall)[0].y, before);
    }

    #[test]
    fn sidebar_links_clip_to_their_band_and_do_not_hit_outside_it() {
        let mut view = ShowcaseView::new(1280.0, 500.0);
        let mut c = Compositor::new();
        view.render(&mut c);
        let band = band(&view);
        let overlay = view.layers.unwrap().overlay;
        let has_band_clip = c.layer(overlay).unwrap().nodes().iter().any(|n| {
            matches!(n, SceneNode::PushClip { x, y, w, h }
                    if *x == band.x && *y == band.y && *w == band.w && *h == band.h)
        });
        assert!(has_band_clip, "sidebar links must be clipped to the band");

        // A point below the band (footer zone) must not hover a link that
        // scrolled-space-wise would live there.
        let below = band.bottom() + 2.0;
        assert!(below < view.height, "test point must be inside the window");
        assert!(
            !view.handle_event(&WidgetEvent::MouseMove { x: 100.0, y: below }),
            "footer zone must not hover a clipped link"
        );
        assert!(view.shell.sidebar.links.iter().all(|l| !l.is_hovered()));
    }

    #[test]
    fn sidebar_labels_fit_their_slot_in_the_ui_font() {
        // Every section title fits the full sidebar's link width with the
        // icon slot, the shortcut hint and the paddings taken out.
        let view = ShowcaseView::new(1280.0, 800.0);
        let t = &view.theme;
        let link_w = link_rects(&view)[0].w;
        let style = NavLink::label_style(t);
        let (hint_w, _) =
            engine::text::TextMeasurer::measure_styled("14", &t.typography.caption_r(), None);
        let slot = link_w
            - t.spacing.sm * 2.0
            - NavLink::icon_slot(t)
            - t.spacing.xs
            - hint_w
            - t.spacing.sm;
        for section in Section::ALL {
            let (w, _) = engine::text::TextMeasurer::measure_styled(section.title(), &style, None);
            assert!(
                w <= slot,
                "{section:?} label measures {w:.1}px, over the {slot:.1}px slot"
            );
        }
    }

    #[test]
    fn sidebar_follows_the_breakpoint() {
        let t = Theme::hoff();
        // Desktop: the full sidebar takes its token width from the page.
        let wide = ShowcaseView::new(1400.0, 900.0);
        assert_eq!(wide.page_x(), t.size.sidebar_w);
        // Narrow desktop window: the rail.
        let medium = ShowcaseView::new(800.0, 700.0);
        assert_eq!(medium.page_x(), t.size.sidebar_rail_w);
        // Phone: a drawer behind the menu button; the page spans the width.
        let mut phone = ShowcaseView::new(390.0, 844.0);
        assert_eq!(phone.page_x(), 0.0);
        let mb = phone
            .shell
            .layout(&t)
            .menu_button
            .expect("menu button on a phone");
        let (x, y) = mb.center();
        phone.handle_event(&WidgetEvent::MouseDown { x, y });
        assert!(phone.handle_event(&WidgetEvent::MouseUp { x, y }));
        assert!(phone.shell.sidebar.drawer_open);
        // Navigating from the drawer switches the section and closes it.
        let rects = link_rects(&phone);
        let (x, y) = rects[4].center();
        phone.handle_event(&WidgetEvent::MouseDown { x, y });
        assert!(phone.handle_event(&WidgetEvent::MouseUp { x, y }));
        assert_eq!(phone.section, Section::Lists);
        assert!(!phone.shell.sidebar.drawer_open);
        // Every section renders on the phone.
        for section in Section::ALL {
            phone.set_section(section);
            let mut c = Compositor::new();
            phone.render(&mut c);
            assert!(
                c.layer(LayerId::DEFAULT).unwrap().nodes().len() > 10,
                "{section:?}"
            );
        }
    }

    #[test]
    fn every_section_renders_in_short_viewports() {
        // Width-only probes missed the sidebar overflow: pin short heights.
        for (w, h) in [(1280.0, 700.0), (1024.0, 600.0)] {
            let mut view = ShowcaseView::new(w, h);
            for section in Section::ALL {
                view.set_section(section);
                let mut c = Compositor::new();
                view.render(&mut c);
                let nodes = c.layer(LayerId::DEFAULT).unwrap().nodes().len();
                assert!(nodes > 10, "{section:?} at {w}x{h} emitted {nodes} nodes");
                // Page scroll limits track the real viewport height.
                assert_eq!(view.page_offset(), 0.0);
            }
        }
    }

    #[test]
    fn jump_to_section_matches_case_insensitively() {
        let mut view = ShowcaseView::new(1200.0, 800.0);
        view.jump_to_section("theme");
        assert_eq!(view.section, Section::Theme);
        view.jump_to_section("CARDS");
        assert_eq!(view.section, Section::Cards);
        view.jump_to_section("nope");
        assert_eq!(view.section, Section::Cards, "unknown names are ignored");
    }

    #[test]
    fn digit_keys_select_sections_and_t_cycles_themes() {
        let mut view = ShowcaseView::new(1200.0, 800.0);
        assert!(view.handle_key("7"));
        assert_eq!(view.section, Section::Theme);
        assert!(view.handle_key("8"));
        assert_eq!(view.section, Section::Charts);
        assert!(view.handle_key("9"));
        assert_eq!(view.section, Section::Dock);
        assert!(view.handle_key("0"), "0 reads as 10");
        assert_eq!(view.section, Section::App);
        assert!(view.handle_key("1"));
        assert_eq!(view.section, Section::Cards);

        assert!(view.handle_key("t"));
        assert_eq!(view.theme_name, "dark");
        assert!(view.handle_key("t"));
        assert_eq!(view.theme_name, "light");
        assert!(view.handle_key("t"));
        assert_eq!(view.theme_name, "hoff");
    }

    #[test]
    fn app_section_add_field_takes_typing_enter_and_escape_through_the_chrome() {
        let mut view = ShowcaseView::new(1200.0, 800.0);
        view.jump_to_section("app");
        let mut c = Compositor::new();
        view.render(&mut c);
        // Click the add field (top of the app card), then type through
        // the chrome's key routes: "t" must type, not switch the theme.
        let content = view.page_rect();
        let (x, y) = view.app.input_rect(content, &view.theme).center();
        assert!(view.handle_event(&WidgetEvent::MouseDown { x, y }));
        assert!(view.handle_key("t"));
        assert_eq!(view.theme_name, "hoff", "typing must not cycle the theme");
        assert!(view.handle_key("x"));
        assert!(view.handle_edit_key(EditKey::Backspace));
        assert!(view.handle_enter(), "Enter adds the todo");
        assert!(view.tick(1.0 / 60.0), "a focused field keeps frames coming");
        assert!(view.close_top_overlay(), "Escape blurs the field first");
        assert!(!view.handle_key("x"), "blurred: characters fall through");
    }

    #[test]
    fn sidebar_offers_one_nav_link_per_section() {
        let view = ShowcaseView::new(1200.0, 800.0);
        let rects = link_rects(&view);
        assert_eq!(rects.len(), Section::ALL.len());
        assert!(rects.iter().all(|r| r.h == NavLink::height(&view.theme)));
    }

    #[test]
    fn every_section_renders_a_scene() {
        let mut view = ShowcaseView::new(1200.0, 800.0);
        for section in Section::ALL {
            view.set_section(section);
            let mut c = Compositor::new();
            view.render(&mut c);
            let nodes = c.layer(LayerId::DEFAULT).unwrap().nodes().len();
            assert!(nodes > 10, "{section:?} emitted only {nodes} nodes");
        }
    }

    // -- Reactivity regression probes (event chain must report changes) ----

    /// Find the y position of a text node on a layer (probe helper).
    fn text_node_y(c: &Compositor, layer: LayerId, needle: &str) -> Option<f32> {
        c.layer(layer).unwrap().nodes().iter().find_map(|n| {
            if let SceneNode::Text { key, y, .. } = n {
                (key.text == needle).then_some(*y)
            } else {
                None
            }
        })
    }

    #[test]
    fn probe_scroll_over_virtual_list_reports_change_and_shifts_rows() {
        let mut view = ShowcaseView::new(1200.0, 800.0);
        view.set_section(Section::Lists);
        let mut c = Compositor::new();
        view.render(&mut c);

        let bounds = view.lists.list_bounds(view.content_rect());
        let (cx, cy) = bounds.center();
        let y_before = text_node_y(&c, view.layers.unwrap().list, "Item 0")
            .expect("Item 0 rendered before scroll");

        let changed = view.handle_event(&WidgetEvent::Scroll {
            x: cx,
            y: cy,
            delta: 48.0,
        });
        assert!(changed, "scroll over the virtual list must request redraw");
        assert!(view.lists.list.scroll.offset() > 0.0, "offset must move");

        view.render(&mut c);
        let y_after = text_node_y(&c, view.layers.unwrap().list, "Item 0")
            .expect("Item 0 still in overscan after 48px scroll");
        assert!(
            y_after < y_before,
            "rows must shift up after scrolling down (before={y_before}, after={y_after})"
        );
    }

    #[test]
    fn probe_hover_over_sidebar_reports_change() {
        let mut view = ShowcaseView::new(1200.0, 800.0);
        let rect = link_rects(&view)[2];
        let (cx, cy) = rect.center();
        assert!(
            view.handle_event(&WidgetEvent::MouseMove { x: cx, y: cy }),
            "hover entering a sidebar item must request redraw"
        );
        assert!(
            !view.handle_event(&WidgetEvent::MouseMove { x: cx, y: cy }),
            "unchanged hover must not request redraw"
        );
        assert!(
            view.handle_event(&WidgetEvent::MouseMove { x: cx, y: 4.0 }),
            "hover leaving the sidebar item must request redraw"
        );
    }

    #[test]
    fn probe_hover_over_buttons_reports_change() {
        let mut view = ShowcaseView::new(1200.0, 800.0);
        view.set_section(Section::Buttons);
        let content = view.content_rect();
        // First button row starts after the group label.
        let (cx, cy) = (content.x + 30.0, content.y + 24.0 + 22.0);
        assert!(
            view.handle_event(&WidgetEvent::MouseMove { x: cx, y: cy }),
            "hover entering a button must request redraw"
        );
        assert!(
            view.handle_event(&WidgetEvent::MouseMove {
                x: cx,
                y: cy - 200.0
            }),
            "hover leaving a button must request redraw"
        );
    }

    #[test]
    fn probe_click_sidebar_switches_section_and_reports_change() {
        let mut view = ShowcaseView::new(1200.0, 800.0);
        let rect = link_rects(&view)[4]; // Lists
        let (cx, cy) = rect.center();
        view.handle_event(&WidgetEvent::MouseDown { x: cx, y: cy });
        assert!(view.handle_event(&WidgetEvent::MouseUp { x: cx, y: cy }));
        assert_eq!(view.section, Section::Lists);
    }

    #[test]
    fn probe_ticks_settle_with_no_input() {
        let mut view = ShowcaseView::new(1200.0, 800.0);
        for section in Section::ALL {
            view.set_section(section);
            let mut c = Compositor::new();
            view.render(&mut c);
            let mut animating = true;
            for _ in 0..600 {
                animating = view.tick(1.0 / 60.0);
                if !animating {
                    break;
                }
            }
            assert!(!animating, "{section:?}: tick never settles (busy loop)");
        }
    }

    #[test]
    fn probe_idle_rerender_is_stable() {
        let mut view = ShowcaseView::new(1200.0, 800.0);
        for section in Section::ALL {
            view.set_section(section);
            let mut c = Compositor::new();
            view.render(&mut c);
            view.tick(1.0 / 60.0);
            c.resolve_scene((1200.0, 800.0));
            for l in c.layers().iter().map(|l| l.id).collect::<Vec<_>>() {
                c.mark_layer_clean(l);
            }
            // No input: the next frame must hash identically.
            view.tick(1.0 / 60.0);
            view.render(&mut c);
            assert!(
                !c.needs_render(),
                "{section:?}: idle re-render produced a different scene"
            );
        }
    }

    // -- Page scroll regressions (HOFF pages overflow the 800px window) ----

    #[test]
    fn wheel_scrolls_overflowing_sections_and_requests_redraw() {
        for section in [Section::Cards, Section::Theme] {
            let mut view = ShowcaseView::new(1200.0, 800.0);
            view.set_section(section);
            let mut c = Compositor::new();
            view.render(&mut c);

            let content = view.content_rect();
            assert!(
                view.section_content_height(content) > content.h,
                "{section:?} fits in 800px now; pick a taller section for this test"
            );

            let (cx, cy) = content.center();
            let changed = view.handle_event(&WidgetEvent::Scroll {
                x: cx,
                y: cy,
                delta: 120.0,
            });
            assert!(changed, "{section:?}: page scroll must request a redraw");
            assert!(
                view.page_offset() > 0.0,
                "{section:?}: page offset must move"
            );

            // Scrolling back past the top clamps at zero and still redraws.
            let changed = view.handle_event(&WidgetEvent::Scroll {
                x: cx,
                y: cy,
                delta: -10_000.0,
            });
            assert!(changed);
            assert_eq!(view.page_offset(), 0.0, "clamped at the top");
        }
    }

    #[test]
    fn page_scroll_shifts_rendered_nodes_and_clips_to_viewport() {
        let mut view = ShowcaseView::new(1200.0, 800.0);
        view.set_section(Section::Theme);
        let mut c = Compositor::new();
        view.render(&mut c);
        let y_before =
            text_node_y(&c, LayerId::DEFAULT, "PALETTES").expect("palettes label rendered");

        let content = view.content_rect();
        let (cx, cy) = content.center();
        assert!(view.handle_event(&WidgetEvent::Scroll {
            x: cx,
            y: cy,
            delta: 100.0,
        }));
        view.render(&mut c);
        let y_after = text_node_y(&c, LayerId::DEFAULT, "PALETTES").expect("still emitted");
        assert!(
            (y_before - y_after - 100.0).abs() < 0.5,
            "nodes must shift up by the scrolled amount (before={y_before}, after={y_after})"
        );

        // The scrolled content is wrapped in a clip to the content viewport
        // (the sidebar band has its own clip at x=0 — find the content one).
        let nodes = c.layer(LayerId::DEFAULT).unwrap().nodes();
        let page_x = view.page_x();
        let clip = nodes.iter().find_map(|n| match n {
            SceneNode::PushClip { x, y, w, h } if *x == page_x => Some((*x, *y, *w, *h)),
            _ => None,
        });
        let clip = clip.expect("section content must be clipped while scrolled");
        assert_eq!(clip.0, page_x);
        assert_eq!(clip.1, content.y);
    }

    #[test]
    fn scroll_over_virtual_list_keeps_priority_over_page_scroll() {
        let mut view = ShowcaseView::new(1200.0, 800.0);
        view.set_section(Section::Lists);
        let mut c = Compositor::new();
        view.render(&mut c);

        let bounds = view.lists.list_bounds(view.content_rect());
        let (cx, cy) = bounds.center();
        assert!(view.handle_event(&WidgetEvent::Scroll {
            x: cx,
            y: cy,
            delta: 48.0,
        }));
        assert!(view.lists.list.scroll.offset() > 0.0);
        assert_eq!(
            view.page_offset(),
            0.0,
            "the lists page itself fits the viewport and must not scroll"
        );
    }

    #[test]
    fn clicks_on_the_header_band_do_not_reach_scrolled_widgets() {
        let mut view = ShowcaseView::new(1200.0, 800.0);
        view.set_section(Section::Theme);
        let mut c = Compositor::new();
        view.render(&mut c);

        let content = view.content_rect();
        let (cx, cy) = content.center();
        // Scroll until palette cards sit underneath the header band.
        view.handle_event(&WidgetEvent::Scroll {
            x: cx,
            y: cy,
            delta: 160.0,
        });
        let before = view.theme_name.clone();
        // Click in the header band — must not pick a (hidden) theme card.
        view.handle_event(&WidgetEvent::MouseDown { x: cx, y: 60.0 });
        assert_eq!(view.theme_name, before, "header clicks must hit nothing");
    }

    #[test]
    fn probe_event_change_marks_compositor_needs_render() {
        let mut view = ShowcaseView::new(1200.0, 800.0);
        view.set_section(Section::Lists);
        let mut c = Compositor::new();
        view.render(&mut c);
        c.resolve_scene((1200.0, 800.0));
        for l in c.layers().iter().map(|l| l.id).collect::<Vec<_>>() {
            c.mark_layer_clean(l);
        }
        assert!(!c.needs_render(), "clean after render+resolve");

        let bounds = view.lists.list_bounds(view.content_rect());
        let (cx, cy) = bounds.center();
        let changed = view.handle_event(&WidgetEvent::Scroll {
            x: cx,
            y: cy,
            delta: 48.0,
        });
        assert!(changed);
        view.render(&mut c);
        assert!(
            c.needs_render(),
            "scrolled scene must be detected as needing render"
        );
    }
}
