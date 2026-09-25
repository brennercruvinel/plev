---
type: adr
status: accepted
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

## phase c: a literal written back (2026-09-25)

task-65 phase c: editing a text in the window writes its literal in the
`.rs`, and the result is a normal `git diff` of one string. what was
decided:

1. the write is the engine's, under `hot_reload`:
   `literals::write(site, from, to, by)`. it takes the site a click
   found (a `sites_of` answer) and the text the literal held, and finds
   the literal's slot (its index in the file's literal order) in the
   sources the click read: the last save that kept the skeleton, else the
   baseline; never a doc comment or test literal.
2. the file on disk must still have the baseline's code: the same
   skeleton hash, as many literals, and every literal that is not a
   string or a hex color as it was. this is the test `apply_change`
   already made (now one function, `same_code`), so "the same skeleton"
   means one thing for the hot reload and for the write. the running
   binary was built from that code; a slot names the same literal only
   while it holds. otherwise `Refused::Code`: rebuild, then edit. the slot
   must also still hold `from` on disk (`Refused::Moved`: a save from the
   editor the watcher has not seen yet).
3. the bytes are read at the write, not kept: the disk text is lexed once
   more and the slot's `Span::byte_range` taken (same `span-locations`
   feature); the token found there must unquote to `from`. the baseline
   keeps no byte ranges, so its memory did not change.
4. exactly the token is replaced. a plain string escapes the quote, the
   backslash, `\n`, `\r`, `\t`, `\0` and any other control character
   as `\u{..}`; every other character, unicode included, is written as
   typed. a raw string stays raw, with as many `#` as the new text needs
   and never fewer than it had (a bare carriage return cannot be raw: it
   goes plain).
5. the file is replaced at once: a dot file beside it (the watcher skips
   dot files), the file's permissions, synced, renamed over it; a failure
   removes the dot file and leaves the file as it was. then the write
   calls `apply_change` itself, so the next frame shows the text without
   waiting for the watcher's debounce; the watcher's own event finds the
   same delta later.
6. the person's alone. the edit is no capability: nothing of it is
   projected or goes through the dispatcher, so the agent port cannot
   name it, and the mirror's hand acts through the dispatcher only. the
   write itself takes the actor and refuses `Actor::Agent` before reading
   anything, so a later caller on the port's path is refused too. every
   refusal leaves the file untouched.
7. the shell (the showcase): cmd+shift+click (ctrl+shift off macOS) on a
   text `origin_at` resolves to one literal opens a
   `comps::form::TextField` over the text run (`view_source::text_run_at`
   gives its box), on a layer above every layer of the frame, from the
   field's inset before the text to the window's right edge, the page
   ground under the text. while it shows it owns the keyboard (the first
   hook of `KeyboardInput`): Enter writes, Escape drops it, a press
   outside drops it and reaches the page. a control, several literals or
   a formatted text are told in a toast; so is a refusal. a toast and the
   log, never the shell's notice, which an agent reads.
8. the same gate as phases a and b: release, wasm and mobile compile none
   of it. checked on the check's metadata: the new names (`write_beside`,
   `text_run_at`, `same_code`, `ViewSource`, `not_one_literal`) are in
   the debug metadata and absent from the release and wasm32 ones.

consequences of phase c:

- an edit on one line is a one-line `git diff`, the literal's line.
- the field draws in the field's body style, not the text's (a title
  shows smaller while edited), and stays where the text was drawn: a
  scroll under it does not move it.
- the new text shows wherever the old one is drawn (the override's find
  and replace, as for any save), until the next build.
- a watch of the window (`PLEV_MIRROR`) sees the field as pixels, like
  anything drawn; nothing of it is in the projection.
- the color picker for hex literals and phase d (who read this token)
  are not done.

## avoid

- guessing among several literals, or opening anything without an
  explicit click.
- a second hit test: a control is found by `Compositor::hit`, a text by
  the scene the frame drew.
- positions for a literal of test code or a doc comment: the app never
  draws them.
- `span-locations` reaching the `proc-macro2` of the proc macros, or the
  lexer's map left to grow on the watcher thread.
- a write through the dispatcher, or the edit projected as a capability.
- a write that skips the code check, or finds its literal by text alone.
- byte ranges kept in the baseline: they are read at the write.
