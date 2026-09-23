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
- execution ends now (`Execution::Done`, `Failed`, `Rejected`) or later
  (`Started`, then `complete`, `fail` or `reject_at_truth`).
  `execute(id, req)` receives the request id, and an execution that ends
  later reports its end with that id, with no side channel.
- validation happens at two points when the truth lives outside the ui
  process (the git index, a server). the dispatcher's check is the first.
  `execute` carries the request to the point of truth, which checks the
  observed values again against its own state, with the same `check`,
  right before the effect. if it disagrees, the receipt goes from executing
  to rejected with the delta seen there. the first check alone leaves the
  race the ide has today: the index can move from a terminal between the
  check and the commit.
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
- a request awaiting confirmation stays open until a person confirms it;
  declining or expiring it needs an api this decision does not define (the
  confirmation ui is task-62 and task-69).
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
