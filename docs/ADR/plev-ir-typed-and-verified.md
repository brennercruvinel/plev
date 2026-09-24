---
type: adr
status: proposed
tags: [uir, plev-ir, verifier, capabilities, sem, comps, theme, schema, generative-runtime]
date: 2026-09-24
---

# the plev IR is typed, references only declared capabilities, and runs only once verified

## context

the generative runtime program (goal 3) needs one representation that a
person, an LLM, an importer or a tool can write, and that the runtime can
refuse: a surface described by meaning, not by pixels, where the model
proposes, the runtime verifies and plev executes (thesis, sections 2, 3
and 15). what existed:

- `crates/sem`: the semantic contract. every control has a `SemId`, a
  closed `Role`, state as `Value`s, and `ActionDecl`s with their
  `ParamSpec` and `Policy` (docs/ADR/semantic-contract-as-pure-projection.md,
  docs/ADR/actions-carry-observed-state.md). an app's `Snapshot` already
  declares everything it can show and run.
- `crates/comps`: the widgets, each with a `project` that declares its
  role, state and actions (task-59), and `engine::theme`: every visual
  value as a token.
- `crates/parser/src/ir.rs`: `ParserNode`, the IR the transpiler prints
  as builder code. its props are builder method names and its values are
  literals or theme expressions in Rust; it carries no identity and no
  semantics.

task-68 asked for a pure crate with the IR and a verifier that is the
only way to a `VerifiedIr`, with the orchestrator's decisions: the crate
is `uir`, the vocabulary v1 is what comps and the engine have, the
registry is built from the projection, and repetition over a list takes
the minimal form.

## decision

a new pure crate, `crates/uir`. dependencies: `sem` (with its serde
form), serde and serde_json. never engine, comps or wire; it compiles to
wasm.

### the IR

```json
{
  "plev_ir": 1,
  "surface": {
    "id": "review",
    "title": "review what goes into the commit",
    "root": {
      "sem": "review.files",
      "component": "list",
      "label": "changed files",
      "repeat": {
        "each": "changes:files",
        "item": {
          "sem": "review.file[{item}]",
          "layout": { "stack": { "axis": "horizontal", "gap": "sm" } },
          "children": [
            { "sem": "review.file[{item}].stage", "component": "button", "label": "stage",
              "bind": { "actions": { "activate": { "target": "changes.file[{item}]", "action": "stage" } } } }
          ]
        }
      }
    }
  }
}
```

- `Ir { plev_ir, surface: Surface { id, title, root } }`. a node
  (`IrNode`) has its identity (`sem`, a `SemId` chosen by whoever wrote
  it), an optional `component`, `label`, `description`, `props`, `layout`,
  `size`, `bind { state, actions }`, and `children` or a `repeat`.
  without a component a node is a layout container (role `Group`): a
  toolbar is a stack of buttons.
- `bind.state` maps a state the component shows to a `StateRef` of the
  app; `bind.actions` maps an event the component sends to a
  `CapabilityRef { target, action }` of the app. the IR only references:
  there is no way for it to declare a capability.
- `props` are tokens only, each named after its theme field or method:
  `padding` (`theme.spacing`), `typography` (`theme.typography`),
  `color` and `background` (`theme.colors`), `elevation`
  (`engine::theme::Elevation`), `control_size` (`ControlSize`), `shape`
  (`theme.shape`). `layout` is one of four primitives, `stack { axis,
  gap, align }`, `grid { min_col, gap }`, `split { axis, ratio }` and
  `overlay`; `size { min, max, flex }` bounds a node by `theme.size`
  tokens. the only numbers are proportions: a split's ratio (10 to 90
  percent) and a flex weight (1 to 12).
- repetition, the minimal form: a list or a container repeats a template
  (`repeat { each, item }`) once per item of a `List` or `Set` of the app
  state. every id in the template carries the key `[{item}]` exactly
  once, which each instance replaces with its item; a binding inside the
  template may carry it too and then names one capability per item. an
  instance is named by its item, so the template root has no label. a
  repeat inside a repeat is refused (it would need two items).

### one reader

JSON is read by one decoder, `uir::decode`, and `Ir`'s `Deserialize` goes
through it. it walks the tree by hand and hands each leaf (an id, a
component, a token, a reference) to the serde form of its type, so every
shape error comes at once with its JSON pointer. the node types derive
only `Serialize`: serde's derived reader also takes a struct written as a
JSON array, a second form the schema does not have, and the property test
found it.

### the capability registry

`Capabilities::from_snapshot(&Snapshot)` is every `StateRef` of the
projection with the `ValueKind` of its value, and every `ActionDecl` with
its target, `ParamSpec`, `Policy` and label, whatever its `enabled` or
`inert` now. it is never declared a second time by hand
(`with_data` and `with_action` exist for tests). a secret field is in it
as the snapshot keeps it, its length, so no surface can show or bind it
as text.

### the verifier

`verify(ir, caps, policy) -> Result<VerifiedIr, Vec<IrError>>`, and
`verify_json` for text. `VerifiedIr` has no public constructor, no
`Deserialize` and no `Default`; it carries the IR and its focus order.
every error has its rule, its JSON pointer and its reason, and all come
at once:

