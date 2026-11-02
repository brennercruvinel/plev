---
id: task-100
title: "task-100: interactive graphs and corpus import"
project: plev
type: task
status: done
date: 2026-10-27
date_basis: git
recorded: 2026-10-31
topics: [graphs, layout, platform]
relations: [adr-0081, adr-0082, learning-data-appearance-and-algorithm]
evidence: [a38eb003, da4be63a, db4b3299, 08420f58, b526a9bd, 4f079852, b8bc7639, 7c8c4a61]
source: reconstruction
source_revision: ec0a224c
revised: 2026-11-02
review: reviewed
---

# task-100: interactive graphs and corpus import

The graph demo now draws d3 and Cytoscape data with plev's model and components, on
desktop and on the web (`b526a9bd`, already on main). The ELK reader and the `elk` and
`elk-groups` pages (`b8bc7639`) reached main with PR #146 (`7c8c4a61`,
2026-10-31), which closes the task.

## Architecture

`Graph` describes nodes, edges and groups (`rust/engine/src/widgets/graph/model.rs:12`);
`Look` describes the appearance; `Layout` chooses the arrangement, force or compound
(`layout/mod.rs:22`). The format readers live in the demo, outside the engine's model.
`LiveGraph` holds positions, simulation, selection, gesture state and the hit-testing
grid (`live_graph/mod.rs:77`). Dragging a group moves its descendants (test
`a_group_drag_brings_its_children_and_a_child_drag_grows_the_box`); pan and zoom change
the transform without rebuilding the grid (`08420f58`).

`App::with_view` hosts the canvas through `HostedView` (`runtime/window/view.rs:15`). The
window converts physical coordinates to logical ones, delivers events and requests new
frames while the view moves (`runtime/window/lifecycle.rs:175` to `191`). See ADRs
[81](../../adr/Structure/0081-a-graph-separates-data-appearance-and-layout/0081-a-graph-separates-data-appearance-and-layout.md)
and [82](../../adr/Rendering/0082-a-hosted-view-renders-on-demand/0082-a-hosted-view-renders-on-demand.md).

## Corpus and limits

On main, the pages are `lesmis` and `compound` (`demo/graphs/src/lib.rs:38`); `elk` and
`elk-groups` exist only on the branch, and appear in the code at `ec0a224c`
(`demo/graphs/src/lib.rs:46`, the `Page::ALL` list with four pages). An invalid page name
produces an error; it does not silently open the first page (`4f079852`, on main). The ELK
reader converts the structure, but the layered layout requested by the source does not yet
exist in the widget: the pages use the force and compound layouts (`b8bc7639`, on the
branch).

Commit `b8bc7639` reports 14 nodes inside their group boxes after 400 frames, per the
`PLEV_REPORT` report. That is the measurement recorded at delivery, not a guarantee for
any graph. The `LiveGraph` tests cover interactions and invariants (`08420f58`); the demo's
position report makes it possible to check the corpus.

## Code

- [rust/engine/src/widgets/graph/model.rs](https://github.com/brennercruvinel/plev/blob/ec0a224c2d6c5a446e70cf4ca9ee92fd66eef713/rust/engine/src/widgets/graph/model.rs)
- [rust/engine/src/widgets/graph/live_graph/mod.rs](https://github.com/brennercruvinel/plev/blob/ec0a224c2d6c5a446e70cf4ca9ee92fd66eef713/rust/engine/src/widgets/graph/live_graph/mod.rs)
- [rust/engine/src/widgets/graph/live_graph/tests/mod.rs](https://github.com/brennercruvinel/plev/blob/ec0a224c2d6c5a446e70cf4ca9ee92fd66eef713/rust/engine/src/widgets/graph/live_graph/tests/mod.rs)
- [demo/graphs/src/read.rs](https://github.com/brennercruvinel/plev/blob/ec0a224c2d6c5a446e70cf4ca9ee92fd66eef713/demo/graphs/src/read.rs)
- [demo/graphs/src/lib.rs](https://github.com/brennercruvinel/plev/blob/ec0a224c2d6c5a446e70cf4ca9ee92fd66eef713/demo/graphs/src/lib.rs)

## Learning in Rust

Separating data, appearance and algorithm makes it possible to test and replace each part
without bringing the source format into the engine's API. Indices into vectors require
explicit validation: `Graph::holds` checks bounds; that check should not be described as
proof of every topological property.

## Evidence

- [`a38eb003`](https://github.com/brennercruvinel/plev/commit/a38eb00392e9d783c236953237890844ab75c1c2)
- [`da4be63a`](https://github.com/brennercruvinel/plev/commit/da4be63ab386c4a7218add9e320ef6e60d842a1a)
- [`db4b3299`](https://github.com/brennercruvinel/plev/commit/db4b32999ce980e924d15f59b196aa4b01562b05)
- [`08420f58`](https://github.com/brennercruvinel/plev/commit/08420f58d0ffc2d5367929224bf1c48e82a1f8a4)
- [`b526a9bd`](https://github.com/brennercruvinel/plev/commit/b526a9bd3439a58111b33b667d6647ed9a992a5a)
- [`4f079852`](https://github.com/brennercruvinel/plev/commit/4f07985284e00363c5a1ee76c1b4e6e9f5aa0a56)
- [`b8bc7639`](https://github.com/brennercruvinel/plev/commit/b8bc76394e2af1597239965c85ba36588a18fdc5) (branch `feat/elk-ir-from-gramorph`, not on main)
