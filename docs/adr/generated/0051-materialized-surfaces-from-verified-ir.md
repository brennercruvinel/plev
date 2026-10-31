---
id: adr-0051
title: "a surface materializes only from verified IR, projects from the IR and asks through the app's dispatcher"
project: plev
type: adr
status: accepted
date: 2026-09-24
date_basis: header
topics: [generated-surfaces, ir, semantics, layout, agents]
relations: [adr-0023, adr-0032, adr-0033, adr-0035, adr-0047, adr-0049, task-68, task-69, task-75, task-79, task-80, task-95, task-97]
source: docs/adr/generated/0051-materialized-surfaces-from-verified-ir.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# a surface materializes only from verified IR, projects from the IR and asks through the app's dispatcher

Defines that a generated surface materializes only from verified IR: `rust/screen` projects the tree through the IR, draws with the `comps` widgets over the engine's layout and the `Theme` tokens, and asks the app's dispatcher for each event, without executing anything. The addenda record the change of shape without losing each widget's state, the item list, the image with the frame the app holds, and the rule that a text never gets narrower than its longest word; the `code` addendum is still proposed. It matters because the renderer gains no power of its own: every effect of a surface becomes a request with the registered actor, in the same place where the actions of the person and of the agent live.

## context

task-68 gave the plev IR (`rust/plevir`): a surface described by meaning,
a registry of what an app can do built from its own projection
(`Capabilities::from_snapshot`), and `verify`, the only way to a
`VerifiedIr` ([`0049-plev-ir-typed-and-verified.md`](0049-plev-ir-typed-and-verified.md)). the runtime could
refuse an IR, but it could not show one: nothing turned a `VerifiedIr`
into something a person sees and operates, or an agent observes.

what existed to build it from:

- the official app pattern, state in structs with retained widgets
  ([`0023-official-app-pattern.md`](../structure/0023-official-app-pattern.md)), and the comps widgets, 19 of them
  with an optional `SemId` that binds the rect they draw and hit-test
  with ([`0035-semantic-binding-beside-the-scene.md`](../semantics/0035-semantic-binding-beside-the-scene.md));
- the engine's `LayoutEngine` (taffy behind `LayoutStyle`, flex only)
  and every visual value as a `Theme` token;
- the semantic contract and its one `Dispatcher` for every actor
  ([`0032-semantic-contract-as-pure-projection.md`](../semantics/0032-semantic-contract-as-pure-projection.md),
  [`0033-actions-carry-observed-state.md`](../semantics/0033-actions-carry-observed-state.md)), and the rule that an act
  rests on what its actor was shown
  ([`0047-an-act-rests-on-what-its-actor-was-shown.md`](../semantics/0047-an-act-rests-on-what-its-actor-was-shown.md)).

task-69 phase 1 asked for the materializer: layout by the engine, tokens
by the theme, drawing by the comps widgets, semantics by construction,
every event a request through the app's dispatcher, nothing executed by
the surface itself. the orchestrator fixed the place (a crate of its own)
and that the projection comes from the IR, not from the widgets' own
`project` (9 of the 21 components of the vocabulary have none).

## decision

### a crate of its own

`rust/screen` depends on `uir`, `sem`, `engine` and `comps`. `comps`
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
apple m4, rustc 1.98.1: `rust/screen/tests/measure.rs`, run by hand with
`cargo test -p screen --test measure -- --ignored --nocapture`. scene
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

## alternatives

- projetar a superfície a partir do `project` de cada widget: descartado, porque 9 dos 21 componentes do vocabulário não têm `project`. A projeção sai da IR.
- um segundo motor de layout: descartado. Os containers (`stack`, `grid`, `split`, `overlay`) são mapeados sobre o `LayoutEngine` do engine.
- materializador dentro de `comps` ou de `uir`: descartado. `comps` continua biblioteca de widgets e `uir` continua puro; o orquestrador fixou uma crate própria.
- materializar uma `Ir` que não passou por `verify`: descartado. Nenhuma função da crate recebe `Ir`.
- fechar a superfície para trocar de forma: descartado na fase 2, porque o foco, o caret e um arraste em curso se perdiam.
- motor de animação próprio, ou duração e mola fora do tema: descartado. As transições usam a mola de `theme.motion` e o `Tween` sobre `theme.duration.slow`.
- a superfície decodificar quadros: descartado. Ela nunca guarda bytes nem pede decodificação; desenha o que o app tem agora.

