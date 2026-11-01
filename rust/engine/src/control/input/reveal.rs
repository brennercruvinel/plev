//! Focus into view (WCAG 2.4.11, Focus Not Obscured): when the keyboard,
//! a screen reader or an agent moves the focus to a control outside the
//! part of a scrolling container a person sees, the container scrolls
//! until it shows, the least that does (the web's
//! `scrollIntoView({block: "nearest"})`), with a margin around the control
//! so its ring breathes, and never behind a band fixed over the
//! container's edge (a header, a docked bar: the web's `scroll-padding`).
//!
//! The geometry is one axis: a [`Span`] of the control and of the
//! container's window, in the same space (logical window pixels, y down
//! for the vertical axis). [`nearest`] says how far to scroll;
//! [`ScrollState::reveal`] applies it within the content's limits;
//! [`reveal_nested`] walks a chain of containers, the innermost first, as
//! the browser does: each one shows the control in its window, and the
//! next one out shows it in its own.
//!
//! The motion is the theme's ([`Glide`]): a spring with `theme.motion`,
//! and none with the thread's motion reduced. [`FocusReveal`] decides when:
//! a focus that moved to another control, not by the pointer (a click
//! lands where the person is already looking).

use accord::SemId;

use super::scroll::ScrollState;
use crate::compose::animation::{self, Spring};
use crate::styling::theme::MotionPhysics;

/// A stretch of one axis: where it starts and how long it is.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Span {
    pub start: f32,
    pub len: f32,
}

impl Span {
    pub fn new(start: f32, len: f32) -> Self {
        Self {
            start,
            len: len.max(0.0),
        }
    }

    pub fn end(&self) -> f32 {
        self.start + self.len
    }

    /// The span moved by `by` along its axis.
    pub fn shifted(self, by: f32) -> Self {
        Self::new(self.start + by, self.len)
    }
}

/// What covers a container's window at its edges and stays put while it
/// scrolls (a sticky header at the start, a docked bar at the end): the
/// control must show clear of it.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Cover {
    pub start: f32,
    pub end: f32,
}

/// How far a container scrolls to show `target` in `view` (positive: the
/// content moves up, as [`ScrollState::scroll_by`]), with `margin` around
/// it, clear of `cover`. Zero when it already shows there. A window too
/// short to hold the target with its margin drops the margin; one with no
/// room left beside the cover drops the cover too. A target longer than
/// every window aligns its start (with the margin), unless it already
/// spans the whole window.
pub fn nearest(target: Span, view: Span, margin: f32, cover: Cover) -> f32 {
    let clear = |inset_start: f32, inset_end: f32| {
        let start = view.start + inset_start;
        let end = view.end() - inset_end;
        (end > start).then(|| Span::new(start, end - start))
    };
    let margin = margin.max(0.0);
    let (cs, ce) = (cover.start.max(0.0), cover.end.max(0.0));
    let breathing = clear(cs + margin, ce + margin);
    let covered = clear(cs, ce);
    let window = match (breathing, covered) {
        (Some(w), _) if w.len >= target.len => w,
        (_, Some(w)) if w.len >= target.len => w,
        (Some(w), _) | (None, Some(w)) => w,
        (None, None) => view,
    };
    let above = target.start < window.start;
    let below = target.end() > window.end();
    if target.len > window.len {
        let spans = target.start <= window.start && target.end() >= window.end();
        return if spans {
            0.0
        } else {
            target.start - window.start
        };
    }
    if above {
        target.start - window.start
    } else if below {
        target.end() - window.end()
    } else {
        0.0
    }
}

impl ScrollState {
    /// Scrolls the least that shows `target` in `view`, the window this
    /// state scrolls (both in the same space, see [`nearest`]), within the
    /// content's limits. The scroll it took: the target has moved by
    /// minus that much.
    pub fn reveal(&mut self, target: Span, view: Span, margin: f32, cover: Cover) -> f32 {
        let before = self.offset();
        self.scroll_by(nearest(target, view, margin, cover));
        self.offset() - before
    }

    /// Where [`ScrollState::reveal`] would leave the offset, without
    /// scrolling: the target a [`Glide`] goes to.
    pub fn revealed(&self, target: Span, view: Span, margin: f32, cover: Cover) -> f32 {
        let mut at = self.clone();
        at.reveal(target, view, margin, cover);
        at.offset()
    }
}

/// One scrolling container of a chain: its state, the window it shows
/// (in the space of the target), and what covers that window's edges.
pub struct Scroller<'a> {
    pub scroll: &'a mut ScrollState,
    pub view: Span,
    pub cover: Cover,
}

