//! Scanner: card recognition through the camera.
//!
//! The one feature that was never built. It was a placeholder in the web UI
//! too, and it stays one here rather than being quietly dropped in the
//! rewrite - the route exists, says what it is waiting for, and points at the
//! two things that already do the job by hand.

use engine::compositor::{Compositor, LayerId};
use engine::theme::Theme;
use engine::ui::widgets::{EmptyState, EventResult, Rect, WidgetEvent};
use spellbook_core::client::Event;

use super::{Route, ScreenCtx};
use crate::art::ArtCache;

pub struct ScannerScreen {
    empty: EmptyState,
}

impl ScannerScreen {
    pub fn new() -> Self {
        Self {
            empty: EmptyState::new(
                "Em breve",
                "Enquanto isso, use Adicionar Carta (com o modo por lista) ou \
                 Importar decklist na tela do deck.",
            )
            .icon("eye"),
        }
    }

    pub fn on_enter(&mut self, _route: Route, _ctx: &mut ScreenCtx) {}

    pub fn on_event(&mut self, _event: &Event, _ctx: &mut ScreenCtx) -> bool {
        false
    }

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
