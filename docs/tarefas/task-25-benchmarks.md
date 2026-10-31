---
id: task-25
title: "task-25: benchmarks"
project: plev
type: tarefa
status: concluido
date: 2022-05-03
date_basis: cabecalho
topics: [task-tracking]
relations: []
source: [docs/log/steps/task-25-benchmarks.md, docs/log/changes/task-25.md]
source_revision: ec0a224c
revised: 2026-10-30
review: pendente
---

# task-25: benchmarks

## resultado
suite de criterion benchmarks cobrindo scene construction, dirty tracking, tessellation, signals e text hashing. resultados documentados em [`benchmarks-v02.md`](../experimentos/benchmarks-v02.md).

## implementacao
- `benches/scene_build.rs`, 6 benchmark groups, ~190 LOC
- `Cargo.toml`, criterion 0.5 dev-dependency
- [`benchmarks-v02.md`](../experimentos/benchmarks-v02.md), resultados tabulados

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

## Registro de entrega

### novos arquivos
- `benches/scene_build.rs`, criterion benchmarks
- [`benchmarks-v02.md`](../experimentos/benchmarks-v02.md), resultados

### modificados
- `Cargo.toml`, criterion dev-dependency, [[bench]] config
