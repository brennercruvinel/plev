---
id: adr-0081
title: "a graph separates data, appearance and layout"
project: plev
type: adr
status: accepted
date: 2026-10-27
date_basis: git
recorded: 2026-10-31
topics: [graphs, layout]
relations: [task-100, learning-data-appearance-and-algorithm]
evidence: [db4b3299, b819b3db, b8bc7639]
source: reconstruction
source_revision: ec0a224c
revised: 2026-11-02
review: reviewed
---

# a graph separates data, appearance and layout

## Context

More than one source format and more than one component draw graphs: `graph_view`
and `LiveGraph` have received the same `Graph` since `b819b3db`. Coupling format,
style and positions would require one renderer per importer.

## Decision

`Graph` holds nodes, edges and groups (`rust/engine/src/widgets/graph/model.rs:12`).
`Look` computes the appearance of each element, with colors resolved by the theme at
drawing time (`look.rs:4` and `:5`). `Layout` chooses force or compound
(`layout/mod.rs:22` and `:26`). Readers for d3 and Cytoscape convert data to `Graph`
outside the engine, in `demo/graphs/src/read.rs`, which is already on main (`b526a9bd`
brought them in as `demo/corpus`). The ELK reader, also described here, exists only in
`b8bc7639`, on the branch `feat/elk-ir-from-gramorph`.

## Consequences

A new input source can reuse simulation, drawing and hit testing. An ELK file does not
imply support for the layout algorithm that ELK asks for: that capability remains
limited to the implemented algorithms, force and compound (`b8bc7639`).

The decision also describes the ELK reader; it stayed `proposed` until the reader
reached main with PR #146 (`7c8c4a61`, 2026-10-31).

## Alternatives

Embedding each source runtime would duplicate the interaction and rendering paths.
Converting the structure keeps plev's drawing contract.

## Evidence

- [`db4b3299`](https://github.com/brennercruvinel/plev/commit/db4b32999ce980e924d15f59b196aa4b01562b05)
- [`b819b3db`](https://github.com/brennercruvinel/plev/commit/b819b3db7f02164cedbfdb04b5c608ed7ab6ec0d)
- [`b8bc7639`](https://github.com/brennercruvinel/plev/commit/b8bc76394e2af1597239965c85ba36588a18fdc5) (branch `feat/elk-ir-from-gramorph`, not on main)

## learnings

- [Separate data, appearance and algorithm in Rust](data-appearance-and-algorithm.md)
