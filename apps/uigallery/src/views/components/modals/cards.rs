//! The two cards of the Modals page that are not a modal of
//! `engine::widgets::overlay::modals`: the delete confirm, which is the
//! confirmation `Modal` with the kit's art (the Alert Dialog page's), and
//! the quick comment, which is front C's.

use accord::{Role, SemId, SnapshotBuilder, Value};
use engine::drawing::compositor::{Compositor, LayerId};
use engine::styling::theme::Theme;
use engine::widgets::feedback::{Modal, ModalAction};
use engine::widgets::overlay::modals::{self, ModalBody};
use engine::widgets::prelude::{EventResult, Rect, WidgetEvent};

use super::super::dialog;

/// The kit's `DeleteFile` as the ModalPage shows it: the destructive
/// question with Blinky over it, its two answers.
pub(super) struct Confirm {
    modal: Modal,
    id: Option<SemId>,
}

impl Confirm {
    pub(super) fn new(id: Option<&SemId>) -> Self {
        let mut modal = dialog::destructive();
        if let Some(id) = id {
            modal = modal.sem(id.clone());
        }
        Self {
            modal,
            id: id.cloned(),
        }
    }

    /// The sheet's rect at `rect`, which holds the art over it.
    fn sheet(&self, rect: Rect, theme: &Theme) -> Rect {
        let mut d = self.modal.dialog_rect_in(theme, rect);
        d.y = rect.y + self.modal.overhang(theme, d.w);
        d
    }

    fn answer(&self, id: &SemId) -> ModalAction {
        let Some(own) = &self.id else {
            return ModalAction::None;
        };
        if *id == own.key("confirm") {
            ModalAction::Confirm
        } else if *id == own.key("cancel") {
            ModalAction::Cancel
        } else {
            ModalAction::None
        }
    }
}

impl ModalBody for Confirm {
    fn title(&self) -> &str {
        &self.modal.title
    }

    /// As wide as a window of `w` lets the sheet be, as tall as the sheet
    /// and the art that rises above it.
    fn size(&self, w: f32, theme: &Theme) -> (f32, f32) {
        let d = self
            .modal
            .dialog_rect_in(theme, Rect::new(0.0, 0.0, w, f32::MAX / 4.0));
        (d.w, d.h + self.modal.overhang(theme, d.w))
    }

    /// The card shows the question as a state, as the Alert Dialog page
    /// does: the pointer leaves it, the keyboard focus reaches its answers.
    fn handle_event(&mut self, _event: &WidgetEvent, _rect: Rect, _theme: &Theme) -> EventResult {
        EventResult::IGNORED
    }

    fn render(&self, c: &mut Compositor, rect: Rect, theme: &Theme) {
        self.modal
            .render_in(c, LayerId::DEFAULT, theme, self.sheet(rect, theme));
    }

    fn project(&self, b: &mut SnapshotBuilder, id: &SemId, parent: &SemId) {
        b.node(id, Role::Group, self.modal.title.as_str())
            .parent(parent)
            .description(self.modal.body.as_str());
        for (key, label) in [("confirm", "Yes, delete it"), ("cancel", "Cancel")] {
            let button = id.key(key);
            b.node(&button, Role::Button, label)
                .parent(id)
                .action("activate");
            let which = self.answer(&button);
            if which != ModalAction::None && self.modal.focused() == which {
                b.focus(&button);
            }
        }
    }

    fn run(
        &mut self,
        target: &SemId,
        action: &str,
        _param: Option<&Value>,
    ) -> Option<Result<bool, String>> {
        match self.answer(target) {
            ModalAction::None => None,
            _ if action == "activate" => Some(Ok(false)),
            _ => Some(Err(format!("{target} has no {action}"))),
        }
    }

    fn focus(&mut self, id: Option<&SemId>) -> bool {
        let which = id.map_or(ModalAction::None, |id| self.answer(id));
        self.modal.set_focused(which);
        which != ModalAction::None
    }
}

/// The kit's `QuickComment`: front C's widget (the Comments tab); until
/// it lands the card names it.
// TODO(task-104 front C): draw front C's QuickComment here once it is in
// `engine::widgets`, as the kit's ModalPage does.
pub(super) struct Pending {
    pub(super) name: &'static str,
    pub(super) owner: &'static str,
}

impl Pending {
    fn line(&self) -> String {
        format!("{}: {}", self.name, self.owner)
    }
}

impl ModalBody for Pending {
    fn title(&self) -> &str {
        self.name
    }

    fn size(&self, w: f32, theme: &Theme) -> (f32, f32) {
        let title = modals::tokens(theme).share_post.title;
        (w, theme.typography.run(title).line_height)
    }

    fn handle_event(&mut self, _event: &WidgetEvent, _rect: Rect, _theme: &Theme) -> EventResult {
        EventResult::IGNORED
    }

    fn render(&self, c: &mut Compositor, rect: Rect, theme: &Theme) {
        let t = modals::tokens(theme);
        let style = &theme.typography.run(t.share_post.date);
        let y = modals::middle(style, rect.y, rect.h);
        let line = self.line();
        modals::text_centered(
            c,
            &line,
            style,
            rect.x + rect.w / 2.0,
            y,
            rect.w,
            t.ink.muted,
        );
    }

    fn project(&self, b: &mut SnapshotBuilder, id: &SemId, parent: &SemId) {
        b.node(id, Role::Group, self.name)
            .parent(parent)
            .description(self.line());
    }

    fn run(
        &mut self,
        _target: &SemId,
        _action: &str,
        _param: Option<&Value>,
    ) -> Option<Result<bool, String>> {
        None
    }

    fn focus(&mut self, _id: Option<&SemId>) -> bool {
        false
    }
}
