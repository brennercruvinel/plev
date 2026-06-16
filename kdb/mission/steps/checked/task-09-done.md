---
project: plev
audience: [ai-agents, contributors]
status: done
last-updated: 2026-03-13
domain: task-tracking
---

# task-09: input system, keyboard + mouse

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

## workflow
- ao iniciar: mover este arquivo para `mission/steps/ongoing/`
- ao concluir: renomear para `TASK-09-DONE.md`, mover para `mission/steps/checked/`
