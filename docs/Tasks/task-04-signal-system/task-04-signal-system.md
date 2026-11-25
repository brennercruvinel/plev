---
id: task-04
title: "task-04: signal system (reactivity)"
project: plev
type: task
status: done
date: 2021-05-06
date_basis: reconstructed
recorded: 2026-03-08
topics: [signals, process]
relations: []
source: [docs/log/steps/task-04.md, docs/log/changes/task-04.md]
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# task-04: signal system (reactivity)

Records the reactive primitives of plev: `create_signal`, `create_effect` and `create_memo`, with automatic dependency tracking and batching of updates. The implementation is a thread-local runtime with a hybrid push-pull model, comparison in the memo to cut propagation, and cycle detection, with 15 reactivity tests and no GPU dependency. When a signal changes, only the views that depend on it are redone, and that is the basis of granular re-rendering.

## objetivo
implementar primitivas reativas: `create_signal`, `create_effect`, `create_memo`. signals são a base para re-render granular, quando um signal muda, apenas as views que dependem dele re-renderizam.

## checklist de conclusão
- [x] `create_signal<T>(initial: T) -> (ReadSignal<T>, WriteSignal<T>)`
- [x] `create_effect(f: impl Fn())`, re-executa quando signals lidos dentro de `f` mudam
- [x] `create_memo(f: impl Fn() -> T) -> ReadSignal<T>`, computed value cacheado
- [x] tracking automático de dependências (quem lê qual signal)
- [x] batching de updates (múltiplos set_signal num frame = um re-render)
- [x] exemplo funcional: counter reativo com signal (`examples/signal_counter.rs`)
- [x] `cargo build` passa sem warnings relevantes
- [x] `cargo test` passa com 15 testes de reatividade (19 total)
- [x] zero alocações em steady state (signal não mudou = zero work)

## concluída em: 2021-05-06
## branch: `task/TASK-04-signal-system`
## commits: f676b7d, 188fb7e

## Delivery record

### 2021-05-02
- iniciada implementação do signal system (reatividade)
- branch: `task/TASK-04-signal-system` a partir de master (e6f1c7e)
- adicionado `slotmap = "1.1"` ao cargo.toml
- criado `src/signal.rs` (~550 linhas) com runtime reativo completo:
  - `create_signal`, `create_effect`, `create_memo`, `batch`, `dispose_node`
  - push-pull hybrid: set() pushes dirty/check, effects pull via sources_changed
  - thread-local runtime com slotmap<nodeid, reactivenode>
  - closures armazenadas como rc<dyn fn> para evitar reentrance no refcell
  - borrow do refcell liberado antes de executar qualquer closure de usuário
  - memo com comparefn type-erased para cortar propagação (diamond problem)
  - detecção de ciclos via flag `running` em notify_subscribers
- adicionado `pub mod signal;` em `src/lib.rs`
- 15 testes unitários passando (zero dependência de GPU)
- criado `examples/signal_counter.rs` demonstrando signals + views + compositor

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [task-02: component state management](../task-02-component-state-management/task-02-component-state-management.md)
- [task-05: declarative API + builder pattern](../task-05-declarative-api-builder-pattern/task-05-declarative-api-builder-pattern.md)
- [task-07: independent layer system](../task-07-independent-layer-system/task-07-independent-layer-system.md)
- [task-14: plev_narrate!, verbal DSL](../task-14-plev-narrate-dsl-verbal/task-14-plev-narrate-dsl-verbal.md)
- [task-27: animation system](../task-27-animation-system/task-27-animation-system.md)
- [task-36: signal system hardening](../task-36-signal-hardening/task-36-signal-hardening.md)
- [component state design (task-02)](../../Research/Structure/component-design.md)
- [signal system design](../../Research/Structure/signal-system-design.md)
<!-- /related -->
