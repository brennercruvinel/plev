//! The confirmation modal: a scrim over the window, a sheet with a title,
//! a wrapped body and two buttons. Every value is `theme.overlay.modal`:
//! on core the kit's dialog (`components/DeleteFile`: the text centered,
//! the buttons sharing a footer band), on every other theme its own
//! (the hoff glass sheet, the buttons at the body's end). The sheet is
//! the modal's `max_w` wide and shrinks to the window less the breakpoint
//! gutter, so the same dialog reads on every device; a page that shows
//! the dialog as a state draws it in a card ([`Modal::render_in`]).
//!
//! With a [`SemId`] ([`Modal::sem`]) the render binds the sheet and its
//! two buttons (`<id>[confirm]`, `<id>[cancel]`) to the rects it draws
//! and hit-tests them with, and [`Modal::project`] declares the dialog in
//! a semantic projection.

use crate::drawing::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use crate::drawing::text::{TextMeasurer, TextStyle};
use crate::styling::theme::{Intent, Theme};
use accord::{NodeBuilder, Policy, Role, SemId, SnapshotBuilder};

use crate::widgets::buttons::{Button, ButtonVariant};
use crate::widgets::recipe::{
    glass_pill, inset_keylight, rounded_rect, rounded_rect_stroke, shadow_node,
};
use crate::widgets::widget::{EventResult, Rect, WidgetEvent};

/// Title, the same for measuring and rendering.
fn title_style(theme: &Theme) -> &TextStyle {
    &theme.overlay.modal.title
}

