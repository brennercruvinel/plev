---
type: adr
status: accepted
tags: [propose, uir, plev-ir, verifier, surface, llm, dispatcher, generative-runtime]
date: 2026-09-24
---

# proposers propose, the runtime verifies

## context

task-68 gave the plev IR and the verifier, the only way to a `VerifiedIr`
(docs/ADR/plev-ir-typed-and-verified.md). task-69 phases 1 and 2 turned
a `VerifiedIr` into a live surface and let it change form in place,
reconciled by `SemId` (docs/ADR/materialized-surfaces-from-verified-ir.md
and its addendum). what was missing is the step before: an intent in text
becoming an IR. the thesis puts it as "the model proposes, the runtime
verifies, plev executes" (sections 2, 5 and 15). phase 3a builds every
part that does not depend on a model provider; the provider (a local
model or an API) is brenner's decision, phase 3b.

## decision

### a crate of its own

`crates/propose`, pure like `uir`: it depends on `uir`, `sem`,
`serde_json` and `thiserror`, never on the engine, the widgets or the
network. a proposer is not part of the IR (the IR stays a language and
its verifier), and phase 3b adds a provider that must not reach `uir`,
which compiles to wasm and has no I/O.

### the trait and the one way out

- `Ask { intent, caps, app, current, policy }`: the intent in text, the
  app's registry (`Capabilities::from_snapshot`), its `Snapshot` (roles,
  parents, labels and values, which the registry does not carry), the
  `VerifiedIr` of the surface it shows now, if any, and the
  `GenerationPolicy` the proposal is verified under.
- `trait Proposer { fn propose(&mut self, ask) -> Result<Ir, ProposeError> }`:
  an `Ir` nobody has checked yet.
- `propose_verified(proposer, ask) -> Result<VerifiedIr, ProposeError>`
  verifies against the same registry and policy. it is the way out; an IR
  the verifier refuses leaves as `ProposeError::Refused` with every
  error, never as an IR.

### the deterministic proposer (`Rules`)

a small grammar, read in english and portuguese because it is the
proposer's own: add (adicionar), filter (filtro), counts or stats
(contagem), compact (compacto), two columns (duas colunas), everything
(tudo); articles and "only" are skipped. each part is found by the
registry's structure, never by an app's names:

- entry: a `text_input` node with a text state and an action whose
  parameter is a text becomes a `text_field` bound to both; the action
  with no parameter on the same node is its `submit` and a button's
  `activate` (without one, a button beside it with one action and no
  parameter);
- choice: tabs, options or radios whose one action takes no parameter
  become chips under their parent;
- counts: each `int` or `num` state becomes a stat named by its key;
- form: compact is one column; two columns is a grid over
  `field_min_w`, the choices and the counts in one column, the entry in
  the other. without a form word, the current surface's form.

every other word of an intent must be a word of the app (a label, an
id's name or key, a state's key) and chooses among the members: a part
the intent names keeps what the word names (all of it when it names none
of it), a part it does not name shows only what the word names. a word
that is neither the grammar's nor the app's is `NoMatch`, which lists the
app's words; an intent with no word the rules read is `NotUnderstood`; a
part the app has none of is `Nothing`. there is no lexicon between
languages: over the todo, "active" chooses and "ativo" is refused.
translating a person's word to the app's is a model's job.

continuity: over a current surface, the surface keeps its id (so the
page transforms it) and every node whose capability stays keeps its id
and label; a container takes the id of the current container that held
at least half of what it holds, from the inside out. anything new is
named `<surface>.<the app node it shows>`, never an id the current
surface uses. the rules bind only what the author may ask for and never a
destructive action.

### the language-model proposer (`Llm`)

- `trait Model { fn complete(&mut self, prompt: &str) -> Result<String, ModelError> }`:
  synchronous, as a proposal is today;
- the prompt carries the intent; the verifier's eight rules (an
  exhaustive match on `uir::Rule`, so a new rule has to be described) and
  the policy's author and limits; every component with its role, how it
  is named, the state it shows and the events it sends (from
  `Component::ALL` and each `Spec`); the registry in a form to read, each
  node with its role, label and parent, its states with kind and value
  now, its actions with what they take, destructive, confirmation, who
  may ask, disabled now; the surface shown now, its id to keep, every
  bound node with what it binds (the ids to keep) and its IR; and the
  JSON schema (`uir::schema()`). about 18 KB over the todo;
- the reply is read as the object inside it (prose and fences are
  common), decoded by `uir::parse` and verified; when refused, the next
  prompt is the first one plus the reply and every error at its JSON
  pointer, with its rule;
- `Llm::ATTEMPTS` is 3: the verifier gives every error at once, so one
  repair usually suffices, and the third bounds the cost of a model that
  does not converge. then `Exhausted` with the last errors. a model that
  fails to answer stops the proposal at once (`Model`);
- proven with scripted models: one that errs and is repaired, one that
  never converges, one that fails. no provider in the repository.

### the demo

the Surface page of the showcase has an intent field
(`surface_page.intent`) and a propose button (`surface_page.propose`,
action `propose`, reading the intent), and `surface_page.status` says
what the last proposal came to. the rules propose, `propose_verified`
admits, and `SurfaceHost::transform` (or `open`) shows it. both are
capabilities of the page run by its one dispatcher, the todo's:
`TodoPage::with_dispatcher` lends it to an `Operable` that projects the
todo, the surface and the row, runs the row's two actions and passes the
rest to the todo's own `execute`. a person's click, the Enter key and an
agent through the port take the same path, and the proposal's author is
whoever asked (`GenerationPolicy::by`).

