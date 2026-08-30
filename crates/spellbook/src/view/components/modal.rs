//! Modal scaffold and confirm dialog built on plev's engine::ui::widgets::Modal.
//!
//! The engine provides a ready-to-use Modal widget with backdrop scrim,
//! glass panel, title, body, and confirm/cancel buttons. We wrap it to
//! integrate with our ScreenCtx (commands, toasts, navigation).

use engine::compositor::{Compositor, LayerId, Rect};
use engine::theme::{Intent, Theme};
use engine::ui::widgets::{EventResult, Modal as EngineModal, WidgetEvent};

use super::ScreenCtx;
use crate::art::ArtCache;

/// Simple wrapper around engine::ui::widgets::Modal that integrates with
/// our ScreenCtx for commands/toasts/navigation.
pub struct Modal {
    engine_modal: engine::ui::widgets::Modal,
    open: bool,
}

impl Modal {
    pub fn new(
        title: impl Into<String>,
        body: impl Into<String>,
        confirm_label: impl Into<String>,
        cancel_label: impl Into<String>,
    ) -> Self {
        Self {
            engine_modal: engine::ui::widgets::Modal::new(
                title,
                body,
                confirm_label,
                cancel_label,
            ),
            open: false,
        }
    }

    pub fn intent(mut self, intent: Intent) -> Self {
        self.engine_modal = self.engine_modal.intent(intent);
        self
    }

    pub fn open(&mut self) {
        self.open = true;
    }

    pub fn close(&mut self) {
        self.open = false;
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    /// Handle overlay events when this modal is the topmost overlay.
    /// Returns true if the event was consumed (handled or swallowed).
    pub fn handle_overlay_event(
        &mut self,
        event: &WidgetEvent,
        vw: f32,
        vh: f32,
    ) -> bool {
        if !self.open {
            return false;
        }

        let (action, result) = self.engine_modal.handle_event(event, vw, vh);
        match action {
            engine::ui::widgets::ModalAction::Confirm | engine::ui::widgets::ModalAction::Cancel => {
                self.open = false;
                true
            }
            engine::ui::widgets::ModalAction::None => result.handled,
        }
    }

    pub fn render(
        &self,
        c: &mut Compositor,
        layer: LayerId,
        theme: &Theme,
        vw: f32,
        vh: f32,
    ) {
        if !self.open {
            return;
        }
        self.engine_modal.render(c, layer, theme, vw, vh);
    }
}

/// Confirm dialog - a simple yes/no dialog built on Modal.
/// Returns the result via take_result() after the user answers.
pub struct ConfirmDialog {
    modal: Modal,
    result: Option<bool>,
}

impl ConfirmDialog {
    pub fn new(
        title: impl Into<String>,
        message: impl Into<String>,
        confirm_label: impl Into<String>,
        cancel_label: impl Into<String>,
        danger: bool,
    ) -> Self {
        let modal = Modal::new(
            title,
            message,
            "Confirmar",
            "Cancelar",
        ).intent(if danger { Intent::Destructive } else { Intent::Constructive });

        Self {
            modal,
            result: None,
        }
    }

    pub fn open(&mut self) {
        self.modal.open();
        self.result = None;
    }

    pub fn close(&mut self) {
        self.modal.close();
    }

    pub fn is_open(&self) -> bool {
        self.modal.is_open()
    }

    /// Take the user's answer (true=confirmed, false=cancelled).
    /// Returns None if no answer yet.
    pub fn take_result(&mut self) -> Option<bool> {
        self.result.take()
    }

    /// Handle overlay events. Returns true if consumed.
    pub fn handle_overlay_event(
        &mut self,
        event: &WidgetEvent,
        vw: f32,
        vh: f32,
    ) -> bool {
        if !self.modal.is_open() {
            return false;
        }

        let consumed = self.modal.handle_overlay_event(event, vw, vh);
        if !self.modal.is_open() && self.result.is_none() {
            // Modal closed without explicit confirm/cancel (backdrop click)
            self.result = Some(false);
        }
        consumed
    }

    pub fn render(
        &self,
        c: &mut Compositor,
        layer: LayerId,
        theme: &Theme,
        vw: f32,
        vh: f32,
    ) {
        self.modal.render(c, layer, theme, vw, vh);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modal_opens_and_closes() {
        let mut modal = Modal::new("Test", "Body", "OK", "Cancel");
        assert!(!modal.is_open());
        modal.open();
        assert!(modal.is_open());
        modal.close();
        assert!(!modal.is_open());
    }

    #[test]
    fn confirm_dialog_creation() {
        let dialog = ConfirmDialog::new(
            "Título",
            "Mensagem",
            "Sim",
            "Não",
            false,
        );
        assert!(!dialog.is_open());
        dialog.open();
        assert!(dialog.is_open());
    }
}