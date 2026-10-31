---
id: task-15
title: "task-15: pipeline DSL -> builder -> compositor"
project: plev
type: task
status: done
date: 2021-10-10
date_basis: header
topics: [dsl, rendering]
relations: [task-01, task-05, task-14]
source: [docs/log/steps/task-15.md, docs/log/changes/task-15.md]
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# task-15: pipeline DSL -> builder -> compositor

Records the end-to-end connection between `plev_narrate!` and the real builder: the macro generates code that produces `SceneNode`s rendered on screen. The stubs were replaced with a bridge to `plev::builder`, with numeric and text conversions and the methods that were missing. The result is the `narrate_demo` example, which compiles and renders, and the suite reached 207 passing tests.

## objetivo
`plev_narrate!` gera código que roda end-to-end: produz scenenodes renderizados na tela.

## dependências
- task-05 (builder API)
- task-14 (plev_narrate! DSL)
- task-01 (view trait)

## checklist
- [x] bridge plev_narrate -> builder real (remover stubs, re-exportar plev::builder)
- [x] adicionar `plev` como dependência em `crates/plev_narrate/Cargo.toml`
- [x] element implementa conversão para vec<scenenode> (view trait, flatten recursivo)
- [x] intof32 trait para aceitar int e float nos métodos do builder
- [x] intoradius trait para rounded() aceitar presets nomeados
- [x] intoview for &str/string, habilita .child("text") pattern do DSL
- [x] text content merge: text("").child("hello") seta conteúdo
- [x] métodos faltantes: centered, px/py, bold/italic, child_if, children_each, etc.
- [x] exemplo end-to-end: `examples/narrate_demo.rs`
- [x] 207 testes passando (141 plev + 12 integration + 54 macro)
- [x] exemplo compila e renderiza visualmente

## arquivos modificados
- `crates/plev_narrate/src/lib.rs` (stubs -> real bridge)
- `crates/plev_narrate/Cargo.toml` (add plev dependency)
- `src/builder.rs` (intof32, intoradius, missing methods, text merge)
- `Cargo.toml` (plev_narrate dev-dependency)
- `examples/narrate_demo.rs` (novo)

## Delivery record

### sessão 1 (2021-10-06)

#### análise
- codegen usa `::plev_narrate::builder::*` com construtores no-arg (`text()`, `button()`)
- `show "Hello"` gera `.child("Hello")`, precisa `IntoView for &str/String`
- DSL gera `.gap(4)` (int literal), real builder aceita f32, precisa trait numérico
- stubs aceitam qualquer tipo via `<V>`, real builder precisa métodos tipados
- método `centered()` no DSL é alias de `center()`
