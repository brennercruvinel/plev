---
id: adr-0031
title: "hot reload on by default: shaders and literals in place, code by rebuild"
project: plev
type: adr
status: accepted
date: 2026-09-22
date_basis: header
recorded: 2026-09-21
topics: [hot-reload, gpu, process]
relations: [adr-0007, adr-0018, adr-0019, learning-hot-reload, task-56]
source: docs/adr/workflows/0031-hot-reload-on-by-default-in-gpu-context.md
source_revision: ec0a224c
revised: 2026-11-02
review: reviewed
---

# hot reload on by default: shaders and literals in place, code by rebuild

Decides that hot reload is on by default in the GPU context: shaders and literals are swapped in place, and code is recompiled with a process restart. The audit that motivated the decision showed that the three 2024 ADRs had never reached a running app: the feature was off, the watcher lived in an `App` that no app uses, and the shader directory pointed to a path that does not exist. The result, verified in the showcase by a window capture with the same pid, is a shader edit applied in about half a second, with no user input.

## context

the three hot reload ADRs (2024) describe a shader tier and a narrate tier
that were written, tested in isolation and never reached a running app.
what the audit found before this change:

- the `hot-reload` feature was off by default and no crate turned it on.
  the showcase, the ide and urnaui never compiled the code.
- the watcher and the poll lived in the engine's own `window::App`. the
  showcase, ide and urnaui have their own `ApplicationHandler` and only
  share `GpuContext`, `EffectProcessor` and `present_frame`, so even with
  the feature on they would have had no watcher.
- `shader_dir()` pointed at `rust/engine/shaders`, which does not exist
  (the shaders live in `src/gpu/shaders`), so the watcher failed to start
  and every read fell back to the embedded copy.
- `reload_shader` rebuilt pipelines with the raw surface format while
  `new_with_config` builds them with the srgb view format
  (render-into-an-srgb-view-format ADR): a reloaded pipeline would not
  match the layer textures.
- the poll sat in `about_to_wait`, which with `ControlFlow::Wait` runs
  once and then sleeps: a saved shader was applied on the next mouse
  move, not on save.
- after a reload only `compositor.invalidate()` was called, which forces
  a frame but not a layer redraw. clean layers kept textures drawn with
  the old shader.
- the narrate watcher watched `rust/engine/src`, while `file!()` in a
  workspace member is relative to the workspace root, so override keys
  could never match.

## decision

one watcher, in the object every app already owns. `GpuContext::new`
starts a `ShaderWatcher` on `src/gpu/shaders`; its background thread
queues the changed paths and calls `window.request_redraw()`, which is
the one wake that works from any thread and needs no proxy. `present_frame`
(and the engine App's `render`) polls `gpu.poll_shader_reload(effects)`
before acquiring the surface and, when a pipeline was swapped, calls
`compositor.mark_all_dirty()` so every layer redraws with the new shader.
`effects` is `&mut` in `present_frame` for that reason.

on by default: `hot-reload` is in the engine's default features, and a
seven-line build.rs emits `cfg(hot_reload)` only when the feature is on,
`PROFILE` is debug and the target os is macos, linux or windows. every
`#[cfg(feature = "hot-reload")]` became `#[cfg(hot_reload)]`. release, wasm
and mobile keep the embedded `include_str!` path and never link a watcher;
notify is a desktop-only optional dependency.

rust source: a saved `.rs` is one of two things, and the engine tells
them apart the way dioxus' rsx hot reload does (`packages/rsx-hotreload/
collect.rs`: blank the reloadable parts, compare the rest). here the
reloadable parts are the literals. at start the watcher snapshots every
file under `crates/*/src` with `proc-macro2`: the token stream with
literals blanked, hashed (the skeleton), plus the literals in order.
on save the file is tokenized again. same skeleton: only literals moved,
`baseline[i] -> current[i]` is the delta, applied in place. different
skeleton, or a changed number: code, and the compiler owns it.

the delta is applied where every value passes before it becomes pixels,
so no app marks anything and every widget gets it: strings at
`TextNodeKey::new`/`from_style` and `TextMeasurer::measure_styled` (layout
follows the new text), hex colors at `Compositor::push_to_layer` (rects,
text, borders, gradients, and the vertices of tessellated paths, which is
where icons carry their color) and at the window clear in `present_frame`.
rgb comes from the literal, alpha stays what the widget set. a revert
removes the override; the baseline is the source at process start.

code changes: `cargo build -p $CARGO_PKG_NAME --bin <name>` (name from
`current_exe`, `--example` under `examples/`, `-p` from the env `cargo run`
exports so the feature set is the same and only the touched crates
rebuild) and on success `exec` the fresh binary with the same args. pid,
terminal and `cargo run` stay; the window closes and reopens. this is a
restart, not a reload, and the log says "code changed". a failed build
leaves the old binary running. the web target keeps its loop in `trunk
serve` (`tool/tasks/webtarget.sh`).

no macro, no dsl, no dylib, no third-party toolchain in the engine.
`proc-macro2` (already in the workspace for the macros) is the one
dependency, desktop debug only.

tier 3 (subsecond) stays where the 2024 design left it: not adopted.
the rebuild loop covers the same edits at the cost of a relaunch, and a
second linker plus a websocket in the engine is more surface than the
gain today.

## consequences

`cargo run -p uigallery` is the whole dev loop. save a `.wgsl`: the window
updates in about half a second with no input. save a string or a hex
color anywhere in the sources: same, in place, state intact (page, scroll,
theme, everything). save code: rebuild plus relaunch, under two seconds
for a change inside the showcase crate. all verified on the showcase by
capturing the window by id with the same pid before and after: a
fragment-shader edit on `quad.wgsl` moved a sidebar pixel from 42,42,42 to
42,18,18 and back after the revert; the page subtitle in
`view/pages/table.rs` and the brand tagline held inside the retained
sidebar widget showed the new strings; the first shade of `hoff_core.rs`
moved the page background from 252,252,252 to 255,220,220 and back, while
the window sat on a page the user had navigated to.

limits, by construction: a literal that appears twice in the sources and
on screen changes in both places (find and replace); a string formatted
before display never matches; a color derived from a token (hover
lighten) does not follow the token; numbers are ambiguous and count as
code. a file the baseline has not seen (new file) is code.

`exec` from the watcher thread prints one macOS line ("Task policy set
failed: 4") on relaunch. harmless, it is the kernel refusing to carry the
old thread's QoS into the new image.

the narrate tier is wired the same way as before (polled by the engine
App only) with the root fixed to the workspace and `crates/` watched.
no app uses the DSL; it stays experimental.

the source watcher fires only under `cargo run` or a direct run of the
debug binary from the tree; release builds have no watcher at all.

editors and sed save through hidden temp files (`.!123!quad.wgsl`,
`.quad.wgsl.swp`); the watcher skips names starting with a dot and
dedups paths within a batch.

the feature flag is no longer the switch. `--no-default-features`, or a
release build, is.

## learnings

- [hot reload session: what worked and what bit us](hot-reload.md)
