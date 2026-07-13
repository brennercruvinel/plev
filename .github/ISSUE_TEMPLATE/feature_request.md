---
name: feature request
about: propose a capability or demo
title: ""
labels: enhancement
---

## the capability

<!-- what it does, and who consumes it (engine, an app, an example). -->

## why the engine cannot already do it

<!-- check first: apps never reimplement engine capabilities. if the engine
already provides this, the request is to use it, not to rebuild it. -->

## shape

- [ ] backend logic first, with unit tests, before any pixels
- [ ] fits the binding contracts (one TextStyle per run, srgb in/out, content-driven geometry, invalidate on visible change)
- [ ] is it a module under `crates/engine/src/`, or its own crate? (own crate only with a distinct identity/deps/separate-compile)
