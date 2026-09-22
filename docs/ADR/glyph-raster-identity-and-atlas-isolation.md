---
type: adr
status: accepted
tags: [text, fonts, cosmic-text, glyph-atlas, rasterization, hidpi]
date: 2026-08-31
---

# glyph raster identity and atlas isolation

## context

the glyph raster path (cosmic-text shaping -> swash rasterization -> R8
atlas -> textured quads) held several defects that only surface under
accumulated state: a grown atlas, evictions, layers retained across frames,
and several font sizes of one face coexisting on screen. symptoms ranged
from sub-pixel drift to glyphs drawn as pieces of other glyphs and other
sizes, strongest in static panels. single-line, single-frame probes cannot
reach any of them; diagnosis required per-glyph audits (shaped advances,
cache keys, entry dimensions) and multi-frame headless rendering.

## decisions

- **the atlas cache key is `cosmic_text::CacheKey` verbatim** (type alias).
  the key must carry every input swash rasterizes with — face, glyph, size,
  weight, subpixel bins, flags — or two bitmaps alias onto one entry. a
  narrowing wrapper is a compile error at the `get_image_uncached` boundary.
- **a fresh swash context per rasterization.** the shared `ScaleContext`
  keeps per-font state across calls, and interleaving sizes of one face can
  make the same key rasterize at a stale size. rasterization happens only on
  an atlas cache miss, so a fresh context costs nothing per frame.
- **every glyph slot has a one-texel transparent gutter on all four sides**,
  uploaded as one zeroed block: the atlas samples with linear filtering, and
  a bilinear tap may reach one texel past the UV rect; a reused slot must
  not expose its previous occupant.
- **quads carry atlas coordinates in texels**; `text.wgsl` divides by
  `textureDimensions`, so a mid-frame atlas grow (which copies the old
  contents to the same origin) never invalidates emitted quads.
- **the atlas grows to `MAX_ATLAS_SIZE` before evicting.** eviction is only
  safe for glyphs nothing still draws, and `glyphs_in_use` covers only the
  layers resolved this frame — skipped layers retain vertex buffers that
  still reference their slots.
- **every operation that frees or reuses a slot records `atlas_disturbed`**
  (eviction, LRU capacity drop, raster-scale change, cache purge);
  `resolve_layer_text` then re-resolves every layer, dirty or not.
  `set_raster_scale` is `#[must_use]` for the same reason.
- **a dropped cache entry's rectangle returns to the allocator** (`push`
  returns the LRU victim), and **empty glyphs carry `alloc_id: None`** —
  they reserve no rectangle, so eviction deallocates nothing for them.

## consequences

- one atlas entry per (face, glyph, size, weight, subpixel phase, flags);
  the atlas is the memory budget and grows to fit.
- text renders pixel-exact and identically regardless of atlas state:
  pinned by `tests_raster.rs` (GPU-free invariants),
  `tests/text_raster_pixels.rs` (renders identical with/without a grow,
  across scale round-trips, and for a static layer under multi-frame churn)
  and `tests/text_drawn_extent.rs` (painted extent matches the measured
  advance at scales 1.0–3.0, fractional included). tests skip without a GPU
  adapter.
- `engine::text::probe` renders text headlessly (offscreen texture, real
  `TextSystem`, real shader) for pixel validation where screen capture is
  unavailable; `probe::render_frames` models retained layers across frames.

## avoid

- never derive a shrunken copy of a dependency's cache key; if the upstream
  key has a field, the upstream rasterizer varies output on it.
- never share a rasterizer context across calls that vary parameters the
  context may cache.
- never bake a texture's dimensions into vertex data that outlives the
  frame; resolve against the bound texture in the shader.
- never free or reuse shared-cache backing without invalidating everything
  that points into it — and never let a bounded cache pick eviction victims
  on its own; only the eviction path checks who is still using a slot.
- a sentinel value for "no allocation" must not be a valid allocation id;
  use `Option`.
- padding a slot is not clearing it: upload the border explicitly.
- do not validate this path with single-frame or single-size probes; use
  the multi-frame probe and compare pixels against measured advances.