/// Shows `target` through every container of `chain`, the innermost
/// first: each scrolls the least that shows the target in its window, and
/// the target moves with it before the next one out. The total scroll.
pub fn reveal_nested(target: Span, chain: &mut [Scroller<'_>], margin: f32) -> f32 {
    let mut at = target;
    let mut total = 0.0;
    for c in chain.iter_mut() {
        let moved = c.scroll.reveal(at, c.view, margin, c.cover);
        at = at.shifted(-moved);
        total += moved;
    }
    total
}

/// A scroll that glides to an offset with the theme's motion, and lands
/// at once when the thread's motion is reduced
/// (`animation::reduced_motion`). It drives a [`ScrollState`] it does
/// not own, so a container keeps its one state for the wheel and the
/// scrollbar.
#[derive(Clone, Debug, Default)]
pub struct Glide {
    spring: Option<Spring<f32>>,
    target: f32,
}

/// How near the offset counts as there, in logical pixels: well under
/// what a display shows. The spring's own rest test is absolute (a
/// thousandth of a pixel) and an `f32` offset thousands of pixels down
/// rounds coarser than that, so the spring can hover a rounding step
/// away forever; the glide lands once it is this close and this still.
const LANDED: f32 = 0.05;

impl Glide {
    /// Glides `scroll` to `offset` (clamped to its limits). With the
    /// motion reduced, `scroll` is there already when this returns.
    pub fn to(&mut self, scroll: &mut ScrollState, offset: f32, motion: &MotionPhysics) {
        let target = offset.clamp(0.0, scroll.max_offset());
        if animation::reduced_motion() || target == scroll.offset() {
            scroll.scroll_to(target);
            self.spring = None;
            return;
        }
        let mut spring = self
            .spring
            .take()
            .unwrap_or_else(|| Spring::new(scroll.offset()).with_motion(motion));
        spring.set_target(target);
        self.spring = Some(spring);
        self.target = target;
    }

    /// Advances the glide by `dt` seconds onto `scroll`. `true` while it
    /// still moves.
    pub fn tick(&mut self, scroll: &mut ScrollState, dt: f32) -> bool {
        let Some(spring) = self.spring.as_mut() else {
            return false;
        };
        let before = spring.get();
        spring.tick(dt);
        let now = spring.get();
        let still = (now - before).abs() < LANDED && (now - self.target).abs() < LANDED;
        if spring.is_animating() && !still {
            scroll.scroll_to(now);
            return true;
        }
        scroll.scroll_to(self.target);
        self.spring = None;
        false
    }

    /// The person took the scroll (the wheel, the scrollbar): the glide
    /// lets go where it is.
    pub fn stop(&mut self) {
        self.spring = None;
    }

    pub fn is_gliding(&self) -> bool {
        self.spring.is_some()
    }
}

/// When the focus asks a container to show it: it moved to another
/// control, and not by the pointer. The owner reports the focus it
/// projects each frame ([`FocusReveal::moved`]) and the pointer going
/// down ([`FocusReveal::pointer`]).
#[derive(Clone, Debug, Default)]
pub struct FocusReveal {
    seen: Option<SemId>,
    pointer: bool,
}

impl FocusReveal {
    pub fn new() -> Self {
        Self::default()
    }

    /// The pointer went down: the focus it moves until the next
    /// [`FocusReveal::moved`] is where the person already looks.
    pub fn pointer(&mut self) {
        self.pointer = true;
    }

    /// The focus as it stands now. The control to show when it moved to
    /// another one by anything but the pointer; `None` when it stayed,
    /// went nowhere, or the pointer moved it.
    pub fn moved(&mut self, focused: Option<&SemId>) -> Option<SemId> {
        let by_pointer = std::mem::take(&mut self.pointer);
        if self.seen.as_ref() == focused {
            return None;
        }
        self.seen = focused.cloned();
        if by_pointer {
            return None;
        }
        self.seen.clone()
    }

    /// The control shown last, so a container that was not ready to show
    /// it (not laid out yet) can ask again.
    pub fn seen(&self) -> Option<&SemId> {
        self.seen.as_ref()
    }

    /// Forgets the focus (the page changed): the focus the next
    /// [`FocusReveal::moved`] finds is taken as it stands, not shown (a
    /// page shows where it was left).
    pub fn reset(&mut self) {
        self.seen = None;
        self.pointer = true;
    }
}

#[cfg(test)]
mod tests;
