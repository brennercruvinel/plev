//! Where an anchored panel goes: against its anchor and the container
//! that receives it, never the window. The order is floating-ui's, as the
//! kit's headlessui runs it (`offset`, `shift`, `flip`, `size`): the panel
//! sits `gap` away from the anchor on the side asked, aligned and skidded
//! by `offset` along the cross axis; it flips to the other side when the
//! side asked cannot hold it and the other can (or holds more of it), it
//! shifts along the cross axis to stay inside, and it is capped to the
//! space it was left, so it never overflows the container.
//!
//! Sides and alignments are logical: `Start` and `End` are left and
//! right while the engine lays out left to right, the only direction it
//! has today.

use crate::widgets::overlay::tokens::Clearance;
use crate::widgets::widget::Rect;

/// The side of the anchor the panel opens on.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Side {
    Top,
    #[default]
    Bottom,
    Start,
    End,
}

impl Side {
    /// The side across the anchor.
    pub fn opposite(self) -> Self {
        match self {
            Side::Top => Side::Bottom,
            Side::Bottom => Side::Top,
            Side::Start => Side::End,
            Side::End => Side::Start,
        }
    }

    /// Whether the panel opens above or below (the main axis is y).
    fn vertical(self) -> bool {
        matches!(self, Side::Top | Side::Bottom)
    }

    /// Whether the panel opens after the anchor along the main axis.
    fn after(self) -> bool {
        matches!(self, Side::Bottom | Side::End)
    }
}

/// Where the panel lines up with the anchor along the cross axis.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Align {
    Start,
    Center,
    #[default]
    End,
}

/// What [`place`] decided.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Placement {
    /// The panel, inside the container (less its edge clearance).
    pub rect: Rect,
    /// The side it opened on, after the flip.
    pub side: Side,
    /// It opened on the other side than the one asked.
    pub flipped: bool,
    /// How far the shift moved it along the cross axis to stay inside
    /// (signed, zero when the aligned position fit).
    pub shift: f32,
    /// It was capped smaller than the size it asked for.
    pub capped: bool,
}

/// One axis of a rect: its start and its length.
#[derive(Clone, Copy)]
struct Span {
    at: f32,
    len: f32,
}

impl Span {
    fn end(self) -> f32 {
        self.at + self.len
    }
}

fn spans(r: Rect, vertical: bool) -> (Span, Span) {
    let x = Span { at: r.x, len: r.w };
    let y = Span { at: r.y, len: r.h };
    if vertical { (y, x) } else { (x, y) }
}

/// The room on each side of the anchor along the main axis, gap taken.
fn room(anchor: Span, area: Span, gap: f32) -> (f32, f32) {
    let before = (anchor.at - gap - area.at).max(0.0);
    let after = (area.end() - anchor.end() - gap).max(0.0);
    (before, after)
}

/// Places a panel of `size` (width, height) by `anchor` inside `bounds`,
/// on `side` and aligned by `align`, keeping `clear`.
pub fn place(
    anchor: Rect,
    size: (f32, f32),
    bounds: Rect,
    side: Side,
    align: Align,
    clear: Clearance,
) -> Placement {
    let vertical = side.vertical();
    let area = bounds.inset(clear.edge);
    let (anchor_main, anchor_cross) = spans(anchor, vertical);
    let (area_main, area_cross) = spans(area, vertical);
    let (want_main, want_cross) = if vertical {
        (size.1, size.0)
    } else {
        (size.0, size.1)
    };

    // Flip: keep the side asked while it holds the panel; take the other
    // when the other holds it, or holds more of it.
    let (before, after) = room(anchor_main, area_main, clear.gap);
    let (asked, other) = if side.after() {
        (after, before)
    } else {
        (before, after)
    };
    let flipped = asked < want_main && other > asked;
    let side = if flipped { side.opposite() } else { side };
    let space = if flipped { other } else { asked };

    // Size: never longer than the room left on the side taken, nor wider
    // than the container.
    let main_len = want_main.min(space);
    let cross_len = want_cross.min(area_cross.len);
    // An anchor scrolled past the container's edge still leaves the
    // panel inside it, against that edge.
    let main_at = if side.after() {
        anchor_main.end() + clear.gap
    } else {
        anchor_main.at - clear.gap - main_len
    }
    .clamp(area_main.at, (area_main.end() - main_len).max(area_main.at));

    // Align and skid, then shift inside the container.
    let aligned = match align {
        Align::Start => anchor_cross.at,
        Align::Center => anchor_cross.at + (anchor_cross.len - cross_len) / 2.0,
        Align::End => anchor_cross.end() - cross_len,
    } + clear.offset;
    let lo = area_cross.at;
    let hi = (area_cross.end() - cross_len).max(lo);
    let cross_at = aligned.clamp(lo, hi);

    let rect = if vertical {
        Rect::new(cross_at, main_at, cross_len, main_len)
    } else {
        Rect::new(main_at, cross_at, main_len, cross_len)
    };
    Placement {
        rect,
        side,
        flipped,
        shift: cross_at - aligned,
        capped: main_len < want_main || cross_len < want_cross,
    }
}
