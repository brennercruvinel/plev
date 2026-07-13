---
project: plev
audience: [ai-agents, contributors]
status: done
last-updated: 2023-08-23
domain: changelog
---

# task-36 changelog: signal system hardening

## summary
hardened the signal system with 3 of 4 planned improvements from leptos/dioxus/slint patterns. f3 (constant-signal sentinel) was intentionally skipped for safety reasons.

## changes

### f1: fxindexset for subscribers
- added indexmap crate dependency
- replaced vec<nodeid> with fxindexset<nodeid> for subscribers/sources
- o(1) contains, no duplicates, insertion-order iteration preserved

### f4: RAII observer drop guard
- created observerguard struct with drop impl that restores previous observer
- replaced explicit push/pop on observer stack
- panic inside create_effect no longer corrupts observer stack

### f2: readsignal::peek()
- added peek() method that reads value without subscribing as dependent
- tests confirm peek() creates no dependency tracking

### f3: constant-signal sentinel, skipped
- intentionally not implemented for safety reasons
- items left unchecked in task file

## status
done (f1, f2, f4 complete; f3 skipped)
