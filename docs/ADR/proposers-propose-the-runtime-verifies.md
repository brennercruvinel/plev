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

## addendum, 2026-09-24: the rules propose the items (task-95, accepted)

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
## addendum, 2026-09-24: the Claude as the model (task-69 phase 3b-i, accepted)

### context

phase 3a left `propose::Model` without a provider. decision 13 of the
program (brenner, 2026-09-24) names it: the Claude, through the `claude
-p` of the Claude Code already installed and logged in on this machine,
with no new paid API and no key in the code. phase 3b splits: 3b-i puts
the provider behind `Model` in a crate of its own and proves it on the
real projections; 3b-ii has the Surface page use it, off the app's
thread, after task-95.

### decision

- **a crate of its own, `crates/claude-model`.** it depends on `propose`
  (the trait) and `serde_json` (the child's result), never on `uir`,
  `sem`, the engine, the widgets or an HTTP client. `propose` stays pure.
- **a child process.** `ClaudeCli` implements `Model` by running the
  program (`claude`, or what `PLEV_CLAUDE` names) once per call, the
  prompt on its stdin, its stdout and stderr read on threads of their
  own. the network is the child's; the crate opens no socket and holds
  no key, and the child uses the login its Claude Code already has.
- **the flags, each for a reason the probe showed.** `-p` and
  `--output-format json` (one object: the reply, or the error with its
  kind, and the cost and turns; an exceeded budget exits 1 with the
  reason on stdout and nothing on stderr, so the json is what tells a
  reply from an error); `--tools ""` (no tool: the child reads no file
  and takes one turn); `--no-session-persistence`; `--setting-sources ""`
  (no settings file: no hooks, no permissions, no user memory);
  `--strict-mcp-config` with no config (no MCP server);
  `--disable-slash-commands` (no skill listed in the prompt);
  `--permission-prompts none` (anything that would ask is denied, so
  nothing waits for a person); `--system-prompt` with two sentences of
  its own in place of Claude Code's (about 4,200 input tokens fewer, and
  no talk of tools, directories or git the child does not have);
  `--model` (`sonnet` by default); `--max-budget-usd` (0.5 by default).
