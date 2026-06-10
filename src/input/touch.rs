use web_time::Instant;

use rustc_hash::FxHashMap;

use super::Point;

// ---------------------------------------------------------------------------
// TouchFinger — per-finger state tracking
// ---------------------------------------------------------------------------

/// Tracks the state of a single touch finger across its lifecycle.
pub struct TouchFinger {
    /// Unique finger ID from the OS.
    pub id: u64,
    /// Position where the finger first touched.
    pub start_position: Point,
    /// Time when the finger first touched.
    pub start_time: Instant,
    /// Current (latest) position.
    pub current_position: Point,
    /// Time of last update.
    pub last_time: Instant,
    /// Previous position (for delta calculations).
    pub previous_position: Point,
    /// Total distance traveled by this finger.
    pub total_distance: f64,
    /// Whether the finger has moved past the touch slop threshold.
    pub moved_past_slop: bool,
}

impl TouchFinger {
    fn new(id: u64, position: Point, time: Instant) -> Self {
        Self {
            id,
            start_position: position,
            start_time: time,
            current_position: position,
            last_time: time,
            previous_position: position,
            total_distance: 0.0,
            moved_past_slop: false,
        }
    }
}

// ---------------------------------------------------------------------------
// TouchTracker — multi-finger tracking via FxHashMap
// ---------------------------------------------------------------------------

/// Tracks all active touch fingers.
pub struct TouchTracker {
    fingers: FxHashMap<u64, TouchFinger>,
}

impl Default for TouchTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl TouchTracker {
    pub fn new() -> Self {
        Self {
            fingers: FxHashMap::default(),
        }
    }

    /// Begin tracking a new finger.
    pub fn start(&mut self, id: u64, position: Point, time: Instant) {
        self.fingers
            .insert(id, TouchFinger::new(id, position, time));
    }

    /// Update an existing finger's position.
    pub fn update(&mut self, id: u64, position: Point, time: Instant, slop: f64) {
        if let Some(finger) = self.fingers.get_mut(&id) {
            let dx = position.x - finger.current_position.x;
            let dy = position.y - finger.current_position.y;
            let dist = (dx * dx + dy * dy).sqrt();
            finger.total_distance += dist;
            finger.previous_position = finger.current_position;
            finger.current_position = position;
            finger.last_time = time;

            if !finger.moved_past_slop {
                let from_start_x = position.x - finger.start_position.x;
                let from_start_y = position.y - finger.start_position.y;
                let from_start = (from_start_x * from_start_x + from_start_y * from_start_y).sqrt();
                if from_start > slop {
                    finger.moved_past_slop = true;
                }
            }
        }
    }

    /// Remove a finger (ended or cancelled).
    pub fn end(&mut self, id: u64) -> Option<TouchFinger> {
        self.fingers.remove(&id)
    }

    /// Cancel a finger.
    pub fn cancel(&mut self, id: u64) -> Option<TouchFinger> {
        self.fingers.remove(&id)
    }

    /// Get a reference to a finger by ID.
    pub fn get(&self, id: u64) -> Option<&TouchFinger> {
        self.fingers.get(&id)
    }

    /// Number of currently active fingers.
    pub fn active_count(&self) -> usize {
        self.fingers.len()
    }

    /// Compute the distance between exactly two active fingers.
    /// Returns None if there aren't exactly 2+ fingers.
    pub fn finger_distance(&self) -> Option<f64> {
        let ids: Vec<u64> = self.fingers.keys().copied().collect();
        if ids.len() < 2 {
            return None;
        }
        let a = &self.fingers[&ids[0]];
        let b = &self.fingers[&ids[1]];
        let dx = a.current_position.x - b.current_position.x;
        let dy = a.current_position.y - b.current_position.y;
        Some((dx * dx + dy * dy).sqrt())
    }

    /// Compute the center point between all active fingers.
    pub fn finger_center(&self) -> Option<Point> {
        if self.fingers.is_empty() {
            return None;
        }
        let (sum_x, sum_y) = self.fingers.values().fold((0.0, 0.0), |(sx, sy), f| {
            (sx + f.current_position.x, sy + f.current_position.y)
        });
        let n = self.fingers.len() as f64;
        Some(Point {
            x: sum_x / n,
            y: sum_y / n,
        })
    }

