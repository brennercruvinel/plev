---
id: adr-0038
title: "accessibility projects the semantic snapshot"
project: plev
type: adr
status: accepted
date: 2026-09-23
date_basis: header
recorded: 2026-09-23
topics: [accessibility, semantics, input]
relations: [adr-0032, adr-0033, adr-0035, task-59, task-61, task-62, task-75]
evidence: [263c00c]
source: docs/adr/interface/0038-accessibility-projects-the-semantic-snapshot.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# accessibility projects the semantic snapshot

Decides that the accessibility tree is a projection of the semantic snapshot, and no longer an estimate made from geometry: `rust/engine/src/control/accessibility` builds the AccessKit tree from the semantic snapshot of the `rust/accord` crate, from the frame's binding table and from the window. The old path, based on click regions and on `ViewId`, was removed, and a screen reader action becomes an `Assistive` actor on the same `Dispatcher`. No shipped app exposed a tree before; the showcase already wires the bridge on desktop, and ide and urnaui are next.

## context

the accessibility tree was guessed from geometry. with the `accessibility`
feature on and a screen reader listening, `engine::window::App::render`
turned every hit region into an AccessKit node: `Button` when focusable,
`GenericContainer` otherwise, never a label, a `NodeId` taken from the
per-frame `ViewId` (`view_id_to_node_id`, positional), a `FocusGraph` built
from the same regions. the action handler only logged, and the whole path
lived in the engine's own `App`, which no shipping app uses: the showcase,
the ide and urnaui have their own shells over `present_frame`, and none of
them created an adapter. no app that ships exposed a tree.

meanwhile the contract that explains an interface already existed: every
control has a `SemId`, a role, a label, its state and the actions it
accepts ([`0032-semantic-contract-as-pure-projection.md`](../semantics/0032-semantic-contract-as-pure-projection.md)), every action
goes through one `Dispatcher` ([`0033-actions-carry-observed-state.md`](../semantics/0033-actions-carry-observed-state.md)),
and the compositor knows where each id sits in the frame
([`0035-semantic-binding-beside-the-scene.md`](../semantics/0035-semantic-binding-beside-the-scene.md)). a structure good enough
for agents explains the interface to assistive technology too (thesis,
section 21), and assistive technology is one more actor, not a fifth path.

## decision

`rust/engine/src/control/accessibility` projects the semantic snapshot. the old
path is removed, not kept beside it: `AccessibilityState`, `id_map`, the
hit-region `FocusGraph` and the hit-region block in `window/render.rs` are
gone.

### the tree (`tree::build`)

a pure function of a `sem::Snapshot`, the frame's `Compositor` (its binding
table) and the window (`Chrome`: title, scale factor, inner size).

- the root is the window, labeled with the app's title; the snapshot's
  roots are its children. every node follows in tree order, children in
  projection order. only nodes reachable from the roots are emitted, so an
  update is always a valid full tree (it is sent whole every time).
- roles map one to one over `Role::ALL` (`role::of`), with three nearest
  counterparts where AccessKit has no exact one: `Toggle` is `Switch`,
  `Select` is `ComboBox` (a collapsed choice, as a native select is),
  `Media` is `EmbeddedObject` (the contract does not say audio or video).
  a secret text input is a `PasswordInput`.
- label and description are the node's. state reads by key: `text` of a
  text input, `value` of any node (a number with the node's range and
  step, or a text) and `selected` of a select are the value; the bool the
  node's own `toggle` action reads (else `checked` or `on`) is `toggled`,
  so a todo row's `completed` is heard as its state; a bool `expanded` is
  `expanded`; a bool `selected` is `selected`; a text `selected` on a
  container names the key of its selected child (`todo.filter` holds
  `all`, so `todo.filter[all]` is selected). `enabled: false` is disabled.
  a secret node has no value at all (the snapshot only holds its length).
- actions advertised are the ones the node can take: `Click` when it has a
  primary action, `Focus` when it is focusable and not inert, `SetValue`,
  `Increment` and `Decrement` when it declares `set_value`, `increment` and
  `decrement`.
