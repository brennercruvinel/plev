---
type: how-to
tags: [wasm, trunk, webgpu, build, deploy, web]
date: 2026-06-10
commit: 55c8aa7
---

# how to build and serve the web target

## build

```
rustup target add wasm32-unknown-unknown   # once
script/web build --release                 # repo root; writes web/dist/
```

trunk reads web/index.html (canvas styled 100vw/100vh, background
#303030 to avoid a white flash) and web/Trunk.toml, builds the showcase
bin for wasm32, runs wasm-bindgen with the CLI version matching
Cargo.lock, and emits hashed artifacts into web/dist/.

`script/web` from the root (port 8080, watch mode) is the development
loop.

## serve

any static server over web/dist/ works; wasm must be served with the
`application/wasm` MIME type for `instantiateStreaming` (python
`http.server` does this correctly):

```
python3 -m http.server 8090 --directory web/dist
```

## runtime requirements

- WebGPU only (`Backends::BROWSER_WEBGPU`): chrome or edge 113+.
  localhost counts as a secure context; no flags needed. safari and
  firefox stable will fail at `request_adapter` until a WebGL fallback is
  introduced (deliberate scope decision, see docs/status.md)
- engine feature `web-entry` stays off for apps that define their own
  entry (the showcase does)

## verification checklist

1. `cargo check --target wasm32-unknown-unknown -p showcase` green
2. `script/web build --release` succeeds, index/js/wasm respond 200
3. browser console shows "GPU context ready (async)"
4. background pixel samples 48,48,48 (see validate-visuals-by-pixel)
5. resizing the browser window reflows the layout (cards change column
   count); if it does not, someone reintroduced a fixed inner size on
   wasm (see ADR async-gpu-init-and-single-wasm-entry)

## known web-target gaps

- favicon missing (one 404 in the console)
- escape with no overlay open exits the winit loop and freezes the canvas;
  reload recovers. gate `event_loop.exit()` off on wasm when touching that
  code
