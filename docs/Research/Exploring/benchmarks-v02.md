---
id: experiment-benchmarks-v02
title: "benchmark results, plev v0.2"
project: plev
type: experiment
status: reference
date: 2026-03-11
date_basis: text
recorded: 2026-03-11
topics: [performance, rendering, signals, text]
relations: [task-25, task-43, task-44, task-45, task-46]
source: docs/pub/benchmark-results.md
source_revision: 5de80aad
revised: 2026-10-30
review: reviewed
---

# benchmark results, plev v0.2

Measurements of plev v0.2 with criterion, on macOS: scene construction on the CPU, dirty tracking, tessellation with lyon, signals and hashing of text nodes. With rects, scene construction ranges from 0.6 µs (100 rects) to 45 µs (10 thousand rects), and 1000 static rects cost 3.3 µs in dirty tracking, which leaves the static scene practically free after the first frame. Tessellation (1.5 to 3.7 µs per shape) is done once per shape, not every frame. Limit: the June 2026 section lists new benches per crate without numbers on this page, and the header date (2022-04-26) does not match the machine (M4) and the Rust version (1.94.0) cited in the record; the date remains open.

machine: macbook pro m4, macos, rust 1.94.0, criterion 0.5
date: 2026-03-11

## scene construction (CPU-side, no GPU)

| benchmark | time | throughput |
|-----------|------|------------|
| push_rects/100 | 629 ns | 159m rects/s |
| push_rects/1000 | 5.48 us | 183m rects/s |
| push_rects/10000 | 45.0 us | 222m rects/s |
| push_paths/circles_100 | 9.19 us | 10.9m paths/s |
| push_paths/rrects_100 | 6.04 us | 16.6m paths/s |
| push_paths/circles_1000 | 106 us | 9.4m paths/s |
| push_paths/rrects_1000 | 83.7 us | 11.9m paths/s |

## dirty tracking

| benchmark | time | notes |
|-----------|------|-------|
| static_1000_rects (steady state) | 3.31 us | hash comparison only, zero GPU work |

## lyon tessellation (per-shape, one-time cost)

| shape | time | notes |
|-------|------|-------|
| circle (r=50) | 3.70 us | ~100 vertices |
| rounded_rect (200x100, r=12) | 2.51 us | ~80 vertices |
| star_5pt (r=80/35) | 1.56 us | 10 line segments |

## signals

| benchmark | time | notes |
|-----------|------|-------|
| create+get+set x1000 | 66.6 us | 67ns per signal cycle |

## text node hashing

| benchmark | time | notes |
|-----------|------|-------|
| 1000 unique textnodekeys | 23.4 us | fxhasher, includes string clone |

## key takeaways

1. **rect throughput**: 100m-200m rects/s CPU-side. scene construction is negligible vs GPU.
2. **path push cost**: ~10x more than rects (clone of vertex buffers). pre-tessellated shapes are the right design, tessellation is user-side, not per-frame.
3. **dirty tracking**: 3.3us for 1000 rects. static scenes are essentially free after frame 1.
4. **tessellation**: 1.5-3.7us per shape. fast enough for hundreds of shapes per frame if needed.
5. **signal overhead**: 67ns per create+get+set cycle, negligible.

## per-crate benches added since (junho 2026)

os numeros acima sao da v0.2 (scene build, dirty tracking, tessellation, signals, text hashing). as ondas de junho adicionaram criterion benches no hot path de cada crate novo, ainda nao medidos aqui em tabela:

| crate | bench | mede | task |
|-------|-------|------|------|
| monster | `benches/codec.rs` | encode/decode/optimize de uma cena snapshot | task-43 |
| lot | `benches/convert.rs` | conversao lottie json para .monster | task-44 |
| parser | `benches/transpile.rs` | transpile end-to-end do separator gpui | task-45 |
| rope | `benches/edit.rs` | build mais insert/delete roundtrip | task-46 |
| engine | `benches/scene_build.rs` | custo de scene build por frame | task-25 |

os benches densos do monster leem amostras grandes de `ref/lottie` (material de estudo gitignored); quando ausente, pulam com aviso em vez de falhar. rodar com `cargo bench` (tudo) ou `cargo bench -p <crate>`.

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [task-25: benchmarks](../../Tasks/task-25-benchmarks/task-25-benchmarks.md)
- [task-43: monster, binary animation codec v0](../../Tasks/task-43-monster-codec/task-43-monster-codec.md)
- [task-44: lot, lottie importer and bridge to .monster](../../Tasks/task-44-lot-lottie-importer/task-44-lot-lottie-importer.md)
- [task-45: parser, transpiler poc (react/gpui to builder)](../../Tasks/task-45-parser-transpiler/task-45-parser-transpiler.md)
- [task-46: rope, text editing core](../../Tasks/task-46-rope-editing-core/task-46-rope-editing-core.md)
<!-- /related -->
