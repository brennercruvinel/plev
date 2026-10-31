---
id: task-09
title: "task-09: input system, keyboard + mouse"
project: plev
type: task
status: done
date: 2021-07-19
date_basis: reconstructed
recorded: 2026-03-13
topics: [input, layout]
relations: [task-01, task-03, task-07]
source: [docs/log/steps/task-09.md, docs/log/changes/task-09.md]
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# task-09: input system, keyboard + mouse

Records the implementation of keyboard and mouse input: winit events are translated into plev's own types (click, key, hover and scroll) and dispatched to the views by hit-testing, with an event queue instead of closures. The result is the `input_demo` example, with hover, click and basic focus, and 20 library tests passing. It is the base that task-10 (touch and gestures) and task-17 (hit-testing per layer) extend.

## objetivo
implementar sistema de input para teclado e mouse/pointer. events de winit são traduzidos para o modelo de eventos do plev e despachados para views via hit-testing.

## contexto
winit já entrega `WindowEvent::KeyboardInput`, `MouseInput`, `CursorMoved`, etc. o plev precisa: (1) traduzir para eventos próprios, (2) fazer hit-testing para determinar qual view recebe o evento, (3) despachar para handlers registrados via `.on_click()`, `.on_key()`.

## dependências
- task-01 (view trait, views registram handlers)
- task-03 (layout engine, hit-testing precisa de bounds calculados)

## checklist de conclusão
- [x] event types próprios: `ClickEvent`, `plevKeyEvent`, `HoverEvent`, `ScrollEvent`
- [x] hit-testing: dado cursor position, determinar qual view está sob o cursor (respeitando z-order)
- [x] despacho de eventos: event queue com hit-testing reverso (bubbling aproximado por containment geométrico)
- [x] focus system básico: click-to-focus (se focusable), escape-to-blur, click fora limpa focus
- [x] integração com winit `WindowEvent` no `window.rs`
- [x] exemplo funcional: `input_demo.rs`, botão com hover (cor muda) e click (counter incrementa)
- [x] `cargo check` passa sem erros
- [x] `cargo test --lib` passa (20/20 testes, 16 de input)
- [ ] hit-testing funciona com camadas (task-07 não implementado, n/a)

## armadilhas
- hit-testing deve usar spatial index se > 100 views, linear scan é o(n) por evento
- focus e tab-order são complexos, começar simples (click para focus, escape para unfocus)
- não processar input no render loop, separar fase de input de fase de render
- winit keyboard events variam por plataforma, usar `key` (logical) não `scancode` (physical)

## Delivery record

### 2021-07-15, implementacao completa

#### decisoes
- event queue (nao closures), evita problemas de borrow checker
- hit-testing linear scan reverso, z-order por ordem de registro
- bubbling only (sem capture phase), simplicidade
- click-to-focus, escape-to-blur, foco simples
- modifierstate proprio do plev, abstrai winit::modifiers
- keyinput enum com named(namedkey) e character(string), mapeia winit::key

#### arquivos criados/modificados
- `src/input.rs` (novo, ~500 linhas), inputstate, event types, hit-testing, 16 testes
- `src/lib.rs`, adicionado `pub mod input;`
- `src/window.rs`, inputstate no app, match arms para eventos de input
- `examples/input_demo.rs` (novo), botao interativo com hover + click counter

#### resultado
- `cargo check` passa
- `cargo test --lib`, 20 testes (16 input + 4 view), todos passam
- `cargo check --example input_demo` passa
