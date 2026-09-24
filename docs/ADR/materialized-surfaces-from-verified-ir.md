---
type: adr
status: proposed
tags: [surface, uir, plev-ir, materialization, layout, theme, comps, sem, dispatcher, generative-runtime]
date: 2026-09-24
---

# a surface materializes only from verified IR, projects from the IR and asks through the app's dispatcher

## context

task-68 gave the plev IR (`crates/uir`): a surface described by meaning,
a registry of what an app can do built from its own projection
(`Capabilities::from_snapshot`), and `verify`, the only way to a
`VerifiedIr` (docs/ADR/plev-ir-typed-and-verified.md). the runtime could
refuse an IR, but it could not show one: nothing turned a `VerifiedIr`
into something a person sees and operates, or an agent observes.

what existed to build it from:

- the official app pattern, state in structs with retained widgets
  (docs/ADR/official-app-pattern.md), and the comps widgets, 19 of them
  with an optional `SemId` that binds the rect they draw and hit-test
  with (docs/ADR/semantic-binding-beside-the-scene.md);
- the engine's `LayoutEngine` (taffy behind `LayoutStyle`, flex only)
  and every visual value as a `Theme` token;
- the semantic contract and its one `Dispatcher` for every actor
  (docs/ADR/semantic-contract-as-pure-projection.md,
  docs/ADR/actions-carry-observed-state.md), and the rule that an act
  rests on what its actor was shown
  (docs/ADR/an-act-rests-on-what-its-actor-was-shown.md).

task-69 phase 1 asked for the materializer: layout by the engine, tokens
by the theme, drawing by the comps widgets, semantics by construction,
every event a request through the app's dispatcher, nothing executed by
the surface itself. the orchestrator fixed the place (a crate of its own)
and that the projection comes from the IR, not from the widgets' own
`project` (9 of the 21 components of the vocabulary have none).

## decision

### a crate of its own

`crates/surface` depends on `uir`, `sem`, `engine` and `comps`. `comps`
stays a widget library and knows nothing of the IR; `uir` stays pure.

### only a verified IR materializes

`Surface::materialize(VerifiedIr, &Snapshot)` is the way in, and no
function of the crate takes an `Ir` (a `compile_fail` doctest shows an
`Ir` does not type-check there). `SurfaceHost` keeps the surfaces an app
has open, refuses a second surface with the same id (changing an open
surface is a transformation, phase 2) and one whose nodes would take the
ids of another open surface's.

### the IR over the live state

the tree unrolls over the app's `Snapshot`: a repeat once per item of the
live value of its `List` or `Set`, the item keyed into every id and every
binding of the template (`review.file[{item}].stage` over `src/lib.rs`
is `review.file[src/lib.rs].stage`). an item a `List` holds twice gives
one instance, the first; a value that is gone, or of another kind now,
gives none. every node gets one retained comps widget, indexed by its
`SemId`. `sync` reads the bindings again only when the app's revision
moved: a node that stays keeps its widget (hover, focus, the caret, a
knob in flight), a new node gets one, a node that left drops it. the
snapshot `sync` read is the one the surface shows.

### each component on its widget

| component | drawn by | binds its rect |
| --- | --- | --- |
| container (no component), `list` | nothing of its own: `background` and `shape` as a `rounded_rect`, `elevation` as the `shadow_stack` | the host |
| `card` | `comps::content::Panel`, the card shell (the comps `Card` is a preview card and holds no children) | the host |
| `text` | a text run measured and drawn with one style (`typography`, `color`) | the host |
| `button` | `Button` (`SubmitButton` when `busy` is bound) | itself, `.sem(id)` |
| `icon_button`, `text_field`, `checkbox`, `switch`, `slider`, `chip` | their comps widget | itself |
| `select`, `tabs` | `Select` (its dropdown drawn last, on top), `Tabs`, over the bound `options` | the host (their own binding names options by index) |
| `table` | `Table`, one column of the bound `rows` | the host |
| `stat`, `badge`, `separator`, `empty_state`, `progress` | their comps widget | the host (a separator has no role and binds nothing) |
| `chart` | `comps::charts::draw`, over the numbers of the bound `data` | the host |
| `dialog` | the `Panel` shell at the overlay elevation, its title, its children, and a cancel and a confirm button | the host |
| `image` | the card shell with its label: the IR names a source, the engine takes a GPU image | the host |

