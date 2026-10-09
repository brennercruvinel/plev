---
type: adr
status: accepted
tags: [sem, semantic-contract, identity, accessibility, agents, recorder, generative-runtime]
date: 2026-09-23
---

# semantic contract as a pure projection of app state

## context

the only identity in the engine is `ViewId`, a counter that
`InputState::begin_frame` resets to zero every frame
(`rust/engine/src/control/input/mod.rs:50-61`). the third control emitted is
`ViewId(2)` in this frame and may be another control in the next. the
accessibility tree is built from hit regions, with the role guessed from
`focusable` and the label always empty
(`rust/engine/src/runtime/window/render.rs:99-125`). in the ide,
`FileEntry.staged` (it goes into a commit) and
`UnassignedView.selected_idx` (a highlighted row) are loose fields
(`apps/idestudio/src/interfaces/panels/unassigned.rs:51-61`); nothing says the first
can invalidate a commit and the second cannot.

the generative runtime program (task-57 to task-80) needs one contract
that rendering, accessibility, agents, tests and the recorder all read:
who a control is, what it means, its state, the actions it accepts, the
state those actions depend on and who may request them. the drawing of a
control can be born, move or disappear while the capability keeps its
identity.

## decision

a new pure crate, `rust/accord`. dependencies: thiserror, indexmap, and
serde behind a default feature. no gpu, no window, no text stack.

- identity is `SemId`: canonical text made of name segments
  `[a-z0-9_]+` joined by `.`, plus bracketed key segments with `]` and
  `\` escaped (`changes.file[src/lib.rs]`). the app or the IR picks it.
  it never comes from a counter or a position.
- a `Node` carries a closed `Role`, a label (required for interactive
  roles), state as `Value`s under `StateKey`s, `ActionDecl`s with the
  `StateRef`s each one reads and its `Policy`, `Constraints` (with
  `secret`), the `enabled`, `inert` and `focusable` flags, and the
  `Source` of the projection site. `inert` is inherited from any
  ancestor.
- the app implements `Semantics::project(&self, &mut SnapshotBuilder)`.
  it runs on demand (a frame, an agent observation, the check of an
  action), always from the live state, and `sem` keeps no reference to
  app state. `finish` validates the whole projection and returns every
  violation at once (`SemErrors`), the way the parser droplist reports
  every construct it drops.
- a `Snapshot` is an immutable value. its `Revision` is FNV-1a 64 over an
  encoding documented in `rust/accord/src/revision.rs` (version tag
  `sem.revision.v1`); it never goes through a std hasher and it leaves
  `source` out. `Snapshot::diff` returns `Change`s, and the diff is empty
  exactly when two snapshots are equal, which is exactly when their
  revisions are equal.
- invalidation is semantic by construction: what is not projected does
  not exist for the contract. hover, animation, cursor position and
  layout are never projected, so they can never invalidate an action. the
  staged set is projected, so it can.

the rule for large content: project what is operable and meaningful, not
the whole content. an editor with a million lines is one node with its
length, selection and cursor, not one node per line. a diff view is its
file and its hunks, not every changed line. which part of the snapshot an
agent observes is a question of scope, and scope belongs to task-61.

## reason

- one contract replaces five representations that drift apart: the
  visual tree, the accessibility tree, test selectors, agent tools and
  recorder events.
- pure data in its own crate can be linked by consumers that must not
  pull the gpu stack (agent protocol, recorder, headless tools, a cli),
  and it is testable alone, wasm included.
- a projection built on demand from the live state leaves no second copy
  of the state to keep in sync, which is what an accumulation per frame
  inside the render would create.
- the revision crosses processes (agent, recorder). std hashers are
  seeded or leave the algorithm unspecified across releases, `Hash` for
  `usize` follows the pointer width, and FxHash makes no stability
  promise. so the hash and its encoding are written and pinned in the
  crate, with a golden value computed independently from the documented
  encoding.

## consequences

- every widget and app screen that wants to be operable projects itself:
  the comps declarations and the bridge to `engine::actions` are
  task-59, accessibility from the snapshot is task-60, the agent
  protocol is task-61, the ide is task-62.
- the text form of `SemId` and `StateRef`, the serde form and the
  revision encoding are a contract with the agent protocol (task-61) and
  the recorder (task-63). changing any of them needs an ADR; changing the
  revision encoding also needs a new version tag.
- a `secret` value leaves the snapshot as its length: `Text` becomes
  `Int(chars)`, `List` and `Set` become `Int(items)`. scalars on a secret
  node stay, since they describe the field (a `revealed` toggle), so a
  secret must be projected as text.
- the revision ignores the order in which state keys were declared and
  how unrelated subtrees interleave in the projection. sibling order
  counts, because it is the order of a list.
- `engine::actions` stays a key binding registry. `ActionName` is the
  capability a binding can target; no binding targets one yet (see the
  update below).
- the cost of projecting on demand is not measured yet (task-59 and
  task-73 measure it).
- updated on 2026-09-23: task-59 did not bridge `engine::actions` to
  `sem`. the keymap (`rust/engine/src/control/keymap/`) still knows nothing of
  `sem` and no binding names a capability; the ide sends the keys that act
  on one through its `Workbench`, which asks the dispatcher as the person,
  and left a keymap entry naming a capability out on purpose
  (docs/adr/semantics/0036-commit-checks-the-index-it-observed.md, "the person asks,
  execute acts").

## avoid

- identity taken from a position, an emission order or a frame counter.
- visual state (hover, pressed, animation progress, scroll offset)
  projected as semantic state.
- a whole document projected node by node.
- a revision computed with a std, seeded or platform hasher.
