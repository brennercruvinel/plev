use web_time::Instant;

use crate::input::touch::TouchTracker;
use crate::input::{
    DOUBLE_TAP_SLOP, DragEvent, GestureEvent, Phase, PinchEvent, Point, TOUCH_SLOP,
};

use super::state::GestureState;

// ---------------------------------------------------------------------------
// GestureRecognizer
// ---------------------------------------------------------------------------

/// Recognizes tap, double-tap, long-press, swipe, drag, and pinch gestures
/// from raw touch events. All methods take explicit `Instant` for testability.
pub struct GestureRecognizer {
    pub(super) state: GestureState,
    pub(super) tracker: TouchTracker,
    /// Pending events to be consumed by the application.
    pub(super) pending_events: Vec<GestureEvent>,

    // State for tap detection
    /// The finger ID of the primary finger (first to touch).
    pub(super) primary_finger: Option<u64>,

    // State for double-tap detection
    pub(super) last_tap_time: Option<Instant>,
    pub(super) last_tap_position: Option<Point>,

    // State for pinch
    pub(super) initial_pinch_distance: Option<f64>,
    pub(super) last_pinch_scale: f64,

    // State for drag
    pub(super) last_drag_position: Option<Point>,
}

impl Default for GestureRecognizer {
    fn default() -> Self {
        Self::new()
    }
}

impl GestureRecognizer {
    pub fn new() -> Self {
        Self {
            state: GestureState::Idle,
            tracker: TouchTracker::new(),
            pending_events: Vec::new(),
            primary_finger: None,
            last_tap_time: None,
            last_tap_position: None,
            initial_pinch_distance: None,
            last_pinch_scale: 1.0,
            last_drag_position: None,
        }
    }

    /// Drain all pending gesture events.
    pub fn drain_events(&mut self) -> Vec<GestureEvent> {
        std::mem::take(&mut self.pending_events)
    }

    /// Whether any touch is currently active.
    pub fn is_touch_active(&self) -> bool {
        self.tracker.active_count() > 0
    }

    // -----------------------------------------------------------------------
    // touch_start + touch_move
    // -----------------------------------------------------------------------

    /// A new finger has touched the screen.
    pub fn touch_start(&mut self, id: u64, position: Point, now: Instant) {
        self.tracker.start(id, position, now);

        match self.state {
            GestureState::Idle => {
                self.primary_finger = Some(id);
                self.state = GestureState::PossibleTap;
            }
            GestureState::WaitingForSecondTap => {
                // Check if this second tap is close enough to the first
                if let Some(last_pos) = self.last_tap_position {
                    let dx = position.x - last_pos.x;
                    let dy = position.y - last_pos.y;
                    let dist = (dx * dx + dy * dy).sqrt();
                    if dist <= DOUBLE_TAP_SLOP {
                        self.primary_finger = Some(id);
                        self.state = GestureState::PossibleTap;
                    } else {
                        // Too far -- treat as new gesture
                        self.last_tap_time = None;
                        self.last_tap_position = None;
                        self.primary_finger = Some(id);
                        self.state = GestureState::PossibleTap;
                    }
                } else {
                    self.primary_finger = Some(id);
                    self.state = GestureState::PossibleTap;
                }
            }
            GestureState::PossibleTap | GestureState::Dragging | GestureState::LongPressing => {
                // Second finger while already tracking -- transition to pinch
                if self.tracker.active_count() >= 2 {
                    self.transition_to_pinch();
                }
            }
            GestureState::Pinching => {
                // Additional finger during pinch -- just track it
            }
        }
    }

    /// A finger has moved.
    pub fn touch_move(&mut self, id: u64, position: Point, now: Instant) {
        self.tracker.update(id, position, now, TOUCH_SLOP);

        match self.state {
            GestureState::PossibleTap => {
                if let Some(finger) = self.tracker.get(id)
                    && finger.moved_past_slop
                {
                    // Moved too far -- it's a drag, not a tap
                    self.state = GestureState::Dragging;
                    if let Some(f) = self.tracker.get(id) {
                        self.last_drag_position = Some(f.start_position);
                        self.pending_events.push(GestureEvent::Drag(DragEvent {
                            position: f.current_position,
                            start_position: f.start_position,
                            delta: Point {
                                x: f.current_position.x - f.start_position.x,
                                y: f.current_position.y - f.start_position.y,
                            },
                            phase: Phase::Started,
                        }));
                    }
                }
            }
            GestureState::Dragging => {
                if let Some(primary_id) = self.primary_finger
                    && id == primary_id
                    && let Some(finger) = self.tracker.get(id)
                {
                    let prev = self.last_drag_position.unwrap_or(finger.previous_position);
                    self.pending_events.push(GestureEvent::Drag(DragEvent {
                        position: finger.current_position,
                        start_position: finger.start_position,
                        delta: Point {
                            x: finger.current_position.x - prev.x,
                            y: finger.current_position.y - prev.y,
                        },
                        phase: Phase::Changed,
                    }));
                    self.last_drag_position = Some(finger.current_position);
                }
            }
            GestureState::LongPressing => {
                // Movement during long press -- could transition to drag
                if let Some(finger) = self.tracker.get(id)
                    && finger.moved_past_slop
                {
                    self.state = GestureState::Dragging;
                    self.last_drag_position = Some(finger.current_position);
                    self.pending_events.push(GestureEvent::Drag(DragEvent {
                        position: finger.current_position,
                        start_position: finger.start_position,
                        delta: Point { x: 0.0, y: 0.0 },
                        phase: Phase::Started,
                    }));
                }
            }
            GestureState::Pinching => {
                if let (Some(dist), Some(center)) =
                    (self.tracker.finger_distance(), self.tracker.finger_center())
                    && let Some(initial) = self.initial_pinch_distance
                    && initial > 0.0
                {
                    let scale = dist / initial;
                    self.pending_events.push(GestureEvent::Pinch(PinchEvent {
                        center,
                        scale,
                        delta_scale: scale - self.last_pinch_scale,
                        phase: Phase::Changed,
                    }));
                    self.last_pinch_scale = scale;
                }
            }
            GestureState::Idle | GestureState::WaitingForSecondTap => {
                // Unexpected move in idle -- ignore
            }
        }
    }

    // -----------------------------------------------------------------------
    // Internal helper (used by touch_start)
    // -----------------------------------------------------------------------

    pub(super) fn transition_to_pinch(&mut self) {
        self.state = GestureState::Pinching;
        self.initial_pinch_distance = self.tracker.finger_distance();
        self.last_pinch_scale = 1.0;
        if let Some(center) = self.tracker.finger_center() {
            self.pending_events.push(GestureEvent::Pinch(PinchEvent {
                center,
                scale: 1.0,
                delta_scale: 0.0,
                phase: Phase::Started,
            }));
        }
    }
}
