---
type: how-to
tags: [plev, engine, widgets, layout, text, color, semantics, contract, checklist]
date: 2026-09-23
status: living
---

# how to code against the plev engine

the operating manual for anyone (human or agent) writing ui or rendering
code on plev. every rule in the engine sections was violated at least once
in this repository, shipped a visible defect, and was root-caused; treat a
violation as a defect even when the screen looks right.

plev is moving from a GPU UI engine to a generative interaction runtime
(docs/mission/steps/ongoing/generative-runtime-program.md). the direction
adds one layer on top of everything below: every control keeps a
persistent identity and meaning while its pixels are born, disappear or
transform. the section "the semantic contract" says which parts of that
layer exist today and which task brings the rest; this manual is updated
in the close-out of each of those tasks.

the mantra every change is judged against:

> a capability keeps identity and semantics while its visual
> representation is born, disappears or transforms in real time.

> invalidation propagates minimally through state, semantics, layout,
> scene construction, hit testing and rendering, independent of the visual
> representation.

## the official app pattern

state lives in plain structs, ui is built from retained widgets (`comps`,
the design system crate: `comps::prelude::*`), and every visible mutation
invalidates (`EventResult::changed` / request redraw). the showcase is the
template app. the declarative builder (`engine::builder`) is supported for
prototypes and demos; `view`, `component`, `signal` and `narrate` are
experimental: do not start app code on them (docs/ADR/official-app-pattern.md).

task-69 adds a second supported pattern (surfaces materialized from
verified plev IR); until its ADR lands, the pattern above is the only one.

## the semantic contract

what the thesis calls "the contract that matters": who am i, what do i
mean, what is my state, which actions do i accept, which conditions make
them valid, who may run them, what happened when they ran.

what exists (updated at each task close-out):

- `crates/sem` (task-57, adr semantic-contract-as-pure-projection): the
  contract as pure data. `SemId`, `Role`, `Value`, `StateRef`,
  `Constraints`, `ActionDecl` with its `reads` and `Policy`, `Node`, the
  `Semantics` trait, `SnapshotBuilder` (text or typed ids, every
  violation reported by `finish`), `Snapshot` with a stable `Revision`
  and `Snapshot::diff`. nothing in the engine, comps or the apps uses it
  yet.
- the dispatcher, also in `crates/sem` (task-58, adr
  actions-carry-observed-state): `Request::observe` records the values the
  actor saw for the action's `reads`; `check` decides in a fixed order and
  goes `Stale` only when one of those values moved, with the delta; one
  `Dispatcher` for every actor projects the app at dispatch, runs
  `Operable::execute(id, req)` only after acceptance or a person's
  confirmation (`Actor::Human` or `Actor::Assistive`), and keeps a
  `Receipt` whose stages sinks see as they happen. `cargo run -p sem
  --example commit` is the thesis scenario in thirty lines of app.
- the binding table (task-59, adr semantic-binding-beside-the-scene): a
  widget with a `SemId` calls `Compositor::bind` (or `bind_to_layer` on
  the layer it draws into) with the rect it draws and hit-tests with;
  the table sits beside the scene, never in a layer hash, and
  `Compositor::hit(x, y)` answers which capability is under a point.
  every interactive comps widget takes `.sem(id)` and has a `project`
  helper that declares its role, state and actions (children as keys
  under its id); an app gives a widget an id, calls its helper in
  `Semantics::project`, and the render binds it.
- the first app on the one path is the showcase todo page
  (`crates/showcase/src/view/apps/app/semantics.rs`): `Semantics +
  Operable`, owner of a `Dispatcher`; every click, the Enter key and the
  filter pills become `Request::observe(.., Actor::Human, ..)`; the model
  changes only inside `execute`; while an overlay holds the pointer the
  view calls `set_inert(true)` and every request comes back `Inert`. copy
  this shape.
- the second app on the one path is the git client (task-62):
  `ide::workbench::Workbench` owns the view, the git session and the
  dispatcher, for the ide binary and the showcase Git page alike. it shows
  the asynchronous shape: `execute` sends the command to the worker and
  returns `Execution::Started`, and the worker's answer ends the receipt
  (`complete`, `fail`, `reject_at_truth`). a precondition whose truth
  lives outside the ui is checked again there (`GitCommand::CommitIf`).
- headless (task-66, adr headless-render-shares-the-frame-path):
  `GpuContext::new_headless` and `window::draw_frame` draw any frame into
  a texture through the window's path; `showcase --render` and
  `monster_player --frame` use it.
- the only identity the engine itself has is still `ViewId`, a per-frame
  counter reset in `InputState::begin_frame`
  (crates/engine/src/input/mod.rs). it is a temporary render identity,
  never a control's identity. do not persist it, do not send it to an
  agent, do not record it.
