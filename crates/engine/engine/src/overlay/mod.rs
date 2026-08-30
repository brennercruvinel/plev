mod types;

#[cfg(test)]
mod tests;

use crate::animation::Spring;
use crate::theme::MotionPhysics;
use types::OverlayAnim;

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
            anim: None,
        });
        id
    }

    /// Push an overlay with an entry animation (fade + scale 0.96 -> 1.0).
    ///
    /// `motion` defines the spring physics; resolve it from the content's
    /// intent so e.g. destructive modals snap in faster:
    /// `theme.intent_motion(Intent::Destructive)`.
    ///
    /// Drive the animation with [`tick`] every frame and apply
    /// [`Overlay::opacity`] / [`Overlay::scale`] when rendering.
    ///
    /// [`tick`]: OverlayManager::tick
    pub fn push_animated(
        &mut self,
        kind: OverlayKind,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        motion: &MotionPhysics,
    ) -> OverlayId {
        let id = self.push(kind, x, y, w, h);
        let mut spring = Spring::new(0.0_f32).with_motion(motion);
        spring.set_target(1.0);
        // push() always appends, so last_mut() is the overlay just created.
        self.stack
            .last_mut()
            .expect("push appended an overlay")
            .anim = Some(OverlayAnim {
            spring,
            closing: false,
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

    /// Begin the exit animation of the topmost overlay that is not already
    /// closing. The overlay stays on the stack (still rendered, fading out)
    /// until its spring settles inside [`tick`].
    ///
    /// Overlays pushed without animation get default exit physics; ones
    /// pushed via [`push_animated`] reuse their intent-derived spring.
    ///
    /// [`tick`]: OverlayManager::tick
    /// [`push_animated`]: OverlayManager::push_animated
    pub fn pop_animated(&mut self) -> Option<OverlayId> {
        let overlay = self.stack.iter_mut().rev().find(|o| !o.is_closing())?;
        let id = overlay.id;
        Self::begin_close(overlay);
        Some(id)
    }

    /// Begin the exit animation of the overlay with `id`. No-op if not found
    /// or already closing.
    pub fn pop_id_animated(&mut self, id: OverlayId) {
        if let Some(o) = self
            .stack
            .iter_mut()
            .find(|o| o.id == id && !o.is_closing())
        {
            Self::begin_close(o);
        }
    }

    fn begin_close(overlay: &mut Overlay) {
        let anim = overlay.anim.get_or_insert_with(|| OverlayAnim {
            // Pushed without animation: fade out with the engine's default
            // motion (mirrors Theme::dark().motion).
            spring: Spring::new(1.0_f32).with_config(170.0, 26.0, 1.0),
            closing: false,
        });
        anim.closing = true;
        anim.spring.set_target(0.0);
    }

    /// Advance overlay animations by `dt` seconds and drop overlays whose
    /// exit animation finished. Returns `true` while anything is still
    /// animating (callers keep requesting frames while it does).
    pub fn tick(&mut self, dt: f32) -> bool {
        let mut removed = false;
        self.stack.retain_mut(|o| {
            let Some(anim) = o.anim.as_mut() else {
                return true;
            };
            anim.spring.tick(dt);
            let done = anim.closing && !anim.spring.is_animating();
            if done {
                removed = true;
            }
            !done
        });
        if removed {
            self.reassign_z();
        }
        self.is_animating()
    }

    /// `true` while any overlay's entry/exit spring is still moving.
    pub fn is_animating(&self) -> bool {
        self.stack.iter().any(|o| o.is_animating())
    }

    /// Returns the topmost overlay, if any.
    pub fn top(&self) -> Option<&Overlay> {
        self.stack.last()
    }

    /// Returns the topmost overlay that is not fading out. Use this for
    /// input routing so closing overlays no longer receive events.
    pub fn top_active(&self) -> Option<&Overlay> {
        self.stack.iter().rev().find(|o| !o.is_closing())
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
    /// # use engine::overlay::{OverlayManager, OverlayKind, MenuItem};
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
