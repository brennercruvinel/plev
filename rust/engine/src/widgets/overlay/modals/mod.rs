//! The kit's modals (`templates/ModalPage`, task-104 front B, onda 3):
//! Variations, the shared post, the shared profile, the shared file,
//! Export with its images, video and 3D object, and Settings with its
//! general, profile, security, notifications and subscription. Delete
//! confirm is the confirmation [`Modal`](crate::widgets::feedback::Modal)
//! with its art.
//!
//! Each modal is a widget over [`ModalBody`]: it measures itself in the
//! room it is given (its kit width, or less: the room caps it, and the
//! side menu of Export and Settings goes above its section when the room
//! is narrower than the kit's sheet), lays its controls out
//! ([`control::Controls`], the engine's own widgets under their ids) and
//! draws around them from `theme.overlay.modals` ([`tokens`]). The
//! controls route the pointer, project, run what an actor asks and hold
//! the keyboard focus; the modal adds what it means (the variation
//! chosen, the section shown).

pub mod control;
pub mod share;
pub mod share_file;
pub mod tokens;
pub mod variations;

use accord::{SemId, SnapshotBuilder, Value};

use crate::drawing::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use crate::drawing::text::{TextMeasurer, TextStyle};
use crate::styling::color::Color;
use crate::styling::theme::Theme;
use crate::widgets::icons;
use crate::widgets::recipe::{
    glass_pill, gradient_fill, inset_keylight, rounded_rect, rounded_rect_stroke, shadow_node,
};
use crate::widgets::widget::{EventResult, Rect, WidgetEvent};

pub use control::{Control, Controls, Item, ItemRole, Kind};
pub use tokens::{ModalInk, ModalsTokens, PanelTokens};

/// A modal of the kit as a page shows it: sized by the room, drawn at a
/// rect, its controls projected and operable.
pub trait ModalBody {
    /// Its title: what its group is named in a projection.
    fn title(&self) -> &str;

    /// Its size in a room `w` wide.
    fn size(&self, w: f32, theme: &Theme) -> (f32, f32);

    fn handle_event(&mut self, event: &WidgetEvent, rect: Rect, theme: &Theme) -> EventResult;

    fn render(&self, c: &mut Compositor, rect: Rect, theme: &Theme);

    /// What floats above the page: an open select's list.
    fn render_overlay(&self, _c: &mut Compositor, _layer: LayerId, _rect: Rect, _theme: &Theme) {}

    /// Its group `id` under `parent` (a `Role::Group` named by its
    /// title) and its controls under it.
    fn project(&self, b: &mut SnapshotBuilder, id: &SemId, parent: &SemId);

    /// `action` on `target`: `None` when it is none of this modal's.
    fn run(
        &mut self,
        target: &SemId,
        action: &str,
        param: Option<&Value>,
    ) -> Option<Result<bool, String>>;

    /// The keyboard focus on `id`, or off: whether one of its controls
    /// took it.
    fn focus(&mut self, id: Option<&SemId>) -> bool;

    /// Whether a select of it is open: the pointer is that select's alone.
    fn has_open(&self) -> bool {
        false
    }

    /// Escape: closes an open select. `true` when one was.
    fn close(&mut self) -> bool {
        false
    }

    /// Animation: `true` while something moves.
    fn tick(&mut self, _dt: f32) -> bool {
        false
    }
}

/// `<parent>.<name>`; `None` without a parent.
pub fn child(parent: Option<&SemId>, name: &str) -> Option<SemId> {
    parent.and_then(|p| p.child(name).ok())
}

/// The modals' recipes of `theme`.
pub fn tokens(theme: &Theme) -> &ModalsTokens {
    &theme.overlay.modals
}

/// A modal's surface at `rect`: its shadows, its paper (the hoff glass
/// on a theme that has it) and its edge.
pub fn panel(c: &mut Compositor, rect: Rect, p: &PanelTokens, theme: &Theme) {
    let layer = LayerId::DEFAULT;
    let radius = p.radius.min(rect.h / 2.0).min(rect.w / 2.0);
    for spec in &p.shadow {
        if let Some(node) = shadow_node(rect, radius, spec) {
            c.push(node);
        }
    }
    let Rect { x, y, w, h } = rect;
    if p.glass {
        c.push(SceneNode::BackdropBlur {
            x,
            y,
            w,
            h,
            corner_radius: radius,
            sigma: theme.effects.blur_sigma,
        });
        for node in glass_pill(rect, radius, p.border.0, p.border_w, p.fill.0, theme) {
            c.push(node);
        }
        inset_keylight(c, layer, rect, radius, theme);
        return;
    }
    c.push(rounded_rect(x, y, w, h, radius, p.fill.0));
    if p.border.0[3] > 0.0 {
        c.push(rounded_rect_stroke(
            x, y, w, h, radius, p.border.0, p.border_w,
        ));
    }
}

