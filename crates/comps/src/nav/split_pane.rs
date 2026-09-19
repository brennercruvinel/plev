//! Split pane: two content regions separated by a draggable divider.
//! The widget owns only the divider's state (ratio, hover, drag); the
//! owner renders the two content panes itself into [`SplitPane::first_rect`]
//! and [`SplitPane::second_rect`]. The ratio persists across resizes
//! (desired vs effective: the px clamps apply at read time, so shrinking
//! the window never destroys the user's ratio).
//!
//! Cursor feedback: widgets can't touch the OS cursor (the window belongs
//! to the app shell). Owners that want the resize cursor should read
//! [`SplitPane::is_hovered`]/[`is_dragging`](SplitPane::is_dragging) and
//! set it themselves.

use crate::compositor::Compositor;
use crate::theme::Theme;

use super::{EventResult, Rect, WidgetEvent, rounded_rect};

/// Divider thickness; the hit area extends [`GRAB_MARGIN`] px on each
/// side so the 2px line isn't a precision game.
const DIVIDER: f32 = 2.0;
const GRAB_MARGIN: f32 = 4.0;

/// Which axis the split runs along.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SplitDirection {
    /// Left | right panes, vertical divider (drag moves along x).
    #[default]
    Horizontal,
    /// Top / bottom panes, horizontal divider (drag moves along y).
    Vertical,
}

/// Two-pane split with a draggable divider. See the module docs for the
/// ownership contract.
#[derive(Clone, Debug)]
pub struct SplitPane {
    pub direction: SplitDirection,
    /// Desired split ratio (0..=1), first pane's share. Never clamped by
    /// px minimums — those apply at rect time.
    ratio: f32,
    /// Minimum pixel size of each pane (clamped at read time).
    pub min_first: f32,
    pub min_second: f32,
    hovered: bool,
    dragging: bool,
}

impl SplitPane {
    pub fn new(direction: SplitDirection, ratio: f32) -> Self {
        Self {
            direction,
            ratio: ratio.clamp(0.0, 1.0),
            min_first: 80.0,
            min_second: 80.0,
            hovered: false,
            dragging: false,
        }
    }

    pub fn ratio(&self) -> f32 {
        self.ratio
    }

    pub fn set_ratio(&mut self, ratio: f32) {
        self.ratio = ratio.clamp(0.0, 1.0);
    }

    pub fn is_hovered(&self) -> bool {
        self.hovered
    }

    pub fn is_dragging(&self) -> bool {
        self.dragging
    }

    /// Effective first-pane size in px, clamped by the px minimums (the
    /// clamps win over the ratio when the bounds are small; if both
    /// minimums don't fit, they share proportionally).
    fn first_size(&self, bounds: Rect) -> f32 {
        let total = match self.direction {
            SplitDirection::Horizontal => bounds.w,
            SplitDirection::Vertical => bounds.h,
        } - DIVIDER;
        let total = total.max(0.0);
        let (min_a, min_b) = (self.min_first.min(total), self.min_second.min(total));
        let raw = total * self.ratio;
        if total < min_a + min_b {
            // Both minimums cannot fit: share proportionally.
            return total * min_a / (min_a + min_b).max(1.0);
        }
        raw.clamp(min_a, total - min_b)
    }

    /// First pane rect.
    pub fn first_rect(&self, bounds: Rect) -> Rect {
        let first = self.first_size(bounds);
        match self.direction {
            SplitDirection::Horizontal => Rect::new(bounds.x, bounds.y, first, bounds.h),
            SplitDirection::Vertical => Rect::new(bounds.x, bounds.y, bounds.w, first),
        }
    }

    /// Divider rect (the visible 2px line, not the grab margin).
    pub fn divider_rect(&self, bounds: Rect) -> Rect {
        let first = self.first_size(bounds);
        match self.direction {
            SplitDirection::Horizontal => Rect::new(bounds.x + first, bounds.y, DIVIDER, bounds.h),
            SplitDirection::Vertical => Rect::new(bounds.x, bounds.y + first, bounds.w, DIVIDER),
        }
    }

