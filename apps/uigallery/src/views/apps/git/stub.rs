//! The git section where no git worker can run (web, mobile): the empty
//! state saying so, centered in the app's rect, same contract as the
//! desktop section.

use engine::drawing::compositor::Compositor;
use engine::styling::theme::Theme;
use engine::widgets::content::EmptyState;
use engine::widgets::prelude::{EventResult, Rect, Wake, WidgetEvent};

use super::{GitWake, NavKey};
use crate::views::text_input::clipboard::Keys;

pub struct GitPage;

impl GitPage {
    pub fn new() -> Self {
        Self
    }

    pub fn connect(&mut self, _wake: GitWake) {}

    pub fn set_theme(&mut self, _theme: &Theme) {}

    pub fn content_height(&self, content: Rect) -> f32 {
        content.h
    }

    pub fn handle_event(&mut self, _event: &WidgetEvent, _content: Rect) -> EventResult {
        EventResult::IGNORED
    }

    /// No field: nobody has the keys.
    pub(in crate::views) fn keys(&mut self) -> Keys<'_> {
        Keys::Nobody
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

    pub fn apply_events(&mut self) -> bool {
        false
    }

    pub fn wake(&mut self, _dt: f32) -> Wake {
        Wake::idle()
    }

    pub fn render(&mut self, c: &mut Compositor, content: Rect, theme: &Theme) {
        Self::empty().render(c, content, theme);
    }

    /// What the page says where it cannot run.
    fn empty() -> EmptyState {
        EmptyState::new(
            "The git workspace runs on desktop",
            "It needs the git CLI and a worker thread, which this target has not: open the showcase on desktop.",
        )
        .icon("git-branch")
    }
}
