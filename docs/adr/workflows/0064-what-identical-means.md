---
id: adr-0064
title: "what identical means"
project: plev
type: adr
status: accepted
date: 2026-09-24
date_basis: header
topics: [determinism, rendering, platform]
relations: [task-77]
source: docs/adr/workflows/0064-what-identical-means.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# what identical means

Defines what “identical” means in the frame comparisons of the `regress` harness: four classes (`equal`, `requested`, `silent` and `resized`) and a versioned table of product decisions, `tool/bench/regress/requested.json`. Two frames from the same target and the same GPU are identical byte for byte; across GPUs, the deviation is measured and bounded and never becomes `equal`, and across scales frames are not compared pixel by pixel. The result is the desktop equal to itself in 1312 of 1312 frames, and web, iOS and Android with the bands declared as `known` or `to_confirm`.

## context

the thesis (section 23) promises the same UI with the same pixels on every
target, as far as that serves predictability, and asks the runtime to tell
a difference the product asked for from one that appeared without anyone
asking. until task-77 the `regress` harness compared a desktop frame with
its own baseline and said equal or changed: it did not measure how much,
did not say whether a difference was a decision, and did not compare one
target with another.

three things make "the same pixels" hard, and each needs its own answer
instead of one tolerance for everything:

- some differences are decisions: the platform draws its status bar over
  the app, the device's scale is the frame's, the browser's
  devicePixelRatio is the page's.
- some are not in the frame at all: the operating system and the browser
  color-manage the bytes on the way to the display, and a scaled display
  mode resamples them.
- the rest come from the GPU and its driver: the precision of `pow`,
  `exp` and `sqrt` in the shaders, the sRGB encode and the blend into an
  8-bit target, the MSAA resolve, the bilinear weights. no shader of the
  engine takes a derivative or dithers (`rust/engine/src/drawing/gpu/shaders/`):
  the SDF edge is `1 - smoothstep(-0.5, 0.5, d)` over a distance in logical
  px, a fixed feather of one logical px.

## decision

a comparison is between two frames of one page, theme, viewport and scale,
and its verdict is one of four classes (`tool/bench/regress/src/
classify.rs`):

- `equal`: every byte the same.
- `requested`: they differ, and every differing pixel falls in an area a
  decision of the product declares for the target.
- `silent`: a differing pixel falls in no declared area. it is measured
  over those pixels only: how many, the largest and the mean channel delta
  (0 to 255, the largest of the four channels of a pixel), the rect that
  holds them. a silent difference needs a cause.
- `resized`: two sizes, not compared by pixel; the reason when a decision
  declares a size difference (a scale, a viewport).

what identical means, by class of comparison:

| comparison | identical means | measured |
|---|---|---|
| the desktop against itself: two GPU contexts, two processes, any order, any resize history | every byte, no tolerance | 1312 of 1312 frames equal (82 pages, 4 themes, scales 1, 2, 1.25, 1.5, apple M4 on metal), and every primitive of the frame path at the four scales (`rust/engine/tests/headless_determinism.rs`) |
| the same target on another machine of the same GPU family | every byte | not yet |
| another target on the same GPU through another API (the web on dawn over metal, the iOS simulator on the host GPU) | every byte outside the declared areas | not yet; the headless chromium already installed reaches the M4 through dawn with `--use-angle=metal` |
| across GPUs and drivers (a software adapter in CI, windows, linux, android) | a bounded delta, measured: proposed at a channel delta of at most 2 per pixel. 0 is expected inside flat fills and glyphs away from their edges (one sRGB color decoded by `pow` in the shader and encoded back by the target; glyphs rasterized on the CPU from embedded fonts and sampled at texel centers), and a delta there is named, not absorbed | not yet; the bound is a proposal that the first cross-GPU capture confirms or replaces with its numbers |
| across scales | not comparable by pixel (`resized`); the layout, the text and every primitive keep their size in logical px | the blur too since task-77 phase 1b (the glass, the layer blur and shadow effects): under sigma 4 an edge's 10% to 90% ramp is 9 logical px at 1x and 2x, 8.7 at 1.5x and 9.6 at 1.25x (a physical px of grain), where it was 6 at 1.5x and 4.5 at 2x; scale 1 kept every byte |

a bounded delta is never a verdict of equal: the comparator says silent,
with its numbers, and the bound decides whether that silent difference
fails a job. a difference inside the bound on an edge is expected; the
same delta on a flat fill or on text is a cause to find.

what is a decision of the product lives in one versioned table,
`tool/bench/regress/requested.json`, one row per
decision (target, name, area, reason in one line, `known` or
`to_confirm`), read by the comparator as the target's decisions:

- desktop: the size (logical viewport times the window's scale factor;
  macOS gives 1 or 2, windows and linux also 1.25 and 1.5).
- web: the size (the canvas fills the page; devicePixelRatio is the
  scale). to confirm.
- iOS: the status bar and the dynamic island (a top band, 62 pt taken),
  the home indicator (a bottom band, 34 pt), the size (402x874 at 3 on the
  iPhone 17 Pro). to confirm; the gallery does not inset for the safe area.
- android: the status bar (24 dp), the navigation bar (48 dp), the size
  (the density is the scale, 2.625 at 420 dpi). to confirm.

