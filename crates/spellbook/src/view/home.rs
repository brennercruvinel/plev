//! Dashboard: collection totals, the deck grid and the activity history.
//!
//! This is the reference screen for the app's data pattern:
//!
//! - `on_enter` queues one command (`LoadHome`); the screen draws its empty
//!   state until the answer arrives.
//! - `on_event` picks out `HomeLoaded`, stores the payload and reports
//!   `changed` so the shell invalidates. Errors become a toast, never a
//!   broken screen.
//! - Hit-testing runs against the same pure layout functions `render` uses,
//!   so the rect a click lands in is the rect that was drawn - there is no
//!   second, drifting copy of the geometry.

use engine::compositor::{Compositor, LayerId};
use engine::theme::{Intent, Theme};
use engine::ui::widgets::{EmptyState, EventResult, Rect, WidgetEvent, rounded_rect};
use spellbook_core::client::{Command, Event, HomeData};

use super::{EditKey, Route, ScreenCtx, deck_tile, grid_columns, group_label, panel, text};
use crate::art::ArtCache;

const STAT_H: f32 = 92.0;
const STAT_GAP: f32 = 12.0;
const DECK_GAP: f32 = 16.0;
const ACTIVITY_ROW_H: f32 = 34.0;

/// What the pointer is over. Rects come from the layout functions below, so
/// hover and click never disagree with the pixels.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Hit {
    /// The five stat cards, in layout order. Each navigates somewhere.
    Stat(usize),
    Deck(i64),
    NewDeckTile,
}

pub struct HomeScreen {
    data: Option<Box<HomeData>>,
    hover: Option<Hit>,
    loading: EmptyState,
}

impl HomeScreen {
    pub fn new() -> Self {
        Self {
            data: None,
            hover: None,
            loading: EmptyState::new(
                "Carregando o laboratório",
                "Lendo a coleção, os decks e o histórico do banco local.",
            )
            .icon("house"),
        }
    }

    pub fn on_enter(&mut self, _route: Route, ctx: &mut ScreenCtx) {
        ctx.send(Command::LoadHome);
    }

    pub fn on_event(&mut self, event: &Event, ctx: &mut ScreenCtx) -> bool {
        let Event::HomeLoaded(result) = event else {
            return false;
        };
        match result {
            Ok(data) => {
                // The shell re-enters the route on navigation, which reloads;
                // here we only store what arrived.
                self.data = Some(data.clone());
                true
            }
            Err(e) => {
                ctx.toast(e.detail().to_string(), Intent::Destructive);
                true
            }
        }
    }

    /// Nothing on the dashboard takes text yet - the search fields live on
    /// the data screens.
    pub fn handle_text(&mut self, _s: &str, _ctx: &mut ScreenCtx) -> bool {
        false
    }

    pub fn handle_edit_key(&mut self, _key: EditKey, _ctx: &mut ScreenCtx) -> bool {
        false
    }

    pub fn handle_escape(&mut self) -> bool {
        false
    }

    pub fn tick(&mut self, _dt: f32) -> bool {
        false
    }

    // -- Layout ---------------------------------------------------------------

    /// The five stat cards across the top, navigable like the old
    /// `data-stat-nav` tiles.
    fn stat_rects(&self, content: Rect) -> Vec<Rect> {
        let (cols, col_w) = grid_columns(content.w, 170.0, 280.0, STAT_GAP);
        (0..5)
            .map(|i| {
                let (row, col) = (i / cols, i % cols);
                Rect::new(
                    content.x + col as f32 * (col_w + STAT_GAP),
                    content.y + row as f32 * (STAT_H + STAT_GAP),
                    col_w,
                    STAT_H,
                )
            })
            .collect()
    }

    fn stat_rows(&self, content: Rect) -> usize {
        let (cols, _) = grid_columns(content.w, 170.0, 280.0, STAT_GAP);
        5usize.div_ceil(cols)
    }

    /// Y where the deck section starts.
    fn decks_y(&self, content: Rect) -> f32 {
        content.y + self.stat_rows(content) as f32 * (STAT_H + STAT_GAP) + 40.0
    }

    /// One rect per deck tile, plus the "Novo Deck" tile at the end.
    fn deck_rects(&self, content: Rect) -> Vec<(Hit, Rect)> {
        let Some(data) = &self.data else {
            return Vec::new();
        };
        let (cols, col_w) = grid_columns(content.w, 240.0, 340.0, DECK_GAP);
        let tile_h = deck_tile::tile_height(col_w);
        let y0 = self.decks_y(content) + 26.0;
        let mut out: Vec<(Hit, Rect)> = data
            .decks
            .iter()
            .enumerate()
            .map(|(i, d)| {
                let (row, col) = (i / cols, i % cols);
                (
                    Hit::Deck(d.id),
                    Rect::new(
                        content.x + col as f32 * (col_w + DECK_GAP),
                        y0 + row as f32 * (tile_h + DECK_GAP),
                        col_w,
                        tile_h,
                    ),
                )
            })
            .collect();
        let i = data.decks.len();
        let (row, col) = (i / cols, i % cols);
        out.push((
            Hit::NewDeckTile,
            Rect::new(
                content.x + col as f32 * (col_w + DECK_GAP),
                y0 + row as f32 * (tile_h + DECK_GAP),
                col_w,
                tile_h,
            ),
        ));
        out
    }

