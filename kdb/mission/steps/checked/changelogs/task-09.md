---
project: phi
audience: [ai-agents, contributors]
status: done
last-updated: 2026-03-08
domain: changelog
---

# task-09 changelog

## 2026-03-08, implementacao completa

### decisoes
- event queue (nao closures), evita problemas de borrow checker
- hit-testing linear scan reverso, z-order por ordem de registro
- bubbling only (sem capture phase), simplicidade
- click-to-focus, escape-to-blur, foco simples
- modifierstate proprio do φ, abstrai winit::modifiers
- keyinput enum com named(namedkey) e character(string), mapeia winit::key

### arquivos criados/modificados
- `src/input.rs` (novo, ~500 linhas), inputstate, event types, hit-testing, 16 testes
- `src/lib.rs`, adicionado `pub mod input;`
- `src/window.rs`, inputstate no app, match arms para eventos de input
- `examples/input_demo.rs` (novo), botao interativo com hover + click counter

### resultado
- `cargo check` passa
- `cargo test --lib`, 20 testes (16 input + 4 view), todos passam
- `cargo check --example input_demo` passa