identical stops at the bytes the surface receives. what the operating
system does after the present is not the frame: the metal layer goes out
without a colorspace (wgpu 28 and winit 0.30.13 never set one; the
default is nil), the WebGPU canvas keeps its default, and a P3 display or
a scaled display mode transforms the bytes on the way to the eye. a
capture of a window or a browser is converted from its embedded profile
to sRGB before it is compared (the scripts do it with sips).

what runs where:

- the gate: the comparator's tests with synthetic frames, the table's
  validation, a small capture on a GPU (two runs against each other, a
  changed png reported as silent), and the engine's determinism test.
  every GPU test skips without an adapter.
- a CI job on a mac with a GPU, when CI runs again: `regress capture`
  (about 70 s for the 1312 frames) must hold at `equal` in every frame,
  across two contexts and against a second process. `regress check`
  stays the structural job it is.
- manual, recorded in the task file with the machine and the adapter:
  `tool/tasks/screencap.sh web`, `tool/tasks/screencap.sh ios`, `tool/tasks/screencap.sh android`,
  each against the desktop's frame of the same page, theme, viewport and
  scale. they become jobs when a runner has the tools (trunk and
  wasm-bindgen for the web; xcodegen and the ios-sim target for iOS; the
  android SDK, emulator, NDK, a JDK 17 and cargo-ndk for android).
- a software adapter in CI (linux) compares with itself at `equal` and
  with the metal pngs only as a measurement, until its bound is set.

## consequences

- the harness can now say how much two frames differ and whether it was
  asked: `regress compare <base.png> <other.png> --scale <s> --target
  <t>` and `regress capture [--against <dir>]`. the frames and the
  report go to `target/determinism/`, outside git; no baseline was added.
- the desktop is deterministic with itself on this machine, at the
  fractional scales too: every byte, every frame.
- the divergence across scales the probe found (the blur's sigma in
  physical texels) is fixed (task-77 phase 1b): at any scale other than 1
  the kernel of 1x is laid over the physical texels by area, scale 1 keeps
  its pipeline and its bytes, and the test it left ignored runs.
- the rows of web, iOS and android are declared from each platform's
  behavior and marked `to_confirm`: the first capture on each target
  measures the bands and corrects the table in the same change.
- the web and android builds take the page and the theme only by key
  (the digits reach the first ten pages, T the four themes): a launch
  channel (a query in the URL, an intent extra) is needed to capture any
  page there.

## alternatives

- uma tolerância única para todas as diferenças: rejeitada. Cada origem de diferença pede resposta própria: a decisão de produto, o que o sistema operacional faz depois do present, e o que vem da GPU e do driver (`## context`).
- tolerância que transforma um delta em `equal`: rejeitada. O delta é reportado, e o limite decide o job, não a classe (`## avoid`).
- comparar duas escalas reamostrando um quadro sobre o outro: rejeitado (`## avoid`). A tabela marca essa comparação como `resized`.
- banda na tabela sem motivo, ou com um palpite marcado `known`: rejeitado (`## avoid`).
- medir a web em swiftshader e tratar o resultado como quadro da GPU: rejeitado; a captura registra o adaptador (`## avoid`).
- tratar o limite de 2 por canal entre GPUs como regra já aceita: ficou como proposta até a primeira medida em outra GPU (`## integração`).

## avoid

- a tolerance that turns a delta into `equal`. the delta is reported, and
  the bound decides a job, not the class.
- a comparison across two scales by resampling one frame onto the other.
- a band in the table without a reason, or a reason that is a guess
  marked `known`.
- sampling a capture tagged with a display profile as if it were sRGB.
- measuring the web on swiftshader and calling it the GPU's frame: the
  capture records the adapter.

## integração (2026-09-24, orquestrador)

aceito. o limite de 2 por canal entre GPUs fica como proposta até a
primeira medida em outra GPU. a divergência do blur (o sigma em texels
físicos, então o vidro espalha 9 px lógicos a 1x e 4,5 a 2x) é defeito,
não decisão de produto: a correção entra como task-77 fase 1b, com o
teste ignorado desta fase como critério.


## adição da fase 2 (2026-09-25, orquestrador): as primeiras capturas

- web (chromium headless, WebGPU sobre Metal, mesma GPU do desktop): das
  82 páginas em core a 1280x800 e 1x, 79 iguais em todo byte; as dez
  primeiras também a 2x e 3x. a linha `size` da web passa a `known`.
- iOS (simulador do iPhone 17 Pro, iOS 27.0, 402x874 a 3x): 78 páginas
  diferem só dentro da barra de status; `status-bar`, `home-indicator` e
  `size` passam a `known`.
- android (emulador Pixel 7, Vulkan do llvmpipe, por software): delta de
  canal de no máximo 2 fora da barra de navegação, dentro do limite entre
  GPUs; `navigation-bar` e `size` passam a `known`, `status-bar` segue
  `to_confirm`.
- as caladas que sobram não são do renderer: a página mostra outra coisa
  porque a plataforma não tem o que o desktop tem (o Claude, o git) ou
  porque o cursor pisca (o Editor no iOS). as causas estão no arquivo da
  task-77.
