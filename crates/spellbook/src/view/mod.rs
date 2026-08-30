//! The Spellbook window: a HOFF sidebar rail and one module per screen.
//!
//! Follows plev's official app pattern (docs/adr/official-app-pattern.md):
//! state lives in plain structs, screens own retained widgets, and every
//! visible mutation reports `changed` so the shell can invalidate. Nothing
//! in here touches the GPU or winit - the whole view builds a scene into a
//! plain `Compositor`, which is what lets the layout tests below run headless.
//!
//! This replaces a hash router over a DOM. The two jobs the router did that
//! still exist are here: `Route` decides which screen is live, and the deck
//! sub-list under "Meus Decks" is shown only while that section is the one
//! being browsed.

mod collection;
mod deck_tile;
mod decks;
mod games;
mod home;
mod scanner;
mod wishlist;

use std::sync::mpsc::Sender;

use engine::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use engine::input::scroll::ScrollState;
use engine::overlay::OverlayManager;
use engine::theme::{Intent, Theme};
use engine::ui::icons;
use engine::ui::widgets::{
    EventResult, Rect, ToastManager, WidgetEvent, rounded_rect, rounded_rect_stroke,
};
use spellbook_core::client::{Command, Event};

use crate::art::ArtCache;

/// Sidebar rail width. The one horizontal constant the layout is allowed:
/// every content width below derives from `width - SIDEBAR_W`.
pub const SIDEBAR_W: f32 = 248.0;
const PAD: f32 = 40.0;
/// Vertical space used by the screen header (title + blurb).
const HEADER_H: f32 = 78.0;
/// HOFF nav link height (48px, radius 12).
const NAV_H: f32 = 48.0;
/// Sidebar logo block height: the nav links start below it.
const SIDEBAR_TOP: f32 = 96.0;
/// Sidebar footer band (the card-index panel) at the window bottom.
const SIDEBAR_FOOTER_H: f32 = 96.0;

// ---------------------------------------------------------------------------
// Routes
// ---------------------------------------------------------------------------

/// Which screen is showing.
///
/// `Deck` carries an id, so it is not one of the sidebar links - it is reached
/// from the deck grid or from the sub-list under "Meus Decks", exactly as
/// `#deck/12` was.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Route {
    Home,
    Decks,
    Deck(i64),
    Collection,
    Wishlist,
    Scanner,
    Games,
}

impl Route {
    /// The links in the rail, in order.
    pub const NAV: [Route; 6] = [
        Route::Home,
        Route::Decks,
        Route::Collection,
        Route::Wishlist,
        Route::Scanner,
        Route::Games,
    ];

    fn title(self) -> &'static str {
        match self {
            Route::Home => "Home",
            Route::Decks | Route::Deck(_) => "Meus Decks",
            Route::Collection => "Coleção",
            Route::Wishlist => "Wishlist",
            Route::Scanner => "Scanner",
            Route::Games => "Partidas",
        }
    }

    fn icon(self) -> &'static str {
        match self {
            Route::Home => "house",
            Route::Decks | Route::Deck(_) => "layers",
            Route::Collection => "book-open",
            Route::Wishlist => "heart",
            Route::Scanner => "eye",
            Route::Games => "layout-grid",
        }
    }

    fn blurb(self) -> &'static str {
        match self {
            Route::Home => "Seu laboratório de coleção e decks, tudo lido do banco local.",
            Route::Decks | Route::Deck(_) => {
                "Todo deck que você montou, com o comandante e as cores de cada um."
            }
            Route::Collection => "Cada cópia física que você tem, livre ou sleevada num deck.",
            Route::Wishlist => "Cartas que você quer e ainda não tem.",
            Route::Scanner => "Reconhecimento de cartas pela câmera.",
            Route::Games => "Partidas registradas e o aproveitamento de cada deck.",
        }
    }

    /// Index into the per-route scroll array. `Deck` shares the decks slot,
    /// because leaving a deck returns to the grid where you left it.
    fn scroll_slot(self) -> usize {
        match self {
            Route::Home => 0,
            Route::Decks | Route::Deck(_) => 1,
            Route::Collection => 2,
            Route::Wishlist => 3,
            Route::Scanner => 4,
            Route::Games => 5,
        }
    }

    /// Whether this route belongs to the decks section, which is what makes
    /// the deck sub-list visible in the rail.
    fn in_decks_section(self) -> bool {
        matches!(self, Route::Decks | Route::Deck(_))
    }
}

