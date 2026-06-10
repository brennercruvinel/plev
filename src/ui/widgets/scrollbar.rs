use crate::animation::Spring;
use crate::compositor::{Compositor, SceneNode};
use crate::scroll::ScrollState;
use crate::theme::Theme;

use super::{EventResult, Rect, WidgetEvent, with_alpha};

const WIDTH: f32 = 6.0;
const WIDTH_HOVER: f32 = 10.0;
const MIN_THUMB: f32 = 24.0;

/// Seconds without scroll activity before the bar fades out.
const IDLE_HIDE_SECS: f32 = 1.0;

/// Vertical scrollbar overlay.
///
/// Renders inside a track rect (typically the right edge of a scrollable
/// region), with macOS-style behavior: appears on scroll, fades out after
/// ~1s idle, widens on hover, and supports thumb dragging plus
/// click-to-jump on the track.
#[derive(Clone, Debug)]
pub struct Scrollbar {
    hovered: bool,
    dragging: bool,
    /// Pointer offset within the thumb when the drag started.
    drag_grab: f32,
    /// Visibility 0.0..=1.0, spring-driven for the fade.
    opacity: Spring<f32>,
    /// Seconds since the last scroll/drag activity.
    idle: f32,
}

impl Default for Scrollbar {
    fn default() -> Self {
        Self::new()
    }
}

impl Scrollbar {
    pub fn new() -> Self {
        Self {
            hovered: false,
            dragging: false,
            drag_grab: 0.0,
            // Stiff, overdamped: fades fast without bouncing.
            opacity: Spring::new(0.0_f32).with_config(120.0, 22.0, 1.0),
            idle: IDLE_HIDE_SECS,
        }
    }

    pub fn is_hovered(&self) -> bool {
        self.hovered
    }

    pub fn is_dragging(&self) -> bool {
        self.dragging
    }

    /// Current fade opacity (0 hidden .. 1 shown).
    pub fn opacity(&self) -> f32 {
        self.opacity.get().clamp(0.0, 1.0)
    }

    /// Call whenever the owner scrolls (wheel, fling, keyboard) so the bar
    /// wakes up and the idle timer restarts.
    pub fn notify_scroll(&mut self) {
        self.idle = 0.0;
        self.opacity.set_target(1.0);
    }

    /// Advance fade animation. Returns `true` while animating.
    pub fn tick(&mut self, dt: f32) -> bool {
        self.idle += dt;
        if self.idle >= IDLE_HIDE_SECS && !self.hovered && !self.dragging {
            self.opacity.set_target(0.0);
        }
        self.opacity.tick(dt);
        self.opacity.is_animating()
    }

    pub fn is_animating(&self) -> bool {
        self.opacity.is_animating()
    }

    /// Track rect: the full-height strip at the right edge of `bounds`.
    pub fn track_rect(&self, bounds: Rect) -> Rect {
        let w = if self.hovered || self.dragging {
            WIDTH_HOVER
        } else {
            WIDTH
        };
        Rect::new(bounds.x + bounds.w - w - 2.0, bounds.y, w, bounds.h)
    }

    /// Thumb rect derived from the scroll state (proportional size).
    pub fn thumb_rect(&self, bounds: Rect, scroll: &ScrollState) -> Rect {
        let track = self.track_rect(bounds);
        let thumb_h = (track.h * scroll.thumb_ratio()).max(MIN_THUMB).min(track.h);
        let y = track.y + (track.h - thumb_h) * scroll.thumb_position();
        Rect::new(track.x, y, track.w, thumb_h)
    }

    fn offset_for_thumb_top(&self, thumb_top: f32, bounds: Rect, scroll: &ScrollState) -> f32 {
        let track = self.track_rect(bounds);
        let thumb_h = (track.h * scroll.thumb_ratio()).max(MIN_THUMB).min(track.h);
        let usable = (track.h - thumb_h).max(1.0);
        let t = ((thumb_top - track.y) / usable).clamp(0.0, 1.0);
        t * scroll.max_offset()
    }

    /// Handle pointer events against the scrollable region `bounds`,
    /// mutating `scroll` on drag/track-jump.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        bounds: Rect,
        scroll: &mut ScrollState,
    ) -> EventResult {
        if !scroll.is_scrollable() {
            return EventResult::IGNORED;
        }
        match *event {
            WidgetEvent::MouseMove { x, y } => {
                let mut result = EventResult::IGNORED;
                // Generous hover band so the 6px bar is easy to reach.
                let track = self.track_rect(bounds);
                let near = Rect::new(track.x - 4.0, track.y, track.w + 6.0, track.h);
                let inside = near.contains(x, y) && self.opacity() > 0.1;
                if inside != self.hovered {
                    self.hovered = inside;
                    if inside {
                        self.opacity.set_target(1.0);
                        self.idle = 0.0;
                    }
                    result = EventResult::changed();
                }
                if self.dragging {
                    let old = scroll.offset();
                    let target = self.offset_for_thumb_top(y - self.drag_grab, bounds, scroll);
                    scroll.scroll_to(target);
                    self.idle = 0.0;
                    if scroll.offset() != old {
                        result = result.merge(EventResult::changed());
                    }
                    result.handled = true;
                }
                result
            }
            WidgetEvent::MouseDown { x, y } => {
                if self.opacity() <= 0.1 {
                    return EventResult::IGNORED;
                }
                let thumb = self.thumb_rect(bounds, scroll);
                if thumb.contains(x, y) {
                    self.dragging = true;
                    self.drag_grab = y - thumb.y;
                    self.idle = 0.0;
                    return EventResult::changed();
                }
                let track = self.track_rect(bounds);
                if track.contains(x, y) {
                    // Jump so the thumb centers on the click, then drag.
                    self.dragging = true;
                    self.drag_grab = thumb.h / 2.0;
                    let target = self.offset_for_thumb_top(y - self.drag_grab, bounds, scroll);
                    scroll.scroll_to(target);
                    self.idle = 0.0;
                    return EventResult::changed();
                }
                EventResult::IGNORED
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

    pub fn render(
        &self,
        compositor: &mut Compositor,
        bounds: Rect,
        scroll: &ScrollState,
        theme: &Theme,
    ) {
        if !scroll.is_scrollable() {
            return;
        }
        let opacity = self.opacity();
        if opacity <= 0.01 {
            return;
        }
        let thumb = self.thumb_rect(bounds, scroll);
        let strength = if self.dragging {
            0.65
        } else if self.hovered {
            0.55
        } else {
            0.35
        };
        compositor.push(SceneNode::RoundedRect {
            x: thumb.x,
            y: thumb.y,
            w: thumb.w,
            h: thumb.h,
            color: with_alpha(theme.colors.text_mid, strength * opacity),
            corner_radius: thumb.w / 2.0,
            border_width: 0.0,
            border_color: [0.0; 4],
        });
    }
}