    fn activity_y(&self, content: Rect) -> f32 {
        let Some(data) = &self.data else {
            return self.decks_y(content) + 26.0;
        };
        let (cols, col_w) = grid_columns(content.w, 240.0, 340.0, DECK_GAP);
        let tile_h = deck_tile::tile_height(col_w);
        let tiles = data.decks.len() + 1;
        let rows = tiles.div_ceil(cols);
        self.decks_y(content) + 26.0 + rows as f32 * (tile_h + DECK_GAP) + 16.0
    }

    fn activity_rects(&self, content: Rect) -> Vec<Rect> {
        let Some(data) = &self.data else {
            return Vec::new();
        };
        let y0 = self.activity_y(content) + 26.0;
        data.activity
            .iter()
            .enumerate()
            .map(|(i, _)| {
                Rect::new(
                    content.x,
                    y0 + i as f32 * (ACTIVITY_ROW_H + 4.0),
                    content.w,
                    ACTIVITY_ROW_H,
                )
            })
            .collect()
    }

    /// Whether a modal or menu is open over the page. While `true` the shell
    /// routes pointer events here first via `handle_overlay_event`.
    pub fn overlay_open(&self) -> bool {
        false
    }

    /// Pointer event while an overlay is open.
    pub fn handle_overlay_event(
        &mut self,
        _event: &WidgetEvent,
        _ctx: &mut ScreenCtx,
    ) -> EventResult {
        EventResult::IGNORED
    }

    /// Modals and menus, drawn over the whole window after the content clip.
    pub fn render_overlay(
        &mut self,
        _c: &mut Compositor,
        _layer: LayerId,
        _window: Rect,
        _theme: &Theme,
        _art: &mut ArtCache,
    ) {
    }

    pub fn content_height(&self, content: Rect) -> f32 {
        let Some(data) = &self.data else {
            return content.h;
        };
        let bottom = self.activity_y(content)
            + 26.0
            + data.activity.len().max(1) as f32 * (ACTIVITY_ROW_H + 4.0);
        (bottom - content.y).max(content.h)
    }

    fn hit_at(&self, x: f32, y: f32, content: Rect) -> Option<Hit> {
        for (i, rect) in self.stat_rects(content).iter().enumerate() {
            if rect.contains(x, y) {
                return Some(Hit::Stat(i));
            }
        }
        for (hit, rect) in self.deck_rects(content) {
            if rect.contains(x, y) {
                return Some(hit);
            }
        }
        None
    }