// ---------------------------------------------------------------------------
// Screen context and actions
// ---------------------------------------------------------------------------

/// Something a screen asked the shell to do. Screens never touch the router,
/// the toaster or the client directly - they push an action and the shell
/// applies it after the event is fully handled, so a click cannot re-enter
/// the screen that is still running its handler.
#[derive(Clone, Debug)]
pub enum ScreenAction {
    Navigate(Route),
    Toast(String, Intent),
}

/// What every screen gets on each call: the command channel to the worker
/// thread, and the action queue.
pub struct ScreenCtx<'a> {
    pub tx: &'a Sender<Command>,
    pub actions: &'a mut Vec<ScreenAction>,
}

impl ScreenCtx<'_> {
    pub fn send(&self, command: Command) {
        let _ = self.tx.send(command);
    }

    pub fn navigate(&mut self, route: Route) {
        self.actions.push(ScreenAction::Navigate(route));
    }

    pub fn toast(&mut self, message: impl Into<String>, intent: Intent) {
        self.actions
            .push(ScreenAction::Toast(message.into(), intent));
    }
}

// ---------------------------------------------------------------------------
// View
// ---------------------------------------------------------------------------

pub struct SpellbookView {
    pub width: f32,
    pub height: f32,
    pub scale_factor: f32,
    pub theme: Theme,
    pub route: Route,

    /// Commands to the data thread. Cloned from the client; screens send
    /// their own loads through `ScreenCtx`.
    tx: Sender<Command>,
    /// Card artwork, filled by the worker one batch at a time.
    pub art: ArtCache,
    /// Actions screens pushed during the current event, drained by the shell
    /// once the handler returns.
    actions: Vec<ScreenAction>,

    sidebar_hover: Option<usize>,
    sidebar_scroll: ScrollState,
    /// Page-level vertical scroll, one per route: these screens are taller
    /// than the window, and each keeps its own position.
    page_scroll: [ScrollState; Route::NAV.len()],

    pub toasts: ToastManager,
    pub overlay_mgr: OverlayManager,
    layers: Option<Layers>,

    home: home::HomeScreen,
    decks: decks::DecksScreen,
    collection: collection::CollectionScreen,
    wishlist: wishlist::WishlistScreen,
    scanner: scanner::ScannerScreen,
    games: games::GamesScreen,
}

#[derive(Clone, Copy)]
struct Layers {
    /// Scrolled, clipped screen content (grids and lists).
    content: LayerId,
    /// Modals and menus, above the content. Unused until the first dialog
    /// lands; the layer exists so z-order is fixed from the start.
    #[allow(dead_code)]
    overlay: LayerId,
    toast: LayerId,
}

impl SpellbookView {
    pub fn new(width: f32, height: f32, tx: Sender<Command>) -> Self {
        Self {
            width,
            height,
            scale_factor: 1.0,
            theme: Theme::hoff(),
            route: Route::Home,
            tx,
            art: ArtCache::new(),
            actions: Vec::new(),
            sidebar_hover: None,
            sidebar_scroll: ScrollState::new(),
            page_scroll: std::array::from_fn(|_| ScrollState::new()),
            toasts: ToastManager::new(),
            overlay_mgr: OverlayManager::new(),
            layers: None,
            home: home::HomeScreen::new(),
            decks: decks::DecksScreen::new(),
            collection: collection::CollectionScreen::new(),
            wishlist: wishlist::WishlistScreen::new(),
            scanner: scanner::ScannerScreen::new(),
            games: games::GamesScreen::new(),
        }
    }

    pub fn resize(&mut self, width: f32, height: f32, scale_factor: f32) {
        self.width = width;
        self.height = height;
        self.scale_factor = scale_factor;
    }

    /// Content area to the right of the rail, below the header. Derived from
    /// the window, never from a fixed page width - the CSS this replaces
    /// capped the layout at 1400px and left the rest of a wide display empty.
    pub fn content_rect(&self) -> Rect {
        Rect::new(
            SIDEBAR_W + PAD,
            PAD + HEADER_H,
            (self.width - SIDEBAR_W - PAD * 2.0).max(200.0),
            (self.height - PAD * 2.0 - HEADER_H).max(120.0),
        )
    }

    /// Content rect shifted up by the page scroll offset. Screens lay out,
    /// hit-test and render against this rect so events and pixels always
    /// agree; render clips it back to the viewport.
    fn page_rect(&self) -> Rect {
        let mut rect = self.content_rect();
        rect.y -= self.page_scroll[self.route.scroll_slot()].offset();
        rect
    }

