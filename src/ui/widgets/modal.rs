use crate::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use crate::text::{TextMeasurer, TextStyle};
use crate::theme::{Intent, Theme};

use super::button::{Button, ButtonVariant};
use super::{EventResult, Rect, WidgetEvent, with_alpha};

const WIDTH: f32 = 400.0;
const PAD: f32 = 20.0;
const TITLE_FONT: f32 = 15.0;
const BODY_FONT: f32 = 13.0;
const BTN_GAP: f32 = 8.0;
const BTN_H: f32 = 30.0;

/// What the user decided.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModalAction {
    None,
    Confirm,
    Cancel,
}

/// Confirmation dialog with backdrop. The confirm button takes the
/// modal's intent (Destructive intent = red confirm + snappier physics
/// when pushed through `OverlayManager::push_animated`).
#[derive(Clone, Debug)]
pub struct Modal {
    pub title: String,
    pub body: String,
    pub intent: Intent,
    confirm: Button,
    cancel: Button,
}

impl Modal {
    pub fn new(
        title: impl Into<String>,
        body: impl Into<String>,
        confirm_label: impl Into<String>,
        cancel_label: impl Into<String>,
    ) -> Self {
        Self {
            title: title.into(),
            body: body.into(),
            intent: Intent::Neutral,
            confirm: Button::new(confirm_label).variant(ButtonVariant::Solid),
            cancel: Button::new(cancel_label).variant(ButtonVariant::Outline),
        }
    }

    pub fn intent(mut self, intent: Intent) -> Self {
        self.intent = intent;
        self.confirm.intent = intent;
        self
    }

    fn body_height(&self) -> f32 {
        let style = TextStyle::new(BODY_FONT).with_line_height(BODY_FONT * 1.5);
        let (_, h) = TextMeasurer::measure_styled(&self.body, &style, Some(WIDTH - PAD * 2.0));
        h.max(BODY_FONT * 1.5)
    }

    /// Centered dialog rect for a viewport.
    pub fn dialog_rect(&self, vw: f32, vh: f32) -> Rect {
        let h = PAD + TITLE_FONT * 1.3 + 10.0 + self.body_height() + 18.0 + BTN_H + PAD;
        Rect::new(
            ((vw - WIDTH) / 2.0).max(0.0),
            ((vh - h) / 2.0).max(0.0),
            WIDTH,
            h,
        )
    }

    fn button_rects(&self, dialog: Rect) -> (Rect, Rect) {
        let (cw, _) = self.confirm.preferred_size();
        let (xw, _) = self.cancel.preferred_size();
        let by = dialog.y + dialog.h - PAD - BTN_H;
        let confirm = Rect::new(dialog.x + dialog.w - PAD - cw, by, cw, BTN_H);
        let cancel = Rect::new(confirm.x - BTN_GAP - xw, by, xw, BTN_H);
        (confirm, cancel)
    }

    /// Route an event. Clicking the backdrop (outside the dialog) cancels,
    /// matching the platform convention.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        vw: f32,
        vh: f32,
    ) -> (ModalAction, EventResult) {
        let dialog = self.dialog_rect(vw, vh);
        let (confirm_rect, cancel_rect) = self.button_rects(dialog);

        let confirm_result = self.confirm.handle_event(event, confirm_rect);
        if confirm_result.clicked {
            return (ModalAction::Confirm, confirm_result);
        }
        let cancel_result = self.cancel.handle_event(event, cancel_rect);
        if cancel_result.clicked {
            return (ModalAction::Cancel, cancel_result);
        }

        let merged = confirm_result.merge(cancel_result);
        if let WidgetEvent::MouseDown { x, y } = *event
            && !dialog.contains(x, y)
        {
            return (ModalAction::Cancel, EventResult::clicked().merge(merged));
        }
        // A modal is blocking: swallow everything else.
        (
            ModalAction::None,
            EventResult {
                handled: true,
                ..merged
            },
        )
    }

    pub fn render(
        &self,
        compositor: &mut Compositor,
        layer: LayerId,
        theme: &Theme,
        vw: f32,
        vh: f32,
    ) {
        // Backdrop.
        compositor.push_to_layer(
            layer,
            SceneNode::Rect {
                x: 0.0,
                y: 0.0,
                w: vw,
                h: vh,
                color: [0.0, 0.0, 0.0, 0.45],
            },
        );

        let dialog = self.dialog_rect(vw, vh);
        compositor.push_to_layer(
            layer,
            SceneNode::RoundedRect {
                x: dialog.x,
                y: dialog.y,
                w: dialog.w,
                h: dialog.h,
                color: with_alpha(theme.colors.bg_panel, 1.0),
                corner_radius: theme.radius.lg,
                border_width: 1.0,
                border_color: with_alpha(theme.colors.divider, 1.0),
            },
        );

        compositor.push_to_layer(
            layer,
            SceneNode::Text {
                key: TextNodeKey::new(
                    &self.title,
                    TITLE_FONT,
                    TITLE_FONT * 1.3,
                    Some(dialog.w - PAD * 2.0),
                )
                .with_weight(600),
                x: dialog.x + PAD,
                y: dialog.y + PAD,
                color: with_alpha(theme.colors.text, 1.0),
            },
        );
        compositor.push_to_layer(
            layer,
            SceneNode::Text {
                key: TextNodeKey::new(
                    &self.body,
                    BODY_FONT,
                    BODY_FONT * 1.5,
                    Some(dialog.w - PAD * 2.0),
                ),
                x: dialog.x + PAD,
                y: dialog.y + PAD + TITLE_FONT * 1.3 + 10.0,
                color: with_alpha(theme.colors.text_mid, 1.0),
            },
        );

        let (confirm_rect, cancel_rect) = self.button_rects(dialog);
        self.cancel
            .render_to_layer(compositor, layer, cancel_rect, theme);
        self.confirm
            .render_to_layer(compositor, layer, confirm_rect, theme);
    }
}