    // -- Events ---------------------------------------------------------------

    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        content: Rect,
        ctx: &mut ScreenCtx,
    ) -> EventResult {
        if self.data.is_none() {
            return self.loading.handle_event(event, content);
        }
        match *event {
            WidgetEvent::MouseMove { x, y } => {
                let hover = self.hit_at(x, y, content);
                if hover != self.hover {
                    self.hover = hover;
                    EventResult::changed()
                } else {
                    EventResult::IGNORED
                }
            }
            WidgetEvent::MouseDown { x, y } => match self.hit_at(x, y, content) {
                Some(Hit::Stat(i)) => {
                    // The old data-stat-nav mapping: decks, collection,
                    // collection, wishlist, games.
                    let route = [
                        Route::Decks,
                        Route::Collection,
                        Route::Collection,
                        Route::Wishlist,
                        Route::Games,
                    ][i];
                    ctx.navigate(route);
                    EventResult::clicked()
                }
                Some(Hit::Deck(id)) => {
                    ctx.navigate(Route::Deck(id));
                    EventResult::clicked()
                }
                Some(Hit::NewDeckTile) => {
                    ctx.navigate(Route::Decks);
                    EventResult::clicked()
                }
                None => EventResult::IGNORED,
            },
            _ => EventResult::IGNORED,
        }
    }

    // -- Render ---------------------------------------------------------------

    pub fn render(
        &mut self,
        c: &mut Compositor,
        _layer: LayerId,
        content: Rect,
        theme: &Theme,
        art: &mut ArtCache,
    ) {
        let Some(data) = &self.data else {
            self.loading.render(c, content, theme);
            return;
        };

        // Stat cards.
        let total_cards: i64 = data.decks.iter().map(|d| d.total_cards).sum();
        let total_games: i64 = data.decks.iter().map(|d| d.wins + d.losses).sum();
        let total_wins: i64 = data.decks.iter().map(|d| d.wins).sum();
        let stats: [(String, String, String); 5] = [
            (
                "DECKS MONTADOS".into(),
                format!("{}", data.decks.len()),
                format!("{total_cards} cartas ao todo"),
            ),
            (
                "CARTAS NA COLEÇÃO".into(),
                format!("{}", data.totals.total_units),
                format!(
                    "{} nomes distintos, com repetidas",
                    data.totals.distinct_cards
                ),
            ),
            (
                "CARTAS LIVRES".into(),
                format!("{}", data.totals.free_units),
                format!("{} em decks", data.totals.allocated_units),
            ),
            (
                "WISHLIST".into(),
                format!("{}", data.wishlist.total_units),
                format!(
                    "{} nomes · ~${:.2}",
                    data.wishlist.distinct_cards, data.wishlist.price_usd
                ),
            ),
            (
                "PARTIDAS REGISTRADAS".into(),
                format!("{total_games}"),
                format!("{total_wins} vitórias"),
            ),
        ];
        for (i, rect) in self.stat_rects(content).iter().enumerate() {
            let hovered = self.hover == Some(Hit::Stat(i));
            panel(c, *rect, theme);
            if hovered {
                c.push(rounded_rect(
                    rect.x,
                    rect.y,
                    rect.w,
                    rect.h,
                    theme.radius.lg,
                    theme.glass.surface_hover.0,
                ));
            }
            let (label, value, sub) = &stats[i];
            text(
                c,
                label,
                10.0,
                600,
                rect.x + 14.0,
                rect.y + 14.0,
                theme.glass.text_placeholder.0,
            );
            text(
                c,
                value,
                26.0,
                600,
                rect.x + 14.0,
                rect.y + 30.0,
                theme.colors.text.0,
            );
            text(
                c,
                sub,
                11.0,
                400,
                rect.x + 14.0,
                rect.y + 68.0,
                theme.colors.text_dim.0,
            );
        }

        // Decks.
        group_label(c, "SEUS DECKS", content.x, self.decks_y(content), theme);
        for (hit, rect) in self.deck_rects(content) {
            match hit {
                Hit::Deck(id) => {
                    let deck = data
                        .decks
                        .iter()
                        .find(|d| d.id == id)
                        .expect("layout match");
                    deck_tile::render(c, rect, deck, self.hover == Some(hit), art, theme);
                }
                Hit::NewDeckTile => {
                    let hovered = self.hover == Some(Hit::NewDeckTile);
                    c.push(rounded_rect(
                        rect.x,
                        rect.y,
                        rect.w,
                        rect.h,
                        theme.radius.lg,
                        if hovered {
                            theme.glass.surface_hover.0
                        } else {
                            theme.glass.surface.0
                        },
                    ));
                    text(
                        c,
                        "+",
                        28.0,
                        300,
                        rect.x + rect.w / 2.0 - 8.0,
                        rect.y + rect.h / 2.0 - 34.0,
                        theme.colors.text_dim.0,
                    );
                    text(
                        c,
                        "Novo Deck",
                        13.0,
                        600,
                        rect.x + rect.w / 2.0 - 30.0,
                        rect.y + rect.h / 2.0 + 2.0,
                        theme.colors.text_dim.0,
                    );
                }
                Hit::Stat(_) => unreachable!("deck layout never yields stat hits"),
            }
        }

        // Activity.
        group_label(
            c,
            "HISTÓRICO DE ATIVIDADES",
            content.x,
            self.activity_y(content),
            theme,
        );
        let rows = self.activity_rects(content);
        if rows.is_empty() {
            text(
                c,
                "Nenhuma atividade ainda - mexa num deck ou na coleção pra ver o histórico aqui.",
                12.0,
                400,
                content.x,
                self.activity_y(content) + 26.0,
                theme.colors.text_dim.0,
            );
        }
        for (rect, row) in rows.iter().zip(data.activity.iter()) {
            c.push(rounded_rect(
                rect.x,
                rect.y,
                rect.w,
                rect.h,
                theme.radius.md,
                theme.glass.surface.0,
            ));
            text(
                c,
                &row.description,
                12.0,
                500,
                rect.x + 12.0,
                rect.y + 9.0,
                theme.colors.text.0,
            );
            let ts = format_ts(&row.ts);
            text(
                c,
                &ts,
                11.0,
                400,
                rect.x + rect.w - ts.len() as f32 * 6.2 - 12.0,
                rect.y + 10.0,
                theme.colors.text_dim.0,
            );
        }
    }
}

/// "2026-08-30 16:04:11" (UTC, SQLite) -> "30/08 16:04", like formatTs did.
fn format_ts(ts: &str) -> String {
    let (date, time) = ts.split_once(' ').unwrap_or((ts, ""));
    let mut parts = date.split('-');
    let (Some(_y), Some(m), Some(d)) = (parts.next(), parts.next(), parts.next()) else {
        return ts.to_string();
    };
    let hm: String = time.chars().take(5).collect();
    format!("{d}/{m} {hm}")
}
