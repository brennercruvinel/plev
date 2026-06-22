---
project: plev
audience: [ai-agents, contributors]
status: done
last-updated: 2026-03-13
domain: task-tracking
---

# task-02: component state management

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
