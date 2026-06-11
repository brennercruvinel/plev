---
type: how-to
tags: [plev, engine, widgets, layout, text, color, clipping, checklist]
date: 2026-06-10
commit: ac40423
---

# how to code against the plev engine

this is the operating manual distilled from every defect found and fixed
while building showcase and basicIDE on plev (wgpu 28, winit 0.30,
cosmic-text 0.18, taffy 0.9). each rule below was violated at least once
in this repository, shipped a visible defect, and was then root-caused.
agents and developers should treat violations as defects even when the
screen currently looks correct.

## before writing anything

check whether the engine already provides the capability. the engine had
flex layout with text measure functions, percent dimensions, touch
recognition and real text measurement while both apps reimplemented these
poorly by hand. the single most expensive failure pattern in this project
was app-layer reimplementation of an existing engine feature. grep src/
first; also look for one screen that already does it right (the theme and
icons galleries were the in-repo proof of correct responsive layout).

## text

- one `TextStyle` per text run, used for both `TextMeasurer::measure_styled`
  and `TextNodeKey::from_style`. never construct measurement and drawing
  parameters separately (see ADR one-text-style-for-measurement-and-drawing)
- never estimate text width arithmetically. `TextMeasurer` is GPU free and
  cached; there is no performance excuse
- a family+weight pair resolves only if that exact face is embedded
  (src/text/fonts.rs). if a new weight is introduced, embed it, then add
  it to the face-resolution test
- letter spacing, weight and line height all change advance. if any new
  typography attribute is added, plumb it through measurement and drawing
  in the same commit

## color

- hex and theme values are sRGB. anything entering GPU memory must be
  linear: `to_linear_array()` for clear colors and uniforms; vertex colors
  are linearized in-shader
- surface render targets are created exclusively through
  `gpu.surface_render_view(&output)`. a default `create_view` skips gamma
  on the web target
- validate color work by sampling pixels, never by inspection. desktop
  background must measure 48,48,48 (and so must web)

## layout and responsiveness

- container geometry derives from `content.w` or from `LayoutEngine`.
  constants only as min, max and gap
- grids: `cols = floor((content.w + gap) / (min_w + gap)).max(1)`, then
  stretch the column width to fill, clamped by a readability maximum
- intrinsic rows wrap against `content.w`
- user-adjusted sizes store desired separately from effective; clamp at
  read time
- keep `layout()` functions pure (no GPU, no window). write viewport
  regression tests at an explicit narrow and wide width for every new
  screen (see showcase view tests for the pattern)

## events and invalidation

- any handler that changes visible state must invalidate (return true /
  request redraw). under render on demand a missed invalidation is a
  frozen app, not a glitch
- clipping rects pushed in logical pixels must be scaled by
  `gpu.clip_scale()` when they become physical scissor rects (HiDPI)
- after `gpu.resize`, the logical projection must be reapplied. the engine
  App does this; standalone event loops must call `set_projection`
  themselves

## platform

- no blocking executor on any path reachable from wasm; GPU init follows
  the spawn_local plus EventLoopProxy pattern (see ADR
  async-gpu-init-and-single-wasm-entry)
- touch arrives as synthesized pointer events; widgets need no special
  handling. multi-finger gestures come from the recognizer when needed
- `cargo check --target wasm32-unknown-unknown -p showcase` must stay
  green; it is the cheapest cross-platform guard in the repo

## definition of done for visual work

1. workspace tests green, including new regression tests for the change
2. wasm check green
3. the affected screen sampled by pixel at two window widths
4. no new constant that encodes a viewport assumption
5. measurement and drawing provably share one style object for any text
   touched
