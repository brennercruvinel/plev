---
id: task-38
title: "task-38: event batching"
project: plev
type: task
status: done
date: 2023-11-09
date_basis: header
topics: [input, performance]
relations: [task-09, research-extracted-patterns]
source: [docs/log/steps/task-38-event-batching.md, docs/log/changes/task-38.md]
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# task-38: event batching

Records the batching of winit's input events: the events that arrive between two frames are accumulated and applied at once, with a single render, to reduce GPU work during fast input. The target of five to ten times fewer render passes comes from yazi's d1 pattern, and the body does not carry the measurement of the informal benchmark that the checklist marks as done. The buffer and the batch application are in `rust/engine/src/runtime/window/events.rs`.

## objetivo
batch-drain de eventos winit para reduzir trabalho GPU durante input rapido. 5-10x reducao.

## justificativa
cada evento winit dispara scene rebuild + GPU submit. em touch 120hz+ ou keyboard burst, isso e o(n) renders por frame. batch-drain acumula eventos e processa todos antes de um unico render.

## dependencias
- task-09 (input), concluida

## referencia
- pattern d1 em [`extracted-patterns.md`](../../research/extracted-patterns.md)
- yazi source: `bunker/repos/devtools/tui-apps/yazi/yazi-fm/src/app/app.rs`

## estimativa
~50-80 LOC

## checklist
- [x] em `window_event()`: acumular eventos em `Vec<InputEvent>` ao inves de processar imediatamente
- [x] em `about_to_wait()`: drenar todos os eventos acumulados, processar, depois um unico `compositor.resolve()` + render_pass
- [x] limite de batch: max 50 eventos por iteracao (evitar starvation)
- [x] testes: verificar que multiplos key events no mesmo frame produzem resultado correto
- [x] benchmark informal: contar render passes por segundo com input rapido (antes vs depois)

## criterios de aceite
1. multiplos eventos no mesmo frame = um unico render pass
2. input nao perde eventos (todos processados, so render que e batched)
3. zero regressao visual ou funcional
4. latencia de input nao aumenta perceptivelmente (max 1 frame de delay)

## Delivery record

### summary
implemented batch-drain pattern for winit events to reduce GPU work during rapid input. events are accumulated in window_event() and processed in bulk during about_to_wait() before a single render pass.

### changes
- events accumulated in vec<inputevent> in window_event() instead of immediate processing
- batch drain in about_to_wait(): all events processed, then single compositor.resolve() + render_pass
- batch limit of 50 events per iteration to prevent starvation
- no input events lost, all processed, only rendering is batched
- latency stays within 1 frame of delay maximum

### status
done
