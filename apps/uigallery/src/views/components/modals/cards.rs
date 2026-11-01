//! The two cards of the Modals page that are not a modal of
//! `engine::widgets::overlay::modals`: the delete confirm, which is the
//! confirmation `Modal` with the kit's art (the Alert Dialog page's), and
//! the quick comment, which is the message widget's.

use accord::{Role, SemId, SnapshotBuilder, Value};
use engine::drawing::compositor::{Compositor, LayerId};
use engine::styling::theme::Theme;
use engine::widgets::feedback::{Modal, ModalAction};
use engine::widgets::form::TextField;
use engine::widgets::message::quick_comment::{QuickComment, QuickPart};
use engine::widgets::overlay::modals::ModalBody;
use engine::widgets::prelude::{EventResult, Rect, WidgetEvent};

use super::super::comments::kit_quick_comment;
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

/// The kit's `QuickComment` as the ModalPage shows it: the message
/// widget's card (`engine::widgets::message::quick_comment`), the same
/// sample the Comments page holds. Its parts are under the card's id; the
/// reply line takes the keys while it has the focus.
pub(super) struct Quick {
    card: QuickComment,
}

impl Quick {
    pub(super) fn new(id: Option<&SemId>) -> Self {
        let mut card = kit_quick_comment();
        if let Some(id) = id {
            card = card.sem(id.clone());
        }
        Self { card }
    }

    fn typing(&self) -> bool {
        self.card.focused() == Some(QuickPart::Reply)
    }
}

impl ModalBody for Quick {
    fn title(&self) -> &str {
        "Shop House Icon"
    }

    fn size(&self, w: f32, theme: &Theme) -> (f32, f32) {
        self.card.size(w, theme)
    }

    fn handle_event(&mut self, event: &WidgetEvent, rect: Rect, theme: &Theme) -> EventResult {
        self.card.handle_event(event, rect, theme)
    }

    fn render(&self, c: &mut Compositor, rect: Rect, theme: &Theme) {
        self.card.render(c, rect, theme);
    }

    fn project(&self, b: &mut SnapshotBuilder, id: &SemId, parent: &SemId) {
        self.card.project(b, id, Some(parent));
        if let Some(at) = self.card.focused().and_then(|p| self.card.part_id(p)) {
            b.focus(&at);
        }
    }

    fn run(
        &mut self,
        target: &SemId,
        action: &str,
        param: Option<&Value>,
    ) -> Option<Result<bool, String>> {
        let part = self.card.part_of(target)?;
        Some(self.card.act(part, action, param).map(|()| false))
    }

    fn focus(&mut self, id: Option<&SemId>) -> bool {
        let part = id.and_then(|id| self.card.part_of(id));
        self.card.focus(part);
        part.is_some()
    }

    fn text_mut(&mut self) -> Option<&mut TextField> {
        if !self.typing() {
            return None;
        }
        Some(self.card.line_mut().field_mut())
    }

    fn ime_area(&self, rect: Rect, theme: &Theme) -> Option<Rect> {
        self.typing().then(|| self.card.ime_area(rect, theme))
    }

    /// Enter posts the reply, as `Send` does.
    fn enter(&mut self) -> bool {
        self.typing() && self.card.send()
    }

    fn escape_text(&mut self) -> bool {
        let typing = self.typing();
        if typing {
            self.card.focus(None);
        }
        typing
    }

    fn end_compose(&mut self) -> bool {
        self.card.line_mut().end_compose()
    }

    fn tick(&mut self, dt: f32) -> bool {
        self.card.wake(dt).now
    }
}
