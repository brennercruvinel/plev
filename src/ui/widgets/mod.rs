//! Retained widgets with internal state.
//!
//! Each widget is a plain struct: callers own it across frames, feed it
//! [`WidgetEvent`]s with the bounds they decided to give it, and call
//! `render(compositor, bounds, theme)` to emit scene nodes. Nothing here
//! touches the GPU — widgets are testable without a window.
//!
//! Visual language: HOFF "dark glass" — monochrome white-on-graphite
//! alphas, pill buttons, top-lit edge borders and translucent surfaces —
//! resolved entirely from [`Theme`] tokens (see [`Theme::hoff`] and
//! `theme.glass`), with [`Intent`](crate::theme::Intent) selecting
//! semantic color + motion physics. Non-HOFF themes derive an equivalent
//! glass recipe, so every widget renders under every palette.

mod button;
mod card;
mod checkbox;
mod context_menu;
mod list;
mod modal;
mod progress;
mod scrollbar;
mod select;
mod slider;
mod switch;
mod tabs;
mod toast;
mod tooltip;
mod tree;

#[cfg(test)]
mod tests;

pub use button::{Button, ButtonSize, ButtonVariant};
pub use card::{Card, CardListRow, CardVariant};
pub use checkbox::Checkbox;
pub use context_menu::{ContextMenu, MenuEntry};
pub use list::VirtualList;
pub use modal::{Modal, ModalAction};
pub use progress::ProgressBar;
pub use scrollbar::Scrollbar;
pub use select::Select;
pub use slider::Slider;
pub use switch::Switch;
pub use tabs::Tabs;
pub use toast::{Toast, ToastManager};
pub use tooltip::Tooltip;
pub use tree::{Tree, TreeNode};

use crate::compositor::SceneNode;
use crate::theme::{Intent, Theme};

// ---------------------------------------------------------------------------
// Geometry
// ---------------------------------------------------------------------------

/// Widget bounds in logical pixels.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    pub fn contains(&self, px: f32, py: f32) -> bool {
        px >= self.x && px <= self.x + self.w && py >= self.y && py <= self.y + self.h
    }

    pub fn center(&self) -> (f32, f32) {
        (self.x + self.w / 2.0, self.y + self.h / 2.0)
    }
}

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

/// Pointer events widgets understand. Coordinates are absolute logical
/// pixels — the same space as render bounds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WidgetEvent {
    MouseMove {
        x: f32,
        y: f32,
    },
    MouseDown {
        x: f32,
        y: f32,
    },
    MouseUp {
        x: f32,
        y: f32,
    },
    /// Wheel/trackpad scroll. Positive `delta` scrolls content down.
    Scroll {
        x: f32,
        y: f32,
        delta: f32,
    },
}

impl WidgetEvent {
    /// Pointer position carried by the event.
    pub fn pos(&self) -> (f32, f32) {
        match *self {
            WidgetEvent::MouseMove { x, y }
            | WidgetEvent::MouseDown { x, y }
            | WidgetEvent::MouseUp { x, y }
            | WidgetEvent::Scroll { x, y, .. } => (x, y),
        }
    }
}

/// What a widget did with an event.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct EventResult {
    /// Event was consumed; don't offer it to widgets underneath.
    pub handled: bool,
    /// Visual or logical state changed — caller should request a frame.
    pub changed: bool,
    /// An activation (click/select/toggle) completed on this event.
    pub clicked: bool,
}

impl EventResult {
    pub const IGNORED: Self = Self {
        handled: false,
        changed: false,
        clicked: false,
    };

    pub fn changed() -> Self {
        Self {
            handled: true,
            changed: true,
            clicked: false,
        }
    }

    pub fn clicked() -> Self {
        Self {
            handled: true,
            changed: true,
            clicked: true,
        }
    }

    pub fn merge(self, other: Self) -> Self {
        Self {
            handled: self.handled || other.handled,
            changed: self.changed || other.changed,
            clicked: self.clicked || other.clicked,
        }
    }
}

// ---------------------------------------------------------------------------
// Shared color helpers
// ---------------------------------------------------------------------------

/// Theme color as `[f32; 4]` with overridden alpha.
pub(crate) fn with_alpha(c: crate::color::Color, a: f32) -> [f32; 4] {
    [c.0[0], c.0[1], c.0[2], a]
}

/// WCAG relative luminance approximation (linear-ish weights are enough
/// for picking a readable foreground).
pub(crate) fn luminance(c: [f32; 4]) -> f32 {
    0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]
}

