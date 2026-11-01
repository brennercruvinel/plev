---
id: operations-build-and-serve-the-web-target
title: "how to build and serve the web target"
project: plev
type: manual
status: active
date: 2026-09-23
date_basis: header
recorded: 2026-06-10
topics: [web-wasm, gpu, platform]
relations: [adr-0001, operations-validate-visuals-by-pixel, task-61, task-75]
source: docs/ref/build-and-serve-the-web-target.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# how to build and serve the web target

Steps to build and serve the web target: uigallery compiled to wasm32 with `trunk`, with the output in `web/dist/`, served as `application/wasm` and opened in a browser with WebGPU. It also records what the target does not have yet (the agent socket, browser accessibility, hot reload) and the check to run when touching this path. The commands are the current guidance and were not run in this review.

## build

```
rustup target add wasm32-unknown-unknown   # once; rust-toolchain.toml lists it
tool/tasks/webtarget.sh build --release                 # repo root; writes web/dist/
```

`tool/tasks/webtarget.sh` runs `trunk --config web <cmd>`: trunk reads `apps/uigallery/web/Trunk.toml`
and `apps/uigallery/web/index.html` (canvas styled 100vw by 100vh, background #303030 to
avoid a white flash), builds the uigallery for wasm32, runs wasm-bindgen
with the CLI version matching Cargo.lock, and emits hashed artifacts into
`web/dist/`. a Homebrew rust ignores rust-toolchain.toml and ships no
wasm32 std; use rustup.

`tool/tasks/webtarget.sh` with no argument is the development loop (127.0.0.1:8080,
watch mode). urnadbapp has its own: `tool/tasks/webtarget.sh urnadbapp` on 8081, from
`apps/urnadbapp/web`.

## serve

any static server over `web/dist/` works as long as wasm is served as
`application/wasm` (python `http.server` does):

```
python3 -m http.server 8090 --directory web/dist
```

## runtime requirements

- WebGPU only (`Backends::BROWSER_WEBGPU`): chrome or edge 113+. localhost
  is a secure context; no flags. safari and firefox stable fail at
  `request_adapter` until a WebGL fallback exists (a deliberate scope
  decision, not a bug)
- the engine feature `web-entry` stays off for apps that define their own
  entry (the uigallery and urnadbapp do)

## what the web target does not have (by design or not yet)

- the agent socket (task-61) is desktop only; an agent reaches the web
  build only through a future transport
- AccessKit has no browser adapter; web accessibility is a decision in
  task-75 (an ARIA mirror projected from the same snapshot, or nothing yet)
- dev-only hot reload and pixel-to-source (`#[cfg(hot_reload)]`) never
  compile for wasm

## verification checklist

1. `cargo check --target wasm32-unknown-unknown -p uigallery` green (the
   gate runs it)
2. `tool/tasks/webtarget.sh build --release` succeeds; index, js and wasm respond 200
3. the browser console shows "GPU context ready (async)"
4. the background samples 48,48,48 ([validate-visuals-by-pixel.md](validate-visuals-by-pixel.md))
5. resizing the window reflows the layout; if it does not, someone
   reintroduced a fixed inner size on wasm (ADR [async-gpu-init-and-single-wasm-entry](../../adr/rendering/0001-async-gpu-init-and-single-wasm-entry.md))

## known gaps to recheck when touching this path

- a missing favicon logs one 404
- the uigallery handles CloseRequested with `event_loop.exit()`
  (`apps/uigallery/src/shell/window/events/mod.rs`); on wasm an exit freezes
  the canvas until reload. Escape no longer exits anywhere (it closes what is
  open, nothing more), and the quit keys are desktop only
