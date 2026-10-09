---
project: plev
audience: [ai-agents, contributors]
status: done
last-updated: 2026-09-22
domain: hot-reload
---

# task-56 (gap-1): hot reload, done

opened 2024-01-25 as gap-1 with three tiers (shader, narrate DSL,
subsecond). closed 2026-09-22 with what an authorial engine can own:
shader and literal reload in place, code by rebuild. adr:
docs/adr/workflows/0031-hot-reload-on-by-default-in-gpu-context.md.

## what was there in the morning

- tier 1 and tier 2 written and unit-tested in 2024, never reached a
  running app: the feature was off by default and no crate enabled it;
  the watcher lived in `window::App`, which showcase, ide and urnaui do
  not use; `shader_dir()` pointed at a directory that did not exist; the
  reloaded pipelines used the raw surface format instead of the srgb
  view format; the poll sat in `about_to_wait`, which sleeps under
  `ControlFlow::Wait`; a reload only forced a frame, clean layers kept
  the old shader; the narrate root did not match `file!()`.

## checklist

- [x] shader watcher in `GpuContext`, wake by `request_redraw`, poll in
      `present_frame`, `mark_all_dirty` on a swap; every app for free
- [x] on by default in desktop debug builds (`cfg(hot_reload)` from
      build.rs); release, wasm, mobile unchanged
- [x] `shader_dir`, srgb view format, workspace root fixed
- [x] literal reload: snapshot + skeleton hash + delta per file
      (proc-macro2), applied at `TextNodeKey`, `TextMeasurer`,
      `push_to_layer` (rects, text, borders, gradients, path vertices)
      and the window clear; revert removes the override
- [x] code changes: `cargo build -p $CARGO_PKG_NAME --bin <name>` and
      exec, same pid, same terminal
- [x] tests: 7 unit tests on the diff and apply; live captures by
      window id on the showcase (shader pixel, page subtitle, retained
      sidebar string, theme shade, all with the pid unchanged)
- [x] gate green
- [ ] tier 3 (subsecond): no-go. it needs dioxus' cli and linker in the
      build loop; the engine is authorial. the rebuild path covers code.
- [ ] narrate DSL: left as it was (root fixed, `crates/*/src` watched),
      experimental, no app uses it

## limits (by construction, see the adr)

value-keyed substitution at the sink: a literal that appears twice on
screen changes twice; formatted strings, derived colors, numbers and
strings used as keys need the compiler. a position-keyed variant (an
engine macro recording file/line/column, the rsx pattern) would lift
the first three at the cost of marking the code; not taken.
