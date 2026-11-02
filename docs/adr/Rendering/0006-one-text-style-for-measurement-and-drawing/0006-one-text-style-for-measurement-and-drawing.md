---
id: adr-0006
title: "one TextStyle per text run, shared by measurement and drawing"
project: plev
type: adr
status: accepted
date: 2024-10-29
date_basis: reconstructed
recorded: 2026-06-10
topics: [text, layout]
relations: [learning-apps-bypass-the-engine, task-48]
evidence: [f15198a]
source: docs/adr/rendering/0006-one-text-style-for-measurement-and-drawing.md
source_revision: ec0a224c
revised: 2026-11-02
review: reviewed
---

# one TextStyle per text run, shared by measurement and drawing

Decides that a text run has exactly one `TextStyle`, used by both measurement and drawing. The reason is that the defect "label larger than the shape" had a single mechanism: the shape was sized with one text model and the text was drawn with another, and the heuristic was off by -10% to +21%. The result is that the shape's width becomes the measured text plus padding, with `TextMeasurer` as the only source of width, without a GPU and with a cache.

## context

the recurring class of visual defects reported as "label larger than its
shape, text spills out of the form" had one mechanism in every instance:
the size of a shape was computed from one model of the text while the
rasterizer drew the text with another model.

observed instances:

- ide sized every shape with a per-character heuristic
  (`chars * font_size * 0.58`) while drawing with real shaping in rubik
  weight 600. measured error ranged from -10% (text overflows) to +21%
  (pill too wide)
- the builder pipeline measured with `letter_spacing: 0.0` hardcoded while
  exposing a `.tracking()` modifier on the drawing side. the modifier was
  inert at the time, which made the divergence latent rather than visible
- an early engine version measured `chars * 0.6` before cosmic-text
  integration, with identical symptoms

## decision

a text run owns exactly one `TextStyle`. that object is the input to both
`TextMeasurer::measure_styled` (sizing) and `TextNodeKey::from_style`
(drawing). constructing the two sides separately is a defect by definition,
regardless of whether the values currently happen to agree.

`TextMeasurer` is the only sanctioned width source: it shapes with the same
FontSystem and font faces as the rasterizer, is GPU free, and caches by a
key that includes weight and letter spacing.

## consequences

- shape width becomes `measured text + padding` by construction. the
  defect class is closed rather than patched per widget
- one replacement point (`hoff::measure_text`) fixed twelve call sites in
  ide simultaneously
- regression tests assert that the measured width differs from the old
  heuristic and that drawn shapes fit real shaped labels

## avoid

- never estimate text width arithmetically (character counts, average
  advance factors). every such formula ignores weight, glyph width
  variance, digits and spacing
- never construct a TextNodeKey field by field next to a separately
  constructed measure spec. use `from_style` on the shared object
- when adding a typography attribute (e.g. a future font family modifier),
  plumb it through measurement and drawing in the same change. the
  letter-spacing hole existed because the attribute landed on one side only

## learnings

- [why the apps bypassed the engine, and what that costs](../0002-content-driven-layout-not-fixed-constants/apps-bypass-the-engine.md)

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0002: container geometry derives from available space, never from constants](../0002-content-driven-layout-not-fixed-constants/0002-content-driven-layout-not-fixed-constants.md)
- [ADR 0086: the adaptive foundation reads preferences once into the theme](../../Interface/0086-the-adaptive-foundation-reads-preferences-once-into-the-theme/0086-the-adaptive-foundation-reads-preferences-once-into-the-theme.md)
- [why the apps bypassed the engine, and what that costs](../0002-content-driven-layout-not-fixed-constants/apps-bypass-the-engine.md)
- [task-48: ide, native git client in plev](../../../Tasks/task-48-ide-git-client/task-48-ide-git-client.md)
- [task-66: headless as a first-class citizen](../../../Tasks/task-66-headless-first-class/task-66-headless-first-class.md)
- [Reorganization of the repository tree (issue #130, PR #131)](../../../Tasks/task-98-workspace-reorganization/change-record.md)
- [how to code against the plev engine](../../../Research/Workflows/code-against-the-plev-engine.md)
<!-- /related -->
