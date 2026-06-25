//! Multi-finger touch tracking ([`TouchTracker`]) plus the touch -> pointer
//! synthesis compatibility layer ([`synth`]).

mod synth;

pub use synth::{SyntheticPointerEvent, TouchPointerSynth};

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
}
