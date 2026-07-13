---
type: how-to
tags: [validation, screenshots, pixels, macos, headless, playwright, qa]
date: 2026-06-10
commit: bb34a1c
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
  src/theme/hoff.rs

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

## color profiles in captures (learned 2026-06-11)

screencapture saves in the MONITOR's ICC profile, not sRGB. on a wide or
calibrated display the raw pixel of a true #303030 background measures
(42,42,42) and produces a false FAIL of delta 6. always convert the
capture to sRGB before sampling (PIL ImageCms profileToProfile with the
embedded profile), after which the same pixel measures (48,48,48) exact.
the capture pipeline must treat the embedded profile as part of the
measurement, never sample raw bytes from an ICC-tagged png.

## machine-in-use etiquette

launching apps for capture steals focus; if the user is typing, their
keystrokes leak into the app under test and corrupt captures (observed:
sections switched, a todo deleted). restore the user's frontmost app
immediately after each capture and re-verify the app state before
measuring.
