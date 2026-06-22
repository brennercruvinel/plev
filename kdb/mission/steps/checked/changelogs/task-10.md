---
project: plev
audience: [ai-agents, contributors]
status: done
last-updated: 2026-03-08
domain: changelog
---

# changelog, task-10: touch + gesture recognition

## 2026-03-08

### implementado
- converted `src/input.rs` to `src/input/mod.rs` (module directory) to accommodate sub-modules
- created `src/input/touch.rs`, touchfinger per-finger state tracking + touchtracker multi-finger management via fxhashmap
- created `src/input/gesture.rs`, gesturerecognizer with 6-state machine (idle, possibletap, waitingforsecondtap, dragging, longpressing, pinching)
- added gesture types to `src/input/mod.rs`: point, phase, swipedirection, gestureevent enum (tap, doubletap, longpress, swipe, drag, pinch), touchinputstate wrapper
- defined gesture thresholds as constants: touch_slop=10px, tap_max=300ms, long_press=500ms, double_tap_timeout=300ms, double_tap_slop=100px, swipe_min_vel=200px/s, swipe_min_dist=50px
- wired windowevent::touch in `src/window.rs`, feeds into touchinputstate, tick() called each frame in redrawrequested
- created `examples/touch_demo.rs`, interactive demo with draggable rect responding to all 6 gesture types

### decisões
- used drain event queue pattern (consistent with task-09 inputevent) instead of per-gesture closures, avoids borrow checker issues with closures
- gesturerecognizer takes explicit instant for all methods, enables deterministic testing
- touchinputstate is separate from inputstate, touch and mouse systems coexist independently
- all timing uses std::time::instant (not platform-specific timers)

### testes
- 18 gesture tests: tap, tap_rejected_moved, double_tap, double_tap_rejected_far, long_press, long_press_rejected_moved, swipe_right, swipe_up, drag_lifecycle, pinch, cancel_mid_drag, three_fingers, long_press_then_drag, drag_to_pinch, swipe_too_slow_is_drag, tap_too_slow_is_ignored, double_tap_timeout
- 4 touch tracker tests: start_and_get, update_tracks_distance, moved_past_slop, finger_distance_two_fingers
- total: 116 lib tests pass, zero warnings
