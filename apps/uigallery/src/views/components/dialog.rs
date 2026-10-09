//! Dialog page: two buttons open a plain and a destructive modal. The
//! view owns the overlay stack; the page only reports which to open.
//! Projected (task-75 phase 2b): `dialog_page.open` and
//! `dialog_page.delete`, whose `activate` asks the view for the modal a
//! click opens (the modal itself is the view's, `dispatch::overlay`).

use accord::{Request, SemId, SnapshotBuilder};
use engine::drawing::compositor::Compositor;
use engine::styling::theme::Theme;
use engine::widgets::prelude::{Button, ButtonVariant, EventResult, Rect, WidgetEvent};

use super::{After, Component, child_id, flow, root_id, under};

pub struct DialogPage {
    root: Option<SemId>,
    open: Button,
    delete: Button,
    /// The button a screen reader focused (its ring shows): the
    /// destructive one when `true`. Escape or a click lets it go.
    focus: Option<bool>,
}

impl DialogPage {
    pub fn new() -> Self {
        let root = root_id("dialog_page");
        let open = Button::new("Open dialog").variant(ButtonVariant::Outline);
        let delete = Button::new("Delete repository…")
            .variant(ButtonVariant::Warning)
            .icon("trash");
        let open = match child_id(root.as_ref(), "open") {
            Some(id) => open.sem(id),
            None => open,
        };
        let delete = match child_id(root.as_ref(), "delete") {
            Some(id) => delete.sem(id),
            None => delete,
        };
        Self {
            root,
            open,
            delete,
            focus: None,
        }
    }

    fn layout(&self, content: Rect, theme: &Theme) -> (Vec<Rect>, f32) {
        flow(
            content,
            content.y,
            theme.spacing.md,
            [&self.open, &self.delete]
                .into_iter()
                .map(|b| b.preferred_size(theme)),
        )
    }

    pub fn content_height(&self, content: Rect, theme: &Theme) -> f32 {
        self.layout(content, theme).1
    }

    /// Which button `id` is: `Some(destructive)`.
    fn which(&self, id: &SemId) -> Option<bool> {
        if self.open.sem_id() == Some(id) {
            Some(false)
        } else if self.delete.sem_id() == Some(id) {
            Some(true)
        } else {
            None
        }
    }

    /// The reader's focus on one button, or on none.
    fn set_focus(&mut self, on: Option<bool>) {
        self.focus = on;
        self.open.set_focused(on == Some(false));
        self.delete.set_focused(on == Some(true));
    }

    /// Route an event; the second value is `Some(destructive)` when a
    /// button asked for its dialog.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        content: Rect,
        theme: &Theme,
    ) -> (EventResult, Option<bool>) {
        let blurred = matches!(event, WidgetEvent::MouseDown { .. }) && self.blur();
        let (rects, _) = self.layout(content, theme);
        let mut action = None;
        let r = self.open.handle_event(event, rects[0]);
        if r.clicked {
            action = Some(false);
        }
        let d = self.delete.handle_event(event, rects[1]);
        if d.clicked {
            action = Some(true);
        }
        let mut result = r.merge(d);
        if blurred {
            result = result.merge(EventResult::changed());
        }
        (result, action)
    }

    pub fn render(&self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let (rects, _) = self.layout(content, theme);
        self.open.render(c, rects[0], theme);
        self.delete.render(c, rects[1], theme);
    }
}

impl Component for DialogPage {
    fn root(&self) -> Option<&SemId> {
        self.root.as_ref()
    }

    fn project(&self, b: &mut SnapshotBuilder, root: &SemId) {
        for button in [&self.open, &self.delete] {
            if let Some(id) = button.sem_id() {
                button.project(b, id, Some(root));
            }
        }
        let focused = match self.focus {
            Some(true) => self.delete.sem_id(),
            Some(false) => self.open.sem_id(),
            None => None,
        };
        if let Some(id) = focused {
            b.focus(id);
        }
    }

    /// A button's `activate` opens its modal, as its click does.
    fn run(&mut self, req: &Request) -> Result<After, String> {
        match (self.which(&req.target), req.action.as_str()) {
            (Some(destructive), "activate") => Ok(After::Modal { destructive }),
            _ => Err(format!(
                "the dialog page has no {} on {}",
                req.action, req.target
            )),
        }
    }

    fn focus(&mut self, id: &SemId) -> bool {
        if !self.root.as_ref().is_some_and(|root| under(root, id)) {
            return false;
        }
        self.set_focus(self.which(id));
        true
    }

    fn blur(&mut self) -> bool {
        let had = self.focus.is_some();
        self.set_focus(None);
        had
    }
}
