//! HOFF confirmation modal: scrim, a raised popover sheet at the pill
//! radius with the overlay shadow stack, title + wrapped body, and two
//! pill buttons. The sheet is `size.modal_max_w` wide on a desktop and
//! shrinks to the viewport minus the breakpoint gutter on a phone, so
//! the same dialog reads on every device.

use engine::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use engine::text::{TextMeasurer, TextStyle};
use engine::theme::{Elevation, Intent, Theme};

use crate::action::{Button, ButtonVariant};
use crate::core::{EventResult, Rect, WidgetEvent};
use crate::recipe::{glass_pill, inset_keylight, shadow_stack};

/// Title: the HOFF `=title` mixin.
fn title_style(theme: &Theme) -> TextStyle {
    theme.typography.title()
}

/// Body: base-2r, the same for measuring and rendering.
fn body_style(theme: &Theme) -> TextStyle {
    theme.typography.base_2r()
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

    /// Sheet padding: the `xxl` step (HOFF: 32).
    fn pad(theme: &Theme) -> f32 {
        theme.spacing.xxl
    }

    /// Sheet width for a viewport: the modal max, or the viewport minus
    /// the breakpoint gutter on each side when that is narrower.
    fn width(theme: &Theme, vw: f32) -> f32 {
        let gutter = theme.layout.gutter(theme.layout.breakpoint(vw));
        theme.size.modal_max_w.min((vw - gutter * 2.0).max(0.0))
    }

    fn body_height(&self, theme: &Theme, width: f32) -> f32 {
        let style = body_style(theme);
        let (_, h) =
            TextMeasurer::measure_styled(&self.body, &style, Some(width - Self::pad(theme) * 2.0));
        h.max(style.line_height)
    }

    /// Centered dialog rect for a viewport.
    pub fn dialog_rect(&self, theme: &Theme, vw: f32, vh: f32) -> Rect {
        let pad = Self::pad(theme);
        let w = Self::width(theme, vw);
        let btn_h = self.confirm.size.height(theme);
        let h = pad
            + title_style(theme).line_height
            + theme.spacing.sm
            + self.body_height(theme, w)
            + theme.spacing.xl
            + btn_h
            + pad;
        Rect::new(((vw - w) / 2.0).max(0.0), ((vh - h) / 2.0).max(0.0), w, h)
    }

    /// Confirm and cancel button rects inside `dialog`.
    pub fn button_rects(&self, theme: &Theme, dialog: Rect) -> (Rect, Rect) {
        let pad = Self::pad(theme);
        let (cw, btn_h) = self.confirm.preferred_size(theme);
        let (xw, _) = self.cancel.preferred_size(theme);
        let by = dialog.y + dialog.h - pad - btn_h;
        let confirm = Rect::new(dialog.x + dialog.w - pad - cw, by, cw, btn_h);
        let cancel = Rect::new(confirm.x - theme.spacing.sm - xw, by, xw, btn_h);
        (confirm, cancel)
    }

    /// Route an event. Clicking the backdrop (outside the dialog) cancels,
    /// matching the platform convention.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        theme: &Theme,
        vw: f32,
        vh: f32,
    ) -> (ModalAction, EventResult) {
        let dialog = self.dialog_rect(theme, vw, vh);
        let (confirm_rect, cancel_rect) = self.button_rects(theme, dialog);

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
        let pad = Self::pad(theme);

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

        // Raised sheet: overlay shadow stack, frost, edge-light, the solid
        // popover body, and the inset key-light glint.
        let dialog = self.dialog_rect(theme, vw, vh);
        let radius = theme.shape.pill;
        shadow_stack(compositor, layer, dialog, radius, Elevation::Overlay, theme);
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
        for node in glass_pill(
            dialog,
            radius,
            glass.edge_soft.0,
            theme.control.edge_width_strong,
            glass.popover.0,
        ) {
            compositor.push_to_layer(layer, node);
        }
        inset_keylight(compositor, layer, dialog, radius, theme);

        let title = title_style(theme);
        let text_w = dialog.w - pad * 2.0;
        compositor.push_to_layer(
            layer,
            SceneNode::Text {
                key: TextNodeKey::from_style(&self.title, &title, Some(text_w)),
                x: dialog.x + pad,
                y: dialog.y + pad,
                color: theme.colors.text.0,
            },
        );
        compositor.push_to_layer(
            layer,
            SceneNode::Text {
                key: TextNodeKey::from_style(&self.body, &body_style(theme), Some(text_w)),
                x: dialog.x + pad,
                y: dialog.y + pad + title.line_height + theme.spacing.sm,
                color: theme.colors.text_mid.0,
            },
        );

        let (confirm_rect, cancel_rect) = self.button_rects(theme, dialog);
        self.cancel
            .render_to_layer(compositor, layer, cancel_rect, theme);
        self.confirm
            .render_to_layer(compositor, layer, confirm_rect, theme);
    }
}