- focus is the snapshot's `focused`, or the window.

### geometry: bounds from the binding, and what is not on screen

AccessKit wants physical pixels relative to the window's client area, y
down (checked in `accesskit_macos`, which divides by the backing scale
factor). a binding is logical, so its rect is clipped by the clip open at
the bind (the part a person can see and point at) and scaled by the
window's scale factor. a container nothing binds (`todo.filter`) takes the
union of its bound descendants.

a node without a binding this frame (scrolled out of the list,
virtualized, the counter nothing binds) stays in the tree without bounds.
AccessKit 0.24 has no offscreen state, and leaving the node out would make
its meaning depend on being drawn: its id, its label and its actions exist
whether or not the row is on screen, and a screen reader can still read
it and act on it through the dispatcher. a node with no bounds is how "not
on screen" reads.

an inert node (an open modal leaves the page inert) is hidden, as the
`inert` attribute is on the web: it keeps its id and is not presented. a
request that still reaches it (one queued before the modal opened) comes
back `Inert` from the dispatcher, the same answer a click gets.

### node ids (`ids`)

a `NodeId` is FNV-1a 64 of the `SemId`'s canonical text, computed by
hand: the same on every run and target, and the same for a control
whatever its layout, so a screen reader stays on the node it was on when
the window is resized or the list scrolls. two ids of one frame with one
hash are resolved deterministically and independently of projection
order: the smallest `SemId` keeps the plain hash, the others probe salted
hashes (the text, a `0xff` byte utf-8 never holds, the attempt) until
they reach a value nothing else in the frame claims; a counting fallback
ends the probe whatever the hash. `NodeId(0)` is the window's and no node
takes it. never two nodes with one id.

### requests (`action`)

an AccessKit `ActionRequest` is read back against what the screen reader
was shown (`Shown`: the snapshot of the last tree sent and its ids) and
becomes a `sem::Request` with `Actor::Assistive`, which the app sends
through its one dispatcher:

- `Click` asks for the node's primary action: the first of `activate`,
  `toggle`, `submit`, `select` that it declares without a parameter (a
  click carries none; a select's `select` takes one and is not a click).
  the todo field's primary action is `submit`: VO-Space on it adds the
  todo, as Enter does.
- `SetValue` asks for `set_value`, the value in the kind the action
  declares when it converts without loss (a number for a slider, a text
  for a field); what does not fit is refused by the dispatcher with the
  reason.
- `Increment` and `Decrement` ask for `increment` and `decrement`.
- `Focus` is not a capability. moving keyboard focus is editing, as a
  click into a field is for a person (semantic-binding-beside-the-scene:
  typing, caret and focus stay with the field), so it comes back as
  `Assist::Focus` and the app moves its own focus, the same hook Tab uses.
  only a focusable node that is not inert takes it. routing it through the
  dispatcher would refuse it everywhere today (no widget declares a
  `focus` action) and a screen reader could never reach a field to type.
- anything else is `Unmapped` with the reason, logged and dropped, as
  AccessKit asks of an unsupported request.

the request is observed over the snapshot the reader was shown, not over
a fresh projection: a person using a screen reader decides on what they
heard, so when that state moved since, the dispatcher answers `Stale` with
the delta, as for any actor. a confirmation the app shows is answered by a
person, and a person using a screen reader is one: the app routes a
request aimed at its confirmation to `Dispatcher::confirm` with the
request's actor, `Actor::Assistive` (tested with a toy app whose purge
asks `Confirm::Always`).

### keyboard order (`focus`)

Tab and Shift-Tab walk the snapshot (`focus::next`): the nodes that are
focusable, enabled and not inert, in the tree order the accessibility
tree presents, wrapping around, starting from the snapshot's focus (or
either end). a disabled control is skipped, as on the web. the order
replaces the hit-region `FocusGraph`, which this task removed; what focus
means for a widget stays the app's.

updated on 2026-09-23: no shell calls `focus::next` yet; its one caller
is a test (`apps/uigallery/src/views/apps/todo/semantics/assistive.rs`).
Tab in the showcase still follows each page's own cycle
(`apps/uigallery/src/keyboard.rs`), as the consequences below say.