    /// Second pane rect.
    pub fn second_rect(&self, bounds: Rect) -> Rect {
        let d = self.divider_rect(bounds);
        match self.direction {
            SplitDirection::Horizontal => Rect::new(
                d.x + DIVIDER,
                bounds.y,
                (bounds.x + bounds.w - d.x - DIVIDER).max(0.0),
                bounds.h,
            ),
            SplitDirection::Vertical => Rect::new(
                bounds.x,
                d.y + DIVIDER,
                bounds.w,
                (bounds.y + bounds.h - d.y - DIVIDER).max(0.0),
            ),
        }
    }

    /// The grab area around the divider (hit target).
    fn grab_rect(&self, bounds: Rect) -> Rect {
        let d = self.divider_rect(bounds);
        match self.direction {
            SplitDirection::Horizontal => {
                Rect::new(d.x - GRAB_MARGIN, d.y, DIVIDER + GRAB_MARGIN * 2.0, d.h)
            }
            SplitDirection::Vertical => {
                Rect::new(d.x, d.y - GRAB_MARGIN, d.w, DIVIDER + GRAB_MARGIN * 2.0)
            }
        }
    }

    pub fn handle_event(&mut self, event: &WidgetEvent, bounds: Rect) -> EventResult {
        match *event {
            WidgetEvent::MouseMove { x, y } => {
                let mut result = EventResult::IGNORED;
                let inside = self.grab_rect(bounds).contains(x, y);
                if inside != self.hovered {
                    self.hovered = inside;
                    result = EventResult::changed();
                }
                if self.dragging {
                    let total = match self.direction {
                        SplitDirection::Horizontal => bounds.w,
                        SplitDirection::Vertical => bounds.h,
                    } - DIVIDER;
                    let pos = match self.direction {
                        SplitDirection::Horizontal => x - bounds.x,
                        SplitDirection::Vertical => y - bounds.y,
                    };
                    let old = self.ratio;
                    self.ratio = (pos / total.max(1.0)).clamp(0.0, 1.0);
                    if self.ratio != old {
                        result = result.merge(EventResult::changed());
                    }
                    return result.merge(EventResult {
                        handled: true,
                        ..EventResult::IGNORED
                    });
                }
                result
            }
            WidgetEvent::MouseDown { x, y } => {
                if self.grab_rect(bounds).contains(x, y) {
                    self.dragging = true;
                    EventResult::changed()
                } else {
                    EventResult::IGNORED
                }
            }
            WidgetEvent::MouseUp { .. } => {
                if self.dragging {
                    self.dragging = false;
                    EventResult::changed()
                } else {
                    EventResult::IGNORED
                }
            }
            WidgetEvent::Scroll { .. } => EventResult::IGNORED,
        }
    }

    pub fn render(&self, compositor: &mut Compositor, bounds: Rect, theme: &Theme) {
        let d = self.divider_rect(bounds);
        // Rest: the soft glass edge. Hover/drag: the active surface token,
        // slightly widened — the HOFF way to say "grab me" without color.
        let (color, grow) = if self.dragging || self.hovered {
            (theme.glass.surface_active.0, 1.0)
        } else {
            (theme.glass.edge_soft.0, 0.0)
        };
        let rect = match self.direction {
            SplitDirection::Horizontal => Rect::new(d.x - grow / 2.0, d.y, d.w + grow, d.h),
            SplitDirection::Vertical => Rect::new(d.x, d.y - grow / 2.0, d.w, d.h + grow),
        };
        compositor.push(rounded_rect(
            rect.x,
            rect.y,
            rect.w,
            rect.h,
            rect.w.min(rect.h) / 2.0,
            color,
        ));
    }
}
