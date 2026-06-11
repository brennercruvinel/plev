---
type: adr
status: accepted
tags: [rendering, events, invalidation, render-on-demand, reactivity]
date: 2026-06-10
commit: 83720e3
---

# render on demand makes invalidation a correctness contract

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
