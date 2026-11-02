---
id: adr-0004
title: "linearize colors before they reach the GPU"
project: plev
type: adr
status: accepted
date: 2024-11-20
date_basis: reconstructed
recorded: 2026-06-10
topics: [rendering, gpu]
relations: [adr-0007, learning-the-two-gamma-bugs]
evidence: [69013d1]
source: docs/adr/rendering/0004-linearize-colors-before-the-gpu.md
source_revision: ec0a224c
revised: 2026-11-02
review: reviewed
---

# linearize colors before they reach the GPU

Decides to convert sRGB to linear once, at the point where the color enters GPU memory. The reason is that the engine handed over sRGB values as if they were linear, and the GPU re-encoded them: the background of token 48 measured 118 on screen. The result is a background measuring 50 after the fix, with `to_linear_array()` as the single path for clear colors and the test `to_linear_array_darkens_srgb_midtones` pinning the curve.

## context

theme tokens, CSS values and hex literals are sRGB encoded. the desktop
window surface uses an sRGB texture format, which means the GPU applies a
linear to sRGB encoding on every write. for months the engine handed sRGB
values to the GPU as if they were linear. the GPU re-encoded them, raising
every midtone by roughly 2.5x. the page background token #303030 (value 48)
measured 118 on screen. the symptom was reported repeatedly as "washed out
gray, no contrast" and was misattributed to token choices several times.
adjusting tokens darker was attempted and rejected as a band-aid: it made
individual screens acceptable while leaving every future color wrong.

## decision

a single conversion boundary: colors are linearized exactly once, at the
point they enter GPU memory.

- vertex colors: `srgb_to_linear` in the WGSL shaders (quad, rect_sdf,
  text, shadow, shadow_analytic), applied before premultiplied alpha
- clear colors and CPU-built uniforms: `Color::to_linear_array()`
  (src/color.rs), alpha untouched
- textures that arrive already decoded (image atlas, backdrop, composite,
  blur) are not converted again

## consequences

- background measured 118 before, 50 after, against a token of 48.
  validation was a pixel measurement, not visual inspection
- every app and example must use `to_linear_array()` for clear colors.
  using `to_array()` for a clear value reintroduces the bug silently
- the regression test `to_linear_array_darkens_srgb_midtones` pins the
  transfer function (#303030 to 0.0296)

## avoid

- do not "fix" washed-out colors by darkening tokens. measure the pixel
  first; if a token of 48 renders as 118, the bug is in the transfer, not
  in the token
- do not convert twice. sampled textures created from decoded images are
  already linear when bound through sRGB texture formats

## learnings

- [the two gamma bugs: one assumption, two opposite failures](the-two-gamma-bugs.md)

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0007: render into an sRGB view when the surface format cannot be sRGB](../0007-render-into-an-srgb-view-format/0007-render-into-an-srgb-view-format.md)
- [the two gamma bugs: one assumption, two opposite failures](the-two-gamma-bugs.md)
<!-- /related -->
