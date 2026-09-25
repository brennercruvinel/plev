---
type: adr
status: accepted
tags: [uir, surface, plev-ir, transform, verifier, identity, history, generative-runtime]
date: 2026-09-24
---

# adaptive transformations are small verified operations that preserve identity

## context

task-68 gave the plev IR and the verifier, the only way to a `VerifiedIr`
(docs/ADR/plev-ir-typed-and-verified.md). task-69 gave the host that
materializes it and changes a surface's form by a whole new `VerifiedIr`,
reconciled by `SemId` (docs/ADR/materialized-surfaces-from-verified-ir.md,
the addendum of phase 2). the checkpoint b ended with the generated
surface no longer worse than the fixed one, and the generation stays the
main line (docs/mission/steps/checked/task-69-ephemeral-generative-surfaces-done.md).

task-70 is the next step of the thesis (sections 4 and 16): the interface
changes without being built again. the only change of form was the whole
one. there was no small operation on the IR a surface shows, verified,
reversible and recorded, and nothing a person or an agent could ask for
("too much here", "put the cuts first") short of a new proposal.

this ADR covers phase 1: the operations, their inverse, the verification
and the host. the proposer of transformations (phase 2) and the composed
capabilities (phase 3) extend it.

## decision

### nine operations, as data

`uir::Op` (`crates/uir/src/transform/mod.rs`) is serde data, the
operation's name in `op`: `{"op": "hide", "id": "review.controls"}`. its
reader denies unknown fields, so a model's typo is an error, not a
default.

