//! Toast page: one button per intent; a click queues a toast (the view
//! owns the toast manager).

use comps::prelude::{Button, EventResult, Rect, WidgetEvent};
use engine::compositor::Compositor;
use engine::theme::{Intent, Theme};

use super::flow;

pub struct ToastPage {
    buttons: Vec<(Intent, Button)>,
}

impl ToastPage {
    pub fn new() -> Self {
        Self {
            buttons: vec![
                (Intent::Neutral, Button::new("Neutral")),
                (
                    Intent::Constructive,
                    Button::new("Success").intent(Intent::Constructive),
                ),
                (
                    Intent::Destructive,
                    Button::new("Error").intent(Intent::Destructive),
                ),
                (
                    Intent::Informational,
                    Button::new("Info").intent(Intent::Informational),
                ),
            ],
        }
    }

    /// The message a toast of `intent` carries.
    pub fn message(intent: Intent) -> &'static str {
        match intent {
            Intent::Neutral => "Neutral toast: plain information.",
            Intent::Constructive => "Saved! Everything went fine.",
            Intent::Destructive => "Failed to push: remote rejected.",
            Intent::Informational => "3 new commits fetched from origin.",
        }
    }

    fn layout(&self, content: Rect, theme: &Theme) -> (Vec<Rect>, f32) {
        flow(
            content,
            content.y,
            theme.spacing.md,
            self.buttons.iter().map(|(_, b)| b.preferred_size(theme)),
        )
    }

    pub fn content_height(&self, content: Rect, theme: &Theme) -> f32 {
        self.layout(content, theme).1
    }

    /// Route an event; the second value is the intent of a clicked button.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        content: Rect,
        theme: &Theme,
    ) -> (EventResult, Option<Intent>) {
        let (rects, _) = self.layout(content, theme);
        let mut result = EventResult::IGNORED;
        let mut queued = None;
        for ((intent, button), rect) in self.buttons.iter_mut().zip(rects) {
            let r = button.handle_event(event, rect);
            if r.clicked {
                queued = Some(*intent);
            }
            result = result.merge(r);
        }
        (result, queued)
    }

    pub fn render(&self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let (rects, _) = self.layout(content, theme);
        for ((_, button), rect) in self.buttons.iter().zip(rects) {
            button.render(c, rect, theme);
        }
    }
}
