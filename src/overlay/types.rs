/// A unique identifier for an overlay instance.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct OverlayId(pub(super) u64);

/// A single item in a context menu.
#[derive(Clone, Debug)]
pub struct MenuItem {
    /// Display label.
    pub label: String,
    /// Opaque id passed back via action dispatch when this item is selected.
    pub id: u64,
}

impl MenuItem {
    pub fn new(label: impl Into<String>, id: u64) -> Self {
        Self {
            label: label.into(),
            id,
        }
    }
}

/// Variant-specific data for each overlay type.
#[derive(Clone, Debug)]
pub enum OverlayKind {
    /// A list of labelled actions (right-click menu).
    ContextMenu { items: Vec<MenuItem> },
    /// A blocking dialog with confirm and cancel buttons.
    Modal {
        title: String,
        body: String,
        confirm: String,
        cancel: String,
    },
    /// A brief informational label (no interaction).
    Tooltip { text: String },
}

/// Entry/exit animation state for an overlay (fade + scale driven by a
/// [`Spring`] whose physics come from the content's [`Intent`]).
///
/// [`Spring`]: crate::animation::Spring
/// [`Intent`]: crate::theme::Intent
#[derive(Debug, Clone)]
pub(super) struct OverlayAnim {
    /// Progress 0.0 (hidden) -> 1.0 (fully shown).
    pub(super) spring: crate::animation::Spring<f32>,
    /// When true the spring is heading back to 0.0; the overlay is removed
    /// once it settles.
    pub(super) closing: bool,
}

/// A single entry in the overlay stack.
#[derive(Debug, Clone)]
pub struct Overlay {
    pub id: OverlayId,
    pub kind: OverlayKind,
    /// Top-left position in screen (logical pixel) coordinates.
    pub x: f32,
    pub y: f32,
    /// Pixel dimensions. `0.0` means unknown; call
    /// [`OverlayManager::set_bounds`] after the first render computes size.
    pub w: f32,
    pub h: f32,
    /// Compositor layer z_order for this overlay.
    /// Monotonically increasing -- the topmost overlay has the highest value.
    pub z_order: i32,
    /// `None` for overlays pushed without animation (always fully shown).
    pub(super) anim: Option<OverlayAnim>,
}

impl Overlay {
    /// Animation progress in 0.0..=1.0 (1.0 when not animated).
    pub fn progress(&self) -> f32 {
        match &self.anim {
            Some(a) => a.spring.get().clamp(0.0, 1.0),
            None => 1.0,
        }
    }

    /// Opacity the renderer should apply (e.g. via layer opacity).
    pub fn opacity(&self) -> f32 {
        self.progress()
    }

    /// Scale the renderer should apply around the overlay center:
    /// 0.96 when hidden -> 1.0 when fully shown.
    pub fn scale(&self) -> f32 {
        0.96 + 0.04 * self.progress()
    }

    /// `true` while the exit animation is running. Closing overlays should
    /// not receive input.
    pub fn is_closing(&self) -> bool {
        self.anim.as_ref().is_some_and(|a| a.closing)
    }

    /// `true` while the entry/exit spring has not settled.
    pub fn is_animating(&self) -> bool {
        self.anim.as_ref().is_some_and(|a| a.spring.is_animating())
    }
}
