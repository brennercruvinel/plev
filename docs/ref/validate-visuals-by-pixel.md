---
type: how-to
tags: [validation, screenshots, pixels, semantics, regression, macos, headless, playwright, qa]
date: 2026-09-23
status: living
---

# how to validate visuals by pixel

"looks the same" approved two broken color systems in this project before
numeric validation was adopted. the protocol below is how a visual claim
is settled. the orchestrator applies it to every delivery itself; there is
no separate verifier agent (docs/ref/generative-runtime-guidelines.md).

## principle

every visual claim is settled by sampling pixels and stating the delta
against the token. whoever cannot produce numbers has not verified.

meaning comes before pixels. once the semantic contract exists (task-57
onward), a claim about a control has two halves: its projected state
(role, label, state, actions, from the snapshot) and its pixels. a change
that should only restyle a control shows "semantic unchanged, visual
changed"; a label change shows up in the snapshot before any pixel is
sampled. the regression harness below asks in that order.

## regression: meaning, then layers, then pixels

`tool/bench/regress/src` keeps a baseline per page, theme and
viewport under `tool/bench/regress/baselines` (the projection, every
layer under a stable name with a stable content hash, every bound
control) and checks the tree against it
(docs/adr/workflows/0045-regression-by-meaning-before-pixels.md):

```
cargo run -p regress -- check [--page Todo] [--json report.json]
cargo run -p regress -- bless [--page Todo | --all] [--theme hoff] [--png | --no-png]
```

- `check` stages every baseline without a window and compares the
  projection (`Snapshot::diff`), then each layer's hash and each
  binding's rect. nothing moved: it ends there, no frame drawn, no GPU
  opened (the last line says so). a layer that moved on a baseline that
  keeps its image draws the frame, compares pixels only inside the
  regions of the layers that moved, and names the controls the changed
  pixels fall in: `todo.item[2]  semantics unchanged; visual changed,
  layer todo, 964 px at 507,274 204x12`. exit 0 held, 1 moved.
- the verdict is meaning and hash; pixels explain it. a hash that moved
  with 0 px changed still fails and says 0 px.
- a change that moves a page is accepted with `bless` in the same
  change. read what it prints: it is the check of the baseline it
  replaces. the CI job `structural regression (macos)` runs `check`.
- the todo baselines at 1280x800 keep their image, so a todo change
  comes back with the exact control and region; every other baseline
  keeps none and names the layer (and every control bound in it).
  `bless --png --dir <tmp>` makes a local copy with images when a page
  change needs regions.
- the hash sees the scene, not the raster: a shader, a font file or the
  rasterizer change pixels without moving a hash. those need real frames.
  for a change that touches every page (a wgpu, winit or text-stack
  upgrade), capture the whole gallery before and after and compare byte
  for byte:
  `regress -- capture --out target/determinism/<base>` on the old code,
  then `regress -- capture --out target/determinism/<new> --against
  target/determinism/<base> --no-png` on the new. every frame must be
  `equal` (docs/adr/rendering/0074-a-renderer-upgrade-is-proven-by-a-full-capture.md);
  for a narrower change, the sampling below.

## headless first

the engine renders offscreen, so a missing window or a missing screen
permission is never a reason to reason about the code instead of
measuring:

```
cargo run -p engine --bin text_probe -- out.png 2.0      # 2.0 = raster scale
PROBE_FILL=1 cargo run -p engine --bin text_probe -- out.png 2.0
```

`PROBE_FILL=1` pushes enough distinct glyph, size and weight combinations
through the atlas that it has to grow partway through the frame, the
state the atlas lifecycle defects only show under
(docs/adr/rendering/0024-glyph-raster-identity-and-atlas-isolation.md). a probe that
cannot reach the failing state proves nothing.

the renderer behind it is `engine::drawing::text::probe` (offscreen texture, real
`TextSystem`, real shader, readback to RGBA8).
`rust/engine/tests/text_raster_pixels.rs` asserts through the same
entry point. tests skip when no GPU adapter is present rather than
failing.

since task-66 the probe renders through `engine::drawing::headless`, which draws
with the window's own frame path (`window::draw_frame`) into an
offscreen texture: same pipelines, same srgb view, no second renderer.
any showcase page and any `.monster` frame go to png the same way:

```
cargo run -p uigallery -- --render Todo --theme hoff --width 1280 --height 800 --out todo.png
cargo run -p player f.monster --frame 0.5 --out f.png
```

two runs on one machine are byte-identical, the hoff background measures
(48,48,48), and a monster frame matches the windowed player's by sha256
(Metal). prefer this to a window capture for anything that does not need
a live window; an adapter-less machine skips the GPU tests.

## desktop windows (macOS)

- plev windows may open on a secondary monitor. full-screen
  `screencapture` grabs whatever sits on the primary display and produced
  false audit results
- resolve the window id first (`CGWindowListCopyWindowInfo` filtered by
  owner name), then `screencapture -l<windowid> out.png`
- sample with python PIL (`Image.open(p).getpixel((x, y))`) at documented
  coordinates (page background, sidebar, one card) and compare to the
  tokens in `rust/engine/src/styling/theme/hoff.rs` and `scales.rs`
- `screencapture` needs Screen Recording permission; a terminal without it
  gets "could not create image from display". go headless instead

## color profiles in captures

screencapture saves in the monitor's ICC profile, not srgb. on a wide or
calibrated display a true #303030 background measures (42,42,42) raw and
produces a false fail. convert the capture to srgb before sampling (PIL
ImageCms `profileToProfile` with the embedded profile); the same pixel
then measures (48,48,48). never sample raw bytes from an ICC-tagged png.

## web target

- chrome `--headless=new --screenshot` hangs on winit and wgpu apps (the
  page never reaches load-idle under the requestAnimationFrame loop); do
  not retry that route
- working route: playwright-core driving an existing chromium with
  `--enable-unsafe-webgpu --enable-features=WebGPU --use-angle=metal`;
  navigate, wait for the engine's readiness line in the console ("GPU
  context ready (async)", captured with `page.on('console')`), screenshot,
  resize, wait, screenshot again
- look for existing binaries (`~/Library/Caches/ms-playwright`, the
  chrome-for-testing bundle) before downloading anything

## across targets

pixel identity between desktop, web and mobile is task-77. until its
harness exists, compare targets by sampling the same documented
coordinates on each and record every difference as either asked for by
the product (safe area, scale factor) or unexplained.

## kill tests (causality)

when a root cause is claimed, revert the fix in a scratch tree and watch
the named regression tests fail with the production symptom. a fix whose
removal changes nothing was not the cause.

## process hygiene during automated runs

parallel capture sessions leak orphan app instances when kills race; a
six gigabyte "leak" was once only orphans while the app measured a stable
115 MB. measure the RSS of the live process before diagnosing a leak and
sweep orphans (`pgrep -fl <app>`) after every capture batch.

## machine in use

launching apps for capture steals focus; if the user is typing, keystrokes
leak into the app under test and corrupt the capture. restore the user's
frontmost app right after each capture and re-verify the app state before
measuring. prefer headless whenever the user is at the machine.
