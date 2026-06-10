//! Showcase view: sidebar navigation + one module per gallery section.

mod buttons;
mod forms;
mod icons_gallery;
mod lists;
mod overlays;
mod theme_gallery;

use plev::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use plev::overlay::{OverlayId, OverlayKind, OverlayManager};
use plev::theme::{Intent, Theme};
use plev::ui::icons;
use plev::ui::widgets::{
    ContextMenu, EventResult, Modal, ModalAction, Rect, ToastManager, WidgetEvent,
    path_rounded_rect, path_rounded_rect_stroke,
};

pub const SIDEBAR_W: f32 = 230.0;
const PAD: f32 = 40.0;
/// Vertical space used by the section header (title + blurb).
const HEADER_H: f32 = 78.0;

// ---------------------------------------------------------------------------
// Sections
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Section {
    Buttons,
    Forms,
    Overlays,
    Lists,
    Icons,
    Theme,
}

impl Section {
    pub const ALL: [Section; 6] = [
        Section::Buttons,
        Section::Forms,
        Section::Overlays,
        Section::Lists,
        Section::Icons,
        Section::Theme,
    ];

    fn title(self) -> &'static str {
        match self {
            Section::Buttons => "Buttons",
            Section::Forms => "Forms",
            Section::Overlays => "Overlays",
            Section::Lists => "Lists",
            Section::Icons => "Icons",
            Section::Theme => "Theme",
        }
    }

    fn icon(self) -> &'static str {
        match self {
            Section::Buttons => "square",
            Section::Forms => "settings",
            Section::Overlays => "copy",
            Section::Lists => "file",
            Section::Icons => "eye",
            Section::Theme => "sun",
        }
    }

    fn blurb(self) -> &'static str {
        match self {
            Section::Buttons => "Variants, sizes, intents and states of plev::ui::widgets::Button.",
            Section::Forms => "Checkbox, switch, slider, progress, select and tabs.",
            Section::Overlays => {
                "Modal, context menu, tooltip and toasts — spring physics per intent."
            }
            Section::Lists => "Virtualized list with 10,000 rows, and a tree view.",
            Section::Icons => "Lucide icon set, tessellated to GPU paths with per-size caching.",
            Section::Theme => "Every built-in palette. Click a card to apply it.",
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
    sidebar_hover: Option<usize>,

    pub overlay_mgr: OverlayManager,
    layers: Option<Layers>,

    pub toasts: ToastManager,
    active_overlay: Option<ActiveOverlay>,

    buttons: buttons::ButtonsSection,
    forms: forms::FormsSection,
    overlays: overlays::OverlaysSection,
    lists: lists::ListsSection,
    icons_gallery: icons_gallery::IconsSection,
    themes: theme_gallery::ThemeSection,
}

#[derive(Clone, Copy)]
struct Layers {
    list: LayerId,
    overlay: LayerId,
    toast: LayerId,
}

impl ShowcaseView {
    pub fn new(width: f32, height: f32) -> Self {
        let theme = Theme::dark();
        Self {
            width,
            height,
            scale_factor: 1.0,
            forms: forms::FormsSection::new(&theme),
            buttons: buttons::ButtonsSection::new(),
            overlays: overlays::OverlaysSection::new(),
            lists: lists::ListsSection::new(),
            icons_gallery: icons_gallery::IconsSection::new(),
            themes: theme_gallery::ThemeSection::new(),
            theme,
            theme_name: "dark".to_string(),
            section: Section::Buttons,
            sidebar_hover: None,
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
    }

    /// Content area to the right of the sidebar, below the header.
    fn content_rect(&self) -> Rect {
        Rect::new(
            SIDEBAR_W + PAD,
            PAD + HEADER_H,
            (self.width - SIDEBAR_W - PAD * 2.0).max(200.0),
            (self.height - PAD * 2.0 - HEADER_H).max(120.0),
        )
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
            self.section = *section;
        }
    }

