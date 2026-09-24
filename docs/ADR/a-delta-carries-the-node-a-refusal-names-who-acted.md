---
type: adr
status: accepted
tags: [sem, wire, rec, ide, protocol, delta, stale, receipts, dispatcher, agents, generative-runtime]
date: 2026-09-23
---

# a delta carries the node it adds, and a stale refusal names who acted

## context

checkpoint a (task-62 phase c) had two LLM agents operate the ide through
the contract only. the contract held, and the agents pointed at three
gaps, none of which stopped the task
(`docs/mission/steps/ongoing/task-62-ide-human-agent-proof.md`, "o que o
experimento mostrou"):

- a `node_added` in a delta carried only the id
  (`Change::NodeAdded(SemId)`). the agent that saw its commit born in
  `stacks` had to observe again to know what it was.
- a stale refusal said what moved (`Rejection::Stale { changes }`) and
  not who moved it: the agent could not tell a person at the screen from
  another agent or from a `git` in a terminal. the dispatcher already held
  the answer: each receipt keeps the revision it was checked against
  (`validated_against`), and the refused request keeps the revision its
  actor saw (`Observation::revision`).
- the rule "an act is stale when a state it reads changed; the revision
  only traces" lived in `sem::check` and in the doc of `Observation`,
  where an agent never looks. both agents deduced it.

## decision

### the node in the delta

`Change::NodeAdded(Box<Node>)`: the node as the `after` snapshot holds
it, every field, its state (a `secret` field is already its length
there) and its `source`. the serde form is `{"node_added": {<the node in
the lean form of task-57b>}}`, pinned in `crates/sem/tests/wire.rs`.
`Snapshot::diff` lists the same changes in the same order, and nothing
exactly when the two snapshots are equal; only the payload of
`node_added` grew. where the node sits among its siblings is still not in
the change, only its `parent`.

the agent port serves the node of a delta's `node_added` as it serves a
node of a snapshot. one function (`serve`, `crates/wire/src/server.rs`)
cuts the state to the observation's `keys` and drops `source` unless the
agent asked for it (`observe {"source": true}`); both replies go through
it. this corrects `docs/ADR/lean-semantic-form-and-one-redaction.md`,
which said a delta never carried `source` because a `Change` named ids
and values only: a delta carries it now exactly when a snapshot would.

the recorder still stores a snapshot whole when a node is added
(`structural` in `crates/rec/src/ring.rs`): what a `Kind::Changes` cannot
carry is the place of the new node in the projection order, not the
node. applying `node_added` in place waits for a change that says where
the node goes.

### who acted

`Rejection::Stale { changes, acted }`, with `acted: Vec<Acted>` and
`pub struct Acted { id: RequestId, actor: Actor, target: SemId, action:
ActionName }`: no parameter and no value, nothing of the state. `acted`
is not written when empty, and reads as empty when absent, so a refusal
written before this change reads as one that names nobody.

what it means, exactly: the actions of other actors that ran since the
state the author of the refused action saw. the rule, among the receipts
the dispatcher holds (the open ones and the finished ones retention
keeps), where a receipt ran when its stage is executing, completed or
failed (a refusal at the check or at the point of truth, a decline and a
wait for confirmation ran nothing): the start is the lowest id of a
receipt that ran and whose `validated_against` is
`Some(request.observed.revision)`, by any actor, the author included;
named are the receipts that ran, with an id from the start on, whose
actor is not the refused request's, in id order. with no start, nobody is
named. the dispatcher numbers requests in dispatch order, and the start
ran on the state the author saw, so it and every later one ran after the
author looked. the author's own receipts are never named (the author
knows what it did) and do not cut the chain; a receipt that ran nothing
starts nothing.

updated on 2026-09-23 (task-90): the rule of task-83 named only the
receipts checked on exactly that revision, so of a person's chain only
the first act was named. the proof of task-86 in a real window wrote
that limit down, and the probe of task-90 ran the window's scene through
the ide's real click: a person who clicks a new row moves the selection
outside the dispatcher before any request, so nothing runs on the state
the agent saw and the refusal names nobody, under either rule. the rule
above is the one inside the process, and the fallback for an anchor on
the author's look (task-91), which covers that scene.

the dispatcher fills it in one place, `push`, the function every stage
goes through, so the refusal at dispatch, at a confirmation's second
check, from `execute` (`Execution::Rejected`) and from `reject_at_truth`
name the same way; whatever a caller put in `acted` is replaced. whoever
builds a stale refusal outside the dispatcher (`check`, the ide's point
of truth in `crates/ide/src/workbench/truth.rs`) builds
`Rejection::stale(changes)`, which names nobody. the search reads only
`open` and `finished`, which were already bounded; the dispatcher keeps
nothing new.

the revision still decides nothing. `check` compares the reads; `acted`
reads revisions only after the refusal was decided, to say who else was
there.

`sem::redact` keeps `acted` as it is: request ids, actors, targets and
action names quote no secret, and a target id is already in every
snapshot. it still reduces the delta of a secret node to lengths, for a
clear target and for a secret one (a test in `crates/sem/src/redact.rs`).

