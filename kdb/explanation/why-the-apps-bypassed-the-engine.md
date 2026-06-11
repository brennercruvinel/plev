---
type: explanation
tags: [architecture, layering, engine, apps, anti-patterns, process]
date: 2026-06-10
commit: ac40423
---

# why the apps bypassed the engine, and what that costs

the most expensive defect cluster in this project (text overflowing
shapes, no responsive reflow, dead touch input) shared one architectural
shape: the engine provided a correct capability, and the application layer
reimplemented it locally, badly. this document records the mechanism so
the pattern is recognized early next time.

## the evidence

- the engine had taffy flex layout with text measure functions wired
  (src/layout/engine.rs); both apps positioned rectangles with hand
  constants. grep found zero app usages of the layout engine
- the engine had real text measurement (TextMeasurer, cached, GPU free);
  basicIDE shipped a per-character width heuristic with errors up to 21%
- the engine recognized taps, drags and pinches; the events were drained
  into a debug log and widgets never saw them
- the correct pattern existed in-repo each time: two showcase pages
  computed columns from available width; showcase widgets measured labels
  through preferred_size. the failure was never absence of capability,
  it was absence of adoption

## why it happens

- incremental app code grows from the first hardcoded prototype, and each
  screen copies the previous screen's skeleton. constants metastasize
  through copy-paste, not through decisions
- the engine capability is invisible at the call site. nothing fails when
  an app positions by hand; the cost appears only at resize, at a longer
  label, on a different monitor
- agents (and developers) given a screen-level task default to local code.
  without an explicit instruction to integrate with the engine layer, the
  path of least resistance is reimplementation

## the working countermeasures

- the operating manual (kdb/how-to/code-against-the-plev-engine.md) opens
  with "check whether the engine already provides it", and agent prompts
  name the engine integration explicitly as the design rule to enforce
- pure layout functions with viewport regression tests make bypasses
  visible: a hand-positioned screen cannot pass a "narrow viewport stacks,
  wide viewport spreads" test
- audits that grep for consumers of an engine capability (who calls
  LayoutEngine? who calls TextMeasurer?) detect silent bypass faster than
  visual inspection ever did

## the general lesson

a capability that the platform layer offers but the product layer does
not consume is not infrastructure, it is unverified inventory. either
wire it into the product path with tests that depend on it, or expect the
product layer to grow a divergent local copy whose defects will be
attributed to the platform.
