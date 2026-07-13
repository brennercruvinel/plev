---
type: adr
status: accepted
tags: [gamma, srgb, color, wgpu, shaders, rendering]
date: 2024-11-20
commit: 69013d1
---

# linearize colors before they reach the GPU

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