    /// Clear all tracked fingers.
    pub fn clear(&mut self) {
        self.fingers.clear();
    }

    /// Get all active finger IDs.
    pub fn active_ids(&self) -> Vec<u64> {
        self.fingers.keys().copied().collect()
    }
}

// ---------------------------------------------------------------------------
// Touch -> pointer synthesis (touch-screen compatibility layer)
// ---------------------------------------------------------------------------

/// A pointer action synthesized from a raw touch event. Each variant maps
/// 1:1 onto the `InputState` entry point that real mouse input goes
/// through (see `InputState::handle_synthetic_pointer`), so touch is
/// dispatched on the exact same path as the mouse and every existing
/// widget responds to taps and drags without changes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SyntheticPointerEvent {
    /// -> `InputState::handle_cursor_moved`.
    CursorMoved { x: f32, y: f32 },
    /// -> `InputState::handle_mouse_input(Left, Pressed)`. Always preceded
    /// by a `CursorMoved`, so the press lands on the touch point.
    PrimaryButtonDown,
    /// -> `InputState::handle_mouse_input(Left, Released)`.
    PrimaryButtonUp,
    /// -> `InputState::handle_cursor_left`. Fingers do not hover: emitted
    /// when the touch sequence ends so no widget keeps a stale hover state.
    CursorLeft,
}

/// Translates the PRIMARY touch (first finger down) into mouse-equivalent
/// pointer events: tap = press/release, finger move = cursor move.
/// Additional fingers are ignored here — multi-finger gestures (pinch,
/// long-press, swipe) remain the `GestureRecognizer`'s job.
///
/// Pure state machine over `(finger id, phase, position)`: no GPU, no
/// window, fully unit-testable.
#[derive(Debug, Default)]
pub struct TouchPointerSynth {
    /// Finger currently acting as the mouse, if any.
    primary: Option<u64>,
}

impl TouchPointerSynth {
    pub fn new() -> Self {
        Self::default()
    }

