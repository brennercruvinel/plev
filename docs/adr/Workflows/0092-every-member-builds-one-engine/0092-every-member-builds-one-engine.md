---
id: adr-0092
title: "every member builds one engine, and the gate builds the other shape"
project: plev
type: adr
status: accepted
date: 2026-11-24
date_basis: git
recorded: 2026-11-25
topics: [workspace, performance, process]
relations: [task-106, adr-0083]
evidence: [ce07c1a2, 0844ce0c]
source: reconstruction
source_revision: 0a8598f9
revised: 2026-11-25
review: reviewed
---

# every member builds one engine, and the gate builds the other shape

Decides that every member of the workspace takes the engine with its default features, so a build compiles one engine, and that the gate compiles the engine without them, the shape only `rust/llmrun` asks for ([task-106](../../../Tasks/task-106-build-weight-and-reuse/task-106-build-weight-and-reuse.md)).

## context

`graphs` was the one member with the engine's default features off and without the `unions` line, so a build of it compiled a second engine; its history gives no reason for either. That shape was also broken: `styling/theme/adapt.rs` read a module of the `accessibility` feature without its `cfg` since 2026-11-01, the engine did not compile without its defaults, and `cargo run -p graphs` failed. The gate never saw it, because every other member takes the defaults and nothing it ran compiled the other shape.

## decision

- A member depends on the engine by `engine.workspace = true`, and on `unions` like every other ([adr-0083](../0083-feature-unification/0083-feature-unification.md)).
- The engine keeps compiling without its default features: `rust/llmrun`'s `plev` feature builds it that way.
- The gate and CI run `cargo clippy -p engine --no-default-features -- -D warnings`.

## consequences

Every package resolves one engine (`accessibility`, `default`, `hot-reload`), and a demo reuses the engine another demo compiled. Code that reads a gated module without its `cfg` is red in the gate the day it is written.

The gate's wasm check covers `uigallery` only. A change to a demo's features on the web is checked by that demo's own web build (`tool/tasks/webtarget.sh graphs build --release`), which is not in the gate.

## alternatives

- A member without the defaults for a lighter demo: a second engine in every build directory, for a 24 MB executable that links in 2 s.
- Dropping the shape without defaults: `rust/llmrun` builds it.

<!-- related: written by tool/tasks/docbase.py index from the relations in the headers; edit the headers, not this list -->
## Related

- [ADR 0083: feature unification reduces duplicate compilations](../0083-feature-unification/0083-feature-unification.md)
- [task-106: what a build directory weighs, and a warm start for a new checkout](../../../Tasks/task-106-build-weight-and-reuse/task-106-build-weight-and-reuse.md)
<!-- /related -->