a widget shows what the app holds: the bound state, and whether its
capability can run (`ActionDecl::enabled`). a capability disabled or gone
leaves the control disabled, so a stale surface fails safe.

### layout by the engine, tokens by the theme

no second layout engine:

- `stack` is a flex row or column with its gap and alignment (the cross
  axis, as the IR defines it); a node without a layout is a column;
- `grid` is a wrapping row whose children start at `min_col` and grow:
  as many columns as fit, none narrower than `min_col`. unlike a CSS
  grid, the last row, when it is not full, stretches its children;
- `split` gives its two children the ratio as flex weights on a zero
  basis;
- `overlay`: the `LayoutStyle` has no absolute position, so the container
  lays out in its parent's flow with its first layer, and every other
  layer is a pass of its own of the same `LayoutEngine` over the
  container's rect;
- `size.min` and `size.max` bound the dimension the token's name says
  (`dropdown_max_h` is the one height); `size.flex` is the flex weight;
  `padding` insets a container;
- a leaf takes the size its widget measures (`preferred_size`, the
  field's height, a text run by the engine's measure function).

every token reads the field of the `Theme` with its name, through one
exhaustive `match` per kind, so a token `uir` adds without a home in the
theme stops the build in this crate. the one approximation: a button does
not read `theme.control`, its heights are its own ladder (`ButtonSize`,
28, 36, 40, 44 in hoff core). the `control_size` of a button is the
tallest step not taller than `theme.control.height` of the token, so
`xs` (36) is `Sm` and `ButtonSize::Xs` is out of the IR's reach.

### semantic by construction

the surface projects from the IR, into the builder that holds the app's
own projection: one snapshot, one contract.

- the surface is a `Region` named by its title, its tree under it;
- each node takes the role `uir::role_of` gives, its label (an instance
  is named by its item; a text or a badge by what it shows), its
  description and its bound state as the snapshot shown holds it; a
  control that sends a value takes the constraints of the node it sends
  it to;
- each bound event is an action of the node, named by the event
  (`activate`, `set_value`, `submit`, `toggle`, `select`, `confirm`,
  `cancel`), with the parameter, the policy and the `enabled` of the
  capability. its reads are the capability's, named by the surface's own
  state where a node of the surface binds it (the add of the todo surface
  reads `todo_panel.input:text` where `submit` reads `todo.input:text`),
  by the app's state otherwise;
- a node with the id of an app node (a fixed screen written as IR) is the
  app's: the app declares it, with the actions the IR may bind on it;
- a separator is left out; a closed dialog is projected, its subtree is
  not.

### every event is a request through the app's dispatcher

