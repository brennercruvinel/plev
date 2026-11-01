//! The area a press lands in, as large as a person needs, whatever the
//! drawing (WCAG 2.2, 2.5.8 Target Size (Minimum), AA). A control drawn
//! smaller than the minimum (a 20 px close dot, a 10 px split divider, a
//! 6 px scrollbar thumb) keeps its look and gets a press area grown
//! around its centre to `theme.control.min_target` on each axis, or to
//! `theme.control.min_touch_target` while the last input was a finger
//! ([`set_touch_input`]). The widget hit-tests and binds that rect; it
//! draws the visual one.
//!
//! The modality is this thread's, like reduced motion
//! (`compose::animation::set_reduced_motion`): the shell sets it from
//! the platform's last input (a touch, or a moving pointer), and every
//! widget reads it through [`min_target`] when it lays out its press
//! areas.

use std::cell::Cell;

use super::Rect;
use crate::styling::theme::Theme;

thread_local! {
    static TOUCH: Cell<bool> = const { Cell::new(false) };
}

/// The last input on this thread was a finger (`true`) or a pointer:
/// press areas grow to the touch minimum while it is a finger.
pub fn set_touch_input(on: bool) {
    TOUCH.with(|t| t.set(on));
}

/// Whether press areas are sized for a finger now ([`set_touch_input`]).
pub fn touch_input() -> bool {
    TOUCH.with(Cell::get)
}

/// The two minima a widget keeps when it hit-tests without the theme at
/// hand (its event handler): read from the theme once, chosen by the
/// modality at each press ([`MinTarget::now`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MinTarget {
    pub pointer: f32,
    pub touch: f32,
}

impl MinTarget {
    pub fn of(theme: &Theme) -> Self {
        let pointer = theme.control.min_target;
        Self {
            pointer,
            touch: theme.control.min_touch_target.max(pointer),
        }
    }

    /// The minimum for the input now.
    pub fn now(self) -> f32 {
        if touch_input() {
            self.touch
        } else {
            self.pointer
        }
    }

    /// `visual` grown to the minimum now ([`hit_rect`]).
    pub fn hit(self, visual: Rect) -> Rect {
        grow(visual, self.now())
    }
}

/// The press minimum a widget learned from the theme at its last render,
/// for its event handler, which has no theme: a person presses what the
/// last frame showed. Before any render it grows nothing.
#[derive(Clone, Debug, Default)]
pub struct HitArea(Cell<Option<MinTarget>>);

impl HitArea {
    /// Keeps the theme's minima (call it in `render`).
    pub fn remember(&self, theme: &Theme) {
        self.0.set(Some(MinTarget::of(theme)));
    }

    /// `visual` grown to the minimum now, once a render taught it.
    pub fn grow(&self, visual: Rect) -> Rect {
        self.0.get().map_or(visual, |m| m.hit(visual))
    }
}

/// The smallest press area on each axis now: the pointer's minimum, or
/// the finger's while the input is touch.
pub fn min_target(theme: &Theme) -> f32 {
    MinTarget::of(theme).now()
}

/// The press area of a control drawn at `visual`: the same rect where it
/// is already as large as [`min_target`] on an axis, else grown on that
/// axis around the same centre. Never smaller than `visual`.
pub fn hit_rect(visual: Rect, theme: &Theme) -> Rect {
    grow(visual, min_target(theme))
}

/// `visual` grown around its centre to at least `min` on each axis.
pub fn grow(visual: Rect, min: f32) -> Rect {
    let w = visual.w.max(min);
    let h = visual.h.max(min);
    Rect::new(
        visual.x - (w - visual.w) / 2.0,
        visual.y - (h - visual.h) / 2.0,
        w,
        h,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_small_control_presses_in_the_minimum_around_its_centre() {
        let theme = Theme::default();
        set_touch_input(false);
        let dot = Rect::new(100.0, 50.0, 20.0, 20.0);
        let hit = hit_rect(dot, &theme);
        assert_eq!(theme.control.min_target, 24.0);
        assert_eq!((hit.w, hit.h), (24.0, 24.0));
        assert_eq!(hit.center(), dot.center(), "the same centre");
        // a large one keeps its rect, an axis at a time
        let row = Rect::new(0.0, 0.0, 200.0, 10.0);
        let hit = hit_rect(row, &theme);
        assert_eq!((hit.x, hit.w), (0.0, 200.0));
        assert_eq!((hit.y, hit.h), (-7.0, 24.0));
        let big = Rect::new(1.0, 2.0, 40.0, 30.0);
        assert_eq!(hit_rect(big, &theme), big);
    }

    #[test]
    fn a_finger_presses_in_the_touch_minimum() {
        let theme = Theme::default();
        set_touch_input(true);
        let hit = hit_rect(Rect::new(0.0, 0.0, 20.0, 30.0), &theme);
        set_touch_input(false);
        assert_eq!(theme.control.min_touch_target, 44.0);
        assert_eq!((hit.w, hit.h), (44.0, 44.0));
        assert!(!touch_input(), "back to the pointer");
        assert_eq!(min_target(&theme), 24.0);
    }

    #[test]
    fn a_hit_area_grows_once_a_render_taught_it() {
        let theme = Theme::default();
        set_touch_input(false);
        let area = HitArea::default();
        let knob = Rect::new(0.0, 0.0, 40.0, 22.0);
        assert_eq!(area.grow(knob), knob, "nothing learned yet");
        area.remember(&theme);
        assert_eq!(area.grow(knob), Rect::new(0.0, -1.0, 40.0, 24.0));
    }
}