## avoid

- a surface that runs anything: no `execute`, no call into the app's
  model; every effect is a request through the app's dispatcher.
- projecting a surface from its widgets' `project`, or binding the ids a
  widget makes by index (`<id>[<index>]`) into a page the IR names by item.
- a second layout engine, a literal visual value, or a token read by a
  name the theme does not have.
- materializing an `Ir` that did not go through `verify`.

## addendum, 2026-09-24: a surface changes form (task-69 phase 2, accepted)

### context

phase 1 kept a node's widget across a `sync`, and `SurfaceHost` refused a
second IR for an open surface. the only way to another form was to close
the surface and open the other: the app's state came back through the
bindings, and everything a widget held (the focus, the caret, a drag in
flight) was lost with it (the probe, in the task file). the program asks
the opposite: a capability keeps its identity and its semantics while its
representation is born, leaves or changes in real time.

### decision

- **transform.** `SurfaceHost::transform(VerifiedIr, &Snapshot)` gives the
  open surface the IR names its new form (`Surface::transform`
  underneath). a form for a surface that is not open (`NotOpen`), or one
  whose nodes would take the ids of another open surface's (`Collision`),
  is refused, and the surface keeps its form. only a `VerifiedIr`
  transforms a surface, as only one materializes it.
- **reconciliation by `SemId` and widget.** a node keeps its retained
  widget while its id stays and it is drawn by the same comps widget
  (`Kind`: the component, and for a `button` whether it binds `busy`,
  a `SubmitButton` then). the widget keeps what lives in it: focus,
  caret, selection, hover, a drag in flight; what a new form may change
  on it (a label, a placeholder, a column's title) is set again. an id
  that stays under another widget gets a new one, and nothing of the old
  widget passes: what it shows is the app's, read again through the
  binding, while the old widget's picture leaves as the new one enters.
  a new id gets a widget; a gone id drops its own. a node that comes
  back before its picture is gone is the same instance again.
- **geometry by `SemId`.** the rect each node was drawn at lives under its
  id. a node that stays keeps it until the next frame, so a click in
  between lands on what the person saw; a node that left loses it at
  once. it is where a move starts and where a leaving node stays.
- **semantics first, frames after.** the projection, the routing and the
  hit test follow the new form at once. a node born is projected and
  bound at once, and fades in. a node gone is out of the projection at
  once, and an act on it is the refusal the contract already gives a
  target that does not exist (`route` passes it on as it is; the app's
  dispatcher answers `NotFound`). its picture, a ghost holding its node,
  its widget, the snapshot it showed and its rect, fades out where it
  was, drawn over the live nodes, bound to nothing, hit by nothing.
- **transitions without an engine of their own.** the entry and the exit
  are the overlay's progress spring (`Spring<f32>` on `theme.motion`,
  as `OverlayManager` runs it), drawn as the opacity: the node draws
  into a compositor of its own and each scene node it pushed goes to the
  frame with its alpha scaled (a path's vertices too, with its hash
  moved; an image or a backdrop blur, which have no color, show whole
  down to half). a move is an engine `Tween` over `theme.duration.slow`
  (`EaseInOutCubic`), from the rect drawn to the rect laid out in each
  frame, so a resize in the middle lands. a transition starts at the
  first frame that draws the new form, with that frame's theme and
  layout. a fade is over within one 8-bit step of its end (1/255): the
  engine spring's own rest threshold asked for half as many frames
  again, all of them the same picture.
- **render on demand.** `Surface::wake` and `SurfaceHost::wake` ask for a
  frame when a change of form waits to be drawn, for every frame while a
  transition runs, and for nothing once it settles.
