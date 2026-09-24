---
type: adr
status: proposed
tags: [hot-reload, view-source, sem, compositor, showcase, dev-loop, generative-runtime]
date: 2026-09-24
---

# view source for pixels: a control leads to its projection, a text to its literal

## context

task-65 (phases a and b) asks that a pixel of the window lead to the code
that made it, in a desktop debug build only. what existed:

- `Compositor::hit(x, y)` names the topmost bound `SemId` at a point of
  the frame on screen (docs/ADR/semantic-binding-beside-the-scene.md).
- `Node.source` is the file, line and column of the call that projected
  the node, captured by `#[track_caller]`
  (docs/ADR/semantic-contract-as-pure-projection.md). the comps helpers
  that project a widget (`TextField::project`, `Button::sem`, ...) are
  `#[track_caller]` too.
- the literal hot reload keeps, per source file, a skeleton hash and its
  literals in order (docs/ADR/hot-reload-on-by-default-in-gpu-context.md),
  but no position: `proc-macro2` was built without `span-locations`.
- `hot_reload` exists only where the engine's build.rs sets the cfg:
  the `hot-reload` feature (on by default), a debug profile and a desktop
  os. the cfg is the engine's alone; another crate does not see it.

## decision

a point resolves in the engine, under `hot_reload`
(`engine::hot_reload::view_source::origin_at`), and the shell only calls
it and opens what it names.

1. the bound id wins. when `hit` names an id, the answer is its node's
   `source` in the page's snapshot, made absolute from the workspace root
   (`Origin::Projected`), or that the snapshot has no source for it
   (`Unsourced`). a text drawn inside a bound control finds the control.
2. the projection site is enough; no render site is captured. the probe
   on the todo page showed every node leading to the line of the app's
   `semantics.rs` that declares that capability (id, role, label, state,
   actions), in the app and not in comps, because the comps helpers pass
   the caller through. the render code (`draw.rs`, which binds the same
   ids) is one step away. capturing the render site would need
   `#[track_caller]` on render paths in comps, and nothing asked for it.
3. with no id at the point, the text: the topmost text run whose measured
   box (position, size from its key's style, inside its clip) holds the
   point, on a visible layer, layers stacked as `hit` stacks them. the
   literals that hold that text (`literals::sites_of`): exactly one opens
   (`Literal`); several are listed in file, line and column order and none
   is opened, a guess would be worse than a list; none means the text was
   formatted before it was drawn (`Formatted`).
4. positions come from `proc-macro2`'s `span-locations`, turned on by the
   engine's `hot-reload` feature only. resolver 2 keeps it away from the
   `proc-macro2` that syn, quote and every proc macro build with (`cargo
   tree -e features -i proc-macro2` shows the engine's feature as its one
   path); wasm and mobile do not have the dependency. each literal keeps
   its line and column from 1, the column in chars, as `Location` counts
   them. the lexer keeps every text it reads in a map of its thread for
   span lookups and never frees it, so the snapshot releases it once the
   positions are read. a save that keeps the skeleton (literals, comments,
   whitespace) replaces the file's positions with the ones on disk.
5. a literal the app never draws is never an answer: the text of a doc
   comment, a literal inside an item under `#[cfg(test)]`, a literal of a
   test file by the file hygiene guard's rule (`tests.rs`, `tests_*.rs`,
   a `tests` or `benches` directory). before this, most texts on the
   gallery that looked repeated were held once by the app and again by
   the tests that assert them.
6. the editor: `PLEV_EDITOR` (its words are the program and its first
   arguments, `file:line:column` goes last), else `zed file:line:column`,
   else `code -g file:line:column`, found on `PATH`; spawned without
   waiting, a thread reaps it. `open_with` takes both values instead of
   reading them, so a test drives the real opener without touching the
   process environment.
7. the shell (the showcase): a left click with cmd held (ctrl off macOS)
   is taken before the page, press and release, even if cmd is let go
   first. no control or text, no source, several literals or a formatted
   text are logged at warn, the level the shell shows by default. it draws
   nothing and asks for no frame.
8. the showcase has no `hot_reload` cfg, so its module, field and call
   are gated on the terms the engine's build.rs uses:
   `all(debug_assertions, any(target_os = "macos", target_os = "linux",
   target_os = "windows"))`. release, wasm and mobile compile none of it.
   if a profile ever turned `debug_assertions` on outside the dev profile,
   the showcase would fail to compile against a missing
   `engine::hot_reload`; it could never ship the code silently. a
   `build.rs` in the showcase repeating the engine's rule would give it
   the same cfg by name, at the cost of a second copy of the rule.

## consequences

- measured on the workspace (580 files, 43k literals): the baseline goes
  from +7.4 MB and about 115 ms to +8.6 MB and 150 ms. without releasing
  the lexer's map it was +14.5 MB, and one more copy of a file per save.
- a node projected in a loop (the todo's items, its filter pills) leads
  every instance to the same line: the code, not the data.
- a text cut or formatted before drawing (an ellipsis, `format!`) finds
  no literal. a test module declared out of line under a name other than
  `tests` still counts (today only showcase's `semantics/assistive.rs`).
- a mouse event posted to a pid reaches a macOS window only where that
  window is on top; with the screen covered and the window kept from the
  focus, no posted click arrives, while a posted key does. the window
  proof of the shortcut goes through the function the click calls
  (`App::view_source_with`), over the page's frame and the workspace's
  literals, with the real opener running a recorder.
- the shell option `PLEV_BACKGROUND=1` (macOS) is the ide's (task-63d) in
  the showcase: prohibited activation policy, the window ordered front
  without becoming key and kept below every other window.
- phase c (writing a literal back) builds on this: it needs the byte
  range of the literal (`Span::byte_range` under the same feature), the
  file's skeleton checked on disk at the write against the one the
  position came from, the new text escaped for the literal's form (plain
  or raw), and a single `Literal` answer as the precondition. colors
  (hex literals) resolve the same way from the color of the node under
  the cursor.

## avoid

- guessing among several literals, or opening anything without an
  explicit click.
- a second hit test: a control is found by `Compositor::hit`, a text by
  the scene the frame drew.
- positions for a literal of test code or a doc comment: the app never
  draws them.
- `span-locations` reaching the `proc-macro2` of the proc macros, or the
  lexer's map left to grow on the watcher thread.
