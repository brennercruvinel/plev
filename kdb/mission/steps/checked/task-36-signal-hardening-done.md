---
project: phi
audience: [ai-agents, contributors]
status: done
last-updated: 2026-03-13
domain: task-tracking
---

# task-36: signal system hardening, P0, done

## objetivo
corrigir bugs latentes e melhorar performance do signal system. 4 patterns extraidos de leptos/dioxus/slint.

## justificativa
bugs de corretude (panic corruption) e performance (o(n) subscribers). descobertos via task-34 pattern extraction.

## dependencias
- task-04 (signal system), concluida
- indexmap crate necessario para fxindexset

## referencia
- patterns f1, f2, f3, f4 em `mission/knowledge/extracted-patterns.md`

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
