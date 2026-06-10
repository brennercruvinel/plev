//! Retained widgets with internal state.
//!
//! Each widget is a plain struct: callers own it across frames, feed it
//! [`WidgetEvent`]s with the bounds they decided to give it, and call
//! `render(compositor, bounds, theme)` to emit scene nodes. Nothing here
//! touches the GPU — widgets are testable without a window.
//!
//! Visual language: shadcn/ui-inspired (neutral surfaces, `radius.md`,
//! subtle 1px borders, explicit hover/active/disabled states) resolved
//! entirely from [`Theme`] tokens, with [`Intent`](crate::theme::Intent)
//! selecting semantic color + motion physics.

mod button;
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

/// Multiply RGB, keep alpha (hover/active shading).
pub(crate) fn scale_rgb(c: [f32; 4], f: f32) -> [f32; 4] {
    [
        (c[0] * f).clamp(0.0, 1.0),
        (c[1] * f).clamp(0.0, 1.0),
        (c[2] * f).clamp(0.0, 1.0),
        c[3],
    ]
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

/// Shade a base color for hover/active: light colors darken, dark colors
/// lighten, so the state change is visible on any palette.
pub(crate) fn shade(c: [f32; 4], strength: f32) -> [f32; 4] {
    if luminance(c) > 0.5 {
        scale_rgb(c, 1.0 - strength)
    } else {
        [
            (c[0] + strength * 0.8).clamp(0.0, 1.0),
            (c[1] + strength * 0.8).clamp(0.0, 1.0),
            (c[2] + strength * 0.8).clamp(0.0, 1.0),
            c[3],
        ]
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
