---
type: adr
status: accepted
tags: [sem, binding, compositor, hit-test, dispatcher, one-path, showcase, generative-runtime]
date: 2026-09-23
---

# semantic binding beside the scene, and the click as a request

## addendum (2026-09-23, task-59 phase b)

19 interactive comps widgets now carry an optional `SemId`, bind the
rect they draw and hit-test with, and project themselves through a
`project` helper (16 widgets beyond the three of phase a; the matrix
covers them all). seven widgets that handle events carry none:
`Scrollbar`, `Card`, `GraphView`, `AppShell`, `EmptyState`, `Sidebar`
and `EditorView`. one rule was settled on the way: every action reads the
state it changes, absolute setters included (`select`, `set_value`,
`set_ratio`, `open`, `close`, a tab's `select`). an actor that decided on
a value another actor has since changed gets `Stale` with the delta
instead of silently overwriting it; a person clicking pays nothing, since
the click observes the page at that instant. phase b also fixed a
pre-existing mismatch in `Select`: the hit test picked a row the panel did
not draw, which is the one-geometry rule this record exists for.

## context

task-57 gave every control a logical identity and a projected meaning
(`docs/ADR/semantic-contract-as-pure-projection.md`), task-58 the one
dispatcher that checks a request against the live state
(`docs/ADR/actions-carry-observed-state.md`). nothing tied either to the
pixels. the only identity the engine has is `ViewId`, a per-frame counter
(`crates/engine/src/input/mod.rs`); the comps widgets hit-test with
`Rect::contains` on their own; and every click in the showcase todo page
called the model directly (`set_filter`, `toggle`, `delete`, `add`), a
second path next to the dispatcher before the first one had a user.

two things were missing: where a capability sits in the frame on screen,
and a click that reaches the capability the way an agent, assistive
technology or a test does.

## decision

### the binding table

- the `Compositor` keeps, per frame, `SemId -> Binding { rect, layer,
  clip, order, view }` in logical window pixels
  (`crates/engine/src/compositor/binding.rs`). `begin_frame` clears it;
  between two frames it answers for the frame on screen, which is what a
  person clicking saw.
- it lives beside the layers, never in them: a binding is not a scene
  node, is not hashed, and binding an id or not can never dirty a layer
  or change a pixel (tested against the layer hash and `needs_render`).
- the layer is named the way drawing already names it: `bind(id, rect)`
  targets the default layer like `push`, `bind_to_layer(layer, id, rect)`
  like `push_to_layer`. the compositor has no "current layer", and
  guessing one from the last push would be wrong for a widget that binds
  before it draws.
- the clip is read, not tracked: at a bind, the table scans the target
  layer's `PushClip`/`PopClip` nodes pushed since its previous bind in
  that layer (one cursor per layer, the existing `ClipStack`), so each
  node is read at most once per frame and no drawing path changed.
- `hit(x, y)` answers the topmost bound id: rect and clip hold the
  point, the layer is visible with an opacity above zero, and the order
  is layer z-order, then layer creation, then bind order, the order the
  frame composites in. edges are inclusive, like `Rect::contains`.
- an id binds once per frame. a second bind is logged once per id and
  ignored; the first binding holds. nothing panics.
- `view` keeps the `ViewId` when a widget also registered a hit region
  (`set_binding_view`). no comps widget does today, so it stays `None`
  there. `ViewId` itself did not change.

### widgets

`TextField`, `Checkbox` and `Button` take an optional `SemId`
(`.sem(id)`); with one, `render`/`render_to_layer` bind the same rect
they draw and hit-test with. each has a projection helper that reads the
widget the app owns and returns the node for the app to add to:
`Checkbox::project` (checkbox, `checked`, `toggle` reading it),
`TextField::project` (text input, `text`, editable, `set_value` with a
constrained text), `Button::project` (button named by its label,
`activate`). the helpers are `#[track_caller]`, so a node's source is the
app's projection site. the other interactive widgets follow in phase b.

### the click becomes a request

the todo page (`crates/showcase/src/view/apps/app.rs`, `app/semantics.rs`)
is the reference. the page is `Semantics + Operable` and owns a
`Dispatcher`; each capability has one handler, inside `execute`. a click
on a checkbox, a delete button or a filter pill, and the Enter key, go:

1. hit-test with the page's own geometry, the rect it also binds;
2. project the page now and `Request::observe(&now, Actor::Human, id,
   action, param)`;
3. `dispatch`, stamped with a monotonic clock the page keeps
   (`web_time::Instant`; `sem` never reads one);
4. ask for a frame when the projected revision moved. a request that did
   nothing (a re-click on the active filter) or was refused redraws
   nothing and still counts as handled.

a test drives the same capabilities by click and by `Actor::Test` and
compares the model and the snapshot. the snapshot a person observes is
projected at the click, so a person's request is never stale; an agent's
or a test's goes stale when the text it read moved, and never when only
the caret, the hover or the focus did.

typing, caret moves and focus stay with the field. they are editing, not
a capability: `set_value` replaces the text for an actor that has no
caret, and a keystroke is not turned into one.

### an overlay leaves the background inert

the view opens the overlays, so the view tells the page. the showcase
view calls `TodoPage::set_inert(overlay_holds_pointer())` right before it
routes a pointer event, the Enter key or a render to the page
(`overlay_holds_pointer` is the condition the pointer chain already used:
a modal or a menu that is not fading out). while inert the page projects
`todo` inert, so every request to it or a descendant comes back `Inert`,
the same answer the click gets by never passing; keys fall through to the
chrome and the overlay.

## cost

measured on the todo page (`semantic_cost_per_frame` in
`crates/showcase/src/view/apps/app/tests/semantics.rs`, release, apple
m4, rustc 1.98.1, best of seven rounds of 200 frames):

| items | projection and finish | render with bindings | render without | bindings |
| --- | --- | --- | --- | --- |
| 3 | 3.7 to 5.4 us | 3.2 to 3.3 us | 2.8 to 2.9 us | 8 |
| 103 | 49 to 50 us | 7.3 to 7.6 us | 6.3 to 6.6 us | 18 |

binding costs about 50 to 60 ns per bound id, most of it the id of each
row built for the frame. projection grows with the items projected
(about half a microsecond each here: ids, labels and the validation in
`finish`), not with what is on screen. no target was set; these are the
numbers task-60 (projection every frame) and task-73 start from.

## consequences

- task-60 builds the accessibility tree from the snapshot and places it
  with `bindings()`; task-65 resolves a cmd+click with `hit`; task-62
  routes the ide commit the way the todo routes its clicks.
- a binding names only what was drawn: a row scrolled out of the list is
  not bound and stays in the snapshot.
- `hit` sees only bound ids, so an unbound draw on top (a modal in phase
  a, the sidebar drawer) does not hide what is bound beneath it. the
  inert flag, not `hit`, is what refuses the covered page until every
  overlay is bound.
- the layer-level scissor (`Layer::clip_rect`, physical pixels) is not
  read by `hit`: the compositor does not know the scale factor. clips
  pushed as scene nodes are.
- the keymap bridge (a key binding naming a capability) is left to
  task-62; the todo's Enter already goes through the dispatcher.

## avoid

- a binding as a scene node, or any semantic data in a layer hash or a
  GPU primitive.
- a click handler that calls the model: the handler is `execute`.
- a second rect for the binding: bind the rect the widget draws and
  hit-tests with.
- persisting, sending or recording a `ViewId`.
