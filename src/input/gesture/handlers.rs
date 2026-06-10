use web_time::Instant;

use crate::input::{
    DOUBLE_TAP_TIMEOUT, DoubleTapEvent, DragEvent, GestureEvent, LONG_PRESS_DURATION,
    LongPressEvent, Phase, PinchEvent, Point, SWIPE_MIN_DIST, SWIPE_MIN_VEL, SwipeEvent,
    TAP_MAX_DURATION, TOUCH_SLOP, TapEvent,
};

use super::state::{GestureState, classify_swipe};

use super::recognizer::GestureRecognizer;

/// touch_end, touch_cancel, tick, and internal end-state helpers.
impl GestureRecognizer {
    /// A finger has been lifted.
    pub fn touch_end(&mut self, id: u64, position: Point, now: Instant) {
        // Update final position before removing
        self.tracker.update(id, position, now, TOUCH_SLOP);

        match self.state {
            GestureState::PossibleTap => {
                self.end_possible_tap(id, now);
            }
            GestureState::Dragging => {
                self.end_drag(id, now);
            }
            GestureState::LongPressing => {
                self.tracker.end(id);
                self.primary_finger = None;
                self.state = GestureState::Idle;
            }
            GestureState::Pinching => {
                self.end_pinch(id);
            }
            GestureState::WaitingForSecondTap | GestureState::Idle => {
                // Stale end event
                self.tracker.end(id);
            }
        }
    }

    /// A finger's touch was cancelled by the OS.
    pub fn touch_cancel(&mut self, id: u64) {
        self.tracker.cancel(id);

        match self.state {
            GestureState::Dragging => {
                self.pending_events.push(GestureEvent::Drag(DragEvent {
                    position: Point { x: 0.0, y: 0.0 },
                    start_position: Point { x: 0.0, y: 0.0 },
                    delta: Point { x: 0.0, y: 0.0 },
                    phase: Phase::Cancelled,
                }));
            }
            GestureState::Pinching => {
                if self.tracker.active_count() < 2 {
                    self.pending_events.push(GestureEvent::Pinch(PinchEvent {
                        center: Point { x: 0.0, y: 0.0 },
                        scale: self.last_pinch_scale,
                        delta_scale: 0.0,
                        phase: Phase::Cancelled,
                    }));
                    self.initial_pinch_distance = None;
                    self.last_pinch_scale = 1.0;
                }
            }
            _ => {}
        }

        if self.tracker.active_count() == 0 {
            self.state = GestureState::Idle;
            self.primary_finger = None;
            self.last_drag_position = None;
        }
    }