- a click completed on a widget, or a key in a field of the surface, is
  the event the component sends, and the surface asks for the capability
  the IR bound: `Request::observe` over the snapshot the surface shows,
  by `Actor::Human`, handed to the app through the `Dispatch` trait
  (the app's one dispatcher and its projection). the surface reads the
  app again after it, and never calls `execute`;
- each edit of a field is its `set_value` with the new text (the text is
  the app's, and the surface has no other way to it); Enter is its
  `submit`, when bound. caret, selection and focus stay in the widget;
- `route` turns an actor's act on a surface node into the capability the
  node is bound to, by the same actor with the same parameter, its
  observation named back by the app's state. the page's dispatcher is
  the app's, so the receipt is the app's too;
- the rule of observation holds: a person acts on what the surface
  showed (a click on state another actor moved and the surface has not
  drawn yet is stale, with the delta), and an agent on what it was shown
  (an act whose capability reads state the surface does not show needs
  that state observed: from the surface's scope alone the port answers
  `not_observed`).

### render on demand

the surface draws when its page draws and never on its own. at rest it
asks for no frame; a focused field asks for its caret's next flip, a
switch for its knob while it moves. the page projects the app at each
frame it draws and the surface re-reads its bindings only when the
revision moved; the layout is computed at each frame drawn.

### the demo

the showcase's Apps Demo has a Surface page: the todo's IR fixture of
task-68 verified against what the todo projects and materialized beside
the todo written by hand, both over the todo's one dispatcher. the page
projects the todo and the surface as one snapshot and routes an agent's
act on a surface node to the capability, through the agent port's hooks
like any page.

## cost

`VerifiedIr` to the first frame, headless, debug build (the gate's),
apple m4, rustc 1.98.1: `crates/surface/tests/measure.rs`, run by hand with
`cargo test -p surface --test measure -- --ignored --nocapture`. scene
is materialize, sync, layout and the scene built into a `Compositor`;
pixels adds drawing it through the window's frame path into an offscreen
target and reading it back (the readback waits for the GPU). the GPU
context exists already, as in a running app; the glyph atlas is shared
across the rounds and the fixtures.

| fixture | nodes | first frame: scene, pixels | best of 20: scene, pixels |
| --- | --- | --- | --- |
| ide-commit-form | 5 | 6.6 ms, 19 to 24 ms (the process's first text and pipelines) | 53 to 61 us, 3.1 to 3.8 ms |
| ide-review-commit | 18 | 0.38 to 0.44 ms, 6.8 to 7.4 ms | 0.16 ms, 3.4 to 3.7 ms |
| showcase-todo | 11 | 0.17 to 0.21 ms, 5.4 to 7.5 ms | 79 to 82 us, 3.2 to 3.8 ms |

the scene of a surface is a fraction of a millisecond; the rest is the
frame path and the readback, which a window does not pay. no target was
set; these are the numbers of the proposer's latency (phase 3) to add to.

## reason

- a crate between the IR and the widgets keeps each one what it is: the
  IR pure, comps a library, the app the owner of its capabilities.
- the projection from the IR makes every component semantic, the nine
  without a comps `project` included, and names each node the way the IR
  does, instances by their item.
- reads named where the surface shows them make the surface a page an
  agent can observe on its own, without hiding that an act depends on
  state it does not show.
- one dispatcher means one record: a person on the fixed todo, a person
  on the generated surface and an agent on either leave receipts in the
  same place, with their actor.

## consequences

- phase 2 (transform by `SemId`) starts from the widget map: a node that
  stays keeps its widget across a `sync`, so a new `VerifiedIr` for an
  open surface can keep every widget whose id stays. what it needs is in
  the task file's handoff.
- a keystroke on a generated field is a request and a receipt. that is
  the contract (the text is the app's), and it is volume for the recorder
  and the port; a batched edit would need an ADR.
- the verifier of task-68 accepts props a widget does not take: a
  `typography` on a `text_field` (it draws with `body`), a `color` or a
  `background` on a control (its recipe decides). they are ignored, the
  widget's default holds. whether the verifier should refuse them is open.
- an `icon_button` shows one icon (`circle`): the IR v1 has no icon
  field. a `chart` reads numbers from the text of a list. a `table` is one
  column. an `image` is its shell and label.
- a surface projects into the app's builder; the reads the surface does
  not show name the app's state, which must be in the same snapshot.
- `view/dispatch.rs` of the showcase lost its agent module to a file of
  its own and left the frozen list of the file hygiene guard; the Surface
  page is one more arm per verb there.

## avoid

- a surface that runs anything: no `execute`, no call into the app's
  model; every effect is a request through the app's dispatcher.
- projecting a surface from its widgets' `project`, or binding the ids a
  widget makes by index (`<id>[<index>]`) into a page the IR names by item.
- a second layout engine, a literal visual value, or a token read by a
  name the theme does not have.
- materializing an `Ir` that did not go through `verify`.
