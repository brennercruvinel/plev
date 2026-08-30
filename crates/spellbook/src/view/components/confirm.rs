//! Yes/no confirmation dialog. Port of `desktop/ui/js/ui/confirm.js` atop
//! `engine::ui::widgets::Modal`, which already encodes the same contract:
//! backdrop click cancels, a blocking dialog swallows everything else.
//!
//! The JS answers a promise; here the owner opens a [`Confirm`], forwards
//! events, and reads [`ConfirmAction`] off each call. Escape cancels and
//! Enter confirms, the keyboard behaviour the native `window.confirm` had
//! (and wry's delegate lacked - the reason this dialog exists at all).

use engine::compositor::{Compositor, LayerId};
use engine::theme::{Intent, Theme};
use engine::ui::widgets::{EventResult, Modal, ModalAction, Rect, WidgetEvent};

use super::super::EditKey;

/// What the user decided.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConfirmAction {
    /// Still open.
    Pending,
    Confirmed,
    Cancelled,
}

pub struct Confirm {
    modal: Modal,
}

impl Confirm {
    /// Defaults match the JS: "Confirmar"/"Cancelar", `danger` recolors the
    /// confirm button.
    pub fn new(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            modal: Modal::new(title, message, "Confirmar", "Cancelar"),
        }
    }

    pub fn labels(mut self, confirm: impl Into<String>, cancel: impl Into<String>) -> Self {
        self.modal = Modal::new(self.modal.title.clone(), self.modal.body.clone(), confirm, cancel)
            .intent(self.modal.intent);
        self
    }

    /// Red confirm button, like `danger = true` in the JS.
    pub fn danger(mut self) -> Self {
        self.modal = self.modal.intent(Intent::Destructive);
        self
    }

    /// Enter confirms, anything else is the buttons' or the backdrop's.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        window: Rect,
    ) -> (ConfirmAction, EventResult) {
        let (action, result) = self.modal.handle_event(event, window.w, window.h);
        let action = match action {
            ModalAction::Confirm => ConfirmAction::Confirmed,
            ModalAction::Cancel => ConfirmAction::Cancelled,
            ModalAction::None => ConfirmAction::Pending,
        };
        (action, result)
    }

    pub fn handle_edit_key(&mut self, key: EditKey) -> ConfirmAction {
        match key {
            EditKey::Enter => ConfirmAction::Confirmed,
            _ => ConfirmAction::Pending,
        }
    }

    /// Escape cancels (and stays the shell's close signal: the owner drops
    /// the dialog when this returns true).
    pub fn handle_escape(&mut self) -> ConfirmAction {
        ConfirmAction::Cancelled
    }

    pub fn render(&mut self, c: &mut Compositor, layer: LayerId, window: Rect, theme: &Theme) {
        self.modal.render(c, layer, theme, window.w, window.h);
    }
}