    /// Called periodically (e.g., each frame) to handle time-based transitions.
    pub fn tick(&mut self, now: Instant) {
        match self.state {
            GestureState::PossibleTap => {
                // Check for long-press
                if let Some(pid) = self.primary_finger
                    && let Some(finger) = self.tracker.get(pid)
                {
                    let elapsed = now.duration_since(finger.start_time);
                    if elapsed >= LONG_PRESS_DURATION && !finger.moved_past_slop {
                        let pos = finger.start_position;
                        self.state = GestureState::LongPressing;
                        self.pending_events
                            .push(GestureEvent::LongPress(LongPressEvent {
                                position: pos,
                                duration: elapsed,
                            }));
                    }
                }
            }
            GestureState::WaitingForSecondTap => {
                // Check if double-tap timeout has expired
                if let Some(last_time) = self.last_tap_time {
                    let elapsed = now.duration_since(last_time);
                    if elapsed > DOUBLE_TAP_TIMEOUT {
                        // Timeout -- it was just a single tap (already emitted)
                        self.state = GestureState::Idle;
                        self.last_tap_time = None;
                        self.last_tap_position = None;
                    }
                }
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    fn end_possible_tap(&mut self, id: u64, now: Instant) {
        if let Some(finger) = self.tracker.get(id) {
            let elapsed = now.duration_since(finger.start_time);
            let start_pos = finger.start_position;

            if elapsed <= TAP_MAX_DURATION && !finger.moved_past_slop {
                // Valid tap timing and distance
                if let Some(last_time) = self.last_tap_time {
                    let since_last = now.duration_since(last_time);
                    if since_last <= DOUBLE_TAP_TIMEOUT {
                        // Double tap!
                        self.pending_events
                            .push(GestureEvent::DoubleTap(DoubleTapEvent {
                                position: start_pos,
                            }));
                        self.last_tap_time = None;
                        self.last_tap_position = None;
                        self.tracker.end(id);
                        self.primary_finger = None;
                        self.state = GestureState::Idle;
                        return;
                    }
                }
                // Single tap -- but wait for possible double tap
                self.pending_events.push(GestureEvent::Tap(TapEvent {
                    position: start_pos,
                }));
                self.last_tap_time = Some(now);
                self.last_tap_position = Some(start_pos);
                self.tracker.end(id);
                self.primary_finger = None;
                self.state = GestureState::WaitingForSecondTap;
                return;
            } else if finger.moved_past_slop {
                // Moved past slop -- check for swipe
                let dx = finger.current_position.x - finger.start_position.x;
                let dy = finger.current_position.y - finger.start_position.y;
                let dist = (dx * dx + dy * dy).sqrt();
                let elapsed_secs = elapsed.as_secs_f64();
                let velocity = if elapsed_secs > 0.0 {
                    dist / elapsed_secs
                } else {
                    0.0
                };

                if dist >= SWIPE_MIN_DIST && velocity >= SWIPE_MIN_VEL {
                    let direction = classify_swipe(dx, dy);
                    self.pending_events.push(GestureEvent::Swipe(SwipeEvent {
                        start_position: finger.start_position,
                        end_position: finger.current_position,
                        direction,
                        velocity,
                    }));
                }
            }
            // else: tap too slow -- just ignore
        }
        self.tracker.end(id);
        self.primary_finger = None;
        self.state = GestureState::Idle;
    }

    fn end_drag(&mut self, id: u64, now: Instant) {
        if let Some(finger) = self.tracker.get(id) {
            let dx = finger.current_position.x - finger.start_position.x;
            let dy = finger.current_position.y - finger.start_position.y;
            let dist = (dx * dx + dy * dy).sqrt();
            let elapsed = now.duration_since(finger.start_time);
            let elapsed_secs = elapsed.as_secs_f64();
            let velocity = if elapsed_secs > 0.0 {
                dist / elapsed_secs
            } else {
                0.0
            };

            let prev = self.last_drag_position.unwrap_or(finger.previous_position);

            self.pending_events.push(GestureEvent::Drag(DragEvent {
                position: finger.current_position,
                start_position: finger.start_position,
                delta: Point {
                    x: finger.current_position.x - prev.x,
                    y: finger.current_position.y - prev.y,
                },
                phase: Phase::Ended,
            }));

            // Also emit swipe if fast enough
            if dist >= SWIPE_MIN_DIST && velocity >= SWIPE_MIN_VEL {
                let direction = classify_swipe(dx, dy);
                self.pending_events.push(GestureEvent::Swipe(SwipeEvent {
                    start_position: finger.start_position,
                    end_position: finger.current_position,
                    direction,
                    velocity,
                }));
            }
        }
        self.tracker.end(id);
        self.primary_finger = None;
        self.last_drag_position = None;
        self.state = GestureState::Idle;
    }

    fn end_pinch(&mut self, id: u64) {
        self.tracker.end(id);
        if self.tracker.active_count() < 2 {
            if let Some(center) = self.tracker.finger_center() {
                self.pending_events.push(GestureEvent::Pinch(PinchEvent {
                    center,
                    scale: self.last_pinch_scale,
                    delta_scale: 0.0,
                    phase: Phase::Ended,
                }));
            }
            self.initial_pinch_distance = None;
            self.last_pinch_scale = 1.0;
            if self.tracker.active_count() == 1 {
                let remaining_ids = self.tracker.active_ids();
                self.primary_finger = remaining_ids.into_iter().next();
                self.state = GestureState::Dragging;
                if let Some(pid) = self.primary_finger
                    && let Some(f) = self.tracker.get(pid)
                {
                    self.last_drag_position = Some(f.current_position);
                    self.pending_events.push(GestureEvent::Drag(DragEvent {
                        position: f.current_position,
                        start_position: f.start_position,
                        delta: Point { x: 0.0, y: 0.0 },
                        phase: Phase::Started,
                    }));
                }
            } else {
                self.primary_finger = None;
                self.state = GestureState::Idle;
            }
        }
    }
}