- accessibility (task-60, adr accessibility-projects-the-semantic-snapshot):
  the AccessKit tree is built from the page's `Snapshot` and the
  compositor's bindings (`engine::accessibility::tree::build`), and a
  screen reader's action comes back as a `Request` with
  `Actor::Assistive` for the same dispatcher. a shell attaches
  `accessibility::Bridge` before its window is visible. a page with no
  projection has no tree: projecting it is the fix, never a fallback.
- agents from outside (task-61, adr agents-act-through-the-semantic-contract):
  `wire::AgentPort` opens a local socket (off unless `PLEV_AGENT=1` in a
  debug build or an explicit `open`); the shell drains it on the ui
  thread through a small `Host` (project the page, dispatch a request,
  register a sink). try it: `PLEV_AGENT=1 cargo run -p showcase -- Todo`,
  then `cargo run -p wire --bin plev -- observe --scope todo` and
  `plev act todo.item[1] toggle`.

how to project (from the crate docs):

```rust
impl sem::Semantics for GitState {
    fn project(&self, b: &mut sem::SnapshotBuilder) {
        b.node("changes", sem::Role::List, "changes")
            .state("staged", self.staged.clone());
        b.node("commit.message", sem::Role::TextInput, "commit message")
            .state("text", self.message.as_str());
        b.node("commit.submit", sem::Role::Button, "commit")
            .enabled(!self.staged.is_empty())
            .action("submit")
            .reads("changes:staged")
            .reads("commit.message:text");
    }
}
```

project what is operable and meaningful, not the whole content: a big
document is one node with its size, selection and cursor, not a node per
line.

as the tasks land, these rules become binding (each line names its task):

- identity is chosen, never counted. a control's `SemId` comes from the
  app or the IR (`commit.submit`, `changes.file[src/lib.rs]`), never from a
  position or an emission order (task-57).
- semantics is a pure projection of app state: the app implements
  `sem::Semantics::project`; nothing semantic accumulates inside render
  (task-57).
- what is not projected does not exist for the contract. hover,
  animation and scroll position are not projected, so they can never
  invalidate an action (task-57, task-58).
- every action declares the state it reads; a request carries the values
  its actor observed; the dispatcher checks them against the current
  state and rejects stale requests with the delta (task-58). every action
  reads at least the state it changes, absolute setters included, so no
  actor overwrites a value it never saw (task-59).
- one path: a click, a keystroke, a screen reader action, an agent and a
  test all become a `sem::Request` and go through the same `Dispatcher`
  into the same `Operable::execute`. a click handler that calls app logic
  directly is a second path and a defect once task-59 lands (task-59).
- when the truth lives outside the ui process (the git index, a server),
  the precondition travels there and is checked again right before the
  effect (task-58, task-62).
- a widget with a `SemId` binds its rect with `Compositor::bind`, the
  same rect it draws and hit-tests with; bindings never touch layer hashes
  (task-59).
- a field marked `secret` never leaves the process: not in snapshots, not
  in recordings, not to agents (task-57). whatever carries a request or a
  stage out of the process reduces what aims at a secret node with
  `sem::redact`, the one rule; never write a second one (task-57b).
- the sem serde form is lean: a field at its default is not written and
  reads back when absent; `source` travels only when the node has one and
  leaves an agent's observation unless asked (`plev observe --source`).
  pin a form in a test by its lean text, and let an old text read
  (task-57b).
- replay never executes: nothing that reads a session creates a request
  (task-63).

## tokens, never literals

every widget takes `&Theme` and reads colors from `theme.colors` and
`theme.glass`, radii from `theme.shape` (by role: card, pill, nav,
tooltip, micro), heights and paddings from `theme.control`
(`ControlSize::Xs..Xl`, icon sizes, rims, focus ring, toggle geometry),
container widths from `theme.size`, timed transitions from
`theme.duration`, breakpoints and gutters from `theme.layout`, shadow
stacks from `theme.shadows` (by `Elevation`), and text styles from the
`theme.typography` ramp. a widget that lays out from tokens takes `&Theme`
on `handle_event` too, so the hit test and the pixels come from one
geometry. a number a screen needs that is not a token yet becomes one in
`crates/engine/src/theme/scales.rs`, with its spec value in
`tests_scales.rs`; it never lands in the screen.