/// Black or white, whichever reads against `bg` (WCAG AA-driven choice).
pub(crate) fn contrast_text(bg: [f32; 4]) -> [f32; 4] {
    if luminance(bg) > 0.45 {
        [0.02, 0.02, 0.04, 1.0]
    } else {
        [0.98, 0.98, 0.99, 1.0]
    }
}

/// Semantic fill color for an intent: Neutral maps to the theme accent
/// (primary action), others to their semantic color.
pub(crate) fn intent_fill(theme: &Theme, intent: Intent) -> [f32; 4] {
    match intent {
        Intent::Neutral => theme.colors.accent.0,
        Intent::Constructive => theme.colors.success.0,
        Intent::Destructive => theme.colors.danger.0,
        Intent::Informational => theme.colors.info.0,
    }
}

/// Linear interpolation between two RGBA colors.
pub(crate) fn mix(a: [f32; 4], b: [f32; 4], t: f32) -> [f32; 4] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
        a[3] + (b[3] - a[3]) * t,
    ]
}

// ---------------------------------------------------------------------------
// HOFF glass recipe (SDF pipeline)
// ---------------------------------------------------------------------------

/// HOFF edge-light: a border that only exists at the top, fading out
/// downward (the CSS original is a masked 165–178° border).
///
/// Emitted as two SDF nodes: a white→transparent vertical [`GradientRect`]
/// underlay the size of `rect`, then the surface fill inset by `width`.
/// The translucent fill lets the underlay shine through, which doubles as
/// the HOFF inset key-light (`inset 2px 4px 16px rgba(248,248,248,.06)`).
///
/// [`GradientRect`]: crate::compositor::SceneNode::GradientRect
pub fn glass_pill(
    rect: Rect,
    radius: f32,
    edge: [f32; 4],
    width: f32,
    fill: [f32; 4],
) -> [SceneNode; 2] {
    [
        SceneNode::GradientRect {
            x: rect.x,
            y: rect.y,
            w: rect.w,
            h: rect.h,
            color: edge,
            color2: [edge[0], edge[1], edge[2], 0.0],
            // CSS 180deg: first stop (the lit edge) at the top.
            angle_deg: 180.0,
            corner_radius: radius,
            border_width: 0.0,
            border_color: [0.0; 4],
        },
        SceneNode::RoundedRect {
            x: rect.x + width,
            y: rect.y + width,
            w: (rect.w - width * 2.0).max(0.0),
            h: (rect.h - width * 2.0).max(0.0),
            color: fill,
            corner_radius: (radius - width).max(0.0),
            border_width: 0.0,
            border_color: [0.0; 4],
        },
    ]
}

/// Floating-menu drop shadow (HOFF: `0 24px 32px -12px rgba(18,18,18,.10)`
/// over the deep stack). One analytic shadow node approximates the stack.
pub fn menu_shadow(rect: Rect, radius: f32) -> SceneNode {
    SceneNode::Shadow {
        x: rect.x,
        y: rect.y,
        w: rect.w,
        h: rect.h,
        corner_radius: radius,
        blur_radius: 32.0,
        offset: [0.0, 16.0],
        color: [18.0 / 255.0, 18.0 / 255.0, 18.0 / 255.0, 0.35],
        inset: false,
    }
}

// ---------------------------------------------------------------------------
// Rounded-rect node shorthands
// ---------------------------------------------------------------------------

/// Solid rounded rect ([`SceneNode::RoundedRect`] without border). The
/// compositor preserves push order across primitive types, so icons
/// (paths) pushed after this stack on top of it.
pub fn rounded_rect(x: f32, y: f32, w: f32, h: f32, radius: f32, color: [f32; 4]) -> SceneNode {
    SceneNode::RoundedRect {
        x,
        y,
        w,
        h,
        color,
        corner_radius: radius,
        border_width: 0.0,
        border_color: [0.0; 4],
    }
}

/// Border-only rounded rect: transparent fill with an SDF border ring,
/// which composites OVER whatever is underneath -- exactly like a stroked
/// path would (the ring sits inside the bounds, like the SDF border).
pub fn rounded_rect_stroke(
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    radius: f32,
    color: [f32; 4],
    width: f32,
) -> SceneNode {
    SceneNode::RoundedRect {
        x,
        y,
        w,
        h,
        color: [0.0; 4],
        corner_radius: radius,
        border_width: width,
        border_color: color,
    }
}