| operation | what it changes in the IR | refused before | what the verifier can refuse |
| --- | --- | --- | --- |
| `hide {id}` | the node leaves its parent's `children`, set aside whole | the root (a surface is closed, not hidden); a repeat's item (the repeat always has one); a node hidden already | a split left with one child (rule 3) |
| `show {id}` | the node set aside goes back to its parent, at its place (the end when the parent has fewer children now) | not hidden; inside a hidden node (show that one); its place hidden or gone | a split with three (rule 3); a second layer of controls in an overlay (rule 7) |
| `group {ids, into}` | siblings become the children of a new container `into`, in their order, in the place of the first; the group lays them out as their parent's stack did | not siblings; `into` taken, in the tree or aside; a name twice | `into` without `[{item}]` inside a template, or what a template makes (rule 2); a split's child (rule 3); a level or a node too many (rule 8) |
| `reorder {parent, ids}` | `ids` first, in order, then the other children as they were | a repeat (its order is the app's list); a stranger; the order already | nothing: no rule looks at the order; the focus order is recomputed in tree order, so it follows what is seen |
| `emphasize {id}` | one step up the token the node is drawn by: a text one size up the type ramp (body to title, title to headline, headline to h4), a button or an image one control size (a button with none goes to `md`), a container, card or list one elevation | the top of the ladder; a component no token of emphasis draws (stat, badge, table, chart, the other controls) | nothing: no rule looks at props |
| `density {step}` | every gap of a stack or grid and every padding set, one step of the spacing scale (`denser` or `sparser`), held at the ends; unset spacing stays the component's | nothing moves | nothing |
| `restyle {id, role}` | a text's `color` or a container's or list's `background`, by a role that part takes (an ink for a text, a fill for a container) | a role of the other kind; a component whose colors are the theme's; the role already | nothing |
| `swap_view {id, component}` | the component, and the name of the prop the datum is bound by, every other binding kept | outside a family; a list that repeats a template (its items' ids would leave the surface); a chip that sends `activate` | the kind (a number is no badge's text, a set is no chart's data: rule 4); a name the target needs (a chart, a stat: rule 7) |
| `promote {target, action, into}` | a `button` bound to `activate` on the capability, labelled as the app labels it, at the end of `into`, its id `into.<action>` | the app declares no such action; its policy keeps it from whoever asked; it takes a parameter (a button sends none); the surface shows it, in the tree or hidden (then it is a `show`); `into` holds no children | the surface's author may not bind it, or it is destructive without a confirmation (rule 6) |

the families of `swap_view` are the components that show one datum alike
under their own names: a text, a stat and a badge (one value); a
checkbox, a switch and a toggle chip (one bool, `toggle`); a select and
tabs (a choice, `select`); a table and the four charts (a list). the
chart draws the numbers each item of its `data` list reads as, and the
`List` of `sem` has no kind of item, so whether the numbers mean
anything is the app's: a chart over a list of ids verifies and draws the
ids.

### an adapted surface, and every result verified

`uir::Adapted` is the `VerifiedIr` the host draws and projects and the
nodes set aside, by id. `apply(op, by, cx)` works on a draft (the IR and
what is aside) and returns the new `Adapted` and the `Inverse`, or a
`TransformError`: `NotFound`, `Instance` (below), `Refused` with why, or
`Unverified` with every error of the verifier. `cx` is the registry
(`Capabilities` from the app's projection) and the policy the surface
is verified under; `by` is the actor who asked, and only `promote` reads
it. `undo(inverse, cx)` verifies again too. no function of the module
returns an `Ir` that did not pass `verify`.

### hidden is aside, not a field

the vocabulary has no hidden node: `IrNode` has no field for it, the
reader refuses an unknown field, and the one node that does not draw is
a closed dialog, which needs an `open` state of the app. so a hidden
node leaves the tree (what is drawn, projected, routed and focused) and
is kept beside it, whole, with its parent and its place, for `show` and
for the inverse. no field, no change to the verifier or the schema.

### identity

every id of the surface is in its tree or set aside, and only `hide`
takes ids out of the tree (the hidden node's). the state of the nodes
that stay is kept by the host's reconciliation (below). `group` and
`promote` add one id each, checked new against the tree and what is
aside.

### the inverse, exact

each operation returns its inverse, and applying it gives the surface
before, byte for byte in the IR's JSON and equal in what is aside. where
the vocabulary has the contrary operation and it is exact, the inverse
is it: `show` for `hide`; for `show`, the node set aside again at the
place it was kept with (the place is kept even when `show` put it at the
end); the whole order before for `reorder`; the component before for
`swap_view` (within a family the rename is a bijection). where the
contrary step would not come back to the same place, the inverse keeps
what the operation replaced: the props and layout of each node that
moved for `emphasize`, `restyle` and `density` (a ladder holds at its
ends, and unset is not a step); the parent before for `group` and
`promote` (the nine have no ungroup, no demote). the inverse is
`Serialize` (the recorder can write it); it holds an `IrNode` in the
last case, which the IR reads only inside a whole document, so it is not
read back on its own.

### templates

the IR holds a template once, with `[{item}]` in its ids; instances
exist only where a surface unrolls it over the app's list. an operation
on a template acts on every item, those of now and those to come. an
operation on an instance's id (`review.cut_apply[1]`) finds no node in
the IR and is refused as `Instance`, naming the template that makes it.
changing one item alone would need an override per item in the IR,
which the vocabulary does not have.

### where it lives

the operations, the inverse and `Adapted` are in `uir`
(`crates/uir/src/transform/`), pure like the rest of the crate: the
operation is data the recorder keeps and the proposer of phase 2 writes,
and `propose` depends on `uir` and `sem`, never on the engine; applying
is a function from IR to IR followed by `verify`, which is here; and the
property test needs `proptest`, a dev-dependency of `uir` alone. the
history, who asked and the reconciliation are in the host (`surface`).

### the host keeps a history

`SurfaceHost::apply(id, op, by, app, policy)` checks and verifies the
operation against the registry of `app` under `policy`, then the surface
takes the form it makes the way a whole new form is taken: the collision
check with the other open surfaces and `Surface::transform`, reconciled
by `SemId`, so a widget whose node stays keeps what lives in it (a drag
in flight, the caret, a press on one cut's button). the change is kept as
a `Change` (the operation, its inverse, the actor). `undo` applies the
last inverse, verified again, the same way, and returns the change.
`changes` lists them, `adapted` gives what is aside. a refused operation
leaves the surface and the history as they were; a whole new form
(`transform`) and `close` end the history. `apply` takes the policy
because a `VerifiedIr` does not keep the one it was verified under.

### what an agent may not do, it still may not

a transformation moves bindings, never widens them. a node bound to an
action the app keeps from agents (`PERSON_ONLY` in the showcase) still
binds it after a group, a reorder or a swap, and the surface projects
the action with the app's own policy. `promote` makes a control only for
an action the registry declares for the actor who asked, and the
verifier still checks the surface's author. nothing here touches the
data a surface shows, so no redaction of `sem` is lost, and no picture
enters the IR: an image binds a frame ref the app projects, as before.

### fixed screens

the fixed screens of the showcase (the Silences, the todo) are not IR:
the Silences computes its geometry in code and draws a timeline that no
component of the vocabulary draws. two ways to adapt one:

- express it in IR, screen by screen, as the ide's commit form fixture
  of task-68 already does with the ide's own ids (the verifier lets a
  node with an app's id bind only that node's actions, and the
  projection treats it as the app's). every operation then applies to
  it, verified, with its inverse and its history, by the same path;
- transform at the level of the projection: the screen draws as it
  wants and the transformation changes what it projects. what a person
  sees and what an agent reads part ways (a reorder in the projection
  moves nothing on the screen), there is no verifier and no inverse by
  construction, and each screen becomes a case.

the first, when the screen fits the vocabulary (the todo does, the
commit form does). a screen that does not fit (the Silences, for its
timeline) stays fixed and untransformed until the component exists. the
second is not taken.

### the demo

over the fixture of the Claude's review of the checkpoint's first round
(`crates/showcase/fixtures/ir/silences-review-claude.ir.json`, the one
that put the list of the cuts last), a `reorder` puts the cuts after the
title and a `hide` takes out the detection card the fixed screen already
offers. at 1280x800 both cuts, with their decision, duration, frame and
buttons, are in the window above the video; the history holds both
changes by the harness; undoing both gives the fixture back byte for
byte. the regress has the state "Silences, review by the Claude
(fixture), adapted" (`examples/regress/state.rs`), blessed with its image.
the chart of the durations the task imagined does not fit the Silences:
its one list is the cuts' ids, and the durations are one number per cut.

## cost

- an operation clones the IR and runs the verifier over all of it: a
  few hundred nodes at most (the policy's limit), once per operation.
- `group` and `promote` keep the whole parent for the inverse; `density`
  keeps the props of every node it moved.
- the swap to a chart over a repeat, and a change to one item of a
  template, are not reachable: both need the vocabulary to grow.
- `emphasize` knows the order of the theme's type ramp and which
  components draw a token of emphasis: the order is the ramp's names,
  and a new component needs its line there.

## reason

- the verifier stays the one door: an operation is only another way to
  propose an IR, and the result is judged like any proposal.
- small operations are what a person asks for and what the recorder and
  an agent can read ("hide review.controls"), where a whole new IR is
  opaque and costs a model call.
- exact inverses make undo a guarantee, not a best effort, and a history
  that is safe to replay.
- identity is what the reconciliation of task-69 needs to keep the
  state of a widget; an operation that drops an id silently would drop
  that state too.

## consequences

- phase 2 writes `Op`s: the rules and the Claude answer "there is too
  much here" or "put the cuts first" with operations, in the JSON form
  above, through the same `Proposer` path, and the page's port action
  applies them by `SurfaceHost::apply` with the actor who asked. the
  preferences of accessibility (a bigger type, more space) are
  `emphasize` and `density` asked on the person's behalf, with the same
  history and undo. a schema of `Op` for the prompt is phase 2's.
- phase 3 reads the history (`changes`, with the actor) next to the
  receipts of task-63 to find a repeated sequence; installing a
  composed capability is a `promote` once the app declares it.
- a transformation is never automatic: it is asked for, by a person or
  on their behalf with their acceptance (the task's rule on muscle
  memory).
- `uir` exports `Adapted`, `Aside`, `Context`, `Inverse`, `Look`, `Op`,
  `Step`, `TransformError`; `surface` exports `Change` and has
  `SurfaceError::Transform` and `NothingToUndo`.

## avoid

- a hidden field in `IrNode`, or hiding by a token: the vocabulary and
  the verifier stay as they are.
- an operation that returns an unverified IR, or an undo that skips the
  verifier because the IR "was verified before" (the app may have moved).
- a swap that drops a binding, a template, or an id.
- a promotion for an actor the app keeps the action from, or of a
  destructive action without the app's confirmation.
- transforming a fixed screen at the projection only.

## integração

accepted on 2026-09-24 by the orchestrator (task-70 phase 1, merge of
`task/70a-transforms`). the whole diff read; the demo seen at 1280x800
(the Claude's first-round review with the cuts right after the title,
each with its frame, its duration and its buttons, the repeated
detection hidden and still on the fixed screen). local gate green, 2861
passed, 0 failed (was 2832); regress 170 unchanged (169 old, 1 new).
open for phase 2: who asks (the rules and the Claude answer operations,
through the app's dispatcher), the policy each surface was verified
under kept by the host, emphasis drawn by `stat` and `badge`.

## addition: phase 2, who asks for a change (2026-09-24)

### context

phase 1 gave the operations and the host that applies them; only a test
or the regress applied one (`ShowcaseView::stage_silences_op`), and no
person or agent could ask for one in words. the checkpoint b of task-69
also left three gaps in the generated review: the model's guidance made
it drop each cut's frame, the rules' row showed numbers without a name
and let a button fall alone to a line, and a decision showed as loose
text, not as a badge.

### decision

#### a transformer beside the proposer

`propose` gets a sibling of `Proposer`, untouched: `Transformer::transform(&Adapt) -> Result<Adaptation, ProposeError>`.
`Adapt` is the intent, the registry, the snapshot, the surface as its
transformations left it (`uir::Adapted`, with what is aside), the policy
it was verified under and who asks. `Adaptation` is `Ops(Vec<Op>)` or
`Undo` (undoing is the host's: it knows the last change). the way out is
`transform_verified`, the pair of `propose_verified`: it applies every
operation in order over the surface, each verified again by
`Adapted::apply`, and returns the operations only with the verified
surface they make (`Transformed::Changed`), or the first refusal with its
place (`ProposeError::Unapplied`). no operation at all is `NoChange`.
`Rules` and `Llm` implement both traits; the `Model` is the same.

#### the rules' grammar of changes

a second reader beside the one of surfaces (`crates/propose/src/change.rs`),
with the same folding, in english and portuguese: **less** ("too much",
"cluttered", "simpler"; "demais", "poluído", "mais simples"), **compare**
only with a word of difficulty ("can't compare"; "não consigo comparar":
a plain "compare the original with the cuts" stays the surface grammar's
comparison of views), **bigger** ("bigger numbers", "can't see them";
"números maiores", "não enxergo") and **undo** ("undo"; "desfazer").
every other word is a filler, a word of the surface grammar taken as a
hint, or a word of the app or of the surface shown, which chooses the
target. anything else is not a change.

the targets come from the structure and the registry, never from an
app's name (`crates/propose/src/reshape.rs`): the task is what binds an
action, context a block that shows state and binds none. less hides each
block of context (down through what holds the task, never inside a
repeat, never a split's child), groups the controls of a stack that runs
them beside readouts, then tightens the spacing. compare swaps a table of
a list of numbers that is not a list of items (its values are not the
keys of its node's children) for a bar chart, a chart for a table, and
refuses with why when nothing fits: on the Silences nothing does, as
phase 1 found, and the page says so. bigger emphasizes every stat and
every text bound to a number. each operation is tried over what the ones
before made and kept only when it applies.

#### the prompt of a change

`Llm` as a transformer asks with its own prompt (`crates/propose/src/ops.rs`):
the intent, how a change reads, the nine operations as JSON with what
each refuses, the rules of identity, the verifier's rules and the
policy, who asks, the components, the registry (the same text as a
proposal's: a secret is its length, a picture the text that names it),
and the surface shown with what is hidden. the answer is one object,
`{"ops": [...]}`; each operation is tried in order, and the first refused
goes back with the reply and why ("attempt N was refused"), at most
`Llm::ATTEMPTS` times, then `Unchanged` with the last reasons, and the
surface does not change. the `SYSTEM` of `claude-model` ("answer exactly
what it asks for, in the form it asks for") serves as it is.

#### the page: the same `propose`

a new node or action on the proposing row would change the projection of
every baseline that shows the row (the regress compares meaning first),
the Claude's review and the fixed screen's included. so there is none:
with a surface open, the row's `propose` reads the intent as a change
first (`surface_page/proposing/changing.rs`, shared by the Surface page
and the Silences). by the rules the change is applied at once through
the host, as the request's actor, and "undo" takes the last ask back; by
the Claude, when the intent reads as a change, the worker asks for
operations off the app's thread, the request starts, the status shows the
attempt and the seconds, and the operations are applied when they arrive
with the attempts, the seconds and the cost; "undo" asks no model; an
intent that is no change proposes a surface as in task-69. an agent asks
through the port the way it proposes, `set_value` on the intent and
`propose`: protocol v1 unchanged, and the actor is the `Change.by`.

#### the host keeps the policy and the asks

`SurfaceHost::open_under` and `transform_under` keep the policy a surface
was verified under; `open` and `transform` keep the default one. `apply`
and `undo` no longer take a policy: every operation is verified under
the surface's own. `apply_all` applies the operations of one ask all or
none, takes the form once and keeps them as one ask; `undo_asked` takes
the last ask back whole; `last_ask`, `current` and `policy` read them.

#### emphasis drawn by `stat` and `badge`

with no change to the vocabulary or to comps: a stat reads its
`typography` as the size of its value (`base_m` and under the inline
readout, `title` the card's, `headline` and over the hero's), and a badge
its `background` as the intent it is filled with (`Badge::count` instead
of the glass tag). `emphasize` climbs a stat `base_m`, `title`,
`headline`, and fills a badge with the accent; the phase 1 table's "stat,
badge" among the components with no token of emphasis no longer holds.
a badge grows in weight, not in size: comps has no bigger badge.

#### the review shows what each item is

the prompt's guidance says, for any app, that reviewing items is seeing
what each one is (its picture when the app has one, its name, its
numbers with their names, its state) and that "only what the intent
needs" never takes that away. the rules' row goes by sense
(`crates/propose/src/row.rs`): what the item is (picture, texts, and a
decision as a badge: a text two actions or more of the item read and
nothing else), its numbers (a small stat each, named by its key), its
actions (small buttons); each sense of two nodes or more is a row of its
own, one under the other.

### consequences

- phase 3 reads `SurfaceHost::changes` and the asks (`last_ask`) beside
  the receipts of task-63.
- the grammar of changes gates the Claude: an intent the rules do not
  read as a change goes to a proposal of a surface. a phrase such as "put
  the cuts first" needs its word in the grammar first.
- the swap for a chart needs the app to declare a list of numbers; the
  Silences does not, and the rules say so.

### integração (fase 2)

proposed by task-70 phase 2 (`task/70b-transform-proposer`); the
orchestrator accepts it.

### integração da fase 2

accepted on 2026-09-24 by the orchestrator (merge of
`task/70b-transform-proposer`). the whole diff read; the images seen:
the rules' review with each cut's frame, its range, a "pending" badge,
start, end and duration named, and the buttons on one line; "too much
information" over it keeps only the cuts; "tem informação demais" over
the Claude's first-round review hides the video and the counts and fits
both cuts in the window; "I can't compare these" refused with why. the
3 baselines of the rules' review changed and were accepted; nothing
else moved. local gate green, 2894 passed, 0 failed (was 2861); regress
173 unchanged. the real Claude answering a change is the orchestrator's
proof, next.
