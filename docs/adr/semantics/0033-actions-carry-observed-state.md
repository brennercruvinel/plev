---
type: adr
status: accepted
tags: [sem, actions, preconditions, receipts, dispatcher, agents, generative-runtime]
date: 2026-09-23
---

# actions carry the state their actor observed

## context

the ide commits with no check at all. `crates/ide/src/views/workspace/input.rs:47`
calls `submit_commit`, which pushes `UiRequest::Commit { message }`, which
becomes `GitCommand::Commit` (`crates/ide/src/session.rs:76`), which runs
`git commit --message` (`crates/git/src/repo.rs:187`). the index is
committed as it is at the instant the worker runs, whatever the person or
an agent saw when they decided, and the result comes back later as
`GitEvent::Committed`. the accessibility action handler only logs
(`crates/engine/src/window/mod.rs:221`).

with an agent in the loop the world moves between observing and acting
(thesis, section 8): the agent sees A, B and C staged and "fix login",
decides to commit, and meanwhile the person unstages B. task-57 gave every
control an identity, a state and, for each action, the state it reads
(`docs/ADR/semantic-contract-as-pure-projection.md`). what was missing is
the rule for acting on it.

## decision

in `crates/sem`, next to the contract:

- a `Request` names the actor (`Human`, `Agent { name }`, `Assistive`,
  `Test { name }`), the target, the action, an optional parameter, and an
  `Observation`: the value the actor saw for each state the action reads,
  and the revision of the snapshot it saw. `Request::observe` builds it
  from that snapshot. a click is the same thing: `Request::observe` over
  the snapshot of the frame the person looked at, with `Actor::Human`.
- `check(req, now)` is pure and decides in a fixed order: not found,
  unknown action, not authorized, stale, inert, disabled, invalid param,
  needs confirmation, accepted. stale comes before inert and disabled
  because its delta explains why the rest changed.
- stale compares only the action's `reads`, as the current snapshot
  declares them; a read the observation lacks counts as observed without a
  value. the revision is recorded and never compared: it moves with every
  projected value, and a moved list cursor must not invalidate a commit.
  the delta is one `Change::StateChanged` per read that moved, `before` the
  observed value, `after` the current one, `added` and `removed` computed
  by the same function `Snapshot::diff` uses.
- one `Dispatcher` serves every actor. `dispatch` projects the app
  (`Operable: Semantics`) at that instant and checks against that
  snapshot, never against a frame drawn earlier. it calls `execute` only
  after accepted, or after a person confirmed and a second check against
  the live state passed. `Confirm::ForAgents` asks agents only,
  `Confirm::Always` asks everyone. a person confirms, at the screen
  (`Actor::Human`) or through assistive technology (`Actor::Assistive`):
  someone using a screen reader is a person, and refusing their
  confirmation would break the accessibility contract (thesis, section
  21). an agent or a test never confirms. `destructive` is information for
  the ui; it asks nothing by itself.
- a `Receipt` holds the request, the revision of the last validation and
  every `Stage` with the time the caller passed (`Millis`; `sem` never
  reads a clock). the legal transitions are requested to accepted,
  rejected or awaiting confirmation; awaiting confirmation to confirmed or
  rejected; accepted or confirmed to executing; executing to completed,
  failed or rejected. a refused action is a `Verdict`; a wrong use of the
  api is a `SemError` (`UnknownRequest`, `IllegalTransition`,
  `ConfirmByMachine`).
- a person may also decline (task-58b). `Dispatcher::decline(id, by,
  now)` keeps `confirm`'s guards in `confirm`'s order: `UnknownRequest`,
  `IllegalTransition` when the request is not awaiting confirmation,
  `ConfirmByMachine` for an agent or a test, the receipt unchanged on
  every error. the receipt goes from awaiting confirmation to rejected with
  `Rejection::Declined { by }`, a transition the table already had.
  nothing runs, so the live state is not checked again and
  `validated_against` stays the one of the dispatch. its JSON,
  `{"rejected":{"declined":{"by":"human"}}}`, is pinned by a test. the ide
  declines as `Actor::Human` when the confirmation modal closes without a
  confirm (cancel, escape, a click outside it).
- execution ends now (`Execution::Done`, `Failed`, `Rejected`) or later
  (`Started`, then `complete`, `fail` or `reject_at_truth`).
  `execute(id, req)` receives the request id, and an execution that ends
  later reports its end with that id, with no side channel.
- validation happens at two points when the truth lives outside the ui
  process (the git index, a server). the dispatcher's check is the first.
  `execute` carries the request to the point of truth, which checks the
  observed values it owns again against its own state, right before the
  effect. if it disagrees, the receipt goes from executing to rejected
  with the delta seen there. the first check alone leaves the race the ide
  has today: the index can move from a terminal between the check and the
  commit.
- `ReceiptSink`s see every stage, in order, as it happens. open receipts
  stay in the dispatcher; finished ones are kept up to a retention limit
  (256 by default) and dropped after it.
- an app projection that does not validate rejects the request with
  `Rejection::InvalidProjection` and no `validated_against`, instead of
  running it unchecked or panicking.
- `Request`, `Receipt`, `Stage`, `Rejection`, `Verdict` and `Execution`
  have a serde form behind the `serde` feature: snake_case, ids and refs as
  canonical text, `RequestId` as decimal text like `Revision` is hex text.
  the JSON of one request and of the thesis receipt is pinned by tests.
