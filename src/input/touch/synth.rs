//! Touch -> pointer synthesis (touch-screen compatibility layer).

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

#[cfg(test)]
mod tests {
    use super::*;
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
        assert!(
            synth
                .synthesize(2, TouchPhase::Started, 90.0, 90.0)
                .is_empty()
        );
        assert!(
            synth
                .synthesize(2, TouchPhase::Moved, 95.0, 95.0)
                .is_empty()
        );
        assert!(
            synth
                .synthesize(2, TouchPhase::Ended, 99.0, 99.0)
                .is_empty()
        );
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
        assert!(
            synth
                .synthesize(5, TouchPhase::Moved, 10.0, 10.0)
                .is_empty()
        );
        assert!(
            synth
                .synthesize(5, TouchPhase::Ended, 10.0, 10.0)
                .is_empty()
        );
    }
}
