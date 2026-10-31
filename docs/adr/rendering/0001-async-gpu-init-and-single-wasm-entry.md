---
id: adr-0001
title: "async GPU initialization and one wasm entry point per module"
project: plev
type: adr
status: accepted
date: 2025-02-10
date_basis: header
topics: [web-wasm, gpu]
relations: []
source: docs/adr/rendering/0001-async-gpu-init-and-single-wasm-entry.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
evidence: [55c8aa7]
---

# async GPU initialization and one wasm entry point per module

Decides that GPU initialization on wasm is asynchronous, with the result returned to the event loop as a user event, and that the engine has an optional wasm entry point, behind the `web-entry` feature. The reason is that the browser cannot block, and a wasm module accepts a single `start`, which collided with the apps'. With this, a single app code serves both targets, with two cfg-gated sections, and the canvas is sized by CSS.

## context

bringing the showcase to the browser surfaced three structural constraints
that desktop code never exercises:

- the browser cannot block. `pollster::block_on(GpuContext::new(...))`
  inside `resumed` deadlocks or panics on wasm, because adapter and device
  requests are real promises
- a wasm module accepts exactly one `#[wasm_bindgen(start)]`. the engine
  exported an unconditional start, which made every downstream app's entry
  collide with it at link time
- winit on web sizes the canvas through a ResizeObserver watching CSS
  layout. setting a fixed `inner_size` pins an inline style and disables
  that tracking permanently

## decision

- on wasm, GPU init runs in `wasm_bindgen_futures::spawn_local`; the
  result returns to the event loop through an `EventLoopProxy` user event
  (GpuReady). the handler installs the context and re-syncs the viewport,
  which also covers resizes that occurred during initialization. desktop
  keeps `block_on`
- the engine's own start lives behind the cargo feature `web-entry`
  (default off). apps define their own entry with `spawn_app`,
  `console_log` and `console_error_panic_hook`, and reuse the public
  `plev::window::setup_wasm_canvas`
- the canvas is styled 100vw/100vh in CSS and never given a fixed inner
  size on wasm. browser window resizes arrive as ordinary
  `WindowEvent::Resized`
- per-target dependencies are split in Cargo.toml (pollster and env_logger
  native only; wasm-bindgen, console_log and the panic hook web only)

## consequences

- one `cargo check --target wasm32-unknown-unknown` in CI guards the
  build; the runtime path is exercised by the trunk build plus a scripted
  browser screenshot
- the same app source serves both targets with two cfg-gated regions
  (init and main), no forked app

## avoid

- never call a blocking executor anywhere reachable from wasm, including
  transitively through helper constructors
- never ship an unconditional `#[wasm_bindgen(start)]` from a library
- never set `with_inner_size` under `target_arch = "wasm32"`
