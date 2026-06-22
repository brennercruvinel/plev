---
project: plev
audience: [ai-agents, contributors]
status: done
last-updated: 2026-03-13
domain: task-tracking
---

# task-25: benchmarks, done

## resultado
suite de criterion benchmarks cobrindo scene construction, dirty tracking, tessellation, signals e text hashing. resultados documentados em `mission/knowledge/benchmark-results.md`.

## implementacao
- `benches/scene_build.rs`, 6 benchmark groups, ~190 LOC
- `Cargo.toml`, criterion 0.5 dev-dependency
- `mission/knowledge/benchmark-results.md`, resultados tabulados

## key metrics (m4 mac)
- push_rects: 159-222m rects/s
- dirty tracking: 3.3us/1000 rects (static scene)
- tessellation: 1.5-3.7us/shape
- signals: 67ns/cycle

## checklist
- [x] scene construction benchmarks (rects, paths)
- [x] dirty tracking effectiveness
- [x] lyon tessellation benchmarks
- [x] signal throughput
- [x] text node hashing
- [x] results document