    /// Translate one raw touch event into the ordered pointer events to
    /// inject. Only the tracked primary finger id is mutated.
    pub fn synthesize(
        &mut self,
        id: u64,
        phase: winit::event::TouchPhase,
        x: f32,
        y: f32,
    ) -> Vec<SyntheticPointerEvent> {
        use SyntheticPointerEvent as E;
        use winit::event::TouchPhase;

        match phase {
            TouchPhase::Started => {
                if self.primary.is_some() {
                    // Secondary finger: recognizer-only (pinch etc.).
                    return Vec::new();
                }
                self.primary = Some(id);
                vec![E::CursorMoved { x, y }, E::PrimaryButtonDown]
            }
            TouchPhase::Moved => {
                if self.primary != Some(id) {
                    return Vec::new();
                }
                vec![E::CursorMoved { x, y }]
            }
            TouchPhase::Ended | TouchPhase::Cancelled => {
                if self.primary != Some(id) {
                    return Vec::new();
                }
                self.primary = None;
                // Release at the final touch point, then clear the cursor:
                // fingers don't hover after lifting. Cancelled also
                // releases — widgets must not be left stuck in a pressed
                // state when the system takes the gesture over.
                vec![E::CursorMoved { x, y }, E::PrimaryButtonUp, E::CursorLeft]
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn now() -> Instant {
        Instant::now()
    }

    fn pt(x: f64, y: f64) -> Point {
        Point { x, y }
    }

    #[test]
    fn start_and_get() {
        let mut tracker = TouchTracker::new();
        let t = now();
        tracker.start(1, pt(100.0, 200.0), t);
        assert_eq!(tracker.active_count(), 1);
        let f = tracker.get(1).unwrap();
        assert_eq!(f.start_position.x, 100.0);
        assert_eq!(f.current_position.y, 200.0);
    }

    #[test]
    fn update_tracks_distance() {
        let mut tracker = TouchTracker::new();
        let t = now();
        tracker.start(1, pt(0.0, 0.0), t);
        tracker.update(1, pt(3.0, 4.0), t, 10.0);
        let f = tracker.get(1).unwrap();
        assert!((f.total_distance - 5.0).abs() < 0.001);
        assert!(!f.moved_past_slop); // 5 < 10 slop
    }

    #[test]
    fn moved_past_slop() {
        let mut tracker = TouchTracker::new();
        let t = now();
        tracker.start(1, pt(0.0, 0.0), t);
        tracker.update(1, pt(11.0, 0.0), t, 10.0);
        let f = tracker.get(1).unwrap();
        assert!(f.moved_past_slop);
    }

    #[test]
    fn finger_distance_two_fingers() {
        let mut tracker = TouchTracker::new();
        let t = now();
        tracker.start(1, pt(0.0, 0.0), t);
        tracker.start(2, pt(100.0, 0.0), t);
        let dist = tracker.finger_distance().unwrap();
        assert!((dist - 100.0).abs() < 0.001);
    }

    // -- touch -> pointer synthesis --

    use SyntheticPointerEvent as E;
    use winit::event::TouchPhase;

    #[test]
    fn touch_start_moves_cursor_then_presses() {
        let mut synth = TouchPointerSynth::new();
        let events = synth.synthesize(7, TouchPhase::Started, 50.0, 60.0);
        assert_eq!(
            events,
            vec![E::CursorMoved { x: 50.0, y: 60.0 }, E::PrimaryButtonDown]
        );
    }

    #[test]
    fn touch_move_synthesizes_cursor_move() {
        let mut synth = TouchPointerSynth::new();
        synth.synthesize(7, TouchPhase::Started, 50.0, 60.0);
        let events = synth.synthesize(7, TouchPhase::Moved, 55.0, 66.0);
        assert_eq!(events, vec![E::CursorMoved { x: 55.0, y: 66.0 }]);
    }

    #[test]
    fn touch_end_releases_at_final_point_and_clears_hover() {
        let mut synth = TouchPointerSynth::new();
        synth.synthesize(7, TouchPhase::Started, 50.0, 60.0);
        let events = synth.synthesize(7, TouchPhase::Ended, 52.0, 61.0);
        assert_eq!(
            events,
            vec![
                E::CursorMoved { x: 52.0, y: 61.0 },
                E::PrimaryButtonUp,
                E::CursorLeft,
            ]
        );
    }

    #[test]
    fn touch_cancel_releases_so_widgets_are_not_stuck_pressed() {
        let mut synth = TouchPointerSynth::new();
        synth.synthesize(7, TouchPhase::Started, 50.0, 60.0);
        let events = synth.synthesize(7, TouchPhase::Cancelled, 50.0, 60.0);
        assert!(events.contains(&E::PrimaryButtonUp));
        assert!(events.contains(&E::CursorLeft));
    }

    #[test]
    fn secondary_finger_is_ignored() {
        let mut synth = TouchPointerSynth::new();
        synth.synthesize(1, TouchPhase::Started, 10.0, 10.0);
        // Second finger lands (pinch start): no pointer events at all.
        assert!(synth.synthesize(2, TouchPhase::Started, 90.0, 90.0).is_empty());
        assert!(synth.synthesize(2, TouchPhase::Moved, 95.0, 95.0).is_empty());
        assert!(synth.synthesize(2, TouchPhase::Ended, 99.0, 99.0).is_empty());
        // Primary finger still drives the pointer afterwards.
        let events = synth.synthesize(1, TouchPhase::Moved, 12.0, 12.0);
        assert_eq!(events, vec![E::CursorMoved { x: 12.0, y: 12.0 }]);
    }

    #[test]
    fn next_finger_becomes_primary_after_previous_lifts() {
        let mut synth = TouchPointerSynth::new();
        synth.synthesize(1, TouchPhase::Started, 10.0, 10.0);
        synth.synthesize(1, TouchPhase::Ended, 10.0, 10.0);
        let events = synth.synthesize(2, TouchPhase::Started, 30.0, 40.0);
        assert_eq!(
            events,
            vec![E::CursorMoved { x: 30.0, y: 40.0 }, E::PrimaryButtonDown]
        );
    }

    #[test]
    fn events_for_untracked_finger_are_ignored() {
        let mut synth = TouchPointerSynth::new();
        // Move/end without a Started (e.g. events from before a focus
        // change): nothing is synthesized.
        assert!(synth.synthesize(5, TouchPhase::Moved, 10.0, 10.0).is_empty());
        assert!(synth.synthesize(5, TouchPhase::Ended, 10.0, 10.0).is_empty());
    }
}
