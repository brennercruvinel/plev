---
id: adr-0008
title: "render on demand makes invalidation a correctness contract"
project: plev
type: adr
status: accepted
date: 2024-10-07
date_basis: reconstructed
recorded: 2026-06-10
topics: [rendering, input, signals]
relations: []
evidence: [83720e3]
source: docs/adr/rendering/0008-render-on-demand-requires-explicit-invalidation.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# render on demand makes invalidation a correctness contract

Fixes the contract that applies when the engine renders only on request: a handler that changes something visible must return `true` or invalidate and request the redraw. The case that motivated the decision was a consumed scroll event, with changed state, a handler returning `false`, and nothing invalidated, which left the whole showcase unresponsive. With this, invalidation is treated as correctness, and the regression tests verify invalidation, not just the mutated state.

## context

the engine renders only when something requests a frame (battery and CPU
discipline borrowed from production editors). under this model an event
handler that mutates visual state but fails to signal it does not produce
a stale frame, it produces a frozen application. this was shipped: scroll
events were consumed, state changed, handlers returned false, nothing
invalidated, and the showcase appeared completely unresponsive. the defect
was initially misread as "scroll not implemented".

## consequences of the model (the contract)

- every event handler that changes anything visible must return true or
  call the invalidation path (compositor invalidate plus request_redraw).
  returning false is a statement that nothing visible changed, and the
  scheduler believes it
- animations keep frames flowing only while active
  (`is_animating || compositor.needs_render()`)
- a new interaction feature is not done when state updates; it is done
  when the state change provably schedules a frame. regression tests
  assert the boolean/invalidations, not just the mutated state

## avoid

- never debug "frozen UI" by adding redraws in a loop. find the handler
  that lied about not changing state
- never copy an event handler skeleton without carrying its invalidation
  discipline along

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0023: official app pattern: state in structs, retained widgets, explicit invalidation](../../Structure/0023-official-app-pattern/0023-official-app-pattern.md)
- [task-105: a dev mode over any app: the frame, the GPU, the zones, the heap and the page's controls](../../../Tasks/task-105-profiler-panel-in-the-gallery/task-105-profiler-panel-in-the-gallery.md)
<!-- /related -->
