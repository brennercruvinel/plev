use engine::animation::Spring;
use engine::compositor::{Compositor, SceneNode};
use engine::input::scroll::ScrollState;
use engine::theme::Theme;

use crate::core::{EventResult, Rect, WidgetEvent, with_alpha};

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
    pub fn track_rect(&self, bounds: Rect, theme: &Theme) -> Rect {
        let c = &theme.control;
        let w = if self.hovered || self.dragging {
            c.scrollbar_w_hover
        } else {
            c.scrollbar_w
        };
        Rect::new(
            bounds.x + bounds.w - w - c.scrollbar_inset,
            bounds.y,
            w,
            bounds.h,
        )
    }

    /// Thumb rect derived from the scroll state (proportional size).
    pub fn thumb_rect(&self, bounds: Rect, scroll: &ScrollState, theme: &Theme) -> Rect {
        let track = self.track_rect(bounds, theme);
        let thumb_h = (track.h * scroll.thumb_ratio())
            .max(theme.control.scrollbar_min_thumb)
            .min(track.h);
        let y = track.y + (track.h - thumb_h) * scroll.thumb_position();
        Rect::new(track.x, y, track.w, thumb_h)
    }

    fn offset_for_thumb_top(
        &self,
        thumb_top: f32,
        bounds: Rect,
        scroll: &ScrollState,
        theme: &Theme,
    ) -> f32 {
        let track = self.track_rect(bounds, theme);
        let thumb_h = (track.h * scroll.thumb_ratio())
            .max(theme.control.scrollbar_min_thumb)
            .min(track.h);
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
        theme: &Theme,
    ) -> EventResult {
        if !scroll.is_scrollable() {
            return EventResult::IGNORED;
        }
        match *event {
            WidgetEvent::MouseMove { x, y } => {
                let mut result = EventResult::IGNORED;
                // Generous hover band so the thin bar is easy to reach:
                // the hovered width on both sides.
                let track = self.track_rect(bounds, theme);
                let reach = theme.control.scrollbar_w_hover - track.w;
                let near = Rect::new(
                    track.x - reach,
                    track.y,
                    track.w + reach + theme.control.scrollbar_inset,
                    track.h,
                );
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
                    let target =
                        self.offset_for_thumb_top(y - self.drag_grab, bounds, scroll, theme);
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
                let thumb = self.thumb_rect(bounds, scroll, theme);
                if thumb.contains(x, y) {
                    self.dragging = true;
                    self.drag_grab = y - thumb.y;
                    self.idle = 0.0;
                    return EventResult::changed();
                }
                let track = self.track_rect(bounds, theme);
                if track.contains(x, y) {
                    // Jump so the thumb centers on the click, then drag.
                    self.dragging = true;
                    self.drag_grab = thumb.h / 2.0;
                    let target =
                        self.offset_for_thumb_top(y - self.drag_grab, bounds, scroll, theme);
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
        let mut nodes = Vec::with_capacity(1);
        self.render_nodes(&mut nodes, bounds, scroll, theme);
        for node in nodes {
            compositor.push(node);
        }
    }

    /// Emit scene nodes without a compositor (callers pick the layer).
    pub fn render_nodes(
        &self,
        out: &mut Vec<SceneNode>,
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
        let thumb = self.thumb_rect(bounds, scroll, theme);
        // Thumb: placeholder alpha at rest, faint on hover, text-default
        // while dragging (the same three text steps every quiet control
        // climbs).
        let g = &theme.glass;
        let tone = if self.dragging {
            g.text_default
        } else if self.hovered {
            g.text_faint
        } else {
            g.text_placeholder
        };
        out.push(SceneNode::RoundedRect {
            x: thumb.x,
            y: thumb.y,
            w: thumb.w,
            h: thumb.h,
            color: with_alpha(tone, tone.0[3] * opacity),
            corner_radius: thumb.w / 2.0,
            border_width: 0.0,
            border_color: [0.0; 4],
        });
    }
}