/// A box at `rect`: its fill, then its edge when it has one.
pub fn boxed(c: &mut Compositor, rect: Rect, radius: f32, fill: Color, edge: Option<(Color, f32)>) {
    let Rect { x, y, w, h } = rect;
    let radius = radius.min(h / 2.0).min(w / 2.0);
    if fill.0[3] > 0.0 {
        c.push(rounded_rect(x, y, w, h, radius, fill.0));
    }
    if let Some((color, width)) = edge.filter(|(c, _)| c.0[3] > 0.0) {
        c.push(rounded_rect_stroke(x, y, w, h, radius, color.0, width));
    }
}

/// `rect`'s edge alone, `width` inside it.
pub fn stroke(c: &mut Compositor, rect: Rect, radius: f32, color: Color, width: f32) {
    boxed(c, rect, radius, Color::TRANSPARENT, Some((color, width)));
}

/// `rect`'s shadows: a raised box.
pub fn raise(
    c: &mut Compositor,
    rect: Rect,
    radius: f32,
    shadows: &[crate::styling::theme::ShadowSpec],
) {
    for spec in shadows {
        if let Some(node) = shadow_node(rect, radius.min(rect.h / 2.0), spec) {
            c.push(node);
        }
    }
}

/// A picture that stands in for a photo: its gradient at `rect`.
pub fn picture(c: &mut Compositor, rect: Rect, radius: f32, colors: [Color; 2]) {
    let radius = radius.min(rect.h / 2.0).min(rect.w / 2.0);
    c.push(gradient_fill(rect, radius, colors[0].0, colors[1].0));
}

/// A picture whose start corners are `start` and end corners `end` (a
/// row of covers rounds its outer corners only).
pub fn picture_corners(c: &mut Compositor, rect: Rect, start: f32, end: f32, colors: [Color; 2]) {
    let half = rect.w / 2.0;
    for (x, radius) in [(rect.x, start), (rect.x + half, end)] {
        c.push(SceneNode::PushClip {
            x,
            y: rect.y,
            w: half,
            h: rect.h,
        });
        picture(c, rect, radius, colors);
        c.push(SceneNode::PopClip);
    }
}

/// A hairline `w` long at `(x, y)`, `width` thick.
pub fn hairline(c: &mut Compositor, x: f32, y: f32, w: f32, width: f32, color: Color) {
    c.push(SceneNode::Rect {
        x,
        y,
        w,
        h: width,
        color: color.0,
    });
}

/// A vertical hairline `h` long at `(x, y)`.
pub fn vline(c: &mut Compositor, x: f32, y: f32, h: f32, width: f32, color: Color) {
    c.push(SceneNode::Rect {
        x,
        y,
        w: width,
        h,
        color: color.0,
    });
}

/// How wide `text` draws in `style`.
pub fn text_w(text: &str, style: &TextStyle) -> f32 {
    TextMeasurer::measure_styled(text, style, None).0.ceil()
}

/// `text` from `(x, y)`, cut with an ellipsis past `w`.
pub fn text(c: &mut Compositor, s: &str, style: &TextStyle, x: f32, y: f32, w: f32, color: Color) {
    let shown = TextMeasurer::truncate_to_width(s, style, w.max(0.0));
    c.push(SceneNode::Text {
        key: TextNodeKey::from_style(&shown, style, None),
        x,
        y,
        color: color.0,
    });
}

/// `text` centered on `cx`, cut to `w`.
pub fn text_centered(
    c: &mut Compositor,
    s: &str,
    style: &TextStyle,
    cx: f32,
    y: f32,
    w: f32,
    color: Color,
) {
    let shown = TextMeasurer::truncate_to_width(s, style, w.max(0.0));
    let tw = text_w(&shown, style);
    text(c, &shown, style, cx - tw / 2.0, y, w, color);
}

/// `text` ending at `right`, cut to `w`.
pub fn text_end(
    c: &mut Compositor,
    s: &str,
    style: &TextStyle,
    right: f32,
    y: f32,
    w: f32,
    color: Color,
) {
    let shown = TextMeasurer::truncate_to_width(s, style, w.max(0.0));
    let tw = text_w(&shown, style);
    text(c, &shown, style, right - tw, y, w, color);
}

/// The top of a line of `style` centered in a band `h` tall at `y`.
pub fn middle(style: &TextStyle, y: f32, h: f32) -> f32 {
    y + TextMeasurer::vertical_center(style, h)
}

/// A Lucide glyph `size` square centered in `rect`.
pub fn glyph(c: &mut Compositor, name: &str, size: f32, color: Color, rect: Rect) {
    let x = rect.x + (rect.w - size) / 2.0;
    let y = rect.y + (rect.h - size) / 2.0;
    if let Some(node) = icons::icon_at(name, size, color.0, x, y) {
        c.push(node);
    }
}

/// A rect `w` by `h` centered in `room`.
pub fn center_in(room: Rect, w: f32, h: f32) -> Rect {
    Rect::new(
        room.x + (room.w - w) / 2.0,
        room.y + (room.h - h) / 2.0,
        w,
        h,
    )
}

/// The modal's own focus and its controls': the focus on `id` when one of
/// `controls` takes it.
pub fn focus_all(sets: &mut [&mut Controls], id: Option<&SemId>) -> bool {
    let mut took = false;
    for set in sets {
        took |= set.focus(id);
    }
    took
}
