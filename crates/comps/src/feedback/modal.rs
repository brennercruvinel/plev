use crate::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use crate::text::{TextMeasurer, TextStyle};
use crate::theme::{Intent, Theme, TypographyScale};

use super::button::{Button, ButtonVariant};
use super::{EventResult, Rect, WidgetEvent, glass_pill};

/// HOFF modal: max-width 400, radius 32, padding 32, title 20/500,
/// scrim rgba(35,34,34,.9), rgba(40,40,40,.7) glass body.
const WIDTH: f32 = 400.0;
const PAD: f32 = 32.0;
const BTN_GAP: f32 = 8.0;
const BTN_H: f32 = 44.0;

/// Title: the HOFF `=title` mixin (20/1.2/500).
fn title_style() -> TextStyle {
    TypographyScale::hoff().title()
}

/// Body: base-2r, the same for measuring and rendering.
fn body_style() -> TextStyle {
    TypographyScale::hoff().base_2r()
}

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
        let style = body_style();
        let (_, h) = TextMeasurer::measure_styled(&self.body, &style, Some(WIDTH - PAD * 2.0));
        h.max(style.line_height)
    }

    /// Centered dialog rect for a viewport.
    pub fn dialog_rect(&self, vw: f32, vh: f32) -> Rect {
        let h = PAD + title_style().line_height + 10.0 + self.body_height() + 24.0 + BTN_H + PAD;
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
        let glass = &theme.glass;

        // Scrim: rgba(35,34,34,.9).
        compositor.push_to_layer(
            layer,
            SceneNode::Rect {
                x: 0.0,
                y: 0.0,
                w: vw,
                h: vh,
                color: glass.scrim.0,
            },
        );

        // Glass dialog: deep floating-menu shadow + real frost + edge-light
        // + the solid #3B3B3B sheet (measured live: dialogs/menus are this
        // lighter graphite, radius 32) + the inset key-light glint.
        let dialog = self.dialog_rect(vw, vh);
        let radius = theme.radius.xl;
        compositor.push_to_layer(
            layer,
            SceneNode::Shadow {
                x: dialog.x,
                y: dialog.y,
                w: dialog.w,
                h: dialog.h,
                corner_radius: radius,
                // 0 24px 32px -12px rgba(18,18,18,..): the measured menu
                // shadow, deepened for the blocking dialog.
                blur_radius: 32.0,
                offset: [0.0, 24.0],
                color: [18.0 / 255.0, 18.0 / 255.0, 18.0 / 255.0, 0.45],
                inset: false,
            },
        );
        compositor.push_to_layer(
            layer,
            SceneNode::BackdropBlur {
                x: dialog.x,
                y: dialog.y,
                w: dialog.w,
                h: dialog.h,
                corner_radius: radius,
                sigma: theme.effects.blur_sigma,
            },
        );
        for node in glass_pill(dialog, radius, glass.edge_soft.0, 1.5, glass.popover.0) {
            compositor.push_to_layer(layer, node);
        }
        // Inset key-light: inset 2px 4px 16px rgba(248,248,248,.06).
        compositor.push_to_layer(
            layer,
            SceneNode::Shadow {
                x: dialog.x,
                y: dialog.y,
                w: dialog.w,
                h: dialog.h,
                corner_radius: radius,
                blur_radius: 16.0,
                offset: [2.0, 4.0],
                color: glass.inset_highlight.0,
                inset: true,
            },
        );

        // Title 20/1.2/500 $text-primary; body base-2r $text-secondary.
        compositor.push_to_layer(
            layer,
            SceneNode::Text {
                key: TextNodeKey::from_style(
                    &self.title,
                    &title_style(),
                    Some(dialog.w - PAD * 2.0),
                ),
                x: dialog.x + PAD,
                y: dialog.y + PAD,
                color: theme.colors.text.0,
            },
        );
        compositor.push_to_layer(
            layer,
            SceneNode::Text {
                key: TextNodeKey::from_style(&self.body, &body_style(), Some(dialog.w - PAD * 2.0)),
                x: dialog.x + PAD,
                y: dialog.y + PAD + title_style().line_height + 10.0,
                color: theme.colors.text_mid.0,
            },
        );

        let (confirm_rect, cancel_rect) = self.button_rects(dialog);
        self.cancel
            .render_to_layer(compositor, layer, cancel_rect, theme);
        self.confirm
            .render_to_layer(compositor, layer, confirm_rect, theme);
    }
}