## cost

headless, debug build, apple m4, rustc 1.98.1, over the todo with the
compact surface shown, intent "only what is active and the add field"
(`surface_page::tests::measure`, ignored, run by hand):

| step | first round | best of 20 |
| --- | --- | --- |
| the rules to a `VerifiedIr` | 0.23 to 0.34 ms | 0.16 ms |
| the whole request (observed, dispatched, proposed, verified, transformed) | 0.54 to 0.55 ms | 0.48 ms |
| and the page's next frame built (scene) | 0.77 to 0.80 ms | 0.66 to 0.68 ms |
| and that frame drawn and read back (pixels) | 32 to 33 ms | 4.2 ms |

the proposal costs a fraction of a millisecond; a model's latency is
what phase 3b adds, orders of magnitude above it.

## reason

- one way out (`propose_verified`) keeps "only a `VerifiedIr` reaches the
  runtime" true for every proposer, trusted or not.
- rules over structure show the registry is enough to build a surface
  without writing an app's names, and give a proposer for tests and
  demos with no network.
- a model gets exactly what the verifier will check, and its errors in
  the verifier's own words; it composes, it never grants itself a
  capability.
- the page's capabilities on the page's one dispatcher keep one request
  numbering and one place for receipts, which the agent port depends on.

## consequences

- what phase 3b must decide and wire: the provider (local or API) behind
  `Model`; where the prompt's system part and user part split for a chat
  API; running the model off the app's thread (the dispatcher already
  ends a request later, `Execution::Started` and `Dispatcher::complete`,
  and `Ask` borrows, so a worker needs owned data); constrained decoding
  with `uir::schema()` if the provider has it; the timeout and what the
  page shows while a model thinks; credentials and cost; whether a
  proposal that renames a kept capability's id is a soft error sent back
  (continuity is not a rule of the verifier).
- the rules' grammar is small by design; an intent outside it is an
  error, not a guess.
- "the active items" as a list is not in the registry: the todo projects
  no list of its items and says `completed: bool`, not "active" (the P2
  of phase 1).
- the verifier still accepts props a widget ignores (phase 1): a model
  that writes them does not learn from an error.
- the Surface page keeps the segmented control of phase 2: after a
  proposal it still names the last hand-written form, and a click on it
  goes back to that form.

## avoid

- returning an `Ir` from anything a runtime consumes: only
  `propose_verified`'s `VerifiedIr`.
- an app's names, or a translation table between languages, in the
  rules.
- a network call, a credential or a provider in this crate.
- a second dispatcher for a page's own capabilities.

## addendum, 2026-09-24: the rules propose the items (task-95, proposed)

### context

the todo of the showcase now projects its items as a list: `todo:items`,
a `List` on the `todo` region with the ids of the visible items in the
order shown, following the filter as the item nodes do, and each item's
text as state (`todo.item[<id>]:text`), since an instance is named by its
item and the IR does not interpolate a label. the consequence above,
"the todo projects no list of its items", no longer holds, and the rules
had no part that repeats.

### decision

- a fourth part of the grammar, items: `item`, `items`, `itens`, `list`,
  `lista`, `listar`.
- a list is found by structure: a `List` or `Set` state of a node whose
  values are the keys of that node's children of one kind
  (`<prefix>[<value>]`), all of them and nothing else, not empty. over
  the todo, `todo:items`; over the ide's workbench, `changes:files`. the
  ide's `changes:staged` holds only some of the files (a selection, not
  the list); an empty list is `Nothing`, since a template over it could
  not verify.
- each item shows and accepts what every item declares alike, as the
  verifier checks a template: a `text`, `int` or `num` state is a
  `text`; a bool with an action of no parameter that reads that bool and
  nothing else is a `checkbox` bound to both (the todo's `toggle`; the
  ide's `stage` reads two states and is not); every other action of no
  parameter, allowed to the author and not destructive, is a `button`
  named by its label or its name (`show_diff` is "show diff"). the
  template root is named by its item, so it has no label.
- the list's words are its key and its items' name (`item`, `file`), so
  the app's own word ("the files") chooses it like any other word of the
  app. no app's name is read.
- the lists go under the entry: after it in the compact form, in its
  column in two columns. everything includes them.
- continuity: the namer walks into templates. a template over the same
  list keeps its root's id, and each node in it keeps the id its anchor
  had there, so every instance (`todo_panel.item[<id>]` and its
  checkbox) keeps its widget through a proposal. a list node's anchor is
  the list it repeats over.

### consequences

- a surface proposed for the items never removes one: the todo's
  `remove` is destructive and confirms with no one, and a destructive
  action binds only to a dialog's confirm (rule 6). a removal on a
  generated surface needs a confirmation the app holds (an open state, a
  pending removal), which the todo does not have.
- the ide's files show their status and their buttons, not their path:
  the instance is named by the path in the projection, and the path is
  no state of the file's node, so nothing can bind it as text.
- "the active items" still does not come from the registry: `completed`
  is a bool, and "active" is the todo's sense of it. "only the active
  items" gives the whole list with the Active chip and count.
- a list the filter empties has no list to find; a surface that already
  shows it keeps its list node, with no instance.
