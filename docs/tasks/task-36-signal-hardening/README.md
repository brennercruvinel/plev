---
id: task-36
title: "task-36: signal system hardening"
project: plev
type: task
status: done
date: 2023-08-28
date_basis: header
topics: [signals, performance]
relations: [task-04, task-34, research-extracted-patterns]
source: [docs/log/steps/task-36-signal-hardening.md, docs/log/changes/task-36.md]
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# task-36: signal system hardening

Records four improvements to the signal system, inspired by the patterns of leptos, dioxus and slint: subscribers in `FxIndexSet`, a RAII guard for the observer stack, `peek()` without creating a dependency, and a sentinel for constant signals. Three were done, and the current code has each one (`rust/engine/src/runtime/signal/runtime.rs:13`, `execution.rs:14` and `api.rs:56`); the sentinel (f3) was left out for a safety reason the page does not detail. The effect is correctness in the face of a panic inside `create_effect`, and O(1) lookup in subscribers.

## objetivo
corrigir bugs latentes e melhorar performance do signal system. 4 patterns extraidos de leptos/dioxus/slint.

## justificativa
bugs de corretude (panic corruption) e performance (o(n) subscribers). descobertos via task-34 pattern extraction.

## dependencias
- task-04 (signal system), concluida
- indexmap crate necessario para fxindexset

## referencia
- patterns f1, f2, f3, f4 em [`extracted-patterns.md`](../../research/extracted-patterns.md)

## estimativa
~150 LOC

## checklist

### f1: fxindexset para subscribers (corretude + perf)
- [x] adicionar `indexmap` ao cargo.toml
- [x] substituir `Vec<NodeId>` por `FxIndexSet<NodeId>` em subscribers/sources
- [x] garantir iteration em ordem de insercao (outer effects antes de inner)
- [x] testes: verificar o(1) contains, sem duplicatas

### f4: RAII observer drop guard (corretude)
- [x] criar `struct ObserverGuard` que restaura observer anterior no drop
- [x] substituir push/pop explicito no observer stack por observerguard
- [x] testar: panic dentro de create_effect nao corrompe observer stack

### f2: readsignal::peek() (feature)
- [x] adicionar `pub fn peek(&self) -> T` que le sem se inscrever como subscriber
- [x] testes: peek() nao cria dependencia, set() nao re-executa efeito que so fez peek

### f3: constant-signal sentinel (otimizacao)
- [ ] signals que nunca tiveram set() chamado apontam para sentinel
- [ ] skip tracking para reads de constant-signals
- [ ] testes: signal constante nao registra subscribers

## criterios de aceite
1. zero panic corruption quando create_effect panic (RAII guard)
2. subscribers em fxindexset (o(1) contains, ordem preservada)
3. peek() funciona sem criar dependencia
4. constant-signal nao registra tracking overhead
5. zero regressao nos testes existentes de signal

## Delivery record

### summary
hardened the signal system with 3 of 4 planned improvements from leptos/dioxus/slint patterns. f3 (constant-signal sentinel) was intentionally skipped for safety reasons.

### changes

#### f1: fxindexset for subscribers
- added indexmap crate dependency
- replaced vec<nodeid> with fxindexset<nodeid> for subscribers/sources
- o(1) contains, no duplicates, insertion-order iteration preserved

#### f4: RAII observer drop guard
- created observerguard struct with drop impl that restores previous observer
- replaced explicit push/pop on observer stack
- panic inside create_effect no longer corrupts observer stack

#### f2: readsignal::peek()
- added peek() method that reads value without subscribing as dependent
- tests confirm peek() creates no dependency tracking

#### f3: constant-signal sentinel, skipped
- intentionally not implemented for safety reasons
- items left unchecked in task file

### status
done (f1, f2, f4 complete; f3 skipped)
