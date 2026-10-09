//! A finger on the gallery (android and iOS; task-18). Only the first
//! finger counts. A tap is a click where it landed: the press and the
//! release both come at the lift, since a palette card or a button may act
//! on the press alone, and a finger that lands on one only to scroll must
//! not choose it. A finger that travels past `SLOP` scrolls the page it
//! landed on and clicks nothing. Dragging a control (a slider) by touch is
//! not there yet.

use engine::widgets::prelude::WidgetEvent;
use winit::event::TouchPhase;
use winit::window::WindowId;

use super::super::App;

/// How far a finger travels, in logical pixels, before it scrolls.
const SLOP: f32 = 8.0;

/// The first finger, from its landing to its lift.
#[derive(Debug, Default)]
pub(crate) struct Finger {
    /// Its id and where it landed, while it is down.
    down: Option<(u64, f32, f32)>,
    /// Where it was at the last move.
    last: f32,
    scrolling: bool,
}

impl Finger {
    /// What one touch asks of the view, in order, in logical pixels.
    pub(super) fn touch(&mut self, id: u64, phase: TouchPhase, x: f32, y: f32) -> Vec<WidgetEvent> {
        if phase == TouchPhase::Started {
            if self.down.is_some() {
                return Vec::new();
            }
            self.down = Some((id, x, y));
            self.last = y;
            self.scrolling = false;
            return vec![WidgetEvent::MouseMove { x, y }];
        }
        let Some((first, lx, ly)) = self.down else {
            return Vec::new();
        };
        if id != first {
            return Vec::new();
        }
        match phase {
            TouchPhase::Moved => {
                if !self.scrolling && (x - lx).abs() <= SLOP && (y - ly).abs() <= SLOP {
                    return Vec::new();
                }
                self.scrolling = true;
                let delta = self.last - y;
                self.last = y;
                vec![WidgetEvent::Scroll {
                    x: lx,
                    y: ly,
                    delta,
                }]
            }
            TouchPhase::Ended => {
                self.down = None;
                if std::mem::take(&mut self.scrolling) {
                    return Vec::new();
                }
                vec![
                    WidgetEvent::MouseDown { x: lx, y: ly },
                    WidgetEvent::MouseUp { x: lx, y: ly },
                ]
            }
            // the system took the gesture: nothing was pressed yet
            TouchPhase::Cancelled | TouchPhase::Started => {
                self.down = None;
                self.scrolling = false;
                Vec::new()
            }
        }
    }
}

impl App {
    /// A touch in window `id`, in physical pixels: the view gets what the
    /// finger asks, through the path the mouse takes.
    pub(crate) fn touched(&mut self, id: WindowId, touch: &winit::event::Touch) {
        let sf = self.windows.get(id).map_or(1.0, |h| h.scale_factor()) as f32;
        let (x, y) = (touch.location.x as f32 / sf, touch.location.y as f32 / sf);
        for event in self.finger.touch(touch.id, touch.phase, x, y) {
            match event {
                WidgetEvent::MouseMove { x, y } => {
                    self.set_cursor(id, (x, y));
                    if self.pointer(id, &event) {
                        self.invalidate_in(id);
                    }
                }
                WidgetEvent::Scroll { .. } => {
                    if self.pointer(id, &event) {
                        self.invalidate_in(id);
                    }
                }
                _ => {
                    if self.pointer(id, &event) {
                        self.changed();
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_tap_clicks_where_the_finger_landed_at_the_lift() {
        let mut finger = Finger::default();
        assert_eq!(
            finger.touch(1, TouchPhase::Started, 40.0, 50.0),
            vec![WidgetEvent::MouseMove { x: 40.0, y: 50.0 }]
        );
        assert!(finger.touch(1, TouchPhase::Moved, 44.0, 55.0).is_empty());
        assert_eq!(
            finger.touch(1, TouchPhase::Ended, 44.0, 55.0),
            vec![
                WidgetEvent::MouseDown { x: 40.0, y: 50.0 },
                WidgetEvent::MouseUp { x: 40.0, y: 50.0 },
            ]
        );
    }

    #[test]
    fn a_drag_scrolls_where_it_landed_and_clicks_nothing() {
        let mut finger = Finger::default();
        finger.touch(1, TouchPhase::Started, 40.0, 300.0);
        assert_eq!(
            finger.touch(1, TouchPhase::Moved, 40.0, 280.0),
            vec![WidgetEvent::Scroll {
                x: 40.0,
                y: 300.0,
                delta: 20.0
            }]
        );
        assert_eq!(
            finger.touch(1, TouchPhase::Moved, 40.0, 290.0),
            vec![WidgetEvent::Scroll {
                x: 40.0,
                y: 300.0,
                delta: -10.0
            }]
        );
        assert!(finger.touch(1, TouchPhase::Ended, 40.0, 290.0).is_empty());
        // the next finger starts over
        assert_eq!(finger.touch(2, TouchPhase::Started, 10.0, 10.0).len(), 1);
        assert_eq!(finger.touch(2, TouchPhase::Ended, 10.0, 10.0).len(), 2);
    }

    #[test]
    fn a_cancelled_touch_clicks_nothing() {
        let mut finger = Finger::default();
        finger.touch(1, TouchPhase::Started, 40.0, 50.0);
        assert!(
            finger
                .touch(1, TouchPhase::Cancelled, 40.0, 50.0)
                .is_empty()
        );
        assert!(finger.touch(1, TouchPhase::Ended, 40.0, 50.0).is_empty());
    }

    #[test]
    fn a_second_finger_is_ignored() {
        let mut finger = Finger::default();
        finger.touch(1, TouchPhase::Started, 40.0, 50.0);
        assert!(finger.touch(2, TouchPhase::Started, 90.0, 90.0).is_empty());
        assert!(finger.touch(2, TouchPhase::Moved, 90.0, 10.0).is_empty());
        assert!(finger.touch(2, TouchPhase::Ended, 90.0, 10.0).is_empty());
    }
}