1. schema and version: the shape the decoder reads; `plev_ir` is 1.
2. identity: ids valid and unique on the surface (the surface id
   included); `[{item}]` only inside a template and exactly once per id;
   no fixed id is also what a template makes for some item; a node that
   takes the id of an app node sends only that node's actions.
3. vocabulary: a known component, tokens only, children, layout and
   repeat only on a container, a split with exactly two children, ratio
   and flex within bounds.
4. state: every bound state exists in the registry and its kind is one
   the component shows for that name; a repeat runs over a `List` or a
   `Set`; in a template the reference fits every matching node, all of
   one kind.
5. action: every bound action exists, and the event's parameter is the
   action's `ParamSpec` kind (an activation sends none); in a template
   every node that fits declares it, alike.
6. policy: the author (`GenerationPolicy::author`, a `sem::Actor`) binds
   only what it may ask for itself (`Policy::actors`); a destructive
   action is bound only to a dialog's `confirm`, unless its policy asks
   everyone to confirm (`Confirm::Always`).
7. accessibility: every control, dialog, image, chart, stat, status and
   progress has a label (a text or a badge may bind its `text` instead);
   a node without a role (a separator) is never named; a control with no
   action bound is only visual and is refused; in an overlay only one
   layer holds controls. the focus order is the controls in tree order.
8. limits: nodes (a template counted once), depth and text length. a
   hard depth of 256 bounds the walk whatever the policy says.

the default `GenerationPolicy` is conservative: an agent wrote it
(`agent:unknown`), 256 nodes, depth 16, 256 characters per title, label or
description.

### the vocabulary v1 and how it grows

the components are the widgets that exist: `button`, `icon_button`,
`text_field`, `checkbox`, `switch`, `slider`, `select`, `tabs`, `list`,
`table`, `card`, `stat`, `chart` (line, bar, area, donut), `separator`,
`dialog`, `empty_state`, `progress`, `badge`, `chip` from comps; `text`
and `image` from the engine builder. each is declared as data (`Spec`):
its role, whether it holds children, how it is named, the state it shows
and the events it sends, with the names its comps `project` uses
(`activate`, `toggle`, `set_value`, `select`, `submit`, `confirm`,
`cancel`). nine have no `project` in comps (card, stat, chart,
separator, empty state, progress, badge, text, image); for those the
role is uir's: group, label, image, none, status, progress bar, label,
label, image. a chip takes its role from what it binds, as the comps chip
projects it.

a new component needs the widget in comps first, then an ADR that adds
it to the table; a new token needs it in the theme first. `media`
(video) and `toolbar` stay out of v1: no widget has them, and a toolbar
is a stack of buttons.

### relation with the other representations

`ParserNode` stays the parser's code generation IR: it prints builder
code from another framework's markup. the plev IR is the runtime's IR:
identity and meaning first, tokens instead of expressions. task-71 makes
the parser emit plev IR. the engine builder is not the materialization
target by default; task-69 decides how a `VerifiedIr` becomes widgets.

### the JSON schema

`uir::schema()` builds the JSON schema (draft 2020-12) from the same
tables the types use; `crates/uir/schema/plev-ir-1.json` is that value
written out, kept equal by a test (`UIR_BLESS=1 cargo test -p uir` writes
it again). it constrains decoding to known components, tokens, fields
and binding names, and the proportions to their bounds. it does not
carry the grammar of an id, a state ref or an action name: the reader
checks it and answers with the pointer.

## reason

- a closed vocabulary and tokens make a generated surface consistent with
  the design system by construction: there is no field for a color.
- building the registry from the projection keeps one declaration of what
  an app can do; a registry by hand would age.
- a reference that must resolve, a parameter that must fit and an author
  that must be allowed mean a model can compose but never grant itself a
  capability, never send the wrong kind of value, and never put a
  destructive action one click away.
- every error at once, with a pointer, is what a model needs to repair
  its IR in one round.

## consequences

- task-69 materializes a `VerifiedIr`: each component on its comps
  widget, each token on its theme field, each binding through the
  dispatcher (every action a person takes on a materialized surface is
  a `Request` like any other). it projects the nine components without a
  comps `project` with the role this table gives, names each repeated
  instance by its item, and skips a repeated item a `List` holds twice
  (the verifier cannot see values).
- a `VerifiedIr` is verified against the registry of one instant. the
  capabilities can move (a file leaves the list); the dispatcher still
  checks every action when it is asked, so a stale surface fails safe.
- a template can only be verified while the app projects at least one
  node that fits it: over an empty list the registry cannot prove the
  per-item capability exists.
- the todo of the showcase projects no list of its items, so no surface
  can repeat over them yet; its filter is one capability per tab, which
  `tabs` (which sends the chosen key) cannot bind, so the todo fixture
  uses chips.
- the fixtures pin two real projections, the ide's workbench and the
  showcase's todo, written by tests in those crates that fail when the
  projection moves away from them.

## avoid

- a literal color, pixel, font size or code in the IR.
- a second reader of the IR, or a way to build a `VerifiedIr` without
  `verify`.
- a registry declared by hand next to the app's projection.
- growing the repeat into a template language (loops in loops,
  expressions, interpolated labels) without an ADR.
