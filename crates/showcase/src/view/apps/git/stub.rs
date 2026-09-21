//! The git section where no git worker can run (web, mobile): one line
//! saying so, same contract as the desktop section.

use comps::prelude::{EventResult, Rect, WidgetEvent};
use engine::compositor::Compositor;
use engine::theme::Theme;

use super::super::text;
use super::{NavKey, Wake};

pub struct GitPage;

impl GitPage {
    pub fn new() -> Self {
        Self
    }

    pub fn connect(&mut self, _wake: Wake) {}

    pub fn set_theme(&mut self, _name: &str) {}

    pub fn content_height(&self, content: Rect) -> f32 {
        content.h
    }

    pub fn handle_event(&mut self, _event: &WidgetEvent, _content: Rect) -> EventResult {
        EventResult::IGNORED
    }

    pub fn handle_right_click(&mut self, _x: f32, _y: f32, _content: Rect) -> bool {
        false
    }

    pub fn handle_text(&mut self, _key: &str) -> bool {
        false
    }

    pub fn handle_nav(&mut self, _key: NavKey) -> bool {
        false
    }

    pub fn handle_backspace(&mut self) -> bool {
        false
    }

    pub fn handle_enter(&mut self) -> bool {
        false
    }

    pub fn handle_escape(&mut self) -> bool {
        false
    }

    pub fn tick(&mut self, _dt: f32) -> bool {
        false
    }

    pub fn render(&mut self, c: &mut Compositor, content: Rect, theme: &Theme) {
        text(
            c,
            "The git workspace needs the git CLI and a worker thread: run the showcase on desktop.",
            14.0,
            400,
            content.x,
            content.y + theme.spacing.md,
            theme.glass.text_placeholder.0,
        );
    }
}
