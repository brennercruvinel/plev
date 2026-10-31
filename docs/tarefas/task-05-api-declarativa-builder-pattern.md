---
id: task-05
title: "task-05: API declarativa + builder pattern"
project: plev
type: tarefa
status: concluido
date: 2021-05-23
date_basis: cabecalho
topics: [dsl, processo]
relations: [task-01, task-03, task-04, task-07, task-08, task-09]
source: [docs/log/steps/task-05.md, docs/log/changes/task-05.md]
source_revision: ec0a224c
revised: 2026-10-30
review: revisado
---

# task-05: API declarativa + builder pattern

Registra a API declarativa de builders: `div()`, `text()` e `button()` encadeáveis, com métodos de layout, estilo e eventos, e o macro `#[component]`, que transforma uma função em view. A entrega inclui o crate `plev_macros` (syn 2 e quote 1), 27 testes e o exemplo `builder_demo`. Os eventos `on_click` e `on_hover` são stubs: a conexão com o input ficou para a task-09, e o layout provisório, empilhamento sequencial, para a task-03.

## objetivo
criar a API ergonômica tipo `div().flex().center().bg("blue").child(...)` e o macro `#[component]` que transforma funções em views.

## dependências
- task-01 (view trait)

## checklist de conclusão
- [x] builder structs: `div()`, `text()`, `button()` que retornam builders encadeáveis
- [x] métodos de layout: `.flex()`, `.center()`, `.p(n)`, `.gap(n)`, `.row()`, `.col()`
- [x] métodos de estilo: `.bg(color)`, `.rounded(r)`, `.shadow(s)`, `.text_color(c)`
- [x] `.child()` e `.children()` para composição
- [x] `.on_click()`, `.on_hover()` para eventos (stubs, input system vem depois)
- [x] macro `#[component]` que transforma `fn(cx: Scope) -> impl View` em componente
- [x] exemplo funcional: mini-app com múltiplos componentes compostos (`builder_demo.rs`)
- [x] `cargo build` passa sem warnings relevantes
- [x] `cargo test` passa (27 testes)

## arquivos criados/modificados
- `src/color.rs`, color struct, intocolor trait, 14 named colors, hex/rgb/rgba
- `src/builder.rs`, element tree, div/text/button, builder chain, naive flatten, scope
- `plev_macros/Cargo.toml`, proc-macro crate (syn 2, quote 1)
- `plev_macros/src/lib.rs`, #[component] attribute macro
- `examples/builder_demo.rs`, visual demo: header, 3 info cards, button row
- `src/lib.rs`, added `pub mod builder; pub mod color;` + re-export macro
- `Cargo.toml`, added `plev_macros` path dependency

## seams para integração futura
- `LayoutConfig` fields armazenados, naive layout (empilha sequencial), task-03 substitui resolve
- `EventHandlers` stubs (on_click, on_hover), task-09 conecta input
- `corner_radius`, `shadow` armazenados no style, task-07/08 renderiza
- `Scope` wrapper mínimo, task-04 adiciona create_signal etc

## Registro de entrega

### 2021-05-19
- task iniciada, branch `task/TASK-05-declarative-api`
- implementado `color.rs`: color, intocolor, constantes nomeadas, hex/rgb/rgba
- implementado `builder.rs`: element tree, div/text/button, builders, flatten, scope
- implementado `plev_macros/`: #[component] proc-macro crate (syn 2 + quote)
- implementado `examples/builder_demo.rs`: demo visual
- 27 testes passando
