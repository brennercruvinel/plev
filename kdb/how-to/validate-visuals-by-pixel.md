---
type: how-to
tags: [validation, screenshots, pixels, macos, headless, playwright, qa]
date: 2026-06-10
commit: ac40423
---

# how to validate visuals by pixel

"looks the same" approved two broken color systems in this project before
numeric validation was adopted. the working protocol follows.

## principle

every visual claim is settled by sampling pixels and stating the delta
against the token. a verifier (human or agent) that cannot produce numbers
has not verified.

## desktop windows (macOS)

- plev windows may open on a secondary monitor (observed at y around
  1130). full-screen `screencapture` grabs whatever browser or editor sits
  on the primary display; this produced multiple false audit results
- correct method: resolve the window id first (a small swift helper using
  `CGWindowListCopyWindowInfo` filtered by owner name), then capture that
  id directly with `screencapture -l<windowid> out.png`. this is robust to
  monitor layout and focus
- sample pixels from the capture with python PIL
  (`Image.open(p).getpixel((x, y))`) at documented coordinates: page
  background, sidebar, one card. compare to the token table in
  kdb/reference/hoff-visual-tokens.md

## web target

- chrome's `--headless=new --screenshot` mode hangs on winit/wgpu apps:
  the page never reaches load-idle because of the requestAnimationFrame
  loop, and neither `--timeout` nor `--virtual-time-budget` reliably fires.
  two runs were lost to this; do not retry that route
- working route: playwright-core (npm, installs in under a second) driving
  an existing chromium binary with explicit waits. launch args for WebGPU:
  `--enable-unsafe-webgpu --enable-features=WebGPU --use-angle=metal`.
  navigate, wait a fixed seven seconds for wasm and GPU init, screenshot,
  resize the viewport, wait, screenshot again
- a machine without google chrome may still carry usable binaries: check
  `~/Library/Caches/ms-playwright` and the chrome-for-testing bundle under
  VS code extension storage before downloading anything
- capture the browser console in the script (`page.on('console')`);
  "GPU context ready (async)" is the engine's own readiness signal

## kill tests (causality)

when a root cause is claimed, prove it by reverting the fix in a scratch
tree and watching the named regression tests fail with the production
symptom. a fix whose removal changes nothing was not the cause. this
protocol caught one false root-cause claim during the typography work.

## memory and process hygiene during automated runs

parallel capture sessions leak orphan app instances when kills race;
six gigabytes of apparent "leak" in this project was orphan processes,
while the app itself measured a stable 115 MB. measure RSS of the live
process before diagnosing a leak, and sweep orphans (`pgrep -fl <app>`)
after every automated capture batch.