- **reduced motion.** the engine exposes no motion preference yet (the
  system's comes with task-75), so the host takes it from whoever knows
  it (`SurfaceHost::set_reduced_motion`, `Surface::set_reduced_motion`).
  with it every change is instant, the transitions in flight included,
  and a dismissal lets go at once.
- **dismissal.** `SurfaceHost::close` takes the surface out of the
  projection, the routing and the hit test at once, fades its picture
  where it was drawn (`draw_closing`), and then lets everything go; it
  answers whether the surface was open.
- **the demo.** the Surface page of the showcase switches its surface
  between two forms with a segmented control: compact (the fixture of
  task-68) and two columns (`rust/screen/tests/fixtures/uigallery-todo-columns.ir.json`,
  a grid over `field_min_w`, one column at a phone's width), each
  verified against what the todo projects when the person asks for it.

### cost

the mixer of the tests (a slider moves to the other panel, a hint is born,
a reset button leaves, a switch becomes a checkbox) draws 37 of 120 turns
of a 60 Hz display after its change of form, one run from the change on,
and none after; with reduced motion, 1 of 120. while a node fades it is
drawn twice (into its own compositor, then copied); at rest the cost is
phase 1's. the time per frame of a transition was not measured.

### consequences

- a sync that changes the tree (a repeat's list moved) shows the same
  transitions as a new form: it is the same reconciliation.
- a surface shows at once when it opens and before its first frame: there
  is nothing to move from.
- a fade is per scene node, not per group: the primitives of one node
  blend with each other while they fade, where a layer's opacity would
  composite them first. the surface draws into the layer it is given and
  owns no layer of its own.
- a ghost is drawn over the live nodes: a leaving container with a
  background would veil the new form while it fades (no fixture has one).
- a `button` that starts or stops binding `busy` changes comps widget
  (`Button`, `SubmitButton`); their hover and press are private in comps
  and do not pass. hover returns at the next pointer move; a press in
  flight at that instant does not complete.
- the page's control is the page's, not a capability: an agent does not
  switch the form. that is a proposer's job (phase 3), through
  `transform`.

### avoid

- a transition that holds the semantics back: a node gone is no
  capability from the change on, whatever its picture shows.
- a second animation engine, or a duration or a spring that is not the
  theme's.
- carrying a widget's state into a widget of another kind by hand: what
  must survive a change of representation is the app's, and it is read
  again through the binding.

## addendum, 2026-09-24: the todo's surface shows its items (task-95, accepted)

- the showcase todo projects `todo:items` (the visible ids, in the order
  shown) and each item's `text`. both forms of the Surface page repeat
  over it with one template: `todo_panel.item[{item}]`, a row of the
  item's text (`todo_panel.item[{item}].text`) and a checkbox "done"
  (`todo_panel.item[{item}].done`, `checked` from `completed`, `toggle`).
  the two-column form holds the entry and the list in
  `todo_panel.main`.
- a check on the surface is the todo's `toggle` on that item, by whoever
  acted, through the todo's one dispatcher; the fixed todo shows it and
  moves the surface back. a change of form keeps each instance: a press
  begun on an item's checkbox completes after the surface takes the other
  form.
- removal stays on the fixed todo. the todo's `remove` is destructive
  with no confirmation of its own, and the verifier binds such an action
  only to a dialog's `confirm`. a dialog that opens because of an unrelated
  state (an item being done) would say the wrong thing, and an always
  open one per item is noise, so the surface binds none; when the item
  leaves the list (removed on the fixed todo, or filtered out), its
  instance leaves the surface.
- a materialized dialog draws its cancel button even when `cancel` is
  not bound, and a closed dialog stays projected with its `confirm`
  (phase 1): both matter to any surface that binds a removal later.

## addendum, 2026-09-24: an image draws the frame the app holds (task-69 phase 4b, accepted)

### context

phase 1 drew an `image` as its shell with its label: the IR names a
source, and the engine takes a GPU image handle the surface has no way to
get. the video editor scenario (the thesis' section 4, the silences app
of phase 4a) needs the generated review to show a frame of the video and
one per cut, decoded by the app's own worker (ffmpeg, one child at a
time, cached by instant), and the comparison to show two views of the
video at two instants. the probe of phase 4b (the task file) showed the
verifier needs nothing new: an `image` binds any text of the app as its
`source`; what was missing was a text that names a frame, and a way for
the host to reach the app's cache.

### decision

- **the frame ref is the app's.** an app that can decode frames projects
  one text state per frame it offers, whose value is the media node's id
  and the instant in seconds: `silences.video@4.017052`. the silences
  project `silences.video:original` (the frame of the original at the
  instant shown), `silences.video:result` (the frame of the result at
  that instant: inside an applied cut, the cut's end, the first frame
  that stays) and `silences.cut[<id>]:frame` (the middle of the cut).
  an `image` binds its `source` to one of them; nothing in the
  vocabulary, the schema or the verifier changes.
- **`Frames`.** a trait of the crate, `fn frame(&self, source: &str) ->
  Option<ImageHandle>`: the app's frames as the surface reads them, by
  the frame ref. `Surface::render_with` and `SurfaceHost::draw_closing_with`
  take one; `render` and `draw_closing` are the same with `NoFrames`. a
  ghost of an image asks the same, so a picture fades out with the rest.
  the surface never holds bytes, never decodes, and never asks for a
  decode: it draws what the app holds now.
- **`Surface::sources`** (and the host's) names the frame refs the
  surface's images show. the app decodes what it lacks on its own time:
  the silences page asks its worker from `wake`, never in a frame, one
  job at a time, the failures cached so nothing is asked in a loop; when
  a frame arrives, the page asks for a frame and the surface draws it.
- **the picture is fitted.** an image draws its shell, and inside it the
  picture fitted and centered, never scaled up; without one, the shell
  and its label as before. its size is the block's (half the narrowest
  card), or, with a `control_size`, that control height at 16:9 (a
  thumbnail in a row). the layout is the same with or without the
  picture, so a frame arriving moves nothing.
- **the silences page** implements the trait over its cache
  (`jobs::PageFrames`, the instant parsed from the ref), and lends its
  `SurfaceHost` out for a surface's events the way the todo lends its
  dispatcher; the surface opens beside the fixed screen (the fixed screen
  takes the left half, or the top at a phone's width) and the fixed
  screen takes the width back when it closes.

### consequences

- the two fixtures of the silences surface,
  `rust/screen/tests/fixtures/uigallery-silences-review.ir.json` and
  `showcase-silences-compare.ir.json`, verify over the app's golden, draw
  with frames made in memory in the crate's tests and with ffmpeg's in
  the page's ignored test, and transform one into the other keeping
  every cut's instance (a press begun on a cut's button completes after
  the change of form).
- the app's list state holds the items' keys (`silences:cuts` is `["1",
  "2"]`, as `todo:items`), which a repeat keys its instances by: the
  phase 4a projection held the ids and gave instances named
  `review.cut[silences.cut[1\]]`.
- a 32 by 18 picture stays 32 by 18 in a 140 tall box: the surface
  never scales up, as the fixed screen does not. a frame decoded wider
  than its box is fitted.
- the surface half at 1200 wide is about 430 px inside its card
  padding: a row of a thumbnail, two texts, a badge and two buttons does
  not fit in one line and wraps its texts; the fixtures put the texts
  in a column between the thumbnail and the buttons.

### avoid

- an image that carries bytes, or a surface that decodes: the frames are
  the app's, by their ref.
- a decode started from a frame: the app asks its worker from its wake,
  after an actor's request opened the surface.
- a frame ref that is not the media node's id and an instant: the rules
  and the page read it by that form.

## addendum, 2026-09-24: a generated surface is usable at the default window (task-69 checkpoint b, accepted)

### context

the checkpoint b, run by the orchestrator through the contract and the
mirror of task-79, saw the review generated by the rules broken at
1200x800 (a cut's row shrank its texts to one glyph and pushed its
buttons 181 px past the card) and the Claude's with sliders without a
label or value and durations as "1.996916". the cause of the row: taffy
takes an item's min-content as its flex minimum, and the engine measures
a text's min-content at width zero. no baseline showed a generated
surface open, so the defect passed every phase.

### decision

- a text is never narrower than its longest word (`layout.rs`,
  `longest_word` as the item's `min_width`).
- a row first shrinks what can shrink; when its children still pass its
  content box, it wraps, and the tree is laid out again until no row
  overflows (one more pass per level of rows that did not fit).
- a slider draws its label on the left and its value on the right on a
  band above its track; the track is what the pointer drags and what the
  slider binds.
- a number shows with at most two decimals and no zero at the right
  (`widget.rs`, `number_text`), on the screen and in the label a text
  projects; the app's state keeps every decimal. the unit stays in the
  label: nothing changes in `uir`.
- the prompt says, after the intent and before the rules, how a surface
  reads, the same for every app: the task first, a label with its unit on
  every control, each control once.

### consequences

- the regress stages a page in a named state (`examples/regress/state.rs`)
  through the port's `act` and two public staging entries of the Silences
  page (`stage_silences_frames`, `stage_silences_ir`), with synthetic
  frames; six baselines of the review open (by the rules and from the
  fixture of the Claude's first-round proposal,
  `rust/plevir/tests/fixtures/silences-review-claude.ir.json`), at
  1280x800 core and hoff and at 1200x800 core.
- a wrapped row goes down in order without grouping by meaning ("reject
  cut" alone on a third line at 1200x800); grouping is the structure the
  rules build.

### avoid

- a generated surface the thesis uses without a baseline in the state a
  person uses it.

## addendum, 2026-09-25: a `code` draws its rows by kind and scrolls (task-97, proposed)

### context

the IR gained `code` ([`0049-plev-ir-typed-and-verified.md`](0049-plev-ir-typed-and-verified.md), addendum
of task-97), so the review of the ide shows its diff with the kind of
each line instead of in a table. what it takes to draw it was open: the
kinds' colors had to come from tokens the theme already has, readable at
WCAG AA over the block, and a diff longer than its room had to scroll.

### decision

- the comps `CodeBlock` gains rows (`CodeLine`: a text and a
  `LineKind`) and a scroll offset, additively: without rows it draws
  exactly as before. with rows, one line box of the mono step per row,
  cut at the inner width with an ellipsis, clipped to the area under the
  caption and moved up by the offset.
- each kind by the tokens the design system already uses for an intent
  wash (a selected chip, a progress fill): behind a line added,
  `colors.success` at `glass.wash_alpha`; behind one removed,
  `colors.danger` at the same alpha; behind a header, `glass.surface_hover`;
  nothing behind context. the text of every row in `colors.text`, the one
  text token that reads at AA over every wash: measured as task-75
  measures (the field over the page or a card, the wash over it, the ink
  over the wash, in linear light), the lowest is 4.59, a line added in
  hoff over the page, in the four themes of task-75 and hoff_light. the
  secondary `text_mid` of the wrapped code would read 3.64 over the added
  wash in hoff.
- the wash alone is 1.06 to 1.67 against the block, so the kind never
  rests on the hue (WCAG 1.4.1): the row's text is the app's, and a
  diff's text carries its `+` and `-`. the block draws no mark of its own.
- the surface keeps a `code` in a retained widget (`rust/screen/src/code.rs`):
  the block, an `engine` `ScrollState` and the comps `Scrollbar`, the
  pair the fixed diff panel scrolls with. its lines are the block's rows,
  each with the kind its `kinds` names at the same place; the label is
  the caption. it is as tall as its rows; bounded by `size.max`, its rows
  scroll inside it with the wheel, the scrollbar shows while they move,
  new lines (another diff) start at the top, and the offset is never
  projected.

### consequences

- the demo 5 of task-80 measured again (task-80, "de novo, com o `code`
  no IR"): 14 of 14, 7 of 7 and 14 of 18 lines of the three diffs
  without scrolling, against 5, 5 and 5 in the table and 14, 7 and 18 in
  the fixed panel. the one height token of the IR (`dropdown_max_h`)
  bounds the block; the fixed panel takes the window's height.
- the showcase's code block page does not show the rows (the showcase
  was outside task-97); no baseline changed.
- three named themes outside the four task-75 measures read below AA on
  a line added (tokyo-night 4.30 to 4.38, gruvbox 3.79 to 3.94,
  one-dark 3.32 to 3.40, and one-dark's removed line 4.00 to 4.12 and
  its header 4.43 over a card): their
  `colors.text` is lower than the four's. a theme's color is the design's
  decision.

### avoid

- a color for a kind that is not a token, or a text token that does not
  read at AA over its row.
- projecting a scroll offset.
