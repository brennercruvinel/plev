//! Toast page: one button per intent; a click queues a toast (the view
//! owns the toast manager). Projected (task-75 phase 2b): each button,
//! `toast_page.button[<intent>]`, whose `activate` asks the view for the
//! toast a click queues.

use accord::{Request, SemId, SnapshotBuilder};
use engine::drawing::compositor::Compositor;
use engine::styling::theme::{Intent, Theme};
use engine::widgets::prelude::{Button, EventResult, Rect, WidgetEvent};

use super::{After, Component, child_id, flow, root_id, under};

pub struct ToastPage {
    root: Option<SemId>,
    buttons: Vec<(Intent, Button)>,
    /// The button a screen reader focused (its ring shows); Escape or a
    /// click lets it go.
    focus: Option<usize>,
}

impl ToastPage {
    pub fn new() -> Self {
        let root = root_id("toast_page");
        let family = child_id(root.as_ref(), "button");
        let button = |key: &str, label: &str, intent: Intent| {
            let b = Button::new(label).intent(intent);
            let b = match &family {
                Some(f) => b.sem(f.key(key)),
                None => b,
            };
            (intent, b)
        };
        Self {
            buttons: vec![
                button("neutral", "Neutral", Intent::Neutral),
                button("success", "Success", Intent::Constructive),
                button("error", "Error", Intent::Destructive),
                button("info", "Info", Intent::Informational),
            ],
            root,
            focus: None,
        }
    }

    /// The button behind `id`.
    fn at(&self, id: &SemId) -> Option<usize> {
        self.buttons
            .iter()
            .position(|(_, b)| b.sem_id() == Some(id))
    }

    /// The reader's focus on the button at `at`, or on none.
    fn set_focus(&mut self, at: Option<usize>) {
        self.focus = at;
        for (i, (_, b)) in self.buttons.iter_mut().enumerate() {
            b.set_focused(at == Some(i));
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
        if matches!(event, WidgetEvent::MouseDown { .. }) && self.blur() {
            result = EventResult::changed();
        }
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

impl Component for ToastPage {
    fn root(&self) -> Option<&SemId> {
        self.root.as_ref()
    }

    fn project(&self, b: &mut SnapshotBuilder, root: &SemId) {
        for (_, button) in &self.buttons {
            if let Some(id) = button.sem_id() {
                button.project(b, id, Some(root));
            }
        }
        let focused = self.focus.and_then(|i| self.buttons.get(i));
        if let Some(id) = focused.and_then(|(_, button)| button.sem_id()) {
            b.focus(id);
        }
    }

    /// A button's `activate` queues its toast, as its click does.
    fn run(&mut self, req: &Request) -> Result<After, String> {
        match (self.at(&req.target), req.action.as_str()) {
            (Some(i), "activate") => Ok(After::Toast(self.buttons[i].0)),
            _ => Err(format!(
                "the toast page has no {} on {}",
                req.action, req.target
            )),
        }
    }

    fn focus(&mut self, id: &SemId) -> bool {
        if !self.root.as_ref().is_some_and(|root| under(root, id)) {
            return false;
        }
        self.set_focus(self.at(id));
        true
    }

    fn blur(&mut self) -> bool {
        let had = self.focus.is_some();
        self.set_focus(None);
        had
    }
}