an app frames its screens with `comps::shell::AppShell` (full sidebar,
rail, or a drawer behind a menu button on a phone; header; content rect;
safe area). the only tokens an app may own are domain semantics derived
from the theme (the ide's file-status colors).

every comps widget renders under every built-in theme at a phone and a
desktop width (`crates/comps/src/tests/matrix.rs`); a new widget joins
that matrix in the same change, and ships its showcase page.

## before writing anything

check whether the engine or comps already provides the capability. the
most expensive failure pattern in this project was app-layer
reimplementation of an existing feature (a second theme, five copied
widgets, per-app ellipsis helpers). grep `crates/engine/src` and
`crates/comps/src` first, read docs/catalog.md, and find one screen that
already does it right (the showcase pages).

the same rule holds for the new layer: one `sem` crate, one dispatcher,
one binding table, one accessibility projection. never a local copy.

## text

- one `TextStyle` per text run, used for both
  `TextMeasurer::measure_styled` and `TextNodeKey::from_style`. never
  construct measurement and drawing parameters separately (ADR
  one-text-style-for-measurement-and-drawing). a guard test fails the
  build on a raw key.
- never estimate text width arithmetically. `TextMeasurer` is GPU free and
  cached.
- a family and weight pair resolves only if that exact face is embedded
  (`crates/engine/src/text/fonts.rs`: Inclusive Sans 300 to 700, Inter 400
  to 600, codicons). a new weight is embedded first, then added to the
  face-resolution test. scripts without an embedded face (arabic, cjk,
  color emoji) fall back and warn once; task-74 decides the fix.
- letter spacing, weight and line height all change advance; a new
  typography attribute is plumbed through measurement and drawing in the
  same commit.
- single-line truncation is `TextMeasurer::truncate_to_width(text, style,
  max_width)`; paths elide by segment with `TextMeasurer::elide_path`.

## small app utilities

- clipboard: `engine::clipboard` (`SystemClipboard` on desktop,
  `LocalClipboard` for tests, wasm and mobile, `default_clipboard()` picks
  per target).
- charts: `comps::charts` (pure geometry plus `charts::draw`); the owning
  view keeps any reveal tween. network canvases: `engine::graph` plus
  `comps::graph::GraphView`.
- text fields: `comps::form::TextField` over the `TextInput` core; the
  editor: `comps::editor::EditorView` with `EditorConfig::from_theme`.
- simple global shortcuts: `engine::actions::shortcuts::ShortcutMap` with
  `match_logical_key`; the full `KeymapMatcher` and `ActionRegistry` only
  for contexts, sequences or user keymaps. a keymap binding is an input
  binding; once task-59 lands it targets a capability and goes through the
  dispatcher.

## color

- hex and theme values are srgb. anything entering GPU memory is linear:
  `to_linear_array()` for clear colors and uniforms; vertex colors are
  linearized in-shader.
- surface render targets come only from `gpu.surface_render_view(&output)`;
  a default `create_view` skips gamma on the web target.
- inside a shader, every `mix` between two colors happens in premultiplied
  linear space, never in straight alpha (the dark corner fringe fixed
  2026-09-21 came from that).
- a shadow with blur 0 is still anti-aliased: `shadow_sigma` floors at
  `MIN_SHADOW_SIGMA`.
- validate color by sampling pixels (docs/how-to/validate-visuals-by-pixel.md);
  the dark background measures 48,48,48 on desktop and web.

## layout and responsiveness

- container geometry derives from `content.w` or from `LayoutEngine`;
  constants only as min, max and gap.
- grids: `cols = floor((content.w + gap) / (min_w + gap)).max(1)`, then
  stretch the column width to fill, clamped by a readability maximum.
- intrinsic rows wrap against `content.w`.
- user-adjusted sizes store desired separately from effective; clamp at
  read time.
- keep `layout()` functions pure; write viewport tests at an explicit
  narrow and wide width for every new screen.
- a control moving between layouts keeps its `SemId`; only its binding
  rect changes (task-59).

## events and invalidation

- any handler that changes visible state invalidates (return true or
  request redraw). under render on demand a missed invalidation is a
  frozen app.
- clip rects are logical; scale by `gpu.clip_scale()` for physical
  scissors (HiDPI).
- after `gpu.resize`, reapply the logical projection (`set_projection`);
  standalone event loops do it themselves.
- a cache added for incrementality (task-73) ships with an invalidation
  test: change key X, only what depends on X is rebuilt, and the pixels
  match a full rebuild.

## platform

- no blocking executor on any path reachable from wasm; GPU init follows
  the `spawn_local` plus `EventLoopProxy` pattern (ADR
  async-gpu-init-and-single-wasm-entry).
- touch arrives as synthesized pointer events; multi-finger gestures come
  from the recognizer.
- work on a background thread wakes the loop through `EventLoopProxy`
  (ADR worker-results-wake-the-event-loop); the agent socket (task-61)
  and the accessibility action handler (task-60) follow the same pattern.
- `cargo check --target wasm32-unknown-unknown -p showcase` stays green;
  a crate that the engine depends on (`sem`) must compile for wasm too.

## definition of done for ui work

1. workspace tests green, including regression tests for the change
2. wasm check green
3. the affected screen sampled by pixel at two window widths
4. no new constant that encodes a viewport assumption
5. measurement and drawing share one style object for any text touched
6. once task-59 has landed: every interactive control on the screen has a
   `SemId`, projects role, label, state and actions, and its click goes
   through the dispatcher; a test drives the same capability by click and
   by `Actor::Test` and gets the same state
7. once task-60 has landed: the screen's accessibility tree is asserted in
   a test (roles, labels, order)