    fn content_height(&self, content: Rect) -> f32 {
        match self.route {
            Route::Home => self.home.content_height(content),
            Route::Decks | Route::Deck(_) => self.decks.content_height(content),
            Route::Collection => self.collection.content_height(content),
            Route::Wishlist => self.wishlist.content_height(content),
            Route::Scanner => self.scanner.content_height(content),
            Route::Games => self.games.content_height(content),
        }
    }

    fn sync_page_scroll(&mut self) {
        let content = self.content_rect();
        let height = self.content_height(content);
        let scroll = &mut self.page_scroll[self.route.scroll_slot()];
        scroll.set_viewport(content.h);
        scroll.set_content(height);
    }

    pub fn navigate(&mut self, route: Route) {
        if route == self.route {
            return;
        }
        self.route = route;
        self.enter_route();
    }

    /// The first load, called once the client exists: the screen on the
    /// current route queues whatever it needs.
    pub fn boot(&mut self) {
        self.enter_route();
    }

    fn enter_route(&mut self) {
        let mut ctx = ScreenCtx {
            tx: &self.tx,
            actions: &mut self.actions,
        };
        match self.route {
            Route::Home => self.home.on_enter(self.route, &mut ctx),
            Route::Decks | Route::Deck(_) => self.decks.on_enter(self.route, &mut ctx),
            Route::Collection => self.collection.on_enter(self.route, &mut ctx),
            Route::Wishlist => self.wishlist.on_enter(self.route, &mut ctx),
            Route::Scanner => self.scanner.on_enter(self.route, &mut ctx),
            Route::Games => self.games.on_enter(self.route, &mut ctx),
        }
        self.drain_actions();
    }

    /// An answer from the worker thread. Every screen sees every event and
    /// picks out the variants it asked for; a screen that did not send the
    /// command ignores the event. Returns `true` when a redraw is needed.
    pub fn handle_data(&mut self, event: &Event) -> bool {
        // Art resolves into the cache, not into a screen.
        if let Event::ArtLoaded { images } = event {
            let mut changed = false;
            for (rel, art) in images {
                changed |= self.art.resolve(
                    rel,
                    art.as_ref().map(|a| crate::art::Rgba {
                        width: a.width,
                        height: a.height,
                        pixels: a.pixels.clone(),
                    }),
                );
            }
            return changed;
        }

        if let Event::Failed(e) = event {
            self.toasts
                .push(e.detail().to_string(), Intent::Destructive, &self.theme);
            return true;
        }

        // A finished data update means art that was missing all session may
        // now be on disk - the misses get another chance.
        if let Event::UpdateStatusLoaded(status) = event
            && status.state == "done"
        {
            self.retry_missing_art();
        }

        let mut changed = false;
        {
            let mut ctx = ScreenCtx {
                tx: &self.tx,
                actions: &mut self.actions,
            };
            changed |= self.home.on_event(event, &mut ctx);
            changed |= self.decks.on_event(event, &mut ctx);
            changed |= self.collection.on_event(event, &mut ctx);
            changed |= self.wishlist.on_event(event, &mut ctx);
            changed |= self.scanner.on_event(event, &mut ctx);
            changed |= self.games.on_event(event, &mut ctx);
        }
        self.drain_actions();
        changed
    }

    /// Art the screens asked for while laying out this frame, drained so the
    /// shell can queue one batched load.
    pub fn take_art_requests(&mut self) -> Vec<String> {
        self.art.take_requests()
    }

    /// After a data update filled the cache, previously-missing art is worth
    /// another try.
    pub fn retry_missing_art(&mut self) {
        self.art.retry_missing();
    }

    /// Apply the actions screens queued during the last event.
    fn drain_actions(&mut self) {
        let actions = std::mem::take(&mut self.actions);
        for action in actions {
            match action {
                ScreenAction::Navigate(route) => self.navigate(route),
                ScreenAction::Toast(message, intent) => {
                    self.toasts.push(message, intent, &self.theme);
                }
            }
        }
    }

    // -- Input ---------------------------------------------------------------

