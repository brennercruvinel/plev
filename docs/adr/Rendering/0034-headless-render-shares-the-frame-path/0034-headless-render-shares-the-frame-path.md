---
id: adr-0034
title: "headless render shares the frame path"
project: plev
type: adr
status: accepted
date: 2026-09-23
date_basis: header
recorded: 2026-09-23
topics: [rendering, gpu, determinism]
relations: [task-77, generative-runtime-thesis]
source: docs/adr/rendering/0034-headless-render-shares-the-frame-path.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# headless render shares the frame path

Decides that headless rendering uses the same frame path as the window: `window::draw_frame` does the drawing, and `present_frame` only acquires the surface, calls `draw_frame` and presents. With this, `engine::headless::Renderer` produces the window's pixels, and the separate `text::probe` pipeline, which no window ever ran, goes away. Measured on macOS, the `hoff` page reads 48,48,48 headless, a value that a window capture also reads after conversion to sRGB, and two runs give the same png bytes across the 312 frames; it matters because CI, reports, the inspector and the agents now compare pixels through a single path.

## context

every app draws a frame the same way after building its scene:
`window::present_frame` (`rust/engine/src/runtime/window/frame.rs`) acquires the
surface texture, resolves the compositor and its text, uploads images,
encodes the layer passes and the composite pass, submits and presents.
drawing and presenting were one function, and `GpuContext` could only be
built from a window: `new_with_config` created the surface from it, took
the size from `inner_size`, picked the format from the surface caps and
built every pipeline in that format.

the only way to get pixels without a window was `engine::text::probe`: its
own device, its own text pipeline and projection, its own readback and a
hand-written png writer, drawing text straight into a texture with no
compositor. the pixel tests of the glyph rasterizer measured that path, a
path no window runs. CI, reports, the session inspector and agents need
the pixels of any page, theme, viewport and scale, and of any `.monster`
frame, and they need them to be the window's pixels (thesis, section 25:
headless is first class).

## decision

one frame path with two targets.

- `GpuContext` has two constructors over one body. `new_with_config`
  keeps the surface part (create, choose the format, configure) and ends
  in `from_device`; `new_headless(width, height, config)` requests an
  adapter with no surface and ends in the same `from_device`, with
  `surface: None` and a `surface_config` holding the size and
  `HEADLESS_FORMAT` (`Rgba8UnormSrgb`). `from_device` builds the
  projection, the layouts, every pipeline and the sampler. no pipeline is
  built twice. `new_headless` returns `None` without an adapter so tests
  skip instead of failing. hot reload watchers start only for a window.
- `window::draw_frame` is the frame after the scene is built: resolve,
  text, images, layer passes, composite into a given view, submit; it
  returns the draw count. `present_frame` is acquire, `draw_frame` with
  the surface view, present: same draws in the same order and the same
  return value as before. the view must be sized like `gpu.surface_config`
  and made through the sRGB view path (`surface_render_view`, or
  `texture_render_view` for a texture that is not a surface texture); the
  layer textures, scissors and backdrop resolve read size and format from
  the context.
- `engine::headless::Renderer` owns a headless context, the text system,
  effect processor and texture pool a window owns, and one target texture
  (`RENDER_ATTACHMENT | COPY_SRC`). `draw` is `draw_frame` into the
  target. `read_texture` copies any RGBA8 or BGRA8 texture back through a
  buffer with rows padded to 256 bytes and returns RGBA rows, so it also
  reads a window's surface texture. `Frame` writes png through the `image`
  crate, already an engine dependency. errors are typed
  (`HeadlessError`), never a panic. the module is compiled off wasm: its
  constructor blocks on the adapter.
- `Rgba8UnormSrgb` as the target: the sRGB variant encodes linear to sRGB
  on write exactly like the window's sRGB view, and RGBA order is png row
  order, so a readback is written as it lands.
- the text probe keeps its public API and builds compositor layers
  rendered by `headless::Renderer`; its pipeline and png writer are gone.
- tools on top: `showcase --render <page> --theme --width --height
  --scale --scroll --out` (the window's launch state, animations stepped
  at a fixed 60 Hz until they settle, at most two seconds), and
  `monster_player <file> --frame <t> --out` (headless) and `--capture`
  (the same frame drawn into the window's surface and read back before the
  present, the check that both targets give the same bytes).

## consequences

- measured on this machine (metal): the hoff page background reads
  48,48,48 headless, the value a window capture reads after conversion to
  srgb; every showcase page (78) in core and hoff at 390x844 scale 3 and
  1280x800 scale 2 renders, and two runs give the same png bytes for all
  312 frames; `monster_player --capture` and `--frame --out` give the same
  sha256 for explosion at 0.5 s and 1.0 s and cards at 0 s and 1.25 s.
- a pass or a resource added to the frame goes into `draw_frame` and
  reaches the window and every headless consumer in the same change.
- the probe tests measure the window's pixels now: layer textures, msaa,
  the composite pass. one test changed meaning: the old probe redrew a
  static layer from vertices retained since frame one; the frame path
  never redraws a clean layer, so the static layer test now asserts that
  its pixels survive frames of atlas churn in the real loop. a dynamic
  probe layer changes its scene hash every frame through a culled
  transparent marker node, which is how animated content redraws.
- the monster player's own copy of the frame tail is gone; it presents
  through `present_frame`.
- the engine's built-in `window::App::render` (`window/render.rs`) still
  has its own frame tail (layer effects, perf hud); no app uses that
  `App`, and folding it into `draw_frame` is its own change.
- CI on linux uses a software adapter: pixels may differ from metal.
  comparison across adapters and targets is task-77, not this path.
- svg export is not part of this change; it needs a shaping entry point
  and the embedded font bytes from `engine::text`, which is its own
  change.

## avoid

- a second offscreen pipeline for a tool or a test. render through
  `engine::headless`, which is the window's path.
- `texture.create_view(&Default::default())` for a render target: go
  through `texture_render_view` or `surface_render_view`.
- `Renderer::new` (it blocks) from anything reachable on wasm.
- drawing into a view whose size differs from `gpu.surface_config`:
  resize the renderer instead.

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [task-66: headless as a first-class citizen](../../../Tasks/task-66-headless-first-class/task-66-headless-first-class.md)
- [task-77: visual determinism](../../../Tasks/task-77-visual-determinism/task-77-visual-determinism.md)
- [phase 1 (task-77a): the probe](../../../Tasks/task-77-visual-determinism/phase-1a-probe.md)
- [plev: thesis of the generative interaction runtime](../../../Research/Synthesis/generative-runtime-thesis.md)
- [how to code against the plev engine](../../../Research/Workflows/code-against-the-plev-engine.md)
<!-- /related -->
