---
id: task-25
title: "task-25: benchmarks"
project: plev
type: task
status: done
date: 2022-05-03
date_basis: header
topics: [performance, rendering, signals]
relations: [experiment-benchmarks-v02]
source: [docs/log/steps/task-25-benchmarks.md, docs/log/changes/task-25.md]
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# task-25: benchmarks

Records the criterion benchmark suite of plev, with six groups covering scene construction, dirty tracking, tessellation, signals and hashing of text nodes. The numbers measured on an M4 (Mac) are 159 to 222 million rects per second in push_rects, 3.3 µs per 1000 rects in a static scene, 1.5 to 3.7 µs per shape in tessellation, and 67 ns per signal cycle. It matters because it is the performance measurement that the `benchmarks-v02` experiment tabulates.

## resultado
suite de criterion benchmarks cobrindo scene construction, dirty tracking, tessellation, signals e text hashing. resultados documentados em [`benchmarks-v02/`](../../experiments/benchmarks-v02/README.md).

## implementacao
- `benches/scene_build.rs`, 6 benchmark groups, ~190 LOC
- `Cargo.toml`, criterion 0.5 dev-dependency
- [`benchmarks-v02/`](../../experiments/benchmarks-v02/README.md), resultados tabulados

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

## Delivery record

### novos arquivos
- `benches/scene_build.rs`, criterion benchmarks
- [`benchmarks-v02/`](../../experiments/benchmarks-v02/README.md), resultados

### modificados
- `Cargo.toml`, criterion dev-dependency, [[bench]] config