    // -- Input ---------------------------------------------------------------

    /// Returns `true` when the key was consumed (and a redraw is needed).
    pub fn handle_key(&mut self, key: &str) -> bool {
        match key {
            "t" | "T" => {
                let next = if self.theme_name == "light" {
                    "dark"
                } else {
                    "light"
                };
                self.set_theme(next);
                true
            }
            d @ ("1" | "2" | "3" | "4" | "5" | "6") => {
                let idx = d.as_bytes()[0] - b'1';
                self.section = Section::ALL[idx as usize];
                true
            }
            _ => false,
        }
    }

    /// Close the topmost overlay (Escape). Returns `false` when there was
    /// nothing to close (caller may quit).
    pub fn close_top_overlay(&mut self) -> bool {
        if self.forms.select_is_open() {
            self.forms.close_select();
            return true;
        }
        self.overlay_mgr.pop_animated().is_some()
    }

    pub fn handle_right_click(&mut self, x: f32, y: f32) -> bool {
        if self.section != Section::Overlays || self.active_overlay.is_some() {
            return false;
        }
        let content = self.content_rect();
        if !self.overlays.menu_area(content).contains(x, y) {
            return false;
        }
        self.open_menu(x, y);
        true
    }

    fn open_menu(&mut self, x: f32, y: f32) {
        let widget = overlays::demo_menu();
        let (w, h) = widget.size();
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
        let dialog = widget.dialog_rect(self.width, self.height);
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
        let toast_result = self.toasts.handle_event(event, vw, vh);
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
                        let (action, r) = widget.handle_event(event, vw, vh);
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
                        let (r, clicked) = widget.handle_event(event, *x, *y);
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
                            let (w, h) = widget.size();
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
            let r = self.forms.route_select(event, self.content_rect());
            if r.handled || r.changed {
                return r.changed;
            }
        }

        result = result.merge(self.handle_sidebar(event));

        let content = self.content_rect();
        let section_result = match self.section {
            Section::Buttons => self.buttons.handle_event(event, content),
            Section::Forms => self.forms.handle_event(event, content),
            Section::Overlays => {
                let (r, action) = self.overlays.handle_event(event, content);
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
            Section::Lists => self.lists.handle_event(event, content),
            Section::Icons => self.icons_gallery.handle_event(event, content),
            Section::Theme => {
                let (r, picked) = self.themes.handle_event(event, content);
                if let Some(name) = picked {
                    self.set_theme(name);
                }
                r
            }
        };
        result = result.merge(section_result);
        result.changed
    }

    fn handle_sidebar(&mut self, event: &WidgetEvent) -> EventResult {
        let items = self.sidebar_item_rects();
        match *event {
            WidgetEvent::MouseMove { x, y } => {
                let hit = items.iter().position(|r| r.contains(x, y));
                if hit != self.sidebar_hover {
                    self.sidebar_hover = hit;
                    EventResult::changed()
                } else {
                    EventResult::IGNORED
                }
            }
            WidgetEvent::MouseDown { x, y } => {
                if let Some(i) = items.iter().position(|r| r.contains(x, y)) {
                    if Section::ALL[i] != self.section {
                        self.section = Section::ALL[i];
                        return EventResult::clicked();
                    }
                    return EventResult {
                        handled: true,
                        ..EventResult::IGNORED
                    };
                }
                EventResult::IGNORED
            }
            _ => EventResult::IGNORED,
        }
    }

