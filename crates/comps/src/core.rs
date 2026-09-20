//! The widget contract: geometry, events, results and the color helpers
//! every category shares.
//!
//! A widget is a plain struct the app owns across frames. It receives
//! [`WidgetEvent`]s in absolute logical pixels together with the bounds the
//! app decided to give it, answers with an [`EventResult`], and renders by
//! pushing scene nodes. No GPU, no window, no global state.

use engine::color::Color;
use engine::theme::{Intent, Theme};

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

    /// Right edge.
    pub fn right(&self) -> f32 {
        self.x + self.w
    }

    /// Bottom edge.
    pub fn bottom(&self) -> f32 {
        self.y + self.h
    }

    /// The rect shrunk by `d` on every side (never negative).
    pub fn inset(&self, d: f32) -> Self {
        Self {
            x: self.x + d,
            y: self.y + d,
            w: (self.w - d * 2.0).max(0.0),
            h: (self.h - d * 2.0).max(0.0),
        }
    }

    /// The rect grown by `d` on every side.
    pub fn outset(&self, d: f32) -> Self {
        self.inset(-d)
    }
}

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

/// Pointer events widgets understand. Coordinates are absolute logical
/// pixels, the same space as render bounds. Touch arrives as these too
/// (the engine synthesizes pointer events from touches).
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
    /// Visual or logical state changed: the caller must request a frame.
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
pub fn with_alpha(c: Color, a: f32) -> [f32; 4] {
    [c.0[0], c.0[1], c.0[2], a]
}

/// WCAG relative luminance approximation (linear-ish weights are enough
/// for picking a readable foreground).
pub fn luminance(c: [f32; 4]) -> f32 {
    0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]
}

/// Black or white, whichever reads against `bg` (WCAG AA-driven choice).
pub fn contrast_text(bg: [f32; 4]) -> [f32; 4] {
    if luminance(bg) > 0.45 {
        [0.02, 0.02, 0.04, 1.0]
    } else {
        [0.98, 0.98, 0.99, 1.0]
    }
}

/// Semantic fill color for an intent: Neutral maps to the theme accent
/// (primary action), others to their semantic color.
pub fn intent_fill(theme: &Theme, intent: Intent) -> [f32; 4] {
    match intent {
        Intent::Neutral => theme.colors.accent.0,
        Intent::Constructive => theme.colors.success.0,
        Intent::Destructive => theme.colors.danger.0,
        Intent::Informational => theme.colors.info.0,
    }
}

/// Linear interpolation between two RGBA colors.
pub fn mix(a: [f32; 4], b: [f32; 4], t: f32) -> [f32; 4] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
        a[3] + (b[3] - a[3]) * t,
    ]
}
