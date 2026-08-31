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

## why it looked resolution-dependent

reported as "the big screen breaks, the small mac screen works". that is the
shape of the atlas defect, not a separate one: a larger window shows more
text, more distinct glyphs fill the atlas, and the atlas grows. on the
smaller panel the same scene may never reach the first grow, so it renders
correctly. dragging the window between displays adds the second trigger
below.

a one-line probe cannot see either. the reproduction is `PROBE_FILL=1`,
which pushes enough glyph/size/weight combinations through the atlas to
force a grow.

## the defects

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

**4. a raster-scale change stranded the layers it did not re-resolve.**
`set_raster_scale` resets the glyph cache and rebuilds the atlas allocator,
handing the entire atlas back as free space. But `resolve_layer_text` skips
layers whose scene did not change, so those layers kept vertices pointing at
texels that the next glyphs were then packed over. This is the second half
of the resolution-dependence: it fires when a window moves between displays
of different DPI (a 2x laptop panel to a 1x external one), not when it is
merely large.

**5. eviction leaked an atlas allocation each time it succeeded.** The
eviction loop called `allocate` to test whether it had freed enough room and
dropped the result, then let the outer loop allocate again. The probe
rectangle stayed reserved in the allocator but was never recorded in
`glyph_cache`, so nothing could evict it. Under sustained eviction the atlas
fills with orphans, grows to `MAX_ATLAS_SIZE`, and then drops glyphs
outright — text silently missing rather than scrambled.

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
- `set_raster_scale` returns whether the scale changed and is
  `#[must_use]`; `resolve_layer_text` re-resolves **every** layer when it
  did, via the named `must_resolve_text` predicate
- eviction keeps the allocation it made room for instead of allocating
  twice and discarding the first

## consequences

- one atlas entry per (face, glyph, size, weight, subpixel phase, flags).
  more entries than before; that is the correct number, and the atlas grows
  to fit
- text is pixel-exact: the quad origin is integral in physical pixels
  (`physical.x` and `placement.left` are both physical integers), and it now
  samples the bitmap rasterized for its own subpixel phase
- `tests_raster.rs` pins the GPU-free invariants and
  `tests/text_raster_pixels.rs` + `tests/text_drawn_extent.rs` pin the
  rendered pixels, including "a string renders identically whether or not
  the atlas grew" and "the painted run is as wide as the measured run" at
  scales 1.0 through 3.0 and at fractional factors. They skip, rather than
  fail, without a GPU adapter
- verified as kill tests: narrowing the key fails to compile,
  `GLYPH_PADDING = 0` fails the gutter test, and normalizing in
  `glyph_uv_rect` fails the atlas-grow pixel test with the production
  symptom

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
- never invalidate a shared cache without invalidating everything that
  points into it. resetting the atlas allocator is not a local operation
- never call an allocator to ask "is there room" and drop the answer
- do not trust a visual read of a render, in either direction. a one-line
  probe looked clean and the engine was not; a stress render looked
  "letter-spaced" and measurement showed the advances were exactly linear.
  the numbers settled both — `tests/text_drawn_extent.rs` compares painted
  extent against the measurer instead of an eye