### the bridge (`Bridge`)

one helper every app shell uses, native only (AccessKit has no browser
adapter; a wasm build does not compile it):

- `Bridge::new(event_loop, window, title, wake)` attaches the
  `accesskit_winit` adapter while the window is still hidden (a visible
  window is `BridgeError::WindowVisible`, never the adapter's panic); the
  shell creates the window with `with_visible(false)`, creates the bridge,
  then shows the window.
- the adapter's handlers run on whatever thread the platform uses, so they
  never touch the app. an action request goes into a channel and `wake`
  runs, which the shell makes send its own event through its
  `EventLoopProxy` (the pattern of worker-results-wake-the-event-loop); on
  the main thread the shell calls `take` and carries each `Assist` out.
  activation also wakes the app and returns no tree: the platform shows
  its placeholder until the next frame sends the full tree.
- `update(window, compositor, project)` runs after the frame's scene is
  built: the adapter calls back only while an assistive client listens,
  and only then is the app projected and the tree built.

the showcase shell wires it (desktop): the window is titled `plev, design
system showcase` and that title labels the root; the view projects the
active page when it has a projection (`ShowcaseView::semantics`, the todo
page today), and a screen reader's request goes through that page's
dispatcher (`ShowcaseView::act`, named `assist` until task-61), its
focus to the page's field (`ShowcaseView::assist_focus`). a page without
a projection has no tree beyond the window yet; that is expected, not a
fallback. the engine's own `App` uses the same bridge with no projection
(the window alone).

## cost

measured on the todo page (`tree_cost_per_frame` in
`apps/uigallery/src/views/apps/todo/semantics/assistive.rs`, release, apple
m4, rustc 1.98.1, best of seven rounds of 200 frames, two runs while other
builds shared the machine):

| items | projection and tree | tree alone | nodes |
| --- | --- | --- | --- |
| 3 | 7.3 to 8.2 us | 2.8 to 3.1 us | 11 |
| 103 | 78 to 102 us | 22 to 26 us | 111 |

the tree costs about 0.2 us per node (ids, role, state, the union of
bounds); projection is the larger half and grows with the items the page
projects, as task-59 measured. both run only while an assistive client
listens. no target was set.

## consequences

- every page that projects a snapshot is accessible in the showcase with
  no accessibility code of its own; each page that gains a projection
  (task-59 phase b widgets, the other pages) gains a tree. the chrome
  (sidebar, header, overlays, the open modal) is not projected yet, so
  with a modal open the reader hears the window and a hidden page.
- the ide wires the same bridge after task-62 lands; urnaui is next.
  mobile shells are not wired (`accesskit_winit` has no ios adapter and
  its android one is behind a feature the engine does not turn on).
- Tab in the showcase still follows each page's own cycle; the pages move
  to `focus::next` over their snapshot when their key handling is next
  touched (the todo page has no Tab cycle today).
- only the primary action is reachable by a click. the others (a row's
  `remove`) wait for AccessKit custom actions, and expand and collapse for
  a mapping to `open`, `close` and `toggle_expanded`; live regions for
  `Status` and `Alert` are task-75.
- a request on a node the reader was shown that is gone now is refused
  by the dispatcher (`NotFound`), never run on a neighbour: a node id is
  never a reused position.
- `engine::window::App::render` keeps its own frame tail: it composites
  per-layer effects (`apply_layer_effects`) between the layer passes and
  the composite pass, and `window::draw_frame` composites with none.
  folding it into `draw_frame` means moving the effects into the frame
  path every app and the headless renderer share, its own change.
- `Theme::intent_role` (an `Intent` to an AccessKit role guess) had no
  caller once `AccessibilityState` left, and was removed (commit
  `263c00c`).

## avoid

- a node, a role or a label taken from geometry, a hit region or a
  `ViewId`.
- a `NodeId` from a position, an emission order or a counter.
- an accessibility handler that calls app logic: the handler queues, the
  shell dispatches.
- building the tree, or projecting for it, while no assistive client
  listens.
- a second accessibility path next to the bridge.
