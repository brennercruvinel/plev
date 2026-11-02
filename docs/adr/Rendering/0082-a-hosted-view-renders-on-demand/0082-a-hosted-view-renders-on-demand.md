---
id: adr-0082
title: "a hosted view uses the engine's window and clock"
project: plev
type: adr
status: accepted
date: 2026-10-27
date_basis: git
recorded: 2026-10-31
topics: [graphs, platform]
relations: [task-100]
evidence: [da4be63a, 4aa9706b]
source: reconstruction
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# a hosted view uses the engine's window and clock

## Context

A single-canvas demo needed events, ticks and composition,
but not its own implementation of the whole window.

## Decision

`App::with_view` takes a `Box<dyn HostedView>` (`rust/engine/src/runtime/window/mod.rs:117`).
The trait separates `handle_event`, `tick` and `render` (`runtime/window/view.rs:15` to `:24`).
The engine delivers events in logical pixels, hosts the view in the frame and requests
frames while there is movement (`runtime/window/lifecycle.rs:183` to `:190`). The view's
clock handles rest: when the view stops moving, the clock stops, and resuming does not
advance the simulation by the full time the window spent without drawing
(`lifecycle.rs:184` to `:186`; `4aa9706b`).

## Consequences

The demo shares the engine's composition and initialization path. The documentation of
the trait says that uigallery and idestudio keep their own shells
(`runtime/window/view.rs:6` and `:7`). The canvas trait does not automatically represent
the full semantic contract of an app.

## Alternatives

A new shell per demo would duplicate lifecycle and event conversion. Forcing the existing
apps to adopt the trait would widen the scope of the delivery.

## Evidence

- [`da4be63a`](https://github.com/brennercruvinel/plev/commit/da4be63ab386c4a7218add9e320ef6e60d842a1a)
- [`4aa9706b`](https://github.com/brennercruvinel/plev/commit/4aa9706bdfdd63f2eff7d9b9b59cf6dda4ab225b)

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [task-100: interactive graphs and corpus import](../../../Tasks/task-100-interactive-graphs/task-100-interactive-graphs.md)
<!-- /related -->
