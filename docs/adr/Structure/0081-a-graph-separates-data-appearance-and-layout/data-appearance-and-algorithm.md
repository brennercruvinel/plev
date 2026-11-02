---
id: learning-data-appearance-and-algorithm
title: "Separate data, appearance and algorithm in Rust"
project: plev
type: learning
status: reference
date: 2026-10-27
date_basis: git
recorded: 2026-10-31
topics: [graphs, layout, importing]
relations: [task-100, adr-0081]
evidence: [db4b3299, b8bc7639]
source: reconstruction
source_revision: ec0a224c
revised: 2026-11-02
review: reviewed
---

# Separate data, appearance and algorithm in Rust

In the plev graph, `Graph` holds what exists and `Look` describes how each
element appears. `Layout` chooses the arrangement. This split lets an ELK
reader reuse the same component that receives d3 data.

`Node` keeps the identity of the source as a `String`, while edges refer to
positions in `Vec<Node>` by `usize`. This avoids references with lifetimes
running through the structure, but transfers to the program the responsibility
of validating the indices and preserving their correspondence.

`Graph::holds` checks whether the indices are within the vectors. By itself it
does not prove uniqueness of parents or the absence of cycles. `descendants`
uses a visited vector so that it does not traverse cyclic groups indefinitely.
These are different guarantees, which deserve different names and tests.

The import contract ends at the model; the layout contract starts there.
Importing ELK JSON does not implement an ELK layout. This boundary is what makes
it possible to record a limitation without coupling the engine to the format.

Source: [rust/engine/src/widgets/graph/model.rs](https://github.com/brennercruvinel/plev/blob/ec0a224c2d6c5a446e70cf4ca9ee92fd66eef713/rust/engine/src/widgets/graph/model.rs) and [task 100](../../../Tasks/task-100-interactive-graphs/task-100-interactive-graphs.md).

## Evidence

- [`db4b3299`](https://github.com/brennercruvinel/plev/commit/db4b32999ce980e924d15f59b196aa4b01562b05)
- [`b8bc7639`](https://github.com/brennercruvinel/plev/commit/b8bc76394e2af1597239965c85ba36588a18fdc5)

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0081: a graph separates data, appearance and layout](0081-a-graph-separates-data-appearance-and-layout.md)
- [task-100: interactive graphs and corpus import](../../../Tasks/task-100-interactive-graphs/task-100-interactive-graphs.md)
<!-- /related -->
