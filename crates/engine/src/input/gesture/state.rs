use crate::input::SwipeDirection;

// ---------------------------------------------------------------------------
// GestureState -- 6-state machine
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum GestureState {
    /// No touch active, waiting for input.
    Idle,
    /// Single finger down, not yet classified.
    PossibleTap,
    /// First tap completed, waiting for second tap within timeout.
    WaitingForSecondTap,
    /// Finger is moving beyond slop -- drag gesture in progress.
    Dragging,
    /// Finger held still past long-press threshold.
    LongPressing,
    /// Two or more fingers active -- pinch gesture.
    Pinching,
}

/// Classify a displacement vector into a cardinal swipe direction.
pub(super) fn classify_swipe(dx: f64, dy: f64) -> SwipeDirection {
    if dx.abs() > dy.abs() {
        if dx > 0.0 {
            SwipeDirection::Right
        } else {
            SwipeDirection::Left
        }
    } else if dy > 0.0 {
        SwipeDirection::Down
    } else {
        SwipeDirection::Up
    }
}
