---
project: plev
audience: [ai-agents, contributors]
status: done
last-updated: 2026-03-13
domain: task-tracking
---

# task-10: input system, touch + gestures (done)

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
- [x] `cargo test --lib` passa com 22 testes de touch/gesture (18 gesture + 4 touch tracker)

## nota sobre escopo
- item "gesture handlers em views: `.on_tap()`, `.on_swipe()`, `.on_pinch()`" foi adaptado para usar o padrão de event queue (drain pattern) já estabelecido em task-09, em vez de closures. isso é mais consistente com a arquitetura existente e evita problemas de borrow checker com closures.

## armadilhas confirmadas
- gesture recognition tem timing (long-press = touch > 500ms sem mover), resolvido com tick() chamado a cada frame
- pinch-to-zoom precisa de 2+ touches simultâneos, touchtracker rastreia por finger id
- touch cancel (ex: notificação do OS) limpa estado do gesture recognizer e emite phase::cancelled
- state machine non-blocking, gesturerecognizer é 6-state: idle, possibletap, waitingforsecondtap, dragging, longpressing, pinching
- macos não emite windowevent::touch, documentado no exemplo
