---
type: explanation
tags: [gamma, srgb, color-theory, wgpu, webgpu, rendering]
date: 2026-06-10
commit: 2a33933
---

# the two gamma bugs: one assumption, two opposite failures

this project shipped the same root defect twice, with opposite symptoms.
understanding the pair as one system prevents a third occurrence.

## the system

sRGB is an encoding, not a color. authoring values (hex, CSS, design
tokens) are sRGB encoded. GPUs blend and interpolate correctly only in
linear space. the contract is: convert sRGB to linear before GPU work,
and encode linear back to sRGB exactly once, at the final write to the
display surface. who performs that final encode depends on the surface
texture format:

- sRGB surface format: the hardware encodes on write. the program must
  hand the GPU linear values
- non-sRGB surface format: nobody encodes. the program receives exactly
  what it wrote

## failure one: desktop, too light (washed out)

the desktop surface is sRGB. the engine handed it sRGB values as if
linear. the hardware encoded them a second time, lifting midtones by
about 2.5x: token 48 measured 118 on screen. perceived as "light gray,
no contrast" and misattributed to the theme for a long time, because every
individual color was plausibly a design choice. fixed by linearizing at
the GPU boundary (shaders plus `to_linear_array`).

## failure two: web, too dark

the WebGPU canvas refuses sRGB surface formats. with the fix from failure
one in place, the program now correctly hands the GPU linear values, but
on the web no one encoded them back: token 48 rendered as 8. the same
assumption ("the surface encodes for us") failed in the opposite
direction. fixed by registering the sRGB variant as a view format and
rendering into that view, restoring encode-on-write on every platform.

## the invariant worth remembering

count the conversions on the full path from hex literal to photon. the
answer must be exactly one decode (sRGB to linear, entering GPU work) and
exactly one encode (linear to sRGB, at the surface write). zero or two of
either produces a wrong image whose direction (washed or crushed) tells
you which conversion is missing or doubled. measure a known token's pixel:
118 for a 48 token means double encode; 8 means missing encode.

## why it survived review twice

both bugs produce internally consistent images: every color is wrong by
the same monotonic curve, so screenshots look "stylistically dark" or
"stylistically flat" rather than broken. only comparing a measured pixel
against the token's intended value exposes the transfer error. this is a
core argument for the pixel-validation protocol
(kdb/how-to/validate-visuals-by-pixel.md).
