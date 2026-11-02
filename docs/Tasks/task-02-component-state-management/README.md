---
id: task-02
title: "task-02: component state management"
project: plev
type: task
status: done
date: 2021-03-28
date_basis: reconstructed
recorded: 2026-03-13
topics: [signals, process]
relations: [task-01, task-04, architecture-component-design]
source: [docs/log/steps/task-02.md, docs/log/changes/task-02.md]
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# task-02: component state management

Records the internal state of views: the `Component<L>` struct, the `on_mount`, `on_update` and `on_unmount` hooks, and typed access to the state through `state()` and `state_mut()`. The final design had render receive `&Self::State` directly, instead of passing the state through the viewcontext. The delivery has 13 tests (9 for components and 4 for views), and the state survives between frames without reallocation. The component is a prerequisite for task-04's signals, because each instance now has its own state.

## objetivo
adicionar estado interno às views, lifecycle de componente (mount, update, unmount) e armazenamento de estado local sem globals.

## contexto
com o view trait (task-01), views são stateless. para componentes interativos, cada instância precisa de estado próprio (ex: counter, toggle, scroll position). o state management é pré-requisito para signals (task-04).

## dependências
- task-01 (view trait + viewcontext)

## checklist de conclusão
- [x] struct `Component<L>` que wrapa um lifecycle com estado genérico `L::State`
- [x] lifecycle hooks: `on_mount`, `on_update`, `on_unmount`
- [x] estado acessível via `state()` / `state_mut()` (acessores diretos, type-safe)
- [x] exemplo funcional: componente counter com estado interno (`examples/counter.rs`)
- [x] `cargo build` passa sem warnings relevantes
- [x] `cargo test` passa (9 testes de component + 4 de view = 13 total)
- [x] estado sobrevive entre frames sem re-alocação (teste de 100 frames)

## nota de design
o plano original mencionava estado acessível via viewcontext durante render. o design final usa acessores `state()` / `state_mut()` no component, e o render do lifecycle recebe `&Self::State` diretamente. mais simples, type-safe, sem genericviewcontext.

## Delivery record

### 2021-03-24
- criado `src/component.rs` com trait `Lifecycle` e struct `Component<L>`
- adicionado `pub mod component;` em `src/lib.rs`
- integrado `Component<Counter>` no `window.rs` (app principal)
- criado `examples/counter.rs` (exemplo standalone)
- 9 testes unitários passando
- registrado design em [`component-design.md`](../../Research/Structure/component-design.md)
