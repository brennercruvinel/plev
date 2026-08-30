//! Modal scaffold: dimmed backdrop + centered panel + title row + close button.
//!
//! Port of the `.modal-backdrop`/`.modal` pattern used across the old JS.
//! This is the base building block for all modals in the app.

use engine::compositor::{Compositor, Rect};
use engine::theme::Theme;
use engine::ui::icons;
use engine::ui::widgets::{EventResult, WidgetEvent};
use engine::ui::widgets::modal::Modal as EngineModal;

use super::ScreenCtx;
use crate::art::ArtCache;

/// A reusable modal frame: backdrop + panel with title + close button.
/// The owning screen provides the inner content via `render_content`.
pub struct ModalFrame {
    engine_modal: EngineModal,
    title: String,
    /// Set to true when the modal should close on backdrop click or Escape.
    closable: bool,
    /// Whether the modal is currently open.
    open: bool,
}

impl ModalFrame {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            engine_modal: EngineModal::new(),
            title: title.into(),
            closable: true,
            open: false,
        }
    }

    pub fn with_closable(mut self, closable: bool) -> Self {
        self.closable = closable;
        self
    }

    /// Opens the modal.
    pub fn open(&mut self) {
        self.open = true;
        self.engine_modal.open();
    }

    /// Closes the modal.
    pub fn close(&mut self) {
        self.open = false;
        self.engine_modal.close();
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    /// Handle overlay events (mouse/keyboard) when this modal is the topmost overlay.
    /// Returns true if the event was consumed.
    pub fn handle_overlay_event(
        &mut self,
        event: &WidgetEvent,
        ctx: &mut ScreenCtx,
    ) -> bool {
        if !self.open {
            return false;
        }

        let mut consumed = false;

        // Escape closes if closable
        if self.closable {
            if let WidgetEvent::Key(key) = event {
                if key == engine::input::types::KeyInput::Named(engine::keyboard::NamedKey::Escape) {
                    self.close();
                    consumed = true;
                }
            }
        }

        // Backdrop click closes if closable
        if self.closable {
            if let WidgetEvent::MouseDown { x, y } = event {
                // Check if click is outside the modal panel
                let dialog_rect = self.engine_modal.dialog_rect(800.0, 600.0); // approximate, will be recalculated in render
                if !dialog_rect.contains(x, y) {
                    self.close();
                    consumed = true;
                }
            }
        }

        consumed
    }

    /// Render the modal frame. The caller provides a closure that renders the inner content.
    pub fn render(
        &mut self,
        c: &mut Compositor,
        window_rect: Rect,
        theme: &Theme,
        _art: &mut ArtCache,
        render_content: impl FnOnce(&mut Compositor, Rect, &Theme, &mut ArtCache),
    ) {
        if !self.open {
            return;
        }

        // Render backdrop + dialog via engine modal
        self.engine_modal.render(
            c,
            window_rect,
            theme,
            |c, bounds, theme| {
                // Title bar
                c.push(engine::ui::widgets::rounded_rect(
                    bounds.x,
                    bounds.y,
                    bounds.w,
                    48.0,
                    theme.radius.md,
                    theme.glass.surface.0,
                ));

                // Title text
                engine::ui::widgets::text(
                    c,
                    &self.title,
                    14.0,
                    600,
                    bounds.x + 16.0,
                    bounds.y + 16.0,
                    theme.colors.text.0,
                );

                // Close button (X)
                let close_x = bounds.x + bounds.w - 40.0;
                let close_y = bounds.y + 8.0;
                if let Some(node) = engine::ui::icons::icon_at(
                    "x",
                    18.0,
                    theme.colors.text_dim.0,
                    close_x,
                    close_y,
                ) {
                    c.push(node);
                }

                // Content area (below title bar)
                let content_rect = Rect::new(
                    bounds.x + 16.0,
                    bounds.y + 48.0 + 12.0,
                    bounds.w - 32.0,
                    bounds.h - 48.0 - 28.0,
                );

                render_content(c, content_rect, theme, &mut crate::art::ArtCache::new());
            },
        );
    }
}

/// A confirm dialog (yes/no) built on top of ModalFrame.
pub struct ConfirmDialog {
    modal: ModalFrame,
    message: String,
    confirm_label: String,
    cancel_label: String,
    danger: bool,
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
        Self {
            modal: ModalFrame::new(title).with_closable(true),
            message: message.into(),
            confirm_label: confirm_label.into(),
            cancel_label: cancel_label.into(),
            danger,
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

    pub fn take_result(&mut self) -> Option<bool> {
        self.result.take()
    }

    pub fn handle_overlay_event(
        &mut self,
        event: &WidgetEvent,
        ctx: &mut ScreenCtx,
    ) -> bool {
        if !self.modal.is_open() {
            return false;
        }

        let consumed = self.modal.handle_overlay_event(event, ctx);

        if let WidgetEvent::MouseDown { x, y } = event {
            // Check confirm/cancel buttons - would need button rects
            // For now, delegate to the modal's close behavior
        }

        consumed
    }

    pub fn render(
        &mut self,
        c: &mut Compositor,
        window_rect: Rect,
        theme: &Theme,
        art: &mut ArtCache,
    ) {
        self.modal.render(c, window_rect, theme, art, |c, bounds, theme, _art| {
            // Title
            engine::ui::widgets::text(
                c,
                "Confirmar",
                14.0,
                600,
                bounds.x + 16.0,
                bounds.y + 16.0,
                theme.colors.text.0,
            );

            // Message
            engine::ui::widgets::text(
                c,
                &self.message,
                13.0,
                400,
                bounds.x + 16.0,
                bounds.y + 48.0,
                theme.colors.text_dim.0,
            );

            // Buttons
            let btn_y = bounds.y + bounds.h - 60.0;
            let cancel_w = 100.0;
            let confirm_w = 100.0;
            let gap = 10.0;
            let start_x = bounds.x + bounds.w - confirm_w - cancel_w - gap - 16.0;

            // Cancel button
            let cancel_rect = engine::ui::Rect::new(
                start_x,
                btn_y,
                cancel_w,
                40.0,
            );
            let cancel_color = theme.glass.surface.0;
            c.push(engine::ui::widgets::rounded_rect(
                cancel_rect.x,
                cancel_rect.y,
                cancel_rect.w,
                cancel_rect.h,
                theme.radius.md,
                cancel_color,
            ));
            engine::ui::widgets::text(
                c,
                &self.cancel_label,
                13.0,
                600,
                cancel_rect.x + cancel_rect.w / 2.0 - 20.0,
                cancel_rect.y + 12.0,
                theme.colors.text.0,
            );

            // Confirm button
            let confirm_x = start_x + cancel_w + gap;
            let confirm_rect = engine::ui::Rect::new(
                confirm_x,
                btn_y,
                confirm_w,
                40.0,
            );
            let confirm_color = if self.danger {
                theme.colors.danger.0
            } else {
                theme.colors.success.0
            };
            c.push(engine::ui::widgets::rounded_rect(
                confirm_rect.x,
                confirm_rect.y,
                confirm_rect.w,
                confirm_rect.h,
                theme.radius.md,
                confirm_color,
            ));
            engine::ui::widgets::text(
                c,
                &self.confirm_label,
                13.0,
                600,
                confirm_rect.x + confirm_rect.w / 2.0 - 25.0,
                confirm_rect.y + 12.0,
                [1.0, 1.0, 1.0, 1.0],
            );
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modal_opens_and_closes() {
        let mut modal = ModalFrame::new("Test");
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