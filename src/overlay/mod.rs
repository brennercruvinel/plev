mod types;

#[cfg(test)]
mod tests;

pub use types::{MenuItem, Overlay, OverlayId, OverlayKind};

/// Z-ordered stack of overlays (context menus, modals, tooltips).
///
/// **Pure data** -- no GPU references.  Callers render overlays by iterating
/// [`stack`] and pushing `SceneNode`s into Compositor layers whose z_order
/// matches each [`Overlay::z_order`].
///
/// # Dismiss semantics
///
/// - **Click-outside**: call [`hit_test_outside`]; if `true`, call [`pop`] or
///   [`pop_all`].
/// - **Escape**: call [`pop`] to close the topmost overlay.
///
/// [`stack`]: OverlayManager::stack
/// [`hit_test_outside`]: OverlayManager::hit_test_outside
/// [`pop`]: OverlayManager::pop
/// [`pop_all`]: OverlayManager::pop_all
pub struct OverlayManager {
    /// Ordered from bottom (index 0) to top (last).
    pub stack: Vec<Overlay>,
    next_id: u64,
    base_z: i32,
}

impl Default for OverlayManager {
    fn default() -> Self {
        Self::new()
    }
}

impl OverlayManager {
    /// All overlays use z_order >= `BASE_Z`, keeping them above main UI layers.
    pub const BASE_Z: i32 = 1000;

    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            next_id: 1,
            base_z: Self::BASE_Z,
        }
    }

    /// Push a new overlay onto the stack.
    ///
    /// `w`/`h` may be `0.0` when dimensions are not yet known (e.g. content-
    /// sized modals).  Use [`set_bounds`] after the first render to fill them in
    /// so that [`hit_test_outside`] works correctly.
    ///
    /// [`set_bounds`]: OverlayManager::set_bounds
    /// [`hit_test_outside`]: OverlayManager::hit_test_outside
    pub fn push(&mut self, kind: OverlayKind, x: f32, y: f32, w: f32, h: f32) -> OverlayId {
        let id = OverlayId(self.next_id);
        self.next_id += 1;
        let z_order =
            self.base_z + i32::try_from(self.stack.len()).expect("overlay stack exceeds i32 range");
        self.stack.push(Overlay {
            id,
            kind,
            x,
            y,
            w,
            h,
            z_order,
        });
        id
    }

    /// Update pixel bounds once layout is known (e.g. after first render pass).
    pub fn set_bounds(&mut self, id: OverlayId, w: f32, h: f32) {
        if let Some(o) = self.stack.iter_mut().find(|o| o.id == id) {
            o.w = w;
            o.h = h;
        }
    }

    /// Remove the topmost overlay and return its id.
    pub fn pop(&mut self) -> Option<OverlayId> {
        self.stack.pop().map(|o| o.id)
    }

    /// Remove the overlay with `id`. No-op if not found.
    /// Z-orders are reassigned after removal to stay contiguous.
    pub fn pop_id(&mut self, id: OverlayId) {
        self.stack.retain(|o| o.id != id);
        self.reassign_z();
    }

    /// Remove all overlays.
    pub fn pop_all(&mut self) {
        self.stack.clear();
    }

    /// Returns the topmost overlay, if any.
    pub fn top(&self) -> Option<&Overlay> {
        self.stack.last()
    }

    /// `true` when no overlays are active.
    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    /// Number of active overlays.
    pub fn len(&self) -> usize {
        self.stack.len()
    }

    /// Returns `true` if `(px, py)` falls **outside** every overlay's bounding
    /// box.  Overlays with zero-size bounds are skipped (bounds not yet known).
    ///
    /// Use this to decide whether a click should dismiss the overlay stack:
    /// ```rust
    /// # use plev::overlay::{OverlayManager, OverlayKind, MenuItem};
    /// # let mut mgr = OverlayManager::new();
    /// # mgr.push(OverlayKind::ContextMenu { items: vec![] }, 10.0, 10.0, 100.0, 80.0);
    /// if mgr.hit_test_outside(5.0, 5.0) {
    ///     mgr.pop_all();
    /// }
    /// ```
    pub fn hit_test_outside(&self, px: f32, py: f32) -> bool {
        !self.stack.iter().any(|o| {
            o.w > 0.0 && o.h > 0.0 && px >= o.x && px <= o.x + o.w && py >= o.y && py <= o.y + o.h
        })
    }

    fn reassign_z(&mut self) {
        for (i, o) in self.stack.iter_mut().enumerate() {
            o.z_order = self.base_z + i32::try_from(i).expect("overlay index exceeds i32 range");
        }
    }
}
