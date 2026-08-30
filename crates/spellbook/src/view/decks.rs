//! Decks: the grid, and the deck detail reached from it.
//!
//! Placeholder: answers the six calls the shell makes (`new`, `on_enter`,
//! `on_event`, `content_height`, `handle_event`, `render`). The data
//! commands and the real layout land in the commit that ports this screen.

use engine::compositor::{Compositor, LayerId};
use engine::theme::Theme;
use engine::ui::widgets::{EmptyState, EventResult, Rect, WidgetEvent};
use spellbook_core::client::Event;

use super::{EditKey, Route, ScreenCtx};
use crate::art::ArtCache;

pub struct DecksScreen {
    empty: EmptyState,
}

impl DecksScreen {
    pub fn new() -> Self {
        Self {
            empty: EmptyState::new(
                "Nenhum deck ainda",
                "Monte seu primeiro deck, ou deixe o autobuild montar um por você.",
            )
            .icon("layers"),
        }
    }

    /// The route became current: queue the loads the screen needs.
    pub fn on_enter(&mut self, _route: Route, _ctx: &mut ScreenCtx) {}

    /// A worker answer arrived. Returns `true` when the frame changed.
    pub fn on_event(&mut self, _event: &Event, _ctx: &mut ScreenCtx) -> bool {
        false
    }

    /// Type characters into the focused field. `false` when nothing is.
    pub fn handle_text(&mut self, _s: &str, _ctx: &mut ScreenCtx) -> bool {
        false
    }

    /// A non-character editing key for the focused field.
    pub fn handle_edit_key(&mut self, _key: EditKey, _ctx: &mut ScreenCtx) -> bool {
        false
    }

    /// Escape blurs or closes whatever is open. `false` when nothing was.
    pub fn handle_escape(&mut self) -> bool {
        false
    }

    /// Cursor blink and friends. `true` while frames are needed.
    pub fn tick(&mut self, _dt: f32) -> bool {
        false
    }

    /// Natural height of the laid-out screen, for the page scrollbar. The
    /// empty state centres in the viewport, so it never overflows.
    pub fn content_height(&self, content: Rect) -> f32 {
        content.h
    }

    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        content: Rect,
        _ctx: &mut ScreenCtx,
    ) -> EventResult {
        self.empty.handle_event(event, content)
    }

    pub fn render(
        &mut self,
        c: &mut Compositor,
        _layer: LayerId,
        content: Rect,
        theme: &Theme,
        _art: &mut ArtCache,
    ) {
        self.empty.render(c, content, theme);
    }
}
