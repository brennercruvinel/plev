---
id: adr-0041
title: "the svg export reads the scene, not the pixels"
project: plev
type: adr
status: accepted
date: 2026-09-23
date_basis: header
recorded: 2026-09-23
topics: [rendering, formats, text]
relations: [adr-0040, task-63, task-66, task-67]
source: docs/adr/rendering/0041-svg-export-reads-the-scene.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# the svg export reads the scene, not the pixels

Decides that SVG export reads the scene, not the pixels: `engine::headless::svg::export` walks the visible layers of the compositor and writes one element per node, with text as outlines of the already shaped glyphs. What has no exact equivalent is approximated or left out, and each case goes into a per-node report, so nothing falls silently. Measured on pages such as Todo, Button, Effects and Card, the rasterized SVG differs from the engine's png between 0,13% and 1,07% of the pixels, and the sum of the glyph advances matches the width measured by the text system within 0,01 px; it matters because the structural regression (task-67) comes to know what it can and cannot trust in the SVG.

## context

task-66 made a frame renderable without a window through the window's own
frame path, and left two things out: a vector export of that frame, and
`showcase --render --semantics`, refused with "not available yet" while the
projection did not exist. both exist now: every page of the showcase pushes
`SceneNode`s the compositor keeps per layer, the todo page projects a
`sem::Snapshot`, and the wire serves that snapshot in the lean serde form
([`0040-lean-semantic-form-and-one-redaction.md`](../../Semantics/0040-lean-semantic-form-and-one-redaction/0040-lean-semantic-form-and-one-redaction.md)).

a vector export has one honest source: the scene. tracing pixels would be
a second renderer; re-shaping text would be a second shaping path, which
the manual forbids (one TextStyle per run, the shaper as the only width
source).

## decision

`engine::headless::svg::export` walks the compositor's visible layers in
composite order and each layer's nodes in push order, and writes one SVG
element per node. what has an exact equivalent is written as it is: a
rect, a rounded rect with its border as an inset stroke, a tessellated
path as the outline of its triangles, a clip as a `clipPath`. text is
written as the outlines of the glyphs the frame shaped: `TextSystem`
gained `shaped_glyphs`, a reader over the same `ensure_shaped` that
`resolve_for_layer` draws from, so a run comes back exactly as drawn (or
as it will be drawn), and the outline of each glyph comes from swash
through cosmic-text, one scale context per glyph because a shared one
keeps per-face state and hands back stale outlines. each run is a
`<g aria-label="<string>">`, readable to a tool that reads and right to
one that draws.

what has no exact equivalent is approximated or left out, and every such
case enters a report, one `Approximation` per node: a gradient as a
`linearGradient` between the two points the shader projects, interpolated
in linear space; a shadow as a blurred silhouette; a backdrop blur and an
image left out with a comment in their place; a clip at logical bounds
where the frame scissors at whole physical pixels. an svg with an empty
report is the frame. the same rule the svg importer applies in the other
direction: nothing is dropped silently.

`showcase --render <page>` stages one frame and writes what is asked of
it: `--out` the png, `--svg` the svg (its report on stderr and, with
`--svg-report`, to a file), `--semantics` the page's projection as the
lean json `plev observe` serves, `--source` keeping each node's source.
the projection needs no gpu; a page without one is an error naming the
page.

## consequences

- three pages rasterized from their svg and compared to the engine's png
  differ in 0.13 percent (Todo), 0.2 (Button), 0.56 (Effects) and 1.07
  (Card, its two images left out) of the pixels at an 8 percent fuzz.
- the sum of the glyph advances the export writes equals the width the
  text system measures for the same run, within 0.01 px, for three faces
  and sizes: one shaping path, proven.
- the report per page names what a structural regression test (task-67)
  can rely on and what it cannot: Todo 8 items, Button 23, Card 24,
  Effects 27, almost all gradients, shadows and clips.
- images are not embedded (the image store keeps no pixels after the
  upload; task-63 phase d adds opt-in retention, which this export can
  then use); layer effects are reported as left out, as the shared frame
  path leaves them out too; `color-interpolation="linearRGB"` is not
  honored by every browser.
- `Approximation.node` is formatted text; a typed location would serve
  task-67 better and is the first thing to change when it starts.