    fn sidebar_item_rects(&self) -> Vec<Rect> {
        let top = 92.0;
        Section::ALL
            .iter()
            .enumerate()
            .map(|(i, _)| Rect::new(10.0, top + i as f32 * 38.0, SIDEBAR_W - 20.0, 34.0))
            .collect()
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

        self.render_sidebar(c, &theme);
        self.render_header(c, &theme);

        let content = self.content_rect();

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

        match self.section {
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
        }

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

    fn render_sidebar(&self, c: &mut Compositor, theme: &Theme) {
        c.push(SceneNode::Rect {
            x: 0.0,
            y: 0.0,
            w: SIDEBAR_W,
            h: self.height,
            color: theme.colors.surface.0,
        });
        c.push(SceneNode::Rect {
            x: SIDEBAR_W - 1.0,
            y: 0.0,
            w: 1.0,
            h: self.height,
            color: theme.colors.divider.0,
        });

        text(c, "plev", 20.0, 700, 20.0, 28.0, theme.colors.text.0);
        text(
            c,
            "design system",
            11.0,
            400,
            22.0,
            54.0,
            theme.colors.text_dim.0,
        );

        for (i, (section, rect)) in Section::ALL
            .iter()
            .zip(self.sidebar_item_rects())
            .enumerate()
        {
            let active = *section == self.section;
            let hovered = self.sidebar_hover == Some(i);
            if active || hovered {
                // Path-based highlight so the section icon stays on top.
                c.push(path_rounded_rect(
                    rect.x,
                    rect.y,
                    rect.w,
                    rect.h,
                    theme.radius.md + 2.0,
                    if active {
                        [
                            theme.colors.accent.0[0],
                            theme.colors.accent.0[1],
                            theme.colors.accent.0[2],
                            0.12,
                        ]
                    } else {
                        theme.colors.bg_hover.0
                    },
                ));
            }
            let fg = if active {
                theme.colors.text.0
            } else {
                theme.colors.text_mid.0
            };
            if let Some(node) =
                icons::icon_at(section.icon(), 15.0, fg, rect.x + 12.0, rect.y + 9.5)
            {
                c.push(node);
            }
            text(
                c,
                section.title(),
                13.0,
                if active { 600 } else { 400 },
                rect.x + 36.0,
                rect.y + (rect.h - 13.0 * 1.3) / 2.0,
                fg,
            );
            text(
                c,
                &format!("{}", i + 1),
                11.0,
                400,
                rect.x + rect.w - 18.0,
                rect.y + (rect.h - 11.0 * 1.3) / 2.0,
                theme.colors.text_dim.0,
            );
        }

        // Footer hints.
        let hint_y = self.height - 54.0;
        text(
            c,
            "T  toggle dark/light",
            11.0,
            400,
            20.0,
            hint_y,
            theme.colors.text_dim.0,
        );
        text(
            c,
            "Esc  close overlays",
            11.0,
            400,
            20.0,
            hint_y + 18.0,
            theme.colors.text_dim.0,
        );
    }

    fn render_header(&self, c: &mut Compositor, theme: &Theme) {
        let x = SIDEBAR_W + PAD;
        text(
            c,
            self.section.title(),
            24.0,
            700,
            x,
            PAD,
            theme.colors.text.0,
        );
        text(
            c,
            self.section.blurb(),
            13.0,
            400,
            x,
            PAD + 36.0,
            theme.colors.text_mid.0,
        );
    }
}

// ---------------------------------------------------------------------------
// Shared drawing helpers for the section modules
// ---------------------------------------------------------------------------

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
        key: TextNodeKey::new(s, size, size * 1.3, None).with_weight(weight),
        x,
        y,
        color,
    });
}

/// Uppercase group label (the small section headings inside content).
pub(crate) fn group_label(c: &mut Compositor, s: &str, x: f32, y: f32, theme: &Theme) {
    text(c, s, 11.0, 600, x, y, theme.colors.text_dim.0);
}

/// Soft panel container (cards behind lists, menu demo area, etc.).
/// Path-based so icon paths drawn on top of it stay visible.
pub(crate) fn panel(c: &mut Compositor, rect: Rect, theme: &Theme) {
    c.push(path_rounded_rect(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        theme.radius.lg,
        theme.colors.surface.0,
    ));
    c.push(path_rounded_rect_stroke(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        theme.radius.lg,
        theme.colors.divider.0,
        1.0,
    ));
}
