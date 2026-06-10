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
}
