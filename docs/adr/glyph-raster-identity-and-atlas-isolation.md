---
type: adr
status: accepted
tags: [text, fonts, cosmic-text, glyph-atlas, rasterization, hidpi]
date: 2026-08-31
---

# the glyph cache key is cosmic-text's key, and every glyph gets a gutter

## context

after the inclusive sans swap, headings and small readouts rendered with
letters piled on top of each other: "Expense Tracker" came out as
"Ex,e,eTra de", "Builder" as "BUide", "2 active · 1 done" with the middle
word collapsed. body text at the same sizes was fine.

the first two hypotheses were wrong, and both are the ones
[[embed-every-font-weight-in-use]] warns about — a tracking bug, then a
family-fallback bug. measurement was the third wrong guess.

what actually located it was refusing to guess further:

- a headless pass over all fourteen showcase sections collected every text
  node per section and grouped them by string. no string was drawn twice at
  the same position, so nothing was double-drawing.
- a probe dumped the shaped glyph stream for the offending strings: per
  glyph, the advance, the resolved `font_id`, and the full cosmic-text
  `CacheKey`. advances were monotonic and correctly spaced, and every glyph
  resolved to the expected face. **shaping and measurement were correct.**

that isolated the defect to the raster path, and the probe's `CacheKey`
column showed it directly.

## the three defects

**1. the atlas cache key dropped three of seven identity fields.**
`GlyphCacheKey` kept `font_id`, `glyph_id`, `font_size_bits` and `flags`,
dropping `x_bin`, `y_bin` and `font_weight`. the subpixel bins are the
damaging omission. `CacheKey::new` splits a glyph's fractional x into
quarter-pixel bins, and swash rasterizes each bin as a *different* bitmap
with its own `placement.left`/`top`. one string routinely spans several
bins for the same character — the three `e`s of "Expense Tracker" at
20px/500 land in bins Zero, Zero and One — so the first `e` rasterized was
reused, bitmap *and* placement, for every later phase. every repeat drew up
to 0.75 physical px off its shaped position with a mask built for the wrong
phase.

**2. the atlas gutter existed on two sides, not four.** slots were padded
`gw + 1` by `gh + 1`, i.e. one texel on the right and bottom only, and UVs
were the exact texel rect. the atlas samples with `FilterMode::Linear`, so
a tap that reaches past the rect on the left or top edge lands in the
neighbouring glyph. worse, a slot reused after eviction still held the
previous occupant's pixels, so even the right/bottom texel was not empty.

**3. UVs were normalized at emit time, against an atlas that can grow
mid-frame.** the atlas doubles when it fills, and `grow_atlas` copies the
old texture into the new one at the same origin. quads emitted before a
grow carried `x / old_size`; after it they sampled half-scale — the wrong
region entirely, which reads as glyphs made of pieces of other glyphs.

defects 1 and 3 compound: 1 guarantees the quad sits at a fractional offset
from the bitmap it samples, and 2 then blends in whatever is adjacent.

## decision

- `GlyphCacheKey` **is** `cosmic_text::CacheKey`, as a type alias. the key
  must name every input swash rasterizes with; a narrowing wrapper is how
  this class of bug gets reintroduced, and the alias makes it a compile
  error at the `get_image_uncached` boundary rather than a silent aliasing
- every glyph slot reserves `GLYPH_PADDING = 1` texel on **all four** sides,
  and the bitmap is uploaded as one zeroed padded block so the gutter is
  transparent by construction even in a reused slot
- quads carry atlas coordinates in **texels**. `text.wgsl` divides by
  `textureDimensions(atlas_texture)`, so the division happens against the
  atlas actually bound at draw time and a grow cannot invalidate a quad.
  this needs no uniform and no bind-group change

## consequences

- one atlas entry per (face, glyph, size, weight, subpixel phase, flags).
  more entries than before; that is the correct number, and the atlas grows
  to fit
- text is pixel-exact: the quad origin is integral in physical pixels
  (`physical.x` and `placement.left` are both physical integers), and it now
  samples the bitmap rasterized for its own subpixel phase
- `tests_raster.rs` pins all three. each was verified as a kill test:
  narrowing the key fails to compile, `GLYPH_PADDING = 0` fails the gutter
  test, and normalizing in `glyph_uv_rect` fails the texel-UV test

## avoid

- never derive a shrunken copy of a dependency's cache key. if the upstream
  key has a field, the upstream rasterizer varies output on it
- never bake a texture's dimensions into vertex data that outlives the frame
  it was emitted in. resolve against the bound texture in the shader
- never diagnose overlapping or doubled glyphs as a shaping, tracking or
  fallback bug before dumping the shaped glyph stream. shaping was correct
  here, and three earlier guesses at the typography layer were all wrong.
  the probe cost minutes and the guesses cost more
- padding a glyph slot is not the same as clearing it. an evicted slot keeps
  its pixels until something writes over them
