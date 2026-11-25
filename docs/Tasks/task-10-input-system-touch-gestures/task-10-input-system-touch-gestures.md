---
id: task-10
title: "task-10: input system, touch + gestures"
project: plev
type: task
status: done
date: 2021-08-02
date_basis: reconstructed
recorded: 2026-03-13
topics: [input, mobile]
relations: [task-03, task-09]
source: [docs/log/steps/task-10.md, docs/log/changes/task-10.md]
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# task-10: input system, touch + gestures

Records the extension of input to multi-touch and gestures (tap, double tap, long press, drag, swipe in the four directions and pinch), with a state-machine recognizer and a touch state separate from the mouse. The result is the `touch_demo` example, with gesture and finger-tracking tests and `cargo check` without warnings. Task-11 considers touch covered by this task.

## objetivo
estender o input system para suportar touch (multi-touch) e reconhecimento de gestos (tap, swipe, pinch, long-press). essencial para mobile.

## contexto
winit entrega `Touch` events com touch id, position e phase (started, moved, ended, cancelled). o plev precisa traduzir sequências de touch em gestos reconhecidos e despachá-los para views.

## dependências
- task-09 (input system base, keyboard + mouse)
- task-03 (layout engine, hit-testing)

## checklist de conclusão
- [x] touch event handling via winit `Touch`
- [x] gesture recognizer: tap, double-tap, long-press, swipe (4 direções), pinch-to-zoom
- [x] multi-touch tracking (touch ids independentes)
- [x] gesture handlers em views: gestureevent enum com drain pattern (consistente com inputevent)
- [x] coexistência touch + mouse (desktop com touchscreen), sistemas independentes, sem conflito
- [x] exemplo funcional: touch_demo com elemento arrastável, tap/double-tap/long-press/swipe/pinch
- [x] `cargo check` passa sem warnings
- [x] `cargo test --lib` passa com 21 testes de touch/gesture (17 gesture + 4 touch tracker)

## nota sobre escopo
- item "gesture handlers em views: `.on_tap()`, `.on_swipe()`, `.on_pinch()`" foi adaptado para usar o padrão de event queue (drain pattern) já estabelecido em task-09, em vez de closures. isso é mais consistente com a arquitetura existente e evita problemas de borrow checker com closures.

## armadilhas confirmadas
- gesture recognition tem timing (long-press = touch > 500ms sem mover), resolvido com tick() chamado a cada frame
- pinch-to-zoom precisa de 2+ touches simultâneos, touchtracker rastreia por finger id
- touch cancel (ex: notificação do OS) limpa estado do gesture recognizer e emite phase::cancelled
- state machine non-blocking, gesturerecognizer é 6-state: idle, possibletap, waitingforsecondtap, dragging, longpressing, pinching
- macos não emite windowevent::touch, documentado no exemplo

## Delivery record

### 2021-07-29

#### implementado
- converted `src/input.rs` to `src/input/mod.rs` (module directory) to accommodate sub-modules
- created `src/input/touch.rs`, touchfinger per-finger state tracking + touchtracker multi-finger management via fxhashmap
- created `src/input/gesture.rs`, gesturerecognizer with 6-state machine (idle, possibletap, waitingforsecondtap, dragging, longpressing, pinching)
- added gesture types to `src/input/mod.rs`: point, phase, swipedirection, gestureevent enum (tap, doubletap, longpress, swipe, drag, pinch), touchinputstate wrapper
- defined gesture thresholds as constants: touch_slop=10px, tap_max=300ms, long_press=500ms, double_tap_timeout=300ms, double_tap_slop=100px, swipe_min_vel=200px/s, swipe_min_dist=50px
- wired windowevent::touch in `src/window.rs`, feeds into touchinputstate, tick() called each frame in redrawrequested
- created `examples/touch_demo.rs`, interactive demo with draggable rect responding to all 6 gesture types

#### decisões
- used drain event queue pattern (consistent with task-09 inputevent) instead of per-gesture closures, avoids borrow checker issues with closures
- gesturerecognizer takes explicit instant for all methods, enables deterministic testing
- touchinputstate is separate from inputstate, touch and mouse systems coexist independently
- all timing uses std::time::instant (not platform-specific timers)

#### testes
- 17 gesture tests: tap, tap_rejected_moved, double_tap, double_tap_rejected_far, long_press, long_press_rejected_moved, swipe_right, swipe_up, drag_lifecycle, pinch, cancel_mid_drag, three_fingers, long_press_then_drag, drag_to_pinch, swipe_too_slow_is_drag, tap_too_slow_is_ignored, double_tap_timeout
- 4 touch tracker tests: start_and_get, update_tracks_distance, moved_past_slop, finger_distance_two_fingers
- total: 116 lib tests pass, zero warnings

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [task-03: layout engine (flexbox-like)](../task-03-layout-engine/task-03-layout-engine.md)
- [task-09: input system, keyboard + mouse](../task-09-input-system-keyboard-mouse/task-09-input-system-keyboard-mouse.md)
- [task-11: android build + lifecycle](../task-11-android-build-lifecycle/task-11-android-build-lifecycle.md)
- [task-13: mobile specifics, safe areas, IME, lifecycle](../task-13-mobile-specifics-safe-areas-ime-lifecycle/task-13-mobile-specifics-safe-areas-ime-lifecycle.md)
<!-- /related -->