    /// Route a pointer event. Returns `true` if a redraw is needed.
    pub fn handle_event(&mut self, event: &WidgetEvent) -> bool {
        let mut result = self.toasts.handle_event(event, self.width, self.height);
        if result.clicked {
            return true;
        }

        result = result.merge(self.handle_sidebar(event));

        // Clicks on the header band belong to the chrome: content scrolled
        // underneath it must not receive them.
        let viewport = self.content_rect();
        if let WidgetEvent::MouseDown { x, y } = *event
            && x >= SIDEBAR_W
            && y < viewport.y
        {
            return result.changed;
        }

        let content = self.page_rect();
        let screen = {
            let mut ctx = ScreenCtx {
                tx: &self.tx,
                actions: &mut self.actions,
            };
            match self.route {
                Route::Home => self.home.handle_event(event, content, &mut ctx),
                Route::Decks | Route::Deck(_) => self.decks.handle_event(event, content, &mut ctx),
                Route::Collection => self.collection.handle_event(event, content, &mut ctx),
                Route::Wishlist => self.wishlist.handle_event(event, content, &mut ctx),
                Route::Scanner => self.scanner.handle_event(event, content, &mut ctx),
                Route::Games => self.games.handle_event(event, content, &mut ctx),
            }
        };
        self.drain_actions();
        result = result.merge(screen);

        // Wheel: the rail scrolls on its own, the page scrolls otherwise.
        // Only an actual offset change asks for a frame.
        if let WidgetEvent::Scroll { x, delta, .. } = *event
            && !result.handled
        {
            if x < SIDEBAR_W {
                self.sync_sidebar_scroll();
                let old = self.sidebar_scroll.offset();
                self.sidebar_scroll.scroll_by(delta);
                if self.sidebar_scroll.offset() != old {
                    result = result.merge(EventResult::changed());
                }
            } else {
                self.sync_page_scroll();
                let scroll = &mut self.page_scroll[self.route.scroll_slot()];
                let old = scroll.offset();
                scroll.scroll_by(delta);
                if scroll.offset() != old {
                    result = result.merge(EventResult::changed());
                }
            }
        }

        result.changed
    }

    /// Scrollable band of the rail: below the logo, above the footer panel.
    /// Derived from the window height, never a constant.
    fn sidebar_viewport(&self) -> Rect {
        Rect::new(
            0.0,
            SIDEBAR_TOP,
            SIDEBAR_W,
            (self.height - SIDEBAR_TOP - SIDEBAR_FOOTER_H).max(NAV_H),
        )
    }

    fn sync_sidebar_scroll(&mut self) {
        let viewport = self.sidebar_viewport();
        self.sidebar_scroll.set_viewport(viewport.h);
        self.sidebar_scroll
            .set_content(Route::NAV.len() as f32 * (NAV_H + 4.0));
    }

    fn sidebar_item_rects(&self) -> Vec<Rect> {
        let offset = self.sidebar_scroll.offset();
        Route::NAV
            .iter()
            .enumerate()
            .map(|(i, _)| {
                Rect::new(
                    12.0,
                    SIDEBAR_TOP + i as f32 * (NAV_H + 4.0) - offset,
                    SIDEBAR_W - 24.0,
                    NAV_H,
                )
            })
            .collect()
    }

    fn handle_sidebar(&mut self, event: &WidgetEvent) -> EventResult {
        let items = self.sidebar_item_rects();
        let viewport = self.sidebar_viewport();
        // Links scrolled outside the band are clipped visually; they must not
        // hover or hit either.
        let hit = |x: f32, y: f32| -> Option<usize> {
            viewport
                .contains(x, y)
                .then(|| items.iter().position(|r| r.contains(x, y)))
                .flatten()
        };
        match *event {
            WidgetEvent::MouseMove { x, y } => {
                let hovered = hit(x, y);
                if hovered != self.sidebar_hover {
                    self.sidebar_hover = hovered;
                    EventResult::changed()
                } else {
                    EventResult::IGNORED
                }
            }
            WidgetEvent::MouseDown { x, y } => match hit(x, y) {
                Some(i) => {
                    let target = Route::NAV[i];
                    // Clicking "Meus Decks" while inside a deck goes back to
                    // the grid; clicking the link you are already on is a
                    // no-op that still swallows the click.
                    if target != self.route {
                        self.navigate(target);
                        return EventResult::clicked();
                    }
                    EventResult {
                        handled: true,
                        ..EventResult::IGNORED
                    }
                }
                None => EventResult::IGNORED,
            },
            _ => EventResult::IGNORED,
        }
    }

    /// Advance animations. Returns `true` while anything is moving.
    pub fn tick(&mut self, dt: f32) -> bool {
        let mut animating = false;
        animating |= self.toasts.tick(dt);
        animating |= self.overlay_mgr.tick(dt);
        animating
    }

    // -- Rendering -------------------------------------------------------------