the ide's toast adds who acted, with the `who` the notice already used
and the action in the past tense of the ui: "Refused claude's submit:
src/session.rs left the staged files (the person unstaged
src/session.rs)". a chain is one clause per actor's run of actions, in
the order they ran, another actor after a semicolon (task-90): "(the
person unstaged B, opened the diff of C and unstaged C)". with `acted`
empty the text is the one it was.

### the rule, where an agent reads it

`plev --help` (and so `plev act --help`), the doc of the protocol's `Act`
(`crates/wire/src/proto.rs`) and this ADR say it: an act is refused as
stale when a state it reads (the action's `reads`, as an observation
shows them) changed since the observation it rests on; the revision only
names what was seen and never decides; `changes` says what moved, and
`acted` says who, among the app's actors, ran something since that
state.

### compatibility

protocol v1 keeps its version. the new `node_added` form ships before
any consumer outside this repository reads a delta, and `Change` itself
does not read the old form, a bare id: `serde_json` refuses it, and a
test pins that. a bare id cannot become a node without the snapshot
that holds it, and `sem` has no snapshot in hand when it reads a change.

a persisted session is the one place the old form was kept. the recorder
never puts `node_added` in a `Kind::Changes`, but a whole snapshot
(`Kind::Snapshot { snapshot, changes }`) keeps the diff that explains
it, and that diff holds `node_added` when the step added a node. the
session reader, which has that snapshot, converts the old line
(`read_event` in `crates/rec/src/persist.rs`): in a snapshot event, each
`node_added` written as a bare id becomes the node the event's own
snapshot holds under that id. nothing is lost, because the node a step
adds is exactly the node the snapshot of that step holds. an id the
snapshot does not hold (a file no recorder wrote) is `FileError::Event`
on that line, typed, and no node is made up. the writer writes only the
new form, and the session format keeps version 0.

the sessions of checkpoint a and of the laboratory runs before this
change (`target/agents/exp/a1` and `a2`, `lab-scripted`, `lab-plev`,
`lab-control`, `lab-control2`; not tracked) all open with it, their
commit's node in place and their `commit.submit` receipts as recorded
(a refusal recorded then names nobody in `acted`).

## consequences

- measured, bytes of the line the server writes (no `\n`), debug build,
  over a real socket. the showcase's todo, a person adding one item
  through the page (`agent_port_new_row` in
  `crates/showcase/src/view/dispatch.rs`), the delta since the revision
  before it:

  | before the item | delta before | delta after | whole observation after it |
  | --- | --- | --- | --- |
  | 3 items | 423 B | 647 B | 2286 B |
  | 103 items | 378 B | 604 B | 26479 B |

  the delta holds the node added, the counter's label and `active`, and
  with 3 items the focus moving to the field; the node adds about 225 B,
  and the delta stays under a third of the whole observation with 3 items
  and under 3 percent with 103.

  the laboratory (`ide-lab --arm plev --human none`, the task done with
  `plev`, then `plev observe --since <the revision the submit rested on>
  --scope stacks`), when the commit is born in `stacks`:

  | | before | after |
  | --- | --- | --- |
  | the delta of `stacks` | 172 B | 427 B |
  | the second observation to know the commit (`--scope stacks.commit[sha]`) | 402 B | not needed |
  | the same delta unscoped | 1748 B | 2003 B |
  | a whole observation of `stacks` | 818 B | 818 B |

  574 bytes in two round trips became 427 in one.
- known limits of `acted`, written in the doc of `Acted`:
  - a projected change no receipt made starts nothing: a change from
    outside the app (a `git` in a terminal), and state an app moves
    outside the dispatcher. in the ide that is the selection by click or
    keys, typing in the commit message, focus, a toast leaving on its
    timer and the diff lines landing from the worker. when one of them
    comes first after the look, the next receipt is checked on another
    revision and nobody is named: the window's scene of a click on a new
    row, then an unstage, is this case, pinned in
    `crates/ide/tests/human_agent.rs`. the change still shows in
    `changes`. task-91 anchors on the look instead.
  - a state that comes back to exactly the revision the author saw can
    start the chain before the look: what ran from the first time the
    state was there is named.
  - the start is read from the revision the act names
    (`Observation::revision`). an act over the port whose read rests on
    an earlier observation (task-89) does not name what ran between that
    observation and the revision; task-91 anchors on the oldest look.
  - it says who ran something since that state, not that their action
    moved what the refused one reads: a person who only moved a list
    cursor or opened a diff is named beside the one who unstaged the
    file. `changes` says what moved.
  - a receipt dropped past the retention (256 finished ones by default)
    is not named, and a start dropped past it takes the chain with it
    (unless another receipt held ran on that revision); a failed action
    is named, since it ran and may have moved something before it
    failed.
- a recorded snapshot event that adds nodes repeats them in its `changes`.
  in the laboratory run above the semantic section went from 53732 to
  54066 bytes, 51 events each.
- the session reader carries one conversion for the form before this
  change, tried only on a line that does not read as it is.
- the recorder weighs the node of a `node_added` and the entries of
  `acted` in its byte estimate.

## avoid

- deciding stale by the revision; `acted` reads revisions only to name.
- a stale refusal that names who acted outside the dispatcher, or a
  second place in the dispatcher that fills `acted`.
- a value, a parameter or anything of the state in `Acted`.
- a delta whose node carries `source` or state keys the observation did
  not ask for.