- **the child is not part of a Claude Code session.** it runs in a
  directory of its own (the system's temporary one by default) and
  without the variables such a session sets for its processes
  (`CLAUDECODE`, `CLAUDE_CODE_SESSION_ID`, `CLAUDE_CODE_MESSAGING_SOCKET`,
  `CLAUDE_EFFORT` and the others of `SESSION_VARS`), which an app run
  from a session would otherwise hand down.
- **a deadline.** past `timeout` (300 s by default since 2026-09-25,
  about twice the slowest call measured, 138 s; it was 180 s, twice the
  89 s of phase 3b-i, until three calls of the Silences review were killed
  in a row) the child is killed by its pid and reaped, and the call
  is a `ModelError`. once the child exits, the end of its output is
  waited for one second at most, so a grandchild that keeps a pipe open
  cannot hold the call; what the child wrote is kept.
- **every failure is a short `ModelError`**: it could not start, it was
  killed at the deadline, it exited with a failure (its stderr, else its
  stdout, on one line), its result says `is_error` (the kind and the
  reason), or it answered no result. `Llm` stops at once on any of them,
  as it did in phase 3a.
- **`propose::prompt` does not change.** both probe replies were the
  JSON alone, with no prose and no fence, and verified at once.
- **proven without the network.** the tests run `tests/fake-claude`, a
  shell script `program` points at, never the real `claude`: a good
  answer, a refused one repaired by the second call, a child past the
  timeout killed (its pid gone), a grandchild holding the pipes, a
  failing program, an exceeded budget, a program that is not there; and
  the child gets exactly the flags, the whole prompt (17 KB, more than a
  pipe holds) and none of the session's variables.
- **the demo** is `cargo run -p claude-model --example propose -- --app
  todo|ide "<intent>"`, over the goldens of task-68.

### cost

the probe (the todo, the compact surface shown, "only what is active and
the add field", sonnet): 40.1 s with Claude Code's system prompt, 24.5 s
with the crate's; 11,150 and 6,983 input tokens; about 2,000 to 3,000
tokens of thinking; 0.069 and 0.054 USD as the child counts it (list
prices; the account is a subscription). both verified at once. the four
runs of the example, each verified at the first attempt: 4.5, 5.6 and
6.3 s on the todo (about 0.034 USD each), 88.9 s on the ide's workbench
(5.6 KB of IR, 0.111 USD). the same prompt again within the hour costs a
third (the service keeps the prompt in its cache for an hour). a proposal by the Claude costs
seconds to a minute and a half, against 0.16 ms for the rules.

### consequences

- the Claude reads what the rules refuse ("a compact card with the input
  on top and the counts below", "show the filter as chips and hide the
  counts", a review of the commit on the ide) and keeps the ids of the
  current surface for what stays.
- it reads values now, not lists: "what is active" became the two items
  not completed today, and the ide's changed files were written one by
  one instead of a `repeat` over `changes:files`. a surface like that
  does not follow the app when an item or a file comes or goes.
- on the ide, the file rows took the ids of the app's nodes and the new
  nodes were named under them. the verifier accepts it and the host
  treats such a node as the app's (the fixed screen case).
- `--max-budget-usd` is checked after the child's turn: it caps a session
  that would go on, it never cuts a call short. the deadline is what
  bounds a call.
- the child writes its own state (`~/.claude.json`) and fetches the list
  of claude.ai connectors at start, as any run of Claude Code does; it
  connects none.
- what 3b-ii must decide and wire: the call off the app's thread
  (`Execution::Started`, then `Dispatcher::complete` or `fail`, with the
  snapshot, the registry and the current IR owned by the worker); what
  the page shows during seconds to minutes, and a cancel (killing the
  child is already how a deadline ends); the rules first and the Claude
  when they do not understand, or the Claude alone; whether to ask for a
  `repeat` over a list in the prompt, and whether an id taken from the
  app belongs on a generated surface.

### avoid

- a key, a credential or a socket in this crate; an HTTP client.
- running the real `claude` in a test of the gate.
- a child that inherits the Claude Code session that runs the app.
- waiting for a child without a deadline.

## addendum, 2026-09-24: the page asks the Claude off the app's thread (task-69 phase 3b-ii, accepted)

### context

phase 3b-i put the Claude behind `propose::Model` (`claude-model`,
`ClaudeCli` over `claude -p`), synchronous: a call holds the thread that
makes it for 5 to 90 s, up to three times per proposal. the Surface page
proposed by the rules alone, inside `execute`, in a fraction of a
millisecond. the probe of 3b-ii measured both: the rules' request is
`completed` when `dispatch` returns; the fake claude holds the caller for
the whole call. the page needed the Claude without freezing the app, with
the wait and the cost visible, a way to cancel, and the rules where the
program is absent.

### decision

- **who proposes is a capability of the page.** `surface_page.proposer`
  is a `select` between `rules` and `claude`; a person clicks it, an
  agent acts `select` with the name through the port. `claude` is
  offered, and chosen by default, only where the program is:
  `ClaudeCli::exists` looks for `claude` (or what `PLEV_CLAUDE` names) as
  a file at its path or on the PATH, and never runs it. off the desktop
  (wasm, mobile) `claude-model` is not a dependency and the select
  offers the rules alone. in the showcase's own tests the program is
  never looked for, so no test of the gate reaches the real `claude`; a
  test that wants the fake sets it.
- **the Claude is asked off the app's thread through the dispatcher's
  own contract.** `propose` with `claude` answers `Execution::Started`
  and keeps the request's id. a worker thread takes a clone of the
  `ClaudeCli` and owned copies of everything a `propose::Ask` borrows
  (the intent, the todo's snapshot and registry, the surface shown, the
  policy of whoever asked), runs `propose_verified(Llm)` there and
  reports back through a channel: each attempt as it begins, then the
  answer with the attempts, the time and the cost. only a `VerifiedIr`
  crosses the channel. the page's `wake` polls the channel every 100 ms
  while it waits; when the answer arrives the surface opens or
  transforms in place (`SurfaceHost::transform`, by `SemId`) and the
  request ends with `Dispatcher::complete` (the surface, the nodes, the
  attempts, the seconds, the cost) or `fail` (the reason), through the
  same lending of the todo's dispatcher (`TodoPage::with_dispatcher`) at
  the page clock's now. the app keeps taking requests, keys and frames
  meanwhile; a `toggle` of the todo completes while the Claude thinks.
- **one proposal at a time.** while one is asked the propose button and
  its action are disabled: a second `propose` is refused by the check as
  `Disabled`, before `execute`.
- **the wait is state.** `surface_page.status` says, as text and as
  state, that it is asking the Claude for the intent, the attempt (1 to
  3) and the seconds (`busy`, `attempt`, `seconds`); when it ends, who
  proposed, the time, the attempts and the cost (`cost_usd`, as the
  child counts it), or the reason it was refused. an agent reads the same
  nodes a person sees.
- **cancel is a signal in the run loop.** `claude_model::Cancel` is a
  shared flag a `ClaudeCli` carries (its clones share it); `run` reads it
  every 10 ms beside the deadline and kills the child by its pid, as at
  the deadline, and a call not started yet does not start. the page's
  `surface_page.cancel` (enabled only while asking) raises it and fails
  the request as `cancelled by <actor>` in the same lending of the
  dispatcher as the cancel itself; the worker's later message is dropped.
- **the prompt does not change.** the probe asked the todo, with the
  compact surface of task-95 shown, for "the items as a list with the add
  field on top" and the Claude answered a `repeat` over `todo:items` with
  the shown template's ids; the ide, with no surface shown, asked for
  "the changed files as a list ..." got a `repeat` over `changes:files`.
  the instruction to repeat was not needed.

### cost

the real calls of this phase (sonnet, list prices under the
subscription): the todo through the example, 7.2 s and 0.038 USD; the
todo through the page and the real port, 9.3 s and 0.040 USD (the
agent's act returned `completed` 9.4 s after `executing`); the ide from
scratch, 77.5 s and 0.102 USD. every one verified at the first attempt.
the page's own cost is a thread and a channel; the poll every 100 ms
while it waits asks for a frame only when the status text moved (once a
second, and at each attempt).

### consequences

- the receipt of a proposal by the Claude has `executing` and
  `completed` seconds apart, with the surface, the attempts, the seconds
  and the cost in its outcome; the agent port forwards the terminal stage
  when it comes, and a client's `act` returns then.
- the IR is verified against the registry at the instant of the ask. a
  capability that disappears while the Claude answers (an item removed)
  gives no instance, and a request on it is refused by the dispatcher;
  nothing unverified is shown.
- a person's click on the proposer changes the widget directly, as a
  key in the intent field does; an agent's `select` goes through the
  dispatcher. the same asymmetry the row had since 3a.
- the template of a surface the Claude proposes from scratch takes the
  app's ids (`changes.file[{item}]`), which the host treats as the app's
  nodes; over a shown surface it keeps the surface's template ids. asking
  for surface-owned ids is a prompt change for a later phase.
- the elapsed seconds and the attempt change the projection's revision
  while the page waits; requests are checked by their reads, so nothing
  else goes stale for it.

### avoid

- calling the model on the app's thread, or a second dispatcher for the
  proposal that ends later.
- an unverified IR crossing the worker's channel.
- a test of the gate that finds or runs the real `claude`.
- the page waiting for a child it cannot cancel.

## addendum, 2026-09-24: the rules show the media by structure (task-69 phase 4b, accepted)

### context

the rules of phase 3a and task-95 had no part for a video or an image:
over the silences app the probe of phase 4b got `"video" is not a word
of the app`, and the review the thesis asks for (the video, the counts,
the cuts with a frame each) was out of the rules' reach. the app now
projects a frame ref per frame it offers (a text naming a media node and
an instant, the addendum of `materialized-surfaces-from-verified-ir.md`),
which is structure the rules can read without an app's name.

### decision

- **a fifth part, media.** found by structure: a node with the media or
  image role whose text states are frame refs (the node named is looked
  up by its role, the instant parsed). each such state is a view of the
  media (`original`, `result`), and the part's words are the node's and
  the views' keys. the grammar's words for it are `video`, `image`,
  `frame` (and `vídeo`, `imagem`, `quadro`, `media`, `preview`).
- **which views.** the first view of each media node; the views a word
  of the intent names by their key (`the original`, `the result`); or,
  with `compare` (`comparar`, `comparison`, `versus`, `diff`), every
  view side by side in a grid over `field_min_w`. `compare` names the
  media part too, so "compare the original with the cuts" needs no other
  grammar word. one view is an `image` named by the node; several are
  named by their key.
- **in a list, an item's frame ref is an image**, a thumbnail
  (`control_size` xl) in front of its texts, never a text.
- **the numbers of a list's items are the list's.** the counts part no
  longer makes a stat of a state of a node a found list repeats over:
  over the silences, a stat per cut's `start` took the id the template
  makes for that item and the verifier refused the whole. the todo and
  the ide have no number on an item, so their proposals are the same.
- **the media goes on top**, before the entries and the lists, in both
  forms.
- **the prompt** says an image shows a frame when its `source` binds a
  text whose value names a media node and an instant. with it the Claude
  bound the cut thumbnails to `silences.cut[{item}]:frame` on the first
  proposal of the review (the calls are in the task file).
- **the example** takes `--app silences` (the golden of phase 4a) and
  `--shown <ir.json>`, an IR verified and shown as the current surface.

### consequences

- over the silences golden, "the cuts as a list with the video and the
  counts" is the review: the video's first view, the counts the words
  name (`cut` names `cut_secs`; "the list with the video and the counts"
  names them all), and the cuts with a frame, the range, the decision
  and the two buttons each; "compare the original with the cuts" is the
  comparison: the original and the result side by side over the same
  list. a comparison over the review keeps the ids of the video's view,
  the list and its template, so the instances go on; the result's view
  is new.
- `cut` is a word of the list (its items' name) and of a count
  (`cut_secs`): "the cuts ... and the counts" keeps the one count the
  word names, by the rule of phase 3a. a person who wants every count
  says "the counts" without naming the cuts, as the todo's "only what is
  active" keeps the one count "active" names.
- the counts of everything over the silences are the video's `duration`
  and `shown` and the sliders' `value` (twice, each labelled by its key)
  beside the summary's five: the rule "each int or num is a stat" reads
  a slider's value as a number the app shows. narrowing it (a number an
  action of its node takes is the control's) is a rule for another
  phase.
- an app without media refuses the media words as `Nothing`, and its
  lists carry no image.

### avoid

- reading a frame ref by a state's name: the value's form and the role
  of the node it names are the structure.
- a stat of a number a list already shows.
