---
id: adr-0002
title: "container geometry derives from available space, never from constants"
project: plev
type: adr
status: accepted
date: 2024-09-18
date_basis: reconstructed
recorded: 2026-06-10
topics: [layout, rendering]
relations: [adr-0006, adr-0009, learning-apps-bypass-the-engine, task-48]
evidence: [15f096f]
source: docs/adr/rendering/0002-content-driven-layout-not-fixed-constants.md
source_revision: ec0a224c
revised: 2026-11-02
review: reviewed
---

# container geometry derives from available space, never from constants

Decides that every container dimension derives from the measured available space or from the flex engine, and that constants are valid only as a minimum, a maximum or a spacing. The reason is that the engine already had full flex layout, but the apps positioned rectangles with fixed constants, and the window did not reflow when shrinking nor use space when growing. The result is geometry tested without a GPU at explicit widths, such as 600 px stacking and 1272 px spreading, and a clamp that keeps the intent, not the clamped value.

## context

the engine ships a complete flex system (taffy behind
`LayoutEngine::compute`, which receives the viewport on every call, with
text measure functions attached). a grep proved that neither shipped app
consumed it: showcase and ide positioned rectangles manually with
fixed constants (column width 320, tab strip 384, card 368, side panels
280/340). the resize pipeline itself was mechanically correct (surface
reconfigured, projection reapplied, scene rebuilt), but the rebuilt
geometry used the same constants, so windows neither reflowed when
narrowed nor used space when widened. two pages (theme and icons
galleries) computed columns from the available width and were the only
responsive screens, demonstrating that the correct pattern already existed
in the repository.

a second defect compounded it: ide clamped panel widths destructively
on shrink, overwriting the stored value, so growing the window never
restored the user's layout.

## decision

- every container dimension derives from the measured available width
  (`content.w`) or from the flex engine. named constants are legal only as
  minimums, maximums and gaps
- grids compute column count as
  `floor((content.w + gap) / (min_w + gap)).max(1)` and stretch columns to
  fill, clamped by a maximum for readability
- rows of intrinsic-width items wrap when the next item would exceed
  `content.w`
- user-adjustable dimensions store intent separately from effect: a
  desired width survives any number of clamp cycles, and the effective
  width is re-derived from it on every layout
- the layout wrapper now expresses percentages
  (`width_percent`/`height_percent` mapping to `Dimension::percent`)

## consequences

- layout functions remained pure, so responsiveness is regression-tested
  without a GPU at explicit viewports (600px stacks, 1272px spreads)
- the browser build demonstrates live reflow, since the canvas tracks the
  window through the winit ResizeObserver

## avoid

- never position a sibling at `x + constant`. the constant encodes an
  assumption about the viewport that resize will violate
- never write a clamp that mutates the stored value it clamps. clamp at
  read time, store intent
- before building layout behavior in an app, check whether the engine
  already provides it. the cost of this failure was weeks: the engine had
  flex, percent and measure functions while the apps reimplemented
  positioning by hand

## learnings

- [why the apps bypassed the engine, and what that costs](apps-bypass-the-engine.md)

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0006: one TextStyle per text run, shared by measurement and drawing](../0006-one-text-style-for-measurement-and-drawing/0006-one-text-style-for-measurement-and-drawing.md)
- [ADR 0009: touch input synthesizes pointer events into the existing mouse path](../../Interface/0009-touch-as-synthesized-pointer-events/0009-touch-as-synthesized-pointer-events.md)
- [ADR 0073: design for ranges, not screens and devices](../0073-design-for-ranges-not-screens-and-devices/0073-design-for-ranges-not-screens-and-devices.md)
- [ADR 0086: the adaptive foundation reads preferences once into the theme](../../Interface/0086-the-adaptive-foundation-reads-preferences-once-into-the-theme/0086-the-adaptive-foundation-reads-preferences-once-into-the-theme.md)
- [why the apps bypassed the engine, and what that costs](apps-bypass-the-engine.md)
- [task-48: ide, native git client in plev](../../../Tasks/task-48-ide-git-client/task-48-ide-git-client.md)
<!-- /related -->
