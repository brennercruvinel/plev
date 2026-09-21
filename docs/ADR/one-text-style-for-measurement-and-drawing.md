---
type: adr
status: accepted
tags: [text, layout, cosmic-text, measurement, typography]
date: 2024-10-29
commit: f15198a
---

# one TextStyle per text run, shared by measurement and drawing

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