/// Body, the same for measuring and rendering.
fn body_style(theme: &Theme) -> &TextStyle {
    &theme.overlay.modal.body
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
    sem: Option<SemId>,
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
            confirm: Button::new(confirm_label).variant(ButtonVariant::Secondary),
            cancel: Button::new(cancel_label).variant(ButtonVariant::Primary),
            sem: None,
        }
    }

    /// The logical identity the render binds the sheet to
    /// (`Compositor::bind_to_layer`); the buttons bind as `<id>[confirm]`
    /// and `<id>[cancel]`.
    pub fn sem(mut self, id: SemId) -> Self {
        self.confirm = self.confirm.sem(id.key("confirm"));
        self.cancel = self.cancel.sem(id.key("cancel"));
        self.sem = Some(id);
        self
    }

    pub fn sem_id(&self) -> Option<&SemId> {
        self.sem.as_ref()
    }

    /// Declares this modal as `id` in a projection: a `Role::Dialog`
    /// named by its title, described by its body, with the actions
    /// `confirm` and `cancel`, and its two buttons as `Role::Button`
    /// children `<id>[confirm]` and `<id>[cancel]`, each named by its
    /// label with an `activate`. The confirm (the dialog's and its
    /// button's) carries the policy the intent implies: destructive for a
    /// destructive modal. It only reads the widget; the app owns it and
    /// runs the actions (a button's `activate` is its dialog action). Pass
    /// the id given to [`Modal::sem`] so the nodes and the bindings name
    /// the same controls. Returns the dialog's node for the app to add to.
    #[track_caller]
    pub fn project<'b>(
        &self,
        b: &'b mut SnapshotBuilder,
        id: &SemId,
        parent: Option<&SemId>,
    ) -> NodeBuilder<'b> {
        let confirm = Policy {
            destructive: self.intent == Intent::Destructive,
            ..Policy::default()
        };
        let mut button = b.node(id.key("confirm"), Role::Button, self.confirm.label.as_str());
        button.parent(id);
        button.action("activate").policy(confirm);
        let mut button = b.node(id.key("cancel"), Role::Button, self.cancel.label.as_str());
        button.parent(id);
        button.action("activate");
        let mut node = b.node(id, Role::Dialog, self.title.as_str());
        if let Some(parent) = parent {
            node.parent(parent);
        }
        node.description(self.body.as_str());
        node.action("confirm").policy(confirm);
        node.action("cancel");
        node
    }

    pub fn intent(mut self, intent: Intent) -> Self {
        self.intent = intent;
        // Destructive confirms wear the warning surface.
        self.confirm.variant = if intent == Intent::Destructive {
            ButtonVariant::Warning
        } else {
            ButtonVariant::Secondary
        };
        self
    }

    /// Sheet width in a `vw`-wide window: the modal's max, or the window
    /// less the breakpoint gutter on each side when that is narrower.
    fn width(theme: &Theme, vw: f32) -> f32 {
        let gutter = theme.layout.gutter(theme.layout.breakpoint(vw));
        theme.overlay.modal.max_w.min((vw - gutter * 2.0).max(0.0))
    }

    fn body_height(&self, theme: &Theme, width: f32) -> f32 {
        let style = body_style(theme);
        let pad = theme.overlay.modal.pad;
        let (_, h) = TextMeasurer::measure_styled(&self.body, style, Some(width - pad * 2.0));
        h.max(style.line_height)
    }

    /// Whether the buttons sit in a footer band of their own.
    fn footed(theme: &Theme) -> bool {
        theme.overlay.modal.footer.0[3] > 0.0
    }

    /// The sheet `w` wide: its height.
    fn height(&self, theme: &Theme, w: f32) -> f32 {
        let m = &theme.overlay.modal;
        let btn_h = self.confirm.size.height(theme);
        let text =
            m.pad + title_style(theme).line_height + m.title_gap + self.body_height(theme, w);
        if Self::footed(theme) {
            text + m.pad + m.border_w + m.pad + btn_h + m.pad
        } else {
            text + m.actions_gap + btn_h + m.pad
        }
    }

    /// The sheet centered in a `vw x vh` window.
    pub fn dialog_rect(&self, theme: &Theme, vw: f32, vh: f32) -> Rect {
        self.dialog_rect_in(theme, Rect::new(0.0, 0.0, vw, vh))
    }

    /// The sheet centered in `bounds`, as wide as a window of that width
    /// lets it be.
    pub fn dialog_rect_in(&self, theme: &Theme, bounds: Rect) -> Rect {
        let w = Self::width(theme, bounds.w);
        let h = self.height(theme, w);
        let x = bounds.x + ((bounds.w - w) / 2.0).max(0.0);
        let y = bounds.y + ((bounds.h - h) / 2.0).max(0.0);
        Rect::new(x, y, w, h)
    }

    /// Keyboard focus on one of the two buttons, driven by the owning view
    /// (the button a screen reader or the Tab key moved it to): that
    /// button's accent ring shows. `ModalAction::None` clears it.
    pub fn set_focused(&mut self, button: ModalAction) {
        self.confirm.set_focused(button == ModalAction::Confirm);
        self.cancel.set_focused(button == ModalAction::Cancel);
    }

    /// The button the keyboard focus is on, `ModalAction::None` for none.
    pub fn focused(&self) -> ModalAction {
        if self.confirm.is_focused() {
            ModalAction::Confirm
        } else if self.cancel.is_focused() {
            ModalAction::Cancel
        } else {
            ModalAction::None
        }
    }

    /// Confirm and cancel button rects inside `dialog`.
    pub fn button_rects(&self, theme: &Theme, dialog: Rect) -> (Rect, Rect) {
        let m = &theme.overlay.modal;
        let (cw, btn_h) = self.confirm.preferred_size(theme);
        let (xw, _) = self.cancel.preferred_size(theme);
        let by = dialog.y + dialog.h - m.pad - btn_h;
        if Self::footed(theme) {
            // `flex-1`: the cancel first, the two sharing the band.
            let w = ((dialog.w - m.pad * 2.0 - m.button_gap) / 2.0).max(0.0);
            let cancel = Rect::new(dialog.x + m.pad, by, w, btn_h);
            let confirm = Rect::new(cancel.right() + m.button_gap, by, w, btn_h);
            return (confirm, cancel);
        }
        let confirm = Rect::new(dialog.x + dialog.w - m.pad - cw, by, cw, btn_h);
        let cancel = Rect::new(confirm.x - m.button_gap - xw, by, xw, btn_h);
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

    /// The scrim over the `vw x vh` window and the sheet centered in it.
    pub fn render(
        &self,
        compositor: &mut Compositor,
        layer: LayerId,
        theme: &Theme,
        vw: f32,
        vh: f32,
    ) {
        let window = Rect::new(0.0, 0.0, vw, vh);
        compositor.push_to_layer(
            layer,
            SceneNode::Rect {
                x: 0.0,
                y: 0.0,
                w: vw,
                h: vh,
                color: theme.overlay.modal.scrim.0,
            },
        );
        self.render_in(compositor, layer, theme, self.dialog_rect_in(theme, window));
    }

    /// The sheet at `dialog`, without a scrim: the modal on screen, or
    /// the dialog shown as a state in a page's card.
    pub fn render_in(
        &self,
        compositor: &mut Compositor,
        layer: LayerId,
        theme: &Theme,
        dialog: Rect,
    ) {
        let m = &theme.overlay.modal;
        if let Some(id) = &self.sem {
            compositor.bind_to_layer(layer, id, [dialog.x, dialog.y, dialog.w, dialog.h]);
        }
        let radius = m.radius.min(dialog.h / 2.0);
        for spec in &m.shadow {
            if let Some(node) = shadow_node(dialog, radius, spec) {
                compositor.push_to_layer(layer, node);
            }
        }
        if m.glass {
            // The hoff sheet: frost, edge-light, the solid body, the
            // inset key-light glint.
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
            for node in glass_pill(dialog, radius, m.border.0, m.border_w, m.fill.0) {
                compositor.push_to_layer(layer, node);
            }
            inset_keylight(compositor, layer, dialog, radius, theme);
        } else {
            let d = dialog;
            compositor.push_to_layer(layer, rounded_rect(d.x, d.y, d.w, d.h, radius, m.fill.0));
        }
        let (confirm_rect, cancel_rect) = self.button_rects(theme, dialog);
        if Self::footed(theme) {
            // The footer band: its fill to the sheet's bottom corners
            // (clipped to the sheet), its rule on top.
            let top = cancel_rect.y - m.pad - m.border_w;
            compositor.push_to_layer(
                layer,
                SceneNode::PushClip {
                    x: dialog.x,
                    y: top,
                    w: dialog.w,
                    h: dialog.bottom() - top,
                },
            );
            let d = dialog;
            compositor.push_to_layer(layer, rounded_rect(d.x, d.y, d.w, d.h, radius, m.footer.0));
            compositor.push_to_layer(layer, SceneNode::PopClip);
            compositor.push_to_layer(
                layer,
                SceneNode::Rect {
                    x: dialog.x,
                    y: top,
                    w: dialog.w,
                    h: m.border_w,
                    color: m.footer_rule.0,
                },
            );
        }
        if !m.glass && m.border.0[3] > 0.0 {
            let d = dialog;
            compositor.push_to_layer(
                layer,
                rounded_rect_stroke(d.x, d.y, d.w, d.h, radius, m.border.0, m.border_w),
            );
        }

        let (title, body) = (title_style(theme), body_style(theme));
        let text_w = dialog.w - m.pad * 2.0;
        // Centered: each block measured, then centered as a whole.
        let x_of = |text: &str, style: &TextStyle| {
            if !m.centered {
                return dialog.x + m.pad;
            }
            let (w, _) = TextMeasurer::measure_styled(text, style, Some(text_w));
            dialog.x + (dialog.w - w) / 2.0
        };
        compositor.push_to_layer(
            layer,
            SceneNode::Text {
                key: TextNodeKey::from_style(&self.title, title, Some(text_w)),
                x: x_of(&self.title, title),
                y: dialog.y + m.pad,
                color: m.title_color.0,
            },
        );
        compositor.push_to_layer(
            layer,
            SceneNode::Text {
                key: TextNodeKey::from_style(&self.body, body, Some(text_w)),
                x: x_of(&self.body, body),
                y: dialog.y + m.pad + title.line_height + m.title_gap,
                color: m.body_color.0,
            },
        );
        self.cancel
            .render_to_layer(compositor, layer, cancel_rect, theme);
        self.confirm
            .render_to_layer(compositor, layer, confirm_rect, theme);
    }
}