    fn ensure_layers(&mut self, c: &mut Compositor) -> Layers {
        *self.layers.get_or_insert_with(|| Layers {
            content: c.create_layer(10),
            overlay: c.create_layer(OverlayManager::BASE_Z),
            toast: c.create_layer(OverlayManager::BASE_Z + 200),
        })
    }

    pub fn render(&mut self, c: &mut Compositor) {
        c.begin_frame();
        let layers = self.ensure_layers(c);
        let theme = self.theme.clone();

        self.sync_sidebar_scroll();
        self.render_sidebar(c, &theme);
        self.render_header(c, &theme);

        // Keep the page scroll clamped to the current viewport and content
        // (a resize can shrink content; the offset must follow).
        self.sync_page_scroll();
        let content = self.page_rect();
        let viewport = self.content_rect();

        // Scrolled screen content clips to the viewport below the header.
        // PushClip rects are logical; the encoder scales them to physical.
        c.push(SceneNode::PushClip {
            x: SIDEBAR_W,
            y: viewport.y,
            w: self.width - SIDEBAR_W,
            h: viewport.h,
        });
        match self.route {
            Route::Home => self
                .home
                .render(c, layers.content, content, &theme, &mut self.art),
            Route::Decks | Route::Deck(_) => {
                self.decks
                    .render(c, layers.content, content, &theme, &mut self.art)
            }
            Route::Collection => {
                self.collection
                    .render(c, layers.content, content, &theme, &mut self.art)
            }
            Route::Wishlist => {
                self.wishlist
                    .render(c, layers.content, content, &theme, &mut self.art)
            }
            Route::Scanner => {
                self.scanner
                    .render(c, layers.content, content, &theme, &mut self.art)
            }
            Route::Games => self
                .games
                .render(c, layers.content, content, &theme, &mut self.art),
        }
        c.push(SceneNode::PopClip);

        self.toasts
            .render(c, layers.toast, &theme, self.width, self.height);
    }

    fn render_sidebar(&self, c: &mut Compositor, theme: &Theme) {
        let glass = &theme.glass;
        let text_c = theme.colors.text;

        // The rail: the raised opaque panel tone, one notch off the page.
        c.push(SceneNode::Rect {
            x: 0.0,
            y: 0.0,
            w: SIDEBAR_W,
            h: self.height,
            color: theme.colors.surface.0,
        });

        text(c, "Spellbook", 20.0, 600, 24.0, 26.0, text_c.0);
        text(
            c,
            "COLEÇÃO E DECKS DE COMMANDER",
            10.0,
            600,
            24.0,
            54.0,
            glass.text_placeholder.0,
        );

        let band = self.sidebar_viewport();
        c.push(SceneNode::PushClip {
            x: band.x,
            y: band.y,
            w: band.w,
            h: band.h,
        });
        for (i, (route, rect)) in Route::NAV.iter().zip(self.sidebar_item_rects()).enumerate() {
            // A deck screen keeps "Meus Decks" lit: you are still inside it.
            let active =
                *route == self.route || (route.in_decks_section() && self.route.in_decks_section());
            let hovered = self.sidebar_hover == Some(i);
            if active || hovered {
                c.push(rounded_rect(
                    rect.x,
                    rect.y,
                    rect.w,
                    rect.h,
                    theme.radius.md,
                    if active {
                        glass.surface_active.0
                    } else {
                        glass.surface_hover.0
                    },
                ));
            }
            if active {
                c.push(rounded_rect_stroke(
                    rect.x,
                    rect.y,
                    rect.w,
                    rect.h,
                    theme.radius.md,
                    glass.edge.0,
                    1.0,
                ));
            }
            let fg = if active {
                with_alpha(text_c.0, text_c.0[3] * 0.8)
            } else if hovered {
                with_alpha(text_c.0, text_c.0[3] * 0.59)
            } else {
                glass.text_faint.0
            };
            if let Some(node) = icons::icon_at(route.icon(), 18.0, fg, rect.x + 13.0, rect.y + 15.0)
            {
                c.push(node);
            }
            text(
                c,
                route.title(),
                14.0,
                600,
                rect.x + 44.0,
                rect.y + (rect.h - 14.0 * 1.4) / 2.0,
                fg,
            );
        }
        c.push(SceneNode::PopClip);

        // Footer: the offline badge the rail has always carried. The card
        // index panel lands here once the data commands are wired.
        let foot_y = self.height - SIDEBAR_FOOTER_H + 24.0;
        if let Some(node) =
            icons::icon_at("circle", 8.0, theme.colors.success.0, 24.0, foot_y + 4.0)
        {
            c.push(node);
        }
        text(
            c,
            "offline-first",
            11.0,
            500,
            40.0,
            foot_y,
            glass.text_placeholder.0,
        );
    }

