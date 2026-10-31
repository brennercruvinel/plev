---
id: task-02
title: "task-02: component state management"
project: plev
type: tarefa
status: concluido
date: 2021-03-28
date_basis: cabecalho
topics: [sinais, processo]
relations: [task-01, task-04, arquitetura-component-design]
source: [docs/log/steps/task-02.md, docs/log/changes/task-02.md]
source_revision: ec0a224c
revised: 2026-10-30
review: revisado
---

# task-02: component state management

Registra o estado interno das views: o struct `Component<L>`, os hooks `on_mount`, `on_update` e `on_unmount`, e o acesso tipado ao estado por `state()` e `state_mut()`. O design final fez o render receber `&Self::State` diretamente, em vez de passar o estado pelo viewcontext. A entrega tem 13 testes (9 de componente e 4 de view), e o estado sobrevive entre frames sem realocação. O componente é pré-requisito dos signals da task-04, porque cada instância passa a ter estado próprio.

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

## Registro de entrega

### 2021-03-24
- criado `src/component.rs` com trait `Lifecycle` e struct `Component<L>`
- adicionado `pub mod component;` em `src/lib.rs`
- integrado `Component<Counter>` no `window.rs` (app principal)
- criado `examples/counter.rs` (exemplo standalone)
- 9 testes unitários passando
- registrado design em [`component-design.md`](../referencia/arquitetura/component-design.md)
