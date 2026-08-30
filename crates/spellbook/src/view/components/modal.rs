//! Modal scaffold: dimmed backdrop + centered panel + title row + close
//! button - the `.modal-backdrop`/`.modal` pattern every dialog in the old
//! UI sat in.
//!
//! Unlike `engine::ui::widgets::Modal` (a fixed 400px confirmation dialog,
//! used by [`super::confirm`]), this is a frame: it computes the panel rect,
//! draws the chrome, and lets the owner place arbitrary content inside. The
//! CSS original was `max-width: 480px; max-height: 85vh; padding: 24px` -
//! here the width is per-dialog and the height comes from the content, so a
//! dialog derives from the window instead of clipping at a viewport measure.

use engine::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use engine::text::TextStyle;
use engine::theme::{Theme, TypographyScale};
use engine::ui::widgets::{
    ButtonVariant, EventResult, IconButton, Rect, WidgetEvent, glass_pill, menu_shadow,
};

/// Inner padding of the panel (CSS: 24px).
pub const PAD: f32 = 24.0;
/// Height of the title row, including its gap to the content.
pub const TITLE_H: f32 = 30.0;
/// Backdrop dim, `rgba(0,0,0,.6)` in the CSS.
const SCRIM: [f32; 4] = [0.0, 0.0, 0.0, 0.6];
/// The panel never hugs the window edge (CSS: backdrop padding 20px).
const VIEW_PAD: f32 = 20.0;
/// Close button square.
const CLOSE: f32 = 40.0;

/// Title style: reuse the HOFF `=title` (20/1.2/500) for every dialog.
fn title_style() -> TextStyle {
    TypographyScale::hoff().title()
}

/// A framed dialog. The owner keeps one per open modal, computes
/// [`ModalFrame::rect`] once, places its content inside it, feeds events to
/// the frame first (backdrop + close button) and renders chrome + content.
pub struct ModalFrame {
    close: IconButton,
}

impl ModalFrame {
    pub fn new() -> Self {
        Self {
            close: IconButton::new("x").variant(ButtonVariant::Ghost),
        }
    }

    /// Panel rect: `width` wide, `content_h` of content below the title row,
    /// centered in the window and capped to it (CSS `max-height: 85vh`
    /// becomes "never overlap the backdrop padding").
    pub fn rect(&self, window: Rect, width: f32, content_h: f32) -> Rect {
        let w = width.min((window.w - VIEW_PAD * 2.0).max(0.0));
        let h = (PAD * 2.0 + TITLE_H + content_h).min((window.h - VIEW_PAD * 2.0).max(0.0));
        Rect::new(
            window.x + (window.w - w) / 2.0,
            window.y + (window.h - h) / 2.0,
            w,
            h,
        )
    }

    /// Close-button hit rect inside a panel rect.
    pub fn close_rect(&self, panel: Rect) -> Rect {
        Rect::new(panel.x + panel.w - PAD - CLOSE + 5.0, panel.y + 12.0, CLOSE, CLOSE)
    }

    /// Rect the dialog's content occupies.
    pub fn content_rect(&self, panel: Rect) -> Rect {
        Rect::new(
            panel.x + PAD,
            panel.y + PAD + TITLE_H,
            panel.w - PAD * 2.0,
            (panel.h - PAD * 2.0 - TITLE_H).max(0.0),
        )
    }

    /// Backdrop and close button. `panel` is the rect from [`ModalFrame::rect`].
    /// Returns `true` when the dialog should close (x button or backdrop
    /// click - both close, like every `.modal-backdrop` in the old UI).
    /// Everything inside the panel is reported handled so a click in a form
    /// never leaks to the screen below.
    pub fn handle_event(&mut self, event: &WidgetEvent, panel: Rect) -> (bool, EventResult) {
        let r = self.close.handle_event(event, self.close_rect(panel));
        if r.clicked {
            return (true, r);
        }
        let mut result = r;
        if let WidgetEvent::MouseDown { x, y } | WidgetEvent::MouseUp { x, y } = *event {
            if !panel.contains(x, y) {
                // A click outside the panel is a backdrop click: close on
                // mouse-down, swallow the pair so nothing under the backdrop
                // reacts.
                let close = matches!(*event, WidgetEvent::MouseDown { .. });
                return (close, EventResult::clicked());
            }
            result = result.merge(EventResult {
                handled: true,
                ..EventResult::IGNORED
            });
        }
        (false, result)
    }

    /// Escape closes the dialog, the platform habit `backdrop.remove()`
    /// inherited from `confirmDialog`.
    pub fn handle_escape(&mut self) -> bool {
        true
    }

    /// Backdrop + panel + title row + close glyph, drawn on the overlay
    /// layer. Content is the owner's job, drawn into
    /// [`ModalFrame::content_rect`] after this call.
    pub fn render(
        &mut self,
        c: &mut Compositor,
        layer: LayerId,
        window: Rect,
        panel: Rect,
        title: &str,
        theme: &Theme,
    ) {
        c.push_to_layer(
            layer,
            SceneNode::Rect {
                x: window.x,
                y: window.y,
                w: window.w,
                h: window.h,
                color: SCRIM,
            },
        );

        let radius = theme.radius.lg;
        c.push_to_layer(layer, menu_shadow(panel, radius));
        for node in glass_pill(panel, radius, theme.glass.edge_soft.0, 1.5, theme.glass.popover.0)
        {
            c.push_to_layer(layer, node);
        }

        let style = title_style();
        c.push_to_layer(
            layer,
            SceneNode::Text {
                key: TextNodeKey::from_style(title, &style, Some(panel.w - PAD * 2.0 - 40.0)),
                x: panel.x + PAD,
                y: panel.y + PAD - 4.0,
                color: theme.colors.text.0,
            },
        );
        self.close.render(c, self.close_rect(panel), theme);
    }
}