- updated on 2026-09-23: the point of truth does not run `check`. the
  ide's git worker compares the staged set the actor observed with the
  paths the index stages (`GitRepo::commit_if`,
  `crates/git/src/repo.rs`), a set comparison of its own: `crates/git`
  does not depend on `sem`, and `check` has one caller, the dispatcher.
  it checks `changes:staged` alone; `commit.message:text` is checked once,
  by the dispatcher, and the commit carries the message the request sent.

## updated on 2026-09-24 (task-92): an agent asks the person

in the third round of checkpoint a (task-62) an agent was refused because
a person had taken a file the task needed out of the index. one agent put
it back over the person; the other stopped, since putting it back would
undo a person's deliberate act "with no way to ask why". the contract told
both what happened and who did it, and gave neither a way to ask.

- an agent may ask a person before its action runs. `Request::ask` is an
  optional short text saying why ("recolocar src/session.rs?"). an
  agent's request that carries it waits for a person
  (`Verdict::NeedsConfirmation`) even when the action's policy asks
  nothing, and is answered like any request that waits: `confirm`, with
  the second check against the live state, or `decline`. the question is
  the last step of `check`: a stale, inert or disabled request is refused
  as it would be without it, and a request that already waited waits the
  same. nobody answering keeps it waiting, as a discard waits.
- a person does not ask a person: the question of `Actor::Human` or
  `Actor::Assistive`, and of a test, asks nothing, as `Confirm::ForAgents`
  asks nothing of them. refusing it would need a new `Rejection`, and the
  recorder matches that enum whole.
- the serde form writes `ask` only when there is one: a request without
  it, and every session recorded before (`MONS`), reads the same, and the
  recorder carries it with no change of its own. `sem::redact` sends it as
  `REDACTED` when the target is secret, like any free text about one.
- the agent port bounds it before the dispatcher (`wire::MAX_ASK`, 280
  characters, one line, not blank; `bad_message` otherwise) and carries it
  as `Act::ask`, still protocol v1: `plev act <target> <action> --ask
  "<question>"`, which waits 300 s for the person unless `--timeout` says
  otherwise. `requested` echoes it in `asked`.
- the ide's confirmation modal says that the agent asks and quotes it, in
  the third person ("test-agent asks the person: ..."); confirming and
  declining go through the dispatcher as `Actor::Human`, as before. it
  never enters the projection: the modal is not projected, and an agent
  reads only its own question, in `asked`. the inspector's `why` shows
  the question and who answered it.

the probe of this task found a P0 on the way, fixed first and alone: the
agent port promised the stages of an agent's requests, but sent them only
when the shell drained it, and the shells drain after the port's wake
(the ide's window also after a git result). a stage that ended outside a
drain with nothing behind it never went out: a person's decline in the
modal (task-58b), a confirmation whose second check refused it, one whose
action ended at once. it waited for the agent's next message, and was
lost when a `plev act` that only waited gave up and closed its
connection. the port's sink now wakes the loop when it hears a stage of a
request the port waits on (`Server::wake_with`, `crates/wire/src/sink.rs`),
so every host that drains after the wake sends it, with no change to its
shell.

## reason

- an agent's action then rests on the world it saw, not on a world it
  presumes unchanged. the interface gets a property close to a
  transaction: act only if the state that was seen still holds.
- invalidation stays semantic. the projection already leaves hover,
  animation and position out; comparing the reads, not the revision,
  also leaves out projected state the action does not depend on.
- one path gives one record for every actor: who observed, who asked, who
  confirmed, which state existed, what happened (thesis, section 10).
- time passed in by the caller keeps tests and replay deterministic.
- the ui process does not own the truth of a git repository, so the check
  that counts last is the one next to the effect.

## consequences

- task-62 routes the ide commit through the dispatcher: the click becomes
  `Request::observe` plus `dispatch`, and the git worker runs the second
  check against the real index before `git commit`, answering
  `reject_at_truth` with the delta, using the id `execute` received.
- task-60 turns accessibility actions into requests with
  `Actor::Assistive`; task-61 carries requests and receipts over the
  socket in the pinned form; task-63 records receipts from a sink.
- replay never creates a request. no `Actor` stands for the player: a
  recording plays receipts back to show what happened and never asks the
  dispatcher for anything, so a replay cannot commit twice.
- a `secret` field is only its length in the snapshot, so a precondition
  on it sees a change of length and nothing else: two secrets of the same
  length compare equal. that is the price of never letting the secret, or
  a hash of it, leave the process. an action whose safety depends on the
  exact secret needs its point of truth to check it.
- a parameter travels in the request as given, so it reaches receipts and
  sinks; whoever sends receipts out of the process (task-61, task-63)
  redacts a parameter aimed at a secret field.
- the actor policy is coarse: four kinds. finer agent identity and scopes
  are future work.
- a request awaiting confirmation stays open until a person confirms or
  declines it (`Dispatcher::decline`, task-58b); expiring it needs an api
  this decision does not define (the confirmation ui is task-62 and
  task-69).
- `Constraints::editable` defaults to false, so a constrained parameter on
  a node that does not declare itself editable is refused. `step` is not
  checked.

## avoid

- deciding stale by the revision.
- a second way to run an action: a click handler, an accessibility
  handler or an agent tool that reaches the effect without `dispatch`.
- executing before accepted or confirmed.
- reading a clock inside `sem`.
- a request created by a replay.