    fn render_header(&self, c: &mut Compositor, theme: &Theme) {
        let x = SIDEBAR_W + PAD;
        text(
            c,
            self.route.title(),
            20.0,
            500,
            x,
            PAD,
            theme.colors.text.0,
        );
        text(
            c,
            self.route.blurb(),
            14.0,
            400,
            x,
            PAD + 32.0,
            theme.colors.text_dim.0,
        );
    }
}

// ---------------------------------------------------------------------------
// Shared drawing helpers for the screen modules
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

/// Uppercase group label - the HOFF accordion head.
pub(crate) fn group_label(c: &mut Compositor, s: &str, x: f32, y: f32, theme: &Theme) {
    text(c, s, 12.0, 600, x, y, theme.glass.text_placeholder.0);
}

/// Soft panel container - the HOFF list card.
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

/// Column count for a card grid, and the width each column stretches to.
///
/// The one grid rule in the app, in one place: as many columns as the content
/// affords at `min_w` each, then stretched to fill the row and clamped by a
/// readability maximum. Every screen that lays out tiles uses this, so none of
/// them can quietly grow a breakpoint constant.
pub(crate) fn grid_columns(content_w: f32, min_w: f32, max_w: f32, gap: f32) -> (usize, f32) {
    let cols = (((content_w + gap) / (min_w + gap)).floor() as usize).max(1);
    let col_w = ((content_w - (cols as f32 - 1.0) * gap) / cols as f32).min(max_w);
    (cols, col_w)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Screens need a command channel to exist; in tests it goes nowhere.
    fn test_view(w: f32, h: f32) -> SpellbookView {
        let (tx, _rx) = std::sync::mpsc::channel();
        SpellbookView::new(w, h, tx)
    }

    #[test]
    fn starts_on_home() {
        let view = test_view(1400.0, 900.0);
        assert_eq!(view.route, Route::Home);
    }

    /// A deck screen is still the decks section: the rail must keep that link
    /// lit and the deck sub-list open, the way the hash router did for
    /// `#deck/12`.
    #[test]
    fn a_deck_screen_counts_as_the_decks_section() {
        assert!(Route::Deck(12).in_decks_section());
        assert!(Route::Decks.in_decks_section());
        assert!(!Route::Collection.in_decks_section());
        assert_eq!(Route::Deck(12).scroll_slot(), Route::Decks.scroll_slot());
    }

    /// The content rect follows the window instead of stopping at a page
    /// width. The CSS this replaces capped at 1400px, which left most of a
    /// wide display empty.
    #[test]
    fn content_fills_any_window_width() {
        let narrow = test_view(1000.0, 800.0).content_rect();
        let wide = test_view(2560.0, 1400.0).content_rect();
        assert!((narrow.w - (1000.0 - SIDEBAR_W - 80.0)).abs() < 0.5);
        assert!((wide.w - (2560.0 - SIDEBAR_W - 80.0)).abs() < 0.5);
        assert!(wide.w > narrow.w * 2.0);
    }

    /// Even at a window narrower than the rail plus padding, the content rect
    /// stays positive - a negative width would make every grid below it
    /// compute a negative column count.
    #[test]
    fn content_rect_never_collapses_on_a_tiny_window() {
        let view = test_view(200.0, 200.0);
        let rect = view.content_rect();
        assert!(rect.w >= 200.0, "got {}", rect.w);
        assert!(rect.h >= 120.0, "got {}", rect.h);
    }

    /// The grid rule: columns come from the space available, stretch to fill
    /// it, and stop at the readability maximum.
    #[test]
    fn grid_columns_derive_from_available_space() {
        let (cols, w) = grid_columns(1272.0, 320.0, 520.0, 16.0);
        assert!(cols >= 3, "wide content must give 3+ columns, got {cols}");
        assert!(w > 320.0 && w <= 520.0, "columns must stretch, got {w}");

        let (cols, w) = grid_columns(400.0, 320.0, 520.0, 16.0);
        assert_eq!(cols, 1);
        assert!((w - 400.0).abs() < 0.5, "one column fills the row, got {w}");

        // Never zero columns, however narrow.
        let (cols, _) = grid_columns(50.0, 320.0, 520.0, 16.0);
        assert_eq!(cols, 1);
    }
}
