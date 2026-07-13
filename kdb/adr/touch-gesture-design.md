---
project: plev
audience: [ai-agents, contributors]
status: reference
last-updated: 2022-10-27
domain: input
---

# touch & gesture recognition design

## architecture
the touch/gesture system is layered:

1. **winit::event::touch** - raw OS events with `{device_id, phase, location, force, id}`
2. **touchtracker** - per-finger state management (fxhashmap<u64, touchfinger>)
3. **gesturerecognizer** - 6-state machine that classifies touch sequences into gestures
4. **touchinputstate** - public wrapper that bridges winit touch events to gesturerecognizer

## state machine
six states: idle, possibletap, waitingforsecondtap, dragging, longpressing, pinching.

key transitions:
- **touch down** in idle -> possibletap
- **move past touch_slop** in possibletap -> dragging
- **tick() past long_press_duration** in possibletap -> longpressing
- **lift within tap_max** in possibletap -> waitingforsecondtap (emits tap)
- **second finger** in any single-finger state -> pinching
- **second tap within double_tap_timeout** in waitingforsecondtap -> emits doubletap -> idle
- **fast lift with distance** in dragging -> emits both drag(ended) and swipe

## thresholds
all constants in `src/input/mod.rs`:
- touch_slop: 10px - max movement for tap
- tap_max_duration: 300ms - max hold for tap
- long_press_duration: 500ms - min hold for long-press
- double_tap_timeout: 300ms - max gap between taps
- double_tap_slop: 100px - max distance between taps
- swipe_min_vel: 200px/s - min velocity for swipe
- swipe_min_dist: 50px - min distance for swipe

## testability
all gesturerecognizer methods take explicit `Instant` instead of calling `Instant::now()`. tests use `Instant::now()` as a base and add `Duration::from_millis()` to simulate timing.

## platform notes
- **macos**: does not emit windowevent::touch events. touch testing requires mobile or touchscreen device.
- **winit 0.30.13**: touch struct has fields `{device_id: DeviceId, phase: TouchPhase, location: PhysicalPosition<f64>, force: Option<Force>, id: u64}`
- **android/ios**: will receive touch events natively. the gesture recognizer is platform-agnostic.

## coexistence with mouse
touchinputstate and inputstate are completely separate. on touchscreen desktops, both systems work independently without conflict. no mutex or shared state.
