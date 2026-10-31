---
id: adr-0002
title: "container geometry derives from available space, never from constants"
project: plev
type: decisao
status: aceito
date: 2024-09-18
date_basis: cabecalho
topics: [layout, renderizacao]
relations: []
evidence: [15f096f]
source: docs/adr/rendering/0002-content-driven-layout-not-fixed-constants.md
source_revision: ec0a224c
revised: 2026-10-30
review: revisado
---

# container geometry derives from available space, never from constants

Decide que toda dimensão de contêiner deriva do espaço disponível medido ou do motor de flex, e que constantes só valem como mínimo, máximo ou espaçamento. O motivo é que a engine já tinha layout flex completo, mas as apps posicionavam retângulos com constantes fixas, e a janela não reflowava ao encolher nem aproveitava espaço ao crescer. O resultado é uma geometria testada sem GPU em larguras explícitas, como 600 px empilhando e 1272 px espalhando, e um clamp que guarda a intenção, não o valor clampado.

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
